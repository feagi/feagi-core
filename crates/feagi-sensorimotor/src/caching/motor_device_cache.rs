use crate::configuration::jsonable::JSONInputOutputDefinition;
use crate::data_pipeline::per_channel_stream_caches::MotorCorticalUnitCache;
use crate::data_pipeline::{PipelineStageProperties};
use crate::data_types::descriptors::*;
use crate::data_types::*;
use crate::neuron_voxel_coding::xyzp::decoders::*;
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPDecoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};
use feagi_serialization::FeagiByteContainer;
use feagi_genomic_context::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, PercentageNeuronPositioning, PoseSchema,
};

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::time::Instant;
use feagi_genomic_context::cortical_unit::CorticalUnitIndex;
use feagi_genomic_context::cortical_unit::motor_cortical_unit::MotorCorticalUnit;


use feagi_data::feagi_data_error::FeagiFailDataEtc;

fn feagi_data_etc_error(message: String) -> FeagiDataError {
    let context: &'static str = Box::leak(message.into_boxed_str());
    FeagiFailDataEtc::new(context).into()
}

macro_rules! motor_unit_functions {
    (
        MotorCorticalUnit {
            $(
                $(#[doc = $doc:expr])?
                $cortical_type_key_name:ident => {
                    friendly_name: $friendly_name:expr,
                    accepted_wrapped_io_data_type: $accepted_wrapped_io_data_type:ident,
                    cortical_id_unit_reference: $cortical_id_unit_reference:expr,
                    number_cortical_areas: $number_cortical_areas:expr,
                    cortical_type_parameters: {
                        $($param_name:ident: $param_type:ty),* $(,)?
                    },
                    $(allowed_frame_change_handling: [$($allowed_frame:ident),* $(,)?],)?
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
            $accepted_wrapped_io_data_type
            );
        )*
    };

    //region Similar Functions
    // Helper macro to generate stage and other similar functions
    (@generate_similar_functions
        $cortical_type_key_name:ident,
        $wrapped_data_type:ident
    ) => {
        ::paste::paste! {
            pub fn [<$cortical_type_key_name:snake _read_preprocessed_cache_value>](
                &self,
                unit: CorticalUnitIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $wrapped_data_type, FeagiDataError> {

                const MOTOR_TYPE: MotorCorticalUnit = MotorCorticalUnit::$cortical_type_key_name;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, unit, channel)?;
                let val: $wrapped_data_type = wrapped.try_into()?;
                Ok(val)
            }

            pub fn [<$cortical_type_key_name:snake _read_postprocessed_cache_value>](
                &self,
                unit: CorticalUnitIndex,
                channel: CorticalChannelIndex,
            ) -> Result< $wrapped_data_type, FeagiDataError> {

                const MOTOR_TYPE: MotorCorticalUnit = MotorCorticalUnit::$cortical_type_key_name;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, unit, channel)?;
                let val: $wrapped_data_type = wrapped.try_into()?;
                Ok(val)
            }

            pub fn [<$cortical_type_key_name:snake _try_register_motor_callback>]<F>(
                &mut self,
                unit: CorticalUnitIndex,
                channel_index: CorticalChannelIndex,
                callback: F
            ) -> Result<FeagiSignalIndex, FeagiDataError>
            where F: Fn(&WrappedIOData) + Send + Sync + 'static,
            {
                const MOTOR_TYPE: MotorCorticalUnit = MotorCorticalUnit::$cortical_type_key_name;
                let signal_index = self.try_register_motor_callback(MOTOR_TYPE, unit, channel_index, callback)?;
                Ok(signal_index)
            }

            pub fn [<$cortical_type_key_name:snake _get_single_stage_properties>](
                &mut self,
                unit: CorticalUnitIndex,
                channel_index: CorticalChannelIndex,
                stage_index: PipelineStagePropertyIndex
            ) -> Result<PipelineStageProperties, FeagiDataError>
            {
                const MOTOR_UNIT_TYPE: MotorCorticalUnit = MotorCorticalUnit::$cortical_type_key_name;
                let stage = self.try_get_single_stage_properties(MOTOR_UNIT_TYPE, unit, channel_index, stage_index)?;
                Ok(stage)
            }

            pub fn [<$cortical_type_key_name:snake _get_all_stage_properties>](
                &mut self,
                unit: CorticalUnitIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<Vec<PipelineStageProperties>, FeagiDataError>
            {
                const MOTOR_UNIT_TYPE: MotorCorticalUnit = MotorCorticalUnit::$cortical_type_key_name;
                let stages = self.try_get_all_stage_properties(MOTOR_UNIT_TYPE, unit, channel_index)?;
                Ok(stages)
            }

            pub fn [<$cortical_type_key_name:snake _update_single_stage_properties>](
                &mut self,
                unit: CorticalUnitIndex,
                channel_index: CorticalChannelIndex,
                pipeline_stage_property_index: PipelineStagePropertyIndex,
                updating_property: PipelineStageProperties
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_UNIT_TYPE: MotorCorticalUnit = MotorCorticalUnit::$cortical_type_key_name;
                self.try_update_single_stage_properties(MOTOR_UNIT_TYPE, unit, channel_index, pipeline_stage_property_index, updating_property)?;
                Ok(())
            }

            pub fn [<$cortical_type_key_name:snake _update_all_stage_properties>](
                &mut self,
                unit: CorticalUnitIndex,
                channel_index: CorticalChannelIndex,
                updated_pipeline_stage_properties: Vec<PipelineStageProperties>
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_UNIT_TYPE: MotorCorticalUnit = MotorCorticalUnit::$cortical_type_key_name;
                self.try_update_all_stage_properties(MOTOR_UNIT_TYPE, unit, channel_index, updated_pipeline_stage_properties)?;
                Ok(())
            }

            pub fn [<$cortical_type_key_name:snake _replace_single_stage>](
                &mut self,
                unit: CorticalUnitIndex,
                channel_index: CorticalChannelIndex,
                pipeline_stage_property_index: PipelineStagePropertyIndex,
                replacing_property: PipelineStageProperties
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_UNIT_TYPE: MotorCorticalUnit = MotorCorticalUnit::$cortical_type_key_name;
                self.try_replace_single_stage(MOTOR_UNIT_TYPE, unit, channel_index, pipeline_stage_property_index, replacing_property)?;
                Ok(())
            }

            pub fn [<$cortical_type_key_name:snake _replace_all_stages>](
                &mut self,
                unit: CorticalUnitIndex,
                channel_index: CorticalChannelIndex,
                new_pipeline_stage_properties: Vec<PipelineStageProperties>
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_UNIT_TYPE: MotorCorticalUnit = MotorCorticalUnit::$cortical_type_key_name;
                self.try_replace_all_stages(MOTOR_UNIT_TYPE, unit, channel_index, new_pipeline_stage_properties)?;
                Ok(())
            }

            pub fn [<$cortical_type_key_name:snake _removing_all_stages>](
                &mut self,
                unit: CorticalUnitIndex,
                channel_index: CorticalChannelIndex
            ) -> Result<(), FeagiDataError>
            {
                const MOTOR_UNIT_TYPE: MotorCorticalUnit = MotorCorticalUnit::$cortical_type_key_name;
                self.try_removing_all_stages(MOTOR_UNIT_TYPE, unit, channel_index)?;
                Ok(())
            }
        }
    };
    //endregion

    // Arm for WrappedIOType::GazeProperties
    (@generate_functions
        $motor_unit:ident,
        GazeProperties
    ) => {
        ::paste::paste! {
            pub fn [<$motor_unit:snake _register>](
                &mut self,
                unit: CorticalUnitIndex,
                number_channels: CorticalChannelCount,
                frame_change_handling: FrameChangeHandling,
                eccentricity_z_neuron_resolution: NeuronDepth,
                modulation_z_neuron_resolution: NeuronDepth,
                percentage_neuron_positioning: PercentageNeuronPositioning
                ) -> Result<(), FeagiDataError>
            {
                let eccentricity_cortical_id: CorticalID = MotorCorticalUnit::[<get_cortical_ids_array_for_ $motor_unit:snake _with_parameters>](frame_change_handling, percentage_neuron_positioning, unit)[0];
                let modularity_cortical_id: CorticalID = MotorCorticalUnit::[<get_cortical_ids_array_for_ $motor_unit:snake _with_parameters>](frame_change_handling, percentage_neuron_positioning, unit)[1];

                let io_props: serde_json::Map<String, serde_json::Value> = json!({
                    "frame_change_handling": frame_change_handling,
                    "percentage_neuron_positioning": percentage_neuron_positioning
                }).as_object().unwrap().clone();

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = GazePropertiesNeuronVoxelXYZPDecoder::new_box(
                    eccentricity_cortical_id,
                    modularity_cortical_id,
                    eccentricity_z_neuron_resolution,
                    modulation_z_neuron_resolution,
                    number_channels,
                    percentage_neuron_positioning,
                )?;

                let initial_val: WrappedIOData = WrappedIOData::GazeProperties(GazeProperties::create_default_centered());
                self.register(MotorCorticalUnit::$motor_unit, unit, decoder, io_props, number_channels, initial_val)?;
                Ok(())
            }
        }

        motor_unit_functions!(@generate_similar_functions $motor_unit, GazeProperties);
    };

    // Arm for WrappedIOType::Percentage
    (@generate_functions
        $motor_unit:ident,
        Percentage
    ) => {
        ::paste::paste! {
            pub fn [<$motor_unit:snake _register>](
                &mut self,
                unit: CorticalUnitIndex,
                number_channels: CorticalChannelCount,
                frame_change_handling: FrameChangeHandling,
                z_neuron_resolution: NeuronDepth,
                percentage_neuron_positioning: PercentageNeuronPositioning
                ) -> Result<(), FeagiDataError>
            {
                let cortical_ids = MotorCorticalUnit::[<get_cortical_ids_array_for_ $motor_unit:snake _with_parameters>](frame_change_handling, percentage_neuron_positioning, unit);

                // Handle both single-area and dual-area percentage types.
                // Dual-area means PositionalServo (absolute + incremental);
                // single-area is a plain percentage (e.g. CountOutput).
                let is_dual_area = cortical_ids.get(1).is_some();
                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = match (cortical_ids.get(0), cortical_ids.get(1)) {
                    (Some(&id0), Some(&id1)) => {
                        PositionalServoNeuronVoxelXYZPDecoder::new_box(
                            id0,
                            id1,
                            z_neuron_resolution,
                            number_channels,
                            percentage_neuron_positioning,
                        )?
                    }
                    (Some(&id0), None) => {
                        // Single-area percentage type (e.g., CountOutput)
                        PercentageNeuronVoxelXYZPDecoder::new_box(
                            id0,
                            z_neuron_resolution,
                            number_channels,
                            percentage_neuron_positioning,
                            false,
                            PercentageChannelDimensionality::D1
                        )?
                    }
                    _ => {
                        return Err(feagi_data_etc_error(
                            "Expected at least one cortical_area ID for Percentage motor unit".to_string()
                        ));
                    }
                };

                let io_props: serde_json::Map<String, serde_json::Value> = json!({
                    "frame_change_handling": frame_change_handling,
                    "percentage_neuron_positioning": percentage_neuron_positioning
                }).as_object().unwrap().clone();

                // Dual-area (PositionalServo) must seed at mid-range so the
                // first incremental tick doesn't snap from 0% to near-min.
                let initial_val: WrappedIOData = if is_dual_area {
                    WrappedIOData::Percentage(
                        Percentage::new_from_0_1(0.5).expect("0.5 is always valid"),
                    )
                } else {
                    WrappedIOData::Percentage(Percentage::new_zero())
                };
                self.register(MotorCorticalUnit::$motor_unit, unit, decoder, io_props, number_channels, initial_val)?;
                Ok(())
            }
        }

        motor_unit_functions!(@generate_similar_functions $motor_unit, Percentage);
    };

    // Arm for WrappedIOType::Percentage3D
    (@generate_functions
        $motor_unit:ident,
        Percentage_3D
    ) => {
        ::paste::paste! {
            pub fn [<$motor_unit:snake _register>](
                &mut self,
                unit: CorticalUnitIndex,
                number_channels: CorticalChannelCount,
                frame_change_handling: FrameChangeHandling,
                z_neuron_resolution: NeuronDepth,
                percentage_neuron_positioning: PercentageNeuronPositioning
                ) -> Result<(), FeagiDataError>
            {
                let cortical_id: CorticalID = MotorCorticalUnit::[<get_cortical_ids_array_for_ $motor_unit:snake _with_parameters>](frame_change_handling, percentage_neuron_positioning, unit)[0];

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = PercentageNeuronVoxelXYZPDecoder::new_box(
                    cortical_id,
                    z_neuron_resolution,
                    number_channels,
                    percentage_neuron_positioning,
                    false,
                    PercentageChannelDimensionality::D3
                )?;

                let io_props: serde_json::Map<String, serde_json::Value> = json!({
                    "frame_change_handling": frame_change_handling,
                    "percentage_neuron_positioning": percentage_neuron_positioning
                }).as_object().unwrap().clone();

                let initial_val: WrappedIOData = WrappedIOData::Percentage_3D(Percentage3D::new_zero());
                self.register(MotorCorticalUnit::$motor_unit, unit, decoder, io_props, number_channels, initial_val)?;
                Ok(())
            }
        }

        motor_unit_functions!(@generate_similar_functions $motor_unit, Percentage3D);
    };

    // Arm for WrappedIOType::SignedPercentage
    (@generate_functions
        $motor_unit:ident,
        SpatialPointer3D
    ) => {
        ::paste::paste! {
            pub fn [<$motor_unit:snake _register>](
                &mut self,
                unit: CorticalUnitIndex,
                number_channels: CorticalChannelCount,
                frame_change_handling: FrameChangeHandling,
                percentage_neuron_positioning: PercentageNeuronPositioning,
                pointer_properties: SpatialPointerProperties,
                ) -> Result<(), FeagiDataError>
            {
                let cortical_id: CorticalID = MotorCorticalUnit::[<get_cortical_ids_array_for_ $motor_unit:snake _with_parameters>](
                    frame_change_handling,
                    percentage_neuron_positioning,
                    unit
                )[0];
                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> =
                    SpatialPointerNeuronVoxelXYZPDecoder::new_box(cortical_id, pointer_properties, number_channels)?;

                let io_props: serde_json::Map<String, serde_json::Value> = json!({
                    "frame_change_handling": frame_change_handling,
                    "percentage_neuron_positioning": percentage_neuron_positioning,
                    "SpatialPointer": {
                        "width": pointer_properties.width,
                        "height": pointer_properties.height,
                        "depth": pointer_properties.depth,
                        "window_ms": pointer_properties.window_ms,
                        "max_axis_velocity": pointer_properties.max_axis_velocity
                    }
                }).as_object().unwrap().clone();

                // Output type follows the area's mode: Absolute -> unsigned position,
                // Incremental -> signed motion vector.
                let initial_val: WrappedIOData = match frame_change_handling {
                    FrameChangeHandling::Absolute => {
                        WrappedIOData::Percentage_3D(Percentage3D::new_zero())
                    }
                    FrameChangeHandling::Incremental => {
                        WrappedIOData::SignedPercentage_3D(SignedPercentage3D::new_zero())
                    }
                };
                self.register(MotorCorticalUnit::$motor_unit, unit, decoder, io_props, number_channels, initial_val)?;
                Ok(())
            }
        }

        motor_unit_functions!(@generate_similar_functions $motor_unit, Percentage3D);
        motor_unit_functions!(@generate_spatial_pointer_signed_read $motor_unit);
    };

    // Bespoke signed read accessors for SpatialPointer (Incremental mode emits
    // SignedPercentage3D, which the generic Percentage3D accessors cannot return).
    (@generate_spatial_pointer_signed_read
        $motor_unit:ident
    ) => {
        ::paste::paste! {
            /// Reads the preprocessed Incremental motion vector for a SpatialPointer channel.
            ///
            /// Use this in Incremental mode; the unsigned `*_read_preprocessed_cache_value`
            /// accessor is for Absolute (position) mode.
            pub fn [<$motor_unit:snake _read_signed_preprocessed_cache_value>](
                &self,
                unit: CorticalUnitIndex,
                channel: CorticalChannelIndex,
            ) -> Result<SignedPercentage3D, FeagiDataError> {
                const MOTOR_TYPE: MotorCorticalUnit = MotorCorticalUnit::$motor_unit;
                let wrapped = self.try_read_preprocessed_cached_value(MOTOR_TYPE, unit, channel)?;
                let val: SignedPercentage3D = wrapped.try_into()?;
                Ok(val)
            }

            /// Reads the postprocessed Incremental motion vector for a SpatialPointer channel.
            pub fn [<$motor_unit:snake _read_signed_postprocessed_cache_value>](
                &self,
                unit: CorticalUnitIndex,
                channel: CorticalChannelIndex,
            ) -> Result<SignedPercentage3D, FeagiDataError> {
                const MOTOR_TYPE: MotorCorticalUnit = MotorCorticalUnit::$motor_unit;
                let wrapped = self.try_read_postprocessed_cached_value(MOTOR_TYPE, unit, channel)?;
                let val: SignedPercentage3D = wrapped.try_into()?;
                Ok(val)
            }
        }
    };

    // Arm for WrappedIOType::SignedPercentage
    (@generate_functions
        $motor_unit:ident,
        SignedPercentage
    ) => {
        ::paste::paste! {
            pub fn [<$motor_unit:snake _register>](
                &mut self,
                unit: CorticalUnitIndex,
                number_channels: CorticalChannelCount,
                frame_change_handling: FrameChangeHandling,
                z_neuron_resolution: NeuronDepth,
                percentage_neuron_positioning: PercentageNeuronPositioning
                ) -> Result<(), FeagiDataError>
            {
                let cortical_id: CorticalID = MotorCorticalUnit::[<get_cortical_ids_array_for_ $motor_unit:snake _with_parameters>](frame_change_handling, percentage_neuron_positioning, unit)[0];
                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = PercentageNeuronVoxelXYZPDecoder::new_box(
                    cortical_id,
                    z_neuron_resolution,
                    number_channels,
                    percentage_neuron_positioning,
                    true,
                    PercentageChannelDimensionality::D1
                )?;

                let io_props: serde_json::Map<String, serde_json::Value> = json!({
                    "frame_change_handling": frame_change_handling,
                    "percentage_neuron_positioning": percentage_neuron_positioning
                }).as_object().unwrap().clone();

                let initial_val: WrappedIOData = WrappedIOData::SignedPercentage(SignedPercentage::new_from_m1_1_unchecked(0.0));
                self.register(MotorCorticalUnit::$motor_unit, unit, decoder, io_props, number_channels, initial_val)?;
                Ok(())
            }
        }

        motor_unit_functions!(@generate_similar_functions $motor_unit, SignedPercentage);
    };

    // Arm for WrappedIOType::MiscData
    (@generate_functions
        $motor_unit:ident,
        MiscData
    ) => {
        ::paste::paste! {
            pub fn [<$motor_unit:snake _register>](
                &mut self,
                unit: CorticalUnitIndex,
                number_channels: CorticalChannelCount,
                frame_change_handling: FrameChangeHandling,
                misc_data_dimensions: MiscDataDimensions
                ) -> Result<(), FeagiDataError>
            {
                let cortical_id: CorticalID = MotorCorticalUnit::[<get_cortical_ids_array_for_ $motor_unit:snake _with_parameters>](frame_change_handling, unit)[0];
                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = MiscDataNeuronVoxelXYZPDecoder::new_box(cortical_id, misc_data_dimensions, number_channels)?;

                let io_props: serde_json::Map<String, serde_json::Value> = json!({
                    "frame_change_handling": frame_change_handling
                }).as_object().unwrap().clone();

                let initial_val: WrappedIOData = WrappedIOType::MiscData(Some(misc_data_dimensions)).create_blank_data_of_type()?;
                self.register(MotorCorticalUnit::$motor_unit, unit, decoder, io_props, number_channels, initial_val)?;
                Ok(())
            }
        }

        motor_unit_functions!(@generate_similar_functions $motor_unit, MiscData);
    };

    // Arm for WrappedIOType::ImageFrame
    (@generate_functions
        $motor_unit:ident,
        ImageFrame
    ) => {
        ::paste::paste! {
            pub fn [<$motor_unit:snake _register>](
                &mut self,
                unit: CorticalUnitIndex,
                number_channels: CorticalChannelCount,
                frame_change_handling: FrameChangeHandling,
                image_properties: ImageFrameProperties,
                ) -> Result<(), FeagiDataError>
            {
                let cortical_id: CorticalID = MotorCorticalUnit::[<get_cortical_ids_array_for_ $motor_unit:snake _with_parameters>](frame_change_handling, unit)[0];
                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = CartesianPlaneNeuronVoxelXYZPDecoder::new_box(cortical_id, &image_properties, number_channels)?;

                let io_props: serde_json::Map<String, serde_json::Value> = json!({
                    "frame_change_handling": frame_change_handling
                }).as_object().unwrap().clone();

                let initial_val: WrappedIOData = WrappedIOType::ImageFrame(Some(image_properties)).create_blank_data_of_type()?;
                self.register(MotorCorticalUnit::$motor_unit, unit, decoder, io_props, number_channels, initial_val)?;
                Ok(())
            }
        }

        motor_unit_functions!(@generate_similar_functions $motor_unit, ImageFrame);
    };

    // Arm for WrappedIOType::ImageFilteringSettings
    (@generate_functions
        $motor_unit:ident,
        ImageFilteringSettings
    ) => {
        ::paste::paste! {
            pub fn [<$motor_unit:snake _register>](
                &mut self,
                unit: CorticalUnitIndex,
                number_channels: CorticalChannelCount,
                frame_change_handling: FrameChangeHandling,
                z_neuron_resolution: NeuronDepth,
                percentage_neuron_positioning: PercentageNeuronPositioning
                ) -> Result<(), FeagiDataError>
            {
                let brightness_cortical_id: CorticalID = MotorCorticalUnit::[<get_cortical_ids_array_for_ $motor_unit:snake _with_parameters>](frame_change_handling, percentage_neuron_positioning, unit)[0];
                let contrast_cortical_id: CorticalID = MotorCorticalUnit::[<get_cortical_ids_array_for_ $motor_unit:snake _with_parameters>](frame_change_handling, percentage_neuron_positioning, unit)[1];
                let diff_cortical_id: CorticalID = MotorCorticalUnit::[<get_cortical_ids_array_for_ $motor_unit:snake _with_parameters>](frame_change_handling, percentage_neuron_positioning, unit)[2];

                let io_props: serde_json::Map<String, serde_json::Value> = json!({
                    "frame_change_handling": frame_change_handling,
                    "percentage_neuron_positioning": percentage_neuron_positioning
                }).as_object().unwrap().clone();

                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = ImageFilteringSettingsNeuronVoxelXYZPDecoder::new_box(
                    brightness_cortical_id,
                    contrast_cortical_id,
                    diff_cortical_id,
                    z_neuron_resolution,
                    z_neuron_resolution,
                    z_neuron_resolution,
                    number_channels,
                    percentage_neuron_positioning)?;



                let initial_val: WrappedIOData = WrappedIOData::ImageFilteringSettings(ImageFilteringSettings::default());

                self.register(MotorCorticalUnit::$motor_unit, unit, decoder, io_props, number_channels, initial_val)?;
                Ok(())
            }
        }

        motor_unit_functions!(@generate_similar_functions $motor_unit, ImageFilteringSettings);
    };

    // Arm for PoseEstimationData
    (@generate_functions
        $motor_unit:ident,
        PoseEstimationData
    ) => {
        ::paste::paste! {
            pub fn [<$motor_unit:snake _register>](
                &mut self,
                unit: CorticalUnitIndex,
                number_channels: CorticalChannelCount,
                frame_change_handling: FrameChangeHandling,
                pose_schema: PoseSchema,
                pose_properties: PoseEstimationProperties,
                ) -> Result<(), FeagiDataError>
            {
                let cortical_id: CorticalID = MotorCorticalUnit::[<get_cortical_ids_array_for_ $motor_unit:snake _with_parameters>](frame_change_handling, pose_schema, unit)[0];
                let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = PoseEstimationNeuronVoxelXYZPDecoder::new_box(cortical_id, pose_properties, number_channels)?;

                let io_props: serde_json::Map<String, serde_json::Value> = json!({
                    "frame_change_handling": frame_change_handling,
                    "pose_schema": pose_schema,
                    "PoseEstimation": {
                        "width": pose_properties.width,
                        "height": pose_properties.height,
                        "depth": pose_properties.depth
                    }
                }).as_object().unwrap().clone();

                let initial_val: WrappedIOData = WrappedIOType::PoseEstimationData(Some(pose_properties)).create_blank_data_of_type()?;
                self.register(MotorCorticalUnit::$motor_unit, unit, decoder, io_props, number_channels, initial_val)?;
                Ok(())
            }
        }

        motor_unit_functions!(@generate_similar_functions $motor_unit, PoseEstimationData);
    };
}

/// A single decoded motor value, flattened to one scalar per axis/channel.
///
/// Multi-dimensional decode outputs (e.g. `SpatialPointer` 3D) are flattened into
/// one entry per axis so downstream consumers can address every value with a
/// `(group, channel)` pair regardless of dimensionality.
#[derive(Debug, Clone, PartialEq)]
pub struct DecodedMotorValue {
    /// Cortical unit index (a.k.a. device group).
    pub group: u32,
    /// Flattened channel index. For scalar units this is the source channel; for
    /// N-dimensional units it is `source_channel * N + axis`.
    pub channel: u32,
    /// Decode mode of the owning unit: `"absolute"` or `"incremental"`.
    pub mode: &'static str,
    /// Decoded scalar value. Unsigned types report `[0, 1]`; signed types report
    /// `[-1, 1]`; booleans report `0.0`/`1.0`.
    pub value: f64,
}

/// Flattens a decoded [`WrappedIOData`] into its scalar axis values.
///
/// Returns `None` for non-scalar payloads (images, raw IMU, misc structures) that
/// have no meaningful single-axis float representation.
fn flatten_motor_scalars(data: &WrappedIOData) -> Option<Vec<f64>> {
    use WrappedIOData::*;
    Some(match data {
        Boolean(value) => vec![if *value { 1.0 } else { 0.0 }],
        Percentage(value) => vec![value.get_as_0_1() as f64],
        Percentage_2D(value) => {
            vec![value.a.get_as_0_1() as f64, value.b.get_as_0_1() as f64]
        }
        Percentage_3D(value) => vec![
            value.a.get_as_0_1() as f64,
            value.b.get_as_0_1() as f64,
            value.c.get_as_0_1() as f64,
        ],
        Percentage_4D(value) => vec![
            value.a.get_as_0_1() as f64,
            value.b.get_as_0_1() as f64,
            value.c.get_as_0_1() as f64,
            value.d.get_as_0_1() as f64,
        ],
        SignedPercentage(value) => vec![value.get_as_m1_1() as f64],
        SignedPercentage_2D(value) => {
            vec![value.a.get_as_m1_1() as f64, value.b.get_as_m1_1() as f64]
        }
        SignedPercentage_3D(value) => vec![
            value.a.get_as_m1_1() as f64,
            value.b.get_as_m1_1() as f64,
            value.c.get_as_m1_1() as f64,
        ],
        SignedPercentage_4D(value) => vec![
            value.a.get_as_m1_1() as f64,
            value.b.get_as_m1_1() as f64,
            value.c.get_as_m1_1() as f64,
            value.d.get_as_m1_1() as f64,
        ],
        _ => return None,
    })
}

pub struct MotorDeviceCache {
    motor_cortical_unit_caches:
        HashMap<(MotorCorticalUnit, CorticalUnitIndex), MotorCorticalUnitCache>,
    neuron_data: CorticalMappedXYZPNeuronVoxels,
    byte_data: FeagiByteContainer,
    previous_burst: Instant,
    #[allow(dead_code)]
    is_active: bool,
}

impl std::fmt::Debug for MotorDeviceCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MotorDeviceCache")
            .field(
                "stream_caches_count",
                &self.motor_cortical_unit_caches.len(),
            )
            .field("neuron_data", &self.neuron_data)
            .field("byte_data", &self.byte_data)
            .field("previous_burst", &self.previous_burst)
            .finish()
    }
}

