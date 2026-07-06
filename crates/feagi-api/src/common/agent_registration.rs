// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Shared registration helpers used across transports.

use crate::common::ApiState;
use base64::{engine::general_purpose, Engine as _};
use feagi_config::load_config;
use feagi_services::types::CreateCorticalAreaParams;
use feagi_genome_definitions::::descriptors::{
    CorticalSubUnitIndex, CorticalUnitIndex,
};
use feagi_genome_definitions::::io_cortical_area_configuration_flag::{
    FrameChangeHandling, PercentageNeuronPositioning,
};
use feagi_structures::genomic::{MotorCorticalUnit, SensoryCorticalUnit, UnitTopology};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use tracing::{info, warn};

const MOTOR_AREA_X_GAP_VOXELS: i32 = 10;
const SEGMENTED_VISION_GROUP_X_GAP_VOXELS: i32 = 10;
/// When grouped Servo encoder registrations declare ``motor_servo_group_id`` and
/// ``sensor_tag`` (jointpos / jointvel), IPU placement follows the corresponding
/// PositionalServo OPU sub-area (same Y/Z), shifted along X so strips sit beside motors.
const SERVO_ENCODER_X_OFFSET_FROM_MATCHED_MOTOR_VOXELS: i32 = -30;

/// First tuple element of `JSONDecoderProperties::Percentage` as JSON: either a bare
/// positive integer (legacy / hand-written payloads) or `NeuronDepth`'s serde shape
/// `{"value": n}` from `serde_json::to_value`.
fn percentage_tuple_first_depth_u32(depth_json: &Value) -> Option<u32> {
    if let Some(u) = depth_json.as_u64() {
        return u32::try_from(u).ok();
    }
    if let Some(i) = depth_json.as_i64() {
        if i > 0 {
            return u32::try_from(i).ok();
        }
        return None;
    }
    depth_json
        .get("value")
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
        .and_then(|u| u32::try_from(u).ok())
}

/// Per-channel (width, height, depth) for motor registration reconciliation.
///
/// For [`MotorCorticalUnit::CountOutput`], Z depth may be taken from the agent's
/// `JSONDecoderProperties::Percentage` tuple (first element = [`NeuronDepth`]) so the
/// connectome matches the embodiment decoder (e.g. Perception Inspector max count).
/// For [`MotorCorticalUnit::ObjectSegmentation`], all three dims may be taken from the
/// agent's `JSONDecoderProperties::MiscData` block so the connectome matches whatever
/// grid size the agent registered with (e.g. Vision Lab user-chosen oseg resolution).
/// Other motor units keep template `channel_dimensions_default` only.
fn per_channel_motor_dimensions_for_registration(
    motor_unit: MotorCorticalUnit,
    unit_topology: &UnitTopology,
    decoder_properties: Option<&Value>,
) -> (usize, usize, usize) {
    let default_w = unit_topology.channel_dimensions_default[0] as usize;
    let default_h = unit_topology.channel_dimensions_default[1] as usize;
    let default_d = unit_topology.channel_dimensions_default[2] as usize;
    if motor_unit != MotorCorticalUnit::CountOutput
        && motor_unit != MotorCorticalUnit::ObjectSegmentation
        && motor_unit != MotorCorticalUnit::PoseEstimation
        && motor_unit != MotorCorticalUnit::SpatialPointer
    {
        return (default_w, default_h, default_d);
    }
    if motor_unit == MotorCorticalUnit::CountOutput {
        let z_min = unit_topology.channel_dimensions_min[2].max(1);
        let z_max = unit_topology.channel_dimensions_max[2].max(1);
        let Some(decode) = decoder_properties else {
            return (default_w, default_h, default_d);
        };
        let Some(arr) = decode.get("Percentage").and_then(|v| v.as_array()) else {
            return (default_w, default_h, default_d);
        };
        let Some(depth_json) = arr.first() else {
            return (default_w, default_h, default_d);
        };
        let Some(d32) = percentage_tuple_first_depth_u32(depth_json) else {
            return (default_w, default_h, default_d);
        };
        if d32 == 0 {
            return (default_w, default_h, default_d);
        }
        let clamped = d32.clamp(z_min, z_max);
        return (default_w, default_h, clamped as usize);
    }
    // ObjectSegmentation: honor MiscData dimensions declared by the agent decoder.
    if motor_unit == MotorCorticalUnit::ObjectSegmentation {
        if let Some(dims) = oseg_dims_from_misc_data_decoder(decoder_properties, unit_topology) {
            return dims;
        }
        return (default_w, default_h, default_d);
    }
    // PoseEstimation: honor dimensions from decoder block.
    // Expected: {"PoseEstimation": {"width": N, "height": N, "depth": N}}
    if motor_unit == MotorCorticalUnit::PoseEstimation {
        if let Some(dims) = pose_dims_from_decoder_properties(decoder_properties, unit_topology) {
            return dims;
        }
        return (default_w, default_h, default_d);
    }
    // SpatialPointer: honor dimensions from decoder block.
    // Expected: {"SpatialPointer": {"width": N, "height": N, "depth": N}}
    if motor_unit == MotorCorticalUnit::SpatialPointer {
        if let Some(dims) =
            spatial_pointer_dims_from_decoder_properties(decoder_properties, unit_topology)
        {
            return dims;
        }
        return (default_w, default_h, default_d);
    }
    (default_w, default_h, default_d)
}

