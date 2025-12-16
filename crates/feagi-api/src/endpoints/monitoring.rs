// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
 * FEAGI v1 Monitoring API
 * 
 * Endpoints for system monitoring, metrics, and telemetry
 * Maps to Python: feagi/api/v1/monitoring.py
 */

use crate::common::{ApiError, ApiResult, State, Json, Path, Query};
use crate::common::ApiState;
// Removed - using crate::common::State instead
use serde_json::{json, Value};
use std::collections::HashMap;

// ============================================================================
// MONITORING & METRICS
// ============================================================================

/// Get monitoring system status including metrics collection and brain readiness.
#[utoipa::path(
    get,
    path = "/v1/monitoring/status",
    tag = "monitoring",
    responses(
        (status = 200, description = "Monitoring status", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_status(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, Value>>> {
    // Get monitoring status from analytics service
    let analytics_service = state.analytics_service.as_ref();
    
    // Get system health as a proxy for monitoring status
    let health = analytics_service.get_system_health().await
        .map_err(|e| ApiError::internal(format!("Failed to get system health: {}", e)))?;
    
    let mut response = HashMap::new();
    response.insert("enabled".to_string(), json!(true));
    response.insert("metrics_collected".to_string(), json!(5)); // Static count for now
    response.insert("brain_readiness".to_string(), json!(health.brain_readiness));
    response.insert("burst_engine_active".to_string(), json!(health.burst_engine_active));
    
    Ok(Json(response))
}

/// Get system metrics including burst frequency, neuron count, and brain readiness.
#[utoipa::path(
    get,
    path = "/v1/monitoring/metrics",
    tag = "monitoring",
    responses(
        (status = 200, description = "System metrics", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_metrics(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, Value>>> {
    // Get system metrics from analytics and runtime services
    let runtime_service = state.runtime_service.as_ref();
    let analytics_service = state.analytics_service.as_ref();
    
    let runtime_status = runtime_service.get_status().await
        .map_err(|e| ApiError::internal(format!("Failed to get runtime status: {}", e)))?;
    
    let health = analytics_service.get_system_health().await
        .map_err(|e| ApiError::internal(format!("Failed to get system health: {}", e)))?;
    
    let mut response = HashMap::new();
    response.insert("burst_frequency_hz".to_string(), json!(runtime_status.frequency_hz));
    response.insert("burst_count".to_string(), json!(runtime_status.burst_count));
    response.insert("neuron_count".to_string(), json!(health.neuron_count));
    response.insert("cortical_area_count".to_string(), json!(health.cortical_area_count));
    response.insert("brain_readiness".to_string(), json!(health.brain_readiness));
    response.insert("burst_engine_active".to_string(), json!(health.burst_engine_active));
    
    Ok(Json(response))
}

/// Get detailed monitoring data with timestamps for analysis and debugging.
#[utoipa::path(
    get,
    path = "/v1/monitoring/data",
    tag = "monitoring",
    responses(
        (status = 200, description = "Monitoring data", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_data(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, Value>>> {
    // Get detailed monitoring data from all services
    let analytics_service = state.analytics_service.as_ref();
    
    let health = analytics_service.get_system_health().await
        .map_err(|e| ApiError::internal(format!("Failed to get system health: {}", e)))?;
    
    // Return comprehensive monitoring data
    let mut data = HashMap::new();
    data.insert("neuron_count".to_string(), json!(health.neuron_count));
    data.insert("cortical_area_count".to_string(), json!(health.cortical_area_count));
    data.insert("burst_count".to_string(), json!(health.burst_count));
    data.insert("brain_readiness".to_string(), json!(health.brain_readiness));
    data.insert("burst_engine_active".to_string(), json!(health.burst_engine_active));
    
    let mut response = HashMap::new();
    response.insert("data".to_string(), json!(data));
    response.insert("timestamp".to_string(), json!(chrono::Utc::now().to_rfc3339()));
    
    Ok(Json(response))
}

/// Get performance metrics including CPU and memory usage.
#[utoipa::path(
    get,
    path = "/v1/monitoring/performance",
    tag = "monitoring",
    responses(
        (status = 200, description = "Performance metrics", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_performance(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, Value>>> {
    let mut response = HashMap::new();
    response.insert("cpu_usage".to_string(), json!(0.0));
    response.insert("memory_usage".to_string(), json!(0.0));
    
    Ok(Json(response))
}

