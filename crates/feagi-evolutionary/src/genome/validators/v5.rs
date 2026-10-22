// Copyright 2026 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Blocking validator for schema v5.
//!
//! Keeps the v4 modulator and spike-train checks, and requires each classifier
//! to use the v5 field names.

use serde_json::Value;

use super::{ValidationReport, Validator};
use crate::genome::schema::GenomeSchemaVersion;
use crate::modulators::parse_modulator_registry;
use feagi_structures::genomic::validate_spike_train;

/// Validator for schema v5.
#[derive(Debug, Default, Clone, Copy)]
pub struct V5Validator;

impl V5Validator {
    pub const fn new() -> Self {
        Self
    }
}

impl Validator for V5Validator {
    fn schema_version(&self) -> GenomeSchemaVersion {
        GenomeSchemaVersion(5)
    }

    fn validate(&self, genome: &Value) -> ValidationReport {
        let mut report = ValidationReport::new(GenomeSchemaVersion(5));
        match parse_modulator_registry(genome.get("modulators")) {
            Ok(_) => {}
            Err(err) => report.errors.push(err.to_string()),
        }
        if genome.get("modulators").is_none() {
            report
                .errors
                .push("v5 genomes require a modulators object".to_string());
        }
        if let Some(blueprint) = genome.get("blueprint").and_then(|v| v.as_object()) {
            for (id, area) in blueprint {
                let enabled = area
                    .get("spike_train")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let limit = area
                    .get("consecutive_fire_cnt_max")
                    .or_else(|| area.get("consecutive_fire_limit"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let limit = u16::try_from(limit).unwrap_or(0);
                if let Err(err) = validate_spike_train(enabled, limit) {
                    report.errors.push(format!("area {id}: {err}"));
                }
            }
        }
        if let Some(classifiers) = genome.get("classifiers").and_then(|v| v.as_object()) {
            let mut ids: Vec<&String> = classifiers.keys().collect();
            ids.sort();
            for id in ids {
                let Some(classifier) = classifiers.get(id).and_then(|v| v.as_object()) else {
                    report
                        .errors
                        .push(format!("classifier {id} must be an object"));
                    continue;
                };
                for field in [
                    "name",
                    "parent_region_id",
                    "sample_memory_id",
                    "class_memory_id",
                    "error_area_id",
                ] {
                    let present = classifier
                        .get(field)
                        .and_then(|v| v.as_str())
                        .is_some_and(|value| !value.trim().is_empty());
                    if !present {
                        report
                            .errors
                            .push(format!("classifier {id} is missing {field}"));
                    }
                }
                if classifier.contains_key("kernel_memory_id") {
                    report.errors.push(format!(
                        "classifier {id} still carries kernel_memory_id after v5 migration"
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
    fn v5_classifier_requires_sample_memory_id() {
        let report = V5Validator::new().validate(&json!({
            "modulators": {},
            "classifiers": {
                "clf-1": { "name": "MNIST", "parent_region_id": "root" }
            }
        }));
        assert!(report
            .errors
            .iter()
            .any(|err| err.contains("sample_memory_id")));
    }
}
