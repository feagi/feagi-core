//! Tests for the data module
//! 
//! This module contains comprehensive tests for data structures including ImageFrame
//! and related functionality for image processing, import, and export operations.

use feagi_data_structures::data::{ImageFrame, SegmentedImageFrame};
use feagi_data_structures::data::descriptors::{ColorChannelLayout, ColorSpace, ImageXYResolution, ImageFrameProperties, MemoryOrderLayout, SegmentedImageFrameProperties, SegmentedXYImageResolutions};
use feagi_data_structures::genomic::{CorticalType, SensorCorticalType};
use feagi_data_structures::genomic::descriptors::CorticalGroupIndex;
use ndarray::Array3;
use std::fs;
use std::path::Path;

#[cfg(test)]
mod test_image_frame {
    use image::GenericImageView;
    use super::*;

    const TEST_BIRD_IMAGE_PATH: &str = "tests/images/bird.jpg";

    #[test]
    fn test_image_frame_creation_new() {
        let resolution = ImageXYResolution::new(100, 200).unwrap();
        let frame = ImageFrame::new(
            &ColorChannelLayout::RGB,
            &ColorSpace::Gamma,
            &resolution
        ).unwrap();

        assert_eq!(frame.get_xy_resolution(), resolution);
        assert_eq!(*frame.get_channel_layout(), ColorChannelLayout::RGB);
        assert_eq!(*frame.get_color_space(), ColorSpace::Gamma);
        assert_eq!(frame.get_color_channel_count(), 3);
        assert_eq!(frame.get_number_elements(), 100 * 200 * 3);
    }

    #[test]
    fn test_image_frame_creation_different_layouts() {
        let resolution = ImageXYResolution::new(50, 50).unwrap();
        
        // Test GrayScale
        let frame_gray = ImageFrame::new(&ColorChannelLayout::GrayScale, &ColorSpace::Gamma, &resolution).unwrap();
        assert_eq!(frame_gray.get_color_channel_count(), 1);
        
        // Test RG
        let frame_rg = ImageFrame::new(&ColorChannelLayout::RG, &ColorSpace::Gamma, &resolution).unwrap();
        assert_eq!(frame_rg.get_color_channel_count(), 2);
        
        // Test RGB
        let frame_rgb = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Gamma, &resolution).unwrap();
        assert_eq!(frame_rgb.get_color_channel_count(), 3);
        
