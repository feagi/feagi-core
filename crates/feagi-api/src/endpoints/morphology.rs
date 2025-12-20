// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Morphology API Endpoints - Exact port from Python `/v1/morphology/*`

// Removed - using crate::common::State instead
use crate::common::ApiState;
use crate::common::{ApiError, ApiResult, Json, Path, State};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MorphologyListResponse {
    pub morphology_list: Vec<String>,
}

/// Get list of all morphology names in alphabetical order.
#[utoipa::path(get, path = "/v1/morphology/morphology_list", tag = "morphology")]
pub async fn get_morphology_list(
    State(state): State<ApiState>,
) -> ApiResult<Json<MorphologyListResponse>> {
    let connectome_service = state.connectome_service.as_ref();

    let morphologies = connectome_service
        .get_morphologies()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get morphologies: {}", e)))?;

    // Sort morphology names alphabetically for consistent UI display
    let mut names: Vec<String> = morphologies.keys().map(|name| name.clone()).collect();
    names.sort();

    Ok(Json(MorphologyListResponse {
        morphology_list: names,
    }))
}

/// Get available morphology types (vectors, patterns, projector).
#[utoipa::path(get, path = "/v1/morphology/morphology_types", tag = "morphology")]
pub async fn get_morphology_types(State(_state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    Ok(Json(vec![
        "vectors".to_string(),
        "patterns".to_string(),
        "projector".to_string(),
    ]))
}

/// Get morphologies categorized by type.
#[utoipa::path(get, path = "/v1/morphology/list/types", tag = "morphology")]
pub async fn get_list_types(
    State(_state): State<ApiState>,
) -> ApiResult<Json<BTreeMap<String, Vec<String>>>> {
    // TODO: Get actual morphology categorization
    // Use BTreeMap for alphabetical ordering in UI
    Ok(Json(BTreeMap::new()))
}

/// Get all morphology definitions with their complete configurations.
#[utoipa::path(
    get,
    path = "/v1/morphology/morphologies",
    tag = "morphology",
    responses(
        (status = 200, description = "All morphology definitions", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_morphologies(
    State(state): State<ApiState>,
) -> ApiResult<Json<BTreeMap<String, serde_json::Value>>> {
    let connectome_service = state.connectome_service.as_ref();

    // Get morphologies from connectome
    let morphologies = connectome_service
        .get_morphologies()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get morphologies: {}", e)))?;

    // Convert to Python-compatible format
    // Use BTreeMap for alphabetical ordering in UI
    let mut result = BTreeMap::new();
    for (name, morphology_info) in morphologies.iter() {
        result.insert(
            name.clone(),
            serde_json::json!({
                "name": name,
                "type": morphology_info.morphology_type,
                "class": morphology_info.class,
                "parameters": morphology_info.parameters,
                "source": "genome"
            }),
        );
    }

    Ok(Json(result))
}

/// Create a new morphology definition.
#[utoipa::path(post, path = "/v1/morphology/morphology", tag = "morphology")]
pub async fn post_morphology(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// Update an existing morphology definition.
#[utoipa::path(put, path = "/v1/morphology/morphology", tag = "morphology")]
pub async fn put_morphology(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// Delete a morphology by name provided in request body.
#[utoipa::path(delete, path = "/v1/morphology/morphology", tag = "morphology")]
pub async fn delete_morphology_by_name(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// Get detailed properties for a specific morphology by name.
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
    Json(req): Json<HashMap<String, String>>,
) -> ApiResult<Json<BTreeMap<String, serde_json::Value>>> {
    use tracing::debug;

    let morphology_name = req
        .get("morphology_name")
        .ok_or_else(|| ApiError::invalid_input("Missing morphology_name"))?;

    debug!(target: "feagi-api", "Getting properties for morphology: {}", morphology_name);

    let connectome_service = state.connectome_service.as_ref();
    let morphologies = connectome_service
        .get_morphologies()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get morphologies: {}", e)))?;

    let morphology_info = morphologies
        .get(morphology_name)
        .ok_or_else(|| ApiError::not_found("Morphology", morphology_name))?;

    // Return properties in expected format
    // Use BTreeMap for alphabetical ordering in UI
    let mut result = BTreeMap::new();
    result.insert(
        "morphology_name".to_string(),
        serde_json::json!(morphology_name),
    );
    result.insert(
        "type".to_string(),
        serde_json::json!(morphology_info.morphology_type),
    );
    result.insert(
        "class".to_string(),
        serde_json::json!(morphology_info.class),
    );
    result.insert("parameters".to_string(), morphology_info.parameters.clone());
    result.insert("source".to_string(), serde_json::json!("genome"));

    Ok(Json(result))
}

/// Get all cortical area pairs that use a specific morphology.
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
    Json(req): Json<HashMap<String, String>>,
) -> ApiResult<Json<Vec<Vec<String>>>> {
    use tracing::debug;

    let morphology_name = req
        .get("morphology_name")
        .ok_or_else(|| ApiError::invalid_input("Missing morphology_name"))?;

    debug!(target: "feagi-api", "Getting usage for morphology: {}", morphology_name);

    let connectome_service = state.connectome_service.as_ref();

    // Get all cortical areas
    let areas = connectome_service
        .list_cortical_areas()
        .await
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
                                arr.first().and_then(|v| v.as_str())
                            } else if let Some(obj) = conn.as_object() {
                                obj.get("morphology_id").and_then(|v| v.as_str())
                            } else {
                                None
                            };

                            if morph_id == Some(morphology_name.as_str()) {
                                usage_pairs
                                    .push(vec![area_info.cortical_id.clone(), dst_id.clone()]);
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

/// Get list of all morphology names.
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

    let morphologies = connectome_service
        .get_morphologies()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get morphologies: {}", e)))?;

    // Sort morphology names alphabetically for consistent UI display
      let mut names: Vec<String> = morphologies
        .keys()
        .map(|name| name.clone())
        .collect();
    names.sort();
    Ok(Json(names))
}

/// Get detailed information about a specific morphology using path parameter.
#[utoipa::path(
    get,
    path = "/v1/morphology/info/{morphology_id}",
    tag = "morphology",
    params(
        ("morphology_id" = String, Path, description = "Morphology name")
    ),
    responses(
        (status = 200, description = "Morphology info", body = BTreeMap<String, serde_json::Value>)
    )
)]
pub async fn get_info(
    State(state): State<ApiState>,
    Path(morphology_id): Path<String>,
) -> ApiResult<Json<BTreeMap<String, serde_json::Value>>> {
    // Delegate to post_morphology_properties (same logic)
    post_morphology_properties(
        State(state),
        Json(HashMap::from([(
            "morphology_name".to_string(),
            morphology_id,
        )])),
    )
    .await
}

/// Create a new morphology with specified parameters.
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
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Morphology creation not yet implemented".to_string(),
    )])))
}

/// Update an existing morphology's parameters.
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
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Morphology update not yet implemented".to_string(),
    )])))
}

/// Delete a morphology using path parameter.
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
    Path(morphology_id): Path<String>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Implement morphology deletion
    tracing::info!(target: "feagi-api", "Delete morphology requested: {}", morphology_id);

    Ok(Json(HashMap::from([(
        "message".to_string(),
        format!("Morphology {} deletion not yet implemented", morphology_id),
    )])))
}
