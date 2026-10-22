// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Persistence of the optional top-level `change_history` genome key.

The genome carries change entries as opaque JSON: they must survive a full
load/save round trip, be omitted when empty, and never affect signatures.
*/

use feagi_evolutionary::{load_genome_from_json, save_genome_to_json};
use serde_json::{json, Value};
use std::fs;

fn barebones_json() -> Value {
    let genome_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("genomes")
        .join("barebones_genome.json");
    let text = fs::read_to_string(&genome_path)
        .unwrap_or_else(|_| panic!("Failed to read genome file: {}", genome_path.display()));
    serde_json::from_str(&text).expect("barebones genome must be valid JSON")
}

fn sample_history() -> Vec<Value> {
    vec![
        json!({"sequence": 1, "change_id": "a", "operation": {"kind": "delete_cortical_area"}}),
        json!({"sequence": 2, "change_id": "b", "agent_id": "member-1"}),
    ]
}

#[test]
fn change_history_survives_load_and_save() {
    let mut genome_json = barebones_json();
    genome_json["change_history"] = Value::Array(sample_history());

    let genome = load_genome_from_json(&genome_json.to_string()).expect("load with history");
    assert_eq!(genome.change_history, sample_history());

    let saved: Value =
        serde_json::from_str(&save_genome_to_json(&genome).expect("save")).expect("saved JSON");
    assert_eq!(saved["change_history"], Value::Array(sample_history()));

    let reloaded = load_genome_from_json(&saved.to_string()).expect("reload");
    assert_eq!(reloaded.change_history, sample_history());
}

#[test]
fn genome_without_history_loads_empty_and_saves_without_key() {
    let genome = load_genome_from_json(&barebones_json().to_string()).expect("load");
    assert!(genome.change_history.is_empty());

    let saved: Value =
        serde_json::from_str(&save_genome_to_json(&genome).expect("save")).expect("saved JSON");
    assert!(saved.get("change_history").is_none());
}

#[test]
fn change_history_does_not_affect_signatures() {
    let plain = load_genome_from_json(&barebones_json().to_string()).expect("load plain");
    let mut with_history_json = barebones_json();
    with_history_json["change_history"] = Value::Array(sample_history());
    let with_history =
        load_genome_from_json(&with_history_json.to_string()).expect("load with history");

    assert_eq!(plain.signatures.genome, with_history.signatures.genome);
    assert_eq!(
        plain.signatures.blueprint,
        with_history.signatures.blueprint
    );
}

#[test]
fn malformed_change_history_is_rejected() {
    let mut genome_json = barebones_json();
    genome_json["change_history"] = json!({"not": "an array"});
    assert!(load_genome_from_json(&genome_json.to_string()).is_err());
}
