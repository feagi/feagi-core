use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::descriptors::{CorticalChannelCount, CorticalChannelIndex};
use feagi_data_structures::neuron_voxels::xyzp::{CorticalMappedXYZPNeuronVoxels};
use crate::data_pipeline::{PipelineStageProperties, PipelineStagePropertyIndex, PipelineStageRunner};
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPEncoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

pub(crate) struct SensoryChannelStreamCaches {
    neuron_encoder: Box<dyn NeuronVoxelXYZPEncoder>,
    pipeline_runners: Vec<PipelineStageRunner>,
    last_update_time: Instant,
}

impl SensoryChannelStreamCaches {

    pub fn new(neuron_encoder: Box<dyn NeuronVoxelXYZPEncoder>, initial_cached_value: WrappedIOData, stage_properties_per_channels: Vec<Vec<Box<dyn PipelineStageProperties + Sync + Send>>>) -> Result<Self, FeagiDataError> {

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

    pub fn number_of_channels(&self) -> CorticalChannelCount {
        (self.pipeline_runners.len() as u32).try_into().unwrap()
    }

    pub fn verify_channel_exists(&self, channel_index: CorticalChannelIndex) -> Result<(), FeagiDataError> {
        _ =  self.try_get_pipeline_runner(channel_index)?;
        Ok(())
    }

    //endregion

    //region Pipeline Runner Data

    pub fn try_get_channel_input_type(&self, cortical_channel_index: CorticalChannelIndex) -> Result<WrappedIOType, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_input_data_type())
    }

    pub fn try_get_channel_recent_preprocessed_value(&self, cortical_channel_index: CorticalChannelIndex) -> Result<&WrappedIOData, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_most_recent_preprocessed_output())
    }

    pub fn try_get_channel_recent_postprocessed_value(&self, cortical_channel_index: CorticalChannelIndex) -> Result<&WrappedIOData, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_most_recent_postprocessed_output())
    }

    pub fn try_update_channel_value(&mut self, cortical_channel_index: CorticalChannelIndex, value: WrappedIOData, update_instant: Instant) -> Result<(), FeagiDataError> {
        let runner = self.try_get_pipeline_runner_mut(cortical_channel_index)?;
        runner.try_update_value(value, update_instant)?;
        self.last_update_time = update_instant;
        Ok(())
    }

    pub fn try_get_channel_update_instant(&self, cortical_channel_index: CorticalChannelIndex) -> Result<Instant, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_last_processed_instant())
    }

    //endregion

    //region Pipeline Runner Stages

    pub fn try_get_single_stage_properties(&self, cortical_channel_index: CorticalChannelIndex, pipeline_stage_property_index: PipelineStagePropertyIndex) -> Result<Box<dyn PipelineStageProperties + Sync + Send>, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        pipeline_runner.try_get_single_stage_properties(pipeline_stage_property_index)
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

    pub fn try_removing_all_stages(&mut self, cortical_channel_index: CorticalChannelIndex) -> Result<(), FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner_mut(cortical_channel_index)?;
        pipeline_runner.try_removing_all_stages()?;
        Ok(())
    }

    //endregion

    pub(crate) fn update_neuron_data_with_recently_updated_cached_sensor_data(&mut self, neuron_data: &mut CorticalMappedXYZPNeuronVoxels, time_of_burst: Instant) -> Result<(), FeagiDataError> {
        // TODO We need a new trait method to just have all channels cleared if not up to date
        // Note: We expect neuron data to be cleared before this step
        self.neuron_encoder.write_neuron_data_multi_channel(&self.pipeline_runners, time_of_burst, neuron_data)?;
        Ok(())
    }

    //region Internal

    #[inline]
    fn try_get_pipeline_runner(&self, cortical_channel_index: CorticalChannelIndex) -> Result<&PipelineStageRunner, FeagiDataError> {
        match self.pipeline_runners.get(*cortical_channel_index as usize) {
            Some(pipeline_runner) => Ok(pipeline_runner),
            None => Err(FeagiDataError::BadParameters(format!("Channel Index {} is out of bounds for SensoryChannelStreamCaches with {} channels!",
                                                              cortical_channel_index, self.pipeline_runners.len())))
        }

    }

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

