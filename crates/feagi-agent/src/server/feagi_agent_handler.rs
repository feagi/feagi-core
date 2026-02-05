use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use feagi_io::traits_and_enums::shared::{FeagiEndpointState, TransportProtocolImplementation};
use feagi_io::traits_and_enums::server::{FeagiServerPublisher, FeagiServerPublisherProperties, FeagiServerPuller, FeagiServerPullerProperties, FeagiServerRouterProperties};
use feagi_serialization::{FeagiByteContainer, SessionID};
use log::warn;
use crate::feagi_agent_server_error::FeagiAgentServerError;
use crate::registration::{AgentCapabilities, AgentDescriptor, RegistrationRequest, RegistrationResponse};
use crate::server::auth::AgentAuth;
use crate::server::translators::command_control_translator::CommandControlTranslator;

pub struct FeagiAgentHandler {
    agent_auth_backend: Box<dyn AgentAuth>,
    registered_agents: HashMap<SessionID, (AgentDescriptor, Vec<AgentCapabilities>)>,

    available_publishers: Vec<Box<dyn FeagiServerPublisherProperties>>,
    available_pullers: Vec<Box<dyn FeagiServerPullerProperties>>,

    active_motor_servers: Vec<Box<dyn FeagiServerPublisher>>,
    active_visualizer_servers: Vec<Box<dyn FeagiServerPublisher>>,
    active_sensor_servers: Vec<Box<dyn FeagiServerPuller>>,

    /// Poll-based registration sources (ZMQ/WS via RouterRegistrationAdapter; future transports).
    pollable_registration_sources: Vec<CommandControlTranslator>,

    sensory_cache: FeagiByteContainer,
}

impl FeagiAgentHandler {


    pub fn new(agent_auth_backend: Box<dyn AgentAuth>) -> FeagiAgentHandler {
        FeagiAgentHandler {
            agent_auth_backend,
            registered_agents: HashMap::new(),
            available_publishers: vec![],
            available_pullers: vec![],
            active_motor_servers: vec![],
            active_visualizer_servers: vec![],
            active_sensor_servers: vec![],
            pollable_registration_sources: vec![],
            sensory_cache: FeagiByteContainer::new_empty(),
        }
    }

    //region Add Servers

    // add at least one registration server  that acts as the endpoint to register, and the publisher pullers you can add a bunch which will act as endpoints for agent capabilties such as sensor motor and visualization


