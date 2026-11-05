// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Golden traces of the rate-modulated leak homeostat for a teaching visual.
//!
//! Each trace drives the real rule burst by burst: `update_neuron_lif` (feagi-npu-neural) for
//! the membrane, a real `FireLedger` for the firing record, and
//! `RateModulatedLeakRegistry::apply_burstal` for the leak update. No NPU is involved. The loop
//! is closed: the leak the homeostat writes is the leak the next burst's LIF step reads.
//! By default (plain `cargo test`) the test regenerates the traces in memory and asserts they
//! equal `tests/fixtures/self-tuning-leak/golden-traces.json`. `feagi_core_commit` is ignored
//! by the comparison.
//!
//! Regenerate both checked-in copies:
//!
//! ```text
//! FEAGI_CORE_COMMIT=$(git rev-parse HEAD) \
//! FEAGI_WRITE_VISUAL_FIXTURES=<feagi-core>/crates/feagi-npu/burst-engine/tests/fixtures/self-tuning-leak:<feagi-react-core-pro>/src/feagi-academy/visuals/self-tuning-leak/fixtures \
//! cargo test -p feagi-npu-burst-engine --test rate_modulated_leak_golden_traces
//! ```
//!
//! `FEAGI_WRITE_VISUAL_FIXTURES` is a `:`-separated list of directories; `golden-traces.json`
//! is written into each. The commit is a note only: the working tree may hold uncommitted
//! changes.
//!
//! Row semantics. `burst` is 1-based: it is the `burst_count` handed to `apply_burstal` and
//! the ledger timestep. Per burst, in this order: (1) `update_neuron_lif` with the leak stored
//! in `storage_leaks` and the constant `drive` as input (elapsed 1 burst); (2) the burst is
//! archived in the ledger; (3) `apply_burstal(burst, ..)` updates the rate EMA and, when due,
//! the leak. `fired` is the return of step 1. `membrane_potential` is V after step 1 (0 after a
//! spike). `rate_ema` is the registry's firing-rate EMA after step 3. `leak` is the value in
//! `storage_leaks` after step 3; the next burst's step 1 uses it. In `per_neuron` mode
//! `armed` is true while the registry is still measuring the neuron's own rate (the leak is
//! held at base then); it comes from the registry's own readout (`regulated` flag false while
//! arming), not from a re-derivation. The readout is taken after step 3, so the last arming
//! burst (`arm_bursts`) already shows `armed = false` (targets freeze inside that burst's pass).

use feagi_npu_burst_engine::fire_ledger::FireLedger;
use feagi_npu_burst_engine::fire_structures::{FireQueue, FiringNeuron, FIRE_KIND_STDP_ELIGIBLE};
use feagi_npu_burst_engine::rate_modulated_leak::RateModulatedLeakRegistry;
use feagi_npu_neural::types::NeuronId;
use feagi_npu_neural::update_neuron_lif;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;

