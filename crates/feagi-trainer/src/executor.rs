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

use crate::binding::profile::{DecoderBindingProfile, EncoderBindingProfile};
use crate::binding::{DecoderPlugin, EncoderPlugin, FeagiRuntime, RewardPolicy};
use crate::contracts::prediction_record::SCHEMA_VERSION as PREDICTION_RECORD_SCHEMA_VERSION;
use crate::contracts::run_summary::SCHEMA_VERSION as RUN_SUMMARY_SCHEMA_VERSION;
use crate::contracts::scorecard::SCHEMA_VERSION as SCORECARD_SCHEMA_VERSION;
use crate::contracts::{
    BackendFingerprint, ContentHash, DatasetAssetId, IRSample, PredictionRecord, RunId, RunSpec,
    RunStatus, RunSummary, Scorecard, ScorecardId, ScorecardStatus, ScorecardVisibility,
};
use crate::error::TrainerError;
use crate::plugins::{MetricPackPlugin, MetricResult};

/// Tuning knobs for one rollout that are not part of the immutable [`RunSpec`] provenance.
///
/// `ticks_per_sample` is how many FEAGI bursts to advance between submitting a sample's
/// sensory frame and collecting its motor frame. It is supplied by the caller (resolved from
/// run/binding configuration) rather than hardcoded, so the same executor serves fast stubs
/// and slower live brains.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    if config.ticks_per_sample == 0 {
        return Err(TrainerError::Config(
            "ExecutorConfig.ticks_per_sample must be non-zero".to_string(),
        ));
    }

    let mut predictions: Vec<PredictionRecord> = Vec::with_capacity(samples.len());
    // The labeled subset that participates in scoring + reward (aligned by construction).
    let mut scored_predictions = Vec::new();
    let mut scored_targets = Vec::new();

    for sample in samples {
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
    }

    let metric_result = metric_pack.evaluate(&scored_predictions, &scored_targets)?;

    let summary = RunSummary {
        schema_version: RUN_SUMMARY_SCHEMA_VERSION,
        run_id: run_id.clone(),
        status: RunStatus::Completed,
        total_samples: samples.len() as u64,
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
        status: provenance.status,
        visibility: provenance.visibility,
        metadata: BTreeMap::new(),
    }
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
