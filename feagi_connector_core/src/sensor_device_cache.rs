use std::collections::HashMap;
use std::time::Instant;
use feagi_data_serialization::FeagiByteContainer;
use feagi_data_structures::{sensor_cortical_units, FeagiDataError, FeagiSignal};
use feagi_data_structures::genomic::cortical_area::descriptors::{CorticalChannelCount, CorticalChannelIndex, CorticalGroupIndex, NeuronDepth};
use feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::{FrameChangeHandling, PercentageNeuronPositioning};
use feagi_data_structures::genomic::cortical_area::{CorticalID, IOCorticalAreaDataType};
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



macro_rules! sensor_unit_functions {
    (
        SensoryCorticalUnit {
            $(
                $(#[doc = $doc:expr])?
                $variant_name:ident => {
                    friendly_name: $friendly_name:expr,
                    snake_case_name: $snake_case_name:expr,
                    accepted_wrapped_io_data_type: $accepted_wrapped_io_data_type:ident,
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
    ) =>
    {
        $(
            sensor_unit_functions!(@generate_functions
            $variant_name,
            $snake_case_name,
            $accepted_wrapped_io_data_type
            );
        )*
    };

    //region Similar Functions
    // Helper macro to generate stage and other similar functions
    (@generate_similar_functions
        $variant_name:ident,
        $snake_case_identifier:expr,
        $wrapped_data_type:ident,
    ) => {
        ::paste::paste! {

            pub fn [<$snake_case_name _write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $wrapped_data_type,
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensoryCorticalUnit = SensoryCorticalUnit::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                self.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<$snake_case_identifier _read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensoryCorticalUnit = SensoryCorticalUnit::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }

            pub fn [<$snake_case_identifier _get_single_stage_properties>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
                stage_index: PipelineStagePropertyIndex
            ) -> Result<Box<dyn PipelineStageProperties + Sync + Send>, FeagiDataError>
            {
                const SENSOR_UNIT_TYPE: SensoryCorticalUnit = SensoryCorticalUnit::$variant_name;
                let stage = self.try_get_single_stage_properties(SENSOR_UNIT_TYPE, group, channel_index, stage_index)?;
                Ok(stage)
            }

            pub fn [<$snake_case_identifier _get_all_stage_properties>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Vec<Box<dyn PipelineStageProperties + Sync + Send>>, FeagiDataError>
            {
                const SENSOR_UNIT_TYPE: SensoryCorticalUnit = SensoryCorticalUnit::$variant_name;
                let stages = self.try_get_all_stage_properties(SENSOR_UNIT_TYPE, group, channel_index)?;
                Ok(stages)
            }

            pub fn [<$snake_case_identifier _update_single_stage_properties>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
                pipeline_stage_property_index: PipelineStagePropertyIndex,
                updating_property: Box<dyn PipelineStageProperties + Sync + Send>
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_UNIT_TYPE: SensoryCorticalUnit = SensoryCorticalUnit::$variant_name;
                self.try_update_single_stage_properties(SENSOR_UNIT_TYPE, group, channel_index, pipeline_stage_property_index, updating_property)?;
                Ok(())
            }

            pub fn [<$snake_case_identifier _update_all_stage_properties>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
                updated_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_UNIT_TYPE: SensoryCorticalUnit = SensoryCorticalUnit::$variant_name;
                self.try_update_all_stage_properties(SENSOR_UNIT_TYPE, group, channel_index, updated_pipeline_stage_properties)?;
                Ok(())
            }

            pub fn [<$snake_case_identifier _replace_single_stage>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
                pipeline_stage_property_index: PipelineStagePropertyIndex,
                replacing_property: Box<dyn PipelineStageProperties + Sync + Send>
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_UNIT_TYPE: SensoryCorticalUnit = SensoryCorticalUnit::$variant_name;
                self.try_replace_single_stage(SENSOR_UNIT_TYPE, group, channel_index, pipeline_stage_property_index, replacing_property)?;
                Ok(())
            }

            pub fn [<$snake_case_identifier _replace_all_stages>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
                new_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_UNIT_TYPE: SensoryCorticalUnit = SensoryCorticalUnit::$variant_name;
                self.try_replace_all_stages(SENSOR_UNIT_TYPE, group, channel_index, new_pipeline_stage_properties)?;
                Ok(())
            }

            pub fn [<$snake_case_identifier _removing_all_stages>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_UNIT_TYPE: SensoryCorticalUnit = SensoryCorticalUnit::$variant_name;
                self.try_removing_all_stages(SENSOR_UNIT_TYPE, group, channel_index)?;
                Ok(())
            }
        }
    };
    //endregion


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
                z_neuron_resolution: NeuronDepth,
                frame_change_handling: FrameChangeHandling,
                percentage_neuron_positioning: PercentageNeuronPositioning
                ) -> Result<(), FeagiDataError>
            {
                let cortical_id: CorticalID = SensoryCorticalUnit::[<get_ $snake_case_name _cortical_ids_array>](frame_change_handling, percentage_neuron_positioning, group)[0];
                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = {
                    match percentage_neuron_positioning { // TODO fix naming of exponential / fractional
                        PercentageNeuronPositioning::Linear => PercentageLinearNeuronVoxelXYZPEncoder::new_box(cortical_id, z_neuron_resolution, number_channels)?,
                        PercentageNeuronPositioning::Fractional => PercentageExponentialNeuronVoxelXYZPEncoder::new_box(cortical_id, z_neuron_resolution, number_channels)?,
                    }
                };

                let initial_val: WrappedIOData = WrappedIOData::Percentage(Percentage::new_zero());
                self.register(SensoryCorticalUnit::$sensory_unit, group, encoder, Vec::new(), initial_val)?;
                Ok(())
            }
        }
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

    sensor_cortical_units!(sensor_unit_functions);


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