// Copyright 2026 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! v4 -> v5 migrator.
//!
//! Schema v4 was published with `kernel_memory_id`. The classifier model then
//! renamed that field to `sample_memory_id` and added `error_area_id` without
//! a version bump. This step rewrites both shapes onto v5:
//!
//! - a v4 record that still has `kernel_memory_id`
//! - a record already written with `sample_memory_id` while the schema integer
//!   was still 4
//!
//! Scanner-only keys (`kernel_size`, `class_count`, `mask_area_id`, `fields`)
//! have no v5 equivalent. Field areas become sample inputs. Scan-twin areas
//! stay in the blueprint and are no longer referenced by the classifier.

use std::collections::BTreeSet;

use feagi_structures::genomic::cortical_area::CorticalID;
use serde_json::{json, Map, Value};

use crate::genome::migration::{MigrationError, MigrationStepDiagnostics, Migrator};
use crate::genome::schema::GenomeSchemaVersion;

const SAMPLE_MORPHOLOGY: &str = "classifier_sample";
const CLASS_MORPHOLOGY: &str = "classifier_class";
const OLD_SAMPLE_MORPHOLOGIES: &[&str] = &["episodic_memory", "episodic_scan"];
const OLD_CLASS_MORPHOLOGY: &[&str] = &["episodic_memory"];

/// Advances a hierarchical genome from schema v4 to schema v5.
#[derive(Debug, Default, Clone, Copy)]
pub struct V4ToV5Migrator;

impl V4ToV5Migrator {
    pub const fn new() -> Self {
        Self
    }
}

impl Migrator for V4ToV5Migrator {
    fn from_version(&self) -> GenomeSchemaVersion {
        GenomeSchemaVersion(4)
    }

    fn to_version(&self) -> GenomeSchemaVersion {
        GenomeSchemaVersion(5)
    }

    fn name(&self) -> &'static str {
        "v4_to_v5"
    }

    fn migrate(&self, genome: &mut Value) -> Result<MigrationStepDiagnostics, MigrationError> {
        let mut diag = MigrationStepDiagnostics::new(self.from_version(), self.to_version());
        let converted =
            migrate_classifiers(genome).map_err(|reason| MigrationError::StepFailed {
                name: self.name(),
                from: self.from_version(),
                to: self.to_version(),
                reason,
            })?;
        if converted == 0 {
            diag.record("no classifier records to convert");
        } else {
            diag.record(format!(
                "converted {converted} classifier records onto sample_memory_id and error_area_id"
            ));
        }
        Ok(diag)
    }
}

fn migrate_classifiers(genome: &mut Value) -> Result<usize, String> {
    let root = genome
        .as_object_mut()
        .ok_or_else(|| "genome must be an object".to_string())?;
    rewrite_kernel_memory_properties(root)?;
    let classifier_ids = classifier_ids(root);
    if classifier_ids.is_empty() {
        return Ok(0);
    }
    let mut used_ids = blueprint_ids(root);
    let mut converted = 0usize;
    for classifier_id in classifier_ids {
        let changed = migrate_one(root, &classifier_id, &mut used_ids)?;
        if changed {
            converted += 1;
        }
    }
    Ok(converted)
}

fn classifier_ids(root: &Map<String, Value>) -> Vec<String> {
    let Some(classifiers) = root.get("classifiers").and_then(|v| v.as_object()) else {
        return Vec::new();
    };
    let mut ids: Vec<String> = classifiers.keys().cloned().collect();
    ids.sort();
    ids
}

fn blueprint_ids(root: &Map<String, Value>) -> BTreeSet<String> {
    root.get("blueprint")
        .and_then(|v| v.as_object())
        .map(|blueprint| blueprint.keys().cloned().collect())
        .unwrap_or_default()
}

