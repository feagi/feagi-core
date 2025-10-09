use std::collections::HashMap;
use std::time::Instant;
use feagi_data_serialization::FeagiByteContainer;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::descriptors::{CorticalChannelIndex, CorticalGroupIndex};
use feagi_data_structures::genomic::SensorCorticalType;
use feagi_data_structures::neuron_voxels::xyzp::{CorticalMappedXYZPNeuronVoxels};
use crate::caching::per_channel_stream_caches::{SensoryChannelStreamCaches};
use crate::data_pipeline::{PipelineStageProperties, PipelineStagePropertyIndex, PipelineStageRunner};
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPEncoder;
use crate::wrapped_io_data::WrappedIOData;

pub(crate) struct IOSensorCache {
    stream_caches: HashMap<(SensorCorticalType, CorticalGroupIndex), SensoryChannelStreamCaches>,
    neuron_data: CorticalMappedXYZPNeuronVoxels,
    byte_data: FeagiByteContainer,
    previous_burst: Instant,
}

impl IOSensorCache {

    pub fn new() -> Self {
        IOSensorCache {
            stream_caches: HashMap::new(),
            neuron_data: CorticalMappedXYZPNeuronVoxels::new(),
            byte_data: FeagiByteContainer::new_empty(),
            previous_burst: Instant::now(),
        }
    }

    //region Sensor Interactions

    pub fn register(&mut self, sensor_type: SensorCorticalType, group_index: CorticalGroupIndex,
                           neuron_encoder: Box<dyn NeuronVoxelXYZPEncoder>,
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

    pub fn try_update_value(&mut self, sensor_type: SensorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex, value: &WrappedIOData, time_of_update: Instant) -> Result<(), FeagiDataError> {
        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches_mut(sensor_type, group_index)?;
        sensor_stream_caches.try_update_channel_value(channel_index, value, time_of_update)?; // Handles checking channel, value type
        Ok(())
    }

    pub fn try_read_postprocessed_cached_value(&self, sensor_type: SensorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<&WrappedIOData, FeagiDataError> {
        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches(sensor_type, group_index)?;
        let value = sensor_stream_caches.try_get_channel_recent_postprocessed_value(channel_index)?;
        Ok(value)
    }
    
    pub(crate) fn get_pipeline_stage_properties(&self, sensor_type: SensorCorticalType, 
                                                group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex, 
                                                pipeline_stage_property_index: PipelineStagePropertyIndex) -> Result<Box<dyn PipelineStageProperties+Send+Sync>, FeagiDataError> {
        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches(sensor_type, group_index)?;
        sensor_stream_caches.try_get_stage_properties(channel_index, pipeline_stage_property_index)
    }

    pub fn try_updating_pipeline_stage(&mut self, sensor_type: SensorCorticalType, group_index: CorticalGroupIndex,
                                       channel_index: CorticalChannelIndex, pipeline_stage_property_index: PipelineStagePropertyIndex,
                                       replacing_property: Box<dyn PipelineStageProperties + Sync + Send>)
                                       -> Result<(), FeagiDataError> {

        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches_mut(sensor_type, group_index)?;
        sensor_stream_caches.try_update_pipeline_stage(channel_index, pipeline_stage_property_index, replacing_property)?;
        Ok(())
    }

    /*
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

     */

    //endregion      

    //region Encoding

    pub fn get_feagi_byte_container(&self) -> &FeagiByteContainer {
        &self.byte_data
    }

    pub fn replace_feagi_byte_container(&mut self, feagi_byte_container: FeagiByteContainer) {
        self.byte_data = feagi_byte_container
    }

    pub fn try_encode_updated_sensor_data_to_neurons(&mut self, encode_instant: Instant) -> Result<(), FeagiDataError> {
        self.neuron_data.clear_neurons_only(); // TODO remove extra occurrences of clearing
        // TODO this can likely be done in parallel. Explore this!
        for sensory_channel_steam_caches in self.stream_caches.values_mut() {
            sensory_channel_steam_caches.update_neuron_data_with_recently_updated_cached_sensor_data(&mut self.neuron_data, self.previous_burst)?;
        }
        self.previous_burst = encode_instant;
        Ok(())
    }

    pub fn try_encode_updated_neuron_data_to_feagi_byte_container(&mut self, data_increment_value: u16) -> Result<(), FeagiDataError> {
        self.byte_data.overwrite_byte_data_with_single_struct_data(&self.neuron_data, data_increment_value)
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













