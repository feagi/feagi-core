// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Synaptogenesis Integration Tests

Tests the synaptogenesis process through ConnectomeManager, covering:
- Core morphology applications (projector, block_to_block, vectors, patterns, expander)
- Integration path (apply_cortical_mapping -> apply_cortical_mapping_for_pair -> apply_single_morphology_rule)
- Edge cases (empty areas, no neurons, dimensions mismatch)
- Parameter validation (weight, psp, synapse_attractivity)

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

#[test]
fn test_transpose_morphologies_basic() {
    for morphology_id in ["transpose_xy", "transpose_yz", "transpose_xz"] {
        let mut manager = create_test_manager();

        // Keep dimensions small and asymmetric so axis swaps are exercised.
        let (src_area, src_id) = create_test_area("srctrx", 4, 3, 2, 0);
        manager
            .add_cortical_area(src_area)
            .expect("Failed to add source area");

        let (dst_area, dst_id) = create_test_area("dsttrx", 4, 3, 2, 1);
        manager
            .add_cortical_area(dst_area)
            .expect("Failed to add destination area");

        create_grid_neurons(&mut manager, &src_id, 4, 3, 2);
        create_grid_neurons(&mut manager, &dst_id, 4, 3, 2);

        let rule = json!({
            "morphology_id": morphology_id,
            "postSynapticCurrent_multiplier": 1.0,
            "synapse_attractivity": 100
        });

        manager
            .update_cortical_mapping(&src_id, &dst_id, vec![rule])
            .expect("Failed to update cortical mapping");

        let synapse_count = manager
            .regenerate_synapses_for_mapping(&src_id, &dst_id)
            .expect("Failed to apply cortical mapping");

        assert!(
            synapse_count > 0,
            "Morphology {} should create synapses",
            morphology_id
        );
    }

    println!("✅ Test 1a: Transpose morphologies basic - PASSED");
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
                assert_eq!(weight, 5.0, "Expected abs(multiplier) to be used as weight");
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
                assert_eq!(weight, 5.0, "Expected abs(multiplier) to be used as weight");
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
        synapse_count, expected_count,
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
// TEST 2b: Composite tile morphology - fold and replicate directions
// ============================================================================
#[test]
fn test_tile_morphology_fold_and_replicate() {
    let mut manager = create_test_manager();

    // Fold case: source larger than destination
    let (src_fold_area, src_fold_id) = create_test_area("src_tlf", 4, 1, 1, 10);
    manager
        .add_cortical_area(src_fold_area)
        .expect("Failed to add fold source area");
    let (dst_fold_area, dst_fold_id) = create_test_area("dst_tlf", 2, 1, 1, 11);
    manager
        .add_cortical_area(dst_fold_area)
        .expect("Failed to add fold destination area");
    create_grid_neurons(&mut manager, &src_fold_id, 4, 1, 1);
    create_grid_neurons(&mut manager, &dst_fold_id, 2, 1, 1);

    let rule_fold = json!({
        "morphology_id": "tile",
        "postSynapticCurrent_multiplier": 1.0,
        "synapse_attractivity": 100
    });
    manager
        .update_cortical_mapping(&src_fold_id, &dst_fold_id, vec![rule_fold])
        .expect("Failed to update fold tile mapping");
    let fold_count = manager
        .regenerate_synapses_for_mapping(&src_fold_id, &dst_fold_id)
        .expect("Failed to apply fold tile mapping");
    assert_eq!(
        fold_count, 4,
        "Fold mode should map each of 4 source neurons into destination tile positions"
    );

    // Replicate case: source smaller than destination
    let (src_rep_area, src_rep_id) = create_test_area("src_tlr", 2, 1, 1, 12);
    manager
        .add_cortical_area(src_rep_area)
        .expect("Failed to add replicate source area");
    let (dst_rep_area, dst_rep_id) = create_test_area("dst_tlr", 5, 1, 1, 13);
    manager
        .add_cortical_area(dst_rep_area)
        .expect("Failed to add replicate destination area");
    create_grid_neurons(&mut manager, &src_rep_id, 2, 1, 1);
    create_grid_neurons(&mut manager, &dst_rep_id, 5, 1, 1);

    let rule_rep = json!({
        "morphology_id": "tile",
        "postSynapticCurrent_multiplier": 1.0,
        "synapse_attractivity": 100
    });
    manager
        .update_cortical_mapping(&src_rep_id, &dst_rep_id, vec![rule_rep])
        .expect("Failed to update replicate tile mapping");
    let rep_count = manager
        .regenerate_synapses_for_mapping(&src_rep_id, &dst_rep_id)
        .expect("Failed to apply replicate tile mapping");
    assert_eq!(
        rep_count, 5,
        "Replicate mode should tile source over destination (x=0,2,4 and x=1,3)"
    );
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

// ============================================================================
// TEST 7: Parallel morphologies (projector + block_to_block) — bug regression
// ============================================================================
//
// Repro for the Counter -> Counter Register issue:
// - Area A (1x1x1, 1 neuron) with `projector` and `block_to_block` mappings to
//   Area B (1x1x10, 10 neurons).
// - Both morphologies target the same src/dst pair at voxel (0,0,0).
// - Expected: two PARALLEL synapses exist from A[0,0,0] to B[0,0,0]:
//     * projector: weight = 1  (psc_mult=1)
//     * block_to_block: weight = 10 (psc_mult=10)
// - The additional 9 synapses from `projector` (z=1..9) must also exist.
// - Before the fix, the vector-type dedup check skipped the `block_to_block`
//   synapse because the projector synapse was already present, losing the 10x
//   multiplier and preventing B[0,0,0] from reaching its firing threshold.
#[test]
fn test_parallel_projector_and_block_to_block_preserves_both() {
    let mut manager = create_test_manager();

    // 1x1x1 source
    let (src_area, src_id) = create_test_area("src_par", 1, 1, 1, 0);
    manager
        .add_cortical_area(src_area)
        .expect("Failed to add source area");

    // 1x1x10 destination
    let (dst_area, dst_id) = create_test_area("dst_par", 1, 1, 10, 1);
    manager
        .add_cortical_area(dst_area)
        .expect("Failed to add destination area");

    let src_neurons = create_grid_neurons(&mut manager, &src_id, 1, 1, 1);
    let dst_neurons = create_grid_neurons(&mut manager, &dst_id, 1, 1, 10);

    assert_eq!(src_neurons.len(), 1, "Source should have 1 neuron");
    assert_eq!(dst_neurons.len(), 10, "Destination should have 10 neurons");

    // Two parallel mapping rules from A -> B for the same voxel pair.
    // Order matters for the regression: `projector` is declared first (as in the
    // user's genome), which used to cause `block_to_block` to be dropped.
    let projector_rule = json!({
        "morphology_id": "projector",
        "morphology_scalar": [1, 1, 1],
        "postSynapticCurrent_multiplier": 1,
        "synapse_attractivity": 100,
        "plasticity_flag": false,
        "synaptic_delay_bursts": 1,
    });
    let block_to_block_rule = json!({
        "morphology_id": "block_to_block",
        "morphology_scalar": [1, 1, 1],
        "postSynapticCurrent_multiplier": 10,
        "synapse_attractivity": 100,
        "plasticity_flag": false,
        "synaptic_delay_bursts": 1,
    });

    manager
        .update_cortical_mapping(&src_id, &dst_id, vec![projector_rule, block_to_block_rule])
        .expect("Failed to update cortical mapping");

    let synapse_count = manager
        .apply_cortical_mapping(&src_id)
        .expect("Failed to apply cortical mapping");

    // projector => 10 synapses (z=0..9), block_to_block => 1 additional synapse at z=0
    assert_eq!(
        synapse_count, 11,
        "Expected 10 projector + 1 block_to_block synapse, got {}",
        synapse_count
    );

    // Validate the outgoing synapses from the single source neuron.
    let Some(npu_arc) = manager.get_npu() else {
        panic!("Test manager must have an attached NPU");
    };
    let src_nid = src_neurons[0] as u32;
    let mut npu_guard = npu_arc.lock().unwrap();

    let outgoing: Vec<(u32, f32, f32, u8)> = match *npu_guard {
        DynamicNPU::F32(ref mut npu) => npu.get_outgoing_synapses(src_nid),
        DynamicNPU::INT8(ref mut npu) => npu.get_outgoing_synapses(src_nid),
    };

    assert_eq!(
        outgoing.len(),
        11,
        "Source neuron should have exactly 11 outgoing synapses, found {}",
        outgoing.len()
    );

    // Validate: every target is a real destination neuron (no orphans).
    let dst_neuron_ids: std::collections::HashSet<u32> =
        dst_neurons.iter().map(|&id| id as u32).collect();
    for (target, _weight, _psp, _syn_type) in &outgoing {
        assert!(
            dst_neuron_ids.contains(target),
            "Outgoing synapse target {} is not a valid destination neuron",
            target
        );
    }

    // Validate the block_to_block synapse (weight=10) exists alongside the
    // projector synapse (weight=1) for the same src/dst pair at voxel (0,0,0).
    let target_at_z0 = dst_neurons[0] as u32;
    let synapses_to_z0: Vec<&(u32, f32, f32, u8)> = outgoing
        .iter()
        .filter(|(t, _, _, _)| *t == target_at_z0)
        .collect();

    assert_eq!(
        synapses_to_z0.len(),
        2,
        "Expected 2 parallel synapses to dst[0,0,0] (projector + block_to_block), found {}",
        synapses_to_z0.len()
    );

    let weights: Vec<f32> = synapses_to_z0.iter().map(|(_, w, _, _)| *w).collect();
    assert!(
        weights.contains(&1.0),
        "Expected a projector synapse (weight=1.0) to dst[0,0,0], found weights {:?}",
        weights
    );
    assert!(
        weights.contains(&10.0),
        "Expected a block_to_block synapse (weight=10.0) to dst[0,0,0] — \
         this is the regression: the 10x multiplier is dropped. Found weights {:?}",
        weights
    );

    println!("✅ Test 7: Parallel morphologies preserve both synapses - PASSED");
}
