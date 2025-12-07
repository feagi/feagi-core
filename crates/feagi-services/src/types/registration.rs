// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Registration DTOs - Shared between feagi-services and feagi-pns

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Registration request from agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationRequest {
    pub agent_id: String,
    pub agent_type: String,
    pub capabilities: serde_json::Value, // Flexible JSON for different formats
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chosen_transport: Option<String>, // Agent reports which transport it chose: "zmq", "websocket", "shm", etc.
}

/// Transport configuration for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    pub transport_type: String, // "zmq" or "websocket"
    pub enabled: bool,
    pub ports: HashMap<String, u16>,
    pub host: String,
}

/// Status of a cortical area during registration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AreaStatus {
    /// Area already existed in genome
    Existing,
    /// Area was auto-created during registration
    Created,
    /// Area missing and auto-create disabled
    Missing,
    /// Error during creation/check
    Error,
}

/// Status information for a cortical area
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorticalAreaStatus {
    pub area_name: String,
    pub cortical_id: String,  // base64 encoded
    pub status: AreaStatus,
    pub dimensions: Option<(usize, usize, usize)>,
    pub message: Option<String>,  // Error message or creation confirmation
}

/// Cortical area availability information for agent registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorticalAreaAvailability {
    /// Required IPU areas (from vision/sensory capabilities)
    pub required_ipu_areas: Vec<CorticalAreaStatus>,
    /// Required OPU areas (from motor capabilities)
    pub required_opu_areas: Vec<CorticalAreaStatus>,
}

/// Registration response to agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResponse {
    pub status: String,
    pub message: Option<String>,
    pub shm_paths: Option<HashMap<String, String>>, // capability_type -> shm_path
    pub zmq_ports: Option<HashMap<String, u16>>,    // ZMQ port mappings
    pub transports: Option<Vec<TransportConfig>>,   // Available transports with their configs
    pub recommended_transport: Option<String>,      // "zmq" or "websocket"
    /// Cortical area availability status for agent operations
    pub cortical_areas: CorticalAreaAvailability,
}

