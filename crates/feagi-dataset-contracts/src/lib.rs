//! # feagi-dataset-contracts
//!
//! Shared, dependency-light primitives for the FEAGI dataset/experience interchange
//! contracts. This crate holds the small building blocks that the larger contract
//! aggregates (`DatasetManifest`, `IRSample`, `RunSpec`, `Scorecard`, and the future
//! Experience Dataset Package) compose: typed identifier newtypes, content/connectome
//! hashes, plugin references, the backend kind, and the cross-cutting taxonomy enums
//! (`Modality`, `Split`, `OutputType`, `MetadataValue`).
//!
//! It is extracted so both `feagi-trainer` (the train/evaluate/benchmark engine) and
//! `feagi-experience-capture` (the capture/packager) can depend on one source of truth for
//! these primitives without either pulling in the other's engine. Keeping it lean is a hard
//! requirement: the Nano deployment profile depends on the capture side, so this crate must
//! stay serde-only (no engine, no I/O, no runtime).
//!
//! Design constraints (see `docs/FEAGI_TRAINER_*` and `docs/EXPERIENCE_CAPTURE_*`):
//! - Statically typed, minimal dynamic behavior (Rust/RTOS migration friendly).
//! - No implicit fallbacks: callers set every field explicitly.
//! - Deterministic serialization: maps use `BTreeMap` so key ordering is stable.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Generates a transparent `String` newtype used as a typed identifier.
///
/// The newtype keeps identifiers from being silently interchanged while still
/// serializing as a plain JSON string (`#[serde(transparent)]`).
macro_rules! string_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            /// Borrows the underlying string slice.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

string_id!(
    /// Identifies an immutable, versioned dataset registry record.
    DatasetVersionId
);
string_id!(
    /// Stable asset identifier reserved for future hosted dataset assets.
    ///
    /// Resolves to a local manifest/content hash today; the same id resolves to a hosted
    /// asset later without a contract change (ADR-012).
    DatasetAssetId
);
string_id!(
    /// Identifies a single sample within a dataset version.
    SampleId
);
string_id!(
    /// Identifies a single train/evaluate/benchmark run.
    RunId
);
string_id!(
    /// Identifies a generated `Scorecard` record.
    ScorecardId
);
string_id!(
    /// Optional lineage reference to the source genome version.
    GenomeVersionId
);
string_id!(
    /// Identifies a dataset split (e.g. `train`, `val`, `test`, custom).
    SplitId
);
string_id!(
    /// Identifies a registered plugin (adapter, sampler, encoder, decoder, metric pack,
    /// reward policy).
    PluginId
);
string_id!(
    /// Content-addressable hash of dataset content or a serialized artifact.
    ContentHash
);
string_id!(
    /// Hash of the pinned, serialized connectome that is the brain under test.
    ///
    /// This is the reproducibility/verification anchor (ADR-003, ADR-012).
    ConnectomeHash
);
string_id!(
    /// Versions the evaluation protocol *semantics* (episode definitions, aggregation
    /// windows, threshold/tie-break policies), not just metric names (Appendix C.3/D.3).
    EvaluationProtocolVersion
);

/// Versioned reference to a plugin on any of the four-axis (+reward) plugin model axes.
///
/// `version` participates in run provenance and, for evaluation/reward, in the run
/// comparability key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginRef {
    /// Registered plugin identifier.
    pub id: PluginId,
    /// Plugin implementation version (semantic version string).
    pub version: String,
}

/// Execution backend used for a run.
///
/// CPU is the deterministic verification baseline; GPU is recorded as a fingerprint only
/// and is not benchmark-grade for published scorecards (ADR-003).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendKind {
    /// CPU execution path (deterministic baseline).
    Cpu,
    /// GPU execution path (fingerprinted, not verification-grade).
    Gpu,
}

/// Data modality declared by a dataset and carried on each sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modality {
    /// Still imagery.
    Image,
    /// Video / frame sequences.
    Video,
    /// Natural-language text.
    Text,
    /// Tabular rows (e.g. CSV/TSV).
    Tabular,
    /// Combination of multiple modalities.
    Multimodal,
}

/// Dataset split assignment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Split {
    /// Training split.
    Train,
    /// Validation split.
    Val,
    /// Test/evaluation split.
    Test,
    /// Named custom split.
    Custom(String),
}

/// Structured-output task taxonomy referenced by `IRSample.target` and, later,
/// `PredictionRecord.output`.
///
/// This is a Trainer-side taxonomy that *maps onto* FEAGI's `WrappedIOType` (Appendix B.2);
/// it is intentionally not the core enum. The mapping to a concrete `WrappedIOType` + coder
/// is resolved and pinned into `RunSpec` at validation time. The mapping is many-to-many at
/// the registry level but resolves to exactly one concrete binding per run (Appendix D.1).
///
/// `Status` notes below reflect FEAGI coder availability (Appendix B.2/B.6):
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputType {
    /// Single-label classification. Status: exists.
    Class,
    /// Multi-label classification. Status: exists.
    ClassSet,
    /// Scalar regression. Status: exists.
    Scalar,
    /// Vector regression. Status: exists.
    Vector,
    /// Dense semantic segmentation mask. Status: exists (`SegmentedImageFrame`).
    SegmentationMask,
    /// 6DOF pose estimation. Status: exists (`PoseEstimationData` decoder present).
    Pose6Dof,
    /// Keypoint sets. Status: partial.
    Keypoints,
    /// Object-detection bounding-box set. Status: gap (new coder required, Appendix B.6).
    BboxSet,
}

/// Fingerprint of the brain-under-test's numeric quantization configuration.
///
/// Quantization (e.g. 8/16/32/64-bit neuron/synapse storage) is a determinant of results
/// on the evolving FEAGI NPU direction, so it must be captured in provenance and treated as
/// part of the brain identity for comparability. Kept as a string `level` + opaque
/// `details` so the contract does not couple to the in-flight `feagi-data` quantization
/// enums; the authoritative configuration is anchored by the pinned `connectome_hash`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuantizationFingerprint {
    /// Coarse quantization level descriptor (e.g. `bit8`, `bit16`, `bit32`, `bit64`, `mixed`).
    pub level: String,
    /// Opaque, structured detail of the quantization configuration.
    pub details: serde_json::Value,
}

/// A scalar-or-list metadata value (the `scalar | list` value type from the IR design).
///
/// Serialized untagged so it maps to natural JSON (`true`, `5`, `5.0`, `"x"`, `[...]`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetadataValue {
    /// Boolean scalar.
    Bool(bool),
    /// Integer scalar.
    Int(i64),
    /// Floating-point scalar.
    Float(f64),
    /// String scalar.
    Text(String),
    /// Homogeneous or heterogeneous list of values.
    List(Vec<MetadataValue>),
}

/// Convenience alias for the provenance/metadata maps used throughout the contracts.
///
/// `BTreeMap` is used (not `HashMap`) so serialized key ordering is deterministic.
pub type MetadataMap = BTreeMap<String, MetadataValue>;
