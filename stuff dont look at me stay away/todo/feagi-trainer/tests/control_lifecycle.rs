//! Integration test: the Control API (`control::ClosureRunControl`) composed with the
//! event-streaming executor (`executor::run_rollout_with_events`).
//!
//! This exercises the full ADR-011 seam a desktop/headless host uses: a host-supplied rollout
//! closure runs a real rollout against a stub runtime (the only stubbed collaborator, per the
//! project's mocking policy) and streams `Progress` / `MetricUpdate` events; the controller layers
//! the run lifecycle (`Running` -> `ScorecardReady` -> `Completed`, or `Failed`) and manages
//! status. Both the successful and cooperatively-cancelled paths are covered.

use std::collections::BTreeMap;

use feagi_trainer::binding::encoding_scheme::{BinSpacing, EncodingScheme};
use feagi_trainer::binding::profile::{DecoderBindingProfile, EncoderBindingProfile};
use feagi_trainer::binding::{DecoderPlugin, EncoderPlugin, PainPleasureReward, StubFeagiRuntime};
use feagi_trainer::contracts::common::{PluginId, Split};
use feagi_trainer::contracts::ir_sample::Payload;
use feagi_trainer::contracts::run_summary::SCHEMA_VERSION as RUN_SUMMARY_SCHEMA_VERSION;
use feagi_trainer::contracts::{
    DatasetVersionId, IRSample, Modality, OutputType, PluginRef, RunEventKind, RunId, RunStatus,
    RunSummary, SampleId, ScorecardId, TypedPrediction, TypedTarget,
};
use feagi_trainer::control::{
    CancelToken, ClosureRunControl, CollectingEventSink, RunControl, RunEventSink,
};
use feagi_trainer::error::TrainerError;
use feagi_trainer::executor::{run_rollout_with_events, ExecutorConfig};
use feagi_trainer::metrics::ClassificationMetricPack;

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
            .max_by(|a, b| a.1.partial_cmp(b.1).expect("no NaN"))
            .map(|(idx, _)| idx as u32)
            .ok_or_else(|| TrainerError::Runtime("empty motor frame".to_string()))?;
        Ok(TypedPrediction::Class {
            class_id: argmax,
            scores: motor,
        })
    }
}

fn one_hot(class_id: u32) -> Vec<f64> {
    (0..3)
        .map(|c| if c == class_id { 1.0 } else { 0.0 })
        .collect()
}

fn sample(idx: usize, class_id: u32) -> IRSample {
    IRSample {
        schema_version: feagi_trainer::contracts::ir_sample::SCHEMA_VERSION,
        sample_id: SampleId(format!("s-{idx:04}")),
        dataset_version_id: DatasetVersionId("ctrl@1".to_string()),
        split: Split::Test,
        modality: Modality::Tabular,
        payload: Payload::Tabular(one_hot(class_id)),
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

fn samples() -> Vec<IRSample> {
    vec![sample(0, 0), sample(1, 1), sample(2, 2)]
}

#[test]
fn controller_runs_full_rollout_and_streams_lifecycle_events() {
    let run_id = RunId("run-ctrl-ok".to_string());
    let scorecard_id = ScorecardId("sc-ctrl-1".to_string());
    let data = samples();

    // The host closure performs the real rollout (streaming Progress/MetricUpdate) and reports a
    // terminal summary that carries the produced scorecard id, so the controller emits
    // ScorecardReady before Completed.
    let rollout = {
        let run_id = run_id.clone();
        let scorecard_id = scorecard_id.clone();
        move |events: &mut dyn RunEventSink,
              cancel: &CancelToken|
              -> Result<RunSummary, TrainerError> {
            let mut runtime = StubFeagiRuntime::identity();
            let mut encoder = PassthroughEncoder;
            let mut decoder = ArgmaxDecoder;
            let reward = PainPleasureReward::new(0.9).unwrap();
            let metric = ClassificationMetricPack::new();

            let outcome = run_rollout_with_events(
                &run_id,
                &data,
                &mut runtime,
                &mut encoder,
                &encoder_profile(),
                &mut decoder,
                &decoder_profile(),
                &reward,
                &metric,
                &ExecutorConfig {
                    ticks_per_sample: 2,
                },
                events,
                cancel,
            )?;

            Ok(RunSummary {
                scorecard_id: Some(scorecard_id.clone()),
                ..outcome.summary
            })
        }
    };

    let mut control = ClosureRunControl::new(run_id, rollout);
    let mut sink = CollectingEventSink::default();
    assert_eq!(control.status(), RunStatus::Created);

    let summary = control.execute(&mut sink).expect("run ok");
    assert_eq!(control.status(), RunStatus::Completed);
    assert_eq!(summary.scorecard_id, Some(scorecard_id));

    let kinds: Vec<&RunEventKind> = sink.events.iter().map(|e| &e.kind).collect();
    // Lifecycle: Running, then 3 Progress, then aggregate MetricUpdate, then ScorecardReady,
    // then Completed.
    assert!(matches!(kinds.first(), Some(RunEventKind::Running)));
    assert!(matches!(kinds.last(), Some(RunEventKind::Completed)));
    let progress = kinds
        .iter()
        .filter(|k| matches!(k, RunEventKind::Progress { .. }))
        .count();
    assert_eq!(progress, 3);
    assert!(kinds
        .iter()
        .any(|k| matches!(k, RunEventKind::MetricUpdate { .. })));
    assert!(kinds
        .iter()
        .any(|k| matches!(k, RunEventKind::ScorecardReady { .. })));
}

#[test]
fn controller_surfaces_cancellation_as_failed() {
    let run_id = RunId("run-ctrl-cancel".to_string());
    let data = samples();

    let rollout = move |events: &mut dyn RunEventSink,
                        cancel: &CancelToken|
          -> Result<RunSummary, TrainerError> {
        let mut runtime = StubFeagiRuntime::identity();
        let mut encoder = PassthroughEncoder;
        let mut decoder = ArgmaxDecoder;
        let reward = PainPleasureReward::new(0.9).unwrap();
        let metric = ClassificationMetricPack::new();

        let outcome = run_rollout_with_events(
            &RunId("run-ctrl-cancel".to_string()),
            &data,
            &mut runtime,
            &mut encoder,
            &encoder_profile(),
            &mut decoder,
            &decoder_profile(),
            &reward,
            &metric,
            &ExecutorConfig {
                ticks_per_sample: 2,
            },
            events,
            cancel,
        )?;
        Ok(RunSummary {
            schema_version: RUN_SUMMARY_SCHEMA_VERSION,
            ..outcome.summary
        })
    };

    let mut control = ClosureRunControl::new(run_id, rollout);
    // Host requests stop before execution begins.
    control.cancel_token().cancel();
    let mut sink = CollectingEventSink::default();

    let error = control.execute(&mut sink).unwrap_err();
    assert!(matches!(error, TrainerError::Cancelled(_)));
    assert_eq!(control.status(), RunStatus::Failed);

    let kinds: Vec<&RunEventKind> = sink.events.iter().map(|e| &e.kind).collect();
    // Running first, Failed last; no Completed.
    assert!(matches!(kinds.first(), Some(RunEventKind::Running)));
    assert!(matches!(kinds.last(), Some(RunEventKind::Failed { .. })));
    assert!(!kinds.iter().any(|k| matches!(k, RunEventKind::Completed)));
}
