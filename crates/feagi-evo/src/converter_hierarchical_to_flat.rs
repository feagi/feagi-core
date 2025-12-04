// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Convert hierarchical genome format (RuntimeGenome) to flat genome format (3.0).

The flat format uses keys like "_____10c-AREA1-cx-property-type" with all
cortical IDs in base64 format. This is the inverse of converter_flat_full.rs.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use std::collections::HashMap;
use serde_json::{json, Value};
use crate::{EvoResult, RuntimeGenome};
use tracing::warn;

/// Convert hierarchical genome (RuntimeGenome) to flat format (3.0)
///
/// This produces the flat genome format compatible with the original essential_genome.json
/// but with all cortical IDs in base64 format.
///
/// # Arguments
///
/// * `genome` - RuntimeGenome to convert
///
/// # Returns
///
/// JSON Value in flat format with version 3.0
///
pub fn convert_hierarchical_to_flat(genome: &RuntimeGenome) -> EvoResult<Value> {
    let mut flat_blueprint = serde_json::Map::new();
    
    // Convert each cortical area to flat format
    for (cortical_id, area) in &genome.cortical_areas {
        let cortical_id_base64 = cortical_id.as_base_64();
        convert_area_to_flat(&cortical_id_base64, area, &mut flat_blueprint)?;
    }
    
    // Build complete flat genome
    let mut flat_genome = serde_json::Map::new();
    
    // Metadata
    flat_genome.insert("genome_id".to_string(), json!(genome.metadata.genome_id));
    flat_genome.insert("genome_title".to_string(), json!(genome.metadata.genome_title));
    flat_genome.insert("genome_description".to_string(), json!(genome.metadata.genome_description));
    flat_genome.insert("version".to_string(), json!("3.0"));
    flat_genome.insert("timestamp".to_string(), json!(genome.metadata.timestamp));
    
    // Blueprint (flat format)
    flat_genome.insert("blueprint".to_string(), Value::Object(flat_blueprint));
    
    // Neuron morphologies (keep as-is)
    let mut morphologies_map = serde_json::Map::new();
    for (morphology_id, morphology) in genome.morphologies.iter() {
        let mut morph_data = serde_json::Map::new();
        
        let type_str = match morphology.morphology_type {
            crate::MorphologyType::Vectors => "vectors",
            crate::MorphologyType::Patterns => "patterns",
            crate::MorphologyType::Functions => "functions",
            crate::MorphologyType::Composite => "composite",
        };
        morph_data.insert("type".to_string(), json!(type_str));
        
        let params = morphology_parameters_to_json(&morphology.parameters);
        morph_data.insert("parameters".to_string(), params);
        
        morph_data.insert("class".to_string(), json!(morphology.class));
        
        morphologies_map.insert(morphology_id.clone(), Value::Object(morph_data));
    }
    flat_genome.insert("neuron_morphologies".to_string(), Value::Object(morphologies_map));
    
    // Physiology
    let physiology = json!({
        "simulation_timestep": genome.physiology.simulation_timestep,
        "max_age": genome.physiology.max_age,
        "evolution_burst_count": genome.physiology.evolution_burst_count,
        "ipu_idle_threshold": genome.physiology.ipu_idle_threshold,
        "plasticity_queue_depth": genome.physiology.plasticity_queue_depth,
        "lifespan_mgmt_interval": genome.physiology.lifespan_mgmt_interval,
        "quantization_precision": "fp32", // Default
    });
    flat_genome.insert("physiology".to_string(), physiology);
    
    // Stats
    let stats = json!({
        "innate_cortical_area_count": genome.stats.innate_cortical_area_count,
        "innate_neuron_count": genome.stats.innate_neuron_count,
        "innate_synapse_count": genome.stats.innate_synapse_count,
    });
    flat_genome.insert("stats".to_string(), stats);
    
    // Signatures
    let signatures = json!({
        "genome": genome.signatures.genome,
        "blueprint": genome.signatures.blueprint,
        "physiology": genome.signatures.physiology,
    });
    flat_genome.insert("signatures".to_string(), signatures);
    
    // Hosts (empty for now)
    flat_genome.insert("hosts".to_string(), json!({}));
    
    // Brain regions (with cortical IDs converted to base64)
    if !genome.brain_regions.is_empty() {
        let mut brain_regions_map = serde_json::Map::new();
        
        for (region_id, region) in &genome.brain_regions {
            let mut region_data = serde_json::Map::new();
            
            // Serialize all properties from the BrainRegion
            let region_json = serde_json::to_value(region)
                .map_err(|e| crate::EvoError::JsonError(e.to_string()))?;
            
            if let Value::Object(mut props) = region_json {
                // Convert cortical ID arrays to base64
                let keys_to_convert = vec!["areas", "inputs", "outputs", "cortical_areas"];
                for key in keys_to_convert {
                    if let Some(Value::Array(ids)) = props.get(key) {
                        let converted_ids: Vec<String> = ids.iter()
                            .filter_map(|v| v.as_str())
                            .map(|id_str| {
                                // Try to parse as CorticalID and convert to base64
                                crate::genome::parser::string_to_cortical_id(id_str)
                                    .map(|cid| cid.as_base_64())
                                    .unwrap_or_else(|_| id_str.to_string())
                            })
                            .collect();
                        props.insert(key.to_string(), json!(converted_ids));
                    }
                }
                
                region_data = props;
            }
            
            brain_regions_map.insert(region_id.clone(), Value::Object(region_data));
        }
        
        flat_genome.insert("brain_regions".to_string(), Value::Object(brain_regions_map));
    }
    
    Ok(Value::Object(flat_genome))
}

