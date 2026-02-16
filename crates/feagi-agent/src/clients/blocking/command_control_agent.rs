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
        let maybe_message = {
            let requester = self.requester.as_mut().ok_or_else(|| {
                FeagiAgentError::ConnectionFailed("No socket is active to poll!".to_string())
            })?;

            let state_snapshot = requester.poll().clone();
            match state_snapshot {
                FeagiEndpointState::Inactive
                | FeagiEndpointState::Pending
                | FeagiEndpointState::ActiveWaiting => Ok(None),
                FeagiEndpointState::ActiveHasData => {
                    let data = requester.consume_retrieved_response()?;
                    self.request_buffer
                        .try_write_data_by_copy_and_verify(&data)?;
                    let feagi_message: FeagiMessage = (&self.request_buffer).try_into()?;

                    match &feagi_message {
                        FeagiMessage::HeartBeat => Ok(Some(FeagiMessage::HeartBeat)),
                        FeagiMessage::AgentRegistration(registration_message) => {
                            match registration_message {
                                AgentRegistrationMessage::ClientRequestRegistration(_) => Err(
                                    FeagiAgentError::ConnectionFailed(
                                        "Client cannot register agents!".to_string(),
                                    ),
                                ),
                                AgentRegistrationMessage::ServerRespondsRegistration(
                                    registration_response,
                                ) => match registration_response {
                                    RegistrationResponse::FailedInvalidRequest => Err(
                                        FeagiAgentError::ConnectionFailed(
                                            "Invalid server responses!".to_string(),
                                        ),
                                    ),
                                    RegistrationResponse::FailedInvalidAuth => Err(
                                        FeagiAgentError::ConnectionFailed(
                                            "Invalid auth token!".to_string(),
                                        ),
                                    ),
                                    RegistrationResponse::AlreadyRegistered => Err(
                                        FeagiAgentError::ConnectionFailed(
                                            "Client already registered!".to_string(),
                                        ),
                                    ),
                                    RegistrationResponse::Success(session_id, endpoints) => {
                                        self.registration_status =
                                            AgentRegistrationStatus::Registered(
                                                session_id.clone(),
                                                endpoints.clone(),
                                            );
                                        Ok(Some(feagi_message))
                                    }
                                },
                                AgentRegistrationMessage::ClientRequestDeregistration(_) => Err(
                                    FeagiAgentError::ConnectionFailed(
                                        "Client cannot receive deregistration request from server!"
                                            .to_string(),
                                    ),
                                ),
                                AgentRegistrationMessage::ServerRespondsDeregistration(
                                    deregistration_response,
                                ) => match deregistration_response {
                                    DeregistrationResponse::Success => {
                                        requester.request_disconnect()?;
                                        self.registration_status =
                                            AgentRegistrationStatus::NotRegistered;
                                        Ok(Some(feagi_message))
                                    }
                                    DeregistrationResponse::NotRegistered => {
                                        Ok(Some(feagi_message))
                                    }
                                },
                            }
                        }
                        _ => Ok(Some(feagi_message)),
                    }
                }
                FeagiEndpointState::Errored(_) => {
                    requester.confirm_error_and_close()?;
                    Err(FeagiAgentError::ConnectionFailed(
                        "Error occurred".to_string(),
                    ))
                }
            }
        }?;

        let state = self
            .requester
            .as_mut()
            .ok_or_else(|| {
                FeagiAgentError::ConnectionFailed("No socket is active to poll!".to_string())
            })?
            .poll();

        Ok((state, maybe_message))
    }

    pub fn send_message(&mut self, message: FeagiMessage, increment_value: u16) -> Result<(), FeagiAgentError> {
        let agent_id = match &self.registration_status {
            AgentRegistrationStatus::Registered(agent_id, _) => *agent_id,
            AgentRegistrationStatus::NotRegistered => {
                // Registration must be possible before a session id exists.
                // FEAGI servers accept a blank agent id for registration requests.
                match &message {
                    FeagiMessage::AgentRegistration(
                        AgentRegistrationMessage::ClientRequestRegistration(_),
                    ) => AgentID::new_blank(),
                    _ => {
                        return Err(FeagiAgentError::UnableToSendData(
                            "Nonregistered agent cannot send message!".to_string(),
                        ));
                    }
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use feagi_io::protocol_implementations::zmq::ZmqUrl;
    use feagi_io::traits_and_enums::shared::{
        FeagiEndpointState, TransportProtocolEndpoint,
    };
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct DummyRequesterProperties {
        endpoint: TransportProtocolEndpoint,
        last_request: Arc<Mutex<Vec<u8>>>,
    }

    struct DummyRequester {
        endpoint: TransportProtocolEndpoint,
        state: FeagiEndpointState,
        last_request: Arc<Mutex<Vec<u8>>>,
    }

    impl feagi_io::traits_and_enums::client::FeagiClient for DummyRequester {
        fn poll(&mut self) -> &FeagiEndpointState {
            &self.state
        }

        fn request_connect(&mut self) -> Result<(), feagi_io::FeagiNetworkError> {
            self.state = FeagiEndpointState::ActiveWaiting;
            Ok(())
        }

        fn request_disconnect(&mut self) -> Result<(), feagi_io::FeagiNetworkError> {
            self.state = FeagiEndpointState::Inactive;
            Ok(())
        }

        fn confirm_error_and_close(&mut self) -> Result<(), feagi_io::FeagiNetworkError> {
            self.state = FeagiEndpointState::Inactive;
            Ok(())
        }

        fn get_endpoint_target(&self) -> TransportProtocolEndpoint {
            self.endpoint.clone()
        }
    }

    impl feagi_io::traits_and_enums::client::FeagiClientRequester for DummyRequester {
        fn publish_request(&mut self, request: &[u8]) -> Result<(), feagi_io::FeagiNetworkError> {
            *self.last_request.lock().expect("lock") = request.to_vec();
            Ok(())
        }

        fn consume_retrieved_response(&mut self) -> Result<&[u8], feagi_io::FeagiNetworkError> {
            Err(feagi_io::FeagiNetworkError::ReceiveFailed(
                "dummy requester has no responses".to_string(),
            ))
        }

        fn as_boxed_requester_properties(
            &self,
        ) -> Box<dyn feagi_io::traits_and_enums::client::FeagiClientRequesterProperties> {
            Box::new(DummyRequesterProperties {
                endpoint: self.endpoint.clone(),
                last_request: self.last_request.clone(),
            })
        }
    }

    impl feagi_io::traits_and_enums::client::FeagiClientRequesterProperties for DummyRequesterProperties {
        fn as_boxed_client_requester(&self) -> Box<dyn feagi_io::traits_and_enums::client::FeagiClientRequester> {
            Box::new(DummyRequester {
                endpoint: self.endpoint.clone(),
                state: FeagiEndpointState::Inactive,
                last_request: self.last_request.clone(),
            })
        }

        fn get_endpoint_target(&self) -> TransportProtocolEndpoint {
            self.endpoint.clone()
        }
    }

    #[test]
    fn registration_request_can_be_sent_before_registration() {
        let endpoint = TransportProtocolEndpoint::Zmq(
            ZmqUrl::new("tcp://example:1").expect("valid dummy endpoint"),
        );
        let last_request: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
        let props = Box::new(DummyRequesterProperties {
            endpoint,
            last_request: last_request.clone(),
        });

        let mut agent = CommandControlAgent::new(props);
        agent.request_connect().expect("connect request should succeed");

        agent
            .request_registration(
                AgentDescriptor::new("m", "n", 1).expect("descriptor"),
                AuthToken::new([0u8; 32]),
                vec![AgentCapabilities::SendSensorData],
            )
            .expect("registration request should be sendable with blank id");

        assert!(
            !last_request.lock().expect("lock").is_empty(),
            "expected a serialized registration request to be published"
        );
    }
}
