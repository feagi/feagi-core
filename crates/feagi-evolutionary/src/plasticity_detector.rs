//! Plasticity detection for genome analysis
//!
//! This module provides utilities to detect whether a genome contains
//! plasticity features (neuroplasticity via memory areas or synaptic plasticity via STDP).

use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, error};

/// Check if a genome JSON contains any form of plasticity
///
/// This function checks for:
/// 1. Memory cortical areas (identified by `memory-b` flag = true)
/// 2. STDP connections (identified by `plasticity_flag` = true in morphologies)
///
/// # Arguments
/// * `genome_json` - The genome JSON Value
///
/// # Returns
/// * `true` if plasticity is detected, `false` otherwise
///
pub fn genome_has_plasticity(genome_json: &Value) -> bool {
    let has_memory = has_memory_areas(genome_json);
    let has_stdp = has_stdp_connections(genome_json);

    debug!(
        target: "feagi-evolutionary",
        "Plasticity detection: memory_areas={}, stdp_connections={}",
        has_memory, has_stdp
    );

    has_memory || has_stdp
}

/// Check if genome has memory cortical areas
///
/// Memory areas are identified by the `memory-b` property set to `true` in the blueprint.
/// The `_group` field may still be "CUSTOM", so we rely on the `memory-b` flag.
///
fn has_memory_areas(genome_json: &Value) -> bool {
    if let Some(blueprint) = genome_json.get("blueprint").and_then(|b| b.as_object()) {
        for (key, value) in blueprint {
            // Check for memory-b flag
            if key.ends_with("-cx-memory-b") {
                if let Some(is_memory) = value.as_bool() {
                    if is_memory {
                        debug!(
                            target: "feagi-evolutionary",
                            "Found memory area via key: {}", key
                        );
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Check if genome has STDP connections (plastic synapses)
///
/// STDP connections are identified by `plasticity_flag: true` in the destination map morphologies.
///
fn has_stdp_connections(genome_json: &Value) -> bool {
    if let Some(blueprint) = genome_json.get("blueprint").and_then(|b| b.as_object()) {
        for (key, value) in blueprint {
            // Check for destination mapping (dstmap)
            if key.ends_with("-cx-dstmap-d") {
                if let Some(dstmap) = value.as_object() {
                    for (dst_area_id, morphology_list) in dstmap {
                        if let Some(morphologies) = morphology_list.as_array() {
                            for morph in morphologies {
                                if let Some(plasticity_flag) = morph.get("plasticity_flag") {
                                    if plasticity_flag.as_bool() == Some(true) {
                                        debug!(
                                            target: "feagi-evolutionary",
                                            "Found STDP connection: key={}, dst={}", key, dst_area_id
                                        );
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

/// Genome key selecting change-based MP encoding for a memory area.
pub const MP_CHANGE_MODE_KEY: &str = "mp_change_mode";
/// Genome key for the differential rounding step, in membrane potential units.
pub const MP_DELTA_QUANTIZATION_KEY: &str = "mp_delta_quantization";
/// Genome key for the ratio rounding step, in percent per compounding bucket.
pub const MP_RATIO_QUANTIZATION_KEY: &str = "mp_ratio_quantization";
/// Genome key for replay-oriented MP learning.
pub const MP_LEARNING_ENABLED_KEY: &str = "mp_learning_enabled";

/// Change-based membrane potential encoding for a memory area.
///
/// When not `None`, pattern identity comes from how upstream MPs change between
/// consecutive frames of the temporal window instead of which neurons fired.
/// Mutually exclusive with `mp_learning_enabled`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MpChangeMode {
    /// Pattern identity from fired neuron sets (standard episodic memory).
    #[default]
    None,
    /// Identity from quantized MP differences; a neuron missing from a frame counts as MP 0.
    Differential,
    /// Identity from compounding percentage changes between positive MPs only.
    Ratio,
}

impl MpChangeMode {
    /// Canonical genome string for this mode.
    pub const fn as_str(self) -> &'static str {
        match self {
            MpChangeMode::None => "none",
            MpChangeMode::Differential => "mp_differential",
            MpChangeMode::Ratio => "mp_ratio",
        }
    }

    /// Parse a canonical genome string.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "none" => Some(MpChangeMode::None),
            "mp_differential" => Some(MpChangeMode::Differential),
            "mp_ratio" => Some(MpChangeMode::Ratio),
            _ => None,
        }
    }
}

/// Validate the MP encoding properties of a memory area.
///
/// Checks the change mode value, that both quantization levels are finite and
/// positive when present, and that MP learning and a change mode are not both on.
/// Absent keys are valid and resolve to defaults.
pub fn validate_memory_mp_properties(properties: &HashMap<String, Value>) -> Result<(), String> {
    let mode = match properties.get(MP_CHANGE_MODE_KEY) {
        None | Some(Value::Null) => MpChangeMode::None,
        Some(Value::String(s)) => MpChangeMode::parse(s).ok_or_else(|| {
            format!(
                "{MP_CHANGE_MODE_KEY} '{s}' is invalid; expected one of 'none', 'mp_differential', 'mp_ratio'"
            )
        })?,
        Some(other) => {
            return Err(format!(
                "{MP_CHANGE_MODE_KEY} must be a string, got {other}"
            ))
        }
    };

    for key in [MP_DELTA_QUANTIZATION_KEY, MP_RATIO_QUANTIZATION_KEY] {
        match properties.get(key) {
            None | Some(Value::Null) => {}
            Some(value) => match value.as_f64() {
                Some(q) if q.is_finite() && q > 0.0 => {}
                _ => {
                    return Err(format!(
                        "{key} must be a number greater than 0, got {value}"
                    ))
                }
            },
        }
    }

    let mp_learning = match properties.get(MP_LEARNING_ENABLED_KEY) {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(other) => {
            return Err(format!(
                "{MP_LEARNING_ENABLED_KEY} must be a boolean, got {other}"
            ))
        }
    };
    if mp_learning && mode != MpChangeMode::None {
        return Err(format!(
            "{MP_LEARNING_ENABLED_KEY} and {MP_CHANGE_MODE_KEY}='{}' are mutually exclusive",
            mode.as_str()
        ));
    }
    Ok(())
}

/// Memory-specific cortical area properties
#[derive(Debug, Clone)]
pub struct MemoryAreaProperties {
    /// Number of timesteps to consider for temporal pattern detection
    pub temporal_depth: u32,
    /// Threshold for long-term memory formation (number of activations)
    pub longterm_threshold: u32,
    /// Rate at which neuron lifespan grows with reactivations
    pub lifespan_growth_rate: f32,
    /// Initial lifespan for newly created memory neurons
    pub init_lifespan: u32,
    /// When true, membrane potentials are captured and stored with replay frames
    pub mp_learning_enabled: bool,
    /// Change-based MP encoding; exclusive with `mp_learning_enabled`.
    pub mp_change_mode: MpChangeMode,
    /// Differential rounding step in membrane potential units.
    pub mp_delta_quantization: f32,
    /// Ratio rounding step in percent per compounding bucket.
    pub mp_ratio_quantization: f32,
    /// Minimum fired voxels inside a scan window for that window to be considered.
    pub min_window_activity: u32,
    /// Skip the entire field scan when current-burst density exceeds this fraction (0.0-1.0).
    pub scan_skip_density: f32,
}

impl Default for MemoryAreaProperties {
    fn default() -> Self {
        Self {
            // Enforce minimum temporal depth of 1; 0 is not a valid configuration because
            // the pattern detector needs at least one timestep of history.
            temporal_depth: 1,
            longterm_threshold: 100,
            lifespan_growth_rate: 1.0,
            init_lifespan: 9,
            mp_learning_enabled: false,
            mp_change_mode: MpChangeMode::None,
            mp_delta_quantization: 1.0,
            mp_ratio_quantization: 20.0,
            min_window_activity: 1,
            scan_skip_density: 1.0,
        }
    }
}

/// Extract memory-specific properties from a cortical area's properties HashMap
///
/// Returns `Some(MemoryAreaProperties)` if the area is a memory area (`is_mem_type` = true),
/// otherwise returns `None`.
///
/// # Arguments
/// * `properties` - The cortical area properties HashMap
///
pub fn extract_memory_properties(
    properties: &HashMap<String, Value>,
) -> Option<MemoryAreaProperties> {
    let is_memory = properties
        .get("is_mem_type")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !is_memory {
        return None;
    }

    let defaults = MemoryAreaProperties::default();
    let mp_change_mode = match properties.get(MP_CHANGE_MODE_KEY).and_then(|v| v.as_str()) {
        None => defaults.mp_change_mode,
        Some(s) => MpChangeMode::parse(s).unwrap_or_else(|| {
            // @architecture:acceptable - emergency fallback; validate_memory_mp_properties
            // rejects unknown modes at genome load and API update.
            error!(
                target: "feagi-evolutionary",
                "Unknown {} '{}'; change encoding disabled for this memory area",
                MP_CHANGE_MODE_KEY, s
            );
            MpChangeMode::None
        }),
    };

    Some(MemoryAreaProperties {
        temporal_depth: properties
            .get("temporal_depth")
            .and_then(|v| v.as_u64())
            .unwrap_or(1)
            .max(1) as u32,
        longterm_threshold: properties
            .get("longterm_mem_threshold")
            .and_then(|v| v.as_u64())
            .unwrap_or(100) as u32,
        lifespan_growth_rate: properties
            .get("lifespan_growth_rate")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32,
        init_lifespan: properties
            .get("init_lifespan")
            .and_then(|v| v.as_u64())
            .unwrap_or(9) as u32,
        mp_learning_enabled: properties
            .get(MP_LEARNING_ENABLED_KEY)
            .and_then(|v| v.as_bool())
            .unwrap_or(defaults.mp_learning_enabled),
        mp_change_mode,
        mp_delta_quantization: properties
            .get(MP_DELTA_QUANTIZATION_KEY)
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(defaults.mp_delta_quantization),
        mp_ratio_quantization: properties
            .get(MP_RATIO_QUANTIZATION_KEY)
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(defaults.mp_ratio_quantization),
        min_window_activity: properties
            .get("min_window_activity")
            .and_then(|v| v.as_u64())
            .unwrap_or(1)
            .max(1) as u32,
        scan_skip_density: properties
            .get("scan_skip_density")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0)
            .clamp(0.0, 1.0) as f32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_no_plasticity() {
        let genome = json!({
            "blueprint": {
                "_____10c-Y2FfX19fX50=-cx-_group-t": "CUSTOM",
                "_____10c-Y2FfX19fX50=-cx-memory-b": false,
                "_____10c-Y2FfX19fX50=-cx-dstmap-d": {
                    "b2ltZwkAAAA=": [{
                        "morphology_id": "projector",
                        "plasticity_flag": false
                    }]
                }
            }
        });

        assert!(!genome_has_plasticity(&genome));
    }

    #[test]
    fn test_has_memory_areas() {
        let genome = json!({
            "blueprint": {
                "_____10c-Y21fX19fXxg=-cx-_group-t": "CUSTOM",
                "_____10c-Y21fX19fXxg=-cx-memory-b": true,
                "_____10c-Y21fX19fXxg=-cx-mem__t-i": 100
            }
        });

        assert!(genome_has_plasticity(&genome));
        assert!(has_memory_areas(&genome));
        assert!(!has_stdp_connections(&genome));
    }

    #[test]
    fn test_has_stdp_connections() {
        let genome = json!({
            "blueprint": {
                "_____10c-Y2FfX19fX50=-cx-_group-t": "CUSTOM",
                "_____10c-Y2FfX19fX50=-cx-memory-b": false,
                "_____10c-Y2FfX19fX50=-cx-dstmap-d": {
                    "b2ltZwkAAAA=": [{
                        "morphology_id": "projector",
                        "plasticity_flag": true,
                        "postSynapticCurrent_multiplier": 1
                    }]
                }
            }
        });

        assert!(genome_has_plasticity(&genome));
        assert!(!has_memory_areas(&genome));
        assert!(has_stdp_connections(&genome));
    }

    #[test]
    fn test_has_both_plasticity_types() {
        let genome = json!({
            "blueprint": {
                "_____10c-Y21fX19fXxg=-cx-memory-b": true,
                "_____10c-Y2FfX19fX50=-cx-dstmap-d": {
                    "b2ltZwkAAAA=": [{
                        "morphology_id": "projector",
                        "plasticity_flag": true
                    }]
                }
            }
        });

        assert!(genome_has_plasticity(&genome));
        assert!(has_memory_areas(&genome));
        assert!(has_stdp_connections(&genome));
    }

    #[test]
    fn test_extract_memory_properties() {
        let mut properties = HashMap::new();
        properties.insert("is_mem_type".to_string(), json!(true));
        properties.insert("temporal_depth".to_string(), json!(5));
        properties.insert("longterm_mem_threshold".to_string(), json!(200));
        properties.insert("lifespan_growth_rate".to_string(), json!(1.5));
        properties.insert("init_lifespan".to_string(), json!(15));

        let mem_props = extract_memory_properties(&properties).expect("Should extract properties");
        assert_eq!(mem_props.temporal_depth, 5);
        assert_eq!(mem_props.longterm_threshold, 200);
        assert_eq!(mem_props.lifespan_growth_rate, 1.5);
        assert_eq!(mem_props.init_lifespan, 15);
    }

    #[test]
    fn test_extract_memory_properties_defaults() {
        let mut properties = HashMap::new();
        properties.insert("is_mem_type".to_string(), json!(true));

        let mem_props = extract_memory_properties(&properties).expect("Should extract properties");
        assert_eq!(mem_props.temporal_depth, 1);
        assert_eq!(mem_props.longterm_threshold, 100);
        assert_eq!(mem_props.lifespan_growth_rate, 1.0);
        assert_eq!(mem_props.init_lifespan, 9);
    }

    fn memory_props(entries: &[(&str, Value)]) -> HashMap<String, Value> {
        let mut properties = HashMap::new();
        properties.insert("is_mem_type".to_string(), json!(true));
        for (k, v) in entries {
            properties.insert(k.to_string(), v.clone());
        }
        properties
    }

    #[test]
    fn test_mp_change_defaults() {
        let mem_props = extract_memory_properties(&memory_props(&[])).unwrap();
        assert_eq!(mem_props.mp_change_mode, MpChangeMode::None);
        assert_eq!(mem_props.mp_delta_quantization, 1.0);
        assert_eq!(mem_props.mp_ratio_quantization, 20.0);
        assert!(validate_memory_mp_properties(&memory_props(&[])).is_ok());
    }

    #[test]
    fn test_mp_change_mode_round_trip() {
        for mode in [
            MpChangeMode::None,
            MpChangeMode::Differential,
            MpChangeMode::Ratio,
        ] {
            assert_eq!(MpChangeMode::parse(mode.as_str()), Some(mode));
        }
        assert_eq!(MpChangeMode::parse("MP_RATIO"), None);
        assert_eq!(MpChangeMode::parse(""), None);
    }

    #[test]
    fn test_extract_mp_change_properties() {
        let props = memory_props(&[
            (MP_CHANGE_MODE_KEY, json!("mp_ratio")),
            (MP_DELTA_QUANTIZATION_KEY, json!(0.5)),
            (MP_RATIO_QUANTIZATION_KEY, json!(10)),
        ]);
        let mem_props = extract_memory_properties(&props).unwrap();
        assert_eq!(mem_props.mp_change_mode, MpChangeMode::Ratio);
        assert_eq!(mem_props.mp_delta_quantization, 0.5);
        assert_eq!(mem_props.mp_ratio_quantization, 10.0);
    }

    #[test]
    fn test_validate_rejects_unknown_mode() {
        let props = memory_props(&[(MP_CHANGE_MODE_KEY, json!("mp_percent"))]);
        assert!(validate_memory_mp_properties(&props)
            .unwrap_err()
            .contains("mp_percent"));
        let props = memory_props(&[(MP_CHANGE_MODE_KEY, json!(1))]);
        assert!(validate_memory_mp_properties(&props).is_err());
    }

    #[test]
    fn test_validate_rejects_non_positive_quantization() {
        for key in [MP_DELTA_QUANTIZATION_KEY, MP_RATIO_QUANTIZATION_KEY] {
            for bad in [json!(0), json!(-1.0), json!("1")] {
                let props = memory_props(&[(key, bad.clone())]);
                assert!(
                    validate_memory_mp_properties(&props).is_err(),
                    "{key}={bad} should be rejected"
                );
            }
        }
    }

    #[test]
    fn test_validate_rejects_mp_learning_with_change_mode() {
        let props = memory_props(&[
            (MP_LEARNING_ENABLED_KEY, json!(true)),
            (MP_CHANGE_MODE_KEY, json!("mp_differential")),
        ]);
        assert!(validate_memory_mp_properties(&props)
            .unwrap_err()
            .contains("mutually exclusive"));

        let props = memory_props(&[
            (MP_LEARNING_ENABLED_KEY, json!(true)),
            (MP_CHANGE_MODE_KEY, json!("none")),
        ]);
        assert!(validate_memory_mp_properties(&props).is_ok());
    }

    #[test]
    fn test_extract_memory_properties_non_memory() {
        let mut properties = HashMap::new();
        properties.insert("is_mem_type".to_string(), json!(false));

        assert!(extract_memory_properties(&properties).is_none());
    }
}
