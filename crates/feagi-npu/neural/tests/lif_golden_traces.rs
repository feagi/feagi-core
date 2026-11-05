// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Golden traces of `update_neuron_lif` for a teaching visual.
//!
//! Each trace drives the real rule burst by burst. By default (plain `cargo test`) the test
//! regenerates the traces in memory and asserts they equal `tests/fixtures/golden-traces.json`,
//! so any change to the rule fails it. The `feagi_core_commit` field is ignored by the
//! comparison (it changes with every commit).
//!
//! Regenerate both checked-in copies (this crate's fixture and the front-end copy):
//!
//! ```text
//! FEAGI_CORE_COMMIT=$(git rev-parse HEAD) \
//! FEAGI_WRITE_VISUAL_FIXTURES=<feagi-core>/crates/feagi-npu/neural/tests/fixtures:<feagi-react-core-pro>/src/feagi-academy/visuals/neuron-firing/fixtures \
//! cargo test -p feagi-npu-neural --test lif_golden_traces
//! ```
//!
//! `FEAGI_WRITE_VISUAL_FIXTURES` is a `:`-separated list of directories; the file
//! `golden-traces.json` is written into each. The commit is a note only: the working tree may
//! hold uncommitted changes.
//!
//! Row semantics: `burst` is 0-based. `input` is the current delivered to the neuron at that
//! burst (0 when none). `membrane_potential` is V at the end of the burst with leak caught up
//! to that burst: after a burst with input it is the value stored by `update_neuron_lif`
//! (0 when the neuron fired and reset); after a burst without input it is
//! `decayed_charge(stored V, leak, bursts since the last update)`, and the stored state is
//! left untouched (lazy leak). `fired` is the return value of `update_neuron_lif`.

use feagi_npu_neural::{decayed_charge, update_neuron_lif};
use serde::{Deserialize, Serialize};
use std::path::Path;

const BURSTS: u64 = 240;
const FILE_NAME: &str = "golden-traces.json";

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Settings {
    fire_threshold: f32,
    leak: f32,
    input_strength: f32,
    bursts_between_inputs: u64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Row {
    burst: u64,
    input: f32,
    membrane_potential: f32,
    fired: bool,
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
    function: String,
    feagi_core_commit: String,
    traces: Vec<Trace>,
}

/// An input arrives at bursts 0, n, 2n, ... where n = `bursts_between_inputs`.
fn run_trace(name: &str, settings: Settings) -> Trace {
    let mut potential = 0.0f32;
    let mut fraction = 0.0f32;
    let mut last_update = 0u64;
    let mut rows = Vec::new();
    for burst in 0..BURSTS {
        let has_input = burst % settings.bursts_between_inputs == 0;
        let (input, v, fired) = if has_input {
            let fired = update_neuron_lif(
                &mut potential,
                &mut fraction,
                settings.fire_threshold,
                0.0,
                settings.leak,
                0.0,
                settings.input_strength,
                burst - last_update,
            );
            last_update = burst;
            (settings.input_strength, potential + fraction, fired)
        } else {
            let v = decayed_charge(potential + fraction, settings.leak, burst - last_update);
            (0.0, v, false)
        };
        rows.push(Row {
            burst,
            input,
            membrane_potential: v,
            fired,
        });
    }
    Trace {
        name: name.to_string(),
        settings,
        rows,
    }
}

fn settings(fire_threshold: f32, leak: f32, input_strength: f32, gap: u64) -> Settings {
    Settings {
        fire_threshold,
        leak,
        input_strength,
        bursts_between_inputs: gap,
    }
}

fn generate_traces() -> Vec<Trace> {
    vec![
        run_trace("no_leak", settings(1.0, 0.0, 0.3, 1)),
        run_trace("leak_input_every_burst", settings(1.0, 0.1, 0.25, 1)),
        run_trace("leak_idle_between_inputs", settings(1.0, 0.1, 0.6, 3)),
        run_trace("input_too_weak", settings(1.0, 0.1, 0.05, 1)),
    ]
}

fn golden(commit: &str, traces: Vec<Trace>) -> Golden {
    Golden {
        source_crate: "feagi-npu-neural".to_string(),
        function: "update_neuron_lif".to_string(),
        feagi_core_commit: commit.to_string(),
        traces,
    }
}

#[test]
fn lif_golden_traces_match_checked_in_json() {
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
        .join("tests/fixtures")
        .join(FILE_NAME);
    let text = std::fs::read_to_string(&path).expect("fixture missing; see module docs");
    let checked_in: Golden = serde_json::from_str(&text).unwrap();
    assert_eq!(checked_in.source_crate, "feagi-npu-neural");
    assert_eq!(checked_in.function, "update_neuron_lif");
    assert_eq!(
        checked_in.traces, traces,
        "update_neuron_lif drifted from the golden traces"
    );

    // Sanity: the traces exercise what they claim to.
    let fires: Vec<usize> = traces
        .iter()
        .map(|t| t.rows.iter().filter(|r| r.fired).count())
        .collect();
    assert!(fires[0] > 0 && fires[1] > 0 && fires[2] > 0);
    assert_eq!(fires[3], 0);
}

#[test]
fn drift_is_detected_negative_case() {
    let traces = generate_traces();
    let mut altered = traces.clone();
    altered[1].rows[50].membrane_potential += 1e-4;
    assert_ne!(altered, traces);
}

#[test]
fn json_round_trips_f32_exactly() {
    let traces = generate_traces();
    let text = serde_json::to_string(&golden("x", traces.clone())).unwrap();
    let back: Golden = serde_json::from_str(&text).unwrap();
    assert_eq!(back.traces, traces);
}
