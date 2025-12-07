use std::sync::{Arc, Mutex, MutexGuard};
use std::fmt;
use crate::motor_device_cache::MotorDeviceCache;
use crate::sensor_device_cache::SensorDeviceCache;

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
}

impl fmt::Display for ConnectorAgent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ConnectorAgent")
    }
}