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
pub async fn get_morphologies(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // TODO: Get all morphologies
    Ok(Json(HashMap::new()))
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
#[utoipa::path(post, path = "/v1/morphology/morphology_properties", tag = "morphology")]
pub async fn post_morphology_properties(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, String>>) -> ApiResult<Json<serde_json::Value>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// POST /v1/morphology/morphology_usage
#[utoipa::path(post, path = "/v1/morphology/morphology_usage", tag = "morphology")]
pub async fn post_morphology_usage(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, String>>) -> ApiResult<Json<serde_json::Value>> {
    Err(ApiError::internal("Not yet implemented"))
}