fn migrate_one(
    root: &mut Map<String, Value>,
    classifier_id: &str,
    used_ids: &mut BTreeSet<String>,
) -> Result<bool, String> {
    let classifier = root
        .get("classifiers")
        .and_then(|v| v.as_object())
        .and_then(|map| map.get(classifier_id))
        .and_then(|v| v.as_object())
        .cloned()
        .ok_or_else(|| format!("classifier {classifier_id} must be an object"))?;

    let sample_memory_id = non_empty(classifier.get("sample_memory_id"));
    let kernel_memory_id = non_empty(classifier.get("kernel_memory_id"));
    let sample_memory_id = match (sample_memory_id, kernel_memory_id) {
        (Some(sample), Some(kernel)) if sample != kernel => {
            return Err(format!(
                "classifier {classifier_id} has conflicting sample_memory_id and kernel_memory_id"
            ));
        }
        (Some(sample), _) => sample.to_string(),
        (None, Some(kernel)) => kernel.to_string(),
        (None, None) => {
            return Err(format!(
                "classifier {classifier_id} is missing sample_memory_id and kernel_memory_id"
            ));
        }
    };
    let class_memory_id = non_empty(classifier.get("class_memory_id"))
        .ok_or_else(|| format!("classifier {classifier_id} is missing class_memory_id"))?
        .to_string();

    let sample_sources = sample_sources(&classifier);
    let class_source = non_empty(classifier.get("class_area_id")).map(str::to_string);
    if let Some(blueprint) = root.get_mut("blueprint").and_then(|v| v.as_object_mut()) {
        for source in &sample_sources {
            rewrite_edge(
                blueprint,
                source,
                &sample_memory_id,
                OLD_SAMPLE_MORPHOLOGIES,
                SAMPLE_MORPHOLOGY,
            );
        }
        if let Some(class_source) = &class_source {
            rewrite_edge(
                blueprint,
                class_source,
                &class_memory_id,
                OLD_CLASS_MORPHOLOGY,
                CLASS_MORPHOLOGY,
            );
        }
    }

    let parent_region_id = non_empty(classifier.get("parent_region_id")).map(str::to_string);
    let name = non_empty(classifier.get("name"))
        .unwrap_or("classifier")
        .to_string();
    let coordinates = coordinates_of(&classifier);
    let needs_error_area = non_empty(classifier.get("error_area_id")).is_none();
    let error_area_id = if needs_error_area {
        let error_area_id = next_error_area_id(used_ids);
        used_ids.insert(error_area_id.clone());
        insert_error_area(
            root,
            &error_area_id,
            &name,
            coordinates,
            parent_region_id.as_deref(),
        )?;
        error_area_id
    } else {
        non_empty(classifier.get("error_area_id"))
            .unwrap_or_default()
            .to_string()
    };

    let classifier = root
        .get_mut("classifiers")
        .and_then(|v| v.as_object_mut())
        .and_then(|map| map.get_mut(classifier_id))
        .and_then(|v| v.as_object_mut())
        .ok_or_else(|| format!("classifier {classifier_id} must be an object"))?;

    let mut changed = false;
    if classifier.get("sample_memory_id") != Some(&json!(sample_memory_id)) {
        classifier.insert("sample_memory_id".to_string(), json!(sample_memory_id));
        changed = true;
    }
    if !classifier.contains_key("sample_area_ids") {
        classifier.insert("sample_area_ids".to_string(), json!(sample_sources));
        changed = true;
    }
    if !classifier.contains_key("class_bindings") {
        let bindings = class_source
            .map(|class_area_id| {
                vec![json!({
                    "class_area_id": class_area_id,
                    "class_output_area_id": ""
                })]
            })
            .unwrap_or_default();
        classifier.insert("class_bindings".to_string(), json!(bindings));
        changed = true;
    }
    if classifier.get("error_area_id") != Some(&json!(error_area_id)) {
        classifier.insert("error_area_id".to_string(), json!(error_area_id));
        changed = true;
    }
    for key in [
        "kernel_memory_id",
        "kernel_area_id",
        "class_area_id",
        "mask_area_id",
        "class_count",
        "kernel_size",
        "fields",
        "training_mode",
    ] {
        if classifier.remove(key).is_some() {
            changed = true;
        }
    }
    Ok(changed || needs_error_area)
}

fn sample_sources(classifier: &Map<String, Value>) -> Vec<String> {
    if let Some(existing) = classifier.get("sample_area_ids").and_then(|v| v.as_array()) {
        return existing
            .iter()
            .filter_map(|v| v.as_str())
            .map(str::trim)
            .filter(|id| !id.is_empty())
            .map(str::to_string)
            .collect();
    }
    let mut sources = Vec::new();
    if let Some(kernel) = non_empty(classifier.get("kernel_area_id")) {
        sources.push(kernel.to_string());
    }
    if let Some(fields) = classifier.get("fields").and_then(|v| v.as_array()) {
        for field in fields {
            if let Some(field_id) = non_empty(field.get("field_area_id")) {
                if !sources.iter().any(|id| id == field_id) {
                    sources.push(field_id.to_string());
                }
            }
        }
    }
    sources
}

