use crate::caching::{MotorDeviceCache, SensorDeviceCache};
use crate::data_pipeline::{PipelineStageProperties, PipelineStagePropertyIndex};
use crate::data_types::descriptors::{
    ColorChannelLayout, ColorSpace, ImageFrameProperties, SegmentedImageFrameProperties,
    SegmentedXYImageResolutions,
};
use crate::data_types::{GazeProperties, ImageFilteringSettings};
use crate::feedbacks::feedback_registrar::FeedbackRegistrar;
use crate::feedbacks::feedback_registration_targets::FeedbackRegistrationTargets;
use crate::wrapped_io_data::WrappedIOData;
use feagi_structures::genomic::{MotorCorticalUnit, SensoryCorticalUnit};
use feagi_structures::{FeagiDataError, FeagiSignalIndex};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FeedBackRegistration {
    SegmentedVisionWithGaze {},
    SegmentedVisionWithImageFiltering {},
    VisionWithImageFiltering {},
}

impl Display for FeedBackRegistration {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FeedBackRegistration::SegmentedVisionWithGaze {} => {
                write!(f, "SegmentedVisionWithGaze")
            }
            FeedBackRegistration::SegmentedVisionWithImageFiltering {} => {
                write!(f, "SegmentedVisionWithImageFiltering")
            }
            FeedBackRegistration::VisionWithImageFiltering {} => {
                write!(f, "VisionWithImageFiltering")
            }
        }
    }
}

impl FeedBackRegistration {
    pub(crate) fn try_registering_feedback_and_save(
        &self,
        feedback_registrar: &mut FeedbackRegistrar,
        sensor_cache: Arc<Mutex<SensorDeviceCache>>,
        motor_cache: Arc<Mutex<MotorDeviceCache>>,
        target: FeedbackRegistrationTargets,
    ) -> Result<(), FeagiDataError> {
        self.try_registering_feedbacks(sensor_cache, motor_cache, target.clone())?;
        let _ = feedback_registrar.push_verified_feedback(target, self.clone());
        Ok(())
    }

    pub(crate) fn try_registering_feedbacks(
        &self,
        sensor_cache: Arc<Mutex<SensorDeviceCache>>,
        motor_cache: Arc<Mutex<MotorDeviceCache>>,
        target: FeedbackRegistrationTargets,
    ) -> Result<(), FeagiDataError> {
        let (sensory_cortical_unit, motor_cortical_unit) = self.get_sensor_motor_cortical_units();

        let target_stage_index: PipelineStagePropertyIndex;

        // Verify required units/channels exist.
        {
            let sensors = sensor_cache.lock().unwrap();
            sensors.verify_existence(
                sensory_cortical_unit,
                target.get_sensor_unit_index(),
                target.get_sensor_channel_index(),
            )?;
            target_stage_index = sensors.try_get_index_of_first_stage_property_type_of(
                sensory_cortical_unit,
                target.get_sensor_unit_index(),
                target.get_sensor_channel_index(),
                &self.create_example_property(),
            )?
        }
        {
            let motors = motor_cache.lock().unwrap();
            motors.verify_existence(
                motor_cortical_unit,
                target.get_motor_unit_index(),
                target.get_motor_channel_index(),
            )?;
        }

        match self {
            FeedBackRegistration::SegmentedVisionWithGaze {} => {
                feedback_segmented_vision_with_gaze(
                    &target,
                    sensor_cache.clone(),
                    motor_cache.clone(),
                    target_stage_index,
                )?;
            }
            FeedBackRegistration::SegmentedVisionWithImageFiltering {} => {
                feedback_segmented_vision_with_image_filtering(
                    &target,
                    sensor_cache.clone(),
                    motor_cache.clone(),
                    target_stage_index,
                )?;
            }
            FeedBackRegistration::VisionWithImageFiltering {} => {
                feedback_simple_vision_with_image_filtering(
                    &target,
                    sensor_cache.clone(),
                    motor_cache.clone(),
                    target_stage_index,
                )?;
            }
        }
        Ok(())
    }

