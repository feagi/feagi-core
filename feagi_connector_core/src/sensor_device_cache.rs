use std::collections::HashMap;
use std::time::Instant;
use feagi_data_serialization::FeagiByteContainer;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::descriptors::{AgentDeviceIndex, CorticalChannelIndex, CorticalGroupIndex};
use feagi_data_structures::genomic::SensorCorticalType;
use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use crate::caching::per_channel_stream_caches::SensoryChannelStreamCaches;
use crate::data_pipeline::{PipelineStageProperties, PipelineStagePropertyIndex};
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPEncoder;
use crate::wrapped_io_data::WrappedIOData;

pub(crate) struct SensorDeviceCache {
    stream_caches: HashMap<(SensorCorticalType, CorticalGroupIndex), SensoryChannelStreamCaches>,
    agent_device_key_lookup: HashMap<AgentDeviceIndex, Vec<(SensorCorticalType, CorticalGroupIndex)>>,
    neuron_data: CorticalMappedXYZPNeuronVoxels,
    byte_data: FeagiByteContainer,
    previous_burst: Instant,
}

impl SensorDeviceCache {

    pub fn new() -> Self {
        SensorDeviceCache {
            stream_caches: HashMap::new(),
            agent_device_key_lookup: HashMap::new(),
            neuron_data: CorticalMappedXYZPNeuronVoxels::new(),
            byte_data: FeagiByteContainer::new_empty(),
            previous_burst: Instant::now(),
        }
    }



    //region Internal

    //region Cache Abstractions

    fn register(&mut self, sensor_type: SensorCorticalType, group_index: CorticalGroupIndex,
                neuron_encoder: Box<dyn NeuronVoxelXYZPEncoder>,
                pipeline_stages_across_channels: Vec<Vec<Box<dyn PipelineStageProperties + Sync + Send>>>,
                initial_cached_value: WrappedIOData)
                -> Result<(), FeagiDataError> {

        // NOTE: The length of pipeline_stages_across_channels denotes the number of channels!

        if self.stream_caches.contains_key(&(sensor_type, group_index)) {
            return Err(FeagiDataError::BadParameters(format!("Already registered sensor {} of group index {}!", sensor_type, group_index)))
        }

        self.stream_caches.insert(
            (sensor_type, group_index),
            SensoryChannelStreamCaches::new(neuron_encoder, initial_cached_value, pipeline_stages_across_channels)?);

        Ok(())
    }

    //region Data

    fn try_update_value(&mut self, sensor_type: SensorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex, value: WrappedIOData, time_of_update: Instant) -> Result<(), FeagiDataError> {
        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches_mut(sensor_type, group_index)?;
        sensor_stream_caches.try_replace_input_channel_cache_value_and_run_pipeline(channel_index, value, time_of_update)?; // Handles checking channel, value type
        Ok(())
    }

