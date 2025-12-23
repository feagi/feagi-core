// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Genome templates for FEAGI.

Provides templates for creating genomes from scratch, including:
- Minimal genome template
- Cortical area templates (IPU, OPU, CORE)
- Default neural parameters
- Embedded default genomes

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::{
    GenomeMetadata, GenomeSignatures, GenomeStats, MorphologyRegistry, PhysiologyConfig,
    RuntimeGenome,
};
use feagi_structures::genomic::cortical_area::CoreCorticalType;
use feagi_structures::genomic::cortical_area::{CorticalArea, CorticalAreaDimensions};
use feagi_structures::genomic::descriptors::GenomeCoordinate3D;
use serde_json::Value;
use std::collections::HashMap;

/// Embedded essential genome (loaded at compile time)
pub const ESSENTIAL_GENOME_JSON: &str = include_str!("../genomes/essential_genome.json");

/// Embedded barebones genome (loaded at compile time)
pub const BAREBONES_GENOME_JSON: &str = include_str!("../genomes/barebones_genome.json");

/// Embedded test genome (loaded at compile time)
pub const TEST_GENOME_JSON: &str = include_str!("../genomes/test_genome.json");

/// Embedded vision genome (loaded at compile time)
pub const VISION_GENOME_JSON: &str = include_str!("../genomes/vision_genome.json");

/// Default neural properties for all cortical areas
pub fn get_default_neural_properties() -> HashMap<String, Value> {
    let mut props = HashMap::new();
    props.insert("per_voxel_neuron_cnt".to_string(), Value::from(1));
    props.insert("synapse_attractivity".to_string(), Value::from(100.0));
    props.insert("degeneration".to_string(), Value::from(0.0));
    props.insert("psp_uniform_distribution".to_string(), Value::from(true));
    props.insert("postsynaptic_current_max".to_string(), Value::from(10000.0));
    props.insert("postsynaptic_current".to_string(), Value::from(500.0));
    props.insert("firing_threshold".to_string(), Value::from(0.1));
    props.insert("refractory_period".to_string(), Value::from(0));
    props.insert("leak_coefficient".to_string(), Value::from(0.0));
    props.insert("leak_variability".to_string(), Value::from(0.0));
    props.insert("consecutive_fire_cnt_max".to_string(), Value::from(0));
    props.insert("snooze_length".to_string(), Value::from(0));
    props.insert("mp_charge_accumulation".to_string(), Value::from(false));
    props.insert("mp_driven_psp".to_string(), Value::from(false));
    props.insert("neuron_excitability".to_string(), Value::from(1.0));
    props.insert("visualization".to_string(), Value::from(true));
    props.insert(
        "cortical_mapping_dst".to_string(),
        Value::Object(serde_json::Map::new()),
    );
    props
}

/// Create _death cortical area (cortical_idx = 0) from template
pub fn create_death_area() -> CorticalArea {
    let cortical_id = CoreCorticalType::Death.to_cortical_id();
    let cortical_type = cortical_id
        .as_cortical_type()
        .expect("Death cortical ID should map to Core type");

    let mut area = CorticalArea::new(
        cortical_id,
        0, // cortical_idx = 0 (reserved)
        "Death".to_string(),
        CorticalAreaDimensions::new(1, 1, 1).expect("Failed to create dimensions"),
        GenomeCoordinate3D::new(0, 0, -10),
        cortical_type,
    )
    .expect("Failed to create _death area");

    let mut props = get_default_neural_properties();
    props.insert("cortical_group".to_string(), Value::from("CORE"));
    props.insert("2d_coordinate".to_string(), Value::from(vec![-10, -20]));
    area.properties = props;
    area
}

