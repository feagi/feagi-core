use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::descriptors::CorticalChannelIndex;
use feagi_data_structures::neurons::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZPDecoder};
use feagi_data_structures::wrapped_io_data::{WrappedIOData, WrappedIOType};
use crate::data_pipeline::{PipelineStage, PipelineStageIndex, PipelineStageRunner};

#[derive(Debug)]
pub(crate) struct MotorChannelStreamCache {
    pipeline_runner: PipelineStageRunner,
    channel: CorticalChannelIndex,
    last_updated: Instant,
    should_emit_stale_data: bool,
    most_recent_direct_decoded_output: WrappedIOData,
}

impl MotorChannelStreamCache {

    pub fn new(pipeline_stages: Vec<Box<dyn PipelineStage + Sync + Send>>,
               channel: CorticalChannelIndex,
               should_emit_stale_data: bool
    ) -> Result<Self, FeagiDataError> {

        let processor_runner = PipelineStageRunner::new(pipeline_stages)?;
        Ok(MotorChannelStreamCache {
            pipeline_runner: processor_runner,
            channel,
            last_updated: Instant::now(),
            should_emit_stale_data,
            most_recent_direct_decoded_output: WrappedIOData::F32(0.0), // Placeholder
        })
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

    /// Returns the most recently processed sensor value.
    ///
    /// Provides access to the latest data that has been processed through
    /// the entire processor chain. This data is ready for neural encoding
    /// or external consumption.
    ///
    /// # Returns
    ///
    /// Reference to the most recent processed sensor data
    pub fn get_most_recent_motor_value(&self) -> &WrappedIOData {
        self.pipeline_runner.get_most_recent_output()
    }

    pub fn decode_from_neurons(&mut self, cortical_mapped_neuron_data: &CorticalMappedXYZPNeuronData, decoder: &Box<dyn NeuronXYZPDecoder + Sync + Send>) -> Result<(), FeagiDataError> {

        let is_updated = decoder.read_neuron_data_single_channel(cortical_mapped_neuron_data, self.channel, self.pipeline_runner.get_current_input_mut())?;
        if is_updated {
            self.last_updated = Instant::now();
        }
        Ok(())
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