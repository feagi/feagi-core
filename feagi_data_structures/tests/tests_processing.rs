//! Tests for the processing module
//! 
//! This module contains comprehensive tests for image processing functionality including
//! ImageFrameProcessor and related operations like cropping, resizing, and color space conversion.

use feagi_data_structures::data::{ImageFrame, SegmentedImageFrame};
use feagi_data_structures::data::descriptors::{ColorChannelLayout, ColorSpace, ImageXYResolution, ImageFrameProperties, CornerPoints, ImageXYPoint, SegmentedXYImageResolutions, SegmentedImageFrameProperties, GazeProperties};
use feagi_data_structures::processing::{ImageFrameProcessor, ImageFrameSegmentator};

#[cfg(test)]
mod test_image_frame_processor {
    use super::*;

    fn create_test_image(width: usize, height: usize, channels: &ColorChannelLayout, color_space: &ColorSpace) -> ImageFrame {
        let resolution = ImageXYResolution::new(width, height).unwrap();
        ImageFrame::new(channels, color_space, &resolution).unwrap()
    }

    fn create_test_image_with_pattern(width: usize, height: usize, channels: &ColorChannelLayout, color_space: &ColorSpace) -> ImageFrame {
        let mut frame = create_test_image(width, height, channels, color_space);
        {
            let mut pixels = frame.get_pixels_view_mut();
            for y in 0..height {
                for x in 0..width {
                    for c in 0..(*channels as usize) {
                        // Create a simple gradient pattern for testing
                        pixels[(y, x, c)] = ((x + y + c * 50) % 256) as u8;
                    }
                }
            }
        }
        frame
    }

