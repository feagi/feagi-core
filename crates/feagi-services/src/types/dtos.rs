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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorticalAreaInfo {
    pub cortical_id: String,
    pub cortical_idx: u32,
    pub name: String,
    pub dimensions: (usize, usize, usize),
    pub position: (i32, i32, i32),
    pub area_type: String, // "Sensory", "Motor", "Memory", "Custom"
    pub neuron_count: usize,
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
    pub properties: Option<HashMap<String, serde_json::Value>>,
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

