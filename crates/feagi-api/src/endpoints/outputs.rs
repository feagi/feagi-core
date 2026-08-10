// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
 * FEAGI v1 Outputs API
 *
 * Endpoints for output/motor target configuration
 * Maps to Python: feagi/api/v1/outputs.py
 */

use crate::common::ApiState;
use crate::common::{ApiError, ApiResult, Json, State};
// Removed - using crate::common::State instead
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;

// ============================================================================
// OUTPUT TARGETS
// ============================================================================

/// Get available output targets from connected motor/output agents.
#[utoipa::path(
    get,
    path = "/v1/output/targets",
    tag = "outputs",
    responses(
        (status = 200, description = "Output targets", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_targets(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, Value>>> {
    // Get motor/output capable agents from PNS
    let agent_service = state
        .agent_service
        .as_ref()
        .ok_or_else(|| ApiError::internal("Agent service not available"))?;

    let agent_ids = agent_service
        .list_agents()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to list agents: {}", e)))?;

    // Filter for agents with motor/output capabilities
    let mut motor_agents = Vec::new();
    for agent_id in agent_ids {
        // Get agent properties to check capabilities
        if let Ok(props) = agent_service.get_agent_properties(&agent_id).await {
            // Check if agent has motor capabilities
            if props.capabilities.contains_key("motor")
                || props.capabilities.contains_key("output")
                || props.agent_type.to_lowercase().contains("motor")
            {
                motor_agents.push(agent_id);
            }
        }
    }

    let mut response = HashMap::new();
    response.insert("targets".to_string(), json!(motor_agents));

    Ok(Json(response))
}

/// Configure output targets and motor agent connections.
#[utoipa::path(
    post,
    path = "/v1/output/configure",
    tag = "outputs",
    responses(
        (status = 200, description = "Outputs configured", body = HashMap<String, String>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_configure(State(_state): State<ApiState>, Json(request): Json<HashMap<String, Value>>) -> ApiResult<Json<HashMap<String, String>>> {
    // Extract configuration from request
    let config = request.get("config").ok_or_else(|| ApiError::invalid_input("Missing 'config' field"))?;

    // TODO: Store output configuration in runtime state
    // For now, just validate the structure
    if !config.is_object() {
        return Err(ApiError::invalid_input("'config' must be an object"));
    }

    tracing::info!(target: "feagi-api", "Output configuration updated: {} targets",
        config.as_object().map(|o| o.len()).unwrap_or(0));

    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Outputs configured successfully".to_string(),
    )])))
}

// ============================================================================
// MOTOR OUTPUT SNAPSHOT (runtime tap)
// ============================================================================

/// Single voxel sample in a motor activity area.
#[derive(Serialize, Clone, Debug, utoipa::ToSchema)]
pub struct MotorTapSample {
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub potential: f32,
}

/// Per-cortical_area-area activity captured by the motor tap.
#[derive(Serialize, Clone, Debug, utoipa::ToSchema)]
pub struct MotorTapArea {
    pub cortical_id: String,
    pub cortical_idx: u32,
    pub neuron_count: usize,
    pub samples: Vec<MotorTapSample>,
}

/// Per-agent publish stats captured by the motor tap.
#[derive(Serialize, Clone, Debug, utoipa::ToSchema)]
pub struct MotorTapAgent {
    pub agent_id: String,
    pub burst_num: u64,
    pub timestamp_ms: i64,
    pub byte_count: usize,
    pub published: bool,
    pub last_error: String,
    pub subscribed_cortical_ids: Vec<String>,
}

