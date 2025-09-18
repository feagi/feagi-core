use std::collections::HashMap;
use std::time::Instant;
use feagi_data_serialization::FeagiByteStructure;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::descriptors::{AgentDeviceIndex, CorticalChannelIndex, CorticalGroupIndex};
use feagi_data_structures::genomic::SensorCorticalType;
use feagi_data_structures::neurons::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZPDecoder, NeuronXYZPEncoder};
use feagi_data_structures::wrapped_io_data::{WrappedIOData, WrappedIOType};
use crate::caching::hashmap_helpers::{AccessAgentLookupKey, CorticalAreaMetadataKey, FullChannelCacheKey};
use crate::caching::motor_channel_stream_cache::MotorChannelStreamCache;
use crate::caching::sensory_channel_stream_cache::SensoryChannelStreamCache;
use crate::data_pipeline::{PipelineStage, PipelineStageIndex};

pub struct IOCache {

    // Sensor stuff

    sensor_channel_caches: HashMap<FullChannelCacheKey, SensoryChannelStreamCache>, // (cortical type, grouping index, channel) -> sensory data cache, the main lookup
    sensor_cortical_area_metadata: HashMap<CorticalAreaMetadataKey, SensoryCorticalAreaCacheDetails>, // (cortical type, grouping index) -> (Vec<FullChannelCacheKey>, number_channels, neuron_encoder), defines all channel caches for a cortical area, and its neuron encoder
    sensor_agent_key_proxy: HashMap<AccessAgentLookupKey, Vec<FullChannelCacheKey>>, // (CorticalType, AgentDeviceIndex) -> Vec<FullChannelCacheKey>, allows users to map any channel of a cortical type to an agent device ID
    sensor_neuron_data: CorticalMappedXYZPNeuronData, // cached sensor neuron data
    sensor_byte_data: FeagiByteStructure, // cached byte data for sensor

    // Motor Stuff
    
    motor_channel_caches: HashMap<FullChannelCacheKey, MotorChannelStreamCache>, // (cortical type, grouping index, channel) -> motor data cache, the main lookup
    motor_cortical_area_metadata: HashMap<CorticalAreaMetadataKey, MotorCorticalAreaCacheDetails>, // (cortical type, grouping index) -> (Vec<FullChannelCacheKey>, number_channels, neuron_decoder), defines all channel caches for a cortical area, and its neuron decoder
    motor_agent_key_proxy: HashMap<AccessAgentLookupKey, Vec<FullChannelCacheKey>>, // (CorticalType, AgentDeviceIndex) -> Vec<FullChannelCacheKey>, allows users to map any channel of a cortical type to an agent device ID
    motor_neuron_data: CorticalMappedXYZPNeuronData, // cached motor neuron data
    motor_byte_data: FeagiByteStructure, // cached byte data for motor

}

impl IOCache {

    // TODO how to handle Deregistering with agent existing?
    pub fn new() -> IOCache {
        IOCache {
            sensor_channel_caches: HashMap::new(),
            sensor_cortical_area_metadata: HashMap::new(),
            sensor_agent_key_proxy: HashMap::new(),
            sensor_neuron_data: CorticalMappedXYZPNeuronData::new(),
            sensor_byte_data: FeagiByteStructure::new(),
            motor_channel_caches: HashMap::new(),
            motor_cortical_area_metadata: HashMap::new(),
            motor_agent_key_proxy: HashMap::new(),
            motor_neuron_data: CorticalMappedXYZPNeuronData::new(),
            motor_byte_data: FeagiByteStructure::new(),
        }
    }

    //region Sensor Cache

    //region Sensor Interfaces

    //region Common

    //endregion

    //region Unique

    //endregion

    //endregion


    //endregion

    //region Internal Functions

    //region Agent Functions


    fn sensor_register_agent_device_index(&mut self, cortical_sensor_type: SensorCorticalType,
                                   group: CorticalGroupIndex, channel: CorticalChannelIndex, agent_device_index: AgentDeviceIndex) -> Result<(), FeagiDataError> {

        _ = self.sensor_try_get_sensory_channel_stream_cache(cortical_sensor_type, group, channel)?; // Check to ensure target exists

        let cortical_type = cortical_sensor_type.into();
        let full_channel_key: FullChannelCacheKey = FullChannelCacheKey::new(cortical_type, group, channel);
        let try_key_vector = self.sensor_agent_key_proxy.get_mut(&AccessAgentLookupKey::new(cortical_type, agent_device_index));

        match try_key_vector {
            Some(key_vector) => {
                if !key_vector.contains(&full_channel_key){
                    key_vector.push(full_channel_key)
                }
            }
            None => {
                let new_vector: Vec<FullChannelCacheKey> = vec![full_channel_key];
                _ = self.sensor_agent_key_proxy.insert(AccessAgentLookupKey::new(cortical_type, agent_device_index), new_vector);
            }
        }
        Ok(())
    }

