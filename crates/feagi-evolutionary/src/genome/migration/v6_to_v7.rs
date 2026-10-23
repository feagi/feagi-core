// Copyright 2026 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! v6 -> v7 migrator.
//!
//! Replaces the per-mapping `gate_source_area` field with `synaptic.gate`
//! modulator instances. Each distinct gate source area gets one instance
//! (`gate_<source name>`) with a 1x1x1 driver area wired from that source,
//! exactly as v3 -> v4 does for reward sources. The old gate was keyed per
//! (source, destination) pair, so the instance is listed on every rule of the pair.

use std::collections::{BTreeMap, BTreeSet};

use feagi_structures::genomic::cortical_area::CorticalID;
use serde_json::{json, Map, Value};

use crate::genome::migration::membership::fold_genome_region_membership;
use crate::genome::migration::v3_to_v4::{
    add_drivers_to_root, driver_area, next_driver_serial, sanitize_id, wire_source_to_driver,
};
use crate::genome::migration::{MigrationError, MigrationStepDiagnostics, Migrator};
use crate::genome::schema::GenomeSchemaVersion;

/// Advances a hierarchical genome from schema v6 to schema v7.
#[derive(Debug, Default, Clone, Copy)]
pub struct V6ToV7Migrator;

impl V6ToV7Migrator {
    pub const fn new() -> Self {
        Self
    }
}

impl Migrator for V6ToV7Migrator {
    fn from_version(&self) -> GenomeSchemaVersion {
        GenomeSchemaVersion(6)
    }

    fn to_version(&self) -> GenomeSchemaVersion {
        GenomeSchemaVersion(7)
    }

    fn name(&self) -> &'static str {
        "v6_to_v7"
    }

    fn migrate(&self, genome: &mut Value) -> Result<MigrationStepDiagnostics, MigrationError> {
        let mut diag = MigrationStepDiagnostics::new(self.from_version(), self.to_version());
        // The v6 normalizer only runs for a genome that is at v6 when the chain stops, so
        // a v6 document with legacy `areas` is folded here before it is stamped v7.
        for note in fold_genome_region_membership(genome).data_loss {
            diag.record(note);
        }
        let converted =
            migrate_gate_sources(genome).map_err(|reason| MigrationError::StepFailed {
                name: self.name(),
                from: self.from_version(),
                to: self.to_version(),
                reason,
            })?;
        if converted == 0 {
            diag.record("no gate_source_area to convert");
        } else {
            diag.record(format!(
                "converted gate_source_area onto {converted} synaptic.gate instances"
            ));
        }
        Ok(diag)
    }
}

