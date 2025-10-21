use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::descriptors::{CorticalChannelCount, CorticalChannelIndex};
use feagi_data_structures::neuron_voxels::xyzp::{CorticalMappedXYZPNeuronVoxels};
use crate::data_pipeline::{PipelineStageProperties, PipelineStagePropertyIndex, PipelineStageRunner};
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPEncoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

/// Manages multiple sensory data streams with independent processing pipelines per channel.
///
/// This structure maintains a collection of processing pipelines, one for each cortical channel,
/// allowing different sensory inputs to be processed independently before being encoded into
/// neuron voxel data. Each channel can have its own pipeline stages for data transformation.
///
/// # Fields
/// - `neuron_encoder`: Encoder that converts processed data into neuron voxel representations
/// - `pipeline_runners`: Collection of pipeline runners, one per cortical channel
/// - `last_update_time`: Timestamp of the most recent update across all channels
pub(crate) struct SensoryChannelStreamCaches {
    neuron_encoder: Box<dyn NeuronVoxelXYZPEncoder>,
    pipeline_runners: Vec<PipelineStageRunner>,
    last_update_time: Instant,
}

impl SensoryChannelStreamCaches {

    /// Creates a new multi-channel sensory stream cache system.
    ///
    /// Initializes a pipeline runner for each channel with the provided stage properties.
    /// Each channel processes data independently through its own pipeline before encoding.
    /// All channels start with the same initial cached value.
    ///
    /// # Arguments
    /// * `neuron_encoder` - Encoder for converting processed data to neuron voxels
    /// * `initial_cached_value` - Starting value for all channel caches
    /// * `stage_properties_per_channels` - Vector of pipeline stage properties for each channel
    ///
    /// # Returns
    /// * `Ok(SensoryChannelStreamCaches)` - Successfully created cache system
    /// * `Err(FeagiDataError)` - If pipeline initialization fails for any channel, or zero channels are submitted
    pub fn new(neuron_encoder: Box<dyn NeuronVoxelXYZPEncoder>, initial_cached_value: WrappedIOData, stage_properties_per_channels: Vec<Vec<Box<dyn PipelineStageProperties + Sync + Send>>>) -> Result<Self, FeagiDataError> {

        if stage_properties_per_channels.is_empty() {
            return Err(FeagiDataError::BadParameters("Cannot create a sensor stream cache with 0 channels!".into()))
        }

        let expected_data_encoded_type: WrappedIOType = neuron_encoder.get_encodable_data_type();

        let mut pipeline_runners: Vec<PipelineStageRunner> = Vec::with_capacity(stage_properties_per_channels.len());
        
        for stage_properties_per_channel in stage_properties_per_channels {
            pipeline_runners.push(PipelineStageRunner::new(stage_properties_per_channel, initial_cached_value.clone(), expected_data_encoded_type)?); // No need to optimize away the clone() here, this isn't being called often
        }

        Ok(Self {
            neuron_encoder,
            pipeline_runners,
            last_update_time: Instant::now(),
        })
    }

    //region Properties

    /// Returns the total number of cortical channels managed by this cache system.
    ///
    /// # Returns
    /// The count of channels as a `CorticalChannelCount`.
    pub fn number_of_channels(&self) -> CorticalChannelCount {
        (self.pipeline_runners.len() as u32).try_into().unwrap()
    }

    /// Verifies that a channel with the given index exists in this cache system.
    ///
    /// # Arguments
    /// * `channel_index` - The index of the channel to verify
    ///
    /// # Returns
    /// * `Ok(())` - If the channel exists
    /// * `Err(FeagiDataError)` - If the channel index is out of bounds
    pub fn verify_channel_exists(&self, channel_index: CorticalChannelIndex) -> Result<(), FeagiDataError> {
        _ =  self.try_get_pipeline_runner(channel_index)?;
        Ok(())
    }

    /// Retrieves the expected input data type for any of the channels
    ///
    ///
    /// # Returns
    /// * `WrappedIOType` - The input type expected
    pub fn get_input_type(&self) -> WrappedIOType {
        self.pipeline_runners.first().unwrap().get_input_data_type()
    }

    //endregion

    //region Pipeline Runner Data

