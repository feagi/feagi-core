use std::collections::{HashMap, HashSet};
use feagi_io::traits_and_enums::shared::{TransportProtocolEndpoint, TransportProtocolImplementation};
use feagi_io::traits_and_enums::server::{FeagiServerPublisherProperties, FeagiServerPullerProperties, FeagiServerRouterProperties};
use feagi_serialization::{FeagiByteContainer, SessionID};
use crate::{AgentCapabilities, AgentDescriptor, FeagiAgentError};
use crate::command_and_control::agent_registration_message::{AgentRegistrationMessage, RegistrationResponse};
use crate::command_and_control::FeagiMessage;
use crate::server::auth::AgentAuth;
use crate::server::CommandControlTranslator;
use crate::server::translators::EmbodimentTranslator;

pub struct FeagiAgentHandler {
    agent_auth_backend: Box<dyn AgentAuth>,
    available_publishers: Vec<Box<dyn FeagiServerPublisherProperties>>,
    available_pullers: Vec<Box<dyn FeagiServerPullerProperties>>,

    all_registered_sessions: HashMap<SessionID, AgentDescriptor>,
    /// Device registrations by AgentDescriptor (REST API configuration storage)
    device_registrations_by_descriptor: HashMap<AgentDescriptor, serde_json::Value>,
    /// Device registrations by SessionID (active connections)
    device_registrations_by_session: HashMap<SessionID, serde_json::Value>,

    command_control_servers: Vec<CommandControlTranslator>,
    registered_embodiments: Vec<EmbodimentTranslator>,

    session_id_command_control_mapping: HashMap<SessionID, usize>,
    session_id_embodiments_mapping: HashMap<SessionID, usize>,

}

impl FeagiAgentHandler {


    pub fn new(agent_auth_backend: Box<dyn AgentAuth>) -> FeagiAgentHandler {
        FeagiAgentHandler {
            agent_auth_backend,
            available_publishers: Vec::new(),
            available_pullers: Vec::new(),

            command_control_servers: Vec::new(),
            all_registered_sessions: HashMap::new(),
            device_registrations_by_descriptor: HashMap::new(),
            device_registrations_by_session: HashMap::new(),
            registered_embodiments: Vec::new(),
            session_id_command_control_mapping: Default::default(),
            session_id_embodiments_mapping: Default::default(),
        }
    }

    //region Device Registration Management (REST API Support)

    /// Store device registrations by AgentDescriptor (REST API - before connection)
    pub fn set_device_registrations_by_descriptor(&mut self, agent_descriptor: AgentDescriptor, device_registrations: serde_json::Value) {
        self.device_registrations_by_descriptor.insert(agent_descriptor, device_registrations);
    }

    /// Get device registrations by AgentDescriptor (REST API queries)
    pub fn get_device_registrations_by_descriptor(&self, agent_descriptor: &AgentDescriptor) -> Option<&serde_json::Value> {
        self.device_registrations_by_descriptor.get(agent_descriptor)
    }

    /// Store device registrations by SessionID (active connection)
    pub fn set_device_registrations_by_session(&mut self, session_id: SessionID, device_registrations: serde_json::Value) {
        self.device_registrations_by_session.insert(session_id, device_registrations);
    }

    /// Get device registrations by SessionID
    pub fn get_device_registrations_by_session(&self, session_id: SessionID) -> Option<&serde_json::Value> {
        self.device_registrations_by_session.get(&session_id)
    }

    /// Get all registered sessions with their descriptors
    pub fn get_registered_agents(&self) -> &HashMap<SessionID, AgentDescriptor> {
        &self.all_registered_sessions
    }

    /// Get agent descriptor by session ID
    pub fn get_agent_descriptor(&self, session_id: SessionID) -> Option<&AgentDescriptor> {
        self.all_registered_sessions.get(&session_id)
    }

