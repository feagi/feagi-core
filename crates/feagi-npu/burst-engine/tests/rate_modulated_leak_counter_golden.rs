// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Golden trace of the rate-modulated leak `counter` update rule.
//!
//! Same closed loop as `rate_modulated_leak_golden_traces.rs` (real `update_neuron_lif`, a real
//! `FireLedger`, `RateModulatedLeakRegistry::apply_burstal`), kept in its own fixture so the
//! proportional traces (and the visual that consumes them) stay untouched.
//!
//! Regenerate: `FEAGI_WRITE_COUNTER_FIXTURE=1 cargo test -p feagi-npu-burst-engine --test
//! rate_modulated_leak_counter_golden`. Row semantics as in the proportional file: `leak` is the
//! slot after the pass; the next burst's LIF step reads it.

use feagi_npu_burst_engine::fire_ledger::FireLedger;
use feagi_npu_burst_engine::fire_structures::{FireQueue, FiringNeuron, FIRE_KIND_STDP_ELIGIBLE};
use feagi_npu_burst_engine::rate_modulated_leak::RateModulatedLeakRegistry;
use feagi_npu_neural::types::NeuronId;
use feagi_npu_neural::update_neuron_lif;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;

const BURSTS: u64 = 200;
const FIXTURE: &str = "tests/fixtures/self-tuning-leak-counter/golden-traces.json";
const CORTICAL_IDX: u32 = 10;
const NEURON_IDX: usize = 3;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Row {
    burst: u64,
    fired: bool,
    membrane_potential: f32,
    rate_ema: f32,
    leak: f32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Trace {
    name: String,
    base_leak: f32,
    drive: f32,
    leak_min: f32,
    leak_max: f32,
    leak_levels: u32,
    counter_timeout_bursts: u32,
    rows: Vec<Row>,
}

#[derive(Serialize, Deserialize)]
struct Golden {
    source_crate: String,
    functions: Vec<String>,
    traces: Vec<Trace>,
}

fn run_trace(name: &str, base_leak: f32, drive: f32) -> Trace {
    let (leak_min, leak_max, leak_levels, timeout) = (0.0_f32, 0.6_f32, 7_u32, 4_u32);
    let mut registry = RateModulatedLeakRegistry::default();
    registry.sync_cortical(
        CORTICAL_IDX,
        &json!({
            "enabled": true,
            "update_rule": "counter",
            "leak_min": leak_min,
            "leak_max": leak_max,
            "leak_levels": leak_levels,
            "counter_timeout_bursts": timeout,
            "rate_ema_tau_bursts": 20.0,
        }),
        base_leak,
        vec![NEURON_IDX],
    );
    let mut ledger = FireLedger::new(8);
    ledger.track_area(CORTICAL_IDX, 8).unwrap();
    let mut leaks = vec![0.0_f32; 8];
    leaks[NEURON_IDX] = base_leak;
    let (mut potential, mut fraction) = (0.0f32, 0.0f32);
    let mut rows = Vec::new();
    for burst in 1..=BURSTS {
        let fired = update_neuron_lif(
            &mut potential,
            &mut fraction,
            1.0,
            0.0,
            leaks[NEURON_IDX],
            0.0,
            drive,
            1,
        );
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
        ledger.archive_burst(burst, &fq).unwrap();
        registry.apply_burstal(burst, &ledger, &mut leaks);
        registry.drain_leak_changes(|_, _| {});
        let state = registry
            .neuron_state_by_neuron(CORTICAL_IDX)
            .unwrap()
            .remove(&NEURON_IDX)
            .unwrap();
        rows.push(Row {
            burst,
            fired,
            membrane_potential: potential + fraction,
            rate_ema: state.rate_ema,
            leak: leaks[NEURON_IDX],
        });
    }
    Trace {
        name: name.to_string(),
        base_leak,
        drive,
        leak_min,
        leak_max,
        leak_levels,
        counter_timeout_bursts: timeout,
        rows,
    }
}

fn generate() -> Golden {
    Golden {
        source_crate: "feagi-npu-burst-engine".to_string(),
        functions: vec![
            "RateModulatedLeakRegistry::apply_burstal".to_string(),
            "feagi_npu_neural::update_neuron_lif".to_string(),
        ],
        traces: vec![run_trace("counter_driven_neuron_settles_on_grid", 0.1, 0.4)],
    }
}

#[test]
fn counter_golden_trace_matches_checked_in_json() {
    let golden = generate();
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    if std::env::var("FEAGI_WRITE_COUNTER_FIXTURE").is_ok() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, serde_json::to_string_pretty(&golden).unwrap() + "\n").unwrap();
        return;
    }
    let text = std::fs::read_to_string(&path).expect("fixture missing; see module docs");
    let checked_in: Golden = serde_json::from_str(&text).unwrap();
    assert_eq!(checked_in.traces, golden.traces, "counter rule drifted");

    // Every stored leak is a grid value (index * 0.1 on this grid) once the pass has run.
    for t in &golden.traces {
        for r in &t.rows {
            let i = (r.leak / 0.1).round();
            assert!((r.leak - i * 0.1).abs() < 1e-5, "off-grid leak {}", r.leak);
        }
        assert!(t.rows.iter().any(|r| r.fired), "trace must spike");
    }
}
