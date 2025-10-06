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
    use feagi_connector_core::data_types::descriptors::{ColorChannelLayout, ColorSpace, GazeProperties, ImageXYResolution, SegmentedImageFrameProperties, SegmentedXYImageResolutions};
    use feagi_data_structures::genomic::descriptors::{CorticalChannelCount, CorticalChannelIndex, CorticalGroupIndex};
    use crate::load_bird_image;

    #[test]
    fn test_segment_bird_image() {

        const CORTICAL_GROUP: CorticalGroupIndex = CorticalGroupIndex(0);
        const NUMBER_CHANNELS: CorticalChannelCount = CorticalChannelCount(1);
        const CHANNEL_INDEX: CorticalChannelIndex = CorticalChannelIndex(0);
        const SEGMENTED_IMAGE_RESOLUTIONS: SegmentedXYImageResolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            (64, 64).try_into().unwrap(), (32, 32).try_into().unwrap()
        );


        let bird_image = load_bird_image();
        let bird_image_properties = bird_image.get_image_frame_properties();
        let segmented_bird_properties = SegmentedImageFrameProperties::new(
            &SEGMENTED_IMAGE_RESOLUTIONS,
            &bird_image_properties.get_color_channel_layout(),
            &bird_image_properties.get_color_channel_layout(),
            &bird_image_properties.get_color_space());
        let initial_gaze = GazeProperties::new((0.5, 0.5), (0.5, 0.5))

        let mut connector_cache = feagi_connector_core::IOCache::new();

        connector_cache.sensor_register_segmented_vision_absolute(CORTICAL_GROUP, NUMBER_CHANNELS, bird_image_properties, segmented_bird_properties, ()).unwrap()


    }

}