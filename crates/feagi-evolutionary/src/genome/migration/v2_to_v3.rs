// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! v2 -> v3 migrator.
//!
//! Wraps the existing `crate::genome::migrator::migrate_genome` body
//! verbatim. That function performs the cortical_area-ID renames that
//! distinguish v2 from v3 (legacy IDs like `iic000`, `_power`, `omot00`
//! become template-compliant IDs like `svi0____`, `___power`, `mot0____`).
//!
//! Step 4 of the schema-versioning rollout repackages the existing
//! monolithic migrator as a single chain step. The function itself is
//! retained as a public API for external consumers (Python bindings,
//! `tools/migrate_genome.rs`, integration tests). Only the loader now
//! reaches it through this wrapper.

use serde_json::Value;

use crate::genome::migration::{MigrationError, MigrationStepDiagnostics, Migrator};
use crate::genome::migrator::migrate_genome;
use crate::genome::schema::GenomeSchemaVersion;

/// Single-step migrator that advances a genome from schema v2 to schema v3.
#[derive(Debug, Default, Clone, Copy)]
pub struct V2ToV3Migrator;

impl V2ToV3Migrator {
    pub const fn new() -> Self {
        Self
    }
}

impl Migrator for V2ToV3Migrator {
    fn from_version(&self) -> GenomeSchemaVersion {
        GenomeSchemaVersion(2)
    }

    fn to_version(&self) -> GenomeSchemaVersion {
        GenomeSchemaVersion(3)
    }

    fn name(&self) -> &'static str {
        "v2_to_v3"
    }

    fn migrate(&self, genome: &mut Value) -> Result<MigrationStepDiagnostics, MigrationError> {
        // Delegate to the existing implementation, which operates on a
        // borrowed Value and returns a fully-migrated owned Value plus
        // statistics. The chain runner contract requires in-place
        // mutation, so we replace `*genome` with the result.
        let result = migrate_genome(genome).map_err(|e| MigrationError::StepFailed {
            name: "v2_to_v3",
            from: GenomeSchemaVersion(2),
            to: GenomeSchemaVersion(3),
            reason: e.to_string(),
        })?;

        let mut diag = MigrationStepDiagnostics::new(GenomeSchemaVersion(2), GenomeSchemaVersion(3));

        if result.cortical_ids_migrated > 0 {
            diag.record(format!(
                "renamed {} cortical_area IDs to v3 template-compliant form",
                result.cortical_ids_migrated
            ));
            // Record up to a small number of example renames so diagnostics
            // are useful but bounded. The full mapping is available on
            // `MigrationResult` for callers that need it.
            for (old, new) in result.id_mapping.iter().take(5) {
                diag.record(format!("'{old}' -> '{new}'"));
            }
        }

        for w in &result.warnings {
            diag.record(format!("warning: {w}"));
        }

        *genome = result.genome;
        Ok(diag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn declares_v2_to_v3_endpoints() {
        let m = V2ToV3Migrator::new();
        assert_eq!(m.from_version(), GenomeSchemaVersion(2));
        assert_eq!(m.to_version(), GenomeSchemaVersion(3));
        assert_eq!(m.name(), "v2_to_v3");
    }

    #[test]
    fn no_op_on_already_compliant_genome() {
        // A genome with no legacy IDs should pass through with zero
        // recorded transformations beyond the empty diagnostics envelope.
        let m = V2ToV3Migrator::new();
        let mut g = json!({
            "blueprint": {},
            "brain_regions": {},
            "neuron_morphologies": {}
        });
        let snapshot = g.clone();
        let diag = m.migrate(&mut g).unwrap();
        assert_eq!(diag.from_version, GenomeSchemaVersion(2));
        assert_eq!(diag.to_version, GenomeSchemaVersion(3));
        assert!(diag.transformations.is_empty());
        // The migrator is allowed to canonicalize internal structures it
        // walks; assert the public-facing sections are byte-equal.
        assert_eq!(g["blueprint"], snapshot["blueprint"]);
        assert_eq!(g["brain_regions"], snapshot["brain_regions"]);
    }

    #[test]
    fn renames_legacy_cortical_ids() {
        // `_power` is one of the legacy IDs the underlying migrator
        // handles; the new ID is the canonical base64 form produced by
        // `CoreCorticalType::Power.to_cortical_id().as_base_64()`. We
        // don't assert the exact value here (already covered by
        // `migrator::tests::test_map_old_id_to_new`); the V2ToV3Migrator
        // contract is "delegate without losing diagnostics".
        let m = V2ToV3Migrator::new();
        let mut g = json!({
            "blueprint": {
                "_power": {
                    "cortical_name": "Power",
                    "block_boundaries": [1, 1, 1],
                    "relative_coordinate": [0, 0, 0],
                    "cortical_type": "INTERCONNECT"
                }
            },
            "brain_regions": {},
            "neuron_morphologies": {}
        });

        let diag = m.migrate(&mut g).unwrap();

        let blueprint = g["blueprint"].as_object().unwrap();
        assert!(
            !blueprint.contains_key("_power"),
            "legacy ID '_power' should have been renamed; blueprint keys: {:?}",
            blueprint.keys().collect::<Vec<_>>()
        );
        assert_eq!(
            blueprint.len(),
            1,
            "rename must preserve area count; blueprint keys: {:?}",
            blueprint.keys().collect::<Vec<_>>()
        );
        assert!(
            diag.transformations.iter().any(|t| t.contains("renamed")),
            "diagnostics should record at least one rename; got: {:?}",
            diag.transformations
        );
    }
}
