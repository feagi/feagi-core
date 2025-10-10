use std::time::Instant;
use std::sync::{Arc, Mutex};
use paste;
use feagi_data_structures::{motor_definition, sensor_definition, FeagiDataError, FeagiSignalIndex};
use feagi_data_structures::genomic::descriptors::{CorticalChannelCount, CorticalChannelIndex, CorticalGroupIndex, NeuronDepth};
use feagi_data_structures::genomic::{MotorCorticalType, SensorCorticalType};
use feagi_data_serialization::FeagiByteContainer;
use crate::caching::io_motor_cache::IOMotorCache;
use crate::caching::io_sensor_cache::IOSensorCache;
use crate::data_pipeline::{PipelineStageProperties, PipelineStagePropertyIndex};
use crate::data_pipeline::stage_properties::{IdentityStageProperties, ImageSegmentorStageProperties};
use crate::data_types::descriptors::{GazeProperties, ImageFrameProperties, MiscDataDimensions, SegmentedImageFrameProperties, SegmentedXYImageResolutions};
use crate::data_types::*;
use crate::neuron_voxel_coding::xyzp::encoders::*;
use crate::neuron_voxel_coding::xyzp::{NeuronVoxelXYZPDecoder, NeuronVoxelXYZPEncoder};
use crate::neuron_voxel_coding::xyzp::decoders::*;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

//region macros

macro_rules! motor_registrations
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
            motor_registrations!(@generate_function
                $cortical_type_key_name,
                $snake_case_identifier,
                $default_coder_type,
                $wrapped_data_type,
                $data_type
            );
        )*
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = PercentageLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = PercentageExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = PercentageLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = PercentageExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = Percentage2DLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = Percentage2DExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = Percentage2DLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = Percentage2DExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = Percentage3DLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = Percentage3DExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = Percentage3DLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = Percentage3DExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = Percentage4DLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = Percentage4DExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = Percentage4DLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = Percentage4DExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentageLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentageExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentageLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentageExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentage2DLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentage2DExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentage2DLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentage2DExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentage3DLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentage3DExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentage3DLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentage3DExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentage4DLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentage4DExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentage4DLinearNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };

                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = SignedPercentage4DExponentialNeuronVoxelXYZPDecoder::new_box(MOTOR_TYPE.to_cortical_id(group) , *z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                
                let mut motors = self.motors.lock().unwrap();
                motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                return Err(FeagiDataError::NotImplemented)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                return Err(FeagiDataError::NotImplemented)
            }
         }
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
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
            pub fn [<motor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                return Err(FeagiDataError::NotImplemented)
            }
         }
    };
}

macro_rules! motor_read_data
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
            motor_read_data!(@generate_function
                $cortical_type_key_name,
                $snake_case_identifier,
                $default_coder_type,
                $wrapped_data_type,
                $data_type
            );
        )*
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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

    // Arm for MiscData with Absolute encoding
    (@generate_function
        $cortical_type_key_name:ident,
        $snake_case_identifier:expr,
        MiscData_Absolute,
        $wrapped_data_type:expr,
        $data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
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
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<ImageFrame, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &ImageFrame = wrapped.try_into()?;
                Ok(value.clone())
            }
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<ImageFrame, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &ImageFrame = wrapped.try_into()?;
                Ok(value.clone())
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<ImageFrame, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &ImageFrame = wrapped.try_into()?;
                Ok(value.clone())
            }
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<ImageFrame, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                let motors = self.motors.lock().unwrap();
                let wrapped = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)?;
                let value: &ImageFrame = wrapped.try_into()?;
                Ok(value.clone())
            }
         }
    };
}


