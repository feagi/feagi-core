// Copyright 2026 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! V7 normalizer: repairs well-known bad values in a genome stamped v7.
//!
//! Three corrections, applied in this order, each recorded by id:
//!
//! 1. `leak_coefficient` values in `(1, 100]` are legacy percents and become
//!    fractions (`v / 100`). Values above 100, negative or non-finite values are
//!    not converted: they stay in the genome and remain blocking validator errors.
//! 2. Memory twin areas whose `memory_twin_of` area is not defined are dropped.
//!    This discards the twin's neurons and learned rows, so each dropped twin is
//!    reported by id. References to a dropped twin (`memory_twin_areas` values
//!    and `cortical_mapping_dst` keys on other areas) are removed with it.
//! 3. Brain-region membership and IO lists (`cortical_areas`, `inputs`,
//!    `outputs`, `designated_inputs`, `designated_outputs`) lose entries that
//!    reference no defined area.

use std::collections::HashSet;

use serde_json::{Map, Value};

use super::{NormalizationDiagnostics, Normalizer};
use crate::genome::migration::MigrationError;
use crate::genome::parser::string_to_cortical_id;
use crate::genome::schema::GenomeSchemaVersion;
use crate::validator::LEAK_FRACTION_MAX;

/// Legacy leak values were percents; dividing by this scale gives the fraction.
const LEGACY_PERCENT_SCALE: f64 = 100.0;

/// Region list keys that hold cortical area ids.
const REGION_AREA_LIST_KEYS: [&str; 5] = [
    "cortical_areas",
    "inputs",
    "outputs",
    "designated_inputs",
    "designated_outputs",
];

/// Normalizer for schema v7.
#[derive(Debug, Default, Clone, Copy)]
pub struct V7Normalizer;

impl V7Normalizer {
    pub const fn new() -> Self {
        Self
    }
}

impl Normalizer for V7Normalizer {
    fn schema_version(&self) -> GenomeSchemaVersion {
        GenomeSchemaVersion(7)
    }

    fn name(&self) -> &'static str {
        "v7_normalizer"
    }

    fn normalize(&self, genome: &mut Value) -> Result<NormalizationDiagnostics, MigrationError> {
        let mut diag = NormalizationDiagnostics::new(GenomeSchemaVersion(7));
        let Some(root) = genome.as_object_mut() else {
            return Ok(diag);
        };
        let Some(Value::Object(blueprint)) = root.get_mut("blueprint") else {
            return Ok(diag);
        };
        convert_legacy_percent_leak(blueprint, &mut diag);
        drop_orphan_memory_twins(blueprint, &mut diag);
        let defined = defined_area_ids(blueprint);
        if let Some(Value::Object(regions)) = root.get_mut("brain_regions") {
            prune_region_references(regions, &defined, &mut diag);
        }
        Ok(diag)
    }
}

/// Canonical (base64) form of an area id, so raw and encoded spellings compare equal.
fn canonical_id(raw: &str) -> String {
    string_to_cortical_id(raw)
        .map(|id| id.as_base_64())
        .unwrap_or_else(|_| raw.to_string())
}

/// Canonical ids of every area in the blueprint.
fn defined_area_ids(blueprint: &Map<String, Value>) -> HashSet<String> {
    blueprint.keys().map(|key| canonical_id(key)).collect()
}

/// Sorted keys, so diagnostics and results are deterministic.
fn sorted_keys(map: &Map<String, Value>) -> Vec<String> {
    let mut keys: Vec<String> = map.keys().cloned().collect();
    keys.sort();
    keys
}

/// Convert legacy percent leaks in `(1, 100]` to fractions.
fn convert_legacy_percent_leak(
    blueprint: &mut Map<String, Value>,
    diag: &mut NormalizationDiagnostics,
) {
    for area_id in sorted_keys(blueprint) {
        let Some(area) = blueprint.get_mut(&area_id).and_then(|v| v.as_object_mut()) else {
            continue;
        };
        let Some(leak) = area.get("leak_coefficient").and_then(|v| v.as_f64()) else {
            continue;
        };
        if !leak.is_finite() || leak <= LEAK_FRACTION_MAX || leak > LEGACY_PERCENT_SCALE {
            continue;
        }
        let fraction = leak / LEGACY_PERCENT_SCALE;
        area.insert("leak_coefficient".to_string(), Value::from(fraction));
        diag.record(format!(
            "area {area_id}: leak_coefficient {leak} is a legacy percent; converted to fraction {fraction}"
        ));
    }
}

