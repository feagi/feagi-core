// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Chain runner: walks a genome from its detected schema version to a
//! target version, invoking the registered migrators in order.
//!
//! See `feagi-core/docs/GENOME_SCHEMA_VERSIONING.md` for the runner
//! contract: between hops it runs the per-version validator as advisory;
//! at the final hop it runs the target validator as blocking.

use serde_json::{json, Value};

use super::{ChainRegistry, ChainResult, MigrationError};
use crate::genome::schema::{detect_schema_version, GenomeSchemaVersion};

/// Walks a JSON genome through the chain of registered migrators.
pub struct ChainRunner<'a> {
    registry: &'a ChainRegistry,
}

impl<'a> ChainRunner<'a> {
    pub fn new(registry: &'a ChainRegistry) -> Self {
        Self { registry }
    }

    /// Run the chain, migrating `genome` from its detected version up to
    /// `target`. The genome is mutated in place. The returned `ChainResult`
    /// captures every hop and the final blocking validator's verdict.
    ///
    /// Errors:
    /// - `DetectionFailed` if `detect_schema_version` rejects the input.
    /// - `DowngradeRefused` if the genome is already past `target`.
    /// - `MissingMigrator` if the chain has a gap somewhere in the range.
    /// - `StepFailed` if any migrator returns an error.
    pub fn run_to(
        &self,
        genome: &mut Value,
        target: GenomeSchemaVersion,
    ) -> Result<ChainResult, MigrationError> {
        let from = detect_schema_version(genome)
            .map_err(|e| MigrationError::DetectionFailed(e.to_string()))?;

        if from > target {
            return Err(MigrationError::DowngradeRefused { from, target });
        }

        let mut migrators_applied: Vec<&'static str> = Vec::new();
        let mut per_step_diagnostics = Vec::new();
        let mut advisory_warnings: Vec<String> = Vec::new();

        let mut current = from;
        while current < target {
            let migrator =
                self.registry
                    .migrator_for(current)
                    .ok_or(MigrationError::MissingMigrator {
                        from: current,
                        target,
                    })?;

            let next = migrator.to_version();
            debug_assert_eq!(
                next.as_u32(),
                current.as_u32() + 1,
                "registry should have rejected non-contiguous migrators at registration"
            );

            let diagnostics = migrator.migrate(genome)?;
            migrators_applied.push(migrator.name());

            // The runner is the single source of truth for the schema
            // version on the wire. Migrators should not touch this field;
            // we stamp it after the step so if a migrator forgets, we
            // still produce a self-consistent output genome.
            stamp_schema_version(genome, next);
            per_step_diagnostics.push(diagnostics);

            // Advisory validation between hops. Errors are demoted to
            // warnings here; only the final-target validator is blocking.
            let intermediate = self.registry.run_validator(next, genome);
            for w in intermediate.warnings {
                advisory_warnings.push(format!("v{}: {}", next.as_u32(), w));
            }
            for e in intermediate.errors {
                advisory_warnings.push(format!("v{} (advisory): {}", next.as_u32(), e));
            }

            current = next;
        }

        // Final blocking validation at the target version.
        let final_report = self.registry.run_validator(target, genome);
        let blocking_errors = final_report.errors;
        for w in final_report.warnings {
            advisory_warnings.push(format!("v{}: {}", target.as_u32(), w));
        }

        Ok(ChainResult {
            from_version: from,
            to_version: target,
            migrators_applied,
            per_step_diagnostics,
            advisory_warnings,
            blocking_errors,
        })
    }
}

