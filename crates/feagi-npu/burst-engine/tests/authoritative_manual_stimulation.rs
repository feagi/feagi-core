// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Manual force-fire merges real neuron IDs into the fire queue after dynamics, bypassing LIF rules.

use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::RustNPU;
use feagi_npu_runtime::StdRuntime;
use feagi_structures::genomic::cortical_area::{CoreCorticalType, CorticalID};

#[test]
fn authoritative_force_fire_includes_neuron_despite_zero_excitability() {
    let runtime = StdRuntime;
    let backend = CPUBackend::new();
    let mut npu = RustNPU::new(runtime, backend, 256, 1024, 16).unwrap();

    let stim_area_id = CorticalID::try_from_bytes(b"cstm0001").unwrap();

    npu.register_cortical_area(0, CoreCorticalType::Death.to_cortical_id().as_base_64());
    npu.register_cortical_area(1, CoreCorticalType::Power.to_cortical_id().as_base_64());
    npu.register_cortical_area(10, stim_area_id.as_base_64());

    // High threshold + zero excitability: candidate stimulation would never fire this neuron.
    let nid = npu
        .add_neuron(
            1000.0,
            f32::MAX,
            0.0,
            0.0,
            0,
            0,
            0.0,
            0,
            0,
            false,
            10,
            0,
            0,
            0,
        )
        .unwrap();

    let injected = npu.inject_force_fire_by_coordinates(&stim_area_id, &[(0, 0, 0, 50.0)]);
    assert_eq!(injected, 1);

    let result = npu.process_burst().unwrap();
    assert!(
        result.fired_neurons.contains(&nid),
        "expected authoritative merge for neuron {:?}, fired={:?}",
        nid,
        result.fired_neurons
    );
}

#[test]
fn authoritative_force_fire_preserves_voxel_coordinates_in_fire_queue() {
    let runtime = StdRuntime;
    let backend = CPUBackend::new();
    let mut npu = RustNPU::new(runtime, backend, 256, 1024, 16).unwrap();

    let stim_area_id = CorticalID::try_from_bytes(b"cstm0002").unwrap();

    npu.register_cortical_area(0, CoreCorticalType::Death.to_cortical_id().as_base_64());
    npu.register_cortical_area(1, CoreCorticalType::Power.to_cortical_id().as_base_64());
    npu.register_cortical_area(10, stim_area_id.as_base_64());

    let _nid = npu
        .add_neuron(
            1000.0,
            f32::MAX,
            0.0,
            0.0,
            0,
            0,
            0.0,
            0,
            0,
            false,
            10,
            3,
            4,
            5,
        )
        .unwrap();

    npu.inject_force_fire_by_coordinates(&stim_area_id, &[(3, 4, 5, 1.0)]);

    npu.process_burst().unwrap();

    let sample = npu.get_current_fire_queue();
    let mut found = false;
    for (_area, rows) in sample {
        let (_ids, xs, ys, zs, _) = rows;
        for i in 0..xs.len() {
            if xs[i] == 3 && ys[i] == 4 && zs[i] == 5 {
                found = true;
            }
        }
    }
    assert!(
        found,
        "fire queue sample should carry storage coordinates for visualization"
    );
}
