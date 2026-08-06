// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Export a branch of the brain-region tree as an independent `RuntimeGenome` suitable for
`save_genome_to_json` (flat format 3.0).

The selected region becomes the new genome root (`metadata.brain_regions_root`).
Cortical mappings that target areas outside the branch are stripped from
`cortical_mapping_dst` to avoid dangling references.

Copyright 2025 Neuraville Inc.
*/

use crate::runtime::{GenomeMetadata, GenomeSignatures, GenomeStats, RuntimeGenome};
use crate::{EvoError, EvoResult};
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};
use feagi_genomic_context::brain_region::BrainRegion;
use feagi_genomic_context::cortical_area::CorticalID;
use feagi_genomic_data::cortical_area_prev::CorticalArea;

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
        return Err(EvoError::invalid_region(format!(
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

/// Collect cortical_area IDs (base64) assigned to any region in `branch_region_ids`.
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
/// - Sets a new `genome_id` / title on the exported genome.
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

    let root_name = genome
        .brain_regions
        .get(root_region_id)
        .map(|r| r.name.clone())
        .unwrap_or_else(|| "Neural circuit".to_string());

    let ts = chrono::Utc::now().timestamp() as f64;
    let new_id = format!("region_export_{}", chrono::Utc::now().timestamp_millis());

    let metadata = GenomeMetadata {
        genome_id: new_id,
        genome_title: root_name,
        genome_description: format!(
            "Neural circuit export rooted at region {} from genome {}",
            root_region_id, genome.metadata.genome_id
        ),
        version: genome.metadata.version.clone(),
        timestamp: ts,
        brain_regions_root: Some(root_region_id.to_string()),
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
    use serde_json::json;
    use std::collections::HashMap;
    use feagi_genomic_context::brain_region::{BrainRegion, RegionID, RegionType};

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
    fn subset_child_region_becomes_new_root() {
        let (g, _parent_key, child_key) = runtime_parent_and_child();
        let sub = subset_runtime_genome_for_region_branch(&g, &child_key).expect("subset");
        assert_eq!(sub.brain_regions.len(), 1);
        assert!(sub.brain_regions.contains_key(&child_key));
        assert_eq!(
            sub.metadata.brain_regions_root.as_deref(),
            Some(child_key.as_str())
        );
        assert!(!sub.brain_regions[&child_key]
            .properties
            .contains_key("parent_region_id"));
    }
}
