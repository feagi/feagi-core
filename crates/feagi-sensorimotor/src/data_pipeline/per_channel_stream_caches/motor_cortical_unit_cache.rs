use crate::data_pipeline::per_channel_stream_caches::pipeline_stage_runner_common::PipelineStageRunner;
use crate::data_pipeline::per_channel_stream_caches::MotorPipelineStageRunner;
use crate::data_pipeline::{PipelineStageProperties, PipelineStagePropertyIndex};
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPDecoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};
use feagi_structures::genomic::cortical_area::descriptors::{CorticalChannelCount, CorticalChannelIndex, CorticalUnitIndex};
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use feagi_structures::{FeagiDataError, FeagiSignal, FeagiSignalIndex};
use rayon::prelude::*;
use std::time::Instant;
use feagi_structures::genomic::MotorCorticalUnit;
use crate::configuration::jsonable::{JSONDecoderProperties, JSONDeviceGrouping, JSONUnitDefinition};

#[derive(Debug)]
pub(crate) struct MotorCorticalUnitCache {
    neuron_decoder: Box<dyn NeuronVoxelXYZPDecoder>,
    io_configuration_flags: serde_json::Map<String, serde_json::Value>,
    pipeline_runners: Vec<MotorPipelineStageRunner>,
    has_channel_been_updated: Vec<bool>,
    value_updated_callbacks: Vec<FeagiSignal<WrappedIOData>>,
    device_friendly_name: Option<String>,
}

impl MotorCorticalUnitCache {
    pub fn new(
        neuron_decoder: Box<dyn NeuronVoxelXYZPDecoder>,
        io_configuration_flags: serde_json::Map<String, serde_json::Value>, // This MUST be formatted correctly
        number_channels: CorticalChannelCount,
        initial_cached_value: WrappedIOData,
    ) -> Result<Self, FeagiDataError> {
        let pipeline_runners: Vec<MotorPipelineStageRunner> = std::iter::repeat_with(|| {
            MotorPipelineStageRunner::new(initial_cached_value.clone()).unwrap()
        })
        .take(*number_channels as usize)
        .collect();

        // One signal per channel for "data processed" callbacks.
        // IMPORTANT: this must be a fully-sized Vec, not just reserved capacity, otherwise
        // callback registration will panic on indexing.
        let callbacks: Vec<FeagiSignal<WrappedIOData>> = (0..*number_channels as usize)
            .map(|_| FeagiSignal::new())
            .collect();

        Ok(Self {
            neuron_decoder,
            io_configuration_flags,
            pipeline_runners,
            has_channel_been_updated: vec![false; *number_channels as usize],
            value_updated_callbacks: callbacks,
            device_friendly_name: None,
        })
    }

    /// Creates new unit, and updates all internal; channel / pipeline runners and metadata
    /// according to the given json
    pub fn new_from_json(
        motor_unit: &MotorCorticalUnit,
        unit_definition: &JSONUnitDefinition,
        decoder_definition: &JSONDecoderProperties
    ) -> Result<Self, FeagiDataError> {

        unit_definition.verify_valid_structure()?;
        let channel_count = unit_definition.get_channel_count()?;
        let cortical_ids = motor_unit.get_cortical_id_vector_from_index_and_serde_io_configuration_flags(
            unit_definition.cortical_unit_index,
            unit_definition.io_configuration_flags.clone()
        )?;

        let initial_value = decoder_definition.default_wrapped_value()?;
        let encoder = decoder_definition.to_box_decoder(
            channel_count,
            &cortical_ids
        )?;

        let mut motor_cortical_unit_cache = MotorCorticalUnitCache::new(
            encoder,
            unit_definition.io_configuration_flags.clone(),
            channel_count,
            initial_value
        )?;

        let _ = motor_cortical_unit_cache.set_friendly_name(unit_definition.friendly_name.clone());

        // Update all the channels
        for (index, device_group) in unit_definition.device_grouping.iter().enumerate() {
            let pipeline_runner = motor_cortical_unit_cache.pipeline_runners.get_mut(index).unwrap();

            pipeline_runner.try_update_all_stage_properties(device_group.pipeline_stages.clone())?;
            pipeline_runner.set_channel_friendly_name(device_group.friendly_name.clone());
            pipeline_runner.set_channel_index_override(device_group.channel_index_override);
            pipeline_runner.set_json_device_properties(device_group.device_properties.clone());
        }

        Ok(motor_cortical_unit_cache)
    }

