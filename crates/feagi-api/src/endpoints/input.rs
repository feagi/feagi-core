// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Input API Endpoints - Exact port from Python `/v1/input/*`

use axum::{extract::State, response::Json};
use std::collections::HashMap;
use crate::common::{ApiError, ApiResult};
use crate::transports::http::server::ApiState;

/// GET /v1/input/vision
#[utoipa::path(get, path = "/v1/input/vision", tag = "input")]
pub async fn get_vision(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // TODO: Get vision input configuration
    Ok(Json(HashMap::new()))
}

/// POST /v1/input/vision
#[utoipa::path(post, path = "/v1/input/vision", tag = "input")]
pub async fn post_vision(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}



