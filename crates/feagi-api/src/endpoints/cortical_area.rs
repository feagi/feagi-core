// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Cortical Area API Endpoints - Exact port from Python `/v1/cortical_area/*`
//!
//! Reference: feagi-py/feagi/api/v1/cortical_area.py

use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::common::ApiState;
use crate::common::{ApiError, ApiResult, Json, Path, Query, State};
use feagi_structures::genomic::{MotorCorticalUnit, SensoryCorticalUnit};
use feagi_structures::genomic::cortical_area::descriptors::CorticalSubUnitIndex;

// ============================================================================
// REQUEST/RESPONSE MODELS
// ============================================================================

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CorticalAreaIdListResponse {
    pub cortical_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CorticalAreaNameListResponse {
    pub cortical_area_name_list: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UnitTopologyData {
    pub relative_position: [i32; 3],
    pub dimensions: [u32; 3],
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CorticalTypeMetadata {
    pub description: String,
    pub encodings: Vec<String>,
    pub formats: Vec<String>,
    pub units: u32,
    pub resolution: Vec<i32>,
    pub structure: String,
    pub unit_default_topology: HashMap<usize, UnitTopologyData>,
}

// ============================================================================
// ENDPOINTS
// ============================================================================

/// List all IPU (Input Processing Unit) cortical area IDs. Returns IDs of all sensory cortical areas.
#[utoipa::path(get, path = "/v1/cortical_area/ipu", tag = "cortical_area")]
pub async fn get_ipu(State(state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_cortical_areas().await {
        Ok(areas) => {
            let ipu_areas: Vec<String> = areas
                .into_iter()
                .filter(|a| a.area_type == "sensory" || a.area_type == "IPU")
                .map(|a| a.cortical_id)
                .collect();
            Ok(Json(ipu_areas))
        }
        Err(e) => Err(ApiError::internal(format!(
            "Failed to get IPU areas: {}",
            e
        ))),
    }
}

/// List all OPU (Output Processing Unit) cortical area IDs. Returns IDs of all motor cortical areas.
#[utoipa::path(get, path = "/v1/cortical_area/opu", tag = "cortical_area")]
pub async fn get_opu(State(state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_cortical_areas().await {
        Ok(areas) => {
            let opu_areas: Vec<String> = areas
                .into_iter()
                .filter(|a| a.area_type == "motor" || a.area_type == "OPU")
                .map(|a| a.cortical_id)
                .collect();
            Ok(Json(opu_areas))
        }
        Err(e) => Err(ApiError::internal(format!(
            "Failed to get OPU areas: {}",
            e
        ))),
    }
}

/// Get a list of all cortical area IDs across the entire genome (IPU, OPU, custom, memory, and core areas).
#[utoipa::path(
    get,
    path = "/v1/cortical_area/cortical_area_id_list",
    tag = "cortical_area",
    responses(
        (status = 200, description = "Cortical area IDs retrieved successfully", body = CorticalAreaIdListResponse),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_cortical_area_id_list(
    State(state): State<ApiState>,
) -> ApiResult<Json<CorticalAreaIdListResponse>> {
    tracing::debug!(target: "feagi-api", "🔍 GET /v1/cortical_area/cortical_area_id_list - handler called");
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.get_cortical_area_ids().await {
        Ok(ids) => {
            tracing::info!(target: "feagi-api", "✅ GET /v1/cortical_area/cortical_area_id_list - success, returning {} IDs", ids.len());
            tracing::debug!(target: "feagi-api", "📋 Cortical area IDs: {:?}", ids.iter().take(20).collect::<Vec<_>>());
            let response = CorticalAreaIdListResponse {
                cortical_ids: ids.clone(),
            };
            match serde_json::to_string(&response) {
                Ok(json_str) => {
                    tracing::debug!(target: "feagi-api", "📤 Response JSON: {}", json_str);
                }
                Err(e) => {
                    tracing::warn!(target: "feagi-api", "⚠️ Failed to serialize response: {}", e);
                }
            }
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!(target: "feagi-api", "❌ GET /v1/cortical_area/cortical_area_id_list - error: {}", e);
            Err(ApiError::internal(format!(
                "Failed to get cortical IDs: {}",
                e
            )))
        }
    }
}

/// Get a list of all cortical area names (human-readable labels for all cortical areas).
#[utoipa::path(
    get,
    path = "/v1/cortical_area/cortical_area_name_list",
    tag = "cortical_area",
    responses(
        (status = 200, description = "Cortical area names retrieved successfully", body = CorticalAreaNameListResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_cortical_area_name_list(
    State(state): State<ApiState>,
) -> ApiResult<Json<CorticalAreaNameListResponse>> {
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_cortical_areas().await {
        Ok(areas) => {
            let names: Vec<String> = areas.into_iter().map(|a| a.name).collect();
            Ok(Json(CorticalAreaNameListResponse {
                cortical_area_name_list: names,
            }))
        }
        Err(e) => Err(ApiError::internal(format!(
            "Failed to get cortical names: {}",
            e
        ))),
    }
}

/// Get a map of cortical area IDs to their human-readable names. Returns {cortical_id: name} pairs.
#[utoipa::path(
    get,
    path = "/v1/cortical_area/cortical_id_name_mapping",
    tag = "cortical_area"
)]
pub async fn get_cortical_id_name_mapping(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let connectome_service = state.connectome_service.as_ref();
    let ids = connectome_service
        .get_cortical_area_ids()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get IDs: {}", e)))?;

    let mut mapping = HashMap::new();
    for id in ids {
        if let Ok(area) = connectome_service.get_cortical_area(&id).await {
            mapping.insert(id, area.name);
        }
    }
    Ok(Json(mapping))
}

/// Get available cortical area types: sensory, motor, memory, and custom.
#[utoipa::path(get, path = "/v1/cortical_area/cortical_types", tag = "cortical_area")]
pub async fn get_cortical_types(State(_state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    Ok(Json(vec![
        "sensory".to_string(),
        "motor".to_string(),
        "memory".to_string(),
        "custom".to_string(),
    ]))
}

/// Get detailed cortical connectivity mappings showing source-to-destination connections with mapping rules.
#[utoipa::path(
    get,
    path = "/v1/cortical_area/cortical_map_detailed",
    tag = "cortical_area",
    responses(
        (status = 200, description = "Detailed cortical area mapping data", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_cortical_map_detailed(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_cortical_areas().await {
        Ok(areas) => {
            let mut map: HashMap<String, serde_json::Value> = HashMap::new();

            for area in areas {
                // Extract cortical_mapping_dst from area properties
                if let Some(cortical_mapping_dst) = area.properties.get("cortical_mapping_dst") {
                    if !cortical_mapping_dst.is_null()
                        && cortical_mapping_dst
                            .as_object()
                            .is_some_and(|obj| !obj.is_empty())
                    {
                        map.insert(area.cortical_id.clone(), cortical_mapping_dst.clone());
                    }
                }
            }

            Ok(Json(map))
        }
        Err(e) => Err(ApiError::internal(format!(
            "Failed to get detailed map: {}",
            e
        ))),
    }
}

/// Get 2D positions of all cortical areas for visualization. Returns {cortical_id: (x, y)} coordinates.
#[utoipa::path(
    get,
    path = "/v1/cortical_area/cortical_locations_2d",
    tag = "cortical_area"
)]
pub async fn get_cortical_locations_2d(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, (i32, i32)>>> {
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_cortical_areas().await {
        Ok(areas) => {
            let locations: HashMap<String, (i32, i32)> = areas
                .into_iter()
                .map(|area| (area.cortical_id, (area.position.0, area.position.1)))
                .collect();
            Ok(Json(locations))
        }
        Err(e) => Err(ApiError::internal(format!(
            "Failed to get 2D locations: {}",
            e
        ))),
    }
}

/// Get complete cortical area data including geometry, neural parameters, and metadata. Used by Brain Visualizer.
#[utoipa::path(
    get,
    path = "/v1/cortical_area/cortical_area/geometry",
    tag = "cortical_area"
)]
pub async fn get_cortical_area_geometry(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_cortical_areas().await {
        Ok(areas) => {
            let geometry: HashMap<String, serde_json::Value> = areas.into_iter()
                .map(|area| {
                    // Return FULL cortical area data (matching Python format)
                    // This is what Brain Visualizer expects for genome loading
                    let data = serde_json::json!({
                        "cortical_id": area.cortical_id,
                        "cortical_name": area.name,
                        "cortical_group": area.cortical_group,
                        "cortical_type": area.cortical_type,  // NEW: Explicitly include cortical_type for BV
                        "cortical_sub_group": area.sub_group.as_ref().unwrap_or(&String::new()),  // Return empty string instead of null
                        "coordinates_3d": [area.position.0, area.position.1, area.position.2],
                        "coordinates_2d": [0, 0],  // TODO: Extract from properties when available
                        "cortical_dimensions": [area.dimensions.0, area.dimensions.1, area.dimensions.2],
                        "cortical_neuron_per_vox_count": area.neurons_per_voxel,
                        "visualization": area.visible,
                        "visible": area.visible,
                        // Also include dictionary-style for backward compatibility
                        "dimensions": {
                            "x": area.dimensions.0,
                            "y": area.dimensions.1,
                            "z": area.dimensions.2
                        },
                        "position": {
                            "x": area.position.0,
                            "y": area.position.1,
                            "z": area.position.2
                        },
                        // Neural parameters
                        "neuron_post_synaptic_potential": area.postsynaptic_current,
                        "neuron_fire_threshold": area.firing_threshold_limit,
                        "plasticity_constant": area.plasticity_constant,
                        "degeneration": area.degeneration,
                        "leak_coefficient": area.leak_coefficient,
                        "refractory_period": area.refractory_period,
                        "snooze_period": area.snooze_period,
                        // Parent region ID (required by Brain Visualizer)
                        "parent_region_id": area.parent_region_id,
                        // Heatmap chunk size for large-area visualization (optional)
                        "heatmap_chunk_size": area.heatmap_chunk_size.map(|(x, y, z)| serde_json::json!([x, y, z])),
                    });
                    (area.cortical_id.clone(), data)
                })
                .collect();
            Ok(Json(geometry))
        }
        Err(e) => Err(ApiError::internal(format!("Failed to get geometry: {}", e))),
    }
}

/// Get visibility status of all cortical areas. Returns {cortical_id: visibility_flag}.
#[utoipa::path(
    get,
    path = "/v1/cortical_area/cortical_visibility",
    tag = "cortical_area"
)]
pub async fn get_cortical_visibility(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, bool>>> {
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_cortical_areas().await {
        Ok(areas) => {
            let visibility: HashMap<String, bool> = areas
                .into_iter()
                .map(|area| (area.cortical_id, area.visible))
                .collect();
            Ok(Json(visibility))
        }
        Err(e) => Err(ApiError::internal(format!(
            "Failed to get visibility: {}",
            e
        ))),
    }
}

/// Get the 2D location of a cortical area by its name. Request: {cortical_name: string}.
#[utoipa::path(
    post,
    path = "/v1/cortical_area/cortical_name_location",
    tag = "cortical_area"
)]
#[allow(unused_variables)] // In development
pub async fn post_cortical_name_location(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, (i32, i32)>>> {
    let connectome_service = state.connectome_service.as_ref();
    let cortical_name = request
        .get("cortical_name")
        .ok_or_else(|| ApiError::invalid_input("cortical_name required"))?;

    match connectome_service.get_cortical_area(cortical_name).await {
        Ok(area) => Ok(Json(HashMap::from([(
            area.cortical_id,
            (area.position.0, area.position.1),
        )]))),
        Err(e) => Err(ApiError::internal(format!("Failed to get location: {}", e))),
    }
}

/// Get detailed properties of a single cortical area by ID. Request: {cortical_id: string}.
#[utoipa::path(
    post,
    path = "/v1/cortical_area/cortical_area_properties",
    tag = "cortical_area"
)]
#[allow(unused_variables)] // In development
pub async fn post_cortical_area_properties(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, String>>,
) -> ApiResult<Json<serde_json::Value>> {
    let connectome_service = state.connectome_service.as_ref();
    let cortical_id = request
        .get("cortical_id")
        .ok_or_else(|| ApiError::invalid_input("cortical_id required"))?;

    match connectome_service.get_cortical_area(cortical_id).await {
        Ok(area_info) => {
            tracing::debug!(target: "feagi-api", "Cortical area properties for {}: cortical_group={}, area_type={}, cortical_type={}", 
                cortical_id, area_info.cortical_group, area_info.area_type, area_info.cortical_type);
            tracing::info!(target: "feagi-api", "[API-RESPONSE] Returning mp_driven_psp={} for area {}", area_info.mp_driven_psp, cortical_id);
            let json_value = serde_json::to_value(&area_info).unwrap_or_default();
            tracing::debug!(target: "feagi-api", "Serialized JSON keys: {:?}", json_value.as_object().map(|o| o.keys().collect::<Vec<_>>()));
            tracing::debug!(target: "feagi-api", "Serialized cortical_type value: {:?}", json_value.get("cortical_type"));
            Ok(Json(json_value))
        }
        Err(e) => Err(ApiError::internal(format!(
            "Failed to get properties: {}",
            e
        ))),
    }
}

/// Get properties for multiple cortical areas. Accepts array [\"id1\", \"id2\"] or object {cortical_id_list: [...]}.
#[utoipa::path(
    post,
    path = "/v1/cortical_area/multi/cortical_area_properties",
    tag = "cortical_area"
)]
#[allow(unused_variables)] // In development
pub async fn post_multi_cortical_area_properties(
    State(state): State<ApiState>,
    Json(request): Json<serde_json::Value>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let connectome_service = state.connectome_service.as_ref();
    let mut result = HashMap::new();

    // Support both formats for backward compatibility
    let cortical_ids: Vec<String> = if request.is_array() {
        // Format 1: Direct array ["id1", "id2"] (Python SDK)
        request
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect()
    } else if request.is_object() {
        // Format 2: Object with cortical_id_list {"cortical_id_list": ["id1", "id2"]} (Brain Visualizer)
        request
            .get("cortical_id_list")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ApiError::invalid_input("cortical_id_list required in object format"))?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect()
    } else {
        return Err(ApiError::invalid_input(
            "Request must be an array of IDs or object with cortical_id_list",
        ));
    };

    for cortical_id in cortical_ids {
        if let Ok(area_info) = connectome_service.get_cortical_area(&cortical_id).await {
            tracing::debug!(target: "feagi-api", 
                "[MULTI] Area {}: cortical_type={}, cortical_group={}, is_mem_type={:?}", 
                cortical_id, area_info.cortical_type, area_info.cortical_group,
                area_info.properties.get("is_mem_type")
            );
            let json_value = serde_json::to_value(&area_info).unwrap_or_default();
            tracing::debug!(target: "feagi-api", 
                "[MULTI] Serialized has cortical_type: {}", 
                json_value.get("cortical_type").is_some()
            );
            result.insert(cortical_id, json_value);
        }
    }
    Ok(Json(result))
}

