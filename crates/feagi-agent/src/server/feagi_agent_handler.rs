use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;
use feagi_io::traits_and_enums::shared::{FeagiEndpointState, TransportProtocolEndpoint, TransportProtocolImplementation};
use feagi_io::traits_and_enums::server::{FeagiServerPublisher, FeagiServerPublisherProperties, FeagiServerPuller, FeagiServerPullerProperties, FeagiServerRouterProperties};
use feagi_serialization::{FeagiByteContainer, SessionID};
use crate::{AgentCapabilities, AgentDescriptor, FeagiAgentError};
use crate::command_and_control::agent_registration_message::{AgentRegistrationMessage, RegistrationRequest, RegistrationResponse};
use crate::command_and_control::FeagiMessage;
use crate::server::auth::AgentAuth;
use crate::server::CommandControlTranslator;
use crate::server::translators::EmbodimentTranslator;

pub struct FeagiAgentHandler {
    agent_auth_backend: Box<dyn AgentAuth>,
    available_publishers: Vec<Box<dyn FeagiServerPublisherProperties>>,
    available_pullers: Vec<Box<dyn FeagiServerPullerProperties>>,

    command_control_servers: Vec<CommandControlTranslator>,
    all_registered_sessions: HashMap<SessionID, AgentDescriptor>,
    registered_embodiments: HashMap<SessionID, EmbodimentTranslator>



}

impl FeagiAgentHandler {


    pub fn new(agent_auth_backend: Box<dyn AgentAuth>) -> FeagiAgentHandler {
        FeagiAgentHandler {

        }
    }

    //region Add Servers

    /// Add a poll-based command/control server (ZMQ/WS). The router is wrapped in a
    /// [`CommandControlTranslator`] that oinly exposes messages.
    pub fn add_and_start_command_control_server(&mut self, router_property: Box<dyn FeagiServerRouterProperties>) -> Result<(), FeagiAgentError> {
        let mut router = router_property.as_boxed_server_router();
        router.request_start()?;
        let translator = CommandControlTranslator::new(router);
        self.command_control_servers.push(translator);
        Ok(())
    }

    pub fn add_publisher_server(&mut self, publisher: Box<dyn FeagiServerPublisherProperties>) {
        // TODO check for collisions
        self.available_publishers.push(publisher);
    }

    pub fn add_puller_server(&mut self, puller: Box<dyn FeagiServerPullerProperties>) {
        // TODO check for collisions
        self.available_pullers.push(puller);
    }
    //endregion

    //region Polling

    /// Poll all command and control servers. Messages for registration request and heartbeat are
    /// handled internally here. Others are raised for FEAGI to act upon
    pub fn poll_command_and_control(&mut self) -> Result<Option<(SessionID, FeagiMessage)>, FeagiAgentError> {
        for translator in self.command_control_servers.iter_mut() {
            // TODO smarter error handling. Many things don't deserve a panic
            let possible_message = translator.poll_for_incoming_messages(&self.all_registered_sessions)?;

            match possible_message {
                None => { continue; }
                Some((session_id, message)) => {
                    if self.handle_registrations_and_heartbeats(session_id, &message)? {
                        // We handled the request internally. Continue
                        continue;
                    }
                    // The request is of some other nature, raise it
                    return Ok(Some((session_id, message)))
                }
            }

        }
        // Nothing to report from anyone!
        Ok(None)
    }

    pub fn send_message_to_agent(&mut self, session_id: SessionID, message: FeagiMessage) -> Result<(), FeagiAgentError> {
        // TODO logic for picking the correct server!
        todo!();
    }

