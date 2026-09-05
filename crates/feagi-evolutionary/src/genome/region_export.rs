// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Export a branch of the brain-region tree as an independent `RuntimeGenome` suitable for
`save_genome_to_json` (flat format 3.0).

A nested circuit is exported as a **child** of a new `Root Brain Region`. The circuit
keeps its original region name and cortical-area membership so reload and new experiments
show internals inside that circuit, not flattened onto the genome root.

Exporting a region already named `Root Brain Region` is left as-is (full-tree export).

Cortical mappings that target areas outside the branch are stripped from
`cortical_mapping_dst` to avoid dangling references.

Copyright 2025 Neuraville Inc.
*/

use crate::runtime::{GenomeMetadata, GenomeSignatures, GenomeStats, RuntimeGenome};
use crate::{EvoError, EvoResult};
use feagi_structures::genomic::brain_regions::{
    BrainRegion, RegionID, RegionType, ROOT_BRAIN_REGION_NAME,
};
use feagi_structures::genomic::cortical_area::{CorticalArea, CorticalID};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet, VecDeque};

/// True when `parent_region_id` is missing, empty, self, or not present in `brain_regions`.
fn region_is_parentless(
    region_id: &str,
    region: &BrainRegion,
    brain_regions: &HashMap<String, BrainRegion>,
) -> bool {
    match region
        .properties
        .get("parent_region_id")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|p| !p.is_empty())
    {
        Some(parent) if parent != region_id && brain_regions.contains_key(parent) => false,
        _ => true,
    }
}

/// If the map has regions but no `Root Brain Region`, insert one and parent every
/// parentless region under it. Existing region names and area membership are unchanged.
///
/// Returns the new root region id when wrapping happened.
pub fn wrap_parentless_regions_under_named_root(
    brain_regions: &mut HashMap<String, BrainRegion>,
) -> Option<String> {
    if brain_regions.is_empty() {
        return None;
    }
    if brain_regions
        .values()
        .any(|r| r.name == ROOT_BRAIN_REGION_NAME)
    {
        return None;
    }

    let parentless: Vec<String> = brain_regions
        .iter()
        .filter(|(rid, region)| region_is_parentless(rid, region, brain_regions))
        .map(|(rid, _)| rid.clone())
        .collect();

    let wrapper_id = RegionID::new();
    let wrapper_key = wrapper_id.to_string();
    // Name is the non-empty canonical root label.
    let wrapper = BrainRegion::new(
        wrapper_id,
        ROOT_BRAIN_REGION_NAME.to_string(),
        RegionType::Undefined,
    )
    .expect("ROOT_BRAIN_REGION_NAME is a non-empty constant");

    for rid in &parentless {
        if let Some(region) = brain_regions.get_mut(rid) {
            region.add_property("parent_region_id".to_string(), json!(wrapper_key.clone()));
        }
    }
    brain_regions.insert(wrapper_key.clone(), wrapper);
    Some(wrapper_key)
}

/// Seed/template labels that are not a user-chosen circuit name.
fn is_placeholder_circuit_title(title: &str) -> bool {
    let t = title.trim();
    t.is_empty()
        || t.eq_ignore_ascii_case("untitled")
        || t.eq_ignore_ascii_case("autogen circuit")
        || t.eq_ignore_ascii_case("neural circuit")
        || t.eq_ignore_ascii_case("the essential genome")
        || t.eq_ignore_ascii_case("current genome")
        || t.eq_ignore_ascii_case("exported genome")
        || t == ROOT_BRAIN_REGION_NAME
}

/// Display name for an exported circuit: live `.name`, then title properties.
fn region_display_name(region: &BrainRegion) -> String {
    let from_name = region.name.trim();
    if !from_name.is_empty() && !is_placeholder_circuit_title(from_name) {
        return from_name.to_string();
    }
    for key in ["region_title", "title", "name"] {
        if let Some(prop) = region
            .properties
            .get(key)
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            if !is_placeholder_circuit_title(prop) {
                return prop.to_string();
            }
        }
    }
    if !from_name.is_empty() {
        return from_name.to_string();
    }
    "Neural circuit".to_string()
}

