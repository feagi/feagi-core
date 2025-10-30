/*!
Transport-agnostic Data Transfer Objects (DTOs).

These types define the stable contract between adapters and services.
They can be serialized to JSON, MessagePack, Protobuf, or any other format.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// NEURON DTOs
// ============================================================================

/// Information about a neuron
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronInfo {
    pub id: u64,
    pub cortical_id: String,
    pub cortical_idx: u32,
    pub coordinates: (u32, u32, u32),
    pub properties: HashMap<String, serde_json::Value>,
}

/// Parameters for creating a neuron
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNeuronParams {
    pub cortical_id: String,
    pub coordinates: (u32, u32, u32),
    pub properties: Option<HashMap<String, serde_json::Value>>,
}

// ============================================================================
// CORTICAL AREA DTOs
// ============================================================================

/// Information about a cortical area
/// This structure matches the Python FEAGI API for full compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorticalAreaInfo {
    pub cortical_id: String,
    pub cortical_idx: u32,
    pub name: String,
    pub dimensions: (usize, usize, usize),
    pub position: (i32, i32, i32),
    pub area_type: String, // "Sensory", "Motor", "Memory", "Custom"
    pub neuron_count: usize,
    pub synapse_count: usize,
    pub visible: bool,
    pub sub_group: Option<String>,
    pub neurons_per_voxel: u32,
    pub postsynaptic_current: f64,
    pub plasticity_constant: f64,
    pub degeneration: f64,
    pub psp_uniform_distribution: bool,
    pub firing_threshold_increment: f64,
    pub firing_threshold_limit: f64,
    pub consecutive_fire_count: u32,
    pub snooze_period: u32,
    pub refractory_period: u32,
    pub leak_coefficient: f64,
    pub leak_variability: f64,
    pub burst_engine_active: bool,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Parameters for creating a cortical area
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCorticalAreaParams {
    pub cortical_id: String,
    pub name: String,
    pub dimensions: (usize, usize, usize),
    pub position: (i32, i32, i32),
    pub area_type: String,
    pub visible: Option<bool>,
    pub sub_group: Option<String>,
    pub neurons_per_voxel: Option<u32>,
    pub postsynaptic_current: Option<f64>,
    pub plasticity_constant: Option<f64>,
    pub degeneration: Option<f64>,
    pub psp_uniform_distribution: Option<bool>,
    pub firing_threshold_increment: Option<f64>,
    pub firing_threshold_limit: Option<f64>,
    pub consecutive_fire_count: Option<u32>,
    pub snooze_period: Option<u32>,
    pub refractory_period: Option<u32>,
    pub leak_coefficient: Option<f64>,
    pub leak_variability: Option<f64>,
    pub burst_engine_active: Option<bool>,
    pub properties: Option<HashMap<String, serde_json::Value>>,
}

/// Parameters for updating a cortical area
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCorticalAreaParams {
    pub name: Option<String>,
    pub position: Option<(i32, i32, i32)>,
    pub dimensions: Option<(usize, usize, usize)>,
    pub area_type: Option<String>,
    pub visible: Option<bool>,
    pub postsynaptic_current: Option<f64>,
    pub plasticity_constant: Option<f64>,
    pub degeneration: Option<f64>,
    pub psp_uniform_distribution: Option<bool>,
    pub firing_threshold_increment: Option<f64>,
    pub firing_threshold_limit: Option<f64>,
    pub consecutive_fire_count: Option<u32>,
    pub snooze_period: Option<u32>,
    pub refractory_period: Option<u32>,
    pub leak_coefficient: Option<f64>,
    pub leak_variability: Option<f64>,
    pub burst_engine_active: Option<bool>,
}

// ============================================================================
// BRAIN REGION DTOs
// ============================================================================

/// Information about a brain region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainRegionInfo {
    pub region_id: String,
    pub name: String,
    pub region_type: String, // "Sensory", "Motor", "Association", "Custom"
    pub parent_id: Option<String>,
    pub cortical_areas: Vec<String>,
    pub child_regions: Vec<String>,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Parameters for creating a brain region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBrainRegionParams {
    pub region_id: String,
    pub name: String,
    pub region_type: String,
    pub parent_id: Option<String>,
    pub properties: Option<HashMap<String, serde_json::Value>>,
}

// ============================================================================
// GENOME DTOs
// ============================================================================

/// Information about a genome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeInfo {
    pub genome_id: String,
    pub genome_title: String,
    pub version: String,
    pub cortical_area_count: usize,
    pub brain_region_count: usize,
}

/// Parameters for loading a genome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadGenomeParams {
    pub json_str: String,
}

/// Parameters for saving a genome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveGenomeParams {
    pub genome_id: Option<String>,
    pub genome_title: Option<String>,
}

// ============================================================================
// CONNECTIVITY DTOs
// ============================================================================

/// Information about a synapse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynapseInfo {
    pub source_neuron: u64,
    pub target_neuron: u64,
    pub weight: u8,
    pub conductance: u8,
    pub synapse_type: String, // "Excitatory" or "Inhibitory"
}

/// Parameters for creating a synapse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSynapseParams {
    pub source_neuron: u64,
    pub target_neuron: u64,
    pub weight: u8,
    pub conductance: u8,
    pub synapse_type: String,
}

// ============================================================================
// ANALYTICS DTOs
// ============================================================================

/// Statistics for a cortical area
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorticalAreaStats {
    pub cortical_id: String,
    pub neuron_count: usize,
    pub synapse_count: usize,
    pub density: f32,
    pub populated: bool,
}

/// Connectivity statistics between two areas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectivityStats {
    pub source_area: String,
    pub target_area: String,
    pub synapse_count: usize,
    pub avg_weight: f32,
    pub excitatory_count: usize,
    pub inhibitory_count: usize,
}

/// System health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub burst_engine_active: bool,
    pub brain_readiness: bool,
    pub neuron_count: usize,
    pub cortical_area_count: usize,
    pub burst_count: u64,
}

// ============================================================================
// RUNTIME DTOs
// ============================================================================

/// Runtime status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStatus {
    /// Whether the burst engine is running
    pub is_running: bool,
    
    /// Whether the burst engine is paused
    pub is_paused: bool,
    
    /// Current burst frequency (Hz)
    pub frequency_hz: f64,
    
    /// Total burst count since start
    pub burst_count: u64,
    
    /// Current burst rate (bursts per second, measured)
    pub current_rate_hz: f64,
    
    /// Total neurons fired in last burst
    pub last_burst_neuron_count: usize,
    
    /// Average processing time per burst (milliseconds)
    pub avg_burst_time_ms: f64,
}

// ============================================================================
// SYSTEM SERVICE DTOs
// ============================================================================

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: String, // "healthy", "degraded", "unhealthy"
    pub message: Option<String>,
}

/// Overall system health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub overall_status: String, // "healthy", "degraded", "unhealthy"
    pub components: Vec<ComponentHealth>,
    pub timestamp: String, // ISO 8601 timestamp
}

/// Comprehensive system status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub is_initialized: bool,
    pub burst_engine_running: bool,
    pub burst_count: u64,
    pub neuron_count: usize,
    pub synapse_count: usize,
    pub cortical_area_count: usize,
    pub brain_region_count: usize,
    pub uptime_seconds: u64,
    pub current_burst_rate_hz: f64,
    pub avg_burst_time_ms: f64,
}

/// Version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub feagi_core_version: String,
    pub feagi_bdu_version: String,
    pub feagi_burst_engine_version: String,
    pub feagi_evo_version: String,
    pub feagi_types_version: String,
    pub build_timestamp: String,
    pub rust_version: String,
}

/// Runtime statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStats {
    pub total_bursts: u64,
    pub total_neurons_fired: u64,
    pub total_processing_time_ms: u64,
    pub avg_burst_time_ms: f64,
    pub avg_neurons_per_burst: f64,
    pub current_rate_hz: f64,
    pub peak_rate_hz: f64,
    pub uptime_seconds: u64,
}

/// Memory usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsage {
    pub npu_neurons_bytes: usize,
    pub npu_synapses_bytes: usize,
    pub npu_total_bytes: usize,
    pub connectome_metadata_bytes: usize,
    pub total_allocated_bytes: usize,
    pub system_total_bytes: usize,
    pub system_available_bytes: usize,
}

/// Capacity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityInfo {
    pub current_neurons: usize,
    pub max_neurons: usize,
    pub neuron_utilization_percent: f64,
    pub current_synapses: usize,
    pub max_synapses: usize,
    pub synapse_utilization_percent: f64,
    pub current_cortical_areas: usize,
    pub max_cortical_areas: usize,
}

