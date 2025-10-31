/*!
Test loading real flat (2.0) genome files.

This tests the full flat-to-hierarchical converter with actual genome files.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use feagi_evo::{convert_flat_to_hierarchical_full, load_genome_from_json};
use std::fs;

#[test]
fn test_load_barebones_flat_genome() {
    let genome_path = "../../../feagi-py/feagi/evo/defaults/genome/barebones_genome.json";
    
    // Read flat genome
    let flat_json = fs::read_to_string(genome_path)
        .expect(&format!("Failed to read genome file: {}", genome_path));
    
    let flat_genome: serde_json::Value = serde_json::from_str(&flat_json)
        .expect("Failed to parse flat genome JSON");
    
    // Convert to hierarchical
    let hierarchical = convert_flat_to_hierarchical_full(&flat_genome)
        .expect("Failed to convert flat to hierarchical");
    
    // Verify conversion
    assert!(hierarchical.get("blueprint").is_some(), "Missing blueprint section");
    let blueprint = hierarchical.get("blueprint").unwrap().as_object().unwrap();
    assert!(!blueprint.is_empty(), "Blueprint should not be empty");
    
    println!("✅ Converted barebones genome: {} cortical areas", blueprint.len());
    
    // Try to load as RuntimeGenome
    let hierarchical_json = serde_json::to_string_pretty(&hierarchical)
        .expect("Failed to serialize hierarchical genome");
    
    let runtime_genome = load_genome_from_json(&hierarchical_json)
        .expect("Failed to load converted genome as RuntimeGenome");
    
    assert!(!runtime_genome.cortical_areas.is_empty(), "Should have cortical areas");
    println!("✅ Loaded as RuntimeGenome: {} cortical areas, {} morphologies",
        runtime_genome.cortical_areas.len(),
        runtime_genome.morphologies.count());
}

#[test]
fn test_load_all_flat_genomes() {
    let genome_files = vec![
        "../../../feagi-py/feagi/evo/defaults/genome/barebones_genome.json",
        "../../../feagi-py/feagi/evo/defaults/genome/essential_genome.json",
        "../../../feagi-py/feagi/evo/defaults/genome/test_genome.json",
        "../../../feagi-py/feagi/evo/defaults/genome/vision_genome.json",
    ];
    
    for genome_path in genome_files {
        println!("\n📂 Testing: {}", genome_path);
        
        match fs::read_to_string(genome_path) {
            Ok(flat_json) => {
                match serde_json::from_str::<serde_json::Value>(&flat_json) {
                    Ok(flat_genome) => {
                        match convert_flat_to_hierarchical_full(&flat_genome) {
                            Ok(hierarchical) => {
                                let blueprint = hierarchical.get("blueprint")
                                    .and_then(|b| b.as_object())
                                    .expect("Missing or invalid blueprint");
                                
                                println!("  ✅ Converted: {} cortical areas", blueprint.len());
                            }
                            Err(e) => {
                                println!("  ❌ Conversion failed: {}", e);
                                panic!("Conversion failed for {}: {}", genome_path, e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("  ❌ JSON parse failed: {}", e);
                        panic!("JSON parse failed for {}: {}", genome_path, e);
                    }
                }
            }
            Err(e) => {
                println!("  ⚠️  File not found: {}", e);
                // Don't fail the test if file doesn't exist (might be in different location)
            }
        }
    }
}