    /// Add a poll-based registration server (ZMQ/WS). The router is wrapped in a
    /// [`CommandControlTranslator`] so the handler only deals with registration types.
    pub fn add_and_start_registration_server(&mut self, router_property: Box<dyn FeagiServerRouterProperties>) -> Result<(), FeagiAgentServerError> {
        let mut router = router_property.as_boxed_server_router();
        router
            .request_start()
            .map_err(|e| FeagiAgentServerError::InitFail(e.to_string()))?;
        let adapter = CommandControlTranslator::new(router);
        self.pollable_registration_sources.push(adapter);
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

    /// Poll all registration sources (ZMQ/WS adapters, etc.). For each pending request,
    /// run core registration and send the response back via the source.
    pub fn poll_registration_handlers(&mut self) -> Result<(), FeagiAgentServerError> {
        let mut pending: Vec<(usize, SessionID, RegistrationRequest)> = Vec::new();
        for (i, source) in self.pollable_registration_sources.iter_mut().enumerate() {
            match source.poll_registration() {
                Ok(Some((session_id, request))) => {
                    pending.push((i, session_id, request));
                }
                Ok(None) => {}
                Err(e) => {
                    warn!(
                        "[feagi-agent] Registration source error: {}",
                        e
                    );
                }
            }
        }
        for (i, session_id, request) in pending {
            let response = self.process_registration(request, Some(session_id))?;
            self.pollable_registration_sources[i]
                .send_response(session_id, &response)
                .map_err(|e| FeagiAgentServerError::UnableToSendData(e.to_string()))?;
        }
        Ok(())
    }

    /// Single entry point for all registration flows. Use `None` for REST (handler generates
    /// session id); use `Some(session_id)` for poll-based transports (ZMQ/WS) that provide it.
    pub fn process_registration(
        &mut self,
        registration_request: RegistrationRequest,
        transport_session_id: Option<SessionID>,
    ) -> Result<RegistrationResponse, FeagiAgentServerError> {
        let session_id = transport_session_id.unwrap_or_else(|| self.new_session_id());
        self.verify_agent_request_and_make_response(&session_id, registration_request)
    }

    /// Register an agent without a transport-level router (REST path). Thin wrapper around
    /// `process_registration(request, None)`.
    pub fn register_agent_direct(
        &mut self,
        registration_request: RegistrationRequest,
    ) -> Result<RegistrationResponse, FeagiAgentServerError> {
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

    pub fn poll_motor_handlers(&mut self, data: Option<&FeagiByteContainer>) -> Result<(), FeagiAgentServerError> {
        let Some(bytes) = data else {
            return Ok(()); // Nothing to publish
        };

        for i in 0..self.active_motor_servers.len() {
            match self.active_motor_servers[i].poll() {
                FeagiEndpointState::ActiveWaiting => {
                    self.active_motor_servers[i]
                        .publish_data(bytes.get_byte_ref())
                        .map_err(|e| FeagiAgentServerError::UnableToSendData(e.to_string()))?;
                }
                FeagiEndpointState::Errored(_e) => {
                    self.active_motor_servers[i].confirm_error_and_close().map_err(
                        |e| FeagiAgentServerError::ConnectionFailed(e.to_string())
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

    pub fn poll_visualization_handlers(&mut self, data: Option<&FeagiByteContainer>) -> Result<(), FeagiAgentServerError> {
        let Some(bytes) = data else {
            return Ok(()); // Nothing to publish
        };

        for i in 0..self.active_visualizer_servers.len() {
            match self.active_visualizer_servers[i].poll() {
                FeagiEndpointState::ActiveWaiting => {
                    self.active_visualizer_servers[i]
                        .publish_data(bytes.get_byte_ref())
                        .map_err(|e| FeagiAgentServerError::UnableToSendData(e.to_string()))?;
                }
                FeagiEndpointState::Errored(_e) => {
                    self.active_visualizer_servers[i].confirm_error_and_close().map_err(
                        |e| FeagiAgentServerError::ConnectionFailed(e.to_string())
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

    fn verify_agent_request_and_make_response(&mut self, session_id: &SessionID, registration_request: RegistrationRequest) -> Result<RegistrationResponse, FeagiAgentServerError> {

        let verify_auth = self.agent_auth_backend.verify_agent_allowed_to_connect(&registration_request);  // TODO how do we make this non blocking????
        if verify_auth.is_err() {
            return Ok(RegistrationResponse::FailedInvalidAuth)
        }

        // TODO verify no duplicates!

        let mut endpoints: HashMap<AgentCapabilities, String> = HashMap::new();
        for requested_capability in registration_request.requested_capabilities() {
            match requested_capability {
                AgentCapabilities::SendSensorData => {
                    let property = self.try_get_puller_property(registration_request.connection_protocol())?;
                    let mut puller = property.as_boxed_server_puller();
                    puller.request_start().map_err(|e| FeagiAgentServerError::ConnectionFailed(e.to_string()))?;
                    self.active_sensor_servers.push(puller);
                    /*
                    endpoints.insert(
                        AgentCapabilities::SendSensorData,
                        self.build_capability_endpoint(registration_request.connection_protocol(), AgentCapabilities::SendSensorData),
                    );

                     */
                }
                AgentCapabilities::ReceiveMotorData => {
                    let property = self.try_get_publisher_property(registration_request.connection_protocol())?;
                    let mut publisher = property.as_boxed_server_publisher();
                    publisher.request_start().map_err(|e| FeagiAgentServerError::ConnectionFailed(e.to_string()))?;
                    self.active_motor_servers.push(publisher);
                    /*
                    endpoints.insert(
                        AgentCapabilities::ReceiveMotorData,
                        self.build_capability_endpoint(registration_request.connection_protocol(), AgentCapabilities::ReceiveMotorData),
                    );

                     */
                }
                AgentCapabilities::ReceiveNeuronVisualizations => {
                    let property = self.try_get_publisher_property(registration_request.connection_protocol())?;
                    let mut publisher = property.as_boxed_server_publisher();
                    publisher.request_start().map_err(|e| FeagiAgentServerError::ConnectionFailed(e.to_string()))?;
                    self.active_visualizer_servers.push(publisher);
                    /*
                    endpoints.insert(
                        AgentCapabilities::ReceiveNeuronVisualizations,
                        self.build_capability_endpoint(registration_request.connection_protocol(), AgentCapabilities::ReceiveNeuronVisualizations),
                    );

                     */
                }
            }
        }



        self.registered_agents.insert(session_id.clone(), (
            registration_request.agent_descriptor().clone(),
            registration_request.requested_capabilities().to_vec()));

        // TODO clearly there needs to be some sort of notification, but not like this
        /*
        if let Some(hook) = &self.registration_hook {
            let dr = registration_request.device_registrations().cloned();
            let capabilities = registration_request.requested_capabilities().to_vec();
            hook(
                session_id.clone(),
                registration_request.agent_descriptor().clone(),
                capabilities,
                dr,
            );
        }

         */

        Ok(RegistrationResponse::Success(session_id.clone(), endpoints))
    }

    fn try_get_puller_property(&mut self, wanted_protocol: &TransportProtocolImplementation) -> Result<Box<dyn FeagiServerPullerProperties>, FeagiAgentServerError> {
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
        Err(FeagiAgentServerError::InitFail("Missing required protocol puller".to_string()))
    }

    fn try_get_publisher_property(&mut self, wanted_protocol: &TransportProtocolImplementation) -> Result<Box<dyn FeagiServerPublisherProperties>, FeagiAgentServerError> {
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
        Err(FeagiAgentServerError::InitFail("Missing required protocol publisher".to_string()))
    }

    pub fn is_session_registered(&self, session_id: &SessionID) -> bool {
        self.registered_agents.contains_key(session_id)
    }



    /*
    pub fn default_protocol(&self) -> Result<TransportProtocolImplementation, FeagiAgentServerError> {
        self.parse_protocol(&self.config.transports.default)
    }

     */

    // TODO no, this handler should not be reading config files from the filesystem
    /*
    pub fn build_capability_endpoint(
        &self,
        protocol: &TransportProtocolImplementation,
        capability: AgentCapabilities,
    ) -> String {
        match protocol {
            TransportProtocolImplementation::ZMQ => {
                let host = &self.config.zmq.host;
                let port = match capability {
                    AgentCapabilities::SendSensorData => self.config.ports.zmq_sensory_port,
                    AgentCapabilities::ReceiveMotorData => self.config.ports.zmq_motor_port,
                    AgentCapabilities::ReceiveNeuronVisualizations => self.config.ports.zmq_visualization_port,
                };
                Self::format_tcp_endpoint(host, port)
            }
            TransportProtocolImplementation::WebSocket => {
                let host = &self.config.websocket.host;
                let port = match capability {
                    AgentCapabilities::SendSensorData => self.config.websocket.sensory_port,
                    AgentCapabilities::ReceiveMotorData => self.config.websocket.motor_port,
                    AgentCapabilities::ReceiveNeuronVisualizations => self.config.websocket.visualization_port,
                };
                Self::format_ws_endpoint(host, port)
            }
        }
    }

     */

    fn format_tcp_endpoint(host: &str, port: u16) -> String {
        if host.contains(':') {
            format!("tcp://[{host}]:{port}")
        } else {
            format!("tcp://{host}:{port}")
        }
    }

    fn format_ws_endpoint(host: &str, port: u16) -> String {
        if host.contains(':') {
            format!("ws://[{host}]:{port}")
        } else {
            format!("ws://{host}:{port}")
        }
    }

    fn new_session_id(&self) -> SessionID {
        loop {
            let session_id = SessionID::new_random();
            if !self.registered_agents.contains_key(&session_id) {
                return session_id;
            }
        }
    }

    // TODO why is the registration handler saving the connectomE??????
    /*
    /// Persist a connectome snapshot to disk using FEAGI serialization.
    pub fn save_connectome_snapshot<P: AsRef<Path>>(
        &self,
        snapshot: &ConnectomeSnapshot,
        path: P,
    ) -> Result<(), FeagiAgentServerError> {
        save_connectome(snapshot, path.as_ref()).map_err(|e| {
            FeagiAgentServerError::PersistenceFailed(format!(
                "Failed to save connectome to {}: {e}",
                path.as_ref().display()
            ))
        })
    }

    /// Persist the current runtime genome to disk.
    pub fn save_genome<P: AsRef<Path>>(
        &self,
        genome: &RuntimeGenome,
        path: P,
    ) -> Result<(), FeagiAgentServerError> {
        save_genome_to_file(genome, path.as_ref()).map_err(|e| {
            FeagiAgentServerError::PersistenceFailed(format!(
                "Failed to save genome to {}: {e}",
                path.as_ref().display()
            ))
        })
    }

     */


    //endregion

    //endregion

}

