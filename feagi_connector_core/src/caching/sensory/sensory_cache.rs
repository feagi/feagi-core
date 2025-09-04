use std::collections::HashMap;
use std::ops::Range;
use std::time::Instant;
use feagi_data_structures::data::image_descriptors::{GazeProperties, ImageFrameProperties, SegmentedImageFrameProperties};
use feagi_data_structures::data::{ImageFrame, SegmentedImageFrame};
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::descriptors::{AgentDeviceIndex, CorticalChannelCount, CorticalChannelIndex, CorticalGroupIndex, NeuronDepth};
use feagi_data_structures::genomic::{CorticalID, SensorCorticalType};
use feagi_data_structures::neurons::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZPEncoder};
use feagi_data_structures::neurons::xyzp::encoders::*;
use feagi_data_structures::processing::{ImageFrameSegmentator, ImageFrameProcessor};
use feagi_data_structures::wrapped_io_data::{WrappedIOData, WrappedIOType};
use feagi_data_structures::sensor_definition;
use feagi_data_serialization::{FeagiByteStructureCompatible, FeagiByteStructure};
use crate::caching::hashmap_helpers::{AccessAgentLookupKey, CorticalAreaMetadataKey, FullChannelCacheKey};
use crate::caching::sensory::sensory_channel_stream_cache::SensoryChannelStreamCache;
use crate::data_pipeline::stages::{ImageFrameProcessorStage, ImageFrameSegmentatorStage, LinearScaleTo0And1Stage, IdentityImageFrameStage};
use crate::data_pipeline::{PipelineStageIndex, PipelineStage};


pub struct SensorCache {
    channel_caches: HashMap<FullChannelCacheKey, SensoryChannelStreamCache>, // (cortical type, grouping index, channel) -> sensory data cache, the main lookup
    cortical_area_metadata: HashMap<CorticalAreaMetadataKey, CorticalAreaCacheDetails>, // (cortical type, grouping index) -> (Vec<FullChannelCacheKey>, number_channels, neuron_encoder), defines all channel caches for a cortical area, and its neuron encoder
    agent_key_proxy: HashMap<AccessAgentLookupKey, Vec<FullChannelCacheKey>>, // (CorticalType, AgentDeviceIndex) -> Vec<FullChannelCacheKey>, allows users to map any channel of a cortical type to an agent device ID
    neuron_data: CorticalMappedXYZPNeuronData, // cached neuron data
    byte_data: FeagiByteStructure
}

impl SensorCache {
    pub fn new() -> SensorCache {
        
        let neuron_data = CorticalMappedXYZPNeuronData::new();
        let byte_data = neuron_data.as_new_feagi_byte_structure().unwrap();
        
        
        SensorCache {
            channel_caches: HashMap::new(),
            cortical_area_metadata: HashMap::new(),
            agent_key_proxy: HashMap::new(),
            neuron_data,
            byte_data
        }

    }
    
    //region Sensor Functions
    
    //region Generic types

    //region F32Normalized0To1_Linear

    pub fn register_f32_normalized_0_to_1_linear(&mut self, sensor_cortical_type: SensorCorticalType,
                                                 group: CorticalGroupIndex, number_channels: CorticalChannelCount,
                                                 allow_stale_data: bool, neuron_depth: NeuronDepth,
                                                 bounds: Range<f32>) -> Result<(), FeagiDataError> {
        match sensor_cortical_type {
            SensorCorticalType::Proximity => {
                // Do Nothing TODO macro
            }
            _ => return Err(FeagiDataError::BadParameters(format!("Expected Sensor type with data type {:?}!", "F32NormalizedTo1Linear")))
        };

        let cortical_id = sensor_cortical_type.to_cortical_id(group);
        let encoder =  Box::new(F32LinearNeuronXYZPEncoder::new(cortical_id, neuron_depth)?);
        let mut processors: Vec<Vec<Box<dyn PipelineStage + Sync + Send>>> = Vec::with_capacity(*number_channels as usize);
        for _i in 0..*number_channels {
            processors.push(vec![Box::new(LinearScaleTo0And1Stage::new(bounds.start, bounds.end, 0.0)?)]);
        };
        self.register_cortical_area_and_channels(sensor_cortical_type, group, encoder, processors, allow_stale_data)?;
        Ok(())
    }

