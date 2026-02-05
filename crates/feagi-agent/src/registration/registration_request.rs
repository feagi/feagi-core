use serde::{Deserialize, Serialize};
use feagi_io::shared::TransportProtocolImplementation;
use feagi_serialization::FeagiByteContainer;
use feagi_structures::{FeagiDataError, FeagiJSON};
use crate::feagi_agent_server_error::FeagiAgentServerError;
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
    connection_protocol: TransportProtocolImplementation,
    /// Optional device_registrations JSON; when present and server config allows,
    /// triggers auto-creation of missing IPU/OPU cortical areas (REST and ZMQ/WS).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    device_registrations: Option<serde_json::Value>, // TODO this must NOT be here. This request is for things like BV as well and is completely irrelevant here
}

impl RegistrationRequest {

    pub const MAX_REQUEST_SIZE: usize = 999999;  // Any request more than this many bytes will be ignored // TODO set to a high value of now due to the existence of device_registrations. This should be set to a much lower value!

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
        connection_protocol: TransportProtocolImplementation,
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
    pub fn connection_protocol(&self) -> &TransportProtocolImplementation {
        &self.connection_protocol
    }
}

impl TryFrom<&FeagiByteContainer> for RegistrationRequest {
    type Error = FeagiAgentServerError;
    fn try_from(value: &FeagiByteContainer) -> Result<Self, Self::Error> {
        let serialized_data = value.try_create_new_struct_from_index(0.into())?;
        let feagi_json: FeagiJSON = serialized_data.try_into()?;
        let json = feagi_json.borrow_json_value().clone();
        serde_json::from_value(json).map_err(|err| FeagiAgentServerError::UnableToDecodeReceivedData(err.to_string()))
    }
}
