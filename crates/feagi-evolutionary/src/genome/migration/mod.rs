// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Stepwise genome migration: traits, registry, and chain runner.
//!
//! A `Migrator` performs a single `vN -> vN+1` transformation on a JSON
//! genome. Migrators are registered in a `ChainRegistry` keyed by their
//! `from_version`. The `ChainRunner` walks a genome from its detected
//! schema version up to a target version, invoking each migrator in
//! sequence and per-version validators between hops.
//!
//! See `feagi-core/docs/GENOME_SCHEMA_VERSIONING.md` for the system-level
//! design, and `crates/feagi-evolutionary/src/genome/README.md` for the
//! contributor contract (invariants, retention, anti-patterns).

use std::collections::BTreeMap;

use serde_json::Value;
use thiserror::Error;

use crate::genome::schema::GenomeSchemaVersion;
use crate::genome::validators::{ValidationReport, Validator};

pub mod chain;

pub use chain::ChainRunner;

/// Errors emitted by migrators and by the chain machinery itself.
#[derive(Debug, Error)]
pub enum MigrationError {
    /// A migrator returned an error during its `migrate` call.
    #[error("Migrator '{name}' ({from} -> {to}) failed: {reason}")]
    StepFailed {
        name: &'static str,
        from: GenomeSchemaVersion,
        to: GenomeSchemaVersion,
        reason: String,
    },

    /// The runner needed a migrator for a version but the registry didn't
    /// have one. Indicates a gap in the chain.
    #[error("No migrator registered with from_version={from} (needed to reach v{target})")]
    MissingMigrator {
        from: GenomeSchemaVersion,
        target: GenomeSchemaVersion,
    },

    /// A migrator's declared `to_version` is not exactly `from_version + 1`,
    /// or two migrators share a `from_version`. The runner refuses to start
    /// in either case; see the README's "registry MUST be contiguous" rule.
    #[error("Registry violates the contiguity invariant: {0}")]
    InvalidRegistry(String),

    /// The genome's detected schema version is newer than the requested
    /// target. Forward-only migrations are by design.
    #[error("Cannot migrate downward: genome is at v{from} but target is v{target}")]
    DowngradeRefused {
        from: GenomeSchemaVersion,
        target: GenomeSchemaVersion,
    },

    /// `detect_schema_version` could not resolve the input genome.
    #[error("Failed to detect genome schema version: {0}")]
    DetectionFailed(String),
}

/// Diagnostic record produced by a single migrator step.
///
/// Per the contributor contract, every transformation a migrator performs
/// MUST contribute at least one entry to `transformations`. A migrator that
/// runs and produces zero diagnostics is a bug.
#[derive(Debug, Clone)]
pub struct MigrationStepDiagnostics {
    pub from_version: GenomeSchemaVersion,
    pub to_version: GenomeSchemaVersion,
    pub transformations: Vec<String>,
}

impl MigrationStepDiagnostics {
    pub fn new(from: GenomeSchemaVersion, to: GenomeSchemaVersion) -> Self {
        Self {
            from_version: from,
            to_version: to,
            transformations: Vec::new(),
        }
    }

    pub fn record(&mut self, msg: impl Into<String>) {
        self.transformations.push(msg.into());
    }
}

/// Single `vN -> vN+1` migration step.
///
/// Implementations operate on `serde_json::Value` and MUST satisfy the
/// invariants documented in the module-level README:
/// determinism, idempotence, bounded compute, no side channels, JSON only,
/// diagnostics over silence.
///
/// Migrators MUST NOT mutate the `genome_schema_version` field on the
/// input `Value`. The chain runner stamps the new version after each
/// successful step. This keeps the migrator focused on shape changes and
/// lets the runner be the single source of truth for version bookkeeping.
#[allow(clippy::wrong_self_convention)]
// The `from_*`/`to_*` accessor names describe the migrator's *schema
// version range*, not constructors. Renaming would hurt readability for
// every caller (`source_version`/`target_version` were considered and
// rejected in design review).
pub trait Migrator: Send + Sync {
    /// Schema version this migrator accepts as input.
    fn from_version(&self) -> GenomeSchemaVersion;

    /// Schema version this migrator produces. MUST equal
    /// `from_version() + 1`; the registry rejects anything else.
    fn to_version(&self) -> GenomeSchemaVersion;

    /// Stable identifier for diagnostics and logs. Should not change
    /// across releases for a given step.
    fn name(&self) -> &'static str;

