// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/// Simple Integration Tests for BDU
///
/// Tests basic functionality without complex genome loading
use feagi_brain_development::{ConnectomeManager, CorticalArea, CorticalID};
use feagi_npu_burst_engine::RustNPU;
use feagi_structures::genomic::cortical_area::CorticalAreaDimensions;
use feagi_npu_burst_engine::TracingMutex;
use std::sync::Arc;

/// Helper to create an isolated test manager with NPU
fn create_test_manager() -> ConnectomeManager {
    let runtime = feagi_npu_runtime::StdRuntime;
    let backend = feagi_npu_burst_engine::backend::CPUBackend::new();
    let npu_result =
        RustNPU::new(runtime, backend, 1_000_000, 10_000_000, 10).expect("Failed to create NPU");
    let npu = Arc::new(TracingMutex::new(
        feagi_npu_burst_engine::DynamicNPU::F32(npu_result),
        "TestNPU",
    ));
    ConnectomeManager::new_for_testing_with_npu(npu)
}

// ============================================================================
// TEST 1: Create Cortical Area
// ============================================================================

#[test]
fn test_create_cortical_area() {
    let mut manager = create_test_manager();

    // Create a cortical area
    let cortical_id = CorticalID::try_from_base_64("cust000").unwrap();
    let cortical_type = cortical_id
        .as_cortical_type()
        .expect("Failed to get cortical type");
    let area = CorticalArea::new(
        cortical_id,
        0, // cortical_idx
        "Test Area".to_string(),
        CorticalAreaDimensions::new(10, 10, 1).unwrap(),
        (0, 0, 0).into(), // position
        cortical_type,
    )
    .expect("Failed to create cortical area");

    // Add to manager
    manager
        .add_cortical_area(area)
        .expect("Failed to add cortical area");

    // Verify it exists
    assert!(manager.cortical_area_exists(&CorticalID::try_from_base_64("cust000").unwrap()));
    assert_eq!(manager.get_cortical_area_count(), 1);

    println!("✅ Test 1: Create cortical area - PASSED");
}

// ============================================================================
// TEST 2: Create and Query Neurons
// ============================================================================

#[test]
fn test_create_and_query_neurons() {
    let mut manager = create_test_manager();

    // Create area
    let cortical_id = CorticalID::try_from_base_64("cust000").unwrap();
    let cortical_type = cortical_id
        .as_cortical_type()
        .expect("Failed to get cortical type");
    let area = CorticalArea::new(
        cortical_id,
        0,
        "Neuron Test".to_string(),
        CorticalAreaDimensions::new(10, 10, 1).unwrap(),
        (0, 0, 0).into(),
        cortical_type,
    )
    .expect("Failed to create area");

    manager.add_cortical_area(area).expect("Failed to add area");

    // Create a neuron
    let neuron_cortical_id = CorticalID::try_from_base_64("neuro1").unwrap();
    let neuron_id = manager
        .add_neuron(
            &neuron_cortical_id,
            5,
            5,
            0,     // x, y, z
            1.0,   // firing_threshold
            f32::MAX, // firing_threshold_limit (MAX = no limit)
            0.1,   // leak_coefficient
            0.0,   // resting_potential
            0,     // neuron_type
            2,     // refractory_period
            1.0,   // excitability
            3,     // consecutive_fire_limit
            5,     // snooze_length
            false, // mp_charge_accumulation
        )
        .expect("Failed to create neuron");

    println!("Created neuron with ID: {}", neuron_id);

    // Verify neuron exists
    assert!(manager.has_neuron(neuron_id));

    // Get neuron position
    let position = manager
        .get_neuron_position(neuron_id)
        .expect("Neuron should have position");
    assert_eq!(position, (5, 5, 0));

    // Find neuron by coordinates
    let found_id = manager
        .get_neuron_by_coordinates(&neuron_cortical_id, 5, 5, 0)
        .expect("Should find neuron by coordinates");
    assert_eq!(found_id, neuron_id);

    // Get neuron properties
    let props = manager
        .get_neuron_properties(neuron_id)
        .expect("Should get neuron properties");

    assert_eq!(props["x"], 5);
    assert_eq!(props["y"], 5);
    assert_eq!(props["z"], 0);
    assert_eq!(props["threshold"], 1.0);

    println!("✅ Test 2: Create and query neurons - PASSED");
}