/// Extracts and clamps oseg (width, height, depth) from a `{"MiscData": {…}}` decoder block.
/// Returns `None` if the block is absent, malformed, or contains any zero dimension.
fn oseg_dims_from_misc_data_decoder(
    decoder_properties: Option<&Value>,
    unit_topology: &UnitTopology,
) -> Option<(usize, usize, usize)> {
    let misc = decoder_properties?.get("MiscData")?;
    let w = misc
        .get("width")
        .and_then(|v| v.as_u64())
        .and_then(|u| u32::try_from(u).ok())?;
    let h = misc
        .get("height")
        .and_then(|v| v.as_u64())
        .and_then(|u| u32::try_from(u).ok())?;
    let d = misc
        .get("depth")
        .and_then(|v| v.as_u64())
        .and_then(|u| u32::try_from(u).ok())?;
    if w == 0 || h == 0 || d == 0 {
        return None;
    }
    let w_min = unit_topology.channel_dimensions_min[0].max(1);
    let h_min = unit_topology.channel_dimensions_min[1].max(1);
    let d_min = unit_topology.channel_dimensions_min[2].max(1);
    let w_max = unit_topology.channel_dimensions_max[0].max(1);
    let h_max = unit_topology.channel_dimensions_max[1].max(1);
    let d_max = unit_topology.channel_dimensions_max[2].max(1);
    Some((
        w.clamp(w_min, w_max) as usize,
        h.clamp(h_min, h_max) as usize,
        d.clamp(d_min, d_max) as usize,
    ))
}

/// Extracts and clamps pose estimation (width, height, depth) from a
/// `{"PoseEstimation": {"width": N, "height": N, "depth": N}}` decoder block.
/// Returns `None` if the block is absent, malformed, or contains any zero dimension.
fn pose_dims_from_decoder_properties(
    decoder_properties: Option<&Value>,
    unit_topology: &UnitTopology,
) -> Option<(usize, usize, usize)> {
    let pose = decoder_properties?.get("PoseEstimation")?;
    let w = pose
        .get("width")
        .and_then(|v| v.as_u64())
        .and_then(|u| u32::try_from(u).ok())?;
    let h = pose
        .get("height")
        .and_then(|v| v.as_u64())
        .and_then(|u| u32::try_from(u).ok())?;
    let d = pose
        .get("depth")
        .and_then(|v| v.as_u64())
        .and_then(|u| u32::try_from(u).ok())?;
    if w == 0 || h == 0 || d == 0 {
        return None;
    }
    let w_min = unit_topology.channel_dimensions_min[0].max(1);
    let h_min = unit_topology.channel_dimensions_min[1].max(1);
    let d_min = unit_topology.channel_dimensions_min[2].max(1);
    let w_max = unit_topology.channel_dimensions_max[0].max(1);
    let h_max = unit_topology.channel_dimensions_max[1].max(1);
    let d_max = unit_topology.channel_dimensions_max[2].max(1);
    Some((
        w.clamp(w_min, w_max) as usize,
        h.clamp(h_min, h_max) as usize,
        d.clamp(d_min, d_max) as usize,
    ))
}

