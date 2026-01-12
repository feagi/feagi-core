//! Large-Scale Performance Test
//!
//! Tests the optimized propagation engine with realistic workloads:
//! - Pre-computed source neuron metadata (reduces HashMap lookups)
//! - FCL batch insertion with pre-allocation
//!
//! Run with: cargo bench --bench largescale_perf_test

use ahash::AHashMap;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use feagi_npu_burst_engine::synaptic_propagation::SynapticPropagationEngine;
use feagi_npu_neural::types::{FireCandidateList, NeuronId};
use feagi_npu_runtime::std_impl::{NeuronArray, SynapseArray};
use feagi_structures::genomic::cortical_area::CorticalID;

/// Create a large-scale test genome
fn create_large_genome(
    neuron_count: usize,
    synapses_per_neuron: usize,
) -> (NeuronArray<f32>, SynapseArray) {
    let mut neuron_array = NeuronArray::new(neuron_count);
    let synapse_count = neuron_count * synapses_per_neuron;
    let mut synapse_array = SynapseArray::new(synapse_count);

    // Initialize neurons
    for _i in 0..neuron_count {
        neuron_array.membrane_potentials.push(0.0);
        neuron_array.thresholds.push(10.0);
        neuron_array.leak_coefficients.push(0.1);
        neuron_array.resting_potentials.push(0.0);
        neuron_array.excitabilities.push(1.0);
        neuron_array.refractory_periods.push(0);
        neuron_array.refractory_countdowns.push(0);
        neuron_array.consecutive_fire_limits.push(u16::MAX);
        neuron_array.consecutive_fire_counts.push(0);
        neuron_array.valid_mask.push(true);
    }
    neuron_array.count = neuron_count;

    // Initialize synapses
    let mut synapse_idx = 0;
    for source in 0..neuron_count {
        for i in 0..synapses_per_neuron {
            let target = (source + i + 1) % neuron_count;
            if synapse_idx < synapse_count {
                synapse_array.add_synapse_simple(
                    source as u32,
                    target as u32,
                    128,
                    200,
                    if i % 4 == 0 {
                        feagi_npu_neural::synapse::SynapseType::Inhibitory
                    } else {
                        feagi_npu_neural::synapse::SynapseType::Excitatory
                    },
                );
                synapse_idx += 1;
            }
        }
    }

    (neuron_array, synapse_array)
}

/// Generate fired neurons (simulate 1% firing rate)
fn generate_fired_neurons(neuron_count: usize, firing_rate: f32) -> Vec<NeuronId> {
    let fire_count = (neuron_count as f32 * firing_rate) as usize;
    (0..fire_count)
        .map(|i| NeuronId((i * (neuron_count / fire_count.max(1))) as u32))
        .collect()
}