impl MotorDeviceCache {
    pub fn new() -> Self {
        MotorDeviceCache {
            motor_cortical_unit_caches: HashMap::new(),
            neuron_data: CorticalMappedXYZPNeuronVoxels::new(),
            byte_data: FeagiByteContainer::new_empty(),
            previous_burst: Instant::now(),
            is_active: false,
        }
    }

    /// Ingest already-decoded motor neuron data and run callbacks.
    ///
    /// This is a zero-copy convenience for callers that already have a decoded
    /// `CorticalMappedXYZPNeuronVoxels` (e.g. from `feagi-agent`'s motor receive path),
    /// avoiding a re-serialization into `FeagiByteContainer`.
    ///
    /// # Arguments
    /// - `neuron_data`: decoded motor neuron voxels as published by FEAGI
    /// - `time_of_decode`: timestamp used for callback timing logic
    pub fn ingest_neuron_data_and_run_callbacks(
        &mut self,
        neuron_data: CorticalMappedXYZPNeuronVoxels,
        time_of_decode: Instant,
    ) -> Result<(), FeagiDataError> {
        self.neuron_data = neuron_data;
        self.try_decode_neural_data_into_cache(time_of_decode)
    }

    // Clears all registered devices and cache, to allow setting up again
    pub fn reset(&mut self) {
        self.motor_cortical_unit_caches.clear();
        self.neuron_data = CorticalMappedXYZPNeuronVoxels::new();
        self.byte_data = FeagiByteContainer::new_empty();
        self.previous_burst = Instant::now();
    }