/// Create _power cortical area (cortical_idx = 1) from template
pub fn create_power_area() -> CorticalArea {
    let cortical_id = CoreCorticalType::Power.to_cortical_id();
    let cortical_type = cortical_id
        .as_cortical_type()
        .expect("Power cortical ID should map to Core type");

    let mut area = CorticalArea::new(
        cortical_id,
        1, // cortical_idx = 1 (reserved)
        "Brain_Power".to_string(),
        CorticalAreaDimensions::new(1, 1, 1).expect("Failed to create dimensions"),
        GenomeCoordinate3D::new(0, 0, -20),
        cortical_type,
    )
    .expect("Failed to create _power area");

    let mut props = get_default_neural_properties();
    props.insert("cortical_group".to_string(), Value::from("CORE"));
    props.insert("2d_coordinate".to_string(), Value::from(vec![-10, -10]));
    props.insert("firing_threshold".to_string(), Value::from(0.1));
    props.insert("postsynaptic_current".to_string(), Value::from(500.0));
    props.insert("neuron_excitability".to_string(), Value::from(100.0));
    area.properties = props;
    area
}

/// Create a minimal empty genome
pub fn create_minimal_genome(genome_id: String, genome_title: String) -> RuntimeGenome {
    RuntimeGenome {
        metadata: GenomeMetadata {
            genome_id,
            genome_title,
            genome_description: "Minimal genome template".to_string(),
            version: "2.0".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            brain_regions_root: None, // Will be set after neuroembryogenesis
        },
        cortical_areas: HashMap::new(),
        brain_regions: HashMap::new(),
        morphologies: MorphologyRegistry::new(),
        physiology: PhysiologyConfig::default(),
        signatures: GenomeSignatures {
            genome: String::new(),
            blueprint: String::new(),
            physiology: String::new(),
            morphologies: None,
        },
        stats: GenomeStats::default(),
    }
}

/// Create a genome with core areas (_death, _power)
pub fn create_genome_with_core_areas(genome_id: String, genome_title: String) -> RuntimeGenome {
    let mut genome = create_minimal_genome(genome_id, genome_title);

    // Add core areas (convert 6-char strings to CorticalID)
    let death_id =
        crate::genome::parser::string_to_cortical_id("_death").expect("Valid cortical ID");
    let power_id =
        crate::genome::parser::string_to_cortical_id("_power").expect("Valid cortical ID");

    genome.cortical_areas.insert(death_id, create_death_area());
    genome.cortical_areas.insert(power_id, create_power_area());

    genome
}

/// Create a genome with core morphologies
pub fn create_genome_with_core_morphologies(
    genome_id: String,
    genome_title: String,
) -> RuntimeGenome {
    let mut genome = create_minimal_genome(genome_id, genome_title);

    // Add core morphologies
    add_core_morphologies(&mut genome.morphologies);

    genome
}

/// CRITICAL: Ensure a genome has all required core components
///
/// This function checks if a genome has:
/// 1. Core cortical areas (_death, _power)
/// 2. Core morphologies (block_to_block, projector, etc.)
///
/// If any are missing, they are automatically added. This ensures every genome
/// can function properly regardless of its source.
///
/// # Arguments
/// * `genome` - The genome to validate and fix
///
/// # Returns
/// A tuple of (areas_added, morphologies_added) indicating what was added
pub fn ensure_core_components(genome: &mut RuntimeGenome) -> (usize, usize) {
    let mut areas_added = 0;
    let mut morphologies_added = 0;

    // Convert core area IDs
    let death_id =
        crate::genome::parser::string_to_cortical_id("_death").expect("Valid cortical ID");
    let power_id =
        crate::genome::parser::string_to_cortical_id("_power").expect("Valid cortical ID");

    // 1. Ensure core cortical areas exist
    if let std::collections::hash_map::Entry::Vacant(e) = genome.cortical_areas.entry(death_id) {
        let death_area = create_death_area();
        e.insert(death_area);
        areas_added += 1;
        tracing::info!("Added missing core area: _death (cortical_idx=0)");
    }

    if let std::collections::hash_map::Entry::Vacant(e) = genome.cortical_areas.entry(power_id) {
        let power_area = create_power_area();
        e.insert(power_area);
        areas_added += 1;
        tracing::info!("Added missing core area: _power (cortical_idx=1)");
    }

    // 2. Ensure core morphologies exist
    let required_morphologies = vec![
        "block_to_block",
        "projector",
        "memory",
        "all_to_0-0-0",
        "0-0-0_to_all",
        "lateral_+x",
        "lateral_-x",
        "lateral_+y",
        "lateral_-y",
        "lateral_+z",
        "lateral_-z",
    ];

    for morph_name in required_morphologies {
        if !genome.morphologies.contains(morph_name) {
            morphologies_added += 1;
        }
    }

    // Add all missing core morphologies in one call
    if morphologies_added > 0 {
        add_core_morphologies(&mut genome.morphologies);
        tracing::info!("Added {} missing core morphologies", morphologies_added);
    }

    (areas_added, morphologies_added)
}

