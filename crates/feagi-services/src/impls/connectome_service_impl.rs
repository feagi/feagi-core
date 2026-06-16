// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Connectome service implementation.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::traits::ConnectomeService;
use crate::types::*;
use async_trait::async_trait;
use feagi_brain_development::models::CorticalAreaExt;
use feagi_brain_development::ConnectomeManager;
use feagi_evolutionary::{get_default_neural_properties, MemoryAreaProperties};
use feagi_npu_burst_engine::BurstLoopRunner;
use feagi_genome_definitions::::RegionID;
use feagi_genome_definitions::::io_cortical_area_configuration_flag::{
    FrameChangeHandling, PercentageNeuronPositioning,
};
use feagi_genome_definitions::::CoreCorticalType;
use feagi_genome_definitions::::CorticalID;
use feagi_genome_definitions::::IOCorticalAreaConfigurationFlag;
use feagi_genome_definitions::::{
    CorticalArea, CorticalAreaDimensions, CorticalAreaType,
};
use feagi_structures::genomic::{MotorCorticalUnit, SensoryCorticalUnit};
// Note: decode_cortical_id removed - use feagi_structures::CorticalID directly
use parking_lot::RwLock;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{debug, info, trace, warn};
use feagi_genome_definitions::::brain_region::BrainRegion;
use feagi_genome_definitions::::region_type::RegionType;

fn derive_friendly_cortical_name(cortical_id: &CorticalID) -> Option<String> {
    let bytes = cortical_id.as_bytes();
    let is_input = bytes[0] == b'i';
    let is_output = bytes[0] == b'o';
    if !is_input && !is_output {
        return None;
    }

    let unit_ref: [u8; 3] = [bytes[1], bytes[2], bytes[3]];
    let subunit_index = bytes[6];
    let unit_index = bytes[7];

    if is_input {
        for unit in SensoryCorticalUnit::list_all() {
            if unit.get_cortical_id_unit_reference() == unit_ref {
                let unit_name = unit.get_friendly_name();
                let has_subunits = unit.get_number_cortical_areas() > 1;
                let name = if has_subunits {
                    format!(
                        "{} Subunit {} Unit {}",
                        unit_name, subunit_index, unit_index
                    )
                } else {
                    format!("{} Unit {}", unit_name, unit_index)
                };
                return Some(name);
            }
        }
    } else {
        for unit in MotorCorticalUnit::list_all() {
            if unit.get_cortical_id_unit_reference() == unit_ref {
                let unit_name = unit.get_friendly_name();
                let has_subunits = unit.get_number_cortical_areas() > 1;
                let name = if matches!(unit, MotorCorticalUnit::Gaze) {
                    let subunit_name = match subunit_index {
                        0 => "Eccentricity",
                        1 => "Modulation",
                        _ => "Subunit",
                    };
                    format!("{} ({}) Unit {}", unit_name, subunit_name, unit_index)
                } else if has_subunits {
                    format!(
                        "{} Subunit {} Unit {}",
                        unit_name, subunit_index, unit_index
                    )
                } else {
                    format!("{} Unit {}", unit_name, unit_index)
                };
                return Some(name);
            }
        }
    }

    None
}

/// Merge default template and memory properties into provided values.
/// Existing values always override defaults.
fn merge_memory_area_properties(
    base: HashMap<String, Value>,
    extra: Option<&HashMap<String, Value>>,
) -> HashMap<String, Value> {
    let mut defaults = get_default_neural_properties();
    let memory_defaults = MemoryAreaProperties::default();
    defaults
        .entry("cortical_group".to_string())
        .or_insert(Value::from("MEMORY"));
    defaults
        .entry("is_mem_type".to_string())
        .or_insert(Value::from(true));
    defaults
        .entry("temporal_depth".to_string())
        .or_insert(Value::from(memory_defaults.temporal_depth));
    defaults
        .entry("longterm_mem_threshold".to_string())
        .or_insert(Value::from(memory_defaults.longterm_threshold));
    defaults
        .entry("lifespan_growth_rate".to_string())
        .or_insert(Value::from(memory_defaults.lifespan_growth_rate));
    defaults
        .entry("init_lifespan".to_string())
        .or_insert(Value::from(memory_defaults.init_lifespan));
    defaults
        .entry("psp_uniform_distribution".to_string())
        .or_insert(Value::from(true));

    defaults.extend(base);
    if let Some(extra_props) = extra {
        defaults.extend(extra_props.clone());
    }
    defaults
}

fn frame_handling_label(frame: FrameChangeHandling) -> &'static str {
    match frame {
        FrameChangeHandling::Absolute => "Absolute",
        FrameChangeHandling::Incremental => "Incremental",
    }
}

fn positioning_label(positioning: PercentageNeuronPositioning) -> &'static str {
    match positioning {
        PercentageNeuronPositioning::Linear => "Linear",
        PercentageNeuronPositioning::Fractional => "Fractional",
    }
}

fn signage_label_from_flag(flag: &IOCorticalAreaConfigurationFlag) -> &'static str {
    match flag {
        IOCorticalAreaConfigurationFlag::SignedPercentage(..)
        | IOCorticalAreaConfigurationFlag::SignedPercentage2D(..)
        | IOCorticalAreaConfigurationFlag::SignedPercentage3D(..)
        | IOCorticalAreaConfigurationFlag::SignedPercentage4D(..) => "Percentage Signed",
        IOCorticalAreaConfigurationFlag::Percentage(..)
        | IOCorticalAreaConfigurationFlag::Percentage2D(..)
        | IOCorticalAreaConfigurationFlag::Percentage3D(..)
        | IOCorticalAreaConfigurationFlag::Percentage4D(..) => "Percentage Unsigned",
        IOCorticalAreaConfigurationFlag::CartesianPlane(..) => "Cartesian Plane",
        IOCorticalAreaConfigurationFlag::Misc(..) => "Misc",
        IOCorticalAreaConfigurationFlag::PoseEstimation(..) => "Pose Estimation",
        IOCorticalAreaConfigurationFlag::Boolean => "Boolean",
    }
}

fn behavior_label_from_flag(flag: &IOCorticalAreaConfigurationFlag) -> &'static str {
    match flag {
        IOCorticalAreaConfigurationFlag::Boolean => "Not Applicable",
        IOCorticalAreaConfigurationFlag::CartesianPlane(frame)
        | IOCorticalAreaConfigurationFlag::Misc(frame)
        | IOCorticalAreaConfigurationFlag::PoseEstimation(frame, _)
        | IOCorticalAreaConfigurationFlag::Percentage(frame, _)
        | IOCorticalAreaConfigurationFlag::Percentage2D(frame, _)
        | IOCorticalAreaConfigurationFlag::Percentage3D(frame, _)
        | IOCorticalAreaConfigurationFlag::Percentage4D(frame, _)
        | IOCorticalAreaConfigurationFlag::SignedPercentage(frame, _)
        | IOCorticalAreaConfigurationFlag::SignedPercentage2D(frame, _)
        | IOCorticalAreaConfigurationFlag::SignedPercentage3D(frame, _)
        | IOCorticalAreaConfigurationFlag::SignedPercentage4D(frame, _) => {
            frame_handling_label(*frame)
        }
    }
}

fn resolve_non_overlapping_position(
    requested_position: (i32, i32, i32),
    area_width: usize,
    occupied_positions: &mut HashSet<(i32, i32, i32)>,
) -> ServiceResult<(i32, i32, i32)> {
    if !occupied_positions.contains(&requested_position) {
        occupied_positions.insert(requested_position);
        return Ok(requested_position);
    }

    let width_for_gap = area_width.max(1);
    let gap = width_for_gap.div_ceil(5).max(1); // ceil(20% of width)
    let step_usize = width_for_gap.saturating_add(gap);
    let step = i32::try_from(step_usize).map_err(|_| {
        ServiceError::InvalidInput(format!(
            "Unable to place cortical area: width {} creates horizontal step {} outside i32 range",
            area_width, step_usize
        ))
    })?;

    let mut candidate = requested_position;
    while occupied_positions.contains(&candidate) {
        candidate.0 = candidate.0.checked_add(step).ok_or_else(|| {
            ServiceError::InvalidInput(format!(
                "Unable to place cortical area: overflow while shifting x from {} by {}",
                candidate.0, step
            ))
        })?;
    }
    occupied_positions.insert(candidate);
    Ok(candidate)
}

fn coding_type_label_from_flag(flag: &IOCorticalAreaConfigurationFlag) -> &'static str {
    match flag {
        IOCorticalAreaConfigurationFlag::Percentage(_, positioning)
        | IOCorticalAreaConfigurationFlag::Percentage2D(_, positioning)
        | IOCorticalAreaConfigurationFlag::Percentage3D(_, positioning)
        | IOCorticalAreaConfigurationFlag::Percentage4D(_, positioning)
        | IOCorticalAreaConfigurationFlag::SignedPercentage(_, positioning)
        | IOCorticalAreaConfigurationFlag::SignedPercentage2D(_, positioning)
        | IOCorticalAreaConfigurationFlag::SignedPercentage3D(_, positioning)
        | IOCorticalAreaConfigurationFlag::SignedPercentage4D(_, positioning) => {
            positioning_label(*positioning)
        }
        IOCorticalAreaConfigurationFlag::CartesianPlane(_)
        | IOCorticalAreaConfigurationFlag::Misc(_)
        | IOCorticalAreaConfigurationFlag::PoseEstimation(..)
        | IOCorticalAreaConfigurationFlag::Boolean => "Not Applicable",
    }
}

fn io_unit_reference_from_cortical_id(cortical_id: &CorticalID) -> Option<[u8; 3]> {
    let bytes = cortical_id.as_bytes();
    if bytes[0] != b'i' && bytes[0] != b'o' {
        return None;
    }
    Some([bytes[1], bytes[2], bytes[3]])
}

fn io_coding_options_for_unit(cortical_id: &CorticalID) -> Option<IOCodingOptions> {
    let unit_ref = io_unit_reference_from_cortical_id(cortical_id)?;
    let is_input = cortical_id.as_bytes()[0] == b'i';

    let (accepted_type, allowed_frames) = if is_input {
        let unit = SensoryCorticalUnit::list_all()
            .iter()
            .find(|u| u.get_cortical_id_unit_reference() == unit_ref)?;
        (
            unit.get_accepted_wrapped_io_data_type(),
            unit.get_allowed_frame_change_handling(),
        )
    } else {
        let unit = MotorCorticalUnit::list_all()
            .iter()
            .find(|u| u.get_cortical_id_unit_reference() == unit_ref)?;
        (
            unit.get_accepted_wrapped_io_data_type(),
            unit.get_allowed_frame_change_handling(),
        )
    };

    let mut signage_options = Vec::new();
    let mut behavior_options = Vec::new();
    let mut coding_type_options = Vec::new();

    let io_flag = match cortical_id.extract_io_data_flag() {
        Ok(flag) => flag,
        Err(err) => {
            warn!(
                target: "feagi-services",
                "[IO-CODING] {} failed to extract io_flag: {} (accepted_type={})",
                cortical_id,
                err,
                accepted_type
            );
            return None;
        }
    };
    signage_options.push(signage_label_from_flag(&io_flag).to_string());

    let supports_frame_handling = !matches!(io_flag, IOCorticalAreaConfigurationFlag::Boolean);
    if supports_frame_handling {
        if let Some(frames) = allowed_frames {
            for frame in frames {
                behavior_options.push(frame_handling_label(*frame).to_string());
            }
        } else {
            behavior_options.push("Absolute".to_string());
            behavior_options.push("Incremental".to_string());
        }
    } else {
        behavior_options.push("Not Applicable".to_string());
    }

    let supports_positioning = matches!(
        io_flag,
        IOCorticalAreaConfigurationFlag::Percentage(..)
            | IOCorticalAreaConfigurationFlag::Percentage2D(..)
            | IOCorticalAreaConfigurationFlag::Percentage3D(..)
            | IOCorticalAreaConfigurationFlag::Percentage4D(..)
            | IOCorticalAreaConfigurationFlag::SignedPercentage(..)
            | IOCorticalAreaConfigurationFlag::SignedPercentage2D(..)
            | IOCorticalAreaConfigurationFlag::SignedPercentage3D(..)
            | IOCorticalAreaConfigurationFlag::SignedPercentage4D(..)
    );
    if supports_positioning {
        coding_type_options.push("Linear".to_string());
        coding_type_options.push("Fractional".to_string());
    } else {
        coding_type_options.push("Not Applicable".to_string());
    }

    if signage_options.is_empty() {
        warn!(
            target: "feagi-services",
            "[IO-CODING] {} empty signage_options (accepted_type={}, io_flag={:?})",
            cortical_id,
            accepted_type,
            io_flag
        );
    }
    Some(IOCodingOptions {
        signage_options,
        behavior_options,
        coding_type_options,
    })
}

/// Update a cortical area's `cortical_mapping_dst` property in-place.
///
/// - When `mapping_data` is empty: remove the destination entry, and if the
///   container becomes empty remove `cortical_mapping_dst` entirely.
/// - When `mapping_data` is non-empty: insert/overwrite the destination entry.
fn update_cortical_mapping_dst_in_properties(
    properties: &mut HashMap<String, serde_json::Value>,
    dst_area_id: &str,
    mapping_data: &[serde_json::Value],
) -> ServiceResult<()> {
    if mapping_data.is_empty() {
        let Some(existing) = properties.get_mut("cortical_mapping_dst") else {
            return Ok(());
        };
        let Some(mapping_dst) = existing.as_object_mut() else {
            return Err(ServiceError::Backend(
                "cortical_mapping_dst is not a JSON object".to_string(),
            ));
        };

        mapping_dst.remove(dst_area_id);
        if mapping_dst.is_empty() {
            properties.remove("cortical_mapping_dst");
        }
        return Ok(());
    }

    let entry = properties
        .entry("cortical_mapping_dst".to_string())
        .or_insert_with(|| serde_json::json!({}));

    let Some(mapping_dst) = entry.as_object_mut() else {
        return Err(ServiceError::Backend(
            "cortical_mapping_dst is not a JSON object".to_string(),
        ));
    };

    mapping_dst.insert(dst_area_id.to_string(), serde_json::json!(mapping_data));
    Ok(())
}

fn get_cortical_mapping_dst_from_properties(
    properties: &HashMap<String, serde_json::Value>,
    dst_area_id: &str,
) -> Option<Vec<serde_json::Value>> {
    properties
        .get("cortical_mapping_dst")
        .and_then(|value| value.as_object())
        .and_then(|mapping_dst| mapping_dst.get(dst_area_id))
        .and_then(|value| value.as_array())
        .map(|rules| rules.to_vec())
}

fn mapping_rule_uses_morphology(rule: &serde_json::Value, morphology_id: &str) -> bool {
    if let Some(arr) = rule.as_array() {
        return arr.first().and_then(|v| v.as_str()) == Some(morphology_id);
    }
    if let Some(obj) = rule.as_object() {
        return obj.get("morphology_id").and_then(|v| v.as_str()) == Some(morphology_id);
    }
    false
}

fn collect_morphology_usage_pairs(
    genome: &feagi_evolutionary::RuntimeGenome,
    morphology_id: &str,
) -> Vec<(String, String)> {
    let mut usage_pairs: HashSet<(String, String)> = HashSet::new();

    for (src_id, area) in &genome.cortical_areas {
        let Some(mapping_dst) = area
            .properties
            .get("cortical_mapping_dst")
            .and_then(|value| value.as_object())
        else {
            continue;
        };

        for (dst_id, rules_value) in mapping_dst {
            let Some(rule_array) = rules_value.as_array() else {
                continue;
            };

            if rule_array
                .iter()
                .any(|rule| mapping_rule_uses_morphology(rule, morphology_id))
            {
                usage_pairs.insert((src_id.as_base_64(), dst_id.clone()));
            }
        }
    }

    let mut ordered_pairs: Vec<(String, String)> = usage_pairs.into_iter().collect();
    ordered_pairs.sort();
    ordered_pairs
}

