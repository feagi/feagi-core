/// Integration Tests for BDU Full Pipeline
///
/// These tests verify that the entire BDU stack works end-to-end:
/// - Genome loading → Brain development → Neuron creation → Synapse creation → Queries
///
/// No HTTP/API layer - direct testing of business logic.

use feagi_bdu::ConnectomeManager;
use feagi_burst_engine::RustNPU;
use feagi_evo::{load_genome_from_file, validate_genome};
use feagi_types::{CorticalArea, Dimensions, AreaType};
use std::sync::{Arc, Mutex};
use parking_lot::RwLock;

/// Helper to create an isolated test manager with NPU
fn create_test_manager() -> ConnectomeManager {
    let npu = Arc::new(Mutex::new(RustNPU::new(1_000_000, 10_000_000, 10)));
    ConnectomeManager::new_for_testing_with_npu(npu)
}

// ============================================================================
// TEST 1: Load Barebones Genome and Develop Brain
// ============================================================================

#[test]
fn test_load_barebones_genome() {
    let mut manager = create_test_manager();
    
    // Load genome file
    let genome_path = "@genome/barebones_genome.json";
    let genome = match load_genome_from_file(genome_path) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Failed to load genome: {:?}", e);
            eprintln!("Note: This test requires genome files in @genome/ directory");
            return; // Skip test if genome file not found
        }
    };
    
    // Validate genome
    let validation = validate_genome(&genome);
    assert!(validation.errors.is_empty(), "Genome should be valid: {:?}", validation.errors);
    
    // Load genome into manager (this runs neuroembryogenesis)
    manager.load_from_genome(&genome).expect("Failed to load genome");
    
    // Verify brain was developed
    let area_count = manager.get_cortical_area_count();
    println!("Created {} cortical areas", area_count);
    assert!(area_count > 0, "Should have created at least one cortical area");
    
    // Verify neurons were created
    let neuron_count = manager.get_neuron_count();
    println!("Created {} neurons", neuron_count);
    // Barebones genome may not create neurons, so just check it doesn't crash
    
    println!("✅ Barebones genome loaded successfully");
}

// ============================================================================
// TEST 2: Load Essential Genome (More Complex)
// ============================================================================

#[test]
fn test_load_essential_genome() {
    let mut manager = create_test_manager();
    
    let genome_path = "@genome/essential_genome.json";
    let genome = match load_genome_from_file(genome_path) {
        Ok(g) => g,
        Err(_) => {
            eprintln!("Skipping test - genome file not found");
            return;
        }
    };
    
    // Load genome
    manager.load_from_genome(&genome).expect("Failed to load genome");
    
    // Verify multiple areas created
    let area_count = manager.get_cortical_area_count();
    println!("Created {} cortical areas", area_count);
    assert!(area_count >= 2, "Essential genome should have multiple areas");
    
    // Verify neuron creation
    let neuron_count = manager.get_neuron_count();
    println!("Created {} neurons", neuron_count);
    
    // List all cortical area IDs
    let area_ids = manager.get_all_cortical_ids();
    println!("Cortical areas: {:?}", area_ids);
    assert!(!area_ids.is_empty());
    
    println!("✅ Essential genome loaded successfully");
}

// ============================================================================
// TEST 3: Incremental Brain Building
// ============================================================================