/// Convert a single cortical area to flat format keys
fn convert_area_to_flat(
    cortical_id_base64: &str,
    area: &feagi_data_structures::genomic::cortical_area::CorticalArea,
    flat_blueprint: &mut serde_json::Map<String, Value>,
) -> EvoResult<()> {
    let prefix = format!("_____10c-{}", cortical_id_base64);
    
    // Name (only if not already in properties)
    if !area.properties.contains_key("cortical_name") {
        flat_blueprint.insert(format!("{}-cx-__name-t", prefix), json!(area.name));
    }
    
    // Dimensions (block_boundaries) - only if not already in properties
    if !area.properties.contains_key("block_boundaries") {
        flat_blueprint.insert(format!("{}-cx-___bbx-i", prefix), json!(area.dimensions.width));
        flat_blueprint.insert(format!("{}-cx-___bby-i", prefix), json!(area.dimensions.height));
        flat_blueprint.insert(format!("{}-cx-___bbz-i", prefix), json!(area.dimensions.depth));
    }
    
    // Position (relative_coordinate) - only if not already in properties
    if !area.properties.contains_key("relative_coordinate") {
        flat_blueprint.insert(format!("{}-cx-rcordx-i", prefix), json!(area.position.0));
        flat_blueprint.insert(format!("{}-cx-rcordy-i", prefix), json!(area.position.1));
        flat_blueprint.insert(format!("{}-cx-rcordz-i", prefix), json!(area.position.2));
    }
    
    // Convert all properties from area.properties using reverse mapping
    // This includes cortical_group (_group-t) which should come from properties, not area_type
    convert_properties_to_flat(&prefix, &area.properties, flat_blueprint)?;
    
    Ok(())
}

/// Convert hierarchical properties to flat format using reverse property mapping
fn convert_properties_to_flat(
    prefix: &str,
    properties: &HashMap<String, Value>,
    flat_blueprint: &mut serde_json::Map<String, Value>,
) -> EvoResult<()> {
    // Reverse property mapping: hierarchical_key -> (flat_suffix, scope)
    // This MUST match converter_flat_full.rs PROPERTY_MAPPINGS exactly (reversed)
    let property_mapping: HashMap<&str, (&str, &str)> = [
        ("per_voxel_neuron_cnt", ("_n_cnt-i", "cx")),
        ("visualization", ("gd_vis-b", "cx")),
        ("cortical_name", ("__name-t", "cx")),
        ("synapse_attractivity", ("synatt-f", "cx")),
        ("postsynaptic_current", ("pstcr_-f", "nx")),
        ("postsynaptic_current_max", ("pstcrm-f", "nx")),
        ("firing_threshold", ("fire_t-f", "nx")),
        ("firing_threshold_increment_x", ("ftincx-f", "nx")),
        ("firing_threshold_increment_y", ("ftincy-f", "nx")),
        ("firing_threshold_increment_z", ("ftincz-f", "nx")),
        ("firing_threshold_limit", ("fthlim-f", "nx")),
        ("refractory_period", ("refrac-i", "nx")),
        ("leak_coefficient", ("leak_c-f", "nx")),
        ("leak_variability", ("leak_v-f", "nx")),
        ("consecutive_fire_cnt_max", ("c_fr_c-i", "nx")),
        ("snooze_length", ("snooze-f", "nx")),  // FIXED: was snooze-i
        ("group_id", ("_group-t", "cx")),
        ("sub_group_id", ("subgrp-t", "cx")),
        ("degeneration", ("de_gen-f", "cx")),
        ("psp_uniform_distribution", ("pspuni-b", "cx")),
        ("mp_charge_accumulation", ("mp_acc-b", "nx")),
        ("mp_driven_psp", ("mp_psp-b", "nx")),
        ("is_mem_type", ("memory-b", "cx")),
        ("longterm_mem_threshold", ("mem__t-i", "cx")),
        ("lifespan_growth_rate", ("mem_gr-i", "cx")),
        ("init_lifespan", ("mem_ls-i", "cx")),
        ("temporal_depth", ("tmpdpt-i", "cx")),
        ("neuron_excitability", ("excite-f", "nx")),
        ("dev_count", ("devcnt-i", "cx")),
    ].iter().cloned().collect();
    
    for (key, value) in properties {
        if key == "cortical_mapping_dst" {
            // dstmap keys are already in base64 format (converted during genome load)
            if let Some(dstmap_obj) = value.as_object() {
                flat_blueprint.insert(
                    format!("{}-cx-dstmap-d", prefix),
                    json!(dstmap_obj)
                );
            }
        } else if key == "2d_coordinate" {
            // Handle 2D coordinates - split array into separate keys
            if let Some(coords) = value.as_array() {
                if coords.len() >= 2 {
                    flat_blueprint.insert(format!("{}-cx-2dcorx-i", prefix), coords[0].clone());
                    flat_blueprint.insert(format!("{}-cx-2dcory-i", prefix), coords[1].clone());
                }
            }
        } else if key == "block_boundaries" {
            // Handle block_boundaries - split array into separate keys
            if let Some(bounds) = value.as_array() {
                if bounds.len() >= 3 {
                    flat_blueprint.insert(format!("{}-cx-___bbx-i", prefix), bounds[0].clone());
                    flat_blueprint.insert(format!("{}-cx-___bby-i", prefix), bounds[1].clone());
                    flat_blueprint.insert(format!("{}-cx-___bbz-i", prefix), bounds[2].clone());
                }
            }
        } else if key == "relative_coordinate" {
            // Handle relative_coordinate - split array into separate keys
            if let Some(coords) = value.as_array() {
                if coords.len() >= 3 {
                    flat_blueprint.insert(format!("{}-cx-rcordx-i", prefix), coords[0].clone());
                    flat_blueprint.insert(format!("{}-cx-rcordy-i", prefix), coords[1].clone());
                    flat_blueprint.insert(format!("{}-cx-rcordz-i", prefix), coords[2].clone());
                }
            }
        } else if key == "cortical_group" {
            // Map cortical_group to _group-t
            flat_blueprint.insert(format!("{}-cx-_group-t", prefix), value.clone());
        } else if let Some((suffix, scope)) = property_mapping.get(key.as_str()) {
            // Use reverse mapping
            flat_blueprint.insert(format!("{}-{}-{}", prefix, scope, suffix), value.clone());
        } else {
            // Skip unknown properties - don't add them to flat format
            // (The original flat genome only has known properties)
            warn!("Skipping unknown property '{}' in cortical area (not in flat format)", key);
        }
    }
    
    Ok(())
}