    /// Perform the transformation in place. Return diagnostics describing
    /// what changed, or an error.
    fn migrate(&self, genome: &mut Value) -> Result<MigrationStepDiagnostics, MigrationError>;
}

/// Aggregate result returned by a successful chain run.
///
/// Contains everything `validate-and-repair` needs to surface to clients
/// per decision #8 in the design doc: the version range traversed, which
/// named migrators ran, per-step diagnostics, advisory warnings collected
/// between hops, and any blocking errors raised by the final validator.
#[derive(Debug, Clone)]
pub struct ChainResult {
    pub from_version: GenomeSchemaVersion,
    pub to_version: GenomeSchemaVersion,
    pub migrators_applied: Vec<&'static str>,
    pub per_step_diagnostics: Vec<MigrationStepDiagnostics>,
    pub advisory_warnings: Vec<String>,
    pub blocking_errors: Vec<String>,
}

impl ChainResult {
    /// True when the final-version validator reported zero errors.
    pub fn is_blocking_clean(&self) -> bool {
        self.blocking_errors.is_empty()
    }
}

/// Holds the registered migrators and validators that the chain runner
/// will dispatch through.
///
/// Migrators are keyed by `from_version` (one per integer; duplicates are
/// rejected at registration). Validators are keyed by `schema_version`.
/// Contiguity (no gaps in the migrator chain) is checked at runner-start
/// time over the actual range being traversed, not at registration time.
pub struct ChainRegistry {
    migrators: BTreeMap<u32, Box<dyn Migrator>>,
    validators: BTreeMap<u32, Box<dyn Validator>>,
}

impl ChainRegistry {
    pub fn new() -> Self {
        Self {
            migrators: BTreeMap::new(),
            validators: BTreeMap::new(),
        }
    }

    /// Register a migrator. Rejects duplicates and migrators whose
    /// `to_version` is not exactly `from_version + 1`.
    pub fn register_migrator(&mut self, migrator: Box<dyn Migrator>) -> Result<(), MigrationError> {
        let from = migrator.from_version();
        let to = migrator.to_version();
        if to.as_u32() != from.as_u32().saturating_add(1) {
            return Err(MigrationError::InvalidRegistry(format!(
                "migrator '{}' declares from={} to={}, expected to=from+1",
                migrator.name(),
                from,
                to
            )));
        }
        if self.migrators.contains_key(&from.as_u32()) {
            return Err(MigrationError::InvalidRegistry(format!(
                "duplicate migrator with from_version={}",
                from
            )));
        }
        self.migrators.insert(from.as_u32(), migrator);
        Ok(())
    }

    /// Register a validator. Replaces any existing validator at the same
    /// schema version (validators are policy-bearing; the latest registered
    /// one wins).
    pub fn register_validator(&mut self, validator: Box<dyn Validator>) {
        let v = validator.schema_version().as_u32();
        self.validators.insert(v, validator);
    }

    /// Look up the migrator that consumes genomes at `from`.
    pub fn migrator_for(&self, from: GenomeSchemaVersion) -> Option<&dyn Migrator> {
        self.migrators.get(&from.as_u32()).map(|b| b.as_ref())
    }

    /// Look up the validator at `version`.
    pub fn validator_for(&self, version: GenomeSchemaVersion) -> Option<&dyn Validator> {
        self.validators.get(&version.as_u32()).map(|b| b.as_ref())
    }

    /// Run the validator at `version` if one is registered, otherwise
    /// return an empty advisory report stamped with that version. Used by
    /// the chain runner so callers always see a consistent shape.
    pub fn run_validator(&self, version: GenomeSchemaVersion, genome: &Value) -> ValidationReport {
        match self.validator_for(version) {
            Some(v) => v.validate(genome),
            None => ValidationReport::new(version),
        }
    }

    /// Number of migrators currently registered.
    pub fn migrator_count(&self) -> usize {
        self.migrators.len()
    }

    /// Number of validators currently registered.
    pub fn validator_count(&self) -> usize {
        self.validators.len()
    }
}

impl Default for ChainRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Test-only helpers shared with `chain.rs`.
///
/// Lives in its own non-`tests` module so that `pub(super)` re-exports
/// don't trip `clippy::items_after_test_module`.
#[cfg(test)]
pub(super) mod test_support {
    use super::*;
    use serde_json::json;

    /// Synthetic migrator that bumps a `step_count` field, used to
    /// exercise the runner mechanics without depending on real domain
    /// transforms.
    pub struct SyntheticMigrator {
        from: GenomeSchemaVersion,
        to: GenomeSchemaVersion,
        name: &'static str,
        fail: bool,
    }

