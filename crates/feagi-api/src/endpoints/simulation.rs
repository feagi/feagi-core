// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
 * FEAGI v1 Simulation API
 *
 * Endpoints for simulation control and stimulation
 * Maps to Python: feagi/api/v1/simulation.py
 */

use crate::common::ApiState;
use crate::common::{ApiError, ApiResult, Json, Path, State};
// Removed - using crate::common::State instead
use serde_json::{json, Value};
use std::collections::HashMap;

// ============================================================================
// SIMULATION CONTROL
// ============================================================================

/// Upload a stimulation script for neural activity simulation.
#[utoipa::path(
    post,
    path = "/v1/simulation/upload/string",
    tag = "simulation",
    responses(
        (status = 200, description = "Stimulation script uploaded", body = HashMap<String, String>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_stimulation_upload(
    State(_state): State<ApiState>,
    Json(request): Json<HashMap<String, Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // Validate stimulation script is provided
    let _script = request
        .get("stimulation_script")
        .ok_or_else(|| ApiError::invalid_input("Missing 'stimulation_script' field"))?;

    // TODO: Upload and apply stimulation script
    tracing::info!(target: "feagi-api", "Stimulation script upload requested");

    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Stimulation script uploaded successfully".to_string(),
    )])))
}

/// Reset simulation state to initial conditions.
#[utoipa::path(
    post,
    path = "/v1/simulation/reset",
    tag = "simulation",
    responses(
        (status = 200, description = "Simulation reset", body = HashMap<String, String>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_reset(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Reset simulation state

    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Simulation reset successfully".to_string(),
    )])))
}

