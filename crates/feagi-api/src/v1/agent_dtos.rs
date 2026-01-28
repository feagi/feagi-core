// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Agent API DTOs - Exact port from Python schemas

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Agent registration request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentRegistrationRequest {
    /// Type of agent (e.g., "brain_visualizer", "video_agent")
    pub agent_type: String,

    /// Unique identifier for the agent
    pub agent_id: String,

    /// Port the agent is listening on for data
    pub agent_data_port: u16,

    /// Version of the agent software
    pub agent_version: String,

    /// Version of the controller
    pub controller_version: String,

    /// Agent capabilities (sensory, motor, visualization, etc.)
    pub capabilities: HashMap<String, serde_json::Value>,

    /// Optional: Agent IP address (extracted from request if not provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_ip: Option<String>,

    /// Optional: Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,

    /// Optional: Transport the agent chose to use ("zmq", "websocket", "shm", etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chosen_transport: Option<String>,
}

/// Transport configuration for an agent (from PNS)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TransportConfig {
    pub transport_type: String,
    pub enabled: bool,
    pub ports: HashMap<String, u16>,
    pub host: String,
}

/// Agent registration response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentRegistrationResponse {
    pub status: String,
    pub message: String,
    pub success: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport: Option<HashMap<String, serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rates: Option<HashMap<String, HashMap<String, f64>>>,

    // FEAGI 2.0: Multi-transport support
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transports: Option<Vec<TransportConfig>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended_transport: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub shm_paths: Option<HashMap<String, String>>,

    /// Cortical area availability status for agent operations
    pub cortical_areas: serde_json::Value,
}

/// Heartbeat request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HeartbeatRequest {
    pub agent_id: String,
}

/// Heartbeat response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HeartbeatResponse {
    pub message: String,
    pub success: bool,
}

/// Agent list response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentListResponse {
    /// List of agent IDs
    #[serde(flatten)]
    pub agent_ids: Vec<String>,
}

/// Agent properties response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentPropertiesResponse {
    pub agent_type: String,
    pub agent_ip: String,
    pub agent_data_port: u16,
    pub agent_router_address: String,
    pub agent_version: String,
    pub controller_version: String,
    pub capabilities: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chosen_transport: Option<String>,
}

/// Agent capabilities summary (optionally includes device registrations)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentCapabilitiesSummary {
    pub capabilities: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_registrations: Option<serde_json::Value>,
}

/// Query parameters for bulk agent capabilities lookup
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentCapabilitiesAllQuery {
    /// Filter by agent type (exact match)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,
    /// Filter by capability key(s), comma-separated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability: Option<String>,
    /// Include device registration payloads per agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_device_registrations: Option<bool>,
}

/// Agent deregistration request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentDeregistrationRequest {
    pub agent_id: String,
}

/// Success response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SuccessResponse {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success: Option<bool>,
}

/// Manual stimulation request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ManualStimulationRequest {
    /// Map of cortical area IDs to lists of coordinates [[x, y, z], ...]
    pub stimulation_payload: HashMap<String, Vec<Vec<i32>>>,
}

/// Manual stimulation response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ManualStimulationResponse {
    pub success: bool,
    pub total_coordinates: usize,
    pub successful_areas: usize,
    pub failed_areas: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Device registration export response
///
/// Contains the complete device registration configuration including
/// sensor and motor device registrations, encoder/decoder properties,
/// and feedback configurations.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeviceRegistrationExportResponse {
    /// Device registration configuration as JSON
    /// This matches the format from ConnectorAgent::get_device_registration_json
    pub device_registrations: serde_json::Value,
    /// Agent ID this configuration belongs to
    pub agent_id: String,
}

/// Device registration import request
///
/// Contains the device registration configuration to import.
/// This will replace all existing device registrations for the agent.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeviceRegistrationImportRequest {
    /// Device registration configuration as JSON
    /// This matches the format expected by ConnectorAgent::set_device_registrations_from_json
    pub device_registrations: serde_json::Value,
}

/// Device registration import response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeviceRegistrationImportResponse {
    pub success: bool,
    pub message: String,
    pub agent_id: String,
}
