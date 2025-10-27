use std::time::Instant;
use std::sync::{Arc, Mutex};
use paste;
use feagi_data_structures::{motor_definition, sensor_definition, FeagiDataError, FeagiSignal, FeagiSignalIndex};
use feagi_data_structures::genomic::descriptors::{CorticalChannelCount, CorticalChannelIndex, CorticalGroupIndex, NeuronDepth};
use feagi_data_structures::genomic::{MotorCorticalType, SensorCorticalType};
use feagi_data_serialization::FeagiByteContainer;
use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use crate::caching::io_motor_cache::IOMotorCache;
use crate::caching::io_sensor_cache::IOSensorCache;
use crate::data_pipeline::{PipelineStageProperties, PipelineStagePropertyIndex};
use crate::data_pipeline::stage_properties::{ImageSegmentorStageProperties};
use crate::data_types::descriptors::{GazeProperties, ImageFrameProperties, MiscDataDimensions, SegmentedImageFrameProperties};
use crate::data_types::*;
use crate::neuron_voxel_coding::xyzp::encoders::*;
use crate::neuron_voxel_coding::xyzp::{NeuronVoxelXYZPDecoder, NeuronVoxelXYZPEncoder};
use crate::neuron_voxel_coding::xyzp::decoders::*;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

//region macros
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
                let motors = self.motors.lock().unwrap();
                let stage = motors.try_get_single_stage_properties(MOTOR_TYPE, group, channel_index, stage_index)?;
                Ok(stage)
            }

            pub fn [<motor_ $snake_case_identifier _try_get_all_stage_properties>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Vec<Box<dyn PipelineStageProperties + Sync + Send>>, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let stages = motors.get_all_stage_properties(MOTOR_TYPE, group, channel_index)?;
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
                let mut motors = self.motors.lock().unwrap();
                motors.try_update_single_stage_properties(MOTOR_TYPE, group, channel_index, pipeline_stage_property_index, updating_property)?;
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
                let mut motors = self.motors.lock().unwrap();
                motors.try_update_all_stage_properties(MOTOR_TYPE, group, channel_index, updated_pipeline_stage_properties)?;
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
                let mut motors = self.motors.lock().unwrap();
                motors.try_replace_single_stage(MOTOR_TYPE, group, channel_index, pipeline_stage_property_index, replacing_property)?;
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
                let mut motors = self.motors.lock().unwrap();
                motors.try_replace_all_stages(MOTOR_TYPE, group, channel_index, new_pipeline_stage_properties)?;
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
                let mut motors = self.motors.lock().unwrap();
                let signal_index = motors.try_register_motor_callback(MOTOR_TYPE, group, channel_index, callback)?;
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
                
                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage2D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage2D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage2D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage2D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage3D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage3D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage3D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Percentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage2D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage2D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage2D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage2D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage3D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage3D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage3D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<SignedPercentage4D, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<MiscData, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline, wrapped_default)
            }

            pub fn [<motor_ $snake_case_identifier _try_read_preprocessed_cached_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<MiscData, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
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
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &MiscData = wrapped.try_into()?;
                Ok(value.clone())
            }
         }

         
         motor_functions!(@generate_stage_and_callback_functions $cortical_type_key_name, $snake_case_identifier);
    };
}

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
        $(
            sensor_functions!(@generate_function
                $cortical_type_key_name,
                $snake_case_identifier,
                $default_coder_type,
                $wrapped_data_type,
                $data_type
            );
        )*
    };

    // Helper macro to generate stage functions
    (@generate_stage_functions
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr
    ) => {
        ::paste::paste! {
            pub fn [<sensor_ $snake_case_identifier _try_get_single_stage_properties>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
                stage_index: PipelineStagePropertyIndex
            ) -> Result<Box<dyn PipelineStageProperties + Sync + Send>, FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let mut sensors = self.sensors.lock().unwrap();
                let stage = sensors.try_get_single_stage_properties(SENSOR_TYPE, group, channel_index, stage_index)?;
                Ok(stage)
            }

            pub fn [<sensor_ $snake_case_identifier _try_get_all_stage_properties>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Vec<Box<dyn PipelineStageProperties + Sync + Send>>, FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let mut sensors = self.sensors.lock().unwrap();
                let stages = sensors.try_get_all_stage_properties(SENSOR_TYPE, group, channel_index)?;
                Ok(stages)
            }

            pub fn [<sensor_ $snake_case_identifier _try_update_single_stage_properties>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
                pipeline_stage_property_index: PipelineStagePropertyIndex,
                updating_property: Box<dyn PipelineStageProperties + Sync + Send>
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_single_stage_properties(SENSOR_TYPE, group, channel_index, pipeline_stage_property_index, updating_property)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_update_all_stage_properties>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
                updated_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_all_stage_properties(SENSOR_TYPE, group, channel_index, updated_pipeline_stage_properties)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_replace_single_stage>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
                pipeline_stage_property_index: PipelineStagePropertyIndex,
                replacing_property: Box<dyn PipelineStageProperties + Sync + Send>
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_replace_single_stage(SENSOR_TYPE, group, channel_index, pipeline_stage_property_index, replacing_property)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_replace_all_stages>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex,
                new_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_replace_all_stages(SENSOR_TYPE, group, channel_index, new_pipeline_stage_properties)?;
                Ok(())
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = PercentageLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = PercentageExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = PercentageLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = PercentageExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = Percentage2DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = Percentage2DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = Percentage2DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = Percentage2DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = Percentage3DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = Percentage3DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = Percentage3DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = Percentage3DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = Percentage4DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = Percentage4DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = Percentage4DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = Percentage4DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage1DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage1DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage1DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage1DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage2DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage2DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage2DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage2DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage3DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage3DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage3DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage3DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage4DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage4DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage4DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage4DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                misc_data_dimensions: MiscDataDimensions
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = WrappedIOType::MiscData(Some(misc_data_dimensions));

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = MiscDataNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , misc_data_dimensions, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                misc_data_dimensions: MiscDataDimensions
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = WrappedIOType::MiscData(Some(misc_data_dimensions));

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = MiscDataNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , misc_data_dimensions, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                image_properties: ImageFrameProperties,
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = WrappedIOType::ImageFrame(Some(image_properties));

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = ImageFrameNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , &image_properties, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
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
            pub fn [<sensor_ $snake_case_identifier _try_register>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                image_properties: ImageFrameProperties,
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = WrappedIOType::ImageFrame(Some(image_properties));

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = ImageFrameNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , &image_properties, number_channels)?;

                let wrapped_default: WrappedIOData = data_type.create_blank_data_of_type()?;

                let mut default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                for _i in 0..*number_channels {
                    default_pipeline.push(Vec::new());
                }
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, wrapped_default)
            }

            pub fn [<sensor_ $snake_case_identifier _try_write>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
                data: $data_type
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let wrapped_data: WrappedIOData = data.into();
                let instant = Instant::now();

                let mut sensors = self.sensors.lock().unwrap();
                sensors.try_update_value(SENSOR_TYPE, group, channel, wrapped_data, instant)?;
                Ok(())
            }

            pub fn [<sensor_ $snake_case_identifier _try_read_postprocessed_cache_value>](
                &mut self,
                group: CorticalGroupIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $data_type, FeagiDataError> {

                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;

                let mut sensors = self.sensors.lock().unwrap();
                let wrapped = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
                let val: $data_type = wrapped.try_into()?;
                Ok(val)
            }
         }

        sensor_functions!(@generate_stage_functions $cortical_type_key_name, $snake_case_identifier);
    };

}