    pub fn store_f32_normalized_0_to_1_linear(&mut self, sensor_cortical_type: SensorCorticalType,
                                              group: CorticalGroupIndex, channel: CorticalChannelIndex,
                                              new_float: f32) -> Result<(), FeagiDataError> {
        let val = WrappedIOData::F32(new_float);
        self.update_value_by_channel(val, sensor_cortical_type, group, channel);
        Ok(())
    }

    pub fn read_cached_f32_normalized_0_to_1_linear(&mut self, sensor_cortical_type: SensorCorticalType,
                                                    group: CorticalGroupIndex, channel: CorticalChannelIndex)
        -> Result<f32, FeagiDataError> {
        let val = self.read_value_by_channel(sensor_cortical_type, group, channel)?;
        Ok(val.try_into()?)
    }

    pub fn set_pipeline_stages_f32_normalized_0_to_1_linear(&mut self, sensor_cortical_type: SensorCorticalType, group: CorticalGroupIndex, channel: CorticalChannelIndex, new_stages: Vec<Box<dyn PipelineStage + Sync + Send>>) -> Result<(), FeagiDataError> {
        self.set_pipeline_stages_for_channel(sensor_cortical_type, group, channel, new_stages)?;
        Ok(())
    }
    pub fn set_pipeline_stage_f32_normalized_0_to_1_linear(&mut self, sensor_cortical_type: SensorCorticalType, group: CorticalGroupIndex, channel: CorticalChannelIndex, new_stage: Box<dyn PipelineStage + Sync + Send>, stage_index: PipelineStageIndex) -> Result<(), FeagiDataError> {
        self.set_pipeline_stage_for_channel(sensor_cortical_type, group, channel, new_stage, stage_index)
    }
    
    pub fn clone_pipeline_stages_f32_normalized_0_to_1_linear(&mut self, sensor_cortical_type: SensorCorticalType, group: CorticalGroupIndex, channel: CorticalChannelIndex) -> Result<Vec<Box<dyn PipelineStage + Sync + Send>>, FeagiDataError> {
        self.clone_pipeline_stages_for_channel(sensor_cortical_type, group, channel)
    }

    pub fn clone_pipeline_stage_f32_normalized_0_to_1_linear(&mut self, sensor_cortical_type: SensorCorticalType, group: CorticalGroupIndex, channel: CorticalChannelIndex, stage_index: PipelineStageIndex) -> Result<Box<dyn PipelineStage + Sync + Send>, FeagiDataError> {
        self.clone_pipeline_stage_for_channel(sensor_cortical_type, group, channel, stage_index)
    }

    pub fn register_device_agent_index_f32_normalized_0_to_1_linear(&mut self, sensor_cortical_type: SensorCorticalType, group: CorticalGroupIndex, channel: CorticalChannelIndex, agent_device_index: AgentDeviceIndex) -> Result<(), FeagiDataError> {

        self.register_agent_device_index(sensor_cortical_type, group, channel, agent_device_index)
    }

    pub fn store_device_agent_index_f32_normalized_0_to_1_linear(&mut self, sensor_cortical_type: SensorCorticalType, agent_device_index: AgentDeviceIndex, value: f32) -> Result<(), FeagiDataError> {

        self.store_value_from_device_index(sensor_cortical_type, agent_device_index, WrappedIOData::F32(value))
    }




    // TODO how to handle Deregistering with agent existing?


    //endregion



    //endregion




    
    // Manual Functions
    //region Segmented Image Camera Manual Functions

    pub fn register_image_camera_with_peripheral(&mut self, cortical_group: CorticalGroupIndex,
                                                                    number_of_channels: CorticalChannelCount, allow_stale_data: bool,
                                                                    input_image_properties: ImageFrameProperties,
                                                                    output_image_properties: SegmentedImageFrameProperties,
                                                                    segmentation_center_properties: GazeProperties) -> Result<(), FeagiDataError> {
        let sensor_cortical_type = SensorCorticalType::ImageCameraCenter;

        let cortical_ids = SegmentedImageFrame::create_ordered_cortical_ids_for_segmented_vision(cortical_group);
        for cortical_id in &cortical_ids {
            let cortical_type = cortical_id.get_cortical_type();
            let cortical_metadata = CorticalAreaMetadataKey::new(cortical_type, cortical_group);
            if self.cortical_area_metadata.contains_key(&cortical_metadata) {
                return Err(FeagiDataError::InternalError("Cortical area already registered!".into()).into())
            }
        }; // ensure no cortical ID is used already

        let segmentator = ImageFrameSegmentator::new(input_image_properties, output_image_properties, segmentation_center_properties)?;
        let neuron_encoder = Box::new(SegmentedImageFrameNeuronXYZPEncoder::new(cortical_ids, output_image_properties)?);
        let mut processors: Vec<Vec<Box<dyn PipelineStage + Sync + Send>>> = Vec::with_capacity(*number_of_channels as usize);
        for _i in 0..*number_of_channels {
            processors.push(vec![Box::new(ImageFrameSegmentatorStage::new(input_image_properties, output_image_properties, segmentator.clone()))]);
        };

        self.register_cortical_area_and_channels(sensor_cortical_type, cortical_group, neuron_encoder, processors, allow_stale_data)?;
        Ok(())
    }


