// API Version 1 - Data Transfer Objects
// These DTOs must match Python FastAPI response structures exactly for backward compatibility

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Health check response (must match Python FastAPI format exactly)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "status": "healthy",
    "brain_readiness": true,
    "burst_engine": true,
    "neuron_count": 1000,
    "synapse_count": 5000,
    "cortical_area_count": 10,
    "genome_validity": true,
    "influxdb_availability": false,
    "connectome_path": "/path/to/connectome",
    "genome_timestamp": "2025-10-29T12:34:56Z",
    "change_state": "saved",
    "changes_saved_externally": false
}))]
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
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "ready": true,
    "components": {
        "api": true,
        "burst_engine": true,
        "state_manager": true,
        "connectome": true
    }
}))]
pub struct ReadinessCheckResponseV1 {
    /// Is the system ready?
    pub ready: bool,
    
    /// Component readiness details
    pub components: ComponentReadiness,
}

/// Component readiness status
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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
