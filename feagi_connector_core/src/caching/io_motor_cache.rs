use std::collections::HashMap;
use std::time::Instant;
use feagi_data_serialization::FeagiByteContainer;
use feagi_data_structures::{FeagiDataError, FeagiSignalIndex};
use feagi_data_structures::genomic::descriptors::{AgentDeviceIndex, CorticalChannelIndex, CorticalGroupIndex};
use feagi_data_structures::genomic::MotorCorticalType;
use feagi_data_structures::neuron_voxels::xyzp::{CorticalMappedXYZPNeuronVoxels};
use crate::caching::per_channel_stream_caches::MotorChannelStreamCaches;
use crate::data_pipeline::{PipelineStageProperties, PipelineStagePropertyIndex, PipelineStageRunner};
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPDecoder;
use crate::wrapped_io_data::WrappedIOData;

/// Internal cache for motor outputs, managing data flow from FEAGI.
///
/// Maintains separate stream caches for each registered motor type and group,
/// handles decoding neuron voxels to motor commands, and manages callbacks
/// for motor updates. Supports pipeline stages for postprocessing.
pub(crate) struct IOMotorCache {
    stream_caches: HashMap<(MotorCorticalType, CorticalGroupIndex), MotorChannelStreamCaches>,
    agent_device_key_lookup: HashMap<AgentDeviceIndex, Vec<(MotorCorticalType, CorticalGroupIndex)>>,
    neuron_data: CorticalMappedXYZPNeuronVoxels,
    byte_data: FeagiByteContainer,
}

impl IOMotorCache {

    pub fn new() -> Self {
        IOMotorCache {
            stream_caches: HashMap::new(),
            agent_device_key_lookup: HashMap::new(),
            neuron_data: CorticalMappedXYZPNeuronVoxels::new(),
            byte_data: FeagiByteContainer::new_empty()
        }
    }

    //region Motor Interactions
    pub fn register(&mut self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex,
                    neuron_decoder: Box<dyn NeuronVoxelXYZPDecoder>,
                    pipeline_stages_across_channels: Vec<Vec<Box<dyn PipelineStageProperties + Sync + Send>>>,
                    initial_cached_value: WrappedIOData)
                    -> Result<(), FeagiDataError> {

        // NOTE: The length of pipeline_stages_across_channels denotes the number of channels!

        if self.stream_caches.contains_key(&(motor_type, group_index)) {
            return Err(FeagiDataError::BadParameters(format!("Already registered motor {} of group index {}!", motor_type, group_index)))
        }

        self.stream_caches.insert(
            (motor_type, group_index),
            MotorChannelStreamCaches::new(neuron_decoder, initial_cached_value, pipeline_stages_across_channels)?);

        Ok(())
    }


