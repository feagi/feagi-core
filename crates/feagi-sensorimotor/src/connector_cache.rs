use crate::caching::MotorDeviceCache;
use crate::caching::SensorDeviceCache;
use crate::configuration::jsonable::JSONInputOutputDefinition;
use crate::feedbacks::{FeedBackRegistration, FeedbackRegistrar, FeedbackRegistrationTargets};
use feagi_structures::FeagiDataError;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

// TODO this file may be redudant, we may want to clear it

fn lock_recover<'a, T>(mutex: &'a Mutex<T>) -> MutexGuard<'a, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            mutex.clear_poison();
            poisoned.into_inner()
        }
    }
}

#[derive(Debug)]
pub struct ConnectorCache {
    sensor_cache: Arc<Mutex<SensorDeviceCache>>,
    motor_cache: Arc<Mutex<MotorDeviceCache>>,
    feedback_registrar: FeedbackRegistrar,
}

impl Default for ConnectorCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectorCache {
    pub fn new() -> Self {
        ConnectorCache {
            sensor_cache: Arc::new(Mutex::new(SensorDeviceCache::new())),
            motor_cache: Arc::new(Mutex::new(MotorDeviceCache::new())),
            feedback_registrar: FeedbackRegistrar::new(),
        }
    }

    pub fn get_sensor_cache(&self) -> MutexGuard<'_, SensorDeviceCache> {
        lock_recover(&self.sensor_cache)
    }

    pub fn get_sensor_cache_ref(&self) -> Arc<Mutex<SensorDeviceCache>> {
        self.sensor_cache.clone()
    }

    pub fn get_motor_cache(&self) -> MutexGuard<'_, MotorDeviceCache> {
        lock_recover(&self.motor_cache)
    }

    pub fn get_motor_cache_ref(&self) -> Arc<Mutex<MotorDeviceCache>> {
        self.motor_cache.clone()
    }

    pub fn register_feedback(
        &mut self,
        feedback: FeedBackRegistration,
        target: FeedbackRegistrationTargets,
    ) -> Result<(), FeagiDataError> {
        let sensors = self.get_sensor_cache_ref();
        let motors = self.get_motor_cache_ref();

        feedback.try_registering_feedback_and_save(
            &mut self.feedback_registrar,
            sensors,
            motors,
            target,
        )?;
        Ok(())
    }

    pub fn export_device_registrations_as_config_json(
        &self,
    ) -> Result<serde_json::Value, FeagiDataError> {
        let mut output = JSONInputOutputDefinition::new();
        self.get_sensor_cache()
            .export_to_input_definition(&mut output)?;
        self.get_motor_cache()
            .export_to_output_definition(&mut output)?;
        output.set_feedbacks(self.feedback_registrar.clone());
        serde_json::to_value(output)
            .map_err(|err| FeagiDataError::SerializationError(err.to_string()))
    }

    pub fn import_device_registrations_as_config_json(
        &mut self,
        json: serde_json::Value,
    ) -> Result<(), FeagiDataError> {
        // NOTE: Wipes all registered devices
        let definition: JSONInputOutputDefinition = serde_json::from_value(json)
            .map_err(|err| FeagiDataError::DeserializationError(err.to_string()))?;
        self.get_motor_cache()
            .import_from_output_definition(&definition)?;
        self.get_sensor_cache()
            .import_from_input_definition(&definition)?;
        self.feedback_registrar = definition.get_feedbacks().clone();
        // Re-activate imported feedback registrations by wiring callbacks against the newly
        // imported motor/sensor caches.
        self.feedback_registrar
            .reload_all_from_self(self.get_sensor_cache_ref(), self.get_motor_cache_ref())?;
        Ok(())
    }
}

impl fmt::Display for ConnectorCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ConnectorAgent")
    }
}
