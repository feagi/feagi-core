use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use feagi_io::next::FeagiNetworkError;
use feagi_io::next::implementations::zmq::{
    FEAGIZMQClientPusherProperties, FEAGIZMQClientSubscriberProperties,
};
use feagi_io::next::traits_and_enums::client::{
    FeagiClient, FeagiClientPusher, FeagiClientRequester, FeagiClientRequesterProperties,
    FeagiClientSubscriber, client_shared::FeagiClientConnectionStateChange,
};
use feagi_io::next::traits_and_enums::client::{
    FeagiClientPusherProperties as _, FeagiClientSubscriberProperties as _,
};
use feagi_sensorimotor::caching::{MotorDeviceCache, SensorDeviceCache};
use feagi_sensorimotor::ConnectorCache;
use feagi_sensorimotor::feedbacks::{FeedBackRegistration, FeedbackRegistrationTargets};
use feagi_structures::FeagiDataError;
use crate::next::client::communication::auth_request::AuthRequest;
use crate::next::client::communication::registration_request::RegistrationRequest;
use crate::next::common::{
    AgentCapabilities, AgentConnectionState, AgentDescriptor, AuthToken, FeagiAgent,
};

pub struct ConnectorAgent {
    agent_id: AgentDescriptor,
    current_connection_state: AgentConnectionState,
    connector_cache: ConnectorCache,
    sensor_sender: Option<Box<dyn FeagiClientPusher>>,
    motor_receiver: Option<Box<dyn FeagiClientSubscriber>>,

}

impl ConnectorAgent {
    pub fn new_empty(agent_id: AgentDescriptor) -> Self {
        ConnectorAgent {
            agent_id,
            current_connection_state: AgentConnectionState::Disconnected,
            connector_cache: ConnectorCache::new(),
            sensor_sender: None,
            motor_receiver: None
        }
    }

    pub fn new_from_device_registration_json(agent_id: AgentDescriptor, json: serde_json::Value) -> Result<Self, FeagiDataError> {
        let mut agent = Self::new_empty(agent_id);
        agent.set_device_registrations_from_json(json)?;
        Ok(agent)
    }

    pub fn get_sensor_cache(&self) -> MutexGuard<'_, SensorDeviceCache> {
        self.connector_cache.get_sensor_cache()
    }

    pub fn get_sensor_cache_ref(&self) -> Arc<Mutex<SensorDeviceCache>> {
        self.connector_cache.get_sensor_cache_ref()
    }

    pub fn get_motor_cache(&self) -> MutexGuard<'_, MotorDeviceCache> {
        self.connector_cache.get_motor_cache()
    }

    pub fn get_motor_cache_ref(&self) -> Arc<Mutex<MotorDeviceCache>> {
        self.connector_cache.get_motor_cache_ref()
    }

    pub fn get_device_registration_json(&self) -> Result<serde_json::Value, FeagiDataError> {
        self.connector_cache.export_device_registrations_as_config_json()
    }

    pub fn register_feedback(
        &mut self,
        feedback: FeedBackRegistration,
        target: FeedbackRegistrationTargets,
    ) -> Result<(), FeagiDataError> {
        self.connector_cache.register_feedback(feedback, target)
    }

    pub fn set_device_registrations_from_json(&mut self, json: serde_json::Value) -> Result<(), FeagiDataError> {
        if self.current_connection_state().is_active() {
            return Err(FeagiDataError::ResourceLockedWhileRunning(
                "Cannot reload device registrations while running!".to_string()
            ))
        }
        self.connector_cache.import_device_registrations_as_config_json(json)
    }

    pub fn send_sensor_data(&self) -> Result<(), FeagiDataError> {
        if !self.current_connection_state().is_active() {
            return Err(FeagiDataError::ResourceLockedWhileRunning(
                "Cannot reload device registrations while running!".to_string()
            ))
        }
        if self.sensor_sender.is_none() {
            return Err(FeagiDataError::ResourceLockedWhileRunning(
                "Cannot reload device registrations while running!".to_string()
            ))
        }
        let Some(sender) = self.sensor_sender.as_ref() else {
            return Err(FeagiDataError::ResourceLockedWhileRunning(
                "Cannot reload device registrations while running!".to_string()
            ));
        };

        let mut sensor_cache = self.get_sensor_cache();
        sensor_cache.encode_neurons_to_bytes();
        let bytes = sensor_cache.get_feagi_byte_container();
        sender.push_data(bytes.get_byte_ref());
        Ok(())
    }

    

}

impl FeagiAgent for ConnectorAgent {
    fn agent_id(&self) -> &AgentDescriptor {
        &self.agent_id
    }

    fn current_connection_state(&self) -> &AgentConnectionState {
        &self.current_connection_state
    }

    fn agent_capabilities(&self) -> &[AgentCapabilities] {
        &[
            AgentCapabilities::SendSensorData,
            AgentCapabilities::ReceiveMotorData
        ]
    }

