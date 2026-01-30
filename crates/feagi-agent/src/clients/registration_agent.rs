//! Registration Agent
//!
//! The Registration Agent is a unique temporary agent whose only purpose is to initiate a
//! connection to FEAGI, authenticating itself, and returning connection and auth information back
//! to be used by an actual main purpose agent. For that reason this agent may actually be used
//! as a sub agent temporarily by such a main agent, then discarded when the required information
//! is retrieved

use std::collections::HashMap;
use feagi_io::traits_and_enums::client::FeagiClientRequester;
use feagi_serialization::SessionID;
use crate::FeagiAgentError;
use crate::registration::{AgentCapabilities, RegistrationRequest, RegistrationResponse};
// TODO registration requests specifies protocol, we need to make sure it matches with the FeagiClientRequester

pub struct RegistrationAgent {
    io_client: Box<dyn FeagiClientRequester>
}

impl RegistrationAgent {
    pub fn new(io_client: Box<dyn FeagiClientRequester>) -> Self {
        Self { io_client }
    }

    /// Tries to register with given settings. If successful, returns the feagi given Session ID
    /// and a hashmap of endpoints to the agent capability. This agent can typically be disposed
    /// after. Otherwise returns an error
    pub async fn try_register(&mut self, registration_request: RegistrationRequest) -> Result<(SessionID, HashMap<AgentCapabilities, String>), FeagiAgentError> {
        // Serialize request to JSON bytes
        let request_bytes = serde_json::to_vec(&registration_request)
            .map_err(|e| FeagiAgentError::GeneralFailure(format!("Failed to serialize request: {}", e)))?;

        // Send the request
        self.io_client.send_request(&request_bytes).await
            .map_err(|e| FeagiAgentError::ConnectionFailed(e.to_string()))?;

        // Wait for response
        let response_bytes = self.io_client.get_response().await
            .map_err(|e| FeagiAgentError::ConnectionFailed(e.to_string()))?;

        // Deserialize response from JSON
        let response: RegistrationResponse = serde_json::from_slice(&response_bytes)
            .map_err(|e| FeagiAgentError::GeneralFailure(format!("Unable to parse response: {}", e)))?;

        match response {
            RegistrationResponse::FailedInvalidRequest => Err(FeagiAgentError::ConnectionFailed(
                "Server rejected request as invalid!".to_string()
            )),
            RegistrationResponse::FailedInvalidAuth => Err(FeagiAgentError::ConnectionFailed(
                "Server rejected authentication!".to_string()
            )),
            RegistrationResponse::AlreadyRegistered => Err(FeagiAgentError::ConnectionFailed(
                "Agent is already registered with this server!".to_string()
            )),
            RegistrationResponse::Success(session_id, mapped_capabilities) => {
                Ok((session_id, mapped_capabilities))
            }
        }
    }

    /// Disconnect from the registration server.
    /// Call this after registration is complete or if registration fails.
    pub async fn disconnect(&mut self) -> Result<(), FeagiAgentError> {
        self.io_client.disconnect().await
            .map_err(|e| FeagiAgentError::ConnectionFailed(e.to_string()))
    }



}