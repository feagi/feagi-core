// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Burst Engine API DTOs
//! 
//! Request/response types for FCL, Fire Queue, and burst engine control

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Fire Candidate List (FCL) response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FCLResponse {
    /// Current timestep
    pub timestep: u64,
    
    /// Total neurons in FCL
    pub total_neurons: usize,
    
    /// Global FCL (all neuron IDs across areas)
    pub global_fcl: Vec<u64>,
    
    /// FCL organized by cortical area
    pub cortical_areas: HashMap<String, Vec<u64>>,
    
    /// Default fire ledger window size
    pub default_window_size: u32,
    
    /// Number of cortical areas with active neurons
    pub active_cortical_count: usize,
}

/// Fire Queue response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FireQueueResponse {
    /// Current timestep
    pub timestep: u64,
    
    /// Total neurons that fired
    pub total_fired: usize,
    
    /// Fired neurons organized by cortical area
    pub cortical_areas: HashMap<String, Vec<u64>>,
}

/// FCL status response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FCLStatusResponse {
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Fire Ledger window configuration response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FireLedgerConfigResponse {
    pub default_window_size: u32,
    pub areas: HashMap<String, u32>,
    pub total_configured_areas: usize,
}

/// Burst engine statistics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BurstEngineStats {
    pub burst_count: u64,
    pub frequency_hz: f64,
    pub active: bool,
    pub paused: bool,
}

/// Burst engine status
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BurstEngineStatus {
    pub active: bool,
    pub paused: bool,
    pub burst_count: u64,
    pub frequency_hz: f64,
}

/// Burst engine control request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BurstEngineControlRequest {
    /// Action to perform: "start", "pause", "stop", "resume"
    pub action: String,
}

