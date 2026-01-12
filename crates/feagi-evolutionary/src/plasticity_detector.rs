//! Plasticity detection for genome analysis
//!
//! This module provides utilities to detect whether a genome contains
//! plasticity features (neuroplasticity via memory areas or synaptic plasticity via STDP).

use serde_json::Value;
use std::collections::HashMap;
use tracing::debug;

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

    #[test]
    fn test_extract_memory_properties_non_memory() {
        let mut properties = HashMap::new();
        properties.insert("is_mem_type".to_string(), json!(false));

        assert!(extract_memory_properties(&properties).is_none());
    }
}
