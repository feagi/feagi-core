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
    response.insert(
        "burst_engine_active".to_string(),
        json!(health.burst_engine_active),
    );

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
    response.insert(
        "burst_frequency_hz".to_string(),
        json!(runtime_status.frequency_hz),
    );
    response.insert("burst_count".to_string(), json!(runtime_status.burst_count));
    response.insert("neuron_count".to_string(), json!(health.neuron_count));
    response.insert(
        "cortical_area_count".to_string(),
        json!(health.cortical_area_count),
    );
    response.insert("brain_readiness".to_string(), json!(health.brain_readiness));
    response.insert(
        "burst_engine_active".to_string(),
        json!(health.burst_engine_active),
    );

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
    data.insert(
        "cortical_area_count".to_string(),
        json!(health.cortical_area_count),
    );
    data.insert("burst_count".to_string(), json!(health.burst_count));
    data.insert("brain_readiness".to_string(), json!(health.brain_readiness));
    data.insert(
        "burst_engine_active".to_string(),
        json!(health.burst_engine_active),
    );

    let mut response = HashMap::new();
    response.insert("data".to_string(), json!(data));
    response.insert(
        "timestamp".to_string(),
        json!(chrono::Utc::now().to_rfc3339()),
    );

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
pub async fn get_performance(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, Value>>> {
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

/// Spikes of one area collected over a monitoring window.
///
/// The burst engine keeps only the latest burst's fire queue, so a poll can return the same
/// burst more than once. Each burst is therefore counted once, keyed by the burst number
/// stored with the sample.
struct ActivityWindow {
    /// Burst counter when the window opened; samples at or before it are not in the window.
    start_burst: u64,
    last_burst: Option<u64>,
    spike_history: Vec<Value>,
    neuron_fire_counts: HashMap<u32, u32>,
    total_membrane_potential: f32,
    potential_samples: usize,
    /// Distinct bursts in the window that a poll saw (whether or not the area fired).
    bursts_observed: u64,
    /// Observed bursts in which the area fired.
    bursts_with_firing: u64,
}

impl ActivityWindow {
    fn new(start_burst: u64) -> Self {
        Self {
            start_burst,
            last_burst: None,
            spike_history: Vec::new(),
            neuron_fire_counts: HashMap::new(),
            total_membrane_potential: 0.0,
            potential_samples: 0,
            bursts_observed: 0,
            bursts_with_firing: 0,
        }
    }

    /// Record one polled sample (`burst` is the sample's own burst number).
    /// Returns false when the burst was already counted or lies before the window.
    fn observe(
        &mut self,
        burst: u64,
        area: Option<&feagi_services::traits::runtime_service::FireQueueAreaSample>,
    ) -> bool {
        if burst <= self.start_burst || self.last_burst.is_some_and(|last| burst <= last) {
            return false;
        }
        self.last_burst = Some(burst);
        self.bursts_observed += 1;

        if let Some((neuron_ids, _x, _y, _z, potentials)) = area {
            if !neuron_ids.is_empty() {
                self.bursts_with_firing += 1;
            }
            for (i, &neuron_id) in neuron_ids.iter().enumerate() {
                *self.neuron_fire_counts.entry(neuron_id).or_insert(0) += 1;

                let potential = potentials.get(i).copied().unwrap_or(0.0);
                self.total_membrane_potential += potential;
                self.potential_samples += 1;

                self.spike_history.push(json!({
                    "burst": burst,
                    "neuron_id": neuron_id,
                    "potential": potential
                }));
            }
        }
        true
    }
}

/// Monitor real-time neuron firing activity for a specific cortical area over a time window.
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
        "Monitoring cortical activity for area={}, duration={}s",
        params.area,
        params.duration
    );

    let area = connectome_service
        .get_cortical_area(&params.area)
        .await
        .map_err(|_| ApiError::not_found("cortical area", &params.area))?;

    let cortical_idx = area.cortical_idx;
    let area_name = area.name.clone();

    let start_burst = runtime_service
        .get_burst_count()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get burst count: {}", e)))?;

    let sample_interval_ms = 50;
    let num_samples = ((params.duration * 1000.0) / sample_interval_ms as f32).ceil() as usize;

    let mut window = ActivityWindow::new(start_burst);

    for _ in 0..num_samples {
        tokio::time::sleep(tokio::time::Duration::from_millis(sample_interval_ms)).await;

        let snapshot = runtime_service
            .get_fire_queue_snapshot()
            .await
            .map_err(|e| ApiError::internal(format!("Failed to get fire queue: {}", e)))?;

        if let Some(snapshot) = snapshot {
            window.observe(snapshot.burst, snapshot.areas.get(&cortical_idx));
        }
    }

    let end_burst = runtime_service
        .get_burst_count()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get burst count: {}", e)))?;

    let ActivityWindow {
        spike_history,
        neuron_fire_counts,
        total_membrane_potential,
        potential_samples,
        bursts_observed,
        bursts_with_firing,
        ..
    } = window;

    let total_spikes = spike_history.len();
    let firing_rate_hz = if params.duration > 0.0 {
        total_spikes as f32 / params.duration
    } else {
        0.0
    };
    // Bursts that ran in the window but were never seen by the 50 ms poll (the cache holds
    // only the latest burst, so above ~20 Hz some bursts are skipped).
    let bursts_in_window = end_burst.saturating_sub(start_burst);
    let bursts_unsampled = bursts_in_window.saturating_sub(bursts_observed);

    let active_neurons: Vec<u32> = neuron_fire_counts.keys().copied().collect();
    let peak_firing_neuron = neuron_fire_counts
        .iter()
        .max_by_key(|(_, &count)| count)
        .map(|(&neuron_id, _)| neuron_id);

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
        (
            "burst_count_sampled".to_string(),
            json!(end_burst - start_burst),
        ),
        ("bursts_in_window".to_string(), json!(bursts_in_window)),
        ("bursts_observed".to_string(), json!(bursts_observed)),
        (
            "bursts_observed_with_firing".to_string(),
            json!(bursts_with_firing),
        ),
        ("bursts_unsampled".to_string(), json!(bursts_unsampled)),
        (
            "firing_statistics".to_string(),
            json!({
                "total_spikes": total_spikes,
                "firing_rate_hz": firing_rate_hz,
                "firing_rate_is_lower_bound": bursts_unsampled > 0,
                "active_neurons": active_neurons,
                "active_neuron_count": active_neurons.len(),
                "peak_firing_neuron": peak_firing_neuron,
                "avg_membrane_potential": avg_membrane_potential,
            }),
        ),
        ("spike_history".to_string(), json!(spike_history)),
    ])))
}

