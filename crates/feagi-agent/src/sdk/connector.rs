// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Connector Agent - High-level interface for device registration and configuration
//!
//! Provides a unified interface for managing sensor and motor device caches,
//! with JSON-based configuration export/import capabilities.

#[cfg(feature = "sdk-video")]
use feagi_sensorimotor::caching::{MotorDeviceCache, SensorDeviceCache};
#[cfg(feature = "sdk-video")]
use feagi_sensorimotor::configuration::jsonable::JSONInputOutputDefinition;
use feagi_structures::FeagiDataError;
use std::fmt;
#[cfg(feature = "sdk-video")]
use std::sync::{Arc, Mutex, MutexGuard};

/// High-level connector agent for managing sensor and motor device registrations
///
/// This struct provides a unified interface for:
/// - Managing sensor and motor device caches
/// - Exporting/importing device configurations as JSON
/// - Thread-safe access to device caches
///
/// # Example
/// ```ignore
/// use feagi_agent::sdk::ConnectorAgent;
///
/// let mut connector = ConnectorAgent::new();
///
/// // Register devices...
///
/// // Export configuration
/// let config_json = connector.export_device_registrations_as_config_json()?;
///
/// // Import configuration
/// connector.import_device_registrations_as_config_json(config_json)?;
/// ```
#[derive(Debug)]
#[cfg(feature = "sdk-video")]
pub struct ConnectorAgent {
    sensor_cache: Arc<Mutex<SensorDeviceCache>>,
    motor_cache: Arc<Mutex<MotorDeviceCache>>,
}

#[cfg(feature = "sdk-video")]
impl Default for ConnectorAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "sdk-video")]
impl ConnectorAgent {
    /// Create a new ConnectorAgent with empty sensor and motor caches
    pub fn new() -> Self {
        let sensors = Arc::new(Mutex::new(SensorDeviceCache::new()));
        ConnectorAgent {
            sensor_cache: sensors.clone(),
            motor_cache: Arc::new(Mutex::new(MotorDeviceCache::new(sensors))),
        }
    }

    /// Get a mutable guard to the sensor cache
    pub fn get_sensor_cache(&self) -> MutexGuard<'_, SensorDeviceCache> {
        self.sensor_cache.lock().unwrap()
    }

    /// Get a shared reference (Arc) to the sensor cache
    pub fn get_sensor_cache_ref(&self) -> Arc<Mutex<SensorDeviceCache>> {
        self.sensor_cache.clone()
    }

    /// Get a mutable guard to the motor cache
    pub fn get_motor_cache(&self) -> MutexGuard<'_, MotorDeviceCache> {
        self.motor_cache.lock().unwrap()
    }

    /// Get a shared reference (Arc) to the motor cache
    pub fn get_motor_cache_ref(&self) -> Arc<Mutex<MotorDeviceCache>> {
        self.motor_cache.clone()
    }

    /// Export all device registrations as a JSON configuration
    ///
    /// This includes both sensor and motor device registrations, along with
    /// their encoder/decoder properties and feedback configurations.
    pub fn export_device_registrations_as_config_json(
        &self,
    ) -> Result<serde_json::Value, FeagiDataError> {
        let mut output = JSONInputOutputDefinition::new();
        self.get_sensor_cache().export_to_input_definition(&mut output)?;
        self.get_motor_cache().export_to_output_definition(&mut output)?;
        Ok(serde_json::to_value(output).unwrap())
    }

    /// Import device registrations from a JSON configuration
    ///
    /// # Warning
    /// This operation **wipes all existing registered devices** before importing
    /// the new configuration.
    pub fn import_device_registrations_as_config_json(
        &mut self,
        json: serde_json::Value,
    ) -> Result<(), FeagiDataError> {
        // NOTE: Wipes all registered devices
        let definition: JSONInputOutputDefinition = serde_json::from_value(json)
            .map_err(|err| FeagiDataError::DeserializationError(err.to_string()))?;
        self.get_motor_cache().import_from_output_definition(&definition)?;
        self.get_sensor_cache().import_from_input_definition(&definition)?;
        Ok(())
    }
}

#[cfg(feature = "sdk-video")]
impl fmt::Display for ConnectorAgent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ConnectorAgent")
    }
}

