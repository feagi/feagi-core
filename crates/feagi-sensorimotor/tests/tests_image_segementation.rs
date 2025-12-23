//! Tests for image segmentation functionality using the public API.
//!
//! These tests verify image segmentation through the ConnectorAgent and sensor cache,
//! testing various gaze positions, resolutions, and color channel configurations.

use std::io::Read;
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelIndex, CorticalGroupIndex,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_data_type::FrameChangeHandling;
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

fn save_png_image(image: &ImageFrame, subpath: String) {
    let image_bytes = image.export_as_png_bytes().unwrap();
    let fullpath = format!("tests/images/{}", subpath);
    let path = std::path::Path::new(&fullpath);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&fullpath, image_bytes).unwrap()
}


//endregion


#[cfg(test)]
mod test_segmented_images {
    use feagi_sensorimotor::data_types::descriptors::{SegmentedImageFrameProperties, SegmentedXYImageResolutions};
    use feagi_sensorimotor::data_types::{GazeProperties, ImageFrame, Percentage, Percentage2D, SegmentedImageFrame};
    use feagi_sensorimotor::data_types::processing::ImageFrameSegmentator;
    use crate::{load_bird_image, save_png_image};

    #[test]
    fn test_segment_bird_image() {
        fn write_with_gaze(image_frame_segmentator: &mut ImageFrameSegmentator, image: &ImageFrame, segmented_image: &mut SegmentedImageFrame, new_gaze: &GazeProperties, gaze_name: &str) {
            image_frame_segmentator.update_gaze(new_gaze).unwrap();
            image_frame_segmentator.segment_image(image, segmented_image).unwrap();

            save_png_image(segmented_image.get_image_lower_left(), format!("{}/0_lower_left.png", gaze_name));
            save_png_image(segmented_image.get_image_lower_middle(), format!("{}/1_lower_middle.png", gaze_name));
            save_png_image(segmented_image.get_image_lower_right(), format!("{}/2_lower_right.png", gaze_name));
            save_png_image(segmented_image.get_image_middle_left(), format!("{}/3_middle_left.png", gaze_name));
            save_png_image(segmented_image.get_image_center(), format!("{}/4_center.png", gaze_name));
            save_png_image(segmented_image.get_image_middle_right(), format!("{}/5_middle_right.png", gaze_name));
            save_png_image(segmented_image.get_image_upper_left(), format!("{}/6_upper_left.png", gaze_name));
            save_png_image(segmented_image.get_image_upper_middle(), format!("{}/7_upper_middle.png", gaze_name));
            save_png_image(segmented_image.get_image_upper_right(), format!("{}/8_upper_right.png", gaze_name));
        }

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

        let gaze =
            GazeProperties::create_default_centered();

        let mut image_frame_segmentator = ImageFrameSegmentator::new(
            bird_image_properties,
            segmented_bird_properties,
            gaze
        ).unwrap();

        let mut segmented_output = image_frame_segmentator.create_blank_segmented_image_for_use_as_write_cache();
        image_frame_segmentator.verify_input_image(&bird_image).unwrap();

        // center
        let gaze = GazeProperties::create_default_centered();
        write_with_gaze(&mut image_frame_segmentator, &bird_image, &mut segmented_output,
                        &gaze, "center_gaze");

        // down
        let gaze = GazeProperties::new(
            Percentage2D::new(
                Percentage::new_from_0_1(0.5).unwrap(),
                Percentage::new_from_0_1(0.25).unwrap()
            ),
            Percentage::new_from_0_1(0.5).unwrap()
        );
        write_with_gaze(&mut image_frame_segmentator, &bird_image, &mut segmented_output,
                        &gaze, "down_gaze");

        // left down
        let gaze = GazeProperties::new(
            Percentage2D::new(
                Percentage::new_from_0_1(0.25).unwrap(),
                Percentage::new_from_0_1(0.25).unwrap()
            ),
            Percentage::new_from_0_1(0.5).unwrap()
        );
        write_with_gaze(&mut image_frame_segmentator, &bird_image, &mut segmented_output,
                        &gaze, "left_down_gaze");

        // down most bottom
        let gaze = GazeProperties::new(
            Percentage2D::new(
                Percentage::new_from_0_1(0.5).unwrap(),
                Percentage::new_from_0_1(0.0).unwrap()
            ),
            Percentage::new_from_0_1(0.5).unwrap()
        );
        write_with_gaze(&mut image_frame_segmentator, &bird_image, &mut segmented_output,
                        &gaze, "down_bottom_gaze");

        // almost entire screen
        let gaze = GazeProperties::new(
            Percentage2D::new(
                Percentage::new_from_0_1(0.2).unwrap(), // these shouldnt matter much in this case
                Percentage::new_from_0_1(0.0).unwrap()
            ),
            Percentage::new_from_0_1(1.0).unwrap()
        );
        write_with_gaze(&mut image_frame_segmentator, &bird_image, &mut segmented_output,
                        &gaze, "whole_screen");
    }


}
