use std::collections::{HashMap, HashSet};
use feagi_io::AgentID;
use feagi_io::traits_and_enums::shared::{TransportProtocolEndpoint, TransportProtocolImplementation};
use feagi_io::traits_and_enums::server::{FeagiServerPublisher, FeagiServerPublisherProperties, FeagiServerPuller, FeagiServerPullerProperties, FeagiServerRouterProperties};
use feagi_serialization::FeagiByteContainer;
use crate::{AgentCapabilities, AgentDescriptor, FeagiAgentError};
use crate::command_and_control::agent_registration_message::{AgentRegistrationMessage, RegistrationResponse};
use crate::command_and_control::FeagiMessage;
use crate::server::auth::AgentAuth;
use crate::server::translators::{CommandControlTranslator, MotorTranslator, SensorTranslator, VisualizationTranslator};

type CommandServerIndex = usize;

pub struct FeagiAgentHandler {
    agent_auth_backend: Box<dyn AgentAuth>,
    available_publishers: Vec<Box<dyn FeagiServerPublisherProperties>>,
    available_pullers: Vec<Box<dyn FeagiServerPullerProperties>>,
    command_control_servers: Vec<CommandControlTranslator>,

    all_registered_agents: HashMap<AgentID, (AgentDescriptor, Vec<AgentCapabilities>)>,
    agent_mapping_to_command_control_server_index: HashMap<AgentID, CommandServerIndex>,
    sensors: HashMap<AgentID,SensorTranslator>,
    motors: HashMap<AgentID,MotorTranslator>,
    visualizations: HashMap<AgentID,VisualizationTranslator>,


    // this stuff is likely redundant
    // REST STUFF
    /// Device registrations by AgentDescriptor (REST API configuration storage)
    device_registrations_by_descriptor: HashMap<AgentDescriptor, serde_json::Value>,
    /// Agent ID (base64) by AgentDescriptor (for REST→WebSocket bridging)
    agent_id_by_descriptor: HashMap<AgentDescriptor, String>,
    /// Device registrations by AgentID (active connections)
    device_registrations_by_agent: HashMap<AgentID, serde_json::Value>,
}

impl FeagiAgentHandler {

    pub fn new(agent_auth_backend: Box<dyn AgentAuth>) -> FeagiAgentHandler {
        FeagiAgentHandler {
            agent_auth_backend,
            available_publishers: Vec::new(),
            available_pullers: Vec::new(),

            command_control_servers: Vec::new(),
            all_registered_agents: HashMap::new(),
            agent_mapping_to_command_control_server_index: HashMap::new(),
            sensors: Default::default(),
            motors: Default::default(),
            visualizations: Default::default(),

            device_registrations_by_descriptor: HashMap::new(),
            agent_id_by_descriptor: HashMap::new(),
            device_registrations_by_agent: HashMap::new(),
        }
    }

    //region Get Properties

    pub fn get_all_registered_agents(&self) -> &HashMap<AgentID, (AgentDescriptor, Vec<AgentCapabilities>)> {
        &self.all_registered_agents
    }

    pub fn get_all_registered_sensors(&self) -> HashSet<AgentID> {
        self.sensors.keys().cloned().collect()
    }

    pub fn get_all_registered_motors(&self) -> HashSet<AgentID> {
        self.motors.keys().cloned().collect()
    }

    pub fn get_all_registered_visualizations(&self) -> HashSet<AgentID> {
        self.visualizations.keys().cloned().collect()
    }

    pub fn get_command_control_server_info(&self) -> Vec<Box<dyn FeagiServerRouterProperties>> {
        let mut output: Vec<Box<dyn FeagiServerRouterProperties>> = Vec::new();
        for command_control_server in &self.command_control_servers {
            output.push(command_control_server.get_running_server_properties())
        }
        output
    }

    //region  REST

    /// Get device registrations by AgentID
    pub fn get_device_registrations_by_agent(&self, agent_id: AgentID) -> Option<&serde_json::Value> {
        self.device_registrations_by_agent.get(&agent_id)
    }

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

    /// Store device registrations by AgentID (active connection)
    pub fn set_device_registrations_by_agent(&mut self, agent_id: AgentID, device_registrations: serde_json::Value) {
        self.device_registrations_by_agent.insert(agent_id, device_registrations);
    }