/// Extracts and clamps spatial pointer (width, height, depth) from a
/// `{"SpatialPointer": {"width": N, "height": N, "depth": N}}` decoder block.
/// Returns `None` if the block is absent, malformed, or contains any zero dimension.
fn spatial_pointer_dims_from_decoder_properties(
    decoder_properties: Option<&Value>,
    unit_topology: &UnitTopology,
) -> Option<(usize, usize, usize)> {
    let pointer = decoder_properties?.get("SpatialPointer")?;
    let w = pointer
        .get("width")
        .and_then(|v| v.as_u64())
        .and_then(|u| u32::try_from(u).ok())?;
    let h = pointer
        .get("height")
        .and_then(|v| v.as_u64())
        .and_then(|u| u32::try_from(u).ok())?;
    let d = pointer
        .get("depth")
        .and_then(|v| v.as_u64())
        .and_then(|u| u32::try_from(u).ok())?;
    if w == 0 || h == 0 || d == 0 {
        return None;
    }
    let w_min = unit_topology.channel_dimensions_min[0].max(1);
    let h_min = unit_topology.channel_dimensions_min[1].max(1);
    let d_min = unit_topology.channel_dimensions_min[2].max(1);
    let w_max = unit_topology.channel_dimensions_max[0].max(1);
    let h_max = unit_topology.channel_dimensions_max[1].max(1);
    let d_max = unit_topology.channel_dimensions_max[2].max(1);
    Some((
        w.clamp(w_min, w_max) as usize,
        h.clamp(h_min, h_max) as usize,
        d.clamp(d_min, d_max) as usize,
    ))
}

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

/// Reads ``motor_servo_group_id`` from the first channel (grouped strips share one ID).
fn motor_servo_group_id_u8_from_first_channel(unit_def: &Value) -> Option<u8> {
    extract_grouping_array(unit_def)
        .first()
        .and_then(|channel| channel.get("device_properties"))
        .and_then(|props| props.get("motor_servo_group_id"))
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
        .and_then(|u| u8::try_from(u).ok())
}

/// ``sensor_tag`` on the first grouped channel (e.g. ``jointpos`` / ``jointvel`` for Servo).
fn primary_sensor_tag_first_channel(unit_def: &Value) -> Option<String> {
    extract_grouping_array(unit_def)
        .first()
        .and_then(|channel| {
            non_empty_string(
                channel
                    .get("device_properties")
                    .and_then(|props| props.get("sensor_tag")),
            )
        })
}