/// Write the integer `genome_schema_version` field on the genome.
fn stamp_schema_version(genome: &mut Value, version: GenomeSchemaVersion) {
    if let Some(obj) = genome.as_object_mut() {
        obj.insert("genome_schema_version".to_string(), json!(version.as_u32()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genome::migration::test_support::{make_failing, make_ok};
    use crate::genome::schema::CURRENT_SCHEMA_VERSION;
    use serde_json::json;

    fn registry_with_chain(from: u32, to_inclusive: u32) -> ChainRegistry {
        let mut reg = ChainRegistry::new();
        for v in from..to_inclusive {
            let name = match v {
                2 => "v2_to_v3_test",
                100 => "v100_to_v101_test",
                101 => "v101_to_v102_test",
                _ => "synthetic",
            };
            reg.register_migrator(make_ok(v, name)).unwrap();
        }
        reg
    }

    #[test]
    fn no_op_when_already_at_target() {
        let reg = ChainRegistry::new();
        let runner = ChainRunner::new(&reg);
        let mut genome = json!({ "genome_schema_version": 3 });

        let result = runner.run_to(&mut genome, CURRENT_SCHEMA_VERSION).unwrap();

        assert_eq!(result.from_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(result.to_version, CURRENT_SCHEMA_VERSION);
        assert!(result.migrators_applied.is_empty());
        assert!(result.per_step_diagnostics.is_empty());
        assert!(result.is_blocking_clean());
    }

    #[test]
    fn single_hop_runs_one_migrator_and_stamps_version() {
        let reg = registry_with_chain(2, 3);
        let runner = ChainRunner::new(&reg);
        let mut genome = json!({ "version": "2.0" });

        let result = runner.run_to(&mut genome, GenomeSchemaVersion(3)).unwrap();

        assert_eq!(result.from_version, GenomeSchemaVersion(2));
        assert_eq!(result.to_version, GenomeSchemaVersion(3));
        assert_eq!(result.migrators_applied, vec!["v2_to_v3_test"]);
        assert_eq!(result.per_step_diagnostics.len(), 1);
        assert_eq!(genome["genome_schema_version"], json!(3));
        assert_eq!(genome["step_count"], json!(1));
    }

    #[test]
    fn multi_hop_runs_each_migrator_in_order() {
        let reg = registry_with_chain(100, 103);
        let runner = ChainRunner::new(&reg);
        let mut genome = json!({ "genome_schema_version": 100 });

        let result = runner
            .run_to(&mut genome, GenomeSchemaVersion(103))
            .unwrap();

        assert_eq!(result.from_version, GenomeSchemaVersion(100));
        assert_eq!(result.to_version, GenomeSchemaVersion(103));
        assert_eq!(
            result.migrators_applied,
            vec!["v100_to_v101_test", "v101_to_v102_test", "synthetic"]
        );
        assert_eq!(result.per_step_diagnostics.len(), 3);
        assert_eq!(genome["genome_schema_version"], json!(103));
        assert_eq!(genome["step_count"], json!(3));
    }

    #[test]
    fn missing_migrator_in_range_errors() {
        let mut reg = ChainRegistry::new();
        // Register only the second hop, not the first.
        reg.register_migrator(make_ok(101, "v101_to_v102_test"))
            .unwrap();
        let runner = ChainRunner::new(&reg);
        let mut genome = json!({ "genome_schema_version": 100 });

        let err = runner
            .run_to(&mut genome, GenomeSchemaVersion(102))
            .unwrap_err();
        assert!(matches!(
            err,
            MigrationError::MissingMigrator { from, target }
                if from == GenomeSchemaVersion(100) && target == GenomeSchemaVersion(102)
        ));
    }

    #[test]
    fn step_failure_aborts_chain() {
        let mut reg = ChainRegistry::new();
        reg.register_migrator(make_ok(100, "v100_to_v101_test"))
            .unwrap();
        reg.register_migrator(make_failing(101, "v101_to_v102_test"))
            .unwrap();
        let runner = ChainRunner::new(&reg);
        let mut genome = json!({ "genome_schema_version": 100 });

        let err = runner
            .run_to(&mut genome, GenomeSchemaVersion(102))
            .unwrap_err();
        assert!(matches!(err, MigrationError::StepFailed { .. }));

        // The first hop did succeed and stamped its version, even though
        // the chain ultimately failed. This is intentional: failure leaves
        // partially-migrated state visible for diagnostics.
        assert_eq!(genome["genome_schema_version"], json!(101));
    }

    #[test]
    fn downgrade_is_refused() {
        let reg = ChainRegistry::new();
        let runner = ChainRunner::new(&reg);
        let mut genome = json!({ "genome_schema_version": 5 });
        let err = runner
            .run_to(&mut genome, GenomeSchemaVersion(3))
            .unwrap_err();
        assert!(matches!(
            err,
            MigrationError::DowngradeRefused { from, target }
                if from == GenomeSchemaVersion(5) && target == GenomeSchemaVersion(3)
        ));
    }

    #[test]
    fn detection_failure_propagates() {
        let reg = ChainRegistry::new();
        let runner = ChainRunner::new(&reg);
        let mut genome = json!({});
        let err = runner
            .run_to(&mut genome, GenomeSchemaVersion(3))
            .unwrap_err();
        assert!(matches!(err, MigrationError::DetectionFailed(_)));
    }
}
