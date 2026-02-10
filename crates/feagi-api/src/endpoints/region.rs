// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Region API Endpoints - Exact port from Python `/v1/region/*`

// Removed - using crate::common::State instead
use crate::common::ApiState;
use crate::common::{ApiError, ApiResult, Json, Path, State};
use feagi_services::types::CreateBrainRegionParams;
use feagi_structures::genomic::brain_regions::RegionID;
use std::collections::HashMap;

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
pub async fn get_regions_members(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    use tracing::trace;
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_brain_regions().await {
        Ok(regions) => {
            trace!(target: "feagi-api", "Found {} brain regions to return", regions.len());
            let mut result = HashMap::new();
            for region in regions {
                trace!(
                    target: "feagi-api",
                    "Region: {} ({}) with {} areas",
                    region.region_id,
                    region.name,
                    region.cortical_areas.len()
                );

                // Extract inputs/outputs from region properties if they exist
                let inputs = region
                    .properties
                    .get("inputs")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect::<Vec<String>>()
                    })
                    .unwrap_or_default();

                let outputs = region
                    .properties
                    .get("outputs")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect::<Vec<String>>()
                    })
                    .unwrap_or_default();

                trace!(
                    target: "feagi-api",
                    "Inputs: {} areas, Outputs: {} areas",
                    inputs.len(),
                    outputs.len()
                );

                // Extract coordinate_3d from properties (set by smart positioning in neuroembryogenesis)
                let coordinate_3d = region
                    .properties
                    .get("coordinate_3d")
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
                let coordinate_2d = region
                    .properties
                    .get("coordinate_2d")
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
                    }),
                );
            }
            trace!(target: "feagi-api", "Returning {} regions in response", result.len());
            Ok(Json(result))
        }
        Err(e) => Err(ApiError::internal(format!("Failed to get regions: {}", e))),
    }
}

/// POST /v1/region/region
#[utoipa::path(post, path = "/v1/region/region", tag = "region")]
pub async fn post_region(
    State(state): State<ApiState>,
    Json(mut req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let connectome_service = state.connectome_service.as_ref();

    let title = req
        .get("title")
        .or_else(|| req.get("name"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ApiError::invalid_input("title required"))?
        .to_string();
    req.remove("title");
    req.remove("name");

    let region_id = match req.get("region_id").and_then(|v| v.as_str()) {
        Some(value) if !value.trim().is_empty() => RegionID::from_string(value)
            .map_err(|e| ApiError::invalid_input(format!("Invalid region_id: {}", e)))?
            .to_string(),
        _ => RegionID::new().to_string(),
    };
    req.remove("region_id");

    let parent_region_id = req
        .get("parent_region_id")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    req.remove("parent_region_id");

    let region_type = req
        .get("region_type")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| "Undefined".to_string());
    req.remove("region_type");

    let coordinate_2d_value = req
        .get("coordinate_2d")
        .or_else(|| req.get("coordinates_2d"))
        .and_then(|v| v.as_array().cloned())
        .ok_or_else(|| ApiError::invalid_input("coordinates_2d required"))?;
    if coordinate_2d_value.len() != 2 {
        return Err(ApiError::invalid_input(
            "coordinates_2d must contain exactly 2 values",
        ));
    }
    req.remove("coordinate_2d");
    req.remove("coordinates_2d");

    let coordinate_3d_value = req
        .get("coordinate_3d")
        .or_else(|| req.get("coordinates_3d"))
        .and_then(|v| v.as_array().cloned())
        .ok_or_else(|| ApiError::invalid_input("coordinates_3d required"))?;
    if coordinate_3d_value.len() != 3 {
        return Err(ApiError::invalid_input(
            "coordinates_3d must contain exactly 3 values",
        ));
    }
    req.remove("coordinate_3d");
    req.remove("coordinates_3d");

    let mut properties: HashMap<String, serde_json::Value> = HashMap::new();
    properties.insert(
        "coordinate_2d".to_string(),
        serde_json::Value::Array(coordinate_2d_value),
    );
    properties.insert(
        "coordinate_3d".to_string(),
        serde_json::Value::Array(coordinate_3d_value),
    );
    if let Some(parent_region_id) = &parent_region_id {
        properties.insert(
            "parent_region_id".to_string(),
            serde_json::json!(parent_region_id),
        );
    }
    if let Some(areas) = req.remove("areas") {
        properties.insert("areas".to_string(), areas);
    }
    if let Some(regions) = req.remove("regions") {
        properties.insert("regions".to_string(), regions);
    }
    for (key, value) in req {
        properties.insert(key, value);
    }

    let params = CreateBrainRegionParams {
        region_id: region_id.clone(),
        name: title.clone(),
        region_type,
        parent_id: parent_region_id.clone(),
        properties: Some(properties),
    };

    let info = connectome_service
        .create_brain_region(params)
        .await
        .map_err(ApiError::from)?;

    let coordinate_2d = info
        .properties
        .get("coordinate_2d")
        .cloned()
        .ok_or_else(|| ApiError::internal("Missing coordinate_2d on created region"))?;
    let coordinate_3d = info
        .properties
        .get("coordinate_3d")
        .cloned()
        .ok_or_else(|| ApiError::internal("Missing coordinate_3d on created region"))?;

    let mut response = HashMap::from([
        ("region_id".to_string(), serde_json::json!(info.region_id)),
        ("title".to_string(), serde_json::json!(info.name)),
        (
            "parent_region_id".to_string(),
            serde_json::json!(info.parent_id),
        ),
        ("coordinate_2d".to_string(), coordinate_2d),
        ("coordinate_3d".to_string(), coordinate_3d),
        ("areas".to_string(), serde_json::json!(info.cortical_areas)),
        ("regions".to_string(), serde_json::json!(info.child_regions)),
    ]);

    if let Some(inputs) = info.properties.get("inputs") {
        response.insert("inputs".to_string(), inputs.clone());
    }
    if let Some(outputs) = info.properties.get("outputs") {
        response.insert("outputs".to_string(), outputs.clone());
    }

    Ok(Json(response))
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
    match connectome_service
        .update_brain_region(&region_id, request)
        .await
    {
        Ok(_) => Ok(Json(HashMap::from([
            ("message".to_string(), "Brain region updated".to_string()),
            ("region_id".to_string(), region_id),
        ]))),
        Err(e) => Err(ApiError::internal(format!(
            "Failed to update brain region: {}",
            e
        ))),
    }
}

/// DELETE /v1/region/region
#[utoipa::path(delete, path = "/v1/region/region", tag = "region")]
pub async fn delete_region(
    State(state): State<ApiState>,
    Json(req): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let connectome_service = state.connectome_service.as_ref();
    let region_id = req
        .get("region_id")
        .or_else(|| req.get("id"))
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ApiError::invalid_input("region_id required"))?
        .to_string();

    connectome_service
        .delete_brain_region(&region_id)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(HashMap::from([
        ("message".to_string(), "Brain region deleted".to_string()),
        ("region_id".to_string(), region_id),
    ])))
}

