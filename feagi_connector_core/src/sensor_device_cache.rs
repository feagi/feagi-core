use std::collections::HashMap;
use std::time::Instant;
use feagi_data_serialization::FeagiByteContainer;
use feagi_data_structures::{sensor_definition, FeagiDataError, FeagiSignal};
use feagi_data_structures::genomic::cortical_area::descriptors::{CorticalChannelCount, CorticalChannelIndex, CorticalGroupIndex};
use feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::FrameChangeHandling;
use feagi_data_structures::genomic::cortical_area::IOCorticalAreaDataType;
use feagi_data_structures::genomic::descriptors::{AgentDeviceIndex};
use feagi_data_structures::genomic::SensoryCorticalUnit;
use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use crate::caching::per_channel_stream_caches::SensoryChannelStreamCaches;
use crate::data_pipeline::{PipelineStageProperties, PipelineStagePropertyIndex};
use crate::data_pipeline::stage_properties::ImageSegmentorStageProperties;
use crate::data_types::*;
use crate::data_types::descriptors::*;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};
use crate::neuron_voxel_coding::xyzp::encoders::*;
use crate::neuron_voxel_coding::xyzp::{NeuronVoxelXYZPEncoder};


macro_rules! sensor_functions
{
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
                    wrapped_data_type: $wrapped_data_type:expr,
                    data_type: $data_type:ident,
                }
            ),* $(,)?
        }
    ) => {

    }
}