    pub fn store_segmented_image_camera(&mut self, new_value: ImageFrame, cortical_grouping_index: CorticalGroupIndex, device_channel: CorticalChannelIndex) -> Result<(), FeagiDataError> {
        let val = WrappedIOData::ImageFrame(new_value);
        let sensor_type = SensorCorticalType::ImageCameraCenter;
        self.update_value_by_channel(val, sensor_type, cortical_grouping_index, device_channel)
    }


    pub fn read_segmented_image_camera(&mut self,cortical_group: CorticalGroupIndex, device_channel: CorticalChannelIndex) -> Result<SegmentedImageFrame, FeagiDataError> {
        let val = self.read_value_by_channel(SensorCorticalType::ImageCameraCenter, cortical_group, device_channel)?;
        Ok(val.try_into()?)
    }

    pub fn set_stages_segmented_image_camera(&mut self, cortical_group: CorticalGroupIndex, device_channel: CorticalChannelIndex, new_stages: Vec<Box<dyn PipelineStage + Sync + Send>>) -> Result<(), FeagiDataError> {
        let sensor_type = SensorCorticalType::ImageCameraCenter;
        self.set_pipeline_stages_for_channel(sensor_type, cortical_group, device_channel, new_stages)
    }
    
    
    //endregion
    
    //endregion



    //region Agent Functions
    

    fn register_agent_device_index(&mut self, cortical_sensor_type: SensorCorticalType,
                                   group: CorticalGroupIndex, channel: CorticalChannelIndex, agent_device_index: AgentDeviceIndex) -> Result<(), FeagiDataError> {

        _ = self.try_get_sensory_channel_stream_cache(cortical_sensor_type, group, channel)?; // Check to ensure target exists

        let cortical_type = cortical_sensor_type.into();
        let full_channel_key: FullChannelCacheKey = FullChannelCacheKey::new(cortical_type, group, channel);
        let try_key_vector = self.agent_key_proxy.get_mut(&AccessAgentLookupKey::new(cortical_type, agent_device_index));

        match try_key_vector {
            Some(key_vector) => {
                if !key_vector.contains(&full_channel_key){
                    key_vector.push(full_channel_key)
                }
            }
            None => {
                let new_vector: Vec<FullChannelCacheKey> = vec![full_channel_key];
                _ = self.agent_key_proxy.insert(AccessAgentLookupKey::new(cortical_type, agent_device_index), new_vector);
            }
        }
        Ok(())
    }

    fn store_value_from_device_index(&mut self, cortical_sensor_type: SensorCorticalType, agent_device_index: AgentDeviceIndex, value: WrappedIOData)-> Result<(), FeagiDataError> {
        // NOTE: Assuming the value is of the correct type

        let agent_key = &AccessAgentLookupKey::new(cortical_sensor_type.into(), agent_device_index);

        if let Some(vec) = self.agent_key_proxy.get_mut(&agent_key) {
            for cache_key in vec.iter_mut().skip(1) {
                let cache = self.channel_caches.get_mut(cache_key).unwrap();
                cache.update_sensor_value(value.clone())?;
            }
            let cache = self.channel_caches.get_mut(vec.get_mut(0).unwrap()).unwrap();
            cache.update_sensor_value(value)?;
        }
        Ok(())

    }

    

    
    //endregion
    
    //region Data Encoding
    
    pub fn encode_cached_data_into_bytes(&mut self, time_send_started: Instant) -> Result<(), FeagiDataError> {
        self.encode_to_neurons(time_send_started)?;
        // TODO for now we will recreate the FBS every time
        self.byte_data = self.neuron_data.as_new_feagi_byte_structure().unwrap();
        Ok(())
    }
    