//endregion

pub struct IOCache {
    sensors: Arc<Mutex<IOSensorCache>>,
    sensor_neuron_callbacks: FeagiSignal<CorticalMappedXYZPNeuronVoxels>,
    sensor_byte_callbacks: FeagiSignal<FeagiByteContainer>,
    motors: Arc<Mutex<IOMotorCache>>,
}



// prefixes:
// cache_ -> cache encoding / decoding / alteration related function
// sensor_ -> sensor device specific function
// motor_ -> motor device specific function

impl IOCache {

    pub fn new() -> Self {
        IOCache {
            sensors: Arc::new(Mutex::new(IOSensorCache::new())),
            sensor_neuron_callbacks: FeagiSignal::new(),
            sensor_byte_callbacks: FeagiSignal::new(),
            motors: Arc::new(Mutex::new(IOMotorCache::new()))
        }
    }


    //region Sensors

    //region Cache Logic

    pub fn sensor_encode_data_to_neurons_only(&mut self) -> Result<(), FeagiDataError> {
        let mut sensors = self.sensors.lock().unwrap();
        _ = sensors.try_encode_updated_sensor_data_to_neurons(Instant::now())?;
        self.sensor_neuron_callbacks.emit(sensors.get_neurons());
        Ok(())
    }

    pub fn sensor_encode_data_to_neurons_then_bytes(&mut self, increment_value: u16) -> Result<(), FeagiDataError> {
        let mut sensors = self.sensors.lock().unwrap();
        _ = sensors.try_encode_updated_sensor_data_to_neurons(Instant::now())?;
        self.sensor_neuron_callbacks.emit(sensors.get_neurons());

        _ = sensors.try_encode_updated_neuron_data_to_feagi_byte_container(increment_value)?;
        self.sensor_byte_callbacks.emit(sensors.get_feagi_byte_container());
        Ok(())
    }