    fn connect_to_feagi(
        &mut self,
        host: String,
        requester_properties: Box<dyn FeagiClientRequesterProperties>,
        agent_descriptor: AgentDescriptor,
    ) -> Result<(), FeagiDataError> {
        if self.current_connection_state().is_active() {
            return Err(FeagiDataError::ResourceLockedWhileRunning(
                "Cannot try to connect to FEAGI while a connection is active!".parse().unwrap()
            ))
        }

        self.current_connection_state = AgentConnectionState::Connecting;

        let mut requester = requester_properties.build(Box::new(|_change: FeagiClientConnectionStateChange| {
            // TODO: track requester connection state changes
        }));

        requester.connect(&host)
            .map_err(|e| FeagiDataError::InternalError(format!("Failed to connect requester: {}", e)))?;

        self.current_connection_state = AgentConnectionState::Authenticating;

        let auth_request = AuthRequest::new(
            &agent_descriptor,
            &AuthToken::new([0; 32]), // TODO: actual token
        );
        let auth_bytes = serde_json::to_vec(&auth_request.to_json())
            .map_err(|e| FeagiDataError::SerializationError(e.to_string()))?;
        requester.send_request(&auth_bytes)
            .map_err(|e| FeagiDataError::InternalError(format!("Failed to send auth request: {}", e)))?;

        let phase1_data = loop {
            if let Some(data) = requester.try_poll_receive()
                .map_err(|e| FeagiDataError::InternalError(format!("Failed to read phase 1 response: {}", e)))?
            {
                break data.to_vec();
            }
            std::thread::sleep(Duration::from_millis(1));
        };

        let phase1_json: serde_json::Value = serde_json::from_slice(&phase1_data)
            .map_err(|e| FeagiDataError::DeserializationError(e.to_string()))?;
        let connection_id = phase1_json.get("connection_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FeagiDataError::DeserializationError("Missing connection_id in phase 1 response".to_string()))?
            .to_string();

        let registration_request = RegistrationRequest {
            connection_id,
            data: serde_json::json!({}),
            capabilities: self.agent_capabilities().to_vec(),
        };
        let registration_bytes = serde_json::to_vec(&registration_request)
            .map_err(|e| FeagiDataError::SerializationError(e.to_string()))?;
        requester.send_request(&registration_bytes)
            .map_err(|e| FeagiDataError::InternalError(format!("Failed to send registration request: {}", e)))?;

        let phase2_data = loop {
            if let Some(data) = requester.try_poll_receive()
                .map_err(|e| FeagiDataError::InternalError(format!("Failed to read phase 2 response: {}", e)))?
            {
                break data.to_vec();
            }
            std::thread::sleep(Duration::from_millis(1));
        };

        let phase2_json: serde_json::Value = serde_json::from_slice(&phase2_data)
            .map_err(|e| FeagiDataError::DeserializationError(e.to_string()))?;
        let endpoints = Self::parse_phase2_endpoints(&phase2_json)?;

        let capabilities = self.agent_capabilities().to_vec();
        for capability in capabilities {
            let endpoint = endpoints.get(&capability).ok_or_else(|| {
                FeagiDataError::DeserializationError(format!(
                    "Missing endpoint for capability {:?}",
                    capability
                ))
            })?;

            match capability {
                AgentCapabilities::SendSensorData => {
                    let mut sensor_sender = Box::new(FEAGIZMQClientPusherProperties::new(endpoint.to_string()))
                        .build(Box::new(|_change: FeagiClientConnectionStateChange| {
                            // TODO: track sensor sender connection state changes
                        }));
                    sensor_sender.connect(endpoint)
                        .map_err(|e| FeagiDataError::InternalError(format!("Failed to connect sensor sender: {}", e)))?;
                    self.sensor_sender = Some(sensor_sender);
                }
                AgentCapabilities::ReceiveMotorData => {
                    let mut motor_receiver = Box::new(FEAGIZMQClientSubscriberProperties::new(endpoint.to_string()))
                        .build(Box::new(|_change: FeagiClientConnectionStateChange| {
                            // TODO: track motor receiver connection state changes
                        }));
                    motor_receiver.connect(endpoint)
                        .map_err(|e| FeagiDataError::InternalError(format!("Failed to connect motor receiver: {}", e)))?;
                    self.motor_receiver = Some(motor_receiver);
                }
                AgentCapabilities::ReceiveNeuronVisualizations => {
                    // TODO: initialize visualization stream
                }
            }
        }

        self.current_connection_state = AgentConnectionState::Running;
        Ok(())
    }

    fn disconnect(&mut self) {
        // TODO: close client connections and update state
        self.current_connection_state = AgentConnectionState::Disconnected;
    }

}

impl ConnectorAgent {
    fn parse_phase2_endpoints(
        phase2_json: &serde_json::Value,
    ) -> Result<HashMap<AgentCapabilities, String>, FeagiDataError> {
        let endpoints = phase2_json.get("endpoints")
            .and_then(|v| v.as_object())
            .ok_or_else(|| FeagiDataError::DeserializationError("Missing or invalid endpoints".to_string()))?;

        let mut parsed = HashMap::new();
        for (key, value) in endpoints {
            let endpoint = value.as_str()
                .ok_or_else(|| FeagiDataError::DeserializationError(format!("Endpoint for {} must be a string", key)))?;
            let capability = match key.as_str() {
                "send_sensor_data" => AgentCapabilities::SendSensorData,
                "receive_motor_data" => AgentCapabilities::ReceiveMotorData,
                "receive_neuron_visualizations" => AgentCapabilities::ReceiveNeuronVisualizations,
                _ => {
                    return Err(FeagiDataError::DeserializationError(format!(
                        "Unknown capability key: {}",
                        key
                    )));
                }
            };
            parsed.insert(capability, endpoint.to_string());
        }

        Ok(parsed)
    }
}