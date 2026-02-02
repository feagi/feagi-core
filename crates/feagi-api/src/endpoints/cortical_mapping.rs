// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Cortical Mapping API Endpoints - Exact port from Python `/v1/cortical_mapping/*`

// Removed - using crate::common::State instead
use crate::common::ApiState;
use crate::common::{ApiError, ApiResult, Json, Query, State};
use std::collections::HashMap;

/// POST /v1/cortical_mapping/afferents
#[utoipa::path(
    post,
    path = "/v1/cortical_mapping/afferents",
    tag = "cortical_mapping"
)]
pub async fn post_afferents(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, String>>,
) -> ApiResult<Json<Vec<String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// POST /v1/cortical_mapping/efferents
#[utoipa::path(
    post,
    path = "/v1/cortical_mapping/efferents",
    tag = "cortical_mapping"
)]
pub async fn post_efferents(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, String>>,
) -> ApiResult<Json<Vec<String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// POST /v1/cortical_mapping/mapping_properties
#[utoipa::path(
    post,
    path = "/v1/cortical_mapping/mapping_properties",
    tag = "cortical_mapping",
    responses(
        (status = 200, description = "Cortical mapping connections", body = Vec<serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_mapping_properties(
    State(state): State<ApiState>,
    Json(req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    use tracing::debug;

    let src_area = req
        .get("src_cortical_area")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::invalid_input("Missing src_cortical_area"))?;

    let dst_area = req
        .get("dst_cortical_area")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::invalid_input("Missing dst_cortical_area"))?;

    debug!(target: "feagi-api", "Getting mapping properties: {} -> {}", src_area, dst_area);

    let connectome_service = state.connectome_service.as_ref();

    // Get source cortical area
    let src_area_info = connectome_service
        .get_cortical_area(src_area)
        .await
        .map_err(|e| {
            ApiError::not_found("Cortical area", &format!("Source area {}: {}", src_area, e))
        })?;

    // Look for cortical_mapping_dst in properties
    let mapping_dst = src_area_info
        .properties
        .get("cortical_mapping_dst")
        .and_then(|v| v.as_object());

    if mapping_dst.is_none() {
        debug!(target: "feagi-api", "No cortical_mapping_dst found for {}", src_area);
        return Ok(Json(vec![]));
    }

    // Get connections for this destination
    let connections = mapping_dst
        .unwrap()
        .get(dst_area)
        .and_then(|v| v.as_array());

    if connections.is_none() {
        debug!(target: "feagi-api", "No connections found from {} to {}", src_area, dst_area);
        return Ok(Json(vec![]));
    }

    // Normalize connections to expected format
    let mut formatted = Vec::new();
    for conn in connections.unwrap() {
        if let Some(arr) = conn.as_array() {
            // Array format:
            // [morphology_id, morphology_scalar, psc_multiplier, plasticity_flag,
            //  plasticity_constant, ltp_multiplier, ltd_multiplier, plasticity_window]
            if arr.len() < 8 {
                return Err(ApiError::invalid_input(format!(
                    "Invalid dstmap rule array (expected 8 elements including plasticity_window), got {}: {:?}",
                    arr.len(),
                    arr
                )));
            }
            // Strict parsing (no implicit defaults).
            let morphology_id = arr[0]
                .as_str()
                .ok_or_else(|| ApiError::invalid_input("morphology_id must be a string"))?;
            let morphology_scalar = arr[1].clone();
            let psc_multiplier = arr[2].as_i64().ok_or_else(|| {
                ApiError::invalid_input("postSynapticCurrent_multiplier must be an integer")
            })?;
            let plasticity_flag = arr[3]
                .as_bool()
                .ok_or_else(|| ApiError::invalid_input("plasticity_flag must be a boolean"))?;
            let plasticity_constant = arr[4]
                .as_i64()
                .ok_or_else(|| ApiError::invalid_input("plasticity_constant must be an integer"))?;
            let ltp_multiplier = arr[5]
                .as_i64()
                .ok_or_else(|| ApiError::invalid_input("ltp_multiplier must be an integer"))?;
            let ltd_multiplier = arr[6]
                .as_i64()
                .ok_or_else(|| ApiError::invalid_input("ltd_multiplier must be an integer"))?;
            let plasticity_window = arr[7]
                .as_i64()
                .ok_or_else(|| ApiError::invalid_input("plasticity_window must be an integer"))?;

            formatted.push(serde_json::json!({
                "morphology_id": morphology_id,
                "morphology_scalar": morphology_scalar,
                "postSynapticCurrent_multiplier": psc_multiplier,
                "plasticity_flag": plasticity_flag,
                "plasticity_constant": plasticity_constant,
                "ltp_multiplier": ltp_multiplier,
                "ltd_multiplier": ltd_multiplier,
                "plasticity_window": plasticity_window,
            }));
        } else if let Some(obj) = conn.as_object() {
            // Dict format - strict schema (no implicit defaults)
            let morphology_id = obj
                .get("morphology_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ApiError::invalid_input("morphology_id must be a string"))?;
            let morphology_scalar = obj
                .get("morphology_scalar")
                .cloned()
                .ok_or_else(|| ApiError::invalid_input("morphology_scalar missing"))?;
            let psc_multiplier = obj
                .get("postSynapticCurrent_multiplier")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| {
                    ApiError::invalid_input("postSynapticCurrent_multiplier must be an integer")
                })?;
            let plasticity_flag = obj
                .get("plasticity_flag")
                .and_then(|v| v.as_bool())
                .ok_or_else(|| ApiError::invalid_input("plasticity_flag must be a boolean"))?;
            let plasticity_constant = obj
                .get("plasticity_constant")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| ApiError::invalid_input("plasticity_constant must be an integer"))?;
            let ltp_multiplier = obj
                .get("ltp_multiplier")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| ApiError::invalid_input("ltp_multiplier must be an integer"))?;
            let ltd_multiplier = obj
                .get("ltd_multiplier")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| ApiError::invalid_input("ltd_multiplier must be an integer"))?;
            let plasticity_window = obj
                .get("plasticity_window")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| ApiError::invalid_input("plasticity_window must be an integer"))?;

            formatted.push(serde_json::json!({
                "morphology_id": morphology_id,
                "morphology_scalar": morphology_scalar,
                "postSynapticCurrent_multiplier": psc_multiplier,
                "plasticity_flag": plasticity_flag,
                "plasticity_constant": plasticity_constant,
                "ltp_multiplier": ltp_multiplier,
                "ltd_multiplier": ltd_multiplier,
                "plasticity_window": plasticity_window,
            }));
        }
    }

    debug!(target: "feagi-api", "Returning {} mapping connections from {} to {}", formatted.len(), src_area, dst_area);
    Ok(Json(formatted))
}

