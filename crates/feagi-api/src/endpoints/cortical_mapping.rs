// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Cortical Mapping API Endpoints - Exact port from Python `/v1/cortical_mapping/*`

use axum::{extract::State, response::Json};
use std::collections::HashMap;
use crate::common::{ApiError, ApiResult};
use crate::transports::http::server::ApiState;

/// POST /v1/cortical_mapping/afferents
#[utoipa::path(post, path = "/v1/cortical_mapping/afferents", tag = "cortical_mapping")]
pub async fn post_afferents(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, String>>) -> ApiResult<Json<Vec<String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// POST /v1/cortical_mapping/efferents
#[utoipa::path(post, path = "/v1/cortical_mapping/efferents", tag = "cortical_mapping")]
pub async fn post_efferents(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, String>>) -> ApiResult<Json<Vec<String>>> {
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
    Json(req): Json<HashMap<String, serde_json::Value>>
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    use tracing::debug;
    
    let src_area = req.get("src_cortical_area")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::invalid_input("Missing src_cortical_area"))?;
    
    let dst_area = req.get("dst_cortical_area")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::invalid_input("Missing dst_cortical_area"))?;
    
    debug!(target: "feagi-api", "Getting mapping properties: {} -> {}", src_area, dst_area);
    
    let connectome_service = state.connectome_service.as_ref();
    
    // Get source cortical area
    let src_area_info = connectome_service.get_cortical_area(src_area).await
        .map_err(|e| ApiError::not_found("Cortical area", &format!("Source area {}: {}", src_area, e)))?;
    
    // Look for cortical_mapping_dst in properties
    let mapping_dst = src_area_info.properties.get("cortical_mapping_dst")
        .and_then(|v| v.as_object());
    
    if mapping_dst.is_none() {
        debug!(target: "feagi-api", "No cortical_mapping_dst found for {}", src_area);
        return Ok(Json(vec![]));
    }
    
    // Get connections for this destination
    let connections = mapping_dst.unwrap()
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
            // Array format: [morphology_id, scalar, multiplier, plasticity_flag, constant, ltp, ltd]
            formatted.push(serde_json::json!({
                "morphology_id": arr.get(0).and_then(|v| v.as_str()).unwrap_or(""),
                "morphology_scalar": arr.get(1).cloned().unwrap_or(serde_json::json!([1, 1, 1])),
                "postSynapticCurrent_multiplier": arr.get(2).and_then(|v| v.as_i64()).unwrap_or(1),
                "plasticity_flag": arr.get(3).and_then(|v| v.as_bool()).unwrap_or(false),
                "plasticity_constant": arr.get(4).and_then(|v| v.as_i64()).unwrap_or(1),
                "ltp_multiplier": arr.get(5).and_then(|v| v.as_i64()).unwrap_or(1),
                "ltd_multiplier": arr.get(6).and_then(|v| v.as_i64()).unwrap_or(1),
            }));
        } else if let Some(obj) = conn.as_object() {
            // Dict format - already in expected schema
            formatted.push(serde_json::json!({
                "morphology_id": obj.get("morphology_id").and_then(|v| v.as_str()).unwrap_or(""),
                "morphology_scalar": obj.get("morphology_scalar").cloned().unwrap_or(serde_json::json!([1, 1, 1])),
                "postSynapticCurrent_multiplier": obj.get("postSynapticCurrent_multiplier").and_then(|v| v.as_i64()).unwrap_or(1),
                "plasticity_flag": obj.get("plasticity_flag").and_then(|v| v.as_bool()).unwrap_or(false),
                "plasticity_constant": obj.get("plasticity_constant").and_then(|v| v.as_i64()).unwrap_or(1),
                "ltp_multiplier": obj.get("ltp_multiplier").and_then(|v| v.as_i64()).unwrap_or(1),
                "ltd_multiplier": obj.get("ltd_multiplier").and_then(|v| v.as_i64()).unwrap_or(1),
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
    Json(req): Json<HashMap<String, serde_json::Value>>
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    use tracing::{info, debug};
    
    let src_area = req.get("src_cortical_area")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::invalid_input("Missing src_cortical_area"))?;
    
    let dst_area = req.get("dst_cortical_area")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::invalid_input("Missing dst_cortical_area"))?;
    
    let mapping_string = req.get("mapping_string")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ApiError::invalid_input("Missing mapping_string"))?;
    
    info!(target: "feagi-api", "PUT cortical mapping: {} -> {} with {} connections", 
          src_area, dst_area, mapping_string.len());
    debug!(target: "feagi-api", "Mapping data: {:?}", mapping_string);
    
    let connectome_service = state.connectome_service.as_ref();
    
    // Update the cortical mapping (this modifies ConnectomeManager and regenerates synapses)
    let synapse_count = connectome_service.update_cortical_mapping(
        src_area.to_string(),
        dst_area.to_string(),
        mapping_string.clone(),
    ).await
        .map_err(|e| ApiError::internal(format!("Failed to update cortical mapping: {}", e)))?;
    
    info!(target: "feagi-api", "Cortical mapping updated successfully: {} synapses created", synapse_count);
    
    // Return success response matching Python format
    let mut response = HashMap::new();
    response.insert("message".to_string(), serde_json::json!(
        format!("Cortical mapping properties updated successfully from {} to {}", src_area, dst_area)
    ));
    response.insert("synapse_count".to_string(), serde_json::json!(synapse_count));
    response.insert("src_region".to_string(), serde_json::json!(null)); // TODO: Add region context
    response.insert("dst_region".to_string(), serde_json::json!(null)); // TODO: Add region context
    
    Ok(Json(response))
}