/// Drop twin areas whose `memory_twin_of` area is not defined, repeating until stable
/// so a twin of a dropped twin goes too.
fn drop_orphan_memory_twins(
    blueprint: &mut Map<String, Value>,
    diag: &mut NormalizationDiagnostics,
) {
    loop {
        let defined = defined_area_ids(blueprint);
        let mut orphans: Vec<(String, String)> = Vec::new();
        for area_id in sorted_keys(blueprint) {
            let twin_of = blueprint[&area_id]
                .get("memory_twin_of")
                .and_then(|v| v.as_str())
                .filter(|id| !id.is_empty());
            if let Some(twin_of) = twin_of {
                if !defined.contains(&canonical_id(twin_of)) {
                    orphans.push((area_id, twin_of.to_string()));
                }
            }
        }
        if orphans.is_empty() {
            return;
        }
        let mut dropped: HashSet<String> = HashSet::new();
        for (twin_id, twin_of) in orphans {
            blueprint.remove(&twin_id);
            dropped.insert(canonical_id(&twin_id));
            diag.record(format!(
                "dropped memory twin area {twin_id}: its memory_twin_of area {twin_of} is not defined (twin neurons and learned rows are discarded)"
            ));
        }
        scrub_dropped_twin_references(blueprint, &dropped, diag);
    }
}

/// Remove `memory_twin_areas` values and `cortical_mapping_dst` keys that name a dropped twin.
fn scrub_dropped_twin_references(
    blueprint: &mut Map<String, Value>,
    dropped: &HashSet<String>,
    diag: &mut NormalizationDiagnostics,
) {
    for owner_id in sorted_keys(blueprint) {
        let Some(owner) = blueprint.get_mut(&owner_id).and_then(|v| v.as_object_mut()) else {
            continue;
        };
        if let Some(Value::Object(index)) = owner.get_mut("memory_twin_areas") {
            for upstream_id in sorted_keys(index) {
                let names_dropped = index[&upstream_id]
                    .as_str()
                    .is_some_and(|twin| dropped.contains(&canonical_id(twin)));
                if names_dropped {
                    if let Some(removed) = index.remove(&upstream_id) {
                        diag.record(format!(
                            "area {owner_id}: removed memory_twin_areas entry {upstream_id} -> {} (twin dropped)",
                            removed.as_str().unwrap_or_default()
                        ));
                    }
                }
            }
        }
        if let Some(Value::Object(dst)) = owner.get_mut("cortical_mapping_dst") {
            for dst_id in sorted_keys(dst) {
                if dropped.contains(&canonical_id(&dst_id)) {
                    dst.remove(&dst_id);
                    diag.record(format!(
                        "area {owner_id}: removed cortical_mapping_dst entry to dropped twin {dst_id}"
                    ));
                }
            }
        }
    }
}

/// Prune region area-id lists, at the region level and under a nested `properties` map.
fn prune_region_references(
    regions: &mut Map<String, Value>,
    defined: &HashSet<String>,
    diag: &mut NormalizationDiagnostics,
) {
    for region_id in sorted_keys(regions) {
        let Some(region) = regions.get_mut(&region_id).and_then(|v| v.as_object_mut()) else {
            continue;
        };
        prune_area_lists(region, &region_id, "", defined, diag);
        if let Some(Value::Object(props)) = region.get_mut("properties") {
            prune_area_lists(props, &region_id, "properties.", defined, diag);
        }
    }
}

