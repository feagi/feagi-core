// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Golden traces of value-to-neuron encoding for a teaching visual.
//!
//! Each trace drives the real functions: `encode_class_potential` / `decode_class_potential`
//! (feagi-structures) for the value a one-layer plane voxel carries, and `update_neuron_lif`
//! (feagi-npu-neural) for every bin neuron of an `expand_value_to_bins` area. By default
//! (plain `cargo test`) the test regenerates the traces in memory and asserts they equal
//! `tests/fixtures/value-encoding/golden-traces.json`. `feagi_core_commit` is ignored by the
//! comparison.
//!
//! Regenerate both checked-in copies:
//!
//! ```text
//! FEAGI_CORE_COMMIT=$(git rev-parse HEAD) \
//! FEAGI_WRITE_VISUAL_FIXTURES=<feagi-core>/crates/feagi-npu/burst-engine/tests/fixtures/value-encoding:<feagi-react-core-pro>/src/feagi-academy/visuals/value-encoding/fixtures \
//! cargo test -p feagi-npu-burst-engine --test value_encoding_golden_traces
//! ```
//!
//! `FEAGI_WRITE_VISUAL_FIXTURES` is a `:`-separated list of directories; `golden-traces.json`
//! is written into each. The commit is a note only: the working tree may hold uncommitted
//! changes.
//!
//! Row semantics. One row per presented value. `input` is the class id (class traces) or the
//! plain value (value traces). `potential` is what the plane voxel carries: for a class it is
//! `encode_class_potential(class_id, levels)`; for a value it is the value itself. A potential
//! of 0 means "nothing here" and the source neuron does not spike, so no bin fires.
//! `decoded_class` is `decode_class_potential(potential, levels)` (class traces only, null
//! when the potential names no class). `fired_bins` lists, in ascending order, the z indices of
//! bin neurons that fire when a spike carrying `potential` reaches the bin area: each bin is a
//! LIF neuron with `leak = 0`, threshold `(z + 0.5) * value_max / levels`, fed one
//! contribution of `potential` (source `mp_driven_psp`, mapping weight 1, PSC multiplier 1),
//! stepped once with `update_neuron_lif`.
//!
//! NOT driven by Rust code here: the threshold ramp itself is built by the feagi-mcp tool
//! `expand_value_to_bins` (`feagi-mcp/src/feagi_mcp/value_coding.py`, `bins_geometry`,
//! `THRESHOLD_LEVEL_FRACTION = 0.5`); this test mirrors that formula as input data. The
//! depth-plane quantisation (`level / depth_levels`) is done by the sensor encoder, which is
//! not in feagi-core; the depth traces present already-quantised values.

use feagi_npu_neural::update_neuron_lif;
use feagi_structures::neuron_voxels::class_potential::{
    decode_class_potential, encode_class_potential,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

const FILE_NAME: &str = "golden-traces.json";
const FIXTURE_DIR: &str = "tests/fixtures/value-encoding";

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Settings {
    /// `"class_map"` (inputs are class ids) or `"value"` (inputs are plain values).
    kind: String,
    /// Class count (class map) or bin count (value); also the number of bins.
    levels: u32,
    value_max: f32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Row {
    input: f32,
    potential: f32,
    decoded_class: Option<u32>,
    fired_bins: Vec<u32>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Trace {
    name: String,
    settings: Settings,
    rows: Vec<Row>,
}

#[derive(Serialize, Deserialize)]
struct Golden {
    source_crates: Vec<String>,
    functions: Vec<String>,
    feagi_core_commit: String,
    traces: Vec<Trace>,
}

fn bin_threshold(z: u32, levels: u32, value_max: f32) -> f32 {
    (z as f32 + 0.5) * (value_max / levels as f32)
}

fn fired_bins(potential: f32, levels: u32, value_max: f32) -> Vec<u32> {
    if potential == 0.0 {
        return Vec::new();
    }
    (0..levels)
        .filter(|&z| {
            let mut v = 0.0f32;
            let mut frac = 0.0f32;
            update_neuron_lif(
                &mut v,
                &mut frac,
                bin_threshold(z, levels, value_max),
                0.0,
                0.0,
                0.0,
                potential,
                1,
            )
        })
        .collect()
}

fn class_trace(name: &str, class_count: u32) -> Trace {
    let rows = (0..class_count)
        .map(|c| {
            let potential = encode_class_potential(c, class_count).unwrap();
            Row {
                input: c as f32,
                potential,
                decoded_class: decode_class_potential(potential, class_count),
                fired_bins: fired_bins(potential, class_count, 1.0),
            }
        })
        .collect();
    Trace {
        name: name.to_string(),
        settings: Settings {
            kind: "class_map".to_string(),
            levels: class_count,
            value_max: 1.0,
        },
        rows,
    }
}

fn value_trace(name: &str, levels: u32, values: &[f32]) -> Trace {
    let rows = values
        .iter()
        .map(|&v| Row {
            input: v,
            potential: v,
            decoded_class: None,
            fired_bins: fired_bins(v, levels, 1.0),
        })
        .collect();
    Trace {
        name: name.to_string(),
        settings: Settings {
            kind: "value".to_string(),
            levels,
            value_max: 1.0,
        },
        rows,
    }
}

fn generate_traces() -> Vec<Trace> {
    let depth8: Vec<f32> = (0..=8).map(|l| l as f32 / 8.0).collect();
    vec![
        class_trace("class_map_4_classes", 4),
        class_trace("class_map_19_classes", 19),
        // Quantised depth with 8 levels; level 0 is "no return" (silent).
        value_trace("depth_8_levels", 8, &depth8),
        // Arbitrary values into 4 bins: a value in level k fires bins 0..k-1.
        value_trace(
            "value_4_bins_arbitrary",
            4,
            &[0.0, 0.05, 0.24, 0.26, 0.5, 0.74, 0.76, 1.0],
        ),
    ]
}

fn golden(commit: &str, traces: Vec<Trace>) -> Golden {
    Golden {
        source_crates: vec![
            "feagi-structures".to_string(),
            "feagi-npu-neural".to_string(),
        ],
        functions: vec![
            "encode_class_potential".to_string(),
            "decode_class_potential".to_string(),
            "update_neuron_lif".to_string(),
        ],
        feagi_core_commit: commit.to_string(),
        traces,
    }
}

#[test]
fn value_encoding_golden_traces_match_checked_in_json() {
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
    assert_eq!(
        checked_in.traces, traces,
        "value encoding drifted from the golden traces"
    );

    // Sanity: class c of N fires bins 0..=c and decodes back to c.
    for t in traces.iter().filter(|t| t.settings.kind == "class_map") {
        for (c, row) in t.rows.iter().enumerate() {
            assert_eq!(row.decoded_class, Some(c as u32));
            let expected: Vec<u32> = (0..=c as u32).collect();
            assert_eq!(row.fired_bins, expected, "{} class {c}", t.name);
        }
    }
    // A silent voxel fires nothing.
    assert!(traces[2].rows[0].fired_bins.is_empty());
}

#[test]
fn drift_is_detected_negative_case() {
    let traces = generate_traces();
    let mut altered = traces.clone();
    altered[1].rows[5].potential += 1e-4;
    assert_ne!(altered, traces);
    let mut shifted = traces.clone();
    shifted[3].rows[3].fired_bins.pop();
    assert_ne!(shifted, traces);
}

#[test]
fn json_round_trips_f32_exactly() {
    let traces = generate_traces();
    let text = serde_json::to_string(&golden("x", traces.clone())).unwrap();
    let back: Golden = serde_json::from_str(&text).unwrap();
    assert_eq!(back.traces, traces);
}
