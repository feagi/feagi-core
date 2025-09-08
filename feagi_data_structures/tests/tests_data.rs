//! Tests for the data module
//! 
//! This module contains comprehensive tests for data structures including ImageFrame
//! and related functionality for image processing, import, and export operations.

use feagi_data_structures::data::ImageFrame;
use feagi_data_structures::data::image_descriptors::{ColorChannelLayout, ColorSpace, ImageXYResolution, ImageFrameProperties, MemoryOrderLayout};
use ndarray::Array3;
use std::fs;
use std::path::Path;

#[cfg(test)]
mod test_image_frame {
    use image::GenericImageView;
    use super::*;

    const TEST_BIRD_IMAGE_PATH: &str = "tests/bird.jpg";

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
}
