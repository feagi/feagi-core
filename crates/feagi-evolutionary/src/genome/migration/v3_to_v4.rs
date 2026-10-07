// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! v3 -> v4 migrator.
//!
//! Adds the top-level `modulators` section and replaces R-STDP
//! `reward_source_area` / `punishment_source_area` with Reward instance
//! subscriptions. Each referenced source area keeps its identity and gains a
//! 1x1x1 driver area wired from that source.

use std::collections::BTreeMap;

use feagi_structures::genomic::cortical_area::CorticalID;
use serde_json::{json, Map, Value};

use crate::genome::migration::{MigrationError, MigrationStepDiagnostics, Migrator};
use crate::genome::schema::GenomeSchemaVersion;

/// Advances a hierarchical genome from schema v3 to schema v4.
#[derive(Debug, Default, Clone, Copy)]
pub struct V3ToV4Migrator;

impl V3ToV4Migrator {
    pub const fn new() -> Self {
        Self
    }
}

impl Migrator for V3ToV4Migrator {
    fn from_version(&self) -> GenomeSchemaVersion {
        GenomeSchemaVersion(3)
    }

    fn to_version(&self) -> GenomeSchemaVersion {
        GenomeSchemaVersion(4)
    }

    fn name(&self) -> &'static str {
        "v3_to_v4"
    }

    fn migrate(&self, genome: &mut Value) -> Result<MigrationStepDiagnostics, MigrationError> {
        let mut diag = MigrationStepDiagnostics::new(self.from_version(), self.to_version());
        let converted =
            migrate_reward_sources(genome).map_err(|reason| MigrationError::StepFailed {
                name: self.name(),
                from: self.from_version(),
                to: self.to_version(),
                reason,
            })?;
        if converted == 0 {
            diag.record("ensured modulators section; no reward or punishment sources to convert");
        } else {
            diag.record(format!(
                "converted {converted} reward or punishment source references onto Reward instances"
            ));
        }
        Ok(diag)
    }
}

fn migrate_reward_sources(genome: &mut Value) -> Result<usize, String> {
    let root = genome
        .as_object_mut()
        .ok_or_else(|| "genome must be an object".to_string())?;
    if !root.contains_key("modulators") {
        root.insert("modulators".to_string(), json!({}));
    }
    let blueprint = root
        .get("blueprint")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    let mut sources: BTreeMap<String, f32> = BTreeMap::new();
    for area in blueprint.values() {
        collect_sources(area, &mut sources);
    }
    if sources.is_empty() {
        return Ok(0);
    }

    let mut next_serial = next_driver_serial(&blueprint);
    let mut source_to_instance: BTreeMap<String, (String, String)> = BTreeMap::new();
    for (source_id, magnitude) in &sources {
        let instance_id = format!("reward_{}", sanitize_id(source_id));
        let driver = CorticalID::modulator_driver(next_serial);
        next_serial = next_serial.saturating_add(1);
        let driver_id = driver.as_base_64();
        source_to_instance.insert(source_id.clone(), (instance_id, driver_id));
        let _ = magnitude;
    }

    {
        let modulators = root
            .get_mut("modulators")
            .and_then(|v| v.as_object_mut())
            .ok_or_else(|| "modulators must be an object".to_string())?;
        for (source_id, magnitude) in &sources {
            let (instance_id, driver_id) = &source_to_instance[source_id];
            if modulators.contains_key(instance_id) {
                continue;
            }
            modulators.insert(
                instance_id.clone(),
                json!({
                    "type": "synaptic.reward",
                    "magnitude_percent": magnitude,
                    "effect_duration_bursts": 1,
                    "rest_bursts": 0,
                    "graded": false,
                    "driver_cortical_id": driver_id,
                }),
            );
        }
    }

    {
        let blueprint = root
            .get_mut("blueprint")
            .and_then(|v| v.as_object_mut())
            .ok_or_else(|| "blueprint must be an object".to_string())?;
        let mut y = 0i32;
        for (source_id, (instance_id, driver_id)) in &source_to_instance {
            if !blueprint.contains_key(driver_id) {
                blueprint.insert(driver_id.clone(), driver_area(instance_id, y));
                y += 2;
            }
            wire_source_to_driver(blueprint, source_id, driver_id)?;
        }
        rewrite_rules(blueprint, &source_to_instance)?;
    }
    retarget_classifier_affect(root, &source_to_instance);

    let driver_ids: Vec<String> = source_to_instance
        .values()
        .map(|(_, driver)| driver.clone())
        .collect();
    add_drivers_to_root(root, &driver_ids);
    Ok(sources.len())
}