/// Create IPU (sensory) or OPU (motor) cortical areas with proper topology and multi-unit support.
#[utoipa::path(post, path = "/v1/cortical_area/cortical_area", tag = "cortical_area")]
#[allow(unused_variables)] // In development - parameters will be used when implemented
pub async fn post_cortical_area(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<serde_json::Value>> {
    use feagi_services::types::CreateCorticalAreaParams;
    use feagi_structures::genomic::{MotorCorticalUnit, SensoryCorticalUnit};

    // ARCHITECTURE: Use genome_service (proper entry point) instead of connectome_service
    let genome_service = state.genome_service.as_ref();

    // Extract required fields
    let cortical_type_key = request
        .get("cortical_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::invalid_input("cortical_id required"))?;

    let group_id = request
        .get("group_id")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u8;

    let device_count = request
        .get("device_count")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| ApiError::invalid_input("device_count required"))?
        as usize;

    let coordinates_3d: Vec<i32> = request
        .get("coordinates_3d")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            if arr.len() == 3 {
                Some(vec![
                    arr[0].as_i64()? as i32,
                    arr[1].as_i64()? as i32,
                    arr[2].as_i64()? as i32,
                ])
            } else {
                None
            }
        })
        .ok_or_else(|| ApiError::invalid_input("coordinates_3d must be [x, y, z]"))?;

    let cortical_type_str = request
        .get("cortical_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::invalid_input("cortical_type required"))?;

    // Extract neurons_per_voxel from request (default to 1 if not provided)
    let neurons_per_voxel = request
        .get("neurons_per_voxel")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as u32;

    // Extract data_type_config from request (default to 0 for backward compatibility)
    // Handle multiple number types: u64, i64, f64, and string representations
    let raw_data_type_config = request.get("data_type_config");
    if let Some(raw_val) = raw_data_type_config {
        tracing::debug!(target: "feagi-api", "Raw data_type_config value: {:?} (type: {:?})", raw_val, raw_val);
    }
    let data_type_config = raw_data_type_config
        .and_then(|v| {
            // Try u64 first (most common case)
            if let Some(u) = v.as_u64() {
                return Some(u);
            }
            // Try i64 (signed integer)
            if let Some(i) = v.as_i64() {
                if i >= 0 {
                    return Some(i as u64);
                }
            }
            // Try f64 (float) - round to nearest integer
            if let Some(f) = v.as_f64() {
                if f >= 0.0 && f <= u16::MAX as f64 {
                    return Some(f.round() as u64);
                }
            }
            // Try string representation
            if let Some(s) = v.as_str() {
                if let Ok(parsed) = s.parse::<u64>() {
                    return Some(parsed);
                }
            }
            None
        })
        .map(|v| {
            if v > u16::MAX as u64 {
                tracing::warn!(target: "feagi-api", "data_type_config value {} exceeds u16::MAX, clamping to {}", v, u16::MAX);
                u16::MAX
            } else {
                v as u16
            }
        })
        .unwrap_or_else(|| {
            tracing::warn!(target: "feagi-api", "data_type_config missing or invalid in request, defaulting to 0 (backward compatibility)");
            0
        });

    // Split data_type_config into two bytes for cortical ID
    let config_byte_4 = (data_type_config & 0xFF) as u8; // Lower byte
    let config_byte_5 = ((data_type_config >> 8) & 0xFF) as u8; // Upper byte

    tracing::info!(target: "feagi-api", "Creating cortical areas for {} with neurons_per_voxel={}, data_type_config={} (bytes: {}, {})",
        cortical_type_key, neurons_per_voxel, data_type_config, config_byte_4, config_byte_5);

    // Determine number of units and get topology
    let (num_units, unit_topology) = if cortical_type_str == "IPU" {
        // Find the matching sensory cortical unit
        let unit = SensoryCorticalUnit::list_all()
            .iter()
            .find(|u| {
                let id_ref = u.get_cortical_id_unit_reference();
                let key = format!("i{}", std::str::from_utf8(&id_ref).unwrap_or(""));
                key == cortical_type_key
            })
            .ok_or_else(|| {
                ApiError::invalid_input(format!("Unknown IPU type: {}", cortical_type_key))
            })?;

        (
            unit.get_number_cortical_areas(),
            unit.get_unit_default_topology(),
        )
    } else if cortical_type_str == "OPU" {
        // Find the matching motor cortical unit
        let unit = MotorCorticalUnit::list_all()
            .iter()
            .find(|u| {
                let id_ref = u.get_cortical_id_unit_reference();
                let key = format!("o{}", std::str::from_utf8(&id_ref).unwrap_or(""));
                key == cortical_type_key
            })
            .ok_or_else(|| {
                ApiError::invalid_input(format!("Unknown OPU type: {}", cortical_type_key))
            })?;

        (
            unit.get_number_cortical_areas(),
            unit.get_unit_default_topology(),
        )
    } else {
        return Err(ApiError::invalid_input("cortical_type must be IPU or OPU"));
    };

    tracing::info!(
        "Creating {} units for cortical type: {}",
        num_units,
        cortical_type_key
    );

    // Build creation parameters for all units
    let mut creation_params = Vec::new();
    for unit_idx in 0..num_units {
        // Get dimensions for this unit from topology
        let dimensions = if let Some(topo) = unit_topology.get(&CorticalSubUnitIndex::from(unit_idx as u8)) {
            let dims = topo.channel_dimensions_default;
            (dims[0] as usize, dims[1] as usize, dims[2] as usize)
        } else {
            (1, 1, 1) // Fallback
        };

        // Calculate position for this unit
        let position = if let Some(topo) = unit_topology.get(&CorticalSubUnitIndex::from(unit_idx as u8)) {
            let rel_pos = topo.relative_position;
            (
                coordinates_3d[0] + rel_pos[0],
                coordinates_3d[1] + rel_pos[1],
                coordinates_3d[2] + rel_pos[2],
            )
        } else {
            (coordinates_3d[0], coordinates_3d[1], coordinates_3d[2])
        };

        // Construct proper 8-byte cortical ID
        // Byte structure: [type(i/o), subtype[0], subtype[1], subtype[2], encoding_type, encoding_format, unit_idx, group_id]
        // Extract the 3-character subtype from cortical_type_key (e.g., "isvi" -> "svi")
        let subtype_bytes = if cortical_type_key.len() >= 4 {
            let subtype_str = &cortical_type_key[1..4]; // Skip the 'i' or 'o' prefix
            let mut bytes = [0u8; 3];
            for (i, c) in subtype_str.chars().take(3).enumerate() {
                bytes[i] = c as u8;
            }
            bytes
        } else {
            return Err(ApiError::invalid_input("Invalid cortical_type_key"));
        };

        // Construct the 8-byte cortical ID
        // Use data_type_config from request for bytes 4-5
        let cortical_id_bytes = [
            if cortical_type_str == "IPU" {
                b'i'
            } else {
                b'o'
            }, // Byte 0: type
            subtype_bytes[0], // Byte 1: subtype[0]
            subtype_bytes[1], // Byte 2: subtype[1]
            subtype_bytes[2], // Byte 3: subtype[2]
            config_byte_4,    // Byte 4: data type config (lower byte)
            config_byte_5,    // Byte 5: data type config (upper byte)
            unit_idx as u8,   // Byte 6: unit index
            group_id,         // Byte 7: group ID
        ];

        // Encode to base64 for use as cortical_id string
        let cortical_id = general_purpose::STANDARD.encode(cortical_id_bytes);

        tracing::debug!(target: "feagi-api",
            "  Unit {}: dims={}x{}x{}, neurons_per_voxel={}, total_neurons={}",
            unit_idx, dimensions.0, dimensions.1, dimensions.2, neurons_per_voxel,
            dimensions.0 * dimensions.1 * dimensions.2 * neurons_per_voxel as usize
        );

        // Store device_count and per-device dimensions in properties for BV compatibility
        let mut properties = HashMap::new();
        properties.insert(
            "dev_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(device_count)),
        );
        properties.insert(
            "cortical_dimensions_per_device".to_string(),
            serde_json::json!([dimensions.0, dimensions.1, dimensions.2]),
        );

        let params = CreateCorticalAreaParams {
            cortical_id: cortical_id.clone(),
            name: format!("{} Unit {}", cortical_type_key, unit_idx),
            dimensions,
            position,
            area_type: cortical_type_str.to_string(),
            visible: Some(true),
            sub_group: None,
            neurons_per_voxel: Some(neurons_per_voxel),
            postsynaptic_current: Some(0.0),
            plasticity_constant: Some(0.0),
            degeneration: Some(0.0),
            psp_uniform_distribution: Some(false),
            firing_threshold_increment: Some(0.0),
            firing_threshold_limit: Some(0.0),
            consecutive_fire_count: Some(0),
            snooze_period: Some(0),
            refractory_period: Some(0),
            leak_coefficient: Some(0.0),
            leak_variability: Some(0.0),
            burst_engine_active: Some(true),
            properties: Some(properties),
        };

        creation_params.push(params);
    }

    tracing::info!(
        "Calling GenomeService to create {} cortical areas",
        creation_params.len()
    );

    // ARCHITECTURE: Call genome_service.create_cortical_areas (proper flow)
    // This will: 1) Update runtime genome, 2) Call neuroembryogenesis, 3) Create neurons/synapses
    let areas_details = genome_service
        .create_cortical_areas(creation_params)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to create cortical areas: {}", e)))?;

    tracing::info!(
        "✅ Successfully created {} cortical areas via GenomeService",
        areas_details.len()
    );

    // Serialize as JSON
    let areas_json = serde_json::to_value(&areas_details).unwrap_or_default();

    // Extract cortical IDs from created areas
    let created_ids: Vec<String> = areas_details
        .iter()
        .map(|a| a.cortical_id.clone())
        .collect();

    // Return comprehensive response
    let first_id = created_ids.first().cloned().unwrap_or_default();
    let mut response = serde_json::Map::new();
    response.insert(
        "message".to_string(),
        serde_json::Value::String(format!("Created {} cortical areas", created_ids.len())),
    );
    response.insert(
        "cortical_id".to_string(),
        serde_json::Value::String(first_id),
    ); // For backward compatibility
    response.insert(
        "cortical_ids".to_string(),
        serde_json::Value::String(created_ids.join(", ")),
    );
    response.insert(
        "unit_count".to_string(),
        serde_json::Value::Number(created_ids.len().into()),
    );
    response.insert("areas".to_string(), areas_json); // Full details for all areas

    Ok(Json(serde_json::Value::Object(response)))
}

