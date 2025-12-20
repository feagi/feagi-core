use crate::data_pipeline::per_channel_stream_caches::MotorChannelStreamCaches;
use crate::data_pipeline::{PipelineStageProperties, PipelineStagePropertyIndex};
use crate::data_types::descriptors::*;
use crate::data_types::*;
use crate::neuron_voxel_coding::xyzp::decoders::*;
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPDecoder;
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
use feagi_data_structures::genomic::MotorCorticalUnit;
use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use feagi_data_structures::{motor_cortical_units, FeagiDataError, FeagiSignalIndex};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::time::Instant;

macro_rules! motor_unit_functions {
    (
        MotorCorticalUnit {
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
            motor_unit_functions!(@generate_functions
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


            pub fn [<$snake_case_name _read_preprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $wrapped_data_type, FeagiDataError> {

                const MOTOR_TYPE: MotorCorticalUnit = MotorCorticalUnit::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel)?;
                let val: $wrapped_data_type = wrapped.try_into()?;
                Ok(val)
            }

            pub fn [<$snake_case_name _read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $wrapped_data_type, FeagiDataError> {

                const MOTOR_TYPE: MotorCorticalUnit = MotorCorticalUnit::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel)?;
                let val: $wrapped_data_type = wrapped.try_into()?;
                Ok(val)
            }

            pub fn [<motor_ $snake_case_name _try_register_motor_callback>]<F>(
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
                callback: F
            ) -> Result<FeagiSignalIndex, FeagiDataError>
            where F: Fn(&WrappedIOData) + Send + Sync + 'static,
            {
                const MOTOR_TYPE: MotorCorticalUnit = MotorCorticalUnit::$cortical_type_key_name;
                let signal_index = self.try_register_motor_callback(MOTOR_TYPE, group, channel_index, callback)?;
                Ok(signal_index)
            }

            pub fn [<$snake_case_name _get_single_stage_properties>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
                stage_index: PipelineStagePropertyIndex
            ) -> Result<PipelineStageProperties, FeagiDataError>
            {
                const MOTOR_UNIT_TYPE: MotorCorticalUnit = MotorCorticalUnit::$cortical_type_key_name;
                let stage = self.try_get_single_stage_properties(MOTOR_UNIT_TYPE, group, channel_index, stage_index)?;
                Ok(stage)
            }

            pub fn [<$snake_case_name _get_all_stage_properties>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Vec<PipelineStageProperties>, FeagiDataError>
            {
                const MOTOR_UNIT_TYPE: MotorCorticalUnit = MotorCorticalUnit::$cortical_type_key_name;
                let stages = self.try_get_all_stage_properties(MOTOR_UNIT_TYPE, group, channel_index)?;
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
                const MOTOR_UNIT_TYPE: MotorCorticalUnit = MotorCorticalUnit::$cortical_type_key_name;
                self.try_update_single_stage_properties(MOTOR_UNIT_TYPE, group, channel_index, pipeline_stage_property_index, updating_property)?;
                Ok(())
            }

            pub fn [<$snake_case_name _update_all_stage_properties>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
                updated_pipeline_stage_properties: Vec<PipelineStageProperties>
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_UNIT_TYPE: MotorCorticalUnit = MotorCorticalUnit::$cortical_type_key_name;
                self.try_update_all_stage_properties(MOTOR_UNIT_TYPE, group, channel_index, updated_pipeline_stage_properties)?;
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
                const MOTOR_UNIT_TYPE: MotorCorticalUnit = MotorCorticalUnit::$cortical_type_key_name;
                self.try_replace_single_stage(MOTOR_UNIT_TYPE, group, channel_index, pipeline_stage_property_index, replacing_property)?;
                Ok(())
            }

            pub fn [<$snake_case_name _replace_all_stages>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
                new_pipeline_stage_properties: Vec<PipelineStageProperties>
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_UNIT_TYPE: MotorCorticalUnit = MotorCorticalUnit::$cortical_type_key_name;
                self.try_replace_all_stages(MOTOR_UNIT_TYPE, group, channel_index, new_pipeline_stage_properties)?;
                Ok(())
            }

            pub fn [<$snake_case_name _removing_all_stages>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_UNIT_TYPE: MotorCorticalUnit = MotorCorticalUnit::$cortical_type_key_name;
                self.try_removing_all_stages(MOTOR_UNIT_TYPE, group, channel_index)?;
                Ok(())
            }
        }
    };
    //endregion

    // Arm for WrappedIOType::GazeProperties
    (@generate_functions
        $motor_unit:ident,
        $snake_case_name:expr,
        GazeProperties
    ) => {
        ::paste::paste! {
            pub fn [<$snake_case_name _register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                frame_change_handling: FrameChangeHandling,
                eccentricity_z_neuron_resolution: NeuronDepth,
                modulation_z_neuron_resolution: NeuronDepth,
                percentage_neuron_positioning: PercentageNeuronPositioning
                ) -> Result<(), FeagiDataError>
            {
                let eccentricity_cortical_id: CorticalID = MotorCorticalUnit::[<get_cortical_ids_array_for_ $snake_case_name >](frame_change_handling, percentage_neuron_positioning, group)[0];
                let modularity_cortical_id: CorticalID = MotorCorticalUnit::[<get_cortical_ids_array_for_ $snake_case_name >](frame_change_handling, percentage_neuron_positioning, group)[1];

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = {
                    match percentage_neuron_positioning {
                        PercentageNeuronPositioning::Linear => GazePropertiesLinearNeuronVoxelXYZPDecoder::new_box(eccentricity_cortical_id, modularity_cortical_id, eccentricity_z_neuron_resolution, modulation_z_neuron_resolution, number_channels)?,
                        PercentageNeuronPositioning::Fractional => GazePropertiesExponentialNeuronVoxelXYZPDecoder::new_box(eccentricity_cortical_id, modularity_cortical_id, eccentricity_z_neuron_resolution, modulation_z_neuron_resolution, number_channels)?
                    }
                };

                let initial_val: WrappedIOData = WrappedIOData::GazeProperties(GazeProperties::create_default_centered());
                self.register(MotorCorticalUnit::$motor_unit, group, decoder, number_channels, initial_val)?;
                Ok(())
            }
        }

        motor_unit_functions!(@generate_similar_functions $motor_unit, $snake_case_name, GazeProperties);
    };

    // Arm for WrappedIOType::Percentage
    (@generate_functions
        $motor_unit:ident,
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
                let cortical_id: CorticalID = MotorCorticalUnit::[<get_cortical_ids_array_for_ $snake_case_name >](frame_change_handling, percentage_neuron_positioning, group)[0];
                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = {
                    match percentage_neuron_positioning { // TODO fix naming of exponential / fractional
                        PercentageNeuronPositioning::Linear => PercentageLinearNeuronVoxelXYZPDecoder::new_box(cortical_id, z_neuron_resolution, number_channels)?,
                        PercentageNeuronPositioning::Fractional => PercentageExponentialNeuronVoxelXYZPDecoder::new_box(cortical_id, z_neuron_resolution, number_channels)?,
                    }
                };

                let initial_val: WrappedIOData = WrappedIOData::Percentage(Percentage::new_zero());
                self.register(MotorCorticalUnit::$motor_unit, group, decoder, number_channels, initial_val)?;
                Ok(())
            }
        }

        motor_unit_functions!(@generate_similar_functions $motor_unit, $snake_case_name, Percentage);
    };

    // Arm for WrappedIOType::Percentage3D
    (@generate_functions
        $motor_unit:ident,
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
                let cortical_id: CorticalID = MotorCorticalUnit::[<get_cortical_ids_array_for_ $snake_case_name >](frame_change_handling, percentage_neuron_positioning, group)[0];
                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = {
                    match percentage_neuron_positioning { // TODO fix naming of exponential / fractional
                        PercentageNeuronPositioning::Linear => Percentage3DLinearNeuronVoxelXYZPDecoder::new_box(cortical_id, z_neuron_resolution, number_channels)?,
                        PercentageNeuronPositioning::Fractional => Percentage3DExponentialNeuronVoxelXYZPDecoder::new_box(cortical_id, z_neuron_resolution, number_channels)?,
                    }
                };

                let initial_val: WrappedIOData = WrappedIOData::Percentage_3D(Percentage3D::new_zero());
                self.register(MotorCorticalUnit::$motor_unit, group, decoder, number_channels, initial_val)?;
                Ok(())
            }
        }

        motor_unit_functions!(@generate_similar_functions $motor_unit, $snake_case_name, Percentage3D);
    };

    // Arm for WrappedIOType::SignedPercentage
    (@generate_functions
        $motor_unit:ident,
        $snake_case_name:expr,
        SignedPercentage
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
                let cortical_id: CorticalID = MotorCorticalUnit::[<get_cortical_ids_array_for_ $snake_case_name >](frame_change_handling, percentage_neuron_positioning, group)[0];
                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = {
                    match percentage_neuron_positioning { // TODO fix naming of exponential / fractional
                        PercentageNeuronPositioning::Linear => SignedPercentageLinearNeuronVoxelXYZPDecoder::new_box(cortical_id, z_neuron_resolution, number_channels)?,
                        PercentageNeuronPositioning::Fractional => SignedPercentageExponentialNeuronVoxelXYZPDecoder::new_box(cortical_id, z_neuron_resolution, number_channels)?,
                    }
                };

                let initial_val: WrappedIOData = WrappedIOData::SignedPercentage(SignedPercentage::new_from_m1_1_unchecked(0.0));
                self.register(MotorCorticalUnit::$motor_unit, group, decoder, number_channels, initial_val)?;
                Ok(())
            }
        }

        motor_unit_functions!(@generate_similar_functions $motor_unit, $snake_case_name, SignedPercentage);
    };

    // Arm for WrappedIOType::MiscData
    (@generate_functions
        $motor_unit:ident,
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
                let cortical_id: CorticalID = MotorCorticalUnit::[<get_cortical_ids_array_for_ $snake_case_name >](frame_change_handling, group)[0];
                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = MiscDataNeuronVoxelXYZPDecoder::new_box(cortical_id, misc_data_dimensions, number_channels)?;

                let initial_val: WrappedIOData = WrappedIOType::MiscData(Some(misc_data_dimensions)).create_blank_data_of_type()?;
                self.register(MotorCorticalUnit::$motor_unit, group, decoder, number_channels, initial_val)?;
                Ok(())
            }
        }

        motor_unit_functions!(@generate_similar_functions $motor_unit, $snake_case_name, MiscData);
    };

}

