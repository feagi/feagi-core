// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Cortical Area API Endpoints - Exact port from Python `/v1/cortical_area/*`
//!
//! Reference: feagi-py/feagi/api/v1/cortical_area.py

use axum::{
    extract::State,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::common::{ApiError, ApiResult};
use crate::transports::http::server::ApiState;

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

// ============================================================================
// ENDPOINTS
// ============================================================================

/// GET /v1/cortical_area/ipu
#[utoipa::path(get, path = "/v1/cortical_area/ipu", tag = "cortical_area")]
pub async fn get_ipu(State(state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_cortical_areas().await {
        Ok(areas) => {
            let ipu_areas: Vec<String> = areas.into_iter()
                .filter(|a| a.area_type == "sensory" || a.area_type == "IPU")
                .map(|a| a.cortical_id)
                .collect();
            Ok(Json(ipu_areas))
        },
        Err(e) => Err(ApiError::internal(format!("Failed to get IPU areas: {}", e))),
    }
}

/// GET /v1/cortical_area/opu
#[utoipa::path(get, path = "/v1/cortical_area/opu", tag = "cortical_area")]
pub async fn get_opu(State(state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_cortical_areas().await {
        Ok(areas) => {
            let opu_areas: Vec<String> = areas.into_iter()
                .filter(|a| a.area_type == "motor" || a.area_type == "OPU")
                .map(|a| a.cortical_id)
                .collect();
            Ok(Json(opu_areas))
        },
        Err(e) => Err(ApiError::internal(format!("Failed to get OPU areas: {}", e))),
    }
}

/// GET /v1/cortical_area/cortical_area_id_list
#[utoipa::path(
    get,
    path = "/v1/cortical_area/cortical_area_id_list",
    responses(
        (status = 200, description = "Cortical area IDs retrieved successfully", body = CorticalAreaIdListResponse),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "cortical_area"
)]
pub async fn get_cortical_area_id_list(State(state): State<ApiState>) -> ApiResult<Json<CorticalAreaIdListResponse>> {
    tracing::debug!(target: "feagi-api", "🔍 GET /v1/cortical_area/cortical_area_id_list - handler called");
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.get_cortical_area_ids().await {
        Ok(ids) => {
            tracing::info!(target: "feagi-api", "✅ GET /v1/cortical_area/cortical_area_id_list - success, returning {} IDs", ids.len());
            tracing::debug!(target: "feagi-api", "📋 Cortical area IDs: {:?}", ids.iter().take(20).collect::<Vec<_>>());
            let response = CorticalAreaIdListResponse { cortical_ids: ids.clone() };
            match serde_json::to_string(&response) {
                Ok(json_str) => {
                    tracing::debug!(target: "feagi-api", "📤 Response JSON: {}", json_str);
                }
                Err(e) => {
                    tracing::warn!(target: "feagi-api", "⚠️ Failed to serialize response: {}", e);
                }
            }
            Ok(Json(response))
        },
        Err(e) => {
            tracing::error!(target: "feagi-api", "❌ GET /v1/cortical_area/cortical_area_id_list - error: {}", e);
            Err(ApiError::internal(format!("Failed to get cortical IDs: {}", e)))
        },
    }
}

/// GET /v1/cortical_area/cortical_area_name_list
#[utoipa::path(
    get,
    path = "/v1/cortical_area/cortical_area_name_list",
    responses(
        (status = 200, description = "Cortical area names retrieved successfully", body = CorticalAreaNameListResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "cortical_area"
)]
pub async fn get_cortical_area_name_list(State(state): State<ApiState>) -> ApiResult<Json<CorticalAreaNameListResponse>> {
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_cortical_areas().await {
        Ok(areas) => {
            let names: Vec<String> = areas.into_iter().map(|a| a.name).collect();
            Ok(Json(CorticalAreaNameListResponse { cortical_area_name_list: names }))
        },
        Err(e) => Err(ApiError::internal(format!("Failed to get cortical names: {}", e))),
    }
}

/// GET /v1/cortical_area/cortical_id_name_mapping
#[utoipa::path(get, path = "/v1/cortical_area/cortical_id_name_mapping", tag = "cortical_area")]
pub async fn get_cortical_id_name_mapping(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, String>>> {
    let connectome_service = state.connectome_service.as_ref();
    let ids = connectome_service.get_cortical_area_ids().await
        .map_err(|e| ApiError::internal(format!("Failed to get IDs: {}", e)))?;
    
    let mut mapping = HashMap::new();
    for id in ids {
        if let Ok(area) = connectome_service.get_cortical_area(&id).await {
            mapping.insert(id, area.name);
        }
    }
    Ok(Json(mapping))
}

/// GET /v1/cortical_area/cortical_types
#[utoipa::path(get, path = "/v1/cortical_area/cortical_types", tag = "cortical_area")]
pub async fn get_cortical_types(State(_state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    Ok(Json(vec!["sensory".to_string(), "motor".to_string(), "memory".to_string(), "custom".to_string()]))
}

/// GET /v1/cortical_area/cortical_map_detailed
#[utoipa::path(
    get, 
    path = "/v1/cortical_area/cortical_map_detailed",
    tag = "cortical_area",
    responses(
        (status = 200, description = "Detailed cortical area mapping data", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_cortical_map_detailed(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_cortical_areas().await {
        Ok(areas) => {
            let map: HashMap<String, serde_json::Value> = areas.into_iter()
                .map(|area| (area.cortical_id.clone(), serde_json::to_value(area).unwrap_or_default()))
                .collect();
            Ok(Json(map))
        },
        Err(e) => Err(ApiError::internal(format!("Failed to get detailed map: {}", e))),
    }
}

/// GET /v1/cortical_area/cortical_locations_2d
#[utoipa::path(get, path = "/v1/cortical_area/cortical_locations_2d", tag = "cortical_area")]
pub async fn get_cortical_locations_2d(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, (i32, i32)>>> {
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_cortical_areas().await {
        Ok(areas) => {
            let locations: HashMap<String, (i32, i32)> = areas.into_iter()
                .map(|area| (area.cortical_id, (area.position.0 as i32, area.position.1 as i32)))
                .collect();
            Ok(Json(locations))
        },
        Err(e) => Err(ApiError::internal(format!("Failed to get 2D locations: {}", e))),
    }
}

/// GET /v1/cortical_area/cortical_area/geometry
/// 
/// Returns FULL cortical area data in Godot-compatible format (not just geometry!)
/// Despite the name, this endpoint returns complete cortical area properties
/// to match Python behavior and support Brain Visualizer genome loading
#[utoipa::path(get, path = "/v1/cortical_area/cortical_area/geometry", tag = "cortical_area")]
pub async fn get_cortical_area_geometry(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
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
                    });
                    (area.cortical_id.clone(), data)
                })
                .collect();
            Ok(Json(geometry))
        },
        Err(e) => Err(ApiError::internal(format!("Failed to get geometry: {}", e))),
    }
}

/// GET /v1/cortical_area/cortical_visibility
#[utoipa::path(get, path = "/v1/cortical_area/cortical_visibility", tag = "cortical_area")]
pub async fn get_cortical_visibility(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, bool>>> {
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_cortical_areas().await {
        Ok(areas) => {
            let visibility: HashMap<String, bool> = areas.into_iter()
                .map(|area| (area.cortical_id, area.visible))
                .collect();
            Ok(Json(visibility))
        },
        Err(e) => Err(ApiError::internal(format!("Failed to get visibility: {}", e))),
    }
}

/// POST /v1/cortical_area/cortical_name_location
#[utoipa::path(post, path = "/v1/cortical_area/cortical_name_location", tag = "cortical_area")]
#[allow(unused_variables)]  // In development
pub async fn post_cortical_name_location(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, (i32, i32)>>> {
    let connectome_service = state.connectome_service.as_ref();
    let cortical_name = request.get("cortical_name").ok_or_else(|| ApiError::invalid_input("cortical_name required"))?;
    
    match connectome_service.get_cortical_area(cortical_name).await {
        Ok(area) => Ok(Json(HashMap::from([(area.cortical_id, (area.position.0 as i32, area.position.1 as i32))]))),
        Err(e) => Err(ApiError::internal(format!("Failed to get location: {}", e))),
    }
}

/// POST /v1/cortical_area/cortical_area_properties
#[utoipa::path(post, path = "/v1/cortical_area/cortical_area_properties", tag = "cortical_area")]
#[allow(unused_variables)]  // In development
pub async fn post_cortical_area_properties(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, String>>,
) -> ApiResult<Json<serde_json::Value>> {
    let connectome_service = state.connectome_service.as_ref();
    let cortical_id = request.get("cortical_id").ok_or_else(|| ApiError::invalid_input("cortical_id required"))?;
    
    match connectome_service.get_cortical_area(cortical_id).await {
        Ok(area) => Ok(Json(serde_json::to_value(area).unwrap_or_default())),
        Err(e) => Err(ApiError::internal(format!("Failed to get properties: {}", e))),
    }
}

/// POST /v1/cortical_area/multi/cortical_area_properties
#[utoipa::path(post, path = "/v1/cortical_area/multi/cortical_area_properties", tag = "cortical_area")]
#[allow(unused_variables)]  // In development
pub async fn post_multi_cortical_area_properties(
    State(state): State<ApiState>,
    Json(request): Json<Vec<String>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let connectome_service = state.connectome_service.as_ref();
    let mut result = HashMap::new();
    
    for cortical_id in request {
        if let Ok(area) = connectome_service.get_cortical_area(&cortical_id).await {
            result.insert(cortical_id, serde_json::to_value(area).unwrap_or_default());
        }
    }
    Ok(Json(result))
}

/// POST /v1/cortical_area/cortical_area
#[utoipa::path(post, path = "/v1/cortical_area/cortical_area", tag = "cortical_area")]
#[allow(unused_variables)]  // In development - parameters will be used when implemented
pub async fn post_cortical_area(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let connectome_service = state.connectome_service.as_ref();
    // TODO: Parse request and create cortical area
    Err(ApiError::internal("Not yet implemented"))
}

/// PUT /v1/cortical_area/cortical_area
#[utoipa::path(put, path = "/v1/cortical_area/cortical_area", tag = "cortical_area")]
#[allow(unused_variables)]  // In development - parameters will be used when implemented
pub async fn put_cortical_area(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let connectome_service = state.connectome_service.as_ref();
    // TODO: Parse request and update cortical area
    Err(ApiError::internal("Not yet implemented"))
}

/// DELETE /v1/cortical_area/cortical_area
#[utoipa::path(delete, path = "/v1/cortical_area/cortical_area", tag = "cortical_area")]
#[allow(unused_variables)]  // In development - parameters will be used when implemented
pub async fn delete_cortical_area(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let connectome_service = state.connectome_service.as_ref();
    let cortical_id = request.get("cortical_id").ok_or_else(|| ApiError::invalid_input("cortical_id required"))?;
    
    match connectome_service.delete_cortical_area(cortical_id).await {
        Ok(_) => Ok(Json(HashMap::from([("message".to_string(), "Cortical area deleted".to_string())]))),
        Err(e) => Err(ApiError::internal(format!("Failed to delete: {}", e))),
    }
}

/// POST /v1/cortical_area/custom_cortical_area
#[utoipa::path(post, path = "/v1/cortical_area/custom_cortical_area", tag = "cortical_area")]
#[allow(unused_variables)]  // In development
pub async fn post_custom_cortical_area(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Create custom cortical area
    Err(ApiError::internal("Not yet implemented"))
}

/// POST /v1/cortical_area/clone
#[utoipa::path(post, path = "/v1/cortical_area/clone", tag = "cortical_area")]
#[allow(unused_variables)]  // In development
pub async fn post_clone(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Clone cortical area
    Err(ApiError::internal("Not yet implemented"))
}

/// PUT /v1/cortical_area/multi/cortical_area
#[utoipa::path(put, path = "/v1/cortical_area/multi/cortical_area", tag = "cortical_area")]
#[allow(unused_variables)]  // In development
pub async fn put_multi_cortical_area(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Update multiple cortical areas
    Err(ApiError::internal("Not yet implemented"))
}

/// DELETE /v1/cortical_area/multi/cortical_area
#[utoipa::path(delete, path = "/v1/cortical_area/multi/cortical_area", tag = "cortical_area")]
#[allow(unused_variables)]  // In development
pub async fn delete_multi_cortical_area(
    State(state): State<ApiState>,
    Json(request): Json<Vec<String>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Delete multiple cortical areas
    Err(ApiError::internal("Not yet implemented"))
}

/// PUT /v1/cortical_area/coord_2d
#[utoipa::path(put, path = "/v1/cortical_area/coord_2d", tag = "cortical_area")]
#[allow(unused_variables)]  // In development
pub async fn put_coord_2d(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Update 2D coordinates
    Err(ApiError::internal("Not yet implemented"))
}

/// PUT /v1/cortical_area/suppress_cortical_visibility
#[utoipa::path(put, path = "/v1/cortical_area/suppress_cortical_visibility", tag = "cortical_area")]
#[allow(unused_variables)]  // In development
pub async fn put_suppress_cortical_visibility(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Suppress cortical visibility
    Err(ApiError::internal("Not yet implemented"))
}

/// PUT /v1/cortical_area/reset
#[utoipa::path(put, path = "/v1/cortical_area/reset", tag = "cortical_area")]
#[allow(unused_variables)]  // In development
pub async fn put_reset(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Reset cortical area
    Err(ApiError::internal("Not yet implemented"))
}

/// GET /v1/cortical_area/visualization
#[utoipa::path(get, path = "/v1/cortical_area/visualization", tag = "cortical_area")]
pub async fn get_visualization(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, bool>>> {
    let mut response = HashMap::new();
    response.insert("enabled".to_string(), true);
    Ok(Json(response))
}

/// POST /v1/cortical_area/batch_operations
#[utoipa::path(post, path = "/v1/cortical_area/batch_operations", tag = "cortical_area")]
pub async fn post_batch_operations(State(_state): State<ApiState>, Json(_ops): Json<Vec<HashMap<String, serde_json::Value>>>) -> ApiResult<Json<HashMap<String, i32>>> {
    let mut response = HashMap::new();
    response.insert("processed".to_string(), 0);
    Ok(Json(response))
}

/// GET /v1/cortical_area/ipu/list
#[utoipa::path(get, path = "/v1/cortical_area/ipu/list", tag = "cortical_area")]
pub async fn get_ipu_list(State(state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    get_ipu(State(state)).await
}

/// GET /v1/cortical_area/opu/list
#[utoipa::path(get, path = "/v1/cortical_area/opu/list", tag = "cortical_area")]
pub async fn get_opu_list(State(state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    get_opu(State(state)).await
}

/// PUT /v1/cortical_area/coordinates_3d
#[utoipa::path(put, path = "/v1/cortical_area/coordinates_3d", tag = "cortical_area")]
pub async fn put_coordinates_3d(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([("message".to_string(), "Not yet implemented".to_string())])))
}

/// DELETE /v1/cortical_area/bulk_delete
#[utoipa::path(delete, path = "/v1/cortical_area/bulk_delete", tag = "cortical_area")]
pub async fn delete_bulk(State(_state): State<ApiState>, Json(_ids): Json<Vec<String>>) -> ApiResult<Json<HashMap<String, i32>>> {
    let mut response = HashMap::new();
    response.insert("deleted_count".to_string(), 0);
    Ok(Json(response))
}

/// POST /v1/cortical_area/clone
#[utoipa::path(post, path = "/v1/cortical_area/clone", tag = "cortical_area")]
pub async fn post_clone_area(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, String>>) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([("message".to_string(), "Not yet implemented".to_string())])))
}

/// POST /v1/cortical_area/resize
#[utoipa::path(post, path = "/v1/cortical_area/resize", tag = "cortical_area")]
pub async fn post_resize(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([("message".to_string(), "Not yet implemented".to_string())])))
}

