// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0
//! Memory-area neurons that fire must appear in the fire-queue sample of that burst.

use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::RustNPU;
use feagi_npu_burst_engine::{FIRE_KIND_EPISODIC_MEMORY, FIRE_KIND_STDP_ELIGIBLE};
use feagi_npu_runtime::StdRuntime;
use feagi_structures::genomic::cortical_area::CoreCorticalType;

const MEMORY_NEURON_ID_START: u32 = 50_000_000;
const MEMORY_AREA_IDX: u32 = 20;

fn npu_with_memory_area() -> RustNPU<StdRuntime, f32, CPUBackend> {
    let mut npu = RustNPU::new(StdRuntime, CPUBackend::new(), 100, 1000, 10).unwrap();
    npu.register_cortical_area(0, CoreCorticalType::Death.to_cortical_id().as_base_64());
    npu.register_cortical_area(1, CoreCorticalType::Power.to_cortical_id().as_base_64());
    npu.register_cortical_area(
        MEMORY_AREA_IDX,
        CoreCorticalType::Death.to_cortical_id().as_base_64(),
    );
    npu
}

#[test]
fn memory_neuron_fire_is_in_the_burst_sample() {
    for (offset, kind) in [
        (0u32, FIRE_KIND_EPISODIC_MEMORY),
        (1u32, FIRE_KIND_STDP_ELIGIBLE),
    ] {
        let mut npu = npu_with_memory_area();
        let id = MEMORY_NEURON_ID_START + offset;
        npu.inject_memory_neuron_to_fcl_with_kind(id, MEMORY_AREA_IDX, 1.5, kind);
        let result = npu.process_burst().unwrap();

        let sample = result
            .fire_queue_sample
            .expect("a burst with a memory fire must produce a sample");
        let (ids, xs, ys, zs, potentials) = sample
            .get(&MEMORY_AREA_IDX)
            .expect("memory area must be present in the sample");
        assert_eq!(ids, &vec![id]);
        assert_eq!(potentials.len(), 1);
        // Memory neurons live outside neuron storage; their voxel is the area origin.
        assert_eq!((xs[0], ys[0], zs[0]), (0, 0, 0));
    }
}

#[test]
fn memory_fire_lasts_one_burst_in_the_sample() {
    let mut npu = npu_with_memory_area();
    npu.inject_memory_neuron_to_fcl(MEMORY_NEURON_ID_START, MEMORY_AREA_IDX, 1.5);
    let fired = npu.process_burst().unwrap();
    assert!(fired
        .fire_queue_sample
        .is_some_and(|s| s.contains_key(&MEMORY_AREA_IDX)));
    // Nothing re-injects it, so the next burst's sample no longer contains the area.
    let next = npu.process_burst().unwrap();
    assert!(next
        .fire_queue_sample
        .is_none_or(|s| !s.contains_key(&MEMORY_AREA_IDX)));
}
