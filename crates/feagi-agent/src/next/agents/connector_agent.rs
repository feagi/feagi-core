use std::sync::{Arc, Mutex, MutexGuard};
use feagi_sensorimotor::caching::{MotorDeviceCache, SensorDeviceCache};
use feagi_sensorimotor::ConnectorCache;
use feagi_sensorimotor::feedbacks::{FeedBackRegistration, FeedbackRegistrationTargets};
use feagi_structures::FeagiDataError;
use crate::next::common::{AgentCapabilities, AgentConnectionState, AgentID, FeagiAgent, FeagiConnectionConfiguration};

pub struct ConnectorAgent {
    agent_id: AgentID,
    current_connection_state: AgentConnectionState,
    connector_cache: ConnectorCache
}

impl ConnectorAgent {
    pub fn new_empty(agent_id: AgentID) -> Self {
        ConnectorAgent {
            agent_id,
            current_connection_state: AgentConnectionState::Disconnected,
            connector_cache: ConnectorCache::new()
        }
    }

    pub fn new_from_device_registration_json(agent_id: AgentID, json: serde_json::Value) -> Result<Self, FeagiDataError> {
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
                "Cannot reload device registrations while running!".into_string()
            ))
        }
        self.connector_cache.import_device_registrations_as_config_json(json)
    }


}

impl FeagiAgent for ConnectorAgent {
    fn agent_id(&self) -> &AgentID {
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

    fn connect_to_feagi(&mut self, connection_configuration: FeagiConnectionConfiguration) -> Result<(), FeagiDataError> {
        if self.current_connection_state().is_active() {
            return Err(FeagiDataError::ResourceLockedWhileRunning(
                "Cannot try to connect to FEAGI while a connection is active!".into_string()
            ))
        }

        todo!()
    }

    fn disconnect(&mut self) -> Result<(), FeagiDataError> {
        if !self.current_connection_state().is_active() {
            return Ok(()) // Already disconnected lol
        }



        todo!()
    }
}