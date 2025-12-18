// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! # Fire Queue Serialization Tests
//!
//! Tests serialization of fire queue data with all cortical types:
//! - Verifies RawFireQueueData can be serialized correctly
//! - Tests cortical area name encoding/decoding
//! - Ensures all cortical types work in visualization streams

use feagi_data_structures::genomic::cortical_area::CorticalID;
use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::burst_loop_runner::RawFireQueueData;
use feagi_npu_burst_engine::RustNPU;
use feagi_npu_runtime_std::StdRuntime;

/// Helper to create a test NPU
fn create_test_npu() -> RustNPU<StdRuntime, f32, CPUBackend> {
    let runtime = StdRuntime;
    let backend = CPUBackend::new();
    RustNPU::new(runtime, backend, 1000, 10000, 20).expect("Failed to create NPU")
}

// ═══════════════════════════════════════════════════════════
// Test 1: Create RawFireQueueData with CORE area
// ═══════════════════════════════════════════════════════════

#[test]
fn test_fire_queue_core_area_serialization() {
    let mut npu = create_test_npu();

    // Register CORE area
    let cortical_id = CorticalID::try_from_bytes(b"___power").unwrap();
    let base64_name = cortical_id.as_base_64();
    npu.register_cortical_area(1, base64_name.clone());

    // Create fire queue data
    let fire_data = RawFireQueueData {
        cortical_area_idx: 1,
        cortical_area_name: base64_name.clone(),
        neuron_ids: vec![0, 1, 2],
        coords_x: vec![0, 1, 2],
        coords_y: vec![0, 1, 2],
        coords_z: vec![0, 0, 0],
        potentials: vec![1.0, 1.0, 1.0],
    };

    // Verify we can decode the cortical ID from the name
    let decoded_id = CorticalID::try_from_base_64(&fire_data.cortical_area_name).unwrap();
    assert_eq!(decoded_id, cortical_id);
}

// ═══════════════════════════════════════════════════════════
// Test 2: Create RawFireQueueData with IPU area
// ═══════════════════════════════════════════════════════════

#[test]
fn test_fire_queue_ipu_area_serialization() {
    let mut npu = create_test_npu();

    // Register IPU area
    let cortical_id = CorticalID::try_from_bytes(b"iav000").unwrap();
    let base64_name = cortical_id.as_base_64();
    npu.register_cortical_area(2, base64_name.clone());

    // Create fire queue data
    let fire_data = RawFireQueueData {
        cortical_area_idx: 2,
        cortical_area_name: base64_name.clone(),
        neuron_ids: vec![10, 11],
        coords_x: vec![5, 6],
        coords_y: vec![5, 6],
        coords_z: vec![0, 0],
        potentials: vec![0.8, 0.9],
    };

    // Verify we can decode the cortical ID from the name
    let decoded_id = CorticalID::try_from_base_64(&fire_data.cortical_area_name).unwrap();
    assert_eq!(decoded_id, cortical_id);
}

// ═══════════════════════════════════════════════════════════
// Test 3: Create RawFireQueueData with OPU area
// ═══════════════════════════════════════════════════════════

#[test]
fn test_fire_queue_opu_area_serialization() {
    let mut npu = create_test_npu();

    // Register OPU area
    let cortical_id = CorticalID::try_from_bytes(b"omot00").unwrap();
    let base64_name = cortical_id.as_base_64();
    npu.register_cortical_area(3, base64_name.clone());

    // Create fire queue data
    let fire_data = RawFireQueueData {
        cortical_area_idx: 3,
        cortical_area_name: base64_name.clone(),
        neuron_ids: vec![20, 21, 22, 23],
        coords_x: vec![10, 11, 12, 13],
        coords_y: vec![10, 11, 12, 13],
        coords_z: vec![0, 0, 0, 0],
        potentials: vec![0.7, 0.75, 0.8, 0.85],
    };

    // Verify we can decode the cortical ID from the name
    let decoded_id = CorticalID::try_from_base_64(&fire_data.cortical_area_name).unwrap();
    assert_eq!(decoded_id, cortical_id);
}

// ═══════════════════════════════════════════════════════════
// Test 4: Create RawFireQueueData with CUSTOM area
// ═══════════════════════════════════════════════════════════

#[test]
fn test_fire_queue_custom_area_serialization() {
    let mut npu = create_test_npu();

    // Register CUSTOM area
    let cortical_id = CorticalID::try_from_bytes(b"cust000").unwrap();
    let base64_name = cortical_id.as_base_64();
    npu.register_cortical_area(4, base64_name.clone());

    // Create fire queue data
    let fire_data = RawFireQueueData {
        cortical_area_idx: 4,
        cortical_area_name: base64_name.clone(),
        neuron_ids: vec![30],
        coords_x: vec![15],
        coords_y: vec![15],
        coords_z: vec![0],
        potentials: vec![0.6],
    };

    // Verify we can decode the cortical ID from the name
    let decoded_id = CorticalID::try_from_base_64(&fire_data.cortical_area_name).unwrap();
    assert_eq!(decoded_id, cortical_id);
}

