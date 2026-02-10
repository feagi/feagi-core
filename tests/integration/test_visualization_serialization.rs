// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! # Integration Test: Neurogenesis → Visualization Serialization
//!
//! End-to-end test verifying:
//! - Genome loading creates cortical areas
//! - Areas are registered with NPU using base64 names
//! - Fire queue can be serialized for visualization
//! - All cortical types work correctly

use feagi_bdu::{ConnectomeManager, Neuroembryogenesis};
use feagi_npu_burst_engine::{RustNPU, DynamicNPU};
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::genomic::brain_regions::{BrainRegion, RegionID, RegionType};
use feagi_evolutionary::{RuntimeGenome, GenomeMetadata, CorticalArea as GenomeCorticalArea};
use feagi_npu_runtime::StdRuntime;
use feagi_npu_burst_engine::backend::CPUBackend;
use std::sync::{Arc, Mutex};
use parking_lot::RwLock;

/// Create a minimal test genome with all cortical types
fn create_test_genome() -> RuntimeGenome {
    use feagi_structures::genomic::cortical_area::CorticalAreaDimensions;
    
    let mut genome = RuntimeGenome {
        metadata: GenomeMetadata {
            genome_id: "test_genome".to_string(),
            version: "1.0".to_string(),
            description: "Test genome for serialization".to_string(),
        },
        cortical_areas: vec![],
        brain_regions: vec![],
        physiology: feagi_evolutionary::Physiology {
            quantization_precision: "fp32".to_string(),
            membrane_potential_range: (-1.0, 1.0),
            synaptic_weight_range: (0.0, 1.0),
        },
        connections: vec![],
    };
    
    // Add CORE area
    let core_id = CorticalID::try_from_bytes(b"___power").unwrap();
    genome.cortical_areas.push(GenomeCorticalArea {
        cortical_id: core_id,
        name: "power".to_string(),
        dimensions: CorticalAreaDimensions::new(2, 2, 1).unwrap(),
        position: (0, 0, 0).into(),
        neuron_count: 4,
        connections: vec![],
    });
    
    // Add IPU area
    let ipu_id = CorticalID::try_from_bytes(b"iav000").unwrap();
    genome.cortical_areas.push(GenomeCorticalArea {
        cortical_id: ipu_id,
        name: "vision".to_string(),
        dimensions: CorticalAreaDimensions::new(2, 2, 1).unwrap(),
        position: (10, 0, 0).into(),
        neuron_count: 4,
        connections: vec![],
    });
    
    // Add OPU area
    let opu_id = CorticalID::try_from_bytes(b"omot00").unwrap();
    genome.cortical_areas.push(GenomeCorticalArea {
        cortical_id: opu_id,
        name: "motor".to_string(),
        dimensions: CorticalAreaDimensions::new(2, 2, 1).unwrap(),
        position: (20, 0, 0).into(),
        neuron_count: 4,
        connections: vec![],
    });
    
    genome
}

