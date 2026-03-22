// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Shared registration helpers used across transports.

use crate::common::ApiState;
use feagi_config::load_config;
use feagi_services::types::CreateCorticalAreaParams;
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalSubUnitIndex, CorticalUnitIndex,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, PercentageNeuronPositioning,
};
use feagi_structures::genomic::{MotorCorticalUnit, SensoryCorticalUnit};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use tracing::{info, warn};

const MOTOR_AREA_X_GAP_VOXELS: i32 = 10;

fn build_friendly_unit_name(unit_label: &str, group: u8, sub_unit_index: usize) -> String {
    format!("{unit_label}-{}-{}", group, sub_unit_index)
}

fn non_empty_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
}

fn extract_grouping_array(unit_def: &Value) -> &[Value] {
    unit_def
        .get("device_grouping")
        .and_then(|v| v.as_array())
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn first_grouping_property(unit_def: &Value, key: &str) -> Option<String> {
    extract_grouping_array(unit_def)
        .iter()
        .find_map(|grouping| {
            non_empty_string(
                grouping
                    .get("device_properties")
                    .and_then(|v| v.as_object())
                    .and_then(|props| props.get(key)),
            )
        })
}

fn resolve_registration_name(unit_def: &Value, default_name: &str) -> String {
    non_empty_string(unit_def.get("friendly_name"))
        .or_else(|| first_grouping_property(unit_def, "bundle_id"))
        .or_else(|| {
            non_empty_string(
                extract_grouping_array(unit_def)
                    .first()?
                    .get("friendly_name"),
            )
        })
        .unwrap_or_else(|| default_name.to_string())
}

fn should_auto_rename(current_name: &str, cortical_id: &str, legacy_default_name: &str) -> bool {
    current_name == cortical_id || current_name == legacy_default_name
}

fn build_io_config_map() -> Result<serde_json::Map<String, serde_json::Value>, String> {
    let mut config = serde_json::Map::new();
    config.insert(
        "frame_change_handling".to_string(),
        serde_json::to_value(FrameChangeHandling::Absolute)
            .map_err(|e| format!("Failed to serialize FrameChangeHandling: {}", e))?,
    );
    config.insert(
        "percentage_neuron_positioning".to_string(),
        serde_json::to_value(PercentageNeuronPositioning::Linear)
            .map_err(|e| format!("Failed to serialize PercentageNeuronPositioning: {}", e))?,
    );
    Ok(config)
}

fn build_io_config_map_from_unit_def(
    unit_def: &Value,
) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    let io_flags = unit_def
        .get("io_configuration_flags")
        .and_then(|v| v.as_object());

    let frame_value = io_flags
        .and_then(|flags| flags.get("frame_change_handling"))
        .cloned()
        .or_else(|| unit_def.get("frame_change_handling").cloned())
        .ok_or_else(|| "unit_def missing frame_change_handling".to_string())?;
    let positioning_value = io_flags
        .and_then(|flags| flags.get("percentage_neuron_positioning"))
        .cloned()
        .or_else(|| unit_def.get("percentage_neuron_positioning").cloned())
        .ok_or_else(|| "unit_def missing percentage_neuron_positioning".to_string())?;

    let frame: FrameChangeHandling = serde_json::from_value(frame_value)
        .map_err(|e| format!("Invalid frame_change_handling value: {}", e))?;
    let positioning: PercentageNeuronPositioning = serde_json::from_value(positioning_value)
        .map_err(|e| format!("Invalid percentage_neuron_positioning value: {}", e))?;

    let mut config = serde_json::Map::new();
    config.insert(
        "frame_change_handling".to_string(),
        serde_json::to_value(frame)
            .map_err(|e| format!("Failed to serialize FrameChangeHandling: {}", e))?,
    );
    config.insert(
        "percentage_neuron_positioning".to_string(),
        serde_json::to_value(positioning)
            .map_err(|e| format!("Failed to serialize PercentageNeuronPositioning: {}", e))?,
    );
    Ok(config)
}

fn as_nonzero_usize(value: Option<&Value>) -> Option<usize> {
    value
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .filter(|v| *v > 0)
}

