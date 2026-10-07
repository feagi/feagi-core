// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Blocking validator for schema v4.
//!
//! Checks the modulator section and spike-train constraints. Structural checks
//! that already run on `RuntimeGenome` stay in `crate::validator`.

use serde_json::Value;

use super::{ValidationReport, Validator};
use crate::genome::schema::GenomeSchemaVersion;
use crate::modulators::parse_modulator_registry;
use feagi_structures::genomic::validate_spike_train;

/// Validator for schema v4.
#[derive(Debug, Default, Clone, Copy)]
pub struct V4Validator;

impl V4Validator {
    pub const fn new() -> Self {
        Self
    }
}

impl Validator for V4Validator {
    fn schema_version(&self) -> GenomeSchemaVersion {
        GenomeSchemaVersion(4)
    }

    fn validate(&self, genome: &Value) -> ValidationReport {
        let mut report = ValidationReport::new(GenomeSchemaVersion(4));
        match parse_modulator_registry(genome.get("modulators")) {
            Ok(_) => {}
            Err(err) => report.errors.push(err.to_string()),
        }
        if genome.get("modulators").is_none() {
            report
                .errors
                .push("v4 genomes require a modulators object".to_string());
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
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn empty_modulators_object_is_valid() {
        let report = V4Validator::new().validate(&json!({ "modulators": {} }));
        assert!(report.errors.is_empty());
    }

    #[test]
    fn missing_modulators_section_is_rejected() {
        let report = V4Validator::new().validate(&json!({}));
        assert!(!report.errors.is_empty());
    }
}
