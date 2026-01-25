use crate::configuration::jsonable::{
    JSONDeviceGrouping, JSONEncoderProperties, JSONUnitDefinition,
};
use crate::data_pipeline::per_channel_stream_caches::{
    PipelineStageRunner, SensoryPipelineStageRunner,
};
use crate::data_pipeline::{PipelineStageProperties, PipelineStagePropertyIndex};
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPEncoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelIndex, CorticalUnitIndex,
};
use feagi_structures::genomic::SensoryCorticalUnit;
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use feagi_structures::FeagiDataError;
use std::time::Instant;

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

#[derive(Debug)]
pub(crate) struct SensoryCorticalUnitCache {
    neuron_encoder: Box<dyn NeuronVoxelXYZPEncoder>,
    io_configuration_flags: serde_json::Map<String, serde_json::Value>,
    pipeline_runners: Vec<SensoryPipelineStageRunner>,
    last_update_time: Instant,
    device_friendly_name: Option<String>,
}

impl SensoryCorticalUnitCache {
    pub fn new(
        neuron_encoder: Box<dyn NeuronVoxelXYZPEncoder>,
        io_configuration_flags: serde_json::Map<String, serde_json::Value>, // This MUST be formatted correctly
        number_channels: CorticalChannelCount,
        initial_cached_value: WrappedIOData,
    ) -> Result<Self, FeagiDataError> {
        let pipeline_runners: Vec<SensoryPipelineStageRunner> = std::iter::repeat_with(|| {
            SensoryPipelineStageRunner::new(initial_cached_value.clone()).unwrap()
        })
        .take(*number_channels as usize)
        .collect();

        Ok(Self {
            neuron_encoder,
            io_configuration_flags,
            pipeline_runners,
            last_update_time: Instant::now(),
            device_friendly_name: None,
        })
    }

    /// Creates new unit, and updates all internal; channel / pipeline runners and metadata
    /// according to the given json
    pub fn new_from_json(
        sensory_unit: &SensoryCorticalUnit,
        unit_definition: &JSONUnitDefinition,
        encoder_definition: &JSONEncoderProperties,
    ) -> Result<Self, FeagiDataError> {
        unit_definition.verify_valid_structure()?;
        let channel_count = unit_definition.get_channel_count()?;
        let cortical_ids = sensory_unit
            .get_cortical_id_vector_from_index_and_serde_io_configuration_flags(
                unit_definition.cortical_unit_index,
                unit_definition.io_configuration_flags.clone(),
            )?;

        let initial_value = encoder_definition.default_wrapped_value()?;
        let encoder = encoder_definition.to_box_encoder(channel_count, &cortical_ids)?;

        let mut sensory_cortical_unit_cache = SensoryCorticalUnitCache::new(
            encoder,
            unit_definition.io_configuration_flags.clone(),
            channel_count,
            initial_value,
        )?;

        let _ =
            sensory_cortical_unit_cache.set_friendly_name(unit_definition.friendly_name.clone());

        // Update all the channels
        for (index, device_group) in unit_definition.device_grouping.iter().enumerate() {
            let pipeline_runner = sensory_cortical_unit_cache
                .pipeline_runners
                .get_mut(index)
                .unwrap();

            // Use replace_all_stages when importing from JSON since we're setting up a new pipeline
            // This works even when pipeline is empty (0 stages) unlike try_update_all_stage_properties
            if !device_group.pipeline_stages.is_empty() {
                pipeline_runner.try_replace_all_stages(device_group.pipeline_stages.clone())?;
            }
            pipeline_runner.set_channel_friendly_name(device_group.friendly_name.clone());
            pipeline_runner.set_channel_index_override(device_group.channel_index_override);
            pipeline_runner.set_json_device_properties(device_group.device_properties.clone());
        }

        Ok(sensory_cortical_unit_cache)
    }