fn color_channel_count_from_value(value: Option<&Value>) -> Option<usize> {
    match value {
        Some(Value::String(layout)) => match layout.as_str() {
            "GrayScale" => Some(1),
            "RG" => Some(2),
            "RGB" => Some(3),
            "RGBA" => Some(4),
            _ => None,
        },
        Some(Value::Number(number)) => number
            .as_u64()
            .map(|v| v as usize)
            .filter(|v| (1..=4).contains(v)),
        _ => None,
    }
}

fn encoder_variant_payload<'a>(encoder_properties: &'a Value, variant: &str) -> Option<&'a Value> {
    let object = encoder_properties.as_object()?;
    if let Some(payload) = object.get(variant) {
        return Some(payload);
    }
    // Accept already-unwrapped payloads when variant tagging is stripped.
    if variant == "CartesianPlane" && object.contains_key("image_resolution") {
        return Some(encoder_properties);
    }
    if variant == "SegmentedImageFrame" && object.contains_key("segment_xy_resolutions") {
        return Some(encoder_properties);
    }
    None
}

fn extract_cartesian_plane_dimensions(encoder_properties: &Value) -> Option<(usize, usize, usize)> {
    let payload = encoder_variant_payload(encoder_properties, "CartesianPlane")?;
    let resolution = payload.get("image_resolution")?;
    let width = as_nonzero_usize(resolution.get("width"))?;
    let height = as_nonzero_usize(resolution.get("height"))?;
    let channels = color_channel_count_from_value(payload.get("color_channel_layout"))?;
    Some((width, height, channels))
}

fn extract_segmented_vision_dimensions(
    encoder_properties: &Value,
    sub_unit_index: usize,
) -> Option<(usize, usize, usize)> {
    let payload = encoder_variant_payload(encoder_properties, "SegmentedImageFrame")?;
    let resolutions = payload.get("segment_xy_resolutions")?.as_object()?;
    let segment_key = match sub_unit_index {
        0 => "lower_left",
        1 => "lower_middle",
        2 => "lower_right",
        3 => "middle_left",
        4 => "center",
        5 => "middle_right",
        6 => "upper_left",
        7 => "upper_middle",
        8 => "upper_right",
        _ => return None,
    };
    let segment_resolution = resolutions.get(segment_key)?;
    let width = as_nonzero_usize(segment_resolution.get("width"))?;
    let height = as_nonzero_usize(segment_resolution.get("height"))?;
    let channels = if sub_unit_index == 4 {
        color_channel_count_from_value(payload.get("center_color_channel"))?
    } else {
        color_channel_count_from_value(payload.get("peripheral_color_channels"))?
    };
    Some((width, height, channels))
}

fn resolve_sensory_dimensions_from_encoder_properties(
    encoder_properties: Option<&Value>,
    sub_unit_index: usize,
    fallback: (usize, usize, usize),
) -> (usize, usize, usize) {
    let Some(encoder_properties) = encoder_properties else {
        return fallback;
    };
    extract_cartesian_plane_dimensions(encoder_properties)
        .or_else(|| extract_segmented_vision_dimensions(encoder_properties, sub_unit_index))
        .unwrap_or(fallback)
}

