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
use tracing::{info, warn};

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

            // Use defaults consistent with FEAGI registration handler.
            let frame_change_handling = FrameChangeHandling::Absolute;
            let percentage_neuron_positioning = PercentageNeuronPositioning::Linear;

            let cortical_ids = match motor_unit {
                MotorCorticalUnit::RotaryMotor => MotorCorticalUnit::get_cortical_ids_array_for_rotary_motor_with_parameters(
                    frame_change_handling,
                    percentage_neuron_positioning,
                    group,
                )
                .to_vec(),
                MotorCorticalUnit::PositionalServo => MotorCorticalUnit::get_cortical_ids_array_for_positional_servo_with_parameters(
                    frame_change_handling,
                    percentage_neuron_positioning,
                    group,
                )
                .to_vec(),
                MotorCorticalUnit::Gaze => MotorCorticalUnit::get_cortical_ids_array_for_gaze_with_parameters(
                    frame_change_handling,
                    percentage_neuron_positioning,
                    group,
                )
                .to_vec(),
                MotorCorticalUnit::MiscData => MotorCorticalUnit::get_cortical_ids_array_for_misc_data_with_parameters(
                    frame_change_handling,
                    group,
                )
                .to_vec(),
                MotorCorticalUnit::TextEnglishOutput => MotorCorticalUnit::get_cortical_ids_array_for_text_english_output_with_parameters(
                    frame_change_handling,
                    group,
                )
                .to_vec(),
                MotorCorticalUnit::CountOutput => MotorCorticalUnit::get_cortical_ids_array_for_count_output_with_parameters(
                    frame_change_handling,
                    percentage_neuron_positioning,
                    group,
                )
                .to_vec(),
                MotorCorticalUnit::ObjectSegmentation => MotorCorticalUnit::get_cortical_ids_array_for_object_segmentation_with_parameters(
                    frame_change_handling,
                    group,
                )
                .to_vec(),
                MotorCorticalUnit::SimpleVisionOutput => MotorCorticalUnit::get_cortical_ids_array_for_simple_vision_output_with_parameters(
                    frame_change_handling,
                    group,
                )
                .to_vec(),
                MotorCorticalUnit::DynamicImageProcessing => MotorCorticalUnit::get_cortical_ids_array_for_dynamic_image_processing_with_parameters(
                    frame_change_handling,
                    percentage_neuron_positioning,
                    group,
                )
                .to_vec(),
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
                    let current = match genome_service.get_cortical_area(&cortical_id_b64).await {
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
                        let desired_name = motor_unit.get_unit_friendly_name(group_u8, i);
                        let changes = serde_json::json!({
                            "name": desired_name,
                        });
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

                let friendly_name = motor_unit.get_unit_friendly_name(group_u8, i);
                let unit_topology = match topology.get(i) {
                    Some(topology) => topology,
                    None => {
                        warn!(
                            "⚠️ [API] Missing unit topology for motor unit '{}' subunit {} (agent device_registrations); cannot auto-create '{}'",
                            motor_unit_key, i, friendly_name
                        );
                        continue;
                    }
                };

                to_create.push(CreateCorticalAreaParams {
                    cortical_id: cortical_id_b64.clone(),
                    name: friendly_name,
                    dimensions: unit_topology.clone(),
                    area_type: "motor".to_string(),
                    opu_mode: true,
                    metadata: None,
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

                let cortical_ids =
                    sensory_unit.get_cortical_ids_array_for_sensory_unit(group).to_vec();
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
                        let current = match genome_service.get_cortical_area(&cortical_id_b64).await {
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
                            let desired_name =
                                sensory_unit.get_unit_friendly_name(group_u8, i);
                            let changes = serde_json::json!({
                                "name": desired_name,
                            });
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

                    let friendly_name = sensory_unit.get_unit_friendly_name(group_u8, i);
                    let unit_topology = match topology.get(i) {
                        Some(topology) => topology,
                        None => {
                            warn!(
                                "⚠️ [API] Missing unit topology for sensory unit '{}' subunit {} (agent device_registrations); cannot auto-create '{}'",
                                sensory_unit_key, i, friendly_name
                            );
                            continue;
                        }
                    };

                    to_create.push(CreateCorticalAreaParams {
                        cortical_id: cortical_id_b64.clone(),
                        name: friendly_name,
                        dimensions: unit_topology.clone(),
                        area_type: "sensory".to_string(),
                        opu_mode: false,
                        metadata: Some(
                            serde_json::json!({
                                "cortical_subunit_index": CorticalSubUnitIndex::new(i as u8).as_u8(),
                            }),
                        ),
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
