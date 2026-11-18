// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Plan the modulators and classifiers of a guest genome for amalgamation.

Both plans are pure: they read the guest genome and a snapshot of the host and return what to
create, which names and ids change, and the notices to show. They validate before the caller
creates anything, so a bad guest fails with nothing to roll back.

- Modulators: a guest instance either owns a driver area (`g` + serial id) or adopts an existing
  area. Owned instances are created through the service (which allocates the driver id); adopted
  instances are registered after their area exists. A name that already exists in the host is
  reused (same-brain clone) or renamed (external genome), see [`ModulatorCollisionPolicy`].
- Classifiers: always get a fresh `classifier_id`; every area field, reward modulator name and the
  parent region are remapped.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::amalgamation_remap::remap_classifier_area_fields;
use crate::modulators::ModulatorWrite;
use crate::runtime::RuntimeGenome;
use crate::types::{EvoError, EvoResult};
use feagi_structures::genomic::classifiers::Classifier;
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::genomic::ModulatorKind;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// What to do when a guest modulator name already exists in the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModulatorCollisionPolicy {
    /// Same-brain clone: use the host modulator of that name. Host settings win.
    ReuseHost,
    /// External genome: rename the guest modulator to `<name> (<block_title>)`, then append
    /// ` 2`, ` 3`, ... until the name is free.
    RenameGuest { block_title: String },
}

/// A modulator already in the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostModulator {
    pub kind: ModulatorKind,
    /// Base64 id of the driver area.
    pub driver_cortical_id: String,
}

/// One guest-owned modulator to create (the service allocates its driver area).
#[derive(Debug, Clone, PartialEq)]
pub struct PlannedOwnedModulator {
    pub guest_name: String,
    pub host_name: String,
    /// Base64 id of the guest driver area; the key of the driver remap.
    pub guest_driver_id: String,
    /// Create request (never carries a driver id).
    pub write: ModulatorWrite,
    /// Guest driver tuning to copy onto the new driver after creation (the service creates a
    /// driver with default values).
    pub driver_tuning: HashMap<String, Value>,
}

/// One guest modulator that adopts an existing area as its driver.
#[derive(Debug, Clone, PartialEq)]
pub struct PlannedAdoptedModulator {
    pub guest_name: String,
    pub host_name: String,
    /// Base64 id of the adopted area (final id after the area remap).
    pub driver_cortical_id: String,
    /// Create request carrying the driver id.
    pub write: ModulatorWrite,
}

/// Modulator work for one amalgamation.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AmalgamationModulatorPlan {
    /// Owned modulators to create, before the areas.
    pub owned: Vec<PlannedOwnedModulator>,
    /// Adopting modulators to create, after the areas.
    pub adopted: Vec<PlannedAdoptedModulator>,
    /// Every guest modulator name -> its name in the host (renamed, reused or unchanged).
    pub name_map: HashMap<String, String>,
    /// Guest driver area (base64) -> host driver area, for owned modulators that reuse a host
    /// modulator. Created owned modulators add their own entry once the service returns the id.
    pub reused_driver_remap: HashMap<String, String>,
    /// Guest modulator driver areas (base64) with no guest modulator instance. They cannot be
    /// imported.
    pub orphan_driver_ids: Vec<String>,
    /// One line per rename or reuse, for the response and the log.
    pub notices: Vec<String>,
}

fn write_from(
    instance: &crate::modulators::ModulatorInstance,
    driver: Option<String>,
) -> ModulatorWrite {
    ModulatorWrite {
        kind: instance.kind.as_str().to_string(),
        magnitude_percent: instance.magnitude_percent,
        effect_duration_bursts: instance.effect_duration_bursts,
        rest_bursts: instance.rest_bursts,
        graded: instance.graded,
        full_scale_potential: instance.full_scale_potential,
        driver_cortical_id: driver,
    }
}

/// Defaults the service gives a new driver; guest values are copied only when they differ.
const DRIVER_TUNING_DEFAULTS: [(&str, f64); 3] = [
    ("firing_threshold", 1.0),
    ("leak_coefficient", 0.0),
    ("excitability", 1.0),
];