/// Update properties of an existing cortical area (position, dimensions, neural parameters, etc.).
#[utoipa::path(put, path = "/v1/cortical_area/cortical_area", tag = "cortical_area")]
pub async fn put_cortical_area(
    State(state): State<ApiState>,
    Json(mut request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let genome_service = state.genome_service.as_ref();

    // Extract cortical_id
    let cortical_id = request
        .get("cortical_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::invalid_input("cortical_id required"))?
        .to_string();

    // Remove cortical_id from changes (it's not a property to update)
    request.remove("cortical_id");

    // Call GenomeService with raw changes (it handles classification and routing)
    match genome_service
        .update_cortical_area(&cortical_id, request)
        .await
    {
        Ok(_) => Ok(Json(HashMap::from([
            ("message".to_string(), "Cortical area updated".to_string()),
            ("cortical_id".to_string(), cortical_id),
        ]))),
        Err(e) => Err(ApiError::internal(format!("Failed to update: {}", e))),
    }
}

/// Delete a cortical area by ID. Removes the area and all associated neurons and synapses.
#[utoipa::path(
    delete,
    path = "/v1/cortical_area/cortical_area",
    tag = "cortical_area"
)]
#[allow(unused_variables)] // In development - parameters will be used when implemented
pub async fn delete_cortical_area(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let connectome_service = state.connectome_service.as_ref();
    let cortical_id = request
        .get("cortical_id")
        .ok_or_else(|| ApiError::invalid_input("cortical_id required"))?;

    match connectome_service.delete_cortical_area(cortical_id).await {
        Ok(_) => Ok(Json(HashMap::from([(
            "message".to_string(),
            "Cortical area deleted".to_string(),
        )]))),
        Err(e) => Err(ApiError::internal(format!("Failed to delete: {}", e))),
    }
}

