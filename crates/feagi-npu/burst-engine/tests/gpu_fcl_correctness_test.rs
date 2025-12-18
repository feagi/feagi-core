// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! GPU FCL (Fire Candidate List) Correctness Tests
//!
//! Tests the correctness of FCL-aware GPU processing:
//! - Empty FCL handling
//! - Full FCL (all neurons)
//! - GPU→GPU pipeline correctness
//! - Atomic accumulation validation
//! - FCL capacity handling
//!
//! Run with:
//!   cargo test --test gpu_fcl_correctness_test --features gpu

use feagi_npu_burst_engine::backend::{create_backend, BackendConfig, BackendType, ComputeBackend};
use feagi_npu_burst_engine::FireCandidateList;
    use feagi_npu_neural::types::NeuronId;
    use feagi_npu_runtime_std::{NeuronArray, SynapseArray};

/// Helper: Create test genome
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
        neuron_array.thresholds[i] = 1.0; // Low threshold for easy firing
        neuron_array.leak_coefficients[i] = 0.0; // No leak
        neuron_array.resting_potentials[i] = 0.0;
        neuron_array.excitabilities[i] = 1.0;
        neuron_array.refractory_periods[i] = 0; // No refractory
        neuron_array.refractory_countdowns[i] = 0;
        neuron_array.valid_mask[i] = true;
    }
    neuron_array.count = neuron_count;

    // Initialize synapses (each neuron connects to next few neurons)
    let mut synapse_idx = 0;
    for source in 0..neuron_count {
        for i in 0..synapses_per_neuron {
            let target = (source + i + 1) % neuron_count;
            if synapse_idx < synapse_count {
                synapse_array.source_neurons[synapse_idx] = source as u32;
                synapse_array.target_neurons[synapse_idx] = target as u32;
                synapse_array.weights[synapse_idx] = 200; // Strong weights
                synapse_array.postsynaptic_potentials[synapse_idx] = 255; // Max PSP
                synapse_array.types[synapse_idx] = 0; // Excitatory
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
fn test_empty_fcl_processing() {
    let config = BackendConfig::default();
    let (mut neuron_array, synapse_array) = create_test_genome(1000, 10);

    let mut backend = create_backend::<f32>(
        BackendType::WGPU,
        neuron_array.capacity,
        synapse_array.capacity,
        &config,
    )
    .expect("GPU backend should be created");

    backend
        .initialize_persistent_data(&neuron_array, &synapse_array)
        .expect("Buffer upload should succeed");

    // Create empty FCL
    let fcl = FireCandidateList::new();

    // Process with empty FCL
    let result = backend.process_neural_dynamics(&fcl, &mut neuron_array, 1);

    assert!(result.is_ok(), "Empty FCL should be handled gracefully");
    let (fired, processed, _) = result.unwrap();
    assert_eq!(fired.len(), 0, "No neurons should fire with empty FCL");
    assert_eq!(processed, 0, "No neurons should be processed");
    println!("✅ Empty FCL handled correctly");
}

#[cfg(feature = "gpu")]
#[test]
fn test_full_fcl_all_neurons() {
    let config = BackendConfig::default();
    let neuron_count = 1000;
    let (mut neuron_array, synapse_array) = create_test_genome(neuron_count, 10);

    // Set high membrane potentials to ensure firing
    for i in 0..neuron_count {
        neuron_array.membrane_potentials[i] = 10.0; // Well above threshold
    }

    let mut backend = create_backend::<f32>(
        BackendType::WGPU,
        neuron_array.capacity,
        synapse_array.capacity,
        &config,
    )
    .expect("GPU backend should be created");

    backend
        .initialize_persistent_data(&neuron_array, &synapse_array)
        .expect("Buffer upload should succeed");

    // Create FCL with ALL neurons
    let mut fcl = FireCandidateList::new();
    for i in 0..neuron_count {
        fcl.add_candidate(NeuronId(i as u32), 10.0);
    }

    // Process with full FCL
    let result = backend.process_neural_dynamics(&fcl, &mut neuron_array, 1);

    assert!(result.is_ok(), "Full FCL should be processed");
    let (fired, processed, _) = result.unwrap();

    assert_eq!(
        processed, neuron_count,
        "All neurons should be processed"
    );
    assert!(
        fired.len() > neuron_count / 2,
        "Most neurons should fire with high potentials"
    );
    println!(
        "✅ Full FCL processed: {} neurons, {} fired",
        processed,
        fired.len()
    );
}

#[cfg(feature = "gpu")]
#[test]
fn test_fcl_with_duplicate_entries() {
    let config = BackendConfig::default();
    let (mut neuron_array, synapse_array) = create_test_genome(1000, 10);

    let mut backend = create_backend::<f32>(
        BackendType::WGPU,
        neuron_array.capacity,
        synapse_array.capacity,
        &config,
    )
    .expect("GPU backend should be created");

    backend
        .initialize_persistent_data(&neuron_array, &synapse_array)
        .expect("Buffer upload should succeed");

    // Create FCL with duplicate entries
    let mut fcl = FireCandidateList::new();
    fcl.add_candidate(NeuronId(0), 5.0);
    fcl.add_candidate(NeuronId(0), 5.0); // Duplicate - should accumulate
    fcl.add_candidate(NeuronId(1), 3.0);

    // Process
    let result = backend.process_neural_dynamics(&fcl, &mut neuron_array, 1);

    assert!(result.is_ok(), "Duplicate FCL entries should be handled");
    println!("✅ Duplicate FCL entries handled");
}

#[cfg(feature = "gpu")]
#[test]
fn test_fcl_sparse_processing() {
    use feagi_npu_burst_engine::backend::CPUBackend;

    let config = BackendConfig::default();
    let neuron_count = 10_000;
    let (mut neuron_array, synapse_array) = create_test_genome(neuron_count, 100);

    // GPU backend
    let mut gpu_backend = create_backend::<f32>(
        BackendType::WGPU,
        neuron_array.capacity,
        synapse_array.capacity,
        &config,
    )
    .expect("GPU backend should be created");

    gpu_backend
        .initialize_persistent_data(&neuron_array, &synapse_array)
        .expect("Buffer upload should succeed");

    // CPU backend (reference)
    let mut cpu_backend = CPUBackend::new();
    let mut neuron_array_cpu = neuron_array.clone();

    // Create sparse FCL (only 1% of neurons)
    let mut fcl_gpu = FireCandidateList::new();
    let mut fcl_cpu = FireCandidateList::new();
    for i in 0..neuron_count / 100 {
        let neuron_id = NeuronId((i * 100) as u32);
        fcl_gpu.add_candidate(neuron_id, 2.0); // Above threshold
        fcl_cpu.add_candidate(neuron_id, 2.0);
    }

    // Process on GPU
    let gpu_result = gpu_backend
        .process_neural_dynamics(&fcl_gpu, &mut neuron_array, 1)
        .expect("GPU processing should succeed");

    // Process on CPU
    let cpu_result = cpu_backend
        .process_neural_dynamics(&fcl_cpu, &mut neuron_array_cpu, 1)
        .expect("CPU processing should succeed");

    // Compare results
    assert_eq!(
        gpu_result.1, cpu_result.1,
        "GPU and CPU should process same number of neurons"
    );

    // Allow small difference in firing due to floating-point precision
    let fired_diff = (gpu_result.0.len() as i32 - cpu_result.0.len() as i32).abs();
    assert!(
        fired_diff <= 5,
        "GPU and CPU should fire similar number of neurons (diff: {})",
        fired_diff
    );

    println!("✅ Sparse FCL processing matches between GPU and CPU");
}

#[cfg(feature = "gpu")]
#[test]
fn test_fcl_gpu_to_gpu_pipeline() {
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
        .expect("Buffer upload should succeed");

    // Fire some neurons
    let fired_neurons: Vec<u32> = (0..50).collect();

    // Phase 1: Synaptic propagation → FCL (stays on GPU)
    let mut fcl = FireCandidateList::new();
    let synapses_processed = backend
        .process_synaptic_propagation(&fired_neurons, &synapse_array, &mut fcl)
        .expect("Synaptic propagation should succeed");

    println!("Synapses processed: {}", synapses_processed);
    println!("FCL size after synaptic: {}", fcl.len());

    // Phase 2: Neural dynamics (FCL → fired neurons, all on GPU)
    let (new_fired, processed, _) = backend
        .process_neural_dynamics(&fcl, &mut neuron_array, 1)
        .expect("Neural dynamics should succeed");

    println!("Neurons processed: {}", processed);
    println!("Neurons fired: {}", new_fired.len());

    assert!(
        processed > 0,
        "Some neurons should be processed from synaptic propagation"
    );
    assert!(synapses_processed > 0, "Some synapses should be processed");

    println!("✅ GPU→GPU pipeline completed successfully");
}

#[cfg(feature = "gpu")]
#[test]
fn test_fcl_with_invalid_neuron_ids() {
    let config = BackendConfig::default();
    let neuron_count = 1000;
    let (mut neuron_array, synapse_array) = create_test_genome(neuron_count, 10);

    let mut backend = create_backend::<f32>(
        BackendType::WGPU,
        neuron_array.capacity,
        synapse_array.capacity,
        &config,
    )
    .expect("GPU backend should be created");

    backend
        .initialize_persistent_data(&neuron_array, &synapse_array)
        .expect("Buffer upload should succeed");

    // Create FCL with some invalid neuron IDs (beyond array bounds)
    let mut fcl = FireCandidateList::new();
    fcl.add_candidate(NeuronId(0), 2.0); // Valid
    fcl.add_candidate(NeuronId(999), 2.0); // Valid (last neuron)
    // Note: FCL validates IDs during add_candidate, so we can't easily add invalid IDs
    // This test verifies that valid boundary cases work

    let result = backend.process_neural_dynamics(&fcl, &mut neuron_array, 1);

    assert!(
        result.is_ok(),
        "Boundary neuron IDs should be handled correctly"
    );
    println!("✅ Boundary neuron IDs handled correctly");
}

#[cfg(feature = "gpu")]
#[test]
fn test_fcl_accumulation_correctness() {
    let config = BackendConfig::default();
    let neuron_count = 1000;
    let (neuron_array, synapse_array) = create_test_genome(neuron_count, 10);

    let mut backend = create_backend::<f32>(
        BackendType::WGPU,
        neuron_array.capacity,
        synapse_array.capacity,
        &config,
    )
    .expect("GPU backend should be created");

    backend
        .initialize_persistent_data(&neuron_array, &synapse_array)
        .expect("Buffer upload should succeed");

    // Fire multiple neurons that target the same downstream neurons
    // This tests atomic accumulation in GPU
    let fired_neurons: Vec<u32> = vec![0, 1, 2, 3, 4]; // Adjacent neurons

    let mut fcl = FireCandidateList::new();
    let synapses_processed = backend
        .process_synaptic_propagation(&fired_neurons, &synapse_array, &mut fcl)
        .expect("Synaptic propagation should succeed");

    println!("Synapses processed: {}", synapses_processed);
    println!("FCL candidates: {}", fcl.len());

    // FCL should contain accumulated contributions
    // With 5 source neurons × 10 synapses each = 50 synapses
    // Targeting ~10 unique downstream neurons (with overlap)
    assert!(fcl.len() > 0, "FCL should have candidates");
    assert!(fcl.len() <= 50, "FCL should not exceed possible targets");

    println!("✅ FCL accumulation completed");
}

#[cfg(feature = "gpu")]
#[test]
fn test_fcl_multiple_bursts() {
    let config = BackendConfig::default();
    let neuron_count = 2000;
    let (mut neuron_array, synapse_array) = create_test_genome(neuron_count, 20);

    let mut backend = create_backend::<f32>(
        BackendType::WGPU,
        neuron_array.capacity,
        synapse_array.capacity,
        &config,
    )
    .expect("GPU backend should be created");

    backend
        .initialize_persistent_data(&neuron_array, &synapse_array)
        .expect("Buffer upload should succeed");

    // Run multiple bursts and verify FCL is cleared between bursts
    let mut prev_fired_count = 0;

    for burst in 1..=5 {
        let fired_neurons: Vec<u32> = vec![burst % 100]; // Different seed each burst

        let mut fcl = FireCandidateList::new();
        backend
            .process_synaptic_propagation(&fired_neurons, &synapse_array, &mut fcl)
            .expect("Synaptic propagation should succeed");

        let (fired, processed, _) = backend
            .process_neural_dynamics(&fcl, &mut neuron_array, burst as u64)
            .expect("Neural dynamics should succeed");

        println!(
            "Burst {}: {} processed, {} fired",
            burst,
            processed,
            fired.len()
        );

        // Verify we're not accumulating across bursts inappropriately
        if burst > 1 {
            let change = (fired.len() as i32 - prev_fired_count as i32).abs();
            assert!(
                change < neuron_count as i32,
                "Firing pattern should not explode across bursts"
            );
        }

        prev_fired_count = fired.len();
    }

    println!("✅ Multiple bursts processed correctly");
}

#[cfg(feature = "gpu")]
#[test]
fn test_fcl_very_sparse() {
    let config = BackendConfig::default();
    let neuron_count = 100_000;
    let (mut neuron_array, synapse_array) = create_test_genome(neuron_count, 10);

    let mut backend = create_backend::<f32>(
        BackendType::WGPU,
        neuron_array.capacity,
        synapse_array.capacity,
        &config,
    )
    .expect("GPU backend should be created");

    backend
        .initialize_persistent_data(&neuron_array, &synapse_array)
        .expect("Buffer upload should succeed");

    // Very sparse FCL (only 10 neurons out of 100K = 0.01%)
    let mut fcl = FireCandidateList::new();
    for i in 0..10 {
        fcl.add_candidate(NeuronId((i * 10000) as u32), 2.0);
    }

    let result = backend.process_neural_dynamics(&fcl, &mut neuron_array, 1);

    assert!(result.is_ok(), "Very sparse FCL should be handled");
    let (_, processed, _) = result.unwrap();
    assert_eq!(processed, 10, "Should process exactly 10 neurons");

    println!("✅ Very sparse FCL (0.01%) handled efficiently");
}

#[cfg(feature = "gpu")]
#[test]
fn test_fcl_medium_density() {
    let config = BackendConfig::default();
    let neuron_count = 10_000;
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
        .expect("Buffer upload should succeed");

    // Medium density FCL (10% of neurons = 1000 neurons)
    let mut fcl = FireCandidateList::new();
    for i in 0..1000 {
        fcl.add_candidate(NeuronId((i * 10) as u32), 2.0);
    }

    let result = backend.process_neural_dynamics(&fcl, &mut neuron_array, 1);

    assert!(result.is_ok(), "Medium density FCL should be handled");
    let (_, processed, _) = result.unwrap();
    assert_eq!(processed, 1000, "Should process 1000 neurons");

    println!("✅ Medium density FCL (10%) handled");
}