/// POST /v1/cortical_area/reposition
#[utoipa::path(post, path = "/v1/cortical_area/reposition", tag = "cortical_area")]
pub async fn post_reposition(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([("message".to_string(), "Not yet implemented".to_string())])))
}

/// POST /v1/cortical_area/voxel_neurons
#[utoipa::path(post, path = "/v1/cortical_area/voxel_neurons", tag = "cortical_area")]
pub async fn post_voxel_neurons(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let mut response = HashMap::new();
    response.insert("neurons".to_string(), serde_json::json!([]));
    Ok(Json(response))
}

// EXACT Python paths:
/// GET /v1/cortical_area/cortical_area_index_list
#[utoipa::path(get, path = "/v1/cortical_area/cortical_area_index_list", tag = "cortical_area")]
pub async fn get_cortical_area_index_list(State(state): State<ApiState>) -> ApiResult<Json<Vec<u32>>> {
    let connectome_service = state.connectome_service.as_ref();
    let areas = connectome_service.list_cortical_areas().await.map_err(|e| ApiError::internal(format!("{}", e)))?;
    // CRITICAL FIX: Return the actual cortical_idx values, not fabricated sequential indices
    let indices: Vec<u32> = areas.iter().map(|a| a.cortical_idx).collect();
    Ok(Json(indices))
}