pub async fn auto_create_cortical_areas_from_device_registrations(
    state: &ApiState,
    device_registrations: &serde_json::Value,
) {
    let config = match load_config(None, None) {
        Ok(config) => config,
        Err(e) => {
            warn!(
                "⚠️ [API] Failed to load FEAGI configuration for auto-create: {}",
                e
            );
            return;
        }
    };

    if !config.agent.auto_create_missing_cortical_areas {
        return;
    }

    let connectome_service = state.connectome_service.as_ref();
    let genome_service = state.genome_service.as_ref();

    // Get root region ID so auto-created OPU/IPU areas appear in root (fixes power area disappearing in BV)
    let root_region_id = connectome_service.get_root_region_id().await.ok().flatten();

    let output_units = device_registrations
        .get("output_units_and_decoder_properties")
        .and_then(|v| v.as_object());
    let input_units = device_registrations
        .get("input_units_and_encoder_properties")
        .and_then(|v| v.as_object());
    if output_units.is_none() && input_units.is_none() {
        return;
    }

    // Build creation params for missing OPU areas based on default topologies.
    let mut to_create: Vec<CreateCorticalAreaParams> = Vec::new();

    if let Some(output_units) = output_units {
        for (motor_unit_key, unit_defs) in output_units {
            // MotorCorticalUnit is serde-deserializable from its string representation.
            let motor_unit: MotorCorticalUnit = match serde_json::from_value::<MotorCorticalUnit>(
                serde_json::Value::String(motor_unit_key.clone()),
            ) {
                Ok(v) => v,
                Err(e) => {
                    warn!(
                    "⚠️ [API] Unable to parse MotorCorticalUnit key '{}' from device_registrations: {}",
                    motor_unit_key, e
                );
                    continue;
                }
            };

            let Some(unit_defs_arr) = unit_defs.as_array() else {
                continue;
            };

            for entry in unit_defs_arr {
                // Expected shape: [<unit_definition>, <decoder_properties>]
                let Some(pair) = entry.as_array() else {
                    continue;
                };
                let Some(unit_def) = pair.first() else {
                    continue;
                };
                let Some(group_u64) = unit_def.get("cortical_unit_index").and_then(|v| v.as_u64())
                else {
                    continue;
                };
                let group_u8: u8 = match group_u64.try_into() {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let group: CorticalUnitIndex = group_u8.into();

                let device_count = unit_def
                    .get("device_grouping")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                if device_count == 0 {
                    warn!(
                    "⚠️ [API] device_grouping is empty for motor unit '{}' group {}; skipping auto-create",
                    motor_unit_key, group_u8
                );
                    continue;
                }

                let config_map = match build_io_config_map_from_unit_def(unit_def) {
                    Ok(map) => map,
                    Err(e) => {
                        warn!(
                            "⚠️ [API] Failed to build motor IO config map from registration for '{}' group {}: {}",
                            motor_unit_key, group_u8, e
                        );
                        continue;
                    }
                };
                let topology = motor_unit.get_unit_default_topology();

                let cortical_ids = match motor_unit
                    .get_cortical_id_vector_from_index_and_serde_io_configuration_flags(
                        group, config_map,
                    ) {
                    Ok(ids) => ids,
                    Err(e) => {
                        warn!(
                            "⚠️ [API] Failed to derive motor cortical IDs for '{}' group {}: {}",
                            motor_unit_key, group_u8, e
                        );
                        continue;
                    }
                };

                // Precompute dimensions and positions for all sub-areas in this
                // motor unit/group. Keep a guaranteed X-gap between neighboring
                // areas regardless of their computed width.
                let mut expected_dimensions_by_sub: Vec<Option<(usize, usize, usize)>> =
                    vec![None; cortical_ids.len()];
                let mut expected_position_by_sub: Vec<Option<(i32, i32, i32)>> =
                    vec![None; cortical_ids.len()];
                let mut previous_position_x: Option<i32> = None;
                let mut previous_width: Option<i32> = None;
                for i in 0..cortical_ids.len() {
                    let sub_index = CorticalSubUnitIndex::from(i as u8);
                    let Some(unit_topology) = topology.get(&sub_index) else {
                        continue;
                    };
                    let per_channel_width = unit_topology.channel_dimensions_default[0] as usize;
                    let per_channel_height = unit_topology.channel_dimensions_default[1] as usize;
                    let per_channel_depth = unit_topology.channel_dimensions_default[2] as usize;
                    let expected_dimensions = (
                        (per_channel_width * device_count).max(1),
                        per_channel_height,
                        per_channel_depth,
                    );
                    expected_dimensions_by_sub[i] = Some(expected_dimensions);

                    let y = unit_topology.relative_position[1] + (group_u8 as i32 * 20);
                    let z = unit_topology.relative_position[2];
                    let width_i32 = expected_dimensions.0 as i32;
                    let x = if let (Some(prev_x), Some(_prev_w)) =
                        (previous_position_x, previous_width)
                    {
                        // Areas are anchored from their minimum X and extend to +X.
                        // To keep a fixed empty gap when placing current area on the
                        // left of previous area:
                        // current_x + current_width + gap <= previous_x
                        prev_x - width_i32 - MOTOR_AREA_X_GAP_VOXELS
                    } else {
                        unit_topology.relative_position[0]
                    };
                    expected_position_by_sub[i] = Some((x, y, z));
                    previous_position_x = Some(x);
                    previous_width = Some(width_i32);
                }

                for (i, cortical_id) in cortical_ids.iter().enumerate() {
                    let cortical_id_b64 = cortical_id.as_base_64();
                    let legacy_default_name =
                        build_friendly_unit_name(motor_unit.get_friendly_name(), group_u8, i);
                    let resolved_base_name =
                        resolve_registration_name(unit_def, &legacy_default_name);
                    let resolved_name =
                        if resolved_base_name == legacy_default_name || cortical_ids.len() == 1 {
                            resolved_base_name.clone()
                        } else {
                            format!("{}-{}", resolved_base_name, i)
                        };
                    let exists = match connectome_service
                        .cortical_area_exists(&cortical_id_b64)
                        .await
                    {
                        Ok(v) => v,
                        Err(e) => {
                            warn!(
                                "⚠️ [API] Failed to check cortical area existence for '{}': {}",
                                cortical_id_b64, e
                            );
                            continue;
                        }
                    };

                    let sub_index = CorticalSubUnitIndex::from(i as u8);
                    let unit_topology = match topology.get(&sub_index) {
                        Some(t) => t,
                        None => {
                            warn!(
                                "⚠️ [API] Missing unit topology for motor unit '{}' subunit {}; skipping",
                                motor_unit_key, i
                            );
                            continue;
                        }
                    };
                    let expected_position = match expected_position_by_sub.get(i).and_then(|v| *v) {
                        Some(pos) => pos,
                        None => {
                            warn!(
                                "⚠️ [API] Missing precomputed motor position for '{}' subunit {}; skipping",
                                motor_unit_key, i
                            );
                            continue;
                        }
                    };
                    let expected_dimensions = match expected_dimensions_by_sub
                        .get(i)
                        .and_then(|v| *v)
                    {
                        Some(dims) => dims,
                        None => {
                            warn!(
                                "⚠️ [API] Missing precomputed motor dimensions for '{}' subunit {}; skipping",
                                motor_unit_key, i
                            );
                            continue;
                        }
                    };

                    if exists {
                        // Area exists: ensure dimensions and dev_count match device_registrations.
                        // If a genome was loaded with wrong dimensions (e.g. 1 channel per limb),
                        // update to the correct channel count from device_grouping.
                        let current =
                            match connectome_service.get_cortical_area(&cortical_id_b64).await {
                                Ok(v) => v,
                                Err(e) => {
                                    warn!(
                                        "⚠️ [API] Failed to fetch existing cortical area '{}': {}",
                                        cortical_id_b64, e
                                    );
                                    continue;
                                }
                            };

                        let current_dev_count = current
                            .properties
                            .get("dev_count")
                            .and_then(|v| v.as_u64())
                            .map(|u| u as usize)
                            .or(current.dev_count);
                        let dimensions_mismatch = current.dimensions != expected_dimensions;
                        let dev_count_mismatch = current_dev_count != Some(device_count);
                        let position_mismatch = current.position != expected_position;

                        if dimensions_mismatch || dev_count_mismatch || position_mismatch {
                            let mut changes: HashMap<String, serde_json::Value> = HashMap::new();
                            // Pass total dimensions. Do NOT pass cortical_dimensions_per_device here:
                            // genome service would treat it as per-device and multiply depth by dev_count.
                            changes.insert(
                                "dimensions".to_string(),
                                serde_json::json!([
                                    expected_dimensions.0,
                                    expected_dimensions.1,
                                    expected_dimensions.2
                                ]),
                            );
                            changes.insert(
                                "dev_count".to_string(),
                                serde_json::Value::Number(serde_json::Number::from(device_count)),
                            );
                            changes.insert(
                                "position".to_string(),
                                serde_json::json!([
                                    expected_position.0,
                                    expected_position.1,
                                    expected_position.2
                                ]),
                            );
                            if let Err(e) = genome_service
                                .update_cortical_area(&cortical_id_b64, changes)
                                .await
                            {
                                warn!(
                                    "⚠️ [API] Failed to update cortical area '{}' dimensions/dev_count/position: {}",
                                    cortical_id_b64, e
                                );
                            } else {
                                info!(
                                    "[API] Updated cortical area '{}' to {} channels (dimensions {:?}, position {:?})",
                                    cortical_id_b64, device_count, expected_dimensions, expected_position
                                );
                            }
                        }

                        // Auto-rename if current name is placeholder (== cortical_id).
                        if should_auto_rename(&current.name, &cortical_id_b64, &legacy_default_name)
                        {
                            let desired_name = resolved_name.clone();
                            let mut changes: HashMap<String, serde_json::Value> = HashMap::new();
                            changes.insert(
                                "name".to_string(),
                                serde_json::Value::String(desired_name),
                            );
                            if let Err(e) = genome_service
                                .update_cortical_area(&cortical_id_b64, changes)
                                .await
                            {
                                warn!(
                                    "⚠️ [API] Failed to auto-rename existing motor cortical area '{}': {}",
                                    cortical_id_b64, e
                                );
                            }
                        }
                        continue;
                    }

                    let friendly_name = resolved_name;
                    // Use template-defined per-channel topology and scale by device_count.
                    // This keeps sizing fully template-driven and consistent across all unit types.
                    let (per_channel_width, per_channel_height, per_channel_depth) = (
                        unit_topology.channel_dimensions_default[0] as usize,
                        unit_topology.channel_dimensions_default[1] as usize,
                        unit_topology.channel_dimensions_default[2] as usize,
                    );
                    let dimensions = expected_dimensions;
                    let per_device_dims =
                        (per_channel_width, per_channel_height, per_channel_depth);
                    let position = expected_position;

                    let mut properties = HashMap::new();
                    properties.insert(
                        "dev_count".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(device_count)),
                    );
                    properties.insert(
                        "cortical_dimensions_per_device".to_string(),
                        serde_json::json!([
                            per_device_dims.0,
                            per_device_dims.1,
                            per_device_dims.2
                        ]),
                    );
                    if let Some(unit_name) = non_empty_string(unit_def.get("friendly_name")) {
                        properties.insert(
                            "registration_unit_friendly_name".to_string(),
                            serde_json::Value::String(unit_name),
                        );
                    }
                    if let Some(bundle_id) = first_grouping_property(unit_def, "bundle_id") {
                        properties.insert(
                            "registration_bundle_id".to_string(),
                            serde_json::Value::String(bundle_id),
                        );
                    }
                    if let Some(bundle_type) = first_grouping_property(unit_def, "bundle_type") {
                        properties.insert(
                            "registration_bundle_type".to_string(),
                            serde_json::Value::String(bundle_type),
                        );
                    }
                    if let Some(ref rid) = root_region_id {
                        properties.insert(
                            "parent_region_id".to_string(),
                            serde_json::Value::String(rid.clone()),
                        );
                    }

                    to_create.push(CreateCorticalAreaParams {
                        cortical_id: cortical_id_b64.clone(),
                        name: friendly_name,
                        dimensions,
                        position,
                        area_type: "motor".to_string(),
                        visible: None,
                        sub_group: None,
                        neurons_per_voxel: None,
                        postsynaptic_current: None,
                        plasticity_constant: None,
                        degeneration: None,
                        psp_uniform_distribution: None,
                        firing_threshold_increment: None,
                        firing_threshold_limit: None,
                        consecutive_fire_count: None,
                        snooze_period: None,
                        refractory_period: None,
                        leak_coefficient: None,
                        leak_variability: None,
                        burst_engine_active: None,
                        properties: Some(properties),
                    });
                }
            }
        }
    }

    // Build creation params for missing IPU areas based on default topologies.
    if let Some(input_units) = input_units {
        for (sensory_unit_key, unit_defs) in input_units {
            let sensory_unit: SensoryCorticalUnit = match serde_json::from_value::<
                SensoryCorticalUnit,
            >(serde_json::Value::String(
                sensory_unit_key.clone(),
            )) {
                Ok(v) => v,
                Err(e) => {
                    warn!(
                            "⚠️ [API] Unable to parse SensoryCorticalUnit key '{}' from device_registrations: {}",
                            sensory_unit_key, e
                        );
                    continue;
                }
            };

            let Some(unit_defs_arr) = unit_defs.as_array() else {
                continue;
            };

            for entry in unit_defs_arr {
                // Expected shape: [<unit_definition>, <encoder_properties>]
                let Some(pair) = entry.as_array() else {
                    continue;
                };
                let Some(unit_def) = pair.first() else {
                    continue;
                };
                let encoder_properties = pair.get(1);
                let Some(group_u64) = unit_def.get("cortical_unit_index").and_then(|v| v.as_u64())
                else {
                    continue;
                };
                let group_u8: u8 = match group_u64.try_into() {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let group: CorticalUnitIndex = group_u8.into();

                let device_count = unit_def
                    .get("device_grouping")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                if device_count == 0 {
                    warn!(
                        "⚠️ [API] device_grouping is empty for sensory unit '{}' group {}; skipping auto-create",
                        sensory_unit_key, group_u8
                    );
                    continue;
                }

                let config_map = match build_io_config_map() {
                    Ok(map) => map,
                    Err(e) => {
                        warn!(
                            "⚠️ [API] Failed to build sensory IO config map for '{}' group {}: {}",
                            sensory_unit_key, group_u8, e
                        );
                        continue;
                    }
                };
                let cortical_ids = match sensory_unit
                    .get_cortical_id_vector_from_index_and_serde_io_configuration_flags(
                        group, config_map,
                    ) {
                    Ok(ids) => ids,
                    Err(e) => {
                        warn!(
                            "⚠️ [API] Failed to derive sensory cortical IDs for '{}' group {}: {}",
                            sensory_unit_key, group_u8, e
                        );
                        continue;
                    }
                };
                let topology = sensory_unit.get_unit_default_topology();

                for (i, cortical_id) in cortical_ids.iter().enumerate() {
                    let cortical_id_b64 = cortical_id.as_base_64();
                    let sub_index = CorticalSubUnitIndex::from(i as u8);
                    let unit_topology = match topology.get(&sub_index) {
                        Some(topology) => topology,
                        None => {
                            warn!(
                                "⚠️ [API] Missing unit topology for sensory unit '{}' subunit {} (agent device_registrations); cannot auto-create/update '{}'",
                                sensory_unit_key, i, cortical_id_b64
                            );
                            continue;
                        }
                    };
                    let expected_dimensions = resolve_sensory_dimensions_from_encoder_properties(
                        encoder_properties,
                        i,
                        (
                            unit_topology.channel_dimensions_default[0] as usize,
                            unit_topology.channel_dimensions_default[1] as usize,
                            unit_topology.channel_dimensions_default[2] as usize,
                        ),
                    );
                    let expected_position = (
                        unit_topology.relative_position[0],
                        unit_topology.relative_position[1],
                        unit_topology.relative_position[2],
                    );
                    let legacy_default_name =
                        build_friendly_unit_name(sensory_unit.get_friendly_name(), group_u8, i);
                    let resolved_base_name =
                        resolve_registration_name(unit_def, &legacy_default_name);
                    let resolved_name =
                        if resolved_base_name == legacy_default_name || cortical_ids.len() == 1 {
                            resolved_base_name.clone()
                        } else {
                            format!("{}-{}", resolved_base_name, i)
                        };
                    let exists = match connectome_service
                        .cortical_area_exists(&cortical_id_b64)
                        .await
                    {
                        Ok(v) => v,
                        Err(e) => {
                            warn!(
                                "⚠️ [API] Failed to check cortical area existence for '{}': {}",
                                cortical_id_b64, e
                            );
                            continue;
                        }
                    };

                    if exists {
                        // Area exists: reconcile dimensions/dev_count/position with current registration.
                        // This keeps pre-existing sensory areas aligned with declared capabilities.
                        let current = match connectome_service
                            .get_cortical_area(&cortical_id_b64)
                            .await
                        {
                            Ok(v) => v,
                            Err(e) => {
                                warn!(
                                    "⚠️ [API] Failed to fetch existing cortical area '{}' for potential rename: {}",
                                    cortical_id_b64, e
                                );
                                continue;
                            }
                        };
                        let current_dev_count = current
                            .properties
                            .get("dev_count")
                            .and_then(|v| v.as_u64())
                            .map(|u| u as usize)
                            .or(current.dev_count);
                        let dimensions_mismatch = current.dimensions != expected_dimensions;
                        let dev_count_mismatch = current_dev_count != Some(device_count);
                        let position_mismatch = current.position != expected_position;
                        if dimensions_mismatch || dev_count_mismatch || position_mismatch {
                            let mut changes: HashMap<String, serde_json::Value> = HashMap::new();
                            changes.insert(
                                "dimensions".to_string(),
                                serde_json::json!([
                                    expected_dimensions.0,
                                    expected_dimensions.1,
                                    expected_dimensions.2
                                ]),
                            );
                            changes.insert(
                                "dev_count".to_string(),
                                serde_json::Value::Number(serde_json::Number::from(device_count)),
                            );
                            changes.insert(
                                "position".to_string(),
                                serde_json::json!([
                                    expected_position.0,
                                    expected_position.1,
                                    expected_position.2
                                ]),
                            );
                            if let Err(e) = genome_service
                                .update_cortical_area(&cortical_id_b64, changes)
                                .await
                            {
                                warn!(
                                    "⚠️ [API] Failed to update sensory cortical area '{}' dimensions/dev_count/position: {}",
                                    cortical_id_b64, e
                                );
                            } else {
                                info!(
                                    "[API] Updated sensory cortical area '{}' to registration dimensions {:?} (dev_count {}, position {:?})",
                                    cortical_id_b64, expected_dimensions, device_count, expected_position
                                );
                            }
                        }

                        // If the area already exists but still has a placeholder name (often equal to the cortical_id),
                        // update it to a deterministic friendly name so UIs (e.g., Brain Visualizer) show readable labels.
                        // IMPORTANT: We only auto-rename if the current name is clearly a placeholder.
                        if should_auto_rename(&current.name, &cortical_id_b64, &legacy_default_name)
                        {
                            let desired_name = resolved_name.clone();
                            let mut changes: HashMap<String, serde_json::Value> = HashMap::new();
                            changes.insert(
                                "name".to_string(),
                                serde_json::Value::String(desired_name),
                            );
                            if let Err(e) = genome_service
                                .update_cortical_area(&cortical_id_b64, changes)
                                .await
                            {
                                warn!(
                                    "⚠️ [API] Failed to auto-rename existing sensory cortical area '{}': {}",
                                    cortical_id_b64, e
                                );
                            }
                        }
                        continue;
                    }

                    let friendly_name = resolved_name;
                    let dimensions = expected_dimensions;
                    let position = expected_position;
                    let mut properties: HashMap<String, serde_json::Value> = HashMap::new();
                    properties.insert(
                        "cortical_subunit_index".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(sub_index.get())),
                    );
                    properties.insert(
                        "dev_count".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(device_count)),
                    );
                    if let Some(unit_name) = non_empty_string(unit_def.get("friendly_name")) {
                        properties.insert(
                            "registration_unit_friendly_name".to_string(),
                            serde_json::Value::String(unit_name),
                        );
                    }
                    if let Some(bundle_id) = first_grouping_property(unit_def, "bundle_id") {
                        properties.insert(
                            "registration_bundle_id".to_string(),
                            serde_json::Value::String(bundle_id),
                        );
                    }
                    if let Some(bundle_type) = first_grouping_property(unit_def, "bundle_type") {
                        properties.insert(
                            "registration_bundle_type".to_string(),
                            serde_json::Value::String(bundle_type),
                        );
                    }
                    if let Some(ref rid) = root_region_id {
                        properties.insert(
                            "parent_region_id".to_string(),
                            serde_json::Value::String(rid.clone()),
                        );
                    }

                    to_create.push(CreateCorticalAreaParams {
                        cortical_id: cortical_id_b64.clone(),
                        name: friendly_name,
                        dimensions,
                        position,
                        area_type: "sensory".to_string(),
                        visible: None,
                        sub_group: None,
                        neurons_per_voxel: None,
                        postsynaptic_current: None,
                        plasticity_constant: None,
                        degeneration: None,
                        psp_uniform_distribution: None,
                        firing_threshold_increment: None,
                        firing_threshold_limit: None,
                        consecutive_fire_count: None,
                        snooze_period: None,
                        refractory_period: None,
                        leak_coefficient: None,
                        leak_variability: None,
                        burst_engine_active: None,
                        properties: Some(properties),
                    });
                }
            }
        }
    }

    if to_create.is_empty() {
        return;
    }

    info!(
        "🦀 [API] Auto-creating {} missing cortical areas from device registrations",
        to_create.len()
    );

    if let Err(e) = genome_service.create_cortical_areas(to_create).await {
        warn!(
            "⚠️ [API] Failed to auto-create cortical areas from device registrations: {}",
            e
        );
    }
}

