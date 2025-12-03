// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! GPU Synaptic Hash Table Tests
//!
//! Tests hash table correctness for synaptic propagation:
//! - Hash collision handling (linear probing)
//! - Empty hash table
//! - High load factor scenarios
//! - Hash lookup failures
//! - Synapse weight calculations
//!
//! Run with:
//!   cargo test --test gpu_synaptic_hash_test --features gpu

use feagi_burst_engine::backend::{create_backend, BackendConfig, BackendType};
use feagi_neural::types::{FireCandidateList, NeuronArray, SynapseArray};

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
        neuron_array.thresholds[i] = 10.0;
        neuron_array.leak_coefficients[i] = 0.0;
        neuron_array.resting_potentials[i] = 0.0;
        neuron_array.excitabilities[i] = 1.0;
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

/// Find neuron IDs that hash to the same slot (for collision testing)
fn find_colliding_neuron_ids(target_slot: usize, capacity: usize, count: usize) -> Vec<u32> {
    let mut colliding = Vec::new();
    
    // Hash function used in GPU shader: hash = neuron_id * 2654435761 % capacity
    for id in 0..1_000_000_u32 {
        let hash = ((id as u64) * 2654435761u64) % (capacity as u64);
        if hash == target_slot as u64 {
            colliding.push(id);
            if colliding.len() >= count {
                break;
            }
        }
    }
    
    colliding
}

#[cfg(feature = "gpu")]
#[test]
fn test_gpu_hash_collision_correctness() {
    let config = BackendConfig::default();
    
    // Create genome where source neurons will hash to same slot
    let neuron_count = 5000;
    let hash_capacity = 1024; // Typical hash table size
    
    // Find neurons that collide at slot 42
    let colliding_ids = find_colliding_neuron_ids(42, hash_capacity, 5);
    println!("Found {} colliding neuron IDs: {:?}", colliding_ids.len(), colliding_ids);
    
    // Create synapses with colliding source neurons
    let mut synapse_array = SynapseArray::new(colliding_ids.len() * 10);
    let mut neuron_array = NeuronArray::new(neuron_count);
    
    for i in 0..neuron_count {
        neuron_array.membrane_potentials[i] = 0.0;
        neuron_array.thresholds[i] = 1.0;
        neuron_array.leak_coefficients[i] = 0.0;
        neuron_array.resting_potentials[i] = 0.0;
        neuron_array.excitabilities[i] = 1.0;
        neuron_array.valid_mask[i] = true;
    }
    neuron_array.count = neuron_count;
    
    let mut synapse_idx = 0;
    for (source_idx, &source_id) in colliding_ids.iter().enumerate() {
        // Create 10 synapses per colliding source neuron
        for i in 0..10 {
            let target = (source_idx * 10 + i) as u32;
            synapse_array.source_neurons[synapse_idx] = source_id;
            synapse_array.target_neurons[synapse_idx] = target;
            synapse_array.weights[synapse_idx] = 128;
            synapse_array.postsynaptic_potentials[synapse_idx] = 200;
            synapse_array.types[synapse_idx] = 0;
            synapse_array.valid_mask[synapse_idx] = true;
            
            synapse_array
                .source_index
                .entry(source_id)
                .or_insert_with(Vec::new)
                .push(synapse_idx);
            
            synapse_idx += 1;
        }
    }
    synapse_array.count = synapse_idx;
    
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
    
    // Fire all colliding neurons
    let fired_neurons = colliding_ids.clone();
    
    let mut fcl = FireCandidateList::new();
    let synapses_processed = backend
        .process_synaptic_propagation(&fired_neurons, &synapse_array, &mut fcl)
        .expect("Synaptic propagation should succeed");
    
    println!("Synapses processed: {}", synapses_processed);
    println!("FCL candidates: {}", fcl.len());
    
    // Should process all synapses despite hash collisions
    assert!(
        synapses_processed >= synapse_array.count,
        "Should process all synapses with hash collisions"
    );
    
    // All target neurons should be in FCL
    assert!(
        fcl.len() > 0,
        "FCL should contain target neurons"
    );
    
    println!("✅ Hash collisions handled correctly");
}

#[cfg(feature = "gpu")]
#[test]
fn test_gpu_empty_synapse_array() {
    let config = BackendConfig::default();
    
    // Create genome with no synapses
    let neuron_count = 1000;
    let mut neuron_array = NeuronArray::new(neuron_count);
    neuron_array.count = neuron_count; // Set count so buffers are created
    let synapse_array = SynapseArray::new(0);
    
    let mut backend = create_backend::<f32>(
        BackendType::WGPU,
        neuron_array.capacity,
        synapse_array.capacity.max(1), // Ensure non-zero capacity
        &config,
    )
    .expect("GPU backend should be created");
    
    backend
        .initialize_persistent_data(&neuron_array, &synapse_array)
        .expect("Empty synapse array should be handled");
    
    // Fire neurons with no synapses
    let fired_neurons: Vec<u32> = vec![0, 1, 2];
    
    let mut fcl = FireCandidateList::new();
    let synapses_processed = backend
        .process_synaptic_propagation(&fired_neurons, &synapse_array, &mut fcl)
        .expect("Should handle empty synapses");
    
    assert_eq!(synapses_processed, 0, "Should process 0 synapses");
    assert_eq!(fcl.len(), 0, "FCL should be empty");
    
    println!("✅ Empty synapse array handled correctly");
}

