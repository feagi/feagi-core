//! Registration Agent
//!
//! Connects to the FEAGI registration endpoint (ZMQ or WS), sends a registration request,
//! and returns session_id and capability endpoints. Use this once; then connect to the
//! returned data endpoints (sensory, motor, visualization). Disconnect after registration
//! so the registration channel is not held open.

use std::collections::HashMap;
use feagi_io::traits_and_enums::shared::{FeagiEndpointState, TransportProtocolEndpoint};
use feagi_io::traits_and_enums::client::{FeagiClientRequester, FeagiClientRequesterProperties};
use feagi_serialization::{FeagiByteContainer, SessionID};
use crate::command_and_control::agent_registration_message::{AgentRegistrationMessage, RegistrationRequest, RegistrationResponse};
use crate::{AgentCapabilities, AgentDescriptor, AuthToken, FeagiAgentError};
use crate::command_and_control::FeagiMessage;

pub struct CommandControlSubAgent {
    properties: Box<dyn FeagiClientRequesterProperties>,
    requester: Option<Box<dyn FeagiClientRequester>>,
    incoming_cache: FeagiByteContainer,
    registration_status: AgentRegistrationStatus
}

impl CommandControlSubAgent {

    pub fn new(endpoint_properties: Box<dyn FeagiClientRequesterProperties>) -> Self {
        Self {
            registration_status: AgentRegistrationStatus::NotRegistered,
            properties: endpoint_properties,
            requester: None,
            incoming_cache: FeagiByteContainer::new_empty()
        }
    }

    pub fn registration_status(&self) -> &AgentRegistrationStatus {
        &self.registration_status
    }

    pub fn request_connect(&mut self) -> Result<(), FeagiAgentError> {
        let mut requester;
        if self.requester.is_none() {
            requester = self.properties.as_boxed_client_requester();
        } else {
            requester = self.requester.take().unwrap();
        }

        match requester.poll() {
            FeagiEndpointState::Inactive => {
                _ = requester.request_connect()?;
                self.requester = Some(requester);
                Ok(())
            }
            _ => {
                Err(FeagiAgentError::ConnectionFailed("Socket is already active!".to_string()))
            }
        }
    }

    pub fn request_registration(&mut self, agent_descriptor: AgentDescriptor, auth_token: AuthToken, requested_capabilities: Vec<AgentCapabilities>) -> Result<(), FeagiAgentError> {
        if self.registration_status == AgentRegistrationStatus::NotRegistered {
            return Err(FeagiAgentError::ConnectionFailed("Agent is already registered!".to_string()));
        }

        if let Some(requester) = &mut self.requester.take() {

            let request = RegistrationRequest::new(
                agent_descriptor,
                auth_token,
                requested_capabilities,
                requester.get_protocol()
            );
            let request_message = FeagiMessage::AgentRegistration(
                AgentRegistrationMessage::ClientRequestRegistration(request)
            );
            let request_bytes: FeagiByteContainer = request_message.into();
            requester.publish_request(request_bytes.get_byte_ref())?;
            Ok(())
        }
        else {
            Err(FeagiAgentError::ConnectionFailed("Cannot register to endpoint when not connected!".to_string()))
        }
    }

    pub fn poll_for_messages(&mut self) -> Result<Option<FeagiMessage>, FeagiAgentError> {
        if let Some(requester) = &mut self.requester.take() {

            let state = requester.poll();
            match state {
                FeagiEndpointState::Inactive => {
                    Ok(None)
                }
                FeagiEndpointState::Pending => {
                    Ok(None)
                }
                FeagiEndpointState::ActiveWaiting => {
                    Ok(None)
                }
                FeagiEndpointState::ActiveHasData => {
                    let data = requester.consume_retrieved_response()?;
                    self.incoming_cache.try_write_data_by_copy_and_verify(&data)?;
                    let feagi_message: FeagiMessage = (&self.incoming_cache).try_into()?;

                    match &feagi_message {
                        FeagiMessage::HeartBeat => {
                            // TODO how should we handle this???
                            return Ok(None)
                        }
                        FeagiMessage::AgentRegistration(registration_message) => {
                            match registration_message {
                                AgentRegistrationMessage::ClientRequestRegistration(_) => {
                                    // Not possible
                                    Err(FeagiAgentError::ConnectionFailed("Client cannot register agents!".to_string()))
                                }
                                AgentRegistrationMessage::ServerRespondsRegistration(registration_response) => {
                                    match registration_response {
                                        RegistrationResponse::FailedInvalidRequest => {
                                            Err(FeagiAgentError::ConnectionFailed("Invalid server responses!".to_string()))
                                        }
                                        RegistrationResponse::FailedInvalidAuth => {
                                            Err(FeagiAgentError::ConnectionFailed("Invalid auth token!".to_string()))
                                        }
                                        RegistrationResponse::AlreadyRegistered => {
                                            Err(FeagiAgentError::ConnectionFailed("Client already registered!".to_string()))
                                        }
                                        RegistrationResponse::Success(session_id, endpoints) => {
                                            self.registration_status = AgentRegistrationStatus::Registered(session_id.clone(), endpoints.clone());
                                            Ok(Some(feagi_message))
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            // just return the message as is
                            Ok(Some(feagi_message))
                        }
                    }

                }
                FeagiEndpointState::Errored(_) => {
                    requester.confirm_error_and_close()?;
                    Err(FeagiAgentError::ConnectionFailed("Error occurred".to_string()))
                }
            }
        }
        else {
            Err(FeagiAgentError::ConnectionFailed("No socket is active to poll!".to_string()))
        }
    }

}

#[derive(Debug,  PartialEq, Clone)]
pub enum AgentRegistrationStatus {
    NotRegistered,
    Registered(SessionID, HashMap<AgentCapabilities, TransportProtocolEndpoint>)
}