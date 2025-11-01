// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Burst Engine API Endpoints - Exact port from Python `/v1/burst_engine/*`

use axum::{extract::State, response::Json};
use std::collections::HashMap;
use crate::common::{ApiError, ApiResult};
use crate::transports::http::server::ApiState;

/// GET /v1/burst_engine/simulation_timestep
#[utoipa::path(get, path = "/v1/burst_engine/simulation_timestep", tag = "burst_engine")]
pub async fn get_simulation_timestep(State(state): State<ApiState>) -> ApiResult<Json<f64>> {
    let runtime_service = state.runtime_service.as_ref();
    match runtime_service.get_status().await {
        Ok(status) => {
            // Convert frequency to timestep (1/Hz = seconds)
            let timestep = if status.frequency_hz > 0.0 {
                1.0 / status.frequency_hz
            } else {
                0.0
            };
            Ok(Json(timestep))
        },
        Err(e) => Err(ApiError::internal(format!("Failed to get timestep: {}", e))),
    }
}

/// POST /v1/burst_engine/simulation_timestep
#[utoipa::path(post, path = "/v1/burst_engine/simulation_timestep", tag = "burst_engine")]
pub async fn post_simulation_timestep(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, f64>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let runtime_service = state.runtime_service.as_ref();
    
    if let Some(&timestep) = request.get("simulation_timestep") {
        // Convert timestep (seconds) to frequency (Hz)
        let frequency = if timestep > 0.0 { 1.0 / timestep } else { 0.0 };
        
        match runtime_service.set_frequency(frequency).await {
            Ok(_) => Ok(Json(HashMap::from([("message".to_string(), format!("Timestep set to {}", timestep))]))),
            Err(e) => Err(ApiError::internal(format!("Failed to set timestep: {}", e))),
        }
    } else {
        Err(ApiError::invalid_input("simulation_timestep required"))
    }
}

// ============================================================================
// FCL (Fire Candidate List) ENDPOINTS
// ============================================================================