    pub fn sensor_encoded_neuron_register_callback<F>(&mut self, callback: F) -> Result<FeagiSignalIndex, FeagiDataError>
    where
        F: Fn(&CorticalMappedXYZPNeuronVoxels) + Send + Sync + 'static,
    {
        let index = self.sensor_neuron_callbacks.connect(callback);
        Ok(index)
    }

    pub fn sensor_encoded_bytes_register_callback<F>(&mut self, callback: F) -> Result<FeagiSignalIndex, FeagiDataError>
    where
        F: Fn(&FeagiByteContainer) + Send + Sync + 'static,
    {
        let index = self.sensor_byte_callbacks.connect(callback);
        Ok(index)
    }

    pub fn sensor_copy_feagi_byte_container(&self) -> FeagiByteContainer {
        let sensors = self.sensors.lock().unwrap();
        sensors.get_feagi_byte_container().clone()
    }

    pub fn sensor_replace_feagi_byte_container(&mut self, feagi_byte_container: FeagiByteContainer) {
        let mut sensors = self.sensors.lock().unwrap();
        sensors.replace_feagi_byte_container(feagi_byte_container);
    }


    //endregion

    //region Devices

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
        let mut sensors = self.sensors.lock().unwrap();
        sensors.register(SENSOR_TYPE, group, encoder, default_pipeline, initial_val)?;
        Ok(())
    }

    /// Writes raw image data to a specific segmented vision sensor channel for processing.
    pub fn sensor_segmented_vision_absolute_try_write(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex, data: WrappedIOData) -> Result<(), FeagiDataError> {
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        let mut sensors = self.sensors.lock().unwrap();
        sensors.try_update_value(SENSOR_TYPE, group, channel, data, Instant::now())?;
        Ok(())
    }
    
    /// Reads the post-processed segmented image frame after pipeline processing.
    pub fn sensor_segmented_vision_absolute_try_read_postprocessed_cache_value(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex,) -> Result<SegmentedImageFrame, FeagiDataError> {
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        let mut sensors = self.sensors.lock().unwrap();
        let wrapped_segmented_frame = sensors.try_read_postprocessed_cached_value(SENSOR_TYPE, group, channel)?;
        Ok(wrapped_segmented_frame.try_into()?)
    }

    /// Retrieves the properties of a single processing stage in the pipeline.
    pub fn sensor_segmented_vision_absolute_try_get_single_stage_properties(&mut self, group: CorticalGroupIndex, channel_index: CorticalChannelIndex, stage_index: PipelineStagePropertyIndex) -> Result<Box<dyn PipelineStageProperties + Sync + Send>, FeagiDataError> {
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        let mut sensors = self.sensors.lock().unwrap();
        let properties = sensors.try_get_single_stage_properties(SENSOR_TYPE, group, channel_index, stage_index)?;
        Ok(properties)
    }

    /// Retrieves the properties of all processing stages in the pipeline.
    pub fn sensor_segmented_vision_absolute_try_get_all_stage_properties(&mut self, group: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<Vec<Box<dyn PipelineStageProperties + Sync + Send>>, FeagiDataError> {
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        let mut sensors = self.sensors.lock().unwrap();
        let properties = sensors.try_get_all_stage_properties(SENSOR_TYPE, group, channel_index)?;
        Ok(properties)
    }

    /// Updates the properties of a single processing stage without changing the stage type.
    pub fn sensor_segmented_vision_absolute_try_update_single_stage_properties(&mut self, group: CorticalGroupIndex, channel_index: CorticalChannelIndex, pipeline_stage_property_index: PipelineStagePropertyIndex, updating_property: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<() , FeagiDataError> {
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        let mut sensors = self.sensors.lock().unwrap();
        sensors.try_update_single_stage_properties(SENSOR_TYPE, group, channel_index, pipeline_stage_property_index, updating_property)?;
        Ok(())
    }

    /// Updates the properties of all processing stages while preserving pipeline structure and stage types.
    pub fn sensor_segmented_vision_absolute_try_update_all_stage_properties(&mut self, group: CorticalGroupIndex, channel_index: CorticalChannelIndex, updated_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<() , FeagiDataError> {
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        let mut sensors = self.sensors.lock().unwrap();
        sensors.try_update_all_stage_properties(SENSOR_TYPE, group, channel_index, updated_pipeline_stage_properties)?;
        Ok(())
    }

    /// Replaces a single processing stage, allowing a different stage type to be used.
    pub fn sensor_segmented_vision_absolute_try_replace_single_stage(&mut self, group: CorticalGroupIndex, channel_index: CorticalChannelIndex, pipeline_stage_property_index: PipelineStagePropertyIndex, updating_property: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<() , FeagiDataError> {
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        let mut sensors = self.sensors.lock().unwrap();
        sensors.try_replace_single_stage(SENSOR_TYPE, group, channel_index, pipeline_stage_property_index, updating_property)?;
        Ok(())
    }

    /// Replaces the entire processing pipeline, allowing changes to the number, types, and order of stages.
    pub fn sensor_segmented_vision_absolute_try_replace_all_stages(&mut self, group: CorticalGroupIndex, channel_index: CorticalChannelIndex, updated_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<() , FeagiDataError> {
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        let mut sensors = self.sensors.lock().unwrap();
        sensors.try_replace_all_stages(SENSOR_TYPE, group, channel_index, updated_pipeline_stage_properties)?;
        Ok(())
    }


    //endregion


    //endregion

    //endregion


    //region Motors

    //region Cache Logic

    pub fn motor_update_data_from_bytes(&mut self) -> Result<bool, FeagiDataError> {
        let mut motors = self.motors.lock().unwrap();
        let decoded_bytes_contain_neuron_data = motors.try_decode_bytes_to_neural_data()?;
        if !decoded_bytes_contain_neuron_data {
            return Ok(false);
        }
        motors.try_decode_neural_data_into_cache(Instant::now())?;
        Ok(true)

    }

    pub fn motor_copy_feagi_byte_container(&self) -> FeagiByteContainer {
        let motors = self.motors.lock().unwrap();
        let byte_container =  motors.get_feagi_byte_container();
        byte_container.clone()
    }

    pub fn motor_replace_feagi_byte_container(&mut self, feagi_byte_container: FeagiByteContainer) {
        let mut motors = self.motors.lock().unwrap();
        motors.replace_feagi_byte_container(feagi_byte_container);
    }

    //endregion

    //region Devices
    motor_definition!(motor_functions);

    //endregion


    //endregion


    //region Reflexes (premade callbacks)

    // TODO we need to discuss how to handle absolute,  linear, and we need to figure out better error handling here
    // TODO we can change the call back signature // TODO feedback
    pub fn reflex_absolute_gaze_to_absolute_segmented_vision(&mut self, gaze_group: CorticalGroupIndex, gaze_channel: CorticalChannelIndex, segmentation_group: CorticalGroupIndex, segmentation_channel: CorticalChannelIndex) -> Result<FeagiSignalIndex, FeagiDataError> {

        // Simple way to check if valid. // TODO we should probably have a proper method
        let mut m = self.motors.lock().unwrap();
        _ = m.try_read_postprocessed_cached_value(MotorCorticalType::GazeAbsoluteLinear, gaze_group, gaze_channel)?;

        let s = self.sensors.lock().unwrap();
        _ = s.try_read_postprocessed_cached_value(SensorCorticalType::ImageCameraCenterAbsolute, segmentation_group, segmentation_channel)?;
        drop(s);

        let sensor_ref = Arc::clone(&self.sensors);

        let closure = move |wrapped_motor: &WrappedIOData| {
            let per_4d: Percentage4D = wrapped_motor.try_into().unwrap();
            let gaze: GazeProperties = GazeProperties::new_from_4d(per_4d);

            let mut sensors = sensor_ref.lock().unwrap();
            let stage_properties = sensors.try_get_single_stage_properties(SensorCorticalType::ImageCameraCenterAbsolute, segmentation_group, segmentation_channel, 0.into()).unwrap();
            let mut segmentation_stage_properties: ImageSegmentorStageProperties = stage_properties.as_any().downcast_ref::<ImageSegmentorStageProperties>().unwrap().clone();
            _ = segmentation_stage_properties.update_from_gaze(gaze);
            _ = sensors.try_update_single_stage_properties(SensorCorticalType::ImageCameraCenterAbsolute, segmentation_group, segmentation_channel, 0.into(), Box::new(segmentation_stage_properties));
        };

        let index = m.try_register_motor_callback(MotorCorticalType::GazeAbsoluteLinear, gaze_group, gaze_channel, closure)?;

        Ok(index)
    }



    //endregion
    
    
}
