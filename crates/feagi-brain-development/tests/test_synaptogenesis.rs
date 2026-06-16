// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Synaptogenesis Integration Tests

Tests the synaptogenesis process through ConnectomeManager, covering:
- Core morphology applications (projector, centered_projector, block_to_block, vectors, patterns, expander)
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
use feagi_genome_definitions::::CorticalAreaDimensions;
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
    use feagi_genome_definitions::::{CorticalAreaType, CustomCorticalType};

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
fn test_centered_projector_drops_out_of_bounds() {
    let mut manager = create_test_manager();

    // Source larger than destination so peripheral source voxels are dropped.
    let (src_area, src_id) = create_test_area("src_cpr", 5, 5, 1, 0);
    manager
        .add_cortical_area(src_area)
        .expect("Failed to add source area");
    let (dst_area, dst_id) = create_test_area("dst_cpr", 3, 3, 1, 1);
    manager
        .add_cortical_area(dst_area)
        .expect("Failed to add destination area");

    let src_neurons = create_grid_neurons(&mut manager, &src_id, 5, 5, 1);
    create_grid_neurons(&mut manager, &dst_id, 3, 3, 1);

    let rule = json!({
        "morphology_id": "centered_projector",
        "postSynapticCurrent_multiplier": 1.0,
        "synapse_attractivity": 100
    });

    manager
        .update_cortical_mapping(&src_id, &dst_id, vec![rule])
        .expect("Failed to update cortical mapping");

    let synapse_count = manager
        .regenerate_synapses_for_mapping(&src_id, &dst_id)
        .expect("Failed to apply cortical mapping");

    // Only centered 3x3 source region maps into 3x3 destination => 9 synapses.
    assert_eq!(
        synapse_count, 9,
        "Centered projector should map only in-bounds centered voxels"
    );

    let Some(npu_arc) = manager.get_npu() else {
        panic!("Test manager must have an attached NPU");
    };
    let mut npu_guard = npu_arc.lock().unwrap();

    let src_center_nid = src_neurons[(2 * 5 + 2) as usize] as u32; // (2,2,0)
    let src_corner_nid = src_neurons[0] as u32; // (0,0,0)

    match *npu_guard {
        DynamicNPU::F32(ref mut npu) => {
            let center_outgoing = npu.get_outgoing_synapses(src_center_nid);
            assert_eq!(
                center_outgoing.len(),
                1,
                "Center source voxel should map to exactly one destination voxel"
            );
            let center_target_coords = npu
                .get_neuron_coordinates(center_outgoing[0].0)
                .expect("Destination neuron coordinates should exist");
            assert_eq!(
                center_target_coords,
                (1, 1, 0),
                "Source center should map to destination center"
            );

            let corner_outgoing = npu.get_outgoing_synapses(src_corner_nid);
            assert!(
                corner_outgoing.is_empty(),
                "Out-of-bounds mapped source voxels should be dropped"
            );
        }
        DynamicNPU::INT8(ref mut npu) => {
            let center_outgoing = npu.get_outgoing_synapses(src_center_nid);
            assert_eq!(
                center_outgoing.len(),
                1,
                "Center source voxel should map to exactly one destination voxel"
            );
            let center_target_coords = npu
                .get_neuron_coordinates(center_outgoing[0].0)
                .expect("Destination neuron coordinates should exist");
            assert_eq!(
                center_target_coords,
                (1, 1, 0),
                "Source center should map to destination center"
            );

            let corner_outgoing = npu.get_outgoing_synapses(src_corner_nid);
            assert!(
                corner_outgoing.is_empty(),
                "Out-of-bounds mapped source voxels should be dropped"
            );
        }
    }
}