/// Add core morphologies to a registry
pub fn add_core_morphologies(registry: &mut MorphologyRegistry) {
    use crate::{Morphology, MorphologyParameters, MorphologyType};

    // block_to_block - Connect neurons in same position
    registry.add_morphology(
        "block_to_block".to_string(),
        Morphology {
            morphology_type: MorphologyType::Vectors,
            parameters: MorphologyParameters::Vectors {
                vectors: vec![[0, 0, 0]],
            },
            class: "core".to_string(),
        },
    );

    // projector - Function-based morphology
    registry.add_morphology(
        "projector".to_string(),
        Morphology {
            morphology_type: MorphologyType::Functions,
            parameters: MorphologyParameters::Functions {},
            class: "core".to_string(),
        },
    );

    // memory - Function-based morphology
    registry.add_morphology(
        "memory".to_string(),
        Morphology {
            morphology_type: MorphologyType::Functions,
            parameters: MorphologyParameters::Functions {},
            class: "core".to_string(),
        },
    );

    // all_to_0-0-0 - Connect all neurons to origin
    registry.add_morphology(
        "all_to_0-0-0".to_string(),
        Morphology {
            morphology_type: MorphologyType::Patterns,
            parameters: MorphologyParameters::Patterns {
                patterns: vec![[
                    vec![
                        crate::PatternElement::Wildcard,
                        crate::PatternElement::Wildcard,
                        crate::PatternElement::Wildcard,
                    ],
                    vec![
                        crate::PatternElement::Value(0),
                        crate::PatternElement::Value(0),
                        crate::PatternElement::Value(0),
                    ],
                ]],
            },
            class: "core".to_string(),
        },
    );

    // 0-0-0_to_all - Connect origin to all neurons
    registry.add_morphology(
        "0-0-0_to_all".to_string(),
        Morphology {
            morphology_type: MorphologyType::Patterns,
            parameters: MorphologyParameters::Patterns {
                patterns: vec![[
                    vec![
                        crate::PatternElement::Value(0),
                        crate::PatternElement::Value(0),
                        crate::PatternElement::Value(0),
                    ],
                    vec![
                        crate::PatternElement::Wildcard,
                        crate::PatternElement::Wildcard,
                        crate::PatternElement::Wildcard,
                    ],
                ]],
            },
            class: "core".to_string(),
        },
    );

    // lateral_+x - Connect along +X axis
    registry.add_morphology(
        "lateral_+x".to_string(),
        Morphology {
            morphology_type: MorphologyType::Vectors,
            parameters: MorphologyParameters::Vectors {
                vectors: vec![[1, 0, 0]],
            },
            class: "core".to_string(),
        },
    );

    // lateral_-x - Connect along -X axis
    registry.add_morphology(
        "lateral_-x".to_string(),
        Morphology {
            morphology_type: MorphologyType::Vectors,
            parameters: MorphologyParameters::Vectors {
                vectors: vec![[-1, 0, 0]],
            },
            class: "core".to_string(),
        },
    );

    // lateral_+y - Connect along +Y axis
    registry.add_morphology(
        "lateral_+y".to_string(),
        Morphology {
            morphology_type: MorphologyType::Vectors,
            parameters: MorphologyParameters::Vectors {
                vectors: vec![[0, 1, 0]],
            },
            class: "core".to_string(),
        },
    );

    // lateral_-y - Connect along -Y axis
    registry.add_morphology(
        "lateral_-y".to_string(),
        Morphology {
            morphology_type: MorphologyType::Vectors,
            parameters: MorphologyParameters::Vectors {
                vectors: vec![[0, -1, 0]],
            },
            class: "core".to_string(),
        },
    );

    // lateral_+z - Connect along +Z axis
    registry.add_morphology(
        "lateral_+z".to_string(),
        Morphology {
            morphology_type: MorphologyType::Vectors,
            parameters: MorphologyParameters::Vectors {
                vectors: vec![[0, 0, 1]],
            },
            class: "core".to_string(),
        },
    );

    // lateral_-z - Connect along -Z axis
    registry.add_morphology(
        "lateral_-z".to_string(),
        Morphology {
            morphology_type: MorphologyType::Vectors,
            parameters: MorphologyParameters::Vectors {
                vectors: vec![[0, 0, -1]],
            },
            class: "core".to_string(),
        },
    );
}