    #[test]
    fn test_image_frame_processor_new() {
        let resolution = ImageXYResolution::new(100, 80).unwrap();
        let properties = ImageFrameProperties::new(resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
        
        let processor = ImageFrameProcessor::new(properties.clone());
        
        assert_eq!(*processor.get_input_image_properties(), properties);
        assert_eq!(processor.get_output_image_properties(), properties);
    }

    #[test]
    fn test_image_frame_processor_new_from_input_output_properties() {
        let input_resolution = ImageXYResolution::new(100, 80).unwrap();
        let output_resolution = ImageXYResolution::new(50, 40).unwrap();
        
        let input_props = ImageFrameProperties::new(input_resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
        let output_props = ImageFrameProperties::new(output_resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
        
        let processor = ImageFrameProcessor::new_from_input_output_properties(&input_props, &output_props).unwrap();
        
        assert_eq!(*processor.get_input_image_properties(), input_props);
        assert_eq!(processor.get_output_image_properties(), output_props);
    }

    #[test]
    fn test_image_frame_processor_new_from_input_output_properties_with_grayscale_conversion() {
        let resolution = ImageXYResolution::new(100, 80).unwrap();
        
        let input_props = ImageFrameProperties::new(resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
        let output_props = ImageFrameProperties::new(resolution, ColorSpace::Gamma, ColorChannelLayout::GrayScale).unwrap();
        
        let processor = ImageFrameProcessor::new_from_input_output_properties(&input_props, &output_props).unwrap();
        
        assert_eq!(*processor.get_input_image_properties(), input_props);
        assert_eq!(processor.get_output_image_properties(), output_props);
    }

    #[test]
    fn test_image_frame_processor_new_from_input_output_properties_unsupported_conversion() {
        let resolution = ImageXYResolution::new(100, 80).unwrap();
        
        let input_props = ImageFrameProperties::new(resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
        let output_props = ImageFrameProperties::new(resolution, ColorSpace::Gamma, ColorChannelLayout::RGBA).unwrap();
        
        let result = ImageFrameProcessor::new_from_input_output_properties(&input_props, &output_props);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_cropping_from() {
        let resolution = ImageXYResolution::new(100, 80).unwrap();
        let properties = ImageFrameProperties::new(resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
        
        let mut processor = ImageFrameProcessor::new(properties);
        
        let upper_left = ImageXYPoint::new(10, 15);
        let lower_right = ImageXYPoint::new(50, 45);
        let corner_points = CornerPoints::new(upper_left, lower_right).unwrap();
        
        let result = processor.set_cropping_from(corner_points);
        assert!(result.is_ok());
        
        // Verify output properties changed
        let output_props = processor.get_output_image_properties();
        assert_eq!(output_props.get_image_resolution().width, 40); // 50 - 10
        assert_eq!(output_props.get_image_resolution().height, 30); // 45 - 15
    }

    #[test]
    fn test_set_cropping_from_invalid_bounds() {
        let resolution = ImageXYResolution::new(100, 80).unwrap();
        let properties = ImageFrameProperties::new(resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
        
        let mut processor = ImageFrameProcessor::new(properties);
        
        // Try to crop outside the image bounds
        let upper_left = ImageXYPoint::new(10, 15);
        let lower_right = ImageXYPoint::new(150, 45); // x=150 is beyond width=100
        let corner_points = CornerPoints::new(upper_left, lower_right).unwrap();
        
        let result = processor.set_cropping_from(corner_points);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_resizing_to() {
        let resolution = ImageXYResolution::new(100, 80).unwrap();
        let properties = ImageFrameProperties::new(resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
        
        let mut processor = ImageFrameProcessor::new(properties);
        
        let new_resolution = ImageXYResolution::new(200, 160).unwrap();
        let result = processor.set_resizing_to(new_resolution);
        assert!(result.is_ok());
        
        // Verify output properties changed
        let output_props = processor.get_output_image_properties();
        assert_eq!(output_props.get_image_resolution(), new_resolution);
    }

    #[test]
    fn test_set_brightness_offset() {
        let resolution = ImageXYResolution::new(100, 80).unwrap();
        let properties = ImageFrameProperties::new(resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
        
        let mut processor = ImageFrameProcessor::new(properties);
        
        let result = processor.set_brightness_offset(50);
        assert!(result.is_ok());
        
        // Setting brightness offset to 0 should clear it
        let result = processor.set_brightness_offset(0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_contrast_change() {
        let resolution = ImageXYResolution::new(100, 80).unwrap();
        let properties = ImageFrameProperties::new(resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
        
        let mut processor = ImageFrameProcessor::new(properties);
        
        let result = processor.set_contrast_change(2.0);
        assert!(result.is_ok());
        
        // Setting contrast to 1.0 should clear it
        let result = processor.set_contrast_change(1.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_color_space_to() {
        let resolution = ImageXYResolution::new(100, 80).unwrap();
        let properties = ImageFrameProperties::new(resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
        
        let mut processor = ImageFrameProcessor::new(properties.clone());
        
        let result = processor.set_color_space_to(&ColorSpace::Linear);
        assert!(result.is_ok());
        
        // Verify output properties changed
        let output_props = processor.get_output_image_properties();
        assert_eq!(output_props.get_color_space(), ColorSpace::Linear);
        
        // Setting to same color space should clear the conversion
        let result = processor.set_color_space_to(&ColorSpace::Gamma);
        assert!(result.is_ok());
        let output_props = processor.get_output_image_properties();
        assert_eq!(output_props.get_color_space(), ColorSpace::Gamma);
    }

    #[test]
    fn test_set_conversion_to_grayscale() {
        let resolution = ImageXYResolution::new(100, 80).unwrap();
        let properties = ImageFrameProperties::new(resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
        
        let mut processor = ImageFrameProcessor::new(properties);
        
        let result = processor.set_conversion_to_grayscale(true);
        assert!(result.is_ok());
        
        // Verify output properties changed
        let output_props = processor.get_output_image_properties();
        assert_eq!(output_props.get_color_channel_layout(), ColorChannelLayout::GrayScale);
        
        // Turn off grayscale conversion
        let result = processor.set_conversion_to_grayscale(false);
        assert!(result.is_ok());
        let output_props = processor.get_output_image_properties();
        assert_eq!(output_props.get_color_channel_layout(), ColorChannelLayout::RGB);
    }

    #[test]
    fn test_set_conversion_to_grayscale_unsupported() {
        let resolution = ImageXYResolution::new(100, 80).unwrap();
        let properties = ImageFrameProperties::new(resolution, ColorSpace::Gamma, ColorChannelLayout::RG).unwrap();
        
        let mut processor = ImageFrameProcessor::new(properties);
        
        let result = processor.set_conversion_to_grayscale(true);
        assert!(result.is_err()); // RG to grayscale is not supported
    }

    #[test]
    fn test_clear_methods() {
        let resolution = ImageXYResolution::new(100, 80).unwrap();
        let properties = ImageFrameProperties::new(resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
        
        let mut processor = ImageFrameProcessor::new(properties.clone());
        
        // Set up some transformations
        let corner_points = CornerPoints::new(ImageXYPoint::new(10, 10), ImageXYPoint::new(50, 50)).unwrap();
        processor.set_cropping_from(corner_points).unwrap();
        processor.set_resizing_to(ImageXYResolution::new(200, 200).unwrap()).unwrap();
        processor.set_brightness_offset(50).unwrap();
        processor.set_contrast_change(2.0).unwrap();
        processor.set_color_space_to(&ColorSpace::Linear).unwrap();
        processor.set_conversion_to_grayscale(true).unwrap();
        
        // Clear individual transformations
        processor.clear_cropping();
        processor.clear_resizing();
        processor.clear_brightness_adjustment();
        processor.clear_contrast_adjustment();
        processor.clear_color_space_conversion();
        processor.clear_grayscale_conversion();
        
        // After clearing, output should match input
        assert_eq!(processor.get_output_image_properties(), properties);
        
        // Test clear all transformations
        processor.set_cropping_from(corner_points).unwrap();
        processor.set_brightness_offset(30).unwrap();
        processor.clear_all_transformations();
        
        assert_eq!(processor.get_output_image_properties(), properties);
    }

    #[test]
    fn test_verify_input_image_allowed() {
        let resolution = ImageXYResolution::new(100, 80).unwrap();
        let properties = ImageFrameProperties::new(resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
        
        let processor = ImageFrameProcessor::new(properties);
        
        // Create matching image
        let valid_image = create_test_image(100, 80, &ColorChannelLayout::RGB, &ColorSpace::Gamma);
        let result = processor.verify_input_image_allowed(&valid_image);
        assert!(result.is_ok());
        
        // Create non-matching image (wrong resolution)
        let invalid_image = create_test_image(50, 40, &ColorChannelLayout::RGB, &ColorSpace::Gamma);
        let result = processor.verify_input_image_allowed(&invalid_image);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_image_no_transformation() {
        let resolution = ImageXYResolution::new(50, 40).unwrap();
        let properties = ImageFrameProperties::new(resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
        
        let processor = ImageFrameProcessor::new(properties);
        
        let source = create_test_image_with_pattern(50, 40, &ColorChannelLayout::RGB, &ColorSpace::Gamma);
        let mut destination = create_test_image(50, 40, &ColorChannelLayout::RGB, &ColorSpace::Gamma);
        
        let result = processor.process_image(&source, &mut destination);
        assert!(result.is_ok());
        
        // Destination should be identical to source
        let source_pixels = source.get_pixels_view();
        let dest_pixels = destination.get_pixels_view();
        
        for y in 0..40 {
            for x in 0..50 {
                for c in 0..3 {
                    assert_eq!(source_pixels[(y, x, c)], dest_pixels[(y, x, c)]);
                }
            }
        }
    }

    #[test]
    fn test_process_image_with_cropping() {
        let resolution = ImageXYResolution::new(100, 80).unwrap();
        let properties = ImageFrameProperties::new(resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
        
        let mut processor = ImageFrameProcessor::new(properties);
        
        // Set up cropping from (10,10) to (50,50) - should result in 40x40 image
        let corner_points = CornerPoints::new(ImageXYPoint::new(10, 10), ImageXYPoint::new(50, 50)).unwrap();
        processor.set_cropping_from(corner_points).unwrap();
        
        let source = create_test_image_with_pattern(100, 80, &ColorChannelLayout::RGB, &ColorSpace::Gamma);
        let mut destination = create_test_image(40, 40, &ColorChannelLayout::RGB, &ColorSpace::Gamma);
        
        let result = processor.process_image(&source, &mut destination);
        assert!(result.is_ok());
        
        // Verify destination has the expected resolution
        assert_eq!(destination.get_xy_resolution().width, 40);
        assert_eq!(destination.get_xy_resolution().height, 40);
    }

    #[test]
    fn test_process_image_with_resizing() {
        let input_resolution = ImageXYResolution::new(100, 80).unwrap();
        let properties = ImageFrameProperties::new(input_resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
        
        let mut processor = ImageFrameProcessor::new(properties);
        
        // Set up resizing to 50x40
        let output_resolution = ImageXYResolution::new(50, 40).unwrap();
        processor.set_resizing_to(output_resolution).unwrap();
        
        let source = create_test_image_with_pattern(100, 80, &ColorChannelLayout::RGB, &ColorSpace::Gamma);
        let mut destination = create_test_image(50, 40, &ColorChannelLayout::RGB, &ColorSpace::Gamma);
        
        let result = processor.process_image(&source, &mut destination);
        assert!(result.is_ok());
        
        // Verify destination has the expected resolution
        assert_eq!(destination.get_xy_resolution().width, 50);
        assert_eq!(destination.get_xy_resolution().height, 40);
    }

    #[test]
    fn test_process_image_with_grayscale_conversion() {
        let resolution = ImageXYResolution::new(50, 40).unwrap();
        let properties = ImageFrameProperties::new(resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
        
        let mut processor = ImageFrameProcessor::new(properties);
        processor.set_conversion_to_grayscale(true).unwrap();
        
        let source = create_test_image_with_pattern(50, 40, &ColorChannelLayout::RGB, &ColorSpace::Gamma);
        let mut destination = create_test_image(50, 40, &ColorChannelLayout::GrayScale, &ColorSpace::Gamma);
        
        let result = processor.process_image(&source, &mut destination);
        assert!(result.is_ok());
        
        // Verify destination is grayscale
        assert_eq!(destination.get_channel_layout(), &ColorChannelLayout::GrayScale);
        assert_eq!(destination.get_color_channel_count(), 1);
    }

    #[test]
    fn test_process_image_crop_and_resize() {
        let input_resolution = ImageXYResolution::new(100, 80).unwrap();
        let properties = ImageFrameProperties::new(input_resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
        
        let mut processor = ImageFrameProcessor::new(properties);
        
        // Set up cropping and resizing
        let corner_points = CornerPoints::new(ImageXYPoint::new(20, 20), ImageXYPoint::new(80, 60)).unwrap();
        processor.set_cropping_from(corner_points).unwrap();
        
        let output_resolution = ImageXYResolution::new(30, 20).unwrap();
        processor.set_resizing_to(output_resolution).unwrap();
        
        let source = create_test_image_with_pattern(100, 80, &ColorChannelLayout::RGB, &ColorSpace::Gamma);
        let mut destination = create_test_image(30, 20, &ColorChannelLayout::RGB, &ColorSpace::Gamma);
        
        let result = processor.process_image(&source, &mut destination);
        assert!(result.is_ok());
        
        // Verify final dimensions
        assert_eq!(destination.get_xy_resolution().width, 30);
        assert_eq!(destination.get_xy_resolution().height, 20);
    }

    #[test]
    fn test_process_image_crop_resize_grayscale() {
        let input_resolution = ImageXYResolution::new(100, 80).unwrap();
        let properties = ImageFrameProperties::new(input_resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
        
        let mut processor = ImageFrameProcessor::new(properties);
        
        // Set up cropping, resizing, and grayscale conversion
        let corner_points = CornerPoints::new(ImageXYPoint::new(10, 10), ImageXYPoint::new(50, 50)).unwrap();
        processor.set_cropping_from(corner_points).unwrap();
        
        let output_resolution = ImageXYResolution::new(20, 20).unwrap();
        processor.set_resizing_to(output_resolution).unwrap();
        processor.set_conversion_to_grayscale(true).unwrap();
        
        let source = create_test_image_with_pattern(100, 80, &ColorChannelLayout::RGB, &ColorSpace::Gamma);
        let mut destination = create_test_image(20, 20, &ColorChannelLayout::GrayScale, &ColorSpace::Gamma);
        
        let result = processor.process_image(&source, &mut destination);
        assert!(result.is_ok());
        
        // Verify final properties
        assert_eq!(destination.get_xy_resolution().width, 20);
        assert_eq!(destination.get_xy_resolution().height, 20);
        assert_eq!(destination.get_channel_layout(), &ColorChannelLayout::GrayScale);
    }

    #[test]
    fn test_process_image_with_brightness_and_contrast() {
        let resolution = ImageXYResolution::new(50, 40).unwrap();
        let properties = ImageFrameProperties::new(resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
        
        let mut processor = ImageFrameProcessor::new(properties);
        processor.set_brightness_offset(50).unwrap();
        processor.set_contrast_change(2.0).unwrap();
        
        let source = create_test_image_with_pattern(50, 40, &ColorChannelLayout::RGB, &ColorSpace::Gamma);
        let mut destination = create_test_image(50, 40, &ColorChannelLayout::RGB, &ColorSpace::Gamma);
        
        let result = processor.process_image(&source, &mut destination);
        assert!(result.is_ok());
        
        // The image should have been processed (we can't easily verify the exact transformation,
        // but we can verify it completed without error)
        assert_eq!(destination.get_xy_resolution(), resolution);
        assert_eq!(destination.get_channel_layout(), &ColorChannelLayout::RGB);
    }

    #[test]
    fn test_display_trait() {
        let resolution = ImageXYResolution::new(100, 80).unwrap();
        let properties = ImageFrameProperties::new(resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
        
        let processor = ImageFrameProcessor::new(properties);
        let display_string = format!("{}", processor);
        
        assert!(display_string.contains("ImageFrameCleanupDefinition"));
        assert!(display_string.contains("100"));
        assert!(display_string.contains("80"));
    }

    #[test]
    fn test_display_trait_with_transformations() {
        let resolution = ImageXYResolution::new(100, 80).unwrap();
        let properties = ImageFrameProperties::new(resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
        
        let mut processor = ImageFrameProcessor::new(properties);
        let corner_points = CornerPoints::new(ImageXYPoint::new(10, 10), ImageXYPoint::new(50, 50)).unwrap();
        processor.set_cropping_from(corner_points).unwrap();
        processor.set_resizing_to(ImageXYResolution::new(200, 160).unwrap()).unwrap();
        processor.set_brightness_offset(25).unwrap();
        processor.set_contrast_change(1.5).unwrap();
        processor.set_conversion_to_grayscale(true).unwrap();
        
        let display_string = format!("{}", processor);
        
        assert!(display_string.contains("ImageFrameCleanupDefinition"));
        assert!(display_string.contains("Cropping"));
        assert!(display_string.contains("resizing"));
        assert!(display_string.contains("brightness"));
        assert!(display_string.contains("contrast"));
        assert!(display_string.contains("grayscale"));
    }

    #[test]
    fn test_clone_trait() {
        let resolution = ImageXYResolution::new(100, 80).unwrap();
        let properties = ImageFrameProperties::new(resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
        
        let mut original_processor = ImageFrameProcessor::new(properties.clone());
        original_processor.set_brightness_offset(50).unwrap();
        original_processor.set_contrast_change(2.0).unwrap();
        
        let cloned_processor = original_processor.clone();
        
        // Both should have the same properties
        assert_eq!(original_processor.get_input_image_properties(), cloned_processor.get_input_image_properties());
        assert_eq!(original_processor.get_output_image_properties(), cloned_processor.get_output_image_properties());
        
        // Modify original - clone should be unaffected
        original_processor.set_brightness_offset(100).unwrap();
        
        // They should now be different (if we had a way to check internal state)
        // For now, we just verify the clone operation doesn't panic
    }

    #[test]
    fn test_debug_trait() {
        let resolution = ImageXYResolution::new(100, 80).unwrap();
        let properties = ImageFrameProperties::new(resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
        
        let processor = ImageFrameProcessor::new(properties);
        let debug_string = format!("{:?}", processor);
        
        assert!(debug_string.contains("ImageFrameProcessor"));
        assert!(debug_string.contains("input_image_properties"));
    }

    #[test]
    fn test_corner_points_creation_and_validation() {
        // Valid corner points
        let upper_left = ImageXYPoint::new(10, 15);
        let lower_right = ImageXYPoint::new(50, 45);
        let corner_points = CornerPoints::new(upper_left, lower_right);
        assert!(corner_points.is_ok());
        
        let cp = corner_points.unwrap();
        assert_eq!(cp.get_width(), 40); // 50 - 10
        assert_eq!(cp.get_height(), 30); // 45 - 15
        
        // Invalid corner points (lower_right not actually lower/right)
        let invalid_points = CornerPoints::new(ImageXYPoint::new(50, 45), ImageXYPoint::new(10, 15));
        assert!(invalid_points.is_err());
        
        // Test fit verification
        let resolution = ImageXYResolution::new(100, 80).unwrap();
        let result = cp.verify_fits_in_resolution(resolution);
        assert!(result.is_ok());
        
        // Test points that don't fit
        let too_big_resolution = ImageXYResolution::new(40, 30).unwrap(); // Smaller than our corner points
        let result = cp.verify_fits_in_resolution(too_big_resolution);
        assert!(result.is_err());
    }

    // Visual tests using the bird image
    const TEST_BIRD_IMAGE_PATH: &str = "tests/images/bird.jpg";

    #[test]
    fn test_processor_visual_cropping_with_bird_image() {
        // Skip if bird image not available
        if !std::path::Path::new(TEST_BIRD_IMAGE_PATH).exists() {
            return;
        }

        let img_bytes = std::fs::read(TEST_BIRD_IMAGE_PATH).unwrap();
        let source_frame = ImageFrame::new_from_jpeg_bytes(&img_bytes, &ColorSpace::Gamma).unwrap();
        
        println!("Processing bird image ({}x{}) with cropping", 
                 source_frame.get_xy_resolution().width, 
                 source_frame.get_xy_resolution().height);

        let input_props = ImageFrameProperties::new(
            source_frame.get_xy_resolution(),
            ColorSpace::Gamma,
            ColorChannelLayout::RGB
        ).unwrap();

        let mut processor = ImageFrameProcessor::new(input_props);
        
        // Crop to center 50% of the image
        let width = source_frame.get_xy_resolution().width;
        let height = source_frame.get_xy_resolution().height;
        let crop_left = width / 4;
        let crop_top = height / 4;
        let crop_right = width * 3 / 4;
        let crop_bottom = height * 3 / 4;
        
        let corner_points = CornerPoints::new(
            ImageXYPoint::new(crop_left as u32, crop_top as u32),
            ImageXYPoint::new(crop_right as u32, crop_bottom as u32)
        ).unwrap();
        
        processor.set_cropping_from(corner_points).unwrap();
        
        let output_resolution = corner_points.enclosed_area_width_height();
        let mut destination = create_test_image(
            output_resolution.width,
            output_resolution.height,
            &ColorChannelLayout::RGB,
            &ColorSpace::Gamma
        );

        let result = processor.process_image(&source_frame, &mut destination);
        assert!(result.is_ok());

        // Always save the result
        let cropped_png = destination.export_as_png_bytes().unwrap();
        std::fs::write("tests/images/processor_cropped_bird.png", &cropped_png).unwrap();
        println!("Saved cropped bird image to processor_cropped_bird.png");

        // Verify dimensions
        assert_eq!(destination.get_xy_resolution().width, crop_right - crop_left);
        assert_eq!(destination.get_xy_resolution().height, crop_bottom - crop_top);
    }

    #[test]
    fn test_processor_visual_resizing_with_bird_image() {
        // Skip if bird image not available
        if !std::path::Path::new(TEST_BIRD_IMAGE_PATH).exists() {
            return;
        }

        let img_bytes = std::fs::read(TEST_BIRD_IMAGE_PATH).unwrap();
        let source_frame = ImageFrame::new_from_jpeg_bytes(&img_bytes, &ColorSpace::Gamma).unwrap();
        
        println!("Processing bird image ({}x{}) with resizing", 
                 source_frame.get_xy_resolution().width, 
                 source_frame.get_xy_resolution().height);

        let input_props = ImageFrameProperties::new(
            source_frame.get_xy_resolution(),
            ColorSpace::Gamma,
            ColorChannelLayout::RGB
        ).unwrap();

        let mut processor = ImageFrameProcessor::new(input_props);
        
        // Resize to 200x150
        let target_resolution = ImageXYResolution::new(200, 150).unwrap();
        processor.set_resizing_to(target_resolution).unwrap();
        
        let mut destination = create_test_image(200, 150, &ColorChannelLayout::RGB, &ColorSpace::Gamma);

        let result = processor.process_image(&source_frame, &mut destination);
        assert!(result.is_ok());

        // Always save the result
        let resized_png = destination.export_as_png_bytes().unwrap();
        std::fs::write("tests/images/processor_resized_bird.png", &resized_png).unwrap();
        println!("Saved resized bird image to processor_resized_bird.png");

        // Verify dimensions
        assert_eq!(destination.get_xy_resolution().width, 200);
        assert_eq!(destination.get_xy_resolution().height, 150);
    }

    #[test]
    fn test_processor_visual_grayscale_conversion_with_bird_image() {
        // Skip if bird image not available
        if !std::path::Path::new(TEST_BIRD_IMAGE_PATH).exists() {
            return;
        }

        let img_bytes = std::fs::read(TEST_BIRD_IMAGE_PATH).unwrap();
        let source_frame = ImageFrame::new_from_jpeg_bytes(&img_bytes, &ColorSpace::Gamma).unwrap();
        
        println!("Processing bird image ({}x{}) with grayscale conversion", 
                 source_frame.get_xy_resolution().width, 
                 source_frame.get_xy_resolution().height);

        let input_props = ImageFrameProperties::new(
            source_frame.get_xy_resolution(),
            ColorSpace::Gamma,
            ColorChannelLayout::RGB
        ).unwrap();

        let mut processor = ImageFrameProcessor::new(input_props);
        processor.set_conversion_to_grayscale(true).unwrap();
        
        let mut destination = create_test_image(
            source_frame.get_xy_resolution().width,
            source_frame.get_xy_resolution().height,
            &ColorChannelLayout::GrayScale,
            &ColorSpace::Gamma
        );

        let result = processor.process_image(&source_frame, &mut destination);
        assert!(result.is_ok());

        // Always save the result
        let grayscale_png = destination.export_as_png_bytes().unwrap();
        std::fs::write("tests/images/processor_grayscale_bird.png", &grayscale_png).unwrap();
        println!("Saved grayscale bird image to processor_grayscale_bird.png");

        // Verify channel layout
        assert_eq!(*destination.get_channel_layout(), ColorChannelLayout::GrayScale);
        assert_eq!(destination.get_color_channel_count(), 1);
    }

    #[test]
    fn test_processor_visual_crop_resize_grayscale_with_bird_image() {
        // Skip if bird image not available
        if !std::path::Path::new(TEST_BIRD_IMAGE_PATH).exists() {
            return;
        }

        let img_bytes = std::fs::read(TEST_BIRD_IMAGE_PATH).unwrap();
        let source_frame = ImageFrame::new_from_jpeg_bytes(&img_bytes, &ColorSpace::Gamma).unwrap();
        
        println!("Processing bird image ({}x{}) with crop + resize + grayscale", 
                 source_frame.get_xy_resolution().width, 
                 source_frame.get_xy_resolution().height);

        let input_props = ImageFrameProperties::new(
            source_frame.get_xy_resolution(),
            ColorSpace::Gamma,
            ColorChannelLayout::RGB
        ).unwrap();

        let mut processor = ImageFrameProcessor::new(input_props);
        
        // Crop to center portion
        let width = source_frame.get_xy_resolution().width;
        let height = source_frame.get_xy_resolution().height;
        let crop_left = width / 6;
        let crop_top = height / 6;
        let crop_right = width * 5 / 6;
        let crop_bottom = height * 5 / 6;
        
        let corner_points = CornerPoints::new(
            ImageXYPoint::new(crop_left as u32, crop_top as u32),
            ImageXYPoint::new(crop_right as u32, crop_bottom as u32)
        ).unwrap();
        
        processor.set_cropping_from(corner_points).unwrap();
        
        // Resize to thumbnail size
        let target_resolution = ImageXYResolution::new(128, 96).unwrap();
        processor.set_resizing_to(target_resolution).unwrap();
        
        // Convert to grayscale
        processor.set_conversion_to_grayscale(true).unwrap();
        
        let mut destination = create_test_image(128, 96, &ColorChannelLayout::GrayScale, &ColorSpace::Gamma);

        let result = processor.process_image(&source_frame, &mut destination);
        assert!(result.is_ok());

        // Always save the result
        let processed_png = destination.export_as_png_bytes().unwrap();
        std::fs::write("tests/images/processor_crop_resize_grayscale_bird.png", &processed_png).unwrap();
        
        println!("Complex processing test images saved:");
        println!("  - processor_crop_resize_grayscale_bird.png (crop + resize + grayscale)");

        // Verify final properties
        assert_eq!(destination.get_xy_resolution().width, 128);
        assert_eq!(destination.get_xy_resolution().height, 96);
        assert_eq!(*destination.get_channel_layout(), ColorChannelLayout::GrayScale);
    }

    #[test]
    fn test_processor_visual_brightness_contrast_with_bird_image() {
        // Skip if bird image not available
        if !std::path::Path::new(TEST_BIRD_IMAGE_PATH).exists() {
            return;
        }

        let img_bytes = std::fs::read(TEST_BIRD_IMAGE_PATH).unwrap();
        let source_frame = ImageFrame::new_from_jpeg_bytes(&img_bytes, &ColorSpace::Gamma).unwrap();
        
        println!("Processing bird image ({}x{}) with brightness and contrast adjustments", 
                 source_frame.get_xy_resolution().width, 
                 source_frame.get_xy_resolution().height);

        let input_props = ImageFrameProperties::new(
            source_frame.get_xy_resolution(),
            ColorSpace::Gamma,
            ColorChannelLayout::RGB
        ).unwrap();

        // Save original for comparison
        let original_png = source_frame.export_as_png_bytes().unwrap();
        std::fs::write("tests/images/processor_original_bird.png", &original_png).unwrap();

        // Test brightness increase
        let mut bright_processor = ImageFrameProcessor::new(input_props.clone());
        bright_processor.set_brightness_offset(50).unwrap();
        
        let mut bright_destination = create_test_image(
            source_frame.get_xy_resolution().width,
            source_frame.get_xy_resolution().height,
            &ColorChannelLayout::RGB,
            &ColorSpace::Gamma
        );

        let result = bright_processor.process_image(&source_frame, &mut bright_destination);
        assert!(result.is_ok());

        // Test high contrast
        let mut contrast_processor = ImageFrameProcessor::new(input_props.clone());
        contrast_processor.set_contrast_change(2.0).unwrap();
        
        let mut contrast_destination = create_test_image(
            source_frame.get_xy_resolution().width,
            source_frame.get_xy_resolution().height,
            &ColorChannelLayout::RGB,
            &ColorSpace::Gamma
        );

        let result = contrast_processor.process_image(&source_frame, &mut contrast_destination);
        assert!(result.is_ok());

        // Test combined brightness + contrast
        let mut combined_processor = ImageFrameProcessor::new(input_props);
        combined_processor.set_brightness_offset(30).unwrap();
        combined_processor.set_contrast_change(1.5).unwrap();
        
        let mut combined_destination = create_test_image(
            source_frame.get_xy_resolution().width,
            source_frame.get_xy_resolution().height,
            &ColorChannelLayout::RGB,
            &ColorSpace::Gamma
        );

        let result = combined_processor.process_image(&source_frame, &mut combined_destination);
        assert!(result.is_ok());

        let bright_png = bright_destination.export_as_png_bytes().unwrap();
        std::fs::write("tests/images/processor_bright_bird.png", &bright_png).unwrap();

        let contrast_png = contrast_destination.export_as_png_bytes().unwrap();
        std::fs::write("tests/images/processor_contrast_bird.png", &contrast_png).unwrap();

        let combined_png = combined_destination.export_as_png_bytes().unwrap();
        std::fs::write("tests/images/processor_combined_bird.png", &combined_png).unwrap();

        println!("Brightness/contrast test images saved:");
        println!("  - processor_original_bird.png (original)");
        println!("  - processor_bright_bird.png (+50 brightness)");
        println!("  - processor_contrast_bird.png (2.0x contrast)");
        println!("  - processor_combined_bird.png (+30 brightness, 1.5x contrast)");


        // Verify processing completed without errors
        assert_eq!(bright_destination.get_xy_resolution(), source_frame.get_xy_resolution());
        assert_eq!(contrast_destination.get_xy_resolution(), source_frame.get_xy_resolution());
        assert_eq!(combined_destination.get_xy_resolution(), source_frame.get_xy_resolution());
    }

    #[test]
    fn test_processor_visual_resize_crop_with_bird_image() {
        // Skip if bird image not available
        if !std::path::Path::new(TEST_BIRD_IMAGE_PATH).exists() {
            return;
        }

        let img_bytes = std::fs::read(TEST_BIRD_IMAGE_PATH).unwrap();
        let source_frame = ImageFrame::new_from_jpeg_bytes(&img_bytes, &ColorSpace::Gamma).unwrap();
        
        println!("Processing bird image ({}x{}) with resize + crop", 
                 source_frame.get_xy_resolution().width, 
                 source_frame.get_xy_resolution().height);

        let input_props = ImageFrameProperties::new(
            source_frame.get_xy_resolution(),
            ColorSpace::Gamma,
            ColorChannelLayout::RGB
        ).unwrap();

        let mut processor = ImageFrameProcessor::new(input_props);
        
        // First resize to larger dimensions (upscale)
        let intermediate_resolution = ImageXYResolution::new(1200, 800).unwrap();
        processor.set_resizing_to(intermediate_resolution).unwrap();
        
        // Then crop from the center of the resized image
        let crop_left = 100;
        let crop_top = 200;
        let crop_right = 400;
        let crop_bottom = 500;
        
        let corner_points = CornerPoints::new(
            ImageXYPoint::new(crop_left, crop_top),
            ImageXYPoint::new(crop_right, crop_bottom)
        ).unwrap();
        
        processor.set_cropping_from(corner_points).unwrap();
        
        let output_resolution = corner_points.enclosed_area_width_height();
        let mut destination = create_test_image(
            output_resolution.width,
            output_resolution.height,
            &ColorChannelLayout::RGB,
            &ColorSpace::Gamma
        );

        let result = processor.process_image(&source_frame, &mut destination);
        assert!(result.is_ok());

        // Always save the result
        let processed_png = destination.export_as_png_bytes().unwrap();
        std::fs::write("tests/images/processor_resize_crop_bird.png", &processed_png).unwrap();
        println!("Saved resize + crop bird image to processor_resize_crop_bird.png");

        // Verify final dimensions
        assert_eq!(destination.get_xy_resolution().width, (crop_right - crop_left) as usize);
        assert_eq!(destination.get_xy_resolution().height, (crop_bottom - crop_top) as usize);
    }

    #[test]
    fn test_processor_visual_resize_crop_grayscale_with_bird_image() {
        // Skip if bird image not available
        if !std::path::Path::new(TEST_BIRD_IMAGE_PATH).exists() {
            return;
        }

        let img_bytes = std::fs::read(TEST_BIRD_IMAGE_PATH).unwrap();
        let source_frame = ImageFrame::new_from_jpeg_bytes(&img_bytes, &ColorSpace::Gamma).unwrap();
        
        println!("Processing bird image ({}x{}) with resize + crop + grayscale", 
                 source_frame.get_xy_resolution().width, 
                 source_frame.get_xy_resolution().height);

        let input_props = ImageFrameProperties::new(
            source_frame.get_xy_resolution(),
            ColorSpace::Gamma,
            ColorChannelLayout::RGB
        ).unwrap();

        let mut processor = ImageFrameProcessor::new(input_props);
        
        // First resize to a different aspect ratio
        let intermediate_resolution = ImageXYResolution::new(800, 600).unwrap();
        processor.set_resizing_to(intermediate_resolution).unwrap();
        
        // Then crop to a square from the center
        let crop_size = 400;
        let crop_left = (800 - crop_size) / 2;
        let crop_top = (600 - crop_size) / 2;
        let crop_right = crop_left + crop_size;
        let crop_bottom = crop_top + crop_size;
        
        let corner_points = CornerPoints::new(
            ImageXYPoint::new(crop_left as u32, crop_top as u32),
            ImageXYPoint::new(crop_right as u32, crop_bottom as u32)
        ).unwrap();
        
        processor.set_cropping_from(corner_points).unwrap();
        
        // Convert to grayscale
        processor.set_conversion_to_grayscale(true).unwrap();
        
        let output_resolution = corner_points.enclosed_area_width_height();
        let mut destination = create_test_image(
            output_resolution.width,
            output_resolution.height,
            &ColorChannelLayout::GrayScale,
            &ColorSpace::Gamma
        );

        let result = processor.process_image(&source_frame, &mut destination);
        assert!(result.is_ok());

        // Always save the result
        let processed_png = destination.export_as_png_bytes().unwrap();
        std::fs::write("tests/images/processor_resize_crop_grayscale_bird.png", &processed_png).unwrap();
        println!("Saved resize + crop + grayscale bird image to processor_resize_crop_grayscale_bird.png");

        // Verify final properties
        assert_eq!(destination.get_xy_resolution().width, crop_size);
        assert_eq!(destination.get_xy_resolution().height, crop_size);
        assert_eq!(*destination.get_channel_layout(), ColorChannelLayout::GrayScale);
    }

    #[test]
    fn test_processor_visual_all_operations_summary() {
        // Skip if bird image not available
        if !std::path::Path::new(TEST_BIRD_IMAGE_PATH).exists() {
            return;
        }

        println!("\n=== ImageFrameProcessor Visual Test Summary ===");
        println!("The following test images are generated from bird.jpg:");
        println!("  📸 processor_original_bird.png - Original image");
        println!("  ✂️  processor_cropped_bird.png - Center 50% crop");
        println!("  📏 processor_resized_bird.png - Resized to 200x150");
        println!("  ⚫ processor_grayscale_bird.png - RGB to grayscale conversion");
        println!("  🔄 processor_crop_resize_grayscale_bird.png - Crop (center 67%) → Resize (128x96) → Grayscale");
        println!("  🔄 processor_resize_crop_bird.png - Resize (1200x800) → Crop (600x400)");
        println!("  🔄 processor_resize_crop_grayscale_bird.png - Resize (800x600) → Crop (400x400) → Grayscale");
        println!("  ☀️  processor_bright_bird.png - +50 brightness");
        println!("  📊 processor_contrast_bird.png - 2.0x contrast");
        println!("  🎨 processor_combined_bird.png - +30 brightness + 1.5x contrast");
        println!("================================================\n");
        
        // This test just prints the summary - the actual processing is done by other tests
        assert!(true);
    }
}

#[cfg(test)]
mod test_image_frame_segmentator {
    use super::*;

    fn create_test_input_properties() -> ImageFrameProperties {
        let resolution = ImageXYResolution::new(640, 480).unwrap();
        ImageFrameProperties::new(resolution, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap()
    }

    fn create_test_output_properties() -> SegmentedImageFrameProperties {
        let center_resolution = ImageXYResolution::new(128, 96).unwrap();
        let peripheral_resolution = ImageXYResolution::new(64, 48).unwrap();
        let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            center_resolution, 
            peripheral_resolution
        );
        
        SegmentedImageFrameProperties::new(
            &resolutions,
            &ColorChannelLayout::RGB,
            &ColorChannelLayout::RGB,
            &ColorSpace::Gamma
        )
    }

    fn create_test_gaze() -> GazeProperties {
        GazeProperties::new((0.5, 0.5), (0.4, 0.4))
    }

    fn create_test_image(width: usize, height: usize, channels: &ColorChannelLayout, color_space: &ColorSpace) -> ImageFrame {
        let resolution = ImageXYResolution::new(width, height).unwrap();
        ImageFrame::new(channels, color_space, &resolution).unwrap()
    }

    fn create_test_segmented_image(properties: &SegmentedImageFrameProperties) -> SegmentedImageFrame {
        SegmentedImageFrame::from_segmented_image_frame_properties(properties).unwrap()
    }

    #[test]
    fn test_image_frame_segmentator_new() {
        let input_props = create_test_input_properties();
        let output_props = create_test_output_properties();
        let gaze = create_test_gaze();
        
        let segmentator = ImageFrameSegmentator::new(input_props.clone(), output_props.clone(), gaze);
        assert!(segmentator.is_ok());
        
        let segmentator = segmentator.unwrap();
        
        // Test that the segmentator was created with correct properties
        // We can't directly access fields, but we can test functionality
        let test_input = create_test_image(640, 480, &ColorChannelLayout::RGB, &ColorSpace::Gamma);
        let verify_result = segmentator.verify_input_image(&test_input);
        assert!(verify_result.is_ok());
    }

    #[test]
    fn test_image_frame_segmentator_new_invalid_gaze() {
        let input_props = create_test_input_properties();
        let output_props = create_test_output_properties();
        
        // Test with invalid gaze that would cause segmentation issues
        // (this may or may not fail depending on implementation - just testing it doesn't panic)
        let extreme_gaze = GazeProperties::new((0.0, 0.0), (1.0, 1.0));
        let segmentator = ImageFrameSegmentator::new(input_props, output_props, extreme_gaze);
        // Should still succeed as GazeProperties clamps values
        assert!(segmentator.is_ok());
    }

    #[test]
    fn test_update_gaze() {
        let input_props = create_test_input_properties();
        let output_props = create_test_output_properties();
        let initial_gaze = create_test_gaze();
        
        let mut segmentator = ImageFrameSegmentator::new(input_props, output_props, initial_gaze).unwrap();
        
        // Update to a different gaze position
        let new_gaze = GazeProperties::new((0.3, 0.7), (0.3, 0.3));
        let result = segmentator.update_gaze(&new_gaze);
        assert!(result.is_ok());
        
        // Update to another valid gaze
        let another_gaze = GazeProperties::new((0.8, 0.2), (0.5, 0.5));
        let result = segmentator.update_gaze(&another_gaze);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_gaze_extreme_values() {
        let input_props = create_test_input_properties();
        let output_props = create_test_output_properties();
        let initial_gaze = create_test_gaze();
        
        let mut segmentator = ImageFrameSegmentator::new(input_props, output_props, initial_gaze).unwrap();
        
        // Test extreme gaze values (should be clamped)
        let extreme_gaze = GazeProperties::new((-1.0, 2.0), (1.5, 0.1));
        let result = segmentator.update_gaze(&extreme_gaze);
        dbg!(&result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_input_image() {
        let input_props = create_test_input_properties();
        let output_props = create_test_output_properties();
        let gaze = create_test_gaze();
        
        let segmentator = ImageFrameSegmentator::new(input_props, output_props, gaze).unwrap();
        
        // Test with matching image
        let valid_input = create_test_image(640, 480, &ColorChannelLayout::RGB, &ColorSpace::Gamma);
        let result = segmentator.verify_input_image(&valid_input);
        assert!(result.is_ok());
        
        // Test with wrong resolution
        let wrong_resolution = create_test_image(320, 240, &ColorChannelLayout::RGB, &ColorSpace::Gamma);
        let result = segmentator.verify_input_image(&wrong_resolution);
        assert!(result.is_err());
        
        // Test with wrong color channels
        let wrong_channels = create_test_image(640, 480, &ColorChannelLayout::GrayScale, &ColorSpace::Gamma);
        let result = segmentator.verify_input_image(&wrong_channels);
        assert!(result.is_err());
        
        // Test with wrong color space
        let wrong_color_space = create_test_image(640, 480, &ColorChannelLayout::RGB, &ColorSpace::Linear);
        let result = segmentator.verify_input_image(&wrong_color_space);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_output_image() {
        let input_props = create_test_input_properties();
        let output_props = create_test_output_properties();
        let gaze = create_test_gaze();
        
        let segmentator = ImageFrameSegmentator::new(input_props, output_props.clone(), gaze).unwrap();
        
        // Test with matching segmented image
        let valid_output = create_test_segmented_image(&output_props);
        let result = segmentator.verify_output_image(&valid_output);
        assert!(result.is_ok());
        
        // Test with wrong properties
        let wrong_center_resolution = ImageXYResolution::new(64, 48).unwrap();
        let wrong_peripheral_resolution = ImageXYResolution::new(32, 24).unwrap();
        let wrong_resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            wrong_center_resolution, 
            wrong_peripheral_resolution
        );
        let wrong_props = SegmentedImageFrameProperties::new(
            &wrong_resolutions,
            &ColorChannelLayout::RGB,
            &ColorChannelLayout::RGB,
            &ColorSpace::Gamma
        );
        let wrong_output = create_test_segmented_image(&wrong_props);
        let result = segmentator.verify_output_image(&wrong_output);
        assert!(result.is_err());
    }

    #[test]
    fn test_segment_image_basic() {
        let input_props = create_test_input_properties();
        let output_props = create_test_output_properties();
        let gaze = create_test_gaze();

        let mut segmentator = ImageFrameSegmentator::new(input_props, output_props.clone(), gaze).unwrap();
        
        // Create test input with pattern
        let mut input = create_test_image(640, 480, &ColorChannelLayout::RGB, &ColorSpace::Gamma);
        {
            let mut pixels = input.get_pixels_view_mut();
            for y in 0..480 {
                for x in 0..640 {
                    for c in 0..3 {
                        // Create a gradient pattern
                        pixels[(y, x, c)] = ((x + y + c * 80) % 256) as u8;
                    }
                }
            }
        }
        
        // Create output segmented image
        let mut output = create_test_segmented_image(&output_props);
        
        // Perform segmentation
        let result = segmentator.segment_image(&input, &mut output);
        assert!(result.is_ok());
        
        // Verify that output segments have been populated
        let frame_refs = output.get_ordered_image_frame_references();
        
        // Check that all 9 segments have expected dimensions
        assert_eq!(frame_refs[0].get_xy_resolution().width, 64); // peripheral
        assert_eq!(frame_refs[0].get_xy_resolution().height, 48); // peripheral
        assert_eq!(frame_refs[4].get_xy_resolution().width, 128); // center
        assert_eq!(frame_refs[4].get_xy_resolution().height, 96); // center
        assert_eq!(frame_refs[8].get_xy_resolution().width, 64); // peripheral
        assert_eq!(frame_refs[8].get_xy_resolution().height, 48); // peripheral
    }

    #[test]
    fn test_segment_image_with_different_gaze_positions() {
        let input_props = create_test_input_properties();
        let output_props = create_test_output_properties();
        
        // Test with different gaze positions
        let gaze_positions = vec![
            GazeProperties::new((0.25, 0.25), (0.3, 0.3)), // Upper left focus
            GazeProperties::new((0.75, 0.75), (0.4, 0.4)), // Lower right focus
            GazeProperties::new((0.5, 0.3), (0.2, 0.6)),   // Top center focus
        ];
        
        for (i, gaze) in gaze_positions.iter().enumerate() {
            let mut segmentator = ImageFrameSegmentator::new(
                input_props.clone(), 
                output_props.clone(), 
                *gaze
            ).unwrap();
            
            let input = create_test_image(640, 480, &ColorChannelLayout::RGB, &ColorSpace::Gamma);
            let mut output = create_test_segmented_image(&output_props);
            
            let result = segmentator.segment_image(&input, &mut output);
            assert!(result.is_ok(), "Segmentation failed for gaze position {}", i);
        }
    }

    #[test]
    fn test_segment_image_with_grayscale_conversion() {
        let input_props = create_test_input_properties();
        
        // Create output properties with grayscale conversion
        let center_resolution = ImageXYResolution::new(128, 96).unwrap();
        let peripheral_resolution = ImageXYResolution::new(64, 48).unwrap();
        let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            center_resolution, 
            peripheral_resolution
        );
        
        let grayscale_output_props = SegmentedImageFrameProperties::new(
            &resolutions,
            &ColorChannelLayout::GrayScale, // Center as grayscale
            &ColorChannelLayout::RGB,       // Peripherals stay RGB
            &ColorSpace::Gamma
        );
        
        let gaze = create_test_gaze();
        let mut segmentator = ImageFrameSegmentator::new(input_props, grayscale_output_props.clone(), gaze).unwrap();
        
        let input = create_test_image(640, 480, &ColorChannelLayout::RGB, &ColorSpace::Gamma);
        let mut output = create_test_segmented_image(&grayscale_output_props);
        
        let result = segmentator.segment_image(&input, &mut output);
        assert!(result.is_ok());
        
        // Verify center is grayscale and peripherals are RGB
        let frame_refs = output.get_ordered_image_frame_references();
        assert_eq!(*frame_refs[4].get_channel_layout(), ColorChannelLayout::GrayScale); // Center
        assert_eq!(*frame_refs[0].get_channel_layout(), ColorChannelLayout::RGB); // Peripheral
        assert_eq!(*frame_refs[8].get_channel_layout(), ColorChannelLayout::RGB); // Peripheral
    }

    #[test]
    fn test_segment_image_error_handling() {
        let input_props = create_test_input_properties();
        let output_props = create_test_output_properties();
        let gaze = create_test_gaze();
        
        let mut segmentator = ImageFrameSegmentator::new(input_props, output_props.clone(), gaze).unwrap();
        
        // Test with wrong input size
        let wrong_input = create_test_image(320, 240, &ColorChannelLayout::RGB, &ColorSpace::Gamma);
        let mut output = create_test_segmented_image(&output_props);
        
        let result = segmentator.segment_image(&wrong_input, &mut output);
        assert!(result.is_err());
        
        // Test with correct input but wrong output
        let correct_input = create_test_image(640, 480, &ColorChannelLayout::RGB, &ColorSpace::Gamma);
        let wrong_center_resolution = ImageXYResolution::new(64, 48).unwrap();
        let wrong_peripheral_resolution = ImageXYResolution::new(32, 24).unwrap();
        let wrong_resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            wrong_center_resolution, 
            wrong_peripheral_resolution
        );
        let wrong_props = SegmentedImageFrameProperties::new(
            &wrong_resolutions,
            &ColorChannelLayout::RGB,
            &ColorChannelLayout::RGB,
            &ColorSpace::Gamma
        );
        let mut wrong_output = create_test_segmented_image(&wrong_props);
        
        let result = segmentator.segment_image(&correct_input, &mut wrong_output);
        assert!(result.is_err());
    }

    #[test]
    fn test_clone_trait() {
        let input_props = create_test_input_properties();
        let output_props = create_test_output_properties();
        let gaze = create_test_gaze();

        let original = ImageFrameSegmentator::new(input_props, output_props, gaze).unwrap();
        let cloned = original.clone();
        
        // Test that both can verify the same input
        let test_input = create_test_image(640, 480, &ColorChannelLayout::RGB, &ColorSpace::Gamma);
        
        let original_result = original.verify_input_image(&test_input);
        let cloned_result = cloned.verify_input_image(&test_input);
        
        assert!(original_result.is_ok());
        assert!(cloned_result.is_ok());
    }

    #[test]
    fn test_debug_trait() {
        let input_props = create_test_input_properties();
        let output_props = create_test_output_properties();
        let gaze = create_test_gaze();
        
        let segmentator = ImageFrameSegmentator::new(input_props, output_props, gaze).unwrap();
        let debug_string = format!("{:?}", segmentator);
        
        assert!(debug_string.contains("ImageFrameSegmentator"));
        assert!(debug_string.contains("input_properties"));
        assert!(debug_string.contains("output_properties"));
        assert!(debug_string.contains("ordered_transformers"));
    }

    #[test]
    fn test_gaze_properties_creation() {
        // Test normal values
        let gaze = GazeProperties::new((0.5, 0.5), (0.4, 0.4));
        let display_string = format!("{}", gaze);
        assert!(display_string.contains("GazeProperties"));
        
        // Test default centered
        let default_gaze = GazeProperties::create_default_centered();
        let display_string = format!("{}", default_gaze);
        assert!(display_string.contains("GazeProperties"));
        
        // Test clamping behavior
        let clamped_gaze = GazeProperties::new((-1.0, 2.0), (1.5, -0.5));
        // Values should be clamped to [0.0, 1.0] range
        // We can't directly access the fields, but we can test that it doesn't panic
        let display_string = format!("{}", clamped_gaze);
        assert!(display_string.contains("GazeProperties"));
    }

    #[test]
    fn test_gaze_corner_points_calculation() {
        let gaze = GazeProperties::new((0.5, 0.5), (0.6, 0.4));
        let source_resolution = ImageXYResolution::new(640, 480).unwrap();
        
        let result = gaze.calculate_source_corner_points_for_segmented_video_frame(source_resolution);
        assert!(result.is_ok());
        
        let corner_points = result.unwrap();
        assert_eq!(corner_points.len(), 9);
        
        // Test that all corner points are valid (no panics)
        for (i, corners) in corner_points.iter().enumerate() {
            assert!(corners.get_width() > 0, "Segment {} has zero width", i);
            assert!(corners.get_height() > 0, "Segment {} has zero height", i);
        }
    }

    #[test]
    fn test_gaze_corner_points_calculation_edge_cases() {
        // Test with minimum allowed resolution
        let gaze = GazeProperties::new((0.5, 0.5), (0.1, 0.1));
        let min_resolution = ImageXYResolution::new(4, 4).unwrap(); // TODO shouldnt this also work with 3x3?
        
        let result = gaze.calculate_source_corner_points_for_segmented_video_frame(min_resolution);
        assert!(result.is_ok());
        
        // Test with too small resolution (should fail)
        let too_small_resolution = ImageXYResolution::new(2, 2).unwrap();
        let result = gaze.calculate_source_corner_points_for_segmented_video_frame(too_small_resolution);
        assert!(result.is_err());
    }

    #[test]
    fn test_segmented_xy_image_resolutions() {
        let center_res = ImageXYResolution::new(128, 96).unwrap();
        let peripheral_res = ImageXYResolution::new(64, 48).unwrap();
        
        // Test create_with_same_sized_peripheral
        let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            center_res, 
            peripheral_res
        );
        
        // Test as_ordered_array
        let ordered = resolutions.as_ordered_array();
        assert_eq!(ordered.len(), 9);
        assert_eq!(*ordered[4], center_res); // Center should be at index 4
        assert_eq!(*ordered[0], peripheral_res); // Peripherals should all be the same
        assert_eq!(*ordered[8], peripheral_res);
        
        // Test Display trait
        let display_string = format!("{}", resolutions);
        assert!(display_string.contains("Center:"));
        assert!(display_string.contains("LowerLeft:"));
        assert!(display_string.contains("128"));
        assert!(display_string.contains("96"));
        assert!(display_string.contains("64"));
        assert!(display_string.contains("48"));
    }

    // Visual test with bird image (if available)
    const TEST_BIRD_IMAGE_PATH: &str = "tests/images/bird.jpg";

    #[test]
    fn test_segmentator_visual_with_bird_image() {
        // Skip if bird image not available
        if !std::path::Path::new(TEST_BIRD_IMAGE_PATH).exists() {
            return;
        }

        let img_bytes = std::fs::read(TEST_BIRD_IMAGE_PATH).unwrap();
        let source_frame = ImageFrame::new_from_jpeg_bytes(&img_bytes, &ColorSpace::Gamma).unwrap();
        
        println!("Segmenting bird image ({}x{}) into 9 regions", 
                 source_frame.get_xy_resolution().width, 
                 source_frame.get_xy_resolution().height);

        // Create properties matching the bird image
        let input_props = ImageFrameProperties::new(
            source_frame.get_xy_resolution(),
            ColorSpace::Gamma,
            ColorChannelLayout::RGB
        ).unwrap();

        // Create segmented output properties
        let center_resolution = ImageXYResolution::new(128, 96).unwrap();
        let peripheral_resolution = ImageXYResolution::new(64, 48).unwrap();
        let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            center_resolution, 
            peripheral_resolution
        );
        
        let output_props = SegmentedImageFrameProperties::new(
            &resolutions,
            &ColorChannelLayout::RGB,
            &ColorChannelLayout::RGB,
            &ColorSpace::Gamma
        );

        // Test with centered gaze
        let centered_gaze = GazeProperties::create_default_centered();
        let mut segmentator = ImageFrameSegmentator::new(input_props.clone(), output_props.clone(), centered_gaze).unwrap();
        
        let mut segmented_output = create_test_segmented_image(&output_props);
        
        let result = segmentator.segment_image(&source_frame, &mut segmented_output);
        assert!(result.is_ok());

        // Save individual segments
        let frame_refs = segmented_output.get_ordered_image_frame_references();
        let segment_names = [
            "lower_left", "lower_middle", "lower_right",
            "middle_left", "center", "middle_right", 
            "upper_left", "upper_middle", "upper_right"
        ];

        for (i, (frame, name)) in frame_refs.iter().zip(segment_names.iter()).enumerate() {
            let png_bytes = frame.export_as_png_bytes().unwrap();
            let filename = format!("tests/images/segmentator_{}_{}.png", name, i);
            std::fs::write(&filename, &png_bytes).unwrap();
        }

        println!("Segmentation test completed. Saved 9 segment images:");
        for (i, name) in segment_names.iter().enumerate() {
            println!("  - segmentator_{}_{}.png", name, i);
        }

        // Test with off-center gaze
        let offset_gaze = GazeProperties::new((0.3, 0.7), (0.4, 0.3));
        let result = segmentator.update_gaze(&offset_gaze);
        assert!(result.is_ok());
        
        let mut offset_segmented_output = create_test_segmented_image(&output_props);
        let result = segmentator.segment_image(&source_frame, &mut offset_segmented_output);
        assert!(result.is_ok());

        // Save offset gaze center segment for comparison
        let offset_frame_refs = offset_segmented_output.get_ordered_image_frame_references();
        let center_png = offset_frame_refs[4].export_as_png_bytes().unwrap();
        std::fs::write("tests/images/segmentator_offset_center.png", &center_png).unwrap();
        
        println!("  - segmentator_offset_center.png (off-center gaze for comparison)");
    }
}
