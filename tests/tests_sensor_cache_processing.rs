//! Test for sensor cache with stream processing and FEAGI bytes structure encoding.
//!
//! This test demonstrates creating a proximity sensor with rolling window and range
//! processing, processing sensor data, and encoding to FEAGI bytes structures.

use feagi_core_data_structures_and_processing::genomic_structures::{CorticalGroupingIndex, SensorCorticalType, SingleChannelDimensions};
use feagi_core_data_structures_and_processing::io_data::IOTypeData;
use feagi_core_data_structures_and_processing::io_processing::processors::{
    LinearAverageRollingWindowProcessor, LinearScaleTo0And1Processor
};
use feagi_core_data_structures_and_processing::io_processing::{SensorCache, StreamCacheProcessor};
use feagi_core_data_structures_and_processing::neuron_data::xyzp::{
    CorticalMappedXYZPNeuronData
};
use feagi_core_data_structures_and_processing::io_processing::byte_structures::{
    FeagiByteStructureCompatible
};
use std::time::Instant;

#[test]
fn test_chained_encoders() -> Result<(), Box<dyn std::error::Error>> {
    // Create the rolling window processor (5-sample window, initial value 0.0)
    let mut rolling_window_processor = LinearAverageRollingWindowProcessor::new(5, 0.0)?;

    // Create the range scaling processor (maps 0-50 to 0-1)
    let mut range_processor = LinearScaleTo0And1Processor::new(0.0, 50.0, 25.0)?;

    // In here, lets try manually running the data through without the help of higher level structures, just to ensure the math is fine in here
    {
        // Verify the processing handle the correct data types
        assert_eq!(range_processor.get_input_data_type(), rolling_window_processor.get_output_data_type());

        // Create the input sensor value (25.0)
        let sensor_value = IOTypeData::new_f32(25.0)?;
        let timestamp = Instant::now();

        // Process through the rolling window processor first
        let windowed_result = rolling_window_processor.process_new_input(&sensor_value, timestamp)?; // 0 + 0 + 0 + 0 + 25 / 5 -> 5

        // Then process through the range scaling processor
        let scaled_result = range_processor.process_new_input(windowed_result, timestamp)?; // 0 <-> 5 <-> 50 -> 0.1


        assert_eq!(f32::try_from(scaled_result)?, 0.1);

        Ok(())
    }
}

#[test]
fn test_simple_sensor_neuron_encodering() -> Result<(), Box<dyn std::error::Error>> {

    // Create Sensor Cache and neuron container
    let mut sensor_cache = SensorCache::new();
    let mut neuron_data = CorticalMappedXYZPNeuronData::new();
    
    // Register a single channel Proximity Sensor with default settings
    sensor_cache.register_cortical_group_for_proximity(
        1.into(),
        1,
        true,
        10,
        0.0,
        100.0,
    )?;
    
    // Send a value
    sensor_cache.send_data_for_proximity(
        10.0, 
        1.into(), 
        0.into()
    )?;
    
    // Encode the data to neurons
    sensor_cache.encode_to_neurons(Instant::now(), &mut neuron_data)?;
    
    let byte_structure = neuron_data.as_new_feagi_byte_structure()?;
    
    let bytes = byte_structure.copy_out_as_byte_vector();
    
    dbg!(bytes);
    
    Ok(())
    
    
}


/*
#[test]
fn test_sensor_cache_with_stream_processors_and_encoding() -> Result<(), Box<dyn std::error::Error>> {
    // Create Sensor Cache
    let mut sensor_cache = SensorCache::new();

    // Register proximity cortical area (group 1)
    _ = sensor_cache.register_single_cortical_area(
        SensorCorticalType::Proximity,
        1.into(),
        3,
        SingleChannelDimensions::new(1, 1, 10)?
    )?;

    // register channel 2 on cortical area with chained cache processing
    _ = sensor_cache.register_single_channel(
        SensorCorticalType::Proximity,
        1.into(),
        2.into(),
        vec![
            Box::new(LinearAverageRollingWindowProcessor::new(5, 0.0)?),
            Box::new(LinearScaleTo0And1Processor::new(0.0, 50.0, 25.0)?)
        ],
        true
    )?;

    // Update the value on that channel
    _ = sensor_cache.update_value_by_channel(IOTypeData::new_f32(25.0)?,
                                             SensorCorticalType::Proximity, 1.into(), 2.into())?;

    // Neuron write target
    let mut cortical_neuron_data = CorticalMappedXYZPNeuronData::new();
    _ = sensor_cache.encode_to_neurons(Instant::now(), &mut cortical_neuron_data)?;

    // Verify the neural data was created
    assert!(cortical_neuron_data.len() == 1, "Cortical neuron data should not be empty after encoding");

    // Encode to FEAGI bytes structure
    let byte_structure = cortical_neuron_data.as_new_feagi_byte_structure()?;
    
    // Convert to raw bytes
    let raw_bytes = byte_structure.copy_out_as_byte_vector();
    
    // Verify we got some bytes
    assert!(!raw_bytes.is_empty(), "Should have non-empty bytes representation");
    println!("Encoded to {} bytes", raw_bytes.len());
    
    Ok(())
}

 */