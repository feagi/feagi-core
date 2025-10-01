//! Channel-level stream caching for sensory data processing.
//!
//! This module provides per-channel caching/processing mechanisms for sensory input streams
//! in FEAGI's neural processing system.

use std::time::{Instant};
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::descriptors::CorticalChannelIndex;
use feagi_data_structures::neurons::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZPEncoder};
use feagi_data_structures::wrapped_io_data::{WrappedIOData, WrappedIOType};
use crate::data_pipeline::{PipelineStagePropertyIndex, PipelineStageRunner, PipelineStage, PipelineStageProperties};

/// Per-channel cache for sensory input data streams.
///
/// `SensoryChannelStreamCache` manages the buffering and processing of sensory
/// data for a single I/O channel. It applies a chain of stream processing to
/// incoming data and tracks when values were last updated
///
/// # Key Features
///
/// - **Stream Processing**: Applies configurable processor chains to incoming data
/// - **Temporal Tracking**: Monitors when data was last updated for freshness checks
/// - **Stale Data Control**: Configurable behavior for sending cached vs. fresh data
/// - **Neural Encoding**: Direct conversion from processed data to neural representations
/// - **Type Safety**: Tracks input and output data types through the processing chain
#[derive(Debug)]
pub(crate) struct SensoryChannelStreamCache {
    pipeline_runner: PipelineStageRunner,
    last_updated: Instant,
}

// NOTE: We aim to generally abstract away [PipelineStageRunner] data operations from here onward

impl SensoryChannelStreamCache {
    
    pub fn new(pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<Self, FeagiDataError> {

        let processor_runner = PipelineStageRunner::new(pipeline_stage_properties)?;
        Ok(SensoryChannelStreamCache {
            pipeline_runner: processor_runner,
            last_updated: Instant::now(),
        })
    }

    //region Properties

    /// Returns the input data type expected by the processor chain.
    ///
    /// Indicates what type of data this cache expects to receive from sensors.
    /// This is determined by the first processor in the chain and enables
    /// type validation before data is fed into the cache.
    ///
    /// # Returns
    ///
    /// The `IOTypeVariant` that represents the expected input data type
    pub fn get_input_data_type(&self) -> WrappedIOType {
        self.pipeline_runner.get_input_data_type()
    }

    /// Returns the output data type produced by the processor chain.
    ///
    /// Indicates what type of data this cache produces after processing.
    /// This is determined by the final processor in the chain and enables
    /// consumers to understand the format of processed data.
    ///
    /// # Returns
    ///
    /// The `IOTypeVariant` that represents the output data type
    pub fn get_output_data_type(&self) -> WrappedIOType {
        self.pipeline_runner.get_output_data_type()
    }

    pub(crate) fn get_pipeline_runner(&self) -> &PipelineStageRunner {
        &self.pipeline_runner
    }

    pub(crate) fn get_pipeline_runner_mut(&mut self) -> &mut PipelineStageRunner {
        &mut self.pipeline_runner
    }

    //endregion

    //region Data

    pub fn try_update_sensor_value(&mut self, value: WrappedIOData) -> Result<(), FeagiDataError> {
        _ = self.pipeline_runner.update_value(&value, Instant::now())?; // Checks data type first
        self.last_updated = Instant::now();
        Ok(())
    }

    pub fn get_most_recent_postprocessed_sensor_value(&self) -> &WrappedIOData {
        self.pipeline_runner.get_most_recent_output()
    }
    
    pub fn get_update_time(&self) -> Instant {
        self.last_updated
    }



    /*
    /// Encodes the cached sensor data into neural representations.
    ///
    /// Converts the most recent processed sensor value into neural activity
    /// patterns using the provided encoder. The encoded data is written
    /// directly into the cortical mapped neuron data structure for this channel.
    ///
    /// # Arguments
    ///
    /// * `cortical_mapped_neuron_data` - Target neuron data structure to write to
    /// * `encoder` - Encoder that converts I/O data to neural patterns
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Successfully encoded data into neural representation
    /// * `Err(FeagiDataProcessingError)` - If encoding fails
    ///
    /// # Errors
    ///
    /// Returns an error if the encoder cannot handle the data type or if
    /// the neural data structure cannot accommodate the encoded patterns.
    pub fn encode_to_neurons(&self, cortical_mapped_neuron_data: &mut CorticalMappedXYZPNeuronData, encoder: &Box<dyn NeuronXYZPEncoder + Sync + Send>) -> Result<(), FeagiDataError> {
        encoder.write_neuron_data_single_channel(self.get_most_recent_sensor_value(), self.channel, cortical_mapped_neuron_data)
    }

     */

    //endregion
}


