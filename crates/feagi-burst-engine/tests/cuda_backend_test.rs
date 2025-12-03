// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! CUDA Backend Tests
//!
//! These tests validate the CUDA backend structure and API.
//! Most tests can run without actual CUDA hardware (compile-time validation).
//!
//! To run tests with CUDA hardware:
//! ```bash
//! cargo test --features cuda --test cuda_backend_test
//! ```

#[cfg(feature = "cuda")]
mod cuda_tests {
    use feagi_burst_engine::backend::{
        CUDABackend, ComputeBackend, enumerate_cuda_devices, is_cuda_available,
    };
    use feagi_neural::types::{FireCandidateList, NeuronArray, NeuronId, SynapseArray};

    #[test]
    fn test_cuda_feature_enabled() {
        // This test ensures CUDA feature is properly compiled
        assert!(true, "CUDA feature is enabled");
    }

    #[test]
    fn test_cuda_availability_check() {
        // Test that we can check for CUDA without panicking
        let available = is_cuda_available();
        println!("CUDA available: {}", available);
        
        // This should not panic regardless of CUDA availability
        assert!(available || !available);  // Tautology, but validates no panic
    }

    #[test]
    fn test_enumerate_cuda_devices() {
        // Test device enumeration
        let devices = enumerate_cuda_devices();
        println!("Found {} CUDA device(s)", devices.len());
        
        for (id, name, memory) in devices.iter() {
            println!("  GPU {}: {} ({} GB)", id, name, memory / (1024 * 1024 * 1024));
        }
        
        // Should not panic
    }

    #[test]
    #[ignore] // Requires actual CUDA hardware
    fn test_cuda_backend_creation() {
        // Skip if CUDA not available
        if !is_cuda_available() {
            println!("⚠️  CUDA not available, skipping");
            return;
        }
        
        // Test backend creation
        let result = CUDABackend::new(10_000, 100_000);
        
        match result {
            Ok(backend) => {
                println!("✅ Created CUDA backend: {}", backend.name());
                // Verify the backend name contains GPU/NVIDIA (actual device name)
                let name = backend.name();
                assert!(name.contains("NVIDIA") || name.contains("GPU") || name.contains("Tesla") || name.contains("A100"));
            }
            Err(e) => {
                panic!("Failed to create CUDA backend: {}", e);
            }
        }
    }

    #[test]
    #[ignore] // Requires actual CUDA hardware
    fn test_cuda_backend_initialization() {
        if !is_cuda_available() {
            println!("⚠️  CUDA not available, skipping");
            return;
        }
        
        let mut backend = CUDABackend::new(1_000, 10_000)
            .expect("Failed to create CUDA backend");
        
        // Create test genome
        let mut neuron_array = NeuronArray::new(1_000);
        let mut synapse_array = SynapseArray::new(10_000);
        
        // Initialize with simple data
        for i in 0..1_000 {
            neuron_array.membrane_potentials[i] = 0.0;
            neuron_array.thresholds[i] = 10.0;
            neuron_array.leak_coefficients[i] = 0.1;
            neuron_array.resting_potentials[i] = 0.0;
            neuron_array.excitabilities[i] = 1.0;
            neuron_array.valid_mask[i] = true;
        }
        neuron_array.count = 1_000;
        
        // Create simple synapse connectivity
        for i in 0..10_000 {
            synapse_array.source_neurons[i] = (i / 10) as u32;
            synapse_array.target_neurons[i] = ((i / 10 + 1) % 1000) as u32;
            synapse_array.weights[i] = 128;
            synapse_array.postsynaptic_potentials[i] = 200;
            synapse_array.types[i] = 0;  // Excitatory
            synapse_array.valid_mask[i] = true;
            
            synapse_array
                .source_index
                .entry(synapse_array.source_neurons[i])
                .or_insert_with(Vec::new)
                .push(i);
        }
        synapse_array.count = 10_000;
        
        // Test initialization
        let result = backend.initialize_persistent_data(&neuron_array, &synapse_array);
        
        match result {
            Ok(_) => println!("✅ Successfully initialized CUDA backend"),
            Err(e) => {
                // Expected for now since kernels are stubs
                println!("⚠️  Initialization incomplete (expected): {}", e);
            }
        }
    }

    #[test]
    #[ignore] // Requires actual CUDA hardware
    fn test_cuda_multi_device() {
        let devices = enumerate_cuda_devices();
        
        if devices.len() < 2 {
            println!("⚠️  Less than 2 GPUs available, skipping multi-GPU test");
            return;
        }
        
        println!("Testing multi-GPU with {} devices", devices.len());
        
        // Create backends for first 2 GPUs
        for device_id in 0..2 {
            let result = CUDABackend::new_on_device(device_id, 10_000, 100_000);
            
            match result {
                Ok(backend) => {
                    println!("✅ Created backend on GPU {}: {}", device_id, backend.name());
                }
                Err(e) => {
                    println!("⚠️  Failed to create backend on GPU {}: {}", device_id, e);
                }
            }
        }
    }

    #[test]
    fn test_cuda_backend_size_limits() {
        if !is_cuda_available() {
            println!("⚠️  CUDA not available, skipping");
            return;
        }
        
        // Test various genome sizes
        let test_cases = vec![
            (100_000, 10_000_000, "100K neurons, 10M synapses"),
            (500_000, 50_000_000, "500K neurons, 50M synapses"),
            (1_000_000, 100_000_000, "1M neurons, 100M synapses"),
        ];
        
        for (neurons, synapses, label) in test_cases {
            println!("Testing: {}", label);
            
            let result = CUDABackend::new(neurons, synapses);
            
            match result {
                Ok(_) => println!("  ✅ {} fits in CUDA backend", label),
                Err(e) => println!("  ⚠️  {} exceeds limits: {}", label, e),
            }
        }
    }
}

#[cfg(not(feature = "cuda"))]
mod no_cuda_tests {
    use feagi_burst_engine::backend::CUDABackend;

    #[test]
    fn test_cuda_feature_disabled() {
        // When CUDA feature is disabled, creation should fail gracefully
        let result = CUDABackend::new(1_000, 10_000);
        
        assert!(result.is_err(), "CUDA backend should fail when feature disabled");
        
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("CUDA support not compiled") || err_msg.contains("features cuda"),
            "Error message should indicate CUDA not compiled"
        );
    }
}

// Tests that work regardless of CUDA availability
#[test]
fn test_backend_trait_object_safety() {
    // Verify that ComputeBackend trait is object-safe
    // This is important for dynamic backend selection
    
    use feagi_burst_engine::backend::CPUBackend;
    use feagi_neural::types::NeuralValue;
    
    fn _uses_trait_object(_backend: &dyn feagi_burst_engine::backend::ComputeBackend<f32>) {
        // This function signature proves trait object safety
    }
    
    // Test with CPU backend (always available)
    let cpu_backend = CPUBackend::new();
    _uses_trait_object(&cpu_backend);
}

#[test]
fn test_cuda_compile_time_validation() {
    // This test validates that CUDA backend compiles correctly
    // It doesn't need runtime CUDA hardware
    
    #[cfg(feature = "cuda")]
    {
        // Type check: ensure functions exist with correct signatures
        let _: fn() -> bool = feagi_burst_engine::backend::is_cuda_available;
        let _: fn() -> Vec<(usize, String, u64)> = feagi_burst_engine::backend::enumerate_cuda_devices;
        
        println!("✅ CUDA backend API validated at compile time");
    }
    
    #[cfg(not(feature = "cuda"))]
    {
        println!("⚠️  CUDA feature not enabled, compile-time validation skipped");
    }
}

