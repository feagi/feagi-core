// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Save RuntimeGenome to JSON file.

This module provides high-level genome saving functionality that works
with RuntimeGenome objects and includes all genome sections.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use std::path::Path;
use std::fs;
use crate::{EvoResult, RuntimeGenome};

/// Save RuntimeGenome to JSON file
pub fn save_genome_to_file<P: AsRef<Path>>(genome: &RuntimeGenome, path: P) -> EvoResult<()> {
    let json_str = save_genome_to_json(genome)?;
    fs::write(path, json_str)?;
    Ok(())
}

/// Save RuntimeGenome to JSON string in flat format (version 3.0)
pub fn save_genome_to_json(genome: &RuntimeGenome) -> EvoResult<String> {
    // Use the hierarchical-to-flat converter
    let json_value = crate::converter_hierarchical_to_flat::convert_hierarchical_to_flat(genome)?;
    let json_str = serde_json::to_string_pretty(&json_value)?;
    Ok(json_str)
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
        assert!(json_str.contains("3.0")); // Version bumped to 3.0
    }
}

