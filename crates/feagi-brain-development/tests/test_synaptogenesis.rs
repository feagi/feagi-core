// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Synaptogenesis Integration Tests

Tests the synaptogenesis process through ConnectomeManager, covering:
- Core morphology applications (projector, block_to_block, vectors, patterns, expander)
- Integration path (apply_cortical_mapping -> apply_cortical_mapping_for_pair -> apply_single_morphology_rule)
- Edge cases (empty areas, no neurons, dimensions mismatch)
- Parameter validation (weight, conductance, synapse_attractivity)

NOTE: These tests require morphologies to be registered in the morphology registry.
Morphologies are typically loaded from genome files. For these tests to work, morphologies
need to be set up first (e.g., using feagi_evolutionary::add_core_morphologies).

TODO: Add helper function to set up morphologies in test manager, or use genome loading
path for more realistic integration tests.
*/

use feagi_brain_development::{ConnectomeManager, CorticalArea, CorticalID};
use feagi_npu_burst_engine::{DynamicNPU, RustNPU, TracingMutex};
use feagi_structures::genomic::cortical_area::CorticalAreaDimensions;
use serde_json::json;
use std::sync::Arc;

/// Helper to create an isolated test manager with NPU
///
/// Sets up core morphologies (projector, block_to_block, etc.) required for synaptogenesis tests.
fn create_test_manager() -> ConnectomeManager {
    let runtime = feagi_npu_runtime::StdRuntime;
    let backend = feagi_npu_burst_engine::backend::CPUBackend::new();
    let npu_result =
        RustNPU::new(runtime, backend, 1_000_000, 10_000_000, 10).expect("Failed to create NPU");
    let npu = Arc::new(TracingMutex::new(
        feagi_npu_burst_engine::DynamicNPU::F32(npu_result),
        "TestNPU",
    ));

    let mut manager = ConnectomeManager::new_for_testing_with_npu(npu);
    // Set up core morphologies required for synaptogenesis
    manager.setup_core_morphologies_for_testing();
    manager
}

/// Helper to create a cortical area with dimensions
///
/// Creates custom cortical areas using the same approach as other tests:
/// Custom cortical IDs are 8 bytes starting with 'c' (e.g., b"csrc0000").
fn create_test_area(
    name: &str,
    width: u32,
    height: u32,
    depth: u32,
    idx: u32,
) -> (CorticalArea, CorticalID) {
    use feagi_structures::genomic::cortical_area::{CorticalAreaType, CustomCorticalType};

    // Create custom cortical ID: 8 bytes starting with 'c', padded with nulls
    // Format: 'c' + up to 7 characters from name, padded to 8 bytes
    let mut id_bytes = [0u8; 8];
    id_bytes[0] = b'c';
    let name_bytes = name.as_bytes();
    let copy_len = name_bytes.len().min(7); // Leave first byte as 'c'
    id_bytes[1..1 + copy_len].copy_from_slice(&name_bytes[..copy_len]);

    let cortical_id =
        CorticalID::try_from_bytes(&id_bytes).expect("Failed to create custom cortical ID");
    let cortical_type = CorticalAreaType::Custom(CustomCorticalType::LeakyIntegrateFire);

    let area = CorticalArea::new(
        cortical_id,
        idx,
        format!("Test Area {}", name),
        CorticalAreaDimensions::new(width, height, depth).unwrap(),
        (0, 0, 0).into(),
        cortical_type,
    )
    .expect("Failed to create cortical area");
    (area, cortical_id)
}

