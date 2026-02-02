// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0
//! Memory replay tests for NPU replay scheduling.

use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::npu::MemoryReplayFrame;
use feagi_npu_burst_engine::RustNPU;
use feagi_npu_neural::types::NeuronId;
use feagi_npu_runtime::StdRuntime;
use feagi_structures::genomic::cortical_area::{CoreCorticalType, CorticalID};

/// Create a minimal NPU instance for replay testing.
fn create_npu() -> RustNPU<StdRuntime, f32, CPUBackend> {
    let runtime = StdRuntime;
    let backend = CPUBackend::new();
    RustNPU::new(runtime, backend, 100, 1000, 10).expect("Failed to create NPU")
}

/// Add a single neuron at the given cortical area and coordinates.
fn add_neuron_at(
    npu: &mut RustNPU<StdRuntime, f32, CPUBackend>,
    cortical_idx: u32,
    x: u32,
    y: u32,
    z: u32,
) -> NeuronId {
    npu.add_neuron(
        1.0,
        f32::MAX,
        0.1,
        0.0,
        0,
        5,
        1.0,
        u16::MAX,
        0,
        true,
        cortical_idx,
        x,
        y,
        z,
    )
    .expect("Failed to add neuron")
}

#[test]
fn test_memory_replay_fires_twin_next_burst() {
    let mut npu = create_npu();

    // Register core areas for deterministic neuron IDs.
    npu.register_cortical_area(0, CoreCorticalType::Death.to_cortical_id().as_base_64());
    npu.register_cortical_area(1, CoreCorticalType::Power.to_cortical_id().as_base_64());

    let memory_area_idx = 20u32;
    let upstream_area_idx = 30u32;
    let twin_area_idx = 31u32;

    let memory_id = CorticalID::try_from_bytes(b"mmem0001").unwrap();
    let upstream_id = CorticalID::try_from_bytes(b"csrc0001").unwrap();
    let twin_id = CorticalID::try_from_bytes(b"csrc0002").unwrap();

    npu.register_cortical_area(memory_area_idx, memory_id.as_base_64());
    npu.register_cortical_area(upstream_area_idx, upstream_id.as_base_64());
    npu.register_cortical_area(twin_area_idx, twin_id.as_base_64());

    npu.configure_fire_ledger_window(twin_area_idx, 1)
        .expect("Failed to configure fire ledger window");

    let twin_a = add_neuron_at(&mut npu, twin_area_idx, 0, 0, 0);
    let twin_b = add_neuron_at(&mut npu, twin_area_idx, 1, 0, 0);

    let memory_neuron_id = 50_000_000u32;
    npu.register_dynamic_neuron_mapping(memory_neuron_id, memory_id.clone());
    npu.register_memory_twin_mapping(
        memory_area_idx,
        upstream_area_idx,
        twin_area_idx,
        10.0,
    );
    npu.register_memory_replay_frames(
        memory_neuron_id,
        vec![MemoryReplayFrame {
            offset: 0,
            upstream_area_idx,
            coords: vec![(0, 0, 0), (1, 0, 0)],
        }],
    );

    npu.inject_memory_neuron_to_fcl(memory_neuron_id, memory_area_idx, 5.0);
    let burst1 = npu.process_burst().expect("Burst failed").burst;
    let burst2 = npu.process_burst().expect("Burst failed").burst;
    assert_eq!(burst2, burst1 + 1);

    let window = npu
        .get_fire_ledger_dense_window_bitmaps(twin_area_idx, burst2, 1)
        .expect("Missing FireLedger window for twin area");
    assert!(
        window.iter().any(|(_, bm)| bm.contains(twin_a.0) && bm.contains(twin_b.0)),
        "Expected replay to fire twin neurons on the next burst"
    );
}

#[test]
fn test_memory_replay_respects_offsets() {
    let mut npu = create_npu();

    npu.register_cortical_area(0, CoreCorticalType::Death.to_cortical_id().as_base_64());
    npu.register_cortical_area(1, CoreCorticalType::Power.to_cortical_id().as_base_64());

    let memory_area_idx = 22u32;
    let upstream_area_idx = 32u32;
    let twin_area_idx = 33u32;

    let memory_id = CorticalID::try_from_bytes(b"mmem0002").unwrap();
    let upstream_id = CorticalID::try_from_bytes(b"csrc0003").unwrap();
    let twin_id = CorticalID::try_from_bytes(b"csrc0004").unwrap();

    npu.register_cortical_area(memory_area_idx, memory_id.as_base_64());
    npu.register_cortical_area(upstream_area_idx, upstream_id.as_base_64());
    npu.register_cortical_area(twin_area_idx, twin_id.as_base_64());

    npu.configure_fire_ledger_window(twin_area_idx, 1)
        .expect("Failed to configure fire ledger window");

    let twin_neuron = add_neuron_at(&mut npu, twin_area_idx, 0, 0, 0);

    let memory_neuron_id = 50_000_010u32;
    npu.register_dynamic_neuron_mapping(memory_neuron_id, memory_id.clone());
    npu.register_memory_twin_mapping(
        memory_area_idx,
        upstream_area_idx,
        twin_area_idx,
        10.0,
    );
    npu.register_memory_replay_frames(
        memory_neuron_id,
        vec![MemoryReplayFrame {
            offset: 2,
            upstream_area_idx,
            coords: vec![(0, 0, 0)],
        }],
    );

    npu.inject_memory_neuron_to_fcl(memory_neuron_id, memory_area_idx, 5.0);
    let _burst1 = npu.process_burst().expect("Burst failed").burst;

    let burst2 = npu.process_burst().expect("Burst failed").burst;
    let window2 = npu
        .get_fire_ledger_dense_window_bitmaps(twin_area_idx, burst2, 1)
        .expect("Missing FireLedger window for burst2");
    assert!(
        window2.iter().all(|(_, bm)| !bm.contains(twin_neuron.0)),
        "Expected no replay firing at burst2"
    );

    let burst3 = npu.process_burst().expect("Burst failed").burst;
    let window3 = npu
        .get_fire_ledger_dense_window_bitmaps(twin_area_idx, burst3, 1)
        .expect("Missing FireLedger window for burst3");
    assert!(
        window3.iter().all(|(_, bm)| !bm.contains(twin_neuron.0)),
        "Expected no replay firing at burst3"
    );

    let burst4 = npu.process_burst().expect("Burst failed").burst;
    let window4 = npu
        .get_fire_ledger_dense_window_bitmaps(twin_area_idx, burst4, 1)
        .expect("Missing FireLedger window for burst4");
    assert!(
        window4.iter().any(|(_, bm)| bm.contains(twin_neuron.0)),
        "Expected replay firing at burst4 (offset=2)"
    );
}