/// Prefer a custom region name; if that is a seed leftover, use `genome_title`.
fn resolve_circuit_title(region_name: &str, genome_title: &str) -> String {
    let region = region_name.trim();
    let genome = genome_title.trim();
    let region_placeholder = is_placeholder_circuit_title(region);
    let genome_placeholder = is_placeholder_circuit_title(genome);
    if !region.is_empty() && !region_placeholder {
        return region.to_string();
    }
    if !genome.is_empty() && !genome_placeholder {
        return genome.to_string();
    }
    if !region.is_empty() {
        return region.to_string();
    }
    if !genome.is_empty() {
        return genome.to_string();
    }
    "Neural circuit".to_string()
}

/// Id of the unique top-level non-root circuit, if the tree has exactly one.
fn unique_top_level_circuit_id(brain_regions: &HashMap<String, BrainRegion>) -> Option<String> {
    let root_id = brain_regions
        .iter()
        .find_map(|(id, region)| (region.name == ROOT_BRAIN_REGION_NAME).then(|| id.clone()));

    let mut candidates: Vec<String> = Vec::new();
    for (id, region) in brain_regions {
        if region.name == ROOT_BRAIN_REGION_NAME {
            continue;
        }
        let is_top = match &root_id {
            Some(root) => region
                .properties
                .get("parent_region_id")
                .and_then(|v| v.as_str())
                .map(|parent| parent == root)
                .unwrap_or_else(|| region_is_parentless(id, region, brain_regions)),
            None => region_is_parentless(id, region, brain_regions),
        };
        if is_top {
            candidates.push(id.clone());
        }
    }
    if candidates.len() == 1 {
        candidates.pop()
    } else {
        None
    }
}

/// When a genome has exactly one top-level circuit, set that circuit's name from
/// `genome_title` if the stored region name is a seed leftover (e.g. "The Essential Genome").
///
/// Returns the name that was written, or `None` when unchanged or not applicable.
pub fn apply_genome_title_to_unique_top_circuit(
    brain_regions: &mut HashMap<String, BrainRegion>,
    genome_title: &str,
) -> Option<String> {
    let id = unique_top_level_circuit_id(brain_regions)?;
    let current = brain_regions.get(&id)?.name.clone();
    let resolved = resolve_circuit_title(&current, genome_title);
    if resolved == current {
        return None;
    }
    let region = brain_regions.get_mut(&id)?;
    region.name = resolved.clone();
    Some(resolved)
}

/// Build a map: parent region id -> direct child region ids (from `parent_region_id` properties).
fn children_by_parent(genome: &RuntimeGenome) -> HashMap<String, Vec<String>> {
    let mut m: HashMap<String, Vec<String>> = HashMap::new();
    for (rid, region) in &genome.brain_regions {
        if let Some(p) = region
            .properties
            .get("parent_region_id")
            .and_then(|v| v.as_str())
        {
            if !p.is_empty() {
                m.entry(p.to_string()).or_default().push(rid.clone());
            }
        }
    }
    m
}

/// Region id and all descendants (BFS), including `root_region_id`.
fn collect_region_branch_ids(
    genome: &RuntimeGenome,
    root_region_id: &str,
    children_by_parent: &HashMap<String, Vec<String>>,
) -> EvoResult<Vec<String>> {
    if !genome.brain_regions.contains_key(root_region_id) {
        return Err(EvoError::InvalidRegion(format!(
            "Unknown region_id: {}",
            root_region_id
        )));
    }

    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut q: VecDeque<String> = VecDeque::new();
    q.push_back(root_region_id.to_string());

    while let Some(rid) = q.pop_front() {
        if !seen.insert(rid.clone()) {
            continue;
        }
        out.push(rid.clone());
        if let Some(kids) = children_by_parent.get(&rid) {
            for k in kids {
                q.push_back(k.clone());
            }
        }
    }

    Ok(out)
}

