use std::collections::HashMap;
use std::ops::Range;
use std::time::Instant;
use feagi_data_serialization::FeagiByteStructure;
use feagi_data_structures::data::descriptors::{GazeProperties, ImageFrameProperties, MiscDataDimensions, SegmentedImageFrameProperties};
use feagi_data_structures::data::{ImageFrame, MiscData, Percentage, SegmentedImageFrame, SignedPercentage};
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::descriptors::{AgentDeviceIndex, CorticalChannelCount, CorticalChannelIndex, CorticalGroupIndex, NeuronDepth};
use feagi_data_structures::genomic::{MotorCorticalType, SensorCorticalType};
use feagi_data_structures::neurons::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZPDecoder, NeuronXYZPEncoder};
use feagi_data_structures::neurons::xyzp::decoders::MiscDataNeuronXYZPDecoder;
use feagi_data_structures::neurons::xyzp::encoders::{F32LinearNeuronXYZPEncoder, F32SplitSignDividedNeuronXYZPEncoder, ImageFrameNeuronXYZPEncoder, MiscDataNeuronXYZPEncoder, SegmentedImageFrameNeuronXYZPEncoder};
use feagi_data_structures::processing::{ImageFrameProcessor, ImageFrameSegmentator};
use feagi_data_structures::wrapped_io_data::{WrappedIOData, WrappedIOType};
use crate::caching::hashmap_helpers::{AccessAgentLookupKey, CorticalAreaMetadataKey, FullChannelCacheKey};
use crate::caching::motor_channel_stream_cache::MotorChannelStreamCache;
use crate::caching::sensory_channel_stream_cache::SensoryChannelStreamCache;
use crate::data_pipeline::{PipelineStage, PipelineStageIndex};
use crate::data_pipeline::stages::{IdentityImageFrameStage, ImageFrameProcessorStage, ImageFrameSegmentatorStage, LinearScaleToPercentageStage, LinearScaleToSignedPercentageStage};

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


    //region Sensor Interfaces

    //region Common

    //region Percentage

    pub fn register_percentage_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                               group: CorticalGroupIndex, number_channels: CorticalChannelCount,
                               neuron_depth: NeuronDepth, bounds: Range<f32>) -> Result<(), FeagiDataError> {

        sensor_cortical_type.verify_is_data_type(WrappedIOType::Percentage)?;
        let cortical_id = sensor_cortical_type.to_cortical_id(group);
        let encoder =  Box::new(F32LinearNeuronXYZPEncoder::new(cortical_id, neuron_depth)?);
        let mut processors: Vec<Vec<Box<dyn PipelineStage + Sync + Send>>> = Vec::with_capacity(*number_channels as usize);
        for _i in 0..*number_channels {
            processors.push(vec![Box::new(LinearScaleToPercentageStage::new(bounds.start, bounds.end, Percentage::new_from_0_1_unchecked(0.0))?)]);
        };
        self.sensor_register_cortical_area_and_channels(sensor_cortical_type, group, encoder, processors)?;
        Ok(())
    }

    pub fn store_percentage_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                            group: CorticalGroupIndex, channel: CorticalChannelIndex,
                            new_float: f32) -> Result<(), FeagiDataError> {
        sensor_cortical_type.verify_is_data_type(WrappedIOType::Percentage)?;
        let val = WrappedIOData::F32(new_float);
        self.sensor_update_value_by_channel(val, sensor_cortical_type, group, channel);
        Ok(())
    }

    pub fn read_cache_percentage_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                                 group: CorticalGroupIndex, channel: CorticalChannelIndex)
                                 -> Result<f32, FeagiDataError> {
        sensor_cortical_type.verify_is_data_type(WrappedIOType::Percentage)?;
        let val = self.sensor_read_value_by_channel(sensor_cortical_type, group, channel)?;
        Ok(val.try_into()?)
    }

    pub fn set_pipeline_stages_percentage_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                                          group: CorticalGroupIndex, channel: CorticalChannelIndex,
                                          new_stages: Vec<Box<dyn PipelineStage + Sync + Send>>) -> Result<(), FeagiDataError> {
        sensor_cortical_type.verify_is_data_type(WrappedIOType::Percentage)?;
        self.sensor_set_pipeline_stages_for_channel(sensor_cortical_type, group, channel, new_stages)?;
        Ok(())
    }
    pub fn set_pipeline_stage_percentage_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                                         group: CorticalGroupIndex, channel: CorticalChannelIndex,
                                         new_stage: Box<dyn PipelineStage + Sync + Send>,
                                         stage_index: PipelineStageIndex) -> Result<(), FeagiDataError> {
        sensor_cortical_type.verify_is_data_type(WrappedIOType::Percentage)?;
        self.sensor_set_pipeline_stage_for_channel(sensor_cortical_type, group, channel, new_stage, stage_index)
    }

    pub fn clone_pipeline_stages_percentage_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                                            group: CorticalGroupIndex, channel: CorticalChannelIndex)
                                            -> Result<Vec<Box<dyn PipelineStage + Sync + Send>>, FeagiDataError> {
        sensor_cortical_type.verify_is_data_type(WrappedIOType::Percentage)?;
        self.sensor_clone_pipeline_stages_for_channel(sensor_cortical_type, group, channel)
    }

    pub fn clone_pipeline_stage_percentage_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                                           group: CorticalGroupIndex, channel: CorticalChannelIndex,
                                           stage_index: PipelineStageIndex) -> Result<Box<dyn PipelineStage + Sync + Send>, FeagiDataError> {
        sensor_cortical_type.verify_is_data_type(WrappedIOType::Percentage)?;
        self.sensor_clone_pipeline_stage_for_channel(sensor_cortical_type, group, channel, stage_index)
    }

    pub fn register_device_agent_index_percentage_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                                                  group: CorticalGroupIndex, channel: CorticalChannelIndex,
                                                  agent_device_index: AgentDeviceIndex) -> Result<(), FeagiDataError> {

        sensor_cortical_type.verify_is_data_type(WrappedIOType::Percentage)?;
        self.sensor_register_agent_device_index(sensor_cortical_type, group, channel, agent_device_index)
    }

    pub fn store_device_agent_index_percentage_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                                               agent_device_index: AgentDeviceIndex,
                                               value: f32) -> Result<(), FeagiDataError> {

        sensor_cortical_type.verify_is_data_type(WrappedIOType::Percentage)?;
        self.sensor_store_value_from_device_index(sensor_cortical_type, agent_device_index, WrappedIOData::F32(value))
    }


    //endregion

    //region SignedPercentage

    pub fn register_signed_percentage_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                                group: CorticalGroupIndex, number_channels: CorticalChannelCount,
                                neuron_depth: NeuronDepth, bounds: Range<f32>) -> Result<(), FeagiDataError> {

        sensor_cortical_type.verify_is_data_type(WrappedIOType::SignedPercentage)?;
        let cortical_id = sensor_cortical_type.to_cortical_id(group);
        let encoder =  Box::new(F32SplitSignDividedNeuronXYZPEncoder::new(cortical_id, neuron_depth)?); // TODO this shouldnt be hard coded, there are different versions
        let mut processors: Vec<Vec<Box<dyn PipelineStage + Sync + Send>>> = Vec::with_capacity(*number_channels as usize);
        for _i in 0..*number_channels {
            processors.push(vec![Box::new(LinearScaleToSignedPercentageStage::new(bounds.start, bounds.end, SignedPercentage::new_from_m1_1_unchecked(0.0))?)]);
        };
        self.sensor_register_cortical_area_and_channels(sensor_cortical_type, group, encoder, processors)?;
        Ok(())
    }

    pub fn store_signed_percentage_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                             group: CorticalGroupIndex, channel: CorticalChannelIndex,
                             new_float: f32) -> Result<(), FeagiDataError> {
        sensor_cortical_type.verify_is_data_type(WrappedIOType::SignedPercentage)?;
        let val = WrappedIOData::F32(new_float);
        self.sensor_update_value_by_channel(val, sensor_cortical_type, group, channel);
        Ok(())
    }

    pub fn read_cache_signed_percentage_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                                  group: CorticalGroupIndex, channel: CorticalChannelIndex)
                                  -> Result<f32, FeagiDataError> {

        sensor_cortical_type.verify_is_data_type(WrappedIOType::SignedPercentage)?;
        let val = self.sensor_read_value_by_channel(sensor_cortical_type, group, channel)?;
        Ok(val.try_into()?)
    }

    pub fn set_pipeline_stages_signed_percentage_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                                           group: CorticalGroupIndex, channel: CorticalChannelIndex,
                                           new_stages: Vec<Box<dyn PipelineStage + Sync + Send>>)
                                           -> Result<(), FeagiDataError> {

        sensor_cortical_type.verify_is_data_type(WrappedIOType::SignedPercentage)?;
        self.sensor_set_pipeline_stages_for_channel(sensor_cortical_type, group, channel, new_stages)?;
        Ok(())
    }
    pub fn set_pipeline_stage_signed_percentage_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                                          group: CorticalGroupIndex, channel: CorticalChannelIndex,
                                          new_stage: Box<dyn PipelineStage + Sync + Send>,
                                          stage_index: PipelineStageIndex) -> Result<(), FeagiDataError> {

        sensor_cortical_type.verify_is_data_type(WrappedIOType::SignedPercentage)?;
        self.sensor_set_pipeline_stage_for_channel(sensor_cortical_type, group, channel, new_stage, stage_index)
    }

    pub fn clone_pipeline_stages_signed_percentage_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                                             group: CorticalGroupIndex,
                                             channel: CorticalChannelIndex)
                                             -> Result<Vec<Box<dyn PipelineStage + Sync + Send>>, FeagiDataError> {

        sensor_cortical_type.verify_is_data_type(WrappedIOType::SignedPercentage)?;
        self.sensor_clone_pipeline_stages_for_channel(sensor_cortical_type, group, channel)
    }

    pub fn clone_pipeline_stage_signed_percentage_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                                            group: CorticalGroupIndex, channel: CorticalChannelIndex,
                                            stage_index: PipelineStageIndex)
                                            -> Result<Box<dyn PipelineStage + Sync + Send>, FeagiDataError> {

        sensor_cortical_type.verify_is_data_type(WrappedIOType::SignedPercentage)?;
        self.sensor_clone_pipeline_stage_for_channel(sensor_cortical_type, group, channel, stage_index)
    }

    pub fn register_device_agent_index_signed_percentage_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                                                   group: CorticalGroupIndex, channel: CorticalChannelIndex,
                                                   agent_device_index: AgentDeviceIndex)
                                                   -> Result<(), FeagiDataError> {

        sensor_cortical_type.verify_is_data_type(WrappedIOType::SignedPercentage)?;
        self.sensor_register_agent_device_index(sensor_cortical_type, group, channel, agent_device_index)
    }

    pub fn store_device_agent_index_signed_percentage_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                                                agent_device_index: AgentDeviceIndex, value: f32)
                                                -> Result<(), FeagiDataError> {

        sensor_cortical_type.verify_is_data_type(WrappedIOType::SignedPercentage)?;
        self.sensor_store_value_from_device_index(sensor_cortical_type, agent_device_index, WrappedIOData::F32(value))
    }






    //endregion

    //region ImageFrame

    pub fn register_image_frame_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                                group: CorticalGroupIndex, number_channels: CorticalChannelCount,
                                input_image_properties: ImageFrameProperties,
                                output_image_properties: ImageFrameProperties) -> Result<(), FeagiDataError> {

        sensor_cortical_type.verify_is_data_type(WrappedIOType::ImageFrame(None))?;
        let cortical_id = sensor_cortical_type.to_cortical_id(group);
        let encoder =  Box::new(ImageFrameNeuronXYZPEncoder::new(cortical_id, &output_image_properties)?);
        let mut stages: Vec<Vec<Box<dyn PipelineStage + Sync + Send>>> = Vec::with_capacity(*number_channels as usize);
        if input_image_properties == output_image_properties {
            // No changes to image, just cache it to avoid processing penalty
            let initial_image = ImageFrame::new_from_image_frame_properties(&input_image_properties)?;
            stages.push(vec![Box::new(IdentityImageFrameStage::new(initial_image)?)])
        }
        else {
            // We are changing the image, add a processor
            let image_transformer_definition = ImageFrameProcessor::new_from_input_output_properties(&input_image_properties, &output_image_properties)?;
            for _i in 0..*number_channels {
                stages.push(vec![Box::new(ImageFrameProcessorStage::new(image_transformer_definition.clone())?)]);
            };
        }
        self.sensor_register_cortical_area_and_channels(sensor_cortical_type, group, encoder, stages)?;
        Ok(())
    }

    pub fn store_image_frame_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                             group: CorticalGroupIndex, channel: CorticalChannelIndex,
                             new_image: ImageFrame) -> Result<(), FeagiDataError> {
        sensor_cortical_type.verify_is_data_type(WrappedIOType::ImageFrame(None))?;
        let val = WrappedIOData::ImageFrame(new_image);
        self.sensor_update_value_by_channel(val, sensor_cortical_type, group, channel);
        Ok(())
    }

    pub fn read_cache_image_frame_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                                  group: CorticalGroupIndex, channel: CorticalChannelIndex)
                                  -> Result<ImageFrame, FeagiDataError> {
        sensor_cortical_type.verify_is_data_type(WrappedIOType::ImageFrame(None))?;
        let val = self.sensor_read_value_by_channel(sensor_cortical_type, group, channel)?;
        Ok(val.try_into()?)
    }

    pub fn set_pipeline_stages_image_frame_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                                           group: CorticalGroupIndex, channel: CorticalChannelIndex,
                                           new_stages: Vec<Box<dyn PipelineStage + Sync + Send>>) -> Result<(), FeagiDataError> {
        sensor_cortical_type.verify_is_data_type(WrappedIOType::ImageFrame(None))?;
        self.sensor_set_pipeline_stages_for_channel(sensor_cortical_type, group, channel, new_stages)?;
        Ok(())
    }
    pub fn set_pipeline_stage_image_frame_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                                          group: CorticalGroupIndex, channel: CorticalChannelIndex,
                                          new_stage: Box<dyn PipelineStage + Sync + Send>,
                                          stage_index: PipelineStageIndex) -> Result<(), FeagiDataError> {
        sensor_cortical_type.verify_is_data_type(WrappedIOType::ImageFrame(None))?;
        self.sensor_set_pipeline_stage_for_channel(sensor_cortical_type, group, channel, new_stage, stage_index)
    }

    pub fn clone_pipeline_stages_image_frame_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                                             group: CorticalGroupIndex, channel: CorticalChannelIndex)
                                             -> Result<Vec<Box<dyn PipelineStage + Sync + Send>>, FeagiDataError> {
        sensor_cortical_type.verify_is_data_type(WrappedIOType::ImageFrame(None))?;
        self.sensor_clone_pipeline_stages_for_channel(sensor_cortical_type, group, channel)
    }

    pub fn clone_pipeline_stage_image_frame_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                                            group: CorticalGroupIndex, channel: CorticalChannelIndex,
                                            stage_index: PipelineStageIndex) -> Result<Box<dyn PipelineStage + Sync + Send>, FeagiDataError> {
        sensor_cortical_type.verify_is_data_type(WrappedIOType::ImageFrame(None))?;
        self.sensor_clone_pipeline_stage_for_channel(sensor_cortical_type, group, channel, stage_index)
    }

    pub fn register_device_agent_index_image_frame_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                                                   group: CorticalGroupIndex, channel: CorticalChannelIndex,
                                                   agent_device_index: AgentDeviceIndex) -> Result<(), FeagiDataError> {

        sensor_cortical_type.verify_is_data_type(WrappedIOType::ImageFrame(None))?;
        self.sensor_register_agent_device_index(sensor_cortical_type, group, channel, agent_device_index)
    }

    pub fn store_device_agent_index_image_frame_sensor(&mut self, sensor_cortical_type: SensorCorticalType,
                                                agent_device_index: AgentDeviceIndex,
                                                value: ImageFrame) -> Result<(), FeagiDataError> {

        sensor_cortical_type.verify_is_data_type(WrappedIOType::ImageFrame(None))?;
        self.sensor_store_value_from_device_index(sensor_cortical_type, agent_device_index, WrappedIOData::ImageFrame(value))
    }

    //endregion

    //region MiscData

    pub fn register_misc_data_sensor(&mut self, group: CorticalGroupIndex, number_channels: CorticalChannelCount,
                                dimensions: MiscDataDimensions, ) -> Result<(), FeagiDataError> {

        // Type will always be Misc
        let sensor_cortical_type = SensorCorticalType::Miscellaneous;
        let cortical_id = sensor_cortical_type.to_cortical_id(group);
        let encoder =  Box::new(MiscDataNeuronXYZPEncoder::new(cortical_id, dimensions)?);
        let mut stages: Vec<Vec<Box<dyn PipelineStage + Sync + Send>>> = Vec::with_capacity(*number_channels as usize);
        // TODO add stage
        self.sensor_register_cortical_area_and_channels(sensor_cortical_type, group, encoder, stages)?;
        Ok(())
    }

    pub fn store_misc_data_sensor(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex,
                             new_data: MiscData) -> Result<(), FeagiDataError> {

        let sensor_cortical_type = SensorCorticalType::Miscellaneous;
        let val = WrappedIOData::MiscData(new_data);
        self.sensor_update_value_by_channel(val, sensor_cortical_type, group, channel);
        Ok(())
    }

    pub fn read_cache_misc_data_sensor(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex)
                                  -> Result<MiscData, FeagiDataError> {

        let sensor_cortical_type = SensorCorticalType::Miscellaneous;
        let val = self.sensor_read_value_by_channel(sensor_cortical_type, group, channel)?;
        Ok(val.try_into()?)
    }

    pub fn set_pipeline_stages_misc_data_sensor(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex,
                                           new_stages: Vec<Box<dyn PipelineStage + Sync + Send>>) -> Result<(), FeagiDataError> {

        let sensor_cortical_type = SensorCorticalType::Miscellaneous;
        self.sensor_set_pipeline_stages_for_channel(sensor_cortical_type, group, channel, new_stages)?;
        Ok(())
    }
    pub fn set_pipeline_stage_misc_data_sensor(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex,
                                          new_stage: Box<dyn PipelineStage + Sync + Send>,
                                          stage_index: PipelineStageIndex) -> Result<(), FeagiDataError> {

        let sensor_cortical_type = SensorCorticalType::Miscellaneous;
        self.sensor_set_pipeline_stage_for_channel(sensor_cortical_type, group, channel, new_stage, stage_index)
    }

    pub fn clone_pipeline_stages_misc_data_sensor(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex)
                                             -> Result<Vec<Box<dyn PipelineStage + Sync + Send>>, FeagiDataError> {

        let sensor_cortical_type = SensorCorticalType::Miscellaneous;
        self.sensor_clone_pipeline_stages_for_channel(sensor_cortical_type, group, channel)
    }

    pub fn clone_pipeline_stage_misc_data_sensor(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex,
                                            stage_index: PipelineStageIndex) -> Result<Box<dyn PipelineStage + Sync + Send>, FeagiDataError> {

        let sensor_cortical_type = SensorCorticalType::Miscellaneous;
        self.sensor_clone_pipeline_stage_for_channel(sensor_cortical_type, group, channel, stage_index)
    }

    pub fn register_device_agent_index_misc_data_sensor(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex,
                                                   agent_device_index: AgentDeviceIndex) -> Result<(), FeagiDataError> {

        let sensor_cortical_type = SensorCorticalType::Miscellaneous;

        self.sensor_register_agent_device_index(sensor_cortical_type, group, channel, agent_device_index)
    }

    pub fn store_device_agent_index_misc_data_sensor(&mut self,  agent_device_index: AgentDeviceIndex,
                                                value: ImageFrame) -> Result<(), FeagiDataError> {

        let sensor_cortical_type = SensorCorticalType::Miscellaneous;
        self.sensor_store_value_from_device_index(sensor_cortical_type, agent_device_index, WrappedIOData::ImageFrame(value))
    }

    //endregion

    //endregion

    //region Unique

    //region Segmented Image Camera Manual Functions

    pub fn register_segmented_image_frame_sensor(&mut self, cortical_group: CorticalGroupIndex,
                                          number_of_channels: CorticalChannelCount,
                                          input_image_properties: ImageFrameProperties,
                                          output_image_properties: SegmentedImageFrameProperties,
                                          segmentation_center_properties: GazeProperties) -> Result<(), FeagiDataError> {
        let sensor_cortical_type = SensorCorticalType::ImageCameraCenter;

        let cortical_ids = SegmentedImageFrame::create_ordered_cortical_ids_for_segmented_vision(cortical_group);
        for cortical_id in &cortical_ids {
            let cortical_type = cortical_id.get_cortical_type();
            let cortical_metadata = CorticalAreaMetadataKey::new(cortical_type, cortical_group);
            if self.sensor_cortical_area_metadata.contains_key(&cortical_metadata) {
                return Err(FeagiDataError::InternalError("Cortical area already registered!".into()).into())
            }
        }; // ensure no cortical ID is used already

        let segmentator = ImageFrameSegmentator::new(input_image_properties, output_image_properties, segmentation_center_properties)?;
        let neuron_encoder = Box::new(SegmentedImageFrameNeuronXYZPEncoder::new(cortical_ids, output_image_properties)?);
        let mut processors: Vec<Vec<Box<dyn PipelineStage + Sync + Send>>> = Vec::with_capacity(*number_of_channels as usize);
        for _i in 0..*number_of_channels {
            processors.push(vec![Box::new(ImageFrameSegmentatorStage::new(input_image_properties, output_image_properties, segmentator.clone()))]);
        };

        self.sensor_register_cortical_area_and_channels(sensor_cortical_type, cortical_group, neuron_encoder, processors)?;
        Ok(())
    }


    pub fn store_segmented_image_frame_sensor(&mut self, new_value: ImageFrame, cortical_grouping_index: CorticalGroupIndex, device_channel: CorticalChannelIndex) -> Result<(), FeagiDataError> {
        let val = WrappedIOData::ImageFrame(new_value);
        let sensor_type = SensorCorticalType::ImageCameraCenter;
        self.sensor_update_value_by_channel(val, sensor_type, cortical_grouping_index, device_channel)
    }


    pub fn read_cache_segmented_image_frame_sensor(&mut self,cortical_group: CorticalGroupIndex, device_channel: CorticalChannelIndex) -> Result<SegmentedImageFrame, FeagiDataError> {
        let val = self.sensor_read_value_by_channel(SensorCorticalType::ImageCameraCenter, cortical_group, device_channel)?;
        Ok(val.try_into()?)
    }

    pub fn set_pipeline_stages_segmented_image_frame_sensor(&mut self, cortical_group: CorticalGroupIndex, device_channel: CorticalChannelIndex, new_stages: Vec<Box<dyn PipelineStage + Sync + Send>>) -> Result<(), FeagiDataError> {
        let sensor_type = SensorCorticalType::ImageCameraCenter;
        self.sensor_set_pipeline_stages_for_channel(sensor_type, cortical_group, device_channel, new_stages)
    }

    pub fn set_pipeline_stage_segmented_image_frame_sensor(&mut self, cortical_group: CorticalGroupIndex, device_channel: CorticalChannelIndex, new_stage: Box<dyn PipelineStage + Sync + Send>, stage_index: PipelineStageIndex) -> Result<(), FeagiDataError> {
        let sensor_type = SensorCorticalType::ImageCameraCenter;
        self.sensor_set_pipeline_stage_for_channel(sensor_type, cortical_group, device_channel, new_stage, stage_index)
    }

    pub fn clone_pipeline_stages_segmented_image_frame_sensor(&mut self, cortical_group: CorticalGroupIndex, device_channel: CorticalChannelIndex) -> Result<Vec<Box<dyn PipelineStage + Sync + Send>>, FeagiDataError> {
        let sensor_type = SensorCorticalType::ImageCameraCenter;
        self.sensor_clone_pipeline_stages_for_channel(sensor_type, cortical_group, device_channel)
    }

    pub fn clone_pipeline_stage_segmented_image_frame_sensor(&mut self, cortical_group: CorticalGroupIndex, device_channel: CorticalChannelIndex, stage_index: PipelineStageIndex) -> Result<Box<dyn PipelineStage + Sync + Send>, FeagiDataError> {
        let sensor_type = SensorCorticalType::ImageCameraCenter;
        self.sensor_clone_pipeline_stage_for_channel(sensor_type, cortical_group, device_channel, stage_index)
    }

    pub fn register_agent_device_segmented_image_frame_sensor(&mut self, cortical_group: CorticalGroupIndex, device_channel: CorticalChannelIndex, agent_device_index: AgentDeviceIndex) -> Result<(), FeagiDataError> {
        let sensor_type = SensorCorticalType::ImageCameraCenter;
        self.sensor_register_agent_device_index(sensor_type, cortical_group, device_channel, agent_device_index)
    }

    pub fn store_agent_device_segmented_image_frame_sensor(&mut self, agent_device_index: AgentDeviceIndex, value: ImageFrame) -> Result<(), FeagiDataError> {
        let sensor_type = SensorCorticalType::ImageCameraCenter;
        self.sensor_store_value_from_device_index(sensor_type, agent_device_index, WrappedIOData::ImageFrame(value))
    }



    //endregion


    //endregion

    //endregion


    //region Motor Interfaces

    //region Common

    //region MiscData

    pub fn register_misc_data_motor(&mut self, group: CorticalGroupIndex, number_channels: CorticalChannelCount,
                                     dimensions: MiscDataDimensions, ) -> Result<(), FeagiDataError> {

        // Type will always be Misc
        let motor_cortical_type = MotorCorticalType::Miscellaneous;
        let cortical_id = motor_cortical_type.to_cortical_id(group);
        let decoder =  Box::new(MiscDataNeuronXYZPDecoder::new(cortical_id, dimensions)?);
        let mut stages: Vec<Vec<Box<dyn PipelineStage + Sync + Send>>> = Vec::with_capacity(*number_channels as usize);
        // TODO add stage
        self.motor_register_cortical_area_and_channels(motor_cortical_type, group, decoder, stages)?;
        Ok(())
    }

    pub fn read_cache_misc_data_motor(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex)
                                       -> Result<MiscData, FeagiDataError> {

        let motor_cortical_type = MotorCorticalType::Miscellaneous;
        let val = self.motor_read_value_by_channel(motor_cortical_type, group, channel)?;
        Ok(val.try_into()?)
    }

    pub fn set_pipeline_stages_misc_data_motor(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex,
                                                new_stages: Vec<Box<dyn PipelineStage + Sync + Send>>) -> Result<(), FeagiDataError> {

        let sensor_cortical_type = SensorCorticalType::Miscellaneous;
        self.sensor_set_pipeline_stages_for_channel(sensor_cortical_type, group, channel, new_stages)?;
        Ok(())
    }

    pub fn set_pipeline_stage_misc_data_motor(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex,
                                               new_stage: Box<dyn PipelineStage + Sync + Send>,
                                               stage_index: PipelineStageIndex) -> Result<(), FeagiDataError> {

        let sensor_cortical_type = SensorCorticalType::Miscellaneous;
        self.sensor_set_pipeline_stage_for_channel(sensor_cortical_type, group, channel, new_stage, stage_index)
    }

    pub fn clone_pipeline_stages_misc_data_motor(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex)
                                                  -> Result<Vec<Box<dyn PipelineStage + Sync + Send>>, FeagiDataError> {

        let sensor_cortical_type = SensorCorticalType::Miscellaneous;
        self.sensor_clone_pipeline_stages_for_channel(sensor_cortical_type, group, channel)
    }

    pub fn clone_pipeline_stage_misc_data_motor(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex,
                                                 stage_index: PipelineStageIndex) -> Result<Box<dyn PipelineStage + Sync + Send>, FeagiDataError> {

        let sensor_cortical_type = SensorCorticalType::Miscellaneous;
        self.sensor_clone_pipeline_stage_for_channel(sensor_cortical_type, group, channel, stage_index)
    }

    pub fn register_device_agent_index_misc_data_motor(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex,
                                                        agent_device_index: AgentDeviceIndex) -> Result<(), FeagiDataError> {

        let sensor_cortical_type = SensorCorticalType::Miscellaneous;

        self.sensor_register_agent_device_index(sensor_cortical_type, group, channel, agent_device_index)
    }

    pub fn store_device_agent_index_misc_data_motor(&mut self,  agent_device_index: AgentDeviceIndex,
                                                     value: ImageFrame) -> Result<(), FeagiDataError> {

        let sensor_cortical_type = SensorCorticalType::Miscellaneous;
        self.sensor_store_value_from_device_index(sensor_cortical_type, agent_device_index, WrappedIOData::ImageFrame(value))
    }

    //endregion

    //endregion

    //endregion



    //region Internal Functions

    //region Sensory

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

    //region Motor

    //region Agent Functions


    fn motor_register_agent_device_index(&mut self, cortical_motor_type: MotorCorticalType,
                                          group: CorticalGroupIndex, channel: CorticalChannelIndex, agent_device_index: AgentDeviceIndex) -> Result<(), FeagiDataError> {

        _ = self.motor_try_get_motory_channel_stream_cache(cortical_motor_type, group, channel)?; // Check to ensure target exists

        let cortical_type = cortical_motor_type.into();
        let full_channel_key: FullChannelCacheKey = FullChannelCacheKey::new(cortical_type, group, channel);
        let try_key_vector = self.motor_agent_key_proxy.get_mut(&AccessAgentLookupKey::new(cortical_type, agent_device_index));

        match try_key_vector {
            Some(key_vector) => {
                if !key_vector.contains(&full_channel_key){
                    key_vector.push(full_channel_key)
                }
            }
            None => {
                let new_vector: Vec<FullChannelCacheKey> = vec![full_channel_key];
                _ = self.motor_agent_key_proxy.insert(AccessAgentLookupKey::new(cortical_type, agent_device_index), new_vector);
            }
        }
        Ok(())
    }


    //endregion

    fn motor_register_cortical_area_and_channels(&mut self, motor_cortical_type: MotorCorticalType, cortical_group: CorticalGroupIndex,
                                                  neuron_decoder: Box<dyn NeuronXYZPDecoder + Sync + Send>,
                                                  mut initial_processor_chains: Vec<Vec<Box<dyn PipelineStage + Sync + Send>>>) -> Result<(), FeagiDataError> {
        // NOTE: initial_processor_chains is a vector of vectors, meaning each channel gets a vector of processing

        let number_supported_channels = initial_processor_chains.len() as u32;
        let cortical_type = motor_cortical_type.into();
        let cortical_metadata = CorticalAreaMetadataKey::new(cortical_type, cortical_group);


        if number_supported_channels == 0 {
            return Err(FeagiDataError::BadParameters("A cortical area cannot be registered with 0 channels!".into()).into())
        }
        if self.motor_cortical_area_metadata.contains_key(&cortical_metadata) {
            return Err(FeagiDataError::InternalError("Cortical area already registered!".into()).into())
        }



        let mut cache_keys: Vec<FullChannelCacheKey> = Vec::with_capacity(number_supported_channels as usize);
        for i in 0..number_supported_channels {

            let channel: CorticalChannelIndex = i.into();
            let motor_key: FullChannelCacheKey = FullChannelCacheKey::new(cortical_type, cortical_group, channel);
            let motor_cache: MotorChannelStreamCache = MotorChannelStreamCache::new(
                initial_processor_chains.pop().unwrap(),
                channel,
            )?;

            _ = self.motor_channel_caches.insert(motor_key.clone(), motor_cache);
            cache_keys.push(motor_key);
        }


        let cortical_cache_details = MotorCorticalAreaCacheDetails::new(cache_keys, number_supported_channels, neuron_decoder);
        _ = self.motor_cortical_area_metadata.insert(cortical_metadata, cortical_cache_details);

        Ok(())
    }

    fn motor_read_value_by_channel(&self, cortical_motor_type: MotorCorticalType, cortical_grouping_index: CorticalGroupIndex, device_channel: CorticalChannelIndex) -> Result<WrappedIOData, FeagiDataError> {
        let channel_cache = self.motor_try_get_motory_channel_stream_cache(cortical_motor_type, cortical_grouping_index, device_channel)?;
        let value = channel_cache.get_most_recent_motor_value();
        Ok(value.clone())
    }

    fn motor_set_pipeline_stages_for_channel(&mut self, cortical_motor_type: MotorCorticalType,
                                              cortical_grouping_index: CorticalGroupIndex, device_channel: CorticalChannelIndex,
                                              new_stages: Vec<Box<dyn PipelineStage + Sync + Send>>) -> Result<(), FeagiDataError> {
        let channel_cache = self.motor_try_get_mut_motory_channel_stream_cache(cortical_motor_type, cortical_grouping_index, device_channel)?;
        channel_cache.attempt_replace_pipeline_stages(new_stages)?;
        Ok(())
    }

    fn motor_set_pipeline_stage_for_channel(&mut self, cortical_motor_type: MotorCorticalType,
                                             cortical_grouping_index: CorticalGroupIndex, device_channel: CorticalChannelIndex,
                                             overwriting_stage: Box<dyn PipelineStage + Sync + Send>,
                                             overwriting_index: PipelineStageIndex) -> Result<(), FeagiDataError> {
        let cache = self.motor_try_get_mut_motory_channel_stream_cache(cortical_motor_type, cortical_grouping_index, device_channel)?;
        cache.attempt_replace_pipeline_stage(overwriting_stage, overwriting_index)?;
        Ok(())
    }

    fn motor_clone_pipeline_stages_for_channel(&mut self, cortical_motor_type: MotorCorticalType,
                                                cortical_grouping_index: CorticalGroupIndex,
                                                device_channel: CorticalChannelIndex) -> Result<(Vec<Box<dyn PipelineStage + Sync + Send>>), FeagiDataError> {
        let cache = self.motor_try_get_mut_motory_channel_stream_cache(cortical_motor_type, cortical_grouping_index, device_channel)?;
        Ok(cache.clone_pipeline_stages())
    }

    fn motor_clone_pipeline_stage_for_channel(&mut self, cortical_motor_type: MotorCorticalType,
                                               cortical_grouping_index: CorticalGroupIndex, device_channel: CorticalChannelIndex,
                                               reading_index: PipelineStageIndex) -> Result<(Box<dyn PipelineStage + Sync + Send>), FeagiDataError> {
        let cache = self.motor_try_get_mut_motory_channel_stream_cache(cortical_motor_type, cortical_grouping_index, device_channel)?;
        cache.clone_pipeline_stage(reading_index)
    }

    fn motor_try_get_motory_channel_stream_cache(&self, cortical_motor_type: MotorCorticalType, cortical_grouping_index: CorticalGroupIndex,
                                                   device_channel: CorticalChannelIndex) -> Result<(&MotorChannelStreamCache), FeagiDataError> {
        let cortical_type = cortical_motor_type.into();
        let channel_cache = match self.motor_channel_caches.get(&FullChannelCacheKey::new(cortical_type, cortical_grouping_index, device_channel)) {
            Some(channel_stream_cache) => channel_stream_cache,
            None => return Err(FeagiDataError::BadParameters(format!("Unable to find Cortical Type {:?}, Group Index {:?}, Channel {:?}!", cortical_type, cortical_grouping_index, device_channel)).into())
        };
        Ok(channel_cache)
    }

    fn motor_try_get_mut_motory_channel_stream_cache(&mut self, cortical_motor_type: MotorCorticalType, cortical_grouping_index: CorticalGroupIndex, device_channel: CorticalChannelIndex) -> Result<(&mut MotorChannelStreamCache), FeagiDataError> {
        let cortical_type = cortical_motor_type.into();
        let channel_cache = match self.motor_channel_caches.get_mut(&FullChannelCacheKey::new(cortical_type, cortical_grouping_index, device_channel)) {
            Some(channel_stream_cache) => channel_stream_cache,
            None => return Err(FeagiDataError::BadParameters(format!("Unable to find Cortical Type {:?}, Group Index {:?}, Channel {:?}!", cortical_type, cortical_grouping_index, device_channel)).into())
        };
        Ok(channel_cache)
    }


    //endregion

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