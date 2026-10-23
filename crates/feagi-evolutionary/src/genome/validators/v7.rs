// Copyright 2026 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Blocking validator for schema v7.
//!
//! v7 gates a mapping with a `synaptic.gate` modulator listed in the rule's
//! `modulators`. The per-mapping `gate_source_area` field is gone.

use serde_json::Value;

use super::{ValidationReport, Validator};
use crate::genome::schema::GenomeSchemaVersion;

/// Validator for schema v7.
#[derive(Debug, Default, Clone, Copy)]
pub struct V7Validator;

impl V7Validator {
    pub const fn new() -> Self {
        Self
    }
}

impl Validator for V7Validator {
    fn schema_version(&self) -> GenomeSchemaVersion {
        GenomeSchemaVersion(7)
    }

    fn validate(&self, genome: &Value) -> ValidationReport {
        let mut report = ValidationReport::new(GenomeSchemaVersion(7));
        let Some(blueprint) = genome.get("blueprint").and_then(|v| v.as_object()) else {
            return report;
        };
        let mut area_ids: Vec<&String> = blueprint.keys().collect();
        area_ids.sort();
        for area_id in area_ids {
            let Some(dst) = blueprint[area_id]
                .get("cortical_mapping_dst")
                .and_then(|v| v.as_object())
            else {
                continue;
            };
            let mut dst_ids: Vec<&String> = dst.keys().collect();
            dst_ids.sort();
            for dst_id in dst_ids {
                let Some(rules) = dst[dst_id].as_array() else {
                    continue;
                };
                if rules.iter().any(|r| r.get("gate_source_area").is_some()) {
                    report.errors.push(format!(
                        "mapping {area_id} -> {dst_id} still has gate_source_area; v7 uses a synaptic.gate modulator listed in modulators"
                    ));
                }
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
    fn rejects_a_rule_that_still_has_gate_source_area() {
        let report = V7Validator::new().validate(&json!({
            "blueprint": {
                "src": { "cortical_mapping_dst": {
                    "dst": [{ "morphology_id": "projector", "gate_source_area": "gate" }]
                }}
            }
        }));
        assert_eq!(report.errors.len(), 1);
        assert!(report.errors[0].contains("gate_source_area"));
    }

    #[test]
    fn accepts_modulator_gated_rules_and_a_missing_blueprint() {
        let clean = V7Validator::new().validate(&json!({
            "blueprint": {
                "src": { "cortical_mapping_dst": {
                    "dst": [{ "morphology_id": "projector", "modulators": ["gate_x"] }]
                }}
            }
        }));
        assert!(clean.errors.is_empty());
        assert!(V7Validator::new().validate(&json!({})).errors.is_empty());
    }
}