macro_rules! sensor_registrations
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
            sensor_registrations!(@generate_function
                $cortical_type_key_name,
                $snake_case_identifier,
                $default_coder_type,
                $wrapped_data_type,
                $data_type
            );
        )*
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = PercentageLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = PercentageExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = PercentageLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = PercentageExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = Percentage2DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = Percentage2DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = Percentage2DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = Percentage2DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = Percentage3DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = Percentage3DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = Percentage3DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = Percentage3DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = Percentage4DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = Percentage4DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = Percentage4DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = Percentage4DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage1DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage1DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage1DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage1DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage2DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage2DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage2DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage2DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage3DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage3DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage3DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage3DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage4DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage4DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage4DLinearNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                z_neuron_resolution: NeuronDepth
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = SignedPercentage4DExponentialNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , z_neuron_resolution, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                misc_data_dimensions: MiscDataDimensions
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = MiscDataNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , misc_data_dimensions, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                misc_data_dimensions: MiscDataDimensions
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = MiscDataNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , misc_data_dimensions, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                image_properties: &ImageFrameProperties,
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = ImageFrameNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , image_properties, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
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
            pub fn [<sensor_register_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                number_channels: CorticalChannelCount,
                image_properties: &ImageFrameProperties,
            ) -> Result<(), FeagiDataError>
            {
                const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::$cortical_type_key_name;
                let data_type: WrappedIOType = $wrapped_data_type;

                let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send> = ImageFrameNeuronVoxelXYZPEncoder::new_box(SENSOR_TYPE.to_cortical_id(group) , image_properties, number_channels)?;

                let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
                    let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
                    for i in 0..*number_channels {
                        output.push( vec![IdentityStageProperties::new_box(data_type)?])
                    };
                    output
                };
                let mut sensors = self.sensors.lock().unwrap();
                sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
            }
         }
    };

}


//endregion




pub struct IOCache {
    sensors: Arc<Mutex<IOSensorCache>>,
    motors: Arc<Mutex<IOMotorCache>>,
}

// TODO
/*
——— No Performant Parallelism? ———
⠀⣞⢽⢪⢣⢣⢣⢫⡺⡵⣝⡮⣗⢷⢽⢽⢽⣮⡷⡽⣜⣜⢮⢺⣜⢷⢽⢝⡽⣝
⠸⡸⠜⠕⠕⠁⢁⢇⢏⢽⢺⣪⡳⡝⣎⣏⢯⢞⡿⣟⣷⣳⢯⡷⣽⢽⢯⣳⣫⠇
⠀⠀⢀⢀⢄⢬⢪⡪⡎⣆⡈⠚⠜⠕⠇⠗⠝⢕⢯⢫⣞⣯⣿⣻⡽⣏⢗⣗⠏⠀
⠀⠪⡪⡪⣪⢪⢺⢸⢢⢓⢆⢤⢀⠀⠀⠀⠀⠈⢊⢞⡾⣿⡯⣏⢮⠷⠁⠀⠀
⠀⠀⠀⠈⠊⠆⡃⠕⢕⢇⢇⢇⢇⢇⢏⢎⢎⢆⢄⠀⢑⣽⣿⢝⠲⠉⠀⠀⠀⠀
⠀⠀⠀⠀⠀⡿⠂⠠⠀⡇⢇⠕⢈⣀⠀⠁⠡⠣⡣⡫⣂⣿⠯⢪⠰⠂⠀⠀⠀⠀
⠀⠀⠀⠀⡦⡙⡂⢀⢤⢣⠣⡈⣾⡃⠠⠄⠀⡄⢱⣌⣶⢏⢊⠂⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⢝⡲⣜⡮⡏⢎⢌⢂⠙⠢⠐⢀⢘⢵⣽⣿⡿⠁⠁⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠨⣺⡺⡕⡕⡱⡑⡆⡕⡅⡕⡜⡼⢽⡻⠏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⣼⣳⣫⣾⣵⣗⡵⡱⡡⢣⢑⢕⢜⢕⡝⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⣴⣿⣾⣿⣿⣿⡿⡽⡑⢌⠪⡢⡣⣣⡟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⡟⡾⣿⢿⢿⢵⣽⣾⣼⣘⢸⢸⣞⡟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠁⠇⠡⠩⡫⢿⣝⡻⡮⣒⢽⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
—————————————————————————————————
 */

// prefixes:
// cache_ -> cache encoding / decoding / alteration related function
// sensor_ -> sensor device specific function
// motor_ -> motor device specific function

impl IOCache {