// ============================================================================
// TEST 3: Create and Query Synapses
// ============================================================================

#[test]
fn test_create_and_query_synapses() {
    let mut manager = create_test_manager();

    // Create area
    let cortical_id = CorticalID::try_from_base_64("cust000").unwrap();
    let cortical_type = cortical_id
        .as_cortical_type()
        .expect("Failed to get cortical type");
    let area = CorticalArea::new(
        cortical_id,
        0,
        "Synapse Test".to_string(),
        CorticalAreaDimensions::new(10, 10, 1).unwrap(),
        (0, 0, 0).into(),
        cortical_type,
    )
    .expect("Failed to create area");

    manager.add_cortical_area(area).expect("Failed to add area");

    // Create two neurons
    let synap_cortical_id = CorticalID::try_from_base_64("synap1").unwrap();
    let neuron1 = manager
        .add_neuron(
            &synap_cortical_id,
            0,
            0,
            0,
            1.0,
            f32::MAX,
            0.1,
            0.0,
            0,
            2,
            1.0,
            3,
            5,
            false,
        )
        .expect("Failed to create neuron1");

    let neuron2 = manager
        .add_neuron(
            &synap_cortical_id,
            1,
            1,
            0,
            1.0,
            f32::MAX,
            0.1,
            0.0,
            0,
            2,
            1.0,
            3,
            5,
            false,
        )
        .expect("Failed to create neuron2");

    // Create synapse
    manager
        .create_synapse(
            neuron1, neuron2, 128, // weight
            200, // conductance
            0,   // excitatory
        )
        .expect("Failed to create synapse");

    // Verify synapse exists
    let synapse = manager
        .get_synapse(neuron1, neuron2)
        .expect("Synapse should exist");

    let (weight, conductance, syn_type) = synapse;
    assert_eq!(weight, 128);
    assert_eq!(conductance, 200);
    assert_eq!(syn_type, 0);

    // Update synapse weight
    manager
        .update_synapse_weight(neuron1, neuron2, 255)
        .expect("Failed to update weight");

    // Verify update
    let updated = manager
        .get_synapse(neuron1, neuron2)
        .expect("Synapse should still exist");
    assert_eq!(updated.0, 255);

    // Get synapse count
    let count = manager.get_synapse_count();
    assert!(count > 0);

    println!("✅ Test 3: Create and query synapses - PASSED");
}

// ============================================================================
// TEST 4: Batch Neuron Operations
// ============================================================================

