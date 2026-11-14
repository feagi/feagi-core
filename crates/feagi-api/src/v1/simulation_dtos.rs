// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Simulation API DTOs
//!
//! Request/response types for simulation control and stimulation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Stimulation script upload request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StimulationUploadRequest {
    pub stimulation_script: HashMap<String, serde_json::Value>,
}

/// Simulation control request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SimulationControlRequest {
    pub config: HashMap<String, serde_json::Value>,
}

/// Simulation status response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SimulationStatusResponse {
    pub active: bool,
    pub stimulation_running: bool,
}

/// Simulation statistics response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SimulationStatsResponse {
    pub total_stimulations: u64,
    pub active_scripts: usize,
}

/// Generic success response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SimulationSuccessResponse {
    pub message: String,
    pub success: bool,
}

/// One burst-indexed stimulation entry of a stimulus schedule.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StimulusScheduleEntryRequest {
    /// Burst offset from the next burst (0 = next burst).
    pub burst_offset: u64,
    /// Same payload shape as `POST /v1/agent/manual_stimulation`'s `stimulation_payload`:
    /// cortical area id (base64) -> list of `[x, y, z]` voxel coordinates. Every area of one
    /// entry fires on the same burst.
    pub stimulation: HashMap<String, Vec<Vec<i32>>>,
    /// Membrane potential the entry's forced neurons fire with, one value for the whole entry.
    /// Optional; 100.0 when absent. Must be finite and greater than 0 (400 otherwise).
    ///
    /// This is a probe: it bypasses threshold and refractory period. A neuron that already
    /// fired naturally in that burst keeps its own potential. Mechanisms that read firing
    /// potential (for example MP-driven PSP or graded modulator drivers) see this value.
    /// A server that predates this field ignores it; check `applied_potential` in the response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub potential: Option<f32>,
}

/// Stimulus schedule request (`POST /v1/simulation/stimulus_schedule`)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StimulusScheduleRequest {
    pub entries: Vec<StimulusScheduleEntryRequest>,
    /// Optional: replay the entries every `period_bursts` until cancelled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat: Option<StimulusScheduleRepeat>,
}

/// Repeat-until-cancelled settings of a stimulus schedule.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StimulusScheduleRepeat {
    /// Bursts between replays; must be greater than the largest `burst_offset`. Entry `i`
    /// fires at `offset_i + k * period_bursts` for k = 0, 1, 2, ...
    pub period_bursts: u64,
}

/// Stimulus schedule receipt: absolute burst indexes of the first and last entry
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StimulusScheduleResponse {
    pub schedule_id: u64,
    pub start_burst: u64,
    /// Absolute burst of the last entry; `null` for a repeating schedule.
    pub end_burst: Option<u64>,
    /// Potential applied by the first entry (100.0 when it gave none); every entry's own
    /// value is validated the same way.
    pub applied_potential: f32,
}

/// One active stimulus schedule
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StimulusScheduleInfo {
    pub schedule_id: u64,
    pub start_burst: u64,
    /// Absolute burst of the last entry; `null` for a repeating schedule.
    pub end_burst: Option<u64>,
    /// Entries not yet fired (for a repeating schedule: entries per cycle)
    pub remaining_entries: usize,
    /// Absolute burst of the next entry to fire
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_burst: Option<u64>,
    /// Present on a repeating schedule
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat: Option<StimulusScheduleRepeat>,
}

/// Active stimulus schedules (`GET /v1/simulation/stimulus_schedule`)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StimulusScheduleListResponse {
    pub schedules: Vec<StimulusScheduleInfo>,
}

#[cfg(test)]
mod stimulus_potential_tests {
    use super::*;

    #[test]
    fn entry_potential_defaults_to_none_and_round_trips() {
        let e: StimulusScheduleEntryRequest =
            serde_json::from_str(r#"{"burst_offset":0,"stimulation":{}}"#).unwrap();
        assert_eq!(e.potential, None);
        let e: StimulusScheduleEntryRequest =
            serde_json::from_str(r#"{"burst_offset":0,"stimulation":{},"potential":0.25}"#)
                .unwrap();
        assert_eq!(e.potential, Some(0.25));
    }

    #[test]
    fn schedule_response_carries_applied_potential() {
        let r = StimulusScheduleResponse {
            schedule_id: 1,
            start_burst: 2,
            end_burst: Some(3),
            applied_potential: 0.25,
        };
        assert_eq!(serde_json::to_value(&r).unwrap()["applied_potential"], 0.25);
    }
}
