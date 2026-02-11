use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use feagi_io::AgentID;
use feagi_io::traits_and_enums::shared::{TransportProtocolEndpoint, TransportProtocolImplementation};
use crate::{AgentCapabilities, AgentDescriptor, AuthToken, FeagiApiVersion};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRegistrationMessage {
    ClientRequestRegistration(RegistrationRequest),
    ServerRespondsRegistration(RegistrationResponse),
}

//region Registration Request

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegistrationRequest {
    agent_descriptor: AgentDescriptor,
    auth_token: AuthToken,
    requested_capabilities: Vec<AgentCapabilities>,
    connection_protocol: TransportProtocolImplementation,
    api_version: FeagiApiVersion,
}

impl RegistrationRequest {

    /// No request should be bigger than this many bytes. If so, assume something malicious is going on
    pub const MAX_REQUEST_SIZE: usize = 1024;

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
        requested_capabilities: Vec<AgentCapabilities>,
        connection_protocol: TransportProtocolImplementation,
    ) -> Self {
        Self {
            agent_descriptor,
            auth_token,
            requested_capabilities,
            connection_protocol,
            api_version: FeagiApiVersion::get_current_api_version()
        }
    }

    /// Get the reported API version
    pub fn api_version(&self) -> &FeagiApiVersion {
        &self.api_version
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
    pub fn connection_protocol(&self) -> &TransportProtocolImplementation {
        &self.connection_protocol
    }
}

//endregion

//region Registration Response

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistrationResponse {
    FailedInvalidRequest, // This may not be sent back if the server ignores bad data
    FailedInvalidAuth, // Usually the auth token, may be the agent too. Server may not send this if configured to ignore invalid auth
    AlreadyRegistered,
    Success(AgentID, HashMap<AgentCapabilities, TransportProtocolEndpoint>),
}

//endregion