pub fn derive_motor_cortical_ids_from_device_registrations(
    device_registrations: &serde_json::Value,
) -> Result<HashSet<String>, String> {
    let output_units = device_registrations
        .get("output_units_and_decoder_properties")
        .and_then(|v| v.as_object())
        .ok_or_else(|| {
            "device_registrations missing output_units_and_decoder_properties".to_string()
        })?;

    let mut cortical_ids: HashSet<String> = HashSet::new();

    for (motor_unit_key, unit_defs) in output_units {
        let motor_unit: MotorCorticalUnit = serde_json::from_value::<MotorCorticalUnit>(
            serde_json::Value::String(motor_unit_key.clone()),
        )
        .map_err(|e| {
            format!(
                "Unable to parse MotorCorticalUnit key '{}': {}",
                motor_unit_key, e
            )
        })?;

        let unit_defs_arr = unit_defs
            .as_array()
            .ok_or_else(|| "Motor unit definitions must be an array".to_string())?;

        for entry in unit_defs_arr {
            let pair = entry
                .as_array()
                .ok_or_else(|| "Motor unit definition entries must be arrays".to_string())?;
            let unit_def = pair
                .first()
                .ok_or_else(|| "Motor unit definition entry missing unit_def".to_string())?;
            let group_u64 = unit_def
                .get("cortical_unit_index")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| "Motor unit definition missing cortical_unit_index".to_string())?;
            let group_u8: u8 = group_u64
                .try_into()
                .map_err(|_| "Motor unit cortical_unit_index out of range for u8".to_string())?;
            let group: CorticalUnitIndex = group_u8.into();

            let device_count = unit_def
                .get("device_grouping")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            if device_count == 0 {
                return Err(format!(
                    "device_grouping is empty for motor unit '{}' group {}",
                    motor_unit_key, group_u8
                ));
            }

            let config = build_io_config_map_from_unit_def(unit_def).map_err(|e| {
                format!(
                    "Failed to build motor IO config map from registration for '{}' group {}: {}",
                    motor_unit_key, group_u8, e
                )
            })?;
            let unit_cortical_ids = motor_unit
                .get_cortical_id_vector_from_index_and_serde_io_configuration_flags(group, config)
                .map_err(|e| format!("Failed to derive cortical IDs: {}", e))?;
            for cortical_id in unit_cortical_ids {
                cortical_ids.insert(cortical_id.as_base_64());
            }
        }
    }

    Ok(cortical_ids)
}

