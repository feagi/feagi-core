// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Input API Endpoints - Exact port from Python `/v1/input/*`

// Removed - using crate::common::State instead
use crate::common::ApiState;
use crate::common::{ApiError, ApiResult, Json, State};
use serde::Serialize;
use std::collections::HashMap;

/// Get vision input configuration and settings.
#[utoipa::path(
    get,
    path = "/v1/input/vision",
    tag = "input",
    responses(
        (status = 200, description = "Vision input configuration", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_vision(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // TODO: Get vision input configuration
    Ok(Json(HashMap::new()))
}

/// Update vision input configuration.
#[utoipa::path(
    post,
    path = "/v1/input/vision",
    tag = "input",
    responses(
        (status = 200, description = "Vision input updated", content_type = "application/json"),
        (status = 500, description = "Not yet implemented")
    )
)]
pub async fn post_vision(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// Get list of available input sources (vision, audio, etc.).
#[utoipa::path(get, path = "/v1/input/sources", tag = "input")]
pub async fn get_sources(State(_state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    Ok(Json(vec!["vision".to_string()]))
}

/// Configure input sources and their parameters.
#[utoipa::path(post, path = "/v1/input/configure", tag = "input")]
pub async fn post_configure(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([("message".to_string(), "Input configured".to_string())])))
}

// ============================================================================
// SENSORY INPUT SNAPSHOT (runtime tap)
// ============================================================================

/// Single voxel sample in a sensory area.
#[derive(Serialize, Clone, Debug, utoipa::ToSchema)]
pub struct SensorTapSample {
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub potential: f32,
}

/// Per-cortical_area-area sensory activity captured by the sensor tap.
#[derive(Serialize, Clone, Debug, utoipa::ToSchema)]
pub struct SensorTapArea {
    pub cortical_id: String,
    pub cortical_idx: u32,
    pub neuron_count: usize,
    pub samples: Vec<SensorTapSample>,
}

/// Response payload for `GET /v1/input/sensor_snapshot/last`.
#[derive(Serialize, Clone, Debug, utoipa::ToSchema)]
pub struct SensorSnapshotResponse {
    /// Burst counter when the snapshot was captured. Zero if no sensory traffic
    /// has been received since FEAGI started.
    pub burst_num: u64,
    /// Wall-clock millisecond timestamp when the snapshot was captured.
    pub timestamp_ms: i64,
    /// Convenience flag for clients - true when at least one area was captured.
    pub has_data: bool,
    /// Total cortical_area areas with sensory input this burst.
    pub total_areas: usize,
    /// Total samples across all sensory areas this burst.
    pub total_neurons: usize,
    /// Per-area sensory samples, ordered as decoded by the burst loop.
    pub areas: Vec<SensorTapArea>,
}

/// Get the most recent sensory input decoded by the burst loop.
///
/// This taps directly into the sensory ingestion pipeline, surfacing exactly
/// what the burst loop consumed. Optional `cortical_id` query parameter
/// filters the `areas` list to a single cortical_area area.
#[utoipa::path(
    get,
    path = "/v1/input/sensor_snapshot/last",
    tag = "input",
    params(
        ("cortical_id" = Option<String>, Query, description = "Filter areas by base64 cortical_area id")
    ),
    responses(
        (status = 200, description = "Latest sensory pipeline snapshot", body = SensorSnapshotResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_sensor_snapshot_last(
    State(_state): State<ApiState>,
    axum::extract::Query(query): axum::extract::Query<HashMap<String, String>>,
) -> ApiResult<Json<SensorSnapshotResponse>> {
    let snap = feagi_npu::runtime_taps::BurstTaps::instance().sensor_snapshot();
    let cortical_filter = query.get("cortical_id").cloned();

    let mut areas: Vec<SensorTapArea> = snap
        .areas
        .into_iter()
        .filter(|a| match &cortical_filter {
            Some(filter) => filter == &a.cortical_id,
            None => true,
        })
        .map(|a| SensorTapArea {
            cortical_id: a.cortical_id,
            cortical_idx: a.cortical_idx,
            neuron_count: a.neuron_count,
            samples: a
                .samples
                .into_iter()
                .map(|s| SensorTapSample {
                    x: s.x,
                    y: s.y,
                    z: s.z,
                    potential: s.potential,
                })
                .collect(),
        })
        .collect();
    areas.sort_by(|a, b| a.cortical_id.cmp(&b.cortical_id));

    let total_areas = areas.len();
    let total_neurons: usize = areas.iter().map(|a| a.neuron_count).sum();
    let has_data = total_areas > 0 && snap.burst_num > 0;

    Ok(Json(SensorSnapshotResponse {
        burst_num: snap.burst_num,
        timestamp_ms: snap.timestamp_ms,
        has_data,
        total_areas,
        total_neurons,
        areas,
    }))
}
