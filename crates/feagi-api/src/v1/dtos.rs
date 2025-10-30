// API Version 1 - Data Transfer Objects
// These DTOs must match Python FastAPI response structures exactly for backward compatibility

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Health check response (must match Python FastAPI format exactly)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthCheckResponseV1 {
    /// Overall system status
    #[schema(example = "healthy")]
    pub status: String,
    
    /// Brain is ready to process
    pub brain_readiness: bool,
    
    /// Burst engine is running
    pub burst_engine: bool,
    
    /// Total neuron count
    pub neuron_count: u64,
    
    /// Total synapse count
    pub synapse_count: u64,
    
    /// Number of cortical areas
    pub cortical_area_count: usize,
    
    /// Genome is valid
    pub genome_validity: bool,
    
    /// InfluxDB is available
    pub influxdb_availability: bool,
    
    /// Path to connectome
    pub connectome_path: String,
    
    /// Genome timestamp (ISO 8601)
    pub genome_timestamp: String,
    
    /// Change state
    #[schema(example = "modified")]
    pub change_state: String,
    
    /// Changes saved externally
    pub changes_saved_externally: bool,
}

/// Readiness check response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReadinessCheckResponseV1 {
    /// Whether the system is ready to accept requests
    pub ready: bool,
    
    /// Detailed component readiness
    pub components: ComponentReadiness,
}

/// Component readiness details
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ComponentReadiness {
    /// API server is ready
    pub api: bool,
    
    /// Burst engine is ready
    pub burst_engine: bool,
    
    /// State manager is ready
    pub state_manager: bool,
    
    /// Connectome manager is ready
    pub connectome: bool,
}
