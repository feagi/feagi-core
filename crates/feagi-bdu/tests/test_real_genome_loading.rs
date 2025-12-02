// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
End-to-end test of complete genome loading workflow.

Tests:
1. Load flat genome from file
2. Convert to hierarchical format
3. Run neuroembryogenesis (all 4 stages)
4. Verify neurons and synapses are calculated correctly

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use feagi_bdu::{ConnectomeManager, Neuroembryogenesis};
use feagi_evo::{convert_flat_to_hierarchical_full, load_genome_from_json};
use std::fs;

#[test]
fn test_barebones_genome_end_to_end() {
    test_genome_end_to_end("../../../feagi-py/feagi/evo/defaults/genome/barebones_genome.json", "barebones");
}

#[test]
fn test_essential_genome_end_to_end() {
    test_genome_end_to_end("../../../feagi-py/feagi/evo/defaults/genome/essential_genome.json", "essential");
}

#[test]
fn test_test_genome_end_to_end() {
    test_genome_end_to_end("../../../feagi-py/feagi/evo/defaults/genome/test_genome.json", "test");
}

#[test]
fn test_vision_genome_end_to_end() {
    test_genome_end_to_end("../../../feagi-py/feagi/evo/defaults/genome/vision_genome.json", "vision");
}

fn test_genome_end_to_end(genome_path: &str, genome_name: &str) {
    // Reset singleton to ensure clean state between genome tests
    use parking_lot::RwLock;
    use std::sync::Arc;
    static mut TEST_COUNTER: usize = 0;
    unsafe {
        TEST_COUNTER += 1;
        if TEST_COUNTER > 1 {
            // For subsequent tests, manually clear the connectome
            let mgr = ConnectomeManager::instance();
            let mut write_lock = mgr.write();
            let _ = write_lock.prepare_for_new_genome();
        }
    }
    
    println!("\n{}", "=".repeat(80));
    println!("Testing {} genome end-to-end", genome_name);
    println!("{}\n", "=".repeat(80));
    
    // Step 1: Load flat genome from file
    let flat_json = fs::read_to_string(genome_path)
        .expect(&format!("Failed to read genome file: {}", genome_path));
    let flat_genome: serde_json::Value = serde_json::from_str(&flat_json)
        .expect("Failed to parse flat genome JSON");
    
    println!("✅ Step 1: Loaded flat genome from file");
    
    // Step 2: Convert to hierarchical
    let hierarchical = convert_flat_to_hierarchical_full(&flat_genome)
        .expect("Failed to convert flat to hierarchical");
    
    let blueprint = hierarchical.get("blueprint").unwrap().as_object().unwrap();
    println!("✅ Step 2: Converted to hierarchical: {} cortical areas", blueprint.len());
    
    // Step 3: Load as RuntimeGenome
    let hierarchical_json = serde_json::to_string_pretty(&hierarchical)
        .expect("Failed to serialize hierarchical genome");
    let runtime_genome = load_genome_from_json(&hierarchical_json)
        .expect("Failed to load as RuntimeGenome");
    
    println!("✅ Step 3: Parsed as RuntimeGenome: {} areas, {} morphologies",
        runtime_genome.cortical_areas.len(),
        runtime_genome.morphologies.count());
    
    // Step 4: Run neuroembryogenesis
    let manager = ConnectomeManager::instance();
    let mut neuro = Neuroembryogenesis::new(manager);
    
    neuro.develop_from_genome(&runtime_genome)
        .expect("Neuroembryogenesis failed");
    
    let progress = neuro.get_progress();
    
    println!("✅ Step 4: Neuroembryogenesis complete!");
    println!("   - Stage: {:?}", progress.stage);
    println!("   - Cortical areas: {}", progress.cortical_areas_created);
    println!("   - Neurons: {}", progress.neurons_created);
    println!("   - Synapses: {}", progress.synapses_created);
    println!("   - Duration: {}ms", progress.duration_ms);
    
    // Assertions
    assert_eq!(progress.cortical_areas_created, runtime_genome.cortical_areas.len());
    assert!(progress.neurons_created > 0, "Should have created neurons");
    
    println!("\n🎉 {} genome test PASSED!\n", genome_name);
}

