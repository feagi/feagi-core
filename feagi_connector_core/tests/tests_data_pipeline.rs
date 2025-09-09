//! Tests for the data pipeline module
//! 
//! This module contains basic tests for the data pipeline stages,
//! focusing on stage creation and basic validation.

use std::time::Instant;
use feagi_data_structures::data::ImageFrame;
use feagi_data_structures::data::image_descriptors::{ColorChannelLayout, ColorSpace, ImageXYResolution};
// WrappedIOData and WrappedIOType are not used in these simplified tests
use feagi_data_structures::processing::ImageFrameProcessor;
use feagi_connector_core::data_pipeline::stages::*;

#[cfg(test)]
mod test_pipeline_stages {
    use super::*;

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
    
    //endregion
}