/// PUT /v1/cortical_mapping/mapping_properties
#[utoipa::path(
    put,
    path = "/v1/cortical_mapping/mapping_properties",
    tag = "cortical_mapping",
    responses(
        (status = 200, description = "Cortical mapping updated successfully", body = HashMap<String, serde_json::Value>),
        (status = 404, description = "Cortical area not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn put_mapping_properties(
    State(state): State<ApiState>,
    Json(req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    use tracing::{debug, info};

    let src_area = req
        .get("src_cortical_area")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::invalid_input("Missing src_cortical_area"))?;

    let dst_area = req
        .get("dst_cortical_area")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::invalid_input("Missing dst_cortical_area"))?;

    let mapping_string = req
        .get("mapping_string")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ApiError::invalid_input("Missing mapping_string"))?;

    info!(
        target: "feagi-api",
        "PUT cortical mapping: {} -> {} with {} connections",
        src_area,
        dst_area,
        mapping_string.len()
    );
    debug!(target: "feagi-api", "Mapping data: {:?}", mapping_string);

    let connectome_service = state.connectome_service.as_ref();

    // Update the cortical mapping (this modifies ConnectomeManager and regenerates synapses)
    let synapse_count = connectome_service
        .update_cortical_mapping(
            src_area.to_string(),
            dst_area.to_string(),
            mapping_string.clone(),
        )
        .await
        .map_err(|e| match e {
            feagi_services::types::ServiceError::InvalidInput(msg) => ApiError::invalid_input(msg),
            _ => ApiError::internal(format!("Failed to update cortical mapping: {}", e)),
        })?;

    info!(target: "feagi-api", "Cortical mapping updated successfully: {} synapses created", synapse_count);

    // Return success response matching Python format
    let mut response = HashMap::new();
    response.insert(
        "message".to_string(),
        serde_json::json!(format!(
            "Cortical mapping properties updated successfully from {} to {}",
            src_area, dst_area
        )),
    );
    response.insert(
        "synapse_count".to_string(),
        serde_json::json!(synapse_count),
    );
    response.insert("src_region".to_string(), serde_json::json!(null)); // TODO: Add region context
    response.insert("dst_region".to_string(), serde_json::json!(null)); // TODO: Add region context

    Ok(Json(response))
}

