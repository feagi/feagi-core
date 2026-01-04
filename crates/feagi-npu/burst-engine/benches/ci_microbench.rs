// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! CI Microbenchmarks
//!
//! Purpose:
//! - Provide a fast, stable(ish) microbenchmark pack suitable for CI gating.
//! - Focus on CPU hot paths relevant to real-time behavior.
//!
//! Notes:
//! - Keep runtime low (GitHub-hosted runners are noisy and slower).
//! - Prefer fixed inputs and avoid I/O.

use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use feagi_npu_neural::synapse::SynapseType;
use feagi_npu_neural::types::{FireCandidateList, NeuronId};
use feagi_npu_runtime::std_impl::{NeuronArray, SynapseArray};

fn create_test_genome(neuron_count: usize, synapses_per_neuron: usize) -> (NeuronArray<f32>, SynapseArray) {
    let mut neuron_array = NeuronArray::new(neuron_count);
    let synapse_count = neuron_count * synapses_per_neuron;
    let mut synapse_array = SynapseArray::new(synapse_count);

    // Initialize neurons (simple, deterministic)
    for _i in 0..neuron_count {
        neuron_array.membrane_potentials.push(0.0);
        neuron_array.thresholds.push(10.0);
        neuron_array.leak_coefficients.push(0.1);
        neuron_array.resting_potentials.push(0.0);
        neuron_array.neuron_types.push(0);
        neuron_array.refractory_periods.push(0);
        neuron_array.refractory_countdowns.push(0);
        neuron_array.excitabilities.push(1.0);
        neuron_array.consecutive_fire_counts.push(0);
        neuron_array.consecutive_fire_limits.push(0);
        neuron_array.snooze_periods.push(0);
        neuron_array.mp_charge_accumulation.push(false);
        neuron_array.cortical_areas.push(0);
        neuron_array.coordinates.extend_from_slice(&[0, 0, 0]);
        neuron_array.valid_mask.push(true);
    }
    neuron_array.count = neuron_count;

    // Initialize synapses: deterministic neighborhood connectivity
    let mut synapse_idx = 0;
    for source in 0..neuron_count {
        for offset in 0..synapses_per_neuron {
            let target = (source + offset + 1) % neuron_count;
            if synapse_idx < synapse_count {
                let synapse_type = if offset % 4 == 0 {
                    SynapseType::Inhibitory
                } else {
                    SynapseType::Excitatory
                };
                synapse_array.add_synapse_simple(
                    source as u32,
                    target as u32,
                    128,
                    200,
                    synapse_type,
                );

                synapse_idx += 1;
            }
        }
    }

    (neuron_array, synapse_array)
}

fn generate_fired_neurons(neuron_count: usize, firing_rate: f32) -> Vec<u32> {
    let fire_count = (neuron_count as f32 * firing_rate) as usize;
    if fire_count == 0 {
        return Vec::new();
    }
    (0..fire_count)
        .map(|i| (i * (neuron_count / fire_count)) as u32)
        .collect()
}