pub struct MotorDeviceCache {
    stream_caches: HashMap<(MotorCorticalUnit, CorticalGroupIndex), MotorChannelStreamCaches>,
    agent_device_key_lookup:
        HashMap<AgentDeviceIndex, Vec<(MotorCorticalUnit, CorticalGroupIndex)>>,
    neuron_data: CorticalMappedXYZPNeuronVoxels,
    byte_data: FeagiByteContainer,
    previous_burst: Instant,
}

impl std::fmt::Debug for MotorDeviceCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MotorDeviceCache")
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

impl Default for MotorDeviceCache {
    fn default() -> Self {
        Self::new()
    }
}

impl MotorDeviceCache {
    pub fn new() -> Self {
        MotorDeviceCache {
            stream_caches: HashMap::new(),
            agent_device_key_lookup: HashMap::new(),
            neuron_data: CorticalMappedXYZPNeuronVoxels::new(),
            byte_data: FeagiByteContainer::new_empty(),
            previous_burst: Instant::now(),
        }
    }

    motor_cortical_units!(motor_unit_functions);

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

    pub fn export_registered_motors_as_config_json(
        &self,
    ) -> Result<serde_json::Value, FeagiDataError> {
        let mut output = serde_json::Map::new();
        for ((motor_cortical_unit, cortical_group_index), motor_channel_stream_caches) in
            &self.stream_caches
        {
            let motor_unit_name = motor_cortical_unit.get_snake_case_name().to_string();
            let cortical_group_name = cortical_group_index.to_string();

            let motor_units_map = output
                .entry(motor_unit_name)
                .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()))
                .as_object_mut()
                .expect("Just inserted an Object");

