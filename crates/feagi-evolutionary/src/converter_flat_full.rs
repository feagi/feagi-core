// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
**COMPLETE** flat genome format (2.0) to hierarchical format converter.

This is the full implementation with:
- ALL property mappings (40+ properties)
- Complete dstmap (cortical_mapping_dst) parsing
- 2D coordinates support
- All neural parameters
- Memory-specific properties

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::{EvoError, EvoResult};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use tracing::warn;

/// Complete genome_2_to_1 property mapping
const PROPERTY_MAPPINGS: &[(&str, &str)] = &[
    ("_n_cnt-i", "per_voxel_neuron_cnt"),
    ("gd_vis-b", "visualization"),
    ("__name-t", "cortical_name"),
    ("rcordx-i", "relative_coordinate"),
    ("rcordy-i", "relative_coordinate"),
    ("rcordz-i", "relative_coordinate"),
    ("2dcorx-i", "2d_coordinate"),
    ("2dcory-i", "2d_coordinate"),
    ("___bbx-i", "block_boundaries"),
    ("___bby-i", "block_boundaries"),
    ("___bbz-i", "block_boundaries"),
    ("__rand-b", "location_generation_type"),
    ("synatt-f", "synapse_attractivity"),
    ("pstcr_-f", "postsynaptic_current"),
    ("pstcrm-f", "postsynaptic_current_max"),
    ("fire_t-f", "firing_threshold"),
    ("ftincx-f", "firing_threshold_increment_x"),
    ("ftincy-f", "firing_threshold_increment_y"),
    ("ftincz-f", "firing_threshold_increment_z"),
    ("fthlim-f", "firing_threshold_limit"),
    ("refrac-i", "refractory_period"),
    ("leak_c-f", "leak_coefficient"),
    ("leak_v-f", "leak_variability"),
    ("c_fr_c-i", "consecutive_fire_cnt_max"),
    ("snooze-f", "snooze_length"),
    ("_group-t", "group_id"),
    ("subgrp-t", "sub_group_id"),
    // Also map _group to cortical_group for classification (needed by neuroembryogenesis)
    ("_group-t", "cortical_group"),
    ("dstmap-d", "cortical_mapping_dst"),
    ("de_gen-f", "degeneration"),
    ("pspuni-b", "psp_uniform_distribution"),
    ("mp_acc-b", "mp_charge_accumulation"),
    ("mp_psp-b", "mp_driven_psp"),
    ("memory-b", "is_mem_type"),
    ("mem__t-i", "longterm_mem_threshold"),
    ("mem_gr-i", "lifespan_growth_rate"),
    ("mem_ls-i", "init_lifespan"),
    ("tmpdpt-i", "temporal_depth"),
    ("excite-f", "neuron_excitability"),
    ("devcnt-i", "dev_count"),
];

