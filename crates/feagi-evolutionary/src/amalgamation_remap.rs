// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Remap **Custom** and **Memory** cortical_area IDs on an imported guest genome so they do not collide
with the host connectome.

Core (`___death`, `___power`, `___fatig`, `___pain_`, `___pleas`, `___fear_`, `___hope_`) and IPU/OPU identifiers are **not** remapped: they are
canonical and must continue to match the host’s shared IO and core regions.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::random::random_bytes;
use crate::runtime::RuntimeGenome;
use feagi_data::feagi_data_error::FeagiDataError;
use feagi_genomic_context::cortical_area::CorticalAreaType;
use feagi_genomic_context::cortical_area::CorticalID;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

use feagi_data::feagi_data_error::FeagiFailDataEtc;
use feagi_genomic_context::brain_region::BrainRegion;
use feagi_genomic_context::genome_positioning::GenomeCoordinate3D;
use feagi_genomic_data::cortical_area_prev::CorticalArea;

fn feagi_data_etc_error(message: String) -> FeagiDataError {
    let context: &'static str = Box::leak(message.into_boxed_str());
    FeagiFailDataEtc::new(context).into()
}

const MAX_ID_ALLOC_ATTEMPTS: u32 = 100_000;

/// Allocate a new cortical_area ID with the given first-byte prefix (`c` or `m`), distinct from
/// `reserved` (inserts the chosen ID into `reserved`).
fn allocate_unique_typed_id(prefix: u8, reserved: &mut HashSet<String>) -> Result<CorticalID, FeagiDataError> {
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
    Err(feagi_data_etc_error(
        "Failed to allocate a unique cortical_area ID for amalgamation remapping".into(),
    ))
}

fn remap_region_io_lists(region: &mut BrainRegion, b64_remap: &HashMap<String, String>) {
    for key in ["inputs", "outputs", "designated_inputs", "designated_outputs"] {
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

fn remap_cortical_mapping_dst_keys(properties: &mut HashMap<String, Value>, b64_remap: &HashMap<String, String>) {
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

/// Remap guest **Custom** and **Memory** cortical_area IDs to fresh values not present in
/// `host_reserved_base64_ids` or elsewhere in this guest genome. Updates cortical_area area keys,
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
        let needs_remapping = matches!(&area.cortical_type, CorticalAreaType::Custom(_) | CorticalAreaType::Memory(_));
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
    let mut new_areas: HashMap<CorticalID, CorticalArea> = HashMap::with_capacity(genome.cortical_areas.len());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::random_bytes;
    use crate::runtime::{GenomeMetadata, GenomeSignatures, GenomeStats, PhysiologyConfig};
    use crate::MorphologyRegistry;
    use crate::RuntimeGenome;
    use feagi_data::neurons::voxel_potentials::wrapped_values::NeuronVoxelDimensionsGenomic;
    use feagi_genomic_context::cortical_area::CustomCorticalType;

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
            NeuronVoxelDimensionsGenomic::new_from_usizes_unchecked(1, 1, 1),
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
            morphologies: MorphologyRegistry::new(),
            physiology: PhysiologyConfig::default(),
            signatures: GenomeSignatures {
                genome: String::new(),
                blueprint: String::new(),
                physiology: String::new(),
                morphologies: None,
            },
            stats: GenomeStats::default(),
        };

        let mut host = HashSet::new();
        let sample_custom = old_id.as_base_64();
        host.insert(sample_custom.clone());

        let pairs = remap_guest_custom_memory_cortical_ids_for_amalgamation(&mut genome, &host).expect("remap");

        assert!(pairs.contains_key(&sample_custom), "expected reserved custom id to be remapped");
        let new_b64 = pairs.get(&sample_custom).unwrap();
        assert_ne!(new_b64, &sample_custom);
        assert!(
            genome.cortical_areas.keys().any(|k| k.as_base_64() == *new_b64),
            "new id should appear as a cortical_areas key"
        );
        assert!(
            !genome.cortical_areas.contains_key(&CorticalID::try_from_base_64(&sample_custom).unwrap()),
            "old custom id should not remain as a key when remapped"
        );
    }
}