    fn try_read_preprocessed_cached_value(&self, sensor_type: SensorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<&WrappedIOData, FeagiDataError> {
        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches(sensor_type, group_index)?;
        let value = sensor_stream_caches.try_get_channel_recent_preprocessed_value(channel_index)?;
        Ok(value)
    }

    fn try_read_postprocessed_cached_value(&self, sensor_type: SensorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<&WrappedIOData, FeagiDataError> {
        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches(sensor_type, group_index)?;
        let value = sensor_stream_caches.try_get_channel_recent_postprocessed_value(channel_index)?;
        Ok(value)
    }

    //endregion

    //region Stages

    fn try_get_single_stage_properties(&self, sensor_type: SensorCorticalType,
                                       group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex,
                                       pipeline_stage_property_index: PipelineStagePropertyIndex) -> Result<Box<dyn PipelineStageProperties+Send+Sync>, FeagiDataError> {
        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches(sensor_type, group_index)?;
        sensor_stream_caches.try_get_single_stage_properties(channel_index, pipeline_stage_property_index)
    }

    fn try_get_all_stage_properties(&self, sensor_type: SensorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<Vec<Box<dyn PipelineStageProperties + Sync + Send>>, FeagiDataError> {
        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches(sensor_type, group_index)?;
        sensor_stream_caches.get_all_stage_properties(channel_index)
    }

    fn try_update_single_stage_properties(&mut self, sensor_type: SensorCorticalType, group_index: CorticalGroupIndex,
                                          channel_index: CorticalChannelIndex, pipeline_stage_property_index: PipelineStagePropertyIndex,
                                          replacing_property: Box<dyn PipelineStageProperties + Sync + Send>)
                                          -> Result<(), FeagiDataError> {

        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches_mut(sensor_type, group_index)?;
        sensor_stream_caches.try_update_single_stage_properties(channel_index, pipeline_stage_property_index, replacing_property)
    }

    fn try_update_all_stage_properties(&mut self, sensor_type: SensorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex, new_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<(), FeagiDataError> {
        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches_mut(sensor_type, group_index)?;
        sensor_stream_caches.try_update_all_stage_properties(channel_index, new_pipeline_stage_properties)
    }

    fn try_replace_single_stage(&mut self, sensor_type: SensorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex, replacing_at_index: PipelineStagePropertyIndex, new_pipeline_stage_properties: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<(), FeagiDataError> {
        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches_mut(sensor_type, group_index)?;
        sensor_stream_caches.try_replace_single_stage(channel_index, replacing_at_index, new_pipeline_stage_properties)
    }

    fn try_replace_all_stages(&mut self, sensor_type: SensorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex, new_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<(), FeagiDataError> {
        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches_mut(sensor_type, group_index)?;
        sensor_stream_caches.try_replace_all_stages(channel_index, new_pipeline_stage_properties)
    }

    //endregion

    //region Agent Device

    fn register_agent_device_key(&mut self, agent_device_index: AgentDeviceIndex, sensor_type: SensorCorticalType, group_index: CorticalGroupIndex) -> Result<(), FeagiDataError> {
        let keys = {
            match self.agent_device_key_lookup.get_mut(&agent_device_index) {
                Some(keys) => keys,
                None => {
                    self.agent_device_key_lookup.insert(agent_device_index, Vec::new());
                    self.agent_device_key_lookup.get_mut(&agent_device_index).unwrap()
                }
            }
        };
        keys.push((sensor_type, group_index));
        Ok(())
    }

    fn try_update_value_by_agent_device(&mut self, agent_device_index: AgentDeviceIndex, channel_index: CorticalChannelIndex, value: WrappedIOData, time_of_update: Instant) -> Result<(), FeagiDataError> {
        let sensor_group_pairs = self.try_get_agent_device_lookup(agent_device_index)?.to_vec();
        for (sensor_type, group_index) in sensor_group_pairs {
            self.try_update_value(sensor_type, group_index, channel_index, value.clone(), time_of_update)?;
        }
        Ok(())
    }

    fn try_read_postprocessed_cached_values_by_agent_device(&self, agent_device_index: AgentDeviceIndex, channel_index: CorticalChannelIndex) -> Result<Vec<&WrappedIOData>, FeagiDataError> {
        let sensor_group_pairs = self.try_get_agent_device_lookup(agent_device_index)?;
        let mut results = Vec::with_capacity(sensor_group_pairs.len());
        for (sensor_type, group_index) in sensor_group_pairs {
            let value = self.try_read_postprocessed_cached_value(*sensor_type, *group_index, channel_index)?;
            results.push(value);
        }
        Ok(results)
    }

    //endregion

    //endregion

    //region Hashmap Interactions

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

    fn try_get_agent_device_lookup(&self, agent_device_index: AgentDeviceIndex) -> Result<&[(SensorCorticalType, CorticalGroupIndex)], FeagiDataError> {
        let val = self.agent_device_key_lookup.get(&agent_device_index).ok_or(
            FeagiDataError::BadParameters(format!("No registered sensor device found in agent's list for agent index {}!", *agent_device_index))
        )?;
        Ok(val)
    }

    fn try_get_agent_device_lookup_mut(&mut self, agent_device_index: AgentDeviceIndex) -> Result<&mut Vec<(SensorCorticalType, CorticalGroupIndex)>, FeagiDataError> {
        let val = self.agent_device_key_lookup.get_mut(&agent_device_index).ok_or(
            FeagiDataError::BadParameters(format!("No registered sensor device found in agent's list for agent index {}!", *agent_device_index))
        )?;
        Ok(val)
    }


    //endregion


    //endregion




}