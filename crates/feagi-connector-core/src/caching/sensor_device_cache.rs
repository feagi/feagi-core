use crate::data_pipeline::per_channel_stream_caches::SensoryChannelStreamCaches;
use crate::data_pipeline::{PipelineStageProperties, PipelineStagePropertyIndex};
use crate::data_types::descriptors::*;
use crate::data_types::*;
use crate::neuron_voxel_coding::xyzp::encoders::*;
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPEncoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};
use feagi_data_serialization::FeagiByteContainer;
use feagi_data_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelIndex, CorticalGroupIndex, NeuronDepth,
};
use feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::{
    FrameChangeHandling, PercentageNeuronPositioning,
};
use feagi_data_structures::genomic::cortical_area::CorticalID;
use feagi_data_structures::genomic::descriptors::AgentDeviceIndex;
use feagi_data_structures::genomic::SensoryCorticalUnit;
use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use feagi_data_structures::{sensor_cortical_units, FeagiDataError, FeagiSignal};
use std::collections::HashMap;
use std::fmt;
use std::time::Instant;

macro_rules! sensor_unit_functions {
    (
        SensoryCorticalUnit {
            $(
                $(#[doc = $doc:expr])?
                $cortical_type_key_name:ident => {
                    friendly_name: $friendly_name:expr,
                    snake_case_name: $snake_case_name:expr,
                    accepted_wrapped_io_data_type: $accepted_wrapped_io_data_type:ident,
                    cortical_id_unit_reference: $cortical_id_unit_reference:expr,
                    number_cortical_areas: $number_cortical_areas:expr,
                    cortical_type_parameters: {
                        $($param_name:ident: $param_type:ty),* $(,)?
                    },
                    cortical_area_properties: {
                        $($area_index:tt => ($cortical_area_type_expr:expr, relative_position: [$rel_x:expr, $rel_y:expr, $rel_z:expr], channel_dimensions_default: [$dim_default_x:expr, $dim_default_y:expr, $dim_default_z:expr], channel_dimensions_min: [$dim_min_x:expr, $dim_min_y:expr, $dim_min_z:expr], channel_dimensions_max: [$dim_max_x:expr, $dim_max_y:expr, $dim_max_z:expr])),* $(,)?
                    }
                }
            ),* $(,)?
        }
    ) =>
    {
        $(
            sensor_unit_functions!(@generate_functions
            $cortical_type_key_name,
            $snake_case_name,
            $accepted_wrapped_io_data_type
            );
        )*
    };

    //region Similar Functions
    // Helper macro to generate stage and other similar functions
    (@generate_similar_functions
        $cortical_type_key_name:ident,
        $snake_case_name:expr,
        $wrapped_data_type:ident
    ) => {
        ::paste::paste! {

            pub fn [<$snake_case_name _write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: WrappedIOData,
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensoryCorticalUnit = SensoryCorticalUnit::$cortical_type_key_name;
                let instant = Instant::now();

                self.try_update_value(SENSOR_TYPE, group, channel, data, instant)?;
                Ok(())
            }

            pub fn [<$snake_case_name _read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $wrapped_data_type, FeagiDataError> {

                const SENSOR_TYPE: SensoryCorticalUnit = SensoryCorticalUnit::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $wrapped_data_type = wrapped.try_into()?;
                Ok(val)
            }

            pub fn [<$snake_case_name _get_single_stage_properties>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
                stage_index: PipelineStagePropertyIndex
            ) -> Result<PipelineStageProperties, FeagiDataError>
            {
                const SENSOR_UNIT_TYPE: SensoryCorticalUnit = SensoryCorticalUnit::$cortical_type_key_name;
                let stage = self.try_get_single_stage_properties(SENSOR_UNIT_TYPE, group, channel_index, stage_index)?;
                Ok(stage)
            }

            pub fn [<$snake_case_name _get_all_stage_properties>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Vec<PipelineStageProperties>, FeagiDataError>
            {
                const SENSOR_UNIT_TYPE: SensoryCorticalUnit = SensoryCorticalUnit::$cortical_type_key_name;
                let stages = self.try_get_all_stage_properties(SENSOR_UNIT_TYPE, group, channel_index)?;
                Ok(stages)
            }

            pub fn [<$snake_case_name _update_single_stage_properties>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
                pipeline_stage_property_index: PipelineStagePropertyIndex,
                updating_property: PipelineStageProperties
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_UNIT_TYPE: SensoryCorticalUnit = SensoryCorticalUnit::$cortical_type_key_name;
                self.try_update_single_stage_properties(SENSOR_UNIT_TYPE, group, channel_index, pipeline_stage_property_index, updating_property)?;
                Ok(())
            }

            pub fn [<$snake_case_name _update_all_stage_properties>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
                updated_pipeline_stage_properties: Vec<PipelineStageProperties>
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_UNIT_TYPE: SensoryCorticalUnit = SensoryCorticalUnit::$cortical_type_key_name;
                self.try_update_all_stage_properties(SENSOR_UNIT_TYPE, group, channel_index, updated_pipeline_stage_properties)?;
                Ok(())
            }

            pub fn [<$snake_case_name _replace_single_stage>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
                pipeline_stage_property_index: PipelineStagePropertyIndex,
                replacing_property: PipelineStageProperties
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_UNIT_TYPE: SensoryCorticalUnit = SensoryCorticalUnit::$cortical_type_key_name;
                self.try_replace_single_stage(SENSOR_UNIT_TYPE, group, channel_index, pipeline_stage_property_index, replacing_property)?;
                Ok(())
            }

            pub fn [<$snake_case_name _replace_all_stages>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
                new_pipeline_stage_properties: Vec<PipelineStageProperties>
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_UNIT_TYPE: SensoryCorticalUnit = SensoryCorticalUnit::$cortical_type_key_name;
                self.try_replace_all_stages(SENSOR_UNIT_TYPE, group, channel_index, new_pipeline_stage_properties)?;
                Ok(())
            }

            pub fn [<$snake_case_name _removing_all_stages>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_UNIT_TYPE: SensoryCorticalUnit = SensoryCorticalUnit::$cortical_type_key_name;
                self.try_removing_all_stages(SENSOR_UNIT_TYPE, group, channel_index)?;
                Ok(())
            }
        }
    };
    //endregion


    // Arm for WrappedIOType::Boolean
    (@generate_functions
        $sensory_unit:ident,
        $snake_case_name:expr,
        Boolean
    ) => {
        ::paste::paste! {
            pub fn [<$snake_case_name _register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                ) -> Result<(), FeagiDataError>
            {
                let cortical_id: CorticalID = SensoryCorticalUnit::[<get_cortical_ids_array_for_ $snake_case_name >](group)[0];
                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = BooleanNeuronVoxelXYZPEncoder::new_box(cortical_id, number_channels)?;

                let initial_val: WrappedIOData = false.into();
                self.register(SensoryCorticalUnit::$sensory_unit, group, encoder, number_channels, initial_val)?;
                Ok(())
            }
        }

        sensor_unit_functions!(@generate_similar_functions $sensory_unit, $snake_case_name, bool);
    };

    // Arm for WrappedIOType::Percentage
    (@generate_functions
        $sensory_unit:ident,
        $snake_case_name:expr,
        Percentage
    ) => {
        ::paste::paste! {
            pub fn [<$snake_case_name _register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                frame_change_handling: FrameChangeHandling,
                z_neuron_resolution: NeuronDepth,
                percentage_neuron_positioning: PercentageNeuronPositioning
                ) -> Result<(), FeagiDataError>
            {
                let cortical_id: CorticalID = SensoryCorticalUnit::[<get_cortical_ids_array_for_ $snake_case_name >](frame_change_handling, percentage_neuron_positioning, group)[0];
                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = {
                    match percentage_neuron_positioning { // TODO fix naming of exponential / fractional
                        PercentageNeuronPositioning::Linear => PercentageLinearNeuronVoxelXYZPEncoder::new_box(cortical_id, z_neuron_resolution, number_channels)?,
                        PercentageNeuronPositioning::Fractional => PercentageExponentialNeuronVoxelXYZPEncoder::new_box(cortical_id, z_neuron_resolution, number_channels)?,
                    }
                };

                let initial_val: WrappedIOData = WrappedIOData::Percentage(Percentage::new_zero());
                self.register(SensoryCorticalUnit::$sensory_unit, group, encoder, number_channels, initial_val)?;
                Ok(())
            }
        }

        sensor_unit_functions!(@generate_similar_functions $sensory_unit, $snake_case_name, Percentage);
    };

    // Arm for WrappedIOType::Percentage_3D
    (@generate_functions
        $sensory_unit:ident,
        $snake_case_name:expr,
        Percentage_3D
    ) => {
        ::paste::paste! {
            pub fn [<$snake_case_name _register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                frame_change_handling: FrameChangeHandling,
                z_neuron_resolution: NeuronDepth,
                percentage_neuron_positioning: PercentageNeuronPositioning
                ) -> Result<(), FeagiDataError>
            {
                let cortical_id: CorticalID = SensoryCorticalUnit::[<get_cortical_ids_array_for_ $snake_case_name >](frame_change_handling, percentage_neuron_positioning, group)[0];
                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = {
                    match percentage_neuron_positioning { // TODO fix naming of exponential / fractional
                        PercentageNeuronPositioning::Linear => Percentage3DLinearNeuronVoxelXYZPEncoder::new_box(cortical_id, z_neuron_resolution, number_channels)?,
                        PercentageNeuronPositioning::Fractional => Percentage3DExponentialNeuronVoxelXYZPEncoder::new_box(cortical_id, z_neuron_resolution, number_channels)?,
                    }
                };

                let initial_val: WrappedIOData = WrappedIOData::Percentage(Percentage::new_zero());
                self.register(SensoryCorticalUnit::$sensory_unit, group, encoder, number_channels, initial_val)?;
                Ok(())
            }
        }

        sensor_unit_functions!(@generate_similar_functions $sensory_unit, $snake_case_name, Percentage3D);
    };

    // Arm for WrappedIOType::SignedPercentage_4D
    (@generate_functions
        $sensory_unit:ident,
        $snake_case_name:expr,
        SignedPercentage_4D
    ) => {
        ::paste::paste! {
            pub fn [<$snake_case_name _register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                frame_change_handling: FrameChangeHandling,
                z_neuron_resolution: NeuronDepth,
                percentage_neuron_positioning: PercentageNeuronPositioning
                ) -> Result<(), FeagiDataError>
            {
                let cortical_id: CorticalID = SensoryCorticalUnit::[<get_cortical_ids_array_for_ $snake_case_name >](frame_change_handling, percentage_neuron_positioning, group)[0];
                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = {
                    match percentage_neuron_positioning { // TODO fix naming of exponential / fractional
                        PercentageNeuronPositioning::Linear => SignedPercentage4DLinearNeuronVoxelXYZPEncoder::new_box(cortical_id, z_neuron_resolution, number_channels)?,
                        PercentageNeuronPositioning::Fractional => SignedPercentage4DExponentialNeuronVoxelXYZPEncoder::new_box(cortical_id, z_neuron_resolution, number_channels)?,
                    }
                };

                let initial_val: WrappedIOData = WrappedIOData::Percentage(Percentage::new_zero());
                self.register(SensoryCorticalUnit::$sensory_unit, group, encoder, number_channels, initial_val)?;
                Ok(())
            }
        }

        sensor_unit_functions!(@generate_similar_functions $sensory_unit, $snake_case_name, SignedPercentage4D);
    };

    // Arm for WrappedIOType::SegmentedImageFrame
    (@generate_functions
        $sensory_unit:ident,
        $snake_case_name:expr,
        SegmentedImageFrame
    ) => {
        ::paste::paste! {
            pub fn [<$snake_case_name _register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                frame_change_handling: FrameChangeHandling,
                input_image_properties: ImageFrameProperties,
                segmented_image_properties: SegmentedImageFrameProperties,
                 initial_gaze: GazeProperties
                ) -> Result<(), FeagiDataError>
            {
                // Bit more unique, we define a custom stage for all channels for segmentation by default
                let cortical_ids: [CorticalID; 9] = SensoryCorticalUnit::[<get_cortical_ids_array_for_ $snake_case_name >](frame_change_handling, group);
                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SegmentedImageFrameNeuronVoxelXYZPEncoder::new_box(cortical_ids, segmented_image_properties, number_channels)?;

                let initial_val: WrappedIOData = WrappedIOType::SegmentedImageFrame(Some(segmented_image_properties)).create_blank_data_of_type()?;
                self.register(SensoryCorticalUnit::$sensory_unit, group, encoder, number_channels, initial_val)?;

                let stage_properties = PipelineStageProperties::new_image_frame_segmentator(input_image_properties.clone(), segmented_image_properties.clone(), initial_gaze.clone());

                for channel_index in 0..*number_channels {
                    let segmentator_pipeline = vec![stage_properties.clone()];
                    self.[<$snake_case_name _replace_all_stages>](group, channel_index.into(), segmentator_pipeline);
                }
                Ok(())
            }
        }

        sensor_unit_functions!(@generate_similar_functions $sensory_unit, $snake_case_name, SegmentedImageFrame);
    };

    // Arm for WrappedIOType::MiscData
    (@generate_functions
        $sensory_unit:ident,
        $snake_case_name:expr,
        MiscData
    ) => {
        ::paste::paste! {
            pub fn [<$snake_case_name _register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                frame_change_handling: FrameChangeHandling,
                misc_data_dimensions: MiscDataDimensions
                ) -> Result<(), FeagiDataError>
            {
                let cortical_id: CorticalID = SensoryCorticalUnit::[<get_cortical_ids_array_for_ $snake_case_name >](frame_change_handling, group)[0];
                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = MiscDataNeuronVoxelXYZPEncoder::new_box(cortical_id, misc_data_dimensions, number_channels)?;

                let initial_val: WrappedIOData = WrappedIOType::MiscData(Some(misc_data_dimensions)).create_blank_data_of_type()?;
                self.register(SensoryCorticalUnit::$sensory_unit, group, encoder, number_channels, initial_val)?;
                Ok(())
            }
        }

        sensor_unit_functions!(@generate_similar_functions $sensory_unit, $snake_case_name, MiscData);
    };


    // Arm for WrappedIOType::ImageFrame
    (@generate_functions
        $sensory_unit:ident,
        $snake_case_name:expr,
        ImageFrame
    ) => {
        ::paste::paste! {
            pub fn [<$snake_case_name _register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                frame_change_handling: FrameChangeHandling,
                image_properties: ImageFrameProperties,
                ) -> Result<(), FeagiDataError>
            {
                let cortical_id: CorticalID = SensoryCorticalUnit::[<get_cortical_ids_array_for_ $snake_case_name >](frame_change_handling, group)[0];
                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = CartesianPlaneNeuronVoxelXYZPEncoder::new_box(cortical_id, &image_properties, number_channels)?;

                let initial_val: WrappedIOData = WrappedIOType::ImageFrame(Some(image_properties)).create_blank_data_of_type()?;
                self.register(SensoryCorticalUnit::$sensory_unit, group, encoder, number_channels, initial_val)?;
                Ok(())
            }
        }

        sensor_unit_functions!(@generate_similar_functions $sensory_unit, $snake_case_name, ImageFrame);
    };
}

pub struct SensorDeviceCache {
    stream_caches: HashMap<(SensoryCorticalUnit, CorticalGroupIndex), SensoryChannelStreamCaches>,
    agent_device_key_lookup:
        HashMap<AgentDeviceIndex, Vec<(SensoryCorticalUnit, CorticalGroupIndex)>>,
    neuron_data: CorticalMappedXYZPNeuronVoxels,
    byte_data: FeagiByteContainer,
    previous_burst: Instant,
    neurons_encoded_signal: FeagiSignal<CorticalMappedXYZPNeuronVoxels>,
    bytes_encoded_signal: FeagiSignal<FeagiByteContainer>,
}

impl std::fmt::Debug for SensorDeviceCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SensorDeviceCache")
            .field("stream_caches_count", &self.stream_caches.len())
            .field(
                "agent_device_key_lookup_count",
                &self.agent_device_key_lookup.len(),
            )
            .field("neuron_data", &self.neuron_data)
            .field("byte_data", &self.byte_data)
            .field("previous_burst", &self.previous_burst)
            .finish()
    }
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

    sensor_cortical_units!(sensor_unit_functions);

    //region Data IO

    pub fn get_feagi_byte_container(&self) -> &FeagiByteContainer {
        &self.byte_data
    }

    pub fn get_feagi_byte_container_mut(&mut self) -> &mut FeagiByteContainer {
        &mut self.byte_data
    }

    pub fn get_neurons(&self) -> &CorticalMappedXYZPNeuronVoxels {
        &self.neuron_data
    }

    /// Encode all cached sensor data to neuron voxel format
    ///
    /// Iterates over all registered sensor stream caches and encodes their
    /// processed data into neuron voxel representations. This populates
    /// the internal neuron_data field.
    ///
    /// # Arguments
    /// * `time_of_burst` - Timestamp for this encoding burst
    ///
    /// # Returns
    /// * `Ok(())` - If encoding succeeded
    /// * `Err(FeagiDataError)` - If encoding fails
    pub fn encode_all_sensors_to_neurons(
        &mut self,
        time_of_burst: Instant,
    ) -> Result<(), FeagiDataError> {
        // Clear neuron data before encoding
        self.neuron_data.clear_neurons_only();

        let previous_burst = self.previous_burst;

        // TODO see if we can parallelize this to work on multiple cortical areas at once
        // Iterate over all registered sensor stream caches and encode them
        // CRITICAL: Pass previous_burst (not time_of_burst) so encoder can check if channels were updated since last encoding
        for ((_sensor_type, _group_index), stream_cache) in self.stream_caches.iter_mut() {
            stream_cache.update_neuron_data_with_recently_updated_cached_sensor_data(
                &mut self.neuron_data,
                previous_burst,
            )?;
        }

        // Update previous_burst for next time
        self.previous_burst = time_of_burst;

        Ok(())
    }

    /// Encode neuron voxel data to byte container format
    ///
    /// Serializes the internal neuron_data into FeagiByteContainer format.
    /// This populates the internal byte_data field.
    ///
    /// # Returns
    /// * `Ok(())` - If encoding succeeded
    /// * `Err(FeagiDataError)` - If encoding fails
    pub fn encode_neurons_to_bytes(&mut self) -> Result<(), FeagiDataError> {
        self.byte_data
            .overwrite_byte_data_with_single_struct_data(&self.neuron_data, 0)
            .map_err(|e| {
                FeagiDataError::BadParameters(format!(
                    "Failed to encode neuron data to bytes: {:?}",
                    e
                ))
            })?;
        Ok(())
    }

    pub fn export_registered_sensors_as_config_json(
        &self,
    ) -> Result<serde_json::Value, FeagiDataError> {
        let mut output = serde_json::Map::new();
        for ((sensor_cortical_unit, cortical_group_index), sensor_channel_stream_caches) in
            &self.stream_caches
        {
            let motor_unit_name = sensor_cortical_unit.get_snake_case_name().to_string();
            let cortical_group_name = cortical_group_index.to_string();

            let sensor_units_map = output
                .entry(motor_unit_name)
                .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()))
                .as_object_mut()
                .expect("Just inserted an Object");

            sensor_units_map.insert(
                cortical_group_name,
                sensor_channel_stream_caches.export_as_json()?,
            );
        }
        Ok(serde_json::Value::Object(output))
    }

    /// Import sensor configurations from JSON
    ///
    /// Updates pipeline stages and friendly names for already-registered sensors.
    /// Sensors must be registered first using the appropriate register functions.
    ///
    /// # Arguments
    /// * `json` - JSON object containing sensor configurations in new format
    ///
    /// # Returns
    /// * `Ok(())` - If import succeeded
    /// * `Err(FeagiDataError)` - If sensor not registered or JSON is malformed
    pub fn import_sensors_from_json(
        &mut self,
        json: &serde_json::Value,
    ) -> Result<(), FeagiDataError> {
        let input_map = json.as_object().ok_or_else(|| {
            FeagiDataError::DeserializationError("Expected input object for sensors".to_string())
        })?;

        for (sensor_type_name, groups) in input_map {
            // Parse sensor type from snake_case name
            let sensor_type = SensoryCorticalUnit::from_snake_case_name(sensor_type_name)
                .ok_or_else(|| {
                    FeagiDataError::DeserializationError(format!(
                        "Unknown sensor type: {}",
                        sensor_type_name
                    ))
                })?;

            let groups_map = groups.as_object().ok_or_else(|| {
                FeagiDataError::DeserializationError(format!(
                    "Expected groups object for sensor type: {}",
                    sensor_type_name
                ))
            })?;

            for (group_id_str, device_config) in groups_map {
                let group_id: CorticalGroupIndex = group_id_str
                    .parse::<u8>()
                    .map_err(|e| {
                        FeagiDataError::DeserializationError(format!(
                            "Invalid group ID '{}': {}",
                            group_id_str, e
                        ))
                    })?
                    .into();

                // Get the stream cache for this sensor type + group
                let stream_cache = self.stream_caches.get_mut(&(sensor_type, group_id))
                    .ok_or_else(|| FeagiDataError::BadParameters(
                        format!("Sensor {}:{} not registered. Register the sensor first before importing configuration.",
                            sensor_type_name, group_id_str)
                    ))?;

                // Import configuration (pipelines, friendly names)
                stream_cache.import_from_json(device_config)?;
            }
        }
        Ok(())
    }

    //endregion

    //region Internal

    //region Cache Abstractions

    fn register(
        &mut self,
        sensor_type: SensoryCorticalUnit,
        group_index: CorticalGroupIndex,
        neuron_encoder: Box<dyn NeuronVoxelXYZPEncoder>,
        number_channels: CorticalChannelCount,
        initial_cached_value: WrappedIOData,
    ) -> Result<(), FeagiDataError> {
        if self.stream_caches.contains_key(&(sensor_type, group_index)) {
            return Err(FeagiDataError::BadParameters(format!(
                "Already registered sensor {} of group index {}!",
                sensor_type, group_index
            )));
        }

        self.stream_caches.insert(
            (sensor_type, group_index),
            SensoryChannelStreamCaches::new(neuron_encoder, number_channels, initial_cached_value)?,
        );

        Ok(())
    }

    //region Data

    fn try_update_value(
        &mut self,
        sensor_type: SensoryCorticalUnit,
        group_index: CorticalGroupIndex,
        channel_index: CorticalChannelIndex,
        value: WrappedIOData,
        time_of_update: Instant,
    ) -> Result<(), FeagiDataError> {
        let sensor_stream_caches =
            self.try_get_sensory_channel_stream_caches_mut(sensor_type, group_index)?;
        sensor_stream_caches.try_replace_input_channel_cache_value_and_run_pipeline(
            channel_index,
            value,
            time_of_update,
        )?; // Handles checking channel, value type
        Ok(())
    }

    fn try_read_preprocessed_cached_value(
        &self,
        sensor_type: SensoryCorticalUnit,
        group_index: CorticalGroupIndex,
        channel_index: CorticalChannelIndex,
    ) -> Result<&WrappedIOData, FeagiDataError> {
        let sensor_stream_caches =
            self.try_get_sensory_channel_stream_caches(sensor_type, group_index)?;
        let value = sensor_stream_caches.try_get_channel_preprocessed_value(channel_index)?;
        Ok(value)
    }

    fn try_read_postprocessed_cached_value(
        &self,
        sensor_type: SensoryCorticalUnit,
        group_index: CorticalGroupIndex,
        channel_index: CorticalChannelIndex,
    ) -> Result<&WrappedIOData, FeagiDataError> {
        let sensor_stream_caches =
            self.try_get_sensory_channel_stream_caches(sensor_type, group_index)?;
        let value =
            sensor_stream_caches.try_get_channel_recent_postprocessed_value(channel_index)?;
        Ok(value)
    }

    //endregion

    //region Stages

    fn try_get_single_stage_properties(
        &self,
        sensor_type: SensoryCorticalUnit,
        group_index: CorticalGroupIndex,
        channel_index: CorticalChannelIndex,
        pipeline_stage_property_index: PipelineStagePropertyIndex,
    ) -> Result<PipelineStageProperties, FeagiDataError> {
        let sensor_stream_caches =
            self.try_get_sensory_channel_stream_caches(sensor_type, group_index)?;
        sensor_stream_caches
            .try_get_single_stage_properties(channel_index, pipeline_stage_property_index)
    }

    fn try_get_all_stage_properties(
        &self,
        sensor_type: SensoryCorticalUnit,
        group_index: CorticalGroupIndex,
        channel_index: CorticalChannelIndex,
    ) -> Result<Vec<PipelineStageProperties>, FeagiDataError> {
        let sensor_stream_caches =
            self.try_get_sensory_channel_stream_caches(sensor_type, group_index)?;
        sensor_stream_caches.get_all_stage_properties(channel_index)
    }

    fn try_update_single_stage_properties(
        &mut self,
        sensor_type: SensoryCorticalUnit,
        group_index: CorticalGroupIndex,
        channel_index: CorticalChannelIndex,
        pipeline_stage_property_index: PipelineStagePropertyIndex,
        replacing_property: PipelineStageProperties,
    ) -> Result<(), FeagiDataError> {
        let sensor_stream_caches =
            self.try_get_sensory_channel_stream_caches_mut(sensor_type, group_index)?;
        sensor_stream_caches.try_update_single_stage_properties(
            channel_index,
            pipeline_stage_property_index,
            replacing_property,
        )
    }

    fn try_update_all_stage_properties(
        &mut self,
        sensor_type: SensoryCorticalUnit,
        group_index: CorticalGroupIndex,
        channel_index: CorticalChannelIndex,
        new_pipeline_stage_properties: Vec<PipelineStageProperties>,
    ) -> Result<(), FeagiDataError> {
        let sensor_stream_caches =
            self.try_get_sensory_channel_stream_caches_mut(sensor_type, group_index)?;
        sensor_stream_caches
            .try_update_all_stage_properties(channel_index, new_pipeline_stage_properties)
    }

    fn try_replace_single_stage(
        &mut self,
        sensor_type: SensoryCorticalUnit,
        group_index: CorticalGroupIndex,
        channel_index: CorticalChannelIndex,
        replacing_at_index: PipelineStagePropertyIndex,
        new_pipeline_stage_properties: PipelineStageProperties,
    ) -> Result<(), FeagiDataError> {
        let sensor_stream_caches =
            self.try_get_sensory_channel_stream_caches_mut(sensor_type, group_index)?;
        sensor_stream_caches.try_replace_single_stage(
            channel_index,
            replacing_at_index,
            new_pipeline_stage_properties,
        )
    }

    fn try_replace_all_stages(
        &mut self,
        sensor_type: SensoryCorticalUnit,
        group_index: CorticalGroupIndex,
        channel_index: CorticalChannelIndex,
        new_pipeline_stage_properties: Vec<PipelineStageProperties>,
    ) -> Result<(), FeagiDataError> {
        let sensor_stream_caches =
            self.try_get_sensory_channel_stream_caches_mut(sensor_type, group_index)?;
        sensor_stream_caches.try_replace_all_stages(channel_index, new_pipeline_stage_properties)
    }

    fn try_removing_all_stages(
        &mut self,
        sensor_type: SensoryCorticalUnit,
        group_index: CorticalGroupIndex,
        channel_index: CorticalChannelIndex,
    ) -> Result<(), FeagiDataError> {
        let sensor_stream_caches =
            self.try_get_sensory_channel_stream_caches_mut(sensor_type, group_index)?;
        sensor_stream_caches.try_removing_all_stages(channel_index)?;
        Ok(())
    }

    //endregion

    //region Agent Device

    fn register_agent_device_key(
        &mut self,
        agent_device_index: AgentDeviceIndex,
        sensor_type: SensoryCorticalUnit,
        group_index: CorticalGroupIndex,
    ) -> Result<(), FeagiDataError> {
        let keys = {
            match self.agent_device_key_lookup.get_mut(&agent_device_index) {
                Some(keys) => keys,
                None => {
                    self.agent_device_key_lookup
                        .insert(agent_device_index, Vec::new());
                    self.agent_device_key_lookup
                        .get_mut(&agent_device_index)
                        .unwrap()
                }
            }
        };
        keys.push((sensor_type, group_index));
        Ok(())
    }

    fn try_update_value_by_agent_device(
        &mut self,
        agent_device_index: AgentDeviceIndex,
        channel_index: CorticalChannelIndex,
        value: WrappedIOData,
        time_of_update: Instant,
    ) -> Result<(), FeagiDataError> {
        let sensor_group_pairs = self
            .try_get_agent_device_lookup(agent_device_index)?
            .to_vec();
        for (sensor_type, group_index) in sensor_group_pairs {
            self.try_update_value(
                sensor_type,
                group_index,
                channel_index,
                value.clone(),
                time_of_update,
            )?;
        }
        Ok(())
    }

    fn try_read_postprocessed_cached_values_by_agent_device(
        &self,
        agent_device_index: AgentDeviceIndex,
        channel_index: CorticalChannelIndex,
    ) -> Result<Vec<&WrappedIOData>, FeagiDataError> {
        let sensor_group_pairs = self.try_get_agent_device_lookup(agent_device_index)?;
        let mut results = Vec::with_capacity(sensor_group_pairs.len());
        for (sensor_type, group_index) in sensor_group_pairs {
            let value = self.try_read_postprocessed_cached_value(
                *sensor_type,
                *group_index,
                channel_index,
            )?;
            results.push(value);
        }
        Ok(results)
    }

    //endregion

    //endregion

    //region Hashmap Interactions

    fn try_get_sensory_channel_stream_caches(
        &self,
        sensor_type: SensoryCorticalUnit,
        group_index: CorticalGroupIndex,
    ) -> Result<&SensoryChannelStreamCaches, FeagiDataError> {
        let check = self.stream_caches.get(&(sensor_type, group_index));
        if check.is_none() {
            return Err(FeagiDataError::BadParameters(format!(
                "Unable to find {} of cortical group index {} in registered sensor's list!",
                sensor_type, group_index
            )));
        }
        let check = check.unwrap();
        Ok(check)
    }

    fn try_get_sensory_channel_stream_caches_mut(
        &mut self,
        sensor_type: SensoryCorticalUnit,
        group_index: CorticalGroupIndex,
    ) -> Result<&mut SensoryChannelStreamCaches, FeagiDataError> {
        let check = self.stream_caches.get_mut(&(sensor_type, group_index));
        if check.is_none() {
            return Err(FeagiDataError::BadParameters(format!(
                "Unable to find {} of cortical group index {} in registered sensor's list!",
                sensor_type, group_index
            )));
        }
        let check = check.unwrap();
        Ok(check)
    }

    fn try_get_agent_device_lookup(
        &self,
        agent_device_index: AgentDeviceIndex,
    ) -> Result<&[(SensoryCorticalUnit, CorticalGroupIndex)], FeagiDataError> {
        let val = self
            .agent_device_key_lookup
            .get(&agent_device_index)
            .ok_or(FeagiDataError::BadParameters(format!(
                "No registered sensor device found in agent's list for agent index {}!",
                *agent_device_index
            )))?;
        Ok(val)
    }

    fn try_get_agent_device_lookup_mut(
        &mut self,
        agent_device_index: AgentDeviceIndex,
    ) -> Result<&mut Vec<(SensoryCorticalUnit, CorticalGroupIndex)>, FeagiDataError> {
        let val = self
            .agent_device_key_lookup
            .get_mut(&agent_device_index)
            .ok_or(FeagiDataError::BadParameters(format!(
                "No registered sensor device found in agent's list for agent index {}!",
                *agent_device_index
            )))?;
        Ok(val)
    }

    //endregion

    //endregion
}

impl fmt::Display for SensorDeviceCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(write!(f, "Motor Device Cache:\n")?)
    }
}