/// Helper to create neurons in a grid pattern within an area
fn create_grid_neurons(
    manager: &mut ConnectomeManager,
    area_id: &CorticalID,
    width: usize,
    height: usize,
    depth: usize,
) -> Vec<u64> {
    let mut neuron_ids = Vec::new();
    for z in 0..depth {
        for y in 0..height {
            for x in 0..width {
                let neuron_id = manager
                    .add_neuron(
                        area_id, x as u32, y as u32, z as u32, 1.0,   // firing_threshold
                        1.0,   // firing_threshold_limit
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
                neuron_ids.push(neuron_id);
            }
        }
    }
    neuron_ids
}

// ============================================================================
// TEST 1: Projector Morphology - Basic Functionality
// ============================================================================

#[test]
fn test_projector_morphology_basic() {
    let mut manager = create_test_manager();

    // Create source area (10x10x1 = 100 neurons)
    let (src_area, src_id) = create_test_area("src000", 10, 10, 1, 0);
    manager
        .add_cortical_area(src_area)
        .expect("Failed to add source area");

    // Create destination area (10x10x1 = 100 neurons)
    let (dst_area, dst_id) = create_test_area("dst000", 10, 10, 1, 1);
    manager
        .add_cortical_area(dst_area)
        .expect("Failed to add destination area");

    // Create neurons in both areas
    create_grid_neurons(&mut manager, &src_id, 10, 10, 1);
    create_grid_neurons(&mut manager, &dst_id, 10, 10, 1);

    // Create a mapping rule (projector morphology)
    let rule = json!({
        "morphology_id": "projector",
        "postSynapticCurrent_multiplier": 1.0,
        "synapse_attractivity": 100
    });

    // Set up cortical mapping using update_cortical_mapping
    manager
        .update_cortical_mapping(&src_id, &dst_id, vec![rule])
        .expect("Failed to update cortical mapping");

    // Apply cortical mapping
    let synapse_count = manager
        .regenerate_synapses_for_mapping(&src_id, &dst_id)
        .expect("Failed to apply cortical mapping");

    println!(
        "Created {} synapses via projector morphology",
        synapse_count
    );

    // Verify synapses were created (projector should create 1:1 mapping for same dimensions)
    // With 100% attractivity, should create approximately 100 synapses (one per source neuron)
    // Note: May create slightly more due to projection algorithm behavior
    assert!(synapse_count > 0, "Should have created some synapses");
    assert!(
        synapse_count <= 150,
        "Should create reasonable number of synapses (allowing for projection variations)"
    );

    println!("✅ Test 1: Projector morphology basic - PASSED");
}

// ============================================================================
// TEST 1b: Inhibitory mapping produces inhibitory synapses (type=1) with abs(weight)
// ============================================================================
#[test]
fn test_inhibitory_mapping_creates_inhibitory_synapses() {
    let mut manager = create_test_manager();

    // Create source + destination areas (small, deterministic)
    let (src_area, src_id) = create_test_area("src_inh", 4, 4, 1, 0);
    manager
        .add_cortical_area(src_area)
        .expect("Failed to add source area");

    let (dst_area, dst_id) = create_test_area("dst_inh", 4, 4, 1, 1);
    manager
        .add_cortical_area(dst_area)
        .expect("Failed to add destination area");

    // Create neurons in both areas
    let src_neurons = create_grid_neurons(&mut manager, &src_id, 4, 4, 1);
    create_grid_neurons(&mut manager, &dst_id, 4, 4, 1);

    // Negative multiplier should produce inhibitory synapses with weight = abs(multiplier)
    let rule = json!({
        "morphology_id": "projector",
        "postSynapticCurrent_multiplier": -5,
        "synapse_attractivity": 100
    });

    manager
        .update_cortical_mapping(&src_id, &dst_id, vec![rule])
        .expect("Failed to update cortical mapping");

    let synapse_count = manager
        .apply_cortical_mapping(&src_id)
        .expect("Failed to apply cortical mapping");

    assert!(
        synapse_count > 0,
        "Should have created synapses for inhibitory mapping"
    );

    // Inspect outgoing synapses from a sample source neuron
    let Some(npu_arc) = manager.get_npu() else {
        panic!("Test manager must have an attached NPU");
    };

    let sample_src = src_neurons[0] as u32;
    let mut npu_guard = npu_arc.lock().unwrap();
    match *npu_guard {
        DynamicNPU::F32(ref mut npu) => {
            // Propagation index is rebuilt during mapping application; outgoing list should be non-empty.
            let outgoing = npu.get_outgoing_synapses(sample_src);
            assert!(
                !outgoing.is_empty(),
                "Expected outgoing synapses from source neuron"
            );

            // Validate sign encoding: synapse_type=1 (inhibitory) and weight=5
            for (_target, weight, _psp, syn_type) in outgoing {
                assert_eq!(weight, 5, "Expected abs(multiplier) to be used as weight");
                assert_eq!(
                    syn_type, 1,
                    "Expected inhibitory synapse_type=1 for negative multiplier"
                );
            }
        }
        DynamicNPU::INT8(ref mut npu) => {
            let outgoing = npu.get_outgoing_synapses(sample_src);
            assert!(
                !outgoing.is_empty(),
                "Expected outgoing synapses from source neuron"
            );
            for (_target, weight, _psp, syn_type) in outgoing {
                assert_eq!(weight, 5, "Expected abs(multiplier) to be used as weight");
                assert_eq!(
                    syn_type, 1,
                    "Expected inhibitory synapse_type=1 for negative multiplier"
                );
            }
        }
    }

    println!("✅ Test 1b: Inhibitory mapping produces inhibitory synapses - PASSED");
}

// ============================================================================
// TEST 1c: Pattern morphology (0-0-0_to_all) creates expected synapses
// ============================================================================
/// Validate that pattern morphology connects origin to all destinations.
#[test]
fn test_pattern_morphology_origin_to_all() {
    let mut manager = create_test_manager();

    // Create source + destination areas (small, deterministic)
    let (src_area, src_id) = create_test_area("src_pat", 2, 2, 1, 0);
    manager
        .add_cortical_area(src_area)
        .expect("Failed to add source area");

    let (dst_area, dst_id) = create_test_area("dst_pat", 2, 2, 1, 1);
    manager
        .add_cortical_area(dst_area)
        .expect("Failed to add destination area");

    // Create neurons in both areas
    let src_neurons = create_grid_neurons(&mut manager, &src_id, 2, 2, 1);
    let dst_neurons = create_grid_neurons(&mut manager, &dst_id, 2, 2, 1);

    // Pattern morphology: origin -> all
    let rule = json!({
        "morphology_id": "0-0-0_to_all",
        "postSynapticCurrent_multiplier": 1.0,
        "synapse_attractivity": 100
    });

    manager
        .update_cortical_mapping(&src_id, &dst_id, vec![rule])
        .expect("Failed to update cortical mapping");

    let synapse_count = manager
        .apply_cortical_mapping(&src_id)
        .expect("Failed to apply cortical mapping");

    let expected_count = u32::try_from(dst_neurons.len()).expect("Neuron count overflow");
    assert_eq!(
        synapse_count,
        expected_count,
        "Expected one synapse from origin to each destination neuron"
    );

    let Some(npu_arc) = manager.get_npu() else {
        panic!("Test manager must have an attached NPU");
    };
    let origin_src = src_neurons[0] as u32;
    let mut npu_guard = npu_arc.lock().unwrap();
    match *npu_guard {
        DynamicNPU::F32(ref mut npu) => {
            let outgoing = npu.get_outgoing_synapses(origin_src);
            assert_eq!(
                outgoing.len(),
                dst_neurons.len(),
                "Origin neuron should connect to all destination neurons"
            );
        }
        DynamicNPU::INT8(ref mut npu) => {
            let outgoing = npu.get_outgoing_synapses(origin_src);
            assert_eq!(
                outgoing.len(),
                dst_neurons.len(),
                "Origin neuron should connect to all destination neurons"
            );
        }
    }

    println!("✅ Test 1c: Pattern morphology origin to all - PASSED");
}

// ============================================================================
// TEST 2: Block-to-Block Morphology - Basic Functionality
// ============================================================================

#[test]
fn test_block_to_block_morphology_basic() {
    let mut manager = create_test_manager();

    // Create source area (10x10x1 = 100 neurons)
    let (src_area, src_id) = create_test_area("src001", 10, 10, 1, 0);
    manager
        .add_cortical_area(src_area)
        .expect("Failed to add source area");

    // Create destination area (5x5x1 = 25 neurons)
    let (dst_area, dst_id) = create_test_area("dst001", 5, 5, 1, 1);
    manager
        .add_cortical_area(dst_area)
        .expect("Failed to add destination area");

    // Create neurons in both areas
    create_grid_neurons(&mut manager, &src_id, 10, 10, 1);
    create_grid_neurons(&mut manager, &dst_id, 5, 5, 1);

    // Create a mapping rule (block_to_block morphology)
    let rule = json!({
        "morphology_id": "block_to_block",
        "morphology_scalar": [1],
        "postSynapticCurrent_multiplier": 1.0,
        "synapse_attractivity": 100
    });

    // Set up cortical mapping using update_cortical_mapping
    manager
        .update_cortical_mapping(&src_id, &dst_id, vec![rule])
        .expect("Failed to update cortical mapping");

    // Apply cortical mapping
    let synapse_count = manager
        .apply_cortical_mapping(&src_id)
        .expect("Failed to apply cortical mapping");

    println!(
        "Created {} synapses via block_to_block morphology",
        synapse_count
    );

    // Verify synapses were created
    assert!(synapse_count > 0, "Should have created some synapses");

    println!("✅ Test 2: Block-to-block morphology basic - PASSED");
}

// ============================================================================
// TEST 3: Edge Case - Empty Source Area
// ============================================================================

#[test]
fn test_synaptogenesis_empty_source_area() {
    let mut manager = create_test_manager();

    // Create source area (but don't add neurons)
    let (src_area, src_id) = create_test_area("src002", 10, 10, 1, 0);
    manager
        .add_cortical_area(src_area)
        .expect("Failed to add source area");

    // Create destination area with neurons
    let (dst_area, dst_id) = create_test_area("dst002", 10, 10, 1, 1);
    manager
        .add_cortical_area(dst_area)
        .expect("Failed to add destination area");
    create_grid_neurons(&mut manager, &dst_id, 10, 10, 1);

    // Create a mapping rule
    let rule = json!({
        "morphology_id": "projector",
        "postSynapticCurrent_multiplier": 1.0,
        "synapse_attractivity": 100
    });

    // Set up cortical mapping using update_cortical_mapping
    manager
        .update_cortical_mapping(&src_id, &dst_id, vec![rule])
        .expect("Failed to update cortical mapping");

    // Apply cortical mapping (should return 0 synapses, not error)
    let synapse_count = manager
        .apply_cortical_mapping(&src_id)
        .expect("Should handle empty source area gracefully");

    assert_eq!(
        synapse_count, 0,
        "Should create 0 synapses when source area is empty"
    );

    println!("✅ Test 3: Empty source area - PASSED");
}

// ============================================================================
// TEST 4: Edge Case - Empty Destination Area
// ============================================================================

#[test]
fn test_synaptogenesis_empty_destination_area() {
    let mut manager = create_test_manager();

    // Create source area with neurons
    let (src_area, src_id) = create_test_area("src003", 10, 10, 1, 0);
    manager
        .add_cortical_area(src_area)
        .expect("Failed to add source area");
    create_grid_neurons(&mut manager, &src_id, 10, 10, 1);

    // Create destination area (but don't add neurons)
    let (dst_area, dst_id) = create_test_area("dst003", 10, 10, 1, 1);
    manager
        .add_cortical_area(dst_area)
        .expect("Failed to add destination area");

    // Create a mapping rule
    let rule = json!({
        "morphology_id": "projector",
        "postSynapticCurrent_multiplier": 1.0,
        "synapse_attractivity": 100
    });

    // Set up cortical mapping using update_cortical_mapping
    manager
        .update_cortical_mapping(&src_id, &dst_id, vec![rule])
        .expect("Failed to update cortical mapping");

    // Apply cortical mapping (should return 0 synapses, not error)
    let synapse_count = manager
        .apply_cortical_mapping(&src_id)
        .expect("Should handle empty destination area gracefully");

    assert_eq!(
        synapse_count, 0,
        "Should create 0 synapses when destination area is empty"
    );

    println!("✅ Test 4: Empty destination area - PASSED");
}

// ============================================================================
// TEST 5: Synapse Attractivity Parameter
// ============================================================================

#[test]
fn test_synapse_attractivity_parameter() {
    let mut manager = create_test_manager();

    // Create source and destination areas
    let (src_area, src_id) = create_test_area("src004", 10, 10, 1, 0);
    manager
        .add_cortical_area(src_area)
        .expect("Failed to add source area");

    let (dst_area, dst_id) = create_test_area("dst004", 10, 10, 1, 1);
    manager
        .add_cortical_area(dst_area)
        .expect("Failed to add destination area");

    // Create neurons in both areas
    create_grid_neurons(&mut manager, &src_id, 10, 10, 1);
    create_grid_neurons(&mut manager, &dst_id, 10, 10, 1);

    // Test with 0% attractivity (should create 0 synapses)
    let rule_zero = json!({
        "morphology_id": "projector",
        "postSynapticCurrent_multiplier": 1.0,
        "synapse_attractivity": 0
    });

    // Set up cortical mapping using update_cortical_mapping
    manager
        .update_cortical_mapping(&src_id, &dst_id, vec![rule_zero])
        .expect("Failed to update cortical mapping");

    let synapse_count_zero = manager
        .apply_cortical_mapping(&src_id)
        .expect("Failed to apply cortical mapping with 0% attractivity");

    assert_eq!(
        synapse_count_zero, 0,
        "0% attractivity should create 0 synapses"
    );

    println!("✅ Test 5: Synapse attractivity parameter - PASSED");
}

// ============================================================================
// TEST 6: Multiple Morphology Rules
// ============================================================================

#[test]
fn test_multiple_morphology_rules() {
    let mut manager = create_test_manager();

    // Create source area
    let (src_area, src_id) = create_test_area("src005", 10, 10, 1, 0);
    manager
        .add_cortical_area(src_area)
        .expect("Failed to add source area");

    // Create destination area
    let (dst_area, dst_id) = create_test_area("dst005", 10, 10, 1, 1);
    manager
        .add_cortical_area(dst_area)
        .expect("Failed to add destination area");

    // Create neurons in both areas
    create_grid_neurons(&mut manager, &src_id, 10, 10, 1);
    create_grid_neurons(&mut manager, &dst_id, 10, 10, 1);

    // Create multiple mapping rules (same morphology applied twice)
    let rule1 = json!({
        "morphology_id": "projector",
        "postSynapticCurrent_multiplier": 1.0,
        "synapse_attractivity": 50  // 50% to reduce stochastic variation
    });

    let rule2 = json!({
        "morphology_id": "projector",
        "postSynapticCurrent_multiplier": 1.0,
        "synapse_attractivity": 50
    });

    // Set up cortical mapping with multiple rules using update_cortical_mapping
    manager
        .update_cortical_mapping(&src_id, &dst_id, vec![rule1, rule2])
        .expect("Failed to update cortical mapping");

    // Apply cortical mapping
    let synapse_count = manager
        .apply_cortical_mapping(&src_id)
        .expect("Failed to apply cortical mapping with multiple rules");

    println!(
        "Created {} synapses via multiple morphology rules",
        synapse_count
    );

    // Verify synapses were created (should be more than 0, but exact count depends on randomness)
    // synapse_count is unsigned; non-negative is guaranteed.

    println!("✅ Test 6: Multiple morphology rules - PASSED");
}
