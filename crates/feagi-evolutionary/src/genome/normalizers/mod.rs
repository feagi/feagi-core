// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Per-version genome normalizers.
//!
//! A `Normalizer` cleans up well-known bad values *within* a single schema
//! version. It is structurally distinct from a `Migrator`, which advances
//! the schema version. The chain runner invokes the normalizer for each
//! "arrived at" version (post-hop or starting version when no migration is
//! needed) before running that version's validator.
//!
//! Normalizers MUST follow the same invariants as migrators:
//! determinism, idempotence, bounded compute, no side channels, JSON only,
//! diagnostics over silence. They MUST NOT touch `genome_schema_version`;
//! the runner is the single source of truth for version bookkeeping.
//!
//! See `feagi-core/docs/GENOME_SCHEMA_VERSIONING.md` and
//! `crates/feagi-evolutionary/src/genome/README.md`.

use serde_json::Value;

use crate::genome::migration::MigrationError;
use crate::genome::schema::GenomeSchemaVersion;

pub mod v3;

pub use v3::V3Normalizer;

/// Diagnostic record produced by a single normalizer pass.
///
/// Every correction the normalizer performs MUST contribute at least one
/// entry to `transformations`. A normalizer that runs and produces zero
/// diagnostics on a genome that needed corrections is a bug.
#[derive(Debug, Clone)]
pub struct NormalizationDiagnostics {
    pub schema_version: GenomeSchemaVersion,
    pub transformations: Vec<String>,
}

impl NormalizationDiagnostics {
    pub fn new(schema_version: GenomeSchemaVersion) -> Self {
        Self {
            schema_version,
            transformations: Vec::new(),
        }
    }

    pub fn record(&mut self, msg: impl Into<String>) {
        self.transformations.push(msg.into());
    }

    /// True when no corrections were applied.
    pub fn is_clean(&self) -> bool {
        self.transformations.is_empty()
    }
}

/// In-place cleanup of a genome at a specific schema version.
///
/// Reuses `MigrationError` for failure reporting because normalizers and
/// migrators raise structurally identical errors (step name, version,
/// reason). Adding a parallel error enum would only duplicate the variants.
pub trait Normalizer: Send + Sync {
    /// Schema version this normalizer targets.
    fn schema_version(&self) -> GenomeSchemaVersion;

    /// Stable identifier for diagnostics and logs.
    fn name(&self) -> &'static str;

    /// Apply corrections in place. Returns diagnostics describing what
    /// changed, or an error on hard failure.
    fn normalize(&self, genome: &mut Value) -> Result<NormalizationDiagnostics, MigrationError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_starts_clean() {
        let d = NormalizationDiagnostics::new(GenomeSchemaVersion(3));
        assert!(d.is_clean());
        assert_eq!(d.transformations.len(), 0);
        assert_eq!(d.schema_version, GenomeSchemaVersion(3));
    }

    #[test]
    fn record_breaks_clean() {
        let mut d = NormalizationDiagnostics::new(GenomeSchemaVersion(3));
        d.record("set width 0 -> 1");
        assert!(!d.is_clean());
        assert_eq!(d.transformations, vec!["set width 0 -> 1".to_string()]);
    }
}