/// Create a custom cortical area for internal processing with specified dimensions and position.
#[utoipa::path(
    post,
    path = "/v1/cortical_area/custom_cortical_area",
    tag = "cortical_area"
)]
pub async fn post_custom_cortical_area(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    use feagi_services::types::CreateCorticalAreaParams;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Extract required fields from request
    let cortical_name = request
        .get("cortical_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::invalid_input("cortical_name required"))?;

    let cortical_dimensions: Vec<u32> = request
        .get("cortical_dimensions")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            if arr.len() == 3 {
                Some(vec![
                    arr[0].as_u64()? as u32,
                    arr[1].as_u64()? as u32,
                    arr[2].as_u64()? as u32,
                ])
            } else {
                None
            }
        })
        .ok_or_else(|| ApiError::invalid_input("cortical_dimensions must be [x, y, z]"))?;

    let coordinates_3d: Vec<i32> = request
        .get("coordinates_3d")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            if arr.len() == 3 {
                Some(vec![
                    arr[0].as_i64()? as i32,
                    arr[1].as_i64()? as i32,
                    arr[2].as_i64()? as i32,
                ])
            } else {
                None
            }
        })
        .ok_or_else(|| ApiError::invalid_input("coordinates_3d must be [x, y, z]"))?;

    let brain_region_id = request
        .get("brain_region_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let cortical_sub_group = request
        .get("cortical_sub_group")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    tracing::info!(target: "feagi-api",
        "Creating custom cortical area '{}' with dimensions: {}x{}x{}, position: ({}, {}, {})",
        cortical_name, cortical_dimensions[0], cortical_dimensions[1], cortical_dimensions[2],
        coordinates_3d[0], coordinates_3d[1], coordinates_3d[2]
    );

    // Generate unique cortical ID for custom cortical area
    // Format: [b'c', 6 random alphanumeric bytes, group_counter]
    // Use timestamp + counter to ensure uniqueness
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    // Create 8-byte cortical ID for custom area
    // Byte 0: 'c' for custom
    // Bytes 1-6: Derived from name (first 6 chars, padded with underscores)
    // Byte 7: Counter based on timestamp lower bits
    let mut cortical_id_bytes = [0u8; 8];
    cortical_id_bytes[0] = b'c'; // Custom cortical area marker

    // Use the cortical name for bytes 1-6 (truncate or pad as needed)
    let name_bytes = cortical_name.as_bytes();
    for i in 1..7 {
        cortical_id_bytes[i] = if i - 1 < name_bytes.len() {
            // Use alphanumeric ASCII only
            let c = name_bytes[i - 1];
            if c.is_ascii_alphanumeric() || c == b'_' {
                c
            } else {
                b'_'
            }
        } else {
            b'_' // Padding
        };
    }

    // Byte 7: Use timestamp lower byte for uniqueness
    cortical_id_bytes[7] = (timestamp & 0xFF) as u8;

    // Encode to base64 for use as cortical_id string
    let cortical_id = general_purpose::STANDARD.encode(cortical_id_bytes);

    tracing::debug!(target: "feagi-api",
        "Generated cortical_id: {} (raw bytes: {:?})",
        cortical_id, cortical_id_bytes
    );

    // Build properties with brain_region_id if provided
    let mut properties = HashMap::new();
    if let Some(region_id) = brain_region_id.clone() {
        properties.insert(
            "parent_region_id".to_string(),
            serde_json::Value::String(region_id),
        );
    }

    // Create cortical area parameters
    let params = CreateCorticalAreaParams {
        cortical_id: cortical_id.clone(),
        name: cortical_name.to_string(),
        dimensions: (
            cortical_dimensions[0] as usize,
            cortical_dimensions[1] as usize,
            cortical_dimensions[2] as usize,
        ),
        position: (coordinates_3d[0], coordinates_3d[1], coordinates_3d[2]),
        area_type: "Custom".to_string(),
        visible: Some(true),
        sub_group: cortical_sub_group,
        neurons_per_voxel: Some(1),
        postsynaptic_current: Some(0.0),
        plasticity_constant: Some(0.0),
        degeneration: Some(0.0),
        psp_uniform_distribution: Some(false),
        firing_threshold_increment: Some(0.0),
        firing_threshold_limit: Some(0.0),
        consecutive_fire_count: Some(0),
        snooze_period: Some(0),
        refractory_period: Some(0),
        leak_coefficient: Some(0.0),
        leak_variability: Some(0.0),
        burst_engine_active: Some(true),
        properties: Some(properties),
    };

    let genome_service = state.genome_service.as_ref();

    tracing::info!(target: "feagi-api", "Calling GenomeService to create custom cortical area");

    // Create the cortical area via GenomeService
    let areas_details = genome_service
        .create_cortical_areas(vec![params])
        .await
        .map_err(|e| ApiError::internal(format!("Failed to create custom cortical area: {}", e)))?;

    let created_area = areas_details
        .first()
        .ok_or_else(|| ApiError::internal("No cortical area was created"))?;

    tracing::info!(target: "feagi-api",
        "✅ Successfully created custom cortical area '{}' with ID: {}",
        cortical_name, created_area.cortical_id
    );

    // Return response
    let mut response = HashMap::new();
    response.insert(
        "message".to_string(),
        "Custom cortical area created successfully".to_string(),
    );
    response.insert("cortical_id".to_string(), created_area.cortical_id.clone());
    response.insert("cortical_name".to_string(), cortical_name.to_string());

    Ok(Json(response))
}

