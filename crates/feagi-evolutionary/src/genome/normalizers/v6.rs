// Copyright 2026 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! V6 normalizer: brain-region membership uses `cortical_areas` only.
//!
//! A document already stamped v6 can still carry the legacy `areas` key.
//! The chain runs this normalizer before the blocking v6 validator, so that
//! document is converted and then accepted. The v5-to-v6 migrator performs
//! the same fold when the document is older.

use serde_json::Value;

use super::{NormalizationDiagnostics, Normalizer};
use crate::genome::migration::MigrationError;
use crate::genome::parser::fold_genome_region_membership;
use crate::genome::schema::GenomeSchemaVersion;

/// Normalizer for schema v6.
#[derive(Debug, Default, Clone, Copy)]
pub struct V6Normalizer;

impl V6Normalizer {
    pub const fn new() -> Self {
        Self
    }
}

impl Normalizer for V6Normalizer {
    fn schema_version(&self) -> GenomeSchemaVersion {
        GenomeSchemaVersion(6)
    }

    fn name(&self) -> &'static str {
        "v6_normalizer"
    }

    fn normalize(&self, genome: &mut Value) -> Result<NormalizationDiagnostics, MigrationError> {
        let mut diag = NormalizationDiagnostics::new(GenomeSchemaVersion(6));
        let folded = fold_genome_region_membership(genome);
        if folded > 0 {
            diag.record(format!(
                "folded legacy areas into cortical_areas on {folded} brain regions"
            ));
        }
        Ok(diag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn folds_legacy_areas_on_an_already_current_genome() {
        let mut genome = json!({
            "genome_schema_version": 6,
            "brain_regions": {
                "root": {
                    "areas": ["_power"],
                    "cortical_areas": ["_death"]
                },
                "untouched": {
                    "cortical_areas": ["_power"]
                },
                "not_an_object": "skip"
            }
        });
        let diag = V6Normalizer::new().normalize(&mut genome).unwrap();
        assert_eq!(diag.transformations.len(), 1);
        assert!(genome["brain_regions"]["root"].get("areas").is_none());
        assert_eq!(
            genome["brain_regions"]["root"]["cortical_areas"],
            json!(["_death", "_power"])
        );
        assert!(genome["brain_regions"]["untouched"].get("areas").is_none());
        assert_eq!(genome["brain_regions"]["not_an_object"], "skip");

        let again = V6Normalizer::new().normalize(&mut genome).unwrap();
        assert!(again.is_clean());
        assert_eq!(
            genome["brain_regions"]["root"]["cortical_areas"],
            json!(["_death", "_power"])
        );
    }
}
