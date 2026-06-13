//! Run executor — turns a planned sample stream into a [`RunSummary`] + [`Scorecard`]
//! (design Section 5.7/5.9, plan Phase 1c).
//!
//! The executor owns the closed-loop orchestration only; every behaviour-bearing decision is
//! delegated to a plugin axis so new dataset/architecture combinations are supported by
//! composing plugins, never by editing this loop (ADR-002):
//!
//! ```text
//! sample -> encoder -> runtime.submit_sensory -> runtime.step -> runtime.collect_motor
//!        -> decoder -> (reward policy -> runtime.submit_reward) -> PredictionRecord
//! ...then: metric pack(predictions, targets) -> RunSummary
//! ```
//!
//! It is transport-agnostic by construction: the same loop drives the deterministic
//! [`StubFeagiRuntime`](crate::binding::StubFeagiRuntime) and the remote ZMQ runtime, because
//! it is generic over [`FeagiRuntime`] and binds the encoder/decoder frame types to that
//! runtime's `SensoryFrame`/`MotorFrame` (so the three cannot be mismatched at compile time).
//!
//! Determinism: the loop performs no wall-clock reads and no I/O of its own. Timestamps on the
//! emitted [`RunSummary`] are left `None` for the caller (CLI) to fill, keeping unit tests
//! reproducible. The `Scorecard` is assembled by a separate pure transform
//! ([`assemble_scorecard`]) so all provenance fields are supplied explicitly by the caller and
//! none are silently defaulted.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::binding::profile::{DecoderBindingProfile, EncoderBindingProfile};
use crate::binding::{
    DecoderPlugin, EncoderPlugin, Environment, EnvironmentRewardPolicy, FeagiRuntime,
    ObservationEncoder, RewardPolicy,
};
use crate::contracts::prediction_record::SCHEMA_VERSION as PREDICTION_RECORD_SCHEMA_VERSION;
use crate::contracts::run_summary::SCHEMA_VERSION as RUN_SUMMARY_SCHEMA_VERSION;
use crate::contracts::scorecard::SCHEMA_VERSION as SCORECARD_SCHEMA_VERSION;
use crate::contracts::TypedPrediction;
use crate::contracts::{
    BackendFingerprint, ContentHash, DatasetAssetId, IRSample, PredictionRecord, RunId, RunSpec,
    RunStatus, RunSummary, Scorecard, ScorecardId, ScorecardStatus, ScorecardVisibility,
};
use crate::contracts::{MetricScope, RunEvent, RunEventKind};
use crate::control::{CancelToken, NoopEventSink, RunEventSink};
use crate::error::TrainerError;
use crate::plugins::{EpisodeOutcome, EpisodeTrajectory, EpisodicMetricPack};
use crate::plugins::{MetricPackPlugin, MetricResult};

/// Tuning knobs for one rollout that are not part of the immutable [`RunSpec`] provenance.
///
/// `ticks_per_sample` is how many FEAGI bursts to advance between submitting a sample's
/// sensory frame and collecting its motor frame. It is supplied by the caller (resolved from
/// run/binding configuration) rather than hardcoded, so the same executor serves fast stubs
/// and slower live brains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutorConfig {
    /// Number of bursts to step the runtime per sample (must be non-zero).
    pub ticks_per_sample: u32,
}

/// The artifacts produced by a single rollout over a sample stream.
///
/// `predictions` are the per-sample evidence (one [`PredictionRecord`] per sample, in visit
/// order); `metric_result` is the aggregate scoring over the labeled subset; `summary` is the
/// terminal [`RunSummary`]. The caller composes these into a [`Scorecard`] via
/// [`assemble_scorecard`] and persists them as it sees fit.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutOutcome {
    /// Terminal lifecycle summary + headline metrics.
    pub summary: RunSummary,
    /// Per-sample decoded predictions, in visit order.
    pub predictions: Vec<PredictionRecord>,
    /// Aggregate metric-pack result over the labeled subset.
    pub metric_result: MetricResult,
}

/// Explicit, caller-sourced provenance required to bind a [`Scorecard`] to exact dataset bytes
/// and an execution environment.
///
/// These fields are intentionally *not* derived inside the executor: `dataset_asset_id`,
/// `dataset_version`, and `dataset_content_hash` come from the resolved
/// [`DatasetManifest`](crate::contracts::DatasetManifest); `backend_fingerprint` describes the
/// concrete runtime. Supplying them explicitly avoids inventing identity-bearing values.
#[derive(Debug, Clone, PartialEq)]
pub struct ScorecardProvenance {
    /// Identity assigned to the produced scorecard.
    pub scorecard_id: ScorecardId,
    /// Stable asset id of the dataset scored against (from the dataset manifest).
    pub dataset_asset_id: DatasetAssetId,
    /// Human-facing dataset version string (from the dataset manifest).
    pub dataset_version: String,
    /// Content hash binding the score to exact dataset bytes/labels (from the manifest).
    pub dataset_content_hash: ContentHash,
    /// Execution-environment fingerprint that produced the metrics.
    pub backend_fingerprint: BackendFingerprint,
    /// Verification state to stamp (Trainer-produced scorecards are `SelfReported`).
    pub status: ScorecardStatus,
    /// Publication state to stamp (Trainer always emits `Local`).
    pub visibility: ScorecardVisibility,
}

