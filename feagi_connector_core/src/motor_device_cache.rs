use std::collections::HashMap;
use std::time::Instant;
use feagi_data_serialization::FeagiByteContainer;
use feagi_data_structures::{motor_definition, FeagiDataError, FeagiSignalIndex};
use feagi_data_structures::genomic::descriptors::{AgentDeviceIndex, CorticalChannelIndex, CorticalGroupIndex, NeuronDepth, CorticalChannelCount};
use feagi_data_structures::genomic::MotorCorticalType;
use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use crate::caching::per_channel_stream_caches::MotorChannelStreamCaches;
use crate::data_pipeline::{PipelineStageProperties, PipelineStagePropertyIndex};
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPDecoder;
use crate::data_types::*;
use crate::data_types::descriptors::*;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};
use crate::neuron_voxel_coding::xyzp::decoders::*;

macro_rules! motor_functions {
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
        $(
            motor_functions!(@generate_function
                $cortical_type_key_name,
                $snake_case_identifier,
                $default_coder_type,
                $wrapped_data_type,
                $data_type
            );
        )*
    };

    // Helper macro to generate stage and callback functions
    (@generate_stage_and_callback_functions
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_get_single_stage_properties>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
                stage_index: PipelineStagePropertyIndex
            ) -> Result<Box<dyn PipelineStageProperties + Sync + Send>, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let stage = self.try_get_single_stage_properties(MOTOR_TYPE, group, channel_index, stage_index)?;
                Ok(stage)
            }

            pub fn [<motor_ $snake_case_identifier _try_get_all_stage_properties>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Vec<Box<dyn PipelineStageProperties + Sync + Send>>, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let stages = self.get_all_stage_properties(MOTOR_TYPE, group, channel_index)?;
                Ok(stages)
            }

            pub fn [<motor_ $snake_case_identifier _try_update_single_stage_properties>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
                pipeline_stage_property_index: PipelineStagePropertyIndex,
                updating_property: Box<dyn PipelineStageProperties + Sync + Send>
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                self.try_update_single_stage_properties(MOTOR_TYPE, group, channel_index, pipeline_stage_property_index, updating_property)?;
                Ok(())
            }

            pub fn [<motor_ $snake_case_identifier _try_update_all_stage_properties>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
                updated_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                self.try_update_all_stage_properties(MOTOR_TYPE, group, channel_index, updated_pipeline_stage_properties)?;
                Ok(())
            }

            pub fn [<motor_ $snake_case_identifier _try_replace_single_stage>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
                pipeline_stage_property_index: PipelineStagePropertyIndex,
                replacing_property: Box<dyn PipelineStageProperties + Sync + Send>
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                self.try_replace_single_stage(MOTOR_TYPE, group, channel_index, pipeline_stage_property_index, replacing_property)?;
                Ok(())
            }

            pub fn [<motor_ $snake_case_identifier _try_replace_all_stages>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
                new_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                self.try_replace_all_stages(MOTOR_TYPE, group, channel_index, new_pipeline_stage_properties)?;
                Ok(())
            }

            pub fn [<motor_ $snake_case_identifier _try_removing_all_stages>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                self.try_removing_all_stages(MOTOR_TYPE, group, channel_index)?;
                Ok(())
            }

            pub fn [<motor_ $snake_case_identifier _try_register_motor_callback>]<F>(
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
                callback: F
            ) -> Result<FeagiSignalIndex, FeagiDataError>
            where F: Fn(&WrappedIOData) + Send + Sync + 'static,
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let signal_index = self.try_register_motor_callback(MOTOR_TYPE, group, channel_index, callback)?;
                Ok(signal_index)
            }
        }
    };

    // Arm for Percentage with Absolute Linear encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        Percentage_Absolute_Linear,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = PercentageLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;
                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage = wrapped.try_into()?;
                Ok(value.clone())
            }
         }

         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for Percentage with Absolute Fractional encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        Percentage_Absolute_Fractional,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = PercentageExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage = wrapped.try_into()?;
                Ok(value.clone())
            }
         }

         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for Percentage with Incremental Linear encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        Percentage_Incremental_Linear,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = PercentageLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage = wrapped.try_into()?;
                Ok(value.clone())
            }
         }

         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for Percentage with Incremental Fractional encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        Percentage_Incremental_Fractional,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = PercentageExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage = wrapped.try_into()?;
                Ok(value.clone())
            }

         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
         }
    };

    // Arm for Percentage2D with Absolute Linear encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        Percentage2D_Absolute_Linear,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = Percentage2DLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage2D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage2D = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage2D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage2D = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for Percentage2D with Absolute Fractional encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        Percentage2D_Absolute_Fractional,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = Percentage2DExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage2D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage2D = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage2D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage2D = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for Percentage2D with Incremental Linear encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        Percentage2D_Incremental_Linear,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = Percentage2DLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage2D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage2D = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage2D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage2D = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for Percentage2D with Incremental Fractional encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        Percentage2D_Incremental_Fractional,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = Percentage2DExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage2D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage2D = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage2D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage2D = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for Percentage3D with Absolute Linear encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        Percentage3D_Absolute_Linear,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = Percentage3DLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage3D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage3D = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage3D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage3D = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for Percentage3D with Absolute Fractional encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        Percentage3D_Absolute_Fractional,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = Percentage3DExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage3D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage3D = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage3D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage3D = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for Percentage3D with Incremental Linear encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        Percentage3D_Incremental_Linear,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = Percentage3DLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage4D = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage4D = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for Percentage3D with Incremental Fractional encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        Percentage3D_Incremental_Fractional,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = Percentage3DExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage3D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage3D = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage3D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage3D = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for Percentage4D with Absolute Linear encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        Percentage4D_Absolute_Linear,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = Percentage4DLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage4D = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage4D = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for Percentage4D with Absolute Fractional encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        Percentage4D_Absolute_Fractional,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = Percentage4DExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage4D = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage4D = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for Percentage4D with Incremental Linear encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        Percentage4D_Incremental_Linear,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = Percentage4DLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage4D = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage4D = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for Percentage4D with Incremental Fractional encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        Percentage4D_Incremental_Fractional,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = Percentage4DExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage4D = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &Percentage4D = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for SignedPercentage with Absolute Linear encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        SignedPercentage_Absolute_Linear,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentageLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for SignedPercentage with Absolute Fractional encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        SignedPercentage_Absolute_Fractional,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentageExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for SignedPercentage with Incremental Linear encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        SignedPercentage_Incremental_Linear,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentageLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for SignedPercentage with Incremental Fractional encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        SignedPercentage_Incremental_Fractional,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentageExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for SignedPercentage2D with Absolute Linear encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        SignedPercentage2D_Absolute_Linear,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentage2DLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for SignedPercentage2D with Absolute Fractional encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        SignedPercentage2D_Absolute_Fractional,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentage2DExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage2D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage2D = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage2D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage2D = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for SignedPercentage2D with Incremental Linear encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        SignedPercentage2D_Incremental_Linear,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentage2DLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage2D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage2D = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage2D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage2D = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for SignedPercentage2D with Incremental Fractional encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        SignedPercentage2D_Incremental_Fractional,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentage2DExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage2D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage2D = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage2D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage2D = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for SignedPercentage3D with Absolute Linear encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        SignedPercentage3D_Absolute_Linear,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentage3DLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage2D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage2D = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage2D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage2D = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for SignedPercentage3D with Absolute Fractional encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        SignedPercentage3D_Absolute_Fractional,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentage3DExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage3D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage3D = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage3D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage3D = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for SignedPercentage3D with Incremental Linear encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        SignedPercentage3D_Incremental_Linear,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentage3DLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage3D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage3D = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage3D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage3D = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for SignedPercentage3D with Incremental Fractional encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        SignedPercentage3D_Incremental_Fractional,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentage3DExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage3D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage3D = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage3D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage3D = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for SignedPercentage4D with Absolute Linear encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        SignedPercentage4D_Absolute_Linear,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentage4DLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage4D = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage4D = wrapped.try_into()?;
                Ok(value.clone())
            }
         }
    };

    // Arm for SignedPercentage4D with Absolute Fractional encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        SignedPercentage4D_Absolute_Fractional,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentage4DExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage4D = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage4D = wrapped.try_into()?;
                Ok(value.clone())
            }
         }
    };

    // Arm for SignedPercentage4D with Incremental Linear encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        SignedPercentage4D_Incremental_Linear,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentage4DLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage4D = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage4D = wrapped.try_into()?;
                Ok(value.clone())
            }
         }
    };

    // Arm for SignedPercentage4D with Incremental Fractional encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        SignedPercentage4D_Incremental_Fractional,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentage4DExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage4D = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &SignedPercentage4D = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for ImageFrame with Absolute encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        ImageFrame_Absolute,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                image_properties: ImageFrameProperties
            ) -> Result<(), FeagiDataError>
            {
                return Err(FeagiDataError::NotImplemented)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<ImageFrame, FeagiDataError>
            {
                return Err(FeagiDataError::NotImplemented)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<ImageFrame, FeagiDataError>
            {
                return Err(FeagiDataError::NotImplemented)
            }
         }
    };

    // Arm for ImageFrame with Incremental encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        ImageFrame_Incremental,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                image_properties: ImageFrameProperties
            ) -> Result<(), FeagiDataError>
            {
                return Err(FeagiDataError::NotImplemented)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<ImageFrame, FeagiDataError>
            {
                return Err(FeagiDataError::NotImplemented)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<ImageFrame, FeagiDataError>
            {
                return Err(FeagiDataError::NotImplemented)
            }
         }
    };

    // Arm for ImageFrame with Incremental encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        ImageFrame_Incremental,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                image_properties: ImageFrameProperties
            ) -> Result<(), FeagiDataError>
            {
                return Err(FeagiDataError::NotImplemented)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<ImageFrame, FeagiDataError>
            {
                return Err(FeagiDataError::NotImplemented)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<ImageFrame, FeagiDataError>
            {
                return Err(FeagiDataError::NotImplemented)
            }
         }
    };

    // Arm for MiscData with Absolute encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        MiscData_Absolute,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                misc_data_dimensions: MiscDataDimensions
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = WrappedIOType::MiscData(Some(misc_data_dimensions));

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = MiscDataNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , misc_data_dimensions, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<MiscData, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &MiscData = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<MiscData, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &MiscData = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };

    // Arm for MiscData with Incremental encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        MiscData_Incremental,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                misc_data_dimensions: MiscDataDimensions
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = WrappedIOType::MiscData(Some(misc_data_dimensions));

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = MiscDataNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , misc_data_dimensions, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }

                                self.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<MiscData, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &MiscData = wrapped.try_into()?;
                Ok(value.clone())
            }

            pub fn [<motor_ $snake_case_identifier _try_read_postprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<MiscData, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &MiscData = wrapped.try_into()?;
                Ok(value.clone())
            }
         }


         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };
}

