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
pub async fn post_configure(
    State(_state): State<ApiState>,
    Json(request): Json<HashMap<String, Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // Extract configuration from request
    let config = request
        .get("config")
        .ok_or_else(|| ApiError::invalid_input("Missing 'config' field"))?;

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

/// Per-cortical-area activity captured by the motor tap.
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
    /// Total motor cortical areas seen this burst (before per-agent filtering).
    pub total_areas: usize,
    /// Total firing neurons across all motor areas this burst.
    pub total_neurons: usize,
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
/// Optional query parameter `agent_id` filters the `agents` list to the named agent.
#[utoipa::path(
    get,
    path = "/v1/output/motor_snapshot/last",
    tag = "outputs",
    params(
        ("agent_id" = Option<String>, Query, description = "Filter agents by id")
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
    let snap = feagi_npu_burst_engine::BurstTaps::instance().motor_snapshot();
    let agent_filter = query.get("agent_id").cloned();

    let total_neurons: usize = snap.areas.iter().map(|a| a.neuron_count).sum();
    let total_areas = snap.areas.len();
    let has_data = total_areas > 0 && snap.burst_num > 0;

    let areas: Vec<MotorTapArea> = snap
        .areas
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
        areas,
        agents,
    }))
}