fn driver_tuning(genome: &RuntimeGenome, driver: &CorticalID) -> HashMap<String, Value> {
    let mut tuning = HashMap::new();
    let Some(area) = genome.cortical_areas.get(driver) else {
        return tuning;
    };
    for (key, default) in DRIVER_TUNING_DEFAULTS {
        if let Some(value) = area.properties.get(key).and_then(|v| v.as_f64()) {
            if (value - default).abs() > f64::EPSILON {
                tuning.insert(key.to_string(), serde_json::json!(value));
            }
        }
    }
    tuning
}

fn free_name(base: &str, title: &str, taken: &HashSet<String>) -> String {
    let title = title.trim();
    let title = if title.is_empty() { "imported" } else { title };
    let first = format!("{base} ({title})");
    if !taken.contains(&first) {
        return first;
    }
    let mut n = 2u32;
    loop {
        let candidate = format!("{first} {n}");
        if !taken.contains(&candidate) {
            return candidate;
        }
        n += 1;
    }
}

/// Plan the guest modulators. Call after the guest area ids were remapped
/// (`remap_guest_custom_memory_cortical_ids_for_amalgamation`), so adopted drivers carry their
/// final ids.
///
/// `host_area_ids` are the base64 ids of the host's areas. Fails, with nothing created, when an
/// instance is invalid, a reused host modulator has another type, an adopted driver is missing,
/// or an adopted driver already drives a host modulator.
pub fn plan_amalgamation_modulators(
    guest: &RuntimeGenome,
    host_modulators: &HashMap<String, HostModulator>,
    host_area_ids: &HashSet<String>,
    policy: &ModulatorCollisionPolicy,
) -> EvoResult<AmalgamationModulatorPlan> {
    let mut plan = AmalgamationModulatorPlan::default();
    let guest_ids = guest.modulators.ids();

    // Names that stay as they are: host names and guest names that do not collide.
    let mut taken: HashSet<String> = host_modulators.keys().cloned().collect();
    for name in &guest_ids {
        if !host_modulators.contains_key(name) {
            taken.insert(name.clone());
        }
    }

    let host_adopted_drivers: HashMap<&str, &str> = host_modulators
        .iter()
        .map(|(name, m)| (m.driver_cortical_id.as_str(), name.as_str()))
        .collect();
    let mut guest_adopted_drivers: HashSet<String> = HashSet::new();
    let mut owned_driver_ids: HashSet<String> = HashSet::new();

    for guest_name in &guest_ids {
        let instance = guest
            .modulators
            .get(guest_name)
            .ok_or_else(|| EvoError::Internal(format!("modulator '{guest_name}' vanished")))?;
        instance
            .validate()
            .map_err(|err| EvoError::InvalidGenome(format!("modulator '{guest_name}': {err}")))?;
        let host_match = host_modulators.get(guest_name);
        let reuse = host_match.is_some() && matches!(policy, ModulatorCollisionPolicy::ReuseHost);
        let host_name = match (host_match, policy) {
            (None, _) => guest_name.clone(),
            (Some(_), ModulatorCollisionPolicy::ReuseHost) => guest_name.clone(),
            (Some(_), ModulatorCollisionPolicy::RenameGuest { block_title }) => {
                let renamed = free_name(guest_name, block_title, &taken);
                taken.insert(renamed.clone());
                plan.notices.push(format!(
                    "Renamed modulator '{guest_name}' to '{renamed}' because it already exists in this brain."
                ));
                renamed
            }
        };
        plan.name_map.insert(guest_name.clone(), host_name.clone());

        if reuse {
            let host = &host_modulators[guest_name];
            if host.kind != instance.kind {
                return Err(EvoError::InvalidGenome(format!(
                    "modulator '{guest_name}' is type {} here but {} in the imported circuit; \
                     it cannot be reused",
                    host.kind, instance.kind
                )));
            }
            plan.notices.push(format!(
                "Reused existing modulator '{guest_name}'; host settings apply."
            ));
            if !instance.adopts_driver() {
                plan.reused_driver_remap.insert(
                    instance.driver_cortical_id.as_base_64(),
                    host.driver_cortical_id.clone(),
                );
                owned_driver_ids.insert(instance.driver_cortical_id.as_base_64());
            }
            continue;
        }

        let driver_b64 = instance.driver_cortical_id.as_base_64();
        if instance.adopts_driver() {
            let write = write_from(instance, Some(driver_b64.clone()));
            write.validate_adopted_timing().map_err(|err| {
                EvoError::InvalidGenome(format!("modulator '{guest_name}': {err}"))
            })?;
            if !host_area_ids.contains(&driver_b64)
                && !guest
                    .cortical_areas
                    .contains_key(&instance.driver_cortical_id)
            {
                return Err(EvoError::InvalidGenome(format!(
                    "modulator '{guest_name}' adopts area {driver_b64}, which is in neither the imported circuit nor this brain"
                )));
            }
            if let Some(other) = host_adopted_drivers.get(driver_b64.as_str()) {
                return Err(EvoError::InvalidGenome(format!(
                    "area {driver_b64} already drives modulator '{other}', so modulator \
                     '{guest_name}' cannot adopt it"
                )));
            }
            if !guest_adopted_drivers.insert(driver_b64.clone()) {
                return Err(EvoError::InvalidGenome(format!(
                    "two imported modulators adopt area {driver_b64}"
                )));
            }
            plan.adopted.push(PlannedAdoptedModulator {
                guest_name: guest_name.clone(),
                host_name,
                driver_cortical_id: driver_b64,
                write,
            });
        } else {
            owned_driver_ids.insert(driver_b64.clone());
            plan.owned.push(PlannedOwnedModulator {
                guest_name: guest_name.clone(),
                host_name,
                guest_driver_id: driver_b64,
                write: write_from(instance, None),
                driver_tuning: driver_tuning(guest, &instance.driver_cortical_id),
            });
        }
    }

    let mut orphans: Vec<String> = guest
        .cortical_areas
        .keys()
        .filter(|id| id.is_modulator_driver())
        .map(|id| id.as_base_64())
        .filter(|id| !owned_driver_ids.contains(id))
        .collect();
    orphans.sort_unstable();
    plan.orphan_driver_ids = orphans;
    Ok(plan)
}