fn collect_sources(area: &Value, sources: &mut BTreeMap<String, f32>) {
    let Some(dst) = area.get("cortical_mapping_dst").and_then(|v| v.as_object()) else {
        return;
    };
    for rules in dst.values() {
        let Some(rules) = rules.as_array() else {
            continue;
        };
        for rule in rules {
            if let Some(id) = rule.get("reward_source_area").and_then(|v| v.as_str()) {
                sources.entry(id.to_string()).or_insert(100.0);
            }
            if let Some(id) = rule.get("punishment_source_area").and_then(|v| v.as_str()) {
                sources.entry(id.to_string()).or_insert(-100.0);
            }
        }
    }
}

fn next_driver_serial(blueprint: &Map<String, Value>) -> u32 {
    let mut serial = 1u32;
    for key in blueprint.keys() {
        if let Ok(id) = CorticalID::try_from_base_64(key) {
            if id.is_modulator_driver() {
                let bytes = id.as_bytes();
                let existing = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
                serial = serial.max(existing.saturating_add(1));
            }
        }
    }
    serial
}

fn sanitize_id(source_id: &str) -> String {
    let mut out = String::new();
    for ch in source_id.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        out.push_str("source");
    }
    out
}

fn driver_area(instance_id: &str, y: i32) -> Value {
    json!({
        "cortical_name": format!("{instance_id} driver"),
        "block_boundaries": [1, 1, 1],
        "relative_coordinate": [0, y, 0],
        "cortical_type": "MODULATOR",
        "cortical_group": "MODULATOR",
        "per_voxel_neuron_cnt": 1,
        "refractory_period": 0,
        "consecutive_fire_cnt_max": 1,
        "snooze_length": 0,
        "mp_driven_psp": false,
        "spike_train": true,
        "firing_threshold": 1.0,
        "leak_coefficient": 0.0,
        "neuron_excitability": 1.0,
        "modulator_instance_id": instance_id
    })
}

fn wire_source_to_driver(
    blueprint: &mut Map<String, Value>,
    source_id: &str,
    driver_id: &str,
) -> Result<(), String> {
    let Some(source) = blueprint.get_mut(source_id) else {
        return Ok(());
    };
    let source = source
        .as_object_mut()
        .ok_or_else(|| format!("cortical area {source_id} must be an object"))?;
    let dst = source
        .entry("cortical_mapping_dst")
        .or_insert_with(|| json!({}));
    let dst = dst
        .as_object_mut()
        .ok_or_else(|| format!("cortical_mapping_dst on {source_id} must be an object"))?;
    let rules = dst.entry(driver_id).or_insert_with(|| json!([]));
    let rules = rules
        .as_array_mut()
        .ok_or_else(|| format!("mapping rules to {driver_id} must be an array"))?;
    if rules.is_empty() {
        rules.push(json!({
            "morphology_id": "projector",
            "morphology_scalar": [1, 1, 1],
            "postSynapticCurrent_multiplier": 1.0
        }));
    }
    Ok(())
}

fn rewrite_rules(
    blueprint: &mut Map<String, Value>,
    source_to_instance: &BTreeMap<String, (String, String)>,
) -> Result<(), String> {
    for area in blueprint.values_mut() {
        let Some(area) = area.as_object_mut() else {
            continue;
        };
        let Some(dst) = area.get_mut("cortical_mapping_dst") else {
            continue;
        };
        let Some(dst) = dst.as_object_mut() else {
            continue;
        };
        for rules in dst.values_mut() {
            let Some(rules) = rules.as_array_mut() else {
                continue;
            };
            for rule in rules {
                let Some(rule) = rule.as_object_mut() else {
                    continue;
                };
                let mut ids: Vec<String> = rule
                    .get("modulators")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(str::to_string))
                            .collect()
                    })
                    .unwrap_or_default();
                if let Some(source) = rule.get("reward_source_area").and_then(|v| v.as_str()) {
                    if let Some((instance_id, _)) = source_to_instance.get(source) {
                        if !ids.contains(instance_id) {
                            ids.push(instance_id.clone());
                        }
                    }
                }
                if let Some(source) = rule.get("punishment_source_area").and_then(|v| v.as_str()) {
                    if let Some((instance_id, _)) = source_to_instance.get(source) {
                        if !ids.contains(instance_id) {
                            ids.push(instance_id.clone());
                        }
                    }
                }
                rule.remove("reward_source_area");
                rule.remove("punishment_source_area");
                if !ids.is_empty() {
                    rule.insert("modulators".to_string(), json!(ids));
                }
            }
        }
    }
    Ok(())
}