#[test]
fn test_batch_neuron_operations() {
    let mut manager = create_test_manager();

    // Create area
    let cortical_id = CorticalID::try_from_base_64("cust000").unwrap();
    let cortical_type = cortical_id
        .as_cortical_type()
        .expect("Failed to get cortical type");
    let area = CorticalArea::new(
        cortical_id,
        0,
        "Batch Test".to_string(),
        CorticalAreaDimensions::new(20, 20, 1).unwrap(),
        (0, 0, 0).into(),
        cortical_type,
    )
    .expect("Failed to create area");

    manager.add_cortical_area(area).expect("Failed to add area");

    // Create 50 neurons in batch
    let mut neurons_to_create = Vec::new();
    for i in 0..50 {
        let x = (i % 20) as u32;
        let y = (i / 20) as u32;
        neurons_to_create.push((
            x, y, 0u32,     // coordinates
            1.0,   // threshold
            f32::MAX, // threshold_limit
            0.1,   // leak
            0.0,   // resting
            0,     // type
            2,     // refractory
            1.0,   // excitability
            3,     // consec_limit
            5,     // snooze
            false, // mp_accum
        ));
    }

    let batch_cortical_id = CorticalID::try_from_base_64("batch1").unwrap();
    let neuron_ids = manager
        .batch_create_neurons(&batch_cortical_id, neurons_to_create)
        .expect("Failed to batch create neurons");

    assert_eq!(neuron_ids.len(), 50);
    println!("Batch created {} neurons", neuron_ids.len());
    println!(
        "First neuron ID: {}, Last neuron ID: {}",
        neuron_ids[0],
        neuron_ids[neuron_ids.len() - 1]
    );
    println!(
        "Total neuron count in manager: {}",
        manager.get_neuron_count()
    );

    // Verify first and last neuron exist
    let first_exists = manager.has_neuron(neuron_ids[0]);
    println!(
        "First neuron (ID {}) exists: {}",
        neuron_ids[0], first_exists
    );
    assert!(first_exists, "First neuron should exist");

    let last_exists = manager.has_neuron(neuron_ids[neuron_ids.len() - 1]);
    println!("Last neuron exists: {}", last_exists);
    assert!(last_exists, "Last neuron should exist");

    // Verify total neuron count increased
    let total_neurons = manager.get_neuron_count();
    assert!(total_neurons >= 50, "Should have at least 50 neurons");

    // Batch delete
    let deleted_count = manager
        .delete_neurons_batch(neuron_ids)
        .expect("Failed to batch delete");

    assert_eq!(deleted_count, 50);

    println!("✅ Test 4: Batch neuron operations - PASSED");
}

// ============================================================================
// TEST 5: Area Queries
// ============================================================================

#[test]
fn test_area_queries() {
    let mut manager = create_test_manager();

    // Create multiple areas
    for i in 0..3 {
        let cortical_id = CorticalID::try_from_base_64(&format!("area{:02}", i)).unwrap();
        let area = CorticalArea::new(
            cortical_id,
            i,
            format!("Area {}", i),
            CorticalAreaDimensions::new(5, 5, 1).unwrap(),
            (0, 0, 0).into(),
            cortical_id
                .as_cortical_type()
                .expect("Failed to get cortical type"),
        )
        .unwrap_or_else(|_| panic!("Failed to create area {}", i));

        manager
            .add_cortical_area(area)
            .unwrap_or_else(|_| panic!("Failed to add area {}", i));
    }

    // Test queries
    let all_ids = manager.get_all_cortical_ids();
    assert_eq!(all_ids.len(), 3);

    let all_names = manager.get_cortical_area_names();
    assert_eq!(all_names.len(), 3);

    let ipu_areas = manager.list_ipu_areas();
    assert_eq!(ipu_areas.len(), 1);

    let opu_areas = manager.list_opu_areas();
    assert_eq!(opu_areas.len(), 0);

    let area00_id = CorticalID::try_from_base_64("area00").unwrap();
    let nothere_id = CorticalID::try_from_base_64("nothere").unwrap();
    assert!(manager.cortical_area_exists(&area00_id));
    assert!(!manager.cortical_area_exists(&nothere_id));

    println!("✅ Test 5: Area queries - PASSED");
}

// ============================================================================
// TEST 6: Update Operations
// ============================================================================

