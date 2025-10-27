use std::sync::{Arc, Mutex};
use crate::motor_device_cache::MotorDeviceCache;
use crate::sensor_device_cache::SensorDeviceCache;

pub struct ConnectorAgent {
    sensor_cache: Arc<Mutex<SensorDeviceCache>>,
    motor_cache: Arc<Mutex<MotorDeviceCache>>,
}

impl ConnectorAgent {

}