fn retarget_classifier_affect(
    root: &mut Map<String, Value>,
    source_to_instance: &BTreeMap<String, (String, String)>,
) {
    let Some(classifiers) = root.get_mut("classifiers").and_then(|v| v.as_object_mut()) else {
        return;
    };
    for classifier in classifiers.values_mut() {
        let Some(classifier) = classifier.as_object_mut() else {
            continue;
        };
        for key in ["pain_area_id", "pleasure_area_id"] {
            let Some(current) = classifier
                .get(key)
                .and_then(|v| v.as_str())
                .map(str::to_string)
            else {
                continue;
            };
            if let Some((_, driver_id)) = source_to_instance.get(&current) {
                classifier.insert(key.to_string(), json!(driver_id));
            }
        }
    }
}

fn add_drivers_to_root(root: &mut Map<String, Value>, driver_ids: &[String]) {
    let root_id_field = root
        .get("brain_regions_root")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let Some(regions) = root
        .get_mut("brain_regions")
        .and_then(|v| v.as_object_mut())
    else {
        return;
    };
    let root_id = root_id_field.or_else(|| {
        regions.iter().find_map(|(id, region)| {
            let parent = region.get("parent_region_id");
            if parent.map(|v| v.is_null()).unwrap_or(true) {
                Some(id.clone())
            } else {
                None
            }
        })
    });
    let Some(root_id) = root_id else {
        return;
    };
    let Some(region) = regions.get_mut(&root_id).and_then(|v| v.as_object_mut()) else {
        return;
    };
    let areas = region.entry("areas").or_insert_with(|| json!([]));
    let Some(areas) = areas.as_array_mut() else {
        return;
    };
    for driver_id in driver_ids {
        let present = areas.iter().any(|v| v.as_str() == Some(driver_id));
        if !present {
            areas.push(json!(driver_id));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn converts_reward_and_punishment_sources_once() {
        let mut genome = json!({
            "genome_schema_version": 3,
            "blueprint": {
                "c3JjMDAwMQ==": {
                    "cortical_name": "src",
                    "cortical_mapping_dst": {
                        "Y2RzdDAwMDE=": [{
                            "morphology_id": "projector",
                            "plasticity_mode": "rstdp",
                            "reward_source_area": "Y3JldzAwMDE=",
                            "punishment_source_area": "Y3B1bjAwMDE="
                        }]
                    }
                },
                "Y3JldzAwMDE=": { "cortical_name": "pleasure" },
                "Y3B1bjAwMDE=": { "cortical_name": "pain" }
            },
            "brain_regions": {
                "root": { "parent_region_id": null, "areas": ["c3JjMDAwMQ=="] }
            },
            "brain_regions_root": "root"
        });
        let migrator = V3ToV4Migrator::new();
        let diag = migrator.migrate(&mut genome).unwrap();
        assert_eq!(diag.transformations.len(), 1);
        let again = migrator.migrate(&mut genome).unwrap();
        assert!(again.transformations[0].contains("no reward"));

        let rule = &genome["blueprint"]["c3JjMDAwMQ=="]["cortical_mapping_dst"]["Y2RzdDAwMDE="][0];
        assert!(rule.get("reward_source_area").is_none());
        assert!(rule.get("punishment_source_area").is_none());
        let mods = rule["modulators"].as_array().unwrap();
        assert_eq!(mods.len(), 2);
        let modulators = genome["modulators"].as_object().unwrap();
        assert_eq!(modulators.len(), 2);
        let pleasure = modulators
            .values()
            .find(|v| v["magnitude_percent"] == 100.0)
            .unwrap();
        let driver = pleasure["driver_cortical_id"].as_str().unwrap();
        assert!(genome["blueprint"].get(driver).is_some());
        assert!(genome["blueprint"]["Y3JldzAwMDE="]["cortical_mapping_dst"]
            .get(driver)
            .is_some());
    }
}
