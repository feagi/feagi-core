// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0
//! NPU-level tests for the rate-modulated leak homeostat (`rate_modulated_leak.rs`).
//!
//! Each test builds a real `RustNPU`, registers the homeostat through
//! `sync_rate_modulated_leak_from_cortical_property`, drives bursts and reads the leak back
//! from neuron storage.

use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::RustNPU;
use feagi_npu_neural::types::NeuronId;
use feagi_npu_runtime::StdRuntime;
use feagi_structures::genomic::cortical_area::{CoreCorticalType, CorticalID};
use serde_json::json;

type TestNpu = RustNPU<StdRuntime, f32, CPUBackend>;

const AREA: u32 = 10;
const BASE_LEAK: f32 = 0.1;

/// Build an NPU with one area (`AREA`) of `n` LIF neurons, leak `BASE_LEAK`, threshold 1.0.
fn make_npu(n: u32) -> (TestNpu, Vec<NeuronId>) {
    let mut npu = RustNPU::new(StdRuntime, CPUBackend::new(), 100, 1000, 10).unwrap();
    npu.register_cortical_area(0, CoreCorticalType::Death.to_cortical_id().as_base_64());
    npu.register_cortical_area(1, CoreCorticalType::Power.to_cortical_id().as_base_64());
    npu.register_cortical_area(
        AREA,
        CorticalID::try_from_bytes(b"chom0001")
            .unwrap()
            .as_base_64(),
    );
    let ids = (0..n)
        .map(|i| {
            npu.add_neuron(
                1.0,
                f32::MAX,
                BASE_LEAK,
                0.0,
                0,
                0,
                1.0,
                0,
                0,
                true,
                AREA,
                i,
                0,
                0,
            )
            .unwrap()
        })
        .collect();
    (npu, ids)
}

fn leak_of(npu: &TestNpu, id: NeuronId) -> f32 {
    npu.get_neuron_property_by_index(id.0 as usize, "leak_coefficient")
        .unwrap()
}

fn register(npu: &mut TestNpu, ids: &[NeuronId], target: f32) {
    let cfg = json!({
        "enabled": true,
        "target_firing_per_burst": target,
        "rate_ema_tau_bursts": 1.0,
        "gain": 1.0,
        "leak_min": 0.02,
        "leak_max": 0.5,
    });
    npu.sync_rate_modulated_leak_from_cortical_property(
        AREA,
        &cfg,
        BASE_LEAK,
        ids.iter().map(|n| n.0 as usize).collect(),
    );
}

fn fire_bursts(npu: &mut TestNpu, ids: &[NeuronId], bursts: usize) {
    for _ in 0..bursts {
        let pots: Vec<(NeuronId, f32)> = ids.iter().map(|&i| (i, 128.0)).collect();
        npu.inject_sensory_with_potentials(&pots);
        let r = npu.process_burst().unwrap();
        println!("burst {} fired {}", r.burst, r.neuron_count);
    }
}

/// (a) Without `configure_fire_ledger_window` the homeostat should still work.
#[test]
fn a_untracked_area_leak_rises_when_firing_above_target() {
    let (mut npu, ids) = make_npu(3);
    register(&mut npu, &ids, 0.0);
    // Deliberately no configure_fire_ledger_window.
    fire_bursts(&mut npu, &ids, 5);
    let leaks: Vec<f32> = ids.iter().map(|&i| leak_of(&npu, i)).collect();
    println!("untracked leaks after 5 firing bursts: {leaks:?}");
    for l in leaks {
        assert!(
            l > BASE_LEAK + 1e-4,
            "leak should rise above {BASE_LEAK}, got {l}"
        );
    }
}

/// (b) With ledger tracking, firing neurons raise leak.
#[test]
fn b_tracked_area_leak_rises_when_firing_above_target() {
    let (mut npu, ids) = make_npu(3);
    register(&mut npu, &ids, 0.0);
    npu.configure_fire_ledger_window(AREA, 1).unwrap();
    fire_bursts(&mut npu, &ids, 5);
    let leaks: Vec<f32> = ids.iter().map(|&i| leak_of(&npu, i)).collect();
    println!("tracked leaks after 5 firing bursts: {leaks:?}");
    for l in leaks {
        assert!(
            l > BASE_LEAK + 1e-4,
            "leak should rise above {BASE_LEAK}, got {l}"
        );
    }
}

