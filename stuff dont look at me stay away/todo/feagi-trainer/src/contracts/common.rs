//! Shared primitives used across the FEAGI Trainer public contracts.
//!
//! These primitives now live in the standalone [`feagi_dataset_contracts`] crate so that
//! `feagi-trainer` and the planned `feagi-experience-capture` crate share one source of
//! truth without coupling to each other's engine (Phase 1a; Option B in
//! `docs/EXPERIENCE_TRAINER_E2E_IMPLEMENTATION_PLAN.md`). This module re-exports them so the
//! Trainer's existing `contracts::common::*` paths and the public `contracts::*` surface are
//! unchanged.

pub use feagi_dataset_contracts::{
    BackendKind, ConnectomeHash, ContentHash, DatasetAssetId, DatasetVersionId,
    EvaluationProtocolVersion, GenomeVersionId, MetadataMap, MetadataValue, Modality, OutputType,
    PluginId, PluginRef, QuantizationFingerprint, RunId, SampleId, ScorecardId, Split, SplitId,
};