    /// Get available transport protocols (for REST registration response)
    pub fn get_available_protocols(&self) -> HashSet<TransportProtocolImplementation> {
        let mut protocols = HashSet::new();
        for puller in &self.available_pullers {
            protocols.insert(puller.get_protocol());
        }
        for publisher in &self.available_publishers {
            protocols.insert(publisher.get_protocol());
        }
        protocols
    }

    /// Get transport endpoints for REST registration response
    pub fn get_transport_endpoints(&self) -> HashMap<TransportProtocolImplementation, Vec<TransportProtocolEndpoint>> {
        let mut endpoints = HashMap::new();
        
        for puller in &self.available_pullers {
            endpoints.entry(puller.get_protocol())
                .or_insert_with(Vec::new)
                .push(puller.get_endpoint());
        }
        
        for publisher in &self.available_publishers {
            endpoints.entry(publisher.get_protocol())
                .or_insert_with(Vec::new)
                .push(publisher.get_endpoint());
        }
        
        endpoints
    }

    //endregion

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

    //region Command and Control

    /// Poll all command and control servers. Messages for registration request and heartbeat are
    /// handled internally here. Others are raised for FEAGI to act upon
    pub fn poll_command_and_control(&mut self) -> Result<Option<(SessionID, FeagiMessage)>, FeagiAgentError> {
        for (command_index, translator) in self.command_control_servers.iter_mut().enumerate() {
            // TODO smarter error handling. Many things don't deserve a panic
            let possible_message = translator.poll_for_incoming_messages(&self.all_registered_sessions)?;

            match possible_message {
                None => { continue; }
                Some((session_id, message, is_new_session)) => {
                    if is_new_session {
                        return self.handle_messages_from_unknown_session_ids(session_id, &message, command_index)
                    }
                    else {
                        return self.handle_messages_from_known_session_ids(session_id, message)
                    }
                }
            }
        }
        // Nothing to report from anyone!
        Ok(None)
    }

    /// Send a command and control message to a specific agent
    pub fn send_message_to_agent(&mut self, session_id: SessionID, message: FeagiMessage, increment_counter: u16) -> Result<(), FeagiAgentError> {
        let command_translator = self.try_get_command_mut(session_id)?;
        match command_translator {
            Some(command_translator) => {
                command_translator.send_message(session_id, message,increment_counter)
            }
            None => {
                Err(FeagiAgentError::UnableToSendData("Unable to send message to unknown Session ID!".to_string()))
            }
        }
    }

    //endregion

    //region Embodiments

    pub fn poll_embodiment_sensors(&mut self) -> Result<Option<&FeagiByteContainer>, FeagiAgentError> {
        for embodiment in self.registered_embodiments.iter_mut() {
            let possible_sensor_data = embodiment.poll_sensor_server()?;
            if possible_sensor_data.is_some() {
                return Ok(possible_sensor_data);
            }
        }
        Ok(None)
    }

    pub fn poll_embodiment_motors(&mut self) -> Result<(), FeagiAgentError> {
        for embodiment in self.registered_embodiments.iter_mut() {
            embodiment.poll_motor_server()?;
        }
        Ok(())
    }

    pub fn send_motor_data(&mut self, session_id: SessionID, motor_data: &FeagiByteContainer) -> Result<(), FeagiAgentError> {
        let embodiment_option = self.try_get_embodiment_mut(session_id)?;
        match embodiment_option {
            Some(embodiment) => {
                embodiment.send_buffered_motor_data(motor_data)?;
                Ok(())
            }
            None => {
                Err(FeagiAgentError::UnableToSendData("Nonexistant Session ID!".to_string()))
            }
        }
    }


    //endregion



    //region Internal

    //region Registration

