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

