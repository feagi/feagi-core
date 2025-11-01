// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Morphology API Endpoints - Exact port from Python `/v1/morphology/*`

use axum::{extract::State, response::Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::common::{ApiError, ApiResult};
use crate::transports::http::server::ApiState;

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MorphologyListResponse {
    pub morphology_list: Vec<String>,
}

/// GET /v1/morphology/morphology_list
#[utoipa::path(get, path = "/v1/morphology/morphology_list", tag = "morphology")]
pub async fn get_morphology_list(State(_state): State<ApiState>) -> ApiResult<Json<MorphologyListResponse>> {
    // TODO: Get from morphology service
    Ok(Json(MorphologyListResponse { morphology_list: vec![] }))
}

/// GET /v1/morphology/morphology_types
#[utoipa::path(get, path = "/v1/morphology/morphology_types", tag = "morphology")]
pub async fn get_morphology_types(State(_state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    Ok(Json(vec!["vectors".to_string(), "patterns".to_string(), "projector".to_string()]))
}

/// GET /v1/morphology/list/types
#[utoipa::path(get, path = "/v1/morphology/list/types", tag = "morphology")]
pub async fn get_list_types(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, Vec<String>>>> {
    // TODO: Get actual morphology categorization
    Ok(Json(HashMap::new()))
}

/// GET /v1/morphology/morphologies
#[utoipa::path(
    get, 
    path = "/v1/morphology/morphologies",
    tag = "morphology",
    responses(
        (status = 200, description = "All morphology definitions", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_morphologies(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let connectome_service = state.connectome_service.as_ref();
    
    // Get morphologies from connectome
    let morphologies = connectome_service.get_morphologies().await
        .map_err(|e| ApiError::internal(format!("Failed to get morphologies: {}", e)))?;
    
    // Convert to Python-compatible format
    let mut result = HashMap::new();
    for (name, morphology_info) in morphologies.iter() {
        result.insert(
            name.clone(),
            serde_json::json!({
                "name": name,
                "type": morphology_info.morphology_type,
                "class": morphology_info.class,
                "parameters": morphology_info.parameters,
                "source": "genome"
            })
        );
    }
    
    Ok(Json(result))
}

/// POST /v1/morphology/morphology
#[utoipa::path(post, path = "/v1/morphology/morphology", tag = "morphology")]
pub async fn post_morphology(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// PUT /v1/morphology/morphology
#[utoipa::path(put, path = "/v1/morphology/morphology", tag = "morphology")]
pub async fn put_morphology(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// DELETE /v1/morphology/morphology
#[utoipa::path(delete, path = "/v1/morphology/morphology", tag = "morphology")]
pub async fn delete_morphology(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, String>>) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// POST /v1/morphology/morphology_properties
#[utoipa::path(
    post, 
    path = "/v1/morphology/morphology_properties", 
    tag = "morphology",
    responses(
        (status = 200, description = "Morphology properties", body = HashMap<String, serde_json::Value>),
        (status = 404, description = "Morphology not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_morphology_properties(
    State(state): State<ApiState>, 
    Json(req): Json<HashMap<String, String>>
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    use tracing::debug;
    
    let morphology_name = req.get("morphology_name")
        .ok_or_else(|| ApiError::invalid_input("Missing morphology_name"))?;
    
    debug!(target: "feagi-api", "Getting properties for morphology: {}", morphology_name);
    
    let connectome_service = state.connectome_service.as_ref();
    let morphologies = connectome_service.get_morphologies().await
        .map_err(|e| ApiError::internal(format!("Failed to get morphologies: {}", e)))?;
    
    let morphology_info = morphologies.get(morphology_name)
        .ok_or_else(|| ApiError::not_found("Morphology", morphology_name))?;
    
    // Return properties in expected format
    let mut result = HashMap::new();
    result.insert("morphology_name".to_string(), serde_json::json!(morphology_name));
    result.insert("type".to_string(), serde_json::json!(morphology_info.morphology_type));
    result.insert("class".to_string(), serde_json::json!(morphology_info.class));
    result.insert("parameters".to_string(), morphology_info.parameters.clone());
    result.insert("source".to_string(), serde_json::json!("genome"));
    
    Ok(Json(result))
}

/// POST /v1/morphology/morphology_usage
#[utoipa::path(
    post, 
    path = "/v1/morphology/morphology_usage", 
    tag = "morphology",
    responses(
        (status = 200, description = "Morphology usage pairs", body = Vec<Vec<String>>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_morphology_usage(
    State(state): State<ApiState>, 
    Json(req): Json<HashMap<String, String>>
) -> ApiResult<Json<Vec<Vec<String>>>> {
    use tracing::debug;
    
    let morphology_name = req.get("morphology_name")
        .ok_or_else(|| ApiError::invalid_input("Missing morphology_name"))?;
    
    debug!(target: "feagi-api", "Getting usage for morphology: {}", morphology_name);
    
    let connectome_service = state.connectome_service.as_ref();
    
    // Get all cortical areas
    let areas = connectome_service.list_cortical_areas().await
        .map_err(|e| ApiError::internal(format!("Failed to list areas: {}", e)))?;
    
    // Find all [src, dst] pairs that use this morphology
    let mut usage_pairs = Vec::new();
    
    for area_info in areas {
        if let Some(mapping_dst) = area_info.properties.get("cortical_mapping_dst") {
            if let Some(dst_map) = mapping_dst.as_object() {
                for (dst_id, connections) in dst_map {
                    if let Some(conn_array) = connections.as_array() {
                        for conn in conn_array {
                            let morph_id = if let Some(arr) = conn.as_array() {
                                arr.get(0).and_then(|v| v.as_str())
                            } else if let Some(obj) = conn.as_object() {
                                obj.get("morphology_id").and_then(|v| v.as_str())
                            } else {
                                None
                            };
                            
                            if morph_id == Some(morphology_name.as_str()) {
                                usage_pairs.push(vec![area_info.cortical_id.clone(), dst_id.clone()]);
                            }
                        }
                    }
                }
            }
        }
    }
    
    debug!(target: "feagi-api", "Found {} usage pairs for morphology: {}", usage_pairs.len(), morphology_name);
    Ok(Json(usage_pairs))
}

/// GET /v1/morphology/list
/// Get list of all morphology names
#[utoipa::path(
    get,
    path = "/v1/morphology/list",
    tag = "morphology",
    responses(
        (status = 200, description = "List of morphology names", body = Vec<String>)
    )
)]
pub async fn get_list(State(state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    let connectome_service = state.connectome_service.as_ref();
    
    let morphologies = connectome_service.get_morphologies().await
        .map_err(|e| ApiError::internal(format!("Failed to get morphologies: {}", e)))?;
    
    let names: Vec<String> = morphologies.iter().map(|m| m.name.clone()).collect();
    Ok(Json(names))
}

/// GET /v1/morphology/info/{morphology_id}
/// Get detailed information about a specific morphology
#[utoipa::path(
    get,
    path = "/v1/morphology/info/{morphology_id}",
    tag = "morphology",
    params(
        ("morphology_id" = String, Path, description = "Morphology name")
    ),
    responses(
        (status = 200, description = "Morphology info", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn get_info(
    State(state): State<ApiState>,
    axum::extract::Path(morphology_id): axum::extract::Path<String>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // Delegate to post_morphology_properties (same logic)
    post_morphology_properties(State(state), Json(HashMap::from([
        ("morphology_name".to_string(), morphology_id)
    ]))).await
}

/// POST /v1/morphology/create
/// Create a new morphology
#[utoipa::path(
    post,
    path = "/v1/morphology/create",
    tag = "morphology",
    responses(
        (status = 200, description = "Morphology created", body = HashMap<String, String>)
    )
)]
pub async fn post_create(
    State(_state): State<ApiState>,
    Json(_request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Implement morphology creation
    Ok(Json(HashMap::from([
        ("message".to_string(), "Morphology creation not yet implemented".to_string())
    ])))
}

/// PUT /v1/morphology/update
/// Update an existing morphology
#[utoipa::path(
    put,
    path = "/v1/morphology/update",
    tag = "morphology",
    responses(
        (status = 200, description = "Morphology updated", body = HashMap<String, String>)
    )
)]
pub async fn put_update(
    State(_state): State<ApiState>,
    Json(_request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Implement morphology update
    Ok(Json(HashMap::from([
        ("message".to_string(), "Morphology update not yet implemented".to_string())
    ])))
}

/// DELETE /v1/morphology/delete/{morphology_id}
/// Delete a morphology
#[utoipa::path(
    delete,
    path = "/v1/morphology/delete/{morphology_id}",
    tag = "morphology",
    params(
        ("morphology_id" = String, Path, description = "Morphology name")
    ),
    responses(
        (status = 200, description = "Morphology deleted", body = HashMap<String, String>)
    )
)]
pub async fn delete_morphology(
    State(_state): State<ApiState>,
    axum::extract::Path(morphology_id): axum::extract::Path<String>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Implement morphology deletion
    tracing::info!(target: "feagi-api", "Delete morphology requested: {}", morphology_id);
    
    Ok(Json(HashMap::from([
        ("message".to_string(), format!("Morphology {} deletion not yet implemented", morphology_id))
    ])))
}