/// Drives one closed-loop rollout over `samples` (already in sampler order) and returns the
/// per-sample predictions, aggregate metrics, and terminal [`RunSummary`].
///
/// For each sample the loop encodes the sample, submits it to the runtime, steps the runtime,
/// collects + decodes the motor frame into a [`TypedPrediction`](crate::contracts::TypedPrediction),
/// and — when the sample is labeled — derives a reward via `reward_policy` and submits it. The
/// labeled subset is then scored by `metric_pack`.
///
/// The encoder/decoder frame types are bound to the runtime's `SensoryFrame`/`MotorFrame`, so a
/// mismatched binding is a compile error rather than a runtime failure.
///
/// # Errors
/// Returns the first [`TrainerError`] raised by any stage (encode/submit/step/collect/decode/
/// reward/metric). The loop is fail-fast and deterministic: it does not partially score.
#[allow(clippy::too_many_arguments)]
pub fn run_rollout<R, E, D, RP, M>(
    run_id: &RunId,
    samples: &[IRSample],
    runtime: &mut R,
    encoder: &mut E,
    encoder_profile: &EncoderBindingProfile,
    decoder: &mut D,
    decoder_profile: &DecoderBindingProfile,
    reward_policy: &RP,
    metric_pack: &M,
    config: &ExecutorConfig,
) -> Result<RolloutOutcome, TrainerError>
where
    R: FeagiRuntime,
    E: EncoderPlugin<Frame = R::SensoryFrame>,
    D: DecoderPlugin<Frame = R::MotorFrame>,
    RP: RewardPolicy,
    M: MetricPackPlugin,
{
    // Non-observed, non-cancellable convenience wrapper: drops events and never cancels, so all
    // existing callers keep identical behaviour.
    let mut sink = NoopEventSink;
    run_rollout_with_events(
        run_id,
        samples,
        runtime,
        encoder,
        encoder_profile,
        decoder,
        decoder_profile,
        reward_policy,
        metric_pack,
        config,
        &mut sink,
        &CancelToken::new(),
    )
}

/// Same closed-loop rollout as [`run_rollout`], but streams [`RunEvent`]s through `events` and
/// honours cooperative cancellation via `cancel` (ADR-011).
///
/// A `Progress` event is emitted after each sample is recorded (`repeat_index = 0`,
/// `repeat_total = 1`), and one final `MetricUpdate { scope: Aggregate }` is emitted after
/// scoring. The lifecycle events (`Running` / `Completed` / `Failed` / `ScorecardReady`) are owned
/// by the [`RunControl`](crate::control::RunControl) layer, not this function.
///
/// Cancellation is checked at the top of each sample iteration; if requested the loop stops
/// immediately with [`TrainerError::Cancelled`] and does not partially score.
///
/// # Errors
/// Returns the first [`TrainerError`] raised by any stage, or [`TrainerError::Cancelled`] if a
/// stop was requested mid-rollout.
#[allow(clippy::too_many_arguments)]
pub fn run_rollout_with_events<R, E, D, RP, M>(
    run_id: &RunId,
    samples: &[IRSample],
    runtime: &mut R,
    encoder: &mut E,
    encoder_profile: &EncoderBindingProfile,
    decoder: &mut D,
    decoder_profile: &DecoderBindingProfile,
    reward_policy: &RP,
    metric_pack: &M,
    config: &ExecutorConfig,
    events: &mut dyn RunEventSink,
    cancel: &CancelToken,
) -> Result<RolloutOutcome, TrainerError>
where
    R: FeagiRuntime,
    E: EncoderPlugin<Frame = R::SensoryFrame>,
    D: DecoderPlugin<Frame = R::MotorFrame>,
    RP: RewardPolicy,
    M: MetricPackPlugin,
{
    if config.ticks_per_sample == 0 {
        return Err(TrainerError::Config(
            "ExecutorConfig.ticks_per_sample must be non-zero".to_string(),
        ));
    }

    let total_samples = samples.len() as u64;
    let mut predictions: Vec<PredictionRecord> = Vec::with_capacity(samples.len());
    // The labeled subset that participates in scoring + reward (aligned by construction).
    let mut scored_predictions = Vec::new();
    let mut scored_targets = Vec::new();

    for (index, sample) in samples.iter().enumerate() {
        if cancel.is_cancelled() {
            return Err(TrainerError::Cancelled(format!(
                "stopped after {index} of {total_samples} samples"
            )));
        }

        let frame = encoder.encode(sample, encoder_profile)?;
        runtime.submit_sensory(frame)?;
        runtime.step(config.ticks_per_sample)?;
        let motor = runtime.collect_motor()?;
        let prediction = decoder.decode(motor, decoder_profile)?;

        if let Some(target) = &sample.target {
            let signals = reward_policy.reward(&prediction, target)?;
            runtime.submit_reward(&signals)?;
            scored_predictions.push(prediction.clone());
            scored_targets.push(target.clone());
        }

        predictions.push(PredictionRecord {
            schema_version: PREDICTION_RECORD_SCHEMA_VERSION,
            run_id: run_id.clone(),
            sample_id: sample.sample_id.clone(),
            output_type: sample.output_type,
            prediction,
            target: sample.target.clone(),
            timestamp: None,
            metadata: BTreeMap::new(),
        });

        events.emit(RunEvent::new(
            run_id.clone(),
            RunEventKind::Progress {
                samples_done: (index as u64) + 1,
                samples_total: total_samples,
                repeat_index: 0,
                repeat_total: 1,
            },
        ));
    }

    let metric_result = metric_pack.evaluate(&scored_predictions, &scored_targets)?;

    events.emit(RunEvent::new(
        run_id.clone(),
        RunEventKind::MetricUpdate {
            scope: MetricScope::Aggregate,
            metrics: metric_result.metrics.clone(),
        },
    ));

    let summary = RunSummary {
        schema_version: RUN_SUMMARY_SCHEMA_VERSION,
        run_id: run_id.clone(),
        status: RunStatus::Completed,
        total_samples,
        evaluated_samples: scored_predictions.len() as u64,
        metrics: metric_result.metrics.clone(),
        started_at: None,
        completed_at: None,
        scorecard_id: None,
        metadata: BTreeMap::new(),
    };

    Ok(RolloutOutcome {
        summary,
        predictions,
        metric_result,
    })
}

