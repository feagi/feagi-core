//! Physiology: genome physiology parameters (timestep, neuron aging, sleep triggers, ...).
//! Converted from the old `physiology` endpoints.

use feagi_basis::feagi_basis_proc_common::define_request_message_category;
use std::collections::HashMap;

/// Physiology parameters of the active genome
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PhysiologyParameters {
    /// Simulation timestep in seconds
    pub simulation_timestep: f64,
    /// Maximum neuron age
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_age: Option<u32>,
    /// Evolution burst count
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evolution_burst_count: Option<u64>,
    /// IPU idle threshold
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ipu_idle_threshold: Option<u32>,
    /// Plasticity queue depth
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plasticity_queue_depth: Option<u32>,
    /// Lifespan management interval
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifespan_mgmt_interval: Option<u32>,
    /// Sleep trigger inactivity window
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sleep_trigger_inactivity_window: Option<u32>,
    /// Sleep trigger neural activity max
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sleep_trigger_neural_activity_max: Option<f64>,
    /// Quantization precision for numeric values: "fp32" (default), "fp16", "int8"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantization_precision: Option<String>,
}

define_request_message_category! {
    category_name: "Physiology",
    base_path: "v1",
    category_description: "Read and update physiology parameters in the active genome",
    read: {
        "Physiology": {
            path: "physiology",
            description: "Get current physiology parameters including simulation timestep and neural aging settings",
            alloc_only: true,
            response: {
                physiology: PhysiologyParameters, "Current physiology parameters",
            },
        },
    },
    edit: {
        "Physiology": {
            path: "physiology",
            description: "Update physiology parameters in the active genome including timestep and aging settings",
            alloc_only: true,
            payload: {
                physiology: HashMap<String, serde_json::Value>, "Parameters to update, keyed by name; unknown keys are ignored",
            },
            response: {
                success: bool, "False if no valid parameters were provided",
                updated: HashMap<String, serde_json::Value>, "The parameters that were accepted",
                message: Option<String>, "Explanation when nothing was updated",
            },
        },
    },
}

// NOTES:
// Old path was "/v1/physiology/" (category root), so base_path is "v1" and messages use "physiology".
// Edit payload/response `physiology`/`updated` are untyped Value maps (whitelist of PhysiologyParameters keys).
// Only simulation_timestep was actually applied by the old PUT handler; other keys were TODO.
// Read response typed from PhysiologyParameters DTO; old handler built it as untyped JSON.