/// Read-only spike-cost / activity aggregate for a burst's tap snapshot.
///
/// Additive convenience derived from the same per-area data already returned in `areas`,
/// so benchmark/diagnostic clients (encoder/substrate experiments) can read spike-cost and
/// activity-intensity summaries without re-deriving them. Mirrors
/// [`feagi_npu::runtime_taps::BurstActivitySummary`].
#[derive(Serialize, Clone, Debug, utoipa::ToSchema)]
pub struct ActivitySummary {
    /// Number of areas that fired at least one neuron this burst (after any filter).
    pub active_area_count: usize,
    /// Total fired neurons across the (filtered) areas this burst.
    pub total_fired_neurons: usize,
    /// Largest single-area fired-neuron count this burst.
    pub peak_area_fired_neurons: usize,
    /// Mean fired neurons per active area (0.0 when none fired).
    pub mean_area_fired_neurons: f64,
    /// Mean potential across captured firing samples (0.0 when none).
    pub mean_sample_potential: f64,
    /// Peak potential across captured firing samples (0.0 when none).
    pub peak_sample_potential: f32,
}

impl From<feagi_npu::runtime_taps::BurstActivitySummary> for ActivitySummary {
    fn from(s: feagi_npu::runtime_taps::BurstActivitySummary) -> Self {
        ActivitySummary {
            active_area_count: s.active_area_count,
            total_fired_neurons: s.total_fired_neurons,
            peak_area_fired_neurons: s.peak_area_fired_neurons,
            mean_area_fired_neurons: s.mean_area_fired_neurons,
            mean_sample_potential: s.mean_sample_potential,
            peak_sample_potential: s.peak_sample_potential,
        }
    }
}

/// Response payload for `GET /v1/output/motor_snapshot/last`.
#[derive(Serialize, Clone, Debug, utoipa::ToSchema)]
pub struct MotorSnapshotResponse {
    /// Burst counter when the area snapshot was captured. Zero if no motor activity
    /// has been recorded since FEAGI started.
    pub burst_num: u64,
    /// Wall-clock millisecond timestamp when the snapshot was captured.
    pub timestamp_ms: i64,
    /// Convenience flag for clients - true when at least one area was captured.
    pub has_data: bool,
    /// Total motor cortical_area areas seen this burst (before per-agent filtering).
    pub total_areas: usize,
    /// Total firing neurons across all motor areas this burst.
    pub total_neurons: usize,
    /// Read-only spike-cost / activity-intensity aggregate over the (filtered) areas.
    pub activity_summary: ActivitySummary,
    /// Per-area activity, ordered as captured by the burst loop.
    pub areas: Vec<MotorTapArea>,
    /// Per-agent publish summary. Empty when no agents have published since FEAGI start.
    pub agents: Vec<MotorTapAgent>,
}

