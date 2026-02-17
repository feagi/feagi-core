use serde_json::Value;

use crate::FeagiAgentClientError;

/// ConnectorAgent for device registration and sensor/motor caches (SDK flows).
#[derive(Debug)]
pub struct ConnectorAgent {
    agent_descriptor: AgentDescriptor,
    device_registrations: Value,
    #[cfg(feature = "sdk-io")]
    cache: feagi_sensorimotor::ConnectorCache,
}

impl ConnectorAgent {
    pub fn new_from_device_registration_json(
        agent_descriptor: AgentDescriptor,
        device_registrations: Value,
    ) -> Result<Self, FeagiAgentClientError> {
        if !device_registrations.is_object() {
            return Err(FeagiAgentClientError::UnableToDecodeReceivedData(
                "device_registrations must be a JSON object".to_string(),
            ));
        }
        Ok(Self {
            agent_descriptor,
            device_registrations,
            #[cfg(feature = "sdk-io")]
            cache: feagi_sensorimotor::ConnectorCache::new(),
        })
    }

    pub fn new_empty(agent_descriptor: AgentDescriptor) -> Self {
        Self {
            agent_descriptor,
            device_registrations: serde_json::json!({}),
            #[cfg(feature = "sdk-io")]
            cache: feagi_sensorimotor::ConnectorCache::new(),
        }
    }

    pub fn set_device_registrations_from_json(
        &mut self,
        device_registrations: Value,
    ) -> Result<(), FeagiAgentClientError> {
        if !device_registrations.is_object() {
            return Err(FeagiAgentClientError::UnableToDecodeReceivedData(
                "device_registrations must be a JSON object".to_string(),
            ));
        }
        self.device_registrations = device_registrations;
        Ok(())
    }

    /// Returns device registrations. When sdk-io is enabled, returns the live export from the
    /// sensor/motor cache (so vision_register, gaze_register, etc. are reflected). Otherwise
    /// returns the stored device_registrations value.
    pub fn get_device_registration_json(&self) -> Result<Value, FeagiAgentClientError> {
        #[cfg(feature = "sdk-io")]
        {
            self.cache
                .export_device_registrations_as_config_json()
                .map_err(|e| FeagiAgentClientError::UnableToDecodeReceivedData(e.to_string()))
        }
        #[cfg(not(feature = "sdk-io"))]
        {
            Ok(self.device_registrations.clone())
        }
    }

    pub fn agent_descriptor(&self) -> &AgentDescriptor {
        &self.agent_descriptor
    }

    #[cfg(feature = "sdk-io")]
    pub fn get_sensor_cache(
        &self,
    ) -> std::sync::MutexGuard<'_, feagi_sensorimotor::caching::SensorDeviceCache> {
        self.cache.get_sensor_cache()
    }

    #[cfg(feature = "sdk-io")]
    pub fn get_sensor_cache_ref(
        &self,
    ) -> std::sync::Arc<std::sync::Mutex<feagi_sensorimotor::caching::SensorDeviceCache>> {
        self.cache.get_sensor_cache_ref()
    }

    #[cfg(feature = "sdk-io")]
    pub fn get_motor_cache(
        &self,
    ) -> std::sync::MutexGuard<'_, feagi_sensorimotor::caching::MotorDeviceCache> {
        self.cache.get_motor_cache()
    }

    #[cfg(feature = "sdk-io")]
    pub fn register_feedback(
        &mut self,
        feedback: feagi_sensorimotor::feedbacks::FeedBackRegistration,
        target: feagi_sensorimotor::feedbacks::FeedbackRegistrationTargets,
    ) -> Result<(), feagi_structures::FeagiDataError> {
        self.cache.register_feedback(feedback, target)
    }
}

impl Clone for ConnectorAgent {
    fn clone(&self) -> Self {
        Self {
            agent_descriptor: self.agent_descriptor.clone(),
            device_registrations: self.device_registrations.clone(),
            #[cfg(feature = "sdk-io")]
            cache: feagi_sensorimotor::ConnectorCache::new(),
        }
    }
}

pub mod base;
pub mod registration;
#[cfg(feature = "sdk-io")]
pub mod types;

#[cfg(feature = "sdk")]
pub mod motor;
#[cfg(feature = "sdk")]
pub mod sensory;

pub use crate::registration::AgentDescriptor;
pub use crate::registration::AuthToken;
