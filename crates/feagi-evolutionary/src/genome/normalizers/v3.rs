// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! V3 normalizer: cleans well-known bad values in a v3 genome before
//! validation.
//!
//! Mirrors the semantics of the legacy `crate::validator::auto_fix_genome`
//! function, which still operates on `RuntimeGenome` and is retained for
//! external consumers (Python bindings, CLI tools). The two paths are
//! intentionally redundant during the rollout window: the chain runner
//! uses this JSON-level normalizer; external callers that already use
//! `auto_fix_genome` continue to work unchanged.
//!
//! Field paths are the **hierarchical** v3 JSON shape (post-flat→hierarchical
//! conversion, post-`migrate_genome`):
//!
//! - `physiology.simulation_timestep` (f64): if present and ≤ 0, replace
//!   with the default. The legacy alias `physiology.burst_delay` is read
//!   only by the runtime parser; this normalizer does not synthesize one
//!   field from the other.
//! - `physiology.max_age` (u64): if present and == 0, replace with default.
//! - `physiology.quantization_precision` (string): if empty/non-canonical/
//!   invalid, normalize or replace with default.
//! - `blueprint[id].block_boundaries` (`[u32; 3]`): any zero element is
//!   replaced with 1.
//! - `blueprint[id].per_voxel_neuron_cnt` (u64): if present and == 0,
//!   replaced with 1.
//!
//! Missing fields are not added here; `parse_physiology` and the cortical_area
//! area parser already fill defaults during JSON→`RuntimeGenome`. Adding
//! defaults here would change the on-wire genome shape, which is out of
//! scope for a normalizer.

use serde_json::{json, Value};

use super::{NormalizationDiagnostics, Normalizer};
use crate::genome::migration::MigrationError;
use crate::genome::schema::{GenomeSchemaVersion, CURRENT_SCHEMA_VERSION};

/// Default quantization precision used when the field is empty or invalid.
/// Mirrors `crate::runtime::default_quantization_precision()` to avoid a
/// runtime crate dependency cycle in module init.
const DEFAULT_QUANTIZATION_PRECISION: &str = "fp32";

/// Default simulation timestep used when the field is present but ≤ 0.
/// Mirrors `crate::runtime::PhysiologyConfig::default().simulation_timestep`.
const DEFAULT_SIMULATION_TIMESTEP: f64 = 0.025;

/// Default max age used when the field is present but == 0.
/// Mirrors `crate::runtime::PhysiologyConfig::default().max_age`.
const DEFAULT_MAX_AGE: u64 = 10_000_000;

#[derive(Debug, Default, Clone, Copy)]
pub struct V3Normalizer;

impl V3Normalizer {
    pub const fn new() -> Self {
        Self
    }
}

impl Normalizer for V3Normalizer {
    fn schema_version(&self) -> GenomeSchemaVersion {
        CURRENT_SCHEMA_VERSION
    }

    fn name(&self) -> &'static str {
        "v3_normalizer"
    }

    fn normalize(&self, genome: &mut Value) -> Result<NormalizationDiagnostics, MigrationError> {
        let mut diag = NormalizationDiagnostics::new(CURRENT_SCHEMA_VERSION);

        normalize_physiology(genome, &mut diag);
        normalize_blueprint(genome, &mut diag);

        Ok(diag)
    }
}

/// Apply corrections to the `physiology` section if present.
///
/// Silent when `physiology` is absent: the parser will fill defaults.
fn normalize_physiology(genome: &mut Value, diag: &mut NormalizationDiagnostics) {
    let physiology = match genome.get_mut("physiology").and_then(Value::as_object_mut) {
        Some(p) => p,
        None => return,
    };

    if let Some(ts) = physiology.get("simulation_timestep").and_then(Value::as_f64) {
        if ts <= 0.0 {
            physiology.insert("simulation_timestep".to_string(), json!(DEFAULT_SIMULATION_TIMESTEP));
            diag.record(format!("physiology.simulation_timestep {ts} -> {DEFAULT_SIMULATION_TIMESTEP} (default)"));
        }
    }

    if let Some(age) = physiology.get("max_age").and_then(Value::as_u64) {
        if age == 0 {
            physiology.insert("max_age".to_string(), json!(DEFAULT_MAX_AGE));
            diag.record(format!("physiology.max_age 0 -> {DEFAULT_MAX_AGE} (default)"));
        }
    }

    let precision_action = match physiology.get("quantization_precision").and_then(Value::as_str) {
        Some("") => Some(PrecisionAction::ReplaceWithDefault { previous: String::new() }),
        Some(other) => match canonicalize_precision(other) {
            Some(canonical) if canonical != other => Some(PrecisionAction::Normalize {
                previous: other.to_string(),
                canonical,
            }),
            Some(_) => None,
            None => Some(PrecisionAction::ReplaceWithDefault { previous: other.to_string() }),
        },
        None => None,
    };

    if let Some(action) = precision_action {
        match action {
            PrecisionAction::Normalize { previous, canonical } => {
                physiology.insert("quantization_precision".to_string(), Value::String(canonical.clone()));
                diag.record(format!("physiology.quantization_precision '{previous}' -> '{canonical}' (normalized)"));
            }
            PrecisionAction::ReplaceWithDefault { previous } => {
                physiology.insert(
                    "quantization_precision".to_string(),
                    Value::String(DEFAULT_QUANTIZATION_PRECISION.to_string()),
                );
                diag.record(format!(
                    "physiology.quantization_precision '{previous}' -> '{DEFAULT_QUANTIZATION_PRECISION}' (default)"
                ));
            }
        }
    }
}