    fn sensor_store_value_from_device_index(&mut self, cortical_sensor_type: SensorCorticalType, agent_device_index: AgentDeviceIndex, value: WrappedIOData)-> Result<(), FeagiDataError> {
        // NOTE: Assuming the value is of the correct type

        let agent_key = &AccessAgentLookupKey::new(cortical_sensor_type.into(), agent_device_index);

        if let Some(vec) = self.sensor_agent_key_proxy.get_mut(&agent_key) {
            for cache_key in vec.iter_mut().skip(1) {
                let cache = self.sensor_channel_caches.get_mut(cache_key).unwrap();
                cache.update_sensor_value(value.clone())?;
            }
            let cache = self.sensor_channel_caches.get_mut(vec.get_mut(0).unwrap()).unwrap();
            cache.update_sensor_value(value)?;
        }
        Ok(())

    }




    //endregion

    fn sensor_register_cortical_area_and_channels(&mut self, sensor_cortical_type: SensorCorticalType, cortical_group: CorticalGroupIndex,
                                           neuron_encoder: Box<dyn NeuronXYZPEncoder + Sync + Send>,
                                           mut initial_processor_chains: Vec<Vec<Box<dyn PipelineStage + Sync + Send>>>) -> Result<(), FeagiDataError> {
        // NOTE: initial_processor_chains is a vector of vectors, meaning each channel gets a vector of processing

        let number_supported_channels = initial_processor_chains.len() as u32;
        let cortical_type = sensor_cortical_type.into();
        let cortical_metadata = CorticalAreaMetadataKey::new(cortical_type, cortical_group);


        if number_supported_channels == 0 {
            return Err(FeagiDataError::BadParameters("A cortical area cannot be registered with 0 channels!".into()).into())
        }
        if self.sensor_cortical_area_metadata.contains_key(&cortical_metadata) {
            return Err(FeagiDataError::InternalError("Cortical area already registered!".into()).into())
        }



        let mut cache_keys: Vec<FullChannelCacheKey> = Vec::with_capacity(number_supported_channels as usize);
        for i in 0..number_supported_channels {

            let channel: CorticalChannelIndex = i.into();
            let sensor_key: FullChannelCacheKey = FullChannelCacheKey::new(cortical_type, cortical_group, channel);
            let sensor_cache: SensoryChannelStreamCache = SensoryChannelStreamCache::new(
                initial_processor_chains.pop().unwrap(),
                channel,
            )?;

            _ = self.sensor_channel_caches.insert(sensor_key.clone(), sensor_cache);
            cache_keys.push(sensor_key);
        }


        let cortical_cache_details = SensoryCorticalAreaCacheDetails::new(cache_keys, number_supported_channels, neuron_encoder);
        _ = self.sensor_cortical_area_metadata.insert(cortical_metadata, cortical_cache_details);

        Ok(())
    }

    fn sensor_update_value_by_channel(&mut self, value: WrappedIOData, cortical_sensor_type: SensorCorticalType, cortical_grouping_index: CorticalGroupIndex, device_channel: CorticalChannelIndex) -> Result<(), FeagiDataError> {
        let mut channel_cache = self.sensor_try_get_mut_sensory_channel_stream_cache(cortical_sensor_type, cortical_grouping_index, device_channel)?;
        if channel_cache.get_input_data_type() != WrappedIOType::from(&value) {
            return Err(FeagiDataError::BadParameters(format!("Got value type {:?} when expected type {:?} for Cortical Type {:?}, Group Index {:?}, Channel {:?}!", WrappedIOType::from(&value),
                                                             channel_cache.get_input_data_type(), cortical_sensor_type, cortical_grouping_index, device_channel)).into());
        }
        _ = channel_cache.update_sensor_value(value);
        Ok(())
    }

    fn sensor_read_value_by_channel(&self, cortical_sensor_type: SensorCorticalType, cortical_grouping_index: CorticalGroupIndex, device_channel: CorticalChannelIndex) -> Result<WrappedIOData, FeagiDataError> {
        let channel_cache = self.sensor_try_get_sensory_channel_stream_cache(cortical_sensor_type, cortical_grouping_index, device_channel)?;
        let value = channel_cache.get_most_recent_sensor_value();
        Ok(value.clone())
    }

    fn sensor_set_pipeline_stages_for_channel(&mut self, cortical_sensor_type: SensorCorticalType,
                                       cortical_grouping_index: CorticalGroupIndex, device_channel: CorticalChannelIndex,
                                       new_stages: Vec<Box<dyn PipelineStage + Sync + Send>>) -> Result<(), FeagiDataError> {
        let channel_cache = self.sensor_try_get_mut_sensory_channel_stream_cache(cortical_sensor_type, cortical_grouping_index, device_channel)?;
        channel_cache.attempt_replace_pipeline_stages(new_stages)?;
        Ok(())
    }

    fn sensor_set_pipeline_stage_for_channel(&mut self, cortical_sensor_type: SensorCorticalType,
                                      cortical_grouping_index: CorticalGroupIndex, device_channel: CorticalChannelIndex,
                                      overwriting_stage: Box<dyn PipelineStage + Sync + Send>,
                                      overwriting_index: PipelineStageIndex) -> Result<(), FeagiDataError> {
        let cache = self.sensor_try_get_mut_sensory_channel_stream_cache(cortical_sensor_type, cortical_grouping_index, device_channel)?;
        cache.attempt_replace_pipeline_stage(overwriting_stage, overwriting_index)?;
        Ok(())
    }

