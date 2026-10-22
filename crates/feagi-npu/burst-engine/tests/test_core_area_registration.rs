// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0
//! Core area registration must be idempotent.
//!
//! `ConnectomeManager::sync_cortical_ids_to_npu` re-registers every area on each
//! agent auto-create pass, so registering a core area that already has its
//! neuron must not add another one.

use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::RustNPU;
use feagi_npu_runtime::StdRuntime;
use feagi_structures::genomic::cortical_area::{CoreCorticalType, CorticalID};

const CORE_TYPES: [CoreCorticalType; 7] = [
    CoreCorticalType::Death,
    CoreCorticalType::Power,
    CoreCorticalType::Fatigue,
    CoreCorticalType::Pain,
    CoreCorticalType::Pleasure,
    CoreCorticalType::Fear,
    CoreCorticalType::Hope,
];

fn create_npu() -> RustNPU<StdRuntime, f32, CPUBackend> {
    RustNPU::new(StdRuntime, CPUBackend::new(), 100, 1000, 10).expect("Failed to create NPU")
}

fn register_core_areas(npu: &mut RustNPU<StdRuntime, f32, CPUBackend>) {
    for (area_id, core) in CORE_TYPES.iter().enumerate() {
        npu.register_cortical_area(area_id as u32, core.to_cortical_id().as_base_64());
    }
}

#[test]
fn re_registering_core_areas_does_not_add_neurons() {
    let mut npu = create_npu();
    register_core_areas(&mut npu);
    let after_first = npu.get_neuron_count();
    assert_eq!(after_first, CORE_TYPES.len());
    for area_id in 0..CORE_TYPES.len() as u32 {
        assert_eq!(npu.get_neurons_in_cortical_area(area_id), vec![area_id]);
    }

    for _ in 0..5 {
        register_core_areas(&mut npu);
    }
    assert_eq!(npu.get_neuron_count(), after_first);
}

#[test]
fn core_neuron_away_from_its_deterministic_id_is_not_duplicated() {
    let mut npu = create_npu();
    // Two ordinary neurons take indices 0 and 1, as after a connectome restore,
    // so the power neuron cannot sit at its deterministic ID.
    let other_area = CorticalID::try_from_bytes(b"cuser001").unwrap();
    npu.register_cortical_area(20, other_area.as_base_64());
    for x in 0..2 {
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
            20,
            x,
            0,
            0,
        )
        .expect("Failed to add neuron");
    }
    npu.register_cortical_area(1, CoreCorticalType::Power.to_cortical_id().as_base_64());
    let power_neurons = npu.get_neurons_in_cortical_area(1);
    assert_eq!(power_neurons.len(), 1);
    assert_ne!(power_neurons[0], 1);
    let count = npu.get_neuron_count();

    for _ in 0..5 {
        npu.register_cortical_area(1, CoreCorticalType::Power.to_cortical_id().as_base_64());
    }
    assert_eq!(npu.get_neuron_count(), count);
    assert_eq!(npu.get_neurons_in_cortical_area(1), power_neurons);
}
