// Copyright 2026 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Blocking validator for schema v7.
//!
//! v7 gates a mapping with a `synaptic.gate` modulator listed in the rule's
//! `modulators`. The per-mapping `gate_source_area` field is gone.
//!
//! The parsed-genome rules (areas, regions, morphologies, physiology, leak and
//! other ranges) have one home, [`crate::validator::validate_genome`]. This
//! validator runs it on the parsed document, so every entry point that
//! reaches v7 reports the same errors. It adds only the JSON-level
//! `gate_source_area` check, which has no parsed-genome form.

use serde_json::Value;

use super::{ValidationReport, Validator};
use crate::genome::converter::to_runtime_genome;
use crate::genome::schema::GenomeSchemaVersion;
use crate::genome::GenomeParser;
use crate::validator::validate_genome;

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
        validate_parsed_genome(genome, &mut report);
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

/// Parse the document and put the errors of `validate_genome` into `report`.
///
/// A document that cannot be parsed into a runtime genome reports that as one
/// blocking error: there is nothing further to validate.
fn validate_parsed_genome(genome: &Value, report: &mut ValidationReport) {
    let json_str = match serde_json::to_string(genome) {
        Ok(json_str) => json_str,
        Err(e) => {
            report.add_error(format!("genome cannot be serialized for validation: {e}"));
            return;
        }
    };
    let runtime =
        GenomeParser::parse(&json_str).and_then(|parsed| to_runtime_genome(parsed, &json_str));
    match runtime {
        Ok(runtime) => {
            for error in validate_genome(&runtime).errors {
                report.add_error(error);
            }
        }
        Err(e) => report.add_error(format!("genome cannot be parsed for validation: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Minimal parseable v7 document around `blueprint` and `brain_regions`.
    fn document(blueprint: Value, brain_regions: Value) -> Value {
        json!({
            "genome_schema_version": 7,
            "genome_id": "t",
            "genome_title": "t",
            "genome_description": "t",
            "version": "2.0",
            "blueprint": blueprint,
            "brain_regions": brain_regions,
            "neuron_morphologies": {},
            "physiology": { "simulation_timestep": 0.025, "max_age": 1 },
            "stats": { "innate_cortical_area_count": 0, "innate_neuron_count": 0, "innate_synapse_count": 0 },
            "signatures": { "genome": "0", "blueprint": "0", "physiology": "0" },
            "timestamp": 0.0
        })
    }

    #[test]
    fn rejects_a_rule_that_still_has_gate_source_area() {
        let report = V7Validator::new().validate(&document(
            json!({
                "src": { "cortical_mapping_dst": {
                    "dst": [{ "morphology_id": "projector", "gate_source_area": "gate" }]
                }}
            }),
            json!({}),
        ));
        let gate_errors: Vec<_> = report
            .errors
            .iter()
            .filter(|e| e.contains("gate_source_area"))
            .collect();
        assert_eq!(gate_errors.len(), 1);
    }

    #[test]
    fn accepts_modulator_gated_rules_and_a_missing_blueprint() {
        let clean = V7Validator::new().validate(&document(
            json!({
                "src": { "cortical_mapping_dst": {
                    "dst": [{ "morphology_id": "projector", "modulators": ["gate_x"] }]
                }}
            }),
            json!({}),
        ));
        assert!(clean.errors.is_empty(), "{:?}", clean.errors);
        let missing = V7Validator::new().validate(&json!({}));
        assert!(!missing
            .errors
            .iter()
            .any(|e| e.contains("gate_source_area")));
    }

    #[test]
    fn reports_the_parsed_genome_errors_of_validate_genome() {
        // A region naming an undefined area and an out-of-range leak.
        let raw = document(
            json!({ "_power": {
                "cortical_name": "Power",
                "block_boundaries": [1, 1, 1],
                "relative_coordinate": [0, 0, 0],
                "cortical_type": "CORE",
                "leak_coefficient": 150
            }}),
            json!({ "01a1a966-c1c3-7b53-8c89-4e91cabe3e92": { "title": "Root", "cortical_areas": ["_power", "cgone001"] } }),
        );
        let report = V7Validator::new().validate(&raw);

        let json_str = serde_json::to_string(&raw).unwrap();
        let parsed = GenomeParser::parse(&json_str).unwrap();
        let runtime = to_runtime_genome(parsed, &json_str).unwrap();
        let mut expected = validate_genome(&runtime).errors;
        expected.sort();
        let mut actual = report.errors.clone();
        actual.sort();

        assert_eq!(actual, expected, "one rule set behind every entry point");
        assert!(actual
            .iter()
            .any(|e| e.contains("non-existent cortical area")));
        assert!(actual.iter().any(|e| e.contains("leak_coefficient")));
    }

    #[test]
    fn an_unparseable_document_is_one_blocking_error() {
        let report = V7Validator::new().validate(&json!({ "blueprint": "not an object" }));
        assert!(report
            .errors
            .iter()
            .any(|e| e.contains("cannot be parsed for validation")));
    }
}