/// GET /v1/cortical_area/cortical_idx_mapping
#[utoipa::path(get, path = "/v1/cortical_area/cortical_idx_mapping", tag = "cortical_area")]
pub async fn get_cortical_idx_mapping(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, u32>>> {
    let connectome_service = state.connectome_service.as_ref();
    let areas = connectome_service.list_cortical_areas().await.map_err(|e| ApiError::internal(format!("{}", e)))?;
    // CRITICAL FIX: Use the actual cortical_idx from CorticalArea, NOT enumerate() which ignores reserved indices!
    let mapping: HashMap<String, u32> = areas.iter().map(|a| (a.cortical_id.clone(), a.cortical_idx)).collect();
    Ok(Json(mapping))
}

/// GET /v1/cortical_area/mapping_restrictions
#[utoipa::path(get, path = "/v1/cortical_area/mapping_restrictions", tag = "cortical_area")]
pub async fn get_mapping_restrictions_query(State(_state): State<ApiState>, axum::extract::Query(_params): axum::extract::Query<HashMap<String, String>>) -> ApiResult<Json<HashMap<String, Vec<String>>>> {
    Ok(Json(HashMap::new()))
}

/// GET /v1/cortical_area/{cortical_id}/memory_usage
#[utoipa::path(get, path = "/v1/cortical_area/{cortical_id}/memory_usage", tag = "cortical_area")]
pub async fn get_memory_usage(State(state): State<ApiState>, axum::extract::Path(cortical_id): axum::extract::Path<String>) -> ApiResult<Json<HashMap<String, i64>>> {
    let connectome_service = state.connectome_service.as_ref();
    
    // CRITICAL FIX: Calculate actual memory usage based on neuron count instead of hardcoded 0
    let area_info = connectome_service.get_cortical_area(&cortical_id).await
        .map_err(|_| ApiError::not_found("CorticalArea", &cortical_id))?;
    
    // Calculate memory usage: neuron_count × bytes per neuron
    // Each neuron in NeuronArray uses ~48 bytes (membrane_potential, threshold, refractory, etc.)
    const BYTES_PER_NEURON: i64 = 48;
    let memory_bytes = (area_info.neuron_count as i64) * BYTES_PER_NEURON;
    
    let mut response = HashMap::new();
    response.insert("memory_bytes".to_string(), memory_bytes);
    Ok(Json(response))
}

/// GET /v1/cortical_area/{cortical_id}/neuron_count
#[utoipa::path(get, path = "/v1/cortical_area/{cortical_id}/neuron_count", tag = "cortical_area")]
pub async fn get_area_neuron_count(State(state): State<ApiState>, axum::extract::Path(cortical_id): axum::extract::Path<String>) -> ApiResult<Json<i64>> {
    let connectome_service = state.connectome_service.as_ref();
    
    // CRITICAL FIX: Get actual neuron count from ConnectomeService instead of hardcoded 0
    let area_info = connectome_service.get_cortical_area(&cortical_id).await
        .map_err(|_| ApiError::not_found("CorticalArea", &cortical_id))?;
    
    Ok(Json(area_info.neuron_count as i64))
}

/// POST /v1/cortical_area/cortical_type_options
#[utoipa::path(post, path = "/v1/cortical_area/cortical_type_options", tag = "cortical_area")]
pub async fn post_cortical_type_options(State(_state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    Ok(Json(vec!["Sensory".to_string(), "Motor".to_string(), "Custom".to_string(), "Memory".to_string()]))
}

/// POST /v1/cortical_area/mapping_restrictions
#[utoipa::path(post, path = "/v1/cortical_area/mapping_restrictions", tag = "cortical_area")]
pub async fn post_mapping_restrictions(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, String>>) -> ApiResult<Json<HashMap<String, Vec<String>>>> {
    Ok(Json(HashMap::new()))
}

/// POST /v1/cortical_area/mapping_restrictions_between_areas
#[utoipa::path(post, path = "/v1/cortical_area/mapping_restrictions_between_areas", tag = "cortical_area")]
pub async fn post_mapping_restrictions_between_areas(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, String>>) -> ApiResult<Json<HashMap<String, Vec<String>>>> {
    Ok(Json(HashMap::new()))
}

/// PUT /v1/cortical_area/coord_3d
#[utoipa::path(put, path = "/v1/cortical_area/coord_3d", tag = "cortical_area")]
pub async fn put_coord_3d(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([("message".to_string(), "Not yet implemented".to_string())])))
}

