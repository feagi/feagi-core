// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Golden traces of the percentage encoders (one reading as neurons along Z) for the Academy
//! visual `percentage-encoding`.
//!
//! Each row drives the real encoders in `coder_shared_functions.rs`:
//! `encode_unsigned_percentage_to_linear_neuron_z_index`,
//! `encode_unsigned_percentage_to_fractional_exponential_neuron_z_indexes`,
//! `encode_signed_percentage_to_linear_neuron_z_index_along_z` and
//! `encode_signed_percentage_to_fractional_exponential_neuron_z_indexes`. By default (plain
//! `cargo test`) the test regenerates the traces in memory and asserts they equal
//! `tests/fixtures/percentage-encoding/golden-traces.json`. `feagi_core_commit` is ignored by
//! the comparison.
//!
//! Regenerate both checked-in copies:
//!
//! ```text
//! FEAGI_CORE_COMMIT=$(git rev-parse HEAD) \
//! FEAGI_WRITE_VISUAL_FIXTURES=<feagi-core>/crates/feagi-sensorimotor/tests/fixtures/percentage-encoding:<feagi-react-core-pro>/src/feagi-academy/visuals/percentage-encoding/fixtures \
//! cargo test -p feagi-sensorimotor --lib percentage_encoding_golden
//! ```
//!
//! `FEAGI_WRITE_VISUAL_FIXTURES` is a `:`-separated list of directories; `golden-traces.json`
//! is written into each. The commit is a note only.
//!
//! Layout choice (mirrored input data, recorded in the file header as `layout`): the visual
//! shows ONE reading (one channel, one percentage dimension). A signed linear reading uses the
//! single-column along-Z layout (`percentage_encoder.rs` `signed_d1_linear_uses_along_z_layout`,
//! which selects `encode_signed_percentage_to_linear_neuron_z_index_along_z`); a signed
//! fractional reading uses two columns, positive then negative. That choice is input here, not
//! something the encoders decide. Signed linear is NOT the two-lobe
//! `encode_signed_percentage_to_linear_neuron_z_index`, which the D1 linear path does not use.
//!
//! Row semantics. `reading` is the f32 input (0..1 unsigned, -1..1 signed). `linear_z` are the
//! Z indices the linear encoder fires (always one). `fractional_positive_z` and
//! `fractional_negative_z` are the Z indices the fractional encoder fires in the positive and
//! negative columns (`fractional_negative_z` is empty for unsigned traces). Indices ascend.
//! NOT mirrored here: any decoder.

use super::coder_shared_functions::{
    encode_signed_percentage_to_fractional_exponential_neuron_z_indexes,
    encode_signed_percentage_to_linear_neuron_z_index_along_z,
    encode_unsigned_percentage_to_fractional_exponential_neuron_z_indexes,
    encode_unsigned_percentage_to_linear_neuron_z_index,
};
use crate::data_types::{Percentage, SignedPercentage};
use serde::{Deserialize, Serialize};
use std::path::Path;