        // Test RGBA
        let frame_rgba = ImageFrame::new(&ColorChannelLayout::RGBA, &ColorSpace::Gamma, &resolution).unwrap();
        assert_eq!(frame_rgba.get_color_channel_count(), 4);
    }

    #[test]
    fn test_load_bird_jpeg() {
        // Test loading the example bird.jpg file
        if Path::new(TEST_BIRD_IMAGE_PATH).exists() {
            let img_bytes = fs::read(TEST_BIRD_IMAGE_PATH).expect("Failed to read bird.jpg");
            let frame = ImageFrame::new_from_jpeg_bytes(&img_bytes, &ColorSpace::Gamma).unwrap();
            
            // Verify the image was loaded
            assert!(*frame.get_channel_layout() == ColorChannelLayout::RGB || 
                    *frame.get_channel_layout() == ColorChannelLayout::GrayScale);
            assert_eq!(*frame.get_color_space(), ColorSpace::Gamma);
            assert!(frame.get_xy_resolution().width > 0);
            assert!(frame.get_xy_resolution().height > 0);
            
            println!("Bird image loaded: {}x{}, {} channels", 
                     frame.get_xy_resolution().width, 
                     frame.get_xy_resolution().height, 
                     frame.get_color_channel_count());
        } else {
            panic!("Bird image not found at {}", TEST_BIRD_IMAGE_PATH);
        }
    }

    #[test]
    fn test_export_import_roundtrip_png() {
        let resolution = ImageXYResolution::new(20, 15).unwrap();
        let mut original_frame = ImageFrame::new(&ColorChannelLayout::RGBA, &ColorSpace::Gamma, &resolution).unwrap();
        
        // Set some test pattern
        {
            let mut pixels = original_frame.get_pixels_view_mut();
            for y in 0..15 {
                for x in 0..20 {
                    pixels[(y, x, 0)] = (x * 12) as u8; // R gradient
                    pixels[(y, x, 1)] = (y * 17) as u8; // G gradient
                    pixels[(y, x, 2)] = 128;            // B constant
                    pixels[(y, x, 3)] = 200;            // A constant
                }
            }
        }

        // Export to PNG bytes
        let png_bytes = original_frame.export_as_png_bytes().unwrap();
        assert!(!png_bytes.is_empty());
        
        // Save to file for manual inspection (optional)
        if std::env::var("SAVE_TEST_IMAGES").is_ok() {
            fs::write("tests/test_output_roundtrip.png", &png_bytes).unwrap();
        }

        // Import back from PNG bytes
        let imported_frame = ImageFrame::new_from_png_bytes(&png_bytes, &ColorSpace::Gamma).unwrap();
        
        // Verify properties match
        assert_eq!(imported_frame.get_xy_resolution(), original_frame.get_xy_resolution());
        assert_eq!(*imported_frame.get_channel_layout(), *original_frame.get_channel_layout());
    }

    #[test]
    fn test_export_all_formats() {
        let resolution = ImageXYResolution::new(8, 8).unwrap();
        let mut frame = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Gamma, &resolution).unwrap();
        
        // Create a test pattern
        {
            let mut pixels = frame.get_pixels_view_mut();
            for y in 0..8 {
                for x in 0..8 {
                    pixels[(y, x, 0)] = (x * 32) as u8; // R gradient
                    pixels[(y, x, 1)] = (y * 32) as u8; // G gradient
                    pixels[(y, x, 2)] = 128;            // B constant
                }
            }
        }

        // Test all export formats
        let png_bytes = frame.export_as_png_bytes().unwrap();
        let bmp_bytes = frame.export_as_bmp_bytes().unwrap();
        let jpeg_bytes = frame.export_as_jpeg_bytes().unwrap();
        let tiff_bytes = frame.export_as_tiff_bytes().unwrap();

        // Verify all exports produced data
        assert!(!png_bytes.is_empty());
        assert!(!bmp_bytes.is_empty());
        assert!(!jpeg_bytes.is_empty());
        assert!(!tiff_bytes.is_empty());

        // Verify we can load each format back
        let _png_frame = ImageFrame::new_from_png_bytes(&png_bytes, &ColorSpace::Gamma).unwrap();
        let _bmp_frame = ImageFrame::new_from_bmp_bytes(&bmp_bytes, &ColorSpace::Gamma).unwrap();
        let _jpeg_frame = ImageFrame::new_from_jpeg_bytes(&jpeg_bytes, &ColorSpace::Gamma).unwrap();
        let _tiff_frame = ImageFrame::new_from_tiff_bytes(&tiff_bytes, &ColorSpace::Gamma).unwrap();
    }

    #[test]
    fn test_export_as_dynamic_image() {
        let resolution = ImageXYResolution::new(4, 4).unwrap();
        let mut frame = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Gamma, &resolution).unwrap();
        
        // Set some test pixel values
        {
            let mut pixels = frame.get_pixels_view_mut();
            pixels[(0, 0, 0)] = 255; // Red pixel at (0,0)
            pixels[(0, 0, 1)] = 0;
            pixels[(0, 0, 2)] = 0;
        }

        let dynamic_img = frame.export_as_dynamic_image().unwrap();
        assert_eq!(dynamic_img.dimensions(), (4, 4));
        assert_eq!(dynamic_img.color(), image::ColorType::Rgb8);
    }

    #[test]
    fn test_brightness_and_contrast_adjustments() {
        let resolution = ImageXYResolution::new(2, 2).unwrap();
        let mut frame = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Gamma, &resolution).unwrap();
        
        // Set initial values
        {
            let mut pixels = frame.get_pixels_view_mut();
            pixels[(0, 0, 0)] = 100;
            pixels[(0, 0, 1)] = 150;
            pixels[(0, 0, 2)] = 200;
        }

        // Test brightness adjustment
        frame.change_brightness(50);
        let pixels = frame.get_pixels_view();
        assert_eq!(pixels[(0, 0, 0)], 150); // 100 + 50
        assert_eq!(pixels[(0, 0, 1)], 200); // 150 + 50
        assert_eq!(pixels[(0, 0, 2)], 250); // 200 + 50

        // Reset and test contrast
        {
            let mut pixels = frame.get_pixels_view_mut();
            pixels[(0, 0, 0)] = 64;  // Below midpoint
            pixels[(0, 0, 1)] = 128; // At midpoint
            pixels[(0, 0, 2)] = 192; // Above midpoint
        }
        
        frame.change_contrast(2.0);
        let pixels = frame.get_pixels_view();
        // Values should move further from midpoint (128)
        assert!(pixels[(0, 0, 0)] < 64);   // Should be lower
        assert_eq!(pixels[(0, 0, 1)], 128); // Midpoint unchanged
        assert!(pixels[(0, 0, 2)] > 192);   // Should be higher
    }

    #[test]
    fn test_from_array_with_different_memory_layouts() {
        // Test HeightsWidthsChannels (default)
        let array_hwc = Array3::<u8>::zeros((10, 20, 3)); // height=10, width=20, channels=3
        let frame_hwc = ImageFrame::from_array(
            array_hwc, 
            &ColorSpace::Gamma, 
            &MemoryOrderLayout::HeightsWidthsChannels
        ).unwrap();
        assert_eq!(frame_hwc.get_xy_resolution().height, 10);
        assert_eq!(frame_hwc.get_xy_resolution().width, 20);
        assert_eq!(frame_hwc.get_color_channel_count(), 3);

        // NOTE: The from_array function determines channel count from dimension 2 of the input array,
        // but then applies the permutation. So we need to make sure dimension 2 has the right channel count
        // and the permutation will fix the height/width ordering.
        
        // Test WidthsHeightsChannels - After permutation [1,0,2], dims become [height, width, channels]
        let array_whc = Array3::<u8>::zeros((20, 10, 3)); // width=20, height=10, channels=3  
        let frame_whc = ImageFrame::from_array(
            array_whc,
            &ColorSpace::Gamma,
            &MemoryOrderLayout::WidthsHeightsChannels
        ).unwrap();
        assert_eq!(frame_whc.get_xy_resolution().height, 10);
        assert_eq!(frame_whc.get_xy_resolution().width, 20);

        // Test HeightsChannelsWidths - After permutation [0,2,1], dims become [height, width, channels]  
        let array_hcw = Array3::<u8>::zeros((10, 3, 20)); // height=10, channels=3, width=20
        let frame_hcw = ImageFrame::from_array(
            array_hcw,
            &ColorSpace::Linear,
            &MemoryOrderLayout::HeightsChannelsWidths
        ).unwrap();
        assert_eq!(frame_hcw.get_xy_resolution().height, 10);
        assert_eq!(frame_hcw.get_xy_resolution().width, 20);
        assert_eq!(*frame_hcw.get_color_space(), ColorSpace::Linear);
    }

    #[test]
    fn test_new_from_image_frame_properties() {
        let resolution = ImageXYResolution::new(64, 48).unwrap();
        let properties = ImageFrameProperties::new(
            resolution, 
            ColorSpace::Linear, 
            ColorChannelLayout::RGBA
        ).unwrap();

        let frame = ImageFrame::new_from_image_frame_properties(&properties).unwrap();
        
        assert_eq!(frame.get_xy_resolution(), resolution);
        assert_eq!(*frame.get_color_space(), ColorSpace::Linear);
        assert_eq!(*frame.get_channel_layout(), ColorChannelLayout::RGBA);
        assert_eq!(frame.get_color_channel_count(), 4);
        
        // Verify the frame properties match
        let frame_properties = frame.get_image_frame_properties();
        assert_eq!(frame_properties.get_image_resolution(), resolution);
        assert_eq!(frame_properties.get_color_space(), ColorSpace::Linear);
        assert_eq!(frame_properties.get_color_channel_layout(), ColorChannelLayout::RGBA);
    }

    #[test]
    fn test_new_from_dynamic_image() {
        // Create a small test image
        let img_buffer = image::RgbImage::new(4, 3);
        let dynamic_img = image::DynamicImage::ImageRgb8(img_buffer);
        
        let frame = ImageFrame::new_from_dynamic_image(dynamic_img, &ColorSpace::Gamma).unwrap();
        
        assert_eq!(frame.get_xy_resolution().width, 4);
        assert_eq!(frame.get_xy_resolution().height, 3);
        assert_eq!(*frame.get_channel_layout(), ColorChannelLayout::RGB);
        assert_eq!(*frame.get_color_space(), ColorSpace::Gamma);
    }

    #[test]
    fn test_brightness_adjustment_linear_color_space() {
        let resolution = ImageXYResolution::new(2, 2).unwrap();
        let mut frame = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Linear, &resolution).unwrap();
        
        // Set initial values
        {
            let mut pixels = frame.get_pixels_view_mut();
            pixels[(0, 0, 0)] = 100;
            pixels[(0, 0, 1)] = 150;
            pixels[(0, 0, 2)] = 200;
        }

        // Test brightness adjustment in linear space
        frame.change_brightness(25); // Should be scaled to linear range
        
        let pixels = frame.get_pixels_view();
        // Values should change but in a non-linear way due to sRGB conversion
        assert_ne!(pixels[(0, 0, 0)], 100);
        assert_ne!(pixels[(0, 0, 1)], 150);
        assert_ne!(pixels[(0, 0, 2)], 200);
    }

    #[test]
    fn test_contrast_adjustment_linear_color_space() {
        let resolution = ImageXYResolution::new(2, 2).unwrap();
        let mut frame = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Linear, &resolution).unwrap();
        
        // Set test values
        {
            let mut pixels = frame.get_pixels_view_mut();
            pixels[(0, 0, 0)] = 64;   // Below linear midpoint equivalent
            pixels[(0, 0, 1)] = 128;  // At gamma midpoint
            pixels[(0, 0, 2)] = 192;  // Above linear midpoint equivalent
        }

        frame.change_contrast(1.5); // Increase contrast
        
        let pixels = frame.get_pixels_view();
        // Values should change due to linear color space processing
        assert_ne!(pixels[(0, 0, 0)], 64);
        assert_ne!(pixels[(0, 0, 1)], 128);
        assert_ne!(pixels[(0, 0, 2)], 192);
    }

    #[test]
    fn test_clone_trait() {
        let resolution = ImageXYResolution::new(3, 3).unwrap();
        let mut original_frame = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Gamma, &resolution).unwrap();
        
        // Set some test data
        {
            let mut pixels = original_frame.get_pixels_view_mut();
            pixels[(0, 0, 0)] = 100;
            pixels[(1, 1, 1)] = 200;
            pixels[(2, 2, 2)] = 50;
        }

        // Clone the frame
        let cloned_frame = original_frame.clone();
        
        // Verify clone has same properties
        assert_eq!(cloned_frame.get_xy_resolution(), original_frame.get_xy_resolution());
        assert_eq!(*cloned_frame.get_channel_layout(), *original_frame.get_channel_layout());
        assert_eq!(*cloned_frame.get_color_space(), *original_frame.get_color_space());
        
        // Verify clone has same data
        let original_pixels = original_frame.get_pixels_view();
        let cloned_pixels = cloned_frame.get_pixels_view();
        assert_eq!(original_pixels[(0, 0, 0)], cloned_pixels[(0, 0, 0)]);
        assert_eq!(original_pixels[(1, 1, 1)], cloned_pixels[(1, 1, 1)]);
        assert_eq!(original_pixels[(2, 2, 2)], cloned_pixels[(2, 2, 2)]);
        
        // Verify they are independent (modify original)
        {
            let mut original_pixels_mut = original_frame.get_pixels_view_mut();
            original_pixels_mut[(0, 0, 0)] = 255;
        }
        
        // Clone should be unchanged
        assert_eq!(cloned_pixels[(0, 0, 0)], 100); // Still original value
        assert_eq!(original_frame.get_pixels_view()[(0, 0, 0)], 255); // Original changed
    }

    #[test]
    fn test_debug_trait() {
        let resolution = ImageXYResolution::new(2, 2).unwrap();
        let frame = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Gamma, &resolution).unwrap();
        
        let debug_string = format!("{:?}", frame);
        assert!(debug_string.contains("ImageFrame"));
        // Should contain the struct fields
        assert!(debug_string.contains("pixels"));
        assert!(debug_string.contains("channel_layout"));
        assert!(debug_string.contains("color_space"));
    }

    #[test]
    fn test_internal_memory_layout_constant() {
        assert_eq!(ImageFrame::INTERNAL_MEMORY_LAYOUT, MemoryOrderLayout::HeightsWidthsChannels);
    }

    #[test]
    fn test_get_internal_byte_data_mut() {
        let resolution = ImageXYResolution::new(2, 2).unwrap();
        let mut frame = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Gamma, &resolution).unwrap();
        
        // Modify data through mutable byte slice
        {
            let byte_data = frame.get_internal_byte_data_mut();
            assert_eq!(byte_data.len(), 2 * 2 * 3); // width * height * channels
            byte_data[0] = 255; // First byte (R channel of first pixel)
            byte_data[1] = 128; // Second byte (G channel of first pixel)
            byte_data[2] = 64;  // Third byte (B channel of first pixel)
        }
        
        // Verify changes through pixel view
        let pixels = frame.get_pixels_view();
        assert_eq!(pixels[(0, 0, 0)], 255);
        assert_eq!(pixels[(0, 0, 1)], 128);
        assert_eq!(pixels[(0, 0, 2)], 64);
    }

    #[test]
    fn test_different_channel_layouts_from_dynamic_image() {
        // Test GrayScale
        let gray_img_buffer = image::GrayImage::new(3, 2);
        let gray_dynamic = image::DynamicImage::ImageLuma8(gray_img_buffer);
        let gray_frame = ImageFrame::new_from_dynamic_image(gray_dynamic, &ColorSpace::Gamma).unwrap();
        assert_eq!(*gray_frame.get_channel_layout(), ColorChannelLayout::GrayScale);
        assert_eq!(gray_frame.get_color_channel_count(), 1);

        // Test RGBA
        let rgba_img_buffer = image::RgbaImage::new(3, 2);
        let rgba_dynamic = image::DynamicImage::ImageRgba8(rgba_img_buffer);
        let rgba_frame = ImageFrame::new_from_dynamic_image(rgba_dynamic, &ColorSpace::Gamma).unwrap();
        assert_eq!(*rgba_frame.get_channel_layout(), ColorChannelLayout::RGBA);
        assert_eq!(rgba_frame.get_color_channel_count(), 4);

        // Test LumaA (RG)
        let luma_a_img_buffer = image::GrayAlphaImage::new(3, 2);
        let luma_a_dynamic = image::DynamicImage::ImageLumaA8(luma_a_img_buffer);
        let luma_a_frame = ImageFrame::new_from_dynamic_image(luma_a_dynamic, &ColorSpace::Gamma).unwrap();
        assert_eq!(*luma_a_frame.get_channel_layout(), ColorChannelLayout::RG);
        assert_eq!(luma_a_frame.get_color_channel_count(), 2);
    }

    #[test]
    fn test_brightness_clamping_negative() {
        let resolution = ImageXYResolution::new(2, 2).unwrap();
        let mut frame = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Gamma, &resolution).unwrap();
        
        // Set low values that will go below 0 when brightness is reduced
        {
            let mut pixels = frame.get_pixels_view_mut();
            pixels[(0, 0, 0)] = 10;
            pixels[(0, 0, 1)] = 5;
            pixels[(0, 0, 2)] = 20;
        }

        frame.change_brightness(-50); // Reduce brightness significantly
        
        let pixels = frame.get_pixels_view();
        assert_eq!(pixels[(0, 0, 0)], 0); // Should be clamped to 0
        assert_eq!(pixels[(0, 0, 1)], 0); // Should be clamped to 0
        assert_eq!(pixels[(0, 0, 2)], 0); // 20 - 50 = -30, clamped to 0
    }

    #[test]
    fn test_contrast_reduction() {
        let resolution = ImageXYResolution::new(2, 2).unwrap();
        let mut frame = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Gamma, &resolution).unwrap();
        
        // Set extreme values
        {
            let mut pixels = frame.get_pixels_view_mut();
            pixels[(0, 0, 0)] = 0;   // Black
            pixels[(0, 0, 1)] = 128; // Mid gray
            pixels[(0, 0, 2)] = 255; // White
        }

        // Store original values for comparison
        let original_black = 0u8;
        let original_white = 255u8;

        // Reduce contrast (factor < 1)
        frame.change_contrast(0.5);
        
        let pixels = frame.get_pixels_view();
        // Black and white should move toward midpoint (128)
        assert!(pixels[(0, 0, 0)] > original_black);    // Black moves toward gray
        assert_eq!(pixels[(0, 0, 1)], 128); // Midpoint unchanged
        assert!(pixels[(0, 0, 2)] < original_white);   // White moves toward gray
        
        // Check that values are indeed closer to the midpoint (128) than they started
        let black_distance_to_mid = (128i32 - pixels[(0, 0, 0)] as i32).abs();
        let white_distance_to_mid = (pixels[(0, 0, 2)] as i32 - 128i32).abs();
        let original_black_distance = (128i32 - original_black as i32).abs();
        let original_white_distance = (original_white as i32 - 128i32).abs();
        
        assert!(black_distance_to_mid < original_black_distance);
        assert!(white_distance_to_mid < original_white_distance);
    }

    #[test] 
    fn test_zero_contrast() {
        let resolution = ImageXYResolution::new(2, 2).unwrap();
        let mut frame = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Gamma, &resolution).unwrap();
        
        // Set different values
        {
            let mut pixels = frame.get_pixels_view_mut();
            pixels[(0, 0, 0)] = 0;
            pixels[(0, 0, 1)] = 64;
            pixels[(0, 0, 2)] = 255;
        }

        // Zero contrast should make everything the midpoint
        frame.change_contrast(0.0);
        
        let pixels = frame.get_pixels_view();
        assert_eq!(pixels[(0, 0, 0)], 128);
        assert_eq!(pixels[(0, 0, 1)], 128);
        assert_eq!(pixels[(0, 0, 2)], 128);
    }

    #[test]
    fn test_brightness_adjustment_visual_with_bird_image() {
        // Skip if bird image not available
        if !Path::new(TEST_BIRD_IMAGE_PATH).exists() {
            return;
        }

        let img_bytes = fs::read(TEST_BIRD_IMAGE_PATH).unwrap();
        let mut original_frame = ImageFrame::new_from_jpeg_bytes(&img_bytes, &ColorSpace::Gamma).unwrap();

        println!("Testing brightness adjustments with bird image ({}x{})", 
                 original_frame.get_xy_resolution().width, 
                 original_frame.get_xy_resolution().height);

        // Save original for comparison
        if std::env::var("SAVE_TEST_IMAGES").is_ok() {
            let original_png = original_frame.export_as_png_bytes().unwrap();
            fs::write("tests/images/brightness_test_original.png", &original_png).unwrap();
        }

        // Test brightness increase
        let mut bright_frame = original_frame.clone();
        bright_frame.change_brightness(50);
        if std::env::var("SAVE_TEST_IMAGES").is_ok() {
            let bright_png = bright_frame.export_as_png_bytes().unwrap();
            fs::write("tests/images/brightness_test_increased.png", &bright_png).unwrap();
        }

        // Test brightness decrease  
        let mut dark_frame = original_frame.clone();
        dark_frame.change_brightness(-50);
        if std::env::var("SAVE_TEST_IMAGES").is_ok() {
            let dark_png = dark_frame.export_as_png_bytes().unwrap();
            fs::write("tests/images/brightness_test_decreased.png", &dark_png).unwrap();
        }

        // Test extreme brightness increase
        let mut very_bright_frame = original_frame.clone();
        very_bright_frame.change_brightness(100);
        if std::env::var("SAVE_TEST_IMAGES").is_ok() {
            let very_bright_png = very_bright_frame.export_as_png_bytes().unwrap();
            fs::write("tests/images/brightness_test_very_bright.png", &very_bright_png).unwrap();
        }

        // Test extreme brightness decrease
        let mut very_dark_frame = original_frame.clone();
        very_dark_frame.change_brightness(-100);
        if std::env::var("SAVE_TEST_IMAGES").is_ok() {
            let very_dark_png = very_dark_frame.export_as_png_bytes().unwrap();
            fs::write("tests/images/brightness_test_very_dark.png", &very_dark_png).unwrap();
            
            println!("Brightness test images saved:");
            println!("  - brightness_test_original.png (original)");
            println!("  - brightness_test_increased.png (+50)");
            println!("  - brightness_test_decreased.png (-50)");
            println!("  - brightness_test_very_bright.png (+100)");
            println!("  - brightness_test_very_dark.png (-100)");
        }

        // Verify that brightness adjustments actually changed the data
        let original_pixels = original_frame.get_pixels_view();
        let bright_pixels = bright_frame.get_pixels_view();
        let dark_pixels = dark_frame.get_pixels_view();

        // Find a non-black, non-white pixel to test (to avoid clamping effects)
        let mut test_pixel_found = false;
        for y in 0..original_frame.get_xy_resolution().height.min(10) {
            for x in 0..original_frame.get_xy_resolution().width.min(10) {
                let orig_val = original_pixels[(y, x, 0)] as i32;
                if orig_val > 50 && orig_val < 205 { // Avoid clamping range
                    let bright_val = bright_pixels[(y, x, 0)] as i32;
                    let dark_val = dark_pixels[(y, x, 0)] as i32;
                    
                    assert!(bright_val > orig_val, "Brightness increase should make pixels brighter");
                    assert!(dark_val < orig_val, "Brightness decrease should make pixels darker");
                    test_pixel_found = true;
                    break;
                }
            }
            if test_pixel_found { break; }
        }
        
        assert!(test_pixel_found, "Should find at least one testable pixel in the image");
    }

    #[test]
    fn test_contrast_adjustment_visual_with_bird_image() {
        // Skip if bird image not available
        if !Path::new(TEST_BIRD_IMAGE_PATH).exists() {
            return;
        }

        let img_bytes = fs::read(TEST_BIRD_IMAGE_PATH).unwrap();
        let mut original_frame = ImageFrame::new_from_jpeg_bytes(&img_bytes, &ColorSpace::Gamma).unwrap();

        println!("Testing contrast adjustments with bird image ({}x{})", 
                 original_frame.get_xy_resolution().width, 
                 original_frame.get_xy_resolution().height);

        // Save original for comparison (if not already saved by brightness test)
        if std::env::var("SAVE_TEST_IMAGES").is_ok() {
            let original_png = original_frame.export_as_png_bytes().unwrap();
            fs::write("tests/images/contrast_test_original.png", &original_png).unwrap();
        }

        // Test contrast increase
        let mut high_contrast_frame = original_frame.clone();
        high_contrast_frame.change_contrast(2.0);
        if std::env::var("SAVE_TEST_IMAGES").is_ok() {
            let high_contrast_png = high_contrast_frame.export_as_png_bytes().unwrap();
            fs::write("tests/images/contrast_test_increased.png", &high_contrast_png).unwrap();
        }

        // Test contrast decrease
        let mut low_contrast_frame = original_frame.clone();
        low_contrast_frame.change_contrast(0.5);
        if std::env::var("SAVE_TEST_IMAGES").is_ok() {
            let low_contrast_png = low_contrast_frame.export_as_png_bytes().unwrap();
            fs::write("tests/images/contrast_test_decreased.png", &low_contrast_png).unwrap();
        }

        // Test extreme contrast increase
        let mut very_high_contrast_frame = original_frame.clone();
        very_high_contrast_frame.change_contrast(3.0);
        if std::env::var("SAVE_TEST_IMAGES").is_ok() {
            let very_high_contrast_png = very_high_contrast_frame.export_as_png_bytes().unwrap();
            fs::write("tests/images/contrast_test_very_high.png", &very_high_contrast_png).unwrap();
        }

        // Test very low contrast (almost flat)
        let mut very_low_contrast_frame = original_frame.clone();
        very_low_contrast_frame.change_contrast(0.1);
        if std::env::var("SAVE_TEST_IMAGES").is_ok() {
            let very_low_contrast_png = very_low_contrast_frame.export_as_png_bytes().unwrap();
            fs::write("tests/images/contrast_test_very_low.png", &very_low_contrast_png).unwrap();
            
            println!("Contrast test images saved:");
            println!("  - contrast_test_original.png (original)");
            println!("  - contrast_test_increased.png (2.0x contrast)");
            println!("  - contrast_test_decreased.png (0.5x contrast)");
            println!("  - contrast_test_very_high.png (3.0x contrast)");
            println!("  - contrast_test_very_low.png (0.1x contrast)");
        }

        // Verify that contrast adjustments actually changed the data
        let original_pixels = original_frame.get_pixels_view();
        let high_contrast_pixels = high_contrast_frame.get_pixels_view();
        let low_contrast_pixels = low_contrast_frame.get_pixels_view();

        // Find pixels with different brightness levels to test contrast effects
        let mut found_dark_pixel = false;
        let mut found_bright_pixel = false;
        
        for y in 0..original_frame.get_xy_resolution().height.min(20) {
            for x in 0..original_frame.get_xy_resolution().width.min(20) {
                let orig_val = original_pixels[(y, x, 0)] as i32;
                let high_contrast_val = high_contrast_pixels[(y, x, 0)] as i32;
                let low_contrast_val = low_contrast_pixels[(y, x, 0)] as i32;
                
                // Test dark pixels (below midpoint)
                if orig_val < 100 && orig_val > 10 {
                    // Higher contrast should make dark pixels darker (move away from 128)
                    assert!(high_contrast_val <= orig_val, 
                           "High contrast should make dark pixels darker or same, got {} -> {}", orig_val, high_contrast_val);
                    // Lower contrast should make dark pixels closer to middle
                    assert!(low_contrast_val >= orig_val, 
                           "Low contrast should make dark pixels lighter, got {} -> {}", orig_val, low_contrast_val);
                    found_dark_pixel = true;
                }
                
                // Test bright pixels (above midpoint)
                if orig_val > 156 && orig_val < 245 {
                    // Higher contrast should make bright pixels brighter (move away from 128)
                    assert!(high_contrast_val >= orig_val, 
                           "High contrast should make bright pixels brighter or same, got {} -> {}", orig_val, high_contrast_val);
                    // Lower contrast should make bright pixels closer to middle
                    assert!(low_contrast_val <= orig_val, 
                           "Low contrast should make bright pixels darker, got {} -> {}", orig_val, low_contrast_val);
                    found_bright_pixel = true;
                }
            }
        }
        
        // Note: We don't require finding both types of pixels since some images might be predominantly light or dark
        assert!(found_dark_pixel || found_bright_pixel, 
               "Should find at least one testable pixel (either dark or bright) in the image");
    }
}