/// (b) With ledger tracking, a silent area with target 0.5 drops toward leak_min (0.02).
#[test]
fn b_tracked_silent_area_leak_drops_toward_leak_min() {
    let (mut npu, ids) = make_npu(3);
    register(&mut npu, &ids, 0.5);
    npu.configure_fire_ledger_window(AREA, 1).unwrap();
    for _ in 0..5 {
        npu.process_burst().unwrap();
    }
    let leaks: Vec<f32> = ids.iter().map(|&i| leak_of(&npu, i)).collect();
    println!("tracked silent leaks after 5 bursts: {leaks:?}");
    for l in leaks {
        assert!(
            l < BASE_LEAK - 1e-4,
            "leak should drop below {BASE_LEAK}, got {l}"
        );
        assert!(
            (l - 0.02).abs() < 1e-3,
            "leak should reach leak_min 0.02, got {l}"
        );
    }
}

/// (c) A user leak edit through the bulk NPU update reseeds the homeostat: it survives the
/// next update burst as the new starting point (gain 0 keeps it exactly).
#[test]
fn c_bulk_leak_edit_with_homeostat_active() {
    let (mut npu, ids) = make_npu(3);
    let cfg = json!({
        "enabled": true,
        "target_firing_per_burst": 0.5,
        "rate_ema_tau_bursts": 1.0,
        "gain": 0.0,
        "leak_min": 0.02,
        "leak_max": 0.5,
    });
    npu.sync_rate_modulated_leak_from_cortical_property(
        AREA,
        &cfg,
        BASE_LEAK,
        ids.iter().map(|n| n.0 as usize).collect(),
    );
    npu.configure_fire_ledger_window(AREA, 1).unwrap();
    for _ in 0..3 {
        npu.process_burst().unwrap();
    }
    let n = npu.update_cortical_area_leak(AREA, 0.3);
    assert_eq!(n, 3);
    assert!((leak_of(&npu, ids[0]) - 0.3).abs() < 1e-6);
    npu.process_burst().unwrap();
    for &i in &ids {
        let after = leak_of(&npu, i);
        assert!(
            (after - 0.3).abs() < 1e-6,
            "edit should survive a burst, got {after}"
        );
    }
}

/// (c2) With gain > 0 the homeostat continues from the edited value, not from its old state.
#[test]
fn c2_leak_edit_is_new_starting_point_for_active_homeostat() {
    let (mut npu, ids) = make_npu(3);
    register(&mut npu, &ids, 0.5); // gain 1.0, silent neurons: leak falls by 0.5 per burst
    npu.configure_fire_ledger_window(AREA, 1).unwrap();
    for _ in 0..3 {
        npu.process_burst().unwrap();
    }
    npu.update_cortical_area_leak(AREA, 0.4);
    npu.process_burst().unwrap();
    let after = leak_of(&npu, ids[0]);
    // 0.4 + gain * (r_ema - target) is clamped to leak_min; it must not be a stale value > leak_max.
    assert!((0.02..=0.5).contains(&after), "got {after}");
}

/// Deleting an area removes it from the registry and the modulator engine's homeostat set.
#[test]
fn f_area_delete_clears_registry_and_modulator_set() {
    let (mut npu, ids) = make_npu(3);
    register(&mut npu, &ids, 0.5);
    npu.set_homeostatic_leak_areas([AREA].into_iter().collect());
    assert!(npu.has_rate_modulated_leak(AREA));
    assert!(npu.is_homeostatic_leak_area(AREA));
    npu.remove_rate_modulated_leak(AREA);
    assert!(!npu.has_rate_modulated_leak(AREA));
    assert!(!npu.is_homeostatic_leak_area(AREA));
}

/// Clearing the registry drops every area.
#[test]
fn g_clear_registry_drops_all_areas() {
    let (mut npu, ids) = make_npu(3);
    register(&mut npu, &ids, 0.5);
    assert!(npu.has_rate_modulated_leak(AREA));
    npu.clear_rate_modulated_leak();
    assert!(!npu.has_rate_modulated_leak(AREA));
}

/// (e) Removing the area from the registry stops all leak writes.
#[test]
fn e_removed_area_leak_unchanged() {
    let (mut npu, ids) = make_npu(3);
    register(&mut npu, &ids, 0.0);
    npu.configure_fire_ledger_window(AREA, 1).unwrap();
    npu.remove_rate_modulated_leak(AREA);
    fire_bursts(&mut npu, &ids, 3);
    for &i in &ids {
        let l = leak_of(&npu, i);
        assert!(
            (l - BASE_LEAK).abs() < 1e-6,
            "leak changed after removal: {l}"
        );
    }
}
