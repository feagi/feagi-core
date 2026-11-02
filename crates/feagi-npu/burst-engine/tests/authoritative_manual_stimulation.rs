// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Manual force-fire merges real neuron IDs into the fire queue after dynamics, bypassing LIF rules.

use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::stimulus_schedule::{ScheduledStimulus, StimulusScheduleStore};
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

/// Same drain the burst loop performs before each burst: due entries go through force-fire.
fn run_scheduled_burst(
    npu: &mut RustNPU<StdRuntime, f32, CPUBackend>,
    store: &StimulusScheduleStore,
) -> Vec<feagi_npu_neural::types::NeuronId> {
    for due in store.take_due(npu.get_burst_count() + 1) {
        for (cortical_id, voxels) in &due.areas {
            npu.inject_force_fire_by_coordinates(cortical_id, voxels);
        }
    }
    npu.process_burst().unwrap().fired_neurons
}

#[test]
fn scheduled_stimulus_fires_all_areas_on_same_burst_and_cancel_stops_later_entries() {
    let mut npu = RustNPU::new(StdRuntime, CPUBackend::new(), 256, 1024, 16).unwrap();
    let area_a = CorticalID::try_from_bytes(b"cstm000a").unwrap();
    let area_b = CorticalID::try_from_bytes(b"cstm000b").unwrap();
    npu.register_cortical_area(0, CoreCorticalType::Death.to_cortical_id().as_base_64());
    npu.register_cortical_area(1, CoreCorticalType::Power.to_cortical_id().as_base_64());
    npu.register_cortical_area(10, area_a.as_base_64());
    npu.register_cortical_area(11, area_b.as_base_64());
    let add = |npu: &mut RustNPU<StdRuntime, f32, CPUBackend>, idx: u32| {
        npu.add_neuron(
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
            idx,
            0,
            0,
            0,
        )
        .unwrap()
    };
    let nid_a = add(&mut npu, 10);
    let nid_b = add(&mut npu, 11);

    let voxel = vec![(0, 0, 0, 50.0)];
    let entry = |burst: u64, areas: Vec<&CorticalID>| ScheduledStimulus {
        burst,
        areas: areas.into_iter().map(|a| (*a, voxel.clone())).collect(),
    };
    // Offsets resolve to absolute bursts: offset 2 from the next burst (1) is burst 3; offset
    // 5 is burst 6.
    let store = StimulusScheduleStore::new();
    let (id, start, end) = store
        .add(vec![
            entry(3, vec![&area_a, &area_b]),
            entry(6, vec![&area_a, &area_b]),
        ])
        .unwrap();
    assert_eq!((start, end), (3, 6));

    let mut fired_at = Vec::new();
    for burst in 1..=3u64 {
        let fired = run_scheduled_burst(&mut npu, &store);
        assert_eq!(
            fired.contains(&nid_a),
            fired.contains(&nid_b),
            "areas must fire together on burst {burst}"
        );
        if fired.contains(&nid_a) {
            fired_at.push(burst);
        }
    }
    assert_eq!(
        fired_at,
        vec![3],
        "both areas fire on the scheduled burst only"
    );

    // Cancel: the burst-6 entry never fires.
    assert!(store.cancel(id));
    for _ in 4..=7 {
        let fired = run_scheduled_burst(&mut npu, &store);
        assert!(!fired.contains(&nid_a) && !fired.contains(&nid_b));
    }
}

fn two_area_npu() -> (
    RustNPU<StdRuntime, f32, CPUBackend>,
    CorticalID,
    feagi_npu_neural::types::NeuronId,
) {
    let mut npu = RustNPU::new(StdRuntime, CPUBackend::new(), 256, 1024, 16).unwrap();
    let area = CorticalID::try_from_bytes(b"cstm000c").unwrap();
    npu.register_cortical_area(0, CoreCorticalType::Death.to_cortical_id().as_base_64());
    npu.register_cortical_area(1, CoreCorticalType::Power.to_cortical_id().as_base_64());
    npu.register_cortical_area(10, area.as_base_64());
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
    (npu, area, nid)
}

#[test]
fn scheduled_stimulus_for_a_cleared_area_never_fires() {
    let (mut npu, area, nid) = two_area_npu();
    let store = StimulusScheduleStore::new();
    store
        .add(vec![ScheduledStimulus {
            burst: 3,
            areas: vec![(area, vec![(0, 0, 0, 50.0)])],
        }])
        .unwrap();
    // Area deleted before the entry is due.
    store.clear_area(&area);
    for _ in 1..=5 {
        let fired = run_scheduled_burst(&mut npu, &store);
        assert!(!fired.contains(&nid));
    }
    assert!(store.list().is_empty());
}

#[test]
fn repeating_schedule_fires_at_offset_plus_period_multiples_until_cancelled() {
    let (mut npu, area, nid) = two_area_npu();
    let store = StimulusScheduleStore::new();
    let (id, start, end) = store
        .add_with_repeat(
            vec![ScheduledStimulus {
                burst: 2,
                areas: vec![(area, vec![(0, 0, 0, 50.0)])],
            }],
            Some(4),
        )
        .unwrap();
    assert_eq!((start, end), (2, None));

    let mut fired_at = Vec::new();
    for burst in 1..=11u64 {
        if run_scheduled_burst(&mut npu, &store).contains(&nid) {
            fired_at.push(burst);
        }
    }
    assert_eq!(fired_at, vec![2, 6, 10]);

    assert!(store.cancel(id));
    for _ in 12..=20 {
        assert!(!run_scheduled_burst(&mut npu, &store).contains(&nid));
    }
}
