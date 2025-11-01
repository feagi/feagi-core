// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Input API Endpoints - Exact port from Python `/v1/input/*`

use axum::{extract::State, response::Json};
use std::collections::HashMap;
use crate::common::{ApiError, ApiResult};
use crate::transports::http::server::ApiState;

/// GET /v1/input/vision
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

/// POST /v1/input/vision
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

/// GET /v1/input/sources
#[utoipa::path(get, path = "/v1/input/sources", tag = "input")]
pub async fn get_sources(State(_state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    Ok(Json(vec!["vision".to_string()]))
}

/// POST /v1/input/configure
#[utoipa::path(post, path = "/v1/input/configure", tag = "input")]
pub async fn post_configure(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([("message".to_string(), "Input configured".to_string())])))
}



