//! Registration Agent
//!
//! Connects to the FEAGI registration endpoint (ZMQ or WS), sends a registration request,
//! and returns session_id and capability endpoints. Use this once; then connect to the
//! returned data endpoints (sensory, motor, visualization). Disconnect after registration
//! so the registration channel is not held open.

use std::collections::HashMap;
use feagi_io::shared::FeagiEndpointState;
use feagi_io::traits_and_enums::client::FeagiClientRequester;
use feagi_serialization::SessionID;

use crate::FeagiAgentClientError;
use crate::registration::{AgentCapabilities, RegistrationRequest, RegistrationResponse};

pub struct RegistrationAgent {
    io_client: Box<dyn FeagiClientRequester>,
}

impl RegistrationAgent {
    pub fn new(io_client: Box<dyn FeagiClientRequester>) -> Self {
        Self { io_client }
    }

    /// Connect to the registration endpoint. Call before `try_register`.
    pub fn connect(&mut self) -> Result<(), FeagiAgentClientError> {
        self.io_client
            .request_connect()
            .map_err(|e| FeagiAgentClientError::ConnectionFailed(e.to_string()))?;
        while !matches!(self.io_client.poll(), FeagiEndpointState::ActiveWaiting) {
            if matches!(self.io_client.poll(), FeagiEndpointState::Errored(_)) {
                return Err(FeagiAgentClientError::ConnectionFailed(
                    "Registration endpoint connection failed".to_string(),
                ));
            }
        }
        Ok(())
    }

    /// Send registration request and wait for response. Returns session_id and endpoints.
    /// Call `connect()` first; call `disconnect()` after to release the registration channel.
    pub fn try_register(
        &mut self,
        registration_request: RegistrationRequest,
    ) -> Result<(SessionID, HashMap<AgentCapabilities, String>), FeagiAgentClientError> {
        let request_bytes = serde_json::to_vec(&registration_request)
            .map_err(|e| FeagiAgentClientError::UnableToSendData(format!("Failed to serialize request: {}", e)))?;

        self.io_client
            .publish_request(&request_bytes)
            .map_err(|e| FeagiAgentClientError::ConnectionFailed(e.to_string()))?;

        loop {
            match self.io_client.poll() {
                FeagiEndpointState::ActiveHasData => {
                    let response_slice = self
                        .io_client
                        .consume_retrieved_response()
                        .map_err(|e| FeagiAgentClientError::ConnectionFailed(e.to_string()))?;
                    let response_bytes = response_slice.to_vec();

                    let response: RegistrationResponse = serde_json::from_slice(&response_bytes)
                        .map_err(|e| FeagiAgentClientError::UnableToDecodeReceivedData(format!("Unable to parse response: {}", e)))?;

                    return match response {
                        RegistrationResponse::FailedInvalidRequest => Err(
                            FeagiAgentClientError::ConnectionFailed(
                                "Server rejected request as invalid!".to_string(),
                            ),
                        ),
                        RegistrationResponse::FailedInvalidAuth => Err(
                            FeagiAgentClientError::ConnectionFailed(
                                "Server rejected authentication!".to_string(),
                            ),
                        ),
                        RegistrationResponse::AlreadyRegistered => Err(
                            FeagiAgentClientError::ConnectionFailed(
                                "Agent is already registered with this server!".to_string(),
                            ),
                        ),
                        RegistrationResponse::Success(session_id, mapped_capabilities) => {
                            Ok((session_id, mapped_capabilities))
                        }
                    };
                }
                FeagiEndpointState::Errored(_) => {
                    return Err(FeagiAgentClientError::ConnectionFailed(
                        "Registration request failed".to_string(),
                    ));
                }
                _ => {}
            }
        }
    }

    /// Disconnect from the registration server. Call after registration is complete.
    pub fn disconnect(&mut self) -> Result<(), FeagiAgentClientError> {
        self.io_client
            .request_disconnect()
            .map_err(|e| FeagiAgentClientError::ConnectionFailed(e.to_string()))?;
        while !matches!(self.io_client.poll(), FeagiEndpointState::Inactive) {
            if matches!(self.io_client.poll(), FeagiEndpointState::Errored(_)) {
                let _ = self.io_client.confirm_error_and_close();
                break;
            }
        }
        Ok(())
    }
}
