//! Tests for the ConnectorCache module
//!
//! Tests cover:
//! - Creation and default implementation
//! - Sensor and motor cache access
//! - Export and import of device registrations as JSON
//! - Display implementation

use feagi_sensorimotor::ConnectorCache;
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalUnitIndex,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, PercentageNeuronPositioning,
};

#[cfg(test)]
mod test_connector_cache_creation {
    use super::*;

    #[test]
    fn test_new_creates_instance() {
        let cache = ConnectorCache::new();
        // Just verify it can be created without panic
        let _ = format!("{}", cache);
    }

    #[test]
    fn test_default_creates_instance() {
        let cache = ConnectorCache::default();
        let _ = format!("{}", cache);
    }
}

#[cfg(test)]
mod test_cache_access {
    use super::*;

    #[test]
    fn test_get_sensor_cache() {
        let cache = ConnectorCache::new();
        let sensor_cache = cache.get_sensor_cache();
        // Verify we can access the sensor cache
        drop(sensor_cache);
    }

    #[test]
    fn test_get_motor_cache() {
        let cache = ConnectorCache::new();
        let motor_cache = cache.get_motor_cache();
        // Verify we can access the motor cache
        drop(motor_cache);
    }

    #[test]
    fn test_get_sensor_cache_ref() {
        let cache = ConnectorCache::new();
        let sensor_cache_ref = cache.get_sensor_cache_ref();
        // Verify we get an Arc that can be locked
        let _guard = sensor_cache_ref.lock().unwrap();
    }

    #[test]
    fn test_get_motor_cache_ref() {
        let cache = ConnectorCache::new();
        let motor_cache_ref = cache.get_motor_cache_ref();
        // Verify we get an Arc that can be locked
        let _guard = motor_cache_ref.lock().unwrap();
    }
}

#[cfg(test)]
mod test_export_import {
    use super::*;

    #[test]
    fn test_export_empty_cache() {
        let cache = ConnectorCache::new();
        let json = cache.export_device_registrations_as_config_json().unwrap();

        // Should be a valid JSON object
        assert!(json.is_object());
    }

    #[test]
    fn test_import_empty_export() {
        let cache1 = ConnectorCache::new();
        let json = cache1.export_device_registrations_as_config_json().unwrap();

        let mut cache2 = ConnectorCache::new();
        cache2
            .import_device_registrations_as_config_json(json)
            .unwrap();
    }

    #[test]
    fn test_export_with_registered_sensor() {
        let cache = ConnectorCache::new();

        // Register a simple sensor
        {
            let mut sensor_cache = cache.get_sensor_cache();
            sensor_cache
                .infrared_register(
                    CorticalUnitIndex::from(0u8),
                    CorticalChannelCount::new(1).unwrap(),
                    FrameChangeHandling::Absolute,
                    feagi_structures::genomic::cortical_area::descriptors::NeuronDepth::new(10)
                        .unwrap(),
                    PercentageNeuronPositioning::Linear,
                )
                .unwrap();
        }

        let json = cache.export_device_registrations_as_config_json().unwrap();

        // Should contain the registered sensor
        assert!(json.is_object());
        let obj = json.as_object().unwrap();
        assert!(obj.contains_key("input_units_and_encoder_properties"));
    }

    #[test]
    fn test_export_with_registered_motor_image_frame() {
        let cache = ConnectorCache::new();

        {
            let mut motor_cache = cache.get_motor_cache();
            let image_props =
                feagi_sensorimotor::data_types::descriptors::ImageFrameProperties::new(
                    feagi_sensorimotor::data_types::descriptors::ImageXYResolution::new(128, 128)
                        .unwrap(),
                    feagi_sensorimotor::data_types::descriptors::ColorSpace::Gamma,
                    feagi_sensorimotor::data_types::descriptors::ColorChannelLayout::RGB,
                )
                .unwrap();

            motor_cache
                .simple_vision_output_register(
                    feagi_structures::genomic::cortical_area::descriptors::CorticalUnitIndex::from(0u8),
                    feagi_structures::genomic::cortical_area::descriptors::CorticalChannelCount::new(1).unwrap(),
                    feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::FrameChangeHandling::Absolute,
                    image_props,
                )
                .unwrap();
        }

        let json = cache.export_device_registrations_as_config_json().unwrap();
        assert!(json.is_object());
        let obj = json.as_object().unwrap();
        assert!(obj.contains_key("output_units_and_decoder_properties"));
    }

    #[test]
    fn test_export_import_roundtrip_with_sensor() {
        let cache1 = ConnectorCache::new();

        // Register a sensor
        {
            let mut sensor_cache = cache1.get_sensor_cache();
            sensor_cache
                .infrared_register(
                    CorticalUnitIndex::from(0u8),
                    CorticalChannelCount::new(2).unwrap(),
                    FrameChangeHandling::Absolute,
                    feagi_structures::genomic::cortical_area::descriptors::NeuronDepth::new(8)
                        .unwrap(),
                    PercentageNeuronPositioning::Fractional,
                )
                .unwrap();
        }

        // Export
        let json = cache1.export_device_registrations_as_config_json().unwrap();

        // Import into a new cache
        let mut cache2 = ConnectorCache::new();
        cache2
            .import_device_registrations_as_config_json(json.clone())
            .unwrap();

        // Export again and compare
        let json2 = cache2.export_device_registrations_as_config_json().unwrap();
        assert_eq!(json, json2);
    }

    #[test]
    fn test_import_invalid_json() {
        let mut cache = ConnectorCache::new();

        // Try to import invalid JSON structure
        let invalid_json = serde_json::json!({
            "invalid_key": "invalid_value"
        });

        let result = cache.import_device_registrations_as_config_json(invalid_json);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod test_display {
    use super::*;

    #[test]
    fn test_display_implementation() {
        let cache = ConnectorCache::new();
        let display = format!("{}", cache);
        assert_eq!(display, "ConnectorAgent");
    }
}