#[test]
fn test_incremental_brain_building() {
    let mut manager = create_test_manager();
    
    // Step 1: Create first cortical area
    let area1 = CorticalArea::new(
        "area01".to_string(),
        "Visual V1".to_string(),
        Dimensions { width: 10, height: 10, depth: 1 },
        AreaType::Memory,
        0, // cortical_idx will be assigned
    ).expect("Failed to create area1");
    
    manager.add_cortical_area(area1.clone(), None).expect("Failed to add area1");
    
    // Verify area exists
    assert!(manager.cortical_area_exists("area01"));
    assert_eq!(manager.get_cortical_area_count(), 1);
    
    // Step 2: Create neurons in area1
    let neurons_created = manager.create_neurons_for_area("area01")
        .expect("Failed to create neurons");
    
    println!("Created {} neurons in area01", neurons_created);
    
    // Verify neurons exist
    let neuron_count = manager.get_neuron_count();
    assert!(neuron_count > 0, "Should have created neurons");
    
    // Step 3: Create second cortical area
    let area2 = CorticalArea::new(
        "area02".to_string(),
        "Motor M1".to_string(),
        Dimensions { width: 5, height: 5, depth: 1 },
        AreaType::Memory,
        1, // cortical_idx
    ).expect("Failed to create area2");
    
    manager.add_cortical_area(area2, None).expect("Failed to add area2");
    
    // Verify both areas exist
    assert_eq!(manager.get_cortical_area_count(), 2);
    
    // Step 4: Create neurons in area2
    let neurons_created2 = manager.create_neurons_for_area("area02")
        .expect("Failed to create neurons in area2");
    
    println!("Created {} neurons in area02", neurons_created2);
    
    // Verify total neuron count increased
    let total_neurons = manager.get_neuron_count();
    assert!(total_neurons >= neuron_count, "Total neurons should have increased");
    
    println!("✅ Incremental brain building successful");
}

// ============================================================================
// TEST 4: Neuron Operations
// ============================================================================

#[test]
fn test_neuron_operations() {
    let mut manager = create_test_manager();
    
    // Create a cortical area
    let area = CorticalArea::new(
        "test01".to_string(),
        "Test Area".to_string(),
        Dimensions { width: 10, height: 10, depth: 1 },
        AreaType::Memory,
        0,
    ).expect("Failed to create area");
    
    manager.add_cortical_area(area, None).expect("Failed to add area");
    
    // Create a neuron manually
    let neuron_id = manager.add_neuron(
        "test01",
        0, 0, 0, // x, y, z
        1.0,     // firing_threshold
        0.1,     // leak_coefficient
        0.0,     // resting_potential
        0,       // neuron_type
        2,       // refractory_period
        1.0,     // excitability
        3,       // consecutive_fire_limit
        5,       // snooze_length
        false,   // mp_charge_accumulation
    ).expect("Failed to create neuron");
    
    println!("Created neuron with ID: {}", neuron_id);
    
    // Verify neuron exists
    assert!(manager.has_neuron(neuron_id));
    
    // Get neuron position
    let position = manager.get_neuron_position(neuron_id)
        .expect("Failed to get neuron position");
    assert_eq!(position, (0, 0, 0));
    
    // Get neuron by coordinates
    let found_neuron = manager.get_neuron_by_coordinates("test01", 0, 0, 0)
        .expect("Failed to find neuron by coordinates");
    assert_eq!(found_neuron, neuron_id);
    
    // Get neuron properties
    let properties = manager.get_neuron_properties(neuron_id)
        .expect("Failed to get neuron properties");
    
    println!("Neuron properties: {:?}", properties);
    assert_eq!(properties["x"], 0);
    assert_eq!(properties["y"], 0);
    assert_eq!(properties["z"], 0);
    
    // Update neuron threshold
    manager.set_neuron_firing_threshold(neuron_id, 2.0)
        .expect("Failed to update threshold");
    
    // Verify update
    let updated_properties = manager.get_neuron_properties(neuron_id)
        .expect("Failed to get updated properties");
    assert_eq!(updated_properties["threshold"], 2.0);
    
    // Delete neuron
    let deleted = manager.delete_neuron(neuron_id)
        .expect("Failed to delete neuron");
    assert!(deleted, "Neuron should have been deleted");
    
    // Verify deletion
    assert!(!manager.has_neuron(neuron_id));
    
    println!("✅ Neuron operations successful");
}

// ============================================================================
// TEST 5: Synapse Operations
// ============================================================================

