/*!
Genome templates for FEAGI.

Provides templates for creating genomes from scratch, including:
- Minimal genome template
- Cortical area templates
- Default neural parameters

For full device templates (sensors, actuators), load from JSON files
in `feagi-py/feagi/evo/templates.py`.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::{RuntimeGenome, GenomeMetadata, MorphologyRegistry, PhysiologyConfig, GenomeSignatures, GenomeStats};
use std::collections::HashMap;

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

/// Add core morphologies to a registry
pub fn add_core_morphologies(registry: &mut MorphologyRegistry) {
    use crate::{Morphology, MorphologyType, MorphologyParameters};
    
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
                patterns: vec![
                    [
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
                    ]
                ],
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
                patterns: vec![
                    [
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
                    ]
                ],
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_minimal_genome() {
        let genome = create_minimal_genome(
            "test_genome".to_string(),
            "Test Genome".to_string(),
        );
        
        assert_eq!(genome.metadata.genome_id, "test_genome");
        assert_eq!(genome.metadata.version, "2.0");
        assert_eq!(genome.cortical_areas.len(), 0);
        assert_eq!(genome.morphologies.count(), 0);
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
}

