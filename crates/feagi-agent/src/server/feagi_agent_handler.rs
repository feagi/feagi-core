use std::cmp::PartialEq;
use std::collections::{HashMap, HashSet};
use feagi_io::core::traits_and_enums::FeagiEndpointState;
use feagi_io::core::traits_and_enums::server::{FeagiServerPublisher, FeagiServerPublisherProperties, FeagiServerPuller, FeagiServerPullerProperties, FeagiServerRouter, FeagiServerRouterProperties};
use feagi_serialization::{FeagiByteContainer, SessionID};
use crate::feagi_agent_server_error::FeagiAgentServerError;
use crate::registration::{AgentCapabilities, AgentDescriptor, RegistrationRequest, RegistrationResponse};
use crate::server::auth::AgentAuth;

pub struct FeagiAgentHandler {
    agent_auth_backend: Box<dyn AgentAuth>,
    registered_agents: HashMap<SessionID, (AgentDescriptor, Vec<AgentCapabilities>)>,


    available_publishers: Vec<Box<dyn FeagiServerPublisherProperties>>,
    available_pullers: Vec<Box<dyn FeagiServerPullerProperties>>,

    active_motor_servers: Vec<Box<dyn FeagiServerPublisher>>,
    active_visualizer_servers: Vec<Box<dyn FeagiServerPublisher>>,
    active_sensor_servers: Vec<Box<dyn FeagiServerPuller>>,
    active_registration_servers: Vec<Box<dyn FeagiServerRouter>>,

}

impl FeagiAgentHandler {

    //region Add Servers

    pub fn add_and_start_registration_server(&mut self, router_property: Box<dyn FeagiServerRouterProperties>) -> Result<(), FeagiAgentServerError> {
        // TODO check for collisions
        let mut router = router_property.as_boxed_server_router();
        router.request_start()
            .map_err(|e| FeagiAgentServerError::InitFail(e.to_string()))?;
        self.active_registration_servers.push(router);
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

    pub fn poll_registration_servers(&mut self) -> Result<(), FeagiAgentServerError> {
        for i in 0..self.active_registration_servers.len() {
            match self.active_registration_servers[i].poll() {
                FeagiEndpointState::Inactive => {
                    continue; // Do nothing
                }
                FeagiEndpointState::Pending => {
                    continue; // Do nothing
                }
                FeagiEndpointState::ActiveWaiting => {
                    continue; // Do nothing
                }
                FeagiEndpointState::ActiveHasData => {
                    // NOTE: Routers ignore the session ID in the bytes!
                    self.poll_registration_server(i)?;
                }
                FeagiEndpointState::Errored(_) => {
                    self.active_registration_servers[i].confirm_error_and_close().map_err(
                        |e| FeagiAgentServerError::ConnectionFailed(e.to_string())
                    )?;
                    continue; // TODO we should do more
                }
            }
        }
        Ok(())
    }

    /*
    pub fn poll_sensory_servers(&mut self) -> Option<&FeagiByteContainer> {

    }

    pub fn poll_motor_servers(&mut self, Option<&FeagiByteContainer>) -> Result<> {

    }

    pub fn poll_visualization_servers(&mut self, Option<&FeagiByteContainer>) -> Result<> {

    }

     */

    //endregion


    //region Internal

    //region Registration

    fn poll_registration_server(&mut self, server_index: usize) -> Result<(), FeagiAgentServerError> {
        let (session_id, data) = self.active_registration_servers[server_index]
            .consume_retrieved_request()
            .map_err(|e| FeagiAgentServerError::UnableToDecodeReceivedData(e.to_string()))?;

        if !self.registered_agents.contains_key(&session_id) {
            // Agent is unknown, we need to register it
            let registration_request: RegistrationRequest = serde_json::from_slice(&data)
                .map_err(|e| FeagiAgentServerError::UnableToDecodeReceivedData(
                    format!("Failed to parse RegistrationRequest: {}", e)
                ))?;
            let registration_response = self.verify_agent_request_and_make_response(&session_id, registration_request)?;

            let response_bytes = serde_json::to_vec(&registration_response)
                .map_err(|e| FeagiAgentServerError::UnableToSendData(
                    format!("Failed to serialize RegistrationResponse: {}", e)
                ))?;

            self.active_registration_servers[server_index]
                .publish_response(session_id, &response_bytes)
                .map_err(|e| FeagiAgentServerError::UnableToSendData(e.to_string()))?;

            Ok(())
        }
        else {
            // We know this Agent. What does it want?
            // TODO How do we signal to FEAGI various commands?

            Ok(())
        }
    }

    fn verify_agent_request_and_make_response(&mut self, session_id: &SessionID, registration_request: RegistrationRequest) -> Result<RegistrationResponse, FeagiAgentServerError> {

        let verify_auth = self.agent_auth_backend.verify_agent_allowed_to_connect(&registration_request);  // TODO how do we make this non blocking????
        if verify_auth.is_err() {
            return Ok(RegistrationResponse::FailedInvalidAuth)
        }

        // TODO verify no duplicates!

        // TODO func for starting new servers and registering

        self.registered_agents.insert(session_id.clone(), (
            registration_request.agent_descriptor().clone(),
            registration_request.requested_capabilities().to_vec()));

        return Ok(RegistrationResponse::Success(session_id.clone()));
    }


    //endregion



    //endregion

}

