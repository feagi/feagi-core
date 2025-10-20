use std::time::Instant;
use rayon::prelude::*;
use feagi_data_structures::{FeagiDataError, FeagiSignal, FeagiSignalIndex};
use feagi_data_structures::genomic::descriptors::{CorticalChannelCount, CorticalChannelIndex};
use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use crate::data_pipeline::{PipelineStageProperties, PipelineStagePropertyIndex, PipelineStageRunner};
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPDecoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

#[derive(Debug)]
pub(crate) struct MotorChannelStreamCaches {
    neuron_decoder: Box<dyn NeuronVoxelXYZPDecoder>,
    pipeline_runners: Vec<PipelineStageRunner>,
    has_channel_been_updated: Vec<bool>,
    value_updated_callbacks: Vec<FeagiSignal<WrappedIOData>>,
}

impl MotorChannelStreamCaches {
    pub fn new(neuron_decoder: Box<dyn NeuronVoxelXYZPDecoder>, initial_cached_value: WrappedIOData, stage_properties_per_channels: Vec<Vec<Box<dyn PipelineStageProperties + Sync + Send>>>) -> Result<Self, FeagiDataError> {

        if stage_properties_per_channels.is_empty() {
            return Err(FeagiDataError::BadParameters("Cannot create a motor stream cache with 0 channels!".into()))
        }

        let expected_data_decoded_type: WrappedIOType = neuron_decoder.get_decoded_data_type();


        let num_channels = stage_properties_per_channels.len();
        let mut pipeline_runners: Vec<PipelineStageRunner> = Vec::with_capacity(num_channels);
        let mut callbacks: Vec<FeagiSignal<()>> = Vec::with_capacity(num_channels);

        for stage_properties_per_channel in stage_properties_per_channels {
            let pipeline_runner = PipelineStageRunner::new(stage_properties_per_channel, initial_cached_value.clone(), expected_data_decoded_type)?;
            callbacks.push(FeagiSignal::new());
            
            pipeline_runners.push(pipeline_runner);
        }

        Ok(Self {
            neuron_decoder,
            pipeline_runners,
            has_channel_been_updated: vec![false; num_channels],
            value_updated_callbacks: callbacks,
        })
    }

    //region Properties

    pub fn number_of_channels(&self) -> CorticalChannelCount {
        (self.pipeline_runners.len() as u32).try_into().unwrap()
    }

    pub fn verify_channel_exists(&self, channel_index: CorticalChannelIndex) -> Result<(), FeagiDataError> {
        _ =  self.try_get_pipeline_runner(channel_index)?;
        Ok(())
    }

    pub fn get_output_type(&self) -> WrappedIOType {
        self.pipeline_runners.first().unwrap().get_output_data_type()
    }

    //endregion

    //region Pipeline Runner Data

