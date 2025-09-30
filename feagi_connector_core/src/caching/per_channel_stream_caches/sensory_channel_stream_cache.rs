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

impl SensoryChannelStreamCache {
    
    pub fn new(pipeline_stages: Vec<Box<dyn PipelineStage + Sync + Send>>) -> Result<Self, FeagiDataError> {

        if pipeline_stages.is_empty() {
            return Err(FeagiDataError::InternalError("SensoryChannelStreamCache Cannot have 0 pipeline stages!".into()))
        }

        let processor_runner = PipelineStageRunner::new(pipeline_stages)?;
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

    //endregion

    //region Data

    pub fn update_sensor_value(&mut self, value: WrappedIOData) -> Result<(), FeagiDataError> {
        self.last_updated = Instant::now();
        _ = self.pipeline_runner.update_value(&value, Instant::now())?;
        Ok(())
    }

    /// Returns the most recently processed sensor value.
    ///
    /// Provides access to the latest data that has been processed through
    /// the entire processor chain. This data is ready for neural encoding
    /// or external consumption.
    ///
    /// # Returns
    ///
    /// Reference to the most recent processed sensor data
    pub fn get_most_recent_sensor_value(&self) -> &WrappedIOData {
        self.pipeline_runner.get_most_recent_output()
    }

    pub(crate) fn get_most_recent_sensor_value_and_time(&self) -> (&WrappedIOData, &Instant) {
        (self.pipeline_runner.get_most_recent_output(), &self.last_updated)
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

    //region Stages
    // This region is essentially just a proxy to a private structure for public access

    pub fn get_all_stage_properties(&self) -> Vec<Box<dyn PipelineStageProperties + Sync + Send>> {
        self.pipeline_runner.get_all_stage_properties()
    }

    pub fn get_single_stage_property(&self, stage_index: PipelineStagePropertyIndex) -> Result<Box<dyn PipelineStageProperties + Sync + Send>, FeagiDataError> {
        self.pipeline_runner.get_single_stage_property(stage_index)
    }

    pub fn try_update_single_stage_properties(&mut self, updating_stage_index: PipelineStagePropertyIndex, updated_properties: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<(), FeagiDataError> {
        self.pipeline_runner.try_update_single_stage_properties(updating_stage_index, updated_properties)
    }

    pub fn try_replace_all_stages(&mut self, new_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<(), FeagiDataError> {
        self.pipeline_runner.try_replace_all_stages(new_pipeline_stage_properties)
    }

    pub fn try_replace_single_stage(&mut self, replacing_at_index: PipelineStagePropertyIndex, new_pipeline_stage_properties: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<(), FeagiDataError> {
        self.pipeline_runner.try_replace_single_stage(replacing_at_index, new_pipeline_stage_properties)
    }

    //endregion

}


