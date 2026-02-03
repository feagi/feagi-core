use std::collections::{HashMap, HashSet};
use feagi_io::core::::zmq::{FeagiZmqServerPublisher, FEAGIZMQServerPuller};
use feagi_io::core::::server::{FeagiServerPublisher, FeagiServerPuller, FeagiServerRouter};
use feagi_serialization::{FeagiByteContainer, SessionID};
use crate::FeagiAgentClientError;
use crate::registration::{AgentCapabilities, RegistrationResponse};
use crate::server::{MotorServer, SensoryServer};

// TODO temp
const MOTOR_ENDPOINT: &str = "127.0.0.1:5000";
const SENSORY_ENDPOINT: &str = "127.0.0.1:5001";

pub struct AgentRegistry {
    io_server: Option<Box<dyn FeagiServerRouter>>,
    connected_sessions: HashSet<SessionID>,

}

impl AgentRegistry {
    pub async fn new() -> Self {

        // TODO hardcoded stuff for now

        let motor_server = MotorServer::new(
            Box::new(FeagiZmqServerPublisher::new(
                MOTOR_ENDPOINT.to_string(),
                Box::new(|_| {}),
            ).unwrap())
        ).await.unwrap();

        let sensory_server = SensoryServer::new(
            Box::new(FEAGIZMQServerPuller::new(
                SENSORY_ENDPOINT.to_string(),
                Box::new(|_| {}),
            ).unwrap())
        ).await.unwrap();

        AgentRegistry {
            io_server: None,
            motor_server,
            sensory_server,
            connected_sessions: HashSet::new(),
        }

    }

    pub async fn set_registry_endpoint(&mut self, mut router: Box<dyn FeagiServerRouter>) -> Result<(), FeagiAgentClientError> {
        if self.io_server.is_none() {
            router.start().await.map_err(
                |e| FeagiAgentClientError::GeneralFailure(format!("{:?}", e))
            )?;
            self.io_server = Some(router);
            Ok(())
        }
        else {
            Err(FeagiAgentClientError::GeneralFailure("Cannot set a router when one is already running!".to_string()))
        }
    }

    pub async fn clear_registry_endpoint(&mut self)  -> Result<(), FeagiAgentClientError> {
        if self.io_server.is_none() {
            return Err(FeagiAgentClientError::GeneralFailure("There is no router to remove!".to_string()));
        }
        let server: &mut Box<dyn FeagiServerRouter> = self.io_server.as_mut().unwrap();
        server.stop().await.map_err(
            |e| FeagiAgentClientError::GeneralFailure(format!("{:?}", e))
        )?;
        self.io_server = None;
        Ok(())
    }

    pub async fn poll_registry_endpoint(&mut self)  -> Result<(), FeagiAgentClientError> {
        if self.io_server.is_none() {
            return Err(FeagiAgentClientError::GeneralFailure("There is no router to poll!".to_string()));
        }

        let server: &mut Box<dyn FeagiServerRouter> = self.io_server.as_mut().unwrap();
        match server.try_poll_receive().await {
            Ok((session_id, request)) => {

                if self.connected_sessions.contains(&session_id) {
                    self.send_response(session_id, RegistrationResponse::AlreadyRegistered).await?;
                    return Ok(())
                }
                self.connected_sessions.insert(session_id);

                // TODO hardcoded for now
                let mut response_abilities: HashMap<AgentCapabilities, String> = Default::default();
                response_abilities.insert(AgentCapabilities::SendSensorData, SENSORY_ENDPOINT.to_string());
                response_abilities.insert(AgentCapabilities::ReceiveMotorData, MOTOR_ENDPOINT.to_string());

                let response: RegistrationResponse = RegistrationResponse::Success(
                    session_id,
                    response_abilities
                );

                let response_bytes = serde_json::to_vec(&response).unwrap();

                server.send_response(session_id, &response_bytes).await
                    .map_err(|e| FeagiAgentClientError::GeneralFailure(format!("{:?}", e)))?;

                Ok(())
            }
            Err(e) => {
                // Some implementations return errors when no data is available
                let err_str = e.to_string();
                if !err_str.contains("No clients") && !err_str.contains("No data") {
                    return Err(FeagiAgentClientError::GeneralFailure("[SERVER] Error polling: {}".to_string()))
                };
                Ok(())
            }
        }
    }

    /// Poll motor socket. Some implementations this does nothing, but others need to do this to stay alive
    pub async fn poll_motor_endpoints(&mut self)  -> Result<(), FeagiAgentClientError> {
        self.motor_server.poll().await?;
        Ok(())
    }

    /// Send motor data to motor socket
    pub async fn send_motor_bytes(&mut self, motor_data_bytes: &FeagiByteContainer) -> Result<(), FeagiAgentClientError> {
        self.motor_server.publish(motor_data_bytes).await
    }

    // Await new sensor bytes, and write it to the byte container
    pub async fn get_sensor_bytes(&mut self, sensory_data_bytes: &mut FeagiByteContainer)  -> Result<(), FeagiAgentClientError> {
        self.sensory_server.poll_for_sensor_data(sensory_data_bytes).await?;
        Ok(())
    }

    pub fn add_agent_endpoints(&mut self, session_id: SessionID) { // todo vector of server props
        todo!()
    }

    pub fn remove_agent_endpoints(&mut self, session_id: SessionID) {
        todo!()
    }

    async fn send_response(&mut self, session_id: SessionID, registration_response: RegistrationResponse) -> Result<(), FeagiAgentClientError> {
        if self.io_server.is_none() {
            return Err(FeagiAgentClientError::GeneralFailure("There is no router to respond with!".to_string()));
        }
        let server: &mut Box<dyn FeagiServerRouter> = self.io_server.as_mut().unwrap();

        let json_bytes = serde_json::to_vec(&registration_response)
            .map_err(|e| FeagiAgentClientError::GeneralFailure(format!("Failed to serialize response: {}", e)))?;
        server.send_response(session_id, &json_bytes)
            .await.map_err(|e| FeagiAgentClientError::GeneralFailure(format!("{:?}", e)))?;
        Ok(())
    }








}