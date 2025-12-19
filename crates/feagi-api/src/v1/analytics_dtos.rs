// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// Analytics DTOs for V1 API
//
// These DTOs must match Python FastAPI response structures exactly.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// System health response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "burst_engine_active": true,
    "brain_readiness": true,
    "neuron_count": 1200000,
    "cortical_area_count": 52,
    "burst_count": 12345
}))]
pub struct SystemHealthResponse {
    /// Whether the burst engine is active
    pub burst_engine_active: bool,

    /// Whether the brain is ready (initialized)
    pub brain_readiness: bool,

    /// Total neuron count
    pub neuron_count: usize,

    /// Total cortical area count
    pub cortical_area_count: usize,

    /// Total burst count
    pub burst_count: u64,
}

/// Cortical area statistics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "cortical_id": "v1",
    "neuron_count": 45000,
    "synapse_count": 2250000,
    "density": 0.85,
    "populated": true
}))]
pub struct CorticalAreaStatsResponse {
    /// Cortical area ID
    pub cortical_id: String,

    /// Number of neurons
    pub neuron_count: usize,

    /// Number of synapses
    pub synapse_count: usize,

    /// Neuron density (0.0 to 1.0)
    pub density: f32,

    /// Whether area has neurons
    pub populated: bool,
}

/// Connectivity statistics between two areas
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "source_area": "v1",
    "target_area": "v2",
    "synapse_count": 125000,
    "avg_weight": 0.75,
    "excitatory_count": 100000,
    "inhibitory_count": 25000
}))]
pub struct ConnectivityStatsResponse {
    /// Source cortical area ID
    pub source_area: String,

    /// Target cortical area ID
    pub target_area: String,

    /// Number of synapses
    pub synapse_count: usize,

    /// Average synaptic weight
    pub avg_weight: f32,

    /// Excitatory synapse count
    pub excitatory_count: usize,

    /// Inhibitory synapse count
    pub inhibitory_count: usize,
}

/// Connectome analytics response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "total_neurons": 1200000,
    "total_synapses": 60000000,
    "total_cortical_areas": 52,
    "populated_areas": 45,
    "avg_density": 0.82,
    "per_area_stats": {}
}))]
pub struct ConnectomeAnalyticsResponse {
    /// Total number of neurons
    pub total_neurons: usize,

    /// Total number of synapses
    pub total_synapses: usize,

    /// Total number of cortical areas
    pub total_cortical_areas: usize,

    /// Number of populated areas
    pub populated_areas: usize,

    /// Average neuron density across all areas
    pub avg_density: f32,

    /// Per-area statistics
    pub per_area_stats: HashMap<String, CorticalAreaStatsResponse>,
}

/// Populated areas response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PopulatedAreasResponse {
    /// List of populated areas with neuron counts
    pub areas: Vec<PopulatedAreaInfo>,

    /// Total populated area count
    pub total_count: usize,
}

/// Populated area information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "cortical_id": "v1",
    "neuron_count": 45000
}))]
pub struct PopulatedAreaInfo {
    /// Cortical area ID
    pub cortical_id: String,

    /// Number of neurons
    pub neuron_count: usize,
}

/// Neuron density response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "cortical_id": "v1",
    "density": 0.85
}))]
pub struct NeuronDensityResponse {
    /// Cortical area ID
    pub cortical_id: String,

    /// Neuron density (0.0 to 1.0)
    pub density: f32,
}