    // TODO redudant, you can simply check if a AgentID has the capability hash?
    /// Check if a agent has visualization capability configured
    /// Returns (agent_id_base64, rate_hz) for registration with RuntimeService
    pub fn get_visualization_info_for_agent(&self, agent_id: AgentID) -> Option<(String, f64)> {
        let device_regs = self.device_registrations_by_agent.get(&agent_id)?;
        let viz = device_regs.get("visualization")?;
        let rate_hz = viz.get("rate_hz").and_then(|v| v.as_f64())?;
        
        if rate_hz > 0.0 {
            let agent_descriptor = self.all_registered_agents.get(&agent_id)?;
            let agent_id = self.agent_id_by_descriptor.get(&agent_descriptor.0)?.clone();
            Some((agent_id, rate_hz))
        } else {
            None
        }
    }

/* // Redundant
    /// Find AgentID by agent_id (base64-encoded AgentDescriptor)
    /// Returns the first matching agent, or None if agent not connected
    pub fn find_agent_by_agent_id(&self, agent_id: &str) -> Option<AgentID> {
        let normalized_agent_id = agent_id.trim();

        // Fast path: parse provided agent ID as descriptor and compare by value.
        if let Ok(agent_descriptor) = crate::AgentDescriptor::try_from_base64(normalized_agent_id) {
            for (agent_id, descriptor) in &self.all_registered_agents {
                if descriptor == &agent_descriptor {
                    return Some(*agent_id);
                }
            }
        }

        // Fallback path: compare textual descriptor representation directly.
        // This covers benign formatting drift where the input contains extra whitespace
        // or comes from a source that preserves a pre-encoded descriptor string.
        for (agent_id, descriptor) in &self.all_registered_agents {
            if descriptor.as_base64() == normalized_agent_id {
                return Some(*agent_id);
            }
        }
        None
    }

    // TODO unclear! What is for what?
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

    // TODO fundimentally flawed. We already passed these to agents, theres no sharing!
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


    //endregion

    //endregion

    //region Add Servers

    /// Add a poll-based command/control server (ZMQ/WS). The router is wrapped in a
    /// [`CommandControlTranslator`] that only exposes messages.
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

    // TODO talk about forcibly starting servers
    /*
    /// Add and start a broadcast publisher server (e.g., visualization on port 9050)
    /// This creates a running server instance that can be polled and broadcast to
    /// NOTE: This does NOT add to available_publishers - broadcast publishers are shared
    pub fn add_and_start_broadcast_publisher(&mut self, publisher_props: Box<dyn FeagiServerPublisherProperties>) -> Result<(), FeagiAgentError> {
        let mut publisher = publisher_props.as_boxed_server_publisher();
        publisher.request_start()?;
        self.broadcast_publishers.push(publisher);
        Ok(())
    }

     */

    //endregion

    //region Command and Control

    /// Poll all command and control servers. Messages for registration request and heartbeat are
    /// handled internally here. Others are raised for FEAGI to act upon
    pub fn poll_command_and_control(&mut self) -> Result<Option<(AgentID, FeagiMessage)>, FeagiAgentError> {
        for (command_index, translator) in self.command_control_servers.iter_mut().enumerate() {
            // TODO smarter error handling. Many things don't deserve a panic
            let possible_message = translator.poll_for_incoming_messages(&self.all_registered_agents)?;

            match possible_message {
                None => { continue; }
                Some((agent_id, message, is_new_agent)) => {
                    if is_new_agent {
                        return self.handle_messages_from_unknown_agent_ids(agent_id, &message, command_index)
                    }
                    else {
                        return self.handle_messages_from_known_agent_ids(agent_id, message)
                    }
                }
            }
        }
        // Nothing to report from anyone!
        Ok(None)
    }