/// GET /v1/cortical_mapping/mapping
/// Get specific cortical mapping between two areas
#[utoipa::path(
    get,
    path = "/v1/cortical_mapping/mapping",
    tag = "cortical_mapping",
    params(
        ("src_cortical_area" = String, Query, description = "Source cortical area ID"),
        ("dst_cortical_area" = String, Query, description = "Destination cortical area ID")
    ),
    responses(
        (status = 200, description = "Mapping properties", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn get_mapping(
    State(state): State<ApiState>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let src_area = params
        .get("src_cortical_area")
        .ok_or_else(|| ApiError::invalid_input("src_cortical_area required"))?;
    let dst_area = params
        .get("dst_cortical_area")
        .ok_or_else(|| ApiError::invalid_input("dst_cortical_area required"))?;

    // Get mapping properties directly (avoid recursion)
    let connectome_service = state.connectome_service.as_ref();

    // Get source cortical area
    let src_area_info = connectome_service
        .get_cortical_area(src_area)
        .await
        .map_err(|e| {
            ApiError::not_found("Cortical area", &format!("Source area {}: {}", src_area, e))
        })?;

    // Look for cortical_mapping_dst in properties
    let mapping_dst = src_area_info
        .properties
        .get("cortical_mapping_dst")
        .and_then(|v| v.as_object());

    if mapping_dst.is_none() {
        return Ok(Json(HashMap::new()));
    }

    // Get connections for this destination
    let connections = mapping_dst
        .unwrap()
        .get(dst_area)
        .and_then(|v| v.as_array());

    let mut response = HashMap::new();
    response.insert(
        "connections".to_string(),
        serde_json::json!(connections.unwrap_or(&vec![])),
    );

    Ok(Json(response))
}

/// GET /v1/cortical_mapping/mapping_list
/// Get list of all cortical mappings
#[utoipa::path(
    get,
    path = "/v1/cortical_mapping/mapping_list",
    tag = "cortical_mapping",
    responses(
        (status = 200, description = "List of all mappings", body = Vec<Vec<String>>)
    )
)]
pub async fn get_mapping_list(State(state): State<ApiState>) -> ApiResult<Json<Vec<Vec<String>>>> {
    let connectome_service = state.connectome_service.as_ref();

    let areas = connectome_service
        .list_cortical_areas()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to list areas: {}", e)))?;

    let mut mappings = Vec::new();

    // Scan all cortical_mapping_dst properties
    for area in &areas {
        if let Ok(area_detail) = connectome_service
            .get_cortical_area(&area.cortical_id)
            .await
        {
            if let Some(mapping_dst) = area_detail.properties.get("cortical_mapping_dst") {
                if let Some(dst_map) = mapping_dst.as_object() {
                    for dst_area_id in dst_map.keys() {
                        mappings.push(vec![area.cortical_id.clone(), dst_area_id.clone()]);
                    }
                }
            }
        }
    }

    Ok(Json(mappings))
}

/// DELETE /v1/cortical_mapping/mapping
/// Delete a cortical mapping
#[utoipa::path(
    delete,
    path = "/v1/cortical_mapping/mapping",
    tag = "cortical_mapping",
    responses(
        (status = 200, description = "Mapping deleted", body = HashMap<String, String>)
    )
)]
pub async fn delete_mapping(
    State(_state): State<ApiState>,
    Json(_request): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Implement mapping deletion
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Mapping deletion not yet implemented".to_string(),
    )])))
}

/// POST /v1/cortical_mapping/batch_update
/// Batch update multiple cortical mappings
#[utoipa::path(
    post,
    path = "/v1/cortical_mapping/batch_update",
    tag = "cortical_mapping",
    responses(
        (status = 200, description = "Batch update completed", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn post_batch_update(
    State(_state): State<ApiState>,
    Json(_request): Json<Vec<HashMap<String, String>>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // TODO: Implement batch update
    let mut response = HashMap::new();
    response.insert(
        "message".to_string(),
        serde_json::json!("Batch update not yet implemented"),
    );
    response.insert("updated_count".to_string(), serde_json::json!(0));

    Ok(Json(response))
}

// EXACT Python paths:
/// POST /v1/cortical_mapping/mapping
#[utoipa::path(post, path = "/v1/cortical_mapping/mapping", tag = "cortical_mapping")]
pub async fn post_mapping(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Not yet implemented".to_string(),
    )])))
}

/// PUT /v1/cortical_mapping/mapping
#[utoipa::path(put, path = "/v1/cortical_mapping/mapping", tag = "cortical_mapping")]
pub async fn put_mapping(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Not yet implemented".to_string(),
    )])))
}
