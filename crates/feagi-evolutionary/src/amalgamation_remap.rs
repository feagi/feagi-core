// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Remap **Custom** and **Memory** cortical IDs on an imported guest genome so they do not collide
with the host connectome.

Core (`___death`, `___power`, `___fatig`, `___pain_`, `___pleas`, `___fear_`, `___hope_`) and IPU/OPU identifiers are **not** remapped: they are
canonical and must continue to match the host’s shared IO and core regions.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::random::random_bytes;
use crate::runtime::RuntimeGenome;
use feagi_structures::genomic::cortical_area::CorticalArea;
use feagi_structures::genomic::cortical_area::CorticalAreaType;
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::genomic::BrainRegion;
use feagi_structures::FeagiDataError;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

const MAX_ID_ALLOC_ATTEMPTS: u32 = 100_000;

/// Allocate a new cortical ID with the given first-byte prefix (`c` or `m`), distinct from
/// `reserved` (inserts the chosen ID into `reserved`).
fn allocate_unique_typed_id(
    prefix: u8,
    reserved: &mut HashSet<String>,
) -> Result<CorticalID, FeagiDataError> {
    for _ in 0..MAX_ID_ALLOC_ATTEMPTS {
        let mut bytes = [0u8; CorticalID::CORTICAL_ID_LENGTH];
        bytes[0] = prefix;
        random_bytes(&mut bytes[1..]);
        if let Ok(id) = CorticalID::try_from_bytes(&bytes) {
            let s = id.as_base_64();
            if !reserved.contains(&s) {
                reserved.insert(s);
                return Ok(id);
            }
        }
    }
    Err(FeagiDataError::InternalError(
        "Failed to allocate a unique cortical ID for amalgamation remapping".into(),
    ))
}

fn remap_region_io_lists(region: &mut BrainRegion, b64_remap: &HashMap<String, String>) {
    for key in [
        "inputs",
        "outputs",
        "designated_inputs",
        "designated_outputs",
    ] {
        let Some(val) = region.properties.get_mut(key) else {
            continue;
        };
        let Some(arr) = val.as_array_mut() else {
            continue;
        };
        for item in arr.iter_mut() {
            if let Some(s) = item.as_str() {
                if let Some(n) = b64_remap.get(s) {
                    *item = Value::String(n.clone());
                }
            }
        }
    }
}

fn remap_cortical_mapping_dst_keys(
    properties: &mut HashMap<String, Value>,
    b64_remap: &HashMap<String, String>,
) {
    let Some(Value::Object(dstmap)) = properties.get_mut("cortical_mapping_dst") else {
        return;
    };
    let taken = std::mem::take(dstmap);
    let mut new_map = serde_json::Map::new();
    for (k, v) in taken {
        let nk = b64_remap.get(&k).cloned().unwrap_or(k);
        new_map.insert(nk, v);
    }
    properties.insert("cortical_mapping_dst".to_string(), Value::Object(new_map));
}