/// Clone an existing cortical area with all its properties and structure. (Not yet implemented)
#[utoipa::path(post, path = "/v1/cortical_area/clone", tag = "cortical_area")]
#[allow(unused_variables)] // In development
pub async fn post_clone(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Clone cortical area
    Err(ApiError::internal("Not yet implemented"))
}

/// Update properties of multiple cortical areas in a single request. (Not yet implemented)
#[utoipa::path(
    put,
    path = "/v1/cortical_area/multi/cortical_area",
    tag = "cortical_area"
)]
pub async fn put_multi_cortical_area(
    State(state): State<ApiState>,
    Json(mut request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let genome_service = state.genome_service.as_ref();

    // Extract cortical_id_list
    let cortical_ids: Vec<String> = request
        .get("cortical_id_list")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ApiError::invalid_input("cortical_id_list required"))?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    if cortical_ids.is_empty() {
        return Err(ApiError::invalid_input("cortical_id_list cannot be empty"));
    }

    // Remove cortical_id_list from changes (it's not a property to update)
    request.remove("cortical_id_list");

    // Update each cortical area with the same properties
    for cortical_id in &cortical_ids {
        match genome_service
            .update_cortical_area(cortical_id, request.clone())
            .await
        {
            Ok(_) => {
                tracing::info!("Updated cortical area: {}", cortical_id);
            }
            Err(e) => {
                tracing::error!("Failed to update cortical area {}: {}", cortical_id, e);
                return Err(ApiError::internal(format!(
                    "Failed to update cortical area {}: {}",
                    cortical_id, e
                )));
            }
        }
    }

    Ok(Json(HashMap::from([
        (
            "message".to_string(),
            format!("Updated {} cortical areas", cortical_ids.len()),
        ),
        (
            "cortical_ids".to_string(),
            cortical_ids.join(", "),
        ),
    ])))
}

