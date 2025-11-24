/*!
Save RuntimeGenome to JSON file.

This module provides high-level genome saving functionality that works
with RuntimeGenome objects and includes all genome sections.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use std::path::Path;
use std::fs;
use serde_json::{json, Value};
use crate::{EvoResult, RuntimeGenome, MorphologyType, MorphologyParameters, PatternElement};

/// Save RuntimeGenome to JSON file
pub fn save_genome_to_file<P: AsRef<Path>>(genome: &RuntimeGenome, path: P) -> EvoResult<()> {
    let json_str = save_genome_to_json(genome)?;
    fs::write(path, json_str)?;
    Ok(())
}

/// Save RuntimeGenome to JSON string
pub fn save_genome_to_json(genome: &RuntimeGenome) -> EvoResult<String> {
    let json_value = genome_to_json_value(genome)?;
    let json_str = serde_json::to_string_pretty(&json_value)?;
    Ok(json_str)
}

/// Convert RuntimeGenome to JSON Value
fn genome_to_json_value(genome: &RuntimeGenome) -> EvoResult<Value> {
    // Build blueprint section
    let mut blueprint = serde_json::Map::new();
    for (cortical_id, area) in &genome.cortical_areas {
        let mut area_data = serde_json::Map::new();
        
        // Required fields
        area_data.insert("cortical_name".to_string(), json!(area.name));
        area_data.insert("block_boundaries".to_string(), json!([
            area.dimensions.width,
            area.dimensions.height,
            area.dimensions.depth
        ]));
        area_data.insert("relative_coordinate".to_string(), json!([
            area.position.0,
            area.position.1,
            area.position.2
        ]));
        
        // Area type
        let cortical_type = match area.area_type {
            feagi_types::AreaType::Sensory => "IPU",
            feagi_types::AreaType::Motor => "OPU",
            feagi_types::AreaType::Memory => "MEMORY",
            feagi_types::AreaType::Custom => "CUSTOM",
        };
        area_data.insert("cortical_type".to_string(), json!(cortical_type));
        
        // Add all properties from the area's properties HashMap
        for (key, value) in &area.properties {
            area_data.insert(key.clone(), value.clone());
        }
        
        // Use base64 encoding for cortical_id (new format)
        let cortical_id_base64 = cortical_id.as_base_64();
        blueprint.insert(cortical_id_base64, Value::Object(area_data));
    }
    
    // Build brain_regions section
    let mut regions_map = serde_json::Map::new();
    for (region_id, region) in &genome.brain_regions {
        let mut region_data = serde_json::Map::new();
        
        region_data.insert("title".to_string(), json!(region.name));
        region_data.insert("parent_region_id".to_string(), Value::Null); // TODO: Track parent relationships
        
        // Cortical areas in this region
        let areas: Vec<String> = region.cortical_areas.iter().cloned().collect();
        region_data.insert("areas".to_string(), json!(areas));
        
        // Child regions
        region_data.insert("regions".to_string(), json!(Vec::<String>::new()));
        
        // Add all properties from the region's properties HashMap
        for (key, value) in &region.properties {
            region_data.insert(key.clone(), value.clone());
        }
        
        regions_map.insert(region_id.clone(), Value::Object(region_data));
    }
    
    // Build neuron_morphologies section
    let mut morphologies_map = serde_json::Map::new();
    for (morphology_id, morphology) in genome.morphologies.iter() {
        let mut morph_data = serde_json::Map::new();
        
        // Type
        let type_str = match morphology.morphology_type {
            MorphologyType::Vectors => "vectors",
            MorphologyType::Patterns => "patterns",
            MorphologyType::Functions => "functions",
            MorphologyType::Composite => "composite",
        };
        morph_data.insert("type".to_string(), json!(type_str));
        
        // Parameters
        let params = morphology_parameters_to_json(&morphology.parameters);
        morph_data.insert("parameters".to_string(), params);
        
        // Class
        morph_data.insert("class".to_string(), json!(morphology.class));
        
        morphologies_map.insert(morphology_id.clone(), Value::Object(morph_data));
    }
    
    // Build physiology section
    let physiology = json!({
        "simulation_timestep": genome.physiology.simulation_timestep,
        "max_age": genome.physiology.max_age,
        "evolution_burst_count": genome.physiology.evolution_burst_count,
        "ipu_idle_threshold": genome.physiology.ipu_idle_threshold,
        "plasticity_queue_depth": genome.physiology.plasticity_queue_depth,
        "lifespan_mgmt_interval": genome.physiology.lifespan_mgmt_interval,
    });
    
    // Build stats section
    let stats = json!({
        "innate_cortical_area_count": genome.stats.innate_cortical_area_count,
        "innate_neuron_count": genome.stats.innate_neuron_count,
        "innate_synapse_count": genome.stats.innate_synapse_count,
    });
    
    // Build signatures section
    let signatures = json!({
        "genome": genome.signatures.genome,
        "blueprint": genome.signatures.blueprint,
        "physiology": genome.signatures.physiology,
        "morphologies": genome.signatures.morphologies,
    });
    
    // Build complete genome JSON
    Ok(json!({
        "genome_id": genome.metadata.genome_id,
        "genome_title": genome.metadata.genome_title,
        "genome_description": genome.metadata.genome_description,
        "version": genome.metadata.version,
        "blueprint": Value::Object(blueprint),
        "brain_regions": Value::Object(regions_map),
        "neuron_morphologies": Value::Object(morphologies_map),
        "physiology": physiology,
        "stats": stats,
        "signatures": signatures,
        "timestamp": genome.metadata.timestamp,
    }))
}

/// Convert morphology parameters to JSON
fn morphology_parameters_to_json(params: &MorphologyParameters) -> Value {
    match params {
        MorphologyParameters::Vectors { vectors } => {
            json!({
                "vectors": vectors
            })
        }
        MorphologyParameters::Patterns { patterns } => {
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
        MorphologyParameters::Functions {} => {
            json!({})
        }
        MorphologyParameters::Composite { src_seed, src_pattern, mapper_morphology } => {
            json!({
                "src_seed": src_seed,
                "src_pattern": src_pattern,
                "mapper_morphology": mapper_morphology
            })
        }
    }
}

/// Convert pattern elements to JSON
fn pattern_elements_to_json(elements: &[PatternElement]) -> Value {
    let json_elements: Vec<Value> = elements.iter().map(|elem| {
        match elem {
            PatternElement::Value(v) => json!(v),
            PatternElement::Wildcard => json!("*"),
            PatternElement::Skip => json!("?"),
            PatternElement::Exclude => json!("!"),
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
    fn test_save_minimal_genome() {
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
        
        let json_str = save_genome_to_json(&genome).unwrap();
        assert!(json_str.contains("test_genome"));
        assert!(json_str.contains("2.0"));
    }
}

