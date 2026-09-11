// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
 * FEAGI v1 Monitoring API
 *
 * Endpoints for system monitoring, metrics, and telemetry
 * Maps to Python: feagi/api/v1/monitoring.py
 */

use crate::common::ApiState;
use crate::common::{ApiError, ApiResult, Json, Query, State};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use utoipa::IntoParams;

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
    let health = analytics_service
        .get_system_health()
        .await
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

    let runtime_status = runtime_service
        .get_status()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get runtime status: {}", e)))?;

    let health = analytics_service
        .get_system_health()
        .await
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

    let health = analytics_service
        .get_system_health()
        .await
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

// ============================================================================
// CORTICAL ACTIVITY MONITORING
// ============================================================================

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct CorticalActivityQuery {
    /// Cortical area ID (Base64 encoded) to monitor
    pub area: String,
    /// Duration to monitor in seconds
    #[serde(default = "default_duration")]
    pub duration: f32,
}

fn default_duration() -> f32 {
    1.0
}

/// Monitor real-time neuron firing activity for a specific cortical_area area over a time window.
#[utoipa::path(
    get,
    path = "/v1/monitoring/cortical_activity",
    tag = "monitoring",
    params(CorticalActivityQuery),
    responses(
        (status = 200, description = "Cortical area firing activity", body = HashMap<String, serde_json::Value>),
        (status = 404, description = "Cortical area not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_cortical_activity(
    State(state): State<ApiState>,
    Query(params): Query<CorticalActivityQuery>,
) -> ApiResult<Json<HashMap<String, Value>>> {
    use tracing::info;

    let runtime_service = state.runtime_service.as_ref();
    let connectome_service = state.connectome_service.as_ref();

    info!(
        target: "feagi-api",
        "Monitoring cortical_area activity for area={}, duration={}s",
        params.area,
        params.duration
    );

    let area = connectome_service
        .get_cortical_area(&params.area)
        .await
        .map_err(|_| ApiError::not_found("cortical_area area", &params.area))?;

    let cortical_idx = area.cortical_idx;
    let area_name = area.name.clone();

    let start_burst = runtime_service
        .get_burst_count()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get burst count: {}", e)))?;

    let sample_interval_ms = 50;
    let num_samples = ((params.duration * 1000.0) / sample_interval_ms as f32).ceil() as usize;

    let mut spike_history = Vec::new();
    let mut neuron_fire_counts: HashMap<u32, u32> = HashMap::new();
    let mut total_membrane_potential = 0.0;
    let mut potential_samples = 0;

    for _ in 0..num_samples {
        tokio::time::sleep(tokio::time::Duration::from_millis(sample_interval_ms)).await;

        let fq_sample = runtime_service
            .get_fire_queue_sample()
            .await
            .map_err(|e| ApiError::internal(format!("Failed to get fire queue: {}", e)))?;

        if let Some((neuron_ids, _x_coords, _y_coords, _z_coords, potentials)) = fq_sample.get(&cortical_idx) {
            let current_burst = runtime_service
                .get_burst_count()
                .await
                .map_err(|e| ApiError::internal(format!("Failed to get burst count: {}", e)))?;

            for (i, &neuron_id) in neuron_ids.iter().enumerate() {
                *neuron_fire_counts.entry(neuron_id).or_insert(0) += 1;

                let potential = potentials.get(i).copied().unwrap_or(0.0);
                total_membrane_potential += potential;
                potential_samples += 1;

                spike_history.push(json!({
                    "burst": current_burst,
                    "neuron_id": neuron_id,
                    "potential": potential
                }));
            }
        }
    }

    let end_burst = runtime_service
        .get_burst_count()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get burst count: {}", e)))?;

    let total_spikes = spike_history.len();
    let firing_rate_hz = if params.duration > 0.0 {
        total_spikes as f32 / params.duration
    } else {
        0.0
    };

    let active_neurons: Vec<u32> = neuron_fire_counts.keys().copied().collect();
    let peak_firing_neuron = neuron_fire_counts.iter().max_by_key(|(_, &count)| count).map(|(&neuron_id, _)| neuron_id);

    let avg_membrane_potential = if potential_samples > 0 {
        total_membrane_potential / potential_samples as f32
    } else {
        0.0
    };

    info!(
        target: "feagi-api",
        "Activity monitoring complete: {} spikes, {} active neurons, {:.2} Hz",
        total_spikes,
        active_neurons.len(),
        firing_rate_hz
    );

    Ok(Json(HashMap::from([
        ("area_id".to_string(), json!(params.area)),
        ("area_name".to_string(), json!(area_name)),
        ("duration_ms".to_string(), json!(params.duration * 1000.0)),
        ("burst_count_sampled".to_string(), json!(end_burst - start_burst)),
        (
            "firing_statistics".to_string(),
            json!({
                "total_spikes": total_spikes,
                "firing_rate_hz": firing_rate_hz,
                "active_neurons": active_neurons,
                "active_neuron_count": active_neurons.len(),
                "peak_firing_neuron": peak_firing_neuron,
                "avg_membrane_potential": avg_membrane_potential,
            }),
        ),
        ("spike_history".to_string(), json!(spike_history)),
    ])))
}
