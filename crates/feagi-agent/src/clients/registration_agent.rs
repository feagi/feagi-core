//! Registration Agent
//!
//! The Registration Agent is a unique temporary agent whose only purpose is to initiate a
//! connection to FEAGI, authenticating itself, and returning connection and auth information back
//! to be used by an actual main purpose agent. For that reason this agent may actually be used
//! as a sub agent temporarily by such a main agent, then discarded when the required information
//! is retrieved

use std::collections::HashMap;
use feagi_io::FeagiNetworkError;
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

    pub async fn register(&mut self, registration_request: RegistrationRequest) -> Result<(SessionID, HashMap<AgentCapabilities, String>), FeagiAgentError> {
        let request_bytes: Vec<u8> = registration_request.into();

        self.io_client.send_request(&request_bytes).await
            .map_err(|e| FeagiAgentError::ConnectionFailed(e.to_string()))?;

        let response_bytes = self.io_client.get_response().await
            .map_err(|e| FeagiAgentError::ConnectionFailed(e.to_string()))?;

        let response: RegistrationResponse = response_bytes.try_into()
            .map_err(|e| FeagiAgentError::GeneralFailure("Unable to parse response!".to_string()));

        match response {
            RegistrationResponse::FailedInvalidRequest => return Err(FeagiAgentError::ConnectionFailed(
                "Failed invalid request!".to_string()
            )),
            RegistrationResponse::FailedInvalidAuth => return Err(FeagiAgentError::ConnectionFailed(
                "Failed invalid auth!".to_string()
            )),
            RegistrationResponse::Success(session_id, mapped_capabilities) => {
                return Ok((session_id, mapped_capabilities))
            }
        }
    }






}