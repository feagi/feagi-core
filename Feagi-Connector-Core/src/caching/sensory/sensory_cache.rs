use std::collections::HashMap;
use std::time::Instant;
use feagi_data_structures::data::image_descriptors::{GazeProperties, ImageFrameProperties, SegmentedImageFrameProperties};
use feagi_data_structures::data::{ImageFrame, SegmentedImageFrame};
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::descriptors::{AgentDeviceIndex, CorticalChannelCount, CorticalChannelIndex, CorticalGroupIndex};
use feagi_data_structures::genomic::{CorticalID, SensorCorticalType};
use feagi_data_structures::neurons::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZPEncoder};
use feagi_data_structures::neurons::xyzp::encoders::*;
use feagi_data_structures::neurons::NeuronCoderType;
use feagi_data_structures::processing::{ImageFrameSegmentator, ImageFrameProcessor};
use feagi_data_structures::wrapped_io_data::{WrappedIOData, WrappedIOType};
use feagi_data_structures::sensor_definition;
use crate::caching::hashmap_helpers::{AccessAgentLookupKey, CorticalAreaMetadataKey, FullChannelCacheKey};
use crate::caching::sensory::sensory_channel_stream_cache::SensoryChannelStreamCache;
use crate::data_pipeline::stages::{ImageFrameProcessorStage, ImageFrameSegmentatorStage, LinearScaleTo0And1Stage};
use crate::data_pipeline::StreamCacheStage;

