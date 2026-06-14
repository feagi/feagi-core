//! Integration test: the N-seed repeat orchestrator (`stats::run_repeated`) composed with the
//! real run executor and the stats-aware scorecard assembler.
//!
//! This wires the publication-credibility path end-to-end without a live FEAGI: each repeat plans
//! the sampler order from a derived seed, runs a full rollout against a **freshly reset** stub
//! runtime (a new `StubFeagiRuntime` per repeat models restoring the pinned connectome), and the
//! per-repeat metrics are aggregated into the `Scorecard.metric_stats` distribution. The runtime
//! is the only stubbed collaborator; the orchestrator, executor, and aggregator under test are
//! real, per the project's mocking policy.

use std::collections::BTreeMap;

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
    RunSpec, ScorecardId, ScorecardStatus, ScorecardVisibility, SplitId, TypedPrediction,
};
use feagi_trainer::error::TrainerError;
use feagi_trainer::executor::{
    assemble_scorecard_with_stats, run_rollout, ExecutorConfig, ScorecardProvenance,
};
use feagi_trainer::metrics::ClassificationMetricPack;
use feagi_trainer::plugins::{AdapterPlugin, DatasetSource, SamplerPlugin};
use feagi_trainer::samplers::SequentialSampler;
use feagi_trainer::stats::{run_repeated, RepeatConfig};

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
        run_id: RunId("run-rep-0001".to_string()),
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
        genome_schema_version: Some(3),
        execution_mode: ExecutionMode::Remote,
        backend: BackendKind::Cpu,
        quantization: None,
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

/// Runs one full rollout for a given seed against a freshly reset stub runtime, returning its
/// metric map. Re-planning the order from `seed` is the variance source for multi-seed repeats.
fn rollout_for_seed(
    spec: &RunSpec,
    samples: &[IRSample],
    seed: u64,
) -> Result<BTreeMap<String, f64>, TrainerError> {
    let order = SequentialSampler::new().plan(samples.len(), seed);
    let ordered: Vec<IRSample> = order.iter().map(|&i| samples[i].clone()).collect();

    // A fresh runtime per repeat models restoring the pinned connectome between repeats.
    let mut runtime = StubFeagiRuntime::identity();
    let mut encoder = PassthroughEncoder;
    let mut decoder = ArgmaxDecoder;
    let reward = PainPleasureReward::new(0.9).unwrap();
    let metric = ClassificationMetricPack::new();

    let outcome = run_rollout(
        &spec.run_id,
        &ordered,
        &mut runtime,
        &mut encoder,
        &encoder_profile(),
        &mut decoder,
        &decoder_profile(),
        &reward,
        &metric,
        &ExecutorConfig {
            ticks_per_sample: 3,
        },
    )?;
    Ok(outcome.summary.metrics)
}

#[test]
fn repeated_rollout_produces_scorecard_with_metric_stats() {
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

    let spec = run_spec();
    let config = RepeatConfig {
        repeats: 5,
        confidence_level: 0.95,
    };

    let repeated = run_repeated(&config, spec.sampler.seed, |seed| {
        rollout_for_seed(&spec, &samples, seed)
    })
    .expect("repeated rollout");

    // Five repeats, each scored.
    assert_eq!(repeated.per_repeat_metrics.len(), 5);
    let acc = &repeated.metric_stats["accuracy"];
    assert_eq!(acc.n, 5);
    // One-hot features through an identity runtime => every repeat scores a perfect 1.0, so the
    // distribution is a degenerate point: mean 1.0, zero spread, interval collapsed.
    assert!((acc.mean - 1.0).abs() < 1e-12);
    assert_eq!(acc.stddev, 0.0);
    assert!((acc.ci_low - 1.0).abs() < 1e-12 && (acc.ci_high - 1.0).abs() < 1e-12);
    assert_eq!(acc.confidence_level, 0.95);
    assert!((repeated.mean_metrics["accuracy"] - 1.0).abs() < 1e-12);

    // Stamp the distribution into a portable scorecard and confirm it round-trips with the stats.
    let card = assemble_scorecard_with_stats(
        &spec,
        &repeated.mean_metrics,
        repeated.metric_stats.clone(),
        ScorecardProvenance {
            scorecard_id: ScorecardId("sc-rep-0001".to_string()),
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

    let stats = card.metric_stats.as_ref().expect("metric_stats present");
    assert_eq!(stats["accuracy"].n, 5);

    let json = serde_json::to_string(&card).expect("serialize scorecard");
    assert!(json.contains("metric_stats"));
    let restored: feagi_trainer::contracts::Scorecard =
        serde_json::from_str(&json).expect("deserialize scorecard");
    assert_eq!(card, restored);
}