/// Delete multiple cortical areas by their IDs. (Not yet implemented)
#[utoipa::path(
    delete,
    path = "/v1/cortical_area/multi/cortical_area",
    tag = "cortical_area"
)]
#[allow(unused_variables)] // In development
pub async fn delete_multi_cortical_area(
    State(state): State<ApiState>,
    Json(request): Json<Vec<String>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Delete multiple cortical areas
    Err(ApiError::internal("Not yet implemented"))
}

/// Update the 2D visualization coordinates of a cortical area. (Not yet implemented)
#[utoipa::path(put, path = "/v1/cortical_area/coord_2d", tag = "cortical_area")]
#[allow(unused_variables)] // In development
pub async fn put_coord_2d(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Update 2D coordinates
    Err(ApiError::internal("Not yet implemented"))
}

/// Hide/show cortical areas in visualizations. (Not yet implemented)
#[utoipa::path(
    put,
    path = "/v1/cortical_area/suppress_cortical_visibility",
    tag = "cortical_area"
)]
#[allow(unused_variables)] // In development
pub async fn put_suppress_cortical_visibility(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Suppress cortical visibility
    Err(ApiError::internal("Not yet implemented"))
}

/// Reset a cortical area to its default state (clear neuron states, etc.). (Not yet implemented)
#[utoipa::path(put, path = "/v1/cortical_area/reset", tag = "cortical_area")]
#[allow(unused_variables)] // In development
pub async fn put_reset(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Reset cortical area
    Err(ApiError::internal("Not yet implemented"))
}

/// Check if visualization is enabled for the system.
#[utoipa::path(get, path = "/v1/cortical_area/visualization", tag = "cortical_area")]
pub async fn get_visualization(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, bool>>> {
    let mut response = HashMap::new();
    response.insert("enabled".to_string(), true);
    Ok(Json(response))
}

/// Execute multiple cortical area operations (create, update, delete) in a single batch.
#[utoipa::path(
    post,
    path = "/v1/cortical_area/batch_operations",
    tag = "cortical_area"
)]
pub async fn post_batch_operations(
    State(_state): State<ApiState>,
    Json(_ops): Json<Vec<HashMap<String, serde_json::Value>>>,
) -> ApiResult<Json<HashMap<String, i32>>> {
    let mut response = HashMap::new();
    response.insert("processed".to_string(), 0);
    Ok(Json(response))
}