fn rename_list(value: Option<&mut Value>, name_map: &HashMap<String, String>) {
    let Some(list) = value.and_then(|value| value.as_array_mut()) else {
        return;
    };
    for entry in list {
        if let Some(new_name) = entry.as_str().and_then(|name| name_map.get(name)) {
            *entry = Value::String(new_name.clone());
        }
    }
}

/// Rewrite every modulator name a guest area refers to: the area's `modulators` list and each
/// mapping rule's `modulators` list. Classifier reward lists are handled by
/// [`plan_amalgamation_classifiers`].
pub fn apply_modulator_name_map(genome: &mut RuntimeGenome, name_map: &HashMap<String, String>) {
    if name_map.iter().all(|(guest, host)| guest == host) {
        return;
    }
    for area in genome.cortical_areas.values_mut() {
        rename_list(area.properties.get_mut("modulators"), name_map);
        let Some(dst) = area
            .properties
            .get_mut("cortical_mapping_dst")
            .and_then(|value| value.as_object_mut())
        else {
            continue;
        };
        for rules in dst.values_mut() {
            let Some(rules) = rules.as_array_mut() else {
                continue;
            };
            for rule in rules {
                if let Some(rule) = rule.as_object_mut() {
                    rename_list(rule.get_mut("modulators"), name_map);
                }
            }
        }
    }
}

