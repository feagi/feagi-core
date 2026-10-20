// Copyright 2026 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Blocking validator for schema v6.
//!
//! v6 brain regions store membership only in `cortical_areas`.

use serde_json::Value;

use super::{ValidationReport, Validator};
use crate::genome::schema::GenomeSchemaVersion;

/// Validator for schema v6.
#[derive(Debug, Default, Clone, Copy)]
pub struct V6Validator;

impl V6Validator {
    pub const fn new() -> Self {
        Self
    }
}

impl Validator for V6Validator {
    fn schema_version(&self) -> GenomeSchemaVersion {
        GenomeSchemaVersion(6)
    }

    fn validate(&self, genome: &Value) -> ValidationReport {
        let mut report = ValidationReport::new(GenomeSchemaVersion(6));
        let Some(regions) = genome
            .get("brain_regions")
            .and_then(|value| value.as_object())
        else {
            return report;
        };
        let mut region_ids: Vec<&String> = regions.keys().collect();
        region_ids.sort();
        for region_id in region_ids {
            let Some(region) = regions.get(region_id).and_then(|value| value.as_object()) else {
                continue;
            };
            if region.contains_key("areas") {
                report.errors.push(format!(
                    "brain region {region_id} still has legacy membership key areas; v6 uses cortical_areas"
                ));
            }
        }
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rejects_a_region_that_still_has_areas() {
        let report = V6Validator::new().validate(&json!({
            "brain_regions": {
                "root": { "cortical_areas": ["_death"] },
                "legacy": { "areas": ["_power"], "cortical_areas": ["_death"] },
                "note": "not a region"
            }
        }));
        assert_eq!(report.errors.len(), 1);
        assert!(report.errors[0].contains("legacy"));
        assert!(report.errors[0].contains("areas"));
    }

    #[test]
    fn accepts_cortical_areas_only_and_a_missing_region_map() {
        let clean = V6Validator::new().validate(&json!({
            "brain_regions": { "root": { "cortical_areas": ["_death"] } }
        }));
        assert!(clean.errors.is_empty());
        let missing = V6Validator::new().validate(&json!({}));
        assert!(missing.errors.is_empty());
    }
}
