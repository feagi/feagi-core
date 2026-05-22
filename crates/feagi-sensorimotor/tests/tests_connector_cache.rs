//! Tests for the ConnectorCache module
//!
//! Tests cover:
//! - Creation and default implementation
//! - Sensor and motor cache access
//! - Export and import of device registrations as JSON
//! - Display implementation

use feagi_sensorimotor::ConnectorCache;
use feagi_genome_definitions::::descriptors::{
    CorticalChannelCount, CorticalUnitIndex,
};
use feagi_genome_definitions::::io_cortical_area_configuration_flag::{
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
                    feagi_genome_definitions::::descriptors::NeuronDepth::new(10)
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
                    feagi_genome_definitions::::descriptors::CorticalUnitIndex::from(0u8),
                    feagi_genome_definitions::::descriptors::CorticalChannelCount::new(1).unwrap(),
                    feagi_genome_definitions::::io_cortical_area_configuration_flag::FrameChangeHandling::Absolute,
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
                    feagi_genome_definitions::::descriptors::NeuronDepth::new(8)
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

/// Regression coverage for the IMU register/write contract.
///
/// Two macro arms in `sensor_unit_functions!` (`Percentage_3D` and
/// `SignedPercentage_4D`) historically seeded the cache slot with
/// `WrappedIOData::Percentage(...)` regardless of the arm's declared data
/// type. Until the introduction of `RawIMU` and `SmartIMU`, no template
/// targeted those arms, so the latent bug went unnoticed. The first
/// `_write` against a SmartIMU group then failed with
/// `Expected Percentage but got SignedPercentage_4D` because the cache slot
/// held the wrong wrapper variant.
///
/// These tests lock in the expected behavior: the register call must seed
/// the slot with the wrapper variant matching the arm's declared type, so
/// a same-typed `_write` is accepted.
#[cfg(test)]
mod test_imu_register_write_contract {
    use super::*;
    use feagi_sensorimotor::data_types::{
        Percentage, RawIMU, SignedPercentage, SignedPercentage3D, SignedPercentage4D,
    };
    use feagi_sensorimotor::wrapped_io_data::WrappedIOData;
    use feagi_genome_definitions::::descriptors::{
        CorticalChannelIndex, NeuronDepth,
    };

    /// Smart IMU is the first user of the `SignedPercentage_4D` macro arm.
    /// Without the initial-value fix this write fails with a type-mismatch
    /// error from the cache's wrapper-type guard.
    #[test]
    fn smart_imu_register_then_quaternion_write_succeeds() {
        let cache = ConnectorCache::new();
        let mut sensor_cache = cache.get_sensor_cache();

        sensor_cache
            .smart_i_m_u_register(
                CorticalUnitIndex::from(0u8),
                CorticalChannelCount::new(1).unwrap(),
                FrameChangeHandling::Absolute,
                NeuronDepth::new(10).unwrap(),
                PercentageNeuronPositioning::Linear,
            )
            .expect("SmartIMU register must succeed");

        let identity_quat = SignedPercentage4D::new(
            SignedPercentage::new_from_m1_1(1.0).unwrap(),
            SignedPercentage::new_from_m1_1(0.0).unwrap(),
            SignedPercentage::new_from_m1_1(0.0).unwrap(),
            SignedPercentage::new_from_m1_1(0.0).unwrap(),
        );

        sensor_cache
            .smart_i_m_u_write(
                CorticalUnitIndex::from(0u8),
                CorticalChannelIndex::from(0u32),
                WrappedIOData::SignedPercentage_4D(identity_quat),
            )
            .expect(
                "SmartIMU _write must accept SignedPercentage_4D; \
                 regression for the `initial_val: Percentage` macro bug",
            );
    }

    /// Reject writes whose wrapper variant does not match the registered slot.
    /// Demonstrates that the type guard is still active after the fix; a
    /// scalar `Percentage` payload must NOT silently coerce into the SmartIMU
    /// slot.
    #[test]
    fn smart_imu_rejects_mismatched_wrapper_type() {
        let cache = ConnectorCache::new();
        let mut sensor_cache = cache.get_sensor_cache();

        sensor_cache
            .smart_i_m_u_register(
                CorticalUnitIndex::from(0u8),
                CorticalChannelCount::new(1).unwrap(),
                FrameChangeHandling::Absolute,
                NeuronDepth::new(10).unwrap(),
                PercentageNeuronPositioning::Linear,
            )
            .unwrap();

        let mismatched = WrappedIOData::Percentage(Percentage::new_from_0_1(0.5).unwrap());
        let result = sensor_cache.smart_i_m_u_write(
            CorticalUnitIndex::from(0u8),
            CorticalChannelIndex::from(0u32),
            mismatched,
        );
        assert!(
            result.is_err(),
            "Mismatched wrapper variant must be rejected by the type guard"
        );
    }

    /// Raw IMU exercises the bespoke composite arm. The first write after
    /// registration must accept a `WrappedIOData::RawIMU(...)` value.
    #[test]
    fn raw_imu_register_then_composite_write_succeeds() {
        let cache = ConnectorCache::new();
        let mut sensor_cache = cache.get_sensor_cache();

        sensor_cache
            .raw_i_m_u_register(
                CorticalUnitIndex::from(0u8),
                CorticalChannelCount::new(1).unwrap(),
                FrameChangeHandling::Absolute,
                NeuronDepth::new(10).unwrap(),
                PercentageNeuronPositioning::Linear,
            )
            .expect("RawIMU register must succeed");

        let signed_pct = |v: f32| SignedPercentage::new_from_m1_1(v).unwrap();
        let triple = SignedPercentage3D::new(signed_pct(0.1), signed_pct(-0.2), signed_pct(0.3));
        let composite = RawIMU::new(triple, triple, triple);

        sensor_cache
            .raw_i_m_u_write(
                CorticalUnitIndex::from(0u8),
                CorticalChannelIndex::from(0u32),
                WrappedIOData::RawIMU(composite),
            )
            .expect("RawIMU _write must accept the composite wrapper variant");
    }

    /// Real-world IMUs frequently expose only a subset of (accel, gyro, mag).
    /// The partial-write API must let a controller update one axis without
    /// touching the other two, so missing axes literally retain their last
    /// written value (or the registered initial zero).
    ///
    /// This regression locks in the read-modify-write contract: writing the
    /// gyroscope axis MUST NOT corrupt the accelerometer or magnetometer
    /// sub-components, and a subsequent magnetometer write MUST preserve the
    /// previously-written gyroscope sample.
    #[test]
    fn raw_imu_partial_write_preserves_other_subaxes() {
        let cache = ConnectorCache::new();
        let mut sensor_cache = cache.get_sensor_cache();

        sensor_cache
            .raw_i_m_u_register(
                CorticalUnitIndex::from(0u8),
                CorticalChannelCount::new(1).unwrap(),
                FrameChangeHandling::Absolute,
                NeuronDepth::new(10).unwrap(),
                PercentageNeuronPositioning::Linear,
            )
            .expect("RawIMU register must succeed");

        let signed_pct = |v: f32| SignedPercentage::new_from_m1_1(v).unwrap();
        let new_gyro = SignedPercentage3D::new(signed_pct(0.4), signed_pct(0.5), signed_pct(0.6));

        sensor_cache
            .raw_i_m_u_write_gyroscope(
                CorticalUnitIndex::from(0u8),
                CorticalChannelIndex::from(0u32),
                new_gyro,
            )
            .expect("partial gyro write must succeed against an initialized RawIMU slot");

        let after_gyro = sensor_cache
            .raw_i_m_u_read_postprocessed_cache_value(
                CorticalUnitIndex::from(0u8),
                CorticalChannelIndex::from(0u32),
            )
            .expect("must be able to read RawIMU back after partial write");

        let zero = signed_pct(0.0);
        assert_eq!(
            after_gyro.get_accelerometer().a,
            zero,
            "accelerometer.a must remain at registered zero after gyro-only write"
        );
        assert_eq!(
            after_gyro.get_magnetometer().c,
            zero,
            "magnetometer.c must remain at registered zero after gyro-only write"
        );
        assert_eq!(after_gyro.get_gyroscope().a, signed_pct(0.4));
        assert_eq!(after_gyro.get_gyroscope().c, signed_pct(0.6));

        let new_mag = SignedPercentage3D::new(signed_pct(-0.1), signed_pct(-0.2), signed_pct(-0.3));
        sensor_cache
            .raw_i_m_u_write_magnetometer(
                CorticalUnitIndex::from(0u8),
                CorticalChannelIndex::from(0u32),
                new_mag,
            )
            .expect("partial mag write must succeed against an already-mutated slot");

        let after_mag = sensor_cache
            .raw_i_m_u_read_postprocessed_cache_value(
                CorticalUnitIndex::from(0u8),
                CorticalChannelIndex::from(0u32),
            )
            .expect("must be able to read RawIMU back after second partial write");

        assert_eq!(
            after_mag.get_gyroscope().a,
            signed_pct(0.4),
            "previously-written gyroscope.a must survive a magnetometer-only write"
        );
        assert_eq!(after_mag.get_magnetometer().a, signed_pct(-0.1));
        assert_eq!(after_mag.get_magnetometer().c, signed_pct(-0.3));
        assert_eq!(
            after_mag.get_accelerometer().b,
            zero,
            "accelerometer.b must still be zero - never written by either partial call"
        );
    }

    /// Calling a partial-write helper before the unit has been registered
    /// must fail loudly. Silent acceptance would mask controller-side
    /// ordering bugs (the controller calls `_write_*` before `_register`).
    #[test]
    fn raw_imu_partial_write_without_register_errors() {
        let cache = ConnectorCache::new();
        let mut sensor_cache = cache.get_sensor_cache();

        let signed_pct = |v: f32| SignedPercentage::new_from_m1_1(v).unwrap();
        let triple = SignedPercentage3D::new(signed_pct(0.1), signed_pct(0.2), signed_pct(0.3));

        let err = sensor_cache.raw_i_m_u_write_accelerometer(
            CorticalUnitIndex::from(0u8),
            CorticalChannelIndex::from(0u32),
            triple,
        );
        assert!(
            err.is_err(),
            "partial accelerometer write must error when unit is not registered"
        );
    }
}
