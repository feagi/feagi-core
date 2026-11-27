// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Declared brain-region IO interface (`designated_inputs` / `designated_outputs`) and policy checks.

Cross-region mappings are validated against these lists so API/MCP and BV share one policy.
Observed boundary traffic is still merged into `inputs`/`outputs` in
`ConnectomeManager::recompute_brain_region_io_registry`.
*/

use crate::connectome_manager::ConnectomeManager;
use crate::types::{BduError, BduResult};
use feagi_structures::genomic::brain_regions::{PortEncoding, PortEncodingSource};
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::genomic::BrainRegion;

/// Genome / API property: cortical areas (base64) intended as region inputs (integration contract).
pub const DESIGNATED_INPUTS_KEY: &str = "designated_inputs";
/// Genome / API property: cortical areas (base64) intended as region outputs.
pub const DESIGNATED_OUTPUTS_KEY: &str = "designated_outputs";

/// Parse a JSON array of base64 cortical ID strings from a region property.
pub fn parse_designated_id_list(value: Option<&serde_json::Value>) -> BduResult<Vec<CorticalID>> {
    let Some(v) = value else {
        return Ok(Vec::new());
    };
    let arr = v.as_array().ok_or_else(|| {
        BduError::InvalidArea(
            "designated_inputs/designated_outputs must be JSON arrays of cortical id strings"
                .to_string(),
        )
    })?;
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        let s = item.as_str().ok_or_else(|| {
            BduError::InvalidArea(
                "designated_inputs/designated_outputs entries must be strings (base64 cortical ids)"
                    .to_string(),
            )
        })?;
        let id = CorticalID::try_from_base_64(s).map_err(|e| {
            BduError::InvalidArea(format!("Invalid cortical id in designated list: {e}"))
        })?;
        out.push(id);
    }
    Ok(out)
}

fn region_designated_lists(region: &BrainRegion) -> BduResult<(Vec<CorticalID>, Vec<CorticalID>)> {
    let inputs = parse_designated_id_list(region.properties.get(DESIGNATED_INPUTS_KEY))?;
    let outputs = parse_designated_id_list(region.properties.get(DESIGNATED_OUTPUTS_KEY))?;
    Ok((inputs, outputs))
}

/// True if `area` appears in the region's `designated_inputs` list.
pub fn area_is_designated_input(region: &BrainRegion, area: &CorticalID) -> BduResult<bool> {
    let (inputs, _) = region_designated_lists(region)?;
    Ok(inputs.iter().any(|a| a == area))
}

/// True if `area` appears in the region's `designated_outputs` list.
pub fn area_is_designated_output(region: &BrainRegion, area: &CorticalID) -> BduResult<bool> {
    let (_, outputs) = region_designated_lists(region)?;
    Ok(outputs.iter().any(|a| a == area))
}

/// Merge current region properties with an incoming PATCH for designated lists only.
pub fn merged_designated_lists(
    region: &BrainRegion,
    patch: &std::collections::HashMap<String, serde_json::Value>,
) -> BduResult<(Vec<CorticalID>, Vec<CorticalID>)> {
    let mut inputs = parse_designated_id_list(region.properties.get(DESIGNATED_INPUTS_KEY))?;
    let mut outputs = parse_designated_id_list(region.properties.get(DESIGNATED_OUTPUTS_KEY))?;

    if let Some(v) = patch.get(DESIGNATED_INPUTS_KEY) {
        inputs = parse_designated_id_list(Some(v))?;
    }
    if let Some(v) = patch.get(DESIGNATED_OUTPUTS_KEY) {
        outputs = parse_designated_id_list(Some(v))?;
    }

    Ok((inputs, outputs))
}

/// One author change to a designated port's encoding: `Some` sets an author entry, `None` clears it.
pub type PortEncodingChange = (CorticalID, Option<PortEncoding>);