    fn handle_messages_from_unknown_session_ids(&mut self, session_id: SessionID, message: &FeagiMessage, command_control_index: usize) -> Result<Option<(SessionID, FeagiMessage)>, FeagiAgentError> {
        match &message{
            FeagiMessage::AgentRegistration(register_message) => {
                match &register_message {
                    AgentRegistrationMessage::ClientRequestRegistration(registration_request) => {
                        let auth_result = self.agent_auth_backend.verify_agent_allowed_to_connect(registration_request);
                        if auth_result.is_err() {
                            return Ok(Some((session_id, FeagiMessage::AgentRegistration(AgentRegistrationMessage::ServerRespondsRegistration(RegistrationResponse::FailedInvalidAuth)))))
                        }
                        // auth passed, check if we have the resources

                        // TODO we should rethink agent roles. For now, just assume embodiment
                        // TODO we shouldnt error like this, we should send a response if we are missing resources
                        let sensor_puller_props = self.try_get_puller_property(registration_request.connection_protocol())?;
                        let motor_pusher_props = self.try_get_publisher_property(registration_request.connection_protocol())?;

                        let mut sensor_puller = sensor_puller_props.as_boxed_server_puller();
                        let mut motor_pusher = motor_pusher_props.as_boxed_server_publisher();

                        sensor_puller.request_start()?;
                        motor_pusher.request_start()?;

                        let embodiment = EmbodimentTranslator::new(session_id, motor_pusher, sensor_puller);
                        self.register_new_embodiment_agent_to_cache(session_id, registration_request.agent_descriptor().clone(), command_control_index, embodiment)?;

                        // we set everything up, send response of success
                        let mut mapping: HashMap<AgentCapabilities, TransportProtocolEndpoint> = HashMap::new();
                        mapping.insert(AgentCapabilities::SendSensorData, sensor_puller_props.get_endpoint());
                        mapping.insert(AgentCapabilities::ReceiveMotorData, motor_pusher_props.get_endpoint());
                        let response = RegistrationResponse::Success(session_id, mapping);
                        let message = FeagiMessage::AgentRegistration(AgentRegistrationMessage::ServerRespondsRegistration(response));
                        Ok(Some((session_id, message)))
                    }
                    _ => {
                        // If not requesting registration, we dont want to hear it
                        Ok(None)
                    }
                }
            }
            _ => {
                // If the new session is not registering, we don't want to hear it
                Ok(None)
            }
        }
    }

    fn handle_messages_from_known_session_ids(&mut self, session_id: SessionID, message: FeagiMessage) -> Result<Option<(SessionID, FeagiMessage)>, FeagiAgentError> {
        match &message{
            FeagiMessage::AgentRegistration(_register_message) => {
                // Already registered? dont dont register again
                // TODO any special exceptions?
                Ok(None)
            }
            FeagiMessage::HeartBeat => {
                // We can handle heartbeat here
                // TODO or maybe we should let the higher levels handle it?
                self.send_message_to_agent(session_id, FeagiMessage::HeartBeat, 0)?;
                Ok(None)
            }
            _ => {
                // Throw up anything else
                Ok(Some((session_id, message)))
            }
        }

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

    fn register_new_embodiment_agent_to_cache(&mut self, session_id: SessionID, agent_descriptor: AgentDescriptor, _command_index: usize, embodiment: EmbodimentTranslator) -> Result<(), FeagiAgentError> {
        let new_embodiment_index = self.registered_embodiments.len();
        self.registered_embodiments.push(embodiment);
        self.all_registered_sessions.insert(session_id, agent_descriptor);
        self.session_id_embodiments_mapping.insert(session_id, new_embodiment_index);
        Ok(())
    }

    //endregion

    fn try_get_command_mut(&mut self, session_id: SessionID) -> Result<Option<&mut CommandControlTranslator>, FeagiAgentError> {
        let index = self.session_id_command_control_mapping.get(&session_id);
        match index {
            Some(index) => {
                let command = self.command_control_servers.get_mut(*index);
                Ok(command)
            }
            None => {
                Ok(None)
            }
        }
    }

    fn try_get_embodiment_mut(&mut self, session_id: SessionID) -> Result<Option<&mut EmbodimentTranslator>, FeagiAgentError> {
        let index = self.session_id_embodiments_mapping.get(&session_id);
        match index {
            Some(index) => {
                let embodiment = self.registered_embodiments.get_mut(*index);
                Ok(embodiment)
            }
            None => {
                Ok(None)
            }
        }
    }

    //endregion

}