#[cfg(test)]
mod test_segmented_image_frame {
    use super::*;

    #[test]
    fn test_segmented_image_frame_creation_new() {
        let center_resolution = ImageXYResolution::new(64, 64).unwrap();
        let peripheral_resolution = ImageXYResolution::new(32, 32).unwrap();
        
        let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            center_resolution,
            peripheral_resolution
        );

        let segmented_frame = SegmentedImageFrame::new(
            &resolutions,
            &ColorSpace::Gamma,
            &ColorChannelLayout::RGB,     // Center with RGB
            &ColorChannelLayout::GrayScale // Peripherals with grayscale
        ).unwrap();

        // Verify center segment properties
        assert_eq!(*segmented_frame.get_center_channel_layout(), ColorChannelLayout::RGB);
        assert_eq!(*segmented_frame.get_color_space(), ColorSpace::Gamma);

        // Verify peripheral segment properties
        assert_eq!(*segmented_frame.get_peripheral_channel_layout(), ColorChannelLayout::GrayScale);

        // Verify resolutions
        let frame_resolutions = segmented_frame.get_segmented_frame_target_resolutions();
        assert_eq!(frame_resolutions.center, center_resolution);
        assert_eq!(frame_resolutions.lower_left, peripheral_resolution);
        assert_eq!(frame_resolutions.upper_right, peripheral_resolution);
    }

    #[test]
    fn test_segmented_image_frame_from_properties() {
        let center_resolution = ImageXYResolution::new(128, 96).unwrap();
        let peripheral_resolution = ImageXYResolution::new(32, 24).unwrap();
        
        let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            center_resolution,
            peripheral_resolution
        );

        let properties = SegmentedImageFrameProperties::new(
            &resolutions,
            &ColorChannelLayout::RGBA,
            &ColorChannelLayout::RGB,
            &ColorSpace::Linear
        );

        let segmented_frame = SegmentedImageFrame::from_segmented_image_frame_properties(&properties).unwrap();

        // Verify properties match
        let frame_properties = segmented_frame.get_segmented_image_frame_properties();
        assert_eq!(*frame_properties.get_center_color_channel(), ColorChannelLayout::RGBA);
        assert_eq!(*frame_properties.get_peripheral_color_channels(), ColorChannelLayout::RGB);
        assert_eq!(*frame_properties.get_color_space(), ColorSpace::Linear);
        
        let frame_resolutions = frame_properties.get_resolutions();
        assert_eq!(frame_resolutions.center, center_resolution);
        assert_eq!(frame_resolutions.lower_left, peripheral_resolution);
    }

    #[test]
    fn test_create_ordered_cortical_ids_for_segmented_vision() {
        let camera_index = CorticalGroupIndex::from(5u8);
        let cortical_ids = SegmentedImageFrame::create_ordered_cortical_ids_for_segmented_vision(camera_index);

        // Verify we got 9 IDs
        assert_eq!(cortical_ids.len(), 9);

        // Verify the order matches the expected layout
        // [0] Bottom-Left, [1] Bottom-Middle, [2] Bottom-Right
        // [3] Middle-Left, [4] Center, [5] Middle-Right  
        // [6] Top-Left, [7] Top-Middle, [8] Top-Right
        let expected_types = [
            SensorCorticalType::ImageCameraBottomLeft,
            SensorCorticalType::ImageCameraBottomMiddle,
            SensorCorticalType::ImageCameraBottomRight,
            SensorCorticalType::ImageCameraMiddleLeft,
            SensorCorticalType::ImageCameraCenter,
            SensorCorticalType::ImageCameraMiddleRight,
            SensorCorticalType::ImageCameraTopLeft,
            SensorCorticalType::ImageCameraTopMiddle,
            SensorCorticalType::ImageCameraTopRight,
        ];

        for (i, expected_type) in expected_types.iter().enumerate() {
            let expected_id = expected_type.to_cortical_id(camera_index);
            assert_eq!(cortical_ids[i], expected_id, "Cortical ID mismatch at index {}", i);
        }
    }

    #[test]
    fn test_create_ordered_cortical_types_for_segmented_vision() {
        let cortical_types = SegmentedImageFrame::create_ordered_cortical_types_for_segmented_vision();

        // Verify we got 9 types
        assert_eq!(cortical_types.len(), 9);

        // Verify the expected order
        let expected_sensor_types = [
            SensorCorticalType::ImageCameraBottomLeft,
            SensorCorticalType::ImageCameraBottomMiddle,
            SensorCorticalType::ImageCameraBottomRight,
            SensorCorticalType::ImageCameraMiddleLeft,
            SensorCorticalType::ImageCameraCenter,
            SensorCorticalType::ImageCameraMiddleRight,
            SensorCorticalType::ImageCameraTopLeft,
            SensorCorticalType::ImageCameraTopMiddle,
            SensorCorticalType::ImageCameraTopRight,
        ];

        for (i, expected_sensor_type) in expected_sensor_types.iter().enumerate() {
            let expected_cortical_type: CorticalType = (*expected_sensor_type).into();
            assert_eq!(cortical_types[i], expected_cortical_type, "Cortical type mismatch at index {}", i);
        }
    }

    #[test]
    fn test_get_image_internal_data() {
        let center_resolution = ImageXYResolution::new(16, 16).unwrap();
        let peripheral_resolution = ImageXYResolution::new(8, 8).unwrap();
        
        let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            center_resolution,
            peripheral_resolution
        );

        let segmented_frame = SegmentedImageFrame::new(
            &resolutions,
            &ColorSpace::Gamma,
            &ColorChannelLayout::RGB,
            &ColorChannelLayout::GrayScale
        ).unwrap();

        let internal_data = segmented_frame.get_image_internal_data();
        
        // Verify we got 9 arrays
        assert_eq!(internal_data.len(), 9);

        // Verify the shapes are correct
        // Index 4 should be center (RGB, 16x16)
        assert_eq!(internal_data[4].shape(), &[16, 16, 3]); // Center: RGB 16x16
        
        // Other indices should be peripheral (Grayscale, 8x8)
        for i in [0, 1, 2, 3, 5, 6, 7, 8] {
            assert_eq!(internal_data[i].shape(), &[8, 8, 1], "Peripheral segment {} should be 8x8x1", i); // Peripheral: Grayscale 8x8
        }
    }

    #[test]
    fn test_get_ordered_image_frame_references() {
        let center_resolution = ImageXYResolution::new(32, 32).unwrap();
        let peripheral_resolution = ImageXYResolution::new(16, 16).unwrap();
        
        let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            center_resolution,
            peripheral_resolution
        );

        let segmented_frame = SegmentedImageFrame::new(
            &resolutions,
            &ColorSpace::Linear,
            &ColorChannelLayout::RGBA,
            &ColorChannelLayout::RGB
        ).unwrap();

        let frame_refs = segmented_frame.get_ordered_image_frame_references();
        
        // Verify we got 9 references
        assert_eq!(frame_refs.len(), 9);

        // The ordering should be: center, lower_left, middle_left, upper_left, upper_middle,
        // upper_right, middle_right, lower_right, lower_middle
        
        // Verify center is fourth and has correct properties
        assert_eq!(*frame_refs[4].get_channel_layout(), ColorChannelLayout::RGBA);
        assert_eq!(frame_refs[4].get_xy_resolution(), center_resolution);

        // Verify peripherals have correct properties
        for i in 0..4 {
            assert_eq!(*frame_refs[i].get_channel_layout(), ColorChannelLayout::RGB);
            assert_eq!(frame_refs[i].get_xy_resolution(), peripheral_resolution);
        }
        for i in 5..9 {
            assert_eq!(*frame_refs[i].get_channel_layout(), ColorChannelLayout::RGB);
            assert_eq!(frame_refs[i].get_xy_resolution(), peripheral_resolution);
        }
    }

    #[test]
    fn test_get_mut_ordered_image_frame_references() {
        let center_resolution = ImageXYResolution::new(16, 16).unwrap();
        let peripheral_resolution = ImageXYResolution::new(8, 8).unwrap();
        
        let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            center_resolution,
            peripheral_resolution
        );

        let mut segmented_frame = SegmentedImageFrame::new(
            &resolutions,
            &ColorSpace::Gamma,
            &ColorChannelLayout::RGB,
            &ColorChannelLayout::GrayScale
        ).unwrap();

        // Test mutable access
        {
            let mut_frame_refs = segmented_frame.get_mut_ordered_image_frame_references();
            assert_eq!(mut_frame_refs.len(), 9);

            // Modify a pixel in the center frame (index 4)
            let mut center_pixels = mut_frame_refs[4].get_pixels_view_mut();
            center_pixels[(0, 0, 0)] = 255;
            center_pixels[(0, 0, 1)] = 128;
            center_pixels[(0, 0, 2)] = 64;
        }

        // Verify the change was made
        let frame_refs = segmented_frame.get_ordered_image_frame_references();
        let center_pixels = frame_refs[4].get_pixels_view();
        assert_eq!(center_pixels[(0, 0, 0)], 255);
        assert_eq!(center_pixels[(0, 0, 1)], 128);
        assert_eq!(center_pixels[(0, 0, 2)], 64);
    }

    #[test]
    fn test_segmented_xy_image_resolutions_create_with_same_sized_peripheral() {
        let center_resolution = ImageXYResolution::new(100, 80).unwrap();
        let peripheral_resolution = ImageXYResolution::new(25, 20).unwrap();
        
        let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            center_resolution,
            peripheral_resolution
        );

        // Verify center resolution
        assert_eq!(resolutions.center, center_resolution);

        // Verify all peripheral resolutions are the same
        assert_eq!(resolutions.lower_left, peripheral_resolution);
        assert_eq!(resolutions.lower_middle, peripheral_resolution);
        assert_eq!(resolutions.lower_right, peripheral_resolution);
        assert_eq!(resolutions.middle_left, peripheral_resolution);
        assert_eq!(resolutions.middle_right, peripheral_resolution);
        assert_eq!(resolutions.upper_left, peripheral_resolution);
        assert_eq!(resolutions.upper_middle, peripheral_resolution);
        assert_eq!(resolutions.upper_right, peripheral_resolution);
    }

    #[test]
    fn test_segmented_xy_image_resolutions_new_with_different_sizes() {
        // Create different resolutions for each segment
        let lower_left = ImageXYResolution::new(16, 12).unwrap();
        let lower_middle = ImageXYResolution::new(32, 24).unwrap();
        let lower_right = ImageXYResolution::new(16, 12).unwrap();
        let middle_left = ImageXYResolution::new(24, 32).unwrap();
        let center = ImageXYResolution::new(64, 64).unwrap();
        let middle_right = ImageXYResolution::new(24, 32).unwrap();
        let upper_left = ImageXYResolution::new(16, 12).unwrap();
        let upper_middle = ImageXYResolution::new(32, 24).unwrap();
        let upper_right = ImageXYResolution::new(16, 12).unwrap();

        let resolutions = SegmentedXYImageResolutions::new(
            lower_left, lower_middle, lower_right,
            middle_left, center, middle_right,
            upper_left, upper_middle, upper_right
        );

        // Verify each resolution is set correctly
        assert_eq!(resolutions.lower_left, lower_left);
        assert_eq!(resolutions.lower_middle, lower_middle);
        assert_eq!(resolutions.lower_right, lower_right);
        assert_eq!(resolutions.middle_left, middle_left);
        assert_eq!(resolutions.center, center);
        assert_eq!(resolutions.middle_right, middle_right);
        assert_eq!(resolutions.upper_left, upper_left);
        assert_eq!(resolutions.upper_middle, upper_middle);
        assert_eq!(resolutions.upper_right, upper_right);
    }

    #[test]
    fn test_clone_trait() {
        let center_resolution = ImageXYResolution::new(32, 32).unwrap();
        let peripheral_resolution = ImageXYResolution::new(16, 16).unwrap();
        
        let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            center_resolution,
            peripheral_resolution
        );

        let mut original_frame = SegmentedImageFrame::new(
            &resolutions,
            &ColorSpace::Gamma,
            &ColorChannelLayout::RGB,
            &ColorChannelLayout::GrayScale
        ).unwrap();

        // Modify some data in the original
        {
            let mut frame_refs = original_frame.get_mut_ordered_image_frame_references();
            let mut center_pixels = frame_refs[0].get_pixels_view_mut();
            center_pixels[(0, 0, 0)] = 200;
        }

        // Clone the frame
        let cloned_frame = original_frame.clone();

        // Verify properties match
        assert_eq!(cloned_frame.get_center_channel_layout(), original_frame.get_center_channel_layout());
        assert_eq!(cloned_frame.get_peripheral_channel_layout(), original_frame.get_peripheral_channel_layout());
        assert_eq!(cloned_frame.get_color_space(), original_frame.get_color_space());
        
        // Verify data was cloned
        let original_refs = original_frame.get_ordered_image_frame_references();
        let cloned_refs = cloned_frame.get_ordered_image_frame_references();
        
        let original_center_pixels = original_refs[0].get_pixels_view();
        let cloned_center_pixels = cloned_refs[0].get_pixels_view();
        assert_eq!(original_center_pixels[(0, 0, 0)], cloned_center_pixels[(0, 0, 0)]);

        // Verify they are independent
        {
            let mut frame_refs = original_frame.get_mut_ordered_image_frame_references();
            let mut center_pixels = frame_refs[0].get_pixels_view_mut();
            center_pixels[(0, 0, 0)] = 100; // Change original
        }

        // Cloned should be unchanged
        let cloned_center_pixels = cloned_refs[0].get_pixels_view();
        assert_eq!(cloned_center_pixels[(0, 0, 0)], 200); // Still original value
    }

    #[test]
    fn test_debug_trait() {
        let center_resolution = ImageXYResolution::new(32, 32).unwrap();
        let peripheral_resolution = ImageXYResolution::new(16, 16).unwrap();
        
        let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            center_resolution,
            peripheral_resolution
        );

        let segmented_frame = SegmentedImageFrame::new(
            &resolutions,
            &ColorSpace::Gamma,
            &ColorChannelLayout::RGB,
            &ColorChannelLayout::GrayScale
        ).unwrap();

        let debug_string = format!("{:?}", segmented_frame);
        assert!(debug_string.contains("SegmentedImageFrame"));
        // Should contain the internal structure
        assert!(debug_string.contains("lower_left"));
        assert!(debug_string.contains("center"));
        assert!(debug_string.contains("upper_right"));
    }

    #[test]
    fn test_display_trait() {
        let center_resolution = ImageXYResolution::new(32, 32).unwrap();
        let peripheral_resolution = ImageXYResolution::new(16, 16).unwrap();
        
        let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            center_resolution,
            peripheral_resolution
        );

        let segmented_frame = SegmentedImageFrame::new(
            &resolutions,
            &ColorSpace::Gamma,
            &ColorChannelLayout::RGB,
            &ColorChannelLayout::GrayScale
        ).unwrap();

        let display_string = format!("{}", segmented_frame);
        assert_eq!(display_string, "SegmentedImageFrame()");
    }

    #[test]
    fn test_different_color_spaces_and_channels() {
        let center_resolution = ImageXYResolution::new(40, 40).unwrap();
        let peripheral_resolution = ImageXYResolution::new(20, 20).unwrap();
        
        let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            center_resolution,
            peripheral_resolution
        );

        // Test Linear color space with RGBA center and RG peripherals
        let segmented_frame = SegmentedImageFrame::new(
            &resolutions,
            &ColorSpace::Linear,
            &ColorChannelLayout::RGBA,  // Center with alpha
            &ColorChannelLayout::RG     // Peripherals with 2 channels
        ).unwrap();

        assert_eq!(*segmented_frame.get_color_space(), ColorSpace::Linear);
        assert_eq!(*segmented_frame.get_center_channel_layout(), ColorChannelLayout::RGBA);
        assert_eq!(*segmented_frame.get_peripheral_channel_layout(), ColorChannelLayout::RG);

        // Verify internal data shapes
        let internal_data = segmented_frame.get_image_internal_data();
        assert_eq!(internal_data[4].shape(), &[40, 40, 4]); // Center: RGBA
        for i in [0, 1, 2, 3, 5, 6, 7, 8] {
            assert_eq!(internal_data[i].shape(), &[20, 20, 2]); // Peripherals: RG
        }
    }
}