    impl SyntheticMigrator {
        pub fn ok(from: u32, name: &'static str) -> Box<Self> {
            Box::new(Self {
                from: GenomeSchemaVersion(from),
                to: GenomeSchemaVersion(from + 1),
                name,
                fail: false,
            })
        }

        pub fn failing(from: u32, name: &'static str) -> Box<Self> {
            Box::new(Self {
                from: GenomeSchemaVersion(from),
                to: GenomeSchemaVersion(from + 1),
                name,
                fail: true,
            })
        }
    }

    impl Migrator for SyntheticMigrator {
        fn from_version(&self) -> GenomeSchemaVersion {
            self.from
        }

        fn to_version(&self) -> GenomeSchemaVersion {
            self.to
        }

        fn name(&self) -> &'static str {
            self.name
        }

        fn migrate(&self, genome: &mut Value) -> Result<MigrationStepDiagnostics, MigrationError> {
            if self.fail {
                return Err(MigrationError::StepFailed {
                    name: self.name,
                    from: self.from,
                    to: self.to,
                    reason: "synthetic failure".to_string(),
                });
            }
            let mut diag = MigrationStepDiagnostics::new(self.from, self.to);
            let count = genome
                .get("step_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                + 1;
            genome
                .as_object_mut()
                .expect("test genome must be a JSON object")
                .insert("step_count".to_string(), json!(count));
            diag.record(format!("incremented step_count to {count}"));
            Ok(diag)
        }
    }

    pub fn make_ok(from: u32, name: &'static str) -> Box<dyn Migrator> {
        SyntheticMigrator::ok(from, name)
    }

    pub fn make_failing(from: u32, name: &'static str) -> Box<dyn Migrator> {
        SyntheticMigrator::failing(from, name)
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::SyntheticMigrator;
    use super::*;
    use serde_json::json;

    #[test]
    fn registry_accepts_a_well_formed_migrator() {
        let mut reg = ChainRegistry::new();
        reg.register_migrator(SyntheticMigrator::ok(2, "v2_to_v3"))
            .unwrap();
        assert_eq!(reg.migrator_count(), 1);
        assert!(reg.migrator_for(GenomeSchemaVersion(2)).is_some());
        assert!(reg.migrator_for(GenomeSchemaVersion(3)).is_none());
    }

    #[test]
    fn registry_rejects_to_version_not_equal_to_from_plus_one() {
        struct Skipping;
        impl Migrator for Skipping {
            fn from_version(&self) -> GenomeSchemaVersion {
                GenomeSchemaVersion(2)
            }
            fn to_version(&self) -> GenomeSchemaVersion {
                GenomeSchemaVersion(4)
            }
            fn name(&self) -> &'static str {
                "skip"
            }
            fn migrate(
                &self,
                _genome: &mut Value,
            ) -> Result<MigrationStepDiagnostics, MigrationError> {
                unreachable!()
            }
        }
        let mut reg = ChainRegistry::new();
        let err = reg.register_migrator(Box::new(Skipping)).unwrap_err();
        assert!(matches!(err, MigrationError::InvalidRegistry(_)));
    }

    #[test]
    fn registry_rejects_duplicate_from_version() {
        let mut reg = ChainRegistry::new();
        reg.register_migrator(SyntheticMigrator::ok(2, "first"))
            .unwrap();
        let err = reg
            .register_migrator(SyntheticMigrator::ok(2, "second"))
            .unwrap_err();
        assert!(matches!(err, MigrationError::InvalidRegistry(_)));
    }

    #[test]
    fn migration_step_diagnostics_records_transformations() {
        let mut diag =
            MigrationStepDiagnostics::new(GenomeSchemaVersion(2), GenomeSchemaVersion(3));
        diag.record("converted blueprint keys");
        diag.record("renamed legacy fields");
        assert_eq!(diag.transformations.len(), 2);
        assert_eq!(diag.from_version, GenomeSchemaVersion(2));
        assert_eq!(diag.to_version, GenomeSchemaVersion(3));
    }

    #[test]
    fn run_validator_returns_empty_when_unregistered() {
        let reg = ChainRegistry::new();
        let report = reg.run_validator(GenomeSchemaVersion(3), &json!({}));
        assert_eq!(report.schema_version, Some(GenomeSchemaVersion(3)));
        assert!(report.is_clean());
    }
}
