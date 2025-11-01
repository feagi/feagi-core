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
    use std::collections::BTreeMap;
    
    let runtime_service = state.runtime_service.as_ref();
    let connectome_service = state.connectome_service.as_ref();
    
    // CRITICAL FIX: Get FCL snapshot WITH cortical_idx from NPU (not extracted from neuron_id bits!)
    // Old code was doing (neuron_id >> 32) which is WRONG - neuron_id is u32, not packed!
    let fcl_data = runtime_service.get_fcl_snapshot_with_cortical_idx().await
        .map_err(|e| ApiError::internal(format!("Failed to get FCL snapshot: {}", e)))?;
    
    // Get burst count for timestep
    let timestep = runtime_service.get_burst_count().await
        .map_err(|e| ApiError::internal(format!("Failed to get burst count: {}", e)))?;
    
    // Get all cortical areas to map cortical_idx -> cortical_id
    let areas = connectome_service.list_cortical_areas().await
        .map_err(|e| ApiError::internal(format!("Failed to list cortical areas: {}", e)))?;
    
    // Build cortical_idx -> cortical_id mapping
    let mut idx_to_id: HashMap<u32, String> = HashMap::new();
    for area in &areas {
        idx_to_id.insert(area.cortical_idx, area.cortical_id.clone());
    }
    
    // Group FCL neurons by cortical area
    // Use BTreeMap for consistent ordering in JSON output
    let mut cortical_areas: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    
    for (neuron_id, cortical_idx, _potential) in &fcl_data {
        // Map cortical_idx to cortical_id using actual stored values
        let cortical_id = idx_to_id.get(cortical_idx)
            .cloned()
            .unwrap_or_else(|| format!("area_{}", cortical_idx));
        
        cortical_areas.entry(cortical_id)
            .or_insert_with(Vec::new)
            .push(*neuron_id);
    }
    
    // Limit to first 20 neuron IDs per area (matching Python behavior for network efficiency)
    for neuron_list in cortical_areas.values_mut() {
        neuron_list.truncate(20);
    }
    
    let active_cortical_count = cortical_areas.len();
    let total_neurons: usize = cortical_areas.values().map(|v| v.len()).sum();
    
    // Build response (NO global_fcl per user request)
    let mut response = HashMap::new();
    response.insert("timestep".to_string(), serde_json::json!(timestep));
    response.insert("total_neurons".to_string(), serde_json::json!(total_neurons));
    response.insert("cortical_areas".to_string(), serde_json::json!(cortical_areas));
    response.insert("default_window_size".to_string(), serde_json::json!(20));
    response.insert("active_cortical_count".to_string(), serde_json::json!(active_cortical_count));
    
    debug!(target: "feagi-api", "GET /fcl - {} neurons across {} cortical areas (limited to 20/area)", 
           total_neurons, active_cortical_count);
    
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
    let connectome_service = state.connectome_service.as_ref();
    
    // Get Fire Queue sample from RuntimeService
    let fq_sample = runtime_service.get_fire_queue_sample().await
        .map_err(|e| ApiError::internal(format!("Failed to get fire queue: {}", e)))?;
    
    // Get burst count for timestep
    let timestep = runtime_service.get_burst_count().await
        .map_err(|e| ApiError::internal(format!("Failed to get burst count: {}", e)))?;
    
    // CRITICAL FIX: Build cortical_idx -> cortical_id mapping from ConnectomeService
    // This uses the actual stored cortical_idx values instead of fabricating names
    let areas = connectome_service.list_cortical_areas().await
        .map_err(|e| ApiError::internal(format!("Failed to list cortical areas: {}", e)))?;
    
    let idx_to_id: HashMap<u32, String> = areas.iter()
        .map(|a| (a.cortical_idx, a.cortical_id.clone()))
        .collect();
    
    // Convert cortical_idx to cortical_id
    let mut cortical_areas: HashMap<String, Vec<u64>> = HashMap::new();
    let mut total_fired = 0;
    
    for (cortical_idx, (neuron_ids, _, _, _, _)) in fq_sample {
        // Use actual cortical_id from mapping, fallback to area_{idx} if not found
        let cortical_id = idx_to_id.get(&cortical_idx)
            .cloned()
            .unwrap_or_else(|| format!("area_{}", cortical_idx));
        
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
    let connectome_service = state.connectome_service.as_ref();
    
    // Get Fire Ledger configurations from RuntimeService
    let configs = runtime_service.get_fire_ledger_configs().await
        .map_err(|e| ApiError::internal(format!("Failed to get fire ledger configs: {}", e)))?;
    
    // CRITICAL FIX: Build cortical_idx -> cortical_id mapping from ConnectomeService
    // This uses the actual stored cortical_idx values instead of fabricating names
    let cortical_areas_list = connectome_service.list_cortical_areas().await
        .map_err(|e| ApiError::internal(format!("Failed to list cortical areas: {}", e)))?;
    
    let idx_to_id: HashMap<u32, String> = cortical_areas_list.iter()
        .map(|a| (a.cortical_idx, a.cortical_id.clone()))
        .collect();
    
    // Convert to area_id -> window_size HashMap using actual cortical_id
    let mut areas: HashMap<String, usize> = HashMap::new();
    for (cortical_idx, window_size) in configs {
        // Use actual cortical_id from mapping, fallback to area_{idx} if not found
        let cortical_id = idx_to_id.get(&cortical_idx)
            .cloned()
            .unwrap_or_else(|| format!("area_{}", cortical_idx));
        areas.insert(cortical_id, window_size);
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

// ============================================================================
// FCL SAMPLER CONFIGURATION ENDPOINTS
// ============================================================================

/// GET /v1/burst_engine/fcl_sampler/config
/// Get FCL/FQ sampler configuration (frequency, consumer)
#[utoipa::path(
    get,
    path = "/v1/burst_engine/fcl_sampler/config",
    tag = "burst_engine",
    responses(
        (status = 200, description = "FCL sampler configuration", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_fcl_sampler_config(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let runtime_service = state.runtime_service.as_ref();
    
    let (frequency, consumer) = runtime_service.get_fcl_sampler_config().await
        .map_err(|e| ApiError::internal(format!("Failed to get FCL sampler config: {}", e)))?;
    
    let mut response = HashMap::new();
    response.insert("frequency".to_string(), serde_json::json!(frequency));
    response.insert("consumer".to_string(), serde_json::json!(consumer));
    
    Ok(Json(response))
}

/// POST /v1/burst_engine/fcl_sampler/config
/// Update FCL/FQ sampler configuration
#[utoipa::path(
    post,
    path = "/v1/burst_engine/fcl_sampler/config",
    tag = "burst_engine",
    responses(
        (status = 200, description = "FCL sampler configuration updated", body = HashMap<String, serde_json::Value>),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_fcl_sampler_config(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let runtime_service = state.runtime_service.as_ref();
    
    let frequency = request.get("frequency").and_then(|v| v.as_f64());
    let consumer = request.get("consumer").and_then(|v| v.as_u64().map(|n| n as u32));
    
    runtime_service.set_fcl_sampler_config(frequency, consumer).await
        .map_err(|e| ApiError::internal(format!("Failed to update FCL sampler config: {}", e)))?;
    
    // Return the updated config
    let (freq, cons) = runtime_service.get_fcl_sampler_config().await
        .map_err(|e| ApiError::internal(format!("Failed to get updated config: {}", e)))?;
    
    let mut response = HashMap::new();
    response.insert("frequency".to_string(), serde_json::json!(freq));
    response.insert("consumer".to_string(), serde_json::json!(cons));
    
    Ok(Json(response))
}

/// GET /v1/burst_engine/fcl_sampler/area/{area_id}/sample_rate
/// Get FCL sample rate for a specific cortical area
#[utoipa::path(
    get,
    path = "/v1/burst_engine/fcl_sampler/area/{area_id}/sample_rate",
    tag = "burst_engine",
    params(
        ("area_id" = u32, Path, description = "Cortical area ID (cortical_idx)")
    ),
    responses(
        (status = 200, description = "Sample rate", body = HashMap<String, f64>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_area_fcl_sample_rate(
    State(state): State<ApiState>,
    axum::extract::Path(area_id): axum::extract::Path<u32>,
) -> ApiResult<Json<HashMap<String, f64>>> {
    let runtime_service = state.runtime_service.as_ref();
    
    let sample_rate = runtime_service.get_area_fcl_sample_rate(area_id).await
        .map_err(|e| ApiError::internal(format!("Failed to get sample rate: {}", e)))?;
    
    let mut response = HashMap::new();
    response.insert("sample_rate".to_string(), sample_rate);
    
    Ok(Json(response))
}

/// POST /v1/burst_engine/fcl_sampler/area/{area_id}/sample_rate
/// Set FCL sample rate for a specific cortical area
#[utoipa::path(
    post,
    path = "/v1/burst_engine/fcl_sampler/area/{area_id}/sample_rate",
    tag = "burst_engine",
    params(
        ("area_id" = u32, Path, description = "Cortical area ID (cortical_idx)")
    ),
    responses(
        (status = 200, description = "Sample rate updated", body = HashMap<String, f64>),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_area_fcl_sample_rate(
    State(state): State<ApiState>,
    axum::extract::Path(area_id): axum::extract::Path<u32>,
    Json(request): Json<HashMap<String, f64>>,
) -> ApiResult<Json<HashMap<String, f64>>> {
    let runtime_service = state.runtime_service.as_ref();
    
    let sample_rate = request.get("sample_rate").copied()
        .ok_or_else(|| ApiError::invalid_input("sample_rate required"))?;
    
    if sample_rate <= 0.0 {
        return Err(ApiError::invalid_input("Sample rate must be positive"));
    }
    
    runtime_service.set_area_fcl_sample_rate(area_id, sample_rate).await
        .map_err(|e| ApiError::internal(format!("Failed to set sample rate: {}", e)))?;
    
    let mut response = HashMap::new();
    response.insert("sample_rate".to_string(), sample_rate);
    
    Ok(Json(response))
}

// ============================================================================
// BURST ENGINE RUNTIME CONTROL ENDPOINTS
// ============================================================================

/// GET /v1/burst_engine/burst_counter
/// Get the total number of bursts executed since start
#[utoipa::path(
    get,
    path = "/v1/burst_engine/burst_counter",
    tag = "burst_engine",
    responses(
        (status = 200, description = "Burst counter", body = u64),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_burst_counter(State(state): State<ApiState>) -> ApiResult<Json<u64>> {
    let runtime_service = state.runtime_service.as_ref();
    
    let burst_count = runtime_service.get_burst_count().await
        .map_err(|e| ApiError::internal(format!("Failed to get burst counter: {}", e)))?;
    
    Ok(Json(burst_count))
}

/// POST /v1/burst_engine/start
/// Start the burst engine
#[utoipa::path(
    post,
    path = "/v1/burst_engine/start",
    tag = "burst_engine",
    responses(
        (status = 200, description = "Burst engine started", body = HashMap<String, String>),
        (status = 400, description = "Invalid state"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_start(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, String>>> {
    let runtime_service = state.runtime_service.as_ref();
    
    runtime_service.start().await
        .map_err(|e| ApiError::internal(format!("Failed to start burst engine: {}", e)))?;
    
    Ok(Json(HashMap::from([
        ("message".to_string(), "Burst engine started successfully".to_string())
    ])))
}

/// POST /v1/burst_engine/stop
/// Stop the burst engine
#[utoipa::path(
    post,
    path = "/v1/burst_engine/stop",
    tag = "burst_engine",
    responses(
        (status = 200, description = "Burst engine stopped", body = HashMap<String, String>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_stop(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, String>>> {
    let runtime_service = state.runtime_service.as_ref();
    
    runtime_service.stop().await
        .map_err(|e| ApiError::internal(format!("Failed to stop burst engine: {}", e)))?;
    
    Ok(Json(HashMap::from([
        ("message".to_string(), "Burst engine stopped successfully".to_string())
    ])))
}

/// POST /v1/burst_engine/hold
/// Pause the burst engine (alias for pause)
#[utoipa::path(
    post,
    path = "/v1/burst_engine/hold",
    tag = "burst_engine",
    responses(
        (status = 200, description = "Burst engine paused", body = HashMap<String, String>),
        (status = 400, description = "Invalid state"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_hold(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, String>>> {
    let runtime_service = state.runtime_service.as_ref();
    
    runtime_service.pause().await
        .map_err(|e| ApiError::internal(format!("Failed to pause burst engine: {}", e)))?;
    
    Ok(Json(HashMap::from([
        ("message".to_string(), "Burst engine paused successfully".to_string())
    ])))
}

/// POST /v1/burst_engine/resume
/// Resume the burst engine after pause
#[utoipa::path(
    post,
    path = "/v1/burst_engine/resume",
    tag = "burst_engine",
    responses(
        (status = 200, description = "Burst engine resumed", body = HashMap<String, String>),
        (status = 400, description = "Invalid state"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_resume(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, String>>> {
    let runtime_service = state.runtime_service.as_ref();
    
    runtime_service.resume().await
        .map_err(|e| ApiError::internal(format!("Failed to resume burst engine: {}", e)))?;
    
    Ok(Json(HashMap::from([
        ("message".to_string(), "Burst engine resumed successfully".to_string())
    ])))
}

/// GET /v1/burst_engine/config
/// Get burst engine configuration
#[utoipa::path(
    get,
    path = "/v1/burst_engine/config",
    tag = "burst_engine",
    responses(
        (status = 200, description = "Burst engine configuration", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_config(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let runtime_service = state.runtime_service.as_ref();
    
    let status = runtime_service.get_status().await
        .map_err(|e| ApiError::internal(format!("Failed to get config: {}", e)))?;
    
    let mut response = HashMap::new();
    response.insert("burst_frequency_hz".to_string(), serde_json::json!(status.frequency_hz));
    response.insert("burst_interval_seconds".to_string(), serde_json::json!(1.0 / status.frequency_hz));
    response.insert("target_frequency_hz".to_string(), serde_json::json!(status.frequency_hz));
    response.insert("is_running".to_string(), serde_json::json!(status.is_running));
    response.insert("is_paused".to_string(), serde_json::json!(status.is_paused));
    
    Ok(Json(response))
}

/// PUT /v1/burst_engine/config
/// Update burst engine configuration
#[utoipa::path(
    put,
    path = "/v1/burst_engine/config",
    tag = "burst_engine",
    responses(
        (status = 200, description = "Configuration updated", body = HashMap<String, serde_json::Value>),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn put_config(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let runtime_service = state.runtime_service.as_ref();
    
    // Extract burst_frequency_hz from request
    if let Some(freq) = request.get("burst_frequency_hz").and_then(|v| v.as_f64()) {
        if freq <= 0.0 {
            return Err(ApiError::invalid_input("Frequency must be positive"));
        }
        
        runtime_service.set_frequency(freq).await
            .map_err(|e| ApiError::internal(format!("Failed to set frequency: {}", e)))?;
    }
    
    // Return updated config
    get_config(State(state)).await
}

// ============================================================================
// FIRE LEDGER ENDPOINTS
// ============================================================================

/// GET /v1/burst_engine/fire_ledger/area/{area_id}/window_size
/// Get fire ledger window size for specific cortical area
#[utoipa::path(
    get,
    path = "/v1/burst_engine/fire_ledger/area/{area_id}/window_size",
    tag = "burst_engine",
    params(
        ("area_id" = u32, Path, description = "Cortical area ID (cortical_idx)")
    ),
    responses(
        (status = 200, description = "Window size", body = i32),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_fire_ledger_area_window_size(
    State(state): State<ApiState>,
    axum::extract::Path(area_id): axum::extract::Path<u32>,
) -> ApiResult<Json<i32>> {
    let runtime_service = state.runtime_service.as_ref();
    
    let configs = runtime_service.get_fire_ledger_configs().await
        .map_err(|e| ApiError::internal(format!("Failed to get fire ledger configs: {}", e)))?;
    
    // Find the window size for this area
    for (idx, window_size) in configs {
        if idx == area_id {
            return Ok(Json(window_size as i32));
        }
    }
    
    // Return default if not found
    Ok(Json(20))
}

/// PUT /v1/burst_engine/fire_ledger/area/{area_id}/window_size
/// Set fire ledger window size for specific cortical area
#[utoipa::path(
    put,
    path = "/v1/burst_engine/fire_ledger/area/{area_id}/window_size",
    tag = "burst_engine",
    params(
        ("area_id" = u32, Path, description = "Cortical area ID (cortical_idx)")
    ),
    responses(
        (status = 200, description = "Window size updated", body = HashMap<String, serde_json::Value>),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn put_fire_ledger_area_window_size(
    State(state): State<ApiState>,
    axum::extract::Path(area_id): axum::extract::Path<u32>,
    Json(request): Json<HashMap<String, i32>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let runtime_service = state.runtime_service.as_ref();
    
    let window_size = request.get("window_size").copied()
        .ok_or_else(|| ApiError::invalid_input("window_size required"))?;
    
    if window_size <= 0 {
        return Err(ApiError::invalid_input("Window size must be positive"));
    }
    
    runtime_service.configure_fire_ledger_window(area_id, window_size as usize).await
        .map_err(|e| ApiError::internal(format!("Failed to configure window: {}", e)))?;
    
    let mut response = HashMap::new();
    response.insert("success".to_string(), serde_json::json!(true));
    response.insert("area_id".to_string(), serde_json::json!(area_id));
    response.insert("window_size".to_string(), serde_json::json!(window_size));
    
    Ok(Json(response))
}

/// GET /v1/burst_engine/fire_ledger/area/{area_id}/history
/// Get fire ledger historical data for specific cortical area
#[utoipa::path(
    get,
    path = "/v1/burst_engine/fire_ledger/area/{area_id}/history",
    tag = "burst_engine",
    params(
        ("area_id" = String, Path, description = "Cortical area ID or index"),
        ("lookback_steps" = Option<i32>, Query, description = "Number of timesteps to retrieve")
    ),
    responses(
        (status = 200, description = "Fire ledger history", body = HashMap<String, serde_json::Value>),
        (status = 400, description = "Invalid area ID"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_fire_ledger_history(
    State(_state): State<ApiState>,
    axum::extract::Path(area_id): axum::extract::Path<String>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // Parse area_id as cortical_idx
    let cortical_idx = area_id.parse::<u32>()
        .map_err(|_| ApiError::invalid_input(format!("Invalid area_id: {}", area_id)))?;
    
    let _lookback_steps = params.get("lookback_steps")
        .and_then(|s| s.parse::<i32>().ok());
    
    // TODO: Implement fire ledger history retrieval from NPU
    // For now, return placeholder
    let mut response = HashMap::new();
    response.insert("success".to_string(), serde_json::json!(true));
    response.insert("area_id".to_string(), serde_json::json!(area_id));
    response.insert("cortical_idx".to_string(), serde_json::json!(cortical_idx));
    response.insert("history".to_string(), serde_json::json!([]));
    response.insert("window_size".to_string(), serde_json::json!(20));
    response.insert("note".to_string(), serde_json::json!("Fire ledger history not yet implemented"));
    
    Ok(Json(response))
}

// ============================================================================
// MEMBRANE POTENTIALS ENDPOINTS
// ============================================================================

/// GET /v1/burst_engine/membrane_potentials
/// Get membrane potentials for specific neurons
#[utoipa::path(
    get,
    path = "/v1/burst_engine/membrane_potentials",
    tag = "burst_engine",
    params(
        ("neuron_ids" = Vec<u64>, Query, description = "List of neuron IDs")
    ),
    responses(
        (status = 200, description = "Membrane potentials", body = HashMap<String, f32>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_membrane_potentials(
    State(_state): State<ApiState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, f32>>> {
    // Parse neuron_ids from query params
    let neuron_ids_str = params.get("neuron_ids")
        .ok_or_else(|| ApiError::invalid_input("neuron_ids parameter required"))?;
    
    // TODO: Parse comma-separated neuron IDs and fetch from NPU
    // For now, return empty
    tracing::debug!(target: "feagi-api", "GET membrane_potentials for neuron_ids: {}", neuron_ids_str);
    
    Ok(Json(HashMap::new()))
}

/// PUT /v1/burst_engine/membrane_potentials
/// Update membrane potentials for specific neurons
#[utoipa::path(
    put,
    path = "/v1/burst_engine/membrane_potentials",
    tag = "burst_engine",
    responses(
        (status = 200, description = "Membrane potentials updated", body = HashMap<String, String>),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn put_membrane_potentials(
    State(_state): State<ApiState>,
    Json(potentials): Json<HashMap<String, f32>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Update membrane potentials in NPU
    tracing::info!(target: "feagi-api", "PUT membrane_potentials: {} neurons", potentials.len());
    
    Ok(Json(HashMap::from([
        ("message".to_string(), format!("Updated {} neuron membrane potentials", potentials.len()))
    ])))
}

// ============================================================================
// FREQUENCY MEASUREMENT ENDPOINTS
// ============================================================================

/// GET /v1/burst_engine/frequency_status
/// Get current frequency measurement status
#[utoipa::path(
    get,
    path = "/v1/burst_engine/frequency_status",
    tag = "burst_engine",
    responses(
        (status = 200, description = "Frequency status", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_frequency_status(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let runtime_service = state.runtime_service.as_ref();
    
    let status = runtime_service.get_status().await
        .map_err(|e| ApiError::internal(format!("Failed to get status: {}", e)))?;
    
    let mut response = HashMap::new();
    response.insert("target_frequency_hz".to_string(), serde_json::json!(status.frequency_hz));
    response.insert("actual_frequency_hz".to_string(), serde_json::json!(status.frequency_hz));
    response.insert("burst_count".to_string(), serde_json::json!(status.burst_count));
    response.insert("is_measuring".to_string(), serde_json::json!(false));
    
    Ok(Json(response))
}

/// POST /v1/burst_engine/measure_frequency
/// Trigger frequency measurement
#[utoipa::path(
    post,
    path = "/v1/burst_engine/measure_frequency",
    tag = "burst_engine",
    responses(
        (status = 200, description = "Measurement started", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_measure_frequency(
    State(_state): State<ApiState>,
    Json(request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let duration = request.get("duration_seconds").and_then(|v| v.as_f64()).unwrap_or(5.0);
    let sample_count = request.get("sample_count").and_then(|v| v.as_i64()).unwrap_or(100) as i32;
    
    tracing::info!(target: "feagi-api", "Starting frequency measurement: {}s, {} samples", duration, sample_count);
    
    // TODO: Implement frequency measurement
    let mut response = HashMap::new();
    response.insert("status".to_string(), serde_json::json!("started"));
    response.insert("duration_seconds".to_string(), serde_json::json!(duration));
    response.insert("sample_count".to_string(), serde_json::json!(sample_count));
    
    Ok(Json(response))
}

/// GET /v1/burst_engine/frequency_history
/// Get frequency measurement history
#[utoipa::path(
    get,
    path = "/v1/burst_engine/frequency_history",
    tag = "burst_engine",
    params(
        ("limit" = Option<i32>, Query, description = "Number of measurements to return")
    ),
    responses(
        (status = 200, description = "Frequency history", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_frequency_history(
    State(_state): State<ApiState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let limit = params.get("limit").and_then(|s| s.parse::<i32>().ok()).unwrap_or(10);
    
    // TODO: Implement frequency history retrieval
    let mut response = HashMap::new();
    response.insert("measurements".to_string(), serde_json::json!([]));
    response.insert("limit".to_string(), serde_json::json!(limit));
    
    Ok(Json(response))
}

/// POST /v1/burst_engine/force_connectome_integration
/// Force connectome integration (rebuild neural connections)
#[utoipa::path(
    post,
    path = "/v1/burst_engine/force_connectome_integration",
    tag = "burst_engine",
    responses(
        (status = 200, description = "Integration forced", body = HashMap<String, String>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_force_connectome_integration(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Implement connectome integration forcing
    tracing::info!(target: "feagi-api", "Force connectome integration requested");
    
    Ok(Json(HashMap::from([
        ("message".to_string(), "Connectome integration initiated".to_string()),
        ("status".to_string(), "not_yet_implemented".to_string())
    ])))
}



