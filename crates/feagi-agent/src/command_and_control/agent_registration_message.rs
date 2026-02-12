use crate::{AgentCapabilities, AgentDescriptor, AuthToken, FeagiApiVersion};
use feagi_io::traits_and_enums::shared::{
    TransportProtocolEndpoint, TransportProtocolImplementation,
};
use feagi_io::AgentID;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRegistrationMessage {
    ClientRequestRegistration(RegistrationRequest),
    ServerRespondsRegistration(RegistrationResponse),
    /// Client-initiated request to tear down an active registration/session.
    ///
    /// This supports voluntary deregistration (graceful disconnect) where the
    /// client explicitly asks FEAGI to release all associated resources.
    ClientRequestDeregistration(DeregistrationRequest),
    /// Server response to a deregistration request.
    ///
    /// The server responds with either `Success` (resources released) or
    /// `NotRegistered` if the session was already absent.
    ServerRespondsDeregistration(DeregistrationResponse),
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
            api_version: FeagiApiVersion::get_current_api_version(),
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
    Success(
        AgentID,
        HashMap<AgentCapabilities, TransportProtocolEndpoint>,
    ),
}

//endregion

//region Deregistration Request/Response

/// Deregistration request sent by a registered client.
///
/// The optional `reason` is intended for observability and diagnostics and
/// does not affect deregistration semantics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeregistrationRequest {
    reason: Option<String>,
}

impl DeregistrationRequest {
    /// Create a request with an optional reason message.
    pub fn new(reason: Option<String>) -> Self {
        Self { reason }
    }

    /// Optional human-readable reason provided by the client.
    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }
}

/// Deregistration response from the server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeregistrationResponse {
    Success,
    NotRegistered,
}

//endregion

#[cfg(test)]
mod tests {
    use super::{AgentRegistrationMessage, DeregistrationRequest, DeregistrationResponse};

    #[test]
    fn deregistration_request_round_trip_serialization_preserves_reason() {
        let request = AgentRegistrationMessage::ClientRequestDeregistration(
            DeregistrationRequest::new(Some("shutdown".to_string())),
        );
        let encoded =
            serde_json::to_string(&request).expect("deregistration request should serialize");
        let decoded: AgentRegistrationMessage =
            serde_json::from_str(&encoded).expect("deregistration request should deserialize");
        assert_eq!(request, decoded);
    }

    #[test]
    fn deregistration_response_round_trip_serialization_preserves_variant() {
        let response = AgentRegistrationMessage::ServerRespondsDeregistration(
            DeregistrationResponse::NotRegistered,
        );
        let encoded =
            serde_json::to_string(&response).expect("deregistration response should serialize");
        let decoded: AgentRegistrationMessage =
            serde_json::from_str(&encoded).expect("deregistration response should deserialize");
        assert_eq!(response, decoded);
    }
}
