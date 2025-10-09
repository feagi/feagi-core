//! Tests for the data pipeline module - focusing on end -> end tests

use feagi_connector_core::data_types::descriptors::ColorSpace;
use feagi_connector_core::data_types::ImageFrame;

//region Helpers


fn load_bird_image() -> ImageFrame {
    let bird_bytes = std::fs::read("tests/images/bird.jpg").expect("Bird image should exist");
    ImageFrame::new_from_jpeg_bytes(&bird_bytes, &ColorSpace::Gamma).expect("Bird image should load correctly")
}



//endregion

#[cfg(test)]
mod test_connector_cache_sensor_load_image {
    use std::time::Instant;
    use feagi_connector_core::data_types::descriptors::{ColorChannelLayout, ColorSpace, GazeProperties, ImageXYResolution, SegmentedImageFrameProperties, SegmentedXYImageResolutions};
    use feagi_data_structures::genomic::descriptors::{CorticalChannelCount, CorticalChannelIndex, CorticalGroupIndex};
    use crate::load_bird_image;

    #[test]
    fn test_segment_bird_image() {
        let time_of_previous_burst: Instant = Instant::now(); // Pretend

        let cortical_group: CorticalGroupIndex = 0.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();
        let segmented_image_resolutions: SegmentedXYImageResolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            (256, 256).try_into().unwrap(), (128, 128).try_into().unwrap()
        );


        let bird_image = load_bird_image();
        let bird_image_properties = bird_image.get_image_frame_properties();
        let segmented_bird_properties = SegmentedImageFrameProperties::new(
            &segmented_image_resolutions,
            &bird_image_properties.get_color_channel_layout(),
            &bird_image_properties.get_color_channel_layout(),
            &bird_image_properties.get_color_space());
        let initial_gaze = GazeProperties::new((0.5, 0.5).try_into().unwrap(), (0.5, 0.5).try_into().unwrap());


        let mut connector_cache = feagi_connector_core::IOCache::new();
        connector_cache.sensor_register_segmented_vision_absolute(cortical_group, number_channels, bird_image_properties, segmented_bird_properties, initial_gaze).unwrap();
        connector_cache.sensor_write_segmented_vision_absolute(cortical_group, channel_index, &bird_image.into()).unwrap();
        let bytes = connector_cache.sensor_get_bytes().unwrap();
    }

    #[test]
    fn test_segment_bird_image_with_moving_gaze() {
        let time_of_previous_burst: Instant = Instant::now(); // Pretend

        let cortical_group: CorticalGroupIndex = 0.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();
        let segmented_image_resolutions: SegmentedXYImageResolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            (256, 256).try_into().unwrap(), (128, 128).try_into().unwrap()
        );


        let bird_image = load_bird_image();
        let bird_image_properties = bird_image.get_image_frame_properties();
        let segmented_bird_properties = SegmentedImageFrameProperties::new(
            &segmented_image_resolutions,
            &bird_image_properties.get_color_channel_layout(),
            &bird_image_properties.get_color_channel_layout(),
            &bird_image_properties.get_color_space());
        let initial_gaze = GazeProperties::new((0.5, 0.5).try_into().unwrap(), (0.5, 0.5).try_into().unwrap());
        let second_gaze = GazeProperties::new((0.3, 0.3).try_into().unwrap(), (0.2, 0.3).try_into().unwrap());


        let mut connector_cache = feagi_connector_core::IOCache::new();
        connector_cache.sensor_register_segmented_vision_absolute(cortical_group, number_channels, bird_image_properties, segmented_bird_properties, initial_gaze).unwrap();
        connector_cache.motor_register_gaze_absolute_linear(cortical_group, number_channels, 10.try_into().unwrap());

        connector_cache.sensor_write_segmented_vision_absolute(cortical_group, channel_index, &bird_image.into()).unwrap();

        // TODO motor bytes sending

        let sensor_bytes = connector_cache.sensor_get_bytes().unwrap();
        let motor_data = connector_cache.motor_try_read_postprocessed_cached_value_gaze_absolute_linear(cortical_group, channel_index).unwrap();
    }

}