    pub fn verify_existence(
        &self,
        motor_cortical_unit: MotorCorticalUnit,
        unit_index: CorticalUnitIndex,
        cortical_channel_index: CorticalChannelIndex,
    ) -> Result<(), FeagiDataError> {
        let motor_stream_caches =
            self.try_get_motor_channel_stream_caches(motor_cortical_unit, unit_index)?;
        motor_stream_caches.verify_channel_exists(cortical_channel_index)
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

    // Returns true if data was retrieved
    pub fn try_decode_bytes_to_neural_data(&mut self) -> Result<bool, FeagiDataError> {
        self.byte_data
            .try_update_struct_from_first_found_struct_of_type(&mut self.neuron_data)
    }

    pub fn try_decode_neural_data_into_cache(
        &mut self,
        time_of_decode: Instant,
    ) -> Result<(), FeagiDataError> {
        for motor_channel_stream_cache in self.motor_cortical_unit_caches.values_mut() {
            motor_channel_stream_cache.try_read_neuron_data_to_cache_and_do_callbacks(
                &self.neuron_data,
                time_of_decode,
            )?;
        }
        Ok(())
    }

    //endregion

    //region  JSON import / export

    pub fn import_from_output_definition(
        &mut self,
        replacing_definition: &JSONInputOutputDefinition,
    ) -> Result<(), FeagiDataError> {
        self.reset();
        let output_units_and_decoder_properties =
            replacing_definition.get_output_units_and_decoder_properties();
        for (motor_unit, unit_and_decoder_definitions) in output_units_and_decoder_properties {
            for unit_and_decoder_definition in unit_and_decoder_definitions {
                let unit_definition = &unit_and_decoder_definition.0;
                let encoder_definition = &unit_and_decoder_definition.1;

                if self
                    .motor_cortical_unit_caches
                    .contains_key(&(*motor_unit, unit_definition.cortical_unit_index))
                {
                    return Err(feagi_data_etc_error(format!(
                        "Already registered motor {} of unit index {}!",
                        *motor_unit, unit_definition.cortical_unit_index
                    )));
                }

                let new_unit = MotorCorticalUnitCache::new_from_json(
                    motor_unit,
                    unit_definition,
                    encoder_definition,
                )?;
                self.motor_cortical_unit_caches
                    .insert((*motor_unit, unit_definition.cortical_unit_index), new_unit);
            }
        }
        Ok(())
    }

    pub fn export_to_output_definition(
        &self,
        filling_definition: &mut JSONInputOutputDefinition,
    ) -> Result<(), FeagiDataError> {
        for ((motor_cortical_unit, cortical_unit_index), motor_channel_stream_caches) in
            self.motor_cortical_unit_caches.iter()
        {
            let unit_and_encoder =
                motor_channel_stream_caches.export_as_jsons(*cortical_unit_index);
            filling_definition.insert_motor(
                *motor_cortical_unit,
                unit_and_encoder.0,
                unit_and_encoder.1,
            );
        }
        Ok(())
    }

    /// Reads every registered motor unit's latest post-processed decode output as a
    /// flat snapshot of scalar `(group, channel, mode, value)` tuples.
    ///
    /// This is the generic, type-erased accessor used to build motor command maps for
    /// downstream clients without needing per-unit typed reads. Multi-axis units are
    /// flattened (see [`DecodedMotorValue`]); non-scalar payloads are skipped.
    pub fn read_decoded_motor_snapshot(&self) -> Vec<DecodedMotorValue> {
        self.read_decoded_motor_snapshot_filtered(false)
    }

    /// Like [`Self::read_decoded_motor_snapshot`], but only includes channels that
    /// were updated on the most recent decode tick.
    ///
    /// Controllers use this for one-shot absolute/incremental command semantics:
    /// apply the decoded value when present, otherwise hold the current pose.
    pub fn read_decoded_motor_snapshot_updated_only(&self) -> Vec<DecodedMotorValue> {
        self.read_decoded_motor_snapshot_filtered(true)
    }

    fn read_decoded_motor_snapshot_filtered(&self, updated_only: bool) -> Vec<DecodedMotorValue> {
        let mut snapshot: Vec<DecodedMotorValue> = Vec::new();
        for ((_motor_unit, cortical_unit_index), unit_cache) in
            self.motor_cortical_unit_caches.iter()
        {
            let group = cortical_unit_index.get() as u32;
            let mode = unit_cache.frame_change_mode_str();
            for channel in 0..unit_cache.channel_count_usize() {
                if updated_only && !unit_cache.channel_updated_last_decode(channel) {
                    continue;
                }
                let channel_index: CorticalChannelIndex = (channel as u32).into();
                let decoded = match unit_cache.get_postprocessed_motor_value(channel_index) {
                    Ok(value) => value,
                    Err(_) => continue,
                };
                let scalars = match flatten_motor_scalars(decoded) {
                    Some(scalars) => scalars,
                    None => continue,
                };
                let axis_count = scalars.len() as u32;
                for (axis, value) in scalars.into_iter().enumerate() {
                    let flattened_channel = if axis_count <= 1 {
                        channel as u32
                    } else {
                        channel as u32 * axis_count + axis as u32
                    };
                    snapshot.push(DecodedMotorValue {
                        group,
                        channel: flattened_channel,
                        mode,
                        value,
                    });
                }
            }
        }
        snapshot
    }

    /// Override PositionalServo preprocessed cache value for a channel.
    ///
    /// This seeds the decoder's internal integration state to a known live value.
    pub fn motor_positional_servo_write_preprocessed_cache_value(
        &mut self,
        unit: CorticalUnitIndex,
        channel: CorticalChannelIndex,
        value: Percentage,
    ) -> Result<(), FeagiDataError> {
        self.try_write_preprocessed_cached_value(
            MotorCorticalUnit::PositionalServo,
            unit,
            channel,
            WrappedIOData::Percentage(value),
        )
    }

    //endregion

    //region Internal

    //region Cache Abstractions

    fn register(
        &mut self,
        motor_type: MotorCorticalUnit,
        unit_index: CorticalUnitIndex,
        neuron_decoder: Box<dyn NeuronVoxelXYZPDecoder>,
        io_configuration_flags: serde_json::Map<String, serde_json::Value>,
        number_channels: CorticalChannelCount,
        initial_cached_value: WrappedIOData,
    ) -> Result<(), FeagiDataError> {
        // NOTE: The length of pipeline_stages_across_channels denotes the number of channels!

        if self
            .motor_cortical_unit_caches
            .contains_key(&(motor_type, unit_index))
        {
            return Err(feagi_data_etc_error(format!(
                "Already registered motor {} of unit index {}!",
                motor_type, unit_index
            )));
        }

        self.motor_cortical_unit_caches.insert(
            (motor_type, unit_index),
            MotorCorticalUnitCache::new(
                neuron_decoder,
                io_configuration_flags,
                number_channels,
                initial_cached_value,
            )?,
        );

        Ok(())
    }

    //region Data

    fn try_read_preprocessed_cached_value(
        &self,
        motor_type: MotorCorticalUnit,
        unit_index: CorticalUnitIndex,
        channel_index: CorticalChannelIndex,
    ) -> Result<&WrappedIOData, FeagiDataError> {
        let motor_stream_caches =
            self.try_get_motor_channel_stream_caches(motor_type, unit_index)?;
        motor_stream_caches.get_preprocessed_motor_value(channel_index)
    }

    fn try_read_postprocessed_cached_value(
        &self,
        motor_type: MotorCorticalUnit,
        unit_index: CorticalUnitIndex,
        channel_index: CorticalChannelIndex,
    ) -> Result<&WrappedIOData, FeagiDataError> {
        let motor_stream_caches =
            self.try_get_motor_channel_stream_caches(motor_type, unit_index)?;
        motor_stream_caches.get_postprocessed_motor_value(channel_index)
    }

    fn try_write_preprocessed_cached_value(
        &mut self,
        motor_type: MotorCorticalUnit,
        unit_index: CorticalUnitIndex,
        channel_index: CorticalChannelIndex,
        value: WrappedIOData,
    ) -> Result<(), FeagiDataError> {
        let motor_stream_caches =
            self.try_get_motor_channel_stream_caches_mut(motor_type, unit_index)?;
        motor_stream_caches.try_set_preprocessed_motor_value(channel_index, value)
    }

    fn try_register_motor_callback<F>(
        &mut self,
        motor_type: MotorCorticalUnit,
        unit_index: CorticalUnitIndex,
        channel_index: CorticalChannelIndex,
        callback: F,
    ) -> Result<FeagiSignalIndex, FeagiDataError>
    where
        F: Fn(&WrappedIOData) + Send + Sync + 'static,
    {
        let motor_stream_caches =
            self.try_get_motor_channel_stream_caches_mut(motor_type, unit_index)?;
        let index =
            motor_stream_caches.try_connect_to_data_processed_signal(channel_index, callback)?;
        Ok(index)
    }

    //endregion

    //region Stages

    fn try_get_single_stage_properties(
        &self,
        motor_type: MotorCorticalUnit,
        unit_index: CorticalUnitIndex,
        channel_index: CorticalChannelIndex,
        stage_index: PipelineStagePropertyIndex,
    ) -> Result<PipelineStageProperties, FeagiDataError> {
        let motor_stream_caches =
            self.try_get_motor_channel_stream_caches(motor_type, unit_index)?;
        motor_stream_caches.try_get_single_stage_properties(channel_index, stage_index)
    }

    fn try_get_all_stage_properties(
        &self,
        motor_type: MotorCorticalUnit,
        unit_index: CorticalUnitIndex,
        channel_index: CorticalChannelIndex,
    ) -> Result<Vec<PipelineStageProperties>, FeagiDataError> {
        let motor_stream_caches =
            self.try_get_motor_channel_stream_caches(motor_type, unit_index)?;
        motor_stream_caches.get_all_stage_properties(channel_index)
    }

    fn try_update_single_stage_properties(
        &mut self,
        motor_type: MotorCorticalUnit,
        unit_index: CorticalUnitIndex,
        channel_index: CorticalChannelIndex,
        pipeline_stage_property_index: PipelineStagePropertyIndex,
        replacing_property: PipelineStageProperties,
    ) -> Result<(), FeagiDataError> {
        let motor_stream_caches =
            self.try_get_motor_channel_stream_caches_mut(motor_type, unit_index)?;
        motor_stream_caches.try_update_single_stage_properties(
            channel_index,
            pipeline_stage_property_index,
            replacing_property,
        )
    }

    fn try_update_all_stage_properties(
        &mut self,
        motor_type: MotorCorticalUnit,
        unit_index: CorticalUnitIndex,
        channel_index: CorticalChannelIndex,
        new_pipeline_stage_properties: Vec<PipelineStageProperties>,
    ) -> Result<(), FeagiDataError> {
        let motor_stream_caches =
            self.try_get_motor_channel_stream_caches_mut(motor_type, unit_index)?;
        motor_stream_caches
            .try_update_all_stage_properties(channel_index, new_pipeline_stage_properties)
    }

    fn try_replace_single_stage(
        &mut self,
        motor_type: MotorCorticalUnit,
        unit_index: CorticalUnitIndex,
        channel_index: CorticalChannelIndex,
        replacing_at_index: PipelineStagePropertyIndex,
        new_pipeline_stage_properties: PipelineStageProperties,
    ) -> Result<(), FeagiDataError> {
        let motor_stream_caches =
            self.try_get_motor_channel_stream_caches_mut(motor_type, unit_index)?;
        motor_stream_caches.try_replace_single_stage(
            channel_index,
            replacing_at_index,
            new_pipeline_stage_properties,
        )
    }

    fn try_replace_all_stages(
        &mut self,
        motor_type: MotorCorticalUnit,
        unit_index: CorticalUnitIndex,
        channel_index: CorticalChannelIndex,
        new_pipeline_stage_properties: Vec<PipelineStageProperties>,
    ) -> Result<(), FeagiDataError> {
        let motor_stream_caches =
            self.try_get_motor_channel_stream_caches_mut(motor_type, unit_index)?;
        motor_stream_caches.try_replace_all_stages(channel_index, new_pipeline_stage_properties)
    }

    fn try_removing_all_stages(
        &mut self,
        sensor_type: MotorCorticalUnit,
        unit_index: CorticalUnitIndex,
        channel_index: CorticalChannelIndex,
    ) -> Result<(), FeagiDataError> {
        let motor_stream_cache =
            self.try_get_motor_channel_stream_caches_mut(sensor_type, unit_index)?;
        motor_stream_cache.try_removing_all_stages(channel_index)?;
        Ok(())
    }

    //endregion

    //endregion

    //region Hashmap Interactions

    fn try_get_motor_channel_stream_caches(
        &self,
        motor_type: MotorCorticalUnit,
        unit_index: CorticalUnitIndex,
    ) -> Result<&MotorCorticalUnitCache, FeagiDataError> {
        let check = self
            .motor_cortical_unit_caches
            .get(&(motor_type, unit_index));
        if check.is_none() {
            return Err(feagi_data_etc_error(format!(
                "Unable to find {} of cortical_area unit index {} in registered motor's list!",
                motor_type, unit_index
            )));
        }
        let check = check.unwrap();
        Ok(check)
    }

    fn try_get_motor_channel_stream_caches_mut(
        &mut self,
        motor_type: MotorCorticalUnit,
        unit_index: CorticalUnitIndex,
    ) -> Result<&mut MotorCorticalUnitCache, FeagiDataError> {
        let check = self
            .motor_cortical_unit_caches
            .get_mut(&(motor_type, unit_index));
        if check.is_none() {
            return Err(feagi_data_etc_error(format!(
                "Unable to find {} of cortical_area unit index {} in registered motor's list!",
                motor_type, unit_index
            )));
        }
        let check = check.unwrap();
        Ok(check)
    }

    //endregion

    //endregion
}

impl Default for MotorDeviceCache {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for MotorDeviceCache {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(writeln!(f, "Motor Device Cache:")?)
    }
}
