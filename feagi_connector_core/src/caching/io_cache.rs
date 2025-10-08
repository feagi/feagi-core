use std::time::Instant;
use feagi_data_structures::{motor_definition, FeagiDataError, FeagiSignalIndex};
use feagi_data_structures::genomic::descriptors::{CorticalChannelCount, CorticalChannelIndex, CorticalGroupIndex, NeuronDepth};
use feagi_data_structures::genomic::{MotorCorticalType, SensorCorticalType};
use paste;
use crate::caching::io_motor_cache::IOMotorCache;
use crate::caching::io_sensor_cache::IOSensorCache;
use crate::data_pipeline::{PipelineStageProperties, PipelineStagePropertyIndex};
use crate::data_pipeline::stage_properties::{IdentityStageProperties, ImageSegmentorStageProperties};
use crate::data_types::descriptors::{GazeProperties, ImageFrameProperties, MiscDataDimensions, SegmentedImageFrameProperties, SegmentedXYImageResolutions};
use crate::data_types::{Percentage4D, SegmentedImageFrame};
use crate::neuron_voxel_coding::xyzp::encoders::{MiscDataNeuronVoxelXYZPEncoder, SegmentedImageFrameNeuronVoxelXYZPEncoder};
use crate::neuron_voxel_coding::xyzp::{NeuronVoxelXYZPDecoder, NeuronVoxelXYZPEncoder};
use crate::neuron_voxel_coding::xyzp::decoders::*;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

//region macros

macro_rules! motor_registrations {
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

                self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
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

macro_rules! motor_read_data {
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
            pub fn [<motor_try_read_preprocessed_cached_value_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<&WrappedIOData, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                self.motors.try_read_preprocessed_cached_value(MOTOR_TYPE, group, channel_index)
            }
            
            pub fn [<motor_try_read_postprocessed_cached_value_ $snake_case_identifier>](
                &mut self,
                group: CorticalGroupIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<&WrappedIOData, FeagiDataError>
            {
                const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::$cortical_type_key_name;
                self.motors.try_read_postprocessed_cached_value(MOTOR_TYPE, group, channel_index)
            }
         }
    };
}


//endregion




pub struct IOCache {
    sensors: IOSensorCache,
    motors: IOMotorCache,
}

// prefixes:
// cache_ -> cache encoding / decoding / alteration related function
// sensor_ -> sensor device specific function
// motor_ -> motor device specific function

impl IOCache {

    pub fn new() -> Self {
        IOCache {
            sensors: IOSensorCache::new(),
            motors: IOMotorCache::new()
        }
    }


    //region Sensors

    pub fn sensor_get_bytes(&mut self) -> Result<&[u8], FeagiDataError> {
        _ = self.sensors.try_encode_updated_sensor_data_to_neurons(Instant::now())?;
        _ = self.sensors.try_encode_updated_neuron_data_to_feagi_byte_container(0)?;
        Ok(self.sensors.export_encoded_bytes())
    }


    //region Misc

    pub fn sensor_register_misc_absolute(&mut self, group: CorticalGroupIndex, number_channels: CorticalChannelCount,
                                         dimensions: MiscDataDimensions) -> Result<(), FeagiDataError> {


        let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send > = MiscDataNeuronVoxelXYZPEncoder::new_box(group, dimensions, number_channels, true)?;
        let data_type = WrappedIOType::MiscData(Some(dimensions.clone()));

        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::MiscellaneousAbsolute;
        let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
            let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
            for i in 0..*number_channels {
                output.push( vec![IdentityStageProperties::new_box(data_type)?]) // TODO properly implement clone so we dont need to do this
            };
            output
        };
        self.sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
    }

    pub fn sensor_write_misc_absolute(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex, data: &WrappedIOData) -> Result<(), FeagiDataError> {
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::MiscellaneousAbsolute;
        self.sensors.try_update_value(SENSOR_TYPE, group, channel, data, Instant::now())
    }
    //endregion


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
        self.sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
    }

    pub fn sensor_write_segmented_vision_absolute(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex, data: &WrappedIOData) -> Result<(), FeagiDataError> {
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        self.sensors.try_update_value(SENSOR_TYPE, group, channel, data, Instant::now())?;
        Ok(())
    }

    pub fn sensor_update_stage_segmented_vision_absolute(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex, pipeline_stage_property_index: PipelineStagePropertyIndex, stage: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<(), FeagiDataError> {
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        self.sensors.try_updating_pipeline_stage(SENSOR_TYPE, group, channel, pipeline_stage_property_index, stage)?;
        Ok(())
    }





    //endregion

    //endregion


    //region Motors

    //region Cache Logic

    pub fn motor_send_bytes(&mut self, incoming_bytes: &[u8]) -> Result<(), FeagiDataError> {
        let mut byte_writer = |buf: &mut Vec<u8>| -> Result<(), FeagiDataError> {
            buf.clear();
            buf.extend_from_slice(incoming_bytes);
            Ok(())
        };
        self.motors.try_import_bytes(&mut byte_writer)?;
        self.motors.try_decode_bytes_to_neural_data()?;
        self.motors.try_decode_neural_data_into_cache(Instant::now())
    }

    //endregion

    motor_definition!(motor_registrations);

    //region Gaze

    pub fn motor_read_post_processed_gaze_absolute_linear(&self, cortical_group_index: CorticalGroupIndex, cortical_channel_index: CorticalChannelIndex) -> Result<Percentage4D, FeagiDataError> {
        const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::GazeAbsoluteLinear;
        let data = self.motors.try_read_postprocessed_cached_value(MOTOR_TYPE, cortical_group_index, cortical_channel_index)?;
        let percentage: Percentage4D = data.try_into()?;
        Ok(percentage)
    }


    pub fn motor_add_callback_gaze_absolute_linear<F>(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex, callback: F) -> Result<FeagiSignalIndex, FeagiDataError>
    where
        F: Fn(&()) + Send + Sync + 'static,
    {
        const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::GazeAbsoluteLinear;
        let index = self.motors.try_register_motor_callback(MOTOR_TYPE, group, channel, callback)?;
        Ok(index)
    }
    //endregion


    //endregion


    
    
    
}