pub(crate) struct MotorDeviceCache {
    stream_caches: HashMap<(MotorCorticalType, CorticalGroupIndex), MotorChannelStreamCaches>,
    agent_device_key_lookup: HashMap<AgentDeviceIndex, Vec<(MotorCorticalType, CorticalGroupIndex)>>,
    neuron_data: CorticalMappedXYZPNeuronVoxels,
    byte_data: FeagiByteContainer,
    previous_burst: Instant,
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

    //region Devices

    motor_definition!(motor_functions);

    //endregion

    //region Internal

    //region Cache Abstractions

    fn register(&mut self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex,
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

    //region Data

    fn try_read_preprocessed_cached_value(&self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<&WrappedIOData, FeagiDataError> {
        let motor_stream_caches = self.try_get_motor_channel_stream_caches(motor_type, group_index)?;
        Ok(motor_stream_caches.try_get_most_recent_preprocessed_motor_value(channel_index)?)
    }

    fn try_read_postprocessed_cached_value(&self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<&WrappedIOData, FeagiDataError> {
        let motor_stream_caches = self.try_get_motor_channel_stream_caches(motor_type, group_index)?;
        Ok(motor_stream_caches.try_get_most_recent_postprocessed_motor_value(channel_index)?)
    }

    fn try_register_motor_callback<F>(&mut self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex, callback: F) -> Result<FeagiSignalIndex, FeagiDataError>
    where
        F: Fn(&WrappedIOData) + Send + Sync + 'static,
    {
        let motor_stream_caches = self.try_get_motor_channel_stream_caches_mut(motor_type, group_index)?;
        let index = motor_stream_caches.try_connect_to_data_processed_signal(channel_index, callback)?;
        Ok(index)
    }

    //endregion

    //region Stages

    fn try_get_single_stage_properties(&self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex, stage_index: PipelineStagePropertyIndex) -> Result<Box<dyn PipelineStageProperties + Sync + Send>, FeagiDataError> {
        let motor_stream_caches = self.try_get_motor_channel_stream_caches(motor_type, group_index)?;
        motor_stream_caches.try_get_single_stage_properties(channel_index, stage_index)
    }

    fn get_all_stage_properties(&self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<Vec<Box<dyn PipelineStageProperties + Sync + Send>>, FeagiDataError> {
        let motor_stream_caches = self.try_get_motor_channel_stream_caches(motor_type, group_index)?;
        motor_stream_caches.get_all_stage_properties(channel_index)
    }

    fn try_update_single_stage_properties(&mut self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex,
                                              channel_index: CorticalChannelIndex, pipeline_stage_property_index: PipelineStagePropertyIndex,
                                              replacing_property: Box<dyn PipelineStageProperties + Sync + Send>)
                                              -> Result<(), FeagiDataError> {

        let motor_stream_caches = self.try_get_motor_channel_stream_caches_mut(motor_type, group_index)?;
        motor_stream_caches.try_update_single_stage_properties(channel_index, pipeline_stage_property_index, replacing_property)
    }

    fn try_update_all_stage_properties(&mut self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex, new_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<(), FeagiDataError> {
        let motor_stream_caches = self.try_get_motor_channel_stream_caches_mut(motor_type, group_index)?;
        motor_stream_caches.try_update_all_stage_properties(channel_index, new_pipeline_stage_properties)
    }

    fn try_replace_single_stage(&mut self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex, replacing_at_index: PipelineStagePropertyIndex, new_pipeline_stage_properties: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<(), FeagiDataError> {
        let motor_stream_caches = self.try_get_motor_channel_stream_caches_mut(motor_type, group_index)?;
        motor_stream_caches.try_replace_single_stage(channel_index, replacing_at_index, new_pipeline_stage_properties)
    }

    fn try_replace_all_stages(&mut self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex, new_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<(), FeagiDataError> {
        let motor_stream_caches = self.try_get_motor_channel_stream_caches_mut(motor_type, group_index)?;
        motor_stream_caches.try_replace_all_stages(channel_index, new_pipeline_stage_properties)
    }

    fn try_removing_all_stages(&mut self, sensor_type: MotorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<(), FeagiDataError> {
        let motor_stream_cache = self.try_get_motor_channel_stream_caches_mut(sensor_type, group_index)?;
        motor_stream_cache.try_removing_all_stages(channel_index)?;
        Ok(())
    }

    //endregion

    //region Agent Devices

    fn register_agent_device_key(&mut self, agent_device_index: AgentDeviceIndex, motor_type: MotorCorticalType, group_index: CorticalGroupIndex) -> Result<(), FeagiDataError> {
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

    fn try_read_preprocessed_cached_values_by_agent_device(&self, agent_device_index: AgentDeviceIndex, channel_index: CorticalChannelIndex) -> Result<Vec<&WrappedIOData>, FeagiDataError> {
        let motor_group_pairs = self.try_get_agent_device_lookup(agent_device_index)?;
        let mut results = Vec::with_capacity(motor_group_pairs.len());
        for (motor_type, group_index) in motor_group_pairs {
            let value = self.try_read_preprocessed_cached_value(*motor_type, *group_index, channel_index)?;
            results.push(value);
        }
        Ok(results)
    }

    fn try_read_postprocessed_cached_values_by_agent_device(&self, agent_device_index: AgentDeviceIndex, channel_index: CorticalChannelIndex) -> Result<Vec<&WrappedIOData>, FeagiDataError> {
        let motor_group_pairs = self.try_get_agent_device_lookup(agent_device_index)?;
        let mut results = Vec::with_capacity(motor_group_pairs.len());
        for (motor_type, group_index) in motor_group_pairs {
            let value = self.try_read_postprocessed_cached_value(*motor_type, *group_index, channel_index)?;
            results.push(value);
        }
        Ok(results)
    }

    //endregion


    //endregion


    //region Hashmap Interactions

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

    //endregion



}