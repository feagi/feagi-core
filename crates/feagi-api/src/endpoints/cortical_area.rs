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
#[utoipa::path(get, path = "/v1/cortical_area/cortical_map_detailed", tag = "cortical_area")]
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
#[utoipa::path(get, path = "/v1/cortical_area/cortical_area/geometry", tag = "cortical_area")]
pub async fn get_cortical_area_geometry(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_cortical_areas().await {
        Ok(areas) => {
            let geometry: HashMap<String, serde_json::Value> = areas.into_iter()
                .map(|area| {
                    let geo = serde_json::json!({
                        "dimensions": {
                            "x": area.dimensions.0,
                            "y": area.dimensions.1,
                            "z": area.dimensions.2
                        },
                        "position": {
                            "x": area.position.0,
                            "y": area.position.1,
                            "z": area.position.2
                        }
                    });
                    (area.cortical_id, geo)
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

