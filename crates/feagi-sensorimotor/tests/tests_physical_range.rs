//! Integration tests for `physical_range` on a device grouping: it survives an
//! import-then-export of the device registrations for both sensory and motor units, and an
//! invalid range is rejected on import.

use feagi_sensorimotor::data_types::descriptors::{
    ColorChannelLayout, ColorSpace, ImageFrameProperties, ImageXYResolution,
};
use feagi_sensorimotor::ConnectorCache;
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalUnitIndex, NeuronDepth,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, PercentageNeuronPositioning,
};
use serde_json::{json, Value};

const SENSOR_UNITS_KEY: &str = "input_units_and_encoder_properties";
const MOTOR_UNITS_KEY: &str = "output_units_and_decoder_properties";

/// Cache with one two-channel sensory unit and one one-channel motor unit.
fn registered_cache() -> ConnectorCache {
    let cache = ConnectorCache::new();
    cache
        .get_sensor_cache()
        .infrared_register(
            CorticalUnitIndex::from(0u16),
            CorticalChannelCount::new(2).unwrap(),
            FrameChangeHandling::Absolute,
            NeuronDepth::new(8).unwrap(),
            PercentageNeuronPositioning::Linear,
        )
        .unwrap();
    let image_props = ImageFrameProperties::new(
        ImageXYResolution::new(16, 16).unwrap(),
        ColorSpace::Gamma,
        ColorChannelLayout::RGB,
    )
    .unwrap();
    cache
        .get_motor_cache()
        .simple_vision_output_register(
            CorticalUnitIndex::from(0u16),
            CorticalChannelCount::new(1).unwrap(),
            FrameChangeHandling::Absolute,
            image_props,
        )
        .unwrap();
    cache
}

/// Every `device_grouping` array in the units under `section_key`.
fn device_groupings<'a>(json: &'a mut Value, section_key: &str) -> Vec<&'a mut Vec<Value>> {
    let mut out = Vec::new();
    for units in json[section_key].as_object_mut().unwrap().values_mut() {
        for entry in units.as_array_mut().unwrap() {
            // each entry is [unit definition, coder properties]
            let unit = &mut entry.as_array_mut().unwrap()[0];
            out.push(unit["device_grouping"].as_array_mut().unwrap());
        }
    }
    out
}

fn range_json(value_at_0: f64, value_at_1: f64, unit: &str) -> Value {
    json!({"value_at_0": value_at_0, "value_at_1": value_at_1, "unit": unit})
}

#[test]
fn export_without_range_omits_the_key() {
    let mut json = registered_cache()
        .export_device_registrations_as_config_json()
        .unwrap();
    for section in [SENSOR_UNITS_KEY, MOTOR_UNITS_KEY] {
        for grouping in device_groupings(&mut json, section) {
            for device in grouping {
                assert!(!device.as_object().unwrap().contains_key("physical_range"));
            }
        }
    }
}

#[test]
fn sensory_and_motor_ranges_survive_import_then_export() {
    let mut json = registered_cache()
        .export_device_registrations_as_config_json()
        .unwrap();
    // Sensory: range on the first channel only, so the second proves "absent stays absent".
    device_groupings(&mut json, SENSOR_UNITS_KEY)[0][0]["physical_range"] =
        range_json(-3.0, 3.0, "rad");
    device_groupings(&mut json, MOTOR_UNITS_KEY)[0][0]["physical_range"] =
        range_json(0.25, 0.0, "m");

    let mut imported = ConnectorCache::new();
    imported
        .import_device_registrations_as_config_json(json.clone())
        .unwrap();
    let mut exported = imported
        .export_device_registrations_as_config_json()
        .unwrap();

    assert_eq!(exported, json);
    assert_eq!(
        device_groupings(&mut exported, SENSOR_UNITS_KEY)[0][0]["physical_range"],
        range_json(-3.0, 3.0, "rad")
    );
    assert!(!device_groupings(&mut exported, SENSOR_UNITS_KEY)[0][1]
        .as_object()
        .unwrap()
        .contains_key("physical_range"));
    assert_eq!(
        device_groupings(&mut exported, MOTOR_UNITS_KEY)[0][0]["physical_range"],
        range_json(0.25, 0.0, "m")
    );
}

#[test]
fn invalid_range_is_rejected_on_import_for_sensory_and_motor() {
    let bad_ranges = [
        range_json(1.0, 1.0, "rad"),
        range_json(0.0, 1.0, ""),
        json!({"value_at_0": 0.0, "value_at_1": 1.0}),
        json!({"value_at_0": "low", "value_at_1": 1.0, "unit": "m"}),
    ];
    for section in [SENSOR_UNITS_KEY, MOTOR_UNITS_KEY] {
        for bad in &bad_ranges {
            let mut json = registered_cache()
                .export_device_registrations_as_config_json()
                .unwrap();
            device_groupings(&mut json, section)[0][0]["physical_range"] = bad.clone();
            let mut target = ConnectorCache::new();
            let result = target.import_device_registrations_as_config_json(json);
            assert!(result.is_err(), "{section}: {bad} must be rejected");
        }
    }
}
