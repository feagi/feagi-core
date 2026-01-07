use crate::caching::MotorDeviceCache;
use crate::caching::SensorDeviceCache;
use feagi_structures::FeagiDataError;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};
use crate::configuration::jsonable::JSONInputOutputDefinition;

#[derive(Debug)]
pub struct ConnectorAgent {
    sensor_cache: Arc<Mutex<SensorDeviceCache>>,
    motor_cache: Arc<Mutex<MotorDeviceCache>>,
}

impl Default for ConnectorAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectorAgent {
    pub fn new() -> Self {
        
        let sensors = Arc::new(Mutex::new(SensorDeviceCache::new()));
        ConnectorAgent {
            sensor_cache: sensors.clone(),
            motor_cache: Arc::new(Mutex::new(MotorDeviceCache::new(sensors))),
        }
    }

    pub fn get_sensor_cache(&self) -> MutexGuard<'_, SensorDeviceCache> {
        self.sensor_cache.lock().unwrap()
    }

    pub fn get_sensor_cache_ref(&self) -> Arc<Mutex<SensorDeviceCache>> {
        self.sensor_cache.clone()
    }

    pub fn get_motor_cache(&self) -> MutexGuard<'_, MotorDeviceCache> {
        self.motor_cache.lock().unwrap()
    }

    pub fn get_motor_cache_ref(&self) -> Arc<Mutex<MotorDeviceCache>> {
        self.motor_cache.clone()
    }


    pub fn export_device_registrations_as_config_json(
        &self,
    ) -> Result<serde_json::Value, FeagiDataError> {
        let mut output = JSONInputOutputDefinition::new();
        self.get_sensor_cache().export_to_input_definition(&mut output)?;
        self.get_motor_cache().export_to_output_definition(&mut output)?;
        Ok(serde_json::to_value(output).unwrap())
    }

    pub fn import_device_registrations_as_config_json(
        &mut self,
        json: serde_json::Value,
    ) -> Result<(), FeagiDataError> {
        // NOTE: Wipes all registered devices
        let definition: JSONInputOutputDefinition = serde_json::from_value(json).map_err(|err | FeagiDataError::DeserializationError(err.to_string()))?;
        self.get_motor_cache().import_from_output_definition(&definition)?;
        self.get_sensor_cache().import_from_input_definition(&definition)?;
        Ok(())
    }

}

impl fmt::Display for ConnectorAgent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ConnectorAgent")
    }
}
