// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Physiology API DTOs
//!
//! Request/response types for physiology parameter management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Response containing physiology parameters
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PhysiologyResponse {
    /// Physiology parameters
    pub physiology: PhysiologyParameters,
}

/// Physiology parameters
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PhysiologyParameters {
    /// Simulation timestep in seconds
    pub simulation_timestep: f64,

    /// Maximum neuron age
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age: Option<u32>,

    /// Evolution burst count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evolution_burst_count: Option<u64>,

    /// IPU idle threshold
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipu_idle_threshold: Option<u32>,

    /// Plasticity queue depth
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plasticity_queue_depth: Option<u32>,

    /// Lifespan management interval
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifespan_mgmt_interval: Option<u32>,

    /// Sleep trigger inactivity window
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sleep_trigger_inactivity_window: Option<u32>,

    /// Sleep trigger neural activity max
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sleep_trigger_neural_activity_max: Option<f64>,

    /// Quantization precision for numeric values
    /// Options: "fp32" (default), "fp16", "int8"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantization_precision: Option<String>,
}

/// Request to update physiology parameters
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PhysiologyUpdateRequest {
    /// Physiology parameters to update
    pub physiology: HashMap<String, serde_json::Value>,
}

/// Response for physiology update
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PhysiologyUpdateResponse {
    pub success: bool,
    pub updated: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}