/// Assembles a portable [`Scorecard`] from the immutable run provenance ([`RunSpec`]), the
/// computed `metrics`, and the explicit dataset/backend `provenance`.
///
/// This is a pure mapping: every scorecard field is copied from a named source (the `RunSpec`,
/// the metric map, or the supplied [`ScorecardProvenance`]). It performs no I/O and invents no
/// identity — consistent with the Scorecard being generated entirely locally and offline
/// (ADR-006, ADR-012).
pub fn assemble_scorecard(
    run_spec: &RunSpec,
    metrics: &BTreeMap<String, f64>,
    provenance: ScorecardProvenance,
) -> Scorecard {
    assemble_scorecard_inner(run_spec, metrics, None, provenance)
}

/// Assembles a [`Scorecard`] for an N-seed repeated run.
///
/// Identical to [`assemble_scorecard`] but additionally stamps the per-metric distribution
/// (`metric_stats`); `metrics` should be the per-metric means (the point estimates). The `n` and
/// `confidence_level` recorded inside each [`MetricStat`](crate::contracts::MetricStat) make the
/// repeat protocol part of the scorecard provenance.
pub fn assemble_scorecard_with_stats(
    run_spec: &RunSpec,
    metrics: &BTreeMap<String, f64>,
    metric_stats: BTreeMap<String, crate::contracts::MetricStat>,
    provenance: ScorecardProvenance,
) -> Scorecard {
    assemble_scorecard_inner(run_spec, metrics, Some(metric_stats), provenance)
}

/// Shared pure mapping for both the single-run and repeated-run scorecard assemblers.
fn assemble_scorecard_inner(
    run_spec: &RunSpec,
    metrics: &BTreeMap<String, f64>,
    metric_stats: Option<BTreeMap<String, crate::contracts::MetricStat>>,
    provenance: ScorecardProvenance,
) -> Scorecard {
    Scorecard {
        schema_version: SCORECARD_SCHEMA_VERSION,
        scorecard_id: provenance.scorecard_id,
        connectome_hash: run_spec.connectome_hash.clone(),
        genome_version_id: run_spec.genome_version_id.clone(),
        dataset_asset_id: provenance.dataset_asset_id,
        dataset_version: provenance.dataset_version,
        dataset_content_hash: provenance.dataset_content_hash,
        evaluation_protocol_version: run_spec.evaluation_protocol_version.clone(),
        metric_pack: run_spec.metric_pack.clone(),
        split_id: run_spec.split_id.clone(),
        backend_fingerprint: provenance.backend_fingerprint,
        metrics: metrics.clone(),
        metric_stats,
        status: provenance.status,
        visibility: provenance.visibility,
        metadata: BTreeMap::new(),
    }
}

/// Tuning knobs for one closed-loop control rollout (plan Phase 1d).
///
/// These are *execution* knobs, not immutable `RunSpec` provenance: `episodes` is the number of
/// episodes rolled (the aggregation window `K`), `max_steps` caps each episode, `ticks_per_step`
/// is how many FEAGI bursts advance per control step, and `seed` is the base RNG seed (episode
/// `e` resets with `seed + e`, so the run is reproducible — plan Section 9). The pinned
/// pendulum values live in the `EvaluationSpec`/`RunConfig`, never hardcoded here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlConfig {
    /// Number of episodes to roll (the aggregation window, `K`).
    pub episodes: u32,
    /// Maximum control steps per episode before truncation.
    pub max_steps: u32,
    /// FEAGI bursts to advance per control step (must be non-zero).
    pub ticks_per_step: u32,
    /// Base RNG seed; episode `e` resets the environment with `seed + e`.
    pub seed: u64,
}

/// The artifacts produced by a single closed-loop control rollout.
#[derive(Debug, Clone, PartialEq)]
pub struct ControlRolloutOutcome {
    /// Terminal lifecycle summary + headline metrics (for control runs a "sample" is an episode).
    pub summary: RunSummary,
    /// Per-episode trajectories, in roll order.
    pub episodes: Vec<EpisodeTrajectory>,
    /// Aggregate episodic-control metric result.
    pub metric_result: MetricResult,
}

