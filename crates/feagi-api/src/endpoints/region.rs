// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Region API Endpoints - Exact port from Python `/v1/region/*`

// Removed - using crate::common::State instead
use std::collections::HashMap;
use crate::common::{ApiError, ApiResult, State, Json};
use crate::transports::http::server::ApiState;

/// GET /v1/region/regions_members
/// 
/// Returns all brain regions with their member cortical areas
/// 
/// Example response:
/// ```json
/// {
///   "root": {
///     "title": "Root Brain Region",
///     "description": "",
///     "parent_region_id": null,
///     "coordinate_2d": [0, 0],
///     "coordinate_3d": [0, 0, 0],
///     "areas": ["area1", "area2"],
///     "regions": [],
///     "inputs": [],
///     "outputs": []
///   }
/// }
/// ```
#[utoipa::path(
    get, 
    path = "/v1/region/regions_members",
    tag = "region",
    responses(
        (status = 200, description = "Brain regions with member areas", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_regions_members(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    use tracing::debug;
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_brain_regions().await {
        Ok(regions) => {
            debug!(target: "feagi-api", "📋 Found {} brain regions to return", regions.len());
            let mut result = HashMap::new();
            for region in regions {
                debug!(target: "feagi-api", "  - Region: {} ({}) with {} areas", region.region_id, region.name, region.cortical_areas.len());
                
                // Extract inputs/outputs from region properties if they exist
                let inputs = region.properties.get("inputs")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<String>>())
                    .unwrap_or_default();
                
                let outputs = region.properties.get("outputs")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<String>>())
                    .unwrap_or_default();
                
                debug!(target: "feagi-api", "    - Inputs: {} areas, Outputs: {} areas", inputs.len(), outputs.len());
                
                // Extract coordinate_3d from properties (set by smart positioning in neuroembryogenesis)
                let coordinate_3d = region.properties.get("coordinate_3d")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| {
                        if arr.len() >= 3 {
                            Some(serde_json::json!([arr[0], arr[1], arr[2]]))
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| serde_json::json!([0, 0, 0]));
                
                // Extract coordinate_2d from properties
                let coordinate_2d = region.properties.get("coordinate_2d")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| {
                        if arr.len() >= 2 {
                            Some(serde_json::json!([arr[0], arr[1]]))
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| serde_json::json!([0, 0]));
                
                result.insert(
                    region.region_id.clone(),
                    serde_json::json!({
                        "title": region.name,
                        "description": "",  // TODO: Add description field to BrainRegionInfo
                        "parent_region_id": region.parent_id,
                        "coordinate_2d": coordinate_2d,
                        "coordinate_3d": coordinate_3d,
                        "areas": region.cortical_areas,
                        "regions": region.child_regions,
                        "inputs": inputs,
                        "outputs": outputs
                    })
                );
            }
            debug!(target: "feagi-api", "📋 Returning {} regions in response", result.len());
            Ok(Json(result))
        },
        Err(e) => Err(ApiError::internal(format!("Failed to get regions: {}", e))),
    }
}

/// POST /v1/region/region
#[utoipa::path(post, path = "/v1/region/region", tag = "region")]
pub async fn post_region(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// PUT /v1/region/region
#[utoipa::path(put, path = "/v1/region/region", tag = "region")]
pub async fn put_region(
    State(state): State<ApiState>,
    Json(mut request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let connectome_service = state.connectome_service.as_ref();
    
    // Extract region_id
    let region_id = request
        .get("region_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::invalid_input("region_id required"))?
        .to_string();
    
    // Remove region_id from properties (it's not a property to update)
    request.remove("region_id");
    
    // Update the brain region
    match connectome_service.update_brain_region(&region_id, request).await {
        Ok(_) => Ok(Json(HashMap::from([
            ("message".to_string(), "Brain region updated".to_string()),
            ("region_id".to_string(), region_id),
        ]))),
        Err(e) => Err(ApiError::internal(format!("Failed to update brain region: {}", e))),
    }
}

/// DELETE /v1/region/region
#[utoipa::path(delete, path = "/v1/region/region", tag = "region")]
pub async fn delete_region(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, String>>) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// POST /v1/region/clone
#[utoipa::path(post, path = "/v1/region/clone", tag = "region")]
pub async fn post_clone(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// PUT /v1/region/relocate_members
#[utoipa::path(put, path = "/v1/region/relocate_members", tag = "region")]
pub async fn put_relocate_members(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// DELETE /v1/region/region_and_members
#[utoipa::path(delete, path = "/v1/region/region_and_members", tag = "region")]
pub async fn delete_region_and_members(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, String>>) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// GET /v1/region/regions
/// Get list of all brain region IDs
#[utoipa::path(
    get,
    path = "/v1/region/regions",
    tag = "region",
    responses(
        (status = 200, description = "List of region IDs", body = Vec<String>)
    )
)]
pub async fn get_regions(State(state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    let connectome_service = state.connectome_service.as_ref();
    
    let regions = connectome_service.list_brain_regions().await
        .map_err(|e| ApiError::internal(format!("Failed to list regions: {}", e)))?;
    
    let region_ids: Vec<String> = regions.iter().map(|r| r.region_id.clone()).collect();
    Ok(Json(region_ids))
}

/// GET /v1/region/region_titles
/// Get mapping of region IDs to titles
#[utoipa::path(
    get,
    path = "/v1/region/region_titles",
    tag = "region",
    responses(
        (status = 200, description = "Region ID to title mapping", body = HashMap<String, String>)
    )
)]
pub async fn get_region_titles(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, String>>> {
    let connectome_service = state.connectome_service.as_ref();
    
    let regions = connectome_service.list_brain_regions().await
        .map_err(|e| ApiError::internal(format!("Failed to list regions: {}", e)))?;
    
    let mut titles = HashMap::new();
    for region in regions {
        titles.insert(region.region_id.clone(), region.name.clone());
    }
    
    Ok(Json(titles))
}

/// GET /v1/region/region/{region_id}
/// Get detailed properties for a specific brain region
#[utoipa::path(
    get,
    path = "/v1/region/region/{region_id}",
    tag = "region",
    params(
        ("region_id" = String, Path, description = "Brain region ID")
    ),
    responses(
        (status = 200, description = "Region properties", body = HashMap<String, serde_json::Value>),
        (status = 404, description = "Region not found")
    )
)]
pub async fn get_region_detail(
    State(state): State<ApiState>,
    axum::extract::Path(region_id): axum::extract::Path<String>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let connectome_service = state.connectome_service.as_ref();
    
    let region = connectome_service.get_brain_region(&region_id).await
        .map_err(|e| ApiError::not_found("region", &e.to_string()))?;
    
    // Extract coordinate_3d from properties (set by smart positioning in neuroembryogenesis)
    let coordinate_3d = region.properties.get("coordinate_3d")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            if arr.len() >= 3 {
                Some(serde_json::json!([arr[0], arr[1], arr[2]]))
            } else {
                None
            }
        })
        .unwrap_or_else(|| serde_json::json!([0, 0, 0]));
    
    // Extract coordinate_2d from properties
    let coordinate_2d = region.properties.get("coordinate_2d")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            if arr.len() >= 2 {
                Some(serde_json::json!([arr[0], arr[1]]))
            } else {
                None
            }
        })
        .unwrap_or_else(|| serde_json::json!([0, 0]));
    
    let mut response = HashMap::new();
    response.insert("region_id".to_string(), serde_json::json!(region.region_id));
    response.insert("title".to_string(), serde_json::json!(region.name));
    response.insert("description".to_string(), serde_json::json!(""));
    response.insert("coordinate_2d".to_string(), coordinate_2d);
    response.insert("coordinate_3d".to_string(), coordinate_3d);
    response.insert("areas".to_string(), serde_json::json!(region.cortical_areas));
    response.insert("regions".to_string(), serde_json::json!(region.child_regions));
    response.insert("parent_region_id".to_string(), serde_json::json!(region.parent_id));
    
    Ok(Json(response))
}

/// PUT /v1/region/change_region_parent
/// Change the parent of a brain region
#[utoipa::path(
    put,
    path = "/v1/region/change_region_parent",
    tag = "region",
    responses(
        (status = 200, description = "Parent changed", body = HashMap<String, String>)
    )
)]
pub async fn put_change_region_parent(
    State(_state): State<ApiState>,
    Json(_request): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([
        ("message".to_string(), "Region parent change not yet implemented".to_string())
    ])))
}

/// PUT /v1/region/change_cortical_area_region
/// Change the region association of a cortical area
#[utoipa::path(
    put,
    path = "/v1/region/change_cortical_area_region",
    tag = "region",
    responses(
        (status = 200, description = "Association changed", body = HashMap<String, String>)
    )
)]
pub async fn put_change_cortical_area_region(
    State(_state): State<ApiState>,
    Json(_request): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([
        ("message".to_string(), "Cortical area region association change not yet implemented".to_string())
    ])))
}

