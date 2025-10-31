// Cortical Area DTOs for V1 API
//
// These DTOs must match Python FastAPI response structures exactly.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Cortical area information (summary)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "cortical_id": "v1",
    "cortical_name": "Primary Visual Cortex",
    "cortical_group": "vision",
    "coordinates_3d": {
        "x": 0,
        "y": 0,
        "z": 0
    },
    "cortical_dimensions": {
        "x": 10,
        "y": 10,
        "z": 10
    },
    "neuron_count": 1000,
    "cortical_visibility": true
}))]
pub struct CorticalAreaSummary {
    /// Cortical area ID
    pub cortical_id: String,
    
    /// Human-readable name
    pub cortical_name: String,
    
    /// Functional group (vision, motor, memory, etc.)
    pub cortical_group: String,
    
    /// 3D coordinates in brain space
    pub coordinates_3d: Coordinates3D,
    
    /// Dimensions of the cortical area
    pub cortical_dimensions: Dimensions3D,
    
    /// Number of neurons in this area
    pub neuron_count: usize,
    
    /// Is this area visible in visualization?
    pub cortical_visibility: bool,
}

/// Detailed cortical area information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "cortical_id": "v1",
    "cortical_name": "Primary Visual Cortex",
    "cortical_group": "vision",
    "coordinates_3d": {
        "x": 0,
        "y": 0,
        "z": 0
    },
    "cortical_dimensions": {
        "x": 10,
        "y": 10,
        "z": 10
    },
    "neuron_count": 1000,
    "synapse_count": 5000,
    "cortical_visibility": true,
    "cortical_sub_group_name": "visual_input",
    "cortical_neuron_per_vox_count": 1,
    "postsynaptic_current": 1.0,
    "plasticity_constant": 0.5,
    "degeneration": 0.0,
    "psp_uniform_distribution": false,
    "firing_threshold_increment": 0.1,
    "firing_threshold_limit": 10.0,
    "consecutive_fire_count": 3,
    "snooze_period": 5,
    "refractory_period": 2,
    "leak_coefficient": 0.01,
    "leak_variability": 0.0,
    "burst_engine_activation": true
}))]
pub struct CorticalAreaDetail {
    /// Cortical area ID
    pub cortical_id: String,
    
    /// Human-readable name
    pub cortical_name: String,
    
    /// Functional group
    pub cortical_group: String,
    
    /// 3D coordinates
    pub coordinates_3d: Coordinates3D,
    
    /// Dimensions
    pub cortical_dimensions: Dimensions3D,
    
    /// Number of neurons
    pub neuron_count: usize,
    
    /// Number of synapses
    pub synapse_count: usize,
    
    /// Visibility flag
    pub cortical_visibility: bool,
    
    /// Sub-group name
    pub cortical_sub_group_name: String,
    
    /// Neurons per voxel
    pub cortical_neuron_per_vox_count: u32,
    
    /// Postsynaptic current
    pub postsynaptic_current: f64,
    
    /// Plasticity constant
    pub plasticity_constant: f64,
    
    /// Degeneration rate
    pub degeneration: f64,
    
    /// PSP uniform distribution
    pub psp_uniform_distribution: bool,
    
    /// Firing threshold increment
    pub firing_threshold_increment: f64,
    
    /// Firing threshold limit
    pub firing_threshold_limit: f64,
    
    /// Consecutive fire count
    pub consecutive_fire_count: u32,
    
    /// Snooze period
    pub snooze_period: u32,
    
    /// Refractory period
    pub refractory_period: u32,
    
    /// Leak coefficient
    pub leak_coefficient: f64,
    
    /// Leak variability
    pub leak_variability: f64,
    
    /// Burst engine activation
    pub burst_engine_activation: bool,
}

/// Create cortical area request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateCorticalAreaRequest {
    /// Cortical area ID
    pub cortical_id: String,
    
    /// Human-readable name
    pub cortical_name: String,
    
    /// Functional group
    pub cortical_group: String,
    
    /// 3D coordinates
    pub coordinates_3d: Coordinates3D,
    
    /// Dimensions
    pub cortical_dimensions: Dimensions3D,
    
    /// Optional: Visibility (default: true)
    #[serde(default = "default_visibility")]
    pub cortical_visibility: bool,
    
    /// Optional: Sub-group name
    #[serde(default)]
    pub cortical_sub_group_name: Option<String>,
    
    /// Optional: Neurons per voxel (default: 1)
    #[serde(default = "default_neurons_per_vox")]
    pub cortical_neuron_per_vox_count: u32,
    
    /// Optional: Postsynaptic current (default: 1.0)
    #[serde(default = "default_postsynaptic_current")]
    pub postsynaptic_current: f64,
    
    /// Optional: Plasticity constant (default: 0.5)
    #[serde(default = "default_plasticity_constant")]
    pub plasticity_constant: f64,
}

/// Update cortical area request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateCorticalAreaRequest {
    /// Optional: New name
    pub cortical_name: Option<String>,
    
    /// Optional: New group
    pub cortical_group: Option<String>,
    
    /// Optional: New coordinates
    pub coordinates_3d: Option<Coordinates3D>,
    
    /// Optional: New dimensions
    pub cortical_dimensions: Option<Dimensions3D>,
    
    /// Optional: Visibility
    pub cortical_visibility: Option<bool>,
    
    /// Optional: Postsynaptic current
    pub postsynaptic_current: Option<f64>,
    
    /// Optional: Plasticity constant
    pub plasticity_constant: Option<f64>,
}

/// 3D coordinates
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub struct Coordinates3D {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// 3D dimensions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub struct Dimensions3D {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

/// List cortical areas response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CorticalAreaListResponse {
    /// List of cortical areas
    pub cortical_areas: Vec<CorticalAreaSummary>,
    
    /// Total count
    pub total_count: usize,
}

// Default values for optional fields
fn default_visibility() -> bool {
    true
}

fn default_neurons_per_vox() -> u32 {
    1
}

fn default_postsynaptic_current() -> f64 {
    1.0
}

fn default_plasticity_constant() -> f64 {
    0.5
}