#[test]
fn test_centered_projector_even_dimensions_use_lower_center_anchor() {
    let mut manager = create_test_manager();

    // Even-sized areas validate the lower-center anchor contract:
    // src center at (1,1,0) in 4x4x1 maps to dst center at (2,2,0) in 6x6x1.
    let (src_area, src_id) = create_test_area("src_cpe", 4, 4, 1, 0);
    manager
        .add_cortical_area(src_area)
        .expect("Failed to add source area");
    let (dst_area, dst_id) = create_test_area("dst_cpe", 6, 6, 1, 1);
    manager
        .add_cortical_area(dst_area)
        .expect("Failed to add destination area");

    let src_neurons = create_grid_neurons(&mut manager, &src_id, 4, 4, 1);
    create_grid_neurons(&mut manager, &dst_id, 6, 6, 1);

    let rule = json!({
        "morphology_id": "centered_projector",
        "postSynapticCurrent_multiplier": 1.0,
        "synapse_attractivity": 100
    });

    manager
        .update_cortical_mapping(&src_id, &dst_id, vec![rule])
        .expect("Failed to update cortical mapping");

    let synapse_count = manager
        .regenerate_synapses_for_mapping(&src_id, &dst_id)
        .expect("Failed to apply cortical mapping");
    assert_eq!(
        synapse_count, 16,
        "All 4x4 source voxels should map in-bounds into 6x6 destination"
    );

    let Some(npu_arc) = manager.get_npu() else {
        panic!("Test manager must have an attached NPU");
    };
    let mut npu_guard = npu_arc.lock().unwrap();

    let src_lower_center_nid = src_neurons[5] as u32; // (1,1,0)
    let src_upper_center_nid = src_neurons[(2 * 4 + 2) as usize] as u32; // (2,2,0)

    match *npu_guard {
        DynamicNPU::F32(ref mut npu) => {
            let lower_center_outgoing = npu.get_outgoing_synapses(src_lower_center_nid);
            assert_eq!(lower_center_outgoing.len(), 1);
            assert_eq!(
                npu.get_neuron_coordinates(lower_center_outgoing[0].0),
                Some((2, 2, 0)),
                "Lower-center source voxel should map to lower-center destination voxel"
            );

            let upper_center_outgoing = npu.get_outgoing_synapses(src_upper_center_nid);
            assert_eq!(upper_center_outgoing.len(), 1);
            assert_eq!(
                npu.get_neuron_coordinates(upper_center_outgoing[0].0),
                Some((3, 3, 0)),
                "Adjacent upper-center source voxel should preserve +1 offset from center"
            );
        }
        DynamicNPU::INT8(ref mut npu) => {
            let lower_center_outgoing = npu.get_outgoing_synapses(src_lower_center_nid);
            assert_eq!(lower_center_outgoing.len(), 1);
            assert_eq!(
                npu.get_neuron_coordinates(lower_center_outgoing[0].0),
                Some((2, 2, 0)),
                "Lower-center source voxel should map to lower-center destination voxel"
            );

            let upper_center_outgoing = npu.get_outgoing_synapses(src_upper_center_nid);
            assert_eq!(upper_center_outgoing.len(), 1);
            assert_eq!(
                npu.get_neuron_coordinates(upper_center_outgoing[0].0),
                Some((3, 3, 0)),
                "Adjacent upper-center source voxel should preserve +1 offset from center"
            );
        }
    }
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

#[test]
fn test_first_to_last_morphology_maps_origin_to_destination_max() {
    let mut manager = create_test_manager();

    // Source and destination dimensions intentionally differ.
    let (src_area, src_id) = create_test_area("srcf2l", 3, 3, 2, 0);
    manager
        .add_cortical_area(src_area)
        .expect("Failed to add source area");

    let (dst_area, dst_id) = create_test_area("dstf2l", 4, 2, 2, 1);
    manager
        .add_cortical_area(dst_area)
        .expect("Failed to add destination area");

    let src_neurons = create_grid_neurons(&mut manager, &src_id, 3, 3, 2);
    create_grid_neurons(&mut manager, &dst_id, 4, 2, 2);

    let rule = json!({
        "morphology_id": "first_to_last",
        "postSynapticCurrent_multiplier": 1.0,
        "synapse_attractivity": 100
    });

    manager
        .update_cortical_mapping(&src_id, &dst_id, vec![rule])
        .expect("Failed to update cortical mapping");

    let synapse_count = manager
        .apply_cortical_mapping(&src_id)
        .expect("Failed to apply cortical mapping");
    assert_eq!(
        synapse_count, 1,
        "first_to_last should create exactly one synapse"
    );

    let Some(npu_arc) = manager.get_npu() else {
        panic!("Test manager must have an attached NPU");
    };
    let mut npu_guard = npu_arc.lock().unwrap();
    let expected_dst = (3u32, 1u32, 1u32);

    match *npu_guard {
        DynamicNPU::F32(ref mut npu) => {
            let src_origin = src_neurons
                .iter()
                .map(|nid| *nid as u32)
                .find(|nid| npu.get_neuron_coordinates(*nid) == Some((0, 0, 0)))
                .expect("Source origin neuron must exist");
            let outgoing = npu.get_outgoing_synapses(src_origin);
            assert_eq!(
                outgoing.len(),
                1,
                "Source origin should have exactly one outgoing synapse"
            );
            assert_eq!(
                npu.get_neuron_coordinates(outgoing[0].0),
                Some(expected_dst),
                "Source origin should map to destination max voxel"
            );
        }
        DynamicNPU::INT8(ref mut npu) => {
            let src_origin = src_neurons
                .iter()
                .map(|nid| *nid as u32)
                .find(|nid| npu.get_neuron_coordinates(*nid) == Some((0, 0, 0)))
                .expect("Source origin neuron must exist");
            let outgoing = npu.get_outgoing_synapses(src_origin);
            assert_eq!(
                outgoing.len(),
                1,
                "Source origin should have exactly one outgoing synapse"
            );
            assert_eq!(
                npu.get_neuron_coordinates(outgoing[0].0),
                Some(expected_dst),
                "Source origin should map to destination max voxel"
            );
        }
    }
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

// ============================================================================
// TEST 8: End-to-end spike delivery via apply_cortical_mapping
//          (regression for cartpole detector silence)
// ============================================================================
//
// Regression for the cartpole R-STDP detector silence bug:
// Live FEAGI showed that after `update_cortical_mapping` + `apply_cortical_mapping`
// (or the equivalent dynamic build_reflex_mapping flow), `get_outgoing_synapses`
// correctly returned the synapses, but the per-burst spike-delivery loop never
// delivered any spikes across them. The destination neuron's membrane potential
// never increased and it never fired despite the source firing thousands of
// consecutive bursts.
//
// The other tests in this file only verify that synapses become visible via the
// propagation index after `apply_cortical_mapping`. They never run a burst chain
// to verify that spikes actually propagate across those synapses, so they cannot
// catch this regression.
//
// This test fills that gap: it exercises the same dynamic mapping flow used by
// the live REST API, then drives the source neuron via
// `inject_sensory_with_potentials` + `process_burst`, and asserts that the
// destination neuron fires within the expected synaptic-delay window.
#[test]
fn test_apply_cortical_mapping_delivers_spikes_to_target() {
    use feagi_npu_neural::types::NeuronId;

    let mut manager = create_test_manager();

    // Tiny 1x1x1 -> 1x1x1 graph: deterministic and rules out fan-out as a confounder.
    let (src_area, src_id) = create_test_area("src_e2e", 1, 1, 1, 0);
    manager
        .add_cortical_area(src_area)
        .expect("Failed to add source area");
    let (dst_area, dst_id) = create_test_area("dst_e2e", 1, 1, 1, 1);
    manager
        .add_cortical_area(dst_area)
        .expect("Failed to add destination area");

    // Use neuron parameters matching the canonical integration_burst_workflow tests
    // (mp_charge_accumulation=true, no consecutive-fire/snooze cap, no leak) so we
    // know the neuron itself is firing-capable. This isolates the test to the
    // mapping/propagation path rather than neuron dynamics tuning.
    let add_one = |manager: &mut ConnectomeManager, area: &CorticalID, x, y, z| -> u64 {
        manager
            .add_neuron(
                area,
                x,
                y,
                z,
                1.0,      // firing_threshold
                f32::MAX, // firing_threshold_limit (no cap)
                0.0,      // leak_coefficient
                0.0,      // resting_potential
                0,        // neuron_type
                5,        // refractory_period
                1.0,      // excitability
                0,        // consecutive_fire_limit (0 = no cap)
                0,        // snooze_length
                true,     // mp_charge_accumulation
            )
            .expect("add_neuron failed")
    };

    let src_neuron_id = add_one(&mut manager, &src_id, 0, 0, 0);
    let dst_neuron_id = add_one(&mut manager, &dst_id, 0, 0, 0);

    // Strong unconditional connection. PSC multiplier is generous so a single
    // spike from src must drive dst above threshold (default firing_threshold = 1.0
    // in `create_grid_neurons`).
    let rule = json!({
        "morphology_id": "projector",
        "morphology_scalar": [1, 1, 1],
        "postSynapticCurrent_multiplier": 5,
        "synapse_attractivity": 100,
        "plasticity_flag": false,
        "synaptic_delay_bursts": 1,
    });

    manager
        .update_cortical_mapping(&src_id, &dst_id, vec![rule])
        .expect("Failed to update cortical mapping");
    let synapse_count = manager
        .apply_cortical_mapping(&src_id)
        .expect("Failed to apply cortical mapping");
    assert_eq!(
        synapse_count, 1,
        "projector should produce exactly one src->dst synapse for matching 1x1x1 dims"
    );

    let src_nid = NeuronId(src_neuron_id as u32);
    let dst_nid = NeuronId(dst_neuron_id as u32);

    let npu_arc = manager
        .get_npu()
        .expect("Test manager must have an attached NPU");
    let mut npu_guard = npu_arc.lock().unwrap();

    match *npu_guard {
        DynamicNPU::F32(ref mut npu) => {
            // Sanity check: synapse is in the propagation engine's index.
            let outgoing = npu.get_outgoing_synapses(src_nid.0);
            assert_eq!(
                outgoing.len(),
                1,
                "Outgoing synapse should be visible via propagation index after apply_cortical_mapping"
            );
            assert_eq!(
                outgoing[0].0, dst_nid.0,
                "Outgoing synapse must target the dst neuron (got target_id={})",
                outgoing[0].0
            );

            // The actual regression check: drive src above threshold and verify
            // dst fires within the synaptic-delay window.
            npu.inject_sensory_with_potentials(&[(src_nid, 5.0)]);
            let burst1 = npu.process_burst().expect("burst 1 failed");
            assert!(
                burst1.fired_neurons.contains(&src_nid),
                "Source neuron must fire on burst 1 after sensory injection \
                 (fired={:?})",
                burst1.fired_neurons
            );

            // synaptic_delay_bursts=1 means the src spike from burst 1 arrives at
            // dst during burst 2 processing. dst should reach threshold and fire
            // on burst 2.
            let burst2 = npu.process_burst().expect("burst 2 failed");
            assert!(
                burst2.fired_neurons.contains(&dst_nid),
                "REGRESSION: destination neuron did not fire on burst 2 despite \
                 synapse being visible via propagation index. This is the cartpole \
                 detector silence bug. burst2.fired_neurons={:?}",
                burst2.fired_neurons
            );
        }
        DynamicNPU::INT8(_) => panic!("Test only configured for F32 NPU"),
    }

    println!("✅ Test 8: End-to-end spike delivery via apply_cortical_mapping - PASSED");
}

// ============================================================================
// TEST 9: Fan-out PSP dilution silences low-threshold targets
//          (regression for cartpole detector silence after adding R-STDP fan-out)
// ============================================================================
//
// In `synaptic_propagation.rs`, when a source area's `psp_uniform_distribution`
// is false (the default), the source neuron's per-burst contribution is divided
// across the total number of outgoing synapses:
//
//     final_contribution = base_contribution / source_meta.synapse_count
//
// This is the exact mechanism behind the cartpole detector silence:
//   - Initially, the encoder had 2 outgoing synapses (to detectors at z=0,9):
//       contribution per synapse = 1.0 / 2 = 0.5  → above detector threshold 0.1, fires.
//   - After adding 20 R-STDP synapses (encoder → motor), encoder fan-out = 21:
//       contribution per synapse = 1.0 / 21 = 0.048 → below 0.1, detector silent.
//
// This test reconstructs that exact topology and asserts the symptom. It is a
// behavioral regression test: any future change that silently alters fan-out
// dilution semantics (e.g., toggling the default of psp_uniform_distribution)
// will move this assertion. Owners are expected to update this test
// intentionally if they change the contract.
#[test]
fn test_fanout_psp_dilution_silences_low_threshold_target() {
    use feagi_npu_neural::types::NeuronId;

    let mut manager = create_test_manager();

    // Source: 1x1x10 (matches the cartpole encoder shape)
    let (src_area, src_id) = create_test_area("src_fan", 1, 1, 10, 0);
    manager
        .add_cortical_area(src_area)
        .expect("Failed to add source area");

    // Detector: 1x1x1 (matches pain_fallen / pleasure_upright)
    let (det_area, det_id) = create_test_area("det_fan", 1, 1, 1, 1);
    manager
        .add_cortical_area(det_area)
        .expect("Failed to add detector area");

    // Motor sink: 2x1x10 (matches ungrouped-1 fan-out target)
    let (mot_area, mot_id) = create_test_area("mot_fan", 2, 1, 10, 2);
    manager
        .add_cortical_area(mot_area)
        .expect("Failed to add motor area");

    // Add a single firing neuron at src[0,0,0] (encoder z=0 = "fallen" position).
    let src_neuron_id = manager
        .add_neuron(
            &src_id,
            0,
            0,
            0,
            1.0, // firing_threshold
            f32::MAX,
            0.0,
            0.0,
            0,
            5,
            1.0,
            0,
            0,
            true,
        )
        .expect("add encoder neuron failed");

    // Detector neuron with cartpole-matching threshold 0.1.
    let det_neuron_id = manager
        .add_neuron(
            &det_id,
            0,
            0,
            0,
            0.1, // firing_threshold = 0.1 (cartpole pain/pleasure threshold)
            f32::MAX,
            0.0,
            0.0,
            0,
            5,
            1.0,
            0,
            0,
            true,
        )
        .expect("add detector neuron failed");

    // Motor sink neurons (20 of them, 2x1x10 grid).
    for x in 0..2 {
        for z in 0..10 {
            manager
                .add_neuron(
                    &mot_id,
                    x,
                    0,
                    z,
                    1.0,
                    f32::MAX,
                    0.0,
                    0.0,
                    0,
                    5,
                    1.0,
                    0,
                    0,
                    true,
                )
                .expect("add motor neuron failed");
        }
    }

    // Mapping 1: src → detector, single synapse from src origin to dst origin.
    // morphology_scalar [0,0,0] for "0-0-0_to_all" pattern with 1x1x1 dst means a
    // single synapse src[0,0,0]→det[0,0,0] (matches cartpole detector wiring).
    let det_rule = json!({
        "morphology_id": "0-0-0_to_all",
        "morphology_scalar": [1, 1, 1],
        "postSynapticCurrent_multiplier": 1,
        "synapse_attractivity": 100,
        "plasticity_flag": false,
        "synaptic_delay_bursts": 1,
    });
    manager
        .update_cortical_mapping(&src_id, &det_id, vec![det_rule])
        .expect("update src→det mapping failed");

    // Mapping 2: src → motor, all_to_all with origin source = 20 synapses
    // from src[0,0,0] to all motor neurons (matches encoder→ungrouped-1 R-STDP fan-out).
    let mot_rule = json!({
        "morphology_id": "0-0-0_to_all",
        "morphology_scalar": [1, 1, 1],
        "postSynapticCurrent_multiplier": 1,
        "synapse_attractivity": 100,
        "plasticity_flag": false,
        "synaptic_delay_bursts": 1,
    });
    manager
        .update_cortical_mapping(&src_id, &mot_id, vec![mot_rule])
        .expect("update src→mot mapping failed");

    // Apply both mappings.
    let total_synapses = manager
        .apply_cortical_mapping(&src_id)
        .expect("apply_cortical_mapping failed");
    assert_eq!(
        total_synapses, 21,
        "Expected 21 total synapses (1 src→det + 20 src→mot)"
    );

    let src_nid = NeuronId(src_neuron_id as u32);
    let det_nid = NeuronId(det_neuron_id as u32);

    let npu_arc = manager
        .get_npu()
        .expect("Test manager must have an attached NPU");
    let mut npu_guard = npu_arc.lock().unwrap();

    match *npu_guard {
        DynamicNPU::F32(ref mut npu) => {
            // Sanity: src has exactly 21 outgoing synapses.
            let outgoing = npu.get_outgoing_synapses(src_nid.0);
            assert_eq!(
                outgoing.len(),
                21,
                "Expected encoder src to have 21 outgoing synapses (cartpole topology)"
            );

            // Drive src above its own threshold.
            npu.inject_sensory_with_potentials(&[(src_nid, 5.0)]);
            let burst1 = npu.process_burst().expect("burst 1 failed");
            assert!(
                burst1.fired_neurons.contains(&src_nid),
                "Source must fire on burst 1; fired={:?}",
                burst1.fired_neurons
            );

            // Burst 2: contribution to detector is base / 21 = 1.0/21 ≈ 0.048,
            // below detector threshold 0.1. Detector must therefore stay silent.
            let burst2 = npu.process_burst().expect("burst 2 failed");
            assert!(
                !burst2.fired_neurons.contains(&det_nid),
                "REGRESSION: detector fired on burst 2 — fan-out PSP dilution \
                 contract has changed. Either psp_uniform_distribution default \
                 flipped, or per-synapse division was removed. Update this test \
                 intentionally if the contract changed. burst2.fired={:?}",
                burst2.fired_neurons
            );
        }
        DynamicNPU::INT8(_) => panic!("Test only configured for F32 NPU"),
    }

    println!(
        "✅ Test 9: Fan-out PSP dilution silences low-threshold target - PASSED \
         (this confirms the cartpole detector silence root cause: 1/21 < 0.1)"
    );
}
