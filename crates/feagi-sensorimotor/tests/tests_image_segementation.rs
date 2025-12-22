//! Tests for image segmentation functionality using the public API.
//!
//! These tests verify image segmentation through the ConnectorAgent and sensor cache,
//! testing various gaze positions, resolutions, and color channel configurations.

use std::io::Read;
use feagi_data_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelIndex, CorticalGroupIndex,
};
use feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::FrameChangeHandling;
use feagi_sensorimotor::data_types::descriptors::{
    ColorChannelLayout, ColorSpace, SegmentedImageFrameProperties, SegmentedXYImageResolutions,
};
use feagi_sensorimotor::data_types::{GazeProperties, ImageFrame, Percentage, Percentage2D};
use feagi_sensorimotor::wrapped_io_data::WrappedIOData;
use feagi_sensorimotor::ConnectorAgent;

//region Helper Functions

#[allow(dead_code)]
fn load_bird_image() -> ImageFrame {
    let bird_bytes = std::fs::read("tests/images/bird.jpg")
        .expect("Bird image should exist at tests/images/bird.jpg");
    ImageFrame::new_from_jpeg_bytes(&bird_bytes, &ColorSpace::Gamma)
        .expect("Bird image should load correctly")
}

fn save_png_image(image: &ImageFrame, subpath: &str) {
    let image_bytes = image.export_as_png_bytes().unwrap();
    let fullpath = "/tests/images/".to_string() + subpath;
    std::fs::write(fullpath, image_bytes).unwrap()
}


//endregion


#[cfg(test)]
mod test_segmented_images {
    use feagi_sensorimotor::data_types::descriptors::{SegmentedImageFrameProperties, SegmentedXYImageResolutions};
    use feagi_sensorimotor::data_types::GazeProperties;
    use feagi_sensorimotor::data_types::processing::ImageFrameSegmentator;
    use crate::{load_bird_image, save_png_image};

    #[test]
    fn test_segment_bird_image() {
        let segmented_image_resolutions: SegmentedXYImageResolutions =
            SegmentedXYImageResolutions::create_with_same_sized_peripheral(
                (256, 256).try_into().unwrap(),
                (128, 128).try_into().unwrap(),
            );
        let bird_image = load_bird_image();
        let bird_image_properties = bird_image.get_image_frame_properties();
        let segmented_bird_properties = SegmentedImageFrameProperties::new(
            segmented_image_resolutions,
            bird_image_properties.get_color_channel_layout(),
            bird_image_properties.get_color_channel_layout(),
            bird_image_properties.get_color_space(),
        );

        let initial_gaze =
            GazeProperties::create_default_centered();

        let mut image_frame_segmentator = ImageFrameSegmentator::new(
            bird_image_properties,
            segmented_bird_properties,
            initial_gaze
        ).unwrap();

        let mut segmented_output = image_frame_segmentator.create_blank_segmented_image_for_use_as_write_cache();
        image_frame_segmentator.verify_input_image(&bird_image).unwrap();

        // center
        image_frame_segmentator.segment_image(&bird_image, &mut segmented_output).unwrap();
        save_png_image(segmented_output.get_image_lower_left(), "center_gaze/0_lower_left.png");
        save_png_image(segmented_output.get_image_lower_middle(), "center_gaze/1_lower_middle.png");
        save_png_image(segmented_output.get_image_lower_right(), "center_gaze/2_lower_right.png");
        save_png_image(segmented_output.get_image_middle_left(), "center_gaze/3_middle_left.png");
        save_png_image(segmented_output.get_image_center(), "center_gaze/4_center.png");
        save_png_image(segmented_output.get_image_middle_right(), "center_gaze/5_middle_right.png");
        save_png_image(segmented_output.get_image_upper_left(), "center_gaze/6_upper_left.png");
        save_png_image(segmented_output.get_image_upper_middle(), "center_gaze/7_upper_middle.png");
        save_png_image(segmented_output.get_image_upper_right(), "center_gaze/8_upper_right.png");
    }


}
