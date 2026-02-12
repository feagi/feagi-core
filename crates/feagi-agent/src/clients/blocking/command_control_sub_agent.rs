//! Registration Agent
//!
//! Connects to the FEAGI registration endpoint (ZMQ or WS), sends a registration request,
//! and returns session_id and capability endpoints. Use this once; then connect to the
//! returned data endpoints (sensory, motor, visualization). Disconnect after registration
//! so the registration channel is not held open.

use crate::command_and_control::agent_registration_message::{
    AgentRegistrationMessage, DeregistrationRequest, DeregistrationResponse, RegistrationRequest,
    RegistrationResponse,
};
use crate::command_and_control::FeagiMessage;
use crate::{AgentCapabilities, AgentDescriptor, AuthToken, FeagiAgentError};
use feagi_io::traits_and_enums::client::{FeagiClientRequester, FeagiClientRequesterProperties};
use feagi_io::traits_and_enums::shared::{FeagiEndpointState, TransportProtocolEndpoint};
use feagi_io::AgentID;
use feagi_serialization::FeagiByteContainer;
use std::collections::HashMap;

pub struct CommandControlSubAgent {
    properties: Box<dyn FeagiClientRequesterProperties>,
    requester: Option<Box<dyn FeagiClientRequester>>,
    incoming_cache: FeagiByteContainer,
    registration_status: AgentRegistrationStatus,
}

impl CommandControlSubAgent {
    pub fn new(endpoint_properties: Box<dyn FeagiClientRequesterProperties>) -> Self {
        Self {
            registration_status: AgentRegistrationStatus::NotRegistered,
            properties: endpoint_properties,
            requester: None,
            incoming_cache: FeagiByteContainer::new_empty(),
        }
    }

    pub fn registration_status(&self) -> &AgentRegistrationStatus {
        &self.registration_status
    }

<<<<<<< HEAD
    // Send connection request
=======
    /// Return the configured command/control endpoint target.
    ///
    /// This works both before and after connect by creating an ephemeral
    /// requester if needed.
    pub fn endpoint_target(&mut self) -> TransportProtocolEndpoint {
        if let Some(requester) = &mut self.requester {
            requester.get_endpoint_target()
        } else {
            let requester = self.properties.as_boxed_client_requester();
            requester.get_endpoint_target()
        }
    }

>>>>>>> origin/heartbeat
    pub fn request_connect(&mut self) -> Result<(), FeagiAgentError> {

        if self.registration_status != AgentRegistrationStatus::NotRegistered {
            return Err(FeagiAgentError::ConnectionFailed("Agent already connected and registered!".to_string()))
        }

        // TODO if it is something, what is it?
        if self.requester.is_none() {
            self.requester = Some(self.properties.as_boxed_client_requester());
        }

        let mut requester = self.requester.take().unwrap();

        match requester.poll() {
            FeagiEndpointState::Inactive => {
                _ = requester.request_connect()?;
                self.requester = Some(requester);
                Ok(())
            }
            _ => {
                self.requester = Some(requester);
                Err(FeagiAgentError::ConnectionFailed(
                    "Socket is already active!".to_string(),
                ))
            }
        }
    }

    pub fn request_registration(
        &mut self,
        agent_descriptor: AgentDescriptor,
        auth_token: AuthToken,
        requested_capabilities: Vec<AgentCapabilities>,
    ) -> Result<(), FeagiAgentError> {
        if self.registration_status != AgentRegistrationStatus::NotRegistered {
            return Err(FeagiAgentError::ConnectionFailed(
                "Agent is already registered!".to_string(),
            ));
        }

        if let Some(requester) = &mut self.requester {
            let request = RegistrationRequest::new(
                agent_descriptor,
                auth_token,
                requested_capabilities,
                requester.get_protocol(),
            );
            let request_message = FeagiMessage::AgentRegistration(
                AgentRegistrationMessage::ClientRequestRegistration(request),
            );
            let request_bytes: FeagiByteContainer = request_message.into();
            requester.publish_request(request_bytes.get_byte_ref())?;
            Ok(())
        } else {
            Err(FeagiAgentError::ConnectionFailed(
                "Cannot register to endpoint when not connected!".to_string(),
            ))
        }
    }

    /// Send a heartbeat over the command/control channel for the provided session.
    ///
    /// This is the deterministic, tick-driven heartbeat primitive used by
    /// higher-level client loops.
    pub fn request_heartbeat(&mut self, session_id: AgentID) -> Result<(), FeagiAgentError> {
        if let Some(requester) = &mut self.requester {
            let heartbeat_message = FeagiMessage::HeartBeat;
            let mut request_bytes = FeagiByteContainer::new_empty();
            heartbeat_message.serialize_to_byte_container(&mut request_bytes, session_id, 0)?;
            requester.publish_request(request_bytes.get_byte_ref())?;
            Ok(())
        } else {
            Err(FeagiAgentError::ConnectionFailed(
                "Cannot send heartbeat when not connected!".to_string(),
            ))
        }
    }