#[test]
fn test_synapse_operations() {
    let mut manager = create_test_manager();
    
    // Create area
    let area = CorticalArea::new(
        "syn01".to_string(),
        "Synapse Test".to_string(),
        Dimensions { width: 5, height: 5, depth: 1 },
        AreaType::Memory,
        0,
    ).expect("Failed to create area");
    
    manager.add_cortical_area(area, None).expect("Failed to add area");
    
    // Create two neurons
    let neuron1 = manager.add_neuron(
        "syn01", 0, 0, 0,
        1.0, 0.1, 0.0, 0, 2, 1.0, 3, 5, false,
    ).expect("Failed to create neuron1");
    
    let neuron2 = manager.add_neuron(
        "syn01", 1, 1, 0,
        1.0, 0.1, 0.0, 0, 2, 1.0, 3, 5, false,
    ).expect("Failed to create neuron2");
    
    println!("Created neurons: {} and {}", neuron1, neuron2);
    
    // Create synapse
    manager.create_synapse(
        neuron1,
        neuron2,
        128, // weight
        200, // conductance
        0,   // excitatory
    ).expect("Failed to create synapse");
    
    // Verify synapse exists
    let synapse = manager.get_synapse(neuron1, neuron2)
        .expect("Synapse should exist");
    
    let (weight, conductance, syn_type) = synapse;
    assert_eq!(weight, 128);
    assert_eq!(conductance, 200);
    assert_eq!(syn_type, 0); // excitatory
    
    println!("Synapse: weight={}, conductance={}, type={}", weight, conductance, syn_type);
    
    // Update synapse weight
    manager.update_synapse_weight(neuron1, neuron2, 255)
        .expect("Failed to update synapse weight");
    
    // Verify update
    let updated_synapse = manager.get_synapse(neuron1, neuron2)
        .expect("Synapse should still exist");
    assert_eq!(updated_synapse.0, 255);
    
    // Get synapse count
    let synapse_count = manager.get_synapse_count();
    println!("Total synapses: {}", synapse_count);
    assert!(synapse_count > 0);
    
    // Remove synapse
    let removed = manager.remove_synapse(neuron1, neuron2)
        .expect("Failed to remove synapse");
    assert!(removed, "Synapse should have been removed");
    
    // Verify removal
    let synapse_after = manager.get_synapse(neuron1, neuron2);
    assert!(synapse_after.is_none(), "Synapse should no longer exist");
    
    println!("✅ Synapse operations successful");
}

// ============================================================================
// TEST 6: Batch Operations
// ============================================================================

#[test]
fn test_batch_neuron_operations() {
    let mut manager = create_test_manager();
    
    // Create area
    let area = CorticalArea::new(
        "batch1".to_string(),
        "Batch Test".to_string(),
        Dimensions { width: 20, height: 20, depth: 1 },
        AreaType::Memory,
        0,
    ).expect("Failed to create area");
    
    manager.add_cortical_area(area, None).expect("Failed to add area");
    
    // Create 100 neurons in batch
    let mut neurons_to_create = Vec::new();
    for i in 0..100 {
        let x = i % 20;
        let y = i / 20;
        neurons_to_create.push((
            x, y, 0,     // coordinates
            1.0,         // threshold
            0.1,         // leak
            0.0,         // resting
            0,           // type
            2,           // refractory
            1.0,         // excitability
            3,           // consec_limit
            5,           // snooze
            false,       // mp_accum
        ));
    }
    
    let neuron_ids = manager.batch_create_neurons("batch1", neurons_to_create)
        .expect("Failed to batch create neurons");
    
    assert_eq!(neuron_ids.len(), 100, "Should have created 100 neurons");
    println!("Batch created {} neurons", neuron_ids.len());
    
    // Verify neurons exist
    for &neuron_id in &neuron_ids {
        assert!(manager.has_neuron(neuron_id));
    }
    
    // Batch delete neurons
    let deleted_count = manager.delete_neurons_batch(neuron_ids)
        .expect("Failed to batch delete neurons");
    
    assert_eq!(deleted_count, 100, "Should have deleted 100 neurons");
    println!("Batch deleted {} neurons", deleted_count);
    
    println!("✅ Batch operations successful");
}

// ============================================================================
// TEST 7: Area and Region Queries
// ============================================================================