/// Maps grouped Servo encoder modality to PositionalServo sub-unit index (0 = absolute strip).
fn servo_motor_subunit_index_for_servo_tag(tag: &str) -> Option<u8> {
    match tag {
        "jointpos" => Some(0),
        "jointvel" => Some(1),
        _ => None,
    }
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
        // Some legacy registration payloads omit this field on motor units.
        // Use the deterministic default used elsewhere in this module.
        .unwrap_or_else(|| serde_json::json!(PercentageNeuronPositioning::Linear));

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
    if let Some(pose_schema_value) = io_flags.and_then(|flags| flags.get("pose_schema")).cloned() {
        config.insert("pose_schema".to_string(), pose_schema_value);
    }
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
    let existing_segmented_vision_yz_by_subunit = connectome_service
        .list_cortical_areas()
        .await
        .ok()
        .and_then(|areas| {
            let mut grouped_yz_by_group: HashMap<u8, HashMap<u8, (i32, i32)>> = HashMap::new();

            for area in areas {
                let Ok(bytes) = general_purpose::STANDARD.decode(&area.cortical_id) else {
                    continue;
                };
                if bytes.len() != 8 || bytes[0] != b'i' || &bytes[1..4] != b"svi" {
                    continue;
                }
                let subunit_index = bytes[6];
                let group_index = bytes[7];
                grouped_yz_by_group
                    .entry(group_index)
                    .or_default()
                    .insert(subunit_index, (area.position.1, area.position.2));
            }

            // Deterministically pick one existing segmented-vision group as alignment anchor:
            // prefer the group with most subunits; ties resolved by lower group index.
            let selected_group = grouped_yz_by_group
                .iter()
                .max_by(|(group_a, map_a), (group_b, map_b)| {
                    map_a
                        .len()
                        .cmp(&map_b.len())
                        .then_with(|| group_b.cmp(group_a))
                })
                .map(|(group_index, _)| *group_index)?;
            let selected_map = grouped_yz_by_group.remove(&selected_group)?;
            if selected_map.len()
                == SensoryCorticalUnit::SegmentedVision.get_number_cortical_areas()
            {
                Some(selected_map)
            } else {
                None
            }
        });

    let output_units = device_registrations
        .get("output_units_and_decoder_properties")
        .and_then(|v| v.as_object());
    let input_units = device_registrations
        .get("input_units_and_encoder_properties")
        .and_then(|v| v.as_object());
    if output_units.is_none() && input_units.is_none() {
        return;
    }

    // For each (limb / motor_servo_group_id, PositionalServo sub-unit): resolved OPU
    // min-corner position (matches grouped Servo motor_servo_group_id + jointpos/jointvel → sub 0/1).
    let mut positional_servo_subarea_world_position: HashMap<(u8, u8), (i32, i32, i32)> =
        HashMap::new();

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
                let decoder_properties = pair.get(1);
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
                            "⚠️ [API] Failed to derive motor cortical_area IDs for '{}' group {}: {}",
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
                    let (per_channel_width, per_channel_height, per_channel_depth) =
                        per_channel_motor_dimensions_for_registration(
                            motor_unit,
                            unit_topology,
                            decoder_properties,
                        );
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

                if motor_unit == MotorCorticalUnit::PositionalServo {
                    for (sub_i, pos_opt) in expected_position_by_sub.iter().enumerate() {
                        if let Some(pos) = *pos_opt {
                            positional_servo_subarea_world_position
                                .insert((group_u8, sub_i as u8), pos);
                        }
                    }
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
                                "⚠️ [API] Failed to check cortical_area area existence for '{}': {}",
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
                        // Area exists: reconcile structural properties from registrations.
                        // Preserve user-defined layout by not mutating existing position.
                        // If a genome was loaded with wrong dimensions (e.g. 1 channel per limb),
                        // update to the correct channel count from device_grouping.
                        let current =
                            match connectome_service.get_cortical_area(&cortical_id_b64).await {
                                Ok(v) => v,
                                Err(e) => {
                                    warn!(
                                        "⚠️ [API] Failed to fetch existing cortical_area area '{}': {}",
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

                        if dimensions_mismatch || dev_count_mismatch {
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
                            if let Err(e) = genome_service
                                .update_cortical_area(&cortical_id_b64, changes)
                                .await
                            {
                                warn!(
                                    "⚠️ [API] Failed to update cortical_area area '{}' dimensions/dev_count: {}",
                                    cortical_id_b64, e
                                );
                            } else {
                                info!(
                                    "[API] Updated cortical_area area '{}' to {} channels (dimensions {:?})",
                                    cortical_id_b64, device_count, expected_dimensions
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
                                    "⚠️ [API] Failed to auto-rename existing motor cortical_area area '{}': {}",
                                    cortical_id_b64, e
                                );
                            }
                        }
                        continue;
                    }

                    let friendly_name = resolved_name;
                    let (per_channel_width, per_channel_height, per_channel_depth) =
                        per_channel_motor_dimensions_for_registration(
                            motor_unit,
                            unit_topology,
                            decoder_properties,
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
                        "⚠️ [API] device_grouping is empty for sensory unit '{}' group {}; skipping auto-create for this entry (SmartIMU/vision/IR need non-empty grouping from connector export)",
                        sensory_unit_key, group_u8
                    );
                    continue;
                }

                let config_map = match build_io_config_map_from_unit_def(unit_def) {
                    Ok(map) => map,
                    Err(e) => {
                        warn!(
                            "⚠️ [API] Failed to build sensory IO config map from registration for '{}' group {}: {}",
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
                            "⚠️ [API] Failed to derive sensory cortical_area IDs for '{}' group {}: {}",
                            sensory_unit_key, group_u8, e
                        );
                        continue;
                    }
                };
                let topology = sensory_unit.get_unit_default_topology();
                let segmented_group_x_offsets =
                    if sensory_unit == SensoryCorticalUnit::SegmentedVision {
                        // For each segmented-vision group, compute the assembly min/max X bounds based on
                        // template relative positions and effective per-subunit dimensions.
                        let mut bounds_by_group: Vec<(u8, i32, i32)> = Vec::new();
                        for grouped_entry in unit_defs_arr {
                            let Some(grouped_pair) = grouped_entry.as_array() else {
                                continue;
                            };
                            let Some(grouped_def) = grouped_pair.first() else {
                                continue;
                            };
                            let Some(grouped_u64) = grouped_def
                                .get("cortical_unit_index")
                                .and_then(|v| v.as_u64())
                            else {
                                continue;
                            };
                            let Ok(grouped_u8) = u8::try_from(grouped_u64) else {
                                continue;
                            };
                            let grouped_encoder_properties = grouped_pair.get(1);

                            let mut assembly_min_x: Option<i32> = None;
                            let mut assembly_max_x: Option<i32> = None;
                            for (sub_index, unit_topology) in &topology {
                                let sub_idx_usize = sub_index.get() as usize;
                                let dimensions = resolve_sensory_dimensions_from_encoder_properties(
                                    grouped_encoder_properties,
                                    sub_idx_usize,
                                    (
                                        unit_topology.channel_dimensions_default[0] as usize,
                                        unit_topology.channel_dimensions_default[1] as usize,
                                        unit_topology.channel_dimensions_default[2] as usize,
                                    ),
                                );
                                let rel_x = unit_topology.relative_position[0];
                                let right_edge_x = rel_x.saturating_add(dimensions.0 as i32);

                                assembly_min_x = Some(match assembly_min_x {
                                    Some(current) => current.min(rel_x),
                                    None => rel_x,
                                });
                                assembly_max_x = Some(match assembly_max_x {
                                    Some(current) => current.max(right_edge_x),
                                    None => right_edge_x,
                                });
                            }

                            if let (Some(min_x), Some(max_x)) = (assembly_min_x, assembly_max_x) {
                                bounds_by_group.push((grouped_u8, min_x, max_x));
                            }
                        }

                        // Sort by cortical_area unit index so lower-index segmented assemblies stay left and
                        // higher-index assemblies are shifted to the right with a fixed gap.
                        bounds_by_group.sort_by_key(|(grouped_u8, _, _)| *grouped_u8);

                        let mut offsets: HashMap<u8, i32> = HashMap::new();
                        let mut previous_shifted_max_x: Option<i32> = None;
                        for (grouped_u8, min_x, max_x) in bounds_by_group {
                            let offset_x = if let Some(prev_max_x) = previous_shifted_max_x {
                                prev_max_x + SEGMENTED_VISION_GROUP_X_GAP_VOXELS - min_x
                            } else {
                                0
                            };
                            previous_shifted_max_x = Some(max_x.saturating_add(offset_x));
                            offsets.insert(grouped_u8, offset_x);
                        }
                        offsets
                    } else {
                        HashMap::new()
                    };

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
                    let template_w = unit_topology.channel_dimensions_default[0] as usize;
                    let template_h = unit_topology.channel_dimensions_default[1] as usize;
                    let template_d = unit_topology.channel_dimensions_default[2] as usize;
                    let mut expected_dimensions =
                        resolve_sensory_dimensions_from_encoder_properties(
                            encoder_properties,
                            i,
                            (template_w, template_h, template_d),
                        );
                    // Grouped Servo strips (``device_grouping`` width) should match motor OPU
                    // layout: multiply default slab width by logical channel count unless the
                    // encoder payload already pinned explicit Cartesian dimensions.
                    if sensory_unit == SensoryCorticalUnit::Servo
                        && expected_dimensions == (template_w, template_h, template_d)
                    {
                        expected_dimensions =
                            ((template_w * device_count).max(1), template_h, template_d);
                    }
                    let group_x_offset = *segmented_group_x_offsets.get(&group_u8).unwrap_or(&0);
                    let existing_segmented_yz =
                        if sensory_unit == SensoryCorticalUnit::SegmentedVision {
                            existing_segmented_vision_yz_by_subunit
                                .as_ref()
                                .and_then(|yz_by_subunit| yz_by_subunit.get(&(i as u8)).copied())
                        } else {
                            None
                        };
                    let base_expected_position = (
                        unit_topology.relative_position[0] + group_x_offset,
                        existing_segmented_yz
                            .map(|(y, _)| y)
                            .unwrap_or(unit_topology.relative_position[1]),
                        existing_segmented_yz
                            .map(|(_, z)| z)
                            .unwrap_or(unit_topology.relative_position[2]),
                    );
                    let mut expected_position = base_expected_position;
                    if sensory_unit == SensoryCorticalUnit::Servo {
                        if let (Some(motor_gid), Some(ref tag)) = (
                            motor_servo_group_id_u8_from_first_channel(unit_def),
                            primary_sensor_tag_first_channel(unit_def),
                        ) {
                            if let Some(motor_sub) = servo_motor_subunit_index_for_servo_tag(tag) {
                                if let Some((mx, my, mz)) = positional_servo_subarea_world_position
                                    .get(&(motor_gid, motor_sub))
                                    .copied()
                                {
                                    expected_position = (
                                        mx + SERVO_ENCODER_X_OFFSET_FROM_MATCHED_MOTOR_VOXELS,
                                        my,
                                        mz,
                                    );
                                }
                            }
                        }
                    }
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
                                "⚠️ [API] Failed to check cortical_area area existence for '{}': {}",
                                cortical_id_b64, e
                            );
                            continue;
                        }
                    };

                    if exists {
                        // Area exists: reconcile structural properties from registrations.
                        // Preserve user-defined layout by not mutating existing position.
                        // This keeps pre-existing sensory areas aligned with declared capabilities.
                        let current = match connectome_service
                            .get_cortical_area(&cortical_id_b64)
                            .await
                        {
                            Ok(v) => v,
                            Err(e) => {
                                warn!(
                                    "⚠️ [API] Failed to fetch existing cortical_area area '{}' for potential rename: {}",
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
                        if dimensions_mismatch || dev_count_mismatch {
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
                            if let Err(e) = genome_service
                                .update_cortical_area(&cortical_id_b64, changes)
                                .await
                            {
                                warn!(
                                    "⚠️ [API] Failed to update sensory cortical_area area '{}' dimensions/dev_count: {}",
                                    cortical_id_b64, e
                                );
                            } else {
                                info!(
                                    "[API] Updated sensory cortical_area area '{}' to registration dimensions {:?} (dev_count {})",
                                    cortical_id_b64, expected_dimensions, device_count
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
                                    "⚠️ [API] Failed to auto-rename existing sensory cortical_area area '{}': {}",
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
                    if let Some(default_firing_threshold) =
                        sensory_unit.get_default_firing_threshold()
                    {
                        properties.insert(
                            "firing_threshold".to_string(),
                            serde_json::json!(default_firing_threshold),
                        );
                    }
                    if let Some(default_mp_charge_accumulation) =
                        sensory_unit.get_default_mp_charge_accumulation()
                    {
                        properties.insert(
                            "mp_charge_accumulation".to_string(),
                            serde_json::json!(default_mp_charge_accumulation),
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
        "🦀 [API] Auto-creating {} missing cortical_area areas from device registrations",
        to_create.len()
    );

    if let Err(e) = genome_service.create_cortical_areas(to_create).await {
        warn!(
            "⚠️ [API] Failed to auto-create cortical_area areas from device registrations: {}",
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
                .map_err(|e| format!("Failed to derive cortical_area IDs: {}", e))?;
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

            let config = build_io_config_map_from_unit_def(unit_def).map_err(|e| {
                format!(
                    "Failed to build sensory IO config map from registration for '{}' group {}: {}",
                    sensory_unit_key, group_u8, e
                )
            })?;
            let unit_cortical_ids = sensory_unit
                .get_cortical_id_vector_from_index_and_serde_io_configuration_flags(group, config)
                .map_err(|e| format!("Failed to derive cortical_area IDs: {}", e))?;
            for cortical_id in unit_cortical_ids {
                cortical_ids.insert(cortical_id.as_base_64());
            }
        }
    }

    Ok(cortical_ids)
}

#[cfg(test)]
mod count_output_registration_tests {
    use super::per_channel_motor_dimensions_for_registration;
    use feagi_genome_definitions::::descriptors::CorticalSubUnitIndex;
    use feagi_structures::genomic::MotorCorticalUnit;
    use serde_json::json;

    #[test]
    fn count_output_uses_percentage_tuple_depth_when_present() {
        let motor = MotorCorticalUnit::CountOutput;
        let topo = motor.get_unit_default_topology();
        let ut = topo.get(&CorticalSubUnitIndex::from(0u8)).unwrap();
        let dec = json!({"Percentage": [100u32, "Linear", false, "D1"]});
        let (w, h, d) = per_channel_motor_dimensions_for_registration(motor, ut, Some(&dec));
        assert_eq!((w, h, d), (1, 1, 100));
    }

    #[test]
    fn count_output_reads_neuron_depth_serde_object_in_percentage_tuple() {
        let motor = MotorCorticalUnit::CountOutput;
        let topo = motor.get_unit_default_topology();
        let ut = topo.get(&CorticalSubUnitIndex::from(0u8)).unwrap();
        // Matches `serde_json::to_value(JSONDecoderProperties::Percentage(NeuronDepth::new(100)...))`.
        let dec = json!({"Percentage": [{"value": 100u32}, "Linear", false, "D1"]});
        let (w, h, d) = per_channel_motor_dimensions_for_registration(motor, ut, Some(&dec));
        assert_eq!((w, h, d), (1, 1, 100));
    }

    #[test]
    fn count_output_clamps_depth_to_template_max() {
        let motor = MotorCorticalUnit::CountOutput;
        let topo = motor.get_unit_default_topology();
        let ut = topo.get(&CorticalSubUnitIndex::from(0u8)).unwrap();
        let dec = json!({"Percentage": [5000u32, "Linear", false, "D1"]});
        let (_w, _h, d) = per_channel_motor_dimensions_for_registration(motor, ut, Some(&dec));
        assert_eq!(d, 1024);
    }

    #[test]
    fn count_output_falls_back_to_defaults_when_decoder_missing() {
        let motor = MotorCorticalUnit::CountOutput;
        let topo = motor.get_unit_default_topology();
        let ut = topo.get(&CorticalSubUnitIndex::from(0u8)).unwrap();
        let (w, h, d) = per_channel_motor_dimensions_for_registration(motor, ut, None);
        assert_eq!((w, h, d), (1, 1, 10));
    }

    #[test]
    fn non_count_motor_ignores_decoder() {
        let motor = MotorCorticalUnit::RotaryMotor;
        let topo = motor.get_unit_default_topology();
        let ut = topo.get(&CorticalSubUnitIndex::from(0u8)).unwrap();
        let dec = json!({"Percentage": [99u32, "Linear", false, "D1"]});
        let (w, h, d) = per_channel_motor_dimensions_for_registration(motor, ut, Some(&dec));
        assert_eq!((w, h, d), (1, 1, 9));
    }

    #[test]
    fn object_segmentation_uses_misc_data_decoder_dimensions() {
        let motor = MotorCorticalUnit::ObjectSegmentation;
        let topo = motor.get_unit_default_topology();
        let ut = topo.get(&CorticalSubUnitIndex::from(0u8)).unwrap();
        let dec = json!({"MiscData": {"width": 128u32, "height": 96u32, "depth": 12u32}});
        let (w, h, d) = per_channel_motor_dimensions_for_registration(motor, ut, Some(&dec));
        assert_eq!((w, h, d), (128, 96, 12));
    }

    #[test]
    fn object_segmentation_clamps_misc_data_to_template_bounds() {
        let motor = MotorCorticalUnit::ObjectSegmentation;
        let topo = motor.get_unit_default_topology();
        let ut = topo.get(&CorticalSubUnitIndex::from(0u8)).unwrap();
        let dec = json!({"MiscData": {"width": 9999u32, "height": 9999u32, "depth": 9999u32}});
        let (w, h, d) = per_channel_motor_dimensions_for_registration(motor, ut, Some(&dec));
        assert_eq!((w, h, d), (4096, 4096, 1024));
    }

    #[test]
    fn object_segmentation_falls_back_to_template_when_no_misc_data_decoder() {
        let motor = MotorCorticalUnit::ObjectSegmentation;
        let topo = motor.get_unit_default_topology();
        let ut = topo.get(&CorticalSubUnitIndex::from(0u8)).unwrap();
        let (w, h, d) = per_channel_motor_dimensions_for_registration(motor, ut, None);
        assert_eq!((w, h, d), (32, 32, 8));
    }

    #[test]
    fn spatial_pointer_uses_decoder_dimensions() {
        let motor = MotorCorticalUnit::SpatialPointer;
        let topo = motor.get_unit_default_topology();
        let ut = topo.get(&CorticalSubUnitIndex::from(0u8)).unwrap();
        let dec = json!({"SpatialPointer": {"width": 64u32, "height": 64u32, "depth": 64u32}});
        let (w, h, d) = per_channel_motor_dimensions_for_registration(motor, ut, Some(&dec));
        assert_eq!((w, h, d), (64, 64, 64));
    }
}

#[cfg(test)]
mod sensory_registration_frame_mode_tests {
    use super::derive_sensory_cortical_ids_from_device_registrations;
    use feagi_structures::genomic::cortical_area::descriptors::CorticalUnitIndex;
    use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
        FrameChangeHandling, PercentageNeuronPositioning,
    };
    use feagi_structures::genomic::SensoryCorticalUnit;
    use serde_json::json;

    #[test]
    fn derive_sensory_ids_honors_incremental_frame_for_servo() {
        let registration = json!({
            "input_units_and_encoder_properties": {
                "Servo": [[
                    {
                        "cortical_unit_index": 60,
                        "io_configuration_flags": {
                            "frame_change_handling": "Incremental",
                            "percentage_neuron_positioning": "Linear"
                        },
                        "device_grouping": [
                            {
                                "friendly_name": "servo_encoder_incremental_right_shoulder",
                                "device_properties": {
                                    "bundle_id": "jointvel_right_shoulder"
                                }
                            }
                        ]
                    },
                    {}
                ]]
            }
        });

        let expected_incremental =
            SensoryCorticalUnit::get_cortical_ids_array_for_servo_with_parameters(
                FrameChangeHandling::Incremental,
                PercentageNeuronPositioning::Linear,
                CorticalUnitIndex::from(60u8),
            )[0]
            .as_base_64();

        let derived = derive_sensory_cortical_ids_from_device_registrations(&registration)
            .expect("derive sensory IDs");

        assert!(
            derived.contains(&expected_incremental),
            "Expected derived sensory IDs to include incremental Servo cortical ID {}",
            expected_incremental
        );
    }
}

#[cfg(test)]
mod servo_encoder_registration_helpers_tests {
    use super::{
        motor_servo_group_id_u8_from_first_channel, primary_sensor_tag_first_channel,
        servo_motor_subunit_index_for_servo_tag,
    };
    use serde_json::json;

    #[test]
    fn servo_tag_maps_jointpos_jointvel_to_motor_subunits() {
        assert_eq!(servo_motor_subunit_index_for_servo_tag("jointpos"), Some(0));
        assert_eq!(servo_motor_subunit_index_for_servo_tag("jointvel"), Some(1));
        assert_eq!(servo_motor_subunit_index_for_servo_tag("other"), None);
    }

    #[test]
    fn reads_motor_servo_group_id_and_sensor_tag_from_first_channel() {
        let unit_def = json!({
            "device_grouping": [
                {
                    "device_properties": {
                        "motor_servo_group_id": 2,
                        "sensor_tag": "jointpos"
                    }
                }
            ]
        });
        assert_eq!(
            motor_servo_group_id_u8_from_first_channel(&unit_def),
            Some(2)
        );
        assert_eq!(
            primary_sensor_tag_first_channel(&unit_def).as_deref(),
            Some("jointpos")
        );
    }
}