const FILE_NAME: &str = "golden-traces.json";
const FIXTURE_DIR: &str = "tests/fixtures/percentage-encoding";

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Settings {
    signed: bool,
    planes: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Row {
    reading: f32,
    linear_z: Vec<u32>,
    fractional_positive_z: Vec<u32>,
    fractional_negative_z: Vec<u32>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Trace {
    name: String,
    settings: Settings,
    rows: Vec<Row>,
}

#[derive(Serialize, Deserialize)]
struct Layout {
    channels: u32,
    linear_signed: String,
    fractional_signed: String,
}

#[derive(Serialize, Deserialize)]
struct Golden {
    source_crates: Vec<String>,
    functions: Vec<String>,
    layout: Layout,
    feagi_core_commit: String,
    traces: Vec<Trace>,
}

const UNSIGNED_READINGS: [f32; 22] = [
    0.0, 0.001, 0.01, 0.1, 0.125, 0.2, 0.25, 0.3, 0.3333333, 0.4, 0.5, 0.6, 0.6666667, 0.7, 0.74,
    0.75, 0.76, 0.875, 0.9, 0.99, 0.9999999, 1.0,
];

const SIGNED_READINGS: [f32; 25] = [
    -1.0, -0.9999999, -0.875, -0.75, -0.6, -0.5, -0.3333333, -0.25, -0.1, -0.001, 0.0, 0.001, 0.1,
    0.25, 0.3333333, 0.5, 0.6, 0.74, 0.75, 0.76, 0.875, 0.9, 0.99, 0.9999999, 1.0,
];

fn run_row(signed: bool, planes: u32, reading: f32) -> Row {
    let mut linear = Vec::new();
    let mut positive = Vec::new();
    let mut negative = Vec::new();
    if signed {
        let value = SignedPercentage::new_from_m1_1_unchecked(reading);
        encode_signed_percentage_to_linear_neuron_z_index_along_z(
            &value,
            planes as f32,
            &mut linear,
        );
        encode_signed_percentage_to_fractional_exponential_neuron_z_indexes(
            &value,
            planes,
            &mut positive,
            &mut negative,
        );
    } else {
        let value = Percentage::new_from_0_1_unchecked(reading);
        encode_unsigned_percentage_to_linear_neuron_z_index(&value, planes as f32, &mut linear);
        encode_unsigned_percentage_to_fractional_exponential_neuron_z_indexes(
            &value,
            planes,
            &mut positive,
        );
    }
    Row {
        reading,
        linear_z: linear,
        fractional_positive_z: positive,
        fractional_negative_z: negative,
    }
}

fn generate_traces() -> Vec<Trace> {
    let mut traces = Vec::new();
    for signed in [false, true] {
        for planes in 1..=12u32 {
            let readings: &[f32] = if signed {
                &SIGNED_READINGS
            } else {
                &UNSIGNED_READINGS
            };
            traces.push(Trace {
                name: format!(
                    "{}_{}_planes",
                    if signed { "signed" } else { "unsigned" },
                    planes
                ),
                settings: Settings { signed, planes },
                rows: readings
                    .iter()
                    .map(|&r| run_row(signed, planes, r))
                    .collect(),
            });
        }
    }
    traces
}

fn golden(commit: &str, traces: Vec<Trace>) -> Golden {
    Golden {
        source_crates: vec!["feagi-sensorimotor".to_string()],
        functions: vec![
            "encode_unsigned_percentage_to_linear_neuron_z_index".to_string(),
            "encode_unsigned_percentage_to_fractional_exponential_neuron_z_indexes".to_string(),
            "encode_signed_percentage_to_linear_neuron_z_index_along_z".to_string(),
            "encode_signed_percentage_to_fractional_exponential_neuron_z_indexes".to_string(),
        ],
        layout: Layout {
            channels: 1,
            linear_signed: "one column, along Z (+1 at Z 0, -1 at the last plane)".to_string(),
            fractional_signed: "two columns, positive and negative".to_string(),
        },
        feagi_core_commit: commit.to_string(),
        traces,
    }
}

#[test]
fn percentage_encoding_golden_traces_match_checked_in_json() {
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
        "percentage encoding drifted from the golden traces"
    );
}

#[test]
fn percentage_encoding_golden_guide_outcomes() {
    // Guide step 1: reading 0.75 with 4 planes.
    let row = run_row(false, 4, 0.75);
    assert_eq!(row.linear_z, vec![1]);
    assert_eq!(row.fractional_positive_z, vec![0, 1]);
    // Step 2: 8 planes: linear moves to Z 2, fractional unchanged.
    let row = run_row(false, 8, 0.75);
    assert_eq!(row.linear_z, vec![2]);
    assert_eq!(row.fractional_positive_z, vec![0, 1]);
    // Step 3: reading 0 fires only the last plane in both.
    for planes in 1..=8u32 {
        let row = run_row(false, planes, 0.0);
        assert_eq!(row.linear_z, vec![planes - 1]);
        assert_eq!(row.fractional_positive_z, vec![planes - 1]);
    }
    // Reading 1 fires Z 0 linear and all planes fractional.
    let row = run_row(false, 5, 1.0);
    assert_eq!(row.linear_z, vec![0]);
    assert_eq!(row.fractional_positive_z, vec![0, 1, 2, 3, 4]);
    // Signed zero: fractional fires the smallest plane in both columns.
    let row = run_row(true, 4, 0.0);
    assert_eq!(row.fractional_positive_z, vec![3]);
    assert_eq!(row.fractional_negative_z, vec![3]);
}

#[test]
fn percentage_encoding_golden_drift_is_detected_negative_case() {
    let traces = generate_traces();
    let mut altered = traces.clone();
    altered[3].rows[5].linear_z[0] += 1;
    assert_ne!(altered, traces);
}

/// The public coder interface (`single_voxel_decode::encode_percentage_to_z_indexes`) selects
/// its coding from the IO configuration flag and must reproduce every checked-in golden row.
#[test]
fn public_encode_reproduces_checked_in_golden_rows() {
    use crate::single_voxel_decode::encode_percentage_to_z_indexes;
    use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
        FrameChangeHandling, IOCorticalAreaConfigurationFlag as Flag, PercentageNeuronPositioning,
    };

    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURE_DIR)
        .join(FILE_NAME);
    let text = std::fs::read_to_string(&path).expect("fixture missing; see module docs");
    let checked_in: Golden = serde_json::from_str(&text).unwrap();
    assert!(!checked_in.traces.is_empty());

    let frame = FrameChangeHandling::Absolute;
    for trace in &checked_in.traces {
        let depth = trace.settings.planes;
        let (linear_flag, fractional_flag) = if trace.settings.signed {
            (
                Flag::SignedPercentage(frame, PercentageNeuronPositioning::Linear),
                Flag::SignedPercentage(frame, PercentageNeuronPositioning::Fractional),
            )
        } else {
            (
                Flag::Percentage(frame, PercentageNeuronPositioning::Linear),
                Flag::Percentage(frame, PercentageNeuronPositioning::Fractional),
            )
        };
        for row in &trace.rows {
            let linear = encode_percentage_to_z_indexes(&linear_flag, depth, row.reading).unwrap();
            assert_eq!(
                linear.positive, row.linear_z,
                "{} linear {}",
                trace.name, row.reading
            );
            assert!(linear.negative.is_empty());
            let fractional =
                encode_percentage_to_z_indexes(&fractional_flag, depth, row.reading).unwrap();
            assert_eq!(
                fractional.positive, row.fractional_positive_z,
                "{} fractional positive {}",
                trace.name, row.reading
            );
            assert_eq!(
                fractional.negative, row.fractional_negative_z,
                "{} fractional negative {}",
                trace.name, row.reading
            );
        }
    }
}
