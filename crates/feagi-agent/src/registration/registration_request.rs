use serde::{Deserialize, Serialize};
use feagi_io::core::protocol_implementations::ProtocolImplementation;
use crate::registration::{AgentDescriptor};
use crate::registration::common::{AgentCapabilities, AuthToken};

/// A request from an agent to register with FEAGI.
///
/// Contains the agent's descriptor, authentication token, requested capabilities,
/// and optionally device_registrations (for auto IPU/OPU creation when enabled).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegistrationRequest {
    agent_descriptor: AgentDescriptor,
    auth_token: AuthToken,
    requested_capabilities: Vec<AgentCapabilities>,
    connection_protocol: ProtocolImplementation,
    /// Optional device_registrations JSON; when present and server config allows,
    /// triggers auto-creation of missing IPU/OPU cortical areas (REST and ZMQ/WS).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    device_registrations: Option<serde_json::Value>,
}

impl RegistrationRequest {
    /// Create a new registration request.
    ///
    /// # Arguments
    /// * `agent_descriptor` - Information identifying the agent
    /// * `auth_token` - Authentication token for secure access
    /// * `requested_capabilities` - List of capabilities the agent is requesting
    /// * `connection_protocol` - The protocol the agent wants to use for communication
    pub fn new(
        agent_descriptor: AgentDescriptor,
        auth_token: AuthToken,
        requested_capabilities: Vec<AgentCapabilities>, // TODO hashset?
        connection_protocol: ProtocolImplementation,
    ) -> Self {
        Self {
            agent_descriptor,
            auth_token,
            requested_capabilities,
            connection_protocol,
            device_registrations: None,
        }
    }

    /// Set device_registrations for this request.
    /// When the server has auto_create_missing_cortical_areas enabled, sending this over ZMQ/WS
    /// triggers the same auto IPU/OPU creation as REST registration.
    pub fn with_device_registrations(mut self, value: Option<serde_json::Value>) -> Self {
        self.device_registrations = value;
        self
    }

    /// Get device_registrations if present.
    pub fn device_registrations(&self) -> Option<&serde_json::Value> {
        self.device_registrations.as_ref()
    }

    /// Get the agent descriptor.
    pub fn agent_descriptor(&self) -> &AgentDescriptor {
        &self.agent_descriptor
    }

    /// Get the authentication token.
    pub fn auth_token(&self) -> &AuthToken {
        &self.auth_token
    }

    /// Get the requested capabilities.
    pub fn requested_capabilities(&self) -> &[AgentCapabilities] {
        &self.requested_capabilities
    }

    /// Get the connection protocol.
    pub fn connection_protocol(&self) -> &ProtocolImplementation {
        &self.connection_protocol
    }
}