/// Load essential genome from embedded JSON
///
/// Automatically ensures core components (_death, _power, core morphologies) are present
pub fn load_essential_genome() -> Result<RuntimeGenome, crate::types::EvoError> {
    use crate::genome::loader::load_genome_from_json;
    let mut genome = load_genome_from_json(ESSENTIAL_GENOME_JSON)?;
    let (areas_added, morphs_added) = ensure_core_components(&mut genome);
    if areas_added > 0 || morphs_added > 0 {
        tracing::info!(
            "Essential genome: added {} core areas, {} core morphologies",
            areas_added,
            morphs_added
        );
    }
    Ok(genome)
}

/// Load barebones genome from embedded JSON
///
/// Automatically ensures core components (_death, _power, core morphologies) are present
pub fn load_barebones_genome() -> Result<RuntimeGenome, crate::types::EvoError> {
    use crate::genome::loader::load_genome_from_json;
    let mut genome = load_genome_from_json(BAREBONES_GENOME_JSON)?;
    let (areas_added, morphs_added) = ensure_core_components(&mut genome);
    if areas_added > 0 || morphs_added > 0 {
        tracing::info!(
            "Barebones genome: added {} core areas, {} core morphologies",
            areas_added,
            morphs_added
        );
    }
    Ok(genome)
}

/// Load test genome from embedded JSON
///
/// Automatically ensures core components (_death, _power, core morphologies) are present
pub fn load_test_genome() -> Result<RuntimeGenome, crate::types::EvoError> {
    use crate::genome::loader::load_genome_from_json;
    let mut genome = load_genome_from_json(TEST_GENOME_JSON)?;
    let (areas_added, morphs_added) = ensure_core_components(&mut genome);
    if areas_added > 0 || morphs_added > 0 {
        tracing::info!(
            "Test genome: added {} core areas, {} core morphologies",
            areas_added,
            morphs_added
        );
    }
    Ok(genome)
}

