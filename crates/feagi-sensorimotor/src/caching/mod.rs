//! Caching infrastructure for FEAGI I/O operations.
//!
//! Provides high-performance caching for sensor inputs and motor outputs,
//! with automatic encoding/decoding to/from neuron voxel representations.
//! The cache handles data preprocessing, pipeline management, and type conversions.
mod motor_device_cache;
mod sensor_device_cache;

pub use motor_device_cache::MotorDeviceCache;
pub use sensor_device_cache::SensorDeviceCache;
