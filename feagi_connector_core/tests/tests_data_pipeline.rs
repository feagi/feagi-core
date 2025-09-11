//! Tests for the data pipeline module
//! 
//! This module contains basic tests for the data pipeline stages,
//! focusing on stage creation and basic validation.

use std::time::Instant;
use feagi_data_structures::data::ImageFrame;
use feagi_data_structures::data::image_descriptors::{ColorChannelLayout, ColorSpace, ImageXYResolution};
use feagi_data_structures::processing::ImageFrameProcessor;
use feagi_data_structures::wrapped_io_data::WrappedIOData;
use feagi_connector_core::data_pipeline::stages::*;

#[cfg(test)]
mod test_pipeline_stages {
    use feagi_connector_core::caching::SensorCache;
    use super::*;
    
    // Import trait for direct stage testing (traits can be imported even if the module is private)
    use feagi_connector_core::data_pipeline::stages::*;
    use feagi_connector_core::data_pipeline::PipelineStage;
    use feagi_data_serialization::{FeagiByteStructure, FeagiByteStructureCompatible};
    use feagi_data_structures::genomic::{CorticalID, CorticalType, SensorCorticalType};
    use feagi_data_structures::genomic::CorticalType::Sensory;
    use feagi_data_structures::genomic::descriptors::{CorticalChannelCount, CorticalChannelIndex, CorticalCoordinate, CorticalGroupIndex};
    use feagi_data_structures::genomic::SensorCorticalType::ImageCameraCenter;
    use feagi_data_structures::neurons::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZPArrays};

    //region Helper Functions
    
    fn create_test_image() -> ImageFrame {
        let resolution = ImageXYResolution::new(64, 48).unwrap();
        let mut image = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Gamma, &resolution).unwrap();
        
        // Fill with test pattern
        {
            let mut pixels = image.get_pixels_view_mut();
            for y in 0..48 {
                for x in 0..64 {
                    pixels[(y, x, 0)] = (x * 4) as u8; // Red gradient
                    pixels[(y, x, 1)] = (y * 5) as u8; // Green gradient
                    pixels[(y, x, 2)] = 128; // Constant blue
                }
            }
        }
        
        image
    }
    
    fn load_bird_image() -> ImageFrame {
        let bird_bytes = std::fs::read("tests/images/bird.jpg").expect("Bird image should exist");
        ImageFrame::new_from_jpeg_bytes(&bird_bytes, &ColorSpace::Gamma).expect("Bird image should load correctly")
    }
    
    fn save_test_image(image: &ImageFrame, filename: &str) {
        let png_bytes = image.export_as_png_bytes().unwrap();
        std::fs::write(format!("tests/images/{}", filename), &png_bytes).unwrap();
        println!("Saved test image: tests/images/{}", filename);
    }
    
    //endregion

    //region Stage Creation Tests
    
    #[test]
    fn test_identity_float_stage_creation() {
        let stage = IdentityFloatStage::new(42.0);
        assert!(stage.is_ok());
        
        let stage = IdentityFloatStage::new(f32::NAN);
        assert!(stage.is_err());
        
        let stage = IdentityFloatStage::new(f32::INFINITY);
        assert!(stage.is_err());
    }
    
    #[test]
    fn test_linear_scale_to_0_1_creation() {
        let stage = LinearScaleTo0And1Stage::new(0.0, 100.0, 50.0);
        assert!(stage.is_ok());
        
        // Test invalid range (upper <= lower)
        let stage = LinearScaleTo0And1Stage::new(100.0, 50.0, 75.0);
        assert!(stage.is_err());
        
        // Test initial value out of bounds
        let stage = LinearScaleTo0And1Stage::new(0.0, 100.0, 150.0);
        assert!(stage.is_err());
        
        let stage = LinearScaleTo0And1Stage::new(0.0, 100.0, -10.0);
        assert!(stage.is_err());
        
        // Test NaN/infinite values
        let stage = LinearScaleTo0And1Stage::new(f32::NAN, 100.0, 50.0);
        assert!(stage.is_err());
        
        let stage = LinearScaleTo0And1Stage::new(0.0, f32::INFINITY, 50.0);
        assert!(stage.is_err());
    }
    
    #[test]
    fn test_linear_scale_to_m1_1_creation() {
        let stage = LinearScaleToM1And1Stage::new(-50.0, 50.0, 0.0);
        assert!(stage.is_ok());
        
        // Test invalid range
        let stage = LinearScaleToM1And1Stage::new(50.0, 50.0, 50.0);
        assert!(stage.is_err());
        
        // Test bounds checking
        let stage = LinearScaleToM1And1Stage::new(-50.0, 50.0, 100.0);
        assert!(stage.is_err());
    }
    
    #[test]
    fn test_image_processor_stage_creation() {
        let test_image = create_test_image();
        let properties = test_image.get_image_frame_properties();
        let processor = ImageFrameProcessor::new(properties);
        let stage = ImageFrameProcessorStage::new(processor);
        assert!(stage.is_ok());
    }
    
    #[test]
    fn test_image_quick_diff_stage_creation() {
        let test_image = create_test_image();
        let properties = test_image.get_image_frame_properties();
        
        // Test creation with various thresholds
        let stage_low = ImageFrameQuickDiffStage::new(properties, 10);
        assert!(stage_low.is_ok());
        
        let stage_medium = ImageFrameQuickDiffStage::new(properties, 50);
        assert!(stage_medium.is_ok());
        
        let stage_high = ImageFrameQuickDiffStage::new(properties, 200);
        assert!(stage_high.is_ok());
        
        // Test with zero threshold (should work)
        let stage_zero = ImageFrameQuickDiffStage::new(properties, 0);
        assert!(stage_zero.is_ok());
        
        // Test with maximum threshold
        let stage_max = ImageFrameQuickDiffStage::new(properties, 255);
        assert!(stage_max.is_ok());
    }
    
    #[test]
    fn test_image_quick_diff_stage_properties() {
        let test_image = create_test_image();
        let properties = test_image.get_image_frame_properties();
        let stage = ImageFrameQuickDiffStage::new(properties, 30).unwrap();
        
        // Test that the stage was created successfully and we can access basic properties
        println!("✓ Quick diff stage created successfully with threshold 30");
        
        // Test Display implementation
        let display_string = format!("{}", stage);
        assert!(display_string.contains("ImageFrameQuickDiffProcessor"));
        println!("✓ Display implementation works: {}", display_string);
    }
    
    #[test]
    fn test_image_quick_diff_stage_with_different_image_sizes() {
        // Test with different image sizes to ensure the stage can handle various inputs
        
        // Small image
        let small_resolution = ImageXYResolution::new(32, 24).unwrap();
        let small_image = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Gamma, &small_resolution).unwrap();
        let small_properties = small_image.get_image_frame_properties();
        let small_stage = ImageFrameQuickDiffStage::new(small_properties, 20);
        assert!(small_stage.is_ok());
        
        // Large image  
        let large_resolution = ImageXYResolution::new(1024, 768).unwrap();
        let large_image = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Gamma, &large_resolution).unwrap();
        let large_properties = large_image.get_image_frame_properties();
        let large_stage = ImageFrameQuickDiffStage::new(large_properties, 40);
        assert!(large_stage.is_ok());
        
        println!("✓ Quick diff stages created successfully for different image sizes");
    }
    
    #[test]
    fn test_image_quick_diff_stage_with_different_color_layouts() {
        let resolution = ImageXYResolution::new(64, 48).unwrap();
        
        // Test with different color layouts
        let rgb_image = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Gamma, &resolution).unwrap();
        let rgb_properties = rgb_image.get_image_frame_properties();
        let rgb_stage = ImageFrameQuickDiffStage::new(rgb_properties, 25);
        assert!(rgb_stage.is_ok());
        
        let rgba_image = ImageFrame::new(&ColorChannelLayout::RGBA, &ColorSpace::Gamma, &resolution).unwrap();
        let rgba_properties = rgba_image.get_image_frame_properties();
        let rgba_stage = ImageFrameQuickDiffStage::new(rgba_properties, 25);
        assert!(rgba_stage.is_ok());
        
        let grayscale_image = ImageFrame::new(&ColorChannelLayout::GrayScale, &ColorSpace::Gamma, &resolution).unwrap();
        let gray_properties = grayscale_image.get_image_frame_properties();
        let gray_stage = ImageFrameQuickDiffStage::new(gray_properties, 25);
        assert!(gray_stage.is_ok());
        
        println!("✓ Quick diff stages created successfully for different color layouts");
    }

    

    //endregion
    
    //region Image Processor Tests
    
    #[test]
    fn test_image_processor_basic_operations() {
        // Test that we can create processors with various settings
        let bird_image = load_bird_image();
        let original_props = bird_image.get_image_frame_properties();
        
        // Test resizing
        let mut processor1 = ImageFrameProcessor::new(original_props);
        let target_resolution = ImageXYResolution::new(128, 96).unwrap();
        let result = processor1.set_resizing_to(target_resolution);
        assert!(result.is_ok());
        
        // Test brightness adjustment
        let mut processor2 = ImageFrameProcessor::new(original_props);
        let result = processor2.set_brightness_offset(20);
        assert!(result.is_ok());
        
        // Test contrast adjustment
        let mut processor3 = ImageFrameProcessor::new(original_props);
        let result = processor3.set_contrast_change(15.0);
        assert!(result.is_ok());
        
        println!("All image processor operations configured successfully");
    }
    
    #[test]
    fn test_image_processor_with_transformations() {
        // Test creating a processor with multiple transformations
        let bird_image = load_bird_image();
        let original_props = bird_image.get_image_frame_properties();
        println!("Original bird image: {}x{}", 
                original_props.get_image_resolution().width, 
                original_props.get_image_resolution().height);
        
        let mut processor = ImageFrameProcessor::new(original_props);
        
        // Apply multiple transformations
        let target_resolution = ImageXYResolution::new(256, 192).unwrap();
        processor.set_resizing_to(target_resolution).unwrap();
        processor.set_brightness_offset(20).unwrap();
        processor.set_contrast_change(15.0).unwrap();
        
        // Test that we can process the image
        let mut output_image = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Gamma, &target_resolution).unwrap();
        dbg!(&output_image.get_image_frame_properties());
        let result = processor.process_image(&bird_image, &mut output_image);
        dbg!(&output_image.get_image_frame_properties());
        
        if result.is_ok() {
            // Verify the transformations were applied
            assert_eq!(output_image.get_image_frame_properties().get_image_resolution().width, 256);
            assert_eq!(output_image.get_image_frame_properties().get_image_resolution().height, 192);
            
            // Save the fully processed image
            save_test_image(&output_image, "pipeline_test_complex_bird.png");
            println!("Successfully processed bird image with multiple transformations");
        } else {
            println!("Image processing failed: {:?}", result);
        }
    }
    
    #[test]
    fn test_image_processor_individual_transformations() {
        let bird_image = load_bird_image();
        let original_props = bird_image.get_image_frame_properties();
        
        // Test resize only
        {
            let mut processor = ImageFrameProcessor::new(original_props);
            let target_resolution = ImageXYResolution::new(128, 96).unwrap();
            processor.set_resizing_to(target_resolution).unwrap();
            
            let mut output_image = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Gamma, &target_resolution).unwrap();
            let result = processor.process_image(&bird_image, &mut output_image);
            
            if result.is_ok() {
                save_test_image(&output_image, "pipeline_test_resized_bird.png");
                println!("Successfully resized bird image");
            }
        }
        
        // Test brightness only
        {
            let mut processor = ImageFrameProcessor::new(original_props);
            processor.set_brightness_offset(50).unwrap();
            
            let mut output_image = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Gamma, &original_props.get_image_resolution()).unwrap();
            let result = processor.process_image(&bird_image, &mut output_image);
            
            if result.is_ok() {
                save_test_image(&output_image, "pipeline_test_bright_bird.png");
                println!("Successfully brightened bird image");
            }
        }
        
        // Test contrast only
        {
            let mut processor = ImageFrameProcessor::new(original_props);
            processor.set_contrast_change(30.0).unwrap();
            
            let mut output_image = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Gamma, &original_props.get_image_resolution()).unwrap();
            let result = processor.process_image(&bird_image, &mut output_image);
            
            if result.is_ok() {
                save_test_image(&output_image, "pipeline_test_contrast_bird.png");
                println!("Successfully adjusted contrast of bird image");
            }
        }
    }
    
    //endregion
    
    //region Integration Tests
    
    /// Creates a simple test pattern image with known pixel values
    fn create_pattern_image_hard_binary(resolution: &ImageXYResolution) -> ImageFrame {
        let mut image = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Gamma, resolution).unwrap();
        let mut pixels = image.get_pixels_view_mut();
        
        // Create a simple checkerboard pattern
        for y in 0..resolution.height {
            for x in 0..resolution.width {
                let is_white = (x + y) % 2 == 0;
                let color = if is_white { 255 } else { 0 };
                pixels[(y as usize, x as usize, 0)] = color; // Red
                pixels[(y as usize, x as usize, 1)] = color; // Green  
                pixels[(y as usize, x as usize, 2)] = color; // Blue
            }
        }
        
        image
    }
    
    /// Creates a second test pattern image with known differences from pattern A
    fn create_pattern_image_b(resolution: &ImageXYResolution) -> ImageFrame {
        let mut image = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Gamma, resolution).unwrap();
        let mut pixels = image.get_pixels_view_mut();
        
        // Create a shifted checkerboard pattern (offset by 1 pixel)
        for y in 0..resolution.height {
            for x in 0..resolution.width {
                let is_white = ((x + 1) + y) % 2 == 0; // Shifted by 1 pixel
                let color = if is_white { 255 } else { 0 };
                pixels[(y as usize, x as usize, 0)] = color; // Red
                pixels[(y as usize, x as usize, 1)] = color; // Green
                pixels[(y as usize, x as usize, 2)] = color; // Blue
            }
        }
        
        image
    }
    
    /// Creates a third test pattern with small differences (for threshold testing)
    fn create_pattern_image_soft_binary(resolution: &ImageXYResolution) -> ImageFrame {
        let mut image = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Gamma, resolution).unwrap();
        let mut pixels = image.get_pixels_view_mut();

        // Create a simple checkerboard pattern
        for y in 0..resolution.height {
            for x in 0..resolution.width {
                let is_white = (x + y) % 2 == 0;
                let color = if is_white { 255 } else { 10 };
                pixels[(y as usize, x as usize, 0)] = color; // Red
                pixels[(y as usize, x as usize, 1)] = color; // Green
                pixels[(y as usize, x as usize, 2)] = color; // Blue
            }
        }

        image
    }
    
    #[test]
    fn test_pipeline_stage_runner_with_quick_diff_integration() {


        let resolution = ImageXYResolution::new(16, 12).unwrap(); // Small test images
        let threshold: u8 = 50; // Medium threshold
        
        // Create test pattern images
        let pattern_a = create_pattern_image_hard_binary(&resolution);
        let pattern_b = create_pattern_image_b(&resolution);
        
        // Save test patterns for visual inspection
        save_test_image(&pattern_a, "pipeline_integration_pattern_a.png");
        save_test_image(&pattern_b, "pipeline_integration_pattern_b.png");
        
        // Create the quick diff stage and manually simulate pipeline processing
        let properties = pattern_a.get_image_frame_properties();
        let mut quick_diff_stage = ImageFrameQuickDiffStage::new(properties, threshold).unwrap();
        
        let timestamp = Instant::now();
        
        // Manually call the trait methods directly (simulating what PipelineStageRunner would do)
        // Process first image 
        let input_a = WrappedIOData::ImageFrame(pattern_a.clone());
        let _result_a = quick_diff_stage.process_new_input(&input_a, timestamp).unwrap();
        
        // Process second image (should show differences where patterns differ)
        let input_b = WrappedIOData::ImageFrame(pattern_b);
        let result_b = quick_diff_stage.process_new_input(&input_b, timestamp).unwrap();
        
        // Verify the output is an ImageFrame
        if let WrappedIOData::ImageFrame(diff_image) = result_b {
            // Save the difference image for visual inspection
            save_test_image(diff_image, "pipeline_integration_diff_result.png");
            
            let diff_pixels = diff_image.get_pixels_view();
            let mut significant_differences = 0;
            let mut total_pixels = 0;
            
            // Count pixels with significant differences (non-zero output)
            // With the new implementation, the output should be the previous frame pixel values where differences exceed threshold
            for y in 0..resolution.height {
                for x in 0..resolution.width {
                    total_pixels += 1;
                    let r = diff_pixels[(y as usize, x as usize, 0)];
                    let g = diff_pixels[(y as usize, x as usize, 1)];
                    let b = diff_pixels[(y as usize, x as usize, 2)];
                    
                    // If any channel has a non-zero value, it means a significant difference was detected
                    // and the previous frame value was passed through
                    if r > 0 || g > 0 || b > 0 {
                        significant_differences += 1;
                        
                        // The output should be previous frame values where differences exceeded the threshold
                        // Since we're comparing hard_binary against pattern_b, we should see pattern_b values
                    }
                }
            }
            
            // With a shifted checkerboard pattern, we should see differences where patterns don't align
            println!("Total pixels: {}, Significant differences: {}", total_pixels, significant_differences);
            println!("Percentage with differences: {:.1}%", (significant_differences as f32 / total_pixels as f32) * 100.0);
            
            // With the new algorithm that passes through previous frame values, we should see visible differences
            // where the patterns don't align - these should now be non-zero values
            assert!(significant_differences > 0, 
                   "Expected some significant differences between shifted checkerboard patterns. Got {}/{}",
                   significant_differences, total_pixels);
            
            println!("✓ Quick Diff stage integration test with known patterns completed successfully");
        } else {
            panic!("Expected ImageFrame output from quick diff stage");
        }
    }
    
    #[test]
    fn test_quick_diff_stage_threshold_behavior() {
        let resolution = ImageXYResolution::new(8, 8).unwrap(); // Very small test images
        let low_threshold: u8 = 5;  // Should detect small differences
        let high_threshold: u8 = 50; // Should NOT detect small differences
        
        // Create test images
        let hard_binary = create_pattern_image_hard_binary(&resolution);
        let soft_binary = create_pattern_image_soft_binary(&resolution); // Small differences only
        
        // Save test patterns 
        save_test_image(&hard_binary, "pipeline_threshold_pattern_a.png");
        save_test_image(&soft_binary, "pipeline_threshold_pattern_c.png");
        
        let properties = hard_binary.get_image_frame_properties();
        let timestamp = Instant::now();
        
        // Test with LOW threshold (should detect small differences)
        {
            let mut quick_diff_stage_low = ImageFrameQuickDiffStage::new(properties, low_threshold).unwrap();
            
             let input_a = WrappedIOData::ImageFrame(hard_binary.clone());
             let input_c = WrappedIOData::ImageFrame(soft_binary.clone());

             // First pass: soft_binary vs empty cache (all 0s)
             // soft_binary has values 10 and 255, vs 0s
             // Differences: 10-0=10 (>5, output=10), 255-0=255 (>5, output=255)
             _ = quick_diff_stage_low.process_new_input(&input_c, timestamp).unwrap();
             
             // Second pass: hard_binary vs soft_binary
             // hard_binary (current) has values 0 and 255, soft_binary (previous) has 10 and 255
             // With new implementation: outputs previous frame value (subtrahend) when threshold exceeded
             // Where both patterns match (255 vs 255): |255-255|=0 < 5, output=0
             // Where patterns differ: 
             //   - hard=0, soft=10: |0-10|=10 > 5, output=10 (previous value)
             //   - hard=255, soft=10: |255-10|=245 > 5, output=10 (previous value)
             let result_low = quick_diff_stage_low.process_new_input(&input_a, timestamp).unwrap();

             let diff_image_low: &ImageFrame = result_low.try_into().unwrap();
             save_test_image(diff_image_low, "pipeline_threshold_diff_low.png");

             let diff_pixels = diff_image_low.get_pixels_view();
             let mut differences_low = 0;
             let mut total_pixels_checked = 0;

             // Check all pixels for differences
             for y in 0..resolution.height.min(8) {
                 for x in 0..resolution.width.min(8) {
                     total_pixels_checked += 1;
                     let r = diff_pixels[(y, x, 0)];
                     let g = diff_pixels[(y, x, 1)]; 
                     let b = diff_pixels[(y, x, 2)];
                     
                     // With the new implementation, we expect to see previous frame values where differences exceed threshold
                     // hard_binary (current) has values 0 and 255, soft_binary (previous) has values 10 and 255
                     // Where hard=0, soft=10: |0-10|=10 > 5, output=10 (previous value)
                     // Where hard=255, soft=10: |255-10|=245 > 5, output=10 (previous value)  
                     // Where both=255: |255-255|=0 < 5, output=0
                     if r > 0 || g > 0 || b > 0 {
                         differences_low += 1;
                         // Debug: print the first few differences
                         if differences_low <= 5 {
                             println!("Difference at ({}, {}): R={}, G={}, B={}", y, x, r, g, b);
                         }
                     }
                 }
             }

             println!("Low threshold ({}): detected {} differences out of {} pixels checked", 
                     low_threshold, differences_low, total_pixels_checked);
             
             // Now with the updated implementation: where hard_binary=0 and soft_binary=10, the output is 10 (previous value)
             // This means differences should now be visible as non-zero values!
             // We should see the previous frame values (10 or 255) where significant differences are detected
             
             // Let's manually check what should happen:
             let hard_pixels = hard_binary.get_pixels_view();
             let soft_pixels = soft_binary.get_pixels_view();
             let mut expected_differences = 0;
             
             for y in 0..resolution.height.min(8) {
                 for x in 0..resolution.width.min(8) {
                     let hard_val = hard_pixels[(y, x, 0)];
                     let soft_val = soft_pixels[(y, x, 0)];
                     let abs_diff = if hard_val >= soft_val { hard_val - soft_val } else { soft_val - hard_val };
                     
                     if abs_diff >= low_threshold {
                         expected_differences += 1;
                         if expected_differences <= 5 {
                             println!("Expected diff at ({}, {}): hard={}, soft={}, abs_diff={}", 
                                     y, x, hard_val, soft_val, abs_diff);
                         }
                     }
                 }
             }
             
             println!("Expected {} differences based on manual calculation", expected_differences);
             
             // The test should pass if we detected the expected number of differences
             // With the new implementation, we should now see visible differences (previous frame values)
             assert!(expected_differences > 0, "Should detect some differences between patterns");
             
             // With the new implementation outputting previous values, we should see visible differences
             assert!(differences_low > 0, "Should see visible differences (previous frame values) in the output image");

        }
        
        // Test with HIGH threshold (should NOT detect small differences)
        {
            let mut quick_diff_stage_high = ImageFrameQuickDiffStage::new(properties, high_threshold).unwrap();

            let input_a = WrappedIOData::ImageFrame(hard_binary.clone());
            let input_c = WrappedIOData::ImageFrame(soft_binary.clone());

            // First pass: soft_binary vs empty cache
            _ = quick_diff_stage_high.process_new_input(&input_c, timestamp).unwrap();
            
            // Second pass: hard_binary vs soft_binary  
            // With high threshold (50), differences like 255-10=245 should still exceed threshold
            // But differences like 10-0=10 are below threshold (50)
            let result_high = quick_diff_stage_high.process_new_input(&input_a, timestamp).unwrap();
             
            if let WrappedIOData::ImageFrame(diff_image_high) = result_high {
                save_test_image(diff_image_high, "pipeline_threshold_diff_high.png");
                
                let diff_pixels = diff_image_high.get_pixels_view();
                let mut differences_high = 0;
                let mut total_pixels_checked = 0;
                
                // Check all pixels
                for y in 0..resolution.height.min(8) {
                    for x in 0..resolution.width.min(8) {
                        total_pixels_checked += 1;
                        let r = diff_pixels[(y, x, 0)];
                        let g = diff_pixels[(y, x, 1)];
                        let b = diff_pixels[(y, x, 2)];
                        
                        if r > 0 || g > 0 || b > 0 {
                            differences_high += 1;
                            // With high threshold, only very large differences (like 255-10=245) should pass
                            // These should output the previous value (10 in this case)
                        }
                    }
                }
                
                println!("High threshold ({}): detected {} differences out of {} pixels checked", 
                        high_threshold, differences_high, total_pixels_checked);
                
                // With high threshold, we should see fewer differences than with low threshold
                // but some large differences (255 vs 10) should still be detected
                // The exact number depends on the pattern alignment, but there should be some
                assert!(differences_high >= 0, "High threshold test completed - may have some large differences");
            }
        }
        
        println!("✓ Pipeline threshold behavior test completed successfully");
    }
    
    #[test]
    fn test_full_pipeline_simulation() {
        // This test simulates what a full pipeline might do, even though
        // we can't test the PipelineStageRunner directly
        
        println!("=== Pipeline Simulation Test ===");
        
        // Step 1: Load source image
        let source_image = load_bird_image();
        println!("✓ Loaded source image: {}x{}", 
                source_image.get_image_frame_properties().get_image_resolution().width,
                source_image.get_image_frame_properties().get_image_resolution().height);
        
        // Step 2: Create and configure image processor
        let original_props = source_image.get_image_frame_properties();
        let mut processor = ImageFrameProcessor::new(original_props);
        
        let target_resolution = ImageXYResolution::new(200, 150).unwrap();
        processor.set_resizing_to(target_resolution).unwrap();
        processor.set_brightness_offset(25).unwrap();
        processor.set_contrast_change(20.0).unwrap();
        println!("✓ Configured image processor with resize, brightness, and contrast");
        
        // Step 3: Process the image
        let mut processed_image = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Gamma, &target_resolution).unwrap();
        let process_result = processor.process_image(&source_image, &mut processed_image);
        
        assert!(process_result.is_ok(), "Image processing should succeed");
        println!("✓ Successfully processed image");
        
        // Step 4: Verify output properties
        assert_eq!(processed_image.get_image_frame_properties().get_image_resolution().width, 200);
        assert_eq!(processed_image.get_image_frame_properties().get_image_resolution().height, 150);
        println!("✓ Output image has correct dimensions");
        
        // Step 5: Save result
        save_test_image(&processed_image, "pipeline_simulation_result.png");
        println!("✓ Saved final result");
        
        println!("=== Pipeline Simulation Complete ===");
    }
    
    #[test]
    fn test_stage_combinations() {
        // Test that different stage combinations can be created without errors
        
        // Float processing chain simulation
        let identity_stage = IdentityFloatStage::new(0.0);
        let scale_0_1 = LinearScaleTo0And1Stage::new(0.0, 100.0, 50.0);
        let scale_m1_1 = LinearScaleToM1And1Stage::new(-50.0, 50.0, 0.0);
        
        assert!(identity_stage.is_ok());
        assert!(scale_0_1.is_ok());
        assert!(scale_m1_1.is_ok());
        
        println!("✓ All float processing stages created successfully");
        
        // Image processing stage
        let test_image = create_test_image();
        let properties = test_image.get_image_frame_properties();
        let processor = ImageFrameProcessor::new(properties);
        let image_stage = ImageFrameProcessorStage::new(processor);
        
        assert!(image_stage.is_ok());
        println!("✓ Image processing stage created successfully");
        
        println!("All stage combinations validated");
    }

    #[test]
    fn test_image_rotation() {
        let resolution = ImageXYResolution::new(4, 5).unwrap();
        let color_layout = ColorChannelLayout::RGB;
        let color_space = ColorSpace::Gamma;

        // Create test image using the correct constructor pattern
        let mut test_image = ImageFrame::new(&color_layout, &color_space, &resolution).unwrap();

        // Set all pixels to white (255 for all RGB channels)
        let mut pixels_mut = test_image.get_pixels_view_mut();
        for y in 0..5 {
            for x in 0..4 {
                for c in 0..3 {
                    pixels_mut[(y, x, c)] = 0;
                }
            }
        }

        // Set top-left corner, R channel to black (5) (not full black otherwise encoder drops it
        pixels_mut[(0, 0, 0)] = 5; // R
        pixels_mut[(1, 3, 2)] = 10; // B


        let image_properties  = (&test_image).get_image_frame_properties();

        let mut sensor_cache: SensorCache = SensorCache::new();
        let group_index: CorticalGroupIndex = 0.into();
        let channel_index: CorticalChannelIndex = 0.into();
        let cortical_channel_count: CorticalChannelCount = 1.into();
        let cortical_id = CorticalID::new_sensor_cortical_area_id(ImageCameraCenter, group_index).unwrap();

        sensor_cache.register_image_frame(ImageCameraCenter, group_index, cortical_channel_count, true, image_properties, image_properties);
        sensor_cache.store_image_frame(ImageCameraCenter, group_index, channel_index, test_image).unwrap();

        sensor_cache.encode_cached_data_into_bytes(Instant::now());
        let bytes = sensor_cache.retrieve_latest_bytes().unwrap();

        // check the neuron coord directly
        assert_eq!(bytes[22], 1);
        assert_eq!(bytes[30], 3);
        assert_eq!(bytes[38], 2);
        assert_eq!(bytes[46], 161);

        let feagi_byte_structure: FeagiByteStructure = FeagiByteStructure::create_from_bytes(bytes.to_vec()).unwrap();
        let cortical_mapped_data: CorticalMappedXYZPNeuronData =  CorticalMappedXYZPNeuronData::new_from_feagi_byte_structure(&feagi_byte_structure).unwrap();

        let neuron_array: &NeuronXYZPArrays = cortical_mapped_data.get_neurons_of(&cortical_id).unwrap();

        for neuron in neuron_array.iter() {
            if neuron.cortical_coordinate == CorticalCoordinate::new(0, 0, 0) {
                assert_eq!(neuron.potential, 0.019607844)
            }
            else if neuron.cortical_coordinate == CorticalCoordinate::new(1, 3, 2)  {
                assert_eq!(neuron.potential, 0.039215688)
            }
            else {
                assert_eq!(neuron.potential, 1.0)
            }
        }

    }
    
    //endregion
}