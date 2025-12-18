// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! # Cortical Area Registration & Serialization Tests
//!
//! Tests the round-trip serialization of cortical areas:
//! - Registration with base64-encoded names
//! - Retrieval and decoding back to CorticalID
//! - All cortical types (CORE, IPU, OPU, CUSTOM, MEMORY)

use feagi_npu_burst_engine::RustNPU;
use feagi_data_structures::genomic::cortical_area::CorticalID;
use feagi_npu_runtime_std::StdRuntime;
use feagi_npu_burst_engine::backend::CPUBackend;

/// Helper to create a test NPU
fn create_test_npu() -> RustNPU<StdRuntime, f32, CPUBackend> {
    let runtime = StdRuntime;
    let backend = CPUBackend::new();
    RustNPU::new(runtime, backend, 1000, 10000, 20)
        .expect("Failed to create NPU")
}

// ═══════════════════════════════════════════════════════════
// Test 1: Round-trip serialization for CORE area
// ═══════════════════════════════════════════════════════════

#[test]
fn test_core_area_roundtrip() {
    let mut npu = create_test_npu();
    
    // Create CORE cortical ID (starts with '_')
    let cortical_id = CorticalID::try_from_bytes(b"___power").unwrap();
    let base64_name = cortical_id.as_base_64();
    
    // Register with NPU using cortical_idx = 1
    npu.register_cortical_area(1, base64_name.clone());
    
    // Retrieve and decode back
    let retrieved = npu.get_cortical_area_name(1).unwrap();
    let decoded_id = CorticalID::try_from_base_64(&retrieved).unwrap();
    
    assert_eq!(decoded_id, cortical_id, "CORE area round-trip failed");
    assert_eq!(retrieved, base64_name, "Base64 encoding should match");
}

// ═══════════════════════════════════════════════════════════
// Test 2: Round-trip serialization for IPU area
// ═══════════════════════════════════════════════════════════

#[test]
fn test_ipu_area_roundtrip() {
    let mut npu = create_test_npu();
    
    // Create IPU cortical ID (starts with 'i')
    let cortical_id = CorticalID::try_from_bytes(b"iav000").unwrap();
    let base64_name = cortical_id.as_base_64();
    
    // Register with NPU using cortical_idx = 2
    npu.register_cortical_area(2, base64_name.clone());
    
    // Retrieve and decode back
    let retrieved = npu.get_cortical_area_name(2).unwrap();
    let decoded_id = CorticalID::try_from_base_64(&retrieved).unwrap();
    
    assert_eq!(decoded_id, cortical_id, "IPU area round-trip failed");
}

// ═══════════════════════════════════════════════════════════
// Test 3: Round-trip serialization for OPU area
// ═══════════════════════════════════════════════════════════

#[test]
fn test_opu_area_roundtrip() {
    let mut npu = create_test_npu();
    
    // Create OPU cortical ID (starts with 'o')
    let cortical_id = CorticalID::try_from_bytes(b"omot00").unwrap();
    let base64_name = cortical_id.as_base_64();
    
    // Register with NPU using cortical_idx = 3
    npu.register_cortical_area(3, base64_name.clone());
    
    // Retrieve and decode back
    let retrieved = npu.get_cortical_area_name(3).unwrap();
    let decoded_id = CorticalID::try_from_base_64(&retrieved).unwrap();
    
    assert_eq!(decoded_id, cortical_id, "OPU area round-trip failed");
}

// ═══════════════════════════════════════════════════════════
// Test 4: Round-trip serialization for CUSTOM area
// ═══════════════════════════════════════════════════════════

#[test]
fn test_custom_area_roundtrip() {
    let mut npu = create_test_npu();
    
    // Create CUSTOM cortical ID (starts with 'c')
    let cortical_id = CorticalID::try_from_bytes(b"cust000").unwrap();
    let base64_name = cortical_id.as_base_64();
    
    // Register with NPU using cortical_idx = 4
    npu.register_cortical_area(4, base64_name.clone());
    
    // Retrieve and decode back
    let retrieved = npu.get_cortical_area_name(4).unwrap();
    let decoded_id = CorticalID::try_from_base_64(&retrieved).unwrap();
    
    assert_eq!(decoded_id, cortical_id, "CUSTOM area round-trip failed");
}

// ═══════════════════════════════════════════════════════════
// Test 5: Round-trip serialization for MEMORY area
// ═══════════════════════════════════════════════════════════

#[test]
fn test_memory_area_roundtrip() {
    let mut npu = create_test_npu();
    
    // Create MEMORY cortical ID (starts with 'm')
    let cortical_id = CorticalID::try_from_bytes(b"memo000").unwrap();
    let base64_name = cortical_id.as_base_64();
    
    // Register with NPU using cortical_idx = 5
    npu.register_cortical_area(5, base64_name.clone());
    
    // Retrieve and decode back
    let retrieved = npu.get_cortical_area_name(5).unwrap();
    let decoded_id = CorticalID::try_from_base_64(&retrieved).unwrap();
    
    assert_eq!(decoded_id, cortical_id, "MEMORY area round-trip failed");
}

// ═══════════════════════════════════════════════════════════
// Test 6: Multiple areas registered simultaneously
// ═══════════════════════════════════════════════════════════

#[test]
fn test_multiple_areas_registration() {
    let mut npu = create_test_npu();
    
    let areas = vec![
        (1, b"___power"),
        (2, b"iav000"),
        (3, b"omot00"),
        (4, b"cust000"),
        (5, b"memo000"),
    ];
    
    // Register all areas
    for (idx, bytes) in &areas {
        let cortical_id = CorticalID::try_from_bytes(bytes).unwrap();
        let base64_name = cortical_id.as_base_64();
        npu.register_cortical_area(*idx, base64_name);
    }
    
    // Verify all can be retrieved and decoded
    for (idx, bytes) in &areas {
        let expected_id = CorticalID::try_from_bytes(bytes).unwrap();
        let retrieved = npu.get_cortical_area_name(*idx).unwrap();
        let decoded_id = CorticalID::try_from_base_64(&retrieved).unwrap();
        
        assert_eq!(decoded_id, expected_id, 
                   "Area at index {} failed round-trip", idx);
    }
}

// ═══════════════════════════════════════════════════════════
// Test 7: Verify incorrect conversion fails (regression test)
// ═══════════════════════════════════════════════════════════

#[test]
fn test_incorrect_bytes_conversion_fails() {
    let mut npu = create_test_npu();
    
    // Register with base64 name
    let cortical_id = CorticalID::try_from_bytes(b"___power").unwrap();
    let base64_name = cortical_id.as_base_64();
    npu.register_cortical_area(1, base64_name.clone());
    
    // Retrieve the base64 string
    let retrieved = npu.get_cortical_area_name(1).unwrap();
    
    // Attempting to convert base64 string as raw bytes should fail
    let name_bytes = retrieved.as_bytes();
    let mut bytes = [b' '; 8];
    let copy_len = name_bytes.len().min(8);
    bytes[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
    
    // This should fail because we're treating base64 string as raw bytes
    let result = CorticalID::try_from_bytes(&bytes);
    assert!(result.is_err(), 
            "Converting base64 string as raw bytes should fail");
    
    // But decoding from base64 should succeed
    let decoded = CorticalID::try_from_base_64(&retrieved);
    assert!(decoded.is_ok(), 
            "Decoding from base64 should succeed");
    assert_eq!(decoded.unwrap(), cortical_id);
}

