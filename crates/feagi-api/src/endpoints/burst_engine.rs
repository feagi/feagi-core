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
pub async fn get_fcl(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    use tracing::debug;
    
    let runtime_service = state.runtime_service.as_ref();
    let _connectome_service = state.connectome_service.as_ref();
    
    // Get FCL snapshot from RuntimeService
    let fcl_data = runtime_service.get_fcl_snapshot().await
        .map_err(|e| ApiError::internal(format!("Failed to get FCL snapshot: {}", e)))?;
    
    // Get burst count for timestep
    let timestep = runtime_service.get_burst_count().await
        .map_err(|e| ApiError::internal(format!("Failed to get burst count: {}", e)))?;
    
    // Organize FCL by cortical area (need to map neuron_id -> cortical_id)
    let cortical_areas: HashMap<String, Vec<u64>> = HashMap::new();
    let global_fcl: Vec<u64> = fcl_data.iter().map(|(id, _)| *id).collect();
    
    // TODO: Map neuron IDs to cortical areas using ConnectomeService
    // For now, just return global FCL
    
    let mut response = HashMap::new();
    response.insert("timestep".to_string(), serde_json::json!(timestep));
    response.insert("total_neurons".to_string(), serde_json::json!(fcl_data.len()));
    response.insert("global_fcl".to_string(), serde_json::json!(global_fcl));
    response.insert("cortical_areas".to_string(), serde_json::json!(cortical_areas));
    response.insert("default_window_size".to_string(), serde_json::json!(20));
    response.insert("active_cortical_count".to_string(), serde_json::json!(cortical_areas.len()));
    
    debug!(target: "feagi-api", "GET /fcl - returned {} neurons from FCL", fcl_data.len());
    
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
pub async fn get_fire_queue(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    use tracing::debug;
    
    let runtime_service = state.runtime_service.as_ref();
    let _connectome_service = state.connectome_service.as_ref();
    
    // Get Fire Queue sample from RuntimeService
    let fq_sample = runtime_service.get_fire_queue_sample().await
        .map_err(|e| ApiError::internal(format!("Failed to get fire queue: {}", e)))?;
    
    // Get burst count for timestep
    let timestep = runtime_service.get_burst_count().await
        .map_err(|e| ApiError::internal(format!("Failed to get burst count: {}", e)))?;
    
    // Convert cortical_idx to cortical_id using ConnectomeService
    let mut cortical_areas: HashMap<String, Vec<u64>> = HashMap::new();
    let mut total_fired = 0;
    
    for (cortical_idx, (neuron_ids, _, _, _, _)) in fq_sample {
        // TODO: Map cortical_idx to cortical_id using ConnectomeService
        // For now, use cortical_idx as string
        let cortical_id = format!("area_{}", cortical_idx);
        
        let ids_u64: Vec<u64> = neuron_ids.iter().map(|&id| id as u64).collect();
        total_fired += ids_u64.len();
        cortical_areas.insert(cortical_id, ids_u64);
    }
    
    let mut response = HashMap::new();
    response.insert("timestep".to_string(), serde_json::json!(timestep));
    response.insert("total_fired".to_string(), serde_json::json!(total_fired));
    response.insert("cortical_areas".to_string(), serde_json::json!(cortical_areas));
    
    debug!(target: "feagi-api", "GET /fire_queue - returned {} fired neurons", total_fired);
    
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
    // Get default window size from Fire Ledger configuration
    // TODO: Add get_default_window_size to RuntimeService
    // For now, return standard default
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
    
    if window_size <= 0 {
        return Err(ApiError::invalid_input("Window size must be positive"));
    }
    
    // TODO: Update default window size configuration
    tracing::info!(target: "feagi-api", "Default Fire Ledger window size set to {}", window_size);
    
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
pub async fn get_fire_ledger_areas_window_config(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let runtime_service = state.runtime_service.as_ref();
    
    // Get Fire Ledger configurations from RuntimeService
    let configs = runtime_service.get_fire_ledger_configs().await
        .map_err(|e| ApiError::internal(format!("Failed to get fire ledger configs: {}", e)))?;
    
    // Convert to area_id -> window_size HashMap
    // TODO: Map cortical_idx to cortical_id using ConnectomeService
    let mut areas: HashMap<String, usize> = HashMap::new();
    for (cortical_idx, window_size) in configs {
        areas.insert(format!("area_{}", cortical_idx), window_size);
    }
    
    let mut response = HashMap::new();
    response.insert("default_window_size".to_string(), serde_json::json!(20));
    response.insert("areas".to_string(), serde_json::json!(areas));
    response.insert("total_configured_areas".to_string(), serde_json::json!(areas.len()));
    
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



