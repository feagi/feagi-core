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
#[utoipa::path(post, path = "/v1/cortical_mapping/mapping_properties", tag = "cortical_mapping")]
pub async fn post_mapping_properties(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<serde_json::Value>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// PUT /v1/cortical_mapping/mapping_properties
#[utoipa::path(put, path = "/v1/cortical_mapping/mapping_properties", tag = "cortical_mapping")]
pub async fn put_mapping_properties(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}