fn bench_largescale_propagation(c: &mut Criterion) {
    use std::time::Instant;

    let mut group = c.benchmark_group("largescale_propagation");
    group.sample_size(10);
    group.warm_up_time(std::time::Duration::from_millis(1000));

    // Match user's actual scale: 8M neurons, ~1 synapse/neuron average
    let test_sizes = vec![
        (100_000, 100, "100K_100syn"), // 10M synapses
        (500_000, 50, "500K_50syn"),   // 25M synapses
        (1_000_000, 10, "1M_10syn"),   // 10M synapses
        (5_000_000, 2, "5M_2syn"),     // 10M synapses
        (8_000_000, 1, "8M_1syn"),     // 8M synapses (matches user's scale)
    ];

    for (neuron_count, synapses_per_neuron, label) in test_sizes {
        let total_synapses = neuron_count * synapses_per_neuron;

        println!(
            "\n📊 Preparing test: {} neurons, {} synapses/neuron = {} total synapses",
            neuron_count, synapses_per_neuron, total_synapses
        );

        let (neuron_array, synapse_array) = create_large_genome(neuron_count, synapses_per_neuron);
        let fired_neurons = generate_fired_neurons(neuron_count, 0.01);

        println!(
            "   ✅ Created genome: {} neurons, {} synapses",
            neuron_array.count, synapse_array.count
        );
        println!("   ✅ Generated {} fired neurons", fired_neurons.len());

        // Build propagation engine
        let mut engine = SynapticPropagationEngine::new();

        // Build synapse index
        engine.build_synapse_index(&synapse_array);
        println!("   ✅ Built synapse index");

        // Create neuron-to-area mapping (all neurons in same area for simplicity)
        let mut neuron_to_area = AHashMap::new();
        // Create a valid CorticalID (custom type: starts with 'c' = 0x63)
        // Format: [type_byte][7 more bytes] where type_byte must match a valid pattern
        let mut area_bytes = [0u8; 8];
        area_bytes[0] = b'c'; // 'c' = custom cortical area type
        area_bytes[1] = 1;
        let area_id = CorticalID::try_from_bytes(&area_bytes).expect("Failed to create CorticalID");
        for i in 0..neuron_count {
            neuron_to_area.insert(NeuronId(i as u32), area_id.clone());
        }
        engine.set_neuron_mapping(neuron_to_area);
        println!("   ✅ Set neuron mapping");

        // Set area flags (all false for simplicity)
        let mut mp_flags = AHashMap::new();
        let mut uniform_flags = AHashMap::new();
        mp_flags.insert(area_id.clone(), false);
        uniform_flags.insert(area_id.clone(), false);
        engine.set_mp_driven_psp_flags(mp_flags);
        engine.set_psp_uniform_distribution_flags(uniform_flags);
        println!("   ✅ Set area flags");

        group.throughput(Throughput::Elements(total_synapses as u64));

        // Test full propagation (the optimized path)
        group.bench_with_input(
            BenchmarkId::new("propagation_with_optimizations", label),
            &total_synapses,
            |b, _| {
                let neuron_membrane_potentials = AHashMap::new();
                let mut first_iter = true;
                b.iter(|| {
                    let start = Instant::now();
                    let result = engine.propagate(
                        black_box(&fired_neurons),
                        black_box(&synapse_array),
                        black_box(&neuron_membrane_potentials),
                    );
                    let elapsed = start.elapsed();

                    // Log timing for first iteration
                    if first_iter {
                        println!(
                            "   ⏱️  First iteration: {:.2}ms",
                            elapsed.as_secs_f64() * 1000.0
                        );
                        first_iter = false;
                    }

                    black_box(result)
                });
            },
        );

        println!("   ✅ Completed benchmark for {}\n", label);
    }

    group.finish();
}

fn bench_fcl_batch_insertion(c: &mut Criterion) {
    let mut group = c.benchmark_group("fcl_batch_insertion");

    let test_sizes = vec![
        (100_000, "100K"),
        (1_000_000, "1M"),
        (5_000_000, "5M"),
        (8_000_000, "8M"), // Match user's scale
    ];

    for (candidate_count, label) in test_sizes {
        group.throughput(Throughput::Elements(candidate_count as u64));

        group.bench_with_input(
            BenchmarkId::new("batch_insert", label),
            &candidate_count,
            |b, &count| {
                b.iter(|| {
                    let mut fcl = FireCandidateList::new();

                    // Pre-allocate (optimization)
                    if count > 100_000 {
                        fcl.reserve(count / 10);
                    }

                    // Create batch of candidates
                    let candidates: Vec<(NeuronId, f32)> =
                        (0..count).map(|i| (NeuronId(i as u32), 2.0)).collect();

                    // Batch insertion (optimized path)
                    fcl.add_candidates_batch(&candidates);

                    black_box(fcl)
                });
            },
        );

        // Compare with old method (individual insertions)
        group.bench_with_input(
            BenchmarkId::new("individual_insert", label),
            &candidate_count,
            |b, &count| {
                b.iter(|| {
                    let mut fcl = FireCandidateList::new();

                    for i in 0..count {
                        fcl.add_candidate(NeuronId(i as u32), 2.0);
                    }

                    black_box(fcl)
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_largescale_propagation,
    bench_fcl_batch_insertion,
);

criterion_main!(benches);