/// Build property mapping lookup table
fn build_property_map() -> HashMap<String, String> {
    PROPERTY_MAPPINGS
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

/// Template for hierarchical genome area
fn create_area_template() -> serde_json::Map<String, Value> {
    let mut template = serde_json::Map::new();

    // Default values
    template.insert("cortical_name".to_string(), json!(""));
    template.insert("group_id".to_string(), json!("CUSTOM"));
    template.insert("block_boundaries".to_string(), json!([1, 1, 1]));
    template.insert("relative_coordinate".to_string(), json!([0, 0, 0]));
    template.insert("2d_coordinate".to_string(), json!([0, 0]));
    template.insert("cortical_mapping_dst".to_string(), json!({}));
    template.insert("location_generation_type".to_string(), json!("sequential"));
    template.insert("per_voxel_neuron_cnt".to_string(), json!(1));
    template.insert("visualization".to_string(), json!(true));

    // Neural parameters with defaults
    template.insert("firing_threshold".to_string(), json!(1.0));
    template.insert("refractory_period".to_string(), json!(0));
    template.insert("leak_coefficient".to_string(), json!(0.0));
    template.insert("neuron_excitability".to_string(), json!(1.0));
    template.insert("postsynaptic_current".to_string(), json!(1.0));
    template.insert("psp_uniform_distribution".to_string(), json!(false));

    template
}

/// Convert flat genome (2.0) to hierarchical format - COMPLETE implementation
pub fn convert_flat_to_hierarchical_full(flat_genome: &Value) -> EvoResult<Value> {
    let flat_blueprint = if let Some(bp) = flat_genome.get("blueprint") {
        bp.as_object().ok_or_else(|| {
            EvoError::InvalidGenome("Flat genome blueprint must be an object".to_string())
        })?
    } else {
        return Err(EvoError::InvalidGenome(
            "Flat genome missing blueprint section".to_string(),
        ));
    };

    // Build property mapping
    let property_map = build_property_map();

    // Extract cortical areas
    let cortical_areas = extract_cortical_areas(flat_blueprint)?;

    // Load visualization_voxel_granularity overrides (if present)
    let visualization_overrides: HashMap<String, Value> = if let Some(overrides_obj) = flat_genome.get("visualization_voxel_granularity_overrides") {
        if let Some(overrides_map) = overrides_obj.as_object() {
            overrides_map.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        } else {
            HashMap::new()
        }
    } else {
        HashMap::new()
    };

    // Build hierarchical blueprint
    let mut hierarchical_blueprint = serde_json::Map::new();

    for cortical_id in &cortical_areas {
        let mut area_data = create_area_template();

        // Process all flat keys for this cortical area
        process_area_properties(cortical_id, flat_blueprint, &property_map, &mut area_data)?;

        // Apply visualization_voxel_granularity override if present
        if let Some(override_value) = visualization_overrides.get(cortical_id) {
            if let Some(properties) = area_data.get_mut("properties") {
                if let Some(properties_obj) = properties.as_object_mut() {
                    properties_obj.insert(
                        "visualization_voxel_granularity".to_string(),
                        override_value.clone(),
                    );
                }
            }
        }

        hierarchical_blueprint.insert(cortical_id.clone(), Value::Object(area_data));
    }

    // Build complete hierarchical genome
    let mut hierarchical = serde_json::Map::new();
    hierarchical.insert(
        "blueprint".to_string(),
        Value::Object(hierarchical_blueprint),
    );

    // Copy other sections
    if let Some(morphologies) = flat_genome.get("neuron_morphologies") {
        hierarchical.insert("neuron_morphologies".to_string(), morphologies.clone());
    }

    if let Some(physiology) = flat_genome.get("physiology") {
        hierarchical.insert("physiology".to_string(), physiology.clone());
    } else {
        hierarchical.insert("physiology".to_string(), json!({}));
    }

    if let Some(stats) = flat_genome.get("stats") {
        hierarchical.insert("stats".to_string(), stats.clone());
    }

    if let Some(signatures) = flat_genome.get("signatures") {
        hierarchical.insert("signatures".to_string(), signatures.clone());
    }

    // Copy metadata
    for field in &["genome_id", "genome_title", "version", "timestamp"] {
        if let Some(value) = flat_genome.get(field) {
            hierarchical.insert(field.to_string(), value.clone());
        }
    }

    hierarchical.insert("brain_regions".to_string(), json!({}));

    Ok(Value::Object(hierarchical))
}

/// Extract cortical area IDs from flat keys
fn extract_cortical_areas(
    flat_blueprint: &serde_json::Map<String, Value>,
) -> EvoResult<HashSet<String>> {
    let mut areas = HashSet::new();

    for key in flat_blueprint.keys() {
        if let Some(cortical_id) = parse_cortical_id(key) {
            areas.insert(cortical_id);
        }
    }

    Ok(areas)
}

/// Parse cortical ID from flat key: "_____10c-AREA1-cx-property-type"
fn parse_cortical_id(key: &str) -> Option<String> {
    if !key.starts_with("_____10c-") {
        return None;
    }

    let parts: Vec<&str> = key.split('-').collect();
    if parts.len() >= 2 {
        Some(parts[1].to_string())
    } else {
        None
    }
}

/// Process all properties for a cortical area
fn process_area_properties(
    cortical_id: &str,
    flat_blueprint: &serde_json::Map<String, Value>,
    property_map: &HashMap<String, String>,
    area_data: &mut serde_json::Map<String, Value>,
) -> EvoResult<()> {
    for (flat_key, flat_value) in flat_blueprint.iter() {
        // Check if this key belongs to our cortical area
        if let Some(key_area_id) = parse_cortical_id(flat_key) {
            if key_area_id != cortical_id {
                continue;
            }

            // Extract property suffix (everything after cortical_id)
            let parts: Vec<&str> = flat_key.split('-').collect();
            if parts.len() < 3 {
                continue;
            }

            // Join parts after cortical_id: "cx-__name-t" or "nx-fire_t-f"
            let exon = parts[2..].join("-");
            let exon_without_prefix = if parts.len() > 3 {
                parts[3..].join("-")
            } else {
                exon.clone()
            };

            // Try both full and without-prefix lookup
            let lookup_key = if property_map.contains_key(&exon) {
                &exon
            } else if property_map.contains_key(&exon_without_prefix) {
                &exon_without_prefix
            } else {
                continue;
            };

            let hierarchical_prop = &property_map[lookup_key];

            // Handle special cases
            match hierarchical_prop.as_str() {
                "cortical_name" => {
                    area_data.insert(hierarchical_prop.clone(), flat_value.clone());
                }

                "location_generation_type" => {
                    let value = if flat_value.as_bool().unwrap_or(false) {
                        "random"
                    } else {
                        "sequential"
                    };
                    area_data.insert(hierarchical_prop.clone(), json!(value));
                }

                "cortical_mapping_dst" => {
                    process_dstmap(flat_value, area_data)?;
                }

                "block_boundaries" | "relative_coordinate" | "2d_coordinate" => {
                    process_coordinate_property(
                        flat_key,
                        flat_value,
                        hierarchical_prop,
                        area_data,
                    )?;
                }

                _ => {
                    // Regular property - direct copy
                    area_data.insert(hierarchical_prop.clone(), flat_value.clone());
                }
            }
        }
    }

    Ok(())
}

/// Process coordinate properties (block_boundaries, relative_coordinate, 2d_coordinate)
fn process_coordinate_property(
    flat_key: &str,
    flat_value: &Value,
    prop_name: &str,
    area_data: &mut serde_json::Map<String, Value>,
) -> EvoResult<()> {
    // Extract axis from key (last character before type specifier)
    let axis_char = flat_key.chars().rev().nth(2).unwrap_or('x');

    let index = match axis_char {
        'x' => 0,
        'y' => 1,
        'z' => 2,
        _ => return Ok(()),
    };

    // Ensure array exists
    if !area_data.contains_key(prop_name) {
        let default_array = if prop_name == "2d_coordinate" {
            json!([0, 0])
        } else {
            json!([0, 0, 0])
        };
        area_data.insert(prop_name.to_string(), default_array);
    }

    // Update the specific index
    if let Some(arr) = area_data.get_mut(prop_name).and_then(|v| v.as_array_mut()) {
        if index < arr.len() {
            arr[index] = flat_value.clone();
        }
    }

    // Also create dict format for coordinates
    if prop_name == "block_boundaries" {
        if !area_data.contains_key("cortical_dimensions") {
            area_data.insert("cortical_dimensions".to_string(), json!({}));
        }
        if let Some(dims) = area_data
            .get_mut("cortical_dimensions")
            .and_then(|v| v.as_object_mut())
        {
            let dim_name = match index {
                0 => "width",
                1 => "height",
                2 => "depth",
                _ => return Ok(()),
            };
            dims.insert(dim_name.to_string(), flat_value.clone());
        }
    } else if prop_name == "relative_coordinate" {
        if !area_data.contains_key("coordinates_3d") {
            area_data.insert("coordinates_3d".to_string(), json!({}));
        }
        if let Some(coords) = area_data
            .get_mut("coordinates_3d")
            .and_then(|v| v.as_object_mut())
        {
            let coord_name = match index {
                0 => "x",
                1 => "y",
                2 => "z",
                _ => return Ok(()),
            };
            coords.insert(coord_name.to_string(), flat_value.clone());
        }
    }

    Ok(())
}

/// Process destination mapping (dstmap) - COMPLETE implementation
fn process_dstmap(
    dstmap_value: &Value,
    area_data: &mut serde_json::Map<String, Value>,
) -> EvoResult<()> {
    let dstmap_obj = match dstmap_value.as_object() {
        Some(obj) => obj,
        None => return Ok(()), // Skip if not an object
    };

    let mut hierarchical_dstmap = serde_json::Map::new();

    for (destination_area, rules) in dstmap_obj {
        let rules_array = match rules.as_array() {
            Some(arr) => arr,
            None => continue,
        };

        let mut converted_rules = Vec::new();

        for rule in rules_array {
            // Support BOTH representations:
            // - Array format (legacy flat): ["projector", 1, 1.0, false, ...]
            // - Object format (already hierarchical-like): {"morphology_id": "...", ...}
            if let Some(rule_obj) = rule.as_object() {
                // Minimal validation to avoid silently accepting garbage.
                if !rule_obj.contains_key("morphology_id")
                    || !rule_obj.contains_key("postSynapticCurrent_multiplier")
                    || !rule_obj.contains_key("plasticity_flag")
                {
                    warn!(
                        target: "feagi-evo",
                        "Invalid dstmap rule object for destination {}: missing required keys",
                        destination_area
                    );
                    continue;
                }

                // Strict plasticity validation (no backward compatibility):
                // If plasticity_flag=true, the full plasticity parameter set must be present.
                if rule_obj.get("plasticity_flag").and_then(|v| v.as_bool()) == Some(true) {
                    let required = [
                        "plasticity_constant",
                        "ltp_multiplier",
                        "ltd_multiplier",
                        "plasticity_window",
                    ];
                    let missing: Vec<&str> = required
                        .iter()
                        .copied()
                        .filter(|k| !rule_obj.contains_key(*k))
                        .collect();
                    if !missing.is_empty() {
                        warn!(
                            target: "feagi-evo",
                            "Invalid plastic dstmap rule object for destination {}: missing keys {:?}",
                            destination_area,
                            missing
                        );
                        continue;
                    }
                }

                converted_rules.push(Value::Object(rule_obj.clone()));
                continue;
            }

            let rule_array = match rule.as_array() {
                Some(arr) => arr,
                None => continue,
            };

            // Validate minimum required elements
            if rule_array.len() < 4 {
                warn!(
                    target: "feagi-evo",
                    "Invalid mapping recipe format (need at least 4 elements): {:?}",
                    rule_array
                );
                continue;
            }

            // Parse rule (flat array format):
            // [morphology_id, morphology_scalar, psc_multiplier, plasticity_flag,
            //  plasticity_constant, ltp_multiplier, ltd_multiplier, plasticity_window]
            //
            // NOTE: We do not maintain backward compatibility here. If a genome uses the array
            // representation it must include the full parameter set (including plasticity_window).
            if rule_array.len() < 8 {
                warn!(
                    target: "feagi-evo",
                    "Invalid mapping recipe format (need 8 elements, including plasticity_window): {:?}",
                    rule_array
                );
                continue;
            }

            let mut rule_dict = serde_json::Map::new();

            rule_dict.insert("morphology_id".to_string(), rule_array[0].clone());
            rule_dict.insert("morphology_scalar".to_string(), rule_array[1].clone());
            rule_dict.insert(
                "postSynapticCurrent_multiplier".to_string(),
                rule_array[2].clone(),
            );
            rule_dict.insert("plasticity_flag".to_string(), rule_array[3].clone());

            // Plasticity parameters (required in new design)
            rule_dict.insert("plasticity_constant".to_string(), rule_array[4].clone());
            rule_dict.insert("ltp_multiplier".to_string(), rule_array[5].clone());
            rule_dict.insert("ltd_multiplier".to_string(), rule_array[6].clone());
            rule_dict.insert("plasticity_window".to_string(), rule_array[7].clone());

            converted_rules.push(Value::Object(rule_dict));
        }

        // Avoid populating cortical_mapping_dst with empty per-destination arrays:
        // an empty array is semantically "no mapping rules", and downstream code
        // treats presence of the destination key as "has mappings".
        if !converted_rules.is_empty() {
            hierarchical_dstmap.insert(destination_area.clone(), Value::Array(converted_rules));
        }
    }

    area_data.insert(
        "cortical_mapping_dst".to_string(),
        Value::Object(hierarchical_dstmap),
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_map_completeness() {
        let map = build_property_map();
        assert_eq!(map.len(), 39); // All 39 property mappings
        assert!(map.contains_key("__name-t"));
        assert!(map.contains_key("dstmap-d"));
        assert!(map.contains_key("fire_t-f"));
    }

    #[test]
    fn test_dstmap_parsing() {
        let dstmap_flat = json!({
            "dest_area": [
                ["block_to_block", 1, 1.0, true, 1, 1, 1, 4],
                ["projector", 2, 0.5, false, 1, 1, 1, 1]
            ]
        });

        let mut area_data = serde_json::Map::new();
        process_dstmap(&dstmap_flat, &mut area_data).unwrap();

        let dstmap = area_data.get("cortical_mapping_dst").unwrap();
        let dest_rules = dstmap.get("dest_area").unwrap().as_array().unwrap();

        assert_eq!(dest_rules.len(), 2);
        assert_eq!(dest_rules[0]["morphology_id"], "block_to_block");
        assert_eq!(dest_rules[0]["plasticity_constant"], 1);
        assert_eq!(dest_rules[0]["plasticity_window"], 4);
        assert_eq!(dest_rules[1]["morphology_id"], "projector");
        assert_eq!(dest_rules[1]["plasticity_constant"], 1);
        assert_eq!(dest_rules[1]["plasticity_window"], 1);
    }

    #[test]
    fn test_dstmap_parsing_object_rules_passthrough() {
        let dstmap_flat = json!({
            "dest_area": [
                {
                    "morphology_id": "projector",
                    "morphology_scalar": [1, 1, 1],
                    "postSynapticCurrent_multiplier": 1,
                    "plasticity_flag": false
                }
            ]
        });

        let mut area_data = serde_json::Map::new();
        process_dstmap(&dstmap_flat, &mut area_data).unwrap();

        let dstmap = area_data.get("cortical_mapping_dst").unwrap();
        let dest_rules = dstmap.get("dest_area").unwrap().as_array().unwrap();

        assert_eq!(dest_rules.len(), 1);
        assert_eq!(dest_rules[0]["morphology_id"], "projector");
        assert_eq!(dest_rules[0]["postSynapticCurrent_multiplier"], 1);
        assert_eq!(dest_rules[0]["plasticity_flag"], false);
    }
}