/// Alias for /v1/cortical_area/ipu - list all IPU cortical area IDs.
#[utoipa::path(get, path = "/v1/cortical_area/ipu/list", tag = "cortical_area")]
pub async fn get_ipu_list(State(state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    get_ipu(State(state)).await
}

/// Alias for /v1/cortical_area/opu - list all OPU cortical area IDs.
#[utoipa::path(get, path = "/v1/cortical_area/opu/list", tag = "cortical_area")]
pub async fn get_opu_list(State(state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    get_opu(State(state)).await
}

/// Update the 3D position of a cortical area. (Not yet implemented)
#[utoipa::path(put, path = "/v1/cortical_area/coordinates_3d", tag = "cortical_area")]
pub async fn put_coordinates_3d(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Not yet implemented".to_string(),
    )])))
}

/// Delete multiple cortical areas by their IDs in a single operation.
#[utoipa::path(delete, path = "/v1/cortical_area/bulk_delete", tag = "cortical_area")]
pub async fn delete_bulk(
    State(_state): State<ApiState>,
    Json(_ids): Json<Vec<String>>,
) -> ApiResult<Json<HashMap<String, i32>>> {
    let mut response = HashMap::new();
    response.insert("deleted_count".to_string(), 0);
    Ok(Json(response))
}

/// Resize a cortical area by changing its dimensions. (Not yet implemented)
#[utoipa::path(post, path = "/v1/cortical_area/resize", tag = "cortical_area")]
pub async fn post_resize(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Not yet implemented".to_string(),
    )])))
}

/// Move a cortical area to a new position. (Not yet implemented)
#[utoipa::path(post, path = "/v1/cortical_area/reposition", tag = "cortical_area")]
pub async fn post_reposition(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Not yet implemented".to_string(),
    )])))
}

/// Get neurons at specific voxel coordinates within a cortical area.
#[utoipa::path(post, path = "/v1/cortical_area/voxel_neurons", tag = "cortical_area")]
pub async fn post_voxel_neurons(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let mut response = HashMap::new();
    response.insert("neurons".to_string(), serde_json::json!([]));
    Ok(Json(response))
}