#[test]
fn test_update_operations() {
    let mut manager = create_test_manager();

    // Create area (cortical_id must be 6 chars)
    let cortical_id = CorticalID::try_from_base_64("upd001").unwrap();
    let area = CorticalArea::new(
        cortical_id,
        0,
        "Update Test".to_string(),
        CorticalAreaDimensions::new(10, 10, 1).unwrap(),
        (0, 0, 0).into(),
        cortical_id
            .as_cortical_type()
            .expect("Failed to get cortical type"),
    )
    .expect("Failed to create area");

    manager.add_cortical_area(area).expect("Failed to add area");

    // Create neuron
    let upd_cortical_id = CorticalID::try_from_base_64("upd001").unwrap();
    let neuron_id = manager
        .add_neuron(
            &upd_cortical_id,
            0,
            0,
            0,
            1.0,
            f32::MAX,
            0.1,
            0.0,
            0,
            2,
            1.0,
            3,
            5,
            false,
        )
        .expect("Failed to create neuron");

    // Update neuron threshold
    manager
        .set_neuron_firing_threshold(neuron_id, 2.5)
        .expect("Failed to update threshold");

    // Verify update
    let props = manager
        .get_neuron_properties(neuron_id)
        .expect("Should get properties");
    assert_eq!(props["threshold"], 2.5);

    // Update multiple properties
    manager
        .update_neuron_properties(
            neuron_id,
            Some(3.0),  // threshold
            Some(0.2),  // leak
            Some(-0.5), // resting
            Some(0.8),  // excitability
        )
        .expect("Failed to update properties");

    // Verify all updates (use epsilon comparison for f32 precision)
    let updated_props = manager
        .get_neuron_properties(neuron_id)
        .expect("Should get updated properties");

    // Helper to compare floats with epsilon
    let epsilon = 0.0001;
    let assert_float_eq = |actual: f64, expected: f64, name: &str| {
        assert!(
            (actual - expected).abs() < epsilon,
            "{} mismatch: got {}, expected {}",
            name,
            actual,
            expected
        );
    };

    assert_float_eq(
        updated_props["threshold"].as_f64().unwrap(),
        3.0,
        "threshold",
    );
    assert_float_eq(
        updated_props["leak_coefficient"].as_f64().unwrap(),
        0.2,
        "leak_coefficient",
    );
    assert_float_eq(
        updated_props["resting_potential"].as_f64().unwrap(),
        -0.5,
        "resting_potential",
    );
    assert_float_eq(
        updated_props["excitability"].as_f64().unwrap(),
        0.8,
        "excitability",
    );

    println!("✅ Test 6: Update operations - PASSED");
}

// ============================================================================
// TEST 7: Delete Operations
// ============================================================================

#[test]
fn test_delete_operations() {
    let mut manager = create_test_manager();

    // Create area (cortical_id must be 6 chars)
    let cortical_id = CorticalID::try_from_base_64("del001").unwrap();
    let area = CorticalArea::new(
        cortical_id,
        0,
        "Delete Test".to_string(),
        CorticalAreaDimensions::new(10, 10, 1).unwrap(),
        (0, 0, 0).into(),
        cortical_id
            .as_cortical_type()
            .expect("Failed to get cortical type"),
    )
    .expect("Failed to create area");

    manager.add_cortical_area(area).expect("Failed to add area");

    // Create neurons
    let del_cortical_id = CorticalID::try_from_base_64("del001").unwrap();
    let neuron1 = manager
        .add_neuron(
            &del_cortical_id,
            0,
            0,
            0,
            1.0,
            f32::MAX,
            0.1,
            0.0,
            0,
            2,
            1.0,
            3,
            5,
            false,
        )
        .expect("Failed to create neuron1");

    let neuron2 = manager
        .add_neuron(
            &del_cortical_id,
            1,
            1,
            0,
            1.0,
            f32::MAX,
            0.1,
            0.0,
            0,
            2,
            1.0,
            3,
            5,
            false,
        )
        .expect("Failed to create neuron2");

    // Create synapse
    manager
        .create_synapse(neuron1, neuron2, 128, 200, 0)
        .expect("Failed to create synapse");

    // Verify synapse exists
    assert!(manager.get_synapse(neuron1, neuron2).is_some());

    // Delete synapse
    let removed = manager
        .remove_synapse(neuron1, neuron2)
        .expect("Failed to remove synapse");
    assert!(removed);

    // Verify synapse gone
    assert!(manager.get_synapse(neuron1, neuron2).is_none());

    // Delete neuron
    let deleted = manager
        .delete_neuron(neuron1)
        .expect("Failed to delete neuron");
    assert!(deleted);

    // Verify neuron gone
    assert!(!manager.has_neuron(neuron1));

    println!("✅ Test 7: Delete operations - PASSED");
}

#[test]
fn test_all_simple_tests_pass() {
    println!("✅ All simple integration tests compiled and can run!");
}