    pub fn try_get_most_recent_preprocessed_motor_value(&self, cortical_channel_index: CorticalChannelIndex) -> Result<&WrappedIOData, FeagiDataError> {
        let pipeline_runner= self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_most_recent_preprocessed_output())
    }

    pub fn try_get_most_recent_postprocessed_motor_value(&self, cortical_channel_index: CorticalChannelIndex) -> Result<&WrappedIOData, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_most_recent_postprocessed_output())
    }

    pub fn try_get_channel_last_processed_instant(&self, cortical_channel_index: CorticalChannelIndex) -> Result<Instant, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_last_processed_instant())
    }


    //endregion

    //region Pipeline Stages

    pub fn try_get_single_stage_properties(&self, cortical_channel_index: CorticalChannelIndex, stage_index: PipelineStagePropertyIndex) -> Result<Box<dyn PipelineStageProperties + Sync + Send>, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        pipeline_runner.try_get_single_stage_properties(stage_index)
    }
    
    pub fn get_all_stage_properties(&self, cortical_channel_index: CorticalChannelIndex) -> Result<Vec<Box<dyn PipelineStageProperties + Sync + Send>>, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_all_stage_properties())
    }

    pub fn try_update_single_stage_properties(&mut self, cortical_channel_index: CorticalChannelIndex, pipeline_stage_property_index: PipelineStagePropertyIndex, replacing_property: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<(), FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner_mut(cortical_channel_index)?;
        pipeline_runner.try_update_single_stage_properties(pipeline_stage_property_index, replacing_property)
    }
    
    pub fn try_update_all_stage_properties(&mut self, cortical_channel_index: CorticalChannelIndex, new_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<(), FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner_mut(cortical_channel_index)?;
        pipeline_runner.try_update_all_stage_properties(new_pipeline_stage_properties)
    }

    pub fn try_replace_single_stage(&mut self, cortical_channel_index: CorticalChannelIndex, replacing_at_index: PipelineStagePropertyIndex, new_pipeline_stage_properties: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<(), FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner_mut(cortical_channel_index)?;
        pipeline_runner.try_replace_single_stage(replacing_at_index, new_pipeline_stage_properties)
    }
    
    pub fn try_replace_all_stages(&mut self, cortical_channel_index: CorticalChannelIndex, new_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<(), FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner_mut(cortical_channel_index)?;
        pipeline_runner.try_replace_all_stages(new_pipeline_stage_properties)
    }

    //endregion

    //region Callbacks

    pub fn try_connect_to_data_processed_signal<F>(&mut self, cortical_channel_index: CorticalChannelIndex, callback: F) -> Result<FeagiSignalIndex, FeagiDataError>
    where
        F: Fn(&WrappedIOData) + Send + Sync + 'static,
    {
        _ = self.try_get_pipeline_runner(cortical_channel_index)?;
        let idx = *cortical_channel_index as usize;
        Ok(self.value_updated_callbacks[idx].connect(callback))
    }

    pub fn try_disconnect_to_data_processed_signal(&mut self, cortical_channel_index: CorticalChannelIndex, signal_index: FeagiSignalIndex) -> Result<(), FeagiDataError> {
        _ = self.try_get_pipeline_runner(cortical_channel_index)?;
        let idx = *cortical_channel_index as usize;
        self.value_updated_callbacks[idx].disconnect(signal_index)
    }

    /// To be called after all neurons have been decoded and processed in the caches,
    /// iterates over callbacks over channels to have them update. We do this after all channels
    /// are updated in case users are doing multichannel data stuff, in order to avoid giving them
    /// race condition issues.
    pub(crate) fn try_run_callbacks_on_changed_channels(&mut self) -> Result<(), FeagiDataError> {
        for channel_index in 0..self.pipeline_runners.len() { // TODO could this be parallelized?
            if !self.has_channel_been_updated[channel_index] {
                continue;
            }
            let data_ref = self.pipeline_runners.get(channel_index).unwrap();
            self.value_updated_callbacks[channel_index].emit(data_ref.get_most_recent_postprocessed_output()); // no value
        }
        self.has_channel_been_updated.fill(false);
        Ok(())
    }

    //endregion

    pub(crate) fn try_read_neuron_data_to_wrapped_io_data(&mut self, neuron_data: &CorticalMappedXYZPNeuronVoxels, time_of_decode: Instant) -> Result<(), FeagiDataError> {

        self.neuron_decoder.read_neuron_data_multi_channel(neuron_data, time_of_decode, &mut self.pipeline_runners, &mut self.has_channel_been_updated)?; // NOTE: This will ONLY write the updated
        self.most_recent_directly_decoded_outputs.par_iter()
            .zip(&self.has_channel_been_updated)
            .zip(&mut self.pipeline_runners)
            .try_for_each(|((output, &has_changed), pipeline_runner)| {
                if has_changed {
                    pipeline_runner.try_update_value(output, time_of_decode)?;
                }
                Ok(())
            })?;
        Ok(())
    }

    //region Internal

    #[inline]
    fn try_get_pipeline_runner(&self, cortical_channel_index: CorticalChannelIndex) -> Result<&PipelineStageRunner, FeagiDataError> {
        match self.pipeline_runners.get(*cortical_channel_index as usize) {
            Some(pipeline_runner) => Ok(pipeline_runner),
            None => Err(FeagiDataError::BadParameters(format!("Channel Index {} is out of bounds for MotorChannelStreamCaches with {} channels!",
                                                              cortical_channel_index, self.pipeline_runners.len())))
        }

    }

    #[inline]
    fn try_get_pipeline_runner_mut(&mut self, cortical_channel_index: CorticalChannelIndex) -> Result<&mut PipelineStageRunner, FeagiDataError> {
        let num_runners = self.pipeline_runners.len();
        match self.pipeline_runners.get_mut(*cortical_channel_index as usize) {
            Some(pipeline_runner) => Ok(pipeline_runner),
            None => Err(FeagiDataError::BadParameters(format!("Channel Index {} is out of bounds for MotorChannelStreamCaches with {} channels!",
                                                              cortical_channel_index, num_runners)))
        }

    }

    //endregion

}