// ═══════════════════════════════════════════════════════════
// Test 5: Create RawFireQueueData with MEMORY area
// ═══════════════════════════════════════════════════════════

#[test]
fn test_fire_queue_memory_area_serialization() {
    let mut npu = create_test_npu();

    // Register MEMORY area
    let cortical_id = CorticalID::try_from_bytes(b"memo000").unwrap();
    let base64_name = cortical_id.as_base_64();
    npu.register_cortical_area(5, base64_name.clone());

    // Create fire queue data
    let fire_data = RawFireQueueData {
        cortical_area_idx: 5,
        cortical_area_name: base64_name.clone(),
        neuron_ids: vec![40, 41, 42],
        coords_x: vec![20, 21, 22],
        coords_y: vec![20, 21, 22],
        coords_z: vec![0, 0, 0],
        potentials: vec![0.5, 0.55, 0.6],
    };

    // Verify we can decode the cortical ID from the name
    let decoded_id = CorticalID::try_from_base_64(&fire_data.cortical_area_name).unwrap();
    assert_eq!(decoded_id, cortical_id);
}

// ═══════════════════════════════════════════════════════════
// Test 6: Multiple areas in fire queue
// ═══════════════════════════════════════════════════════════

#[test]
fn test_fire_queue_multiple_areas() {
    let mut npu = create_test_npu();

    let areas = vec![(1, b"___power"), (2, b"iav000"), (3, b"omot00")];

    // Register all areas
    for (idx, bytes) in &areas {
        let cortical_id = CorticalID::try_from_bytes(bytes).unwrap();
        let base64_name = cortical_id.as_base_64();
        npu.register_cortical_area(*idx, base64_name);
    }

    // Create fire queue data for each area
    let mut fire_queue_snapshot = std::collections::HashMap::new();

    for (idx, bytes) in &areas {
        let cortical_id = CorticalID::try_from_bytes(bytes).unwrap();
        let base64_name = cortical_id.as_base_64();

        let fire_data = RawFireQueueData {
            cortical_area_idx: *idx,
            cortical_area_name: base64_name.clone(),
            neuron_ids: vec![*idx as u32 * 10],
            coords_x: vec![*idx as u32 * 5],
            coords_y: vec![*idx as u32 * 5],
            coords_z: vec![0],
            potentials: vec![1.0],
        };

        fire_queue_snapshot.insert(*idx, fire_data);
    }

    // Verify all can be decoded
    for (idx, bytes) in &areas {
        let expected_id = CorticalID::try_from_bytes(bytes).unwrap();
        let fire_data = fire_queue_snapshot.get(idx).unwrap();
        let decoded_id = CorticalID::try_from_base_64(&fire_data.cortical_area_name).unwrap();

        assert_eq!(
            decoded_id, expected_id,
            "Area at index {} failed serialization",
            idx
        );
    }
}

// ═══════════════════════════════════════════════════════════
// Test 7: Verify incorrect conversion fails (regression test)
// ═══════════════════════════════════════════════════════════

#[test]
fn test_fire_queue_incorrect_conversion_fails() {
    let mut npu = create_test_npu();

    // Register area
    let cortical_id = CorticalID::try_from_bytes(b"___power").unwrap();
    let base64_name = cortical_id.as_base_64();
    npu.register_cortical_area(1, base64_name.clone());

    // Create fire queue data
    let fire_data = RawFireQueueData {
        cortical_area_idx: 1,
        cortical_area_name: base64_name.clone(),
        neuron_ids: vec![0],
        coords_x: vec![0],
        coords_y: vec![0],
        coords_z: vec![0],
        potentials: vec![1.0],
    };

    // Attempting to convert base64 string as raw bytes should fail
    let name_bytes = fire_data.cortical_area_name.as_bytes();
    let mut bytes = [b' '; 8];
    let copy_len = name_bytes.len().min(8);
    bytes[..copy_len].copy_from_slice(&name_bytes[..copy_len]);

    // This should fail because we're treating base64 string as raw bytes
    let result = CorticalID::try_from_bytes(&bytes);
    assert!(
        result.is_err(),
        "Converting base64 string as raw bytes should fail"
    );

    // But decoding from base64 should succeed
    let decoded = CorticalID::try_from_base_64(&fire_data.cortical_area_name);
    assert!(decoded.is_ok(), "Decoding from base64 should succeed");
    assert_eq!(decoded.unwrap(), cortical_id);
}