/// Get metadata for all available IPU types (vision, infrared, etc.). Includes encodings, formats, units, and topology.
#[utoipa::path(
    get,
    path = "/v1/cortical_area/ipu/types",
    tag = "cortical_area",
    responses(
        (status = 200, description = "IPU type metadata", body = HashMap<String, CorticalTypeMetadata>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_ipu_types(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, CorticalTypeMetadata>>> {
    let mut types = HashMap::new();

    // Dynamically generate metadata from feagi_data_structures templates
    for unit in SensoryCorticalUnit::list_all() {
        let id_ref = unit.get_cortical_id_unit_reference();
        let key = format!("i{}", std::str::from_utf8(&id_ref).unwrap_or("???"));

        // All IPU types support both absolute and incremental encodings
        let encodings = vec!["absolute".to_string(), "incremental".to_string()];

        // Determine if formats are supported based on snake_case_name
        // Vision and SegmentedVision use CartesianPlane (no formats)
        // MiscData uses Misc (no formats)
        // All others use Percentage-based types (have formats)
        let snake_name = unit.get_snake_case_name();
        let formats = if snake_name == "vision"
            || snake_name == "segmented_vision"
            || snake_name == "miscellaneous"
        {
            vec![]
        } else {
            vec!["linear".to_string(), "fractional".to_string()]
        };

        // Default resolution based on type
        let resolution = if snake_name == "vision" {
            vec![64, 64, 1] // Vision sensors typically 64x64
        } else if snake_name == "segmented_vision" {
            vec![32, 32, 1] // Segmented vision segments are smaller
        } else {
            vec![1, 1, 1] // Most sensors are scalar (1x1x1)
        };

        // Most sensors are asymmetric
        let structure = "asymmetric".to_string();

        // Get unit default topology
        let topology_map = unit.get_unit_default_topology();
        let unit_default_topology: HashMap<usize, UnitTopologyData> = topology_map
            .into_iter()
            .map(|(idx, topo)| {
                (
                    *idx as usize,
                    UnitTopologyData {
                        relative_position: topo.relative_position,
                        dimensions: topo.channel_dimensions_default,
                    },
                )
            })
            .collect();

        types.insert(
            key,
            CorticalTypeMetadata {
                description: unit.get_friendly_name().to_string(),
                encodings,
                formats,
                units: unit.get_number_cortical_areas() as u32,
                resolution,
                structure,
                unit_default_topology,
            },
        );
    }

    Ok(Json(types))
}

/// Get metadata for all available OPU types (motors, servos, etc.). Includes encodings, formats, units, and topology.
#[utoipa::path(
    get,
    path = "/v1/cortical_area/opu/types",
    tag = "cortical_area",
    responses(
        (status = 200, description = "OPU type metadata", body = HashMap<String, CorticalTypeMetadata>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_opu_types(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, CorticalTypeMetadata>>> {
    let mut types = HashMap::new();

    // Dynamically generate metadata from feagi_data_structures templates
    for unit in MotorCorticalUnit::list_all() {
        let id_ref = unit.get_cortical_id_unit_reference();
        let key = format!("o{}", std::str::from_utf8(&id_ref).unwrap_or("???"));

        // All OPU types support both absolute and incremental encodings
        let encodings = vec!["absolute".to_string(), "incremental".to_string()];

        // Determine if formats are supported based on snake_case_name
        // MiscData uses Misc (no formats)
        // All others use Percentage-based types (have formats)
        let snake_name = unit.get_snake_case_name();
        let formats = if snake_name == "miscellaneous" {
            vec![]
        } else {
            vec!["linear".to_string(), "fractional".to_string()]
        };

        // Default resolution - all motors/actuators are typically scalar
        let resolution = vec![1, 1, 1];

        // All actuators are asymmetric
        let structure = "asymmetric".to_string();

        // Get unit default topology
        let topology_map = unit.get_unit_default_topology();
        let unit_default_topology: HashMap<usize, UnitTopologyData> = topology_map
            .into_iter()
            .map(|(idx, topo)| {
                (
                    *idx as usize,
                    UnitTopologyData {
                        relative_position: topo.relative_position,
                        dimensions: topo.channel_dimensions_default,
                    },
                )
            })
            .collect();

        types.insert(
            key,
            CorticalTypeMetadata {
                description: unit.get_friendly_name().to_string(),
                encodings,
                formats,
                units: unit.get_number_cortical_areas() as u32,
                resolution,
                structure,
                unit_default_topology,
            },
        );
    }

    Ok(Json(types))
}

/// Get list of all cortical area indices (numerical indices used internally for indexing).
#[utoipa::path(
    get,
    path = "/v1/cortical_area/cortical_area_index_list",
    tag = "cortical_area"
)]
pub async fn get_cortical_area_index_list(
    State(state): State<ApiState>,
) -> ApiResult<Json<Vec<u32>>> {
    let connectome_service = state.connectome_service.as_ref();
    let areas = connectome_service
        .list_cortical_areas()
        .await
        .map_err(|e| ApiError::internal(format!("{}", e)))?;
    // CRITICAL FIX: Return the actual cortical_idx values, not fabricated sequential indices
    let indices: Vec<u32> = areas.iter().map(|a| a.cortical_idx).collect();
    Ok(Json(indices))
}

/// Get mapping from cortical area IDs to their internal indices. Returns {cortical_id: index}.
#[utoipa::path(
    get,
    path = "/v1/cortical_area/cortical_idx_mapping",
    tag = "cortical_area"
)]
pub async fn get_cortical_idx_mapping(
    State(state): State<ApiState>,
) -> ApiResult<Json<std::collections::BTreeMap<String, u32>>> {
    use std::collections::BTreeMap;

    let connectome_service = state.connectome_service.as_ref();
    let areas = connectome_service
        .list_cortical_areas()
        .await
        .map_err(|e| ApiError::internal(format!("{}", e)))?;
    // CRITICAL FIX: Use the actual cortical_idx from CorticalArea, NOT enumerate() which ignores reserved indices!
    // Use BTreeMap for consistent alphabetical ordering
    let mapping: BTreeMap<String, u32> = areas
        .iter()
        .map(|a| (a.cortical_id.clone(), a.cortical_idx))
        .collect();
    Ok(Json(mapping))
}

/// Get restrictions on which cortical areas can connect to which (connection validation rules).
#[utoipa::path(
    get,
    path = "/v1/cortical_area/mapping_restrictions",
    tag = "cortical_area"
)]
pub async fn get_mapping_restrictions_query(
    State(_state): State<ApiState>,
    Query(_params): Query<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, Vec<String>>>> {
    Ok(Json(HashMap::new()))
}

/// Get memory usage of a specific cortical area in bytes (calculated from neuron count).
#[utoipa::path(
    get,
    path = "/v1/cortical_area/{cortical_id}/memory_usage",
    tag = "cortical_area"
)]
pub async fn get_memory_usage(
    State(state): State<ApiState>,
    Path(cortical_id): Path<String>,
) -> ApiResult<Json<HashMap<String, i64>>> {
    let connectome_service = state.connectome_service.as_ref();

    // CRITICAL FIX: Calculate actual memory usage based on neuron count instead of hardcoded 0
    let area_info = connectome_service
        .get_cortical_area(&cortical_id)
        .await
        .map_err(|_| ApiError::not_found("CorticalArea", &cortical_id))?;

    // Calculate memory usage: neuron_count × bytes per neuron
    // Each neuron in NeuronArray uses ~48 bytes (membrane_potential, threshold, refractory, etc.)
    const BYTES_PER_NEURON: i64 = 48;
    let memory_bytes = (area_info.neuron_count as i64) * BYTES_PER_NEURON;

    let mut response = HashMap::new();
    response.insert("memory_bytes".to_string(), memory_bytes);
    Ok(Json(response))
}

/// Get the total number of neurons in a specific cortical area.
#[utoipa::path(
    get,
    path = "/v1/cortical_area/{cortical_id}/neuron_count",
    tag = "cortical_area"
)]
pub async fn get_area_neuron_count(
    State(state): State<ApiState>,
    Path(cortical_id): Path<String>,
) -> ApiResult<Json<i64>> {
    let connectome_service = state.connectome_service.as_ref();

    // CRITICAL FIX: Get actual neuron count from ConnectomeService instead of hardcoded 0
    let area_info = connectome_service
        .get_cortical_area(&cortical_id)
        .await
        .map_err(|_| ApiError::not_found("CorticalArea", &cortical_id))?;

    Ok(Json(area_info.neuron_count as i64))
}

/// Get available cortical type options for UI selection: Sensory, Motor, Custom, Memory.
#[utoipa::path(
    post,
    path = "/v1/cortical_area/cortical_type_options",
    tag = "cortical_area"
)]
pub async fn post_cortical_type_options(
    State(_state): State<ApiState>,
) -> ApiResult<Json<Vec<String>>> {
    Ok(Json(vec![
        "Sensory".to_string(),
        "Motor".to_string(),
        "Custom".to_string(),
        "Memory".to_string(),
    ]))
}

/// Get mapping restrictions for specific cortical areas (POST version with request body).
#[utoipa::path(
    post,
    path = "/v1/cortical_area/mapping_restrictions",
    tag = "cortical_area"
)]
pub async fn post_mapping_restrictions(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, Vec<String>>>> {
    Ok(Json(HashMap::new()))
}

/// Get mapping restrictions between two specific cortical areas (connection validation).
#[utoipa::path(
    post,
    path = "/v1/cortical_area/mapping_restrictions_between_areas",
    tag = "cortical_area"
)]
pub async fn post_mapping_restrictions_between_areas(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, Vec<String>>>> {
    Ok(Json(HashMap::new()))
}

/// Update 3D coordinates of a cortical area (alternative endpoint). (Not yet implemented)
#[utoipa::path(put, path = "/v1/cortical_area/coord_3d", tag = "cortical_area")]
pub async fn put_coord_3d(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Not yet implemented".to_string(),
    )])))
}
