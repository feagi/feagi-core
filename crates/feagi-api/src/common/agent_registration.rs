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
use std::collections::{HashMap, HashSet};
use tracing::{info, warn};

fn build_friendly_unit_name(unit_label: &str, group: u8, sub_unit_index: usize) -> String {
    format!("{unit_label}-{}-{}", group, sub_unit_index)
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

    let Some(output_units) = device_registrations
        .get("output_units_and_decoder_properties")
        .and_then(|v| v.as_object())
    else {
        return;
    };
    let input_units = device_registrations
        .get("input_units_and_encoder_properties")
        .and_then(|v| v.as_object());

    // Build creation params for missing OPU areas based on default topologies.
    let mut to_create: Vec<CreateCorticalAreaParams> = Vec::new();

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

            let config_map = match build_io_config_map() {
                Ok(map) => map,
                Err(e) => {
                    warn!(
                        "⚠️ [API] Failed to build motor IO config map for '{}' group {}: {}",
                        motor_unit_key, group_u8, e
                    );
                    continue;
                }
            };

            let cortical_ids = match motor_unit
                .get_cortical_id_vector_from_index_and_serde_io_configuration_flags(
                    group,
                    config_map,
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

            let topology = motor_unit.get_unit_default_topology();

            for (i, cortical_id) in cortical_ids.iter().enumerate() {
                let cortical_id_b64 = cortical_id.as_base_64();
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
                    // If the area already exists but still has a placeholder name (often equal to the cortical_id),
                    // update it to a deterministic friendly name so UIs (e.g., Brain Visualizer) show readable labels.
                    //
                    // IMPORTANT: We only auto-rename if the current name is clearly a placeholder (== cortical_id).
                    // This avoids clobbering user-defined names.
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
                    if current.name == cortical_id_b64 {
                        let desired_name = build_friendly_unit_name(
                            motor_unit.get_friendly_name(),
                            group_u8,
                            i,
                        );
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

                let friendly_name = build_friendly_unit_name(
                    motor_unit.get_friendly_name(),
                    group_u8,
                    i,
                );
                let sub_index = CorticalSubUnitIndex::from(i as u8);
                let unit_topology = match topology.get(&sub_index) {
                    Some(topology) => topology,
                    None => {
                        warn!(
                            "⚠️ [API] Missing unit topology for motor unit '{}' subunit {} (agent device_registrations); cannot auto-create '{}'",
                            motor_unit_key, i, friendly_name
                        );
                        continue;
                    }
                };

                let dimensions = (
                    unit_topology.channel_dimensions_default[0] as usize,
                    unit_topology.channel_dimensions_default[1] as usize,
                    unit_topology.channel_dimensions_default[2] as usize,
                );
                let position = (
                    unit_topology.relative_position[0],
                    unit_topology.relative_position[1],
                    unit_topology.relative_position[2],
                );

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
                    properties: None,
                });
            }
        }
    }

    // Build creation params for missing IPU areas based on default topologies.
    if let Some(input_units) = input_units {
        for (sensory_unit_key, unit_defs) in input_units {
            let sensory_unit: SensoryCorticalUnit =
                match serde_json::from_value::<SensoryCorticalUnit>(
                    serde_json::Value::String(sensory_unit_key.clone()),
                ) {
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
                        group,
                        config_map,
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
                        // If the area already exists but still has a placeholder name (often equal to the cortical_id),
                        // update it to a deterministic friendly name so UIs (e.g., Brain Visualizer) show readable labels.
                        //
                        // IMPORTANT: We only auto-rename if the current name is clearly a placeholder (== cortical_id).
                        // This avoids clobbering user-defined names.
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
                        if current.name == cortical_id_b64 {
                            let desired_name = build_friendly_unit_name(
                                sensory_unit.get_friendly_name(),
                                group_u8,
                                i,
                            );
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

                    let friendly_name = build_friendly_unit_name(
                        sensory_unit.get_friendly_name(),
                        group_u8,
                        i,
                    );
                    let sub_index = CorticalSubUnitIndex::from(i as u8);
                    let unit_topology = match topology.get(&sub_index) {
                        Some(topology) => topology,
                        None => {
                            warn!(
                                "⚠️ [API] Missing unit topology for sensory unit '{}' subunit {} (agent device_registrations); cannot auto-create '{}'",
                                sensory_unit_key, i, friendly_name
                            );
                            continue;
                        }
                    };

                    let dimensions = (
                        unit_topology.channel_dimensions_default[0] as usize,
                        unit_topology.channel_dimensions_default[1] as usize,
                        unit_topology.channel_dimensions_default[2] as usize,
                    );
                    let position = (
                        unit_topology.relative_position[0],
                        unit_topology.relative_position[1],
                        unit_topology.relative_position[2],
                    );
                    let mut properties: HashMap<String, serde_json::Value> = HashMap::new();
                    properties.insert(
                        "cortical_subunit_index".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(sub_index.get())),
                    );

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
        let motor_unit: MotorCorticalUnit =
            serde_json::from_value::<MotorCorticalUnit>(serde_json::Value::String(
                motor_unit_key.clone(),
            ))
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
            let group_u8: u8 = group_u64.try_into().map_err(|_| {
                "Motor unit cortical_unit_index out of range for u8".to_string()
            })?;
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

            let mut config = serde_json::Map::new();
            config.insert(
                "frame_change_handling".to_string(),
                serde_json::to_value(FrameChangeHandling::Absolute)
                    .map_err(|e| format!("Failed to serialize FrameChangeHandling: {}", e))?,
            );
            config.insert(
                "percentage_neuron_positioning".to_string(),
                serde_json::to_value(PercentageNeuronPositioning::Linear).map_err(|e| {
                    format!("Failed to serialize PercentageNeuronPositioning: {}", e)
                })?,
            );

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