enum PrecisionAction {
    Normalize { previous: String, canonical: String },
    ReplaceWithDefault { previous: String },
}

/// Returns the canonical lowercase form of a known precision token, or
/// `None` if the input is unrecognized.
///
/// The set of recognized tokens mirrors `feagi_npu_neural::types::Precision`
/// without taking a dependency on it, since this normalizer should not
/// pull in NPU types just to canonicalize a string.
fn canonicalize_precision(input: &str) -> Option<String> {
    match input.to_lowercase().as_str() {
        "fp32" | "f32" => Some("fp32".to_string()),
        "fp16" | "f16" => Some("fp16".to_string()),
        "int8" => Some("int8".to_string()),
        _ => None,
    }
}

/// Apply per-cortical_area-area corrections to the `blueprint` section if
/// present.
fn normalize_blueprint(genome: &mut Value, diag: &mut NormalizationDiagnostics) {
    let blueprint = match genome.get_mut("blueprint").and_then(Value::as_object_mut) {
        Some(b) => b,
        None => return,
    };

    let area_ids: Vec<String> = blueprint.keys().cloned().collect();
    for cortical_id in area_ids {
        let area = match blueprint.get_mut(&cortical_id).and_then(Value::as_object_mut) {
            Some(a) => a,
            None => continue,
        };

        normalize_block_boundaries(area, &cortical_id, diag);
        normalize_per_voxel_neuron_cnt(area, &cortical_id, diag);
    }
}

fn normalize_block_boundaries(area: &mut serde_json::Map<String, Value>, cortical_id: &str, diag: &mut NormalizationDiagnostics) {
    let boundaries = match area.get_mut("block_boundaries").and_then(Value::as_array_mut) {
        Some(b) if b.len() == 3 => b,
        _ => return,
    };

    static AXIS_NAMES: [&str; 3] = ["width", "height", "depth"];
    for (i, slot) in boundaries.iter_mut().enumerate() {
        if slot.as_u64() == Some(0) {
            *slot = json!(1u32);
            diag.record(format!("blueprint['{cortical_id}'].block_boundaries[{i}] ({}) 0 -> 1", AXIS_NAMES[i]));
        }
    }
}

