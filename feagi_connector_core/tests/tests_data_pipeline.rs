//! Tests for the data pipeline module
//! 
//! This module contains basic tests for the data pipeline stages,
//! focusing on stage creation and basic validation.

use std::time::Instant;
use feagi_data_structures::data::{ImageFrame, SegmentedImageFrame};
use feagi_data_structures::data::descriptors::{ColorChannelLayout, ColorSpace, ImageXYResolution};
use feagi_data_structures::processing::ImageFrameProcessor;
use feagi_data_structures::wrapped_io_data::WrappedIOData;
use feagi_connector_core::data_pipeline::stages::*;

#[cfg(test)]
mod test_pipeline_stages {
    use feagi_connector_core::caching::{IOCache};
    use super::*;
    
    // Import trait for direct stage testing (traits can be imported even if the module is private)
    use feagi_connector_core::data_pipeline::stages::*;
    use feagi_connector_core::data_pipeline::PipelineStage;
    use feagi_data_serialization::{FeagiByteStructure, FeagiByteStructureCompatible};
    use feagi_data_structures::data::{Percentage, Percentage2D, SignedPercentage};
    use feagi_data_structures::data::descriptors::{GazeProperties, SegmentedImageFrameProperties, SegmentedXYImageResolutions};
    use feagi_data_structures::genomic::{CorticalID, CorticalType, MotorCorticalType, SensorCorticalType};
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
    
    fn save_test_segmented_images(segmented_frame: &SegmentedImageFrame, filename_prefix: &str) {
        let image_refs = segmented_frame.get_ordered_image_frame_references();
        let segment_names = [
            "lower_left", "lower_middle", "lower_right",
            "middle_left", "center", "middle_right", 
            "upper_left", "upper_middle", "upper_right"
        ];
        
        for (index, image) in image_refs.iter().enumerate() {
            let filename = format!("{}_{}.png", filename_prefix, segment_names[index]);
            save_test_image(image, &filename);
        }
        println!("Saved all 9 segments for: {}", filename_prefix);
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
        let scale_0_1 = LinearScaleToPercentageStage::new(0.0, 100.0, Percentage::new_from_0_100(50.0).unwrap());
        let scale_m1_1 = LinearScaleToSignedPercentageStage::new(-50.0, 50.0, SignedPercentage::new_from_m1_1_unchecked(0.0));
        
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

        let mut sensor_cache: IOCache = IOCache::new();
        let group_index: CorticalGroupIndex = 0.into();
        let channel_index: CorticalChannelIndex = 0.into();
        let cortical_channel_count: CorticalChannelCount = 1.into();
        let cortical_id = CorticalID::new_sensor_cortical_area_id(ImageCameraCenter, group_index).unwrap();

        sensor_cache.register_image_frame_sensor(ImageCameraCenter, group_index, cortical_channel_count, image_properties, image_properties);
        sensor_cache.store_image_frame_sensor(ImageCameraCenter, group_index, channel_index, test_image).unwrap();

        sensor_cache.sensor_encode_cached_data_into_bytes(Instant::now());
        let bytes = sensor_cache.sensor_retrieve_latest_bytes().unwrap();

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

    #[test]
    fn test_image_quickdiff() {
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

        let mut sensor_cache: IOCache = IOCache::new();
        let group_index: CorticalGroupIndex = 0.into();
        let channel_index: CorticalChannelIndex = 0.into();
        let cortical_channel_count: CorticalChannelCount = 1.into();
        let cortical_id = CorticalID::new_sensor_cortical_area_id(ImageCameraCenter, group_index).unwrap();

        sensor_cache.register_image_frame_sensor(ImageCameraCenter, group_index, cortical_channel_count, image_properties, image_properties);
        sensor_cache.store_image_frame_sensor(ImageCameraCenter, group_index, channel_index, test_image).unwrap();

        sensor_cache.sensor_encode_cached_data_into_bytes(Instant::now());
        let bytes = sensor_cache.sensor_retrieve_latest_bytes().unwrap();

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



    #[test]
    fn test_image_segmentation_bird_with_eccentricity_changes() {
        let image = load_bird_image();

        let image_properties  = image.get_image_frame_properties();
        let segmented_properties = SegmentedImageFrameProperties::new(
            &SegmentedXYImageResolutions::create_with_same_sized_peripheral(
                ImageXYResolution::new(80, 80).unwrap(),
                ImageXYResolution::new(40, 40).unwrap(),
            ),
            &image_properties.get_color_channel_layout(),
            &image_properties.get_color_channel_layout(),
            &image_properties.get_color_space()
        );


        let group_index: CorticalGroupIndex = 0.into();
        let channel_index: CorticalChannelIndex = 0.into();
        let channel_count: CorticalChannelCount = 1.into();
        let gaze = GazeProperties::new(
            Percentage2D::new(Percentage::new_from_0_1_unchecked(0.25), Percentage::new_from_0_1_unchecked(0.25)),
            Percentage2D::new(Percentage::new_from_0_1_unchecked(0.25), Percentage::new_from_0_1_unchecked(0.25))
        );

        let mut io_cache: IOCache = IOCache::new();

        io_cache.register_segmented_image_frame_sensor(group_index, channel_count, image_properties, segmented_properties, gaze).unwrap();
        io_cache.register_percentage_4d_data_motor(MotorCorticalType::Gaze, group_index, channel_count, 4).unwrap();
        // TODO for now registration between the 2 is automatic

        io_cache.store_segmented_image_frame_sensor(image, group_index, channel_index).unwrap();
        io_cache.sensor_encode_cached_data_into_bytes(Instant::now());

        save_test_segmented_images(&io_cache.read_cache_segmented_image_frame_sensor(group_index, channel_index).unwrap(), "birb_segmentation_a");

        let sensor_bytes_initial = io_cache.sensor_retrieve_latest_bytes().unwrap();

        let mut neurons_for_gaze = CorticalMappedXYZPNeuronData::new();
        let mut neuron_data = NeuronXYZPArrays::new(); // Lets pretend this has a length of 4
        //neuron_data.push_raw(0,0,0,1.0);
        neuron_data.push_raw(1, 0, 0, 1.0);
        neuron_data.push_raw(2, 0, 1, 1.0);
        neuron_data.push_raw(3, 0, 0, 1.0);
        neuron_data.push_raw(3, 0, 1, 1.0);
        neuron_data.push_raw(3, 0, 2, 1.0);
        neuron_data.push_raw(3, 0, 3, 1.0);

        neurons_for_gaze.insert(MotorCorticalType::Gaze.to_cortical_id(group_index), neuron_data);
        let motor_bytes_struct = neurons_for_gaze.as_new_feagi_byte_structure().unwrap();
        let mut cache_motor_fbs = io_cache.get_motor_byte_structure_mut();
        *cache_motor_fbs = motor_bytes_struct;
        io_cache.process_motor_byte_structure_data().unwrap();




    }

    
    //endregion
}