    pub fn retrieve_latest_bytes(&self) -> Result<&[u8], FeagiDataError> {
        Ok(self.byte_data.borrow_data_as_slice())
    }
    
    //endregion
    
    
    //region Internal Functions
    
    fn register_cortical_area_and_channels(&mut self, sensor_cortical_type: SensorCorticalType, cortical_group: CorticalGroupIndex,
                                           neuron_encoder: Box<dyn NeuronXYZPEncoder + Sync + Send>,
                                           mut initial_processor_chains: Vec<Vec<Box<dyn PipelineStage + Sync + Send>>>,
                                           allow_stale_data: bool) -> Result<(), FeagiDataError> {
        // NOTE: initial_processor_chains is a vector of vectors, meaning each channel gets a vector of processing
        
        let number_supported_channels = initial_processor_chains.len() as u32;
        let cortical_type = sensor_cortical_type.into();
        let cortical_metadata = CorticalAreaMetadataKey::new(cortical_type, cortical_group);
        
        
        if number_supported_channels == 0 {
            return Err(FeagiDataError::BadParameters("A cortical area cannot be registered with 0 channels!".into()).into())
        }
        if self.cortical_area_metadata.contains_key(&cortical_metadata) {
            return Err(FeagiDataError::InternalError("Cortical area already registered!".into()).into())
        }

        
        
        let mut cache_keys: Vec<FullChannelCacheKey> = Vec::with_capacity(number_supported_channels as usize);
        for i in 0..number_supported_channels {
            
            let channel: CorticalChannelIndex = i.into();
            let sensor_key: FullChannelCacheKey = FullChannelCacheKey::new(cortical_type, cortical_group, channel);
            let sensor_cache: SensoryChannelStreamCache = SensoryChannelStreamCache::new(
                initial_processor_chains.pop().unwrap(),
                channel,
                allow_stale_data
            )?;
            
            _ = self.channel_caches.insert(sensor_key.clone(), sensor_cache);
            cache_keys.push(sensor_key);
        }
        
        
        let cortical_cache_details = CorticalAreaCacheDetails::new(cache_keys, number_supported_channels, neuron_encoder);
        _ = self.cortical_area_metadata.insert(cortical_metadata, cortical_cache_details);
        
        Ok(())
    }
    
    fn update_value_by_channel(&mut self, value: WrappedIOData, cortical_sensor_type: SensorCorticalType, cortical_grouping_index: CorticalGroupIndex, device_channel: CorticalChannelIndex) -> Result<(), FeagiDataError> {
        let mut channel_cache = self.try_get_mut_sensory_channel_stream_cache(cortical_sensor_type, cortical_grouping_index, device_channel)?;
        if channel_cache.get_input_data_type() != WrappedIOType::from(&value) {
            return Err(FeagiDataError::BadParameters(format!("Got value type {:?} when expected type {:?} for Cortical Type {:?}, Group Index {:?}, Channel {:?}!", WrappedIOType::from(&value),
                                                              channel_cache.get_input_data_type(), cortical_sensor_type, cortical_grouping_index, device_channel)).into());
        }
        _ = channel_cache.update_sensor_value(value);
        Ok(())
    }

    fn read_value_by_channel(&self, cortical_sensor_type: SensorCorticalType, cortical_grouping_index: CorticalGroupIndex, device_channel: CorticalChannelIndex) -> Result<WrappedIOData, FeagiDataError> {
        let channel_cache = self.try_get_sensory_channel_stream_cache(cortical_sensor_type, cortical_grouping_index, device_channel)?;
        let value = channel_cache.get_most_recent_sensor_value();
        Ok(value.clone())
    }
    
    fn set_pipeline_stages_for_channel(&mut self, cortical_sensor_type: SensorCorticalType,
                                       cortical_grouping_index: CorticalGroupIndex, device_channel: CorticalChannelIndex,
                                       new_stages: Vec<Box<dyn PipelineStage + Sync + Send>>) -> Result<(), FeagiDataError> {
        let channel_cache = self.try_get_mut_sensory_channel_stream_cache(cortical_sensor_type, cortical_grouping_index, device_channel)?;
        channel_cache.attempt_replace_pipeline_stages(new_stages)?;
        Ok(())
    }