/// Drop string entries that name no defined area from each area-id list in `holder`.
fn prune_area_lists(
    holder: &mut Map<String, Value>,
    region_id: &str,
    label_prefix: &str,
    defined: &HashSet<String>,
    diag: &mut NormalizationDiagnostics,
) {
    for key in REGION_AREA_LIST_KEYS {
        let Some(Value::Array(entries)) = holder.get_mut(key) else {
            continue;
        };
        entries.retain(|entry| {
            let Some(id) = entry.as_str() else {
                return true;
            };
            if defined.contains(&canonical_id(id)) {
                return true;
            }
            diag.record(format!(
                "brain region {region_id}: pruned {label_prefix}{key} entry {id} (no defined area)"
            ));
            false
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn leak_genome(leak: Value) -> Value {
        json!({
            "genome_schema_version": 7,
            "blueprint": { "area_a": { "leak_coefficient": leak } },
            "brain_regions": {}
        })
    }

    #[test]
    fn fraction_leak_is_unchanged() {
        let mut genome = leak_genome(json!(1.0));
        let diag = V7Normalizer::new().normalize(&mut genome).unwrap();
        assert!(diag.is_clean());
        assert_eq!(
            genome["blueprint"]["area_a"]["leak_coefficient"],
            json!(1.0)
        );
    }

    #[test]
    fn legacy_percent_leak_is_converted_and_recorded() {
        let mut genome = leak_genome(json!(10));
        let diag = V7Normalizer::new().normalize(&mut genome).unwrap();
        assert_eq!(
            genome["blueprint"]["area_a"]["leak_coefficient"],
            json!(0.1)
        );
        assert_eq!(diag.transformations.len(), 1);
        assert!(diag.transformations[0].contains("area_a"));

        let mut full = leak_genome(json!(100));
        V7Normalizer::new().normalize(&mut full).unwrap();
        assert_eq!(full["blueprint"]["area_a"]["leak_coefficient"], json!(1.0));
    }

    #[test]
    fn leak_outside_percent_range_is_not_converted() {
        for bad in [json!(150), json!(-0.5), json!(100.5)] {
            let mut genome = leak_genome(bad.clone());
            let diag = V7Normalizer::new().normalize(&mut genome).unwrap();
            assert!(diag.is_clean());
            assert_eq!(genome["blueprint"]["area_a"]["leak_coefficient"], bad);
        }
    }

    #[test]
    fn normalizing_twice_changes_nothing_more() {
        let mut genome = leak_genome(json!(10));
        V7Normalizer::new().normalize(&mut genome).unwrap();
        let again = V7Normalizer::new().normalize(&mut genome).unwrap();
        assert!(again.is_clean());
        assert_eq!(
            genome["blueprint"]["area_a"]["leak_coefficient"],
            json!(0.1)
        );
    }

    #[test]
    fn region_entries_without_a_defined_area_are_pruned_by_id() {
        let mut genome = json!({
            "blueprint": { "area_a": {}, "area_b": {} },
            "brain_regions": {
                "root": {
                    "cortical_areas": ["area_a", "gone_aa"],
                    "inputs": ["gone_bb"],
                    "outputs": ["area_b"],
                    "designated_inputs": ["area_a", "gone_cc"],
                    "designated_outputs": ["gone_dd"],
                    "properties": { "inputs": ["area_a", "gone_ee"] }
                }
            }
        });
        let diag = V7Normalizer::new().normalize(&mut genome).unwrap();
        let root = &genome["brain_regions"]["root"];
        assert_eq!(root["cortical_areas"], json!(["area_a"]));
        assert_eq!(root["inputs"], json!([]));
        assert_eq!(root["outputs"], json!(["area_b"]));
        assert_eq!(root["designated_inputs"], json!(["area_a"]));
        assert_eq!(root["designated_outputs"], json!([]));
        assert_eq!(root["properties"]["inputs"], json!(["area_a"]));
        for id in ["gone_aa", "gone_bb", "gone_cc", "gone_dd", "gone_ee"] {
            assert!(
                diag.transformations.iter().any(|t| t.contains(id)),
                "{id} must be listed"
            );
        }
        assert_eq!(diag.transformations.len(), 5);
    }

    #[test]
    fn region_membership_matches_across_raw_and_encoded_ids() {
        let power = string_to_cortical_id("_power").unwrap().as_base_64();
        let mut genome = json!({
            "blueprint": { "_power": {} },
            "brain_regions": { "root": { "cortical_areas": [power, "_power"] } }
        });
        let diag = V7Normalizer::new().normalize(&mut genome).unwrap();
        assert!(diag.is_clean());
    }

    #[test]
    fn orphan_twin_is_dropped_by_id_with_its_references() {
        let mut genome = json!({
            "blueprint": {
                "area_a": {},
                "memory_m": {
                    "memory_twin_areas": { "gone_up": "twin_t", "area_a": "twin_ok" },
                    "cortical_mapping_dst": { "twin_t": [{}], "twin_ok": [{}] }
                },
                "twin_t": { "memory_twin_of": "gone_up" },
                "twin_ok": { "memory_twin_of": "area_a" },
                "plain": { "memory_twin_of": null }
            },
            "brain_regions": { "root": { "cortical_areas": ["area_a", "twin_t", "twin_ok"] } }
        });
        let diag = V7Normalizer::new().normalize(&mut genome).unwrap();
        assert!(genome["blueprint"].get("twin_t").is_none());
        assert!(genome["blueprint"].get("twin_ok").is_some());
        assert!(genome["blueprint"].get("plain").is_some());
        assert_eq!(
            genome["blueprint"]["memory_m"]["memory_twin_areas"],
            json!({ "area_a": "twin_ok" })
        );
        assert_eq!(
            genome["blueprint"]["memory_m"]["cortical_mapping_dst"],
            json!({ "twin_ok": [{}] })
        );
        assert_eq!(
            genome["brain_regions"]["root"]["cortical_areas"],
            json!(["area_a", "twin_ok"])
        );
        assert!(diag
            .transformations
            .iter()
            .any(|t| t.contains("dropped memory twin area twin_t")));
        assert!(diag
            .transformations
            .iter()
            .any(|t| t.contains("pruned cortical_areas entry twin_t")));
    }

    #[test]
    fn twin_of_a_dropped_twin_is_dropped_too() {
        let mut genome = json!({
            "blueprint": {
                "twin_one": { "memory_twin_of": "gone_up" },
                "twin_two": { "memory_twin_of": "twin_one" }
            },
            "brain_regions": {}
        });
        let diag = V7Normalizer::new().normalize(&mut genome).unwrap();
        assert!(genome["blueprint"].as_object().unwrap().is_empty());
        assert!(diag.transformations.iter().any(|t| t.contains("twin_one")));
        assert!(diag.transformations.iter().any(|t| t.contains("twin_two")));
    }

    #[test]
    fn genome_without_a_blueprint_is_left_alone() {
        let mut genome = json!({
            "brain_regions": { "root": { "cortical_areas": ["anything"] } }
        });
        let diag = V7Normalizer::new().normalize(&mut genome).unwrap();
        assert!(diag.is_clean());
        assert_eq!(
            genome["brain_regions"]["root"]["cortical_areas"],
            json!(["anything"])
        );
    }
}