fn normalize_per_voxel_neuron_cnt(area: &mut serde_json::Map<String, Value>, cortical_id: &str, diag: &mut NormalizationDiagnostics) {
    if area.get("per_voxel_neuron_cnt").and_then(Value::as_u64) == Some(0) {
        area.insert("per_voxel_neuron_cnt".to_string(), json!(1u32));
        diag.record(format!("blueprint['{cortical_id}'].per_voxel_neuron_cnt 0 -> 1"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reports_current_schema_version() {
        let n = V3Normalizer::new();
        assert_eq!(n.schema_version(), CURRENT_SCHEMA_VERSION);
        assert_eq!(n.name(), "v3_normalizer");
    }

    #[test]
    fn clean_genome_yields_clean_diagnostics() {
        let n = V3Normalizer::new();
        let mut g = json!({
            "physiology": {
                "simulation_timestep": 0.025,
                "max_age": 10_000_000,
                "quantization_precision": "fp32"
            },
            "blueprint": {
                "abc12345": {
                    "block_boundaries": [10, 10, 10],
                    "per_voxel_neuron_cnt": 1
                }
            }
        });
        let d = n.normalize(&mut g).unwrap();
        assert!(d.is_clean());
    }

    #[test]
    fn fixes_negative_simulation_timestep() {
        let n = V3Normalizer::new();
        let mut g = json!({
            "physiology": { "simulation_timestep": -0.1 }
        });
        let d = n.normalize(&mut g).unwrap();
        assert_eq!(g["physiology"]["simulation_timestep"], json!(0.025));
        assert_eq!(d.transformations.len(), 1);
        assert!(d.transformations[0].contains("simulation_timestep"));
    }

    #[test]
    fn fixes_zero_simulation_timestep() {
        let n = V3Normalizer::new();
        let mut g = json!({ "physiology": { "simulation_timestep": 0.0 } });
        let d = n.normalize(&mut g).unwrap();
        assert_eq!(g["physiology"]["simulation_timestep"], json!(0.025));
        assert!(!d.is_clean());
    }

    #[test]
    fn leaves_burst_delay_alone() {
        // The legacy alias is parser territory; the normalizer must not
        // synthesize fields. parse_physiology will fold burst_delay into
        // simulation_timestep at deserialize time.
        let n = V3Normalizer::new();
        let mut g = json!({ "physiology": { "burst_delay": 0.030 } });
        let d = n.normalize(&mut g).unwrap();
        assert!(d.is_clean());
        assert_eq!(g["physiology"]["burst_delay"], json!(0.030));
        assert!(g["physiology"].get("simulation_timestep").is_none());
    }

    #[test]
    fn fixes_zero_max_age() {
        let n = V3Normalizer::new();
        let mut g = json!({ "physiology": { "max_age": 0 } });
        let d = n.normalize(&mut g).unwrap();
        assert_eq!(g["physiology"]["max_age"], json!(DEFAULT_MAX_AGE));
        assert!(!d.is_clean());
    }

    #[test]
    fn replaces_empty_precision_with_default() {
        let n = V3Normalizer::new();
        let mut g = json!({ "physiology": { "quantization_precision": "" } });
        let d = n.normalize(&mut g).unwrap();
        assert_eq!(g["physiology"]["quantization_precision"], json!("fp32"));
        assert_eq!(d.transformations.len(), 1);
    }

    #[test]
    fn normalizes_uppercase_precision() {
        let n = V3Normalizer::new();
        let mut g = json!({ "physiology": { "quantization_precision": "FP32" } });
        let d = n.normalize(&mut g).unwrap();
        assert_eq!(g["physiology"]["quantization_precision"], json!("fp32"));
        assert!(d.transformations[0].contains("normalized"));
    }

    #[test]
    fn normalizes_f32_alias_precision() {
        let n = V3Normalizer::new();
        let mut g = json!({ "physiology": { "quantization_precision": "f32" } });
        let d = n.normalize(&mut g).unwrap();
        assert_eq!(g["physiology"]["quantization_precision"], json!("fp32"));
        assert!(d.transformations[0].contains("normalized"));
    }

    #[test]
    fn replaces_invalid_precision_with_default() {
        let n = V3Normalizer::new();
        let mut g = json!({ "physiology": { "quantization_precision": "garbage" } });
        let d = n.normalize(&mut g).unwrap();
        assert_eq!(g["physiology"]["quantization_precision"], json!("fp32"));
        assert!(d.transformations[0].contains("default"));
    }

    #[test]
    fn fixes_zero_block_boundaries_per_axis() {
        let n = V3Normalizer::new();
        let mut g = json!({
            "blueprint": {
                "abc12345": { "block_boundaries": [0, 5, 0] }
            }
        });
        let d = n.normalize(&mut g).unwrap();
        assert_eq!(g["blueprint"]["abc12345"]["block_boundaries"], json!([1, 5, 1]));
        assert_eq!(d.transformations.len(), 2);
    }

    #[test]
    fn fixes_zero_per_voxel_neuron_cnt() {
        let n = V3Normalizer::new();
        let mut g = json!({
            "blueprint": {
                "abc12345": { "per_voxel_neuron_cnt": 0 }
            }
        });
        let d = n.normalize(&mut g).unwrap();
        assert_eq!(g["blueprint"]["abc12345"]["per_voxel_neuron_cnt"], json!(1));
        assert_eq!(d.transformations.len(), 1);
    }

    #[test]
    fn handles_missing_fields_silently() {
        // Missing fields are parser territory; the normalizer must not
        // synthesize them. This pins the contract.
        let n = V3Normalizer::new();
        let mut g = json!({});
        let d = n.normalize(&mut g).unwrap();
        assert!(d.is_clean());
        assert_eq!(g, json!({}));
    }

    #[test]
    fn is_idempotent() {
        // Running the normalizer twice must produce the same output and
        // an empty diagnostics on the second pass.
        let n = V3Normalizer::new();
        let mut g = json!({
            "physiology": {
                "simulation_timestep": 0.0,
                "max_age": 0,
                "quantization_precision": ""
            },
            "blueprint": {
                "abc12345": {
                    "block_boundaries": [0, 0, 0],
                    "per_voxel_neuron_cnt": 0
                }
            }
        });

        let d1 = n.normalize(&mut g).unwrap();
        assert!(!d1.is_clean());
        let snapshot = g.clone();

        let d2 = n.normalize(&mut g).unwrap();
        assert!(d2.is_clean());
        assert_eq!(g, snapshot);
    }
}
