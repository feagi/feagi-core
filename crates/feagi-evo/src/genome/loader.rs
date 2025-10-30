/*!
High-level genome loading API.

Provides convenient functions for loading genomes from files or JSON strings,
automatically parsing and converting to RuntimeGenome.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use std::path::Path;
use std::fs;
use crate::{EvoResult, RuntimeGenome};
use super::{GenomeParser, converter::to_runtime_genome};

/// Load a genome from a JSON file
pub fn load_genome_from_file<P: AsRef<Path>>(path: P) -> EvoResult<RuntimeGenome> {
    let json_str = fs::read_to_string(path)?;
    load_genome_from_json(&json_str)
}

/// Load a genome from a JSON string
pub fn load_genome_from_json(json_str: &str) -> EvoResult<RuntimeGenome> {
    // Parse JSON to ParsedGenome
    let parsed = GenomeParser::parse(json_str)?;
    
    // Convert to RuntimeGenome
    to_runtime_genome(parsed, json_str)
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

