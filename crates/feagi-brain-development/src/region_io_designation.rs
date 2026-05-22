// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Declared brain-region IO interface (`designated_inputs` / `designated_outputs`) and policy checks.

Cross-region mappings are validated against these lists so API/MCP and BV share one policy.
Observed boundary traffic is still merged into `inputs`/`outputs` in
`ConnectomeManager::recompute_brain_region_io_registry`.
*/

use std::collections::HashSet;

use crate::connectome_manager::ConnectomeManager;
use crate::types::{BduError, BduResult};
use feagi_genome_definitions::::CorticalID;
use feagi_genome_definitions::::brain_region::BrainRegion;

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

/// Reject duplicate ids across input and output designation.
fn validate_no_overlap(inputs: &[CorticalID], outputs: &[CorticalID]) -> BduResult<()> {
    let mut seen: HashSet<String> = HashSet::new();
    for i in inputs {
        seen.insert(i.as_base_64());
    }
    for o in outputs {
        if seen.contains(&o.as_base_64()) {
            return Err(BduError::RegionIoPolicyViolation(format!(
                "Cortical area {} cannot appear in both designated_inputs and designated_outputs",
                o.as_base_64()
            )));
        }
    }
    Ok(())
}

/// Every designated id must be a member of the region's cortical set.
fn validate_membership(
    region: &BrainRegion,
    inputs: &[CorticalID],
    outputs: &[CorticalID],
) -> BduResult<()> {
    for id in inputs.iter().chain(outputs.iter()) {
        if !region.contains_area(id) {
            return Err(BduError::RegionIoPolicyViolation(format!(
                "Designated area {} is not contained in brain region {}",
                id.as_base_64(),
                region.region_id
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
    validate_no_overlap(inputs, outputs)?;
    validate_membership(region, inputs, outputs)?;

    for id in inputs {
        if manager.has_cross_region_outgoing(id) {
            return Err(BduError::RegionIoPolicyViolation(format!(
                "Cannot set designated_inputs for area {}: it has outgoing mapping(s) to another brain region; remove those mappings first",
                id.as_base_64()
            )));
        }
    }
    for id in outputs {
        if manager.has_cross_region_incoming(id) {
            return Err(BduError::RegionIoPolicyViolation(format!(
                "Cannot set designated_outputs for area {}: it has incoming mapping(s) from another brain region; remove those mappings first",
                id.as_base_64()
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
        if area_is_designated_input(region, src)? {
            return Err(BduError::RegionIoPolicyViolation(format!(
                "Mapping {} -> {} rejected: source area is designated as a region input and cannot project outside its brain region",
                src.as_base_64(),
                dst.as_base_64()
            )));
        }
    }

    if let Some(region) = manager.get_brain_region(&dst_region_id) {
        if area_is_designated_output(region, dst)? {
            return Err(BduError::RegionIoPolicyViolation(format!(
                "Mapping {} -> {} rejected: destination area is designated as a region output and cannot receive projections from outside its brain region",
                src.as_base_64(),
                dst.as_base_64()
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connectome_manager::ConnectomeManager;
    use feagi_genome_definitions::::RegionID;
    use feagi_genome_definitions::::{
        CorticalArea, CorticalAreaDimensions, CorticalAreaType, CorticalID, CustomCorticalType,
    };
    use feagi_genome_definitions::::brain_region::BrainRegion;
    use feagi_genome_definitions::::region_type::RegionType;

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
    fn designated_lists_reject_overlap() {
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
        let err = m.update_brain_region_properties(&r1, patch).unwrap_err();
        assert!(matches!(err, BduError::RegionIoPolicyViolation(_)));
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