macro_rules! sensor_unit_functions {
    (
        SensoryCorticalUnit {
            $(
                $(#[doc = $doc:expr])?
                $variant_name:ident => {
                    friendly_name: $friendly_name:expr,
                    snake_case_name: $snake_case_name:expr,
                    accepted_wrapped_io_data_type: $accepted_wrapped_io_data_type:expr,
                    cortical_id_unit_reference: $cortical_id_unit_reference:expr,
                    number_cortical_areas: $number_cortical_areas:expr,
                    cortical_type_parameters: {
                        $($param_name:ident: $param_type:ty),* $(,)?
                    },
                    cortical_area_types: {
                        $(($cortical_area_type_expr:expr, $area_index:expr)),* $(,)?
                    }
                }
            ),* $(,)?
        }
    ) => {

        // Arm for WrappedIOType::Percentage
        (@generate_function
            WrappedIOType::Percentage


        )



    };
}

pub(crate) struct SensorDeviceCache {
    stream_caches: HashMap<(SensoryCorticalUnit, CorticalGroupIndex), SensoryChannelStreamCaches>,
    agent_device_key_lookup: HashMap<AgentDeviceIndex, Vec<(SensoryCorticalUnit, CorticalGroupIndex)>>,
    neuron_data: CorticalMappedXYZPNeuronVoxels,
    byte_data: FeagiByteContainer,
    previous_burst: Instant,
    neurons_encoded_signal: FeagiSignal<CorticalMappedXYZPNeuronVoxels>,
    bytes_encoded_signal: FeagiSignal<FeagiByteContainer>,
}

impl SensorDeviceCache {

    pub fn new() -> Self {
        SensorDeviceCache {
            stream_caches: HashMap::new(),
            agent_device_key_lookup: HashMap::new(),
            neuron_data: CorticalMappedXYZPNeuronVoxels::new(),
            byte_data: FeagiByteContainer::new_empty(),
            previous_burst: Instant::now(),
            neurons_encoded_signal: FeagiSignal::new(),
            bytes_encoded_signal: FeagiSignal::new(),
        }
    }

    //region Callbacks



    //endregion
    
    //region Devices

    /*

    sensor_definition!(sensor_functions);

    //region Segmented Vision

    /// Registers a new segmented vision sensor group with absolute gaze positioning. Sets up a processing pipeline that extracts regions of interest from full-resolution images based on gaze properties.
    pub fn sensor_segmented_vision_absolute_try_register(&mut self, group: CorticalGroupIndex, number_channels: CorticalChannelCount, input_image_properties: ImageFrameProperties, segmented_image_properties: SegmentedImageFrameProperties, initial_gaze: GazeProperties) -> Result<(), FeagiDataError> {

        let cortical_ids = SegmentedImageFrame::create_ordered_cortical_ids_for_segmented_vision(group, false);
        let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send > = SegmentedImageFrameNeuronVoxelXYZPEncoder::new_box(cortical_ids, segmented_image_properties, number_channels)?;

        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        let initial_val: WrappedIOData = WrappedIOType::SegmentedImageFrame(Some(segmented_image_properties)).create_blank_data_of_type()?;
        let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
            let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
            for _i in 0..*number_channels {
                output.push( vec![ImageSegmentorStageProperties::new_box(input_image_properties, segmented_image_properties, initial_gaze)?]) // TODO properly implement clone so we dont need to do this
            };
            output
        };
        self.register(SENSOR_TYPE, group, encoder, default_pipeline, initial_val)?;
        Ok(())
    }

    /// Writes raw image data to a specific segmented vision sensor channel for processing.
    pub fn sensor_segmented_vision_absolute_try_write(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex, data: WrappedIOData) -> Result<(), FeagiDataError> {
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        self.try_update_value(SENSOR_TYPE, group, channel, data, Instant::now())?;
        Ok(())
    }

    /// Reads the post-processed segmented image frame after pipeline processing.
    pub fn sensor_segmented_vision_absolute_try_read_postprocessed_cache_value(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex,) -> Result<SegmentedImageFrame, FeagiDataError> {
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        let wrapped_segmented_frame = self.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
        Ok(wrapped_segmented_frame.try_into()?)
    }

    /// Retrieves the properties of a single processing stage in the pipeline.
    pub fn sensor_segmented_vision_absolute_try_get_single_stage_properties(&mut self, group: CorticalGroupIndex, channel_index: CorticalChannelIndex, stage_index: PipelineStagePropertyIndex) -> Result<Box<dyn PipelineStageProperties + Sync + Send>, FeagiDataError> {
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        let properties = self.try_get_single_stage_properties(SENSOR_TYPE, group, channel_index, stage_index)?;
        Ok(properties)
    }

    /// Retrieves the properties of all processing stages in the pipeline.
    pub fn sensor_segmented_vision_absolute_try_get_all_stage_properties(&mut self, group: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<Vec<Box<dyn PipelineStageProperties + Sync + Send>>, FeagiDataError> {
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        let properties = self.try_get_all_stage_properties(SENSOR_TYPE, group, channel_index)?;
        Ok(properties)
    }

    /// Updates the properties of a single processing stage without changing the stage type.
    pub fn sensor_segmented_vision_absolute_try_update_single_stage_properties(&mut self, group: CorticalGroupIndex, channel_index: CorticalChannelIndex, pipeline_stage_property_index: PipelineStagePropertyIndex, updating_property: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<() , FeagiDataError> {
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        self.try_update_single_stage_properties(SENSOR_TYPE, group, channel_index, pipeline_stage_property_index, updating_property)?;
        Ok(())
    }

    /// Updates the properties of all processing stages while preserving pipeline structure and stage types.
    pub fn sensor_segmented_vision_absolute_try_update_all_stage_properties(&mut self, group: CorticalGroupIndex, channel_index: CorticalChannelIndex, updated_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<() , FeagiDataError> {
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        self.try_update_all_stage_properties(SENSOR_TYPE, group, channel_index, updated_pipeline_stage_properties)?;
        Ok(())
    }

    /// Replaces a single processing stage, allowing a different stage type to be used.
    pub fn sensor_segmented_vision_absolute_try_replace_single_stage(&mut self, group: CorticalGroupIndex, channel_index: CorticalChannelIndex, pipeline_stage_property_index: PipelineStagePropertyIndex, updating_property: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<() , FeagiDataError> {
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        self.try_replace_single_stage(SENSOR_TYPE, group, channel_index, pipeline_stage_property_index, updating_property)?;
        Ok(())
    }

    /// Replaces the entire processing pipeline, allowing changes to the number, types, and order of stages.
    pub fn sensor_segmented_vision_absolute_try_replace_all_stages(&mut self, group: CorticalGroupIndex, channel_index: CorticalChannelIndex, updated_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<() , FeagiDataError> {
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        self.try_replace_all_stages(SENSOR_TYPE, group, channel_index, updated_pipeline_stage_properties)?;
        Ok(())
    }


    //endregion

     */

    pub fn segmented_vision_register(&mut self, group: CorticalGroupIndex, number_channels: CorticalChannelCount, input_image_properties: ImageFrameProperties, segmented_image_properties: SegmentedImageFrameProperties, initial_gaze: GazeProperties, frame_change_handling: FrameChangeHandling) -> Result<(), FeagiDataError> {
        let cortical_ids = SensoryCorticalUnit::get_segmented_vision_cortical_ids_array(frame_change_handling, group);
        let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send > = SegmentedImageFrameNeuronVoxelXYZPEncoder::new_box(cortical_ids, segmented_image_properties, number_channels)?;

        let initial_val: WrappedIOData = WrappedIOType::SegmentedImageFrame(Some(segmented_image_properties)).create_blank_data_of_type()?;
        let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
            let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
            for _i in 0..*number_channels {
                output.push( vec![ImageSegmentorStageProperties::new_box(input_image_properties, segmented_image_properties, initial_gaze)?]) // TODO properly implement clone so we dont need to do this
            };
            output
        };
        self.register(SensoryCorticalUnit::SegmentedVision, group, encoder, default_pipeline, initial_val)?;
        Ok(())
    }

    pub fn sensor_segmented_vision_try_write(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex, data: WrappedIOData) -> Result<(), FeagiDataError> {
        const SENSOR_TYPE: SensoryCorticalUnit = SensoryCorticalUnit::SegmentedVision;
        self.try_update_value(SENSOR_TYPE, group, channel, data, Instant::now())?;
        Ok(())
    }

    //endregion

    //region Internal

    //region Cache Abstractions

    fn register(&mut self, sensor_type: SensoryCorticalUnit, group_index: CorticalGroupIndex,
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

    fn try_update_value(&mut self, sensor_type: SensoryCorticalUnit, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex, value: WrappedIOData, time_of_update: Instant) -> Result<(), FeagiDataError> {
        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches_mut(sensor_type, group_index)?;
        sensor_stream_caches.try_replace_input_channel_cache_value_and_run_pipeline(channel_index, value, time_of_update)?; // Handles checking channel, value type
        Ok(())
    }

    fn try_read_preprocessed_cached_value(&self, sensor_type: SensoryCorticalUnit, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<&WrappedIOData, FeagiDataError> {
        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches(sensor_type, group_index)?;
        let value = sensor_stream_caches.try_get_channel_recent_preprocessed_value(channel_index)?;
        Ok(value)
    }

    fn try_read_postprocessed_cached_value(&self, sensor_type: SensoryCorticalUnit, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<&WrappedIOData, FeagiDataError> {
        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches(sensor_type, group_index)?;
        let value = sensor_stream_caches.try_get_channel_recent_postprocessed_value(channel_index)?;
        Ok(value)
    }

    //endregion

    //region Stages

    fn try_get_single_stage_properties(&self, sensor_type: SensoryCorticalUnit,
                                       group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex,
                                       pipeline_stage_property_index: PipelineStagePropertyIndex) -> Result<Box<dyn PipelineStageProperties+Send+Sync>, FeagiDataError> {
        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches(sensor_type, group_index)?;
        sensor_stream_caches.try_get_single_stage_properties(channel_index, pipeline_stage_property_index)
    }

    fn try_get_all_stage_properties(&self, sensor_type: SensoryCorticalUnit, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<Vec<Box<dyn PipelineStageProperties + Sync + Send>>, FeagiDataError> {
        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches(sensor_type, group_index)?;
        sensor_stream_caches.get_all_stage_properties(channel_index)
    }

    fn try_update_single_stage_properties(&mut self, sensor_type: SensoryCorticalUnit, group_index: CorticalGroupIndex,
                                          channel_index: CorticalChannelIndex, pipeline_stage_property_index: PipelineStagePropertyIndex,
                                          replacing_property: Box<dyn PipelineStageProperties + Sync + Send>)
                                          -> Result<(), FeagiDataError> {

        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches_mut(sensor_type, group_index)?;
        sensor_stream_caches.try_update_single_stage_properties(channel_index, pipeline_stage_property_index, replacing_property)
    }

    fn try_update_all_stage_properties(&mut self, sensor_type: SensoryCorticalUnit, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex, new_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<(), FeagiDataError> {
        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches_mut(sensor_type, group_index)?;
        sensor_stream_caches.try_update_all_stage_properties(channel_index, new_pipeline_stage_properties)
    }

    fn try_replace_single_stage(&mut self, sensor_type: SensoryCorticalUnit, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex, replacing_at_index: PipelineStagePropertyIndex, new_pipeline_stage_properties: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<(), FeagiDataError> {
        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches_mut(sensor_type, group_index)?;
        sensor_stream_caches.try_replace_single_stage(channel_index, replacing_at_index, new_pipeline_stage_properties)
    }

    fn try_replace_all_stages(&mut self, sensor_type: SensoryCorticalUnit, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex, new_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<(), FeagiDataError> {
        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches_mut(sensor_type, group_index)?;
        sensor_stream_caches.try_replace_all_stages(channel_index, new_pipeline_stage_properties)
    }

    fn try_removing_all_stages(&mut self, sensor_type: SensoryCorticalUnit, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<(), FeagiDataError> {
        let sensor_stream_caches = self.try_get_sensory_channel_stream_caches_mut(sensor_type, group_index)?;
        sensor_stream_caches.try_removing_all_stages(channel_index)?;
        Ok(())
    }

    //endregion

    //region Agent Device

    fn register_agent_device_key(&mut self, agent_device_index: AgentDeviceIndex, sensor_type: SensoryCorticalUnit, group_index: CorticalGroupIndex) -> Result<(), FeagiDataError> {
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

    fn try_get_sensory_channel_stream_caches(&self, sensor_type: SensoryCorticalUnit, group_index: CorticalGroupIndex) -> Result<&SensoryChannelStreamCaches, FeagiDataError> {
        let check = self.stream_caches.get(&(sensor_type, group_index));
        if check.is_none() {
            return Err(FeagiDataError::BadParameters(format!("Unable to find {} of cortical group index {} in registered sensor's list!", sensor_type, group_index)))
        }
        let check = check.unwrap();
        Ok(check)
    }

    fn try_get_sensory_channel_stream_caches_mut(&mut self, sensor_type: SensoryCorticalUnit, group_index: CorticalGroupIndex) -> Result<&mut SensoryChannelStreamCaches, FeagiDataError> {
        let check = self.stream_caches.get_mut(&(sensor_type, group_index));
        if check.is_none() {
            return Err(FeagiDataError::BadParameters(format!("Unable to find {} of cortical group index {} in registered sensor's list!", sensor_type, group_index)))
        }
        let check = check.unwrap();
        Ok(check)
    }

    fn try_get_agent_device_lookup(&self, agent_device_index: AgentDeviceIndex) -> Result<&[(SensoryCorticalUnit, CorticalGroupIndex)], FeagiDataError> {
        let val = self.agent_device_key_lookup.get(&agent_device_index).ok_or(
            FeagiDataError::BadParameters(format!("No registered sensor device found in agent's list for agent index {}!", *agent_device_index))
        )?;
        Ok(val)
    }

    fn try_get_agent_device_lookup_mut(&mut self, agent_device_index: AgentDeviceIndex) -> Result<&mut Vec<(SensoryCorticalUnit, CorticalGroupIndex)>, FeagiDataError> {
        let val = self.agent_device_key_lookup.get_mut(&agent_device_index).ok_or(
            FeagiDataError::BadParameters(format!("No registered sensor device found in agent's list for agent index {}!", *agent_device_index))
        )?;
        Ok(val)
    }


    //endregion


    //endregion




}