    pub fn export_as_jsons(&self, cortical_unit_index: CorticalUnitIndex) -> (JSONUnitDefinition, JSONDecoderProperties) {
        let encoder_properties = self.neuron_decoder.get_as_properties();
        let json_unit_definition = JSONUnitDefinition {
            friendly_name: self.device_friendly_name.clone(),
            cortical_unit_index,
            io_configuration_flags: self.io_configuration_flags.clone(),
            device_grouping: self.get_all_device_grouping()
        };
        (json_unit_definition, encoder_properties)
    }



    //region Properties

    #[allow(dead_code)]
    pub fn number_of_channels(&self) -> CorticalChannelCount {
        (self.pipeline_runners.len() as u32).try_into().unwrap()
    }

    #[allow(dead_code)]
    pub fn verify_channel_exists(
        &self,
        channel_index: CorticalChannelIndex,
    ) -> Result<(), FeagiDataError> {
        _ = self.try_get_pipeline_runner(channel_index)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_output_type_for_channel(
        &self,
        cortical_channel_index: CorticalChannelIndex,
    ) -> Result<WrappedIOType, FeagiDataError> {
        let runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(runner.get_expected_type_to_output_after_processing())
    }

    //endregion

    //region Pipeline Runner Data

    pub fn get_preprocessed_motor_value(
        &self,
        cortical_channel_index: CorticalChannelIndex,
    ) -> Result<&WrappedIOData, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_preprocessed_cached_value())
    }

    pub fn get_postprocessed_motor_value(
        &self,
        cortical_channel_index: CorticalChannelIndex,
    ) -> Result<&WrappedIOData, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_postprocessed_cached_value())
    }

    #[allow(dead_code)]
    pub fn try_get_channel_last_processed_instant(
        &self,
        cortical_channel_index: CorticalChannelIndex,
    ) -> Result<Instant, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_last_processed_instant())
    }

    //endregion

    //region Pipeline Runner Stages

    pub fn try_get_single_stage_properties(
        &self,
        cortical_channel_index: CorticalChannelIndex,
        stage_index: PipelineStagePropertyIndex,
    ) -> Result<PipelineStageProperties, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        pipeline_runner.try_get_single_stage_properties(stage_index)
    }

    pub fn get_all_stage_properties(
        &self,
        cortical_channel_index: CorticalChannelIndex,
    ) -> Result<Vec<PipelineStageProperties>, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_all_stage_properties())
    }

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

    pub fn try_update_all_stage_properties(
        &mut self,
        cortical_channel_index: CorticalChannelIndex,
        new_pipeline_stage_properties: Vec<PipelineStageProperties>,
    ) -> Result<(), FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner_mut(cortical_channel_index)?;
        pipeline_runner.try_update_all_stage_properties(new_pipeline_stage_properties)
    }

    pub fn try_replace_single_stage(
        &mut self,
        cortical_channel_index: CorticalChannelIndex,
        replacing_at_index: PipelineStagePropertyIndex,
        new_pipeline_stage_properties: PipelineStageProperties,
    ) -> Result<(), FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner_mut(cortical_channel_index)?;
        pipeline_runner.try_replace_single_stage(replacing_at_index, new_pipeline_stage_properties)
    }

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

    pub fn set_friendly_name(&mut self, friendly_name: Option<String>) -> Result<(), FeagiDataError> {
        self.device_friendly_name = friendly_name;
        Ok(())
    }

    //endregion

    //region Callbacks

    pub fn try_connect_to_data_processed_signal<F>(
        &mut self,
        cortical_channel_index: CorticalChannelIndex,
        callback: F,
    ) -> Result<FeagiSignalIndex, FeagiDataError>
    where
        F: Fn(&WrappedIOData) + Send + Sync + 'static,
    {
        _ = self.try_get_pipeline_runner(cortical_channel_index)?;
        let idx = *cortical_channel_index as usize;
        let callbacks_len = self.value_updated_callbacks.len();
        let runners_len = self.pipeline_runners.len();
        let signal = self
            .value_updated_callbacks
            .get_mut(idx)
            .ok_or_else(|| {
                FeagiDataError::BadParameters(format!(
                    "Callback signal index {} is out of bounds (callbacks_len={}, runners_len={}). \
                     This indicates an internal cache invariant violation.",
                    idx,
                    callbacks_len,
                    runners_len
                ))
            })?;
        Ok(signal.connect(callback))
    }

    #[allow(dead_code)]
    pub fn try_disconnect_to_data_processed_signal(
        &mut self,
        cortical_channel_index: CorticalChannelIndex,
        signal_index: FeagiSignalIndex,
    ) -> Result<(), FeagiDataError> {
        _ = self.try_get_pipeline_runner(cortical_channel_index)?;
        let idx = *cortical_channel_index as usize;
        let callbacks_len = self.value_updated_callbacks.len();
        let runners_len = self.pipeline_runners.len();
        let signal = self
            .value_updated_callbacks
            .get_mut(idx)
            .ok_or_else(|| {
                FeagiDataError::BadParameters(format!(
                    "Callback signal index {} is out of bounds (callbacks_len={}, runners_len={}). \
                     This indicates an internal cache invariant violation.",
                    idx,
                    callbacks_len,
                    runners_len
                ))
            })?;
        signal.disconnect(signal_index)
    }

    /// To be called after all neurons have been decoded and processed in the caches,
    /// iterates over callbacks over channels to have them update. We do this after all channels
    /// are updated in case users are doing multichannel data stuff, in order to avoid giving them
    /// race condition issues.
    fn try_run_callbacks_on_changed_channels(&mut self) -> Result<(), FeagiDataError> {
        for channel_index in 0..self.pipeline_runners.len() {
            // TODO could this be parallelized?
            if !self.has_channel_been_updated[channel_index] {
                continue;
            }
            let callbacks_len = self.value_updated_callbacks.len();
            let runners_len = self.pipeline_runners.len();
            let signal = self
                .value_updated_callbacks
                .get_mut(channel_index)
                .ok_or_else(|| {
                    FeagiDataError::BadParameters(format!(
                        "Callback signal index {} is out of bounds (callbacks_len={}, runners_len={}). \
                         This indicates an internal cache invariant violation.",
                        channel_index,
                        callbacks_len,
                        runners_len
                    ))
                })?;
            let data_ref = self.pipeline_runners.get(channel_index).unwrap();
            signal.emit(data_ref.get_postprocessed_motor_value()); // no value
        }
        Ok(())
    }

    //endregion

    pub(crate) fn try_read_neuron_data_to_cache_and_do_callbacks(
        &mut self,
        neuron_data: &CorticalMappedXYZPNeuronVoxels,
        time_of_decode: Instant,
    ) -> Result<(), FeagiDataError> {
        self.try_read_neuron_data_to_wrapped_io_data(neuron_data, time_of_decode)?;
        self.try_run_callbacks_on_changed_channels()?;
        self.has_channel_been_updated.fill(false);
        Ok(())
    }

    fn try_read_neuron_data_to_wrapped_io_data(
        &mut self,
        neuron_data: &CorticalMappedXYZPNeuronVoxels,
        time_of_decode: Instant,
    ) -> Result<(), FeagiDataError> {
        self.neuron_decoder
            .read_neuron_data_multi_channel_into_pipeline_input_cache(
                neuron_data,
                time_of_decode,
                &mut self.pipeline_runners,
                &mut self.has_channel_been_updated,
            )?; // Only writes to cache, does not process
        self.pipeline_runners
            .par_iter_mut()
            .zip(&self.has_channel_been_updated)
            .try_for_each(|(pipeline_runner, has_channel_been_updated)| {
                if *has_channel_been_updated {
                    _ = pipeline_runner.process_cached_decoded_motor_value(time_of_decode)?;
                    // Don't do call backs here, we want everything to be done first
                }
                Ok(())
            })?;
        Ok(())
    }

    //region Internal

    #[inline]
    fn try_get_pipeline_runner(
        &self,
        cortical_channel_index: CorticalChannelIndex,
    ) -> Result<&MotorPipelineStageRunner, FeagiDataError> {
        match self.pipeline_runners.get(*cortical_channel_index as usize) {
            Some(pipeline_runner) => Ok(pipeline_runner),
            None => Err(FeagiDataError::BadParameters(format!(
                "Channel Index {} is out of bounds for MotorChannelStreamCaches with {} channels!",
                cortical_channel_index,
                self.pipeline_runners.len()
            ))),
        }
    }

    #[inline]
    fn try_get_pipeline_runner_mut(
        &mut self,
        cortical_channel_index: CorticalChannelIndex,
    ) -> Result<&mut MotorPipelineStageRunner, FeagiDataError> {
        let num_runners = self.pipeline_runners.len();
        match self
            .pipeline_runners
            .get_mut(*cortical_channel_index as usize)
        {
            Some(pipeline_runner) => Ok(pipeline_runner),
            None => Err(FeagiDataError::BadParameters(format!(
                "Channel Index {} is out of bounds for MotorChannelStreamCaches with {} channels!",
                cortical_channel_index, num_runners
            ))),
        }
    }

    fn get_all_device_grouping(&self) -> Vec<JSONDeviceGrouping> {
        let mut output: Vec<JSONDeviceGrouping> = Vec::new();
        for group in self.pipeline_runners.iter() {
            output.push(group.export_as_json_device_grouping())
        };
        output
    }

    //endregion
}