/// Load vision genome from embedded JSON
///
/// Automatically ensures core components (_death, _power, core morphologies) are present
pub fn load_vision_genome() -> Result<RuntimeGenome, crate::types::EvoError> {
    use crate::genome::loader::load_genome_from_json;
    let mut genome = load_genome_from_json(VISION_GENOME_JSON)?;
    let (areas_added, morphs_added) = ensure_core_components(&mut genome);
    if areas_added > 0 || morphs_added > 0 {
        tracing::info!(
            "Vision genome: added {} core areas, {} core morphologies",
            areas_added,
            morphs_added
        );
    }
    Ok(genome)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_minimal_genome() {
        let genome = create_minimal_genome("test_genome".to_string(), "Test Genome".to_string());

        assert_eq!(genome.metadata.genome_id, "test_genome");
        assert_eq!(genome.metadata.version, "2.0");
        assert_eq!(genome.cortical_areas.len(), 0);
        assert_eq!(genome.morphologies.count(), 0);
    }

    #[test]
    fn test_create_genome_with_core_areas() {
        let genome =
            create_genome_with_core_areas("test_genome".to_string(), "Test Genome".to_string());

        assert_eq!(genome.metadata.genome_id, "test_genome");
        assert_eq!(genome.cortical_areas.len(), 2);

        let death_id = crate::genome::parser::string_to_cortical_id("_death").expect("Valid ID");
        let power_id = crate::genome::parser::string_to_cortical_id("_power").expect("Valid ID");
        assert!(genome.cortical_areas.contains_key(&death_id));
        assert!(genome.cortical_areas.contains_key(&power_id));

        // Verify _power has correct properties
        let power = genome.cortical_areas.get(&power_id).unwrap();
        assert_eq!(power.cortical_id.as_base_64(), power_id.as_base_64());
        assert_eq!(power.cortical_idx, 1);
        assert_eq!(power.dimensions.width, 1);
        assert_eq!(power.dimensions.height, 1);
        assert_eq!(power.dimensions.depth, 1);
    }

    #[test]
    fn test_create_genome_with_core_morphologies() {
        let genome = create_genome_with_core_morphologies(
            "test_genome".to_string(),
            "Test Genome".to_string(),
        );

        assert_eq!(genome.metadata.genome_id, "test_genome");
        assert!(genome.morphologies.count() > 0);
        assert!(genome.morphologies.contains("block_to_block"));
        assert!(genome.morphologies.contains("projector"));
        assert!(genome.morphologies.contains("lateral_+x"));
    }

    #[test]
    fn test_add_core_morphologies() {
        let mut registry = MorphologyRegistry::new();
        add_core_morphologies(&mut registry);

        // Should have at least 11 core morphologies
        assert!(registry.count() >= 11);
        assert!(registry.contains("block_to_block"));
        assert!(registry.contains("projector"));
        assert!(registry.contains("all_to_0-0-0"));
        assert!(registry.contains("lateral_+x"));
        assert!(registry.contains("lateral_-z"));
    }

    #[test]
    fn test_embedded_genomes_exist() {
        // Test that embedded genome strings are not empty
        // These are compile-time constants, so they're always non-empty
        // The assertions verify the constants are defined correctly
        #[allow(clippy::const_is_empty)]
        {
            assert!(!ESSENTIAL_GENOME_JSON.is_empty());
            assert!(!BAREBONES_GENOME_JSON.is_empty());
            assert!(!TEST_GENOME_JSON.is_empty());
            assert!(!VISION_GENOME_JSON.is_empty());
        }
    }

    #[test]
    fn test_load_essential_genome() {
        let genome = load_essential_genome().expect("Failed to load essential genome");
        assert!(!genome.cortical_areas.is_empty());
        // Essential genome should have _power
        let power_id = crate::genome::parser::string_to_cortical_id("_power").expect("Valid ID");
        assert!(genome.cortical_areas.contains_key(&power_id));
    }

    #[test]
    fn test_ensure_core_components_adds_missing_areas() {
        // Create a minimal genome without core areas
        let mut genome = create_minimal_genome("test".to_string(), "Test".to_string());

        assert_eq!(genome.cortical_areas.len(), 0);

        // Ensure core components
        let (areas_added, _) = ensure_core_components(&mut genome);

        // Should have added _death and _power
        assert_eq!(areas_added, 2);

        let death_id = crate::genome::parser::string_to_cortical_id("_death").expect("Valid ID");
        let power_id = crate::genome::parser::string_to_cortical_id("_power").expect("Valid ID");
        assert!(genome.cortical_areas.contains_key(&death_id));
        assert!(genome.cortical_areas.contains_key(&power_id));

        // Verify cortical_idx assignments
        assert_eq!(
            genome.cortical_areas.get(&death_id).unwrap().cortical_idx,
            0
        );
        assert_eq!(
            genome.cortical_areas.get(&power_id).unwrap().cortical_idx,
            1
        );
    }

    #[test]
    fn test_ensure_core_components_adds_missing_morphologies() {
        // Create a genome with core areas but no morphologies
        let mut genome = create_genome_with_core_areas("test".to_string(), "Test".to_string());

        assert_eq!(genome.morphologies.count(), 0);

        // Ensure core components
        let (_, morphs_added) = ensure_core_components(&mut genome);

        // Should have added core morphologies
        assert!(morphs_added > 0);
        assert!(genome.morphologies.contains("block_to_block"));
        assert!(genome.morphologies.contains("projector"));
        assert!(genome.morphologies.contains("memory"));
        assert!(genome.morphologies.contains("lateral_+x"));
    }

    #[test]
    fn test_ensure_core_components_idempotent() {
        // Create a genome with all core components
        let mut genome = create_genome_with_core_areas("test".to_string(), "Test".to_string());
        add_core_morphologies(&mut genome.morphologies);

        // Run ensure_core_components
        let (areas_added, morphs_added) = ensure_core_components(&mut genome);

        // Should not add anything (already present)
        assert_eq!(areas_added, 0);
        assert_eq!(morphs_added, 0);
    }
}