fn rewrite_kernel_memory_properties(root: &mut Map<String, Value>) -> Result<(), String> {
    let Some(blueprint) = root.get_mut("blueprint").and_then(|v| v.as_object_mut()) else {
        return Ok(());
    };
    let mut area_ids: Vec<String> = blueprint.keys().cloned().collect();
    area_ids.sort();
    for area_id in area_ids {
        let Some(area) = blueprint.get_mut(&area_id).and_then(|v| v.as_object_mut()) else {
            continue;
        };
        if area.get("classifier_role").and_then(|v| v.as_str()) == Some("kernel_memory") {
            area.insert("classifier_role".to_string(), json!("sample_memory"));
        }
        let old_key = "classifier_kernel_memory_id";
        let new_key = "classifier_sample_memory_id";
        if let Some(old_value) = area.get(old_key).cloned() {
            match area.get(new_key) {
                Some(new_value) if new_value != &old_value => {
                    return Err(format!(
                        "area {area_id} has conflicting {old_key} and {new_key}"
                    ));
                }
                Some(_) => {
                    area.remove(old_key);
                }
                None => {
                    area.insert(new_key.to_string(), old_value);
                    area.remove(old_key);
                }
            }
        }
    }
    Ok(())
}

fn rewrite_edge(blueprint: &mut Map<String, Value>, src: &str, dst: &str, from: &[&str], to: &str) {
    let Some(rules) = blueprint
        .get_mut(src)
        .and_then(|v| v.as_object_mut())
        .and_then(|area| area.get_mut("cortical_mapping_dst"))
        .and_then(|v| v.as_object_mut())
        .and_then(|dst_map| dst_map.get_mut(dst))
        .and_then(|v| v.as_array_mut())
    else {
        return;
    };
    let has_target = rules
        .iter()
        .any(|rule| rule.get("morphology_id").and_then(|v| v.as_str()) == Some(to));
    if !has_target {
        if let Some(rule) = rules.iter_mut().find(|rule| {
            let morphology = rule.get("morphology_id").and_then(|v| v.as_str());
            from.iter().any(|name| morphology == Some(*name))
        }) {
            if let Some(rule) = rule.as_object_mut() {
                rule.insert("morphology_id".to_string(), json!(to));
            }
        }
    }
    let mut kept_target = false;
    rules.retain(|rule| {
        let morphology = rule.get("morphology_id").and_then(|v| v.as_str());
        if morphology == Some(to) {
            if kept_target {
                return false;
            }
            kept_target = true;
            return true;
        }
        !from.iter().any(|name| morphology == Some(*name))
    });
}

fn insert_error_area(
    root: &mut Map<String, Value>,
    error_area_id: &str,
    classifier_name: &str,
    coordinates: [i32; 3],
    parent_region_id: Option<&str>,
) -> Result<(), String> {
    let blueprint = root
        .entry("blueprint".to_string())
        .or_insert_with(|| json!({}));
    let blueprint = blueprint
        .as_object_mut()
        .ok_or_else(|| "blueprint must be an object".to_string())?;
    if !blueprint.contains_key(error_area_id) {
        blueprint.insert(
            error_area_id.to_string(),
            json!({
                "cortical_name": format!("{classifier_name} error"),
                "block_boundaries": [1, 1, 1],
                "relative_coordinate": [coordinates[0], coordinates[1] + 2, coordinates[2]],
                "cortical_type": "CUSTOM",
                "classifier_role": "error",
                "classifier_assembly": true
            }),
        );
    }
    if let Some(parent_region_id) = parent_region_id {
        append_region_area(root, parent_region_id, error_area_id);
    }
    Ok(())
}

fn append_region_area(root: &mut Map<String, Value>, region_id: &str, area_id: &str) {
    let Some(regions) = root
        .get_mut("brain_regions")
        .and_then(|v| v.as_object_mut())
    else {
        return;
    };
    let Some(region) = regions.get_mut(region_id).and_then(|v| v.as_object_mut()) else {
        return;
    };
    let areas = region
        .entry("areas".to_string())
        .or_insert_with(|| json!([]));
    let Some(areas) = areas.as_array_mut() else {
        return;
    };
    if !areas.iter().any(|value| value.as_str() == Some(area_id)) {
        areas.push(json!(area_id));
    }
}

fn next_error_area_id(used_ids: &BTreeSet<String>) -> String {
    let mut serial = 1u32;
    loop {
        let mut bytes = [b'c', b'_', b'_', b'_', 0, 0, 0, 0];
        bytes[4..8].copy_from_slice(&serial.to_le_bytes());
        let id = CorticalID::try_from_bytes(&bytes)
            .expect("custom cortical id bytes are a valid cortical id")
            .as_base_64();
        if !used_ids.contains(&id) {
            return id;
        }
        serial = serial.saturating_add(1);
    }
}