fn collect_morphology_usage_pairs_from_area_infos(
    area_infos: &[CorticalAreaInfo],
    morphology_id: &str,
) -> Vec<(String, String)> {
    let mut usage_pairs: HashSet<(String, String)> = HashSet::new();

    for area_info in area_infos {
        let Some(mapping_dst) = area_info
            .properties
            .get("cortical_mapping_dst")
            .and_then(|value| value.as_object())
        else {
            continue;
        };

        for (dst_id, rules_value) in mapping_dst {
            let Some(rule_array) = rules_value.as_array() else {
                continue;
            };

            if rule_array
                .iter()
                .any(|rule| mapping_rule_uses_morphology(rule, morphology_id))
            {
                usage_pairs.insert((area_info.cortical_id.clone(), dst_id.clone()));
            }
        }
    }

    let mut ordered_pairs: Vec<(String, String)> = usage_pairs.into_iter().collect();
    ordered_pairs.sort();
    ordered_pairs
}

fn parse_cortical_id_flexible(raw_id: &str) -> Result<CorticalID, String> {
    if let Ok(parsed) = CorticalID::try_from_base_64(raw_id) {
        return Ok(parsed);
    }
    if let Ok(parsed) = CorticalID::try_from_legacy_ascii(raw_id) {
        return Ok(parsed);
    }
    Err(format!(
        "Unable to parse cortical ID '{}' as base64 or legacy ASCII format",
        raw_id
    ))
}

/// Recursively replace morphology_id in JSON values.
///
/// Handles object format `{"morphology_id": "x", ...}` and array format
/// `["morphology_id", scalar, ...]` where morphology_id is the first element.
fn replace_morphology_id_in_value(
    value: &mut Value,
    old_id: &str,
    new_id: &str,
    replaced_count: &mut usize,
) {
    match value {
        Value::Object(obj) => {
            if let Some(morphology_id) = obj.get_mut("morphology_id") {
                if morphology_id.as_str() == Some(old_id) {
                    *morphology_id = Value::String(new_id.to_string());
                    *replaced_count += 1;
                }
            }
            for child in obj.values_mut() {
                replace_morphology_id_in_value(child, old_id, new_id, replaced_count);
            }
        }
        Value::Array(arr) => {
            if let Some(first) = arr.first_mut() {
                if first.as_str() == Some(old_id) {
                    *first = Value::String(new_id.to_string());
                    *replaced_count += 1;
                }
            }
            for child in arr.iter_mut() {
                replace_morphology_id_in_value(child, old_id, new_id, replaced_count);
            }
        }
        _ => {}
    }
}

/// Default implementation of ConnectomeService
pub struct ConnectomeServiceImpl {
    connectome: Arc<RwLock<ConnectomeManager>>,
    /// Currently loaded genome (source of truth for genome persistence)
    /// Shared with GenomeServiceImpl to ensure cortical mappings are saved
    current_genome: Arc<RwLock<Option<feagi_evolutionary::RuntimeGenome>>>,
    /// Optional reference to RuntimeService for accessing NPU (for connectome I/O)
    #[cfg(feature = "connectome-io")]
    runtime_service: Arc<RwLock<Option<Arc<dyn crate::traits::RuntimeService + Send + Sync>>>>,
    /// Optional burst runner for refreshing cortical_id cache
    burst_runner: Option<Arc<RwLock<BurstLoopRunner>>>,
}

impl ConnectomeServiceImpl {
    pub fn new(
        connectome: Arc<RwLock<ConnectomeManager>>,
        current_genome: Arc<RwLock<Option<feagi_evolutionary::RuntimeGenome>>>,
    ) -> Self {
        Self {
            connectome,
            current_genome,
            #[cfg(feature = "connectome-io")]
            runtime_service: Arc::new(RwLock::new(None)),
            burst_runner: None,
        }
    }

    /// Set the burst runner for cache refresh
    pub fn set_burst_runner(&mut self, burst_runner: Arc<RwLock<BurstLoopRunner>>) {
        self.burst_runner = Some(burst_runner);
    }

    /// Refresh cortical_id cache in burst runner
    fn refresh_burst_runner_cache(&self) {
        if let Some(ref burst_runner) = self.burst_runner {
            let manager = self.connectome.read();
            let mappings = manager.get_all_cortical_idx_to_id_mappings();
            let chunk_sizes = manager.get_all_visualization_granularities();
            let mapping_count = mappings.len();
            let burst_runner_write = burst_runner.write();
            burst_runner_write.refresh_cortical_id_mappings(mappings);
            burst_runner_write.refresh_visualization_granularities(chunk_sizes);
            debug!(target: "feagi-services", "Refreshed burst runner cache with {} cortical areas", mapping_count);
        }
    }

    /// Set the runtime service (required for connectome export/import)
    ///
    /// This must be called after creating ConnectomeServiceImpl to enable
    /// connectome I/O operations.
    #[cfg(feature = "connectome-io")]
    pub fn set_runtime_service(
        &self,
        runtime_service: Arc<dyn crate::traits::RuntimeService + Send + Sync>,
    ) {
        *self.runtime_service.write() = Some(runtime_service);
        info!(target: "feagi-services", "RuntimeService connected to ConnectomeService for connectome I/O");
    }

    /// Convert RegionType enum to string
    fn region_type_to_string(region_type: &RegionType) -> String {
        match region_type {
            RegionType::Undefined => "Undefined".to_string(),
        }
    }

    /// Convert string to RegionType enum
    fn string_to_region_type(s: &str) -> Result<RegionType, ServiceError> {
        match s {
            "Undefined" | "Sensory" | "Motor" | "Memory" | "Custom" => Ok(RegionType::Undefined),
            _ => Err(ServiceError::InvalidInput(format!(
                "Invalid region type: {}",
                s
            ))),
        }
    }
}

#[async_trait]
impl ConnectomeService for ConnectomeServiceImpl {
    // ========================================================================
    // CORTICAL AREA OPERATIONS
    // ========================================================================

