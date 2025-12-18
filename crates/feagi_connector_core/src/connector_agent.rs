use std::sync::{Arc, Mutex, MutexGuard};
use std::fmt;
use feagi_data_structures::FeagiDataError;
use crate::caching::MotorDeviceCache;
use crate::caching::SensorDeviceCache;

#[derive(Debug)]
pub struct ConnectorAgent {
    sensor_cache: Arc<Mutex<SensorDeviceCache>>,
    motor_cache: Arc<Mutex<MotorDeviceCache>>,
}

impl ConnectorAgent {

    pub fn new() -> Self {
        ConnectorAgent {
            sensor_cache: Arc::new(Mutex::new(SensorDeviceCache::new())),
            motor_cache: Arc::new(Mutex::new(MotorDeviceCache::new())),
        }
    }

    pub fn get_sensor_cache(&self) -> MutexGuard<SensorDeviceCache> {
        self.sensor_cache.lock().unwrap()
    }

    pub fn get_sensor_cache_ref(&self) -> Arc<Mutex<SensorDeviceCache>> {
        self.sensor_cache.clone()
    }

    pub fn get_motor_cache(&self) -> MutexGuard<MotorDeviceCache> {
        self.motor_cache.lock().unwrap()
    }

    pub fn get_motor_cache_ref(&self) -> Arc<Mutex<MotorDeviceCache>> {
        self.motor_cache.clone()
    }

    pub fn export_device_registrations_as_config_json(&self) -> Result<serde_json::Value, FeagiDataError> {
        let mut capabilities = serde_json::Map::new();
        capabilities.insert("input".to_string(), self.get_sensor_cache().export_registered_sensors_as_config_json()?);
        capabilities.insert("output".to_string(), self.get_motor_cache().export_registered_motors_as_config_json()?);
        let mut output = serde_json::Map::new();
        output.insert("capabilities".to_string(), serde_json::Value::Object(capabilities));
        Ok(serde_json::Value::Object(output))
    }
}

impl fmt::Display for ConnectorAgent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ConnectorAgent")
    }
}