    pub fn new() -> Self {
        IOCache {
            sensors: Arc::new(Mutex::new(IOSensorCache::new())),
            motors: Arc::new(Mutex::new(IOMotorCache::new()))
        }
    }


    //region Sensors

    //region Cache Logic

    pub fn sensor_encode_data_to_bytes(&mut self, increment_value: u16) -> Result<(), FeagiDataError> {
        let mut sensors = self.sensors.lock().unwrap();
        _ = sensors.try_encode_updated_sensor_data_to_neurons(Instant::now())?;
        _ = sensors.try_encode_updated_neuron_data_to_feagi_byte_container(increment_value)?;
        Ok(())
    }

    pub fn sensor_copy_feagi_byte_container(&self) -> FeagiByteContainer {
        let mut sensors = self.sensors.lock().unwrap();
        sensors.get_feagi_byte_container().clone()
    }

    pub fn sensor_replace_feagi_byte_container(&mut self, feagi_byte_container: FeagiByteContainer) {
        let mut sensors = self.sensors.lock().unwrap();
        sensors.replace_feagi_byte_container(feagi_byte_container);
    }

    //endregion

    //region Devices

    sensor_definition!(sensor_registrations);





    /*
    //region Misc


    pub fn sensor_write_misc_absolute(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex, data: &WrappedIOData) -> Result<(), FeagiDataError> {
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::MiscellaneousAbsolute;
        self.sensors.try_update_value(SENSOR_TYPE, group, channel, data, Instant::now())
    }
    //endregion
    */

    //region Segmented Vision

    pub fn sensor_register_segmented_vision_absolute(&mut self, group: CorticalGroupIndex, number_channels: CorticalChannelCount, input_image_properties: ImageFrameProperties, segmented_image_properties: SegmentedImageFrameProperties, initial_gaze: GazeProperties) -> Result<(), FeagiDataError> {

        let cortical_ids = SegmentedImageFrame::create_ordered_cortical_ids_for_segmented_vision(group, false);
        let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send > = SegmentedImageFrameNeuronVoxelXYZPEncoder::new_box(cortical_ids, segmented_image_properties, number_channels)?;

        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
            let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
            for i in 0..*number_channels {
                output.push( vec![ImageSegmentorStageProperties::new_box(input_image_properties, segmented_image_properties, initial_gaze)?]) // TODO properly implement clone so we dont need to do this
            };
            output
        };
        let mut sensors = self.sensors.lock().unwrap();
        sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)?;
        Ok(())
    }

    pub fn sensor_write_segmented_vision_absolute(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex, data: &WrappedIOData) -> Result<(), FeagiDataError> {
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        let mut sensors = self.sensors.lock().unwrap();
        sensors.try_update_value(SENSOR_TYPE, group, channel, data, Instant::now())?;
        Ok(())
    }

    pub fn sensor_update_stage_segmented_vision_absolute(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex, pipeline_stage_property_index: PipelineStagePropertyIndex, stage: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<(), FeagiDataError> {
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        let mut sensors = self.sensors.lock().unwrap();
        sensors.try_updating_pipeline_stage(SENSOR_TYPE, group, channel, pipeline_stage_property_index, stage)?;
        Ok(())
    }


    //endregion


    //endregion

    //endregion


    //region Motors

    //region Cache Logic

    pub fn motor_update_data_from_bytes(&mut self) -> Result<bool, FeagiDataError> {
        let mut motors = self.motors.lock().unwrap();
        let has_decoded_neuron_data = motors.try_decode_bytes_to_neural_data()?;
        if !has_decoded_neuron_data {
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

    motor_definition!(motor_registrations);
    motor_definition!(motor_read_data);

    //region Gaze


    pub fn motor_add_callback_gaze_absolute_linear<F>(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex, callback: F) -> Result<FeagiSignalIndex, FeagiDataError>
    where
        F: Fn(&()) + Send + Sync + 'static,
    {
        const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::GazeAbsoluteLinear;
        let mut motors = self.motors.lock().unwrap();
        let index = motors.try_register_motor_callback(MOTOR_TYPE, group, channel, callback)?;
        Ok(index)
    }
    //endregion


    //endregion


    
    
    
}
