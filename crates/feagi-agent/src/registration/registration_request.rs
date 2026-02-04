use serde::{Deserialize, Serialize};
use feagi_io::core::protocol_implementations::ProtocolImplementation;
use crate::registration::{AgentDescriptor};
use crate::registration::common::{AgentCapabilities, AuthToken};

/// A request from an agent to register with FEAGI.
///
/// Contains the agent's descriptor, authentication token, and requested capabilities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegistrationRequest {
    agent_descriptor: AgentDescriptor,
    auth_token: AuthToken,
    requested_capabilities: Vec<AgentCapabilities>,
    connection_protocol: ProtocolImplementation
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
        }
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