/// POST /v1/region/clone
#[utoipa::path(post, path = "/v1/region/clone", tag = "region")]
pub async fn post_clone(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// PUT /v1/region/relocate_members
#[utoipa::path(put, path = "/v1/region/relocate_members", tag = "region")]
pub async fn put_relocate_members(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let connectome_service = state.connectome_service.as_ref();

    if request.is_empty() {
        return Err(ApiError::invalid_input("Request cannot be empty"));
    }

    let mut updated_regions: Vec<String> = Vec::new();

    for (region_id, payload) in request {
        let payload_obj = payload.as_object().ok_or_else(|| {
            ApiError::invalid_input(format!("Region '{}' entry must be an object", region_id))
        })?;

        if payload_obj.contains_key("parent_region_id") {
            return Err(ApiError::invalid_input(
                "parent_region_id relocation is not implemented via relocate_members",
            ));
        }

        let mut properties: HashMap<String, serde_json::Value> = HashMap::new();
        if let Some(value) = payload_obj
            .get("coordinate_2d")
            .or_else(|| payload_obj.get("coordinates_2d"))
        {
            properties.insert("coordinate_2d".to_string(), value.clone());
        }
        if let Some(value) = payload_obj
            .get("coordinate_3d")
            .or_else(|| payload_obj.get("coordinates_3d"))
        {
            properties.insert("coordinate_3d".to_string(), value.clone());
        }

        if properties.is_empty() {
            return Err(ApiError::invalid_input(format!(
                "Region '{}' has no supported properties to update",
                region_id
            )));
        }

        connectome_service
            .update_brain_region(&region_id, properties)
            .await
            .map_err(|e| {
                ApiError::internal(format!("Failed to update region {}: {}", region_id, e))
            })?;

        updated_regions.push(region_id);
    }

    Ok(Json(HashMap::from([
        (
            "message".to_string(),
            format!("Updated {} brain regions", updated_regions.len()),
        ),
        ("region_ids".to_string(), updated_regions.join(", ")),
    ])))
}

/// DELETE /v1/region/region_and_members
#[utoipa::path(delete, path = "/v1/region/region_and_members", tag = "region")]
pub async fn delete_region_and_members(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, String>>> {
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

    let regions = connectome_service
        .list_brain_regions()
        .await
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
pub async fn get_region_titles(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let connectome_service = state.connectome_service.as_ref();

    let regions = connectome_service
        .list_brain_regions()
        .await
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
    Path(region_id): Path<String>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let connectome_service = state.connectome_service.as_ref();

    let region = connectome_service
        .get_brain_region(&region_id)
        .await
        .map_err(|e| ApiError::not_found("region", &e.to_string()))?;

    // Extract coordinate_3d from properties (set by smart positioning in neuroembryogenesis)
    let coordinate_3d = region
        .properties
        .get("coordinate_3d")
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
    let coordinate_2d = region
        .properties
        .get("coordinate_2d")
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
    response.insert(
        "areas".to_string(),
        serde_json::json!(region.cortical_areas),
    );
    response.insert(
        "regions".to_string(),
        serde_json::json!(region.child_regions),
    );
    response.insert(
        "parent_region_id".to_string(),
        serde_json::json!(region.parent_id),
    );

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
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Region parent change not yet implemented".to_string(),
    )])))
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
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Cortical area region association change not yet implemented".to_string(),
    )])))
}