fn bench_ci_cpu_backend(c: &mut Criterion) {
    use feagi_npu_burst_engine::backend::{ComputeBackend, CPUBackend};

    let mut group = c.benchmark_group("ci_cpu_backend");
    group.sample_size(20);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(1));

    let neuron_count = 10_000;
    let synapses_per_neuron = 50;
    let (mut neuron_array, synapse_array) = create_test_genome(neuron_count, synapses_per_neuron);
    let fired_neurons = generate_fired_neurons(neuron_count, 0.01);

    group.throughput(Throughput::Elements(neuron_count as u64));

    group.bench_with_input(
        BenchmarkId::new("full_burst", "10k_50syn"),
        &neuron_count,
        |b, _| {
            let mut backend: CPUBackend = CPUBackend::new();
            let mut fcl = FireCandidateList::new();
            b.iter(|| {
                fcl.clear();
                let _ = <CPUBackend as ComputeBackend<f32, NeuronArray<f32>, SynapseArray>>::process_synaptic_propagation(
                    &mut backend,
                    black_box(&fired_neurons),
                    black_box(&synapse_array),
                    black_box(&mut fcl),
                );
                let _ = <CPUBackend as ComputeBackend<f32, NeuronArray<f32>, SynapseArray>>::process_neural_dynamics(
                    &mut backend,
                    black_box(&fcl),
                    black_box(&mut neuron_array),
                    black_box(1),
                );
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::new("synaptic_only", "10k_50syn"),
        &neuron_count,
        |b, _| {
            let mut backend: CPUBackend = CPUBackend::new();
            let mut fcl = FireCandidateList::new();
            b.iter(|| {
                fcl.clear();
                let _ = <CPUBackend as ComputeBackend<f32, NeuronArray<f32>, SynapseArray>>::process_synaptic_propagation(
                    &mut backend,
                    black_box(&fired_neurons),
                    black_box(&synapse_array),
                    black_box(&mut fcl),
                );
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::new("neural_only", "10k_50syn"),
        &neuron_count,
        |b, _| {
            let mut backend: CPUBackend = CPUBackend::new();
            let mut fcl = FireCandidateList::new();
            // Pre-populate FCL deterministically (1% of neurons)
            for i in 0..(neuron_count / 100) {
                fcl.add_candidate(NeuronId(i as u32), 2.0);
            }
            b.iter(|| {
                let _ = <CPUBackend as ComputeBackend<f32, NeuronArray<f32>, SynapseArray>>::process_neural_dynamics(
                    &mut backend,
                    black_box(&fcl),
                    black_box(&mut neuron_array),
                    black_box(1),
                );
            });
        },
    );

    group.finish();
}

fn bench_large_candidate_counts(c: &mut Criterion) {
    use feagi_npu_burst_engine::backend::{ComputeBackend, CPUBackend};

    let mut group = c.benchmark_group("large_candidates");
    group.sample_size(10); // Fewer samples for large tests
    group.warm_up_time(Duration::from_millis(1000));
    group.measurement_time(Duration::from_secs(5));

    // Match production scenario: 8M neurons (512x512x22 area ≈ 5.7M, but use 8M for safety)
    let neuron_count = 8_000_000;
    
    // Create neuron array with proper initialization
    let mut neuron_array = NeuronArray::new(neuron_count);
    for _i in 0..neuron_count {
        neuron_array.membrane_potentials.push(0.0);
        neuron_array.thresholds.push(10.0);
        neuron_array.leak_coefficients.push(0.1);
        neuron_array.resting_potentials.push(0.0);
        neuron_array.neuron_types.push(0);
        neuron_array.refractory_periods.push(0);
        neuron_array.refractory_countdowns.push(0);
        neuron_array.excitabilities.push(1.0);
        neuron_array.consecutive_fire_counts.push(0);
        neuron_array.consecutive_fire_limits.push(0);
        neuron_array.snooze_periods.push(0);
        neuron_array.mp_charge_accumulation.push(false);
        neuron_array.cortical_areas.push(0);
        neuron_array.coordinates.extend_from_slice(&[0, 0, 0]);
        neuron_array.valid_mask.push(true);
    }
    neuron_array.count = neuron_count;

    let candidate_counts = vec![
        100_000,      // 100k candidates
        500_000,      // 500k candidates
        1_000_000,    // 1M candidates
        2_500_000,    // 2.5M candidates (production scenario)
    ];

    for candidate_count in candidate_counts {
        group.bench_with_input(
            BenchmarkId::new("neural_dynamics", format!("{}_candidates", candidate_count)),
            &candidate_count,
            |b, &count| {
                let mut backend: CPUBackend = CPUBackend::new();
                let mut fcl = FireCandidateList::new();
                
                // Create sparse candidate distribution (realistic scenario)
                // Distribute candidates across the neuron array to simulate real-world sparse patterns
                for i in 0..count {
                    // Use modulo to distribute candidates across the neuron array
                    let neuron_id = ((i as u64 * neuron_count as u64) / count as u64) as u32;
                    // Use small contribution (0.5) to simulate realistic synaptic inputs
                    fcl.add_candidate(NeuronId(neuron_id), 0.5);
                }
                
                b.iter(|| {
                    let _ = <CPUBackend as ComputeBackend<f32, NeuronArray<f32>, SynapseArray>>::process_neural_dynamics(
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

fn criterion_config() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(1))
        .sample_size(20)
}

criterion_group! {
    name = ci_microbench;
    config = criterion_config();
    targets = bench_ci_cpu_backend, bench_large_candidate_counts
}
criterion_main!(ci_microbench);


