use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthCheckMessage {
    FeagiHealthCheckRequest,
    FeagiHealthCheckResponse(HealthCheckResponse),
}

// TODO: This is a copy of "crates/feagi-api/src/endpoints/system.rs/HealthCheckResponse" that is stored here for now. We should consider moving this potentially
// TODO: I see a lot of generic types here, especially string keys. Maybe we need somehting better?
#[allow(non_snake_case)] // Field name matches Python API for compatibility
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub burst_engine: bool,
    pub connected_agents: Option<i32>,
    pub influxdb_availability: bool,
    pub neuron_count_max: i64,
    pub synapse_count_max: i64,
    pub latest_changes_saved_externally: bool,
    pub genome_availability: bool,
    pub genome_validity: Option<bool>,
    pub brain_readiness: bool,
    pub feagi_session: Option<i64>,
    pub fitness: Option<f64>,
    pub cortical_area_count: Option<i32>,
    pub neuron_count: Option<i64>,
    pub memory_neuron_count: Option<i64>,
    pub regular_neuron_count: Option<i64>,
    pub synapse_count: Option<i64>,
    pub estimated_brain_size_in_MB: Option<f64>,
    pub genome_num: Option<i32>,
    pub genome_timestamp: Option<i64>,
    pub simulation_timestep: Option<f64>,
    pub memory_area_stats: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    pub amalgamation_pending: Option<HashMap<String, serde_json::Value>>,
    /// Hash of brain regions (hierarchy, membership, and properties)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brain_regions_hash: Option<u64>,
    /// Hash of cortical areas and properties (excluding mappings)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cortical_areas_hash: Option<u64>,
    /// Hash of brain geometry (area positions/dimensions and 2D coordinates)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brain_geometry_hash: Option<u64>,
    /// Hash of morphology registry
    #[serde(skip_serializing_if = "Option::is_none")]
    pub morphologies_hash: Option<u64>,
    /// Hash of cortical mappings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cortical_mappings_hash: Option<u64>,
    /// Hash of agent data (ids, capabilities, connection properties)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_data_hash: Option<u64>,
    /// Root brain region ID (UUID string) for O(1) root lookup
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brain_regions_root: Option<String>,
    // /// Fatigue information (index, active state, and breakdown of contributing elements)
    //#[serde(skip_serializing_if = "Option::is_none")]
    //pub fatigue: Option<FatigueInfo>,
}
