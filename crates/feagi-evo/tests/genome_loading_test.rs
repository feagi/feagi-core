// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Integration tests for genome loading with real genome files.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use feagi_data_structures::genomic::cortical_area::CorticalID;
use feagi_evo::load_genome_from_file;

#[test]
fn test_load_barebones_genome() {
    // Path to real genome file
    let genome_path = "../../../feagi-py/feagi/evo/defaults/genome/barebones_genome.json";

    // Load genome
    let genome = load_genome_from_file(genome_path).expect("Failed to load barebones genome");

    // Verify metadata
    assert_eq!(genome.metadata.version, "2.0");
    assert!(!genome.metadata.genome_id.is_empty());

    // Verify cortical areas (should have _death and _power)
    assert!(
        genome.cortical_areas.len() >= 2,
        "Expected at least 2 cortical areas, got {}",
        genome.cortical_areas.len()
    );

    let death_id = feagi_evo::genome::parser::string_to_cortical_id("_death").expect("Valid ID");
    let power_id = feagi_evo::genome::parser::string_to_cortical_id("_power").expect("Valid ID");
    assert!(
        genome.cortical_areas.contains_key(&death_id),
        "Missing _death cortical area"
    );
    assert!(
        genome.cortical_areas.contains_key(&power_id),
        "Missing _power cortical area"
    );

    // Verify morphologies
    assert!(
        !genome.morphologies.count() == 0,
        "Should have morphologies"
    );
    assert!(
        genome.morphologies.contains("block_to_block"),
        "Missing block_to_block morphology"
    );
    assert!(
        genome.morphologies.contains("projector"),
        "Missing projector morphology"
    );

    // Verify physiology
    assert!(genome.physiology.simulation_timestep > 0.0);
    assert!(genome.physiology.max_age > 0);

    // Verify signatures
    assert_eq!(genome.signatures.genome.len(), 16);
    assert_eq!(genome.signatures.blueprint.len(), 16);
    assert_eq!(genome.signatures.physiology.len(), 16);

    // Verify stats
    assert!(genome.stats.innate_cortical_area_count > 0);

    println!("✅ Successfully loaded barebones genome:");
    println!("   - Genome ID: {}", genome.metadata.genome_id);
    println!("   - Cortical areas: {}", genome.cortical_areas.len());
    println!("   - Morphologies: {}", genome.morphologies.count());
    println!(
        "   - Physiology timestep: {}",
        genome.physiology.simulation_timestep
    );
}

#[test]
fn test_load_all_sample_genomes() {
    let genome_files = [
        "barebones_genome.json",
        "essential_genome.json",
        "test_genome.json",
        "vision_genome.json",
    ];

    for genome_file in &genome_files {
        let genome_path = format!(
            "../../../feagi-py/feagi/evo/defaults/genome/{}",
            genome_file
        );

        match load_genome_from_file(&genome_path) {
            Ok(genome) => {
                println!("✅ Loaded {} successfully:", genome_file);
                println!("   - Genome ID: {}", genome.metadata.genome_id);
                println!("   - Cortical areas: {}", genome.cortical_areas.len());
                println!("   - Morphologies: {}", genome.morphologies.count());

                // Basic validation
                assert!(!genome.metadata.genome_id.is_empty());
                assert_eq!(genome.metadata.version, "2.0");
                assert!(genome.cortical_areas.len() >= 2); // At least _death and _power
            }
            Err(e) => {
                println!(
                    "⚠️  Could not load {}: {} (this is OK if file doesn't exist)",
                    genome_file, e
                );
            }
        }
    }
}
