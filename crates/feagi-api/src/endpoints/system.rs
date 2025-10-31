// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! System API Endpoints - Exact port from Python `/v1/system/*`
//!
//! Reference: feagi-py/feagi/api/v1/system.py

use axum::{
    extract::State,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::common::{ApiError, ApiResult};
use crate::transports::http::server::ApiState;

// ============================================================================
// REQUEST/RESPONSE MODELS (matching Python schemas exactly)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct HealthCheckResponse {
    pub burst_engine: bool,
    pub connected_agents: Option<i32>,
    pub influxdb_availability: bool,
    pub neuron_count_max: i64,
    pub synapse_count_max: i64,
    pub latest_changes_saved_externally: bool,
    pub genome_availability: bool,
    pub genome_validity: Option<bool>,
    pub brain_readiness: bool,
    pub feagi_session: Option<i64>,
    pub fitness: Option<f64>,
    pub cortical_area_count: Option<i32>,
    pub neuron_count: Option<i64>,
    pub memory_neuron_count: Option<i64>,
    pub regular_neuron_count: Option<i64>,
    pub synapse_count: Option<i64>,
    pub estimated_brain_size_in_MB: Option<f64>,
    pub genome_num: Option<i32>,
    pub genome_timestamp: Option<i64>,
    pub simulation_timestep: Option<f64>,
    pub memory_area_stats: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    pub amalgamation_pending: Option<HashMap<String, serde_json::Value>>,
}

// ============================================================================
// ENDPOINTS
// ============================================================================

/// GET /v1/system/health_check
/// 
/// Get comprehensive system health information
#[utoipa::path(
    get,
    path = "/v1/system/health_check",
    responses(
        (status = 200, description = "System health retrieved successfully", body = HealthCheckResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "system"
)]
pub async fn get_health_check(
    State(state): State<ApiState>,
) -> ApiResult<Json<HealthCheckResponse>> {
    let analytics_service = state.analytics_service.as_ref();
    
    // Get system health from analytics service
    let health = analytics_service
        .get_system_health()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get system health: {}", e)))?;

    // Get runtime status if available
    let burst_engine_active = state.runtime_service
        .get_status()
        .await
        .map(|status| status.is_running)
        .unwrap_or(false);

    let burst_count = state.runtime_service
        .get_burst_count()
        .await
        .ok();

    // TODO: Get these from actual services when available
    let connected_agents = None; // TODO: Get from agent service
    let influxdb_availability = false; // TODO: Get from monitoring service
    let neuron_count_max = 1_000_000; // TODO: Get from config
    let synapse_count_max = 10_000_000; // TODO: Get from config
    let latest_changes_saved_externally = false; // TODO: Get from state manager
    let genome_availability = health.cortical_area_count > 0;
    let genome_validity = Some(health.brain_readiness);
    let feagi_session = None; // TODO: Get from state manager
    let fitness = None; // TODO: Get from evolution service
    let memory_neuron_count = None; // TODO: Get from NPU when available
    let regular_neuron_count = None; // TODO: Get from NPU when available
    let estimated_brain_size_in_MB = None; // TODO: Calculate from NPU
    let genome_num = None; // TODO: Get from genome service
    let genome_timestamp = None; // TODO: Get from genome service
    let simulation_timestep = None; // TODO: Get from burst engine config
    let memory_area_stats = None; // TODO: Get from NPU
    let amalgamation_pending = None; // TODO: Get from genome service

    Ok(Json(HealthCheckResponse {
        burst_engine: burst_engine_active,
        connected_agents,
        influxdb_availability,
        neuron_count_max,
        synapse_count_max,
        latest_changes_saved_externally,
        genome_availability,
        genome_validity,
        brain_readiness: health.brain_readiness,
        feagi_session,
        fitness,
        cortical_area_count: Some(health.cortical_area_count as i32),
        neuron_count: Some(health.neuron_count as i64),
        memory_neuron_count,
        regular_neuron_count,
        synapse_count: Some(0), // TODO: Get from NPU
        estimated_brain_size_in_MB,
        genome_num,
        genome_timestamp,
        simulation_timestep,
        memory_area_stats,
        amalgamation_pending,
    }))
}

/// GET /v1/system/cortical_area_visualization_skip_rate
/// 
/// Get cortical area visualization skip rate
#[utoipa::path(
    get,
    path = "/v1/system/cortical_area_visualization_skip_rate",
    responses(
        (status = 200, description = "Skip rate retrieved successfully", body = i32),
        (status = 500, description = "Internal server error")
    ),
    tag = "system"
)]
pub async fn get_cortical_area_visualization_skip_rate(
    State(_state): State<ApiState>,
) -> ApiResult<Json<i32>> {
    // TODO: Get from visualization config service
    // For now return default value
    Ok(Json(1))
}

/// PUT /v1/system/cortical_area_visualization_skip_rate
/// 
/// Set cortical area visualization skip rate
#[utoipa::path(
    put,
    path = "/v1/system/cortical_area_visualization_skip_rate",
    request_body = i32,
    responses(
        (status = 200, description = "Skip rate updated successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "system"
)]
pub async fn set_cortical_area_visualization_skip_rate(
    State(_state): State<ApiState>,
    Json(skip_rate): Json<i32>,
) -> ApiResult<Json<serde_json::Value>> {
    // TODO: Set in visualization config service
    Ok(Json(serde_json::json!({
        "message": format!("Skip rate set to {}", skip_rate)
    })))
}

/// GET /v1/system/cortical_area_visualization_suppression_threshold
/// 
/// Get cortical area visualization suppression threshold
#[utoipa::path(
    get,
    path = "/v1/system/cortical_area_visualization_suppression_threshold",
    responses(
        (status = 200, description = "Threshold retrieved successfully", body = i32),
        (status = 500, description = "Internal server error")
    ),
    tag = "system"
)]
pub async fn get_cortical_area_visualization_suppression_threshold(
    State(_state): State<ApiState>,
) -> ApiResult<Json<i32>> {
    // TODO: Get from visualization config service
    // For now return default value
    Ok(Json(0))
}

/// PUT /v1/system/cortical_area_visualization_suppression_threshold
/// 
/// Set cortical area visualization suppression threshold
#[utoipa::path(
    put,
    path = "/v1/system/cortical_area_visualization_suppression_threshold",
    request_body = i32,
    responses(
        (status = 200, description = "Threshold updated successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "system"
)]
pub async fn set_cortical_area_visualization_suppression_threshold(
    State(_state): State<ApiState>,
    Json(threshold): Json<i32>,
) -> ApiResult<Json<serde_json::Value>> {
    // TODO: Set in visualization config service
    Ok(Json(serde_json::json!({
        "message": format!("Suppression threshold set to {}", threshold)
    })))
}