#[cfg(test)]
mod tests {
    use super::*;

    use feagi_services::traits::runtime_service::FireQueueAreaSample as AreaSample;

    fn area(ids: &[u32]) -> AreaSample {
        (
            ids.to_vec(),
            vec![0; ids.len()],
            vec![0; ids.len()],
            vec![0; ids.len()],
            vec![1.5; ids.len()],
        )
    }

    #[test]
    fn same_burst_polled_repeatedly_is_counted_once() {
        let mut window = ActivityWindow::new(100);
        let fired = area(&[7, 9]);
        // Ten 50 ms polls of a 2 Hz burst all return burst 101.
        let accepted = (0..10)
            .filter(|_| window.observe(101, Some(&fired)))
            .count();
        assert_eq!(accepted, 1);
        assert_eq!(window.spike_history.len(), 2);
        assert_eq!(window.bursts_observed, 1);
        assert_eq!(window.bursts_with_firing, 1);
        assert!(window
            .spike_history
            .iter()
            .all(|spike| spike["burst"] == json!(101)));
    }

    #[test]
    fn spikes_carry_the_sample_burst_and_distinct_bursts_accumulate() {
        let mut window = ActivityWindow::new(100);
        assert!(window.observe(101, Some(&area(&[1]))));
        assert!(window.observe(103, None)); // burst 102 was never sampled
        assert!(window.observe(104, Some(&area(&[1, 2]))));
        assert_eq!(window.bursts_observed, 3);
        assert_eq!(window.bursts_with_firing, 2);
        assert_eq!(window.spike_history.len(), 3);
        assert_eq!(window.neuron_fire_counts[&1], 2);
        let labels: Vec<_> = window
            .spike_history
            .iter()
            .map(|s| s["burst"].clone())
            .collect();
        assert_eq!(labels, vec![json!(101), json!(104), json!(104)]);
    }

    #[test]
    fn stale_samples_from_before_the_window_are_ignored() {
        let mut window = ActivityWindow::new(100);
        assert!(!window.observe(100, Some(&area(&[5]))));
        assert!(!window.observe(99, Some(&area(&[5]))));
        assert!(window.spike_history.is_empty());
        assert_eq!(window.bursts_observed, 0);
    }
}
