// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0
//! NPU-level tests for the rate-modulated leak homeostat (`rate_modulated_leak.rs`).
//!
//! Each test builds a real `RustNPU`, registers the homeostat through
//! `sync_rate_modulated_leak_from_cortical_property`, drives bursts and reads the leak back
//! from neuron storage.

use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::{NeuronStateField, RustNPU};
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

// ---------------------------------------------------------------------------------------
// target_mode = "per_neuron"
// ---------------------------------------------------------------------------------------

const ARM_BURSTS: usize = 400;

fn per_neuron_cfg(epoch: u32) -> serde_json::Value {
    json!({
        "enabled": true,
        "target_mode": "per_neuron",
        "arm_bursts": ARM_BURSTS,
        "arm_tau_bursts": 100.0,
        "min_target_firing_per_burst": 0.01,
        "arm_epoch": epoch,
        "rate_ema_tau_bursts": 20.0,
        "gain": 0.02,
        "leak_min": 0.0,
        "leak_max": 0.9,
    })
}

fn register_per_neuron(npu: &mut TestNpu, ids: &[NeuronId], epoch: u32) {
    npu.sync_rate_modulated_leak_from_cortical_property(
        AREA,
        &per_neuron_cfg(epoch),
        BASE_LEAK,
        ids.iter().map(|n| n.0 as usize).collect(),
    );
    npu.configure_fire_ledger_window(AREA, 1).unwrap();
}

/// Drive each neuron with its own constant per-burst input for `bursts` bursts.
fn drive(npu: &mut TestNpu, ids: &[NeuronId], inputs: &[f32], bursts: usize) {
    for _ in 0..bursts {
        let pots: Vec<(NeuronId, f32)> = ids
            .iter()
            .zip(inputs)
            .filter(|(_, &p)| p > 0.0)
            .map(|(&i, &p)| (i, p))
            .collect();
        npu.inject_sensory_with_potentials(&pots);
        npu.process_burst().unwrap();
    }
}

fn column(npu: &TestNpu, field: NeuronStateField) -> Vec<f32> {
    npu.read_neuron_state_columns(AREA, &[field])
        .fields
        .remove(0)
        .1
        .into_iter()
        .map(|v| v.expect("area is registered"))
        .collect()
}

/// Arming sets each neuron's target to its own rate (no leak change), the silent neuron is not
/// regulated, and after a constant extra input the rate moves back toward the target.
#[test]
fn per_neuron_arms_to_own_rate_then_holds_under_added_input() {
    let (mut npu, ids) = make_npu(3);
    register_per_neuron(&mut npu, &ids, 0);
    drive(&mut npu, &ids, &[0.3, 0.0, 0.45], ARM_BURSTS);

    let ema = column(&npu, NeuronStateField::RateEma);
    let target = column(&npu, NeuronStateField::RateTarget);
    let reg = column(&npu, NeuronStateField::HomeostatRegulated);
    let leak = column(&npu, NeuronStateField::LeakCoefficient);
    println!("armed: ema {ema:?} target {target:?} regulated {reg:?} leak {leak:?}");
    // Each target is the neuron's own rate, not an area mean.
    assert!((target[0] - ema[0]).abs() < 0.05, "{target:?} vs {ema:?}");
    assert!((target[2] - ema[2]).abs() < 0.05, "{target:?} vs {ema:?}");
    assert!(
        target[0] > 0.15 && target[2] > target[0] + 0.03,
        "{target:?}"
    );
    assert_eq!(target[1], 0.0);
    assert_eq!(
        reg,
        vec![1.0, 0.5, 1.0],
        "silent neuron is one-sided by default"
    );
    // No leak adjustment while arming, and the silent neuron is never pinned.
    assert!(
        leak.iter().all(|&l| (l - BASE_LEAK).abs() < 1e-6),
        "{leak:?}"
    );

    // Constant extra input: rate jumps above the frozen target, then the leak pulls it back.
    drive(&mut npu, &ids, &[0.4, 0.0, 0.6], 50);
    let early = column(&npu, NeuronStateField::RateEma);
    drive(&mut npu, &ids, &[0.4, 0.0, 0.6], 600);
    let late = column(&npu, NeuronStateField::RateEma);
    let leak = column(&npu, NeuronStateField::LeakCoefficient);
    let target_after = column(&npu, NeuronStateField::RateTarget);
    println!("early {early:?} late {late:?} leak {leak:?}");
    assert_eq!(target_after, target, "targets stay frozen");
    for j in [0usize, 2] {
        assert!(
            early[j] > target[j] + 0.03,
            "disturbance raises rate: {early:?}"
        );
        assert!(
            (late[j] - target[j]).abs() < (early[j] - target[j]).abs() * 0.6,
            "neuron {j} rate should return toward target: early {early:?} late {late:?} target {target:?}"
        );
        assert!(leak[j] > BASE_LEAK + 0.01, "leak rose: {leak:?}");
    }
    assert!(
        (leak[1] - BASE_LEAK).abs() < 1e-6,
        "silent neuron leak stays at base: {leak:?}"
    );
}

/// A re-sync with the same `arm_epoch` keeps the armed targets; a new epoch restarts arming
/// from the base leak.
#[test]
fn per_neuron_rearm_resets() {
    let (mut npu, ids) = make_npu(2);
    register_per_neuron(&mut npu, &ids, 0);
    drive(&mut npu, &ids, &[0.3, 0.0], ARM_BURSTS);
    drive(&mut npu, &ids, &[0.5, 0.0], 200);
    let leak = column(&npu, NeuronStateField::LeakCoefficient);
    let target = column(&npu, NeuronStateField::RateTarget);
    assert!(
        leak[0] > BASE_LEAK + 0.005,
        "regulated leak moved: {leak:?}"
    );

    // Same epoch: parameters update, armed state kept.
    register_per_neuron(&mut npu, &ids, 0);
    assert_eq!(column(&npu, NeuronStateField::RateTarget), target);
    assert_eq!(column(&npu, NeuronStateField::HomeostatRegulated)[0], 1.0);

    // New epoch: arming restarts (not regulated), leaks go back to the base value.
    register_per_neuron(&mut npu, &ids, 1);
    drive(&mut npu, &ids, &[0.5, 0.0], 10);
    assert_eq!(
        column(&npu, NeuronStateField::HomeostatRegulated),
        vec![0.0, 0.0]
    );
    let leak = column(&npu, NeuronStateField::LeakCoefficient);
    assert!(
        leak.iter().all(|&l| (l - BASE_LEAK).abs() < 1e-6),
        "{leak:?}"
    );
    // It re-arms to the new input's own rate.
    drive(&mut npu, &ids, &[0.5, 0.0], ARM_BURSTS);
    let target2 = column(&npu, NeuronStateField::RateTarget);
    assert_eq!(
        column(&npu, NeuronStateField::HomeostatRegulated),
        vec![1.0, 0.5]
    );
    assert!(target2[0] > target[0] + 0.03, "{target2:?} vs {target:?}");
}

/// Area mode reports its single target and `regulated = 1` for every neuron.
#[test]
fn area_mode_reports_area_target_and_regulated() {
    let (mut npu, ids) = make_npu(2);
    register(&mut npu, &ids, 0.2);
    assert_eq!(column(&npu, NeuronStateField::RateTarget), vec![0.2, 0.2]);
    assert_eq!(
        column(&npu, NeuronStateField::HomeostatRegulated),
        vec![1.0, 1.0]
    );
}