macro_rules! define_cortical_group_funcs {
    (
        $cortical_io_type_enum_name:ident {
            $(
                $(#[doc = $doc:expr])?
                $cortical_type_key_name:ident => {
                    friendly_name: $display_name:expr,
                    snake_case_identifier: $snake_case_identifier:expr,
                    base_ascii: $base_ascii:expr,
                    channel_dimension_range: $channel_dimension_range:expr,
                    default_coder_type: $default_coder_type:ident,
                }
            ),* $(,)?
        }
    ) => {
        

    }
 }

pub struct SensorCache {
    channel_caches: HashMap<FullChannelCacheKey, SensoryChannelStreamCache>, // (cortical type, grouping index, channel) -> sensory data cache, the main lookup
    cortical_area_metadata: HashMap<CorticalAreaMetadataKey, CorticalAreaCacheDetails>, // (cortical type, grouping index) -> (Vec<FullChannelCacheKey>, number_channels, neuron_encoder), defines all channel caches for a cortical area, and its neuron encoder
    agent_key_proxy: HashMap<AccessAgentLookupKey, Vec<FullChannelCacheKey>>, // (CorticalType, AgentDeviceIndex) -> Vec<FullChannelCacheKey>, allows users to map any channel of a cortical type to an agent device ID
    neuron_data: CorticalMappedXYZPNeuronData // cached neuron data
}

impl SensorCache {
    pub fn new() -> SensorCache {
        SensorCache {
            channel_caches: HashMap::new(),
            cortical_area_metadata: HashMap::new(),
            agent_key_proxy: HashMap::new(),
            neuron_data: CorticalMappedXYZPNeuronData::new(),
        }
    }
    
    //region Registration

    //region Generated macro functions
    
    // Generate registration functions for all sensor types with default_coder_type
    sensor_definition!(define_cortical_group_registration);

    //endregion

    //region Manual Registration Functions
    // Manually-defined functions for sensor types that need special handling
    

    pub fn register_cortical_group_for_image_camera_with_peripheral(&mut self, cortical_group: CorticalGroupIndex,
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
        let mut processors: Vec<Vec<Box<dyn StreamCacheStage + Sync + Send>>> = Vec::with_capacity(*number_of_channels as usize);
        for i in 0..*number_of_channels {
            processors.push(vec![Box::new(ImageFrameSegmentatorStage::new(input_image_properties, output_image_properties, segmentator.clone()))]);
        };
        
        self.register_cortical_area_and_channels(sensor_cortical_type, cortical_group, neuron_encoder, processors, allow_stale_data)?;
        Ok(())


    }

    //endregion

    fn register_agent_device_index(&mut self, agent_device_index: AgentDeviceIndex, cortical_sensor_type: SensorCorticalType,
                                   cortical_grouping_index: CorticalGroupIndex, device_channel: CorticalChannelIndex) -> Result<(), FeagiDataError> {

        let cortical_type = cortical_sensor_type.into();
        _ = self.channel_caches.get(&FullChannelCacheKey::new(cortical_type, cortical_grouping_index, device_channel))
            .ok_or_else(|| FeagiDataError::BadParameters(format!("Unable to find Cortical Type {:?}, Group Index {:?}, Channel {:?}!", cortical_type, cortical_grouping_index, device_channel)))?;

        let full_channel_key: FullChannelCacheKey = FullChannelCacheKey::new(cortical_type, cortical_grouping_index, device_channel);
        let try_key_vector = self.agent_key_proxy.get_mut(&AccessAgentLookupKey::new(cortical_type, agent_device_index));

        match try_key_vector {
            Some(key_vector) => {
                // There already is a mapping. Verify the input data types match
                let new_checking_cache = self.channel_caches.get(&full_channel_key).unwrap();
                let first_key = key_vector.first().unwrap();
                let first_checking_cache = self.channel_caches.get(first_key).unwrap();
                if new_checking_cache.get_input_data_type() != first_checking_cache.get_input_data_type() {
                    return Err(FeagiDataError::BadParameters(format!("Cannot to the same Agent Device Index {:?} that already contains a device channel accepting {} another device channel that accepts {}! Types must match!",
                                                                      agent_device_index, first_checking_cache.get_input_data_type(), new_checking_cache.get_input_data_type())).into())
                }

                key_vector.push(full_channel_key)
            }
            None => {
                // No listing exists, create one
                let new_vector: Vec<FullChannelCacheKey> = vec![full_channel_key];
                _ = self.agent_key_proxy.insert(AccessAgentLookupKey::new(cortical_type, agent_device_index), new_vector);
            }
        }

        Ok(())
    }
    
    //endregion
    
    
    
    //region Send Data
    
    //region macro
    
    pub fn send_data_for_proximity(&mut self, new_value: f32, cortical_grouping_index: CorticalGroupIndex, device_channel: CorticalChannelIndex) -> Result<(), FeagiDataError> {
        let val = WrappedIOData::F32(new_value);
        let sensor_type = SensorCorticalType::Proximity;
        self.update_value_by_channel(val, sensor_type, cortical_grouping_index, device_channel)
    }
    
    //endregion
    
    //region Custom Calls
    
    pub fn send_data_for_image_camera(&mut self, new_value: ImageFrame, cortical_grouping_index: CorticalGroupIndex, device_channel: CorticalChannelIndex) -> Result<(), FeagiDataError> {
        let val = WrappedIOData::ImageFrame(new_value);
        let sensor_type = SensorCorticalType::ImageCameraCenter;
        self.update_value_by_channel(val, sensor_type, cortical_grouping_index, device_channel)
    }

    pub fn send_data_for_segmented_image_camera(&mut self, new_value: ImageFrame, cortical_grouping_index: CorticalGroupIndex, device_channel: CorticalChannelIndex) -> Result<(), FeagiDataError> {
        let val = WrappedIOData::ImageFrame(new_value);
        let sensor_type = SensorCorticalType::ImageCameraCenter;
        self.update_value_by_channel(val, sensor_type, cortical_grouping_index, device_channel)// TODO ????
    }
    
    //endregion

    pub fn encode_to_neurons(&self, past_send_time: Instant, neurons_to_encode_to: &mut CorticalMappedXYZPNeuronData) -> Result<(), FeagiDataError> {
        // TODO move to using iter(), I'm using for loops now cause im still a rust scrub
        for cortical_area_details in self.cortical_area_metadata.values() {
            let channel_cache_keys = &cortical_area_details.relevant_channel_lookups;
            let neuron_encoder = &cortical_area_details.neuron_encoder;
            for channel_cache_key in channel_cache_keys {
                let sensor_cache = self.channel_caches.get(channel_cache_key).unwrap();
                sensor_cache.encode_to_neurons(neurons_to_encode_to, neuron_encoder)?
            }
        }
        Ok(())
    }
    
    //endregion
    
    

    




    //region Internal Functions
    
    //region By-Type Registration
    
    
    //endregion
    
    
    fn register_cortical_area_and_channels(&mut self, sensor_cortical_type: SensorCorticalType, cortical_group: CorticalGroupIndex,
                                           neuron_encoder: Box<dyn NeuronXYZPEncoder + Sync + Send>,
                                           mut initial_processor_chains: Vec<Vec<Box<dyn StreamCacheStage + Sync + Send>>>,
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
    
    
    pub fn update_value_by_channel(&mut self, value: WrappedIOData, cortical_sensor_type: SensorCorticalType, cortical_grouping_index: CorticalGroupIndex, device_channel: CorticalChannelIndex) -> Result<(), FeagiDataError> {
        let cortical_type = cortical_sensor_type.into();
        let channel_cache = match self.channel_caches.get_mut(&FullChannelCacheKey::new(cortical_type, cortical_grouping_index, device_channel)) {
            Some(channel_stream_cache) => channel_stream_cache,
            None => return Err(FeagiDataError::BadParameters(format!("Unable to find Cortical Type {:?}, Group Index {:?}, Channel {:?}!", cortical_type, cortical_grouping_index, device_channel)).into())
        };
        if channel_cache.get_input_data_type() != WrappedIOType::from(&value) {
            return Err(FeagiDataError::BadParameters(format!("Got value type {:?} when expected type {:?} for Cortical Type {:?}, Group Index {:?}, Channel {:?}!", WrappedIOType::from(&value),
                                                              channel_cache.get_input_data_type(), cortical_type, cortical_grouping_index, device_channel)).into());
        }
        _ = channel_cache.update_sensor_value(value);
        Ok(())
    }
    
    

    //endregion
    
    
    
}


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