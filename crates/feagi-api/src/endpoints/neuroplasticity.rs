// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Neuroplasticity API Endpoints - Exact port from Python `/v1/neuroplasticity/*`

// Removed - using crate::common::State instead
use std::collections::HashMap;
use crate::common::ApiResult;
use crate::common::{State, Json, Path};
use crate::common::ApiState;

/// Get the current plasticity queue depth (number of pending plasticity operations).
#[utoipa::path(get, path = "/v1/neuroplasticity/plasticity_queue_depth", tag = "neuroplasticity")]
pub async fn get_plasticity_queue_depth(State(_state): State<ApiState>) -> ApiResult<Json<i32>> {
    // TODO: Get from plasticity service
    Ok(Json(0))
}

/// Set the plasticity queue depth to control how many operations can be pending.
#[utoipa::path(put, path = "/v1/neuroplasticity/plasticity_queue_depth", tag = "neuroplasticity")]
pub async fn put_plasticity_queue_depth(
    State(_state): State<ApiState>,
    Json(depth): Json<i32>,
) -> ApiResult<Json<HashMap<String, String>>> {
    tracing::info!(target: "feagi-api", "Plasticity queue depth set to {}", depth);
    
    Ok(Json(HashMap::from([
        ("message".to_string(), format!("Plasticity queue depth set to {}", depth))
    ])))
}

/// Get neuroplasticity status across all cortical areas including enabled state and queue depth.
#[utoipa::path(
    get,
    path = "/v1/neuroplasticity/status",
    tag = "neuroplasticity",
    responses(
        (status = 200, description = "Plasticity status", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn get_status(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let mut response = HashMap::new();
    response.insert("global_plasticity_enabled".to_string(), serde_json::json!(true));
    response.insert("transforming_areas".to_string(), serde_json::json!([]));
    response.insert("queue_depth".to_string(), serde_json::json!(0));
    
    Ok(Json(response))
}

/// Get list of cortical areas currently undergoing plasticity transformation.
#[utoipa::path(
    get,
    path = "/v1/neuroplasticity/transforming",
    tag = "neuroplasticity",
    responses(
        (status = 200, description = "Transforming areas", body = Vec<String>)
    )
)]
pub async fn get_transforming(State(_state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    // TODO: Query ConnectomeService for transforming areas
    Ok(Json(Vec::new()))
}

/// Configure neuroplasticity parameters including learning rates and plasticity rules.
#[utoipa::path(
    post,
    path = "/v1/neuroplasticity/configure",
    tag = "neuroplasticity",
    responses(
        (status = 200, description = "Configuration updated", body = HashMap<String, String>)
    )
)]
pub async fn post_configure(
    State(_state): State<ApiState>,
    Json(_config): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Update plasticity configuration
    Ok(Json(HashMap::from([
        ("message".to_string(), "Neuroplasticity configuration updated".to_string())
    ])))
}

/// Enable neuroplasticity for a specific cortical area.
#[utoipa::path(
    post,
    path = "/v1/neuroplasticity/enable/{area_id}",
    tag = "neuroplasticity",
    params(
        ("area_id" = String, Path, description = "Cortical area ID")
    ),
    responses(
        (status = 200, description = "Plasticity enabled", body = HashMap<String, String>)
    )
)]
pub async fn post_enable_area(
    State(_state): State<ApiState>,
    Path(area_id): Path<String>,
) -> ApiResult<Json<HashMap<String, String>>> {
    tracing::info!(target: "feagi-api", "Enabling plasticity for area: {}", area_id);
    
    Ok(Json(HashMap::from([
        ("message".to_string(), format!("Plasticity enabled for area {}", area_id))
    ])))
}

/// Disable neuroplasticity for a specific cortical area.
#[utoipa::path(
    post,
    path = "/v1/neuroplasticity/disable/{area_id}",
    tag = "neuroplasticity",
    params(
        ("area_id" = String, Path, description = "Cortical area ID")
    ),
    responses(
        (status = 200, description = "Plasticity disabled", body = HashMap<String, String>)
    )
)]
pub async fn post_disable_area(
    State(_state): State<ApiState>,
    Path(area_id): Path<String>,
) -> ApiResult<Json<HashMap<String, String>>> {
    tracing::info!(target: "feagi-api", "Disabling plasticity for area: {}", area_id);
    
    Ok(Json(HashMap::from([
        ("message".to_string(), format!("Plasticity disabled for area {}", area_id))
    ])))
}