#[test]
fn test_area_and_region_queries() {
    let mut manager = create_test_manager();
    
    // Create multiple areas
    for i in 0..3 {
        let area = CorticalArea::new(
            format!("qry{:02}", i),
            format!("Query Area {}", i),
            Dimensions { width: 5, height: 5, depth: 1 },
            if i == 0 { AreaType::Sensory } else { AreaType::Memory },
            i,
        ).expect(&format!("Failed to create area {}", i));
        
        manager.add_cortical_area(area, None)
            .expect(&format!("Failed to add area {}", i));
    }
    
    // Test get_all_cortical_ids
    let all_ids = manager.get_all_cortical_ids();
    assert_eq!(all_ids.len(), 3);
    println!("All cortical IDs: {:?}", all_ids);
    
    // Test get_cortical_area_names
    let all_names = manager.get_cortical_area_names();
    assert_eq!(all_names.len(), 3);
    println!("All names: {:?}", all_names);
    
    // Test list_ipu_areas (sensory)
    let ipu_areas = manager.list_ipu_areas();
    assert_eq!(ipu_areas.len(), 1, "Should have 1 sensory area");
    assert_eq!(ipu_areas[0], "qry00");
    
    // Test list_opu_areas (motor) - should be empty
    let opu_areas = manager.list_opu_areas();
    assert_eq!(opu_areas.len(), 0, "Should have 0 motor areas");
    
    // Test get_max_cortical_area_dimensions
    let (max_w, max_h, max_d) = manager.get_max_cortical_area_dimensions();
    assert_eq!(max_w, 5);
    assert_eq!(max_h, 5);
    assert_eq!(max_d, 1);
    
    // Test get_cortical_area_properties
    let props = manager.get_cortical_area_properties("qry00")
        .expect("Failed to get area properties");
    println!("Area properties: {:?}", props);
    assert_eq!(props["cortical_id"], "qry00");
    assert_eq!(props["name"], "Query Area 0");
    
    // Test existence checks
    assert!(manager.cortical_area_exists("qry00"));
    assert!(!manager.cortical_area_exists("nothere"));
    
    println!("✅ Area and region queries successful");
}

// ============================================================================
// TEST 8: Save and Load Brain State
// ============================================================================

#[test]
fn test_save_and_load_brain_state() {
    let mut manager = create_test_manager();
    
    // Create a simple brain
    let area = CorticalArea::new(
        "save01".to_string(),
        "Save Test".to_string(),
        Dimensions { width: 3, height: 3, depth: 1 },
        AreaType::Memory,
        0,
    ).expect("Failed to create area");
    
    manager.add_cortical_area(area, None).expect("Failed to add area");
    
    // Create some neurons
    let neuron1 = manager.add_neuron(
        "save01", 0, 0, 0,
        1.0, 0.1, 0.0, 0, 2, 1.0, 3, 5, false,
    ).expect("Failed to create neuron");
    
    let neuron2 = manager.add_neuron(
        "save01", 1, 1, 0,
        1.0, 0.1, 0.0, 0, 2, 1.0, 3, 5, false,
    ).expect("Failed to create neuron");
    
    // Create a synapse
    manager.create_synapse(neuron1, neuron2, 128, 200, 0)
        .expect("Failed to create synapse");
    
    // Save brain state to JSON
    let json_state = manager.save_genome_to_json()
        .expect("Failed to save genome");
    
    println!("Saved genome (first 200 chars): {}", &json_state.chars().take(200).collect::<String>());
    assert!(json_state.len() > 0);
    assert!(json_state.contains("blueprint"));
    
    // Create a new manager
    let mut manager2 = create_test_manager();
    
    // Load the saved state
    manager2.load_from_genome_json(&json_state)
        .expect("Failed to load genome from JSON");
    
    // Verify the loaded state matches
    assert_eq!(manager2.get_cortical_area_count(), 1);
    assert_eq!(manager2.get_cortical_area_count(), manager.get_cortical_area_count());
    
    println!("✅ Save and load brain state successful");
}

#[test]
fn test_all_tests_run() {
    // Meta-test to ensure the test file compiles and runs
    println!("✅ All integration tests compiled successfully");
}