    async fn create_cortical_area(
        &self,
        params: CreateCorticalAreaParams,
    ) -> ServiceResult<CorticalAreaInfo> {
        info!(target: "feagi-services","Creating cortical area: {}", params.cortical_id);

        // Convert String to CorticalID
        let cortical_id_typed = CorticalID::try_from_base_64(&params.cortical_id)
            .map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?;

        // Get cortical area type from the cortical ID
        let area_type = cortical_id_typed.as_cortical_type().map_err(|e| {
            ServiceError::InvalidInput(format!("Failed to determine cortical area type: {}", e))
        })?;

        let mut occupied_positions: HashSet<(i32, i32, i32)> = {
            let manager = self.connectome.read();
            manager
                .get_cortical_area_ids()
                .iter()
                .filter_map(|id| manager.get_cortical_area(id))
                .map(|area| (area.position.x, area.position.y, area.position.z))
                .collect()
        };
        let requested_position = params.position;
        let resolved_position = resolve_non_overlapping_position(
            requested_position,
            params.dimensions.0,
            &mut occupied_positions,
        )?;
        if resolved_position != requested_position {
            info!(
                target: "feagi-services",
                "Adjusted cortical area position to avoid overlap: id={} requested=({},{},{}) resolved=({},{},{}) width={} gap_rule=20pct",
                params.cortical_id,
                requested_position.0,
                requested_position.1,
                requested_position.2,
                resolved_position.0,
                resolved_position.1,
                resolved_position.2,
                params.dimensions.0
            );
        }

        // Create CorticalArea
        let mut area = CorticalArea::new(
            cortical_id_typed,
            0, // Auto-assigned by ConnectomeManager
            params.name.clone(),
            CorticalAreaDimensions::new(
                params.dimensions.0 as u32,
                params.dimensions.1 as u32,
                params.dimensions.2 as u32,
            )?,
            resolved_position.into(), // Convert (i32, i32, i32) to GenomeCoordinate3D
            area_type,
        )?;

        // Set the cortical type
        // Note: cortical_type_new field removed - type is encoded in CorticalID

        // Apply all neural parameters from params
        if let Some(visible) = params.visible {
            area.add_property_mut("visible".to_string(), serde_json::json!(visible));
        }
        if let Some(sub_group) = params.sub_group {
            area.add_property_mut("sub_group".to_string(), serde_json::json!(sub_group));
        }
        if let Some(neurons_per_voxel) = params.neurons_per_voxel {
            area.add_property_mut(
                "neurons_per_voxel".to_string(),
                serde_json::json!(neurons_per_voxel),
            );
        }
        if let Some(postsynaptic_current) = params.postsynaptic_current {
            area.add_property_mut(
                "postsynaptic_current".to_string(),
                serde_json::json!(postsynaptic_current),
            );
        }
        if let Some(plasticity_constant) = params.plasticity_constant {
            area.add_property_mut(
                "plasticity_constant".to_string(),
                serde_json::json!(plasticity_constant),
            );
        }
        if let Some(degeneration) = params.degeneration {
            area.add_property_mut("degeneration".to_string(), serde_json::json!(degeneration));
        }
        if let Some(psp_uniform_distribution) = params.psp_uniform_distribution {
            area.add_property_mut(
                "psp_uniform_distribution".to_string(),
                serde_json::json!(psp_uniform_distribution),
            );
        }
        if let Some(firing_threshold_increment) = params.firing_threshold_increment {
            area.add_property_mut(
                "firing_threshold_increment".to_string(),
                serde_json::json!(firing_threshold_increment),
            );
        }
        if let Some(firing_threshold_limit) = params.firing_threshold_limit {
            area.add_property_mut(
                "firing_threshold_limit".to_string(),
                serde_json::json!(firing_threshold_limit),
            );
        }
        if let Some(consecutive_fire_count) = params.consecutive_fire_count {
            area.add_property_mut(
                "consecutive_fire_limit".to_string(),
                serde_json::json!(consecutive_fire_count),
            );
        }
        if let Some(snooze_period) = params.snooze_period {
            area.add_property_mut(
                "snooze_period".to_string(),
                serde_json::json!(snooze_period),
            );
        }
        if let Some(refractory_period) = params.refractory_period {
            area.add_property_mut(
                "refractory_period".to_string(),
                serde_json::json!(refractory_period),
            );
        }
        if let Some(leak_coefficient) = params.leak_coefficient {
            area.add_property_mut(
                "leak_coefficient".to_string(),
                serde_json::json!(leak_coefficient),
            );
        }
        if let Some(leak_variability) = params.leak_variability {
            area.add_property_mut(
                "leak_variability".to_string(),
                serde_json::json!(leak_variability),
            );
        }
        if let Some(burst_engine_active) = params.burst_engine_active {
            area.add_property_mut(
                "burst_engine_active".to_string(),
                serde_json::json!(burst_engine_active),
            );
        }

        // Extract parent_region_id before moving properties
        let parent_region_id = params
            .properties
            .as_ref()
            .and_then(|props| props.get("parent_region_id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let is_memory_area = matches!(area_type, CorticalAreaType::Memory(_));
        if is_memory_area {
            let merged =
                merge_memory_area_properties(area.properties.clone(), params.properties.as_ref());
            area.properties = merged;
        } else if let Some(properties) = params.properties {
            area.properties = properties;
        }

        // Add to connectome
        self.connectome
            .write()
            .add_cortical_area(area)
            .map_err(ServiceError::from)?;

        // Refresh burst runner cache after creating area
        self.refresh_burst_runner_cache();

        // CRITICAL: If parent_region_id is specified, add this cortical area
        // to the parent brain region's cortical_areas set so it persists in genome
        if let Some(region_id) = parent_region_id {
            let mut manager = self.connectome.write();
            if let Some(region) = manager.get_brain_region_mut(&region_id) {
                region.add_area(cortical_id_typed);
                info!(target: "feagi-services",
                    "Added cortical area {} to parent region {}",
                    params.cortical_id, region_id
                );
            } else {
                warn!(target: "feagi-services",
                    "Parent region {} not found for cortical area {}",
                    region_id, params.cortical_id
                );
            }
        }

        // Return info
        self.get_cortical_area(&params.cortical_id).await
    }

    async fn delete_cortical_area(&self, cortical_id: &str) -> ServiceResult<()> {
        info!(target: "feagi-services","Deleting cortical area: {}", cortical_id);

        // Convert String to CorticalID
        let cortical_id_typed = CorticalID::try_from_base_64(cortical_id)
            .map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?;
        let deleted_id_base64 = cortical_id_typed.as_base_64();
        let deleted_cortical_idx = {
            let manager = self.connectome.read();
            manager.get_cortical_idx(&cortical_id_typed)
        };
        let mut removed_mapping_count = 0usize;
        let mut removed_upstream_count = 0usize;

        // Remove from the live connectome, and also scrub from brain-region membership
        // so UI + region-based operations don't keep referencing a deleted area.
        //
        // Note: ConnectomeManager::remove_cortical_area currently does NOT remove the
        // ID from brain regions, so we do it explicitly here.
        let mut pruned_outgoing_synapses = 0usize;
        let mut pruned_incoming_synapses = 0usize;
        let region_io = {
            let mut manager = self.connectome.write();
            let region_ids: Vec<String> = manager
                .get_brain_region_ids()
                .into_iter()
                .cloned()
                .collect();
            for region_id in region_ids {
                if let Some(region) = manager.get_brain_region_mut(&region_id) {
                    region.remove_area(&cortical_id_typed);
                }
            }

            // CRITICAL: Cascade synapse cleanup BEFORE removing the cortical area itself.
            //
            // `ConnectomeManager::remove_cortical_area` is a thin dictionary delete: it removes
            // the area entry and lookup maps but does NOT prune synapses. If we let it run first,
            // every src→deleted and deleted→dst synapse becomes orphaned in the NPU, retaining
            // (potentially saturated) learned R-STDP weights and consuming compute every burst.
            //
            // The canonical pruning path is `update_cortical_mapping(src, dst, [])` followed by
            // `regenerate_synapses_for_mapping(src, dst)` — the latter requires both endpoints to
            // still be resolvable in `cortical_id_to_idx`. We therefore clear all mapping rules
            // (incoming AND outgoing relative to the doomed area) and prune their synapses first,
            // then remove the area as the final step.
            let cortical_ids: Vec<CorticalID> = manager
                .get_cortical_area_ids()
                .into_iter()
                .cloned()
                .collect();

            // 1) Incoming side: src → deleted_id. For every other area mapping to the doomed area,
            //    clear rules and prune the resulting orphan synapses out of the NPU.
            for src_id in &cortical_ids {
                if src_id == &cortical_id_typed {
                    continue;
                }
                let has_mapping = manager
                    .get_cortical_area(src_id)
                    .and_then(|area| area.properties.get("cortical_mapping_dst"))
                    .and_then(|value| value.as_object())
                    .map(|mapping| mapping.contains_key(&deleted_id_base64))
                    .unwrap_or(false);
                if has_mapping {
                    manager
                        .update_cortical_mapping(src_id, &cortical_id_typed, Vec::new())
                        .map_err(ServiceError::from)?;
                    let pruned = manager
                        .regenerate_synapses_for_mapping(src_id, &cortical_id_typed)
                        .map_err(|e| {
                            ServiceError::Backend(format!(
                                "Failed to prune synapses for {} -> {}: {}",
                                src_id, cortical_id_typed, e
                            ))
                        })?;
                    pruned_incoming_synapses = pruned_incoming_synapses.saturating_add(pruned);
                    removed_mapping_count += 1;
                }
            }

            // 2) Outgoing side: deleted_id → dst. Same treatment so the doomed area leaves no
            //    orphan synapses pointing into still-live targets.
            let outgoing_targets: Vec<CorticalID> = manager
                .get_cortical_area(&cortical_id_typed)
                .and_then(|area| area.properties.get("cortical_mapping_dst"))
                .and_then(|value| value.as_object())
                .map(|mapping| {
                    mapping
                        .keys()
                        .filter_map(|k| CorticalID::try_from_base_64(k).ok())
                        .collect()
                })
                .unwrap_or_default();
            for dst_id in outgoing_targets {
                manager
                    .update_cortical_mapping(&cortical_id_typed, &dst_id, Vec::new())
                    .map_err(ServiceError::from)?;
                let pruned = manager
                    .regenerate_synapses_for_mapping(&cortical_id_typed, &dst_id)
                    .map_err(|e| {
                        ServiceError::Backend(format!(
                            "Failed to prune synapses for {} -> {}: {}",
                            cortical_id_typed, dst_id, e
                        ))
                    })?;
                pruned_outgoing_synapses = pruned_outgoing_synapses.saturating_add(pruned);
                removed_mapping_count += 1;
            }

            // 3) Upstream-cortical-areas property cleanup on every remaining area. Done while
            //    the deleted_idx is still valid (post-removal we'd have to scan numerically).
            for src_id in &cortical_ids {
                if src_id == &cortical_id_typed {
                    continue;
                }
                if let Some(deleted_idx) = deleted_cortical_idx {
                    if let Some(area) = manager.get_cortical_area_mut(src_id) {
                        if let Some(upstream) = area
                            .properties
                            .get_mut("upstream_cortical_areas")
                            .and_then(|value| value.as_array_mut())
                        {
                            let before = upstream.len();
                            upstream.retain(|value| {
                                value
                                    .as_u64()
                                    .map(|id| id != deleted_idx as u64)
                                    .unwrap_or(true)
                            });
                            if upstream.len() != before {
                                removed_upstream_count += before - upstream.len();
                            }
                        }
                    }
                }
            }

            // 4) Finally remove the cortical area itself, after all synapses involving it have
            //    been pruned. Anything that remains is purely connectome bookkeeping.
            manager
                .remove_cortical_area(&cortical_id_typed)
                .map_err(ServiceError::from)?;

            Some(manager.recompute_brain_region_io_registry().map_err(|e| {
                ServiceError::Backend(format!("Failed to recompute region IO registry: {}", e))
            })?)
        };

        // CRITICAL: Persist deletion into RuntimeGenome (source of truth for save/export).
        if let Some(genome) = self.current_genome.write().as_mut() {
            let removed = genome.cortical_areas.remove(&cortical_id_typed).is_some();
            for region in genome.brain_regions.values_mut() {
                region.remove_area(&cortical_id_typed);
            }
            for area in genome.cortical_areas.values_mut() {
                update_cortical_mapping_dst_in_properties(
                    &mut area.properties,
                    &deleted_id_base64,
                    &[],
                )?;
                if let Some(deleted_idx) = deleted_cortical_idx {
                    if let Some(upstream) = area
                        .properties
                        .get_mut("upstream_cortical_areas")
                        .and_then(|value| value.as_array_mut())
                    {
                        upstream.retain(|value| {
                            value
                                .as_u64()
                                .map(|id| id != deleted_idx as u64)
                                .unwrap_or(true)
                        });
                    }
                }
            }
            if let Some(region_io) = region_io {
                for (region_id, (inputs, outputs)) in region_io {
                    if let Some(region) = genome.brain_regions.get_mut(&region_id) {
                        if inputs.is_empty() {
                            region.properties.remove("inputs");
                        } else {
                            region
                                .properties
                                .insert("inputs".to_string(), serde_json::json!(inputs));
                        }

                        if outputs.is_empty() {
                            region.properties.remove("outputs");
                        } else {
                            region
                                .properties
                                .insert("outputs".to_string(), serde_json::json!(outputs));
                        }
                    } else {
                        warn!(
                            target: "feagi-services",
                            "Region '{}' not found in RuntimeGenome while persisting IO registry",
                            region_id
                        );
                    }
                }
            }

            if removed {
                info!(
                    target: "feagi-services",
                    "[GENOME-UPDATE] Removed cortical area {} from RuntimeGenome",
                    cortical_id
                );
            } else {
                warn!(
                    target: "feagi-services",
                    "[GENOME-UPDATE] Cortical area {} not found in RuntimeGenome - deletion will not persist to saved genome",
                    cortical_id
                );
            }
        } else {
            warn!(
                target: "feagi-services",
                "[GENOME-UPDATE] No RuntimeGenome loaded - deletion will not persist to saved genome"
            );
        }
        if removed_mapping_count > 0
            || removed_upstream_count > 0
            || pruned_incoming_synapses > 0
            || pruned_outgoing_synapses > 0
        {
            info!(
                target: "feagi-services",
                "Deleted area cleanup: {} mapping references removed, {} upstream references pruned, \
                 {} incoming synapses pruned, {} outgoing synapses pruned",
                removed_mapping_count,
                removed_upstream_count,
                pruned_incoming_synapses,
                pruned_outgoing_synapses
            );
        }

        // Refresh burst runner cache after deleting area
        self.refresh_burst_runner_cache();

        Ok(())
    }

    async fn update_cortical_area(
        &self,
        cortical_id: &str,
        _params: UpdateCorticalAreaParams,
    ) -> ServiceResult<CorticalAreaInfo> {
        info!(target: "feagi-services","Updating cortical area: {}", cortical_id);

        // TODO: This should be routed through GenomeService for proper genome update
        // and change classification (PARAMETER vs STRUCTURAL vs METADATA)
        // Currently this is a stub - needs architecture alignment with Python implementation

        Err(ServiceError::NotImplemented(
            "Cortical area updates must go through GenomeService for proper genome synchronization"
                .to_string(),
        ))
    }

    async fn get_cortical_area(&self, cortical_id: &str) -> ServiceResult<CorticalAreaInfo> {
        trace!(target: "feagi-services", "Getting cortical area: {}", cortical_id);

        // Accept base64 or legacy ASCII (clients may send either form).
        let cortical_id_typed =
            parse_cortical_id_flexible(cortical_id).map_err(ServiceError::InvalidInput)?;

        let manager = self.connectome.read();

        let area = manager
            .get_cortical_area(&cortical_id_typed)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "CorticalArea".to_string(),
                id: cortical_id.to_string(),
            })?;

        let cortical_idx = manager
            .get_cortical_idx(&cortical_id_typed)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "CorticalArea".to_string(),
                id: cortical_id.to_string(),
            })?;

        let neuron_count = manager.get_neuron_count_in_area(&cortical_id_typed);
        let outgoing_synapse_count = manager.get_outgoing_synapse_count_in_area(&cortical_id_typed);
        let incoming_synapse_count = manager.get_incoming_synapse_count_in_area(&cortical_id_typed);
        let synapse_count = outgoing_synapse_count;

        // Get cortical_group from the area (uses cortical_type_new if available)
        let cortical_group = area.get_cortical_group();

        // Note: decode_cortical_id removed - IPU/OPU metadata now in CorticalID
        let memory_props = {
            use feagi_evolutionary::extract_memory_properties;
            extract_memory_properties(&area.properties)
        };

        let cortical_bytes = cortical_id_typed.as_bytes();
        let is_io_area = cortical_bytes[0] == b'i' || cortical_bytes[0] == b'o';
        let io_flag = if is_io_area {
            cortical_id_typed
                .extract_io_data_flag()
                .ok()
                .or(match &area.cortical_type {
                    CorticalAreaType::BrainInput(flag) | CorticalAreaType::BrainOutput(flag) => {
                        Some(*flag)
                    }
                    _ => None,
                })
        } else {
            None
        };
        let cortical_subtype = if is_io_area {
            String::from_utf8(cortical_bytes[0..4].to_vec()).ok()
        } else {
            None
        };
        // Byte 6 = CorticalSubUnitIndex, byte 7 = CorticalUnitIndex (see feagi-structures
        // genomic cortical ID layout). BV and motor decoders use byte 7 for device group.
        let subunit_id = if is_io_area {
            Some(cortical_bytes[6])
        } else {
            None
        };
        let cortical_unit_index = if is_io_area {
            Some(cortical_bytes[7])
        } else {
            None
        };
        let coding_signage = io_flag
            .as_ref()
            .map(|flag| signage_label_from_flag(flag).to_string());
        let coding_behavior = io_flag
            .as_ref()
            .map(|flag| behavior_label_from_flag(flag).to_string());
        let coding_type = io_flag
            .as_ref()
            .map(|flag| coding_type_label_from_flag(flag).to_string());
        let coding_options = if is_io_area {
            io_coding_options_for_unit(&cortical_id_typed)
        } else {
            None
        };
        if is_io_area {
            if let Some(opts) = &coding_options {
                trace!(
                    target: "feagi-services",
                    "[IO-CODING] {} options signage={:?} behavior={:?} type={:?} io_flag={:?}",
                    cortical_id,
                    opts.signage_options,
                    opts.behavior_options,
                    opts.coding_type_options,
                    io_flag
                );
            } else {
                warn!(
                    target: "feagi-services",
                    "[IO-CODING] {} options missing (io_flag={:?})",
                    cortical_id,
                    io_flag
                );
            }
        }

        let name = if area.name.is_empty() || area.name == area.cortical_id.to_string() {
            derive_friendly_cortical_name(&area.cortical_id).unwrap_or_else(|| area.name.clone())
        } else {
            area.name.clone()
        };

        let mut filtered_properties = area.properties.clone();
        let duplicate_keys: HashSet<&str> = [
            "coordinates_3d",
            "cortical_dimensions",
            "cortical_dimensions_per_device",
            "cortical_group",
            "group_id",
            "sub_group_id",
            "neurons_per_voxel",
            "firing_threshold",
            "firing_threshold_increment_x",
            "firing_threshold_increment_y",
            "firing_threshold_increment_z",
            "firing_threshold_limit",
            "consecutive_fire_count",
            "refractory_period",
            "snooze_period",
            "leak_coefficient",
            "leak_variability",
            "mp_charge_accumulation",
            "mp_driven_psp",
            "neuron_excitability",
            "postsynaptic_current",
            "postsynaptic_current_max",
            "degeneration",
            "plasticity_constant",
            "psp_uniform_distribution",
            "init_lifespan",
            "lifespan_growth_rate",
            "longterm_mem_threshold",
            "visible",
        ]
        .into_iter()
        .collect();
        filtered_properties.retain(|key, _| !duplicate_keys.contains(key.as_str()));

        Ok(CorticalAreaInfo {
            cortical_id: cortical_id.to_string(),
            cortical_id_s: area.cortical_id.to_string(), // Human-readable ASCII string
            cortical_idx,
            name,
            dimensions: (
                area.dimensions.width as usize,
                area.dimensions.height as usize,
                area.dimensions.depth as usize,
            ),
            position: area.position.into(), // Convert GenomeCoordinate3D to (i32, i32, i32)
            area_type: cortical_group
                .clone()
                .unwrap_or_else(|| "CUSTOM".to_string()),
            cortical_group: cortical_group
                .clone()
                .unwrap_or_else(|| "CUSTOM".to_string()),
            // Determine cortical_type based on properties
            cortical_type: {
                if memory_props.is_some() {
                    "memory".to_string()
                } else if let Some(group) = &cortical_group {
                    match group.as_str() {
                        "IPU" => "sensory".to_string(),
                        "OPU" => "motor".to_string(),
                        "CORE" => "core".to_string(),
                        "MEMORY" => "memory".to_string(),
                        _ => "custom".to_string(),
                    }
                } else {
                    "custom".to_string()
                }
            },
            neuron_count,
            synapse_count,
            incoming_synapse_count,
            outgoing_synapse_count,
            // All neural parameters come from the actual CorticalArea struct
            visible: area.visible(),
            sub_group: area.sub_group(),
            neurons_per_voxel: area.neurons_per_voxel(),
            postsynaptic_current: area.postsynaptic_current() as f64,
            postsynaptic_current_max: area.postsynaptic_current_max() as f64,
            plasticity_constant: area.plasticity_constant() as f64,
            degeneration: area.degeneration() as f64,
            psp_uniform_distribution: area.psp_uniform_distribution(),
            mp_driven_psp: area.mp_driven_psp(),
            firing_threshold: area.firing_threshold() as f64,
            firing_threshold_increment: [
                area.firing_threshold_increment_x() as f64,
                area.firing_threshold_increment_y() as f64,
                area.firing_threshold_increment_z() as f64,
            ],
            firing_threshold_limit: area.firing_threshold_limit() as f64,
            consecutive_fire_count: area.consecutive_fire_count(),
            snooze_period: area.snooze_period() as u32,
            refractory_period: area.refractory_period() as u32,
            leak_coefficient: area.leak_coefficient() as f64,
            leak_variability: area.leak_variability() as f64,
            mp_charge_accumulation: area.mp_charge_accumulation(),
            neuron_excitability: area.neuron_excitability() as f64,
            burst_engine_active: area.burst_engine_active(),
            init_lifespan: area.init_lifespan(),
            lifespan_growth_rate: area.lifespan_growth_rate() as f64,
            longterm_mem_threshold: area.longterm_mem_threshold(),
            temporal_depth: memory_props.as_ref().map(|p| p.temporal_depth.max(1)),
            mp_learning_enabled: memory_props.as_ref().map(|p| p.mp_learning_enabled),
            properties: filtered_properties,
            // IPU/OPU-specific decoded fields (only populated for IPU/OPU areas)
            cortical_subtype,
            encoding_type: coding_behavior.clone(),
            encoding_format: coding_type.clone(),
            unit_id: cortical_unit_index,
            subunit_id,
            group_id: cortical_unit_index,
            coding_signage,
            coding_behavior,
            coding_type,
            coding_options,
            parent_region_id: manager.get_parent_region_id_for_area(&cortical_id_typed),
            // Extract dev_count and cortical_dimensions_per_device from properties for IPU/OPU
            dev_count: area
                .properties
                .get("dev_count")
                .and_then(|v| v.as_u64().map(|n| n as usize)),
            cortical_dimensions_per_device: {
                // Try to get from properties first
                let from_properties = area
                    .properties
                    .get("cortical_dimensions_per_device")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| {
                        if arr.len() == 3 {
                            Some((
                                arr[0].as_u64()? as usize,
                                arr[1].as_u64()? as usize,
                                arr[2].as_u64()? as usize,
                            ))
                        } else {
                            None
                        }
                    });

                // If not in properties, compute from dimensions and dev_count for IPU/OPU areas
                if from_properties.is_none() {
                    if let Some(dev_count) = area
                        .properties
                        .get("dev_count")
                        .and_then(|v| v.as_u64().map(|n| n as usize))
                    {
                        let total_width = area.dimensions.width as usize;
                        let height = area.dimensions.height as usize;
                        let depth = area.dimensions.depth as usize;
                        total_width
                            .checked_div(dev_count)
                            .map(|w| (w, height, depth))
                    } else {
                        from_properties
                    }
                } else {
                    from_properties
                }
            },
            visualization_voxel_granularity: {
                // Default is 1x1x1 if not in properties (user-driven, not stored)
                // Handle both integer and float JSON values
                area.properties
                    .get("visualization_voxel_granularity")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| {
                        if arr.len() == 3 {
                            let x_opt = arr[0]
                                .as_u64()
                                .or_else(|| arr[0].as_f64().map(|f| f as u64));
                            let y_opt = arr[1]
                                .as_u64()
                                .or_else(|| arr[1].as_f64().map(|f| f as u64));
                            let z_opt = arr[2]
                                .as_u64()
                                .or_else(|| arr[2].as_f64().map(|f| f as u64));
                            if let (Some(x), Some(y), Some(z)) = (x_opt, y_opt, z_opt) {
                                Some((x as u32, y as u32, z as u32))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .or(Some((1, 1, 1))) // Default is 1x1x1
            },
        })
    }

    async fn list_cortical_areas(&self) -> ServiceResult<Vec<CorticalAreaInfo>> {
        trace!(target: "feagi-services", "Listing all cortical areas");

        {
            // Auto-heal legacy loaded genomes that only include the original core set.
            let mut manager = self.connectome.write();
            let has_any_core = [
                CoreCorticalType::Death,
                CoreCorticalType::Power,
                CoreCorticalType::Fatigue,
                CoreCorticalType::Pain,
                CoreCorticalType::Pleasure,
                CoreCorticalType::Fear,
                CoreCorticalType::Hope,
            ]
            .iter()
            .any(|core| manager.cortical_area_exists(&core.to_cortical_id()));
            if has_any_core {
                manager
                    .ensure_core_cortical_areas()
                    .map_err(ServiceError::from)?;
            }
        }

        let cortical_ids: Vec<String> = {
            let manager = self.connectome.read();
            manager
                .get_cortical_area_ids()
                .into_iter()
                .map(|id| id.as_base_64())
                .collect()
        };

        let mut areas = Vec::new();
        for cortical_id in cortical_ids {
            if let Ok(area_info) = self.get_cortical_area(&cortical_id).await {
                areas.push(area_info);
            }
        }

        Ok(areas)
    }

    async fn get_cortical_area_ids(&self) -> ServiceResult<Vec<String>> {
        debug!(target: "feagi-services","Getting cortical area IDs");

        // CRITICAL: Use try_read() instead of read() to avoid blocking forever
        // If write lock is held (e.g., during genome loading), return error instead of hanging
        let ids: Vec<String> = {
            let manager = match self.connectome.try_read() {
                Some(guard) => guard,
                None => {
                    warn!(target: "feagi-services", "⚠️ ConnectomeManager write lock is held - cannot read cortical area IDs");
                    return Err(ServiceError::Backend("ConnectomeManager is currently being modified (e.g., genome loading in progress). Please try again in a moment.".to_string()));
                }
            };

            let area_count = manager.get_cortical_area_count();
            let ids_refs = manager.get_cortical_area_ids();
            info!(target: "feagi-services", "Found {} cortical areas in ConnectomeManager", area_count);
            info!(target: "feagi-services", "Cortical area IDs (references): {:?}", ids_refs.iter().take(10).collect::<Vec<_>>());
            ids_refs.into_iter().map(|id| id.as_base_64()).collect()
        }; // Lock dropped here
        info!(target: "feagi-services", "Returning {} cortical area IDs: {:?}", ids.len(), ids.iter().take(10).collect::<Vec<_>>());
        Ok(ids)
    }

    async fn cortical_area_exists(&self, cortical_id: &str) -> ServiceResult<bool> {
        trace!(target: "feagi-services","Checking if cortical area exists: {}", cortical_id);

        // Convert String to CorticalID
        let cortical_id_typed = CorticalID::try_from_base_64(cortical_id)
            .map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?;

        Ok(self.connectome.read().has_cortical_area(&cortical_id_typed))
    }

    async fn get_cortical_area_properties(
        &self,
        cortical_id: &str,
    ) -> ServiceResult<std::collections::HashMap<String, serde_json::Value>> {
        debug!(target: "feagi-services","Getting cortical area properties: {}", cortical_id);

        // Convert String to CorticalID
        let cortical_id_typed = CorticalID::try_from_base_64(cortical_id)
            .map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?;

        let manager = self.connectome.read();
        manager
            .get_cortical_area_properties(&cortical_id_typed)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "CorticalArea".to_string(),
                id: cortical_id.to_string(),
            })
    }

    async fn get_all_cortical_area_properties(
        &self,
    ) -> ServiceResult<Vec<std::collections::HashMap<String, serde_json::Value>>> {
        debug!(target: "feagi-services","Getting all cortical area properties");

        let manager = self.connectome.read();
        Ok(manager.get_all_cortical_area_properties())
    }

    async fn get_neuron_properties(
        &self,
        neuron_id: u64,
    ) -> ServiceResult<HashMap<String, serde_json::Value>> {
        let manager = self.connectome.read();
        manager
            .get_neuron_properties(neuron_id)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "Neuron".to_string(),
                id: neuron_id.to_string(),
            })
    }

    // ========================================================================
    // BRAIN REGION OPERATIONS
    // ========================================================================

    async fn create_brain_region(
        &self,
        params: CreateBrainRegionParams,
    ) -> ServiceResult<BrainRegionInfo> {
        info!(target: "feagi-services","Creating brain region: {}", params.region_id);

        // Convert string to RegionType
        let region_type = Self::string_to_region_type(&params.region_type)?;

        let (areas, child_regions, properties) = {
            let mut properties = params.properties.clone().unwrap_or_default();
            let area_values = properties
                .remove("areas")
                .or_else(|| properties.remove("cortical_areas"))
                .map(|value| {
                    value.as_array().cloned().ok_or_else(|| {
                        ServiceError::InvalidInput(
                            "areas must be an array of cortical area IDs".to_string(),
                        )
                    })
                })
                .transpose()?
                .unwrap_or_default();
            let child_values = properties
                .remove("regions")
                .or_else(|| properties.remove("child_regions"))
                .map(|value| {
                    value.as_array().cloned().ok_or_else(|| {
                        ServiceError::InvalidInput(
                            "regions must be an array of brain region IDs".to_string(),
                        )
                    })
                })
                .transpose()?
                .unwrap_or_default();

            let mut areas: Vec<String> = Vec::new();
            for value in area_values {
                let id = value.as_str().ok_or_else(|| {
                    ServiceError::InvalidInput(
                        "areas must be an array of cortical area IDs".to_string(),
                    )
                })?;
                areas.push(id.to_string());
            }

            let mut child_regions: Vec<String> = Vec::new();
            for value in child_values {
                let id = value.as_str().ok_or_else(|| {
                    ServiceError::InvalidInput(
                        "regions must be an array of brain region IDs".to_string(),
                    )
                })?;
                child_regions.push(id.to_string());
            }

            (areas, child_regions, properties)
        };

        // Create BrainRegion
        let mut region = BrainRegion::new(
            RegionID::from_string(&params.region_id)
                .map_err(|e| ServiceError::InvalidInput(format!("Invalid region ID: {}", e)))?,
            params.name.clone(),
            region_type,
        )
        .map_err(ServiceError::from)?;

        // Apply initial properties (persisted into ConnectomeManager and RuntimeGenome).
        if !properties.is_empty() {
            region = region.with_properties(properties);
        }

        // Add to connectome
        self.connectome
            .write()
            .add_brain_region(region, params.parent_id.clone())
            .map_err(ServiceError::from)?;

        // Persist into RuntimeGenome (source of truth for genome save/export).
        //
        // NOTE: GenomeServiceImpl::create_cortical_areas requires that parent_region_id exists
        // in the RuntimeGenome brain_regions map. Without this, any subsequent cortical-area
        // creation that targets this region will fail.
        if let Some(genome) = self.current_genome.write().as_mut() {
            // Fetch the canonical region instance from ConnectomeManager to ensure any internal
            // normalization is reflected in the persisted copy.
            if let Some(created) = self
                .connectome
                .read()
                .get_brain_region(&params.region_id)
                .cloned()
            {
                genome
                    .brain_regions
                    .insert(params.region_id.clone(), created);
            }
        }

        // Reassign areas and child regions (if provided).
        if !areas.is_empty() || !child_regions.is_empty() {
            let mut manager = self.connectome.write();

            for area_id in &areas {
                let cortical_id =
                    feagi_evolutionary::string_to_cortical_id(area_id).map_err(|e| {
                        ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e))
                    })?;

                if !manager.has_cortical_area(&cortical_id) {
                    return Err(ServiceError::NotFound {
                        resource: "CorticalArea".to_string(),
                        id: area_id.clone(),
                    });
                }

                if let Some(existing_parent) = manager.get_parent_region_id_for_area(&cortical_id) {
                    if let Some(parent_region) = manager.get_brain_region_mut(&existing_parent) {
                        parent_region.remove_area(&cortical_id);
                    }
                }

                let Some(region) = manager.get_brain_region_mut(&params.region_id) else {
                    return Err(ServiceError::NotFound {
                        resource: "BrainRegion".to_string(),
                        id: params.region_id.clone(),
                    });
                };
                region.add_area(cortical_id);

                if let Some(area) = manager.get_cortical_area_mut(&cortical_id) {
                    area.properties.insert(
                        "parent_region_id".to_string(),
                        serde_json::json!(params.region_id),
                    );
                }
            }

            for child_id in &child_regions {
                if child_id == &params.region_id {
                    return Err(ServiceError::InvalidInput(
                        "regions cannot include the new region_id".to_string(),
                    ));
                }

                if manager.get_brain_region(child_id).is_none() {
                    return Err(ServiceError::NotFound {
                        resource: "BrainRegion".to_string(),
                        id: child_id.clone(),
                    });
                }

                manager
                    .change_brain_region_parent(child_id, &params.region_id)
                    .map_err(ServiceError::from)?;

                if let Some(child_region) = manager.get_brain_region_mut(child_id) {
                    child_region.add_property(
                        "parent_region_id".to_string(),
                        serde_json::json!(params.region_id),
                    );
                }
            }
        }

        if !areas.is_empty() || !child_regions.is_empty() {
            if let Some(genome) = self.current_genome.write().as_mut() {
                for area_id in &areas {
                    let Ok(cortical_id) = feagi_evolutionary::string_to_cortical_id(area_id) else {
                        continue;
                    };
                    for region in genome.brain_regions.values_mut() {
                        region.remove_area(&cortical_id);
                    }
                    if let Some(region) = genome.brain_regions.get_mut(&params.region_id) {
                        region.add_area(cortical_id);
                    }
                    if let Some(area) = genome.cortical_areas.get_mut(&cortical_id) {
                        area.properties.insert(
                            "parent_region_id".to_string(),
                            serde_json::json!(params.region_id),
                        );
                    }
                }

                for child_id in &child_regions {
                    if let Some(child_region) = genome.brain_regions.get_mut(child_id) {
                        child_region.add_property(
                            "parent_region_id".to_string(),
                            serde_json::json!(params.region_id),
                        );
                    }
                }
            } else {
                warn!(
                    target: "feagi-services",
                    "[GENOME-UPDATE] No RuntimeGenome loaded - region membership updates will not persist to saved genome"
                );
            }
        }

        // Return info
        self.get_brain_region(&params.region_id).await
    }

    async fn delete_brain_region(&self, region_id: &str) -> ServiceResult<()> {
        info!(target: "feagi-services","Deleting brain region: {}", region_id);

        let (region_ids, cortical_area_ids) = {
            let manager = self.connectome.read();
            if manager.get_brain_region(region_id).is_none() {
                return Err(ServiceError::NotFound {
                    resource: "BrainRegion".to_string(),
                    id: region_id.to_string(),
                });
            }

            let hierarchy = manager.get_brain_region_hierarchy();
            let mut region_ids: Vec<String> = hierarchy
                .get_all_descendants(region_id)
                .into_iter()
                .cloned()
                .collect();
            region_ids.push(region_id.to_string());

            let mut region_ids_with_depth: Vec<(String, usize)> = Vec::new();
            for region_id in &region_ids {
                let mut depth = 0;
                let mut current = region_id.as_str();
                while let Some(parent) = hierarchy.get_parent(current) {
                    depth += 1;
                    current = parent;
                }
                region_ids_with_depth.push((region_id.clone(), depth));
            }
            region_ids_with_depth.sort_by(|(a_id, a_depth), (b_id, b_depth)| {
                b_depth.cmp(a_depth).then_with(|| a_id.cmp(b_id))
            });
            let region_ids_sorted = region_ids_with_depth
                .into_iter()
                .map(|(id, _)| id)
                .collect::<Vec<String>>();

            let cortical_area_ids = hierarchy
                .get_all_areas_recursive(region_id)
                .into_iter()
                .collect::<Vec<String>>();

            (region_ids_sorted, cortical_area_ids)
        };

        for cortical_id in cortical_area_ids {
            self.delete_cortical_area(&cortical_id).await?;
        }

        {
            let mut manager = self.connectome.write();
            for region_id in &region_ids {
                manager
                    .remove_brain_region(region_id)
                    .map_err(ServiceError::from)?;
            }
        }

        if let Some(genome) = self.current_genome.write().as_mut() {
            for region_id in &region_ids {
                genome.brain_regions.remove(region_id);
            }
        } else {
            warn!(
                target: "feagi-services",
                "[GENOME-UPDATE] No RuntimeGenome loaded - brain region deletions will not persist to saved genome"
            );
        }

        Ok(())
    }

    async fn update_brain_region(
        &self,
        region_id: &str,
        properties: std::collections::HashMap<String, serde_json::Value>,
    ) -> ServiceResult<BrainRegionInfo> {
        info!(target: "feagi-services", "Updating brain region: {}", region_id);

        let io_registry = self
            .connectome
            .write()
            .update_brain_region_properties(region_id, properties.clone())
            .map_err(ServiceError::from)?;

        if let Some(genome) = self.current_genome.write().as_mut() {
            if let Some(region) = genome.brain_regions.get_mut(region_id) {
                for (k, v) in &properties {
                    region.properties.insert(k.clone(), v.clone());
                }
            }
            if let Some(registry) = io_registry {
                for (rid, (inputs, outputs)) in registry {
                    if let Some(gr) = genome.brain_regions.get_mut(&rid) {
                        if inputs.is_empty() {
                            gr.properties.remove("inputs");
                        } else {
                            gr.properties
                                .insert("inputs".to_string(), serde_json::json!(inputs));
                        }
                        if outputs.is_empty() {
                            gr.properties.remove("outputs");
                        } else {
                            gr.properties
                                .insert("outputs".to_string(), serde_json::json!(outputs));
                        }
                    } else {
                        warn!(
                            target: "feagi-services",
                            "Region '{}' not found in RuntimeGenome while persisting IO registry after designated IO update",
                            rid
                        );
                    }
                }
            }
        }

        // Return updated info
        self.get_brain_region(region_id).await
    }

    async fn get_brain_region(&self, region_id: &str) -> ServiceResult<BrainRegionInfo> {
        trace!(target: "feagi-services", "Getting brain region: {}", region_id);

        {
            // Auto-heal legacy loaded genomes that only include the original core set.
            let mut manager = self.connectome.write();
            let has_any_core = [
                CoreCorticalType::Death,
                CoreCorticalType::Power,
                CoreCorticalType::Fatigue,
                CoreCorticalType::Pain,
                CoreCorticalType::Pleasure,
                CoreCorticalType::Fear,
                CoreCorticalType::Hope,
            ]
            .iter()
            .any(|core| manager.cortical_area_exists(&core.to_cortical_id()));
            if has_any_core {
                manager
                    .ensure_core_cortical_areas()
                    .map_err(ServiceError::from)?;
            }
        }

        let manager = self.connectome.read();

        let region = manager
            .get_brain_region(region_id)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "BrainRegion".to_string(),
                id: region_id.to_string(),
            })?;

        let hierarchy = manager.get_brain_region_hierarchy();
        let parent_id = hierarchy.get_parent(region_id).map(|s| s.to_string());
        let mut child_regions: Vec<String> = hierarchy
            .get_children(region_id)
            .into_iter()
            .cloned()
            .collect();
        child_regions.retain(|child_id| child_id != region_id);

        let mut cortical_areas: Vec<String> = region
            .cortical_areas
            .iter()
            .map(|id| id.as_base_64())
            .collect();

        // Ensure root region always includes invariant core areas if they exist (fixes BV disappearing)
        if parent_id.is_none() {
            for core in [
                CoreCorticalType::Death,
                CoreCorticalType::Power,
                CoreCorticalType::Fatigue,
                CoreCorticalType::Pain,
                CoreCorticalType::Pleasure,
                CoreCorticalType::Fear,
                CoreCorticalType::Hope,
            ] {
                let core_id = core.to_cortical_id();
                let core_id_b64 = core_id.as_base_64();
                if manager.cortical_area_exists(&core_id) && !cortical_areas.contains(&core_id_b64)
                {
                    cortical_areas.push(core_id_b64);
                }
            }
        }

        Ok(BrainRegionInfo {
            region_id: region_id.to_string(),
            name: region.name.clone(),
            region_type: Self::region_type_to_string(&region.region_type),
            parent_id,
            cortical_areas,
            child_regions,
            properties: region.properties.clone(),
        })
    }

    async fn list_brain_regions(&self) -> ServiceResult<Vec<BrainRegionInfo>> {
        trace!(target: "feagi-services", "Listing all brain regions");

        let region_ids: Vec<String> = {
            let manager = self.connectome.read();
            let ids = manager.get_brain_region_ids();
            trace!(target: "feagi-services", "Found {} brain region IDs from ConnectomeManager", ids.len());
            ids.into_iter().map(|s| s.to_string()).collect()
        };

        trace!(target: "feagi-services", "Processing {} regions...", region_ids.len());
        let mut regions = Vec::new();
        for region_id in region_ids {
            trace!(target: "feagi-services", "Getting region: {}", region_id);
            match self.get_brain_region(&region_id).await {
                Ok(region_info) => {
                    trace!(
                        target: "feagi-services",
                        "Got region: {} with {} areas",
                        region_info.name,
                        region_info.cortical_areas.len()
                    );
                    regions.push(region_info);
                }
                Err(e) => {
                    warn!(target: "feagi-services", "Failed to get region {}: {}", region_id, e);
                }
            }
        }

        trace!(target: "feagi-services", "Returning {} brain regions", regions.len());
        Ok(regions)
    }

    async fn get_brain_region_ids(&self) -> ServiceResult<Vec<String>> {
        debug!(target: "feagi-services","Getting brain region IDs");
        Ok(self
            .connectome
            .read()
            .get_brain_region_ids()
            .into_iter()
            .map(|s| s.to_string())
            .collect())
    }

    async fn brain_region_exists(&self, region_id: &str) -> ServiceResult<bool> {
        debug!(target: "feagi-services","Checking if brain region exists: {}", region_id);
        Ok(self.connectome.read().get_brain_region(region_id).is_some())
    }

    async fn get_root_region_id(&self) -> ServiceResult<Option<String>> {
        Ok(self.connectome.read().get_root_region_id())
    }

    async fn get_morphologies(&self) -> ServiceResult<HashMap<String, MorphologyInfo>> {
        let manager = self.connectome.read();
        let registry = manager.get_morphologies();

        let mut result = HashMap::new();
        for (id, morphology) in registry.iter() {
            result.insert(
                id.clone(),
                MorphologyInfo {
                    morphology_type: format!("{:?}", morphology.morphology_type).to_lowercase(),
                    class: morphology.class.clone(),
                    parameters: serde_json::to_value(&morphology.parameters)
                        .unwrap_or(serde_json::json!({})),
                },
            );
        }

        trace!(target: "feagi-services", "Retrieved {} morphologies", result.len());
        Ok(result)
    }

    async fn create_morphology(
        &self,
        morphology_id: String,
        morphology: feagi_evolutionary::Morphology,
    ) -> ServiceResult<()> {
        if morphology_id.trim().is_empty() {
            return Err(ServiceError::InvalidInput(
                "morphology_id must be non-empty".to_string(),
            ));
        }

        // Require a loaded RuntimeGenome for persistence (source of truth).
        let mut genome_guard = self.current_genome.write();
        let Some(genome) = genome_guard.as_mut() else {
            return Err(ServiceError::InvalidState(
                "No RuntimeGenome loaded - cannot create morphology".to_string(),
            ));
        };

        if genome.morphologies.contains(&morphology_id) {
            return Err(ServiceError::AlreadyExists {
                resource: "morphology".to_string(),
                id: morphology_id,
            });
        }

        genome
            .morphologies
            .add_morphology(morphology_id.clone(), morphology.clone());

        // Keep ConnectomeManager registry in sync (used by mapping/synapse generation).
        self.connectome
            .write()
            .upsert_morphology(morphology_id, morphology);

        Ok(())
    }

    async fn update_morphology(
        &self,
        morphology_id: String,
        morphology: feagi_evolutionary::Morphology,
    ) -> ServiceResult<()> {
        if morphology_id.trim().is_empty() {
            return Err(ServiceError::InvalidInput(
                "morphology_id must be non-empty".to_string(),
            ));
        }

        tracing::info!(
            target: "feagi-services",
            "[MORPH-AUDIT][SERVICE] update_morphology start name={} type={:?}",
            morphology_id,
            morphology.morphology_type
        );

        let mut usage_pairs = {
            let mut genome_guard = self.current_genome.write();
            let Some(genome) = genome_guard.as_mut() else {
                return Err(ServiceError::InvalidState(
                    "No RuntimeGenome loaded - cannot update morphology".to_string(),
                ));
            };

            if !genome.morphologies.contains(&morphology_id) {
                return Err(ServiceError::NotFound {
                    resource: "morphology".to_string(),
                    id: morphology_id,
                });
            }

            let usage_pairs = collect_morphology_usage_pairs(genome, &morphology_id);
            tracing::info!(
                target: "feagi-services",
                "[MORPH-AUDIT][SERVICE] RuntimeGenome usage pairs for {}: {}",
                morphology_id,
                usage_pairs.len()
            );
            if !usage_pairs.is_empty() && !self.connectome.read().has_npu() {
                return Err(ServiceError::InvalidState(
                    "Cannot rebuild synapses for morphology update because NPU is not connected"
                        .to_string(),
                ));
            }

            genome
                .morphologies
                .add_morphology(morphology_id.clone(), morphology.clone());

            usage_pairs
        };

        if usage_pairs.is_empty() {
            let area_infos = self.list_cortical_areas().await?;
            let fallback_pairs =
                collect_morphology_usage_pairs_from_area_infos(&area_infos, &morphology_id);
            if !fallback_pairs.is_empty() {
                warn!(
                    target: "feagi-services",
                    "RuntimeGenome had no usage pairs for morphology '{}'; recovered {} pairs from ConnectomeManager snapshot",
                    morphology_id,
                    fallback_pairs.len()
                );
                usage_pairs = fallback_pairs;
            }
            tracing::info!(
                target: "feagi-services",
                "[MORPH-AUDIT][SERVICE] Connectome fallback usage pairs for {}: {}",
                morphology_id,
                usage_pairs.len()
            );
        }

        if !usage_pairs.is_empty() && !self.connectome.read().has_npu() {
            return Err(ServiceError::InvalidState(
                "Cannot rebuild synapses for morphology update because NPU is not connected"
                    .to_string(),
            ));
        }

        let mut manager = self.connectome.write();
        manager.upsert_morphology(morphology_id.clone(), morphology);

        let mut regenerated_pairs = 0usize;
        let mut total_synapses = 0usize;
        let mut skipped_pairs = 0usize;

        for (raw_src_id, raw_dst_id) in usage_pairs {
            tracing::debug!(
                target: "feagi-services",
                "[MORPH-AUDIT][SERVICE] Rebuilding mapping {} -> {} for morphology {}",
                raw_src_id,
                raw_dst_id,
                morphology_id
            );
            let src_id = match parse_cortical_id_flexible(&raw_src_id) {
                Ok(id) => id,
                Err(error) => {
                    warn!(
                        target: "feagi-services",
                        "Skipping morphology regeneration for invalid source cortical ID {}: {}",
                        raw_src_id,
                        error
                    );
                    skipped_pairs += 1;
                    continue;
                }
            };
            let dst_id = match parse_cortical_id_flexible(&raw_dst_id) {
                Ok(id) => id,
                Err(error) => {
                    warn!(
                        target: "feagi-services",
                        "Skipping morphology regeneration for invalid destination cortical ID {}: {}",
                        raw_dst_id,
                        error
                    );
                    skipped_pairs += 1;
                    continue;
                }
            };

            let synapses = manager
                .regenerate_synapses_for_mapping(&src_id, &dst_id)
                .map_err(|e| {
                    ServiceError::Backend(format!(
                        "Failed to regenerate synapses for morphology update ({} -> {}): {}",
                        src_id, dst_id, e
                    ))
                })?;
            regenerated_pairs += 1;
            total_synapses += synapses;
        }

        info!(
            target: "feagi-services",
            "Updated morphology '{}' and regenerated {} mapping pairs (synapses_created={}, skipped_pairs={})",
            morphology_id,
            regenerated_pairs,
            total_synapses,
            skipped_pairs
        );

        Ok(())
    }

    async fn delete_morphology(&self, morphology_id: &str) -> ServiceResult<()> {
        if morphology_id.trim().is_empty() {
            return Err(ServiceError::InvalidInput(
                "morphology_id must be non-empty".to_string(),
            ));
        }

        let mut genome_guard = self.current_genome.write();
        let Some(genome) = genome_guard.as_mut() else {
            return Err(ServiceError::InvalidState(
                "No RuntimeGenome loaded - cannot delete morphology".to_string(),
            ));
        };

        if !genome.morphologies.remove_morphology(morphology_id) {
            return Err(ServiceError::NotFound {
                resource: "morphology".to_string(),
                id: morphology_id.to_string(),
            });
        }

        // Mirror deletion into the ConnectomeManager registry.
        self.connectome.write().remove_morphology(morphology_id);

        Ok(())
    }

    async fn rename_morphology(&self, old_id: &str, new_id: &str) -> ServiceResult<()> {
        let old_id = old_id.trim();
        let new_id = new_id.trim();

        if old_id.is_empty() {
            return Err(ServiceError::InvalidInput(
                "old_id must be non-empty".to_string(),
            ));
        }
        if new_id.is_empty() {
            return Err(ServiceError::InvalidInput(
                "new_id must be non-empty".to_string(),
            ));
        }
        if old_id == new_id {
            return Err(ServiceError::InvalidInput(
                "old_id and new_id must differ".to_string(),
            ));
        }

        let mut genome_guard = self.current_genome.write();
        let Some(genome) = genome_guard.as_mut() else {
            return Err(ServiceError::InvalidState(
                "No RuntimeGenome loaded - cannot rename morphology".to_string(),
            ));
        };

        let morphology =
            genome
                .morphologies
                .get(old_id)
                .cloned()
                .ok_or_else(|| ServiceError::NotFound {
                    resource: "morphology".to_string(),
                    id: old_id.to_string(),
                })?;

        if morphology.class == "core" {
            return Err(ServiceError::InvalidInput(format!(
                "Core morphologies cannot be renamed: '{}'",
                old_id
            )));
        }

        if genome.morphologies.contains(new_id) {
            return Err(ServiceError::AlreadyExists {
                resource: "morphology".to_string(),
                id: new_id.to_string(),
            });
        }

        genome.morphologies.remove_morphology(old_id);
        genome
            .morphologies
            .add_morphology(new_id.to_string(), morphology.clone());

        let mut replaced_count: usize = 0;
        for area in genome.cortical_areas.values_mut() {
            for prop_value in area.properties.values_mut() {
                replace_morphology_id_in_value(prop_value, old_id, new_id, &mut replaced_count);
            }
        }

        for region in genome.brain_regions.values_mut() {
            for prop_value in region.properties.values_mut() {
                replace_morphology_id_in_value(prop_value, old_id, new_id, &mut replaced_count);
            }
        }

        drop(genome_guard);

        let mut manager = self.connectome.write();
        manager.remove_morphology(old_id);
        manager.upsert_morphology(new_id.to_string(), morphology);

        info!(
            target: "feagi-services",
            "Renamed morphology '{}' to '{}' ({} references updated)",
            old_id,
            new_id,
            replaced_count
        );

        Ok(())
    }

    async fn update_cortical_mapping(
        &self,
        src_area_id: String,
        dst_area_id: String,
        mapping_data: Vec<serde_json::Value>,
    ) -> ServiceResult<usize> {
        info!(target: "feagi-services", "Updating cortical mapping: {} -> {} with {} connections",
              src_area_id, dst_area_id, mapping_data.len());

        // Convert String to CorticalID
        use feagi_genome_definitions::::CorticalID;
        let src_id = CorticalID::try_from_base_64(&src_area_id).map_err(|e| {
            ServiceError::InvalidInput(format!("Invalid source cortical ID: {}", e))
        })?;
        let dst_id = CorticalID::try_from_base_64(&dst_area_id).map_err(|e| {
            ServiceError::InvalidInput(format!("Invalid destination cortical ID: {}", e))
        })?;

        let existing_mapping = {
            let manager = self.connectome.read();
            manager.get_cortical_area(&src_id).and_then(|area| {
                get_cortical_mapping_dst_from_properties(&area.properties, &dst_area_id)
            })
        };

        let mut existing_plasticity_by_morphology: HashMap<
            String,
            serde_json::Map<String, serde_json::Value>,
        > = HashMap::new();
        if let Some(existing_rules) = existing_mapping.as_ref() {
            for rule in existing_rules {
                if let Some(obj) = rule.as_object() {
                    if let Some(morphology_id) = obj.get("morphology_id").and_then(|v| v.as_str()) {
                        existing_plasticity_by_morphology
                            .insert(morphology_id.to_string(), obj.clone());
                    }
                }
            }
        }

        let mut normalized_mapping_data = Vec::with_capacity(mapping_data.len());
        for rule in &mapping_data {
            if let Some(obj) = rule.as_object() {
                let mut normalized = obj.clone();
                let morphology_id = normalized
                    .get("morphology_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let is_associative = morphology_id == "associative_memory";
                let mut plasticity_flag = normalized
                    .get("plasticity_flag")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if is_associative {
                    normalized.insert("plasticity_flag".to_string(), serde_json::json!(true));
                    plasticity_flag = true;
                }
                if let Some(existing) = existing_plasticity_by_morphology.get(&morphology_id) {
                    if !normalized.contains_key("synaptic_delay_bursts")
                        && existing.contains_key("synaptic_delay_bursts")
                    {
                        if let Some(value) = existing.get("synaptic_delay_bursts") {
                            normalized.insert("synaptic_delay_bursts".to_string(), value.clone());
                        }
                    }
                }
                if plasticity_flag || is_associative {
                    let required = [
                        "plasticity_constant",
                        "ltp_multiplier",
                        "ltd_multiplier",
                        "plasticity_window",
                    ];
                    if let Some(existing) = existing_plasticity_by_morphology.get(&morphology_id) {
                        for key in required {
                            if !normalized.contains_key(key) && existing.contains_key(key) {
                                if let Some(value) = existing.get(key) {
                                    normalized.insert(key.to_string(), value.clone());
                                }
                            }
                        }
                    }
                    let missing: Vec<&str> = required
                        .iter()
                        .copied()
                        .filter(|key| !normalized.contains_key(*key))
                        .collect();
                    if !missing.is_empty() {
                        return Err(ServiceError::InvalidInput(format!(
                            "Associative memory mapping for {} is missing required plasticity keys {:?}",
                            morphology_id, missing
                        )));
                    }
                }
                normalized_mapping_data.push(serde_json::Value::Object(normalized));
            } else {
                normalized_mapping_data.push(rule.clone());
            }
        }

        debug!(
            target: "feagi-services",
            "Mapping update request: {} -> {} (existing_rules={}, new_rules={})",
            src_area_id,
            dst_area_id,
            existing_mapping.as_ref().map(|rules| rules.len()).unwrap_or(0),
            normalized_mapping_data.len()
        );

        if existing_mapping
            .as_ref()
            .is_some_and(|rules| rules == &normalized_mapping_data)
        {
            info!(
                target: "feagi-services",
                "Mapping unchanged for {} -> {}; skipping regeneration",
                src_area_id,
                dst_area_id
            );
            return Ok(0);
        }

        // Update RuntimeGenome if available (CRITICAL for save/load persistence!)
        if let Some(genome) = self.current_genome.write().as_mut() {
            if let Some(src_area) = genome.cortical_areas.get_mut(&src_id) {
                update_cortical_mapping_dst_in_properties(
                    &mut src_area.properties,
                    &dst_area_id,
                    &normalized_mapping_data,
                )?;
                info!(
                    target: "feagi-services",
                    "[GENOME-UPDATE] Updated cortical_mapping_dst for {} -> {} (connections={})",
                    src_area_id,
                    dst_area_id,
                    normalized_mapping_data.len()
                );
            } else {
                warn!(target: "feagi-services", "[GENOME-UPDATE] Source area {} not found in RuntimeGenome", src_area_id);
            }
        } else {
            warn!(target: "feagi-services", "[GENOME-UPDATE] No RuntimeGenome loaded - mapping will not persist");
        }

        // Update the cortical_mapping_dst property in ConnectomeManager
        let region_io = {
            let mut manager = self.connectome.write();
            manager
                .update_cortical_mapping(&src_id, &dst_id, normalized_mapping_data.clone())
                .map_err(|e| ServiceError::Backend(format!("Failed to update mapping: {}", e)))?;

            // Regenerate synapses for this mapping
            let synapse_count = manager
                .regenerate_synapses_for_mapping(&src_id, &dst_id)
                .map_err(|e| {
                    ServiceError::Backend(format!("Failed to regenerate synapses: {}", e))
                })?;

            // Recompute region IO registries after mapping change (critical for BV region boundary behavior)
            let region_io = manager.recompute_brain_region_io_registry().map_err(|e| {
                ServiceError::Backend(format!("Failed to recompute region IO registry: {}", e))
            })?;

            info!(
                target: "feagi-services",
                "Cortical mapping updated: {} synapses created",
                synapse_count
            );

            (synapse_count, region_io)
        };

        // Refresh burst-runner caches so newly created twin areas are visualized immediately.
        // IMPORTANT: Call this after releasing the connectome write lock to avoid deadlocks.
        self.refresh_burst_runner_cache();

        // Persist updated region IO into RuntimeGenome so genome save/export stays consistent.
        if let Some(genome) = self.current_genome.write().as_mut() {
            for (region_id, (inputs, outputs)) in region_io.1 {
                if let Some(region) = genome.brain_regions.get_mut(&region_id) {
                    if inputs.is_empty() {
                        region.properties.remove("inputs");
                    } else {
                        region
                            .properties
                            .insert("inputs".to_string(), serde_json::json!(inputs));
                    }

                    if outputs.is_empty() {
                        region.properties.remove("outputs");
                    } else {
                        region
                            .properties
                            .insert("outputs".to_string(), serde_json::json!(outputs));
                    }
                } else {
                    warn!(
                        target: "feagi-services",
                        "Region '{}' not found in RuntimeGenome while persisting IO registry",
                        region_id
                    );
                }
            }

            let manager = self.connectome.read();
            if let Some(memory_area) = manager.get_cortical_area(&dst_id) {
                if let Some(twin_map) = memory_area
                    .properties
                    .get("memory_twin_areas")
                    .and_then(|v| v.as_object())
                {
                    if let Some(genome_area) = genome.cortical_areas.get_mut(&dst_id) {
                        genome_area
                            .properties
                            .insert("memory_twin_areas".to_string(), serde_json::json!(twin_map));
                        if let Some(mapping) = memory_area.properties.get("cortical_mapping_dst") {
                            genome_area
                                .properties
                                .insert("cortical_mapping_dst".to_string(), mapping.clone());
                        }
                    }
                    for twin_id_str in twin_map.values().filter_map(|v| v.as_str()) {
                        let Ok(twin_id) = CorticalID::try_from_base_64(twin_id_str) else {
                            continue;
                        };
                        if genome.cortical_areas.contains_key(&twin_id) {
                            continue;
                        }
                        if let Some(twin_area) = manager.get_cortical_area(&twin_id) {
                            genome.cortical_areas.insert(twin_id, twin_area.clone());
                            if let Some(parent_region_id) = twin_area
                                .properties
                                .get("parent_region_id")
                                .and_then(|v| v.as_str())
                            {
                                if let Some(region) = genome.brain_regions.get_mut(parent_region_id)
                                {
                                    region.add_area(twin_id);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(region_io.0)
    }

    // Note: unit tests for mapping persistence behavior are below in this module.

    // ========================================================================
    // CONNECTOME I/O OPERATIONS
    // ========================================================================

    #[cfg(feature = "connectome-io")]
    async fn export_connectome(
        &self,
    ) -> ServiceResult<feagi_npu_neural::types::connectome::ConnectomeSnapshot> {
        info!(target: "feagi-services", "Exporting connectome via service layer");

        // Get NPU from ConnectomeManager (which has reference to NPU)
        // Note: get_npu() returns Option<&Arc<...>>, so we need to clone the Arc
        // to use it outside the lock scope
        let npu_arc = {
            let connectome = self.connectome.read();
            let npu_opt = connectome.get_npu();
            npu_opt
                .ok_or_else(|| {
                    ServiceError::Backend("NPU not connected to ConnectomeManager".to_string())
                })?
                .clone()
        };

        // Export connectome from NPU
        // Note: export_connectome() is on RustNPU, but we have DynamicNPU
        // We need to handle both F32 and INT8 variants
        use tracing::debug;
        let lock_start = std::time::Instant::now();
        let thread_id = std::thread::current().id();
        debug!(
            "[NPU-LOCK] CONNECTOME-SERVICE: Thread {:?} attempting NPU lock for export_connectome at {:?}",
            thread_id, lock_start
        );
        let snapshot = {
            let npu_lock = npu_arc.lock().unwrap();
            let lock_acquired = std::time::Instant::now();
            let lock_wait = lock_acquired.duration_since(lock_start);
            debug!(
                "[NPU-LOCK] CONNECTOME-SERVICE: Thread {:?} acquired lock after {:.2}ms wait for export_connectome",
                thread_id,
                lock_wait.as_secs_f64() * 1000.0
            );
            match &*npu_lock {
                feagi_npu_burst_engine::DynamicNPU::F32(npu_f32) => npu_f32.export_connectome(),
                feagi_npu_burst_engine::DynamicNPU::INT8(npu_int8) => npu_int8.export_connectome(),
            }
        };
        let lock_released = std::time::Instant::now();
        let total_duration = lock_released.duration_since(lock_start);
        debug!(
            "[NPU-LOCK] CONNECTOME-SERVICE: Thread {:?} RELEASED NPU lock after export_connectome (total: {:.2}ms)",
            thread_id,
            total_duration.as_secs_f64() * 1000.0
        );

        info!(target: "feagi-services", "✅ Connectome exported: {} neurons, {} synapses",
            snapshot.neurons.count, snapshot.synapses.count);

        Ok(snapshot)
    }

    #[cfg(feature = "connectome-io")]
    async fn import_connectome(
        &self,
        snapshot: feagi_npu_neural::types::connectome::ConnectomeSnapshot,
    ) -> ServiceResult<()> {
        info!(target: "feagi-services", "Importing connectome via service layer: {} neurons, {} synapses",
            snapshot.neurons.count, snapshot.synapses.count);

        // NOTE: NPU.import_connectome_with_config() is a constructor that creates a NEW NPU.
        // This means importing requires replacing the entire NPU instance, which involves:
        // 1. Stopping the burst engine
        // 2. Creating a new NPU from the snapshot
        // 3. Replacing the NPU in ConnectomeManager and BurstLoopRunner
        // 4. Restarting the burst engine
        //
        // This is a complex operation that requires coordination across multiple components.
        // For now, we return NotImplemented and recommend using the NPU constructor directly
        // during application initialization, or implementing a higher-level "replace NPU" operation.

        warn!(target: "feagi-services", "⚠️ Connectome import via service layer not yet fully implemented");
        warn!(target: "feagi-services", "   NPU.import_connectome_with_config() creates a new NPU instance");
        warn!(target: "feagi-services", "   This requires stopping burst engine, replacing NPU, and restarting");
        warn!(target: "feagi-services", "   Recommendation: Use NPU.import_connectome_with_config() during initialization");

        Err(ServiceError::NotImplemented(
            "Connectome import via service layer requires NPU replacement coordination. Use NPU.import_connectome_with_config() during application initialization, or implement a 'replace NPU' operation that coordinates with BurstLoopRunner.".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        collect_morphology_usage_pairs, parse_cortical_id_flexible, replace_morphology_id_in_value,
        update_cortical_mapping_dst_in_properties,
    };
    use crate::types::ServiceResult;
    use std::collections::HashMap;
    use feagi_genome_definitions::::brain_region::BrainRegion;
    use feagi_genome_definitions::::region_type::RegionType;

    #[test]
    fn empty_mapping_deletes_destination_key_and_prunes_container() -> ServiceResult<()> {
        let mut props: HashMap<String, serde_json::Value> = HashMap::new();
        props.insert(
            "cortical_mapping_dst".to_string(),
            serde_json::json!({
                "dstA": [{"morphology_id": "m1"}],
                "dstB": []
            }),
        );

        update_cortical_mapping_dst_in_properties(&mut props, "dstA", &[])?;
        let dst = props
            .get("cortical_mapping_dst")
            .and_then(|v| v.as_object())
            .expect("cortical_mapping_dst should remain with dstB");
        assert!(!dst.contains_key("dstA"));
        assert!(dst.contains_key("dstB"));

        // Now remove last remaining destination, container should be removed entirely
        update_cortical_mapping_dst_in_properties(&mut props, "dstB", &[])?;
        assert!(!props.contains_key("cortical_mapping_dst"));
        Ok(())
    }

    #[test]
    fn non_empty_mapping_sets_destination_key() -> ServiceResult<()> {
        let mut props: HashMap<String, serde_json::Value> = HashMap::new();
        update_cortical_mapping_dst_in_properties(
            &mut props,
            "dstX",
            &[serde_json::json!({"morphology_id": "m1"})],
        )?;

        let dst = props
            .get("cortical_mapping_dst")
            .and_then(|v| v.as_object())
            .expect("cortical_mapping_dst should exist");
        let arr = dst
            .get("dstX")
            .and_then(|v| v.as_array())
            .expect("dstX should be an array");
        assert_eq!(arr.len(), 1);
        Ok(())
    }

    #[test]
    fn collect_morphology_usage_pairs_scans_object_and_array_rules() {
        use feagi_genome_definitions::::{
            CorticalArea, CorticalAreaDimensions, CorticalAreaType, CorticalID,
        };

        let src_a = CorticalID::try_from_bytes(b"csrc0001").unwrap();
        let src_b = CorticalID::try_from_bytes(b"csrc0002").unwrap();
        let dst_a = CorticalID::try_from_bytes(b"csrc0003").unwrap();
        let dst_b = CorticalID::try_from_bytes(b"csrc0004").unwrap();

        let mut area_a = CorticalArea::new(
            src_a,
            0,
            "Source A".to_string(),
            CorticalAreaDimensions::new(1, 1, 1).unwrap(),
            (0, 0, 0).into(),
            CorticalAreaType::Custom(
                feagi_genome_definitions::::CustomCorticalType::LeakyIntegrateFire,
            ),
        )
        .unwrap();
        area_a.properties.insert(
            "cortical_mapping_dst".to_string(),
            serde_json::json!({
                dst_a.as_base_64(): [
                    {"morphology_id": "m_shared"},
                    {"morphology_id": "m_shared"}
                ],
                dst_b.as_base_64(): [
                    ["m_shared", 1, 1.0, false]
                ]
            }),
        );

        let mut area_b = CorticalArea::new(
            src_b,
            1,
            "Source B".to_string(),
            CorticalAreaDimensions::new(1, 1, 1).unwrap(),
            (0, 0, 0).into(),
            CorticalAreaType::Custom(
                feagi_genome_definitions::::CustomCorticalType::LeakyIntegrateFire,
            ),
        )
        .unwrap();
        area_b.properties.insert(
            "cortical_mapping_dst".to_string(),
            serde_json::json!({
                dst_a.as_base_64(): [{"morphology_id": "m_other"}]
            }),
        );

        let mut cortical_areas = HashMap::new();
        cortical_areas.insert(area_a.cortical_id, area_a);
        cortical_areas.insert(area_b.cortical_id, area_b);
        let genome = feagi_evolutionary::RuntimeGenome {
            metadata: feagi_evolutionary::GenomeMetadata {
                genome_id: "test".to_string(),
                genome_title: "test".to_string(),
                genome_description: "".to_string(),
                version: "2.0".to_string(),
                timestamp: 0.0,
                brain_regions_root: None,
            },
            cortical_areas,
            brain_regions: HashMap::new(),
            morphologies: feagi_evolutionary::MorphologyRegistry::new(),
            physiology: feagi_evolutionary::PhysiologyConfig::default(),
            signatures: feagi_evolutionary::GenomeSignatures {
                genome: "0".to_string(),
                blueprint: "0".to_string(),
                physiology: "0".to_string(),
                morphologies: None,
            },
            stats: feagi_evolutionary::GenomeStats::default(),
        };

        let pairs = collect_morphology_usage_pairs(&genome, "m_shared");
        assert_eq!(
            pairs,
            vec![
                (src_a.as_base_64(), dst_a.as_base_64()),
                (src_a.as_base_64(), dst_b.as_base_64()),
            ]
        );
    }

    #[test]
    fn parse_cortical_id_flexible_accepts_base64_and_legacy_formats() {
        use feagi_genome_definitions::::CorticalID;

        let original = CorticalID::try_from_bytes(b"csrc0001").unwrap();
        let as_base64 = original.as_base_64();
        let from_base64 = parse_cortical_id_flexible(&as_base64).expect("base64 should parse");
        assert_eq!(from_base64, original);

        let legacy_id = "csrc0001";
        let from_legacy = parse_cortical_id_flexible(legacy_id).expect("legacy should parse");
        assert_eq!(from_legacy, original);
    }

    #[tokio::test]
    async fn morphology_create_update_delete_roundtrip() -> ServiceResult<()> {
        use super::ConnectomeServiceImpl;
        use crate::traits::ConnectomeService;
        use parking_lot::RwLock;
        use std::sync::Arc;

        // Isolated connectome manager instance for this test.
        let connectome = Arc::new(RwLock::new(
            feagi_brain_development::ConnectomeManager::new_for_testing(),
        ));

        // Minimal RuntimeGenome (source of truth) for persistence.
        let genome = feagi_evolutionary::RuntimeGenome {
            metadata: feagi_evolutionary::GenomeMetadata {
                genome_id: "test".to_string(),
                genome_title: "test".to_string(),
                genome_description: "".to_string(),
                version: "2.0".to_string(),
                timestamp: 0.0,
                brain_regions_root: None,
            },
            cortical_areas: HashMap::new(),
            brain_regions: HashMap::new(),
            morphologies: feagi_evolutionary::MorphologyRegistry::new(),
            physiology: feagi_evolutionary::PhysiologyConfig::default(),
            signatures: feagi_evolutionary::GenomeSignatures {
                genome: "0".to_string(),
                blueprint: "0".to_string(),
                physiology: "0".to_string(),
                morphologies: None,
            },
            stats: feagi_evolutionary::GenomeStats::default(),
        };
        let current_genome = Arc::new(RwLock::new(Some(genome)));

        let svc = ConnectomeServiceImpl::new(connectome.clone(), current_genome.clone());

        // Create
        let morph_id = "m_test_vectors".to_string();
        let morph = feagi_evolutionary::Morphology {
            morphology_type: feagi_evolutionary::MorphologyType::Vectors,
            parameters: feagi_evolutionary::MorphologyParameters::Vectors {
                vectors: vec![[1, 2, 3]],
            },
            class: "custom".to_string(),
        };
        svc.create_morphology(morph_id.clone(), morph).await?;

        // Verify both source-of-truth and connectome registry were updated
        {
            let genome_guard = current_genome.read();
            let genome = genome_guard.as_ref().expect("genome must exist");
            assert!(genome.morphologies.contains(&morph_id));
        }
        {
            let mgr = connectome.read();
            assert!(mgr.get_morphologies().contains(&morph_id));
        }

        // Update (overwrite vectors)
        let morph2 = feagi_evolutionary::Morphology {
            morphology_type: feagi_evolutionary::MorphologyType::Vectors,
            parameters: feagi_evolutionary::MorphologyParameters::Vectors {
                vectors: vec![[9, 9, 9]],
            },
            class: "custom".to_string(),
        };
        svc.update_morphology(morph_id.clone(), morph2).await?;
        {
            let mgr = connectome.read();
            let stored = mgr
                .get_morphologies()
                .get(&morph_id)
                .expect("morphology must exist");
            match &stored.parameters {
                feagi_evolutionary::MorphologyParameters::Vectors { vectors } => {
                    assert_eq!(vectors.as_slice(), &[[9, 9, 9]]);
                }
                other => panic!("unexpected parameters: {:?}", other),
            }
        }

        // Delete
        svc.delete_morphology(&morph_id).await?;
        {
            let genome_guard = current_genome.read();
            let genome = genome_guard.as_ref().expect("genome must exist");
            assert!(!genome.morphologies.contains(&morph_id));
        }
        {
            let mgr = connectome.read();
            assert!(!mgr.get_morphologies().contains(&morph_id));
        }

        Ok(())
    }

    #[tokio::test]
    async fn morphology_rename_updates_registry_and_cortical_mapping_references(
    ) -> ServiceResult<()> {
        use super::ConnectomeServiceImpl;
        use crate::traits::ConnectomeService;
        use feagi_genome_definitions::::{
            CorticalArea, CorticalAreaDimensions, CorticalAreaType, CorticalID,
        };
        use parking_lot::RwLock;
        use std::collections::HashMap;
        use std::sync::Arc;

        let connectome = Arc::new(RwLock::new(
            feagi_brain_development::ConnectomeManager::new_for_testing(),
        ));

        let src_id = CorticalID::try_from_bytes(b"csrc0001").unwrap();
        let dst_id = CorticalID::try_from_bytes(b"csrc0002").unwrap();

        let mut src_area = CorticalArea::new(
            src_id,
            0,
            "Source".to_string(),
            CorticalAreaDimensions::new(1, 1, 1).unwrap(),
            (0, 0, 0).into(),
            CorticalAreaType::Custom(
                feagi_genome_definitions::::CustomCorticalType::LeakyIntegrateFire,
            ),
        )
        .unwrap();
        src_area.properties.insert(
            "cortical_mapping_dst".to_string(),
            serde_json::json!({
                dst_id.as_base_64(): [
                    {"morphology_id": "m_old", "morphology_scalar": 1},
                    ["m_old", 2, 1.0, false]
                ]
            }),
        );

        let dst_area = CorticalArea::new(
            dst_id,
            1,
            "Target".to_string(),
            CorticalAreaDimensions::new(1, 1, 1).unwrap(),
            (0, 0, 0).into(),
            CorticalAreaType::Custom(
                feagi_genome_definitions::::CustomCorticalType::LeakyIntegrateFire,
            ),
        )
        .unwrap();

        let morph = feagi_evolutionary::Morphology {
            morphology_type: feagi_evolutionary::MorphologyType::Vectors,
            parameters: feagi_evolutionary::MorphologyParameters::Vectors {
                vectors: vec![[1, 0, 0]],
            },
            class: "custom".to_string(),
        };

        let mut morphologies = feagi_evolutionary::MorphologyRegistry::new();
        morphologies.add_morphology("m_old".to_string(), morph.clone());

        let genome = feagi_evolutionary::RuntimeGenome {
            metadata: feagi_evolutionary::GenomeMetadata {
                genome_id: "test".to_string(),
                genome_title: "test".to_string(),
                genome_description: "".to_string(),
                version: "2.0".to_string(),
                timestamp: 0.0,
                brain_regions_root: None,
            },
            cortical_areas: HashMap::from([(src_id, src_area.clone()), (dst_id, dst_area.clone())]),
            brain_regions: HashMap::new(),
            morphologies,
            physiology: feagi_evolutionary::PhysiologyConfig::default(),
            signatures: feagi_evolutionary::GenomeSignatures {
                genome: "0".to_string(),
                blueprint: "0".to_string(),
                physiology: "0".to_string(),
                morphologies: None,
            },
            stats: feagi_evolutionary::GenomeStats::default(),
        };

        {
            let mut manager = connectome.write();
            manager.upsert_morphology("m_old".to_string(), morph);
        }

        let current_genome = Arc::new(RwLock::new(Some(genome)));
        let svc = ConnectomeServiceImpl::new(connectome.clone(), current_genome.clone());

        svc.rename_morphology("m_old", "m_new").await?;

        {
            let genome_guard = current_genome.read();
            let genome = genome_guard.as_ref().expect("genome must exist");
            assert!(!genome.morphologies.contains("m_old"));
            assert!(genome.morphologies.contains("m_new"));

            let area = genome
                .cortical_areas
                .get(&src_id)
                .expect("src area must exist");
            let dstmap = area
                .properties
                .get("cortical_mapping_dst")
                .and_then(|v| v.as_object())
                .expect("cortical_mapping_dst must exist");
            let rules = dstmap
                .get(&dst_id.as_base_64())
                .and_then(|v| v.as_array())
                .expect("rules must exist");
            assert_eq!(rules.len(), 2);
            assert_eq!(
                rules[0].get("morphology_id").and_then(|v| v.as_str()),
                Some("m_new")
            );
            assert_eq!(
                rules[1]
                    .as_array()
                    .and_then(|a| a.first())
                    .and_then(|v| v.as_str()),
                Some("m_new")
            );
        }

        {
            let mgr = connectome.read();
            assert!(!mgr.get_morphologies().contains("m_old"));
            assert!(mgr.get_morphologies().contains("m_new"));
        }

        Ok(())
    }

    #[tokio::test]
    async fn morphology_rename_rejects_core_morphologies() -> ServiceResult<()> {
        use super::ConnectomeServiceImpl;
        use crate::traits::ConnectomeService;
        use crate::types::ServiceError;
        use parking_lot::RwLock;
        use std::collections::HashMap;
        use std::sync::Arc;

        let connectome = Arc::new(RwLock::new(
            feagi_brain_development::ConnectomeManager::new_for_testing(),
        ));
        {
            let mut manager = connectome.write();
            manager.setup_core_morphologies_for_testing();
        }

        let mut morphologies = feagi_evolutionary::MorphologyRegistry::new();
        feagi_evolutionary::add_core_morphologies(&mut morphologies);

        let genome = feagi_evolutionary::RuntimeGenome {
            metadata: feagi_evolutionary::GenomeMetadata {
                genome_id: "test".to_string(),
                genome_title: "test".to_string(),
                genome_description: "".to_string(),
                version: "2.0".to_string(),
                timestamp: 0.0,
                brain_regions_root: None,
            },
            cortical_areas: HashMap::new(),
            brain_regions: HashMap::new(),
            morphologies,
            physiology: feagi_evolutionary::PhysiologyConfig::default(),
            signatures: feagi_evolutionary::GenomeSignatures {
                genome: "0".to_string(),
                blueprint: "0".to_string(),
                physiology: "0".to_string(),
                morphologies: None,
            },
            stats: feagi_evolutionary::GenomeStats::default(),
        };

        let current_genome = Arc::new(RwLock::new(Some(genome)));
        let svc = ConnectomeServiceImpl::new(connectome.clone(), current_genome.clone());

        let err = svc
            .rename_morphology("projector", "projector_renamed")
            .await
            .unwrap_err();
        assert!(
            matches!(err, ServiceError::InvalidInput(_)),
            "Expected InvalidInput, got {:?}",
            err
        );

        Ok(())
    }

    #[test]
    fn replace_morphology_id_in_value_handles_object_and_array_formats() {
        let mut value = serde_json::json!({
            "nested": {
                "morphology_id": "old_id",
                "other": "x"
            },
            "arr": ["old_id", 1, 2]
        });
        let mut count = 0;
        replace_morphology_id_in_value(&mut value, "old_id", "new_id", &mut count);
        assert_eq!(count, 2);
        assert_eq!(value["nested"]["morphology_id"], "new_id");
        assert_eq!(value["arr"][0], "new_id");
    }

    #[tokio::test]
    async fn associative_memory_missing_window_is_filled_from_existing_mapping() -> ServiceResult<()>
    {
        use super::ConnectomeServiceImpl;
        use crate::traits::ConnectomeService;
        use feagi_genome_definitions::::{
            CorticalArea, CorticalAreaDimensions, CorticalAreaType, CorticalID,
        };
        use parking_lot::RwLock;
        use std::collections::HashMap;
        use std::sync::Arc;

        let connectome = Arc::new(RwLock::new(
            feagi_brain_development::ConnectomeManager::new_for_testing(),
        ));

        let src_id = CorticalID::try_from_bytes(b"csrc0003").unwrap();
        let dst_id = CorticalID::try_from_bytes(b"csrc0004").unwrap();

        let mut src_area = CorticalArea::new(
            src_id,
            0,
            "Source".to_string(),
            CorticalAreaDimensions::new(1, 1, 1).unwrap(),
            (0, 0, 0).into(),
            CorticalAreaType::Custom(
                feagi_genome_definitions::::CustomCorticalType::LeakyIntegrateFire,
            ),
        )
        .unwrap();
        let dst_area = CorticalArea::new(
            dst_id,
            0,
            "Target".to_string(),
            CorticalAreaDimensions::new(1, 1, 1).unwrap(),
            (0, 0, 0).into(),
            CorticalAreaType::Custom(
                feagi_genome_definitions::::CustomCorticalType::LeakyIntegrateFire,
            ),
        )
        .unwrap();

        src_area.properties.insert(
            "cortical_mapping_dst".to_string(),
            serde_json::json!({
                dst_id.as_base_64(): [{
                    "morphology_id": "associative_memory",
                    "morphology_scalar": 1,
                    "postSynapticCurrent_multiplier": 1.0,
                    "plasticity_flag": true,
                    "plasticity_constant": 1,
                    "ltp_multiplier": 1,
                    "ltd_multiplier": 1,
                    "plasticity_window": 5
                }]
            }),
        );

        {
            let mut manager = connectome.write();
            manager.add_cortical_area(src_area.clone())?;
            manager.add_cortical_area(dst_area.clone())?;
        }

        let genome = feagi_evolutionary::RuntimeGenome {
            metadata: feagi_evolutionary::GenomeMetadata {
                genome_id: "test".to_string(),
                genome_title: "test".to_string(),
                genome_description: "".to_string(),
                version: "2.0".to_string(),
                timestamp: 0.0,
                brain_regions_root: None,
            },
            cortical_areas: HashMap::from([(src_id, src_area), (dst_id, dst_area)]),
            brain_regions: HashMap::new(),
            morphologies: feagi_evolutionary::MorphologyRegistry::new(),
            physiology: feagi_evolutionary::PhysiologyConfig::default(),
            signatures: feagi_evolutionary::GenomeSignatures {
                genome: "0".to_string(),
                blueprint: "0".to_string(),
                physiology: "0".to_string(),
                morphologies: None,
            },
            stats: feagi_evolutionary::GenomeStats::default(),
        };
        let current_genome = Arc::new(RwLock::new(Some(genome)));

        let svc = ConnectomeServiceImpl::new(connectome.clone(), current_genome.clone());

        let mapping_data = vec![serde_json::json!({
            "morphology_id": "associative_memory",
            "morphology_scalar": 1,
            "postSynapticCurrent_multiplier": 1.0,
            "plasticity_flag": true,
            "plasticity_constant": 1,
            "ltp_multiplier": 1,
            "ltd_multiplier": 1
        })];

        svc.update_cortical_mapping(src_id.as_base_64(), dst_id.as_base_64(), mapping_data)
            .await?;

        let genome_guard = current_genome.read();
        let genome = genome_guard.as_ref().expect("genome must exist");
        let updated = genome
            .cortical_areas
            .get(&src_id)
            .and_then(|area| area.properties.get("cortical_mapping_dst"))
            .and_then(|v| v.as_object())
            .and_then(|map| map.get(&dst_id.as_base_64()))
            .and_then(|v| v.as_array())
            .and_then(|rules| rules.first())
            .and_then(|v| v.as_object())
            .and_then(|obj| obj.get("plasticity_window"))
            .and_then(|v| v.as_i64());
        assert_eq!(updated, Some(5));

        Ok(())
    }

    #[tokio::test]
    async fn delete_cortical_area_persists_to_runtime_genome() -> ServiceResult<()> {
        use super::ConnectomeServiceImpl;
        use crate::traits::ConnectomeService;
        use feagi_genome_definitions::::{RegionID};
        use feagi_genome_definitions::::{
            CoreCorticalType, CorticalArea, CorticalAreaDimensions,
        };
        use feagi_genome_definitions::descriptors::GenomeCoordinate3D;
        use parking_lot::RwLock;
        use std::collections::HashMap;
        use std::sync::Arc;

        // Isolated connectome manager instance for this test.
        let connectome = Arc::new(RwLock::new(
            feagi_brain_development::ConnectomeManager::new_for_testing(),
        ));

        // Use a known-valid cortical ID/type pair to avoid ID encoding intricacies in this unit test.
        let cortical_id = CoreCorticalType::Power.to_cortical_id();

        let dims = CorticalAreaDimensions::new(1, 1, 1).expect("dimensions must be valid");
        let pos = GenomeCoordinate3D::new(0, 0, 0);
        let cortical_type = cortical_id
            .as_cortical_type()
            .expect("cortical type must be derivable from id");

        let area = CorticalArea::new(
            cortical_id,
            0, // Let ConnectomeManager assign a proper idx
            "test_area".to_string(),
            dims,
            pos,
            cortical_type,
        )
        .expect("area must be valid");

        // Create a region that contains the test area.
        let region_id = RegionID::new();
        let region_key = region_id.to_string();
        let region = BrainRegion::new(region_id, "root".to_string(), RegionType::Undefined)
            .expect("region must be valid")
            .with_areas([cortical_id]);

        // Seed RuntimeGenome with the area + region membership (this is what genome save/export uses).
        let genome = feagi_evolutionary::RuntimeGenome {
            metadata: feagi_evolutionary::GenomeMetadata {
                genome_id: "test".to_string(),
                genome_title: "test".to_string(),
                genome_description: "".to_string(),
                version: "3.0".to_string(),
                timestamp: 0.0,
                brain_regions_root: Some(region_key.clone()),
            },
            cortical_areas: HashMap::from([(cortical_id, area.clone())]),
            brain_regions: HashMap::from([(region_key.clone(), region.clone())]),
            morphologies: feagi_evolutionary::MorphologyRegistry::new(),
            physiology: feagi_evolutionary::PhysiologyConfig::default(),
            signatures: feagi_evolutionary::GenomeSignatures {
                genome: "0".to_string(),
                blueprint: "0".to_string(),
                physiology: "0".to_string(),
                morphologies: None,
            },
            stats: feagi_evolutionary::GenomeStats::default(),
        };
        let current_genome = Arc::new(RwLock::new(Some(genome)));

        // Seed ConnectomeManager with the same region + area (this is what BV and runtime uses).
        {
            let mut mgr = connectome.write();
            mgr.add_brain_region(region, None)
                .expect("brain region should be addable");
            mgr.add_cortical_area(area)
                .expect("cortical area should be addable");
        }

        let svc = ConnectomeServiceImpl::new(connectome.clone(), current_genome.clone());

        // Act: delete by base64 string.
        let cortical_id_base64 = cortical_id.as_base_64();
        svc.delete_cortical_area(&cortical_id_base64).await?;

        // Assert: RuntimeGenome no longer contains the area nor region membership.
        {
            let genome_guard = current_genome.read();
            let genome = genome_guard.as_ref().expect("genome must exist");
            assert!(!genome.cortical_areas.contains_key(&cortical_id));
            let region = genome
                .brain_regions
                .get(&region_key)
                .expect("region must exist in genome");
            assert!(!region.contains_area(&cortical_id));
        }

        Ok(())
    }

    /// Deleting a region should remove descendants, areas, and RuntimeGenome entries.
    #[tokio::test]
    async fn delete_brain_region_deletes_descendants_and_persists() -> ServiceResult<()> {
        use super::ConnectomeServiceImpl;
        use crate::traits::ConnectomeService;
        use feagi_brain_development::ConnectomeManager;
        use feagi_genome_definitions::::{RegionID};
        use feagi_genome_definitions::::{
            CoreCorticalType, CorticalArea, CorticalAreaDimensions,
        };
        use feagi_genome_definitions::descriptors::GenomeCoordinate3D;
        use parking_lot::RwLock;
        use std::collections::HashMap;
        use std::sync::Arc;

        let connectome = Arc::new(RwLock::new(ConnectomeManager::new_for_testing()));

        let root_id = RegionID::new();
        let root_key = root_id.to_string();
        let child_id = RegionID::new();
        let child_key = child_id.to_string();
        let grandchild_id = RegionID::new();
        let grandchild_key = grandchild_id.to_string();

        let power_id = CoreCorticalType::Power.to_cortical_id();
        let death_id = CoreCorticalType::Death.to_cortical_id();

        let power_area = CorticalArea::new(
            power_id,
            0,
            "power_area".to_string(),
            CorticalAreaDimensions::new(1, 1, 1)?,
            GenomeCoordinate3D::new(0, 0, 0),
            power_id.as_cortical_type()?,
        )?;
        let death_area = CorticalArea::new(
            death_id,
            0,
            "death_area".to_string(),
            CorticalAreaDimensions::new(1, 1, 1)?,
            GenomeCoordinate3D::new(0, 0, 0),
            death_id.as_cortical_type()?,
        )?;

        let root = BrainRegion::new(root_id, "root".to_string(), RegionType::Undefined)?;
        let child = BrainRegion::new(child_id, "child".to_string(), RegionType::Undefined)?
            .with_areas([power_id]);
        let grandchild =
            BrainRegion::new(grandchild_id, "grand".to_string(), RegionType::Undefined)?
                .with_areas([death_id]);

        let genome = feagi_evolutionary::RuntimeGenome {
            metadata: feagi_evolutionary::GenomeMetadata {
                genome_id: "test".to_string(),
                genome_title: "test".to_string(),
                genome_description: "".to_string(),
                version: "3.0".to_string(),
                timestamp: 0.0,
                brain_regions_root: Some(root_key.clone()),
            },
            cortical_areas: HashMap::from([
                (power_id, power_area.clone()),
                (death_id, death_area.clone()),
            ]),
            brain_regions: HashMap::from([
                (root_key.clone(), root.clone()),
                (child_key.clone(), child.clone()),
                (grandchild_key.clone(), grandchild.clone()),
            ]),
            morphologies: feagi_evolutionary::MorphologyRegistry::new(),
            physiology: feagi_evolutionary::PhysiologyConfig::default(),
            signatures: feagi_evolutionary::GenomeSignatures {
                genome: "0".to_string(),
                blueprint: "0".to_string(),
                physiology: "0".to_string(),
                morphologies: None,
            },
            stats: feagi_evolutionary::GenomeStats::default(),
        };
        let current_genome = Arc::new(RwLock::new(Some(genome)));

        {
            let mut manager = connectome.write();
            manager.add_brain_region(root, None)?;
            manager.add_brain_region(child, Some(root_key.clone()))?;
            manager.add_brain_region(grandchild, Some(child_key.clone()))?;
            manager.add_cortical_area(power_area)?;
            manager.add_cortical_area(death_area)?;
        }

        let svc = ConnectomeServiceImpl::new(connectome.clone(), current_genome.clone());
        svc.delete_brain_region(&child_key).await?;

        {
            let manager = connectome.read();
            assert!(manager.get_brain_region(&child_key).is_none());
            assert!(manager.get_brain_region(&grandchild_key).is_none());
            assert!(manager.get_brain_region(&root_key).is_some());
            assert!(!manager.has_cortical_area(&power_id));
            assert!(!manager.has_cortical_area(&death_id));
        }

        {
            let genome_guard = current_genome.read();
            let genome = genome_guard.as_ref().expect("genome must exist");
            assert!(!genome.brain_regions.contains_key(&child_key));
            assert!(!genome.brain_regions.contains_key(&grandchild_key));
            assert!(genome.brain_regions.contains_key(&root_key));
            assert!(!genome.cortical_areas.contains_key(&power_id));
            assert!(!genome.cortical_areas.contains_key(&death_id));
        }

        Ok(())
    }

    /// Regression: deleting a cortical area must cascade-prune outgoing synapses owned by every
    /// source area that mapped into it. Prior to the cascade fix, `delete_cortical_area` cleared
    /// `cortical_mapping_dst` rules but never invoked `regenerate_synapses_for_mapping`, leaving
    /// orphaned synapses (with potentially saturated R-STDP weights) in the source area.
    ///
    /// This guards both directions:
    ///   1) src -> deleted_area synapses must be pruned (the explicit user-reported bug).
    ///   2) deleted_area -> downstream synapses must also be pruned (full cascade hygiene).
    #[tokio::test]
    async fn delete_cortical_area_cascade_prunes_orphan_synapses() -> ServiceResult<()> {
        use super::ConnectomeServiceImpl;
        use crate::traits::ConnectomeService;
        use feagi_brain_development::ConnectomeManager;
        use feagi_npu_burst_engine::backend::CPUBackend;
        use feagi_npu_burst_engine::{DynamicNPU, RustNPU, TracingMutex};
        use feagi_npu_runtime::StdRuntime;
        use feagi_genome_definitions::::{
            CorticalArea, CorticalAreaDimensions, CorticalAreaType, CorticalID,
            IOCorticalAreaConfigurationFlag,
        };
        use parking_lot::RwLock;
        use std::sync::Arc;

        let runtime = StdRuntime;
        let backend = CPUBackend::new();
        let npu = RustNPU::new(runtime, backend, 10_000, 10_000, 10).expect("npu construct");
        let dyn_npu = Arc::new(TracingMutex::new(DynamicNPU::F32(npu), "TestNPU"));

        let mut mgr = ConnectomeManager::new_for_testing_with_npu(dyn_npu.clone());

        // src --(mapping)--> doomed --(mapping)--> sink. After deleting `doomed`, neither src's
        // outgoing synapses nor doomed's outgoing synapses may remain in the NPU.
        let src_id = CorticalID::try_from_bytes(b"cstcsrc1").expect("src id");
        let doomed_id = CorticalID::try_from_bytes(b"cstcdoom").expect("doomed id");
        let sink_id = CorticalID::try_from_bytes(b"cstsink1").expect("sink id");

        for (id, label) in [(src_id, "src"), (doomed_id, "doomed"), (sink_id, "sink")] {
            let area = CorticalArea::new(
                id,
                0,
                label.to_string(),
                CorticalAreaDimensions::new(2, 1, 1).unwrap(),
                (0, 0, 0).into(),
                CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean),
            )
            .unwrap();
            mgr.add_cortical_area(area).unwrap();
        }

        // Two neurons per area for non-trivial fan-out.
        let s0 = mgr
            .add_neuron(&src_id, 0, 0, 0, 1.0, 0.0, 0.1, 0.0, 0, 1, 1.0, 3, 1, false)
            .unwrap();
        let s1 = mgr
            .add_neuron(&src_id, 1, 0, 0, 1.0, 0.0, 0.1, 0.0, 0, 1, 1.0, 3, 1, false)
            .unwrap();
        let d0 = mgr
            .add_neuron(
                &doomed_id, 0, 0, 0, 1.0, 0.0, 0.1, 0.0, 0, 1, 1.0, 3, 1, false,
            )
            .unwrap();
        let d1 = mgr
            .add_neuron(
                &doomed_id, 1, 0, 0, 1.0, 0.0, 0.1, 0.0, 0, 1, 1.0, 3, 1, false,
            )
            .unwrap();
        let k0 = mgr
            .add_neuron(
                &sink_id, 0, 0, 0, 1.0, 0.0, 0.1, 0.0, 0, 1, 1.0, 3, 1, false,
            )
            .unwrap();

        // Pre-existing synapses src->doomed (the orphan-source path) and doomed->sink (the other
        // direction we also clean up). Use heavy fake weights to mimic R-STDP saturation.
        mgr.create_synapse(s0, d0, 882_000.0, 200.0, 0).unwrap();
        mgr.create_synapse(s1, d1, 882_000.0, 200.0, 0).unwrap();
        mgr.create_synapse(d0, k0, 1.0, 1.0, 0).unwrap();

        // Register the mapping rule entries so `delete_cortical_area` can discover the pairs to
        // prune via `cortical_mapping_dst` lookups.
        mgr.update_cortical_mapping(
            &src_id,
            &doomed_id,
            vec![serde_json::json!({"morphology_id": "all_to_all"})],
        )
        .unwrap();
        mgr.update_cortical_mapping(
            &doomed_id,
            &sink_id,
            vec![serde_json::json!({"morphology_id": "all_to_all"})],
        )
        .unwrap();

        {
            let mut npu = dyn_npu.lock().unwrap();
            npu.rebuild_synapse_index();
            assert_eq!(
                npu.get_synapse_count(),
                3,
                "fixture should expose 3 synapses"
            );
            assert_eq!(npu.get_outgoing_synapses(s0 as u32).len(), 1);
            assert_eq!(npu.get_outgoing_synapses(s1 as u32).len(), 1);
            assert_eq!(npu.get_outgoing_synapses(d0 as u32).len(), 1);
        }

        let connectome = Arc::new(RwLock::new(mgr));
        let current_genome = Arc::new(RwLock::new(None));
        let svc = ConnectomeServiceImpl::new(connectome.clone(), current_genome.clone());

        svc.delete_cortical_area(&doomed_id.as_base_64()).await?;

        {
            let mut npu = dyn_npu.lock().unwrap();
            npu.rebuild_synapse_index();
            assert_eq!(
                npu.get_synapse_count(),
                0,
                "delete_cortical_area must cascade-prune all synapses involving the deleted area"
            );
            assert!(
                npu.get_outgoing_synapses(s0 as u32).is_empty(),
                "src->doomed orphan synapses must be cleared from src"
            );
            assert!(
                npu.get_outgoing_synapses(s1 as u32).is_empty(),
                "src->doomed orphan synapses must be cleared from src"
            );
            assert!(
                npu.get_outgoing_synapses(d0 as u32).is_empty(),
                "doomed->sink synapses must also be pruned"
            );
        }

        // Connectome should no longer expose the deleted area, and src's mapping_dst entry for the
        // doomed area must be removed (to keep RuntimeGenome consistent on subsequent saves).
        {
            let mgr = connectome.read();
            assert!(!mgr.has_cortical_area(&doomed_id));
            let src = mgr.get_cortical_area(&src_id).expect("src must remain");
            let mapping_dst = src.properties.get("cortical_mapping_dst");
            if let Some(mapping_dst) = mapping_dst {
                assert!(
                    !mapping_dst
                        .as_object()
                        .map(|m| m.contains_key(&doomed_id.as_base_64()))
                        .unwrap_or(false),
                    "src must not retain a mapping rule pointing at the deleted area"
                );
            }
        }

        Ok(())
    }

    /// Regression: `update_cortical_mapping` with an empty rule list (the canonical path used by
    /// the `DELETE /v1/cortical_mapping/mapping` endpoint) must prune all existing synapses for
    /// that src->dst pair, even when the previous mapping has no entry yet (guards a TOCTOU edge
    /// where `existing_mapping` is `None` and the early-return-on-equal-rules optimization would
    /// otherwise short-circuit).
    #[tokio::test]
    async fn delete_mapping_via_empty_update_clears_synapses() -> ServiceResult<()> {
        use super::ConnectomeServiceImpl;
        use crate::traits::ConnectomeService;
        use feagi_brain_development::ConnectomeManager;
        use feagi_npu_burst_engine::backend::CPUBackend;
        use feagi_npu_burst_engine::{DynamicNPU, RustNPU, TracingMutex};
        use feagi_npu_runtime::StdRuntime;
        use feagi_genome_definitions::::{
            CorticalArea, CorticalAreaDimensions, CorticalAreaType, CorticalID,
            IOCorticalAreaConfigurationFlag,
        };
        use parking_lot::RwLock;
        use std::sync::Arc;

        let runtime = StdRuntime;
        let backend = CPUBackend::new();
        let npu = RustNPU::new(runtime, backend, 10_000, 10_000, 10).expect("npu construct");
        let dyn_npu = Arc::new(TracingMutex::new(DynamicNPU::F32(npu), "TestNPU"));
        let mut mgr = ConnectomeManager::new_for_testing_with_npu(dyn_npu.clone());

        let src_id = CorticalID::try_from_bytes(b"cstdms01").expect("src id");
        let dst_id = CorticalID::try_from_bytes(b"cstdmd01").expect("dst id");

        for (id, label) in [(src_id, "src"), (dst_id, "dst")] {
            mgr.add_cortical_area(
                CorticalArea::new(
                    id,
                    0,
                    label.to_string(),
                    CorticalAreaDimensions::new(2, 1, 1).unwrap(),
                    (0, 0, 0).into(),
                    CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean),
                )
                .unwrap(),
            )
            .unwrap();
        }
        let s0 = mgr
            .add_neuron(&src_id, 0, 0, 0, 1.0, 0.0, 0.1, 0.0, 0, 1, 1.0, 3, 1, false)
            .unwrap();
        let t0 = mgr
            .add_neuron(&dst_id, 0, 0, 0, 1.0, 0.0, 0.1, 0.0, 0, 1, 1.0, 3, 1, false)
            .unwrap();
        mgr.create_synapse(s0, t0, 882_000.0, 200.0, 0).unwrap();
        mgr.update_cortical_mapping(
            &src_id,
            &dst_id,
            vec![serde_json::json!({"morphology_id": "all_to_all"})],
        )
        .unwrap();
        {
            let mut npu = dyn_npu.lock().unwrap();
            npu.rebuild_synapse_index();
            assert_eq!(npu.get_synapse_count(), 1);
        }

        let connectome = Arc::new(RwLock::new(mgr));
        let current_genome = Arc::new(RwLock::new(None));
        let svc = ConnectomeServiceImpl::new(connectome.clone(), current_genome.clone());

        // Drive the service path used by `DELETE /v1/cortical_mapping/mapping`.
        svc.update_cortical_mapping(src_id.as_base_64(), dst_id.as_base_64(), vec![])
            .await?;

        {
            let mut npu = dyn_npu.lock().unwrap();
            npu.rebuild_synapse_index();
            assert_eq!(
                npu.get_synapse_count(),
                0,
                "DELETE mapping must drop all synapses for the src->dst pair"
            );
            assert!(npu.get_outgoing_synapses(s0 as u32).is_empty());
        }

        Ok(())
    }
}