/// Collect cortical IDs (base64) assigned to any region in `branch_region_ids`.
fn cortical_ids_in_branch(genome: &RuntimeGenome, branch_region_ids: &[String]) -> HashSet<String> {
    let mut ids = HashSet::new();
    for rid in branch_region_ids {
        if let Some(br) = genome.brain_regions.get(rid) {
            for cid in &br.cortical_areas {
                ids.insert(cid.as_base_64());
            }
        }
    }
    ids
}

/// Remove `cortical_mapping_dst` entries whose destination keys are not in `kept`.
fn strip_dst_mappings_outside_branch(area: &mut CorticalArea, kept: &HashSet<String>) {
    let Some(Value::Object(dst_map)) = area.properties.get_mut("cortical_mapping_dst") else {
        return;
    };
    dst_map.retain(|dst_key, _| kept.contains(dst_key));
}

/// Clone [`RuntimeGenome`] to only include the subtree rooted at `root_region_id`.
///
/// - Preserves physiology and full morphology registry from the source genome.
/// - Sets a new `genome_id`; `genome_title` is the exported circuit's region name.
/// - Nested circuits are wrapped under a new `Root Brain Region`; the circuit name is kept.
/// - Strips synapse destination mappings that leave the branch.
pub fn subset_runtime_genome_for_region_branch(
    genome: &RuntimeGenome,
    root_region_id: &str,
) -> EvoResult<RuntimeGenome> {
    let children = children_by_parent(genome);
    let branch_ids = collect_region_branch_ids(genome, root_region_id, &children)?;
    let branch_set: HashSet<String> = branch_ids.iter().cloned().collect();

    let kept_cortical = cortical_ids_in_branch(genome, &branch_ids);

    let mut cortical_areas: HashMap<CorticalID, CorticalArea> = HashMap::new();
    for (cid, area) in &genome.cortical_areas {
        let b64 = cid.as_base_64();
        if kept_cortical.contains(&b64) {
            let mut a = area.clone();
            strip_dst_mappings_outside_branch(&mut a, &kept_cortical);
            cortical_areas.insert(*cid, a);
        }
    }

    let mut brain_regions: HashMap<String, BrainRegion> = HashMap::new();
    for rid in &branch_ids {
        let Some(mut br) = genome.brain_regions.get(rid).cloned() else {
            continue;
        };
        if rid == root_region_id {
            br.properties.remove("parent_region_id");
        } else if let Some(parent) = br
            .properties
            .get("parent_region_id")
            .and_then(|v| v.as_str())
        {
            if !branch_set.contains(parent) {
                br.properties.remove("parent_region_id");
            }
        }
        brain_regions.insert(rid.clone(), br);
    }

    let circuit_name = genome
        .brain_regions
        .get(root_region_id)
        .map(region_display_name)
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| "Neural circuit".to_string());
    if let Some(circuit) = brain_regions.get_mut(root_region_id) {
        circuit.name = circuit_name.clone();
    }

    let genome_root_id = wrap_parentless_regions_under_named_root(&mut brain_regions)
        .unwrap_or_else(|| root_region_id.to_string());

    let ts = chrono::Utc::now().timestamp() as f64;
    let new_id = format!("region_export_{}", chrono::Utc::now().timestamp_millis());

    let metadata = GenomeMetadata {
        genome_id: new_id,
        genome_title: circuit_name,
        genome_description: format!(
            "Neural circuit export rooted at region {} from genome {}",
            root_region_id, genome.metadata.genome_id
        ),
        version: genome.metadata.version.clone(),
        timestamp: ts,
        brain_regions_root: Some(genome_root_id),
    };

    Ok(RuntimeGenome {
        metadata,
        cortical_areas,
        brain_regions,
        morphologies: genome.morphologies.clone(),
        physiology: genome.physiology.clone(),
        signatures: GenomeSignatures {
            genome: "0".to_string(),
            blueprint: "0".to_string(),
            physiology: "0".to_string(),
            morphologies: None,
        },
        stats: GenomeStats::default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{
        GenomeMetadata, GenomeSignatures, GenomeStats, MorphologyRegistry, PhysiologyConfig,
        RuntimeGenome,
    };
    use feagi_structures::genomic::brain_regions::{
        BrainRegion, RegionID, RegionType, ROOT_BRAIN_REGION_NAME,
    };
    use feagi_structures::genomic::cortical_area::CorticalID;
    use serde_json::json;
    use std::collections::HashMap;

    fn runtime_parent_and_child() -> (RuntimeGenome, String, String) {
        let parent_rid = RegionID::new();
        let child_rid = RegionID::new();
        let parent_key = parent_rid.to_string();
        let child_key = child_rid.to_string();

        let parent = BrainRegion::new(parent_rid, "Parent".to_string(), RegionType::Undefined)
            .expect("parent region");
        let mut child = BrainRegion::new(child_rid, "Child".to_string(), RegionType::Undefined)
            .expect("child region");
        child.add_property("parent_region_id".to_string(), json!(parent_key.clone()));

        let mut brain_regions = HashMap::new();
        brain_regions.insert(parent_key.clone(), parent);
        brain_regions.insert(child_key.clone(), child);

        let g = RuntimeGenome {
            metadata: GenomeMetadata {
                genome_id: "fixture".to_string(),
                genome_title: "Fixture".to_string(),
                genome_description: String::new(),
                version: "3.0".to_string(),
                timestamp: 0.0,
                brain_regions_root: Some(parent_key.clone()),
            },
            cortical_areas: HashMap::new(),
            brain_regions,
            morphologies: MorphologyRegistry::new(),
            physiology: PhysiologyConfig::default(),
            signatures: GenomeSignatures {
                genome: "0".to_string(),
                blueprint: "0".to_string(),
                physiology: "0".to_string(),
                morphologies: None,
            },
            stats: GenomeStats::default(),
        };
        (g, parent_key, child_key)
    }

    #[test]
    fn subset_rejects_unknown_region() {
        let (g, _p, _c) = runtime_parent_and_child();
        let err = subset_runtime_genome_for_region_branch(&g, "nonexistent-region-id").unwrap_err();
        assert!(matches!(err, EvoError::InvalidRegion(_)));
    }

    #[test]
    fn wrap_is_noop_when_empty_or_root_already_present() {
        let mut empty: HashMap<String, BrainRegion> = HashMap::new();
        assert!(wrap_parentless_regions_under_named_root(&mut empty).is_none());

        let root_id = RegionID::new();
        let root_key = root_id.to_string();
        let root = BrainRegion::new(
            root_id,
            ROOT_BRAIN_REGION_NAME.to_string(),
            RegionType::Undefined,
        )
        .expect("root");
        let mut with_root = HashMap::new();
        with_root.insert(root_key, root);
        assert!(wrap_parentless_regions_under_named_root(&mut with_root).is_none());
        assert_eq!(with_root.len(), 1);
    }

    #[test]
    fn wrap_parents_circuit_under_new_root_and_keeps_circuit_name() {
        let circuit_id = RegionID::new();
        let circuit_key = circuit_id.to_string();
        let area_id = CorticalID::try_from_bytes(b"cst_neur").expect("cortical id");
        let circuit = BrainRegion::new(
            circuit_id,
            "The Essential Genome".to_string(),
            RegionType::Undefined,
        )
        .expect("circuit")
        .with_areas([area_id]);
        let mut regions = HashMap::new();
        regions.insert(circuit_key.clone(), circuit);

        let wrapper_key = wrap_parentless_regions_under_named_root(&mut regions).expect("wrapped");
        assert_eq!(regions.len(), 2);
        assert_eq!(regions[&wrapper_key].name, ROOT_BRAIN_REGION_NAME);
        assert_eq!(regions[&circuit_key].name, "The Essential Genome");
        assert!(regions[&circuit_key].contains_area(&area_id));
        assert!(!regions[&wrapper_key].contains_area(&area_id));
        assert_eq!(
            regions[&circuit_key]
                .properties
                .get("parent_region_id")
                .and_then(|v| v.as_str()),
            Some(wrapper_key.as_str())
        );
        assert!(regions[&wrapper_key].cortical_areas.is_empty());
    }

    #[test]
    fn wrap_keeps_nested_child_parent_and_only_reparents_parentless() {
        let parent_id = RegionID::new();
        let child_id = RegionID::new();
        let parent_key = parent_id.to_string();
        let child_key = child_id.to_string();
        let parent = BrainRegion::new(parent_id, "Circuit".to_string(), RegionType::Undefined)
            .expect("parent");
        let mut child =
            BrainRegion::new(child_id, "Inner".to_string(), RegionType::Undefined).expect("child");
        child.add_property("parent_region_id".to_string(), json!(parent_key.clone()));

        let mut regions = HashMap::new();
        regions.insert(parent_key.clone(), parent);
        regions.insert(child_key.clone(), child);

        let wrapper_key = wrap_parentless_regions_under_named_root(&mut regions).expect("wrapped");
        assert_eq!(
            regions[&parent_key]
                .properties
                .get("parent_region_id")
                .and_then(|v| v.as_str()),
            Some(wrapper_key.as_str())
        );
        assert_eq!(
            regions[&child_key]
                .properties
                .get("parent_region_id")
                .and_then(|v| v.as_str()),
            Some(parent_key.as_str())
        );
        assert_eq!(regions[&parent_key].name, "Circuit");
        assert_eq!(regions[&child_key].name, "Inner");
    }

    #[test]
    fn subset_child_region_is_wrapped_under_new_root_preserving_name() {
        let (g, _parent_key, child_key) = runtime_parent_and_child();
        let sub = subset_runtime_genome_for_region_branch(&g, &child_key).expect("subset");
        assert_eq!(sub.brain_regions.len(), 2);
        assert!(sub.brain_regions.contains_key(&child_key));
        assert_eq!(sub.brain_regions[&child_key].name, "Child");
        assert_eq!(sub.metadata.genome_title, "Child");
        let wrapper_id = sub
            .metadata
            .brain_regions_root
            .as_deref()
            .expect("wrapped root id");
        assert_ne!(wrapper_id, child_key.as_str());
        assert_eq!(sub.brain_regions[wrapper_id].name, ROOT_BRAIN_REGION_NAME);
        assert_eq!(
            sub.brain_regions[&child_key]
                .properties
                .get("parent_region_id")
                .and_then(|v| v.as_str()),
            Some(wrapper_id)
        );
    }

    #[test]
    fn subset_named_root_is_not_double_wrapped() {
        let root_id = RegionID::new();
        let child_id = RegionID::new();
        let root_key = root_id.to_string();
        let child_key = child_id.to_string();
        let root = BrainRegion::new(
            root_id,
            ROOT_BRAIN_REGION_NAME.to_string(),
            RegionType::Undefined,
        )
        .expect("root");
        let mut child = BrainRegion::new(child_id, "Circuit".to_string(), RegionType::Undefined)
            .expect("child");
        child.add_property("parent_region_id".to_string(), json!(root_key.clone()));

        let mut brain_regions = HashMap::new();
        brain_regions.insert(root_key.clone(), root);
        brain_regions.insert(child_key.clone(), child);

        let g = RuntimeGenome {
            metadata: GenomeMetadata {
                genome_id: "fixture".to_string(),
                genome_title: "Fixture".to_string(),
                genome_description: String::new(),
                version: "3.0".to_string(),
                timestamp: 0.0,
                brain_regions_root: Some(root_key.clone()),
            },
            cortical_areas: HashMap::new(),
            brain_regions,
            morphologies: MorphologyRegistry::new(),
            physiology: PhysiologyConfig::default(),
            signatures: GenomeSignatures {
                genome: "0".to_string(),
                blueprint: "0".to_string(),
                physiology: "0".to_string(),
                morphologies: None,
            },
            stats: GenomeStats::default(),
        };

        let sub = subset_runtime_genome_for_region_branch(&g, &root_key).expect("subset");
        let named_roots = sub
            .brain_regions
            .values()
            .filter(|r| r.name == ROOT_BRAIN_REGION_NAME)
            .count();
        assert_eq!(named_roots, 1);
        assert_eq!(sub.brain_regions.len(), 2);
        assert_eq!(
            sub.metadata.brain_regions_root.as_deref(),
            Some(root_key.as_str())
        );
        assert_eq!(sub.brain_regions[&child_key].name, "Circuit");
    }

    #[test]
    fn subset_uses_region_title_property_when_name_is_seed_leftover() {
        let (mut g, _parent_key, child_key) = runtime_parent_and_child();
        {
            let child = g.brain_regions.get_mut(&child_key).expect("child");
            child.name = "The Essential Genome".to_string();
            child.add_property("region_title".to_string(), json!("Goal Seeker"));
        }
        let sub = subset_runtime_genome_for_region_branch(&g, &child_key).expect("subset");
        assert_eq!(sub.brain_regions[&child_key].name, "Goal Seeker");
        assert_eq!(sub.metadata.genome_title, "Goal Seeker");
    }

    #[test]
    fn apply_replaces_essential_genome_name_with_custom_genome_title() {
        let circuit_id = RegionID::new();
        let circuit_key = circuit_id.to_string();
        let root_id = RegionID::new();
        let root_key = root_id.to_string();
        let root = BrainRegion::new(
            root_id,
            ROOT_BRAIN_REGION_NAME.to_string(),
            RegionType::Undefined,
        )
        .expect("root");
        let mut circuit = BrainRegion::new(
            circuit_id,
            "The Essential Genome".to_string(),
            RegionType::Undefined,
        )
        .expect("circuit");
        circuit.add_property("parent_region_id".to_string(), json!(root_key.clone()));
        let mut regions = HashMap::new();
        regions.insert(root_key, root);
        regions.insert(circuit_key.clone(), circuit);

        let written =
            apply_genome_title_to_unique_top_circuit(&mut regions, "Goal Seeker").expect("renamed");
        assert_eq!(written, "Goal Seeker");
        assert_eq!(regions[&circuit_key].name, "Goal Seeker");
        assert_eq!(
            regions
                .values()
                .filter(|r| r.name == ROOT_BRAIN_REGION_NAME)
                .count(),
            1
        );
    }

    #[test]
    fn apply_keeps_custom_region_name_when_genome_title_is_generic() {
        let circuit_id = RegionID::new();
        let circuit_key = circuit_id.to_string();
        let circuit =
            BrainRegion::new(circuit_id, "Goal Seeker".to_string(), RegionType::Undefined)
                .expect("circuit");
        let mut regions = HashMap::new();
        regions.insert(circuit_key.clone(), circuit);
        wrap_parentless_regions_under_named_root(&mut regions);

        assert!(
            apply_genome_title_to_unique_top_circuit(&mut regions, "The Essential Genome")
                .is_none()
        );
        assert_eq!(regions[&circuit_key].name, "Goal Seeker");
    }

    #[test]
    fn apply_skips_when_two_top_level_circuits() {
        let a_id = RegionID::new();
        let b_id = RegionID::new();
        let a_key = a_id.to_string();
        let b_key = b_id.to_string();
        let mut regions = HashMap::new();
        regions.insert(
            a_key,
            BrainRegion::new(a_id, "Circuit A".to_string(), RegionType::Undefined).expect("a"),
        );
        regions.insert(
            b_key,
            BrainRegion::new(b_id, "Circuit B".to_string(), RegionType::Undefined).expect("b"),
        );
        wrap_parentless_regions_under_named_root(&mut regions);
        assert!(apply_genome_title_to_unique_top_circuit(&mut regions, "Hub Title").is_none());
    }
}