    fn get_sensor_motor_cortical_units(&self) -> (SensoryCorticalUnit, MotorCorticalUnit) {
        match &self {
            FeedBackRegistration::SegmentedVisionWithGaze {} => (
                SensoryCorticalUnit::SegmentedVision,
                MotorCorticalUnit::Gaze,
            ),
            FeedBackRegistration::SegmentedVisionWithImageFiltering {} => (
                SensoryCorticalUnit::SegmentedVision,
                MotorCorticalUnit::DynamicImageProcessing,
            ),
            FeedBackRegistration::VisionWithImageFiltering {} => (
                SensoryCorticalUnit::Vision,
                MotorCorticalUnit::DynamicImageProcessing,
            ),
        }
    }

    fn create_example_property(&self) -> PipelineStageProperties {
        match &self {
            FeedBackRegistration::SegmentedVisionWithGaze {} => {
                PipelineStageProperties::ImageFrameSegmentator {
                    input_image_properties: ImageFrameProperties::new(
                        (1, 1).try_into().unwrap(),
                        ColorSpace::Linear,
                        ColorChannelLayout::GrayScale,
                    )
                    .unwrap(),
                    output_image_properties: SegmentedImageFrameProperties::new(
                        SegmentedXYImageResolutions::create_with_same_sized_peripheral(
                            (1, 1).try_into().unwrap(),
                            (1, 1).try_into().unwrap(),
                        ),
                        ColorChannelLayout::GrayScale,
                        ColorChannelLayout::GrayScale,
                        ColorSpace::Linear,
                    ),
                    segmentation_gaze: GazeProperties::create_default_centered(),
                }
            }
            FeedBackRegistration::SegmentedVisionWithImageFiltering {} => {
                todo!()
            }
            FeedBackRegistration::VisionWithImageFiltering {} => {
                PipelineStageProperties::ImageQuickDiff {
                    per_pixel_allowed_range: 0..=1,
                    acceptable_amount_of_activity_in_image: 0.0.try_into().unwrap()
                        ..=0.1.try_into().unwrap(),
                    image_properties: ImageFrameProperties::new(
                        (1, 1).try_into().unwrap(),
                        ColorSpace::Linear,
                        ColorChannelLayout::GrayScale,
                    )
                    .unwrap(),
                }
            }
        }
    }
}

fn feedback_segmented_vision_with_gaze(
    target: &FeedbackRegistrationTargets,
    sensors: Arc<Mutex<SensorDeviceCache>>,
    motors: Arc<Mutex<MotorDeviceCache>>,
    stage_index: PipelineStagePropertyIndex,
) -> Result<FeagiSignalIndex, FeagiDataError> {
    let sensor_unit = target.get_sensor_unit_index();
    let sensor_channel = target.get_sensor_channel_index();

    let sensor_ref = sensors.clone();

    let closure = move |wrapped_data: &WrappedIOData| {
        let gaze_properties: GazeProperties = wrapped_data.try_into().unwrap();

        let mut sensors = sensor_ref.lock().unwrap();
        let stage_properties = sensors
            .segmented_vision_get_single_stage_properties(sensor_unit, sensor_channel, stage_index)
            .unwrap();
        let new_properties: PipelineStageProperties = match stage_properties {
            PipelineStageProperties::ImageFrameSegmentator {
                input_image_properties,
                output_image_properties,
                segmentation_gaze: _,
            } => PipelineStageProperties::ImageFrameSegmentator {
                input_image_properties,
                output_image_properties,
                segmentation_gaze: gaze_properties,
            },
            _ => {
                panic!("Invalid pipeline stage properties for segmented gaze vision feedback!")
            }
        };

        _ = sensors.segmented_vision_replace_single_stage(
            sensor_unit,
            sensor_channel,
            0.into(),
            new_properties,
        );
    };

    let motor_ref = motors.clone();
    let mut motors = motor_ref.lock().unwrap();

    let index = motors.gaze_try_register_motor_callback(
        target.get_motor_unit_index(),
        target.get_motor_channel_index(),
        closure,
    )?;
    Ok(index)
}

