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

pub struct CommandControlAgent {
    properties: Box<dyn FeagiClientRequesterProperties>,
    requester: Option<Box<dyn FeagiClientRequester>>,
    request_buffer: FeagiByteContainer,
    send_buffer: FeagiByteContainer,
    registration_status: AgentRegistrationStatus,
}

impl CommandControlAgent {
    pub fn new(endpoint_properties: Box<dyn FeagiClientRequesterProperties>) -> Self {
        Self {
            registration_status: AgentRegistrationStatus::NotRegistered,
            properties: endpoint_properties,
            requester: None,
            request_buffer: FeagiByteContainer::new_empty(),
            send_buffer: FeagiByteContainer::new_empty(),
        }
    }

    //region Properties
    pub fn registration_status(&self) -> &AgentRegistrationStatus {
        &self.registration_status
    }

    pub fn registered_endpoint_target(&mut self) -> TransportProtocolEndpoint {
        self.properties.get_endpoint_target()
    }
    //endregion

    //region Helpers

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


        let transport_protocol = if let Some(requester) = &mut self.requester {
            requester.get_endpoint_target().as_transport_protocol_implementation()
        } else {
            return Err(FeagiAgentError::ConnectionFailed(
                "Cannot register to endpoint when not connected!".to_string(),
            ))
        };

        let request = RegistrationRequest::new(
            agent_descriptor,
            auth_token,
            requested_capabilities,
            transport_protocol,
        );

        let request_message = FeagiMessage::AgentRegistration(
            AgentRegistrationMessage::ClientRequestRegistration(request),
        );

        self.send_message(request_message, 0)?;
        Ok(())
    }

    /// Request voluntary deregistration for the given session.
    ///
    /// The optional `reason` string is forwarded for observability and does not
    /// alter deregistration behavior on the server.
    pub fn request_deregistration(
        &mut self,
        reason: Option<String>,  // TODO Please dont use strings, use ENUMS!
    ) -> Result<(), FeagiAgentError> {

        let request = DeregistrationRequest::new(reason);
        let message = FeagiMessage::AgentRegistration(
            AgentRegistrationMessage::ClientRequestDeregistration(request),
        );

        self.send_message(message, 0)?;
        Ok(())
    }


    /// Send a heartbeat over the command/control channel for the provided session.
    ///
    /// This is the deterministic, tick-driven heartbeat primitive used by
    /// higher-level client loops.
    pub fn send_heartbeat(&mut self) -> Result<(), FeagiAgentError> {
        let heartbeat_message = FeagiMessage::HeartBeat;
        self.send_message(heartbeat_message, 0)
    }

    //endregion

    //region Base Functions

    pub fn poll_for_messages(&mut self) -> Result<(&FeagiEndpointState, Option<FeagiMessage>), FeagiAgentError> {
        if let Some(mut requester) = self.requester.take() {
            let state = requester.poll();
            let result:  Result<(&FeagiEndpointState, Option<FeagiMessage>), FeagiAgentError> = match state {
                FeagiEndpointState::Inactive => {
                    self.requester = Some(requester);
                    Ok((state, None))
                }
                FeagiEndpointState::Pending => {
                    self.requester = Some(requester);
                    Ok((state, None))
                }
                FeagiEndpointState::ActiveWaiting => {
                    self.requester = Some(requester);
                    Ok((state, None))
                }
                FeagiEndpointState::ActiveHasData => {
                    let data = requester.consume_retrieved_response()?;
                    self.request_buffer
                        .try_write_data_by_copy_and_verify(&data)?;
                    let feagi_message: FeagiMessage = (&self.request_buffer).try_into()?;

                    let result: Result<(&FeagiEndpointState, Option<FeagiMessage>), FeagiAgentError> = match &feagi_message {
                        FeagiMessage::HeartBeat => {
                            Ok((state, Some(FeagiMessage::HeartBeat)))
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
                                        Ok((state, Some(feagi_message)))
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
                                    DeregistrationResponse::Success => {
                                        requester.request_disconnect();
                                        self.registration_status = AgentRegistrationStatus::NotRegistered;
                                            Ok((state, Some(feagi_message)))
                                    },
                                    DeregistrationResponse::NotRegistered => {
                                        Ok((state, Some(feagi_message)))
                                    }
                                },
                            }
                        }
                        _ => {
                            // just return the message as is
                            Ok((state, Some(feagi_message)))
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

    pub fn send_message(&mut self, message: FeagiMessage, increment_value: u16) -> Result<(), FeagiAgentError> {
        let agent_id = match self.registration_status {
            AgentRegistrationStatus::Registered(agent_id, _) => { agent_id },
            _ => {return Err(FeagiAgentError::UnableToSendData("Nonregistered agent cannot send message!".to_string()))}
        };

        if let Some(requester) = &mut self.requester {
            message.serialize_to_byte_container(&mut self.send_buffer, agent_id, increment_value)?;
            requester.publish_request(&mut self.send_buffer.get_byte_ref())?;
            Ok(())
        }
        else {
            // This state should be impossible. something went very wrong
            panic!("Active state but no socket!!")
        }


    }

    //endregion

}

#[derive(Debug, PartialEq, Clone)]
pub enum AgentRegistrationStatus {
    NotRegistered,
    Registered(
        AgentID,
        HashMap<AgentCapabilities, TransportProtocolEndpoint>,
    ),
}
