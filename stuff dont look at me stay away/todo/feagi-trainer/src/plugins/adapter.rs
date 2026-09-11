//! Adapter axis — ingest source data and map it into the canonical IR (design Section 5.2).
//!
//! Adapters carry all dataset-specific parsing. They emit `DatasetManifest`s (discovery),
//! validate them, and stream `IRSample`s for a split. The trait deliberately takes an
//! in-memory [`DatasetSource`] rather than performing file/URI I/O, so the axis is
//! platform-agnostic and unit-testable; resolving a URI to bytes is an orchestration concern.

use crate::contracts::common::PluginRef;
use crate::contracts::{DatasetManifest, IRSample, SplitId};
use crate::error::TrainerError;

/// An in-memory dataset source handed to an adapter.
///
/// Holds the logical `uri` (recorded in provenance) and the raw `bytes`. The adapter
/// interprets the bytes according to its format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatasetSource {
    /// Logical source location, recorded in the manifest for provenance.
    pub uri: String,
    /// Raw source bytes.
    pub bytes: Vec<u8>,
}

/// Outcome of validating a discovered manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    /// `true` when no blocking issues were found.
    pub passed: bool,
    /// Human-readable descriptions of any issues found.
    pub issues: Vec<String>,
}

/// Converts a source format into the canonical FEAGI Trainer intermediate representation.
pub trait AdapterPlugin {
    /// Identifies this adapter (axis provenance).
    fn plugin_ref(&self) -> PluginRef;

    /// Inspects a source and produces a candidate dataset manifest.
    fn discover(&self, source: &DatasetSource) -> Result<DatasetManifest, TrainerError>;

    /// Validates a manifest, returning a structured report (never panics on bad data).
    fn validate(&self, manifest: &DatasetManifest) -> Result<ValidationReport, TrainerError>;

    /// Streams the samples assigned to `split` as canonical `IRSample`s.
    ///
    /// Returns an explicit error for an unknown split or malformed rows; it never silently
    /// drops or coerces data.
    fn stream(
        &self,
        source: &DatasetSource,
        split: &SplitId,
    ) -> Result<Vec<IRSample>, TrainerError>;
}