fn feedback_segmented_vision_with_image_filtering(
    target: &FeedbackRegistrationTargets,
    sensors: Arc<Mutex<SensorDeviceCache>>,
    motors: Arc<Mutex<MotorDeviceCache>>,
    stage_index: PipelineStagePropertyIndex,
) -> Result<FeagiSignalIndex, FeagiDataError> {
    let sensor_unit = target.get_sensor_unit_index();
    let sensor_channel = target.get_sensor_channel_index();

    let sensor_ref = sensors.clone();

    let closure = move |wrapped_data: &WrappedIOData| {
        let image_filtering_settings: ImageFilteringSettings = wrapped_data.try_into().unwrap();

        let mut sensors = sensor_ref.lock().unwrap();
        let stage_properties = sensors
            .segmented_vision_get_single_stage_properties(sensor_unit, sensor_channel, stage_index)
            .unwrap();
        let new_properties: PipelineStageProperties = match stage_properties {
            PipelineStageProperties::ImageQuickDiff {
                image_properties, ..
            } => {
                let pixel_range = image_filtering_settings
                    .per_pixel_diff_threshold
                    .a
                    .get_as_u8()
                    ..=image_filtering_settings
                        .per_pixel_diff_threshold
                        .b
                        .get_as_u8();
                let image_range = image_filtering_settings.image_diff_threshold.a
                    ..=image_filtering_settings.image_diff_threshold.b;

                PipelineStageProperties::ImageQuickDiff {
                    per_pixel_allowed_range: pixel_range,
                    acceptable_amount_of_activity_in_image: image_range,
                    image_properties,
                }
            }
            _ => {
                panic!("Invalid pipeline stage properties for image transform!")
            }
        };

        _ = sensors.segmented_vision_replace_single_stage(
            sensor_unit,
            sensor_channel,
            0.into(),
            new_properties,
        );
    };

    let motor_ref = motors.clone();
    let mut motors = motor_ref.lock().unwrap();

    let index = motors.dynamic_image_processing_try_register_motor_callback(
        target.get_motor_unit_index(),
        target.get_motor_channel_index(),
        closure,
    )?;
    Ok(index)
}

fn feedback_simple_vision_with_image_filtering(
    target: &FeedbackRegistrationTargets,
    sensors: Arc<Mutex<SensorDeviceCache>>,
    motors: Arc<Mutex<MotorDeviceCache>>,
    stage_index: PipelineStagePropertyIndex,
) -> Result<FeagiSignalIndex, FeagiDataError> {
    let sensor_unit = target.get_sensor_unit_index();
    let sensor_channel = target.get_sensor_channel_index();

    let sensor_ref = sensors.clone();

    let closure = move |wrapped_data: &WrappedIOData| {
        let image_filtering_settings: ImageFilteringSettings = wrapped_data.try_into().unwrap();

        let mut sensors = sensor_ref.lock().unwrap();
        let stage_properties = sensors
            .vision_get_single_stage_properties(sensor_unit, sensor_channel, stage_index)
            .unwrap();
        let new_properties: PipelineStageProperties = match stage_properties {
            PipelineStageProperties::ImageQuickDiff {
                image_properties, ..
            } => {
                let pixel_range = image_filtering_settings
                    .per_pixel_diff_threshold
                    .a
                    .get_as_u8()
                    ..=image_filtering_settings
                        .per_pixel_diff_threshold
                        .b
                        .get_as_u8();
                let image_range = image_filtering_settings.image_diff_threshold.a
                    ..=image_filtering_settings.image_diff_threshold.b;

                PipelineStageProperties::ImageQuickDiff {
                    per_pixel_allowed_range: pixel_range,
                    acceptable_amount_of_activity_in_image: image_range,
                    image_properties,
                }
            }
            _ => {
                panic!("Invalid pipeline stage properties for image transform!")
            }
        };

        _ = sensors.vision_replace_single_stage(
            sensor_unit,
            sensor_channel,
            0.into(),
            new_properties,
        );
    };

    let motor_ref = motors.clone();
    let mut motors = motor_ref.lock().unwrap();

    let index = motors.dynamic_image_processing_try_register_motor_callback(
        target.get_motor_unit_index(),
        target.get_motor_channel_index(),
        closure,
    )?;
    Ok(index)
}