    pub fn try_read_preprocessed_cached_value(&self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<&WrappedIOData, FeagiDataError> {
        let motor_stream_caches = self.try_get_motor_channel_stream_caches(motor_type, group_index)?;
        Ok(motor_stream_caches.try_get_most_recent_preprocessed_motor_value(channel_index)?)
    }

    pub fn try_read_postprocessed_cached_value(&self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<&WrappedIOData, FeagiDataError> {
        let motor_stream_caches = self.try_get_motor_channel_stream_caches(motor_type, group_index)?;
        Ok(motor_stream_caches.try_get_most_recent_postprocessed_motor_value(channel_index)?)
    }

    pub fn try_register_motor_callback<F>(&mut self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex, callback: F) -> Result<FeagiSignalIndex, FeagiDataError>
    where
        F: Fn(&WrappedIOData) + Send + Sync + 'static,
    {
        let motor_stream_caches = self.try_get_motor_channel_stream_caches_mut(motor_type, group_index)?;
        let index = motor_stream_caches.try_connect_to_data_processed_signal(channel_index, callback)?;
        Ok(index)
    }

    //endregion

    //region Pipeline Stages

    pub fn try_get_single_stage_properties(&self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex, stage_index: PipelineStagePropertyIndex) -> Result<Box<dyn PipelineStageProperties + Sync + Send>, FeagiDataError> {
        let motor_stream_caches = self.try_get_motor_channel_stream_caches(motor_type, group_index)?;
        motor_stream_caches.try_get_single_stage_properties(channel_index, stage_index)
    }

    pub fn get_all_stage_properties(&self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<Vec<Box<dyn PipelineStageProperties + Sync + Send>>, FeagiDataError> {
        let motor_stream_caches = self.try_get_motor_channel_stream_caches(motor_type, group_index)?;
        motor_stream_caches.get_all_stage_properties(channel_index)
    }

    pub fn try_update_single_stage_properties(&mut self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex,
                                              channel_index: CorticalChannelIndex, pipeline_stage_property_index: PipelineStagePropertyIndex,
                                              replacing_property: Box<dyn PipelineStageProperties + Sync + Send>)
                                              -> Result<(), FeagiDataError> {

        let motor_stream_caches = self.try_get_motor_channel_stream_caches_mut(motor_type, group_index)?;
        motor_stream_caches.try_update_single_stage_properties(channel_index, pipeline_stage_property_index, replacing_property)
    }

    pub fn try_update_all_stage_properties(&mut self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex, new_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<(), FeagiDataError> {
        let motor_stream_caches = self.try_get_motor_channel_stream_caches_mut(motor_type, group_index)?;
        motor_stream_caches.try_update_all_stage_properties(channel_index, new_pipeline_stage_properties)
    }

    pub fn try_replace_single_stage(&mut self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex, replacing_at_index: PipelineStagePropertyIndex, new_pipeline_stage_properties: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<(), FeagiDataError> {
        let motor_stream_caches = self.try_get_motor_channel_stream_caches_mut(motor_type, group_index)?;
        motor_stream_caches.try_replace_single_stage(channel_index, replacing_at_index, new_pipeline_stage_properties)
    }

    pub fn try_replace_all_stages(&mut self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex, new_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<(), FeagiDataError> {
        let motor_stream_caches = self.try_get_motor_channel_stream_caches_mut(motor_type, group_index)?;
        motor_stream_caches.try_replace_all_stages(channel_index, new_pipeline_stage_properties)
    }

    //endregion

    //region AgentDeviceKey

    pub fn register_agent_device_key(&mut self, agent_device_index: AgentDeviceIndex, motor_type: MotorCorticalType, group_index: CorticalGroupIndex) -> Result<(), FeagiDataError> {
        let keys = {
            match self.agent_device_key_lookup.get_mut(&agent_device_index) {
                Some(keys) => keys,
                None => {
                    self.agent_device_key_lookup.insert(agent_device_index, Vec::new());
                    self.agent_device_key_lookup.get_mut(&agent_device_index).unwrap()
                }
            }
        };
        keys.push((motor_type, group_index));
        Ok(())
    }

    pub fn try_read_preprocessed_cached_values_by_agent_device(&self, agent_device_index: AgentDeviceIndex, channel_index: CorticalChannelIndex) -> Result<Vec<&WrappedIOData>, FeagiDataError> {
        let motor_group_pairs = self.try_get_agent_device_lookup(agent_device_index)?;
        let mut results = Vec::with_capacity(motor_group_pairs.len());
        for (motor_type, group_index) in motor_group_pairs {
            let value = self.try_read_preprocessed_cached_value(*motor_type, *group_index, channel_index)?;
            results.push(value);
        }
        Ok(results)
    }

    pub fn try_read_postprocessed_cached_values_by_agent_device(&self, agent_device_index: AgentDeviceIndex, channel_index: CorticalChannelIndex) -> Result<Vec<&WrappedIOData>, FeagiDataError> {
        let motor_group_pairs = self.try_get_agent_device_lookup(agent_device_index)?;
        let mut results = Vec::with_capacity(motor_group_pairs.len());
        for (motor_type, group_index) in motor_group_pairs {
            let value = self.try_read_postprocessed_cached_value(*motor_type, *group_index, channel_index)?;
            results.push(value);
        }
        Ok(results)
    }

    //endregion

    //region Decoding

    pub fn get_feagi_byte_container(&self) -> &FeagiByteContainer {
        &self.byte_data
    }

    pub fn replace_feagi_byte_container(&mut self, feagi_byte_container: FeagiByteContainer) {
        self.byte_data = feagi_byte_container
    }

    pub fn try_import_bytes<F>(&mut self, byte_writing_function: &mut F) -> Result<(), FeagiDataError>
    where F: FnMut(&mut Vec<u8>) -> Result<(), FeagiDataError> {
        self.byte_data.try_write_data_to_container_and_verify(byte_writing_function)?;
        Ok(())
    }

    // Returns true if data was retrieved
    pub fn try_decode_bytes_to_neural_data(&mut self) -> Result<bool, FeagiDataError> {
        self.byte_data.try_update_struct_from_first_found_struct_of_type(&mut self.neuron_data)
    }

    pub fn try_decode_neural_data_into_cache(&mut self, time_of_decode: Instant) -> Result<(), FeagiDataError> {
        for motor_channel_stream_cache in self.stream_caches.values_mut() {
            motor_channel_stream_cache.try_read_neuron_data_to_cache_and_do_callbacks(&mut self.neuron_data, time_of_decode)?;
        };
        Ok(())
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

    fn try_get_agent_device_lookup(&self, agent_device_index: AgentDeviceIndex) -> Result<&[(MotorCorticalType, CorticalGroupIndex)], FeagiDataError> {
        let val = self.agent_device_key_lookup.get(&agent_device_index).ok_or(
            FeagiDataError::BadParameters(format!("No registered motor device found in agent's list for agent index {}!", *agent_device_index))
        )?;
        Ok(val)
    }

    fn try_get_agent_device_lookup_mut(&mut self, agent_device_index: AgentDeviceIndex) -> Result<&mut Vec<(MotorCorticalType, CorticalGroupIndex)>, FeagiDataError> {
        let val = self.agent_device_key_lookup.get_mut(&agent_device_index).ok_or(
            FeagiDataError::BadParameters(format!("No registered motor device found in agent's list for agent index {}!", *agent_device_index))
        )?;
        Ok(val)
    }

    //endregion

}