            motor_units_map.insert(
                cortical_group_name,
                motor_channel_stream_caches.export_as_json()?,
            );
        }
        Ok(serde_json::Value::Object(output))
    }

    /// Import motor configurations from JSON
    ///
    /// Updates pipeline stages and friendly names for already-registered motors.
    /// Motors must be registered first using the appropriate register functions.
    ///
    /// # Arguments
    /// * `json` - JSON object containing motor configurations in new format
    ///
    /// # Returns
    /// * `Ok(())` - If import succeeded
    /// * `Err(FeagiDataError)` - If motor not registered or JSON is malformed
    pub fn import_motors_from_json(
        &mut self,
        json: &serde_json::Value,
    ) -> Result<(), FeagiDataError> {
        let output_map = json.as_object().ok_or_else(|| {
            FeagiDataError::DeserializationError("Expected output object for motors".to_string())
        })?;

        for (motor_type_name, groups) in output_map {
            // Parse motor type from snake_case name
            let motor_type =
                MotorCorticalUnit::from_snake_case_name(motor_type_name).ok_or_else(|| {
                    FeagiDataError::DeserializationError(format!(
                        "Unknown motor type: {}",
                        motor_type_name
                    ))
                })?;

            let groups_map = groups.as_object().ok_or_else(|| {
                FeagiDataError::DeserializationError(format!(
                    "Expected groups object for motor type: {}",
                    motor_type_name
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

                // Get the stream cache for this motor type + group
                let stream_cache = self.stream_caches.get_mut(&(motor_type, group_id))
                    .ok_or_else(|| FeagiDataError::BadParameters(
                        format!("Motor {}:{} not registered. Register the motor first before importing configuration.",
                            motor_type_name, group_id_str)
                    ))?;

                // Import configuration (pipelines, friendly names)
                stream_cache.import_from_json(device_config)?;
            }
        }
        Ok(())
    }

    // Returns true if data was retrieved
    pub fn try_decode_bytes_to_neural_data(&mut self) -> Result<bool, FeagiDataError> {
        self.byte_data
            .try_update_struct_from_first_found_struct_of_type(&mut self.neuron_data)
    }

    pub fn try_decode_neural_data_into_cache(
        &mut self,
        time_of_decode: Instant,
    ) -> Result<(), FeagiDataError> {
        for motor_channel_stream_cache in self.stream_caches.values_mut() {
            motor_channel_stream_cache.try_read_neuron_data_to_cache_and_do_callbacks(
                &mut self.neuron_data,
                time_of_decode,
            )?;
        }
        Ok(())
    }

    //endregion

    //region Internal

    //region Cache Abstractions

    fn register(
        &mut self,
        motor_type: MotorCorticalUnit,
        group_index: CorticalGroupIndex,
        neuron_decoder: Box<dyn NeuronVoxelXYZPDecoder>,
        number_channels: CorticalChannelCount,
        initial_cached_value: WrappedIOData,
    ) -> Result<(), FeagiDataError> {
        // NOTE: The length of pipeline_stages_across_channels denotes the number of channels!

        if self.stream_caches.contains_key(&(motor_type, group_index)) {
            return Err(FeagiDataError::BadParameters(format!(
                "Already registered motor {} of group index {}!",
                motor_type, group_index
            )));
        }

        self.stream_caches.insert(
            (motor_type, group_index),
            MotorChannelStreamCaches::new(neuron_decoder, number_channels, initial_cached_value)?,
        );

        Ok(())
    }

    //region Data

    fn try_read_preprocessed_cached_value(
        &self,
        motor_type: MotorCorticalUnit,
        group_index: CorticalGroupIndex,
        channel_index: CorticalChannelIndex,
    ) -> Result<&WrappedIOData, FeagiDataError> {
        let motor_stream_caches =
            self.try_get_motor_channel_stream_caches(motor_type, group_index)?;
        Ok(motor_stream_caches.get_preprocessed_motor_value(channel_index)?)
    }

    fn try_read_postprocessed_cached_value(
        &self,
        motor_type: MotorCorticalUnit,
        group_index: CorticalGroupIndex,
        channel_index: CorticalChannelIndex,
    ) -> Result<&WrappedIOData, FeagiDataError> {
        let motor_stream_caches =
            self.try_get_motor_channel_stream_caches(motor_type, group_index)?;
        Ok(motor_stream_caches.get_postprocessed_motor_value(channel_index)?)
    }

    fn try_register_motor_callback<F>(
        &mut self,
        motor_type: MotorCorticalUnit,
        group_index: CorticalGroupIndex,
        channel_index: CorticalChannelIndex,
        callback: F,
    ) -> Result<FeagiSignalIndex, FeagiDataError>
    where
        F: Fn(&WrappedIOData) + Send + Sync + 'static,
    {
        let motor_stream_caches =
            self.try_get_motor_channel_stream_caches_mut(motor_type, group_index)?;
        let index =
            motor_stream_caches.try_connect_to_data_processed_signal(channel_index, callback)?;
        Ok(index)
    }

    //endregion

    //region Stages

    fn try_get_single_stage_properties(
        &self,
        motor_type: MotorCorticalUnit,
        group_index: CorticalGroupIndex,
        channel_index: CorticalChannelIndex,
        stage_index: PipelineStagePropertyIndex,
    ) -> Result<PipelineStageProperties, FeagiDataError> {
        let motor_stream_caches =
            self.try_get_motor_channel_stream_caches(motor_type, group_index)?;
        motor_stream_caches.try_get_single_stage_properties(channel_index, stage_index)
    }

    fn try_get_all_stage_properties(
        &self,
        motor_type: MotorCorticalUnit,
        group_index: CorticalGroupIndex,
        channel_index: CorticalChannelIndex,
    ) -> Result<Vec<PipelineStageProperties>, FeagiDataError> {
        let motor_stream_caches =
            self.try_get_motor_channel_stream_caches(motor_type, group_index)?;
        motor_stream_caches.get_all_stage_properties(channel_index)
    }

    fn try_update_single_stage_properties(
        &mut self,
        motor_type: MotorCorticalUnit,
        group_index: CorticalGroupIndex,
        channel_index: CorticalChannelIndex,
        pipeline_stage_property_index: PipelineStagePropertyIndex,
        replacing_property: PipelineStageProperties,
    ) -> Result<(), FeagiDataError> {
        let motor_stream_caches =
            self.try_get_motor_channel_stream_caches_mut(motor_type, group_index)?;
        motor_stream_caches.try_update_single_stage_properties(
            channel_index,
            pipeline_stage_property_index,
            replacing_property,
        )
    }

    fn try_update_all_stage_properties(
        &mut self,
        motor_type: MotorCorticalUnit,
        group_index: CorticalGroupIndex,
        channel_index: CorticalChannelIndex,
        new_pipeline_stage_properties: Vec<PipelineStageProperties>,
    ) -> Result<(), FeagiDataError> {
        let motor_stream_caches =
            self.try_get_motor_channel_stream_caches_mut(motor_type, group_index)?;
        motor_stream_caches
            .try_update_all_stage_properties(channel_index, new_pipeline_stage_properties)
    }

    fn try_replace_single_stage(
        &mut self,
        motor_type: MotorCorticalUnit,
        group_index: CorticalGroupIndex,
        channel_index: CorticalChannelIndex,
        replacing_at_index: PipelineStagePropertyIndex,
        new_pipeline_stage_properties: PipelineStageProperties,
    ) -> Result<(), FeagiDataError> {
        let motor_stream_caches =
            self.try_get_motor_channel_stream_caches_mut(motor_type, group_index)?;
        motor_stream_caches.try_replace_single_stage(
            channel_index,
            replacing_at_index,
            new_pipeline_stage_properties,
        )
    }

    fn try_replace_all_stages(
        &mut self,
        motor_type: MotorCorticalUnit,
        group_index: CorticalGroupIndex,
        channel_index: CorticalChannelIndex,
        new_pipeline_stage_properties: Vec<PipelineStageProperties>,
    ) -> Result<(), FeagiDataError> {
        let motor_stream_caches =
            self.try_get_motor_channel_stream_caches_mut(motor_type, group_index)?;
        motor_stream_caches.try_replace_all_stages(channel_index, new_pipeline_stage_properties)
    }

    fn try_removing_all_stages(
        &mut self,
        sensor_type: MotorCorticalUnit,
        group_index: CorticalGroupIndex,
        channel_index: CorticalChannelIndex,
    ) -> Result<(), FeagiDataError> {
        let motor_stream_cache =
            self.try_get_motor_channel_stream_caches_mut(sensor_type, group_index)?;
        motor_stream_cache.try_removing_all_stages(channel_index)?;
        Ok(())
    }

    //endregion

    //region Agent Devices

    fn register_agent_device_key(
        &mut self,
        agent_device_index: AgentDeviceIndex,
        motor_type: MotorCorticalUnit,
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
        keys.push((motor_type, group_index));
        Ok(())
    }

    fn try_read_preprocessed_cached_values_by_agent_device(
        &self,
        agent_device_index: AgentDeviceIndex,
        channel_index: CorticalChannelIndex,
    ) -> Result<Vec<&WrappedIOData>, FeagiDataError> {
        let motor_group_pairs = self.try_get_agent_device_lookup(agent_device_index)?;
        let mut results = Vec::with_capacity(motor_group_pairs.len());
        for (motor_type, group_index) in motor_group_pairs {
            let value =
                self.try_read_preprocessed_cached_value(*motor_type, *group_index, channel_index)?;
            results.push(value);
        }
        Ok(results)
    }

    fn try_read_postprocessed_cached_values_by_agent_device(
        &self,
        agent_device_index: AgentDeviceIndex,
        channel_index: CorticalChannelIndex,
    ) -> Result<Vec<&WrappedIOData>, FeagiDataError> {
        let motor_group_pairs = self.try_get_agent_device_lookup(agent_device_index)?;
        let mut results = Vec::with_capacity(motor_group_pairs.len());
        for (motor_type, group_index) in motor_group_pairs {
            let value =
                self.try_read_postprocessed_cached_value(*motor_type, *group_index, channel_index)?;
            results.push(value);
        }
        Ok(results)
    }

    //endregion

    //endregion

    //region Hashmap Interactions

    fn try_get_motor_channel_stream_caches(
        &self,
        motor_type: MotorCorticalUnit,
        group_index: CorticalGroupIndex,
    ) -> Result<&MotorChannelStreamCaches, FeagiDataError> {
        let check = self.stream_caches.get(&(motor_type, group_index));
        if check.is_none() {
            return Err(FeagiDataError::BadParameters(format!(
                "Unable to find {} of cortical group index {} in registered motor's list!",
                motor_type, group_index
            )));
        }
        let check = check.unwrap();
        Ok(check)
    }

    fn try_get_motor_channel_stream_caches_mut(
        &mut self,
        motor_type: MotorCorticalUnit,
        group_index: CorticalGroupIndex,
    ) -> Result<&mut MotorChannelStreamCaches, FeagiDataError> {
        let check = self.stream_caches.get_mut(&(motor_type, group_index));
        if check.is_none() {
            return Err(FeagiDataError::BadParameters(format!(
                "Unable to find {} of cortical group index {} in registered motor's list!",
                motor_type, group_index
            )));
        }
        let check = check.unwrap();
        Ok(check)
    }

    fn try_get_agent_device_lookup(
        &self,
        agent_device_index: AgentDeviceIndex,
    ) -> Result<&[(MotorCorticalUnit, CorticalGroupIndex)], FeagiDataError> {
        let val = self
            .agent_device_key_lookup
            .get(&agent_device_index)
            .ok_or(FeagiDataError::BadParameters(format!(
                "No registered motor device found in agent's list for agent index {}!",
                *agent_device_index
            )))?;
        Ok(val)
    }

    fn try_get_agent_device_lookup_mut(
        &mut self,
        agent_device_index: AgentDeviceIndex,
    ) -> Result<&mut Vec<(MotorCorticalUnit, CorticalGroupIndex)>, FeagiDataError> {
        let val = self
            .agent_device_key_lookup
            .get_mut(&agent_device_index)
            .ok_or(FeagiDataError::BadParameters(format!(
                "No registered motor device found in agent's list for agent index {}!",
                *agent_device_index
            )))?;
        Ok(val)
    }

    //endregion

    //endregion
}

impl Display for MotorDeviceCache {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(write!(f, "Motor Device Cache:\n")?)
    }
}
