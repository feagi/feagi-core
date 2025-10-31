// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Connectome API Endpoints - Exact port from Python `/v1/connectome/*`

use axum::{extract::State, response::Json};
use std::collections::HashMap;
use crate::common::{ApiError, ApiResult};
use crate::transports::http::server::ApiState;

/// GET /v1/connectome/cortical_areas/list/detailed
#[utoipa::path(get, path = "/v1/connectome/cortical_areas/list/detailed", tag = "connectome")]
pub async fn get_cortical_areas_list_detailed(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_cortical_areas().await {
        Ok(areas) => {
            let detailed: HashMap<String, serde_json::Value> = areas.into_iter()
                .map(|area| (area.cortical_id.clone(), serde_json::to_value(area).unwrap_or_default()))
                .collect();
            Ok(Json(detailed))
        },
        Err(e) => Err(ApiError::internal(format!("Failed to get detailed list: {}", e))),
    }
}

/// GET /v1/connectome/properties/dimensions
#[utoipa::path(get, path = "/v1/connectome/properties/dimensions", tag = "connectome")]
pub async fn get_properties_dimensions(State(_state): State<ApiState>) -> ApiResult<Json<(usize, usize, usize)>> {  // Will use state when wired to NPU
    // TODO: Get max dimensions from connectome manager
    Ok(Json((0, 0, 0)))
}

/// GET /v1/connectome/properties/mappings
#[utoipa::path(get, path = "/v1/connectome/properties/mappings", tag = "connectome")]
pub async fn get_properties_mappings(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, Vec<String>>>> {
    // TODO: Get all cortical mappings
    Ok(Json(HashMap::new()))
}



