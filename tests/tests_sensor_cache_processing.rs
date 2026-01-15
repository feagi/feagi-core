//! Test for sensor cache with stream processing and FEAGI bytes structure encoding.
//!
//! This test demonstrates creating a proximity sensor with rolling window and range
//! processing, processing sensor data, and encoding to FEAGI bytes structures.

use feagi_sensorimotor::caching::SensorDeviceCache;
use feagi_sensorimotor::data_types::Percentage;
use feagi_sensorimotor::wrapped_io_data::WrappedIOData;
use feagi_serialization::FeagiByteStructureType;
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelIndex, CorticalUnitIndex, NeuronDepth,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, PercentageNeuronPositioning,
};
use std::time::Instant;

#[test]
fn test_chained_encoders() -> Result<(), Box<dyn std::error::Error>> {
    let mut sensor_cache = SensorDeviceCache::new();
    sensor_cache.proximity_register(
        CorticalUnitIndex::from(0),
        CorticalChannelCount::new(1).unwrap(),
        FrameChangeHandling::Absolute,
        NeuronDepth::new(10).unwrap(),
        PercentageNeuronPositioning::Linear,
    )?;

    let input_value = Percentage::new_from_0_1(0.5)?;
    sensor_cache.proximity_write(
        CorticalUnitIndex::from(0),
        CorticalChannelIndex::from(0),
        WrappedIOData::from(input_value),
    )?;

    sensor_cache.encode_all_sensors_to_neurons(Instant::now())?;
    sensor_cache.encode_neurons_to_bytes()?;

    let container = sensor_cache.get_feagi_byte_container();
    assert!(container.is_valid());
    assert_eq!(
        container.try_get_number_contained_structures().unwrap(),
        1
    );
    assert_eq!(
        container.get_contained_struct_types(),
        vec![FeagiByteStructureType::NeuronCategoricalXYZP]
    );

    Ok(())
}

#[test]
fn test_simple_sensor_neuron_encodering() -> Result<(), Box<dyn std::error::Error>> {
    let mut sensor_cache = SensorDeviceCache::new();
    sensor_cache.proximity_register(
        CorticalUnitIndex::from(0),
        CorticalChannelCount::new(1).unwrap(),
        FrameChangeHandling::Absolute,
        NeuronDepth::new(10).unwrap(),
        PercentageNeuronPositioning::Linear,
    )?;

    let input_value = Percentage::new_from_0_1(0.1)?;
    sensor_cache.proximity_write(
        CorticalUnitIndex::from(0),
        CorticalChannelIndex::from(0),
        WrappedIOData::from(input_value),
    )?;

    let cached_value = sensor_cache.proximity_read_postprocessed_cache_value(
        CorticalUnitIndex::from(0),
        CorticalChannelIndex::from(0),
    )?;
    assert!((cached_value.get_as_0_1() - 0.1).abs() < f32::EPSILON);

    sensor_cache.encode_all_sensors_to_neurons(Instant::now())?;
    assert_eq!(sensor_cache.get_neurons().len(), 1);

    Ok(())
}
