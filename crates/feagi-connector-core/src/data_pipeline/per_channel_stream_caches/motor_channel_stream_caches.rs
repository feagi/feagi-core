use crate::data_pipeline::per_channel_stream_caches::pipeline_stage_runner_common::PipelineStageRunner;
use crate::data_pipeline::per_channel_stream_caches::MotorPipelineStageRunner;
use crate::data_pipeline::{PipelineStageProperties, PipelineStagePropertyIndex};
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPDecoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};
use feagi_data_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelIndex,
};
use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use feagi_data_structures::{FeagiDataError, FeagiSignal, FeagiSignalIndex};
use rayon::prelude::*;
use std::time::Instant;

#[derive(Debug)]
pub(crate) struct MotorChannelStreamCaches {
    neuron_decoder: Box<dyn NeuronVoxelXYZPDecoder>,
    pipeline_runners: Vec<MotorPipelineStageRunner>,
    has_channel_been_updated: Vec<bool>,
    value_updated_callbacks: Vec<FeagiSignal<WrappedIOData>>,
    device_friendly_name: String,
}

impl MotorChannelStreamCaches {
    pub fn new(
        neuron_decoder: Box<dyn NeuronVoxelXYZPDecoder>,
        number_channels: CorticalChannelCount,
        initial_cached_value: WrappedIOData,
    ) -> Result<Self, FeagiDataError> {
        let pipeline_runners: Vec<MotorPipelineStageRunner> = std::iter::repeat_with(|| {
            MotorPipelineStageRunner::new(initial_cached_value.clone()).unwrap()
        })
        .take(*number_channels as usize)
        .collect();

        let callbacks: Vec<FeagiSignal<WrappedIOData>> =
            Vec::with_capacity(*number_channels as usize);

        Ok(Self {
            neuron_decoder,
            pipeline_runners,
            has_channel_been_updated: vec![false; *number_channels as usize],
            value_updated_callbacks: callbacks,
            device_friendly_name: String::new(),
        })
    }

    //region Properties

    pub fn number_of_channels(&self) -> CorticalChannelCount {
        (self.pipeline_runners.len() as u32).try_into().unwrap()
    }

    pub fn verify_channel_exists(
        &self,
        channel_index: CorticalChannelIndex,
    ) -> Result<(), FeagiDataError> {
        _ = self.try_get_pipeline_runner(channel_index)?;
        Ok(())
    }

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

    pub fn try_get_channel_last_processed_instant(
        &self,
        cortical_channel_index: CorticalChannelIndex,
    ) -> Result<Instant, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_last_processed_instant())
    }

    //endregion

    //region Pipeline Stages

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

    pub(crate) fn export_as_json(&self) -> Result<serde_json::Value, FeagiDataError> {
        let mut output = serde_json::Map::new();
        output.insert(
            "friendly_name".to_string(),
            serde_json::Value::String(self.device_friendly_name.clone()),
        );

        let mut channels_data: Vec<serde_json::Value> = Vec::new();
        for pipeline_stage_runner in &self.pipeline_runners {
            let channel_data = pipeline_stage_runner.export_as_json()?;
            channels_data.push(channel_data);
        }
        output.insert(
            "channels".to_string(),
            serde_json::Value::Array(channels_data),
        );
        Ok(output.into())
    }

    pub(crate) fn import_from_json(
        &mut self,
        json: &serde_json::Value,
    ) -> Result<(), FeagiDataError> {
        let json_map = json.as_object().ok_or_else(|| {
            FeagiDataError::DeserializationError(
                "Expected JSON object for MotorChannelStreamCaches".to_string(),
            )
        })?;

        // Get friendly name
        if let Some(friendly_name) = json_map.get("friendly_name") {
            self.device_friendly_name = friendly_name.as_str().unwrap_or("").to_string();
        }

        // Get channels array
        let channels = json_map
            .get("channels")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                FeagiDataError::DeserializationError("Expected 'channels' array".to_string())
            })?;

        self.pipeline_runners.clear();

        // Import each channel
        for (runner, channel_json) in self.pipeline_runners.iter_mut().zip(channels.iter()) {
            let channel_map = channel_json.as_object().ok_or_else(|| {
                FeagiDataError::DeserializationError(
                    "Expected channel to be JSON object".to_string(),
                )
            })?;
            runner.import_from_json(channel_map)?;
        }

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
        Ok(self.value_updated_callbacks[idx].connect(callback))
    }

    pub fn try_disconnect_to_data_processed_signal(
        &mut self,
        cortical_channel_index: CorticalChannelIndex,
        signal_index: FeagiSignalIndex,
    ) -> Result<(), FeagiDataError> {
        _ = self.try_get_pipeline_runner(cortical_channel_index)?;
        let idx = *cortical_channel_index as usize;
        self.value_updated_callbacks[idx].disconnect(signal_index)
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
            let data_ref = self.pipeline_runners.get(channel_index).unwrap();
            self.value_updated_callbacks[channel_index]
                .emit(data_ref.get_postprocessed_motor_value()); // no value
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

    //endregion
}
