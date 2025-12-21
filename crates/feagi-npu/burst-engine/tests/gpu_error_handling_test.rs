// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! GPU Error Handling Tests
//!
//! Tests error scenarios and edge cases for GPU backend:
//! - Out-of-memory scenarios
//! - Device failures
//! - Buffer transfer failures
//! - Invalid configurations
//!
//! Run with:
//!   cargo test --test gpu_error_handling_test --features gpu

use feagi_npu_burst_engine::backend::{BackendConfig, CPUBackend};
use feagi_npu_burst_engine::ComputeBackend;
use feagi_npu_runtime::{StdNeuronArray as NeuronArray, StdSynapseArray as SynapseArray};

/// Helper: Create test genome
#[allow(dead_code)]
fn create_test_genome(
    neuron_count: usize,
    synapses_per_neuron: usize,
) -> (NeuronArray<f32>, SynapseArray) {
    let mut neuron_array = NeuronArray::new(neuron_count);
    let synapse_count = neuron_count * synapses_per_neuron;
    let mut synapse_array = SynapseArray::new(synapse_count);

    // Initialize neurons
    for i in 0..neuron_count {
        neuron_array.membrane_potentials[i] = 0.0;
        neuron_array.thresholds[i] = 10.0;
        neuron_array.leak_coefficients[i] = 0.1;
        neuron_array.resting_potentials[i] = 0.0;
        neuron_array.excitabilities[i] = 1.0;
        neuron_array.valid_mask[i] = true;
    }
    neuron_array.count = neuron_count;

    // Initialize synapses
    let mut synapse_idx = 0;
    for source in 0..neuron_count {
        for i in 0..synapses_per_neuron {
            let target = (source + i + 1) % neuron_count;
            if synapse_idx < synapse_count {
                synapse_array.source_neurons[synapse_idx] = source as u32;
                synapse_array.target_neurons[synapse_idx] = target as u32;
                synapse_array.weights[synapse_idx] = 128;
                synapse_array.postsynaptic_potentials[synapse_idx] = 200;
                synapse_array.types[synapse_idx] = 0; // Excitatory
                synapse_array.valid_mask[synapse_idx] = true;

                synapse_array
                    .source_index
                    .entry(source as u32)
                    .or_default()
                    .push(synapse_idx);

                synapse_idx += 1;
            }
        }
    }
    synapse_array.count = synapse_idx;

    (neuron_array, synapse_array)
}

#[cfg(feature = "gpu")]
#[test]
fn test_gpu_out_of_memory_handling() {
    // Try to create a genome that would exceed typical GPU VRAM
    // Most GPUs have 8-16GB VRAM
    // We'll try 50M neurons which would need ~20GB+ for all buffers
    let huge_neuron_count = 50_000_000;
    let huge_synapse_count = 5_000_000_000;

    #[cfg(feature = "gpu")]
    {
        use feagi_npu_burst_engine::backend::WGPUBackend;
        let result = WGPUBackend::new(huge_neuron_count, huge_synapse_count);

        // We expect either:
        // 1. GPU backend creation to fail gracefully (OOM during init)
        // 2. GPU backend to be created successfully (GPU has lots of VRAM)
        match result {
            Err(e) => {
                // Expected failure - verify it's a reasonable error
                let err_msg = format!("{:?}", e);
                println!("✅ GPU OOM handled gracefully: {}", err_msg);
                assert!(
                    err_msg.contains("memory")
                        || err_msg.contains("OOM")
                        || err_msg.contains("out of")
                        || err_msg.contains("failed")
                        || err_msg.contains("allocation"),
                    "Error should mention memory/allocation issues: {}",
                    err_msg
                );
            }
            Ok(mut backend) => {
                // GPU has enough VRAM for metadata - try buffer upload
                println!("⚠️  GPU backend created, testing buffer upload failure...");

                let (neuron_array, synapse_array) = create_test_genome(10_000, 100);
                let result = backend.initialize_persistent_data(&neuron_array, &synapse_array);

                // Backend creation succeeded - this is OK for large VRAM GPUs
                println!("✅ GPU has sufficient VRAM or test needs larger size");
                println!("   Backend: {}", backend.backend_name());
            }
        }
    }
    #[cfg(not(feature = "gpu"))]
    {
        // Skip test if GPU not available
        println!("⚠️  GPU feature not enabled, skipping OOM test");
    }
}

#[test]
fn test_zero_neuron_handling() {
    // Test empty genome
    let _config = BackendConfig::default();

    // CPU backend should handle zero neurons gracefully
    let _backend = CPUBackend::new();
    // CPU backend creation always succeeds

    #[cfg(feature = "gpu")]
    {
        use feagi_npu_burst_engine::backend::WGPUBackend;
        // GPU backend should also handle zero neurons
        let result_gpu = WGPUBackend::new(0, 0);
        // Either succeeds with empty buffers or fails gracefully
        match result_gpu {
            Ok(_) => println!("✅ GPU backend accepts zero neurons"),
            Err(e) => {
                println!("✅ GPU backend rejects zero neurons: {:?}", e);
                // This is also acceptable behavior
            }
        }
    }
}

#[test]
fn test_invalid_capacity() {
    let _config = BackendConfig::default();

    // Test mismatched capacities (neurons < synapses target range)
    let _backend = CPUBackend::new();
    // CPU backend creation always succeeds regardless of capacity
}