fn coordinates_of(classifier: &Map<String, Value>) -> [i32; 3] {
    let Some(coords) = classifier.get("coordinates_3d").and_then(|v| v.as_array()) else {
        return [0, 0, 0];
    };
    if coords.len() != 3 {
        return [0, 0, 0];
    }
    [
        coords[0].as_i64().unwrap_or(0) as i32,
        coords[1].as_i64().unwrap_or(0) as i32,
        coords[2].as_i64().unwrap_or(0) as i32,
    ]
}

fn non_empty(value: Option<&Value>) -> Option<&str> {
    value
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|text| !text.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn kernel_genome() -> Value {
        json!({
            "genome_schema_version": 4,
            "modulators": {},
            "blueprint": {
                "sample-src": {
                    "cortical_name": "sample",
                    "cortical_mapping_dst": {
                        "kernel-mem": [{ "morphology_id": "episodic_memory" }]
                    }
                },
                "kernel-mem": {
                    "cortical_name": "kernel memory",
                    "classifier_role": "kernel_memory",
                    "classifier_kernel_memory_id": "class-mem"
                },
                "class-mem": { "cortical_name": "class memory" }
            },
            "brain_regions": {
                "circuit": { "parent_region_id": "root", "areas": ["kernel-mem", "class-mem"] }
            },
            "classifiers": {
                "clf-1": {
                    "name": "MNIST",
                    "parent_region_id": "circuit",
                    "coordinates_3d": [1, 2, 3],
                    "training_mode": "kernel",
                    "kernel_area_id": "sample-src",
                    "class_area_id": "class-src",
                    "kernel_memory_id": "kernel-mem",
                    "class_memory_id": "class-mem",
                    "fields": [{ "field_area_id": "field-1", "scan_twin_id": "twin-1" }]
                }
            }
        })
    }

    #[test]
    fn converts_kernel_memory_id_once() {
        let mut genome = kernel_genome();
        let migrator = V4ToV5Migrator::new();
        let diag = migrator.migrate(&mut genome).unwrap();
        assert_eq!(diag.transformations.len(), 1);
        let classifier = &genome["classifiers"]["clf-1"];
        assert_eq!(classifier["sample_memory_id"], "kernel-mem");
        assert!(classifier.get("kernel_memory_id").is_none());
        assert_eq!(
            classifier["sample_area_ids"],
            json!(["sample-src", "field-1"])
        );
        assert_eq!(
            classifier["class_bindings"][0]["class_area_id"],
            "class-src"
        );
        let error_area_id = classifier["error_area_id"].as_str().unwrap().to_string();
        assert!(genome["blueprint"].get(&error_area_id).is_some());
        assert!(genome["brain_regions"]["circuit"]["areas"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value.as_str() == Some(error_area_id.as_str())));
        assert_eq!(
            genome["blueprint"]["sample-src"]["cortical_mapping_dst"]["kernel-mem"][0]
                ["morphology_id"],
            "classifier_sample"
        );
        assert_eq!(
            genome["blueprint"]["kernel-mem"]["classifier_role"],
            "sample_memory"
        );
        assert_eq!(
            genome["blueprint"]["kernel-mem"]["classifier_sample_memory_id"],
            "class-mem"
        );

        let again = migrator.migrate(&mut genome).unwrap();
        assert!(again.transformations[0].contains("no classifier"));
        assert_eq!(
            genome["classifiers"]["clf-1"]["error_area_id"],
            error_area_id
        );
        assert_eq!(
            genome["blueprint"].as_object().unwrap().len(),
            4,
            "a second pass must not allocate another error area"
        );
    }

    #[test]
    fn leaves_an_already_renamed_v4_classifier_in_place() {
        let mut genome = json!({
            "modulators": {},
            "blueprint": { "err-1": { "cortical_name": "error" } },
            "classifiers": {
                "clf-1": {
                    "name": "MNIST",
                    "parent_region_id": "circuit",
                    "sample_area_ids": ["sample-src"],
                    "class_bindings": [],
                    "sample_memory_id": "sample-mem",
                    "class_memory_id": "class-mem",
                    "error_area_id": "err-1"
                }
            }
        });
        V4ToV5Migrator::new().migrate(&mut genome).unwrap();
        assert_eq!(
            genome["classifiers"]["clf-1"]["sample_memory_id"],
            "sample-mem"
        );
        assert_eq!(genome["classifiers"]["clf-1"]["error_area_id"], "err-1");
        assert_eq!(genome["blueprint"].as_object().unwrap().len(), 1);
    }

    #[test]
    fn empty_classifier_map_records_a_diagnostic() {
        let mut genome = json!({ "classifiers": {}, "modulators": {} });
        let diag = V4ToV5Migrator::new().migrate(&mut genome).unwrap();
        assert!(diag.transformations[0].contains("no classifier"));
    }
}
