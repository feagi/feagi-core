// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Timing baseline for a windowed memory area: one fixed workload of bursts, each followed by
//! one synchronous plasticity pass (`PlasticityService::run_burst_sync`).
//!
//! Workload: 64 upstream neurons, 8 fixed patterns of 8 neurons cycled one per burst, a memory
//! area of temporal depth 3, 200 bursts per iteration. Rig setup is excluded from the timing.
//! Run before and after a memory change, on the same machine:
//!
//! ```text
//! cargo bench -p feagi-npu-plasticity --bench memory_burst
//! ```

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::{DynamicNPU, TracingMutex};
use feagi_npu_neural::types::NeuronId;
use feagi_npu_plasticity::{
    MemoryMpMode, MemoryNeuronLifecycleConfig, PlasticityCommand, PlasticityConfig,
    PlasticityService,
};
use feagi_npu_runtime::StdRuntime;
use feagi_structures::genomic::cortical_area::CorticalID;
use std::sync::Arc;

const UPSTREAM_IDX: u32 = 7;
const MEMORY_IDX: u32 = 100;
const UPSTREAM_NEURONS: u32 = 64;
const PATTERNS: usize = 8;
const PATTERN_WIDTH: usize = 8;
const BURSTS: usize = 200;

struct Rig {
    npu: Arc<TracingMutex<DynamicNPU>>,
    service: PlasticityService,
    upstream: Vec<u32>,
    memory_cortical_id: CorticalID,
}

fn build_rig() -> Rig {
    let npu = Arc::new(TracingMutex::new(
        DynamicNPU::new_f32(StdRuntime::new(), CPUBackend::new(), 4096, 4096, 8).unwrap(),
        "memory-burst-bench",
    ));
    let upstream_id = CorticalID::try_from_bytes(b"csrc0001").unwrap();
    let memory_cortical_id = CorticalID::try_from_bytes(b"mmem0001").unwrap();
    let upstream = {
        let mut guard = npu.lock().unwrap();
        guard.register_cortical_area(UPSTREAM_IDX, upstream_id.as_base_64());
        guard.register_cortical_area(MEMORY_IDX, memory_cortical_id.as_base_64());
        guard
            .create_cortical_area_neurons(
                UPSTREAM_IDX,
                UPSTREAM_NEURONS,
                1,
                1,
                1,
                0.1,
                0.0,
                0.0,
                0.0,
                f32::MAX,
                0.0,
                0.0,
                0,
                0,
                1.0,
                0,
                0,
                false,
            )
            .unwrap();
        guard.get_neurons_in_cortical_area(UPSTREAM_IDX)
    };
    let service = PlasticityService::new(
        PlasticityConfig::default(),
        feagi_npu_plasticity::create_memory_stats_cache(),
        Arc::clone(&npu),
    );
    assert!(service.register_memory_area(
        MEMORY_IDX,
        memory_cortical_id.as_base_64(),
        3,
        vec![UPSTREAM_IDX],
        Some(MemoryNeuronLifecycleConfig::default()),
        MemoryMpMode::PatternOnly,
    ));
    Rig {
        npu,
        service,
        upstream,
        memory_cortical_id,
    }
}

fn run_workload(rig: &Rig) -> usize {
    let mut created = 0usize;
    for burst in 0..BURSTS {
        let pattern = burst % PATTERNS;
        let stim: Vec<(NeuronId, f32)> = rig.upstream
            [pattern * PATTERN_WIDTH..(pattern + 1) * PATTERN_WIDTH]
            .iter()
            .map(|id| (NeuronId(*id), 10.0))
            .collect();
        let result = {
            let mut guard = rig.npu.lock().unwrap();
            guard.inject_sensory_with_potentials(&stim);
            guard.process_burst().unwrap()
        };
        rig.service.run_burst_sync(result.burst);
        let commands = rig.service.drain_commands();
        let mut guard = rig.npu.lock().unwrap();
        for command in commands {
            match command {
                PlasticityCommand::RegisterMemoryNeuron { neuron_id, .. } => {
                    created += 1;
                    guard.register_dynamic_neuron_mapping(neuron_id, rig.memory_cortical_id);
                }
                PlasticityCommand::InjectMemoryNeuronToFCL {
                    neuron_id,
                    area_idx,
                    membrane_potential,
                    ..
                } => guard.inject_memory_neuron_to_fcl(neuron_id, area_idx, membrane_potential),
                _ => {}
            }
        }
    }
    created
}

fn memory_burst(c: &mut Criterion) {
    c.bench_function("memory_depth3_200_bursts", |b| {
        b.iter_batched(build_rig, |rig| run_workload(&rig), BatchSize::LargeInput)
    });
}

criterion_group!(benches, memory_burst);
criterion_main!(benches);