pub fn derive_sensory_cortical_ids_from_device_registrations(
    device_registrations: &serde_json::Value,
) -> Result<HashSet<String>, String> {
    let input_units = device_registrations
        .get("input_units_and_encoder_properties")
        .and_then(|v| v.as_object())
        .ok_or_else(|| {
            "device_registrations missing input_units_and_encoder_properties".to_string()
        })?;

    let mut cortical_ids: HashSet<String> = HashSet::new();

    for (sensory_unit_key, unit_defs) in input_units {
        let sensory_unit: SensoryCorticalUnit = serde_json::from_value::<SensoryCorticalUnit>(
            serde_json::Value::String(sensory_unit_key.clone()),
        )
        .map_err(|e| {
            format!(
                "Unable to parse SensoryCorticalUnit key '{}': {}",
                sensory_unit_key, e
            )
        })?;

        let unit_defs_arr = unit_defs
            .as_array()
            .ok_or_else(|| "Sensory unit definitions must be an array".to_string())?;

        for entry in unit_defs_arr {
            let pair = entry
                .as_array()
                .ok_or_else(|| "Sensory unit definition entries must be arrays".to_string())?;
            let unit_def = pair
                .first()
                .ok_or_else(|| "Sensory unit definition entry missing unit_def".to_string())?;
            let group_u64 = unit_def
                .get("cortical_unit_index")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| "Sensory unit definition missing cortical_unit_index".to_string())?;
            let group_u8: u8 = group_u64
                .try_into()
                .map_err(|_| "Sensory unit cortical_unit_index out of range for u8".to_string())?;
            let group: CorticalUnitIndex = group_u8.into();

            let device_count = unit_def
                .get("device_grouping")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            if device_count == 0 {
                return Err(format!(
                    "device_grouping is empty for sensory unit '{}' group {}",
                    sensory_unit_key, group_u8
                ));
            }

            let config = build_io_config_map()
                .map_err(|e| format!("Failed to build sensory IO config map: {}", e))?;
            let unit_cortical_ids = sensory_unit
                .get_cortical_id_vector_from_index_and_serde_io_configuration_flags(group, config)
                .map_err(|e| format!("Failed to derive cortical IDs: {}", e))?;
            for cortical_id in unit_cortical_ids {
                cortical_ids.insert(cortical_id.as_base_64());
            }
        }
    }

    Ok(cortical_ids)
}