/// Convert morphology parameters to JSON
fn morphology_parameters_to_json(params: &crate::MorphologyParameters) -> Value {
    match params {
        crate::MorphologyParameters::Vectors { vectors } => {
            json!({
                "vectors": vectors
            })
        }
        crate::MorphologyParameters::Patterns { patterns } => {
            let patterns_json: Vec<Value> = patterns.iter().map(|pattern| {
                json!([
                    pattern_elements_to_json(&pattern[0]),
                    pattern_elements_to_json(&pattern[1])
                ])
            }).collect();
            
            json!({
                "patterns": patterns_json
            })
        }
        crate::MorphologyParameters::Functions {} => {
            json!({})
        }
        crate::MorphologyParameters::Composite { src_seed, src_pattern, mapper_morphology } => {
            json!({
                "src_seed": src_seed,
                "src_pattern": src_pattern,
                "mapper_morphology": mapper_morphology
            })
        }
    }
}

/// Convert pattern elements to JSON
fn pattern_elements_to_json(elements: &[crate::PatternElement]) -> Value {
    let json_elements: Vec<Value> = elements.iter().map(|elem| {
        match elem {
            crate::PatternElement::Value(v) => json!(v),
            crate::PatternElement::Wildcard => json!("*"),
            crate::PatternElement::Skip => json!("?"),
            crate::PatternElement::Exclude => json!("!"),
        }
    }).collect();
    
    json!(json_elements)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RuntimeGenome, GenomeMetadata, PhysiologyConfig, GenomeSignatures, GenomeStats};
    use std::collections::HashMap;
    
    #[test]
    fn test_convert_minimal_genome() {
        let genome = RuntimeGenome {
            metadata: GenomeMetadata {
                genome_id: "test_genome".to_string(),
                genome_title: "Test Genome".to_string(),
                genome_description: "A test genome".to_string(),
                version: "2.0".to_string(),
                timestamp: 1234567890.0,
            },
            cortical_areas: HashMap::new(),
            brain_regions: HashMap::new(),
            morphologies: crate::MorphologyRegistry::new(),
            physiology: PhysiologyConfig::default(),
            signatures: GenomeSignatures {
                genome: "0000000000000000".to_string(),
                blueprint: "0000000000000000".to_string(),
                physiology: "0000000000000000".to_string(),
                morphologies: None,
            },
            stats: GenomeStats::default(),
        };
        
        let flat = convert_hierarchical_to_flat(&genome).unwrap();
        
        assert_eq!(flat["genome_id"], "test_genome");
        assert_eq!(flat["version"], "3.0");
        assert!(flat["blueprint"].is_object());
        assert!(flat["neuron_morphologies"].is_object());
        assert!(flat["physiology"].is_object());
    }
}

