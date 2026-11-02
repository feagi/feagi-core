// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Save RuntimeGenome to JSON file.

This module provides high-level genome saving functionality that works
with RuntimeGenome objects and includes all genome sections.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::{EvoResult, RuntimeGenome};
use std::fs;
use std::path::Path;

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
    use crate::{GenomeMetadata, GenomeSignatures, GenomeStats, PhysiologyConfig, RuntimeGenome};
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
                brain_regions_root: None,
            },
            cortical_areas: HashMap::new(),
            brain_regions: HashMap::new(),
            classifiers: HashMap::new(),
            morphologies: crate::MorphologyRegistry::new(),
            modulators: crate::modulators::ModulatorRegistry::new(),
            physiology: PhysiologyConfig::default(),
            signatures: GenomeSignatures {
                genome: "0000000000000000".to_string(),
                blueprint: "0000000000000000".to_string(),
                physiology: "0000000000000000".to_string(),
                morphologies: None,
            },
            stats: GenomeStats::default(),
            change_history: Vec::new(),
        };

        let json_str = save_genome_to_json(&genome).unwrap();
        assert!(json_str.contains("test_genome"));
        assert!(json_str.contains("3.0")); // Version bumped to 3.0
    }

    /// A genome saved from a brain that has modulators must come back with them: a start from
    /// the saved genome alone has no other source for the registry.
    #[test]
    fn modulators_survive_save_and_load() {
        use feagi_structures::genomic::cortical_area::{
            CorticalArea, CorticalAreaDimensions, CorticalAreaType, CorticalID, CustomCorticalType,
        };
        use feagi_structures::genomic::ModulatorKind;

        let driver = CorticalID::try_from_bytes(b"cdrvmod1").unwrap();
        let area = CorticalArea::new(
            driver,
            9,
            "Reward_Drive".to_string(),
            CorticalAreaDimensions::new(1, 1, 1).unwrap(),
            (0, 0, 0).into(),
            CorticalAreaType::Custom(CustomCorticalType::LeakyIntegrateFire),
        )
        .unwrap();
        let mut modulators = crate::modulators::ModulatorRegistry::new();
        modulators.insert(
            "Reward_Pos".to_string(),
            crate::modulators::ModulatorInstance {
                kind: ModulatorKind::Reward,
                magnitude_percent: 100.0,
                effect_duration_bursts: 1,
                rest_bursts: 0,
                graded: false,
                full_scale_potential: None,
                driver_cortical_id: driver,
            },
        );
        let genome = RuntimeGenome {
            metadata: GenomeMetadata {
                genome_id: "mods".to_string(),
                genome_title: "Mods".to_string(),
                genome_description: String::new(),
                version: "3.0".to_string(),
                timestamp: 1.0,
                brain_regions_root: None,
            },
            cortical_areas: HashMap::from([(driver, area)]),
            brain_regions: HashMap::new(),
            classifiers: HashMap::new(),
            morphologies: crate::MorphologyRegistry::new(),
            modulators,
            physiology: PhysiologyConfig::default(),
            signatures: GenomeSignatures {
                genome: "0000000000000000".to_string(),
                blueprint: "0000000000000000".to_string(),
                physiology: "0000000000000000".to_string(),
                morphologies: None,
            },
            stats: GenomeStats::default(),
            change_history: Vec::new(),
        };
        let json = save_genome_to_json(&genome).unwrap();
        let loaded = crate::load_genome_from_json(&json).unwrap();
        assert!(
            loaded.modulators.contains("Reward_Pos"),
            "the saved genome carries the modulator registry: {json}"
        );
    }
}