/// Scales a normalized action (each component in `[-1, 1]`) to the environment's per-dimension
/// actuator bounds `(low, high)`, clamping defensively. Length mismatch is an explicit error.
fn scale_action(
    normalized: &[f64],
    bounds: &(Vec<f64>, Vec<f64>),
) -> Result<Vec<f64>, TrainerError> {
    let (low, high) = bounds;
    if normalized.len() != low.len() || low.len() != high.len() {
        return Err(TrainerError::Config(format!(
            "decoded action dim {} does not match environment bounds dim {}",
            normalized.len(),
            low.len()
        )));
    }
    Ok(normalized
        .iter()
        .zip(low.iter())
        .zip(high.iter())
        .map(|((n, lo), hi)| (lo + (n + 1.0) / 2.0 * (hi - lo)).clamp(*lo, *hi))
        .collect())
}

/// Drives a closed-loop embodied/control rollout over `config.episodes` episodes and scores the
/// episodic outcome.
///
/// PARKED (ADR-014/ADR-015): this is the executor for the superseded "Topology C — Trainer drives
/// the sim" model. The live embodied path is the parallel co-agent model (the controller owns
/// physics; the Trainer injects training signals on disjoint cortical I/O). Retained only for a
/// possible trainer-owned, no-controller sim path; not wired into `RunConfig`/CLI.
///
/// Per step the loop encodes the environment observation, submits it, steps FEAGI, collects +
/// decodes the motor frame into a normalized action, scales it to the environment's actuator
/// bounds, applies it, and injects the environment-derived reward into FEAGI's affect channels.
/// An episode ends on environment failure (`terminated`) or at `max_steps` (`truncated`); the
/// completed [`EpisodeTrajectory`]s are scored by the episodic metric pack.
///
/// This is the embodied counterpart of [`run_rollout`]: it shares no state with the offline
/// path and leaves it unchanged. The decoder must emit [`TypedPrediction::Vector`] (a continuous
/// action); any other variant is an explicit error.
///
/// # Errors
/// Returns the first [`TrainerError`] from any stage; the loop is fail-fast and does not
/// partially score.
#[allow(clippy::too_many_arguments)]
pub fn run_control_rollout<Env, R, E, D, RP, M>(
    run_id: &RunId,
    env: &mut Env,
    runtime: &mut R,
    encoder: &mut E,
    encoder_profile: &EncoderBindingProfile,
    decoder: &mut D,
    decoder_profile: &DecoderBindingProfile,
    reward_policy: &RP,
    metric_pack: &M,
    config: &ControlConfig,
) -> Result<ControlRolloutOutcome, TrainerError>
where
    Env: Environment,
    R: FeagiRuntime,
    E: ObservationEncoder<Frame = R::SensoryFrame>,
    D: DecoderPlugin<Frame = R::MotorFrame>,
    RP: EnvironmentRewardPolicy,
    M: EpisodicMetricPack,
{
    if config.episodes == 0 {
        return Err(TrainerError::Config(
            "ControlConfig.episodes must be non-zero".to_string(),
        ));
    }
    if config.max_steps == 0 {
        return Err(TrainerError::Config(
            "ControlConfig.max_steps must be non-zero".to_string(),
        ));
    }
    if config.ticks_per_step == 0 {
        return Err(TrainerError::Config(
            "ControlConfig.ticks_per_step must be non-zero".to_string(),
        ));
    }

    let bounds = env.action_bounds();
    let mut episodes = Vec::with_capacity(config.episodes as usize);

    for episode_index in 0..config.episodes {
        let mut observation = env.reset(config.seed.wrapping_add(episode_index as u64))?;
        let mut step_rewards = Vec::new();
        let mut terminated = false;

        for _ in 0..config.max_steps {
            let frame = encoder.encode_observation(&observation, encoder_profile)?;
            runtime.submit_sensory(frame)?;
            runtime.step(config.ticks_per_step)?;
            let motor = runtime.collect_motor()?;
            let prediction = decoder.decode(motor, decoder_profile)?;

            let normalized = match prediction {
                TypedPrediction::Vector(v) => v,
                other => {
                    return Err(TrainerError::Evaluation(format!(
                        "control rollout requires Vector predictions, got {other:?}"
                    )))
                }
            };
            let action = scale_action(&normalized, &bounds)?;
            let outcome = env.step(&action)?;

            let signals = reward_policy.reward(outcome.reward, outcome.terminated)?;
            runtime.submit_reward(&signals)?;

            step_rewards.push(outcome.reward);
            observation = outcome.observation;

            if outcome.terminated {
                terminated = true;
                break;
            }
            if outcome.truncated {
                break;
            }
        }

        episodes.push(EpisodeTrajectory {
            step_rewards,
            outcome: if terminated {
                EpisodeOutcome::Terminated
            } else {
                EpisodeOutcome::Truncated
            },
        });
    }

    let metric_result = metric_pack.evaluate(&episodes)?;

    let summary = RunSummary {
        schema_version: RUN_SUMMARY_SCHEMA_VERSION,
        run_id: run_id.clone(),
        status: RunStatus::Completed,
        // For control runs a "sample" is an episode.
        total_samples: episodes.len() as u64,
        evaluated_samples: episodes.len() as u64,
        metrics: metric_result.metrics.clone(),
        started_at: None,
        completed_at: None,
        scorecard_id: None,
        metadata: BTreeMap::new(),
    };

    Ok(ControlRolloutOutcome {
        summary,
        episodes,
        metric_result,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::encoding_scheme::{BinSpacing, EncodingScheme};
    use crate::binding::reward::{AffectChannel, PainPleasureReward};
    use crate::binding::StubFeagiRuntime;
    use crate::contracts::common::{
        BackendKind, ConnectomeHash, EvaluationProtocolVersion, PluginId, Split,
    };
    use crate::contracts::ir_sample::Payload;
    use crate::contracts::run_spec::{
        CoderBinding, ExecutionMode, PinnedBinding, RewardPolicyBinding, SamplerBinding,
    };
    use crate::contracts::{
        DatasetVersionId, Modality, OutputType, PluginRef, SampleId, SplitId, TypedPrediction,
        TypedTarget,
    };
    use crate::metrics::ClassificationMetricPack;
    use serde_json::json;

    /// A minimal encoder test double: emits the sample's tabular payload as the sensory frame.
    /// (The executor is the subject under test; the encoder is a collaborator.)
    struct PassthroughEncoder;

    impl EncoderPlugin for PassthroughEncoder {
        type Frame = Vec<f64>;

        fn plugin_ref(&self) -> PluginRef {
            PluginRef {
                id: PluginId("test.passthrough_encoder".to_string()),
                version: "1.0.0".to_string(),
            }
        }

        fn encode(
            &mut self,
            sample: &IRSample,
            _profile: &EncoderBindingProfile,
        ) -> Result<Self::Frame, TrainerError> {
            match &sample.payload {
                Payload::Tabular(features) => Ok(features.clone()),
                other => Err(TrainerError::Config(format!(
                    "passthrough encoder supports tabular payloads only, got {other:?}"
                ))),
            }
        }
    }

    /// A minimal decoder test double: argmax over the motor channel vector -> class id.
    struct ArgmaxDecoder;

    impl DecoderPlugin for ArgmaxDecoder {
        type Frame = Vec<f64>;

        fn plugin_ref(&self) -> PluginRef {
            PluginRef {
                id: PluginId("test.argmax_decoder".to_string()),
                version: "1.0.0".to_string(),
            }
        }

        fn decode(
            &mut self,
            motor: Self::Frame,
            _profile: &DecoderBindingProfile,
        ) -> Result<TypedPrediction, TrainerError> {
            let argmax = motor
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).expect("no NaN in test vectors"))
                .map(|(idx, _)| idx as u32)
                .ok_or_else(|| TrainerError::Runtime("empty motor frame".to_string()))?;
            Ok(TypedPrediction::Class {
                class_id: argmax,
                scores: motor,
            })
        }
    }

    fn one_hot(class_id: u32, classes: u32) -> Vec<f64> {
        (0..classes)
            .map(|c| if c == class_id { 1.0 } else { 0.0 })
            .collect()
    }

    /// One labeled tabular sample whose features one-hot encode its own class, so an identity
    /// runtime + argmax decoder reproduce the class exactly (lets us assert orchestration).
    fn one_hot_sample(idx: usize, class_id: u32, classes: u32) -> IRSample {
        IRSample {
            schema_version: crate::contracts::ir_sample::SCHEMA_VERSION,
            sample_id: SampleId(format!("s-{idx:04}")),
            dataset_version_id: DatasetVersionId("test@1".to_string()),
            split: Split::Test,
            modality: Modality::Tabular,
            payload: Payload::Tabular(one_hot(class_id, classes)),
            target: Some(TypedTarget::Class {
                class_id,
                label: None,
            }),
            output_type: OutputType::Class,
            coordinate_frame: None,
            timestamp: None,
            metadata: BTreeMap::new(),
        }
    }

    fn encoder_profile() -> EncoderBindingProfile {
        EncoderBindingProfile {
            cortical_area_id: "iv00_C".to_string(),
            channels: 3,
            scheme: EncodingScheme::PopulationSingleSpike {
                bins: 1,
                spacing: BinSpacing::Linear,
            },
        }
    }

    fn decoder_profile() -> DecoderBindingProfile {
        DecoderBindingProfile {
            cortical_area_id: "o____C".to_string(),
            class_count: 3,
            bins: 1,
        }
    }

    fn config() -> ExecutorConfig {
        ExecutorConfig {
            ticks_per_sample: 4,
        }
    }

    #[test]
    fn rollout_scores_all_correct_and_rewards_pleasure() {
        let samples = vec![
            one_hot_sample(0, 0, 3),
            one_hot_sample(1, 1, 3),
            one_hot_sample(2, 2, 3),
        ];
        let mut runtime = StubFeagiRuntime::identity();
        let mut encoder = PassthroughEncoder;
        let mut decoder = ArgmaxDecoder;
        let reward = PainPleasureReward::new(0.8).unwrap();
        let metric = ClassificationMetricPack::new();

        let outcome = run_rollout(
            &RunId("run-0001".to_string()),
            &samples,
            &mut runtime,
            &mut encoder,
            &encoder_profile(),
            &mut decoder,
            &decoder_profile(),
            &reward,
            &metric,
            &config(),
        )
        .expect("rollout");

        assert_eq!(outcome.summary.status, RunStatus::Completed);
        assert_eq!(outcome.summary.total_samples, 3);
        assert_eq!(outcome.summary.evaluated_samples, 3);
        assert_eq!(outcome.predictions.len(), 3);
        assert!((outcome.metric_result.metrics["accuracy"] - 1.0).abs() < 1e-12);

        // Every sample matched, so every reward must be Pleasure, and the runtime stepped once
        // per sample at the configured tick count.
        let rewards = runtime.submitted_rewards();
        assert_eq!(rewards.len(), 3);
        assert!(rewards.iter().all(|r| r.channel == AffectChannel::Pleasure));
        assert_eq!(runtime.burst_count(), 3 * 4);
    }

    #[test]
    fn rollout_reflects_misclassification_and_rewards_pain() {
        let samples = vec![
            one_hot_sample(0, 0, 3),
            one_hot_sample(1, 1, 3),
            one_hot_sample(2, 2, 3),
        ];
        // A runtime that rotates the one-hot frame forces a wrong argmax for every sample.
        let mut runtime = StubFeagiRuntime::new(
            |s| {
                let mut rotated = s.to_vec();
                rotated.rotate_right(1);
                rotated
            },
            false,
        );
        let mut encoder = PassthroughEncoder;
        let mut decoder = ArgmaxDecoder;
        let reward = PainPleasureReward::new(0.5).unwrap();
        let metric = ClassificationMetricPack::new();

        let outcome = run_rollout(
            &RunId("run-0002".to_string()),
            &samples,
            &mut runtime,
            &mut encoder,
            &encoder_profile(),
            &mut decoder,
            &decoder_profile(),
            &reward,
            &metric,
            &config(),
        )
        .expect("rollout");

        assert!((outcome.metric_result.metrics["accuracy"] - 0.0).abs() < 1e-12);
        let rewards = runtime.submitted_rewards();
        assert_eq!(rewards.len(), 3);
        assert!(rewards.iter().all(|r| r.channel == AffectChannel::Pain));
    }

    #[test]
    fn unlabeled_samples_are_recorded_but_not_scored() {
        let mut labeled = one_hot_sample(0, 0, 3);
        let mut unlabeled = one_hot_sample(1, 1, 3);
        unlabeled.target = None;
        labeled.target = Some(TypedTarget::Class {
            class_id: 0,
            label: None,
        });
        let samples = vec![labeled, unlabeled];

        let mut runtime = StubFeagiRuntime::identity();
        let mut encoder = PassthroughEncoder;
        let mut decoder = ArgmaxDecoder;
        let reward = PainPleasureReward::new(0.8).unwrap();
        let metric = ClassificationMetricPack::new();

        let outcome = run_rollout(
            &RunId("run-0003".to_string()),
            &samples,
            &mut runtime,
            &mut encoder,
            &encoder_profile(),
            &mut decoder,
            &decoder_profile(),
            &reward,
            &metric,
            &config(),
        )
        .expect("rollout");

        // Both samples produce a record, but only the labeled one is scored / rewarded.
        assert_eq!(outcome.predictions.len(), 2);
        assert_eq!(outcome.summary.total_samples, 2);
        assert_eq!(outcome.summary.evaluated_samples, 1);
        assert_eq!(runtime.submitted_rewards().len(), 1);
    }

    #[test]
    fn rollout_with_events_streams_progress_then_aggregate_metrics() {
        use crate::control::CollectingEventSink;

        let samples = vec![
            one_hot_sample(0, 0, 3),
            one_hot_sample(1, 1, 3),
            one_hot_sample(2, 2, 3),
        ];
        let mut runtime = StubFeagiRuntime::identity();
        let mut encoder = PassthroughEncoder;
        let mut decoder = ArgmaxDecoder;
        let reward = PainPleasureReward::new(0.8).unwrap();
        let metric = ClassificationMetricPack::new();
        let mut sink = CollectingEventSink::default();

        run_rollout_with_events(
            &RunId("run-evt".to_string()),
            &samples,
            &mut runtime,
            &mut encoder,
            &encoder_profile(),
            &mut decoder,
            &decoder_profile(),
            &reward,
            &metric,
            &config(),
            &mut sink,
            &CancelToken::new(),
        )
        .expect("rollout");

        // One Progress per sample, in order, then exactly one aggregate MetricUpdate last.
        let progress: Vec<_> = sink
            .events
            .iter()
            .filter_map(|e| match &e.kind {
                RunEventKind::Progress { samples_done, .. } => Some(*samples_done),
                _ => None,
            })
            .collect();
        assert_eq!(progress, vec![1, 2, 3]);

        match sink.events.last().map(|e| &e.kind) {
            Some(RunEventKind::MetricUpdate { scope, metrics }) => {
                assert_eq!(*scope, MetricScope::Aggregate);
                assert!((metrics["accuracy"] - 1.0).abs() < 1e-12);
            }
            other => panic!("expected trailing aggregate MetricUpdate, got {other:?}"),
        }
    }

    #[test]
    fn rollout_with_events_stops_on_cancellation() {
        use crate::control::CollectingEventSink;

        let samples = vec![
            one_hot_sample(0, 0, 3),
            one_hot_sample(1, 1, 3),
            one_hot_sample(2, 2, 3),
        ];
        let mut runtime = StubFeagiRuntime::identity();
        let mut encoder = PassthroughEncoder;
        let mut decoder = ArgmaxDecoder;
        let reward = PainPleasureReward::new(0.8).unwrap();
        let metric = ClassificationMetricPack::new();
        let mut sink = CollectingEventSink::default();
        // Pre-cancel: the very first iteration's guard trips before any work happens.
        let cancel = CancelToken::new();
        cancel.cancel();

        let result = run_rollout_with_events(
            &RunId("run-cancel".to_string()),
            &samples,
            &mut runtime,
            &mut encoder,
            &encoder_profile(),
            &mut decoder,
            &decoder_profile(),
            &reward,
            &metric,
            &config(),
            &mut sink,
            &cancel,
        );

        assert!(matches!(result, Err(TrainerError::Cancelled(_))));
        // No samples were stepped or scored before the stop.
        assert_eq!(runtime.burst_count(), 0);
        assert!(sink.events.is_empty());
    }

    #[test]
    fn zero_ticks_is_rejected() {
        let samples = vec![one_hot_sample(0, 0, 3)];
        let mut runtime = StubFeagiRuntime::identity();
        let mut encoder = PassthroughEncoder;
        let mut decoder = ArgmaxDecoder;
        let reward = PainPleasureReward::new(0.8).unwrap();
        let metric = ClassificationMetricPack::new();

        let result = run_rollout(
            &RunId("run-0004".to_string()),
            &samples,
            &mut runtime,
            &mut encoder,
            &encoder_profile(),
            &mut decoder,
            &decoder_profile(),
            &reward,
            &metric,
            &ExecutorConfig {
                ticks_per_sample: 0,
            },
        );
        assert!(matches!(result, Err(TrainerError::Config(_))));
    }

    fn iris_run_spec() -> RunSpec {
        RunSpec {
            schema_version: crate::contracts::run_spec::SCHEMA_VERSION,
            run_id: RunId("run-0001".to_string()),
            dataset_version_id: DatasetVersionId("iris@1".to_string()),
            split_id: SplitId("test".to_string()),
            adapter: PluginRef {
                id: PluginId("tabular_csv".to_string()),
                version: "1.0.0".to_string(),
            },
            sampler: SamplerBinding {
                plugin: PluginRef {
                    id: PluginId("sequential".to_string()),
                    version: "1.0.0".to_string(),
                },
                seed: 42,
            },
            transform_graph_version: None,
            binding: PinnedBinding {
                encoder: CoderBinding {
                    io_type: "Percentage".to_string(),
                    coder_id: "percentage_encoder".to_string(),
                    cortical_area_id: "iv00_C".to_string(),
                    properties: json!({}),
                },
                decoder: CoderBinding {
                    io_type: "Percentage".to_string(),
                    coder_id: "percentage_decoder".to_string(),
                    cortical_area_id: "o____C".to_string(),
                    properties: json!({}),
                },
            },
            reward_policy: RewardPolicyBinding {
                plugin: PluginRef {
                    id: PluginId("reward.pain_pleasure".to_string()),
                    version: "1.0.0".to_string(),
                },
                config: json!({}),
            },
            metric_pack: PluginRef {
                id: PluginId("classification".to_string()),
                version: "1.0.0".to_string(),
            },
            evaluation_protocol_version: EvaluationProtocolVersion("clf-v1".to_string()),
            connectome_hash: ConnectomeHash("sha256:connectome".to_string()),
            genome_version_id: None,
            execution_mode: ExecutionMode::Remote,
            backend: BackendKind::Cpu,
            quantization: None,
        }
    }

    // --- Closed-loop control rollout test doubles + tests (Phase 1d) ---

    use crate::binding::environment::{Observation, StubEnvironment};
    use crate::binding::reward::SurvivalReward;
    use crate::metrics::EpisodicControlMetricPack;
    use crate::plugins::EpisodeOutcome as Eo;

    /// Passthrough observation encoder: emits the observation as the sensory frame (the executor
    /// is the subject under test; the encoder is a collaborator).
    struct PassthroughObsEncoder;

    impl ObservationEncoder for PassthroughObsEncoder {
        type Frame = Vec<f64>;

        fn plugin_ref(&self) -> PluginRef {
            PluginRef {
                id: PluginId("test.passthrough_obs_encoder".to_string()),
                version: "1.0.0".to_string(),
            }
        }

        fn encode_observation(
            &mut self,
            observation: &Observation,
            _profile: &EncoderBindingProfile,
        ) -> Result<Self::Frame, TrainerError> {
            Ok(observation.clone())
        }
    }

    /// Decoder that always emits a fixed normalized action, ignoring the motor frame.
    struct FixedActionDecoder(f64);

    impl DecoderPlugin for FixedActionDecoder {
        type Frame = Vec<f64>;

        fn plugin_ref(&self) -> PluginRef {
            PluginRef {
                id: PluginId("test.fixed_action_decoder".to_string()),
                version: "1.0.0".to_string(),
            }
        }

        fn decode(
            &mut self,
            _motor: Self::Frame,
            _profile: &DecoderBindingProfile,
        ) -> Result<TypedPrediction, TrainerError> {
            Ok(TypedPrediction::Vector(vec![self.0]))
        }
    }

    fn control_config(episodes: u32, max_steps: u32) -> ControlConfig {
        ControlConfig {
            episodes,
            max_steps,
            ticks_per_step: 2,
            seed: 7,
        }
    }

    #[test]
    fn control_rollout_truncates_and_scores_success() {
        // High fail threshold + zero action -> every episode survives to the executor cap.
        let mut env = StubEnvironment::new(10.0, 10_000, -1.0, 1.0).unwrap();
        let mut runtime = StubFeagiRuntime::identity();
        let mut encoder = PassthroughObsEncoder;
        let mut decoder = FixedActionDecoder(0.0); // normalized 0 -> mid of [-1,1] = 0 force
        let reward = SurvivalReward::new(0.6, 0.9).unwrap();
        let metric = EpisodicControlMetricPack::new(5).unwrap();

        let outcome = run_control_rollout(
            &RunId("ctrl-0001".to_string()),
            &mut env,
            &mut runtime,
            &mut encoder,
            &encoder_profile(),
            &mut decoder,
            &decoder_profile(),
            &reward,
            &metric,
            &control_config(3, 5),
        )
        .expect("control rollout");

        assert_eq!(outcome.episodes.len(), 3);
        assert!(outcome.episodes.iter().all(|e| e.duration() == 5));
        assert!(outcome.episodes.iter().all(|e| e.outcome == Eo::Truncated));
        assert!((outcome.metric_result.metrics["mean_episode_length"] - 5.0).abs() < 1e-12);
        assert!((outcome.metric_result.metrics["success_rate"] - 1.0).abs() < 1e-12);
        assert!((outcome.metric_result.metrics["mean_return"] - 5.0).abs() < 1e-12);
        assert_eq!(outcome.summary.total_samples, 3);
        // 3 episodes * 5 steps * 2 ticks.
        assert_eq!(runtime.burst_count(), 3 * 5 * 2);
        // Every surviving step yields one Pleasure signal; no failure -> no Pain.
        let rewards = runtime.submitted_rewards();
        assert_eq!(rewards.len(), 3 * 5);
        assert!(rewards
            .iter()
            .all(|r| r.channel == crate::binding::AffectChannel::Pleasure));
    }

    #[test]
    fn control_rollout_failure_emits_pain_and_terminates() {
        // Low fail threshold + max positive action -> state diverges and the pole "falls".
        let mut env = StubEnvironment::new(1.0, 10_000, -1.0, 1.0).unwrap();
        let mut runtime = StubFeagiRuntime::identity();
        let mut encoder = PassthroughObsEncoder;
        let mut decoder = FixedActionDecoder(1.0); // normalized 1 -> max force = 1.0
        let reward = SurvivalReward::new(0.6, 0.9).unwrap();
        let metric = EpisodicControlMetricPack::new(1).unwrap();

        let outcome = run_control_rollout(
            &RunId("ctrl-0002".to_string()),
            &mut env,
            &mut runtime,
            &mut encoder,
            &encoder_profile(),
            &mut decoder,
            &decoder_profile(),
            &reward,
            &metric,
            &control_config(2, 100),
        )
        .expect("control rollout");

        assert_eq!(outcome.episodes.len(), 2);
        assert!(outcome.episodes.iter().all(|e| e.outcome == Eo::Terminated));
        // The terminating step of each episode injects exactly one Pain signal.
        let pain = runtime
            .submitted_rewards()
            .iter()
            .filter(|r| r.channel == crate::binding::AffectChannel::Pain)
            .count();
        assert_eq!(pain, 2);
    }

    #[test]
    fn control_rollout_rejects_zero_episodes() {
        let mut env = StubEnvironment::new(10.0, 100, -1.0, 1.0).unwrap();
        let mut runtime = StubFeagiRuntime::identity();
        let mut encoder = PassthroughObsEncoder;
        let mut decoder = FixedActionDecoder(0.0);
        let reward = SurvivalReward::new(0.6, 0.9).unwrap();
        let metric = EpisodicControlMetricPack::new(1).unwrap();

        let result = run_control_rollout(
            &RunId("ctrl-0003".to_string()),
            &mut env,
            &mut runtime,
            &mut encoder,
            &encoder_profile(),
            &mut decoder,
            &decoder_profile(),
            &reward,
            &metric,
            &control_config(0, 5),
        );
        assert!(matches!(result, Err(TrainerError::Config(_))));
    }

    #[test]
    fn assemble_scorecard_maps_run_spec_and_metrics() {
        let run_spec = iris_run_spec();
        let mut metrics = BTreeMap::new();
        metrics.insert("accuracy".to_string(), 0.75);

        let provenance = ScorecardProvenance {
            scorecard_id: ScorecardId("sc-0001".to_string()),
            dataset_asset_id: DatasetAssetId("local:iris".to_string()),
            dataset_version: "1.0.0".to_string(),
            dataset_content_hash: ContentHash("sha256:abc".to_string()),
            backend_fingerprint: BackendFingerprint {
                backend: BackendKind::Cpu,
                descriptor: "stub-cpu".to_string(),
                quantization: None,
                trainer_version: env!("CARGO_PKG_VERSION").to_string(),
                feagi_core_version: "0.0.12".to_string(),
            },
            status: ScorecardStatus::SelfReported,
            visibility: ScorecardVisibility::Local,
        };

        let card = assemble_scorecard(&run_spec, &metrics, provenance);

        assert_eq!(card.connectome_hash, run_spec.connectome_hash);
        assert_eq!(card.metric_pack, run_spec.metric_pack);
        assert_eq!(card.split_id, run_spec.split_id);
        assert_eq!(
            card.evaluation_protocol_version,
            run_spec.evaluation_protocol_version
        );
        assert!((card.metrics["accuracy"] - 0.75).abs() < 1e-12);
        assert_eq!(card.status, ScorecardStatus::SelfReported);
        assert_eq!(card.visibility, ScorecardVisibility::Local);
    }
}