/// Remap guest **Custom** and **Memory** cortical IDs to fresh values not present in
/// `host_reserved_base64_ids` or elsewhere in this guest genome. Updates cortical area keys,
/// `cortical_mapping_dst` destination keys, brain region membership, and region IO lists.
///
/// Returns `(old_base64, new_base64)` pairs only for IDs that changed.
pub fn remap_guest_custom_memory_cortical_ids_for_amalgamation(
    genome: &mut RuntimeGenome,
    host_reserved_base64_ids: &HashSet<String>,
) -> Result<HashMap<String, String>, FeagiDataError> {
    let mut reserved: HashSet<String> = host_reserved_base64_ids.clone();
    for id in genome.cortical_areas.keys() {
        reserved.insert(id.as_base_64());
    }

    let mut id_remap: HashMap<CorticalID, CorticalID> = HashMap::new();

    for (old_id, area) in genome.cortical_areas.iter() {
        let needs_remapping = matches!(
            &area.cortical_type,
            CorticalAreaType::Custom(_) | CorticalAreaType::Memory(_)
        );
        if !needs_remapping {
            continue;
        }
        let prefix = old_id.as_bytes()[0];
        let new_id = allocate_unique_typed_id(prefix, &mut reserved)?;
        id_remap.insert(*old_id, new_id);
    }

    let mut b64_remap: HashMap<String, String> = HashMap::new();
    for (old_id, new_id) in &id_remap {
        let ob = old_id.as_base_64();
        let nb = new_id.as_base_64();
        if ob != nb {
            b64_remap.insert(ob, nb);
        }
    }

    if id_remap.is_empty() {
        return Ok(b64_remap);
    }

    // Re-key cortical_areas and refresh each area's `cortical_id`.
    let mut new_areas: HashMap<CorticalID, CorticalArea> =
        HashMap::with_capacity(genome.cortical_areas.len());
    for (old_id, mut area) in std::mem::take(&mut genome.cortical_areas) {
        let new_id = id_remap.get(&old_id).copied().unwrap_or(old_id);
        area.cortical_id = new_id;
        new_areas.insert(new_id, area);
    }
    genome.cortical_areas = new_areas;

    for region in genome.brain_regions.values_mut() {
        let mut new_set: HashSet<CorticalID> = HashSet::with_capacity(region.cortical_areas.len());
        for cid in region.cortical_areas.drain() {
            let nid = id_remap.get(&cid).copied().unwrap_or(cid);
            new_set.insert(nid);
        }
        region.cortical_areas = new_set;
        remap_region_io_lists(region, &b64_remap);
    }

    for area in genome.cortical_areas.values_mut() {
        remap_cortical_mapping_dst_keys(&mut area.properties, &b64_remap);
    }

    Ok(b64_remap)
}

fn json_string_list(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Declared region IO on the circuit that region-export wrapped under `Root Brain Region`.
///
/// Amalgamation hosts that circuit in a new region. These lists are already remapped to the
/// guest cortical IDs and must be copied onto that new region or the clone drops input/output roles.
pub fn designated_io_lists_for_cloned_circuit(
    genome: &RuntimeGenome,
) -> (Vec<String>, Vec<String>) {
    use feagi_structures::genomic::brain_regions::ROOT_BRAIN_REGION_NAME;

    let root_ids: HashSet<String> = genome
        .brain_regions
        .iter()
        .filter(|(_, region)| region.name == ROOT_BRAIN_REGION_NAME)
        .map(|(id, _)| id.clone())
        .collect();

    let circuit = genome.brain_regions.values().find(|region| {
        let parent = region
            .properties
            .get("parent_region_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if !root_ids.is_empty() {
            root_ids.contains(parent)
        } else {
            parent.is_empty() && region.name != ROOT_BRAIN_REGION_NAME
        }
    });

    let Some(region) = circuit else {
        return (Vec::new(), Vec::new());
    };
    (
        json_string_list(region.properties.get("designated_inputs")),
        json_string_list(region.properties.get("designated_outputs")),
    )
}

/// One sub-region to create under the amalgamation host region.
#[derive(Debug, Clone, PartialEq)]
pub struct AmalgamationRegionSpec {
    /// Fresh region id (UUID string) for the new region.
    pub region_id: String,
    /// Guest region name.
    pub name: String,
    /// Parent region id: another planned region, or the host amalgamation region.
    pub parent_region_id: String,
    /// Guest region properties to carry over (coordinate offset, designated IO remapped and
    /// filtered to imported areas). Never contains parent or membership keys.
    pub properties: HashMap<String, Value>,
}

/// Region tree to build for an amalgamated guest genome.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AmalgamationRegionPlan {
    /// Sub-regions in parent-before-child order.
    pub sub_regions: Vec<AmalgamationRegionSpec>,
    /// Imported area (base64 id) -> region id it joins. Areas absent here belong to the host
    /// amalgamation region.
    pub area_region: HashMap<String, String>,
    /// Designated inputs for the host region (set only when exactly one guest top-level region
    /// maps onto it).
    pub host_designated_inputs: Vec<String>,
    /// Designated outputs for the host region (same rule as inputs).
    pub host_designated_outputs: Vec<String>,
}

fn guest_parent_id(region: &BrainRegion) -> Option<&str> {
    region
        .properties
        .get("parent_region_id")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|p| !p.is_empty())
}