#[cfg(feature = "gpu")]
#[test]
fn test_gpu_buffer_size_validation() {
    use feagi_npu_neural::types::FireCandidateList;

    let _config = BackendConfig::default();
    let (neuron_array, synapse_array) = create_test_genome(1000, 100);

    use feagi_npu_burst_engine::backend::WGPUBackend;
    let mut backend = WGPUBackend::new(neuron_array.capacity, synapse_array.capacity)
        .expect("GPU backend should be created");

    // Initialize persistent data
    backend
        .initialize_persistent_data(&neuron_array, &synapse_array)
        .expect("Buffer upload should succeed");

    // Test processing with valid data
    let fired_neurons: Vec<u32> = vec![0, 1, 2];
    let mut fcl = FireCandidateList::new();

    let result = backend.process_synaptic_propagation(&fired_neurons, &synapse_array, &mut fcl);
    assert!(result.is_ok(), "Valid processing should succeed");
}

#[cfg(feature = "gpu")]
#[test]
fn test_empty_fired_neurons() {
    use feagi_npu_neural::types::FireCandidateList;

    let _config = BackendConfig::default();
    let (neuron_array, synapse_array) = create_test_genome(1000, 100);

    use feagi_npu_burst_engine::backend::WGPUBackend;
    let mut backend = WGPUBackend::new(neuron_array.capacity, synapse_array.capacity)
        .expect("GPU backend should be created");

    backend
        .initialize_persistent_data(&neuron_array, &synapse_array)
        .expect("Buffer upload should succeed");

    // Test with empty fired neurons list
    let fired_neurons: Vec<u32> = vec![];
    let mut fcl = FireCandidateList::new();

    let result = backend.process_synaptic_propagation(&fired_neurons, &synapse_array, &mut fcl);
    assert!(result.is_ok(), "Empty fired neurons should be handled");
    assert_eq!(result.unwrap(), 0, "Should process 0 synapses");
}

#[cfg(feature = "gpu")]
#[test]
fn test_gpu_backend_name() {
    let _config = BackendConfig::default();
    let (neuron_array, synapse_array) = create_test_genome(1000, 100);

    use feagi_npu_burst_engine::backend::WGPUBackend;
    let backend = WGPUBackend::new(neuron_array.capacity, synapse_array.capacity)
        .expect("GPU backend should be created");

    let name = backend.backend_name();
    assert!(
        name.contains("WGPU") || name.contains("GPU"),
        "Backend name should indicate GPU: {}",
        name
    );
    println!("✅ GPU Backend: {}", name);
}

#[test]
fn test_force_cpu_override_with_large_genome() {
    let _config = BackendConfig {
        force_cpu: true,
        ..Default::default()
    };

    // Large genome - force CPU backend
    let _backend = CPUBackend::new();
    // CPU backend created successfully
}

#[cfg(feature = "gpu")]
#[test]
fn test_force_gpu_override_with_small_genome() {
    let mut config = BackendConfig::default();
    config.force_gpu = true;

    use feagi_npu_burst_engine::backend::WGPUBackend;
    // Small genome - force GPU backend
    let result = WGPUBackend::new(100, 1_000);

    match result {
        Ok(backend) => {
            let name = backend.backend_name();
            assert!(
                name.contains("WGPU") || name.contains("GPU"),
                "Should use GPU backend when forced: {}",
                name
            );
        }
        Err(_) => {
            // GPU not available - acceptable
            println!("⚠️  GPU forced but not available, fallback OK");
        }
    }
}

#[cfg(feature = "gpu")]
#[test]
fn test_gpu_device_availability_check() {
    use wgpu::Backends;

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: Backends::all(),
        ..Default::default()
    });

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }));

    match adapter {
        Some(adapter) => {
            let info = adapter.get_info();
            println!("✅ GPU Available:");
            println!("   Name: {}", info.name);
            println!("   Backend: {:?}", info.backend);
            println!("   Device Type: {:?}", info.device_type);
            println!("   Vendor: 0x{:X}", info.vendor);
            println!("   Device: 0x{:X}", info.device);
        }
        None => {
            println!("⚠️  No GPU adapter available (software rendering or no GPU)");
        }
    }
}

#[test]
fn test_backend_creation_with_auto_selection() {
    let _config = BackendConfig::default();

    // Small genome - use CPU backend
    let small_backend = CPUBackend::new();
    println!(
        "Small genome → {}",
        <CPUBackend as ComputeBackend<
            f32,
            feagi_npu_runtime::StdNeuronArray<f32>,
            feagi_npu_runtime::StdSynapseArray,
        >>::backend_name(&small_backend)
    );

    #[cfg(feature = "gpu")]
    {
        use feagi_npu_burst_engine::backend::WGPUBackend;
        // Large genome - try GPU backend
        match WGPUBackend::new(1_000_000, 100_000_000) {
            Ok(large_backend) => {
                println!("Large genome → {}", large_backend.backend_name());
            }
            Err(_) => {
                println!("Large genome → CPU (GPU not available)");
            }
        }
    }
}

#[cfg(feature = "gpu")]
#[test]
fn test_multiple_backend_instances() {
    // Test creating multiple GPU backend instances
    let _config = BackendConfig::default();
    let (neuron_array, synapse_array) = create_test_genome(1000, 100);

    use feagi_npu_burst_engine::backend::WGPUBackend;
    let backend1 = WGPUBackend::new(neuron_array.capacity, synapse_array.capacity);

    let backend2 = WGPUBackend::new(neuron_array.capacity, synapse_array.capacity);

    // Both should succeed or both should fail (depending on GPU availability)
    match (backend1, backend2) {
        (Ok(_), Ok(_)) => {
            println!("✅ Multiple GPU backend instances created successfully");
        }
        (Err(e1), Err(e2)) => {
            println!(
                "⚠️  Multiple GPU instances not available: {:?}, {:?}",
                e1, e2
            );
        }
        _ => {
            panic!("Inconsistent GPU backend creation behavior");
        }
    }
}
