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
use feagi_sensorimotor::ConnectorCache;

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
    cache: ConnectorCache
}

#[cfg(feature = "sdk-video")]
impl Default for ConnectorAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "sdk-video")]
impl ConnectorAgent {
    pub fn new() -> Self {
        ConnectorAgent {
            cache: ConnectorCache::new()
        }
    }

    /// Get a mutable guard to the sensor cache
    pub fn get_sensor_cache(&self) -> MutexGuard<'_, SensorDeviceCache> {
        self.cache.get_sensor_cache()
    }

    /// Get a shared reference (Arc) to the sensor cache, useful for callbacks
    pub fn get_sensor_cache_ref(&self) -> Arc<Mutex<SensorDeviceCache>> {
        self.cache.get_sensor_cache_ref()
    }

    /// Get a mutable guard to the motor cache
    pub fn get_motor_cache(&self) -> MutexGuard<'_, MotorDeviceCache> {
        self.cache.get_motor_cache()
    }

    /// Get a shared reference (Arc) to the motor cache, useful for callbacks
    pub fn get_motor_cache_ref(&self) -> Arc<Mutex<MotorDeviceCache>> {
        self.cache.get_motor_cache_ref()
    }

    /// Export all device registrations as a JSON configuration
    ///
    /// This includes both sensor and motor device registrations, along with
    /// their encoder/decoder properties and feedback configurations.
    pub fn export_device_registrations_as_config_json(
        &self,
    ) -> Result<serde_json::Value, FeagiDataError> {
        self.cache.export_device_registrations_as_config_json()
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
        self.cache.import_device_registrations_as_config_json(json)?;
        Ok(())
    }
}

#[cfg(feature = "sdk-video")]
impl fmt::Display for ConnectorAgent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ConnectorAgent")
    }
}

