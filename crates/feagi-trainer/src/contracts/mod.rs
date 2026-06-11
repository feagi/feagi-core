//! Public, versioned data contracts — the stable seam between `feagi-trainer` and its
//! consumers (the closed-source "FEAGI Trainer" app, Composer, and headless callers).
//!
//! Each contract carries a `schema_version` (wire/format) and is additionally versioned by
//! the crate's semver (API). Evolution is additive where possible; a breaking format change
//! bumps the relevant `SCHEMA_VERSION` and the crate major (ADR-006).
//!
//! v1 contracts: [`DatasetManifest`], [`IRSample`], [`RunSpec`], [`Scorecard`],
//! [`EvaluationSpec`], [`PredictionRecord`], [`RunSummary`]. (`RunEvent` and plugin-axis
//! descriptors arrive with later engine/UI wiring.)

pub mod common;
pub mod dataset_manifest;
pub mod evaluation_spec;
pub mod ir_sample;
pub mod metric_stats;
pub mod prediction_record;
pub mod run_spec;
pub mod run_summary;
pub mod scorecard;

pub use common::{
    BackendKind, ConnectomeHash, ContentHash, DatasetAssetId, DatasetVersionId,
    EvaluationProtocolVersion, GenomeVersionId, MetadataMap, MetadataValue, Modality, OutputType,
    PluginId, PluginRef, QuantizationFingerprint, RunId, SampleId, ScorecardId, Split, SplitId,
};
pub use dataset_manifest::{DatasetManifest, SplitDescriptor};
pub use evaluation_spec::EvaluationSpec;
pub use ir_sample::{CoordinateFrame, IRSample, Payload, TypedTarget};
pub use metric_stats::MetricStat;
pub use prediction_record::{PredictionRecord, TypedPrediction};
pub use run_spec::{
    CoderBinding, ExecutionMode, PinnedBinding, RewardPolicyBinding, RunSpec, SamplerBinding,
};
pub use run_summary::{RunStatus, RunSummary};
pub use scorecard::{BackendFingerprint, Scorecard, ScorecardStatus, ScorecardVisibility};