    /// Request voluntary deregistration for the given session.
    ///
    /// The optional `reason` string is forwarded for observability and does not
    /// alter deregistration behavior on the server.
    pub fn request_deregistration(
        &mut self,
        session_id: AgentID,
        reason: Option<String>,
    ) -> Result<(), FeagiAgentError> {
        if let Some(requester) = &mut self.requester {
            let request = DeregistrationRequest::new(reason);
            let message = FeagiMessage::AgentRegistration(
                AgentRegistrationMessage::ClientRequestDeregistration(request),
            );
            let mut request_bytes = FeagiByteContainer::new_empty();
            message.serialize_to_byte_container(&mut request_bytes, session_id, 0)?;
            requester.publish_request(request_bytes.get_byte_ref())?;
            Ok(())
        } else {
            Err(FeagiAgentError::ConnectionFailed(
                "Cannot deregister when not connected!".to_string(),
            ))
        }
    }

    pub fn poll_state(&mut self) -> Result<FeagiEndpointState, FeagiAgentError> {
        if let Some(requester) = &mut self.requester {
            Ok(requester.poll().clone())
        } else {
            Err(FeagiAgentError::ConnectionFailed("No socket active".to_string()))
        }
    }

    pub fn poll_for_messages(&mut self) -> Result<Option<FeagiMessage>, FeagiAgentError> {
        if let Some(mut requester) = self.requester.take() {
            let state = requester.poll();
            let result = match state {
                FeagiEndpointState::Inactive => {
                    self.requester = Some(requester);
                    Ok(None)
                }
                FeagiEndpointState::Pending => {
                    self.requester = Some(requester);
                    Ok(None)
                }
                FeagiEndpointState::ActiveWaiting => {
                    self.requester = Some(requester);
                    Ok(None)
                }
                FeagiEndpointState::ActiveHasData => {
                    let data = requester.consume_retrieved_response()?;
                    self.incoming_cache
                        .try_write_data_by_copy_and_verify(&data)?;
                    let feagi_message: FeagiMessage = (&self.incoming_cache).try_into()?;

                    let result = match &feagi_message {
                        FeagiMessage::HeartBeat => {
                            // TODO how should we handle this???
                            Ok(None)
                        }
                        FeagiMessage::AgentRegistration(registration_message) => {
                            match registration_message {
                                AgentRegistrationMessage::ClientRequestRegistration(_) => {
                                    // Not possible
                                    Err(FeagiAgentError::ConnectionFailed(
                                        "Client cannot register agents!".to_string(),
                                    ))
                                }
                                AgentRegistrationMessage::ServerRespondsRegistration(
                                    registration_response,
                                ) => match registration_response {
                                    RegistrationResponse::FailedInvalidRequest => {
                                        Err(FeagiAgentError::ConnectionFailed(
                                            "Invalid server responses!".to_string(),
                                        ))
                                    }
                                    RegistrationResponse::FailedInvalidAuth => {
                                        Err(FeagiAgentError::ConnectionFailed(
                                            "Invalid auth token!".to_string(),
                                        ))
                                    }
                                    RegistrationResponse::AlreadyRegistered => {
                                        Err(FeagiAgentError::ConnectionFailed(
                                            "Client already registered!".to_string(),
                                        ))
                                    }
                                    RegistrationResponse::Success(session_id, endpoints) => {
                                        self.registration_status =
                                            AgentRegistrationStatus::Registered(
                                                session_id.clone(),
                                                endpoints.clone(),
                                            );
                                        Ok(Some(feagi_message))
                                    }
                                },
                                AgentRegistrationMessage::ClientRequestDeregistration(_) => {
                                    Err(FeagiAgentError::ConnectionFailed(
                                        "Client cannot receive deregistration request from server!"
                                            .to_string(),
                                    ))
                                }
                                AgentRegistrationMessage::ServerRespondsDeregistration(
                                    deregistration_response,
                                ) => match deregistration_response {
                                    DeregistrationResponse::Success => Ok(Some(feagi_message)),
                                    DeregistrationResponse::NotRegistered => {
                                        Ok(Some(feagi_message))
                                    }
                                },
                            }
                        }
                        _ => {
                            // just return the message as is
                            Ok(Some(feagi_message))
                        }
                    };

                    // Restore requester before returning
                    self.requester = Some(requester);
                    result
                }
                FeagiEndpointState::Errored(_) => {
                    requester.confirm_error_and_close()?;
                    // Don't restore requester - it's been closed
                    Err(FeagiAgentError::ConnectionFailed(
                        "Error occurred".to_string(),
                    ))
                }
            };

            result
        } else {
            Err(FeagiAgentError::ConnectionFailed(
                "No socket is active to poll!".to_string(),
            ))
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AgentRegistrationStatus {
    NotRegistered,
    Registered(
        AgentID,
        HashMap<AgentCapabilities, TransportProtocolEndpoint>,
    ),
}
