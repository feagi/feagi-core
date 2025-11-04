// Neuron DTOs for V1 API
//
// These DTOs must match Python FastAPI response structures exactly.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Neuron information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "neuron_id": 12345,
    "cortical_area": "v1",
    "coordinates": [10, 15, 2],
    "membrane_potential": -70.0,
    "is_firing": false,
    "synaptic_inputs": 120,
    "synaptic_outputs": 85
}))]
pub struct NeuronInfoResponse {
    /// Global neuron ID
    pub neuron_id: u64,
    
    /// Cortical area this neuron belongs to
    pub cortical_area: String,
    
    /// 3D coordinates within the cortical area [x, y, z]
    pub coordinates: [u32; 3],
    
    /// Current membrane potential (mV)
    pub membrane_potential: f32,
    
    /// Whether the neuron is currently firing
    pub is_firing: bool,
    
    /// Number of incoming synapses
    pub synaptic_inputs: usize,
    
    /// Number of outgoing synapses
    pub synaptic_outputs: usize,
}

/// Create neuron request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateNeuronRequest {
    /// Cortical area ID
    pub cortical_area: String,
    
    /// 3D coordinates within the cortical area [x, y, z]
    pub coordinates: [u32; 3],
}

/// List neurons response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NeuronListResponse {
    /// List of neurons
    pub neurons: Vec<NeuronInfoResponse>,
    
    /// Total count
    pub total_count: usize,
    
    /// Cortical area (if filtered by area)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cortical_area: Option<String>,
}

/// Neuron count response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NeuronCountResponse {
    /// Cortical area ID
    pub cortical_area: String,
    
    /// Number of neurons in the area
    pub neuron_count: usize,
}




