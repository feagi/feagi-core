use crate::caching::io_motor_cache::IOMotorCache;
use crate::caching::io_sensor_cache::IOSensorCache;

pub struct IOCache {
    sensors: IOSensorCache,
    motors: IOMotorCache,
}

// prefixes:
// cache_ -> cache encoding / decoding / alteration related function
// sensor_ -> sensor device specific function
// motor_ -> motor device specific function