    pub fn export_as_jsons(
        &self,
        cortical_unit_index: CorticalUnitIndex,
    ) -> (JSONUnitDefinition, JSONEncoderProperties) {
        let encoder_properties = self.neuron_encoder.get_as_properties();
        let json_unit_definition = JSONUnitDefinition {
            friendly_name: self.device_friendly_name.clone(),
            cortical_unit_index,
            io_configuration_flags: self.io_configuration_flags.clone(),
            device_grouping: self.get_all_device_grouping(),
        };
        (json_unit_definition, encoder_properties)
    }

    //region Properties

    /// Returns the total number of cortical channels managed by this cache system.
    ///
    /// # Returns
    /// The count of channels as a `CorticalChannelCount`.
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn verify_channel_exists(
        &self,
        channel_index: CorticalChannelIndex,
    ) -> Result<(), FeagiDataError> {
        _ = self.try_get_pipeline_runner(channel_index)?;
        Ok(())
    }

    /// Retrieves the expected input data type for a channel
    #[allow(dead_code)]
    pub fn get_input_type_for_channel(
        &self,
        cortical_channel_index: CorticalChannelIndex,
    ) -> Result<WrappedIOType, FeagiDataError> {
        let runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(runner.get_expected_type_to_input_and_process())
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
    #[allow(dead_code)]
    pub fn try_get_channel_preprocessed_value(
        &self,
        cortical_channel_index: CorticalChannelIndex,
    ) -> Result<&WrappedIOData, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_preprocessed_cached_value())
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
    pub fn try_get_channel_recent_postprocessed_value(
        &self,
        cortical_channel_index: CorticalChannelIndex,
    ) -> Result<&WrappedIOData, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_postprocessed_sensor_value())
    }

    pub fn try_replace_input_channel_cache_value_and_run_pipeline(
        &mut self,
        cortical_channel_index: CorticalChannelIndex,
        value: WrappedIOData,
        update_instant: Instant,
    ) -> Result<&WrappedIOData, FeagiDataError> {
        // We assume value is of correct type
        self.last_update_time = update_instant; // TODO cant this cause weird issues?
        let runner = self.try_get_pipeline_runner_mut(cortical_channel_index)?;
        runner.verify_input_sensor_data(&value)?;
        let mut_val = runner.get_preprocessed_cached_value_mut();
        *mut_val = value;
        let processed_data = runner.process_cached_sensor_value(update_instant)?;
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
    #[allow(dead_code)]
    pub fn try_get_channel_update_instant(
        &self,
        cortical_channel_index: CorticalChannelIndex,
    ) -> Result<Instant, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_last_processed_instant())
    }

    //endregion

    //region Pipeline Runner Stages

    pub fn try_get_first_index_of_stage_property_type(
        &self,
        cortical_channel_index: CorticalChannelIndex,
        pipeline_stage_property_type: &PipelineStageProperties,
    ) -> Result<PipelineStagePropertyIndex, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        pipeline_runner.try_get_index_of_first_stage_property_of_type(pipeline_stage_property_type)
    }

    /// Retrieves the properties of a single pipeline stage for a specific channel.
    ///
    /// # Arguments
    /// * `cortical_channel_index` - The channel to query
    /// * `pipeline_stage_property_index` - The index of the stage within the channel's pipeline
    ///
    /// # Returns
    /// * `Ok(Box<dyn PipelineStageProperties>)` - The stage's properties
    /// * `Err(FeagiDataError)` - If channel or stage index is out of bounds
    #[allow(dead_code)]
    pub fn try_get_single_stage_properties(
        &self,
        cortical_channel_index: CorticalChannelIndex,
        pipeline_stage_property_index: PipelineStagePropertyIndex,
    ) -> Result<PipelineStageProperties, FeagiDataError> {
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
    #[allow(dead_code)]
    pub fn get_all_stage_properties(
        &self,
        cortical_channel_index: CorticalChannelIndex,
    ) -> Result<Vec<PipelineStageProperties>, FeagiDataError> {
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
    #[allow(dead_code)]
    pub fn try_update_single_stage_properties(
        &mut self,
        cortical_channel_index: CorticalChannelIndex,
        pipeline_stage_property_index: PipelineStagePropertyIndex,
        replacing_property: PipelineStageProperties,
    ) -> Result<(), FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner_mut(cortical_channel_index)?;
        pipeline_runner
            .try_update_single_stage_properties(pipeline_stage_property_index, replacing_property)
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
    #[allow(dead_code)]
    pub fn try_update_all_stage_properties(
        &mut self,
        cortical_channel_index: CorticalChannelIndex,
        new_pipeline_stage_properties: Vec<PipelineStageProperties>,
    ) -> Result<(), FeagiDataError> {
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
    #[allow(dead_code)]
    pub fn try_replace_single_stage(
        &mut self,
        cortical_channel_index: CorticalChannelIndex,
        replacing_at_index: PipelineStagePropertyIndex,
        new_pipeline_stage_properties: PipelineStageProperties,
    ) -> Result<(), FeagiDataError> {
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
    #[allow(dead_code)]
    pub fn try_replace_all_stages(
        &mut self,
        cortical_channel_index: CorticalChannelIndex,
        new_pipeline_stage_properties: Vec<PipelineStageProperties>,
    ) -> Result<(), FeagiDataError> {
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
    #[allow(dead_code)]
    pub fn try_removing_all_stages(
        &mut self,
        cortical_channel_index: CorticalChannelIndex,
    ) -> Result<(), FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner_mut(cortical_channel_index)?;
        pipeline_runner.try_removing_all_stages()?;
        Ok(())
    }

    //endregion

    //region Metadata

    #[allow(dead_code)]
    pub fn get_friendly_name(&self) -> &Option<String> {
        &self.device_friendly_name
    }

    pub fn set_friendly_name(
        &mut self,
        friendly_name: Option<String>,
    ) -> Result<(), FeagiDataError> {
        self.device_friendly_name = friendly_name;
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
    pub(crate) fn update_neuron_data_with_recently_updated_cached_sensor_data(
        &mut self,
        neuron_data: &mut CorticalMappedXYZPNeuronVoxels,
        time_of_burst: Instant,
    ) -> Result<(), FeagiDataError> {
        // TODO We need a new trait method to just have all channels cleared if not up to date
        // Note: We expect neuron data to be cleared before this step
        self.neuron_encoder
            .write_neuron_data_multi_channel_from_processed_cache(
                &self.pipeline_runners,
                time_of_burst,
                neuron_data,
            )?;

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
    fn try_get_pipeline_runner(
        &self,
        cortical_channel_index: CorticalChannelIndex,
    ) -> Result<&SensoryPipelineStageRunner, FeagiDataError> {
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
    fn try_get_pipeline_runner_mut(
        &mut self,
        cortical_channel_index: CorticalChannelIndex,
    ) -> Result<&mut SensoryPipelineStageRunner, FeagiDataError> {
        let runner_count = self.pipeline_runners.len();
        match self.pipeline_runners.get_mut(*cortical_channel_index as usize) {
            Some(pipeline_runner) => Ok(pipeline_runner),
            None => Err(FeagiDataError::BadParameters(format!("Channel Index {} is out of bounds for SensoryChannelStreamCaches with {} channels!",
                                                              cortical_channel_index, runner_count)))
        }
    }

    #[allow(dead_code)]
    fn get_encoder_json_properties(&self) -> Result<JSONEncoderProperties, FeagiDataError> {
        Ok(self.neuron_encoder.get_as_properties())
    }

    fn get_all_device_grouping(&self) -> Vec<JSONDeviceGrouping> {
        let mut output: Vec<JSONDeviceGrouping> = Vec::new();
        for group in self.pipeline_runners.iter() {
            output.push(group.export_as_json_device_grouping())
        }
        output
    }

    //endregion
}
