/*!
High-level genome loading API.

Provides convenient functions for loading genomes from files or JSON strings,
automatically parsing and converting to RuntimeGenome.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use std::path::Path;
use std::fs;
use serde_json::Value;
use crate::{EvoResult, RuntimeGenome};
use super::{GenomeParser, converter::to_runtime_genome};

/// Load a genome from a JSON file
pub fn load_genome_from_file<P: AsRef<Path>>(path: P) -> EvoResult<RuntimeGenome> {
    let json_str = fs::read_to_string(path)?;
    load_genome_from_json(&json_str)
}

/// Load a genome from a JSON string
pub fn load_genome_from_json(json_str: &str) -> EvoResult<RuntimeGenome> {
    // Parse JSON to Value first to check format
    let json_value: Value = serde_json::from_str(json_str)
        .map_err(|e| crate::types::EvoError::InvalidGenome(format!("Failed to parse JSON: {}", e)))?;
    
    // Check if genome is in flat format and convert if needed
    let hierarchical_json = if is_flat_format(&json_value) {
        // Convert flat format to hierarchical format
        crate::converter_flat_full::convert_flat_to_hierarchical_full(&json_value)?
    } else {
        json_value
    };
    
    // Convert back to JSON string for parsing
    let hierarchical_json_str = serde_json::to_string(&hierarchical_json)
        .map_err(|e| crate::types::EvoError::InvalidGenome(format!("Failed to serialize converted genome: {}", e)))?;
    
    // Parse JSON to ParsedGenome
    let parsed = GenomeParser::parse(&hierarchical_json_str)?;
    
    // Convert to RuntimeGenome
    to_runtime_genome(parsed, &hierarchical_json_str)
}

/// Check if genome is in flat format
/// Flat format has blueprint keys like "_____10c-_power-cx-subgrp-t" (with underscores)
/// Hierarchical format has blueprint keys like "cortical_id" that map to objects
fn is_flat_format(genome_value: &Value) -> bool {
    let blueprint = match genome_value.get("blueprint") {
        Some(bp) => bp,
        None => return false,
    };
    
    let blueprint_obj = match blueprint.as_object() {
        Some(obj) => obj,
        None => return false,
    };
    
    // Check if any keys look like flat format (contain multiple underscores and hyphens)
    // Flat format keys typically look like: "_____10c-_power-cx-subgrp-t"
    // Hierarchical format keys are simple IDs like: "cortical_id"
    blueprint_obj.keys().any(|key| {
        // Flat format keys typically have:
        // - Multiple underscores at start
        // - Hyphens separating parts
        // - Ending with a single letter suffix like "-t", "-i", "-f", "-b", "-d"
        key.starts_with("___") && key.contains('-') && key.len() > 20
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_load_minimal_genome() {
        let json = r#"{
            "genome_id": "test_genome",
            "genome_title": "Test Genome",
            "genome_description": "A test genome",
            "version": "2.0",
            "blueprint": {
                "tst001": {
                    "cortical_name": "Test Area",
                    "block_boundaries": [10, 10, 10],
                    "relative_coordinate": [0, 0, 0],
                    "cortical_type": "INTERCONNECT"
                }
            },
            "brain_regions": {},
            "neuron_morphologies": {},
            "physiology": {
                "simulation_timestep": 0.025,
                "max_age": 10000000
            },
            "stats": {
                "innate_cortical_area_count": 1,
                "innate_neuron_count": 0,
                "innate_synapse_count": 0
            },
            "signatures": {
                "genome": "0000000000000000",
                "blueprint": "0000000000000000",
                "physiology": "0000000000000000"
            },
            "timestamp": 1234567890.0
        }"#;
        
        let genome = load_genome_from_json(json).unwrap();
        
        assert_eq!(genome.metadata.genome_id, "test_genome");
        assert_eq!(genome.metadata.version, "2.0");
        assert_eq!(genome.cortical_areas.len(), 1);
        assert_eq!(genome.physiology.simulation_timestep, 0.025);
    }
}

