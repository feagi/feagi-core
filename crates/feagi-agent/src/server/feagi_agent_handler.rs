use std::collections::{HashMap, HashSet};
use feagi_io::traits_and_enums::shared::{TransportProtocolEndpoint, TransportProtocolImplementation};
use feagi_io::traits_and_enums::server::{FeagiServerPublisher, FeagiServerPublisherProperties, FeagiServerPuller, FeagiServerPullerProperties, FeagiServerRouterProperties};
use feagi_serialization::{FeagiByteContainer, SessionID};
use crate::{AgentCapabilities, AgentDescriptor, FeagiAgentError};
use crate::command_and_control::agent_registration_message::{AgentRegistrationMessage, RegistrationResponse};
use crate::command_and_control::FeagiMessage;
use crate::server::auth::AgentAuth;
use crate::server::CommandControlTranslator;
use crate::server::translators::{MotorTranslator, SensorTranslator, VisualizationTranslator};

pub struct FeagiAgentHandler {
    agent_auth_backend: Box<dyn AgentAuth>,
    available_pullers: Vec<Box<dyn FeagiServerPullerProperties>>,
    available_publishers: Vec<Box<dyn FeagiServerPublisherProperties>>,
    command_control_servers: Vec<CommandControlTranslator>,

    all_registered_sessions: HashMap<SessionID, (AgentDescriptor, Vec<AgentCapabilities>)>,
    sensors: HashMap<SessionID,SensorTranslator>,
    motors: HashMap<SessionID,MotorTranslator>,
    visualizations: HashMap<SessionID,VisualizationTranslator>,
}



impl FeagiAgentHandler {


    pub fn new(agent_auth_backend: Box<dyn AgentAuth>) -> FeagiAgentHandler {
        FeagiAgentHandler {
            agent_auth_backend,
            available_publishers: Vec::new(),
            available_pullers: Vec::new(),

            command_control_servers: Vec::new(),
            all_registered_sessions: HashMap::new(),
            sensors: Default::default(),
            motors: Default::default(),
            visualizations: Default::default(),
        }
    }

    //region Get Properties

    pub fn get_all_registered_sessions(&self) -> &HashMap<SessionID, (AgentDescriptor, Vec<AgentCapabilities>)> {
        &self.all_registered_sessions
    }

    pub fn get_all_registered_sensors(&self) -> HashSet<SessionID> {
        self.sensors.keys().cloned().collect()
    }

    pub fn get_all_registered_motors(&self) -> HashSet<SessionID> {
        self.motors.keys().cloned().collect()
    }

    pub fn get_all_registered_visualizations(&self) -> HashSet<SessionID> {
        self.visualizations.keys().cloned().collect()
    }

    //endregion

    //region Device Registration Management (REST API Support)

    // NOTE: REST stuff will have to be updated
    /*
    /// Store device registrations by AgentDescriptor (REST API - before connection)
    /// Also stores the original agent_id for later WebSocket→REST bridging
    pub fn set_device_registrations_by_descriptor(&mut self, agent_id_base64: String, agent_descriptor: AgentDescriptor, device_registrations: serde_json::Value) {
        self.device_registrations_by_descriptor.insert(agent_descriptor.clone(), device_registrations);
        self.agent_id_by_descriptor.insert(agent_descriptor, agent_id_base64);
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

     */



    /// Get all registered sessions with their descriptors
    pub fn get_registered_agents(&self) -> &HashMap<SessionID, AgentDescriptor> {
        &self.all_registered_sessions
    }
    


    /// Get agent descriptor by session ID
    pub fn get_agent_descriptor(&self, session_id: SessionID) -> Option<&AgentDescriptor> {
        self.all_registered_sessions.get(&session_id)
    }

