use crate::caching::MotorDeviceCache;
use crate::caching::SensorDeviceCache;
use feagi_data_structures::FeagiDataError;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

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

    pub fn export_device_registrations_as_config_json(
        &self,
    ) -> Result<serde_json::Value, FeagiDataError> {
        let mut capabilities = serde_json::Map::new();
        capabilities.insert(
            "input".to_string(),
            self.get_sensor_cache()
                .export_registered_sensors_as_config_json()?,
        );
        capabilities.insert(
            "output".to_string(),
            self.get_motor_cache()
                .export_registered_motors_as_config_json()?,
        );
        let mut output = serde_json::Map::new();
        output.insert(
            "capabilities".to_string(),
            serde_json::Value::Object(capabilities),
        );
        Ok(serde_json::Value::Object(output))
    }

    /// Import device registrations from JSON configuration string
    ///
    /// Parses JSON and updates pipeline configurations and friendly names for already-registered devices.
    /// Devices must be registered first using the appropriate registration functions.
    ///
    /// # Arguments
    /// * `json_str` - JSON string in the new capabilities format
    ///
    /// # Returns
    /// * `Ok(())` - If import succeeded
    /// * `Err(FeagiDataError)` - If JSON is malformed or devices not registered
    ///
    /// # Example JSON Format
    /// ```json
    /// {
    ///   "capabilities": {
    ///     "input": {
    ///       "simple_vision": {
    ///         "0": {
    ///           "friendly_name": "Main Camera",
    ///           "channels": [...]
    ///         }
    ///       }
    ///     },
    ///     "output": {}
    ///   }
    /// }
    /// ```
    pub fn import_device_registrations_from_config_json(
        &mut self,
        json_str: &str,
    ) -> Result<(), FeagiDataError> {
        let json: serde_json::Value = serde_json::from_str(json_str).map_err(|e| {
            FeagiDataError::DeserializationError(format!("Failed to parse JSON: {}", e))
        })?;

        let capabilities = json.get("capabilities")
            .ok_or_else(|| FeagiDataError::DeserializationError(
                "Missing 'capabilities' key in JSON. Expected format: {\"capabilities\": {\"input\": {...}, \"output\": {...}}}".to_string()
            ))?;

        // Import sensors (input)
        if let Some(input) = capabilities.get("input") {
            self.get_sensor_cache().import_sensors_from_json(input)?;
        }

        // Import motors (output)
        if let Some(output) = capabilities.get("output") {
            self.get_motor_cache().import_motors_from_json(output)?;
        }

        Ok(())
    }
}

impl fmt::Display for ConnectorAgent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ConnectorAgent")
    }
}
