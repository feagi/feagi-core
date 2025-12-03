// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! GPU Memory Management and Stability Tests
//!
//! Tests memory management and long-run stability:
//! - Buffer lifecycle management
//! - Memory leak detection
//! - Multi-burst stability (1000+ bursts)
//! - Performance consistency over time
//!
//! Run with:
//!   cargo test --test gpu_memory_stability_test --features gpu

use feagi_burst_engine::backend::{create_backend, BackendConfig, BackendType};
use feagi_neural::types::{FireCandidateList, NeuronArray, NeuronId, SynapseArray};

/// Helper: Create test genome
fn create_test_genome(
    neuron_count: usize,
    synapses_per_neuron: usize,
) -> (NeuronArray<f32>, SynapseArray) {
    let mut neuron_array = NeuronArray::new(neuron_count);
    let synapse_count = neuron_count * synapses_per_neuron;
    let mut synapse_array = SynapseArray::new(synapse_count);

    for i in 0..neuron_count {
        neuron_array.membrane_potentials[i] = 0.0;
        neuron_array.thresholds[i] = 1.0;
        neuron_array.leak_coefficients[i] = 0.1;
        neuron_array.resting_potentials[i] = 0.0;
        neuron_array.excitabilities[i] = 1.0;
        neuron_array.refractory_periods[i] = 2;
        neuron_array.refractory_countdowns[i] = 0;
        neuron_array.valid_mask[i] = true;
    }
    neuron_array.count = neuron_count;

    let mut synapse_idx = 0;
    for source in 0..neuron_count {
        for i in 0..synapses_per_neuron {
            let target = (source + i + 1) % neuron_count;
            if synapse_idx < synapse_count {
                synapse_array.source_neurons[synapse_idx] = source as u32;
                synapse_array.target_neurons[synapse_idx] = target as u32;
                synapse_array.weights[synapse_idx] = 128;
                synapse_array.postsynaptic_potentials[synapse_idx] = 200;
                synapse_array.types[synapse_idx] = 0;
                synapse_array.valid_mask[synapse_idx] = true;

                synapse_array
                    .source_index
                    .entry(source as u32)
                    .or_insert_with(Vec::new)
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
fn test_gpu_1000_burst_stability() {
    let config = BackendConfig::default();
    let neuron_count = 5_000;
    let (mut neuron_array, synapse_array) = create_test_genome(neuron_count, 50);

    let mut backend = create_backend::<f32>(
        BackendType::WGPU,
        neuron_array.capacity,
        synapse_array.capacity,
        &config,
    )
    .expect("GPU backend should be created");

    backend
        .initialize_persistent_data(&neuron_array, &synapse_array)
        .expect("GPU initialization should succeed");

    println!("Starting 1000-burst stability test...");
    
    let mut total_fired = 0u64;
    let mut total_processed = 0u64;

    for burst in 1..=1000 {
        // Vary fired neurons each burst to simulate realistic activity
        let fired_neurons: Vec<u32> = ((burst % 100)..(burst % 100 + 10)).collect();

        let mut fcl = FireCandidateList::new();
        backend
            .process_synaptic_propagation(&fired_neurons, &synapse_array, &mut fcl)
            .expect(&format!("Burst {} synaptic propagation failed", burst));

        let (fired, processed, _) = backend
            .process_neural_dynamics(&fcl, &mut neuron_array, burst as u64)
            .expect(&format!("Burst {} neural dynamics failed", burst));

        total_fired += fired.len() as u64;
        total_processed += processed as u64;

        if burst % 100 == 0 {
            println!(
                "Burst {}: {} fired, {} processed (total: {} fired)",
                burst,
                fired.len(),
                processed,
                total_fired
            );
        }
    }

    println!("✅ 1000 bursts completed successfully");
    println!("   Total neurons processed: {}", total_processed);
    println!("   Total neurons fired: {}", total_fired);
    println!("   Average fired per burst: {}", total_fired / 1000);

    assert!(total_fired > 0, "Some neurons should have fired");
}

#[cfg(feature = "gpu")]
#[test]
fn test_gpu_buffer_reuse_across_bursts() {
    let config = BackendConfig::default();
    let neuron_count = 2_000;
    let (mut neuron_array, synapse_array) = create_test_genome(neuron_count, 30);

    let mut backend = create_backend::<f32>(
        BackendType::WGPU,
        neuron_array.capacity,
        synapse_array.capacity,
        &config,
    )
    .expect("GPU backend should be created");

    backend
        .initialize_persistent_data(&neuron_array, &synapse_array)
        .expect("GPU initialization should succeed");

    // Run multiple bursts and verify buffers are reused correctly
    for burst in 1..=50 {
        let fired_neurons: Vec<u32> = vec![burst % 100];

        let mut fcl = FireCandidateList::new();
        backend
            .process_synaptic_propagation(&fired_neurons, &synapse_array, &mut fcl)
            .expect("Synaptic propagation should succeed");

        let (fired, _, _) = backend
            .process_neural_dynamics(&fcl, &mut neuron_array, burst as u64)
            .expect("Neural dynamics should succeed");

        // Verify output is valid
        for &neuron_id in &fired {
            assert!(
                neuron_id < neuron_count as u32,
                "Fired neuron ID should be valid"
            );
        }
    }

    println!("✅ Buffer reuse across 50 bursts works correctly");
}

#[cfg(feature = "gpu")]
#[test]
fn test_gpu_performance_consistency() {
    use std::time::Instant;

    let config = BackendConfig::default();
    let neuron_count = 10_000;
    let (mut neuron_array, synapse_array) = create_test_genome(neuron_count, 100);

    let mut backend = create_backend::<f32>(
        BackendType::WGPU,
        neuron_array.capacity,
        synapse_array.capacity,
        &config,
    )
    .expect("GPU backend should be created");

    backend
        .initialize_persistent_data(&neuron_array, &synapse_array)
        .expect("GPU initialization should succeed");

    let mut burst_times = Vec::new();
    let fired_neurons: Vec<u32> = (0..100).collect();

    // Warm-up
    for _ in 0..5 {
        let mut fcl = FireCandidateList::new();
        backend
            .process_synaptic_propagation(&fired_neurons, &synapse_array, &mut fcl)
            .unwrap();
        backend
            .process_neural_dynamics(&fcl, &mut neuron_array, 1)
            .unwrap();
    }

    // Measure 100 bursts
    for burst in 1..=100 {
        let start = Instant::now();

        let mut fcl = FireCandidateList::new();
        backend
            .process_synaptic_propagation(&fired_neurons, &synapse_array, &mut fcl)
            .unwrap();
        backend
            .process_neural_dynamics(&fcl, &mut neuron_array, burst)
            .unwrap();

        let elapsed = start.elapsed().as_micros() as f64;
        burst_times.push(elapsed);
    }

    // Calculate statistics
    let mean_time: f64 = burst_times.iter().sum::<f64>() / burst_times.len() as f64;
    let variance: f64 = burst_times
        .iter()
        .map(|&x| (x - mean_time).powi(2))
        .sum::<f64>()
        / burst_times.len() as f64;
    let std_dev = variance.sqrt();

    println!("Performance statistics (100 bursts):");
    println!("  Mean: {:.1} μs", mean_time);
    println!("  Std Dev: {:.1} μs", std_dev);
    println!("  Coefficient of Variation: {:.1}%", (std_dev / mean_time) * 100.0);

    // Performance should be relatively consistent (CV < 50%)
    let cv = std_dev / mean_time;
    assert!(
        cv < 0.5,
        "Performance should be consistent (CV={:.1}% > 50%)",
        cv * 100.0
    );

    println!("✅ GPU performance is consistent over 100 bursts");
}

#[cfg(feature = "gpu")]
#[test]
fn test_gpu_repeated_initialization() {
    let config = BackendConfig::default();
    let neuron_count = 1_000;
    let (neuron_array, synapse_array) = create_test_genome(neuron_count, 20);

    // Create and destroy backend multiple times
    for iteration in 1..=10 {
        let mut backend = create_backend::<f32>(
            BackendType::WGPU,
            neuron_array.capacity,
            synapse_array.capacity,
            &config,
        )
        .expect(&format!("GPU backend creation {} should succeed", iteration));

        backend
            .initialize_persistent_data(&neuron_array, &synapse_array)
            .expect(&format!("Initialization {} should succeed", iteration));

        // Process one burst
        let mut fcl = FireCandidateList::new();
        fcl.add_candidate(NeuronId(0), 2.0);

        let mut neuron_array_copy = neuron_array.clone();
        backend
            .process_neural_dynamics(&fcl, &mut neuron_array_copy, 1)
            .expect("Processing should succeed");

        // Backend is dropped here
    }

    println!("✅ Repeated GPU backend creation/destruction works");
}

#[cfg(feature = "gpu")]
#[test]
fn test_gpu_varying_fcl_sizes() {
    let config = BackendConfig::default();
    let neuron_count = 5_000;
    let (mut neuron_array, synapse_array) = create_test_genome(neuron_count, 50);

    let mut backend = create_backend::<f32>(
        BackendType::WGPU,
        neuron_array.capacity,
        synapse_array.capacity,
        &config,
    )
    .expect("GPU backend should be created");

    backend
        .initialize_persistent_data(&neuron_array, &synapse_array)
        .expect("GPU initialization should succeed");

    // Test different FCL sizes
    let fcl_sizes = vec![1, 10, 100, 500, 1000, 2500, 5000];

    for size in fcl_sizes {
        let mut fcl = FireCandidateList::new();
        for i in 0..size.min(neuron_count) {
            fcl.add_candidate(NeuronId(i as u32), 2.0);
        }

        let result = backend.process_neural_dynamics(&fcl, &mut neuron_array, 1);
        assert!(
            result.is_ok(),
            "FCL size {} should be handled",
            size
        );

        let (_, processed, _) = result.unwrap();
        assert_eq!(
            processed,
            size.min(neuron_count),
            "Should process {} neurons",
            size
        );
    }

    println!("✅ Varying FCL sizes handled correctly");
}

#[cfg(feature = "gpu")]
#[test]
fn test_gpu_continuous_activity_pattern() {
    let config = BackendConfig::default();
    let neuron_count = 3_000;
    let (mut neuron_array, synapse_array) = create_test_genome(neuron_count, 40);

    let mut backend = create_backend::<f32>(
        BackendType::WGPU,
        neuron_array.capacity,
        synapse_array.capacity,
        &config,
    )
    .expect("GPU backend should be created");

    backend
        .initialize_persistent_data(&neuron_array, &synapse_array)
        .expect("GPU initialization should succeed");

    // Simulate continuous activity (each burst's output feeds into next)
    let mut fired_neurons: Vec<u32> = vec![0, 1, 2]; // Initial seed

    for burst in 1..=100 {
        let mut fcl = FireCandidateList::new();
        backend
            .process_synaptic_propagation(&fired_neurons, &synapse_array, &mut fcl)
            .expect("Synaptic propagation should succeed");

        let (new_fired, processed, _) = backend
            .process_neural_dynamics(&fcl, &mut neuron_array, burst)
            .expect("Neural dynamics should succeed");

        if burst % 20 == 0 {
            println!(
                "Burst {}: {} processed, {} fired",
                burst,
                processed,
                new_fired.len()
            );
        }

        // Use new fired neurons for next burst
        fired_neurons = if new_fired.is_empty() {
            vec![(burst % neuron_count as u64) as u32] // Inject activity if needed
        } else {
            new_fired
        };

        // Verify activity doesn't explode or die out completely
        assert!(
            fired_neurons.len() < neuron_count / 2,
            "Activity should not explode"
        );
    }

    println!("✅ Continuous activity pattern stable over 100 bursts");
}

#[cfg(feature = "gpu")]
#[test]
fn test_gpu_zero_activity_periods() {
    let config = BackendConfig::default();
    let neuron_count = 2_000;
    let (mut neuron_array, synapse_array) = create_test_genome(neuron_count, 30);

    let mut backend = create_backend::<f32>(
        BackendType::WGPU,
        neuron_array.capacity,
        synapse_array.capacity,
        &config,
    )
    .expect("GPU backend should be created");

    backend
        .initialize_persistent_data(&neuron_array, &synapse_array)
        .expect("GPU initialization should succeed");

    // Alternate between activity and silence
    for burst in 1..=50 {
        let fired_neurons: Vec<u32> = if burst % 2 == 0 {
            vec![] // Silent burst
        } else {
            vec![burst % 100] // Active burst
        };

        let mut fcl = FireCandidateList::new();
        backend
            .process_synaptic_propagation(&fired_neurons, &synapse_array, &mut fcl)
            .expect("Synaptic propagation should succeed");

        let result = backend.process_neural_dynamics(&fcl, &mut neuron_array, burst as u64);
        assert!(
            result.is_ok(),
            "Burst {} should succeed",
            burst
        );
    }

    println!("✅ Alternating activity/silence handled correctly");
}

#[cfg(feature = "gpu")]
#[test]
fn test_gpu_burst_activity() {
    let config = BackendConfig::default();
    let neuron_count = 5_000;
    let (mut neuron_array, synapse_array) = create_test_genome(neuron_count, 60);

    let mut backend = create_backend::<f32>(
        BackendType::WGPU,
        neuron_array.capacity,
        synapse_array.capacity,
        &config,
    )
    .expect("GPU backend should be created");

    backend
        .initialize_persistent_data(&neuron_array, &synapse_array)
        .expect("GPU initialization should succeed");

    // Simulate burst activity (occasional high activity)
    for burst in 1..=100 {
        let fired_neurons: Vec<u32> = if burst % 10 == 0 {
            // Large burst every 10 cycles
            (0..500).collect()
        } else {
            // Low activity
            vec![burst % 100]
        };

        let mut fcl = FireCandidateList::new();
        backend
            .process_synaptic_propagation(&fired_neurons, &synapse_array, &mut fcl)
            .expect("Synaptic propagation should succeed");

        let (fired, processed, _) = backend
            .process_neural_dynamics(&fcl, &mut neuron_array, burst as u64)
            .expect("Neural dynamics should succeed");

        if burst % 10 == 0 {
            println!(
                "Burst {} (high activity): {} processed, {} fired",
                burst,
                processed,
                fired.len()
            );
        }
    }

    println!("✅ Burst activity pattern handled correctly");
}

#[cfg(feature = "gpu")]
#[test]
fn test_gpu_no_memory_leak_indicator() {
    let config = BackendConfig::default();
    let neuron_count = 2_000;
    let (mut neuron_array, synapse_array) = create_test_genome(neuron_count, 40);

    let mut backend = create_backend::<f32>(
        BackendType::WGPU,
        neuron_array.capacity,
        synapse_array.capacity,
        &config,
    )
    .expect("GPU backend should be created");

    backend
        .initialize_persistent_data(&neuron_array, &synapse_array)
        .expect("GPU initialization should succeed");

    // Run many bursts - if there's a memory leak, test will eventually fail or slow down
    let fired_neurons: Vec<u32> = (0..50).collect();

    for burst in 1..=200 {
        let mut fcl = FireCandidateList::new();
        backend
            .process_synaptic_propagation(&fired_neurons, &synapse_array, &mut fcl)
            .expect(&format!("Burst {} should not fail", burst));

        backend
            .process_neural_dynamics(&fcl, &mut neuron_array, burst as u64)
            .expect(&format!("Burst {} should not fail", burst));

        // If we make it this far without OOM, no obvious leak
    }

    println!("✅ No obvious memory leak detected in 200 bursts");
}

#[cfg(feature = "gpu")]
#[test]
fn test_gpu_large_then_small_fcl() {
    let config = BackendConfig::default();
    let neuron_count = 5_000;
    let (mut neuron_array, synapse_array) = create_test_genome(neuron_count, 50);

    let mut backend = create_backend::<f32>(
        BackendType::WGPU,
        neuron_array.capacity,
        synapse_array.capacity,
        &config,
    )
    .expect("GPU backend should be created");

    backend
        .initialize_persistent_data(&neuron_array, &synapse_array)
        .expect("GPU initialization should succeed");

    // Test going from large FCL to small FCL (tests buffer cleanup)
    
    // Large FCL
    let mut fcl_large = FireCandidateList::new();
    for i in 0..2500 {
        fcl_large.add_candidate(NeuronId(i), 2.0);
    }

    let (_, processed_large, _) = backend
        .process_neural_dynamics(&fcl_large, &mut neuron_array, 1)
        .expect("Large FCL should succeed");
    assert_eq!(processed_large, 2500);

    // Small FCL immediately after
    let mut fcl_small = FireCandidateList::new();
    for i in 0..10 {
        fcl_small.add_candidate(NeuronId(i), 2.0);
    }

    let (_, processed_small, _) = backend
        .process_neural_dynamics(&fcl_small, &mut neuron_array, 2)
        .expect("Small FCL should succeed");
    assert_eq!(processed_small, 10);

    println!("✅ Transition from large to small FCL works correctly");
}