/// Get the most recent motor output produced by the burst loop.
///
/// This taps directly into the motor pipeline before per-agent transport
/// filtering, so debuggers can confirm OPU activity even when no embodiment is
/// connected. The `agents` array shows what was actually published per agent.
///
/// Optional `agent_id` filters the `agents` list. Optional `cortical_id` keeps
/// only the matching OPU in `areas` and recomputes `total_*` (same base64 as
/// ``MotorTapArea.cortical_id`` in JSON responses).
#[utoipa::path(
    get,
    path = "/v1/output/motor_snapshot/last",
    tag = "outputs",
    params(
        ("agent_id" = Option<String>, Query, description = "Filter agents by id"),
        ("cortical_id" = Option<String>, Query, description = "Filter motor areas to one cortical_area id (base64)")
    ),
    responses(
        (status = 200, description = "Latest motor pipeline snapshot", body = MotorSnapshotResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_motor_snapshot_last(
    State(_state): State<ApiState>,
    axum::extract::Query(query): axum::extract::Query<HashMap<String, String>>,
) -> ApiResult<Json<MotorSnapshotResponse>> {
    let snap = feagi_npu::runtime_taps::BurstTaps::instance().motor_snapshot();
    let agent_filter = query.get("agent_id").cloned();
    let area_filter = query.get("cortical_id").cloned();

    // Filter raw tap areas first so the activity summary and the returned areas stay
    // consistent under the optional `cortical_id` filter.
    let mut raw_areas = snap.areas;
    if let Some(ref cid) = area_filter {
        if !cid.is_empty() {
            raw_areas.retain(|a| a.cortical_id == *cid);
        }
    }

    let activity_summary: ActivitySummary =
        feagi_npu::runtime_taps::BurstActivitySummary::from_areas(snap.burst_num, snap.timestamp_ms, &raw_areas).into();

    let areas: Vec<MotorTapArea> = raw_areas
        .into_iter()
        .map(|a| MotorTapArea {
            cortical_id: a.cortical_id,
            cortical_idx: a.cortical_idx,
            neuron_count: a.neuron_count,
            samples: a
                .samples
                .into_iter()
                .map(|s| MotorTapSample {
                    x: s.x,
                    y: s.y,
                    z: s.z,
                    potential: s.potential,
                })
                .collect(),
        })
        .collect();

    let total_areas = areas.len();
    let total_neurons: usize = areas.iter().map(|a| a.neuron_count).sum();
    let has_data = total_areas > 0 && snap.burst_num > 0;

    let mut agents: Vec<MotorTapAgent> = snap
        .per_agent
        .into_iter()
        .filter(|(id, _)| match &agent_filter {
            Some(filter) => filter == id,
            None => true,
        })
        .map(|(agent_id, stats)| MotorTapAgent {
            agent_id,
            burst_num: stats.burst_num,
            timestamp_ms: stats.timestamp_ms,
            byte_count: stats.byte_count,
            published: stats.published,
            last_error: stats.last_error,
            subscribed_cortical_ids: stats.subscribed_cortical_ids,
        })
        .collect();
    agents.sort_by(|a, b| a.agent_id.cmp(&b.agent_id));

    Ok(Json(MotorSnapshotResponse {
        burst_num: snap.burst_num,
        timestamp_ms: snap.timestamp_ms,
        has_data,
        total_areas,
        total_neurons,
        activity_summary,
        areas,
        agents,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use feagi_npu::runtime_taps::{AreaActivity, BurstActivitySummary, TapSample};

    fn area(cortical_id: &str, neuron_count: usize, potentials: &[f32]) -> AreaActivity {
        AreaActivity {
            cortical_id: cortical_id.to_string(),
            cortical_idx: 0,
            neuron_count,
            samples: potentials
                .iter()
                .map(|&p| TapSample {
                    x: 0,
                    y: 0,
                    z: 0,
                    potential: p,
                })
                .collect(),
        }
    }

    /// The API `ActivitySummary` must mirror the core `BurstActivitySummary` field-for-field
    /// so the read-side spike-cost projection stays a faithful copy of the burst-engine source.
    #[test]
    fn activity_summary_from_core_preserves_fields() {
        let core = BurstActivitySummary {
            burst_num: 7,
            timestamp_ms: 123,
            active_area_count: 2,
            total_fired_neurons: 5,
            peak_area_fired_neurons: 3,
            mean_area_fired_neurons: 2.5,
            mean_sample_potential: 0.4,
            peak_sample_potential: 0.9,
        };
        let api: ActivitySummary = core.clone().into();
        assert_eq!(api.active_area_count, core.active_area_count);
        assert_eq!(api.total_fired_neurons, core.total_fired_neurons);
        assert_eq!(api.peak_area_fired_neurons, core.peak_area_fired_neurons);
        assert_eq!(api.mean_area_fired_neurons, core.mean_area_fired_neurons);
        assert_eq!(api.mean_sample_potential, core.mean_sample_potential);
        assert_eq!(api.peak_sample_potential, core.peak_sample_potential);
    }

    /// Filtering raw tap areas before summarizing must scope spike-cost to the selected area,
    /// matching the `cortical_id` filter applied to the returned `areas`.
    #[test]
    fn summary_respects_cortical_id_filter() {
        let raw = [area("AAAA", 2, &[0.2, 0.8]), area("BBBB", 3, &[0.5, 0.5, 0.5])];

        let filtered: Vec<AreaActivity> = raw.iter().filter(|a| a.cortical_id == "BBBB").cloned().collect();
        let summary: ActivitySummary = BurstActivitySummary::from_areas(1, 0, &filtered).into();

        assert_eq!(summary.active_area_count, 1);
        assert_eq!(summary.total_fired_neurons, 3);
        assert_eq!(summary.peak_area_fired_neurons, 3);
    }
}
