//! Integration test: the run executor drives a full pipeline end-to-end against a
//! deterministic stub runtime (plan Phase 1 test strategy).
//!
//! This exercises the public surface only — adapter -> sampler -> executor(encoder ->
//! StubFeagiRuntime -> decoder -> reward -> metric pack) -> RunSummary + Scorecard — without a
//! live FEAGI. The runtime is the only stubbed collaborator; the executor (the subject under
//! test) is real, per the project's mocking policy.

use feagi_trainer::adapters::{TabularCsvAdapter, TabularCsvConfig};
use feagi_trainer::binding::encoding_scheme::{BinSpacing, EncodingScheme};
use feagi_trainer::binding::profile::{DecoderBindingProfile, EncoderBindingProfile};
use feagi_trainer::binding::{DecoderPlugin, EncoderPlugin, PainPleasureReward, StubFeagiRuntime};
use feagi_trainer::contracts::common::{
    BackendKind, ConnectomeHash, EvaluationProtocolVersion, PluginId, Split,
};
use feagi_trainer::contracts::ir_sample::Payload;
use feagi_trainer::contracts::run_spec::{
    CoderBinding, ExecutionMode, PinnedBinding, RewardPolicyBinding, SamplerBinding,
};
use feagi_trainer::contracts::{
    BackendFingerprint, ContentHash, DatasetAssetId, DatasetVersionId, IRSample, PluginRef, RunId,
    RunSpec, RunStatus, ScorecardId, ScorecardStatus, ScorecardVisibility, SplitId,
    TypedPrediction,
};
use feagi_trainer::error::TrainerError;
use feagi_trainer::executor::{
    assemble_scorecard, run_rollout, ExecutorConfig, ScorecardProvenance,
};
use feagi_trainer::metrics::ClassificationMetricPack;
use feagi_trainer::plugins::{AdapterPlugin, DatasetSource, SamplerPlugin};
use feagi_trainer::samplers::SequentialSampler;

/// IRIS-shaped rows whose features deliberately one-hot the class, so an identity stub runtime
/// + argmax decoder reproduce the class — letting the test assert a known-good score.
const ONE_HOT_CSV: &str = "f0,f1,f2,species\n\
1,0,0,setosa\n\
0,1,0,versicolor\n\
0,0,1,virginica\n\
1,0,0,setosa\n";

/// Emits the sample's tabular payload directly as the sensory frame.
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

/// Argmax over the motor channel vector -> predicted class id.
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

fn adapter_config() -> TabularCsvConfig {
    TabularCsvConfig {
        dataset_name: "one_hot".to_string(),
        has_header: true,
        feature_columns: vec![0, 1, 2],
        label_column: 3,
        class_labels: vec![
            "setosa".to_string(),
            "versicolor".to_string(),
            "virginica".to_string(),
        ],
        split: Split::Test,
        split_id: SplitId("test".to_string()),
    }
}

fn run_spec() -> RunSpec {
    RunSpec {
        schema_version: feagi_trainer::contracts::run_spec::SCHEMA_VERSION,
        run_id: RunId("run-e2e-0001".to_string()),
        dataset_version_id: DatasetVersionId("one_hot@1".to_string()),
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
                properties: serde_json::json!({}),
            },
            decoder: CoderBinding {
                io_type: "Percentage".to_string(),
                coder_id: "percentage_decoder".to_string(),
                cortical_area_id: "o____C".to_string(),
                properties: serde_json::json!({}),
            },
        },
        reward_policy: RewardPolicyBinding {
            plugin: PluginRef {
                id: PluginId("reward.pain_pleasure".to_string()),
                version: "1.0.0".to_string(),
            },
            config: serde_json::json!({}),
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
fn full_pipeline_produces_summary_and_scorecard() {
    // Ingest + stream + plan, exactly as the real pipeline does.
    let adapter = TabularCsvAdapter::new(adapter_config());
    let source = DatasetSource {
        uri: "mem://one_hot.csv".to_string(),
        bytes: ONE_HOT_CSV.as_bytes().to_vec(),
    };
    let manifest = adapter.discover(&source).expect("discover");
    assert!(adapter.validate(&manifest).expect("validate").passed);
    let samples = adapter
        .stream(&source, &SplitId("test".to_string()))
        .expect("stream");
    let order = SequentialSampler::new().plan(samples.len(), 42);
    let ordered: Vec<IRSample> = order.iter().map(|&i| samples[i].clone()).collect();
    assert_eq!(ordered.len(), 4);

    // Drive the closed loop against the deterministic stub runtime.
    let mut runtime = StubFeagiRuntime::identity();
    let mut encoder = PassthroughEncoder;
    let mut decoder = ArgmaxDecoder;
    let reward = PainPleasureReward::new(0.9).unwrap();
    let metric = ClassificationMetricPack::new();
    let spec = run_spec();

    let outcome = run_rollout(
        &spec.run_id,
        &ordered,
        &mut runtime,
        &mut encoder,
        &EncoderBindingProfile {
            cortical_area_id: "iv00_C".to_string(),
            channels: 3,
            scheme: EncodingScheme::PopulationSingleSpike {
                bins: 1,
                spacing: BinSpacing::Linear,
            },
        },
        &mut decoder,
        &DecoderBindingProfile {
            cortical_area_id: "o____C".to_string(),
            class_count: 3,
            bins: 1,
        },
        &reward,
        &metric,
        &ExecutorConfig {
            ticks_per_sample: 3,
        },
    )
    .expect("rollout");

    // One-hot features through an identity runtime => perfect classification.
    assert_eq!(outcome.summary.status, RunStatus::Completed);
    assert_eq!(outcome.summary.total_samples, 4);
    assert_eq!(outcome.summary.evaluated_samples, 4);
    assert_eq!(outcome.predictions.len(), 4);
    assert!((outcome.metric_result.metrics["accuracy"] - 1.0).abs() < 1e-12);
    assert_eq!(runtime.submitted_rewards().len(), 4);

    // Assemble a portable scorecard from the run provenance + computed metrics.
    let card = assemble_scorecard(
        &spec,
        &outcome.summary.metrics,
        ScorecardProvenance {
            scorecard_id: ScorecardId("sc-e2e-0001".to_string()),
            dataset_asset_id: DatasetAssetId("local:one_hot".to_string()),
            dataset_version: "1.0.0".to_string(),
            dataset_content_hash: ContentHash("sha256:one_hot".to_string()),
            backend_fingerprint: BackendFingerprint {
                backend: BackendKind::Cpu,
                descriptor: "stub-cpu".to_string(),
                quantization: None,
                trainer_version: env!("CARGO_PKG_VERSION").to_string(),
                feagi_core_version: "0.0.12".to_string(),
            },
            status: ScorecardStatus::SelfReported,
            visibility: ScorecardVisibility::Local,
        },
    );

    assert_eq!(card.connectome_hash, spec.connectome_hash);
    assert_eq!(card.split_id, spec.split_id);
    assert!((card.metrics["accuracy"] - 1.0).abs() < 1e-12);

    // The scorecard must round-trip as a portable artifact.
    let json = serde_json::to_string(&card).expect("serialize scorecard");
    let restored: feagi_trainer::contracts::Scorecard =
        serde_json::from_str(&json).expect("deserialize scorecard");
    assert_eq!(card, restored);
}
