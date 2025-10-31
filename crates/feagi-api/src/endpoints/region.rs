// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Region API Endpoints - Exact port from Python `/v1/region/*`

use axum::{extract::State, response::Json};
use std::collections::HashMap;
use crate::common::{ApiError, ApiResult};
use crate::transports::http::server::ApiState;

/// GET /v1/region/regions_members
#[utoipa::path(get, path = "/v1/region/regions_members", tag = "region")]
pub async fn get_regions_members(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, Vec<String>>>> {
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_brain_regions().await {
        Ok(regions) => {
            let mut result = HashMap::new();
            for region in regions {
                result.insert(region.region_id, region.cortical_areas);
            }
            Ok(Json(result))
        },
        Err(e) => Err(ApiError::internal(format!("Failed to get regions: {}", e))),
    }
}

/// POST /v1/region/region
#[utoipa::path(post, path = "/v1/region/region", tag = "region")]
pub async fn post_region(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// PUT /v1/region/region
#[utoipa::path(put, path = "/v1/region/region", tag = "region")]
pub async fn put_region(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// DELETE /v1/region/region
#[utoipa::path(delete, path = "/v1/region/region", tag = "region")]
pub async fn delete_region(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, String>>) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// POST /v1/region/clone
#[utoipa::path(post, path = "/v1/region/clone", tag = "region")]
pub async fn post_clone(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// PUT /v1/region/relocate_members
#[utoipa::path(put, path = "/v1/region/relocate_members", tag = "region")]
pub async fn put_relocate_members(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// DELETE /v1/region/region_and_members
#[utoipa::path(delete, path = "/v1/region/region_and_members", tag = "region")]
pub async fn delete_region_and_members(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, String>>) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

