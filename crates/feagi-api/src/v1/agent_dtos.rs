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

    /// Optional: Auth token (base64, required by unified registration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,

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
    pub agent_name: String,
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
    pub agent_name: String,
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
    /// Legacy optional field. Server applies authoritative force-fire semantics for manual
    /// stimulation (resolved neurons are always placed in the fire queue); this field may be ignored.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    /// Membrane potential the forced neurons fire with, one value for the whole request.
    /// Optional; 100.0 when absent. Must be finite and greater than 0 (400 otherwise).
    ///
    /// This is a probe: it bypasses threshold and refractory period. A neuron that already
    /// fired naturally in the same burst keeps its own potential. Mechanisms that read firing
    /// potential (for example MP-driven PSP or graded modulator drivers) see this value.
    /// A server that predates this field ignores it; check `applied_potential` in the response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub potential: Option<f32>,
}

/// Manual stimulation response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ManualStimulationResponse {
    pub success: bool,
    pub total_coordinates: usize,
    pub requested_coordinates: usize,
    pub matched_coordinates: usize,
    pub unique_neuron_ids: usize,
    pub mode: String,
    pub successful_areas: usize,
    pub failed_areas: Vec<String>,
    /// Potential the forced neurons actually fired with (100.0 when the request gave none).
    pub applied_potential: f32,
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

/// One cortical area a device registration declares, next to the live area it maps to.
///
/// `declared_*` is what the agent's stored device registrations resolve to (the same
/// derivation auto-create uses); `live_*` is the connectome's current area, `None` when
/// the area does not exist yet. `matches` is true only when dimensions, per-device
/// dimensions and device count are all equal.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RegistrationAreaDiff {
    /// Base64 cortical ID.
    pub cortical_id: String,
    /// Live area name when it exists, otherwise the name registration would assign.
    pub name: String,
    /// "ipu" (sensory) or "opu" (motor).
    pub direction: String,
    pub declared_dimensions: [usize; 3],
    pub declared_per_device_dimensions: [usize; 3],
    pub declared_device_count: usize,
    pub live_dimensions: Option<[usize; 3]>,
    pub live_per_device_dimensions: Option<[usize; 3]>,
    pub live_device_count: Option<usize>,
    pub matches: bool,
}

/// Registration diff for one agent.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RegistrationDiffAgent {
    /// Base64 agent (session) ID.
    pub agent_id: String,
    pub areas: Vec<RegistrationAreaDiff>,
}

/// Response of `GET /v1/agent/registration_diff`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RegistrationDiffResponse {
    pub agents: Vec<RegistrationDiffAgent>,
}

/// Query parameters for `GET /v1/agent/registration_diff`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RegistrationDiffQuery {
    /// Restrict the diff to this base64 agent ID; all agents when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
}

#[cfg(test)]
mod manual_stimulation_potential_tests {
    use super::*;

    #[test]
    fn request_without_potential_deserialises_to_none() {
        let r: ManualStimulationRequest =
            serde_json::from_str(r#"{"stimulation_payload":{}}"#).unwrap();
        assert_eq!(r.potential, None);
    }

    #[test]
    fn request_with_potential_keeps_value() {
        let r: ManualStimulationRequest =
            serde_json::from_str(r#"{"stimulation_payload":{},"potential":0.25}"#).unwrap();
        assert_eq!(r.potential, Some(0.25));
    }

    #[test]
    fn response_carries_applied_potential() {
        let r = ManualStimulationResponse {
            success: true,
            total_coordinates: 1,
            requested_coordinates: 1,
            matched_coordinates: 1,
            unique_neuron_ids: 1,
            mode: "force_fire".into(),
            successful_areas: 1,
            failed_areas: vec![],
            applied_potential: 0.25,
            error: None,
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["applied_potential"], 0.25);
    }
}