    /// Send a command and control message to a specific agent
    pub fn send_message_to_agent(&mut self, agent_id: AgentID, message: FeagiMessage, increment_counter: u16) -> Result<(), FeagiAgentError> {
        let translator_index = match self.agent_mapping_to_command_control_server_index.get(&agent_id) {
            None => {
                return Err(FeagiAgentError::Other("No such Agent ID exists!".to_string()))
            }
            Some(index) => {
                index
            }
        };

        let command_translator = match self.command_control_servers.get_mut(*translator_index) {
            None => {
                panic!("Missing Index for command control server!") // something went wrong
            }
            Some(translator) => {
                translator
            }
        };
        command_translator.send_message(agent_id, message,increment_counter)
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

    pub fn send_motor_data(&mut self, agent_id: AgentID, motor_data: &FeagiByteContainer) -> Result<(), FeagiAgentError> {
        let embodiment_option = self.motors.get_mut(&agent_id);
        match embodiment_option {
            Some(embodiment) => {
                embodiment.poll_and_send_buffered_motor_data(motor_data)?;
                Ok(())
            }
            None => {
                Err(FeagiAgentError::UnableToSendData("Nonexistant Agent ID!".to_string()))
            }
        }
    }

    /// Send visualization data to a specific agent via dedicated visualization channel
    pub fn send_visualization_data(&mut self, agent_id: AgentID, viz_data: &FeagiByteContainer) -> Result<(), FeagiAgentError> {
        let embodiment_option = self.visualizations.get_mut(&agent_id);
        match embodiment_option {
            Some(embodiment) => {
                embodiment.poll_and_send_visualization_data(viz_data)?;
                Ok(())
            }
            None => {
                Err(FeagiAgentError::UnableToSendData("Nonexistant Agent ID!".to_string()))
            }
        }
    }

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

    fn handle_messages_from_unknown_agent_ids(&mut self, agent_id: AgentID, message: &FeagiMessage, command_control_index: CommandServerIndex) -> Result<Option<(AgentID, FeagiMessage)>, FeagiAgentError> {
        match &message{
            FeagiMessage::AgentRegistration(register_message) => {
                match &register_message {
                    AgentRegistrationMessage::ClientRequestRegistration(registration_request) => {
                        let auth_result = self.agent_auth_backend.verify_agent_allowed_to_connect(registration_request);
                        if auth_result.is_err() {
                            return Ok(Some((agent_id, FeagiMessage::AgentRegistration(AgentRegistrationMessage::ServerRespondsRegistration(RegistrationResponse::FailedInvalidAuth)))))
                        }
                        // auth passed, check if we have the resources

                        let mut mappings = self.register_agent(agent_id,
                                                               *registration_request.connection_protocol(),
                                                               registration_request.requested_capabilities().to_vec(),
                                                               registration_request.agent_descriptor().clone(),
                                                               command_control_index)?;

                        let response = RegistrationResponse::Success(agent_id, mappings);
                        let message = FeagiMessage::AgentRegistration(AgentRegistrationMessage::ServerRespondsRegistration(response));
                        Ok(Some((agent_id, message)))
                    }
                    _ => {
                        // If not requesting registration, we dont want to hear it
                        Ok(None)
                    }
                }
            }
            _ => {
                // If the new agent is not registering, we don't want to hear it
                Ok(None)
            }
        }
    }

    fn handle_messages_from_known_agent_ids(&mut self, agent_id: AgentID, message: FeagiMessage) -> Result<Option<(AgentID, FeagiMessage)>, FeagiAgentError> {
        match &message{
            FeagiMessage::AgentRegistration(_register_message) => {
                // Already registered? dont dont register again
                // TODO any special exceptions?
                Ok(None)
            }
            FeagiMessage::HeartBeat => {
                // We can handle heartbeat here
                // TODO or maybe we should let the higher levels handle it?
                self.send_message_to_agent(agent_id, FeagiMessage::HeartBeat, 0)?;
                Ok(None)
            }
            _ => {
                // Throw up anything else
                Ok(Some((agent_id, message)))
            }
        }

    }

    //endregion

    //region Registration

    fn register_agent(&mut self, agent_id: AgentID, wanted_protocol: TransportProtocolImplementation, agent_capabilities: Vec<AgentCapabilities>, descriptor: AgentDescriptor, command_server_index: CommandServerIndex) -> Result<HashMap<AgentCapabilities, TransportProtocolEndpoint>, FeagiAgentError> {

        if self.all_registered_agents.contains_key(&agent_id) {
            return Err(FeagiAgentError::ConnectionFailed("Agent Already registered".to_string()));
        }

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
            let sensor_translator: SensorTranslator = SensorTranslator::new(agent_id, sensor_server);
            self.sensors.insert(agent_id, sensor_translator);
        }

        for motor_server in motor_servers {
            let motor_translator: MotorTranslator = MotorTranslator::new(agent_id, motor_server);
            self.motors.insert(agent_id, motor_translator);
        }

        for visualizer_server in visualizer_servers {
            let visualizer_translator: VisualizationTranslator = VisualizationTranslator::new(agent_id, visualizer_server);
            self.visualizations.insert(agent_id, visualizer_translator);
        }


        self.all_registered_agents.insert(agent_id, (descriptor, agent_capabilities));
        self.agent_mapping_to_command_control_server_index.insert(agent_id, command_server_index);

        Ok(endpoint_mappings)
    }

    //endregion

    //endregion
}

