use std::collections::HashMap;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::descriptors::{CorticalChannelIndex, CorticalGroupIndex};
use feagi_data_structures::genomic::SensorCorticalType;
use feagi_data_structures::neurons::xyzp::NeuronXYZPEncoder;
use feagi_data_structures::wrapped_io_data::WrappedIOData;
use crate::caching::per_channel_stream_caches::{SensoryChannelStreamCache, SensoryChannelStreamCaches};
use crate::data_pipeline::PipelineStageProperties;

pub(crate) struct IOSensorCache {
    stream_caches: HashMap<(SensorCorticalType, CorticalGroupIndex), SensoryChannelStreamCaches>,
}

impl IOSensorCache {

    pub fn register(&mut self, sensor_type: SensorCorticalType, group_index: CorticalGroupIndex,
                           neuron_encoder: Box<dyn NeuronXYZPEncoder>,
                           pipeline_stages_across_channels: Vec<Vec<Box<dyn PipelineStageProperties>>>)
        -> Result<(), FeagiDataError> {

        // NOTE: The length of pipeline_stages_across_channels denotes the number of channels!

        if self.stream_caches.contains_key(&(sensor_type, group_index)) {
            return Err(FeagiDataError::BadParameters(format!("Already registered sensor {} of group index {}!", sensor_type, group_index)))
        }

        self.stream_caches.insert(
            (sensor_type, group_index),
            SensoryChannelStreamCaches::new(neuron_encoder, pipeline_stages_across_channels)?);

        Ok(())
    }

    pub fn try_update_value(&mut self, sensor_type: SensorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex, value: WrappedIOData) -> Result<(), FeagiDataError> {
        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches_mut(sensor_type, group_index)?;
        let sensor_stream_cache = sensor_stream_caches.try_get_sensory_channel_stream_cache_mut(channel_index)?; // Handles checking index validity
        sensor_stream_cache.try_update_sensor_value(value)?; // Handles checking value type
        Ok(())
    }

    pub fn try_read_previously_cached_value_pre_preprocessing(&self, sensor_type: SensorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<&WrappedIOData, FeagiDataError> {
        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches(sensor_type, group_index)?;
        let sensor_stream_cache = sensor_stream_caches.try_get_sensory_channel_stream_cache(channel_index)?; // Handles checking index validity
    }






    fn try_get_sensory_channel_stream_caches(&self, sensor_type: SensorCorticalType, group_index: CorticalGroupIndex) -> Result<&SensoryChannelStreamCaches, FeagiDataError> {
        let check = self.stream_caches.get(&(sensor_type, group_index));
        if check.is_none() {
            return Err(FeagiDataError::BadParameters(format!("Unable to find {} of cortical group index {} in registered sensor's list!", sensor_type, group_index)))
        }
        let check = check.unwrap();
        Ok(check)
    }

    fn try_get_sensory_channel_stream_caches_mut(&mut self, sensor_type: SensorCorticalType, group_index: CorticalGroupIndex) -> Result<&mut SensoryChannelStreamCaches, FeagiDataError> {
        let check = self.stream_caches.get_mut(&(sensor_type, group_index));
        if check.is_none() {
            return Err(FeagiDataError::BadParameters(format!("Unable to find {} of cortical group index {} in registered sensor's list!", sensor_type, group_index)))
        }
        let check = check.unwrap();
        Ok(check)
    }

}













