// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Input API Endpoints - Exact port from Python `/v1/input/*`

// Removed - using crate::common::State instead
use std::collections::HashMap;
use crate::common::{ApiError, ApiResult, State, Json, Path, Query};
use crate::common::ApiState;

/// Get vision input configuration and settings.
#[utoipa::path(
    get, 
    path = "/v1/input/vision",
    tag = "input",
    responses(
        (status = 200, description = "Vision input configuration", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_vision(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // TODO: Get vision input configuration
    Ok(Json(HashMap::new()))
}

/// Update vision input configuration.
#[utoipa::path(
    post, 
    path = "/v1/input/vision",
    tag = "input",
    responses(
        (status = 200, description = "Vision input updated", content_type = "application/json"),
        (status = 500, description = "Not yet implemented")
    )
)]
pub async fn post_vision(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// Get list of available input sources (vision, audio, etc.).
#[utoipa::path(get, path = "/v1/input/sources", tag = "input")]
pub async fn get_sources(State(_state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    Ok(Json(vec!["vision".to_string()]))
}

/// Configure input sources and their parameters.
#[utoipa::path(post, path = "/v1/input/configure", tag = "input")]
pub async fn post_configure(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([("message".to_string(), "Input configured".to_string())])))
}