const BURSTS: u64 = 300;
const FILE_NAME: &str = "golden-traces.json";
const FIXTURE_DIR: &str = "tests/fixtures/self-tuning-leak";
const CORTICAL_IDX: u32 = 10;
const NEURON_IDX: usize = 3;
const FIRE_THRESHOLD: f32 = 1.0;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Settings {
    /// `"area"` or `"per_neuron"`.
    target_mode: String,
    base_leak: f32,
    drive: f32,
    target_firing_per_burst: f32,
    rate_ema_tau_bursts: f32,
    gain: f32,
    leak_min: f32,
    leak_max: f32,
    update_every_n_bursts: u32,
    arm_bursts: u32,
    arm_tau_bursts: f32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Row {
    burst: u64,
    fired: bool,
    membrane_potential: f32,
    rate_ema: f32,
    leak: f32,
    armed: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Trace {
    name: String,
    settings: Settings,
    rows: Vec<Row>,
}

#[derive(Serialize, Deserialize)]
struct Golden {
    source_crate: String,
    functions: Vec<String>,
    feagi_core_commit: String,
    traces: Vec<Trace>,
}

fn fire_queue(fired: bool) -> FireQueue {
    let mut fq = FireQueue::new();
    if fired {
        fq.add_neuron(FiringNeuron {
            neuron_id: NeuronId(NEURON_IDX as u32),
            membrane_potential: 1.0,
            cortical_idx: CORTICAL_IDX,
            x: 0,
            y: 0,
            z: 0,
            fire_kind: FIRE_KIND_STDP_ELIGIBLE,
        });
    }
    fq
}

fn run_trace(name: &str, s: Settings) -> Trace {
    let mut registry = RateModulatedLeakRegistry::default();
    registry.sync_cortical(
        CORTICAL_IDX,
        &json!({
            "enabled": true,
            "target_mode": s.target_mode,
            "target_firing_per_burst": s.target_firing_per_burst,
            "rate_ema_tau_bursts": s.rate_ema_tau_bursts,
            "gain": s.gain,
            "leak_min": s.leak_min,
            "leak_max": s.leak_max,
            "update_every_n_bursts": s.update_every_n_bursts,
            "arm_bursts": s.arm_bursts,
            "arm_tau_bursts": s.arm_tau_bursts,
        }),
        s.base_leak,
        vec![NEURON_IDX],
    );
    let mut ledger = FireLedger::new(8);
    ledger.track_area(CORTICAL_IDX, 8).unwrap();
    let mut leaks = vec![0.0_f32; 8];
    leaks[NEURON_IDX] = s.base_leak;

    let mut potential = 0.0f32;
    let mut fraction = 0.0f32;
    let mut rows = Vec::new();
    for burst in 1..=BURSTS {
        let fired = update_neuron_lif(
            &mut potential,
            &mut fraction,
            FIRE_THRESHOLD,
            0.0,
            leaks[NEURON_IDX],
            0.0,
            s.drive,
            1,
        );
        let v = potential + fraction;
        ledger.archive_burst(burst, &fire_queue(fired)).unwrap();
        registry.apply_burstal(burst, &ledger, &mut leaks);
        registry.drain_leak_changes(|_, _| {});
        let state = registry
            .neuron_state_by_neuron(CORTICAL_IDX)
            .unwrap()
            .remove(&NEURON_IDX)
            .unwrap();
        let armed = s.target_mode == "per_neuron" && !state.regulated;
        rows.push(Row {
            burst,
            fired,
            membrane_potential: v,
            rate_ema: state.rate_ema,
            leak: leaks[NEURON_IDX],
            armed,
        });
    }
    Trace {
        name: name.to_string(),
        settings: s,
        rows,
    }
}

#[allow(clippy::too_many_arguments)]
fn area(
    base_leak: f32,
    drive: f32,
    target: f32,
    tau: f32,
    gain: f32,
    leak_min: f32,
    leak_max: f32,
    every: u32,
) -> Settings {
    Settings {
        target_mode: "area".to_string(),
        base_leak,
        drive,
        target_firing_per_burst: target,
        rate_ema_tau_bursts: tau,
        gain,
        leak_min,
        leak_max,
        update_every_n_bursts: every,
        arm_bursts: 500,
        arm_tau_bursts: 125.0,
    }
}

fn generate_traces() -> Vec<Trace> {
    let per_neuron = Settings {
        target_mode: "per_neuron".to_string(),
        base_leak: 0.1,
        drive: 0.35,
        target_firing_per_burst: 0.1,
        rate_ema_tau_bursts: 20.0,
        gain: 0.2,
        leak_min: 0.02,
        leak_max: 0.5,
        update_every_n_bursts: 1,
        arm_bursts: 100,
        arm_tau_bursts: 25.0,
    };
    vec![
        // Starts at a low leak: it fires above target, so the leak rises until the rate settles.
        run_trace(
            "area_low_leak_start_leak_rises",
            area(0.05, 0.4, 0.1, 20.0, 0.2, 0.02, 0.5, 1),
        ),
        // Leak so high the neuron is silent: the leak falls until it starts to fire.
        run_trace(
            "area_silent_leak_falls",
            area(0.45, 0.3, 0.1, 20.0, 0.2, 0.02, 0.5, 1),
        ),
        // Same as the first, but the leak step runs only every 5th burst (slower correction).
        run_trace(
            "area_leak_step_every_5_bursts",
            area(0.05, 0.4, 0.1, 20.0, 0.2, 0.02, 0.5, 5),
        ),
        // Per-neuron target: measure its own rate for 100 bursts, then hold it there.
        run_trace("per_neuron_arm_then_hold", per_neuron),
    ]
}

fn golden(commit: &str, traces: Vec<Trace>) -> Golden {
    Golden {
        source_crate: "feagi-npu-burst-engine".to_string(),
        functions: vec![
            "RateModulatedLeakRegistry::apply_burstal".to_string(),
            "feagi_npu_neural::update_neuron_lif".to_string(),
        ],
        feagi_core_commit: commit.to_string(),
        traces,
    }
}

#[test]
fn rate_modulated_leak_golden_traces_match_checked_in_json() {
    let traces = generate_traces();

    if let Ok(dirs) = std::env::var("FEAGI_WRITE_VISUAL_FIXTURES") {
        let commit = std::env::var("FEAGI_CORE_COMMIT").unwrap_or_else(|_| "unknown".to_string());
        let text = serde_json::to_string_pretty(&golden(&commit, traces.clone())).unwrap() + "\n";
        for dir in dirs.split(':').filter(|d| !d.is_empty()) {
            std::fs::create_dir_all(dir).unwrap();
            std::fs::write(Path::new(dir).join(FILE_NAME), &text).unwrap();
        }
        return;
    }

    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURE_DIR)
        .join(FILE_NAME);
    let text = std::fs::read_to_string(&path).expect("fixture missing; see module docs");
    let checked_in: Golden = serde_json::from_str(&text).unwrap();
    assert_eq!(checked_in.source_crate, "feagi-npu-burst-engine");
    assert_eq!(
        checked_in.traces, traces,
        "rate-modulated leak drifted from the golden traces"
    );

    // Sanity: the traces show what they claim.
    let last = |t: &Trace| t.rows.last().unwrap().clone();
    let first = |t: &Trace| t.rows.first().unwrap().clone();
    assert!(
        last(&traces[0]).leak > first(&traces[0]).leak,
        "too-active neuron must raise its leak"
    );
    assert!(
        last(&traces[1]).leak < first(&traces[1]).leak,
        "silent neuron must lower its leak"
    );
    // `arm_bursts` = 100 arming bursts; the readout after the last one already shows the frozen
    // state, so `armed` is true for bursts 1..=99 and false from burst 100.
    assert!(traces[3].rows.iter().take(99).all(|r| r.armed));
    assert!(traces[3].rows.iter().skip(99).all(|r| !r.armed));
}

#[test]
fn drift_is_detected_negative_case() {
    let traces = generate_traces();
    let mut altered = traces.clone();
    altered[0].rows[120].leak += 1e-4;
    assert_ne!(altered, traces);
    // A changed rule parameter changes the trace: a different gain must not reproduce it.
    let mut other = area(0.05, 0.4, 0.1, 20.0, 0.2, 0.02, 0.5, 1);
    other.gain = 0.21;
    assert_ne!(run_trace("x", other).rows, traces[0].rows);
}

#[test]
fn json_round_trips_f32_exactly() {
    let traces = generate_traces();
    let text = serde_json::to_string(&golden("x", traces.clone())).unwrap();
    let back: Golden = serde_json::from_str(&text).unwrap();
    assert_eq!(back.traces, traces);
}
