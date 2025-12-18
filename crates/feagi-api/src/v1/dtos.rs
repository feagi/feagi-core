// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// API Version 1 - Data Transfer Objects
// These DTOs must match Python FastAPI response structures exactly for backward compatibility

use serde::{Deserialize, Serialize};

/// Health check response (must match Python FastAPI format exactly)
#[derive(Clone, Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct HealthCheckResponseV1 {
    /// Overall system status
    pub status: String,

    /// Is the brain ready to process data?
    pub brain_readiness: bool,

    /// Is the burst engine running?
    pub burst_engine: bool,

    /// Total number of neurons
    pub neuron_count: usize,

    /// Total number of synapses
    /// TODO: Get from NPU when available
    pub synapse_count: usize,

    /// Number of cortical areas
    pub cortical_area_count: usize,

    /// Is the genome valid?
    /// TODO: Get from genome validator
    pub genome_validity: bool,

    /// Is InfluxDB available?
    /// TODO: Get from analytics service
    pub influxdb_availability: bool,

    /// Path to connectome file
    /// TODO: Get from state manager
    pub connectome_path: String,

    /// Genome last modified timestamp
    /// TODO: Get from genome service
    pub genome_timestamp: String,

    /// Change tracking state
    /// TODO: Get from state manager
    pub change_state: String,

    /// Are changes saved externally?
    /// TODO: Get from state manager
    pub changes_saved_externally: bool,
}

/// Readiness check response
#[derive(Clone, Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ReadinessCheckResponseV1 {
    /// Is the system ready?
    pub ready: bool,

    /// Component readiness details
    pub components: ComponentReadiness,
}

/// Component readiness status
#[derive(Clone, Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ComponentReadiness {
    /// API server ready
    pub api: bool,

    /// Burst engine ready
    pub burst_engine: bool,

    /// State manager ready
    pub state_manager: bool,

    /// Connectome loaded
    pub connectome: bool,
}
