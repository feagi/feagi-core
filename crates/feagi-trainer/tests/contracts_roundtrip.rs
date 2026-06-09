//! Integration tests for the public contracts.
//!
//! These exercise the crate as an external consumer would: build each v1 contract from the
//! public API, serialize to JSON, deserialize, and assert structural equality and that the
//! `schema_version` is preserved on the wire.

use std::collections::BTreeMap;

use feagi_trainer::contracts::common::PluginId;
use feagi_trainer::contracts::{
    dataset_manifest, ir_sample, run_spec, scorecard, BackendFingerprint, BackendKind,
    CoderBinding, ConnectomeHash, ContentHash, DatasetAssetId, DatasetManifest, DatasetVersionId,
    EvaluationProtocolVersion, IRSample, Modality, OutputType, Payload, PinnedBinding, PluginRef,
    RewardPolicyBinding, RunId, RunSpec, SampleId, SamplerBinding, Scorecard, ScorecardId,
    ScorecardStatus, ScorecardVisibility, Split, SplitDescriptor, SplitId, TypedTarget,
};
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Serializes `value`, deserializes it back, and asserts equality.
fn assert_json_round_trip<T>(value: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let json = serde_json::to_string(value).expect("serialize");
    let restored: T = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(value, &restored);
}

/// Asserts the serialized JSON carries the expected `schema_version`.
fn assert_schema_version<T: Serialize>(value: &T, expected: u32) {
    let json = serde_json::to_value(value).expect("to_value");
    assert_eq!(json["schema_version"], serde_json::json!(expected));
}

fn plugin(id: &str) -> PluginRef {
    PluginRef {
        id: PluginId(id.to_string()),
        version: "1.0.0".to_string(),
    }
}

fn iris_manifest() -> DatasetManifest {
    DatasetManifest {
        schema_version: dataset_manifest::SCHEMA_VERSION,
        dataset_version_id: DatasetVersionId("iris@1".to_string()),
        dataset_asset_id: DatasetAssetId("local:iris".to_string()),
        dataset_version: "1.0.0".to_string(),
        source_uri: "file:///datasets/iris.csv".to_string(),
        content_hash: ContentHash("sha256:abc".to_string()),
        schema_fingerprint: ContentHash("sha256:schema".to_string()),
        modality: Modality::Tabular,
        output_type: OutputType::Class,
        splits: vec![
            SplitDescriptor {
                id: SplitId("train".to_string()),
                split: Split::Train,
                sample_count: 120,
            },
            SplitDescriptor {
                id: SplitId("test".to_string()),
                split: Split::Test,
                sample_count: 30,
            },
        ],
        metadata: BTreeMap::new(),
    }
}

fn iris_sample() -> IRSample {
    IRSample {
        schema_version: ir_sample::SCHEMA_VERSION,
        sample_id: SampleId("iris-0001".to_string()),
        dataset_version_id: DatasetVersionId("iris@1".to_string()),
        split: Split::Train,
        modality: Modality::Tabular,
        payload: Payload::Tabular(vec![5.1, 3.5, 1.4, 0.2]),
        target: Some(TypedTarget::Class {
            class_id: 0,
            label: Some("setosa".to_string()),
        }),
        output_type: OutputType::Class,
        coordinate_frame: None,
        timestamp: None,
        metadata: BTreeMap::new(),
    }
}

fn iris_run_spec() -> RunSpec {
    RunSpec {
        schema_version: run_spec::SCHEMA_VERSION,
        run_id: RunId("run-0001".to_string()),
        dataset_version_id: DatasetVersionId("iris@1".to_string()),
        split_id: SplitId("test".to_string()),
        adapter: plugin("tabular_csv"),
        sampler: SamplerBinding {
            plugin: plugin("sequential"),
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
            plugin: plugin("supervised_pain_pleasure"),
            config: serde_json::json!({"correct": "pleasure", "incorrect": "pain"}),
        },
        metric_pack: plugin("classification"),
        evaluation_protocol_version: EvaluationProtocolVersion("clf-v1".to_string()),
        connectome_hash: ConnectomeHash("sha256:connectome".to_string()),
        genome_version_id: None,
        execution_mode: run_spec::ExecutionMode::Embedded,
        backend: BackendKind::Cpu,
        quantization: None,
    }
}

fn iris_scorecard() -> Scorecard {
    let mut metrics = BTreeMap::new();
    metrics.insert("accuracy".to_string(), 0.9667);
    metrics.insert("macro_f1".to_string(), 0.9655);
    Scorecard {
        schema_version: scorecard::SCHEMA_VERSION,
        scorecard_id: ScorecardId("sc-0001".to_string()),
        connectome_hash: ConnectomeHash("sha256:connectome".to_string()),
        genome_version_id: None,
        dataset_asset_id: DatasetAssetId("local:iris".to_string()),
        dataset_version: "1.0.0".to_string(),
        dataset_content_hash: ContentHash("sha256:abc".to_string()),
        evaluation_protocol_version: EvaluationProtocolVersion("clf-v1".to_string()),
        metric_pack: plugin("classification"),
        split_id: SplitId("test".to_string()),
        backend_fingerprint: BackendFingerprint {
            backend: BackendKind::Cpu,
            descriptor: "x86_64-cpu".to_string(),
            quantization: None,
            trainer_version: env!("CARGO_PKG_VERSION").to_string(),
            feagi_core_version: "0.0.12".to_string(),
        },
        metrics,
        status: ScorecardStatus::SelfReported,
        visibility: ScorecardVisibility::Local,
        metadata: BTreeMap::new(),
    }
}

#[test]
fn dataset_manifest_round_trips() {
    let manifest = iris_manifest();
    assert_schema_version(&manifest, dataset_manifest::SCHEMA_VERSION);
    assert_json_round_trip(&manifest);
}

#[test]
fn ir_sample_round_trips() {
    let sample = iris_sample();
    assert_schema_version(&sample, ir_sample::SCHEMA_VERSION);
    assert_json_round_trip(&sample);
}

#[test]
fn run_spec_round_trips() {
    let spec = iris_run_spec();
    assert_schema_version(&spec, run_spec::SCHEMA_VERSION);
    assert_json_round_trip(&spec);
}

#[test]
fn scorecard_round_trips() {
    let card = iris_scorecard();
    assert_schema_version(&card, scorecard::SCHEMA_VERSION);
    assert_json_round_trip(&card);
}