    fn sensor_clone_pipeline_stages_for_channel(&mut self, cortical_sensor_type: SensorCorticalType,
                                         cortical_grouping_index: CorticalGroupIndex,
                                         device_channel: CorticalChannelIndex) -> Result<(Vec<Box<dyn PipelineStage + Sync + Send>>), FeagiDataError> {
        let cache = self.sensor_try_get_mut_sensory_channel_stream_cache(cortical_sensor_type, cortical_grouping_index, device_channel)?;
        Ok(cache.clone_pipeline_stages())
    }

    fn sensor_clone_pipeline_stage_for_channel(&mut self, cortical_sensor_type: SensorCorticalType,
                                        cortical_grouping_index: CorticalGroupIndex, device_channel: CorticalChannelIndex,
                                        reading_index: PipelineStageIndex) -> Result<(Box<dyn PipelineStage + Sync + Send>), FeagiDataError> {
        let cache = self.sensor_try_get_mut_sensory_channel_stream_cache(cortical_sensor_type, cortical_grouping_index, device_channel)?;
        cache.clone_pipeline_stage(reading_index)
    }

    fn sensor_try_get_sensory_channel_stream_cache(&self, cortical_sensor_type: SensorCorticalType, cortical_grouping_index: CorticalGroupIndex,
                                            device_channel: CorticalChannelIndex) -> Result<(&SensoryChannelStreamCache), FeagiDataError> {
        let cortical_type = cortical_sensor_type.into();
        let channel_cache = match self.sensor_channel_caches.get(&FullChannelCacheKey::new(cortical_type, cortical_grouping_index, device_channel)) {
            Some(channel_stream_cache) => channel_stream_cache,
            None => return Err(FeagiDataError::BadParameters(format!("Unable to find Cortical Type {:?}, Group Index {:?}, Channel {:?}!", cortical_type, cortical_grouping_index, device_channel)).into())
        };
        Ok(channel_cache)
    }

    fn sensor_try_get_mut_sensory_channel_stream_cache(&mut self, cortical_sensor_type: SensorCorticalType, cortical_grouping_index: CorticalGroupIndex, device_channel: CorticalChannelIndex) -> Result<(&mut SensoryChannelStreamCache), FeagiDataError> {
        let cortical_type = cortical_sensor_type.into();
        let channel_cache = match self.sensor_channel_caches.get_mut(&FullChannelCacheKey::new(cortical_type, cortical_grouping_index, device_channel)) {
            Some(channel_stream_cache) => channel_stream_cache,
            None => return Err(FeagiDataError::BadParameters(format!("Unable to find Cortical Type {:?}, Group Index {:?}, Channel {:?}!", cortical_type, cortical_grouping_index, device_channel)).into())
        };
        Ok(channel_cache)
    }


    fn sensor_encode_to_neurons(&mut self, past_send_time: Instant) -> Result<(), FeagiDataError> {
        // TODO move to using iter(), I'm using for loops now cause im still a rust scrub
        for cortical_area_details in self.sensor_cortical_area_metadata.values() {
            let channel_cache_keys = &cortical_area_details.relevant_channel_lookups;
            let neuron_encoder = &cortical_area_details.neuron_encoder;
            for channel_cache_key in channel_cache_keys {
                let sensor_cache = self.sensor_channel_caches.get(channel_cache_key).unwrap();
                sensor_cache.encode_to_neurons(&mut self.sensor_neuron_data, neuron_encoder)?
            }
        }
        Ok(())
    }


    //endregion

}


//region Cortical Area Details

struct SensoryCorticalAreaCacheDetails {
    relevant_channel_lookups: Vec<FullChannelCacheKey>,
    number_channels: u32,
    neuron_encoder: Box<dyn NeuronXYZPEncoder + Sync + Send>
}

impl SensoryCorticalAreaCacheDetails {
    pub fn new(relevant_channel_lookups: Vec<FullChannelCacheKey>, number_channels: u32, neuron_encoder: Box<dyn NeuronXYZPEncoder + Sync + Send>) -> Self {
        SensoryCorticalAreaCacheDetails {
            relevant_channel_lookups,
            number_channels,
            neuron_encoder
        }

    }
}

struct MotorCorticalAreaCacheDetails {
    relevant_channel_lookups: Vec<FullChannelCacheKey>,
    number_channels: u32,
    neuron_decoder: Box<dyn NeuronXYZPDecoder + Sync + Send>
}

impl MotorCorticalAreaCacheDetails {
    pub fn new(relevant_channel_lookups: Vec<FullChannelCacheKey>, number_channels: u32, neuron_decoder: Box<dyn NeuronXYZPDecoder + Sync + Send>) -> Self {
        MotorCorticalAreaCacheDetails {
            relevant_channel_lookups,
            number_channels,
            neuron_decoder
        }

    }
}

//endregion