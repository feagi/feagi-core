// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! # Backend Performance Benchmarks
//!
//! Validates that GPU acceleration provides expected benefits for large genomes.
//! Tests multiple genome sizes to find the crossover point where GPU becomes beneficial.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use feagi_npu_neural::types::{FireCandidateList, NeuronArray, NeuronId, SynapseArray};

/// Create a test genome with specified size
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
        neuron_array.refractory_periods[i] = 0;
        neuron_array.refractory_countdowns[i] = 0;
        neuron_array.consecutive_fire_limits[i] = u16::MAX; // MAX = no limit (SIMD-friendly encoding)
        neuron_array.consecutive_fire_counts[i] = 0;
        neuron_array.valid_mask[i] = true;
    }
    neuron_array.count = neuron_count;

    // Initialize synapses (fully connected within small neighborhoods)
    let mut synapse_idx = 0;
    for source in 0..neuron_count {
        for i in 0..synapses_per_neuron {
            let target = (source + i + 1) % neuron_count;
            if synapse_idx < synapse_count {
                synapse_array.source_neurons[synapse_idx] = source as u32;
                synapse_array.target_neurons[synapse_idx] = target as u32;
                synapse_array.weights[synapse_idx] = 128;
                synapse_array.postsynaptic_potentials[synapse_idx] = 200;
                synapse_array.types[synapse_idx] = if i % 4 == 0 { 1 } else { 0 }; // 75% excitatory
                synapse_array.valid_mask[synapse_idx] = true;

                // Build index
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

/// Generate fired neurons (simulate 1% firing rate)
fn generate_fired_neurons(neuron_count: usize, firing_rate: f32) -> Vec<u32> {
    let fire_count = (neuron_count as f32 * firing_rate) as usize;
    (0..fire_count)
        .map(|i| (i * (neuron_count / fire_count)) as u32)
        .collect()
}

/// Benchmark CPU backend
fn bench_cpu_backend(c: &mut Criterion) {
    use feagi_npu_burst_engine::backend::{CPUBackend, ComputeBackend};

    let mut group = c.benchmark_group("cpu_backend");

    let test_sizes = vec![
        (10_000, "10K"),
        (50_000, "50K"),
        (100_000, "100K"),
        (500_000, "500K"),
    ];

    for (neuron_count, label) in test_sizes {
        let synapses_per_neuron = 100;
        let (mut neuron_array, synapse_array) =
            create_test_genome(neuron_count, synapses_per_neuron);
        let fired_neurons = generate_fired_neurons(neuron_count, 0.01);

        group.throughput(Throughput::Elements(neuron_count as u64));

        // Full burst (synaptic + neural)
        group.bench_with_input(
            BenchmarkId::new("full_burst", label),
            &neuron_count,
            |b, _| {
                let mut backend: CPUBackend = CPUBackend::new();
                let mut fcl = FireCandidateList::new();
                b.iter(|| {
                    fcl.clear();
                    let _ = <CPUBackend as ComputeBackend<f32>>::process_synaptic_propagation(
                        &mut backend,
                        black_box(&fired_neurons),
                        black_box(&synapse_array),
                        black_box(&mut fcl),
                    );
                    let _ = <CPUBackend as ComputeBackend<f32>>::process_neural_dynamics(
                        &mut backend,
                        black_box(&fcl),
                        black_box(&mut neuron_array),
                        black_box(1),
                    );
                });
            },
        );

        // Synaptic propagation only
        group.bench_with_input(
            BenchmarkId::new("synaptic_only", label),
            &neuron_count,
            |b, _| {
                let mut backend: CPUBackend = CPUBackend::new();
                let mut fcl = FireCandidateList::new();
                b.iter(|| {
                    fcl.clear();
                    let _ = <CPUBackend as ComputeBackend<f32>>::process_synaptic_propagation(
                        &mut backend,
                        black_box(&fired_neurons),
                        black_box(&synapse_array),
                        black_box(&mut fcl),
                    );
                });
            },
        );

        // Neural dynamics only
        group.bench_with_input(
            BenchmarkId::new("neural_only", label),
            &neuron_count,
            |b, _| {
                let mut backend: CPUBackend = CPUBackend::new();
                let mut fcl = FireCandidateList::new();
                // Pre-populate FCL
                for i in 0..neuron_count / 100 {
                    fcl.add_candidate(NeuronId(i as u32), 2.0);
                }
                b.iter(|| {
                    let _ = <CPUBackend as ComputeBackend<f32>>::process_neural_dynamics(
                        &mut backend,
                        black_box(&fcl),
                        black_box(&mut neuron_array),
                        black_box(1),
                    );
                });
            },
        );
    }

    group.finish();
}

/// Benchmark GPU backend (if available)
#[cfg(feature = "gpu")]
fn bench_gpu_backend(c: &mut Criterion) {
    use feagi_npu_burst_engine::backend::{ComputeBackend, WGPUBackend};

    // Check if GPU is available
    if !is_gpu_available() {
        println!("⚠️  GPU not available, skipping GPU benchmarks");
        return;
    }

    let mut group = c.benchmark_group("gpu_backend");

    // GPU limits (Metal on macOS):
    // - max_buffer_size: 256MB total
    // - max_storage_buffer_binding_size: 128MB per binding
    // Synapse data: 3 u32 per synapse = 12 bytes per synapse
    // Max synapses: 128MB / 12 bytes ≈ 10.6M synapses safe limit
    const GPU_MAX_SYNAPSES: usize = 10_000_000;

    let test_sizes = vec![
        (10_000, 100, "10K_100syn"),   // 1M synapses
        (50_000, 100, "50K_100syn"),   // 5M synapses
        (100_000, 100, "100K_100syn"), // 10M synapses (at binding limit)
        (250_000, 40, "250K_40syn"),   // 10M synapses (different density)
        (500_000, 20, "500K_20syn"),   // 10M synapses (sparse)
    ];

    for (neuron_count, synapses_per_neuron, label) in test_sizes {
        let total_synapses = neuron_count * synapses_per_neuron;

        if total_synapses > GPU_MAX_SYNAPSES {
            println!(
                "⚠️  Skipping {} - exceeds GPU buffer limit ({} > {})",
                label, total_synapses, GPU_MAX_SYNAPSES
            );
            continue;
        }

        let (mut neuron_array, synapse_array) =
            create_test_genome(neuron_count, synapses_per_neuron);
        let fired_neurons = generate_fired_neurons(neuron_count, 0.01);

        println!(
            "📊 GPU Benchmark: {} neurons, {} synapses/neuron = {} total synapses",
            neuron_count, synapses_per_neuron, total_synapses
        );

        // Create GPU backend
        let mut backend = match WGPUBackend::new(neuron_count * 2, total_synapses) {
            Ok(b) => b,
            Err(e) => {
                println!("⚠️  Failed to create GPU backend for {}: {:?}", label, e);
                continue;
            }
        };

        // Initialize persistent data (one-time cost)
        if let Err(e) = backend.initialize_persistent_data(&neuron_array, &synapse_array) {
            println!("⚠️  Failed to initialize GPU data for {}: {:?}", label, e);
            continue;
        }

        group.throughput(Throughput::Elements(neuron_count as u64));

        // Full burst (synaptic + neural)
        group.bench_with_input(
            BenchmarkId::new("full_burst", label),
            &neuron_count,
            |b, _| {
                let mut fcl = FireCandidateList::new();
                b.iter(|| {
                    fcl.clear();
                    let _ = backend.process_synaptic_propagation(
                        black_box(&fired_neurons),
                        black_box(&synapse_array),
                        black_box(&mut fcl),
                    );
                    let fcl_size = fcl.len();
                    let _ = backend.process_neural_dynamics(
                        black_box(&fcl),
                        black_box(&mut neuron_array),
                        black_box(1),
                    );
                    black_box(fcl_size)
                });
            },
        );
    }

    group.finish();
}

#[cfg(feature = "gpu")]
fn is_gpu_available() -> bool {
    use wgpu::Backends;

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: Backends::all(),
        ..Default::default()
    });

    pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .is_some()
}

/// Compare CPU vs GPU for different genome sizes
#[cfg(feature = "gpu")]
fn bench_cpu_vs_gpu_comparison(c: &mut Criterion) {
    use feagi_npu_burst_engine::backend::{CPUBackend, ComputeBackend, WGPUBackend};

    if !is_gpu_available() {
        println!("⚠️  GPU not available, skipping comparison benchmarks");
        return;
    }

    let mut group = c.benchmark_group("cpu_vs_gpu");

    const GPU_MAX_SYNAPSES: usize = 10_000_000;

    // Adjusted to stay within GPU buffer limits (128MB binding limit)
    let test_sizes = vec![
        (10_000, 100, "10K"),   // 1M synapses
        (100_000, 100, "100K"), // 10M synapses (at GPU binding limit)
        (250_000, 40, "250K"),  // 10M synapses
        (500_000, 20, "500K"),  // 10M synapses (sparse)
    ];

    for (neuron_count, synapses_per_neuron, label) in test_sizes {
        let total_synapses = neuron_count * synapses_per_neuron;

        if total_synapses > GPU_MAX_SYNAPSES {
            println!("⚠️  Skipping {} - exceeds GPU buffer limit", label);
            continue;
        }

        let (mut neuron_array_cpu, synapse_array) =
            create_test_genome(neuron_count, synapses_per_neuron);
        let mut neuron_array_gpu = neuron_array_cpu.clone();
        let fired_neurons = generate_fired_neurons(neuron_count, 0.01);

        group.throughput(Throughput::Elements(neuron_count as u64));

        // CPU benchmark
        group.bench_with_input(BenchmarkId::new("cpu", label), &neuron_count, |b, _| {
            let mut backend: CPUBackend = CPUBackend::new();
            let mut fcl = FireCandidateList::new();
            b.iter(|| {
                fcl.clear();
                let _ = <CPUBackend as ComputeBackend<f32>>::process_synaptic_propagation(
                    &mut backend,
                    black_box(&fired_neurons),
                    black_box(&synapse_array),
                    black_box(&mut fcl),
                );
                let fcl_size = fcl.len();
                let _ = <CPUBackend as ComputeBackend<f32>>::process_neural_dynamics(
                    &mut backend,
                    black_box(&fcl),
                    black_box(&mut neuron_array_cpu),
                    black_box(1),
                );
                black_box(fcl_size)
            });
        });

        // GPU benchmark
        if let Ok(mut backend) = WGPUBackend::new(neuron_count * 2, total_synapses) {
            if backend
                .initialize_persistent_data(&neuron_array_gpu, &synapse_array)
                .is_ok()
            {
                group.bench_with_input(BenchmarkId::new("gpu", label), &neuron_count, |b, _| {
                    let mut fcl = FireCandidateList::new();
                    b.iter(|| {
                        fcl.clear();
                        let _ = backend.process_synaptic_propagation(
                            black_box(&fired_neurons),
                            black_box(&synapse_array),
                            black_box(&mut fcl),
                        );
                        let fcl_size = fcl.len();
                        let _ = backend.process_neural_dynamics(
                            black_box(&fcl),
                            black_box(&mut neuron_array_gpu),
                            black_box(1),
                        );
                        black_box(fcl_size)
                    });
                });
            } else {
                println!("⚠️  Failed to initialize GPU data for {}", label);
            }
        } else {
            println!("⚠️  Failed to create GPU backend for {}", label);
        }
    }

    group.finish();
}

/// Test backend auto-selection logic
fn bench_auto_selection(c: &mut Criterion) {
    use feagi_npu_burst_engine::backend::{select_backend, BackendConfig};

    let mut group = c.benchmark_group("auto_selection");

    let test_cases = vec![
        (10_000, 1_000_000, "small_genome"),
        (100_000, 10_000_000, "medium_genome"),
        (500_000, 50_000_000, "large_genome"),
        (1_000_000, 100_000_000, "xlarge_genome"),
    ];

    for (neuron_count, synapse_count, label) in test_cases {
        group.bench_with_input(
            BenchmarkId::new("select_backend", label),
            &(neuron_count, synapse_count),
            |b, &(neurons, synapses)| {
                let config = BackendConfig::default();
                b.iter(|| black_box(select_backend(neurons, synapses, &config)));
            },
        );
    }

    group.finish();
}

/// Benchmark different firing rates (CPU backend)
fn bench_firing_rate_cpu(c: &mut Criterion) {
    use feagi_npu_burst_engine::backend::{CPUBackend, ComputeBackend};

    let mut group = c.benchmark_group("firing_rate_cpu");

    // Test different firing rates at fixed genome size
    let neuron_count = 100_000;
    let synapses_per_neuron = 100;
    let (mut neuron_array, synapse_array) = create_test_genome(neuron_count, synapses_per_neuron);

    let firing_rates = vec![
        (0.001, "0.1%"), // 100 neurons fire
        (0.01, "1%"),    // 1,000 neurons fire
        (0.05, "5%"),    // 5,000 neurons fire
        (0.10, "10%"),   // 10,000 neurons fire
        (0.25, "25%"),   // 25,000 neurons fire
    ];

    for (rate, label) in firing_rates {
        let fired_neurons = generate_fired_neurons(neuron_count, rate);
        let fire_count = fired_neurons.len();

        group.throughput(Throughput::Elements(fire_count as u64));
        group.bench_with_input(BenchmarkId::new("full_burst", label), &rate, |b, _| {
            let mut backend: CPUBackend = CPUBackend::new();
            let mut fcl = FireCandidateList::new();
            b.iter(|| {
                fcl.clear();
                let _ = <CPUBackend as ComputeBackend<f32>>::process_synaptic_propagation(
                    &mut backend,
                    black_box(&fired_neurons),
                    black_box(&synapse_array),
                    black_box(&mut fcl),
                );
                let fcl_size = fcl.len();
                let _ = <CPUBackend as ComputeBackend<f32>>::process_neural_dynamics(
                    &mut backend,
                    black_box(&fcl),
                    black_box(&mut neuron_array),
                    black_box(1),
                );
                // Return FCL size for measurement
                black_box(fcl_size)
            });
        });
    }

    group.finish();
}

/// Benchmark different firing rates (GPU backend)
#[cfg(feature = "gpu")]
fn bench_firing_rate_gpu(c: &mut Criterion) {
    use feagi_npu_burst_engine::backend::{ComputeBackend, WGPUBackend};

    if !is_gpu_available() {
        println!("⚠️  GPU not available, skipping firing rate GPU benchmarks");
        return;
    }

    let mut group = c.benchmark_group("firing_rate_gpu");

    // Test different firing rates at fixed genome size (smaller due to buffer limits)
    let neuron_count = 50_000;
    let synapses_per_neuron = 100;
    let (mut neuron_array, synapse_array) = create_test_genome(neuron_count, synapses_per_neuron);

    // Create GPU backend once
    let synapse_count = neuron_count * synapses_per_neuron;
    let mut backend = match WGPUBackend::new(neuron_count * 2, synapse_count) {
        Ok(b) => b,
        Err(_) => {
            println!("⚠️  Failed to create GPU backend for firing rate tests");
            return;
        }
    };

    if backend
        .initialize_persistent_data(&neuron_array, &synapse_array)
        .is_err()
    {
        println!("⚠️  Failed to initialize GPU data for firing rate tests");
        return;
    }

    let firing_rates = vec![
        (0.001, "0.1%"), // 50 neurons fire
        (0.01, "1%"),    // 500 neurons fire
        (0.05, "5%"),    // 2,500 neurons fire
        (0.10, "10%"),   // 5,000 neurons fire
        (0.25, "25%"),   // 12,500 neurons fire
    ];

    for (rate, label) in firing_rates {
        let fired_neurons = generate_fired_neurons(neuron_count, rate);
        let fire_count = fired_neurons.len();

        group.throughput(Throughput::Elements(fire_count as u64));
        group.bench_with_input(BenchmarkId::new("full_burst", label), &rate, |b, _| {
            let mut fcl = FireCandidateList::new();
            b.iter(|| {
                fcl.clear();
                let _ = backend.process_synaptic_propagation(
                    black_box(&fired_neurons),
                    black_box(&synapse_array),
                    black_box(&mut fcl),
                );
                let fcl_size = fcl.len();
                let _ = backend.process_neural_dynamics(
                    black_box(&fcl),
                    black_box(&mut neuron_array),
                    black_box(1),
                );
                black_box(fcl_size)
            });
        });
    }

    group.finish();
}

#[cfg(feature = "gpu")]
criterion_group!(
    benches,
    bench_cpu_backend,
    bench_gpu_backend,
    bench_cpu_vs_gpu_comparison,
    bench_auto_selection,
    bench_firing_rate_cpu,
    bench_firing_rate_gpu,
);

#[cfg(not(feature = "gpu"))]
criterion_group!(
    benches,
    bench_cpu_backend,
    bench_auto_selection,
    bench_firing_rate_cpu,
);

criterion_main!(benches);
