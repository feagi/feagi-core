// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Cost of the episode-link filter in `propagate_delayed`.
//!
//! The filter reads an edge flag only for memory-neuron sources, and only after some area has
//! registered a `sequential_memory` loop (`sequential_links_possible`). This bench fires 2,000
//! ordinary neurons (10 synapses each) with the switch off and on: the two must match, because
//! an area without capture never reaches the flag check.
//!
//! ```text
//! cargo bench -p feagi-npu-burst-engine --bench propagation_filter
//! ```

use ahash::AHashMap;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use feagi_npu_burst_engine::synaptic_propagation::SynapticPropagationEngine;
use feagi_npu_neural::synapse::SynapseType;
use feagi_npu_neural::types::NeuronId;
use feagi_npu_runtime::std_impl::SynapseArray;
use feagi_structures::genomic::cortical_area::CoreCorticalType;

const NEURONS: u32 = 20_000;
const FIRED: u32 = 2_000;
const PER_NEURON: u32 = 10;

fn rig() -> (SynapseArray, SynapticPropagationEngine, Vec<NeuronId>) {
    let mut synapses = SynapseArray::new((NEURONS * PER_NEURON) as usize);
    for source in 0..NEURONS {
        for offset in 1..=PER_NEURON {
            synapses.add_synapse_simple(
                source,
                (source + offset) % NEURONS,
                1.0,
                1.0,
                SynapseType::Excitatory,
            );
        }
    }
    let mut engine = SynapticPropagationEngine::new();
    engine.build_synapse_index(&synapses);
    let area = CoreCorticalType::Power.to_cortical_id();
    engine.set_neuron_mapping((0..NEURONS).map(|id| (NeuronId(id), area)).collect());
    let fired = (0..FIRED)
        .map(|i| NeuronId(i * (NEURONS / FIRED)))
        .collect();
    (synapses, engine, fired)
}

fn bench(c: &mut Criterion) {
    let (synapses, mut engine, fired) = rig();
    let mps = AHashMap::new();
    let mut group = c.benchmark_group("propagation_filter_2000_fired_x10");
    for (name, on) in [("no_loop_in_brain", false), ("loop_elsewhere", true)] {
        engine.sequential_links_possible = on;
        group.bench_function(name, |b| {
            b.iter(|| {
                black_box(
                    engine
                        .propagate_delayed(black_box(&fired), &synapses, &mps)
                        .unwrap(),
                )
            })
        });
    }
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