/// Get simulation status including active state and running scripts.
#[utoipa::path(
    get,
    path = "/v1/simulation/status",
    tag = "simulation",
    responses(
        (status = 200, description = "Simulation status", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_status(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, Value>>> {
    // TODO: Retrieve simulation status
    let mut response = HashMap::new();
    response.insert("active".to_string(), json!(false));
    response.insert("stimulation_running".to_string(), json!(false));

    Ok(Json(response))
}

/// Get simulation statistics including total stimulations and active scripts.
#[utoipa::path(
    get,
    path = "/v1/simulation/stats",
    tag = "simulation",
    responses(
        (status = 200, description = "Simulation statistics", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_stats(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, Value>>> {
    // TODO: Retrieve simulation statistics
    let mut response = HashMap::new();
    response.insert("total_stimulations".to_string(), json!(0));
    response.insert("active_scripts".to_string(), json!(0));

    Ok(Json(response))
}

/// Configure simulation parameters and behavior settings.
#[utoipa::path(
    post,
    path = "/v1/simulation/config",
    tag = "simulation",
    responses(
        (status = 200, description = "Simulation configured", body = HashMap<String, String>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_config(
    State(_state): State<ApiState>,
    Json(_request): Json<HashMap<String, Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Apply simulation configuration

    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Simulation configured successfully".to_string(),
    )])))
}

// ============================================================================
// BURST-INDEXED STIMULUS SCHEDULE
// ============================================================================

/// Schedule multi-area force-fire stimulation at burst offsets.
///
/// Contract:
/// - Request: `{ "entries": [ { "burst_offset": u64, "stimulation": { "<cortical id base64>":
///   [[x, y, z], ...], ... } } ] }`. `stimulation` has the shape of `stimulation_payload` in
///   `POST /v1/agent/manual_stimulation`; coordinates must be non-negative.
/// - Offsets resolve to ABSOLUTE burst indexes when the request is accepted: offset 0 is the
///   next burst. Entries fire in the burst loop through the manual-stimulation force-fire path,
///   so all areas of one entry land on the same burst. An entry that is already late fires on
///   the next burst, never earlier.
/// - Optional `"repeat": { "period_bursts": u64 }`: the entries replay every `period_bursts`
///   (entry fires at `offset + k * period_bursts`, k = 0, 1, 2, ...) until the schedule is
///   cancelled with DELETE. `period_bursts` must be greater than the largest `burst_offset`
///   (400 otherwise, and for 0). A replay that is already late fires on the next burst and
///   missed slots are skipped, never queued up.
/// - Each entry may carry `"potential": f32` (finite, > 0; default 100.0; 400 otherwise): the
///   membrane potential its forced neurons fire with. A probe that bypasses threshold and
///   refractory; a neuron that already fired naturally in that burst keeps its own potential.
/// - Response 200: `{ "schedule_id": u64, "start_burst": u64, "end_burst": u64 | null,
///   "applied_potential": f32 }` (`applied_potential` = the first entry's potential)
///   (absolute indexes of the first and last entry; `end_burst` is null for a repeating
///   schedule). 400 on empty `entries`, unknown cortical area, invalid coordinate, an entry
///   without voxels or an invalid `repeat`.
/// - `DELETE /v1/simulation/stimulus_schedule/{schedule_id}` cancels the entries not yet fired
///   (`{ "schedule_id", "cancelled": true }`, 404 if not active).
/// - `GET /v1/simulation/stimulus_schedule` returns `{ "schedules": [ { "schedule_id",
///   "start_burst", "end_burst", "remaining_entries", "next_burst", "repeat" } ] }`. A one-shot
///   schedule is removed once its last entry has fired; a repeating one (`end_burst` null,
///   `repeat: { "period_bursts" }`, `remaining_entries` = entries per cycle, `next_burst` =
///   next replay) stays until cancelled.
/// - Schedules are cleared on genome load / connectome reset, and a deleted cortical area's
///   scheduled injections are dropped (a schedule left empty is removed).
#[utoipa::path(
    post,
    path = "/v1/simulation/stimulus_schedule",
    tag = "simulation",
    request_body = crate::v1::StimulusScheduleRequest,
    responses(
        (status = 200, description = "Schedule accepted", body = crate::v1::StimulusScheduleResponse),
        (status = 400, description = "Invalid schedule"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_stimulus_schedule(
    State(state): State<ApiState>,
    Json(request): Json<crate::v1::StimulusScheduleRequest>,
) -> ApiResult<Json<crate::v1::StimulusScheduleResponse>> {
    use feagi_services::traits::runtime_service::StimulusScheduleEntry;

    if request.entries.is_empty() {
        return Err(ApiError::invalid_input("'entries' must not be empty"));
    }
    let mut entries = Vec::with_capacity(request.entries.len());
    let mut applied_potential = None;
    for (index, entry) in request.entries.into_iter().enumerate() {
        let potential = feagi_services::traits::agent_service::resolve_stimulation_potential(
            entry.potential,
            &format!("entries[{index}].potential"),
        )
        .map_err(ApiError::invalid_input)?;
        applied_potential.get_or_insert(potential);
        let mut areas = Vec::with_capacity(entry.stimulation.len());
        for (cortical_id, coordinates) in entry.stimulation {
            if !state
                .connectome_service
                .cortical_area_exists(&cortical_id)
                .await?
            {
                return Err(ApiError::invalid_input(format!(
                    "entries[{index}]: unknown cortical area '{cortical_id}'"
                )));
            }
            let mut voxels = Vec::with_capacity(coordinates.len());
            for coord in coordinates {
                match coord.as_slice() {
                    [x, y, z] if *x >= 0 && *y >= 0 && *z >= 0 => {
                        voxels.push((*x as u32, *y as u32, *z as u32, potential))
                    }
                    _ => {
                        return Err(ApiError::invalid_input(format!(
                            "entries[{index}]: area '{cortical_id}': coordinate {coord:?} must be \
                             non-negative [x, y, z]"
                        )))
                    }
                }
            }
            if !voxels.is_empty() {
                areas.push((cortical_id, voxels));
            }
        }
        if areas.is_empty() {
            return Err(ApiError::invalid_input(format!(
                "entries[{index}]: 'stimulation' has no voxels"
            )));
        }
        entries.push(StimulusScheduleEntry {
            burst_offset: entry.burst_offset,
            areas,
        });
    }

    let repeat_period = request.repeat.map(|r| r.period_bursts);
    if let Some(period) = repeat_period {
        let max_offset = entries.iter().map(|e| e.burst_offset).max().unwrap_or(0);
        if period == 0 || period <= max_offset {
            return Err(ApiError::invalid_input(format!(
                "'repeat.period_bursts' ({period}) must be greater than the largest \
                 burst_offset ({max_offset})"
            )));
        }
    }
    let receipt = state
        .runtime_service
        .schedule_stimulus(entries, repeat_period)
        .await?;
    Ok(Json(crate::v1::StimulusScheduleResponse {
        schedule_id: receipt.schedule_id,
        start_burst: receipt.start_burst,
        end_burst: receipt.end_burst,
        applied_potential: applied_potential
            .unwrap_or(feagi_services::traits::agent_service::DEFAULT_STIMULATION_POTENTIAL),
    }))
}

/// Cancel a stimulus schedule; entries not yet fired are dropped.
#[utoipa::path(
    delete,
    path = "/v1/simulation/stimulus_schedule/{schedule_id}",
    tag = "simulation",
    params(("schedule_id" = u64, Path, description = "Schedule id from POST")),
    responses(
        (status = 200, description = "Schedule cancelled", body = HashMap<String, serde_json::Value>),
        (status = 404, description = "Schedule not active"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_stimulus_schedule(
    State(state): State<ApiState>,
    Path(schedule_id): Path<u64>,
) -> ApiResult<Json<HashMap<String, Value>>> {
    if !state
        .runtime_service
        .cancel_stimulus_schedule(schedule_id)
        .await?
    {
        return Err(ApiError::not_found(
            "stimulus_schedule",
            &schedule_id.to_string(),
        ));
    }
    Ok(Json(HashMap::from([
        ("schedule_id".to_string(), json!(schedule_id)),
        ("cancelled".to_string(), json!(true)),
    ])))
}

/// List active stimulus schedules with their remaining entries.
#[utoipa::path(
    get,
    path = "/v1/simulation/stimulus_schedule",
    tag = "simulation",
    responses(
        (status = 200, description = "Active schedules", body = crate::v1::StimulusScheduleListResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_stimulus_schedules(
    State(state): State<ApiState>,
) -> ApiResult<Json<crate::v1::StimulusScheduleListResponse>> {
    let schedules = state
        .runtime_service
        .list_stimulus_schedules()
        .await?
        .into_iter()
        .map(|s| crate::v1::StimulusScheduleInfo {
            schedule_id: s.schedule_id,
            start_burst: s.start_burst,
            end_burst: s.end_burst,
            remaining_entries: s.remaining_entries,
            next_burst: s.next_burst,
            repeat: s
                .repeat_period_bursts
                .map(|period_bursts| crate::v1::StimulusScheduleRepeat { period_bursts }),
        })
        .collect();
    Ok(Json(crate::v1::StimulusScheduleListResponse { schedules }))
}
