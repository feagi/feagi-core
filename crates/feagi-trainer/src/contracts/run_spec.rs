//! `RunSpec` v1 — the immutable specification of a train/evaluate/benchmark run.
//!
//! A `RunSpec` is created and frozen at validation time (design Section 5.7). It captures
//! every determinant of a run's result so the run is reproducible and comparable:
//! dataset + split, the four plugin axes, the pinned reward policy, the evaluation protocol
//! version, the pinned connectome under test, and the execution backend.
//!
//! Reproducibility model (ADR-003): benchmark mode pins a serialized connectome
//! (`connectome_hash`) as the brain under test, sidestepping the unseeded development RNG.
//! CPU is the deterministic baseline; GPU is fingerprinted only.
//!
//! Comparability (Appendix D.2/D.3): `(evaluation_protocol_version, reward_policy.version)`
//! is part of the run comparability key — runs differing on either are not comparable.

use serde::{Deserialize, Serialize};

use super::common::{
    BackendKind, ConnectomeHash, DatasetVersionId, EvaluationProtocolVersion, GenomeVersionId,
    PluginRef, QuantizationFingerprint, RunId, SplitId,
};

/// Wire/format version of the `RunSpec` contract.
pub const SCHEMA_VERSION: u32 = 1;

/// How the run reaches the FEAGI runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    /// In-process, tick-locked FEAGI engine (preferred for benchmark determinism).
    Embedded,
    /// Drive an existing FEAGI runtime over a transport (interactive/non-benchmark).
    ///
    /// Endpoint/timeout configuration is resolved via `feagi-config` at execution time and
    /// is intentionally not stored here, to avoid hardcoded endpoints in provenance.
    Remote,
}

/// Sampler axis selection plus the seed that makes ordering deterministic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SamplerBinding {
    /// Selected sampler plugin.
    pub plugin: PluginRef,
    /// Seed for deterministic ordering.
    pub seed: u64,
}

/// One resolved-and-pinned coder binding (encoder or decoder side).
///
/// Encoder/decoder plugins are thin binding *selectors* over FEAGI's existing coders; this
/// records the concrete selection (the `WrappedIOType` identifier, the chosen coder, the
/// target/source cortical area, and the frozen coder JSON properties) so the binding is
/// reproducible without this crate depending on the coder library (choice locked in review).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoderBinding {
    /// FEAGI `WrappedIOType` identifier the `OutputType` resolved to.
    pub io_type: String,
    /// Concrete FEAGI coder selected for this side of the binding.
    pub coder_id: String,
    /// Target (encoder) or source (decoder) cortical area id.
    pub cortical_area_id: String,
    /// Frozen coder JSON properties (mirrors `JSON{Encoder,Decoder}Properties`).
    pub properties: serde_json::Value,
}

/// The single concrete binding the run is pinned to (Appendix D.1).
///
/// `OutputType -> WrappedIOType` is many-to-many at the registry level but must resolve to
/// exactly one encoder + decoder binding per run, frozen here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PinnedBinding {
    /// Sensory-side (IR -> FEAGI) binding.
    pub encoder: CoderBinding,
    /// Motor/OPU-side (FEAGI -> prediction) binding.
    pub decoder: CoderBinding,
}

/// The pinned, versioned reward policy (first-class fifth axis — Appendix D.2).
///
/// Maps label/correctness/outcome to a reward signal delivered into FEAGI's native affect
/// channels (Pain/Pleasure/Fear/Hope). Its `version` is part of the comparability key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RewardPolicyBinding {
    /// Selected reward-policy plugin.
    pub plugin: PluginRef,
    /// Frozen reward-policy configuration.
    pub config: serde_json::Value,
}

/// Immutable run specification (design Section 5.7, refined by Appendix D.4).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunSpec {
    /// Wire/format version; must equal [`SCHEMA_VERSION`].
    pub schema_version: u32,
    /// Unique run identifier.
    pub run_id: RunId,
    /// Dataset version under test.
    pub dataset_version_id: DatasetVersionId,
    /// Split to run against.
    pub split_id: SplitId,
    /// Adapter axis selection.
    pub adapter: PluginRef,
    /// Sampler axis selection + seed.
    pub sampler: SamplerBinding,
    /// Optional transform-graph version id (transforms are versioned in provenance).
    pub transform_graph_version: Option<String>,
    /// Resolved + pinned encoder/decoder binding.
    pub binding: PinnedBinding,
    /// Pinned, versioned reward policy.
    pub reward_policy: RewardPolicyBinding,
    /// Metric pack axis selection.
    pub metric_pack: PluginRef,
    /// Versioned evaluation-protocol semantics (comparability key).
    pub evaluation_protocol_version: EvaluationProtocolVersion,
    /// Pinned serialized connectome — the brain under test / verification anchor.
    pub connectome_hash: ConnectomeHash,
    /// Optional source-genome lineage reference.
    pub genome_version_id: Option<GenomeVersionId>,
    /// FEAGI execution mode.
    pub execution_mode: ExecutionMode,
    /// Execution backend (CPU baseline; GPU fingerprinted only).
    pub backend: BackendKind,
    /// Optional quantization fingerprint of the brain under test (forward-compatible with
    /// the quantization-capable NPU direction; `None` when not applicable).
    pub quantization: Option<QuantizationFingerprint>,
}

#[cfg(test)]
mod tests {
    use super::super::common::PluginId;
    use super::*;
    use serde_json::json;

    fn plugin(id: &str) -> PluginRef {
        PluginRef {
            id: PluginId(id.to_string()),
            version: "1.0.0".to_string(),
        }
    }

    fn iris_run_spec() -> RunSpec {
        RunSpec {
            schema_version: SCHEMA_VERSION,
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
                plugin: plugin("supervised_pain_pleasure"),
                config: json!({"correct": "pleasure", "incorrect": "pain"}),
            },
            metric_pack: plugin("classification"),
            evaluation_protocol_version: EvaluationProtocolVersion("clf-v1".to_string()),
            connectome_hash: ConnectomeHash("sha256:connectome".to_string()),
            genome_version_id: None,
            execution_mode: ExecutionMode::Embedded,
            backend: BackendKind::Cpu,
            quantization: None,
        }
    }

    #[test]
    fn schema_version_is_pinned() {
        assert_eq!(iris_run_spec().schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn json_round_trip_preserves_run_spec() {
        let spec = iris_run_spec();
        let json = serde_json::to_string(&spec).expect("serialize");
        let restored: RunSpec = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(spec, restored);
    }
}
