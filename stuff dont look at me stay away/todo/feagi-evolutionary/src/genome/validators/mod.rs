// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Per-version genome validators.
//!
//! A `Validator` checks structural integrity, parameter ranges, and
//! cross-references at a *specific* `GenomeSchemaVersion`. The chain runner
//! invokes per-version validators between hops as **advisory** and the
//! validator at the final target version as **blocking**.
//!
//! See `feagi-core/docs/GENOME_SCHEMA_VERSIONING.md` and
//! `crates/feagi-evolutionary/src/genome/README.md` for the design contract.
//!
//! Validators MUST NOT mutate the genome. Mutation belongs in
//! `crate::genome::migration::Migrator`. See the README's anti-patterns.

use serde_json::Value;

use crate::genome::schema::GenomeSchemaVersion;

pub mod v3;

pub use v3::V3Validator;

/// Outcome of running a single validator against a genome.
///
/// `errors` are blocking issues. `warnings` are advisory. The validator does
/// not decide whether to abort the chain; the chain runner makes that call
/// based on whether the validator was the latest (blocking) or intermediate
/// (advisory).
#[derive(Debug, Clone, Default)]
pub struct ValidationReport {
    /// Schema version this report describes. `None` only when constructed
    /// via `Default` for testing scaffolding; production validators always
    /// stamp their version.
    pub schema_version: Option<GenomeSchemaVersion>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationReport {
    pub fn new(schema_version: GenomeSchemaVersion) -> Self {
        Self {
            schema_version: Some(schema_version),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, msg: impl Into<String>) {
        self.errors.push(msg.into());
    }

    pub fn add_warning(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }

    /// True when the report carries at least one blocking error.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// True when the report carries no errors and no warnings.
    pub fn is_clean(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }
}

/// Validates a genome at a specific schema version.
///
/// Validators are stateless; the trait is `Send + Sync` so registries can be
/// shared across threads. Implementations must not mutate the input or
/// perform I/O.
pub trait Validator: Send + Sync {
    /// The schema version this validator targets.
    fn schema_version(&self) -> GenomeSchemaVersion;

    /// Inspect a genome and return findings. Never mutates.
    fn validate(&self, genome: &Value) -> ValidationReport;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_starts_clean() {
        let r = ValidationReport::new(GenomeSchemaVersion(3));
        assert!(r.is_clean());
        assert!(!r.has_errors());
        assert_eq!(r.schema_version, Some(GenomeSchemaVersion(3)));
    }

    #[test]
    fn add_error_breaks_clean_and_blocks() {
        let mut r = ValidationReport::new(GenomeSchemaVersion(3));
        r.add_error("missing field");
        assert!(r.has_errors());
        assert!(!r.is_clean());
        assert_eq!(r.errors, vec!["missing field".to_string()]);
    }

    #[test]
    fn warning_is_advisory_only() {
        let mut r = ValidationReport::new(GenomeSchemaVersion(3));
        r.add_warning("nudge");
        assert!(!r.has_errors());
        assert!(!r.is_clean());
        assert_eq!(r.warnings, vec!["nudge".to_string()]);
    }
}