/// Plan the guest classifiers: fresh `classifier_id`, area fields through `area_remap`, reward
/// modulators through `name_map`, and `parent_region_id` through `region_map` (the host region
/// when the guest parent was not planned). Fails when a classifier's sample or class memory is
/// not among `guest` areas, so a half-imported assembly is never created.
///
/// Call after the area remap is final (custom/memory ids and created driver ids).
pub fn plan_amalgamation_classifiers(
    guest: &RuntimeGenome,
    area_remap: &HashMap<String, String>,
    name_map: &HashMap<String, String>,
    region_map: &HashMap<String, String>,
    host_region_id: &str,
) -> EvoResult<Vec<Classifier>> {
    let mut guest_ids: Vec<&String> = guest.classifiers.keys().collect();
    guest_ids.sort_unstable();
    let mut planned = Vec::with_capacity(guest_ids.len());
    for guest_id in guest_ids {
        let mut classifier = guest.classifiers[guest_id].clone();
        remap_classifier_area_fields(&mut classifier, area_remap);
        for id in [&classifier.sample_memory_id, &classifier.class_memory_id] {
            let present = CorticalID::try_from_base_64(id)
                .map(|cid| guest.cortical_areas.contains_key(&cid))
                .unwrap_or(false);
            if !present {
                return Err(EvoError::InvalidGenome(format!(
                    "classifier '{}' refers to memory area {} that is not in the imported circuit",
                    classifier.name, id
                )));
            }
        }
        for id in classifier.reward_modulator_ids.iter_mut() {
            if let Some(new_name) = name_map.get(id.as_str()) {
                *id = new_name.clone();
            }
        }
        classifier.parent_region_id = region_map
            .get(&classifier.parent_region_id)
            .cloned()
            .unwrap_or_else(|| host_region_id.to_string());
        classifier.classifier_id =
            feagi_structures::genomic::brain_regions::RegionID::new().to_string();
        planned.push(classifier);
    }
    Ok(planned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amalgamation_remap::apply_driver_remap;
    use crate::modulators::{ModulatorInstance, ModulatorRegistry};
    use crate::runtime::{GenomeMetadata, GenomeSignatures, GenomeStats, PhysiologyConfig};
    use crate::MorphologyRegistry;
    use feagi_structures::genomic::classifiers::ClassifierClassBinding;
    use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaDimensions;
    use feagi_structures::genomic::cortical_area::{
        CorticalArea, CorticalAreaType, CustomCorticalType,
    };
    use feagi_structures::genomic::descriptors::GenomeCoordinate3D;

    fn custom_id(tag: u8) -> CorticalID {
        let mut bytes = *b"c_______";
        bytes[1] = tag;
        CorticalID::try_from_bytes(&bytes).expect("custom id")
    }

    fn area(id: CorticalID, kind: CorticalAreaType) -> CorticalArea {
        CorticalArea::new(
            id,
            0,
            "a".to_string(),
            CorticalAreaDimensions::new(1, 1, 1).expect("dims"),
            GenomeCoordinate3D::new(0, 0, 0),
            kind,
        )
        .expect("area")
    }

    fn custom_area(id: CorticalID) -> CorticalArea {
        area(
            id,
            CorticalAreaType::Custom(CustomCorticalType::LeakyIntegrateFire),
        )
    }

    fn instance(kind: ModulatorKind, driver: CorticalID) -> ModulatorInstance {
        ModulatorInstance {
            kind,
            magnitude_percent: 100.0,
            effect_duration_bursts: 1,
            rest_bursts: 0,
            graded: false,
            full_scale_potential: None,
            driver_cortical_id: driver,
        }
    }

    fn empty_genome(title: &str) -> RuntimeGenome {
        RuntimeGenome {
            metadata: GenomeMetadata {
                genome_id: "g".to_string(),
                genome_title: title.to_string(),
                genome_description: String::new(),
                version: "3.0".to_string(),
                timestamp: 0.0,
                brain_regions_root: None,
            },
            cortical_areas: HashMap::new(),
            brain_regions: HashMap::new(),
            classifiers: HashMap::new(),
            morphologies: MorphologyRegistry::new(),
            modulators: ModulatorRegistry::new(),
            physiology: PhysiologyConfig::default(),
            signatures: GenomeSignatures {
                genome: String::new(),
                blueprint: String::new(),
                physiology: String::new(),
                morphologies: None,
            },
            stats: GenomeStats::default(),
            change_history: Vec::new(),
        }
    }

    /// Guest with one owned reward modulator `boost` (driver serial 0), one custom area that
    /// maps into the driver and subscribes a rule to `boost`.
    fn guest_with_owned(name: &str) -> (RuntimeGenome, CorticalID, CorticalID) {
        let mut genome = empty_genome("Block A");
        let driver = CorticalID::modulator_driver(0);
        let src = custom_id(b'a');
        genome
            .cortical_areas
            .insert(driver, area(driver, CorticalAreaType::Modulator));
        let mut src_area = custom_area(src);
        src_area.properties.insert(
            "cortical_mapping_dst".to_string(),
            serde_json::json!({
                driver.as_base_64(): [{"morphology_id": "m", "modulators": [name]}],
                custom_id(b'b').as_base_64(): [{"morphology_id": "m", "modulators": [name, "other"]}],
            }),
        );
        src_area
            .properties
            .insert("modulators".to_string(), serde_json::json!([name]));
        genome.cortical_areas.insert(src, src_area);
        genome
            .modulators
            .insert(name.to_string(), instance(ModulatorKind::Reward, driver));
        (genome, driver, src)
    }

    fn host(entries: &[(&str, ModulatorKind, CorticalID)]) -> HashMap<String, HostModulator> {
        entries
            .iter()
            .map(|(name, kind, driver)| {
                (
                    (*name).to_string(),
                    HostModulator {
                        kind: *kind,
                        driver_cortical_id: driver.as_base_64(),
                    },
                )
            })
            .collect()
    }

    fn rename_policy() -> ModulatorCollisionPolicy {
        ModulatorCollisionPolicy::RenameGuest {
            block_title: "Block A".to_string(),
        }
    }

    #[test]
    fn no_collision_keeps_names_and_plans_owned_creation() {
        let (genome, driver, _) = guest_with_owned("boost");
        let plan = plan_amalgamation_modulators(
            &genome,
            &HashMap::new(),
            &HashSet::new(),
            &rename_policy(),
        )
        .expect("plan");
        assert_eq!(plan.owned.len(), 1);
        assert_eq!(plan.owned[0].host_name, "boost");
        assert_eq!(plan.owned[0].guest_driver_id, driver.as_base_64());
        assert!(plan.owned[0].write.driver_cortical_id.is_none());
        assert_eq!(plan.name_map["boost"], "boost");
        assert!(plan.notices.is_empty());
        assert!(plan.orphan_driver_ids.is_empty());
    }

    #[test]
    fn collision_renames_with_block_title_and_rewrites_every_reference() {
        let (mut genome, _, src) = guest_with_owned("boost");
        let host_driver = CorticalID::modulator_driver(0);
        let plan = plan_amalgamation_modulators(
            &genome,
            &host(&[("boost", ModulatorKind::Reward, host_driver)]),
            &HashSet::new(),
            &rename_policy(),
        )
        .expect("plan");
        assert_eq!(plan.name_map["boost"], "boost (Block A)");
        assert_eq!(
            plan.notices,
            vec![
                "Renamed modulator 'boost' to 'boost (Block A)' because it already exists in this brain."
                    .to_string()
            ]
        );
        apply_modulator_name_map(&mut genome, &plan.name_map);
        let props = &genome.cortical_areas[&src].properties;
        assert_eq!(props["modulators"], serde_json::json!(["boost (Block A)"]));
        let dst = props["cortical_mapping_dst"].as_object().unwrap();
        let lists: Vec<Value> = dst
            .values()
            .map(|rules| rules[0]["modulators"].clone())
            .collect();
        assert!(lists.contains(&serde_json::json!(["boost (Block A)"])));
        assert!(lists.contains(&serde_json::json!(["boost (Block A)", "other"])));
    }

    #[test]
    fn rename_of_rename_numbers_from_two() {
        let (genome, _, _) = guest_with_owned("boost");
        let d0 = CorticalID::modulator_driver(0);
        let d1 = CorticalID::modulator_driver(1);
        let d2 = CorticalID::modulator_driver(2);
        let plan = plan_amalgamation_modulators(
            &genome,
            &host(&[
                ("boost", ModulatorKind::Reward, d0),
                ("boost (Block A)", ModulatorKind::Reward, d1),
            ]),
            &HashSet::new(),
            &rename_policy(),
        )
        .expect("plan");
        assert_eq!(plan.name_map["boost"], "boost (Block A) 2");

        let plan = plan_amalgamation_modulators(
            &genome,
            &host(&[
                ("boost", ModulatorKind::Reward, d0),
                ("boost (Block A)", ModulatorKind::Reward, d1),
                ("boost (Block A) 2", ModulatorKind::Reward, d2),
            ]),
            &HashSet::new(),
            &rename_policy(),
        )
        .expect("plan");
        assert_eq!(plan.name_map["boost"], "boost (Block A) 3");
    }

    #[test]
    fn clone_reuses_host_modulator_and_redirects_driver_mappings() {
        let (mut genome, driver, src) = guest_with_owned("boost");
        // The host driver has the same serial as the guest driver: the ids collide.
        let host_driver = CorticalID::modulator_driver(0);
        let plan = plan_amalgamation_modulators(
            &genome,
            &host(&[("boost", ModulatorKind::Reward, host_driver)]),
            &HashSet::new(),
            &ModulatorCollisionPolicy::ReuseHost,
        )
        .expect("plan");
        assert!(plan.owned.is_empty(), "nothing is created under reuse");
        assert_eq!(plan.name_map["boost"], "boost");
        assert_eq!(
            plan.notices,
            vec!["Reused existing modulator 'boost'; host settings apply.".to_string()]
        );
        assert_eq!(
            plan.reused_driver_remap[&driver.as_base_64()],
            host_driver.as_base_64()
        );
        assert!(plan.orphan_driver_ids.is_empty());
        apply_driver_remap(&mut genome, &plan.reused_driver_remap).expect("remap");
        assert!(genome.cortical_areas.contains_key(&src));
    }

    #[test]
    fn reuse_with_a_different_type_is_rejected_up_front() {
        let (genome, _, _) = guest_with_owned("boost");
        let err = plan_amalgamation_modulators(
            &genome,
            &host(&[(
                "boost",
                ModulatorKind::Leak,
                CorticalID::modulator_driver(0),
            )]),
            &HashSet::new(),
            &ModulatorCollisionPolicy::ReuseHost,
        )
        .expect_err("type mismatch");
        assert!(err.to_string().contains("cannot be reused"), "{err}");
    }

    #[test]
    fn created_owned_driver_remap_rewrites_area_keys_and_destination_keys() {
        let (mut genome, driver, src) = guest_with_owned("boost");
        let new_driver = CorticalID::modulator_driver(7);
        let remap = HashMap::from([(driver.as_base_64(), new_driver.as_base_64())]);
        apply_driver_remap(&mut genome, &remap).expect("remap");
        assert!(genome.cortical_areas.contains_key(&new_driver));
        assert!(!genome.cortical_areas.contains_key(&driver));
        let dst = genome.cortical_areas[&src].properties["cortical_mapping_dst"]
            .as_object()
            .unwrap();
        assert!(dst.contains_key(&new_driver.as_base_64()));
        assert!(!dst.contains_key(&driver.as_base_64()));
    }

    #[test]
    fn adopted_driver_follows_the_area_remap_and_is_checked_up_front() {
        let mut genome = empty_genome("Block A");
        let adopted_old = custom_id(b'x');
        let adopted_new = custom_id(b'y');
        genome
            .cortical_areas
            .insert(adopted_new, custom_area(adopted_new));
        genome.modulators.insert(
            "calm".to_string(),
            instance(ModulatorKind::Leak, adopted_old),
        );
        // The area remap moves the adopted driver from `x` to `y`.
        let remap = HashMap::from([(adopted_old.as_base_64(), adopted_new.as_base_64())]);
        crate::amalgamation_remap::rewrite_area_references(&mut genome, &remap);
        assert_eq!(
            genome.modulators.get("calm").unwrap().driver_cortical_id,
            adopted_new
        );
        let plan = plan_amalgamation_modulators(
            &genome,
            &HashMap::new(),
            &HashSet::new(),
            &rename_policy(),
        )
        .expect("plan");
        assert_eq!(plan.adopted.len(), 1);
        assert_eq!(plan.adopted[0].driver_cortical_id, adopted_new.as_base_64());
        assert_eq!(
            plan.adopted[0].write.driver_cortical_id.as_deref(),
            Some(adopted_new.as_base_64().as_str())
        );

        // A host modulator that already uses the area blocks the import before anything exists.
        let err = plan_amalgamation_modulators(
            &genome,
            &host(&[("other", ModulatorKind::Leak, adopted_new)]),
            &HashSet::from([adopted_new.as_base_64()]),
            &rename_policy(),
        )
        .expect_err("already drives");
        assert!(err.to_string().contains("already drives"), "{err}");

        // A missing adopted area is rejected.
        genome.cortical_areas.clear();
        let err = plan_amalgamation_modulators(
            &genome,
            &HashMap::new(),
            &HashSet::new(),
            &rename_policy(),
        )
        .expect_err("missing driver");
        assert!(err.to_string().contains("neither"), "{err}");
    }

    #[test]
    fn guest_mapping_naming_a_modulator_has_a_defined_host_name() {
        // Zero dropped guest mappings: every modulator id on a rule resolves in the name map.
        let (genome, _, src) = guest_with_owned("boost");
        let plan = plan_amalgamation_modulators(
            &genome,
            &host(&[(
                "boost",
                ModulatorKind::Reward,
                CorticalID::modulator_driver(3),
            )]),
            &HashSet::new(),
            &rename_policy(),
        )
        .expect("plan");
        let created: HashSet<&str> = plan
            .owned
            .iter()
            .map(|m| m.host_name.as_str())
            .chain(plan.adopted.iter().map(|m| m.host_name.as_str()))
            .collect();
        let dst = genome.cortical_areas[&src].properties["cortical_mapping_dst"]
            .as_object()
            .unwrap();
        for rules in dst.values() {
            for id in rules[0]["modulators"].as_array().unwrap() {
                let id = id.as_str().unwrap();
                if id == "other" {
                    continue; // not a guest modulator: validated by the host as before
                }
                assert!(
                    created.contains(plan.name_map[id].as_str()),
                    "'{id}' must exist in the host after the import"
                );
            }
        }
    }

    #[test]
    fn orphan_driver_areas_are_reported() {
        let mut genome = empty_genome("Block A");
        let driver = CorticalID::modulator_driver(4);
        genome
            .cortical_areas
            .insert(driver, area(driver, CorticalAreaType::Modulator));
        let plan = plan_amalgamation_modulators(
            &genome,
            &HashMap::new(),
            &HashSet::new(),
            &rename_policy(),
        )
        .expect("plan");
        assert_eq!(plan.orphan_driver_ids, vec![driver.as_base_64()]);
    }

    #[test]
    fn owned_driver_tuning_differing_from_defaults_is_carried() {
        let (mut genome, driver, _) = guest_with_owned("boost");
        genome
            .cortical_areas
            .get_mut(&driver)
            .unwrap()
            .properties
            .insert("firing_threshold".to_string(), serde_json::json!(2.5));
        genome
            .cortical_areas
            .get_mut(&driver)
            .unwrap()
            .properties
            .insert("leak_coefficient".to_string(), serde_json::json!(0.0));
        let plan = plan_amalgamation_modulators(
            &genome,
            &HashMap::new(),
            &HashSet::new(),
            &rename_policy(),
        )
        .expect("plan");
        assert_eq!(
            plan.owned[0].driver_tuning,
            HashMap::from([("firing_threshold".to_string(), serde_json::json!(2.5))])
        );
    }

    fn classifier(name: &str, parent: &str, ids: [CorticalID; 4]) -> Classifier {
        Classifier {
            classifier_id: "guest-clf".to_string(),
            name: name.to_string(),
            parent_region_id: parent.to_string(),
            coordinates_3d: [0, 0, 0],
            sample_area_ids: vec![ids[3].as_base_64()],
            class_bindings: vec![ClassifierClassBinding {
                class_area_id: ids[3].as_base_64(),
                class_output_area_id: ids[2].as_base_64(),
            }],
            sample_memory_id: ids[0].as_base_64(),
            class_memory_id: ids[1].as_base_64(),
            error_area_id: ids[2].as_base_64(),
            correct_area_id: String::new(),
            reward_training: true,
            answer_feedback_area_id: None,
            reward_modulator_ids: vec!["boost".to_string(), "keep".to_string()],
            answer_latency_bursts: 0,
            learn_area_id: Some(ids[3].as_base_64()),
            confidence_area_id: None,
            properties: HashMap::new(),
        }
    }

    #[test]
    fn classifier_gets_fresh_id_and_remapped_areas_modulators_and_region() {
        let mut genome = empty_genome("Block A");
        let old = [
            custom_id(b'1'),
            custom_id(b'2'),
            custom_id(b'3'),
            custom_id(b'4'),
        ];
        let new = [
            custom_id(b'5'),
            custom_id(b'6'),
            custom_id(b'7'),
            custom_id(b'8'),
        ];
        for id in new {
            genome.cortical_areas.insert(id, custom_area(id));
        }
        genome.classifiers.insert(
            "guest-clf".to_string(),
            classifier("clf", "guest-region", old),
        );
        let area_remap: HashMap<String, String> = old
            .iter()
            .zip(new.iter())
            .map(|(o, n)| (o.as_base_64(), n.as_base_64()))
            .collect();
        let name_map = HashMap::from([("boost".to_string(), "boost (Block A)".to_string())]);
        let region_map = HashMap::from([("guest-region".to_string(), "new-region".to_string())]);

        let planned =
            plan_amalgamation_classifiers(&genome, &area_remap, &name_map, &region_map, "host")
                .expect("plan");
        assert_eq!(planned.len(), 1);
        let c = &planned[0];
        assert_ne!(c.classifier_id, "guest-clf");
        assert_eq!(c.sample_memory_id, new[0].as_base_64());
        assert_eq!(c.class_memory_id, new[1].as_base_64());
        assert_eq!(c.error_area_id, new[2].as_base_64());
        assert_eq!(c.sample_area_ids, vec![new[3].as_base_64()]);
        assert_eq!(c.class_bindings[0].class_area_id, new[3].as_base_64());
        assert_eq!(
            c.class_bindings[0].class_output_area_id,
            new[2].as_base_64()
        );
        assert_eq!(c.learn_area_id, Some(new[3].as_base_64()));
        assert_eq!(
            c.reward_modulator_ids,
            vec!["boost (Block A)".to_string(), "keep".to_string()]
        );
        assert_eq!(c.parent_region_id, "new-region");

        // Unplanned guest parent falls back to the host region.
        let planned =
            plan_amalgamation_classifiers(&genome, &area_remap, &name_map, &HashMap::new(), "host")
                .expect("plan");
        assert_eq!(planned[0].parent_region_id, "host");

        // Two plans never share an id.
        let again =
            plan_amalgamation_classifiers(&genome, &area_remap, &name_map, &HashMap::new(), "host")
                .expect("plan");
        assert_ne!(planned[0].classifier_id, again[0].classifier_id);
    }

    #[test]
    fn classifier_with_a_missing_memory_area_is_rejected() {
        let mut genome = empty_genome("Block A");
        let old = [
            custom_id(b'1'),
            custom_id(b'2'),
            custom_id(b'3'),
            custom_id(b'4'),
        ];
        genome
            .classifiers
            .insert("guest-clf".to_string(), classifier("clf", "r", old));
        let err = plan_amalgamation_classifiers(
            &genome,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            "host",
        )
        .expect_err("missing memory");
        assert!(
            err.to_string().contains("not in the imported circuit"),
            "{err}"
        );
    }
}
