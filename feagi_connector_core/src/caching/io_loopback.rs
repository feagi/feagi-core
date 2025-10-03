use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::descriptors::{CorticalChannelIndex, CorticalGroupIndex};
use feagi_data_structures::genomic::{MotorCorticalType, SensorCorticalType};
use crate::caching::io_motor_cache::IOMotorCache;
use crate::caching::io_sensor_cache::IOSensorCache;
use crate::data_pipeline::{PipelineStageProperties, PipelineStagePropertyIndex};
use crate::data_pipeline::stage_properties::ImageSegmentorStageProperties;
use crate::data_types::descriptors::{GazeProperties, ImageFrameProperties, SegmentedImageFrameProperties};
use crate::data_types::Percentage4D;

pub fn loopback_absolute_gaze_to_segmentation(sensors: &mut IOSensorCache, motors: &IOMotorCache,
                                     gaze_group: CorticalGroupIndex, gaze_channel: CorticalChannelIndex,
                                     segmentation_group: CorticalGroupIndex,
                                     segmentation_channel: CorticalChannelIndex)
                                     -> Result<(), FeagiDataError> {


    fn update_segmentator_from_gaze(sensors: &mut IOSensorCache, motors: &IOMotorCache, gaze_group: CorticalGroupIndex, gaze_channel: CorticalChannelIndex, segmentation_group: CorticalGroupIndex, segmentation_channel: CorticalChannelIndex) {
        // Get current segmentator property
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        let temp: PipelineStagePropertyIndex = 0.into(); // TODO fIX me!
        let stage = sensors.get_pipeline_stage_properties(SENSOR_TYPE, segmentation_group, segmentation_channel, temp).unwrap();
        let mut segmentator_property: ImageSegmentorStageProperties = stage.as_any().downcast_ref::<ImageSegmentorStageProperties>().unwrap().clone();

        // get new gaze data
        const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::GazeAbsoluteLinear;
        let recent_output = motors.try_read_postprocessed_cached_value(MOTOR_TYPE, gaze_group, gaze_channel).unwrap();
        let percentage_output: Percentage4D = recent_output.try_into().unwrap();
        let new_gaze = GazeProperties::new_4d(percentage_output);

        // overwrite and apply
        segmentator_property.update_from_gaze(new_gaze).unwrap();
        let segmentator_property: Box<dyn PipelineStageProperties + Send + Sync + 'static> = Box::new(segmentator_property);
        sensors.try_updating_pipeline_stage(SENSOR_TYPE, segmentation_group, segmentation_channel, temp, segmentator_property).unwrap()
    }




    Ok(())
}