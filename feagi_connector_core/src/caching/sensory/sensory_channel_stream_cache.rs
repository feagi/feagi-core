//! Channel-level stream caching for sensory data processing.
//!
//! This module provides per-channel caching/processing mechanisms for sensory input streams
//! in FEAGI's neural processing system.

use std::time::{Instant};
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::descriptors::CorticalChannelIndex;
use feagi_data_structures::neurons::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZPEncoder};
use feagi_data_structures::wrapped_io_data::{WrappedIOData, WrappedIOType};
use crate::data_pipeline::{PipelineStageIndex, PipelineStageRunner, PipelineStage};

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
    channel: CorticalChannelIndex,
    last_updated: Instant,
    should_allow_sending_stale_data: bool,
}

impl SensoryChannelStreamCache {
    
    pub fn new(pipeline_stages: Vec<Box<dyn PipelineStage + Sync + Send>>,
               channel: CorticalChannelIndex,
               should_allow_sending_stale_data: bool
                ) -> Result<Self, FeagiDataError> {
        
        let processor_runner = PipelineStageRunner::new(pipeline_stages)?;
        Ok(SensoryChannelStreamCache {
            pipeline_runner: processor_runner,
            channel,
            last_updated: Instant::now(),
            should_allow_sending_stale_data: should_allow_sending_stale_data
        })
    }
    
    /// Updates the cache with a new sensor value and processes it through the chain.
    ///
    /// Takes raw sensor data, applies all configured processing in sequence,
    /// and updates the internal timestamp to mark when fresh data was received.
    /// The processed result becomes available through `get_most_recent_sensor_value()`.
    ///
    /// # Arguments
    ///
    /// * `value` - Raw sensor data to process and cache
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Data successfully processed and cached
    /// * `Err(FeagiDataProcessingError)` - If processing fails
    ///
    /// # Errors
    ///
    /// Returns an error if any processor in the chain fails to handle the data,
    /// typically due to data type mismatches or processing-specific failures.
    pub fn update_sensor_value(&mut self, value: WrappedIOData) -> Result<(), FeagiDataError> {
        self.last_updated = Instant::now();
        _ = self.pipeline_runner.update_value(&value, Instant::now())?;
        Ok(())
    }
    
    pub fn attempt_replace_pipeline_stages(&mut self, pipeline_stages: Vec<Box<dyn PipelineStage + Sync + Send>>) -> Result<(), FeagiDataError> {
        self.pipeline_runner.attempt_replace_stages(pipeline_stages)
    }

    pub fn attempt_replace_pipeline_stage(&mut self, pipeline_stage: Box<dyn PipelineStage + Sync + Send>, replacing_at: PipelineStageIndex) -> Result<(), FeagiDataError> {
        self.pipeline_runner.attempt_replace_stage(pipeline_stage, replacing_at)
    }

    pub fn clone_pipeline_stages(&self) -> Vec<Box<dyn PipelineStage + Sync + Send>> {
        self.pipeline_runner.clone_stages()
    }

    pub fn clone_pipeline_stage(&self, pipeline_stage_index: PipelineStageIndex) -> Result<Box<dyn PipelineStage + Sync + Send>, FeagiDataError> {
        self.pipeline_runner.clone_stage(pipeline_stage_index)
    }
    
    /// Determines whether new data should be pushed based on staleness policy.
    ///
    /// Evaluates whether the cache should provide data for neural processing
    /// based on the configured staleness policy and data freshness. This enables
    /// both real-time (fresh-only) and cached (always-available) data strategies.
    ///
    /// # Arguments
    ///
    /// * `past_push_time` - Timestamp of when data was last pushed to neural system
    ///
    /// # Returns
    ///
    /// `true` if data should be pushed, `false` if it should be skipped
    ///
    /// # Behavior
    ///
    /// - **Stale data allowed**: Always returns `true`
    /// - **Fresh data only**: Returns `true` only if data was updated after `past_push_time`
    pub fn should_push_new_value(&self, past_push_time: Instant) -> bool {
        self.should_allow_sending_stale_data || past_push_time < self.last_updated
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
    
    /// Returns the cortical I/O channel index for this cache.
    ///
    /// Provides the channel identifier that this cache is responsible for.
    /// This is useful for mapping between cached data and specific channels
    /// in the cortical area configuration.
    ///
    /// # Returns
    ///
    /// The `CorticalIOChannelIndex` for this cache
    pub fn get_cortical_io_channel_index(&self) -> CorticalChannelIndex {
        self.channel
    }

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
}


/*
// TODO add callback for only on change

pub struct MotorChannelStreamCache {
    stream_cache_processor: Box<dyn StreamCacheProcessor>,
    neuron_xyzp_decoder: Box<dyn NeuronXYZPDecoder>,
    channel: CorticalIOChannelIndex,
    last_updated: Instant,
    callbacks_all_bursts: CallBackManager
}

impl MotorChannelStreamCache {
    
    pub fn new(stream_cache_processor: Box<dyn StreamCacheProcessor>, 
               neuron_xyzp_decoder: Box<dyn NeuronXYZPDecoder>,
               channel: CorticalIOChannelIndex) -> Result<Self, FeagiDataProcessingError> {

        if stream_cache_processor.get_data_type() != neuron_xyzp_decoder.get_data_type() {
            return Err(FeagiDataProcessingError::InternalError("Stream Cache Processor and Neuron Decoder do not have matching data types!".into()));
        }
        
        Ok(MotorChannelStreamCache{
            stream_cache_processor,
            neuron_xyzp_decoder,
            channel,
            last_updated: Instant::now(),
            callbacks_all_bursts: CallBackManager::new()
        })
        
    }
    
    pub fn decode_from_neurons(&mut self, neuron_data: &NeuronXYZPArrays) -> Result<&IOTypeData, FeagiDataProcessingError> {
        let decoded_value: IOTypeData = self.neuron_xyzp_decoder.read_neuron_data_single_channel(
            neuron_data,
            self.channel
        )?;
        self.last_updated = Instant::now();
        self.stream_cache_processor.process_new_input(&decoded_value)
    }
    
    pub fn is_more_recent_than_given(&self, time: Instant) -> bool {
        time < self.last_updated
    }
    
    pub fn get_most_recent_motor_value(&self) -> &IOTypeData {
        self.stream_cache_processor.get_most_recent_output()
    }

    pub fn get_input_data_type(&self) -> IOTypeVariant {
        self.stream_cache_processor.get_input_data_type()
    }

    pub fn get_output_data_type(&self) -> IOTypeVariant {
        self.stream_cache_processor.get_output_data_type()
    }
    
    // TODO allow registering callbacks
}

 */

