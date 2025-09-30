use crate::caching::io_motor_cache::IOMotorCache;
use crate::caching::io_sensor_cache::IOSensorCache;

pub struct IOCache {
    sensors: IOSensorCache,
    motors: IOMotorCache,
}