    fn set_pipeline_stage_for_channel(&mut self, cortical_sensor_type: SensorCorticalType,
                                      cortical_grouping_index: CorticalGroupIndex, device_channel: CorticalChannelIndex,
                                      overwriting_stage: Box<dyn PipelineStage + Sync + Send>,
                                      overwriting_index: PipelineStageIndex) -> Result<(), FeagiDataError> {
        let cache = self.try_get_mut_sensory_channel_stream_cache(cortical_sensor_type, cortical_grouping_index, device_channel)?;
        cache.attempt_replace_pipeline_stage(overwriting_stage, overwriting_index)?;
        Ok(())
    }

    fn clone_pipeline_stages_for_channel(&mut self, cortical_sensor_type: SensorCorticalType,
                                         cortical_grouping_index: CorticalGroupIndex,
                                         device_channel: CorticalChannelIndex) -> Result<(Vec<Box<dyn PipelineStage + Sync + Send>>), FeagiDataError> {
        let cache = self.try_get_mut_sensory_channel_stream_cache(cortical_sensor_type, cortical_grouping_index, device_channel)?;
        Ok(cache.clone_pipeline_stages())
    }

    fn clone_pipeline_stage_for_channel(&mut self, cortical_sensor_type: SensorCorticalType,
                                        cortical_grouping_index: CorticalGroupIndex, device_channel: CorticalChannelIndex,
                                        reading_index: PipelineStageIndex) -> Result<(Box<dyn PipelineStage + Sync + Send>), FeagiDataError> {
        let cache = self.try_get_mut_sensory_channel_stream_cache(cortical_sensor_type, cortical_grouping_index, device_channel)?;
        cache.clone_pipeline_stage(reading_index)
    }
    
    fn try_get_sensory_channel_stream_cache(&self, cortical_sensor_type: SensorCorticalType, cortical_grouping_index: CorticalGroupIndex,
                                            device_channel: CorticalChannelIndex) -> Result<(&SensoryChannelStreamCache), FeagiDataError> {
        let cortical_type = cortical_sensor_type.into();
        let channel_cache = match self.channel_caches.get(&FullChannelCacheKey::new(cortical_type, cortical_grouping_index, device_channel)) {
            Some(channel_stream_cache) => channel_stream_cache,
            None => return Err(FeagiDataError::BadParameters(format!("Unable to find Cortical Type {:?}, Group Index {:?}, Channel {:?}!", cortical_type, cortical_grouping_index, device_channel)).into())
        };
        Ok(channel_cache)
    }

    fn try_get_mut_sensory_channel_stream_cache(&mut self, cortical_sensor_type: SensorCorticalType, cortical_grouping_index: CorticalGroupIndex, device_channel: CorticalChannelIndex) -> Result<(&mut SensoryChannelStreamCache), FeagiDataError> {
        let cortical_type = cortical_sensor_type.into();
        let channel_cache = match self.channel_caches.get_mut(&FullChannelCacheKey::new(cortical_type, cortical_grouping_index, device_channel)) {
            Some(channel_stream_cache) => channel_stream_cache,
            None => return Err(FeagiDataError::BadParameters(format!("Unable to find Cortical Type {:?}, Group Index {:?}, Channel {:?}!", cortical_type, cortical_grouping_index, device_channel)).into())
        };
        Ok(channel_cache)
    }
    
    
    fn encode_to_neurons(&mut self, past_send_time: Instant) -> Result<(), FeagiDataError> {
        // TODO move to using iter(), I'm using for loops now cause im still a rust scrub
        for cortical_area_details in self.cortical_area_metadata.values() {
            let channel_cache_keys = &cortical_area_details.relevant_channel_lookups;
            let neuron_encoder = &cortical_area_details.neuron_encoder;
            for channel_cache_key in channel_cache_keys {
                let sensor_cache = self.channel_caches.get(channel_cache_key).unwrap();
                sensor_cache.encode_to_neurons(&mut self.neuron_data, neuron_encoder)?
            }
        }
        Ok(())
    }
    

    //endregion
    
}


//region Cortical Area Details

struct CorticalAreaCacheDetails {
    relevant_channel_lookups: Vec<FullChannelCacheKey>,
    number_channels: u32,
    neuron_encoder: Box<dyn NeuronXYZPEncoder + Sync + Send>
}

impl  CorticalAreaCacheDetails {
    pub(crate) fn new(relevant_channel_lookups: Vec<FullChannelCacheKey>, number_channels: u32, neuron_encoder: Box<dyn NeuronXYZPEncoder + Sync + Send>) -> Self {
        CorticalAreaCacheDetails{
            relevant_channel_lookups,
            number_channels,
            neuron_encoder
        }

    }
}

//endregion