/// Returns the number of gate instances created.
fn migrate_gate_sources(genome: &mut Value) -> Result<usize, String> {
    let root = genome
        .as_object_mut()
        .ok_or_else(|| "genome must be an object".to_string())?;
    let blueprint = root
        .get("blueprint")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    // (source area, destination area) -> gate source area. The old runtime kept one
    // gate per pair, so the last rule that named one wins.
    let mut pair_gate: BTreeMap<(String, String), String> = BTreeMap::new();
    for (src_id, area) in &blueprint {
        let Some(dst) = area.get("cortical_mapping_dst").and_then(|v| v.as_object()) else {
            continue;
        };
        for (dst_id, rules) in dst {
            let Some(rules) = rules.as_array() else {
                continue;
            };
            for rule in rules {
                if let Some(gate) = rule.get("gate_source_area").and_then(|v| v.as_str()) {
                    pair_gate.insert((src_id.clone(), dst_id.clone()), gate.to_string());
                }
            }
        }
    }
    let has_field = blueprint.values().any(|area| {
        area.get("cortical_mapping_dst")
            .and_then(|v| v.as_object())
            .is_some_and(|dst| {
                dst.values().any(|rules| {
                    rules.as_array().is_some_and(|rules| {
                        rules.iter().any(|r| r.get("gate_source_area").is_some())
                    })
                })
            })
    });
    if !has_field {
        return Ok(0);
    }

    let sources: BTreeSet<String> = pair_gate.values().cloned().collect();
    let mut next_serial = next_driver_serial(&blueprint);
    let mut taken: BTreeSet<String> = root
        .get("modulators")
        .and_then(|v| v.as_object())
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    let mut source_to_instance: BTreeMap<String, (String, String)> = BTreeMap::new();
    for source_id in &sources {
        let name = blueprint
            .get(source_id)
            .and_then(|a| a.get("cortical_name"))
            .and_then(|v| v.as_str())
            .filter(|n| !n.is_empty())
            .unwrap_or(source_id);
        let base = format!("gate_{}", sanitize_id(name));
        let mut instance_id = base.clone();
        let mut suffix = 2u32;
        while taken.contains(&instance_id) {
            instance_id = format!("{base}_{suffix}");
            suffix += 1;
        }
        taken.insert(instance_id.clone());
        let driver_id = CorticalID::modulator_driver(next_serial).as_base_64();
        next_serial = next_serial.saturating_add(1);
        source_to_instance.insert(source_id.clone(), (instance_id, driver_id));
    }

    {
        let modulators = root
            .entry("modulators".to_string())
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .ok_or_else(|| "modulators must be an object".to_string())?;
        for (instance_id, driver_id) in source_to_instance.values() {
            modulators.insert(
                instance_id.clone(),
                json!({
                    "type": "synaptic.gate",
                    "magnitude_percent": 100.0,
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
        let mut y = 2 * blueprint
            .keys()
            .filter(|k| {
                CorticalID::try_from_base_64(k).is_ok_and(|id| id.is_modulator_driver())
            })
            .count() as i32;
        for (source_id, (instance_id, driver_id)) in &source_to_instance {
            blueprint.insert(driver_id.clone(), driver_area(instance_id, y));
            y += 2;
            wire_source_to_driver(blueprint, source_id, driver_id)?;
        }
        rewrite_rules(blueprint, &pair_gate, &source_to_instance);
    }

    let driver_ids: Vec<String> = source_to_instance
        .values()
        .map(|(_, driver)| driver.clone())
        .collect();
    add_drivers_to_root(root, &driver_ids);
    Ok(source_to_instance.len())
}

fn rewrite_rules(
    blueprint: &mut Map<String, Value>,
    pair_gate: &BTreeMap<(String, String), String>,
    source_to_instance: &BTreeMap<String, (String, String)>,
) {
    for (src_id, area) in blueprint.iter_mut() {
        let Some(dst) = area
            .as_object_mut()
            .and_then(|a| a.get_mut("cortical_mapping_dst"))
            .and_then(|v| v.as_object_mut())
        else {
            continue;
        };
        for (dst_id, rules) in dst.iter_mut() {
            let instance_id = pair_gate
                .get(&(src_id.clone(), dst_id.clone()))
                .and_then(|gate| source_to_instance.get(gate))
                .map(|(instance_id, _)| instance_id);
            let Some(rules) = rules.as_array_mut() else {
                continue;
            };
            for rule in rules {
                let Some(rule) = rule.as_object_mut() else {
                    continue;
                };
                rule.remove("gate_source_area");
                let Some(instance_id) = instance_id else {
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
                if !ids.contains(instance_id) {
                    ids.push(instance_id.clone());
                }
                rule.insert("modulators".to_string(), json!(ids));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gated_genome() -> Value {
        json!({
            "genome_schema_version": 6,
            "blueprint": {
                "c3JjMDAwMQ==": {
                    "cortical_name": "src",
                    "cortical_mapping_dst": {
                        "Y2RzdDAwMDE=": [
                            {
                                "morphology_id": "projector",
                                "gate_source_area": "Z2F0ZTAwMDE="
                            },
                            { "morphology_id": "block_to_block", "modulators": ["eta"] }
                        ],
                        "Y2RzdDAwMDI=": [{
                            "morphology_id": "projector",
                            "gate_source_area": "Z2F0ZTAwMDE="
                        }]
                    }
                },
                "Z2F0ZTAwMDE=": { "cortical_name": "open door" },
                "Y2RzdDAwMDE=": { "cortical_name": "dst one" },
                "Y2RzdDAwMDI=": { "cortical_name": "dst two" }
            },
            "modulators": {},
            "brain_regions": {
                "root": { "parent_region_id": null, "cortical_areas": ["c3JjMDAwMQ=="] }
            },
            "brain_regions_root": "root"
        })
    }

    #[test]
    fn converts_gated_pair_and_reuses_the_instance_per_source() {
        let mut genome = gated_genome();
        let migrator = V6ToV7Migrator::new();
        let diag = migrator.migrate(&mut genome).unwrap();
        assert_eq!(diag.transformations.len(), 1);
        assert!(diag.transformations[0].contains("1 synaptic.gate"));

        let src = &genome["blueprint"]["c3JjMDAwMQ=="]["cortical_mapping_dst"];
        for rule in src["Y2RzdDAwMDE="].as_array().unwrap() {
            assert!(rule.get("gate_source_area").is_none());
            assert!(rule["modulators"]
                .as_array()
                .unwrap()
                .contains(&json!("gate_open_door")));
        }
        // Existing modulator ids on the rule are kept.
        assert_eq!(
            src["Y2RzdDAwMDE="][1]["modulators"],
            json!(["eta", "gate_open_door"])
        );
        assert_eq!(src["Y2RzdDAwMDI="][0]["modulators"], json!(["gate_open_door"]));

        let modulators = genome["modulators"].as_object().unwrap();
        assert_eq!(modulators.len(), 1);
        let gate = &modulators["gate_open_door"];
        assert_eq!(gate["type"], "synaptic.gate");
        assert_eq!(gate["magnitude_percent"], 100.0);
        assert_eq!(gate["effect_duration_bursts"], 1);
        assert_eq!(gate["rest_bursts"], 0);
        assert_eq!(gate["graded"], false);

        let driver = gate["driver_cortical_id"].as_str().unwrap();
        assert_eq!(
            genome["blueprint"][driver]["modulator_instance_id"],
            "gate_open_door"
        );
        let projector = &genome["blueprint"]["Z2F0ZTAwMDE="]["cortical_mapping_dst"][driver][0];
        assert_eq!(projector["morphology_id"], "projector");
        assert!(genome["brain_regions"]["root"]["cortical_areas"]
            .as_array()
            .unwrap()
            .contains(&json!(driver)));

        let again = migrator.migrate(&mut genome).unwrap();
        assert!(again.transformations[0].contains("no gate_source_area"));
        assert_eq!(genome["modulators"].as_object().unwrap().len(), 1);
    }

    #[test]
    fn genome_without_gates_is_unchanged() {
        let mut genome = json!({
            "genome_schema_version": 6,
            "blueprint": { "c3JjMDAwMQ==": { "cortical_name": "src" } }
        });
        let before = genome.clone();
        V6ToV7Migrator::new().migrate(&mut genome).unwrap();
        assert_eq!(genome, before);
    }
}