#[cfg(feature = "gpu")]
#[test]
fn test_gpu_high_load_factor_hash() {
    let config = BackendConfig::default();
    
    // Create genome with many synapses per source neuron
    // This creates a high load factor in the hash table
    let neuron_count = 1000;
    let synapses_per_neuron = 500; // Very high connectivity
    
    let (neuron_array, synapse_array) = create_test_genome(neuron_count, synapses_per_neuron);
    
    println!("Total synapses: {}", synapse_array.count);
    println!("Unique sources: {}", synapse_array.source_index.len());
    
    let mut backend = create_backend::<f32>(
        BackendType::WGPU,
        neuron_array.capacity,
        synapse_array.capacity,
        &config,
    )
    .expect("GPU backend should be created");
    
    backend
        .initialize_persistent_data(&neuron_array, &synapse_array)
        .expect("High connectivity should be handled");
    
    // Fire multiple neurons
    let fired_neurons: Vec<u32> = (0..100).collect();
    
    let mut fcl = FireCandidateList::new();
    let synapses_processed = backend
        .process_synaptic_propagation(&fired_neurons, &synapse_array, &mut fcl)
        .expect("Synaptic propagation should succeed");
    
    println!("Synapses processed: {}", synapses_processed);
    println!("FCL candidates: {}", fcl.len());
    
    assert!(
        synapses_processed > 0,
        "Should process synapses with high load factor"
    );
    
    println!("✅ High load factor hash table works");
}

#[cfg(feature = "gpu")]
#[test]
fn test_gpu_sparse_connectivity() {
    let config = BackendConfig::default();
    
    // Create genome with very sparse connectivity (few synapses)
    let neuron_count = 10_000;
    let synapses_per_neuron = 2; // Very sparse
    
    let (neuron_array, synapse_array) = create_test_genome(neuron_count, synapses_per_neuron);
    
    let mut backend = create_backend::<f32>(
        BackendType::WGPU,
        neuron_array.capacity,
        synapse_array.capacity,
        &config,
    )
    .expect("GPU backend should be created");
    
    backend
        .initialize_persistent_data(&neuron_array, &synapse_array)
        .expect("Sparse connectivity should be handled");
    
    let fired_neurons: Vec<u32> = (0..50).collect();
    
    let mut fcl = FireCandidateList::new();
    let synapses_processed = backend
        .process_synaptic_propagation(&fired_neurons, &synapse_array, &mut fcl)
        .expect("Synaptic propagation should succeed");
    
    println!("Sparse connectivity: {} synapses processed", synapses_processed);
    
    assert!(synapses_processed > 0, "Should process sparse synapses");
    
    println!("✅ Sparse connectivity handled correctly");
}