    /// Some messages can be handled here (namely registration and heartbeat). If the message is one of those, handle it and return true.
    /// Otherwise, return false
    fn handle_registrations_and_heartbeats(&mut self, session_id: SessionID, message: &FeagiMessage) -> Result<bool, FeagiAgentError> {
        match &message {
            FeagiMessage::HeartBeat => {
                // Send a heartbeat back
                // TODO prevent spam
                self.send_message_to_agent(session_id, FeagiMessage::HeartBeat)?;
                return Ok(true);
            }
            FeagiMessage::AgentRegistration(registration_message) => {
                // A registration message came in, and the translator checked its not obviously invalid
                match &registration_message {
                    AgentRegistrationMessage::ClientRequestRegistration(registration_request) => {

                        let response = self.verify_agent_registration_request_and_make_response(session_id, registration_request)?;
                        self.send_message_to_agent(session_id, FeagiMessage::AgentRegistration(AgentRegistrationMessage::ServerRespondsRegistration(response)))?;
                        return Ok(true);
                    }
                    AgentRegistrationMessage::ServerRespondsRegistration(_) => {
                        // This is the server, we do the responding!
                        return Err(FeagiAgentError::UnableToDecodeReceivedData("Client tried sending registration response!".to_stirng()))
                    }
                }
            }
            _ => {
                // Any other message type should be handled on a higher level
                return Ok(false);
            }
        }

    }








    /// Single entry point for all registration flows. Use `None` for REST (handler generates
    /// session id); use `Some(session_id)` for poll-based transports (ZMQ/WS) that provide it.
    pub fn process_registration(
        &mut self,
        registration_request: RegistrationRequest,
        transport_session_id: Option<SessionID>,
    ) -> Result<RegistrationResponse, FeagiAgentError> {
        let session_id = transport_session_id.unwrap_or_else(|| self.new_session_id());
        self.verify_agent_request_and_make_response(&session_id, registration_request)
    }

    /// Register an agent without a transport-level router (REST path). Thin wrapper around
    /// `process_registration(request, None)`.
    pub fn register_agent_direct(
        &mut self,
        registration_request: RegistrationRequest,
    ) -> Result<RegistrationResponse, FeagiAgentError> {
        self.process_registration(registration_request, None)
    }

    pub fn poll_sensory_handlers(&mut self) -> Option<&FeagiByteContainer> {
        for i in 0..self.active_sensor_servers.len() {
            match self.active_sensor_servers[i].poll() {
                FeagiEndpointState::ActiveHasData => {
                    if self.sensory_cache.try_write_data_by_copy_and_verify(
                        self.active_sensor_servers[i].consume_retrieved_data().unwrap() // TODO error handling
                    ).is_err() {
                        warn!("Failed to decode incoming sensory bytes into FeagiByteContainer");
                        return None;
                    }
                    let session_id = match self.sensory_cache.get_session_id() {
                        Ok(id) => id,
                        Err(_) => {
                            warn!("Rejected sensory payload with invalid session ID");
                            return None;
                        }
                    };
                    if !self.is_session_registered(&session_id) {
                        warn!("Rejected sensory payload from unregistered session ID");
                        return None;
                    }
                    return Some(&self.sensory_cache);
                }
                FeagiEndpointState::Errored(_e) => {
                    return None; // TODO we need to do better here
                }
                _ => {
                    continue;
                }
            }
        }
        None
    }

    pub fn poll_motor_handlers(&mut self, data: Option<&FeagiByteContainer>) -> Result<(), FeagiAgentError> {
        let Some(bytes) = data else {
            return Ok(()); // Nothing to publish
        };

        for i in 0..self.active_motor_servers.len() {
            match self.active_motor_servers[i].poll() {
                FeagiEndpointState::ActiveWaiting => {
                    self.active_motor_servers[i]
                        .publish_data(bytes.get_byte_ref())
                        .map_err(|e| FeagiAgentError::UnableToSendData(e.to_string()))?;
                }
                FeagiEndpointState::Errored(_e) => {
                    self.active_motor_servers[i].confirm_error_and_close().map_err(
                        |e| FeagiAgentError::ConnectionFailed(e.to_string())
                    )?;
                    // TODO we need to do better here
                }
                _ => {
                    continue;
                }
            }
        }
        Ok(())
    }

