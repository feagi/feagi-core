use std::collections::HashMap;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::descriptors::{CorticalChannelIndex, CorticalGroupIndex};
use feagi_data_structures::genomic::MotorCorticalType;
use feagi_data_structures::neurons::xyzp::NeuronXYZPDecoder;
use feagi_data_structures::wrapped_io_data::WrappedIOData;
use crate::caching::per_channel_stream_caches::MotorChannelStreamCaches;
use crate::data_pipeline::{PipelineStageProperties, PipelineStageRunner};

pub(crate) struct IOMotorCache {
    stream_caches: HashMap<(MotorCorticalType, CorticalGroupIndex), MotorChannelStreamCaches>
}

impl IOMotorCache {

    pub fn new() -> Self {
        IOMotorCache {
            stream_caches: HashMap::new(),
        }
    }

    //region Interactions
    pub fn register(&mut self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex,
                    neuron_decoder: Box<dyn NeuronXYZPDecoder>,
                    pipeline_stages_across_channels: Vec<Vec<Box<dyn PipelineStageProperties + Sync + Send>>>)
                    -> Result<(), FeagiDataError> {

        // NOTE: The length of pipeline_stages_across_channels denotes the number of channels!

        if self.stream_caches.contains_key(&(motor_type, group_index)) {
            return Err(FeagiDataError::BadParameters(format!("Already registered motor {} of group index {}!", motor_type, group_index)))
        }

        self.stream_caches.insert(
            (motor_type, group_index),
            MotorChannelStreamCaches::new(neuron_decoder, pipeline_stages_across_channels)?);

        Ok(())
    }


    pub fn try_read_postprocessed_cached_value(&self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<&WrappedIOData, FeagiDataError> {
        let motor_stream_caches = self.try_get_motor_channel_stream_caches(motor_type, group_index)?;
        let motor_stream_cache = motor_stream_caches.try_get_motor_channel_stream_cache(channel_index)?;
        Ok(motor_stream_cache.get_most_recent_postprocessed_motor_value())
    }

    pub fn try_read_preprocessed_cached_value(&self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<&WrappedIOData, FeagiDataError> {
        let motor_stream_caches = self.try_get_motor_channel_stream_caches(motor_type, group_index)?;
        let motor_stream_cache = motor_stream_caches.try_get_motor_channel_stream_cache(channel_index)?;
        Ok(motor_stream_cache.get_most_recent_preprocessed_motor_value())
    }

    pub fn try_get_pipeline_stage_runner(&self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<&PipelineStageRunner, FeagiDataError> {
        let motor_stream_caches = self.try_get_motor_channel_stream_caches(motor_type, group_index)?;
        let motor_stream_cache = motor_stream_caches.try_get_motor_channel_stream_cache(channel_index)?;
        Ok(motor_stream_cache.get_pipeline_runner())
    }

    pub fn try_get_pipeline_stage_runner_mut(&mut self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<&mut PipelineStageRunner, FeagiDataError> {
        let motor_stream_caches = self.try_get_motor_channel_stream_caches_mut(motor_type, group_index)?;
        let motor_stream_cache = motor_stream_caches.try_get_motor_channel_stream_cache_mut(channel_index)?;
        Ok(motor_stream_cache.get_pipeline_runner_mut())
    }
    
    //endregion


    //region Internal
    fn try_get_motor_channel_stream_caches(&self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex) -> Result<&MotorChannelStreamCaches, FeagiDataError> {
        let check = self.stream_caches.get(&(motor_type, group_index));
        if check.is_none() {
            return Err(FeagiDataError::BadParameters(format!("Unable to find {} of cortical group index {} in registered motor's list!", motor_type, group_index)))
        }
        let check = check.unwrap();
        Ok(check)
    }

    fn try_get_motor_channel_stream_caches_mut(&mut self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex) -> Result<&mut MotorChannelStreamCaches, FeagiDataError> {
        let check = self.stream_caches.get_mut(&(motor_type, group_index));
        if check.is_none() {
            return Err(FeagiDataError::BadParameters(format!("Unable to find {} of cortical group index {} in registered motor's list!", motor_type, group_index)))
        }
        let check = check.unwrap();
        Ok(check)
    }
    //endregion

}