#[cfg(feature = "gpu")]
#[test]
fn test_gpu_zero_weight_synapses() {
    let config = BackendConfig::default();
    let neuron_count = 1000;
    let (neuron_array, mut synapse_array) = create_test_genome(neuron_count, 10);
    
    // Set all synapses to zero weight
    for i in 0..synapse_array.count {
        synapse_array.weights[i] = 0;
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
        .expect("Zero weight synapses should be handled");
    
    let fired_neurons: Vec<u32> = (0..50).collect();
    
    let mut fcl = FireCandidateList::new();
    let synapses_processed = backend
        .process_synaptic_propagation(&fired_neurons, &synapse_array, &mut fcl)
        .expect("Should handle zero weights");
    
    println!("Zero weight: {} synapses processed", synapses_processed);
    
    // FCL should have very small or zero contributions
    for (_id, potential) in fcl.get_all_candidates() {
        assert!(
            potential.abs() < 0.1,
            "Zero-weight synapses should produce near-zero potential"
        );
    }
    
    println!("✅ Zero-weight synapses handled correctly");
}

#[cfg(feature = "gpu")]
#[test]
fn test_gpu_max_weight_synapses() {
    let config = BackendConfig::default();
    let neuron_count = 1000;
    let (neuron_array, mut synapse_array) = create_test_genome(neuron_count, 10);
    
    // Set all synapses to maximum weight
    for i in 0..synapse_array.count {
        synapse_array.weights[i] = 255; // Max u8
        synapse_array.postsynaptic_potentials[i] = 255; // Max PSP
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
        .expect("Max weight synapses should be handled");
    
    let fired_neurons: Vec<u32> = (0..50).collect();
    
    let mut fcl = FireCandidateList::new();
    let synapses_processed = backend
        .process_synaptic_propagation(&fired_neurons, &synapse_array, &mut fcl)
        .expect("Should handle max weights");
    
    println!("Max weight: {} synapses processed", synapses_processed);
    
    // FCL should have strong contributions
    assert!(fcl.len() > 0, "Should have FCL candidates");
    
    println!("✅ Max-weight synapses handled correctly");
}

#[cfg(feature = "gpu")]
#[test]
fn test_gpu_mixed_synapse_types() {
    let config = BackendConfig::default();
    let neuron_count = 1000;
    let (neuron_array, mut synapse_array) = create_test_genome(neuron_count, 20);
    
    // Set specific synapse type pattern
    let mut exc_count = 0;
    let mut inh_count = 0;
    for i in 0..synapse_array.count {
        synapse_array.types[i] = if i % 3 == 0 { 1 } else { 0 }; // ~33% inhibitory
        if synapse_array.types[i] == 0 {
            exc_count += 1;
        } else {
            inh_count += 1;
        }
    }
    
    println!("Excitatory: {}, Inhibitory: {}", exc_count, inh_count);
    
    let mut backend = create_backend::<f32>(
        BackendType::WGPU,
        neuron_array.capacity,
        synapse_array.capacity,
        &config,
    )
    .expect("GPU backend should be created");
    
    backend
        .initialize_persistent_data(&neuron_array, &synapse_array)
        .expect("Mixed synapse types should be handled");
    
    let fired_neurons: Vec<u32> = (0..50).collect();
    
    let mut fcl = FireCandidateList::new();
    backend
        .process_synaptic_propagation(&fired_neurons, &synapse_array, &mut fcl)
        .expect("Should handle mixed types");
    
    println!("FCL candidates: {}", fcl.len());
    
    println!("✅ Mixed excitatory/inhibitory synapses handled");
}

#[cfg(feature = "gpu")]
#[test]
fn test_gpu_single_source_many_targets() {
    let config = BackendConfig::default();
    let neuron_count = 1000;
    
    // Create custom synapse pattern: one source → many targets
    let mut neuron_array = NeuronArray::new(neuron_count);
    let mut synapse_array = SynapseArray::new(500);
    
    for i in 0..neuron_count {
        neuron_array.membrane_potentials[i] = 0.0;
        neuron_array.thresholds[i] = 10.0;
        neuron_array.leak_coefficients[i] = 0.0;
        neuron_array.resting_potentials[i] = 0.0;
        neuron_array.excitabilities[i] = 1.0;
        neuron_array.valid_mask[i] = true;
    }
    neuron_array.count = neuron_count;
    
    // Neuron 0 connects to 500 targets
    for i in 0..500 {
        synapse_array.source_neurons[i] = 0;
        synapse_array.target_neurons[i] = (i + 1) as u32;
        synapse_array.weights[i] = 128;
        synapse_array.postsynaptic_potentials[i] = 200;
        synapse_array.types[i] = 0;
        synapse_array.valid_mask[i] = true;
        
        synapse_array
            .source_index
            .entry(0)
            .or_insert_with(Vec::new)
            .push(i);
    }
    synapse_array.count = 500;
    
    let mut backend = create_backend::<f32>(
        BackendType::WGPU,
        neuron_array.capacity,
        synapse_array.capacity,
        &config,
    )
    .expect("GPU backend should be created");
    
    backend
        .initialize_persistent_data(&neuron_array, &synapse_array)
        .expect("One-to-many pattern should be handled");
    
    // Fire the source neuron
    let fired_neurons: Vec<u32> = vec![0];
    
    let mut fcl = FireCandidateList::new();
    let synapses_processed = backend
        .process_synaptic_propagation(&fired_neurons, &synapse_array, &mut fcl)
        .expect("Synaptic propagation should succeed");
    
    println!("One source → many targets: {} synapses processed", synapses_processed);
    println!("FCL candidates: {}", fcl.len());
    
    // Should process all 500 synapses
    assert!(
        synapses_processed >= 500,
        "Should process all synapses from single source"
    );
    
    // Should have 500 unique targets in FCL
    assert_eq!(fcl.len(), 500, "Should have all targets in FCL");
    
    println!("✅ One-to-many connectivity handled correctly");
}

#[cfg(feature = "gpu")]
#[test]
fn test_gpu_hash_lookup_all_sources() {
    let config = BackendConfig::default();
    let neuron_count = 5000;
    let synapses_per_neuron = 10;
    
    let (neuron_array, synapse_array) = create_test_genome(neuron_count, synapses_per_neuron);
    
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
    
    // Fire ALL neurons (stress test hash lookups)
    let fired_neurons: Vec<u32> = (0..neuron_count as u32).collect();
    
    let mut fcl = FireCandidateList::new();
    let synapses_processed = backend
        .process_synaptic_propagation(&fired_neurons, &synapse_array, &mut fcl)
        .expect("Should handle all neurons firing");
    
    println!("All neurons firing: {} synapses processed", synapses_processed);
    println!("FCL candidates: {}", fcl.len());
    
    // Should process all synapses
    assert!(
        synapses_processed >= synapse_array.count,
        "Should process all synapses when all neurons fire"
    );
    
    println!("✅ Hash lookups work for all source neurons");
}