/// Parse the `port_encoding` part of a region PATCH into author changes.
///
/// The PATCH value is a delta, not the whole map: `{ "<area id, base64>": <entry> | null }`.
/// An object entry (`kind`, `signed`, `frame`, `positioning`; absent means null) sets an author
/// entry; `null` clears the area's author entry. Returns an empty list when the PATCH has no
/// `port_encoding`.
pub fn parse_port_encoding_patch(
    patch: &std::collections::HashMap<String, serde_json::Value>,
) -> BduResult<Vec<PortEncodingChange>> {
    let Some(value) = patch.get(feagi_structures::genomic::brain_regions::PORT_ENCODING_KEY) else {
        return Ok(Vec::new());
    };
    let object = value.as_object().ok_or_else(|| {
        BduError::InvalidArea(
            "port_encoding must be a JSON object keyed by designated area id".to_string(),
        )
    })?;
    let mut changes = Vec::with_capacity(object.len());
    for (area_id, entry) in object {
        let id = CorticalID::try_from_base_64(area_id).map_err(|e| {
            BduError::InvalidArea(format!(
                "port_encoding key '{area_id}' is not a base64 cortical id: {e}"
            ))
        })?;
        let parsed = if entry.is_null() {
            None
        } else {
            Some(PortEncoding::from_author_value(entry).map_err(|e| {
                BduError::InvalidArea(format!("port_encoding entry for {area_id}: {e}"))
            })?)
        };
        changes.push((id, parsed));
    }
    Ok(changes)
}

/// Parse stored-form `port_encoding` entries (each with a `source`) keyed by base64 area id.
pub fn parse_stored_port_encoding(
    entries: &serde_json::Map<String, serde_json::Value>,
) -> BduResult<std::collections::BTreeMap<String, PortEncoding>> {
    feagi_structures::genomic::brain_regions::parse_port_encoding(&serde_json::Value::Object(
        entries.clone(),
    ))
    .map_err(|e| BduError::InvalidArea(format!("Invalid port_encoding: {e}")))
}

/// Friendly name of a cortical area for messages; falls back to the id when no name is known.
pub fn area_label(manager: &ConnectomeManager, id: &CorticalID) -> String {
    manager
        .get_cortical_area(id)
        .map(|a| a.name.clone())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| id.as_base_64())
}

/// Friendly name of a brain region for messages; falls back to the region id.
fn region_label(region: &BrainRegion) -> String {
    if region.name.is_empty() {
        region.region_id.to_string()
    } else {
        region.name.clone()
    }
}

/// A set (not a clear) must name a port that is designated after the same PATCH.
pub fn validate_port_encoding_changes(
    manager: &ConnectomeManager,
    region: &BrainRegion,
    changes: &[PortEncodingChange],
    merged_inputs: &[CorticalID],
    merged_outputs: &[CorticalID],
) -> BduResult<()> {
    for (id, entry) in changes {
        if entry.is_some() && !merged_inputs.iter().chain(merged_outputs).any(|d| d == id) {
            return Err(BduError::RegionIoPolicyViolation(format!(
                "Cannot set port_encoding for area {}: it is not a designated input or output of brain region {}",
                area_label(manager, id),
                region_label(region)
            )));
        }
    }
    Ok(())
}

/// Apply author changes to the region's stored `port_encoding`: a set stores an `author` entry,
/// a clear removes the entry only when its source is `author` (wiring entries are not the author's).
pub fn apply_port_encoding_changes(
    region: &mut BrainRegion,
    changes: &[PortEncodingChange],
) -> BduResult<()> {
    let mut entries = feagi_structures::genomic::brain_regions::region_port_encoding(region)
        .map_err(|e| BduError::InvalidArea(format!("Stored port_encoding is invalid: {e}")))?;
    for (id, entry) in changes {
        let key = id.as_base_64();
        match entry {
            Some(author) => {
                entries.insert(key, author.clone());
            }
            None => {
                if entries
                    .get(&key)
                    .is_some_and(|stored| stored.source == PortEncodingSource::Author)
                {
                    entries.remove(&key);
                }
            }
        }
    }
    feagi_structures::genomic::brain_regions::set_region_port_encoding(region, &entries);
    Ok(())
}

/// Every designated id must be a member of the region's cortical set.
fn validate_membership(
    manager: &ConnectomeManager,
    region: &BrainRegion,
    inputs: &[CorticalID],
    outputs: &[CorticalID],
) -> BduResult<()> {
    for id in inputs.iter().chain(outputs.iter()) {
        if !region.contains_area(id) {
            return Err(BduError::RegionIoPolicyViolation(format!(
                "Designated area {} is not contained in brain region {}",
                area_label(manager, id),
                region_label(region)
            )));
        }
    }
    Ok(())
}