    pub fn poll_visualization_handlers(&mut self, data: Option<&FeagiByteContainer>) -> Result<(), FeagiAgentError> {
        let Some(bytes) = data else {
            return Ok(()); // Nothing to publish
        };

        for i in 0..self.active_visualizer_servers.len() {
            match self.active_visualizer_servers[i].poll() {
                FeagiEndpointState::ActiveWaiting => {
                    self.active_visualizer_servers[i]
                        .publish_data(bytes.get_byte_ref())
                        .map_err(|e| FeagiAgentError::UnableToSendData(e.to_string()))?;
                }
                FeagiEndpointState::Errored(_e) => {
                    self.active_visualizer_servers[i].confirm_error_and_close().map_err(
                        |e| FeagiAgentError::ConnectionFailed(e.to_string())
                    )?;
                    // TODO we need to do better here
                }
                _ => {
                    continue;
                }
            }
        }
        Ok(())
    }

    //endregion

    //region Internal

    //region Registration

    // TODO we need to have a proper discussion about endpoints. Possibly when defining the pushers / pollers, we also couple an endpoint URL or something?

    fn verify_agent_registration_request_and_make_response(&mut self, session_id: SessionID, registration_request: &RegistrationRequest) -> Result<RegistrationResponse, FeagiAgentError> {

        let verify_auth = self.agent_auth_backend.verify_agent_allowed_to_connect(&registration_request);  // TODO how do we make this non blocking????
        if verify_auth.is_err() {
            return Ok(RegistrationResponse::FailedInvalidAuth)
        }
        // TODO verify no duplicates!


        if registration_request.requested_capabilities().contains(&AgentCapabilities::SendSensorData) &&
            registration_request.requested_capabilities().contains(&AgentCapabilities::ReceiveMotorData) {
            // Agent is requesting a motor and sensor data set. Its an embodiment

            let sensor_property = self.try_get_puller_property(registration_request.connection_protocol())?;
            let motor_property = self.try_get_publisher_property(registration_request.connection_protocol())?;
            let mut puller = sensor_property.as_boxed_server_puller();
            let mut publisher = motor_property.as_boxed_server_publisher();

            puller.request_start().map_err(|e| FeagiAgentError::ConnectionFailed(e.to_string()))?;
            publisher.request_start().map_err(|e| FeagiAgentError::ConnectionFailed(e.to_string()))?;

            _ = self.registered_embodiments.insert(session_id, EmbodimentTranslator::new(
                session_id,
                publisher,
                puller,
            ));

            let mut endpoints: HashMap<AgentCapabilities, TransportProtocolEndpoint> = HashMap::new();
            endpoints.insert(AgentCapabilities::SendSensorData, motor_property.get_protocol())
            endpoints.insert(AgentCapabilities::ReceiveMotorData, sensor_property.get_protocol())
            return Ok(RegistrationResponse::Success(session_id, endpoints));
        }

        // TODO other types. Is this really the best way?
        return Err(FeagiAgentError::InitFail("TODO".to_string()))
    }

    fn try_get_puller_property(&mut self, wanted_protocol: &TransportProtocolImplementation) -> Result<Box<dyn FeagiServerPullerProperties>, FeagiAgentError> {
        for i in 0..self.available_pullers.len() {
            let available_puller = &self.available_pullers[i];
            if &available_puller.get_protocol() != wanted_protocol {
                // not the protocol we are looking for
                continue;
            } else {
                // found the protocol we want
                return Ok(self.available_pullers.remove(i));
            }
        }
        Err(FeagiAgentError::InitFail("Missing required protocol puller".to_string()))
    }

    fn try_get_publisher_property(&mut self, wanted_protocol: &TransportProtocolImplementation) -> Result<Box<dyn FeagiServerPublisherProperties>, FeagiAgentError> {
        for i in 0..self.available_publishers.len() {
            let available_publisher = &self.available_publishers[i];
            if &available_publisher.get_protocol() != wanted_protocol {
                // not the protocol we are looking for
                continue;
            } else {
                // found the protocol we want
                return Ok(self.available_publishers.remove(i));
            }
        }
        Err(FeagiAgentError::InitFail("Missing required protocol publisher".to_string()))
    }






    //endregion

    //endregion

}

