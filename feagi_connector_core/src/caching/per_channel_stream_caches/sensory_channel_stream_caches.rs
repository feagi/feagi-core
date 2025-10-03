use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::descriptors::{CorticalChannelCount, CorticalChannelIndex};
use feagi_data_structures::neurons::xyzp::{CorticalMappedXYZPNeuronData};
use crate::data_pipeline::{PipelineStageProperties, PipelineStageRunner};
use crate::neuron_coding::xyzp::NeuronXYZPEncoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

pub(crate) struct SensoryChannelStreamCaches {
    neuron_encoder: Box<dyn NeuronXYZPEncoder>,
    pipeline_runners: Vec<PipelineStageRunner>,
    last_update_time: Instant,
}

impl SensoryChannelStreamCaches {

    pub fn new(neuron_encoder: Box<dyn NeuronXYZPEncoder>, stage_properties_per_channels: Vec<Vec<Box<dyn PipelineStageProperties + Sync + Send>>>) -> Result<Self, FeagiDataError> {
        if stage_properties_per_channels.is_empty() { // Yes checks exist below, but this error has more context
            return Err(FeagiDataError::InternalError("SensoryChannelStreamCaches Cannot be initialized with 0 channels!".into()))
        }

        let num_channels = stage_properties_per_channels.len();
        let mut pipeline_runners: Vec<PipelineStageRunner> = Vec::with_capacity(num_channels);
        
        for stage_properties_per_channel in stage_properties_per_channels {
            pipeline_runners.push(PipelineStageRunner::new(stage_properties_per_channel)?);
        }

        Ok(Self {
            neuron_encoder,
            pipeline_runners,
            last_update_time: Instant::now(),
        })
    }

    //region Properties

    pub fn number_of_channels(&self) -> CorticalChannelCount {
        (self.pipeline_runners.len() as u32).into()
    }

    //endregion

    //region Pipeline Runner Data

    pub fn try_get_channel_input_type(&self, cortical_channel_index: CorticalChannelIndex) -> Result<WrappedIOType, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_input_data_type())
    }

    pub fn try_get_channel_recent_postprocessed_value(&self, cortical_channel_index: CorticalChannelIndex) -> Result<&WrappedIOData, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_most_recent_output())
    }

    pub fn try_get_channel_update_instant(&self, cortical_channel_index: CorticalChannelIndex) -> Result<Instant, FeagiDataError> {
        let pipeline_runner = self.try_get_pipeline_runner(cortical_channel_index)?;
        Ok(pipeline_runner.get_last_processed_instant())
    }

    pub fn try_update_channel_value(&mut self, cortical_channel_index: CorticalChannelIndex, value: &WrappedIOData, update_instant: Instant) -> Result<(), FeagiDataError> {
        let runner = self.try_get_pipeline_runner_mut(cortical_channel_index)?;
        runner.try_update_value(value, update_instant)?;
        self.last_update_time = update_instant;
        Ok(())
    }
    
    //endregion

    pub fn update_neuron_data_with_recently_updated_cached_sensor_data(&mut self, neuron_data: &mut CorticalMappedXYZPNeuronData, time_of_burst: Instant) -> Result<(), FeagiDataError> {
        // TODO We need a new trait method to just have all channels cleared if not up to date
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