/// Cannot declare as input while an outgoing cross-region mapping exists; cannot declare as output while an incoming cross-region mapping exists.
pub fn validate_merged_designations_against_connectivity(
    manager: &ConnectomeManager,
    region: &BrainRegion,
    inputs: &[CorticalID],
    outputs: &[CorticalID],
) -> BduResult<()> {
    validate_membership(manager, region, inputs, outputs)?;

    // An area on both lists is a both-way port: it may map in and out, so it is exempt.
    let is_both = |id: &CorticalID| inputs.contains(id) && outputs.contains(id);
    for id in inputs {
        if !is_both(id) && manager.has_cross_region_outgoing(id) {
            return Err(BduError::RegionIoPolicyViolation(format!(
                "Cannot set designated_inputs for area {}: it has outgoing mapping(s) to another brain region; remove those mappings first",
                area_label(manager, id)
            )));
        }
    }
    for id in outputs {
        if !is_both(id) && manager.has_cross_region_incoming(id) {
            return Err(BduError::RegionIoPolicyViolation(format!(
                "Cannot set designated_outputs for area {}: it has incoming mapping(s) from another brain region; remove those mappings first",
                area_label(manager, id)
            )));
        }
    }
    Ok(())
}

/// When adding or replacing a cross-region edge, ensure it does not violate designated roles.
pub fn validate_cross_region_mapping_proposal(
    manager: &ConnectomeManager,
    src: &CorticalID,
    dst: &CorticalID,
    mapping_data: &[serde_json::Value],
) -> BduResult<()> {
    if mapping_data.is_empty() {
        return Ok(());
    }

    let Some(src_region_id) = manager.get_parent_region_id_for_area(src) else {
        return Ok(());
    };
    let Some(dst_region_id) = manager.get_parent_region_id_for_area(dst) else {
        return Ok(());
    };

    if src_region_id == dst_region_id {
        return Ok(());
    }

    if let Some(region) = manager.get_brain_region(&src_region_id) {
        if area_is_designated_input(region, src)? && !area_is_designated_output(region, src)? {
            return Err(BduError::RegionIoPolicyViolation(format!(
                "Mapping {} -> {} rejected: source area is designated as a region input and cannot project outside its brain region",
                area_label(manager, src),
                area_label(manager, dst)
            )));
        }
    }

    if let Some(region) = manager.get_brain_region(&dst_region_id) {
        if area_is_designated_output(region, dst)? && !area_is_designated_input(region, dst)? {
            return Err(BduError::RegionIoPolicyViolation(format!(
                "Mapping {} -> {} rejected: destination area is designated as a region output and cannot receive projections from outside its brain region",
                area_label(manager, src),
                area_label(manager, dst)
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connectome_manager::ConnectomeManager;
    use feagi_structures::genomic::brain_regions::RegionID;
    use feagi_structures::genomic::cortical_area::{
        CorticalArea, CorticalAreaDimensions, CorticalAreaType, CorticalID, CustomCorticalType,
    };
    use feagi_structures::genomic::{BrainRegion, RegionType};

    fn projector_rule() -> serde_json::Value {
        serde_json::json!({
            "morphology_id": "projector",
            "postSynapticCurrent_multiplier": 1.0,
            "synapse_attractivity": 100
        })
    }

    fn make_custom_area(bytes: &[u8; 8], name: &str, idx: u32) -> (CorticalArea, CorticalID) {
        let id = CorticalID::try_from_bytes(bytes).unwrap();
        let area = CorticalArea::new(
            id,
            idx,
            name.to_string(),
            CorticalAreaDimensions::new(4, 4, 1).unwrap(),
            (0, 0, 0).into(),
            CorticalAreaType::Custom(CustomCorticalType::LeakyIntegrateFire),
        )
        .unwrap();
        (area, id)
    }

    #[test]
    fn designated_input_blocks_cross_region_efferent() {
        ConnectomeManager::reset_for_testing();
        let inst = ConnectomeManager::instance();
        let mut m = inst.write();

        let (area_in, id_in) = make_custom_area(b"cst_io01", "In", 0);
        let (area_mid, id_mid) = make_custom_area(b"cst_io02", "Mid", 1);
        let (area_ext, id_ext) = make_custom_area(b"cst_io03", "Ext", 2);
        m.add_cortical_area(area_in).unwrap();
        m.add_cortical_area(area_mid).unwrap();
        m.add_cortical_area(area_ext).unwrap();

        let rid1 = RegionID::new();
        let rid2 = RegionID::new();
        let mut br1 = BrainRegion::new(rid1, "R1".to_string(), RegionType::Undefined)
            .unwrap()
            .with_areas([id_in, id_mid]);
        br1.properties.insert(
            DESIGNATED_INPUTS_KEY.to_string(),
            serde_json::json!([id_in.as_base_64()]),
        );
        let br2 = BrainRegion::new(rid2, "R2".to_string(), RegionType::Undefined)
            .unwrap()
            .with_areas([id_ext]);

        m.add_brain_region(br1, None).unwrap();
        m.add_brain_region(br2, None).unwrap();

        let err = m
            .update_cortical_mapping(&id_in, &id_ext, vec![projector_rule()])
            .unwrap_err();
        assert!(
            matches!(err, BduError::RegionIoPolicyViolation(_)),
            "expected RegionIoPolicyViolation, got {:?}",
            err
        );
    }

    #[test]
    fn designated_output_blocks_cross_region_afferent() {
        ConnectomeManager::reset_for_testing();
        let inst = ConnectomeManager::instance();
        let mut m = inst.write();

        let (area_in, id_in) = make_custom_area(b"cst_io11", "In", 0);
        let (area_out, id_out) = make_custom_area(b"cst_io12", "Out", 1);
        let (area_ext, id_ext) = make_custom_area(b"cst_io13", "Ext", 2);
        m.add_cortical_area(area_in).unwrap();
        m.add_cortical_area(area_out).unwrap();
        m.add_cortical_area(area_ext).unwrap();

        let rid1 = RegionID::new();
        let rid2 = RegionID::new();
        let mut br1 = BrainRegion::new(rid1, "R1".to_string(), RegionType::Undefined)
            .unwrap()
            .with_areas([id_in, id_out]);
        br1.properties.insert(
            DESIGNATED_OUTPUTS_KEY.to_string(),
            serde_json::json!([id_out.as_base_64()]),
        );
        let br2 = BrainRegion::new(rid2, "R2".to_string(), RegionType::Undefined)
            .unwrap()
            .with_areas([id_ext]);

        m.add_brain_region(br1, None).unwrap();
        m.add_brain_region(br2, None).unwrap();

        let err = m
            .update_cortical_mapping(&id_ext, &id_out, vec![projector_rule()])
            .unwrap_err();
        assert!(matches!(err, BduError::RegionIoPolicyViolation(_)));
    }

    #[test]
    fn designated_lists_accept_overlap() {
        ConnectomeManager::reset_for_testing();
        let inst = ConnectomeManager::instance();
        let mut m = inst.write();

        let (area_a, id_a) = make_custom_area(b"cst_ovr1", "A", 0);
        m.add_cortical_area(area_a).unwrap();
        let rid1 = RegionID::new();
        let r1 = rid1.to_string();
        let br1 = BrainRegion::new(rid1, "R1".to_string(), RegionType::Undefined)
            .unwrap()
            .with_areas([id_a]);
        m.add_brain_region(br1, None).unwrap();

        let mut patch = std::collections::HashMap::new();
        patch.insert(
            DESIGNATED_INPUTS_KEY.to_string(),
            serde_json::json!([id_a.as_base_64()]),
        );
        patch.insert(
            DESIGNATED_OUTPUTS_KEY.to_string(),
            serde_json::json!([id_a.as_base_64()]),
        );
        m.update_brain_region_properties(&r1, patch)
            .expect("an area on both lists is a both-way port");
    }

    #[test]
    fn both_way_area_maps_in_and_out_of_its_region() {
        ConnectomeManager::reset_for_testing();
        let inst = ConnectomeManager::instance();
        let mut m = inst.write();

        let (area_port, id_port) = make_custom_area(b"cst_bw01", "Relay", 0);
        let (area_ext, id_ext) = make_custom_area(b"cst_bw02", "Ext", 1);
        m.add_cortical_area(area_port).unwrap();
        m.add_cortical_area(area_ext).unwrap();

        let rid1 = RegionID::new();
        let rid2 = RegionID::new();
        let r1 = rid1.to_string();
        let br1 = BrainRegion::new(rid1, "R1".to_string(), RegionType::Undefined)
            .unwrap()
            .with_areas([id_port]);
        let br2 = BrainRegion::new(rid2, "R2".to_string(), RegionType::Undefined)
            .unwrap()
            .with_areas([id_ext]);
        m.add_brain_region(br1, None).unwrap();
        m.add_brain_region(br2, None).unwrap();

        let mut patch = std::collections::HashMap::new();
        patch.insert(
            DESIGNATED_INPUTS_KEY.to_string(),
            serde_json::json!([id_port.as_base_64()]),
        );
        patch.insert(
            DESIGNATED_OUTPUTS_KEY.to_string(),
            serde_json::json!([id_port.as_base_64()]),
        );
        m.update_brain_region_properties(&r1, patch).unwrap();

        m.update_cortical_mapping(&id_ext, &id_port, vec![projector_rule()])
            .expect("both-way port accepts a mapping from outside");
        m.update_cortical_mapping(&id_port, &id_ext, vec![projector_rule()])
            .expect("both-way port maps out to outside");

        // Re-saving the lists with mappings in both directions still passes.
        let mut patch = std::collections::HashMap::new();
        patch.insert(
            DESIGNATED_INPUTS_KEY.to_string(),
            serde_json::json!([id_port.as_base_64()]),
        );
        patch.insert(
            DESIGNATED_OUTPUTS_KEY.to_string(),
            serde_json::json!([id_port.as_base_64()]),
        );
        m.update_brain_region_properties(&r1, patch).unwrap();
    }

    #[test]
    fn policy_errors_name_the_area_not_the_id() {
        ConnectomeManager::reset_for_testing();
        let inst = ConnectomeManager::instance();
        let mut m = inst.write();

        let (area_in, id_in) = make_custom_area(b"cst_nm01", "Sensor Port", 0);
        let (area_ext, id_ext) = make_custom_area(b"cst_nm02", "Outside", 1);
        m.add_cortical_area(area_in).unwrap();
        m.add_cortical_area(area_ext).unwrap();
        let rid1 = RegionID::new();
        let rid2 = RegionID::new();
        let mut br1 = BrainRegion::new(rid1, "R1".to_string(), RegionType::Undefined)
            .unwrap()
            .with_areas([id_in]);
        br1.properties.insert(
            DESIGNATED_INPUTS_KEY.to_string(),
            serde_json::json!([id_in.as_base_64()]),
        );
        let br2 = BrainRegion::new(rid2, "R2".to_string(), RegionType::Undefined)
            .unwrap()
            .with_areas([id_ext]);
        m.add_brain_region(br1, None).unwrap();
        m.add_brain_region(br2, None).unwrap();

        let err = m
            .update_cortical_mapping(&id_in, &id_ext, vec![projector_rule()])
            .unwrap_err();
        let text = err.to_string();
        assert!(text.contains("Sensor Port"), "{text}");
        assert!(text.contains("Outside"), "{text}");
        assert!(!text.contains(&id_in.as_base_64()), "{text}");
        assert!(!text.contains(&id_ext.as_base_64()), "{text}");
    }

    #[test]
    fn cannot_add_designated_output_while_incoming_from_outside_exists() {
        ConnectomeManager::reset_for_testing();
        let inst = ConnectomeManager::instance();
        let mut m = inst.write();

        let (area_out, id_out) = make_custom_area(b"cst_dein", "Out", 0);
        let (area_ext, id_ext) = make_custom_area(b"cst_deex", "Ext", 1);
        m.add_cortical_area(area_out).unwrap();
        m.add_cortical_area(area_ext).unwrap();

        let rid1 = RegionID::new();
        let rid2 = RegionID::new();
        let r1 = rid1.to_string();
        let br1 = BrainRegion::new(rid1, "R1".to_string(), RegionType::Undefined)
            .unwrap()
            .with_areas([id_out]);
        let br2 = BrainRegion::new(rid2, "R2".to_string(), RegionType::Undefined)
            .unwrap()
            .with_areas([id_ext]);
        m.add_brain_region(br1, None).unwrap();
        m.add_brain_region(br2, None).unwrap();

        m.update_cortical_mapping(&id_ext, &id_out, vec![projector_rule()])
            .unwrap();

        let mut patch = std::collections::HashMap::new();
        patch.insert(
            DESIGNATED_OUTPUTS_KEY.to_string(),
            serde_json::json!([id_out.as_base_64()]),
        );
        let err = m.update_brain_region_properties(&r1, patch).unwrap_err();
        assert!(matches!(err, BduError::RegionIoPolicyViolation(_)));
    }
}
