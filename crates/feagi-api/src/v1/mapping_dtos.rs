// Mapping (Connectome) DTOs for V1 API
//
// These DTOs must match Python FastAPI response structures exactly.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Mapping rule information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "src_cortical_area": "v1",
    "dst_cortical_area": "v2",
    "mapping_type": "one_to_many",
    "mapping_data": {
        "type": "topological",
        "parameters": {}
    }
}))]
pub struct MappingInfo {
    /// Source cortical area ID
    pub src_cortical_area: String,
    
    /// Destination cortical area ID
    pub dst_cortical_area: String,
    
    /// Mapping type (e.g., "one_to_one", "one_to_many", "topological")
    pub mapping_type: String,
    
    /// Mapping configuration data
    pub mapping_data: serde_json::Value,
}

/// Create mapping request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateMappingRequest {
    /// Source cortical area ID
    pub src_cortical_area: String,
    
    /// Destination cortical area ID
    pub dst_cortical_area: String,
    
    /// Mapping type
    pub mapping_type: String,
    
    /// Optional: Mapping configuration data
    #[serde(default)]
    pub mapping_data: Option<serde_json::Value>,
}

/// List mappings response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MappingListResponse {
    /// List of all mappings
    pub mappings: Vec<MappingInfo>,
    
    /// Total count
    pub total_count: usize,
}

/// Synapse statistics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "cortical_area_id": "v1",
    "total_synapses": 1000000,
    "active_synapses": 45000,
    "inactive_synapses": 955000,
    "total_neurons": 10000
}))]
pub struct SynapseStats {
    /// Cortical area ID
    pub cortical_area_id: String,
    
    /// Total number of synapses
    pub total_synapses: usize,
    
    /// Number of active synapses
    pub active_synapses: usize,
    
    /// Number of inactive synapses
    pub inactive_synapses: usize,
    
    /// Total neurons in the area
    pub total_neurons: usize,
}

/// Connectome statistics response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConnectomeStatsResponse {
    /// Total number of cortical areas
    pub total_cortical_areas: usize,
    
    /// Total number of mappings
    pub total_mappings: usize,
    
    /// Total number of synapses across all areas
    pub total_synapses: usize,
    
    /// Per-area synapse statistics
    pub per_area_stats: HashMap<String, SynapseStats>,
}