fn offset_coordinate_3d(value: Option<&Value>, origin: (i32, i32, i32)) -> Value {
    let get = |i: usize| -> i64 {
        value
            .and_then(|v| v.as_array())
            .and_then(|a| a.get(i))
            .and_then(|n| n.as_i64())
            .unwrap_or(0)
    };
    serde_json::json!([
        i64::from(origin.0).saturating_add(get(0)),
        i64::from(origin.1).saturating_add(get(1)),
        i64::from(origin.2).saturating_add(get(2)),
    ])
}

fn filtered_io(region: &BrainRegion, key: &str, member_area_ids: &HashSet<String>) -> Vec<String> {
    let mut list = json_string_list(region.properties.get(key));
    list.retain(|id| member_area_ids.contains(id));
    list
}

/// Plan the guest region tree for amalgamation. Pure: no I/O, no host state.
///
/// Call after [`remap_guest_custom_memory_cortical_ids_for_amalgamation`] so membership and IO
/// lists already use the final area ids. `member_area_ids` are the imported areas (base64) that
/// may live inside the amalgamation tree (the caller excludes IPU/OPU, modulator and
/// already-present areas). Top-level rule (as in region export): the guest regions whose parent
/// is the guest root (or that have no known parent) are the top level. Exactly one maps onto
/// `host_region_id` (its designated IO is returned for the host region); several each become a
/// child of the host region. Guest nesting is kept. Areas listed in no region are not in
/// `area_region` and fall back to the host region.
pub fn plan_amalgamation_regions(
    genome: &RuntimeGenome,
    host_region_id: &str,
    origin: (i32, i32, i32),
    member_area_ids: &HashSet<String>,
) -> AmalgamationRegionPlan {
    use feagi_structures::genomic::brain_regions::ROOT_BRAIN_REGION_NAME;

    let regions = &genome.brain_regions;
    let root_ids: HashSet<&str> = regions
        .iter()
        .filter(|(_, r)| r.name == ROOT_BRAIN_REGION_NAME)
        .map(|(id, _)| id.as_str())
        .collect();

    // Children by parent, sorted by guest id for deterministic output.
    let mut children: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut top_level: Vec<&str> = Vec::new();
    for (id, region) in regions {
        if root_ids.contains(id.as_str()) {
            continue;
        }
        match guest_parent_id(region) {
            Some(p) if p != id && root_ids.contains(p) => top_level.push(id),
            Some(p) if p != id && regions.contains_key(p) => {
                children.entry(p).or_default().push(id)
            }
            _ => top_level.push(id),
        }
    }
    top_level.sort_unstable();
    for list in children.values_mut() {
        list.sort_unstable();
    }

    let mut plan = AmalgamationRegionPlan::default();
    // guest region id -> new region id (host id for the single top-level region).
    let mut mapped: HashMap<&str, String> = HashMap::new();
    // Breadth-first order from the top level keeps parents before children.
    let mut queue: std::collections::VecDeque<&str> = top_level.iter().copied().collect();
    let single_top = top_level.len() == 1;
    let mut visited: HashSet<&str> = HashSet::new();

    while let Some(gid) = queue.pop_front() {
        if !visited.insert(gid) {
            continue;
        }
        let region = &regions[gid];
        let is_host = single_top && top_level[0] == gid;
        if is_host {
            mapped.insert(gid, host_region_id.to_string());
            plan.host_designated_inputs = filtered_io(region, "designated_inputs", member_area_ids);
            plan.host_designated_outputs =
                filtered_io(region, "designated_outputs", member_area_ids);
        } else {
            let parent_region_id = match guest_parent_id(region) {
                Some(p) if mapped.contains_key(p) => mapped[p].clone(),
                _ => host_region_id.to_string(),
            };
            let new_id = feagi_structures::genomic::brain_regions::RegionID::new().to_string();
            let mut properties: HashMap<String, Value> = HashMap::new();
            for (k, v) in &region.properties {
                match k.as_str() {
                    "parent_region_id" | "areas" | "cortical_areas" | "regions"
                    | "child_regions" | "coordinate_3d" | "inputs" | "outputs"
                    | "designated_inputs" | "designated_outputs" => {}
                    _ => {
                        properties.insert(k.clone(), v.clone());
                    }
                }
            }
            properties.insert(
                "coordinate_3d".to_string(),
                offset_coordinate_3d(region.properties.get("coordinate_3d"), origin),
            );
            for key in ["designated_inputs", "designated_outputs"] {
                let list = filtered_io(region, key, member_area_ids);
                if !list.is_empty() {
                    properties.insert(key.to_string(), serde_json::json!(list));
                }
            }
            mapped.insert(gid, new_id.clone());
            plan.sub_regions.push(AmalgamationRegionSpec {
                region_id: new_id,
                name: region.name.clone(),
                parent_region_id,
                properties,
            });
        }
        if let Some(kids) = children.get(gid) {
            queue.extend(kids.iter().copied());
        }
    }

    // Area membership: guest region lists first (sorted for a stable pick when an area is listed
    // twice), then the area's own `parent_region_id` property.
    let mut guest_ids: Vec<&str> = mapped.keys().copied().collect();
    guest_ids.sort_unstable();
    for gid in guest_ids {
        let target = &mapped[gid];
        if target == host_region_id {
            continue;
        }
        let mut members: Vec<String> = regions[gid]
            .cortical_areas
            .iter()
            .map(|c| c.as_base_64())
            .collect();
        members.sort_unstable();
        for m in members {
            if member_area_ids.contains(&m) {
                plan.area_region.entry(m).or_insert_with(|| target.clone());
            }
        }
    }
    for area in genome.cortical_areas.values() {
        let b64 = area.cortical_id.as_base_64();
        if !member_area_ids.contains(&b64) || plan.area_region.contains_key(&b64) {
            continue;
        }
        if let Some(target) = area
            .properties
            .get("parent_region_id")
            .and_then(|v| v.as_str())
            .and_then(|p| mapped.get(p))
        {
            if target != host_region_id {
                plan.area_region.insert(b64, target.clone());
            }
        }
    }
    plan
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::random_bytes;
    use crate::runtime::{GenomeMetadata, GenomeSignatures, GenomeStats, PhysiologyConfig};
    use crate::MorphologyRegistry;
    use crate::RuntimeGenome;
    use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaDimensions;
    use feagi_structures::genomic::cortical_area::CorticalArea;
    use feagi_structures::genomic::cortical_area::CustomCorticalType;
    use feagi_structures::genomic::descriptors::GenomeCoordinate3D;

    fn sample_custom_cortical_id() -> CorticalID {
        let mut bytes = [0u8; CorticalID::CORTICAL_ID_LENGTH];
        bytes[0] = b'c';
        random_bytes(&mut bytes[1..]);
        CorticalID::try_from_bytes(&bytes).expect("valid custom id")
    }

    #[test]
    fn remap_changes_custom_ids_when_host_reserves_them() {
        let old_id = sample_custom_cortical_id();
        let area = CorticalArea::new(
            old_id,
            0,
            "test-area".to_string(),
            CorticalAreaDimensions::new(1, 1, 1).expect("dims"),
            GenomeCoordinate3D::new(0, 0, 0),
            CorticalAreaType::Custom(CustomCorticalType::LeakyIntegrateFire),
        )
        .expect("area");

        let mut cortical_areas = HashMap::new();
        cortical_areas.insert(old_id, area);

        let mut genome = RuntimeGenome {
            metadata: GenomeMetadata {
                genome_id: "t".to_string(),
                genome_title: "t".to_string(),
                genome_description: "".to_string(),
                version: "3.0".to_string(),
                timestamp: 0.0,
                brain_regions_root: None,
            },
            cortical_areas,
            brain_regions: HashMap::new(),
            classifiers: HashMap::new(),
            morphologies: MorphologyRegistry::new(),
            modulators: crate::modulators::ModulatorRegistry::new(),
            physiology: PhysiologyConfig::default(),
            signatures: GenomeSignatures {
                genome: String::new(),
                blueprint: String::new(),
                physiology: String::new(),
                morphologies: None,
            },
            stats: GenomeStats::default(),
            change_history: Vec::new(),
        };

        let mut host = HashSet::new();
        let sample_custom = old_id.as_base_64();
        host.insert(sample_custom.clone());

        let pairs = remap_guest_custom_memory_cortical_ids_for_amalgamation(&mut genome, &host)
            .expect("remap");

        assert!(
            pairs.contains_key(&sample_custom),
            "expected reserved custom id to be remapped"
        );
        let new_b64 = pairs.get(&sample_custom).unwrap();
        assert_ne!(new_b64, &sample_custom);
        assert!(
            genome
                .cortical_areas
                .keys()
                .any(|k| k.as_base_64() == *new_b64),
            "new id should appear as a cortical_areas key"
        );
        assert!(
            !genome
                .cortical_areas
                .contains_key(&CorticalID::try_from_base_64(&sample_custom).unwrap()),
            "old custom id should not remain as a key when remapped"
        );
    }

    #[test]
    fn cloned_circuit_designated_io_follows_remapped_area_ids() {
        use feagi_structures::genomic::brain_regions::{
            BrainRegion, RegionID, RegionType, ROOT_BRAIN_REGION_NAME,
        };

        let old_id = sample_custom_cortical_id();
        let old_b64 = old_id.as_base_64();
        let area = CorticalArea::new(
            old_id,
            0,
            "io-area".to_string(),
            CorticalAreaDimensions::new(1, 1, 1).expect("dims"),
            GenomeCoordinate3D::new(0, 0, 0),
            CorticalAreaType::Custom(CustomCorticalType::LeakyIntegrateFire),
        )
        .expect("area");

        let root_id = RegionID::new();
        let circuit_id = RegionID::new();
        let root_key = root_id.to_string();
        let circuit_key = circuit_id.to_string();
        let root = BrainRegion::new(
            root_id,
            ROOT_BRAIN_REGION_NAME.to_string(),
            RegionType::Undefined,
        )
        .expect("root");
        let mut circuit_props = HashMap::new();
        circuit_props.insert(
            "parent_region_id".to_string(),
            serde_json::json!(root_key.clone()),
        );
        circuit_props.insert(
            "designated_inputs".to_string(),
            serde_json::json!([old_b64.clone()]),
        );
        circuit_props.insert("designated_outputs".to_string(), serde_json::json!([]));
        let circuit = BrainRegion::new(circuit_id, "Circuit".to_string(), RegionType::Undefined)
            .expect("circuit")
            .with_areas([old_id])
            .with_properties(circuit_props);

        let mut brain_regions = HashMap::new();
        brain_regions.insert(root_key, root);
        brain_regions.insert(circuit_key, circuit);
        let mut cortical_areas = HashMap::new();
        cortical_areas.insert(old_id, area);

        let mut genome = RuntimeGenome {
            metadata: GenomeMetadata {
                genome_id: "t".to_string(),
                genome_title: "t".to_string(),
                genome_description: "".to_string(),
                version: "3.0".to_string(),
                timestamp: 0.0,
                brain_regions_root: None,
            },
            cortical_areas,
            brain_regions,
            classifiers: HashMap::new(),
            morphologies: MorphologyRegistry::new(),
            modulators: crate::modulators::ModulatorRegistry::new(),
            physiology: PhysiologyConfig::default(),
            signatures: GenomeSignatures {
                genome: String::new(),
                blueprint: String::new(),
                physiology: String::new(),
                morphologies: None,
            },
            stats: GenomeStats::default(),
            change_history: Vec::new(),
        };

        let (inputs_before, _) = designated_io_lists_for_cloned_circuit(&genome);
        assert_eq!(inputs_before, vec![old_b64.clone()]);

        let mut host = HashSet::new();
        host.insert(old_b64.clone());
        let pairs = remap_guest_custom_memory_cortical_ids_for_amalgamation(&mut genome, &host)
            .expect("remap");
        let new_b64 = pairs.get(&old_b64).expect("remapped id");

        let (inputs_after, outputs_after) = designated_io_lists_for_cloned_circuit(&genome);
        assert_eq!(inputs_after, vec![new_b64.clone()]);
        assert!(outputs_after.is_empty());
        assert_ne!(inputs_after[0], old_b64);
    }

    fn plan_area(prefix: u8) -> CorticalArea {
        let mut bytes = [0u8; CorticalID::CORTICAL_ID_LENGTH];
        bytes[0] = prefix;
        random_bytes(&mut bytes[1..]);
        let id = CorticalID::try_from_bytes(&bytes).expect("id");
        let ty = if prefix == b'm' {
            CorticalAreaType::Memory(
                feagi_structures::genomic::cortical_area::MemoryCorticalType::Memory,
            )
        } else {
            CorticalAreaType::Custom(CustomCorticalType::LeakyIntegrateFire)
        };
        CorticalArea::new(
            id,
            0,
            "a".to_string(),
            CorticalAreaDimensions::new(1, 1, 1).expect("dims"),
            GenomeCoordinate3D::new(0, 0, 0),
            ty,
        )
        .expect("area")
    }

    fn plan_genome(areas: Vec<CorticalArea>, regions: Vec<(String, BrainRegion)>) -> RuntimeGenome {
        RuntimeGenome {
            metadata: GenomeMetadata {
                genome_id: "t".to_string(),
                genome_title: "t".to_string(),
                genome_description: "".to_string(),
                version: "3.0".to_string(),
                timestamp: 0.0,
                brain_regions_root: None,
            },
            cortical_areas: areas.into_iter().map(|a| (a.cortical_id, a)).collect(),
            brain_regions: regions.into_iter().collect(),
            classifiers: HashMap::new(),
            morphologies: MorphologyRegistry::new(),
            modulators: crate::modulators::ModulatorRegistry::new(),
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

    fn plan_region(
        name: &str,
        parent: Option<&str>,
        areas: &[&CorticalArea],
        props: Vec<(&str, Value)>,
    ) -> (String, BrainRegion) {
        use feagi_structures::genomic::brain_regions::{RegionID, RegionType};
        let rid = RegionID::new();
        let mut p: HashMap<String, Value> = HashMap::new();
        if let Some(par) = parent {
            p.insert("parent_region_id".to_string(), serde_json::json!(par));
        }
        for (k, v) in props {
            p.insert(k.to_string(), v);
        }
        let r = BrainRegion::new(rid, name.to_string(), RegionType::Undefined)
            .expect("region")
            .with_areas(areas.iter().map(|a| a.cortical_id))
            .with_properties(p);
        (rid.to_string(), r)
    }

    fn all_ids(areas: &[&CorticalArea]) -> HashSet<String> {
        areas.iter().map(|a| a.cortical_id.as_base_64()).collect()
    }

    #[test]
    fn plan_nested_regions_with_memory_area_and_ordering() {
        use feagi_structures::genomic::brain_regions::{
            BrainRegion, RegionID, RegionType, ROOT_BRAIN_REGION_NAME,
        };
        let c1 = plan_area(b'c');
        let m1 = plan_area(b'm');
        let root_id = RegionID::new().to_string();
        let root = BrainRegion::new(
            RegionID::from_string(&root_id).unwrap(),
            ROOT_BRAIN_REGION_NAME.to_string(),
            RegionType::Undefined,
        )
        .unwrap();
        let (top_id, top) = plan_region("top", Some(&root_id), &[&c1], vec![]);
        let (mid_id, mid) = plan_region(
            "mid",
            Some(&top_id),
            &[&m1],
            vec![("coordinate_3d", serde_json::json!([3, 4, 5]))],
        );
        let g = plan_genome(
            vec![c1.clone(), m1.clone()],
            vec![(root_id, root), (top_id, top), (mid_id, mid)],
        );
        let host = "host-region";
        let plan = plan_amalgamation_regions(&g, host, (10, 20, 30), &all_ids(&[&c1, &m1]));
        // single top-level maps onto host; one sub-region (mid) under host
        assert_eq!(plan.sub_regions.len(), 1);
        let sub = &plan.sub_regions[0];
        assert_eq!(sub.name, "mid");
        assert_eq!(sub.parent_region_id, host);
        assert_eq!(
            sub.properties["coordinate_3d"],
            serde_json::json!([13, 24, 35])
        );
        assert_eq!(
            plan.area_region[&m1.cortical_id.as_base_64()],
            sub.region_id
        );
        assert!(!plan.area_region.contains_key(&c1.cortical_id.as_base_64()));
    }

    #[test]
    fn plan_three_levels_parent_before_child() {
        let a = plan_area(b'c');
        let (t, top) = plan_region("t", None, &[], vec![]);
        let (m, mid) = plan_region("m", Some(&t), &[], vec![]);
        let (l, leaf) = plan_region("l", Some(&m), &[&a], vec![]);
        let g = plan_genome(vec![a.clone()], vec![(l, leaf), (m, mid), (t, top)]);
        let plan = plan_amalgamation_regions(&g, "h", (0, 0, 0), &all_ids(&[&a]));
        let names: Vec<&str> = plan.sub_regions.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["m", "l"]);
        assert_eq!(
            plan.sub_regions[1].parent_region_id,
            plan.sub_regions[0].region_id
        );
        assert_eq!(
            plan.area_region[&a.cortical_id.as_base_64()],
            plan.sub_regions[1].region_id
        );
    }

    #[test]
    fn plan_multiple_top_level_regions_become_children_of_host() {
        let a = plan_area(b'c');
        let b = plan_area(b'c');
        let (id1, r1) = plan_region(
            "one",
            None,
            &[&a],
            vec![(
                "designated_inputs",
                serde_json::json!([a.cortical_id.as_base_64()]),
            )],
        );
        let (id2, r2) = plan_region("two", None, &[&b], vec![]);
        let g = plan_genome(vec![a.clone(), b.clone()], vec![(id1, r1), (id2, r2)]);
        let plan = plan_amalgamation_regions(&g, "h", (0, 0, 0), &all_ids(&[&a, &b]));
        assert_eq!(plan.sub_regions.len(), 2);
        assert!(plan.sub_regions.iter().all(|s| s.parent_region_id == "h"));
        assert!(plan.host_designated_inputs.is_empty());
        let one = plan.sub_regions.iter().find(|s| s.name == "one").unwrap();
        assert_eq!(
            one.properties["designated_inputs"],
            serde_json::json!([a.cortical_id.as_base_64()])
        );
        assert_eq!(plan.area_region[&a.cortical_id.as_base_64()], one.region_id);
        assert_ne!(
            plan.area_region[&a.cortical_id.as_base_64()],
            plan.area_region[&b.cortical_id.as_base_64()]
        );
    }

    #[test]
    fn plan_area_in_no_region_falls_back_to_host() {
        let a = plan_area(b'c');
        let loose = plan_area(b'c');
        let (id1, r1) = plan_region("one", None, &[&a], vec![]);
        let (id2, r2) = plan_region("two", None, &[], vec![]);
        let g = plan_genome(vec![a.clone(), loose.clone()], vec![(id1, r1), (id2, r2)]);
        let plan = plan_amalgamation_regions(&g, "h", (0, 0, 0), &all_ids(&[&a, &loose]));
        assert!(!plan
            .area_region
            .contains_key(&loose.cortical_id.as_base_64()));
        assert!(plan.area_region.contains_key(&a.cortical_id.as_base_64()));
    }

    #[test]
    fn plan_nested_designated_io_is_remapped_and_filtered() {
        let a = plan_area(b'c');
        let outside = plan_area(b'c');
        let (t, top) = plan_region("t", None, &[], vec![]);
        let (m, mid) = plan_region(
            "m",
            Some(&t),
            &[&a],
            vec![
                (
                    "designated_inputs",
                    serde_json::json!([
                        a.cortical_id.as_base_64(),
                        outside.cortical_id.as_base_64()
                    ]),
                ),
                (
                    "designated_outputs",
                    serde_json::json!([a.cortical_id.as_base_64()]),
                ),
            ],
        );
        let mut g = plan_genome(vec![a.clone(), outside.clone()], vec![(t, top), (m, mid)]);
        // remap through the real remap so lists follow new ids
        let mut host = HashSet::new();
        host.insert(a.cortical_id.as_base_64());
        let pairs = remap_guest_custom_memory_cortical_ids_for_amalgamation(&mut g, &host).unwrap();
        let new_a = pairs
            .get(&a.cortical_id.as_base_64())
            .cloned()
            .expect("remapped");
        let members: HashSet<String> = [new_a.clone()].into_iter().collect();
        let plan = plan_amalgamation_regions(&g, "h", (0, 0, 0), &members);
        let sub = &plan.sub_regions[0];
        assert_eq!(
            sub.properties["designated_inputs"],
            serde_json::json!([new_a.clone()])
        );
        assert_eq!(
            sub.properties["designated_outputs"],
            serde_json::json!([new_a])
        );
    }

    #[test]
    fn plan_flat_single_region_maps_to_host_without_sub_regions() {
        let a = plan_area(b'c');
        let (id1, r1) = plan_region(
            "only",
            None,
            &[&a],
            vec![(
                "designated_inputs",
                serde_json::json!([a.cortical_id.as_base_64()]),
            )],
        );
        let g = plan_genome(vec![a.clone()], vec![(id1, r1)]);
        let plan = plan_amalgamation_regions(&g, "h", (0, 0, 0), &all_ids(&[&a]));
        assert!(plan.sub_regions.is_empty());
        assert!(plan.area_region.is_empty());
        assert_eq!(
            plan.host_designated_inputs,
            vec![a.cortical_id.as_base_64()]
        );
    }

    #[test]
    fn plan_tree_survives_save_and_reload() {
        use feagi_structures::genomic::brain_regions::{
            BrainRegion, RegionID, RegionType, ROOT_BRAIN_REGION_NAME,
        };
        let a = plan_area(b'c');
        let b = plan_area(b'm');
        let (t, top) = plan_region("t", None, &[&a], vec![]);
        let (m, mid) = plan_region("m", Some(&t), &[&b], vec![]);
        let guest = plan_genome(vec![a.clone(), b.clone()], vec![(t, top), (m, mid)]);

        // Host: root -> amalgamation region, then execute the plan as the API handler does.
        let root_id = RegionID::new();
        let host_id = RegionID::new();
        let root_key = root_id.to_string();
        let host_key = host_id.to_string();
        let root = BrainRegion::new(
            root_id,
            ROOT_BRAIN_REGION_NAME.to_string(),
            RegionType::Undefined,
        )
        .unwrap();
        let mut host_props = HashMap::new();
        host_props.insert(
            "parent_region_id".to_string(),
            serde_json::json!(root_key.clone()),
        );
        let host_region = BrainRegion::new(host_id, "host".to_string(), RegionType::Undefined)
            .unwrap()
            .with_properties(host_props);
        let members = all_ids(&[&a, &b]);
        let plan = plan_amalgamation_regions(&guest, &host_key, (0, 0, 0), &members);
        let mut regions: Vec<(String, BrainRegion)> =
            vec![(root_key, root), (host_key.clone(), host_region)];
        for spec in &plan.sub_regions {
            let mut props = spec.properties.clone();
            props.insert(
                "parent_region_id".to_string(),
                serde_json::json!(spec.parent_region_id),
            );
            let r = BrainRegion::new(
                RegionID::from_string(&spec.region_id).unwrap(),
                spec.name.clone(),
                RegionType::Undefined,
            )
            .unwrap()
            .with_properties(props);
            regions.push((spec.region_id.clone(), r));
        }
        let mut host_areas = Vec::new();
        for src in [&a, &b] {
            let mut area = src.clone();
            let target = plan
                .area_region
                .get(&src.cortical_id.as_base_64())
                .cloned()
                .unwrap_or_else(|| host_key.clone());
            area.properties.insert(
                "parent_region_id".to_string(),
                serde_json::json!(target.clone()),
            );
            for (id, r) in regions.iter_mut() {
                if *id == target {
                    r.add_area(src.cortical_id);
                }
            }
            host_areas.push(area);
        }
        let genome = plan_genome(host_areas, regions);

        let json = crate::genome::runtime_saver::save_genome_to_json(&genome).expect("save");
        let reloaded = crate::load_genome_from_json(&json).expect("reload");
        let sub = &plan.sub_regions[0];
        let r = reloaded
            .brain_regions
            .get(&sub.region_id)
            .expect("sub-region persisted");
        assert_eq!(r.name, "m");
        assert_eq!(
            r.properties
                .get("parent_region_id")
                .and_then(|v| v.as_str()),
            Some(host_key.as_str())
        );
        assert!(r.contains_area(&b.cortical_id));
    }
}