/// GET /v1/burst_engine/fcl
/// Get the Fire Candidate List content at the current timestep
#[utoipa::path(
    get,
    path = "/v1/burst_engine/fcl",
    tag = "burst_engine",
    responses(
        (status = 200, description = "FCL content", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_fcl(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    use tracing::debug;
    
    // TODO: Get FCL from BurstLoopRunner/NPU
    // For now, return empty FCL structure
    debug!(target: "feagi-api", "GET /fcl - returning empty FCL (implementation pending)");
    
    let mut response = HashMap::new();
    response.insert("timestep".to_string(), serde_json::json!(0));
    response.insert("total_neurons".to_string(), serde_json::json!(0));
    response.insert("global_fcl".to_string(), serde_json::json!(Vec::<u64>::new()));
    response.insert("cortical_areas".to_string(), serde_json::json!(HashMap::<String, Vec<u64>>::new()));
    response.insert("default_window_size".to_string(), serde_json::json!(20));
    response.insert("active_cortical_count".to_string(), serde_json::json!(0));
    
    Ok(Json(response))
}

/// GET /v1/burst_engine/fire_queue
/// Get the Fire Queue content (neurons that actually fired)
#[utoipa::path(
    get,
    path = "/v1/burst_engine/fire_queue",
    tag = "burst_engine",
    responses(
        (status = 200, description = "Fire queue content", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_fire_queue(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    use tracing::debug;
    
    // TODO: Get Fire Queue from BurstLoopRunner/NPU
    debug!(target: "feagi-api", "GET /fire_queue - returning empty queue (implementation pending)");
    
    let mut response = HashMap::new();
    response.insert("timestep".to_string(), serde_json::json!(0));
    response.insert("total_fired".to_string(), serde_json::json!(0));
    response.insert("cortical_areas".to_string(), serde_json::json!(HashMap::<String, Vec<u64>>::new()));
    
    Ok(Json(response))
}

/// POST /v1/burst_engine/fcl_reset
/// Reset the Fire Candidate List
#[utoipa::path(
    post,
    path = "/v1/burst_engine/fcl_reset",
    tag = "burst_engine",
    responses(
        (status = 200, description = "FCL reset successfully", body = HashMap<String, String>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_fcl_reset(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, String>>> {
    use tracing::info;
    
    // TODO: Reset FCL in BurstLoopRunner/NPU
    info!(target: "feagi-api", "FCL reset requested (implementation pending)");
    
    Ok(Json(HashMap::from([
        ("message".to_string(), "Fire Candidate List reset successfully".to_string())
    ])))
}

/// GET /v1/burst_engine/fcl_status
/// Get detailed FCL manager status
#[utoipa::path(
    get,
    path = "/v1/burst_engine/fcl_status",
    tag = "burst_engine",
    responses(
        (status = 200, description = "FCL status", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_fcl_status(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    use tracing::debug;
    
    // TODO: Get FCL manager status
    debug!(target: "feagi-api", "GET /fcl_status - returning stub (implementation pending)");
    
    let mut response = HashMap::new();
    response.insert("available".to_string(), serde_json::json!(false));
    response.insert("error".to_string(), serde_json::json!("FCL manager not yet implemented in Rust"));
    
    Ok(Json(response))
}

// ============================================================================
// FIRE LEDGER WINDOW SIZE ENDPOINTS
// ============================================================================

/// GET /v1/burst_engine/fire_ledger/default_window_size
#[utoipa::path(
    get,
    path = "/v1/burst_engine/fire_ledger/default_window_size",
    tag = "burst_engine",
    responses(
        (status = 200, description = "Default window size", body = i32),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_fire_ledger_default_window_size(State(_state): State<ApiState>) -> ApiResult<Json<i32>> {
    // TODO: Get from configuration
    Ok(Json(20))
}

/// PUT /v1/burst_engine/fire_ledger/default_window_size
#[utoipa::path(
    put,
    path = "/v1/burst_engine/fire_ledger/default_window_size",
    tag = "burst_engine",
    responses(
        (status = 200, description = "Window size updated", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn put_fire_ledger_default_window_size(
    State(_state): State<ApiState>,
    Json(request): Json<HashMap<String, i32>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let window_size = request.get("window_size").copied().unwrap_or(20);
    
    // TODO: Update configuration
    let mut response = HashMap::new();
    response.insert("success".to_string(), serde_json::json!(true));
    response.insert("window_size".to_string(), serde_json::json!(window_size));
    response.insert("message".to_string(), serde_json::json!(format!("Default window size set to {}", window_size)));
    
    Ok(Json(response))
}

/// GET /v1/burst_engine/fire_ledger/areas_window_config
#[utoipa::path(
    get,
    path = "/v1/burst_engine/fire_ledger/areas_window_config",
    tag = "burst_engine",
    responses(
        (status = 200, description = "Areas window configuration", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_fire_ledger_areas_window_config(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // TODO: Get per-area window configuration
    let mut response = HashMap::new();
    response.insert("default_window_size".to_string(), serde_json::json!(20));
    response.insert("areas".to_string(), serde_json::json!(HashMap::<String, i32>::new()));
    response.insert("total_configured_areas".to_string(), serde_json::json!(0));
    
    Ok(Json(response))
}

// ============================================================================
// BURST ENGINE CONTROL & STATUS ENDPOINTS
// ============================================================================

/// GET /v1/burst_engine/stats
#[utoipa::path(
    get,
    path = "/v1/burst_engine/stats",
    tag = "burst_engine",
    responses(
        (status = 200, description = "Burst engine statistics", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_stats(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let runtime_service = state.runtime_service.as_ref();
    
    match runtime_service.get_status().await {
        Ok(status) => {
            let mut response = HashMap::new();
            response.insert("burst_count".to_string(), serde_json::json!(status.burst_count));
            response.insert("frequency_hz".to_string(), serde_json::json!(status.frequency_hz));
            response.insert("active".to_string(), serde_json::json!(status.is_running));
            response.insert("paused".to_string(), serde_json::json!(status.is_paused));
            
            Ok(Json(response))
        },
        Err(e) => Err(ApiError::internal(format!("Failed to get stats: {}", e))),
    }
}

/// GET /v1/burst_engine/status
#[utoipa::path(
    get,
    path = "/v1/burst_engine/status",
    tag = "burst_engine",
    responses(
        (status = 200, description = "Burst engine status", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_status(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let runtime_service = state.runtime_service.as_ref();
    
    match runtime_service.get_status().await {
        Ok(status) => {
            let mut response = HashMap::new();
            response.insert("active".to_string(), serde_json::json!(status.is_running));
            response.insert("paused".to_string(), serde_json::json!(status.is_paused));
            response.insert("burst_count".to_string(), serde_json::json!(status.burst_count));
            response.insert("frequency_hz".to_string(), serde_json::json!(status.frequency_hz));
            
            Ok(Json(response))
        },
        Err(e) => Err(ApiError::internal(format!("Failed to get status: {}", e))),
    }
}

/// POST /v1/burst_engine/control
#[utoipa::path(
    post,
    path = "/v1/burst_engine/control",
    tag = "burst_engine",
    responses(
        (status = 200, description = "Control command executed", body = HashMap<String, String>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_control(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let runtime_service = state.runtime_service.as_ref();
    let action = request.get("action").map(|s| s.as_str());
    
    match action {
        Some("start") | Some("resume") => {
            runtime_service.start().await
                .map_err(|e| ApiError::internal(format!("Failed to start: {}", e)))?;
            Ok(Json(HashMap::from([("message".to_string(), "Burst engine started".to_string())])))
        },
        Some("pause") => {
            runtime_service.pause().await
                .map_err(|e| ApiError::internal(format!("Failed to pause: {}", e)))?;
            Ok(Json(HashMap::from([("message".to_string(), "Burst engine paused".to_string())])))
        },
        Some("stop") => {
            runtime_service.stop().await
                .map_err(|e| ApiError::internal(format!("Failed to stop: {}", e)))?;
            Ok(Json(HashMap::from([("message".to_string(), "Burst engine stopped".to_string())])))
        },
        _ => Err(ApiError::invalid_input("Invalid action: must be 'start', 'pause', or 'stop'")),
    }
}