    /// Retrieves the most recent raw input value for a channel before pipeline processing.
    ///
    /// Returns the cached input data that was provided to the channel, before any
    /// pipeline stages were applied.
    ///
    /// # Arguments
    /// * `cortical_channel_index` - The channel to query
    ///
    /// # Returns
    /// * `Ok(&WrappedIOData)` - Reference to the unprocessed input data
    /// * `Err(FeagiDataError)` - If the channel index is out of bounds
    pub fn try_get_channel_recent_preprocessed_value(&self, cortical_channel_index: CorticalChannelIndex) -> Result<&WrappedIOData, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_most_recent_preprocessed_output())
    }

    /// Retrieves the most recent processed output value for a channel after pipeline processing.
    ///
    /// Returns the output data after all pipeline stages have been applied. If no pipeline
    /// stages exist, returns the cached input data.
    ///
    /// # Arguments
    /// * `cortical_channel_index` - The channel to query
    ///
    /// # Returns
    /// * `Ok(&WrappedIOData)` - Reference to the processed output data
    /// * `Err(FeagiDataError)` - If the channel index is out of bounds
    pub fn try_get_channel_recent_postprocessed_value(&self, cortical_channel_index: CorticalChannelIndex) -> Result<&WrappedIOData, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_most_recent_postprocessed_output())
    }


    pub fn try_replace_input_channel_cache_value_and_run_pipeline(&mut self, cortical_channel_index: CorticalChannelIndex, value: WrappedIOData, update_instant: Instant) -> Result<&WrappedIOData, FeagiDataError> {
        // We assume value is of correct type
        self.last_update_time = update_instant;// TODO cant this cause weird issues?
        let runner = self.try_get_pipeline_runner_mut(cortical_channel_index)?;
        let mut_val = runner.get_cached_input_mut();
        *mut_val = value;
        let processed_data = runner.process_cached_input_value(update_instant)?;
        Ok(processed_data)
    }

    pub fn try_running_pipeline_runner_from_input_cache(&mut self, cortical_channel_index: CorticalChannelIndex, update_instant: Instant) -> Result<&WrappedIOData, FeagiDataError> {
        let runner = self.try_get_pipeline_runner_mut(cortical_channel_index)?;
        let processed_data = runner.process_cached_input_value(update_instant)?;
        Ok(processed_data)
    }

    /// Retrieves the timestamp of the last data update for a specific channel.
    ///
    /// # Arguments
    /// * `cortical_channel_index` - The channel to query
    ///
    /// # Returns
    /// * `Ok(Instant)` - The timestamp when the channel was last updated
    /// * `Err(FeagiDataError)` - If the channel index is out of bounds
    pub fn try_get_channel_update_instant(&self, cortical_channel_index: CorticalChannelIndex) -> Result<Instant, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_last_processed_instant())
    }

    //endregion

    //region Pipeline Runner Stages

    /// Retrieves the properties of a single pipeline stage for a specific channel.
    ///
    /// # Arguments
    /// * `cortical_channel_index` - The channel to query
    /// * `pipeline_stage_property_index` - The index of the stage within the channel's pipeline
    ///
    /// # Returns
    /// * `Ok(Box<dyn PipelineStageProperties>)` - The stage's properties
    /// * `Err(FeagiDataError)` - If channel or stage index is out of bounds
    pub fn try_get_single_stage_properties(&self, cortical_channel_index: CorticalChannelIndex, pipeline_stage_property_index: PipelineStagePropertyIndex) -> Result<Box<dyn PipelineStageProperties + Sync + Send>, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        pipeline_runner.try_get_single_stage_properties(pipeline_stage_property_index)
    }

    /// Retrieves the properties of all pipeline stages for a specific channel.
    ///
    /// # Arguments
    /// * `cortical_channel_index` - The channel to query
    ///
    /// # Returns
    /// * `Ok(Vec<Box<dyn PipelineStageProperties>>)` - Properties for all stages in order
    /// * `Err(FeagiDataError)` - If the channel index is out of bounds
    pub fn get_all_stage_properties(&self, cortical_channel_index: CorticalChannelIndex) -> Result<Vec<Box<dyn PipelineStageProperties + Sync + Send>>, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_all_stage_properties())
    }

    /// Updates the properties of a single pipeline stage for a specific channel.
    ///
    /// # Arguments
    /// * `cortical_channel_index` - The channel containing the stage
    /// * `pipeline_stage_property_index` - The index of the stage to update
    /// * `replacing_property` - The new properties to apply
    ///
    /// # Returns
    /// * `Ok(())` - If the properties were successfully updated
    /// * `Err(FeagiDataError)` - If channel/stage index is invalid or update fails
    pub fn try_update_single_stage_properties(&mut self, cortical_channel_index: CorticalChannelIndex, pipeline_stage_property_index: PipelineStagePropertyIndex, replacing_property: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<(), FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner_mut(cortical_channel_index)?;
        pipeline_runner.try_update_single_stage_properties(pipeline_stage_property_index, replacing_property)
    }

    /// Updates the properties of all pipeline stages for a specific channel.
    ///
    /// # Arguments
    /// * `cortical_channel_index` - The channel to update
    /// * `new_pipeline_stage_properties` - New properties for all stages
    ///
    /// # Returns
    /// * `Ok(())` - If all properties were successfully updated
    /// * `Err(FeagiDataError)` - If channel is invalid, count mismatch, or update fails
    pub fn try_update_all_stage_properties(&mut self, cortical_channel_index: CorticalChannelIndex, new_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<(), FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner_mut(cortical_channel_index)?;
        pipeline_runner.try_update_all_stage_properties(new_pipeline_stage_properties)
    }

    /// Replaces a single pipeline stage with a new stage for a specific channel.
    ///
    /// # Arguments
    /// * `cortical_channel_index` - The channel containing the stage
    /// * `replacing_at_index` - The index of the stage to replace
    /// * `new_pipeline_stage_properties` - Properties for the new stage
    ///
    /// # Returns
    /// * `Ok(())` - If the stage was successfully replaced
    /// * `Err(FeagiDataError)` - If indices are invalid or types are incompatible
    pub fn try_replace_single_stage(&mut self, cortical_channel_index: CorticalChannelIndex, replacing_at_index: PipelineStagePropertyIndex, new_pipeline_stage_properties: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<(), FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner_mut(cortical_channel_index)?;
        pipeline_runner.try_replace_single_stage(replacing_at_index, new_pipeline_stage_properties)
    }

    /// Replaces all pipeline stages with new stages for a specific channel.
    ///
    /// # Arguments
    /// * `cortical_channel_index` - The channel to update
    /// * `new_pipeline_stage_properties` - Properties for all new stages
    ///
    /// # Returns
    /// * `Ok(())` - If all stages were successfully replaced
    /// * `Err(FeagiDataError)` - If channel is invalid or stages are incompatible
    pub fn try_replace_all_stages(&mut self, cortical_channel_index: CorticalChannelIndex, new_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<(), FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner_mut(cortical_channel_index)?;
        pipeline_runner.try_replace_all_stages(new_pipeline_stage_properties)
    }

    /// Removes all pipeline stages from a specific channel.
    ///
    /// This is only possible if the channel's input and output types are the same,
    /// allowing data to pass through unchanged.
    ///
    /// # Arguments
    /// * `cortical_channel_index` - The channel to clear
    ///
    /// # Returns
    /// * `Ok(())` - If all stages were successfully removed
    /// * `Err(FeagiDataError)` - If channel is invalid or input/output types don't match
    pub fn try_removing_all_stages(&mut self, cortical_channel_index: CorticalChannelIndex) -> Result<(), FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner_mut(cortical_channel_index)?;
        pipeline_runner.try_removing_all_stages()?;
        Ok(())
    }

    //endregion

    /// Encodes recently updated sensor data into neuron voxel representations.
    ///
    /// Uses the configured neuron encoder to convert processed pipeline data from all
    /// channels into neuron voxel data. This updates the provided neuron data structure
    /// with encoded representations of the sensor inputs.
    ///
    /// # Arguments
    /// * `neuron_data` - Neuron voxel data structure to update (should be cleared beforehand)
    /// * `time_of_burst` - Timestamp for this encoding burst
    ///
    /// # Returns
    /// * `Ok(())` - If encoding succeeded
    /// * `Err(FeagiDataError)` - If encoding fails
    ///
    /// # Note
    /// The neuron data is expected to be cleared before calling this method.
    pub(crate) fn update_neuron_data_with_recently_updated_cached_sensor_data(&mut self, neuron_data: &mut CorticalMappedXYZPNeuronVoxels, time_of_burst: Instant) -> Result<(), FeagiDataError> {
        // TODO We need a new trait method to just have all channels cleared if not up to date
        // Note: We expect neuron data to be cleared before this step
        self.neuron_encoder.write_neuron_data_multi_channel_from_processed_cache(&self.pipeline_runners, time_of_burst, neuron_data)?;
        Ok(())
    }

    //region Internal

    /// Retrieves an immutable reference to a pipeline runner for a specific channel.
    ///
    /// Internal helper method for accessing pipeline runners by channel index.
    ///
    /// # Arguments
    /// * `cortical_channel_index` - The channel index to retrieve
    ///
    /// # Returns
    /// * `Ok(&PipelineStageRunner)` - Reference to the pipeline runner
    /// * `Err(FeagiDataError)` - If the channel index is out of bounds
    #[inline]
    fn try_get_pipeline_runner(&self, cortical_channel_index: CorticalChannelIndex) -> Result<&PipelineStageRunner, FeagiDataError> {
        match self.pipeline_runners.get(*cortical_channel_index as usize) {
            Some(pipeline_runner) => Ok(pipeline_runner),
            None => Err(FeagiDataError::BadParameters(format!("Channel Index {} is out of bounds for SensoryChannelStreamCaches with {} channels!",
                                                              cortical_channel_index, self.pipeline_runners.len())))
        }

    }

    /// Retrieves a mutable reference to a pipeline runner for a specific channel.
    ///
    /// Internal helper method for accessing pipeline runners by channel index when
    /// mutation is required.
    ///
    /// # Arguments
    /// * `cortical_channel_index` - The channel index to retrieve
    ///
    /// # Returns
    /// * `Ok(&mut PipelineStageRunner)` - Mutable reference to the pipeline runner
    /// * `Err(FeagiDataError)` - If the channel index is out of bounds
    #[inline]
    fn try_get_pipeline_runner_mut(&mut self, cortical_channel_index: CorticalChannelIndex) -> Result<&mut PipelineStageRunner, FeagiDataError> {
        let runner_count = self.pipeline_runners.len();
        match self.pipeline_runners.get_mut(*cortical_channel_index as usize) {
            Some(pipeline_runner) => Ok(pipeline_runner),
            None => Err(FeagiDataError::BadParameters(format!("Channel Index {} is out of bounds for SensoryChannelStreamCaches with {} channels!",
                                                              cortical_channel_index, runner_count)))
        }

    }

    //endregion

}

