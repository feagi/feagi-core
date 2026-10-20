// Copyright 2026 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! v5 -> v6 migrator.
//!
//! Schema v6 writes brain-region membership only as `cortical_areas`.
//! Older documents use `areas`, and some carry both keys. Both names are
//! one membership list; serde rejects the pair. This step folds `areas`
//! into `cortical_areas` and deletes `areas`.

use serde_json::Value;

use crate::genome::migration::membership::fold_genome_region_membership;
use crate::genome::migration::{MigrationError, MigrationStepDiagnostics, Migrator};
use crate::genome::schema::GenomeSchemaVersion;

/// Advances a hierarchical genome from schema v5 to schema v6.
#[derive(Debug, Default, Clone, Copy)]
pub struct V5ToV6Migrator;

impl V5ToV6Migrator {
    pub const fn new() -> Self {
        Self
    }
}

impl Migrator for V5ToV6Migrator {
    fn from_version(&self) -> GenomeSchemaVersion {
        GenomeSchemaVersion(5)
    }

    fn to_version(&self) -> GenomeSchemaVersion {
        GenomeSchemaVersion(6)
    }

    fn name(&self) -> &'static str {
        "v5_to_v6"
    }

    fn migrate(&self, genome: &mut Value) -> Result<MigrationStepDiagnostics, MigrationError> {
        let mut diag = MigrationStepDiagnostics::new(self.from_version(), self.to_version());
        let outcome = fold_genome_region_membership(genome);
        if outcome.folded == 0 {
            diag.record("brain region membership already uses cortical_areas");
        } else {
            diag.record(format!(
                "folded legacy areas into cortical_areas on {} brain regions",
                outcome.folded
            ));
        }
        for note in outcome.data_loss {
            diag.record(note);
        }
        Ok(diag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn folds_legacy_areas_and_drops_the_duplicate_key() {
        let mut genome = json!({
            "genome_schema_version": 5,
            "brain_regions": {
                "root": {
                    "areas": ["_power", "_death"],
                    "cortical_areas": ["_death"]
                },
                "legacy_only": {
                    "areas": ["_power"]
                }
            }
        });
        let diag = V5ToV6Migrator::new().migrate(&mut genome).unwrap();
        assert_eq!(diag.transformations.len(), 1);
        assert!(genome["brain_regions"]["root"].get("areas").is_none());
        assert_eq!(
            genome["brain_regions"]["root"]["cortical_areas"],
            json!(["_death", "_power"])
        );
        assert!(genome["brain_regions"]["legacy_only"]
            .get("areas")
            .is_none());
        assert_eq!(
            genome["brain_regions"]["legacy_only"]["cortical_areas"],
            json!(["_power"])
        );

        let again = V5ToV6Migrator::new().migrate(&mut genome).unwrap();
        assert!(again.transformations[0].contains("already uses cortical_areas"));
        assert_eq!(
            genome["brain_regions"]["root"]["cortical_areas"],
            json!(["_death", "_power"])
        );
    }

    #[test]
    fn records_a_diagnostic_when_a_non_array_membership_value_is_dropped() {
        let mut genome = json!({
            "genome_schema_version": 5,
            "brain_regions": {
                "root": { "areas": "_power", "cortical_areas": ["_death"] }
            }
        });
        let diag = V5ToV6Migrator::new().migrate(&mut genome).unwrap();
        assert_eq!(diag.transformations.len(), 2);
        assert!(diag.transformations[1].contains("root"));
        assert!(diag.transformations[1].contains("not an array"));
        assert_eq!(
            genome["brain_regions"]["root"]["cortical_areas"],
            json!(["_death"])
        );
    }
}
