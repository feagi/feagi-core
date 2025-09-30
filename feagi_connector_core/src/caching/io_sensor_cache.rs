use std::collections::HashMap;
use std::time::Instant;
use feagi_data_serialization::FeagiByteContainer;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::descriptors::{CorticalChannelIndex, CorticalGroupIndex};
use feagi_data_structures::genomic::SensorCorticalType;
use feagi_data_structures::neurons::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZPEncoder};
use feagi_data_structures::wrapped_io_data::WrappedIOData;
use feagi_data_serialization::FeagiSerializable;
use crate::caching::per_channel_stream_caches::{SensoryChannelStreamCaches};
use crate::data_pipeline::{PipelineStageProperties, PipelineStageRunner};

pub(crate) struct IOSensorCache {
    stream_caches: HashMap<(SensorCorticalType, CorticalGroupIndex), SensoryChannelStreamCaches>,
    neuron_data: CorticalMappedXYZPNeuronData,
    byte_data: FeagiByteContainer,
}

impl IOSensorCache {

    pub fn new() -> Self {
        IOSensorCache {
            stream_caches: HashMap::new(),
            neuron_data: CorticalMappedXYZPNeuronData::new(),
            byte_data: FeagiByteContainer::new_empty()

        }
    }

    //region Sensor Interactions

    pub fn register(&mut self, sensor_type: SensorCorticalType, group_index: CorticalGroupIndex,
                           neuron_encoder: Box<dyn NeuronXYZPEncoder>,
                           pipeline_stages_across_channels: Vec<Vec<Box<dyn PipelineStageProperties + Sync + Send>>>)
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

    pub fn try_read_postprocessed_cached_value(&self, sensor_type: SensorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<&WrappedIOData, FeagiDataError> {
        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches(sensor_type, group_index)?;
        let sensor_stream_cache = sensor_stream_caches.try_get_sensory_channel_stream_cache(channel_index)?;
        Ok(sensor_stream_cache.get_most_recent_postprocessed_sensor_value())
    }

    pub fn try_get_pipeline_stage_runner(&self, sensor_type: SensorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<&PipelineStageRunner, FeagiDataError> {
        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches(sensor_type, group_index)?;
        let sensor_stream_cache = sensor_stream_caches.try_get_sensory_channel_stream_cache(channel_index)?;
        Ok(sensor_stream_cache.get_pipeline_runner())
    }

    pub fn try_get_pipeline_stage_runner_mut(&mut self, sensor_type: SensorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<&mut PipelineStageRunner, FeagiDataError> {
        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches_mut(sensor_type, group_index)?;
        let sensor_stream_cache = sensor_stream_caches.try_get_sensory_channel_stream_cache_mut(channel_index)?;
        Ok(sensor_stream_cache.get_pipeline_runner_mut())
    }

    //endregion

    //region Encoding

    pub fn try_encode_updated_sensor_data_to_neurons(&mut self, encode_instant: Instant) -> Result<(), FeagiDataError> {
        self.neuron_data.clear_neurons_only();
        for sensory_channel_steam_cache in self.stream_caches.values() {
            sensory_channel_steam_cache.update_neuron_data_with_recently_updated_cached_sensor_data(&mut self.neuron_data, encode_instant)?;
        }
        Ok(())
    }

    pub fn try_encode_updated_neuron_data_to_feagi_byte_container(&mut self, data_increment_value: u16) -> Result<(), FeagiDataError> {
        let a: Box<dyn FeagiSerializable> = Box::new(self.neuron_data.clone());
        self.byte_data.overwrite_byte_data_with_multiple_struct_data(vec![&a], data_increment_value)
    }

    pub fn get_encoded_bytes(&self) -> &[u8] {
        self.byte_data.get_byte_ref()
    }

    //endregion


    //region Internal
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
    //endregion

}