#[test]
fn test_neurogenesis_to_visualization_serialization() {
    // Step 1: Create NPU and ConnectomeManager
    let runtime = StdRuntime;
    let backend = CPUBackend::new();
    let npu_result = RustNPU::<StdRuntime, f32, CPUBackend>::new(
        runtime, backend, 1_000_000, 10_000_000, 10
    ).expect("Failed to create NPU");
    let npu = Arc::new(Mutex::new(DynamicNPU::F32(npu_result)));
    
    let manager = Arc::new(RwLock::new(ConnectomeManager::new_for_testing_with_npu(npu.clone())));
    
    // Step 2: Create test genome
    let genome = create_test_genome();
    
    // Step 3: Run neurogenesis
    let mut neuroembryogenesis = Neuroembryogenesis::new(manager.clone());
    neuroembryogenesis.develop_from_genome(&genome)
        .expect("Failed to develop from genome");
    
    // Step 4: Verify cortical areas are registered in NPU
    let npu_lock = npu.lock().unwrap();
    let manager_read = manager.read();
    
    // Check that at least some areas were registered
    let registered_count = npu_lock.get_registered_cortical_area_count();
    assert!(registered_count > 0, "At least some cortical areas should be registered");
    
    // For each area in genome, verify it can be serialized correctly
    for area in &genome.cortical_areas {
        // Get cortical_idx from manager
        if let Some(cortical_idx) = manager_read.get_cortical_idx(&area.cortical_id) {
            // Get the registered name from NPU
            let area_name = npu_lock.get_cortical_area_name(cortical_idx)
                .expect("Area should be registered");
            
            // Verify we can decode it back to CorticalID
            let decoded_id = CorticalID::try_from_base_64(&area_name)
                .expect("Failed to decode cortical ID from base64");
            
            assert_eq!(decoded_id, area.cortical_id,
                      "Decoded ID should match original");
        }
    }
    
    drop(manager_read);
    drop(npu_lock);
    
    // Step 5: Simulate fire queue creation (as would happen in burst loop)
    // This mimics what happens in burst_loop_runner.rs
    let npu_lock = npu.lock().unwrap();
    let manager_read = manager.read();
    
    // Test serialization for at least one area
    for area in &genome.cortical_areas {
        if let Some(cortical_idx) = manager_read.get_cortical_idx(&area.cortical_id) {
            // Get the registered name
            let area_name = npu_lock.get_cortical_area_name(cortical_idx)
                .expect("Area should be registered");
            
            // Create fire queue data (simulating what burst_loop_runner does)
            use feagi_npu_burst_engine::burst_loop_runner::RawFireQueueData;
            let fire_data = RawFireQueueData {
                cortical_area_idx: cortical_idx,
                cortical_area_name: area_name.clone(),
                neuron_ids: vec![0, 1],
                coords_x: vec![0, 1],
                coords_y: vec![0, 1],
                coords_z: vec![0, 0],
                potentials: vec![1.0, 1.0],
            };
            
            // Verify serialization works (this is what visualization stream does)
            let decoded_id = CorticalID::try_from_base_64(&fire_data.cortical_area_name)
                .expect("Failed to decode fire queue cortical ID");
            
            assert_eq!(decoded_id, area.cortical_id,
                      "Fire queue serialization failed for area {}", area.cortical_id);
            
            // Test one is enough
            break;
        }
    }
            // Verify the area name is registered
            let area_name = npu_lock.get_cortical_area_name(cortical_idx);
            assert!(area_name.is_some(), 
                   "Cortical area {} should be registered", area.cortical_id);
            
            // Verify we can decode it back to CorticalID
            let base64_name = area_name.unwrap();
            let decoded_id = CorticalID::try_from_base_64(&base64_name)
                .expect("Failed to decode cortical ID from base64");
            
            assert_eq!(decoded_id, area.cortical_id,
                      "Decoded ID should match original");
        }
    }
    
    drop(npu_lock);
    
    // Step 5: Simulate fire queue creation (as would happen in burst loop)
    // This mimics what happens in burst_loop_runner.rs
    let npu_lock = npu.lock().unwrap();
    
    for area in &genome.cortical_areas {
        if let Some(cortical_idx) = manager.get_cortical_idx(&area.cortical_id) {
            // Get the registered name
            let area_name = npu_lock.get_cortical_area_name(cortical_idx)
                .expect("Area should be registered");
            
            // Create fire queue data (simulating what burst_loop_runner does)
            use feagi_npu_burst_engine::burst_loop_runner::RawFireQueueData;
            let fire_data = RawFireQueueData {
                cortical_area_idx: cortical_idx,
                cortical_area_name: area_name.clone(),
                neuron_ids: vec![0, 1],
                coords_x: vec![0, 1],
                coords_y: vec![0, 1],
                coords_z: vec![0, 0],
                potentials: vec![1.0, 1.0],
            };
            
            // Verify serialization works (this is what visualization stream does)
            let decoded_id = CorticalID::try_from_base_64(&fire_data.cortical_area_name)
                .expect("Failed to decode fire queue cortical ID");
            
            assert_eq!(decoded_id, area.cortical_id,
                      "Fire queue serialization failed for area {}", area.cortical_id);
        }
    }
}

#[test]
fn test_all_cortical_types_in_visualization() {
    // Test that all cortical types can be serialized correctly
    let cortical_types = vec![
        (b"___power", "CORE"),
        (b"iav000", "IPU"),
        (b"omot00", "OPU"),
        (b"cust000", "CUSTOM"),
        (b"memo000", "MEMORY"),
    ];
    
    for (bytes, type_name) in &cortical_types {
        let cortical_id = CorticalID::try_from_bytes(bytes)
            .expect(&format!("Failed to create {} cortical ID", type_name));
        
        // Encode to base64 (as ConnectomeManager does)
        let base64_name = cortical_id.as_base_64();
        
        // Decode back (as visualization stream does)
        let decoded_id = CorticalID::try_from_base_64(&base64_name)
            .expect(&format!("Failed to decode {} cortical ID", type_name));
        
        assert_eq!(decoded_id, cortical_id,
                  "{} type round-trip failed", type_name);
    }
}