    /// Find SessionID by agent_id (base64-encoded AgentDescriptor)
    /// Returns the first matching session, or None if agent not connected
    pub fn find_session_by_agent_id(&self, agent_id: &str) -> Option<SessionID> {
        // Parse agent_id to AgentDescriptor
        let agent_descriptor = crate::AgentDescriptor::try_from_base64(agent_id).ok()?;
        
        // Find session with matching descriptor
        for (session_id, descriptor) in &self.all_registered_sessions {
            if descriptor == &agent_descriptor {
                return Some(*session_id);
            }
        }
        None
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



    //endregion

    //region Adding Servers

    /// Add a poll-based command/control server (ZMQ/WS). The router is wrapped in a
    /// [`CommandControlTranslator`] that oinly exposes messages.
    pub fn add_and_start_command_control_server(&mut self, router_property: Box<dyn FeagiServerRouterProperties>) -> Result<(), FeagiAgentError> {
        let mut router = router_property.as_boxed_server_router();
        router.request_start()?;
        let translator = CommandControlTranslator::new(router);
        self.command_control_servers.push(translator);
        Ok(())
    }

    pub fn add_publisher_endpoint(&mut self, publisher: Box<dyn FeagiServerPublisherProperties>) {
        // TODO check for collisions
        self.available_publishers.push(publisher);
    }

    pub fn add_puller_endpoint(&mut self, puller: Box<dyn FeagiServerPullerProperties>) {
        // TODO check for collisions
        self.available_pullers.push(puller);
    }


    //endregion

    //region Command and Control

    /// Poll all command and control servers. Messages for registration request and heartbeat are
    /// handled internally here. Others are raised for FEAGI to act upon
    pub fn poll_command_and_controls(&mut self) -> Result<Option<(SessionID, FeagiMessage)>, FeagiAgentError> {
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
        let command_translator = self.command_control_servers.get_mut(0); // TODO this is not sufficient logic. We need a lookup table!
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

    //region Agents

    pub fn poll_agent_sensors(&mut self) -> Result<Option<&FeagiByteContainer>, FeagiAgentError> {
        for (_id, translator) in self.sensors.iter_mut() {
            let possible_sensor_data = translator.poll_sensor_server()?;
            if possible_sensor_data.is_some() {
                return Ok(possible_sensor_data);
            }
        }
        Ok(None)
    }

    pub fn poll_agent_motors(&mut self) -> Result<(), FeagiAgentError> {
        for (_id, translator) in self.motors.iter_mut() {
            translator.poll_motor_server()?;
        }
        Ok(())
    }

    pub fn poll_agent_visualizers(&mut self) -> Result<(), FeagiAgentError> {
        for (_id, translator) in self.visualizations.iter_mut() {
            translator.poll_visualization_server()?;
        }
        Ok(())
    }

    pub fn send_motor_data(&mut self, session_id: SessionID, motor_data: &FeagiByteContainer) -> Result<(), FeagiAgentError> {
        let embodiment_option = self.motors.get_mut(&session_id);
        match embodiment_option {
            Some(embodiment) => {
                embodiment.poll_and_send_buffered_motor_data(motor_data)?;
                Ok(())
            }
            None => {
                Err(FeagiAgentError::UnableToSendData("Nonexistant Session ID!".to_string()))
            }
        }
    }

    /// Send visualization data to a specific agent via dedicated visualization channel
    pub fn send_visualization_data(&mut self, session_id: SessionID, viz_data: &FeagiByteContainer) -> Result<(), FeagiAgentError> {
        let embodiment_option = self.visualizations.get_mut(&session_id);
        match embodiment_option {
            Some(embodiment) => {
                embodiment.poll_and_send_visualization_data(viz_data)?;
                Ok(())
            }
            None => {
                Err(FeagiAgentError::UnableToSendData("Nonexistant Session ID!".to_string()))
            }
        }
    }


/* // There seem to be some duplicates here?
    /// Broadcast visualization data directly to all broadcast publisher servers
    /// Used for visualization-only agents that connect without embodiment registration
    pub fn broadcast_visualization_data(&mut self, viz_data: &FeagiByteContainer) -> Result<(), FeagiAgentError> {
        for publisher in &mut self.broadcast_publishers {
            match publisher.publish_data(viz_data.get_byte_ref()) {
                Ok(_) => {
                    log::trace!("[BROADCAST-VIZ] Published {} bytes", viz_data.get_byte_ref().len());
                }
                Err(e) => {
                    log::warn!("[BROADCAST-VIZ] Failed to broadcast: {}", e);
                }
            }
        }
        Ok(())
    }
    
    /// Broadcast raw bytes directly to all broadcast publisher servers
    /// Used for visualization-only agents that expect raw Type 11 format
    pub fn broadcast_raw_visualization_data(&mut self, raw_bytes: &[u8]) -> Result<(), FeagiAgentError> {
        for publisher in &mut self.broadcast_publishers {
            match publisher.publish_data(raw_bytes) {
                Ok(_) => {
                    log::trace!("[BROADCAST-VIZ] Published {} raw bytes", raw_bytes.len());
                }
                Err(e) => {
                    log::warn!("[BROADCAST-VIZ] Failed to broadcast: {}", e);
                }
            }
        }
        Ok(())
    }


 */
    //endregion



    //region Internal

    //region Get property

    fn try_get_puller_property_index(&mut self, wanted_protocol: &TransportProtocolImplementation) -> Result<usize, FeagiAgentError> {
        for i in 0..self.available_pullers.len() {
            let available_puller = &self.available_pullers[i];
            if &available_puller.get_protocol() != wanted_protocol {
                // not the protocol we are looking for
                continue;
            } else {
                // found the protocol we want
                return Ok(i);
            }
        }
        Err(FeagiAgentError::InitFail("Missing required protocol puller".to_string()))
    }

    fn try_get_publisher_property_index(&mut self, wanted_protocol: &TransportProtocolImplementation) -> Result<usize, FeagiAgentError> {
        for i in 0..self.available_publishers.len() {
            let available_publisher = &self.available_publishers[i];
            if &available_publisher.get_protocol() != wanted_protocol {
                // not the protocol we are looking for
                continue;
            } else {
                // found the protocol we want
                return Ok(i);
            }
        }
        Err(FeagiAgentError::InitFail("Missing required protocol publisher".to_string()))
    }

    //endregion

    //region Message Handling

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

                        let mut mappings = self.register_agent(session_id,
                                                               *registration_request.connection_protocol(),
                                                               registration_request.requested_capabilities().to_vec(),
                                                               registration_request.agent_descriptor().clone())?;

                        let response = RegistrationResponse::Success(session_id, mappings);
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

    //endregion


    //region Registration


    fn register_agent(&mut self, session_id: SessionID, wanted_protocol: TransportProtocolImplementation, agent_capabilities: Vec<AgentCapabilities>, descriptor: AgentDescriptor) -> Result<HashMap<AgentCapabilities, TransportProtocolEndpoint>, FeagiAgentError> {

        // NOTE: ASSUMES SESSION_ID IS NOT USED!

        let mut sensor_index: usize = 0;
        let mut motor_index: usize = 0;
        let mut visualizer_index: usize = 0;
        let mut sensor_servers: Vec<Box<dyn FeagiServerPuller>> = Vec::new();
        let mut motor_servers: Vec<Box<dyn FeagiServerPublisher>> = Vec::new();
        let mut visualizer_servers: Vec<Box<dyn FeagiServerPublisher>> = Vec::new();
        let mut endpoint_mappings: HashMap<AgentCapabilities, TransportProtocolEndpoint> = HashMap::new();

        // We try spawning all the servers first without taking any properties out mof circulation
        for agent_capability in &agent_capabilities {
            match agent_capability {
                AgentCapabilities::SendSensorData => {
                    let puller_property_index= self.try_get_puller_property_index(&wanted_protocol)?;
                    let puller_property = &self.available_pullers[puller_property_index];
                    let mut sensor_server = puller_property.as_boxed_server_puller();
                    _ = sensor_server.request_start()?;
                    sensor_servers.push(sensor_server);
                    endpoint_mappings.insert(AgentCapabilities::SendSensorData, puller_property.get_endpoint());
                    sensor_index += 1;
                }
                AgentCapabilities::ReceiveMotorData => {
                    let publisher_index = self.try_get_publisher_property_index(&wanted_protocol)?;
                    let publisher_property = &self.available_publishers[publisher_index];
                    let mut publisher_server = publisher_property.as_boxed_server_publisher();
                    _ = publisher_server.request_start()?;
                    motor_servers.push(publisher_server);
                    endpoint_mappings.insert(AgentCapabilities::ReceiveMotorData, publisher_property.get_endpoint());
                    motor_index += 1;
                }
                AgentCapabilities::ReceiveNeuronVisualizations => {
                    let publisher_index = self.try_get_publisher_property_index(&wanted_protocol)?;
                    let publisher_property = &self.available_publishers[publisher_index];
                    let mut publisher_server = publisher_property.as_boxed_server_publisher();
                    _ = publisher_server.request_start()?;
                    visualizer_servers.push(publisher_server);
                    endpoint_mappings.insert(AgentCapabilities::ReceiveNeuronVisualizations, publisher_property.get_endpoint());
                    visualizer_index += 1;
                }
                AgentCapabilities::ReceiveSystemMessages => {
                    todo!()
                }
            }
        }

        // everything is good, take used properties out of circulation
        self.available_pullers.drain(0..sensor_index);
        self.available_publishers.drain(0..motor_index + visualizer_index);

        // insert the servers into the cache
        for sensor_server in sensor_servers {
            let sensor_translator: SensorTranslator = SensorTranslator::new(session_id, sensor_server);
            self.sensors.insert(session_id, sensor_translator);
        }

        for motor_server in motor_servers {
            let motor_translator: MotorTranslator = MotorTranslator::new(session_id, motor_server);
            self.motors.insert(session_id, motor_translator);
        }

        for visualizer_server in visualizer_servers {
            let visualizer_translator: VisualizationTranslator = VisualizationTranslator::new(session_id, visualizer_server);
            self.visualizations.insert(session_id, visualizer_translator);
        }



        self.all_registered_sessions.insert(session_id, (descriptor, agent_capabilities));
        Ok(endpoint_mappings)
    }

    //endregion

    //endregion


}


/// Implement EmbodimentSensoryPoller trait for integration with BurstLoopRunner
/// This allows the burst loop to poll for sensory data from ZMQ/WS embodiment agents
impl feagi_npu_burst_engine::EmbodimentSensoryPoller for FeagiAgentHandler {
    fn poll_sensory_data(&mut self) -> Result<Option<Vec<u8>>, String> {
        match self.poll_agent_sensors() {
            Ok(Some(byte_container)) => {
                // Return the serialized bytes
                Ok(Some(byte_container.get_byte_ref().to_vec()))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(format!("Sensory poll failed: {:?}", e)),
        }
    }
}

