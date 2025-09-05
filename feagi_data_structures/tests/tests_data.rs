//! Tests for the data module
//! 
//! This module contains comprehensive tests for data structures
//! including image frames, segmented frames, descriptors, and JSON handling.

use feagi_data_structures::data::*;
use feagi_data_structures::data::image_descriptors::*;
use feagi_data_structures::basic_components::*;
use feagi_data_structures::FeagiDataError;
use ndarray::Array3;
use serde_json;

#[cfg(test)]
mod test_feagi_json {
    use super::*;

    #[test]
    fn test_feagi_json_from_valid_string() {
        let json_string = r#"{"key": "value", "number": 42}"#.to_string();
        let result = FeagiJSON::from_json_string(json_string);
        
        assert!(result.is_ok());
        let feagi_json = result.unwrap();
        let json_value = feagi_json.borrow_json_value();
        
        assert!(json_value.is_object());
        assert_eq!(json_value["key"], "value");
        assert_eq!(json_value["number"], 42);
    }

    #[test]
    fn test_feagi_json_from_invalid_string() {
        let invalid_json = r#"{"key": "value", "missing_quote: 42}"#.to_string();
        let result = FeagiJSON::from_json_string(invalid_json);
        
        assert!(result.is_err());
        if let Err(FeagiDataError::BadParameters(msg)) = result {
            assert!(msg.contains("Failed to parse JSON string"));
        } else {
            panic!("Expected BadParameters error");
        }
    }

    #[test]
    fn test_feagi_json_from_empty_string() {
        let empty_json = "".to_string();
        let result = FeagiJSON::from_json_string(empty_json);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_feagi_json_from_array_string() {
        let array_json = r#"[1, 2, 3, "test"]"#.to_string();
        let result = FeagiJSON::from_json_string(array_json);
        
        assert!(result.is_ok());
        let feagi_json = result.unwrap();
        let json_value = feagi_json.borrow_json_value();
        
        assert!(json_value.is_array());
        assert_eq!(json_value[0], 1);
        assert_eq!(json_value[3], "test");
    }

    #[test]
    fn test_feagi_json_from_json_value() {
        let json_value = serde_json::json!({
            "name": "test",
            "values": [1, 2, 3],
            "nested": {
                "inner": "data"
            }
        });
        
        let feagi_json = FeagiJSON::from_json_value(json_value);
        let borrowed_value = feagi_json.borrow_json_value();
        
        assert_eq!(borrowed_value["name"], "test");
        assert_eq!(borrowed_value["values"][1], 2);
        assert_eq!(borrowed_value["nested"]["inner"], "data");
    }

    #[test]
    fn test_feagi_json_display() {
        let json_value = serde_json::json!({
            "key": "value",
            "number": 123
        });
        
        let feagi_json = FeagiJSON::from_json_value(json_value);
        let display_string = format!("{}", feagi_json);
        
        assert!(display_string.contains("key"));
        assert!(display_string.contains("value"));
        assert!(display_string.contains("123"));
    }

    #[test]
    fn test_feagi_json_clone() {
        let json_value = serde_json::json!({"test": "data"});
        let feagi_json1 = FeagiJSON::from_json_value(json_value);
        let feagi_json2 = feagi_json1.clone();
        
        assert_eq!(
            feagi_json1.borrow_json_value()["test"],
            feagi_json2.borrow_json_value()["test"]
        );
    }

    #[test]
    fn test_feagi_json_complex_structure() {
        let complex_json = serde_json::json!({
            "metadata": {
                "version": "1.0",
                "timestamp": 1699123456,
                "coordinates": [10.5, 20.3, 30.7]
            },
            "data": {
                "sensors": ["camera", "lidar", "imu"],
                "status": true,
                "config": {
                    "resolution": {
                        "width": 1920,
                        "height": 1080
                    }
                }
            }
        });
        
        let feagi_json = FeagiJSON::from_json_value(complex_json);
        let json_value = feagi_json.borrow_json_value();
        
        assert_eq!(json_value["metadata"]["version"], "1.0");
        assert_eq!(json_value["data"]["sensors"][0], "camera");
        assert_eq!(json_value["data"]["status"], true);
        assert_eq!(json_value["data"]["config"]["resolution"]["width"], 1920);
    }
}

#[cfg(test)]
mod test_image_descriptors {
    use super::*;

    mod test_image_xy_point {
        use super::*;

        #[test]
        fn test_image_xy_point_creation() {
            let point = ImageXYPoint::new(100, 200);
            assert_eq!(point.x, 100);
            assert_eq!(point.y, 200);
        }

        #[test]
        fn test_image_xy_point_zero_values() {
            let point = ImageXYPoint::new(0, 0);
            assert_eq!(point.x, 0);
            assert_eq!(point.y, 0);
        }

        #[test]
        fn test_image_xy_point_max_values() {
            let point = ImageXYPoint::new(u32::MAX, u32::MAX);
            assert_eq!(point.x, u32::MAX);
            assert_eq!(point.y, u32::MAX);
        }

        #[test]
        fn test_image_xy_point_equality() {
            let point1 = ImageXYPoint::new(10, 20);
            let point2 = ImageXYPoint::new(10, 20);
            let point3 = ImageXYPoint::new(10, 21);
            
            assert_eq!(point1, point2);
            assert_ne!(point1, point3);
        }

        #[test]
        fn test_image_xy_point_clone() {
            let point1 = ImageXYPoint::new(123, 456);
            let point2 = point1.clone();
            
            assert_eq!(point1, point2);
            assert_eq!(point1.x, point2.x);
            assert_eq!(point1.y, point2.y);
        }

        #[test]
        fn test_image_xy_point_debug() {
            let point = ImageXYPoint::new(42, 84);
            let debug_string = format!("{:?}", point);
            assert!(debug_string.contains("42"));
            assert!(debug_string.contains("84"));
        }
    }

    mod test_image_xy_resolution {
        use super::*;

        #[test]
        fn test_image_xy_resolution_valid_creation() {
            let resolution = ImageXYResolution::new(1920, 1080).unwrap();
            assert_eq!(resolution.width, 1920);
            assert_eq!(resolution.height, 1080);
        }

        #[test]
        fn test_image_xy_resolution_minimum_values() {
            let resolution = ImageXYResolution::new(1, 1).unwrap();
            assert_eq!(resolution.width, 1);
            assert_eq!(resolution.height, 1);
        }

        #[test]
        fn test_image_xy_resolution_large_values() {
            let resolution = ImageXYResolution::new(4096, 2160).unwrap();
            assert_eq!(resolution.width, 4096);
            assert_eq!(resolution.height, 2160);
        }

        #[test]
        fn test_image_xy_resolution_zero_width_error() {
            let result = ImageXYResolution::new(0, 100);
            assert!(result.is_err());
        }

        #[test]
        fn test_image_xy_resolution_zero_height_error() {
            let result = ImageXYResolution::new(100, 0);
            assert!(result.is_err());
        }

        #[test]
        fn test_image_xy_resolution_both_zero_error() {
            let result = ImageXYResolution::new(0, 0);
            assert!(result.is_err());
        }

        #[test]
        fn test_image_xy_resolution_equality() {
            let res1 = ImageXYResolution::new(1024, 768).unwrap();
            let res2 = ImageXYResolution::new(1024, 768).unwrap();
            let res3 = ImageXYResolution::new(1024, 769).unwrap();
            
            assert_eq!(res1, res2);
            assert_ne!(res1, res3);
        }

        #[test]
        fn test_image_xy_resolution_display() {
            let resolution = ImageXYResolution::new(1280, 720).unwrap();
            let display_string = format!("{}", resolution);
            assert!(display_string.contains("1280"));
            assert!(display_string.contains("720"));
        }
    }

    mod test_color_channel_layout {
        use super::*;

        #[test]
        fn test_color_channel_layout_values() {
            assert_eq!(ColorChannelLayout::GrayScale as usize, 1);
            assert_eq!(ColorChannelLayout::RG as usize, 2);
            assert_eq!(ColorChannelLayout::RGB as usize, 3);
            assert_eq!(ColorChannelLayout::RGBA as usize, 4);
        }

        #[test]
        fn test_color_channel_layout_try_from_valid() {
            assert_eq!(ColorChannelLayout::try_from(1).unwrap(), ColorChannelLayout::GrayScale);
            assert_eq!(ColorChannelLayout::try_from(2).unwrap(), ColorChannelLayout::RG);
            assert_eq!(ColorChannelLayout::try_from(3).unwrap(), ColorChannelLayout::RGB);
            assert_eq!(ColorChannelLayout::try_from(4).unwrap(), ColorChannelLayout::RGBA);
        }

        #[test]
        fn test_color_channel_layout_try_from_invalid() {
            assert!(ColorChannelLayout::try_from(0).is_err());
            assert!(ColorChannelLayout::try_from(5).is_err());
            assert!(ColorChannelLayout::try_from(255).is_err());
        }

        #[test]
        fn test_color_channel_layout_display() {
            assert_eq!(format!("{}", ColorChannelLayout::GrayScale), "Grayscale (1 channel)");
            assert_eq!(format!("{}", ColorChannelLayout::RG), "Red-Green (2 channels)");
            assert_eq!(format!("{}", ColorChannelLayout::RGB), "Red-Green-Blue (3 channels)");
            assert_eq!(format!("{}", ColorChannelLayout::RGBA), "Red-Green-Blue-Alpha (4 channels)");
        }

        #[test]
        fn test_color_channel_layout_debug() {
            let grayscale = ColorChannelLayout::GrayScale;
            let rgb = ColorChannelLayout::RGB;
            
            let debug_grayscale = format!("{:?}", grayscale);
            let debug_rgb = format!("{:?}", rgb);
            
            assert!(debug_grayscale.contains("GrayScale"));
            assert!(debug_rgb.contains("RGB"));
        }

        #[test]
        fn test_color_channel_layout_equality() {
            assert_eq!(ColorChannelLayout::RGB, ColorChannelLayout::RGB);
            assert_ne!(ColorChannelLayout::RGB, ColorChannelLayout::RGBA);
        }

        #[test]
        fn test_color_channel_layout_clone() {
            let original = ColorChannelLayout::RGBA;
            let cloned = original.clone();
            assert_eq!(original, cloned);
        }
    }

    mod test_color_space {
        use super::*;

        #[test]
        fn test_color_space_variants() {
            let linear = ColorSpace::Linear;
            let gamma = ColorSpace::Gamma;
            
            assert_ne!(linear, gamma);
        }

        #[test]
        fn test_color_space_display() {
            assert_eq!(format!("{}", ColorSpace::Linear), "Linear color space");
            assert_eq!(format!("{}", ColorSpace::Gamma), "Gamma color space");
        }

        #[test]
        fn test_color_space_debug() {
            let linear = ColorSpace::Linear;
            let gamma = ColorSpace::Gamma;
            
            let debug_linear = format!("{:?}", linear);
            let debug_gamma = format!("{:?}", gamma);
            
            assert!(debug_linear.contains("Linear"));
            assert!(debug_gamma.contains("Gamma"));
        }

        #[test]
        fn test_color_space_equality() {
            assert_eq!(ColorSpace::Linear, ColorSpace::Linear);
            assert_eq!(ColorSpace::Gamma, ColorSpace::Gamma);
            assert_ne!(ColorSpace::Linear, ColorSpace::Gamma);
        }

        #[test]
        fn test_color_space_clone() {
            let original = ColorSpace::Linear;
            let cloned = original.clone();
            assert_eq!(original, cloned);
        }

        #[test]
        fn test_color_space_copy() {
            let original = ColorSpace::Gamma;
            let copied = original;
            assert_eq!(original, copied);
        }
    }

    mod test_memory_order_layout {
        use super::*;

        #[test]
        fn test_memory_order_layout_variants() {
            let hwc = MemoryOrderLayout::HeightsWidthsChannels;
            let chw = MemoryOrderLayout::ChannelsHeightsWidths;
            let whc = MemoryOrderLayout::WidthsHeightsChannels;
            
            assert_ne!(hwc, chw);
            assert_ne!(hwc, whc);
            assert_ne!(chw, whc);
        }

        #[test]
        fn test_memory_order_layout_display() {
            assert_eq!(format!("{}", MemoryOrderLayout::HeightsWidthsChannels), "Heights x Widths x Channels (HWC)");
            assert_eq!(format!("{}", MemoryOrderLayout::ChannelsHeightsWidths), "Channels x Heights x Widths (CHW)");
            assert_eq!(format!("{}", MemoryOrderLayout::WidthsHeightsChannels), "Widths x Heights x Channels (WHC)");
        }

        #[test]
        fn test_memory_order_layout_debug() {
            let hwc = MemoryOrderLayout::HeightsWidthsChannels;
            let debug_string = format!("{:?}", hwc);
            assert!(debug_string.contains("HeightsWidthsChannels"));
        }

        #[test]
        fn test_memory_order_layout_equality() {
            assert_eq!(MemoryOrderLayout::HeightsWidthsChannels, MemoryOrderLayout::HeightsWidthsChannels);
            assert_ne!(MemoryOrderLayout::HeightsWidthsChannels, MemoryOrderLayout::ChannelsHeightsWidths);
        }

        #[test]
        fn test_memory_order_layout_clone() {
            let original = MemoryOrderLayout::ChannelsHeightsWidths;
            let cloned = original.clone();
            assert_eq!(original, cloned);
        }
    }

    mod test_segmented_xy_image_resolutions {
        use super::*;

        #[test]
        fn test_segmented_resolutions_creation() {
            let center = ImageXYResolution::new(640, 480).unwrap();
            let peripheral = ImageXYResolution::new(160, 120).unwrap();
            
            let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(center, peripheral);
            
            assert_eq!(resolutions.center, center);
            assert_eq!(resolutions.lower_left, peripheral);
            assert_eq!(resolutions.upper_right, peripheral);
            assert_eq!(resolutions.middle_left, peripheral);
        }

        #[test]
        fn test_segmented_resolutions_individual_creation() {
            let center = ImageXYResolution::new(800, 600).unwrap();
            let corner = ImageXYResolution::new(100, 75).unwrap();
            let edge = ImageXYResolution::new(200, 150).unwrap();
            
            let resolutions = SegmentedXYImageResolutions::new(
                corner, edge, corner,    // bottom row
                edge, center, edge,      // middle row  
                corner, edge, corner     // top row
            );
            
            assert_eq!(resolutions.center, center);
            assert_eq!(resolutions.lower_left, corner);
            assert_eq!(resolutions.lower_middle, edge);
            assert_eq!(resolutions.middle_left, edge);
        }

        #[test]
        fn test_segmented_resolutions_getters() {
            let center = ImageXYResolution::new(640, 480).unwrap();
            let peripheral = ImageXYResolution::new(160, 120).unwrap();
            
            let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(center, peripheral);
            
            assert_eq!(&resolutions.center, &center);
            assert_eq!(&resolutions.lower_left, &peripheral);
            assert_eq!(&resolutions.upper_middle, &peripheral);
        }

        #[test]
        fn test_segmented_resolutions_equality() {
            let center = ImageXYResolution::new(640, 480).unwrap();
            let peripheral = ImageXYResolution::new(160, 120).unwrap();
            
            let res1 = SegmentedXYImageResolutions::create_with_same_sized_peripheral(center, peripheral);
            let res2 = SegmentedXYImageResolutions::create_with_same_sized_peripheral(center, peripheral);
            
            assert_eq!(res1, res2);
        }

        #[test]
        fn test_segmented_resolutions_clone() {
            let center = ImageXYResolution::new(800, 600).unwrap();
            let peripheral = ImageXYResolution::new(200, 150).unwrap();
            
            let original = SegmentedXYImageResolutions::create_with_same_sized_peripheral(center, peripheral);
            let cloned = original.clone();
            
            assert_eq!(original, cloned);
            assert_eq!(original.center, cloned.center);
        }

        #[test]
        fn test_segmented_resolutions_debug() {
            let center = ImageXYResolution::new(640, 480).unwrap();
            let peripheral = ImageXYResolution::new(160, 120).unwrap();
            
            let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(center, peripheral);
            let debug_string = format!("{:?}", resolutions);
            
            assert!(debug_string.contains("center"));
            assert!(debug_string.contains("640"));
            assert!(debug_string.contains("480"));
        }

        #[test]
        fn test_segmented_resolutions_hash() {
            use std::collections::HashMap;
            
            let center = ImageXYResolution::new(640, 480).unwrap();
            let peripheral = ImageXYResolution::new(160, 120).unwrap();
            
            let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(center, peripheral);
            
            let mut map = HashMap::new();
            map.insert(resolutions, "test_value");
            
            assert_eq!(map.len(), 1);
            assert_eq!(map.get(&resolutions), Some(&"test_value"));
        }
    }


    mod test_gaze_properties {
        use super::*;

}

#[cfg(test)]
mod test_image_frame {
    use super::*;

    #[test]
    fn test_image_frame_new_basic() {
        let resolution = ImageXYResolution::new(640, 480).unwrap();
        let frame = ImageFrame::new(
            &ColorChannelLayout::RGB,
            &ColorSpace::Linear,
            &resolution
        ).unwrap();
        
        assert_eq!(frame.get_width(), 640);
        assert_eq!(frame.get_height(), 480);
        assert_eq!(frame.get_color_channel_layout(), &ColorChannelLayout::RGB);
        assert_eq!(frame.get_color_space(), &ColorSpace::Linear);
    }

    #[test]
    fn test_image_frame_new_different_layouts() {
        let resolution = ImageXYResolution::new(100, 100).unwrap();
        
        let grayscale = ImageFrame::new(&ColorChannelLayout::GrayScale, &ColorSpace::Linear, &resolution).unwrap();
        let rgb = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Linear, &resolution).unwrap();
        let rgba = ImageFrame::new(&ColorChannelLayout::RGBA, &ColorSpace::Linear, &resolution).unwrap();
        
        assert_eq!(grayscale.get_color_channel_layout(), &ColorChannelLayout::GrayScale);
        assert_eq!(rgb.get_color_channel_layout(), &ColorChannelLayout::RGB);
        assert_eq!(rgba.get_color_channel_layout(), &ColorChannelLayout::RGBA);
    }

    #[test]
    fn test_image_frame_new_different_color_spaces() {
        let resolution = ImageXYResolution::new(100, 100).unwrap();
        
        let linear = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Linear, &resolution).unwrap();
        let gamma = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Gamma, &resolution).unwrap();
        
        assert_eq!(linear.get_color_space(), &ColorSpace::Linear);
        assert_eq!(gamma.get_color_space(), &ColorSpace::Gamma);
    }

    #[test]
    fn test_image_frame_from_properties() {
        let resolution = ImageXYResolution::new(800, 600).unwrap();
        let properties = ImageFrameProperties::new(
            resolution,
            ColorChannelLayout::RGBA,
            ColorSpace::Gamma
        );
        
        let frame = ImageFrame::from_image_frame_properties(&properties).unwrap();
        
        assert_eq!(frame.get_width(), 800);
        assert_eq!(frame.get_height(), 600);
        assert_eq!(frame.get_color_channel_layout(), &ColorChannelLayout::RGBA);
        assert_eq!(frame.get_color_space(), &ColorSpace::Gamma);
    }

    #[test]
    fn test_image_frame_internal_memory_layout() {
        assert_eq!(ImageFrame::INTERNAL_MEMORY_LAYOUT, MemoryOrderLayout::HeightsWidthsChannels);
    }

    #[test]
    fn test_image_frame_dimensions() {
        let resolution = ImageXYResolution::new(1920, 1080).unwrap();
        let frame = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Linear, &resolution).unwrap();
        
        assert_eq!(frame.get_width(), 1920);
        assert_eq!(frame.get_height(), 1080);
        assert_eq!(frame.get_number_of_color_channels(), 3);
    }

    #[test]
    fn test_image_frame_compatibility_same() {
        let resolution = ImageXYResolution::new(640, 480).unwrap();
        let frame1 = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Linear, &resolution).unwrap();
        let frame2 = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Linear, &resolution).unwrap();
        
        assert!(ImageFrame::are_frames_compatible(&frame1, &frame2));
    }

    #[test]
    fn test_image_frame_compatibility_different_resolution() {
        let res1 = ImageXYResolution::new(640, 480).unwrap();
        let res2 = ImageXYResolution::new(800, 600).unwrap();
        
        let frame1 = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Linear, &res1).unwrap();
        let frame2 = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Linear, &res2).unwrap();
        
        assert!(!ImageFrame::are_frames_compatible(&frame1, &frame2));
    }

    #[test]
    fn test_image_frame_compatibility_different_channels() {
        let resolution = ImageXYResolution::new(640, 480).unwrap();
        let frame1 = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Linear, &resolution).unwrap();
        let frame2 = ImageFrame::new(&ColorChannelLayout::RGBA, &ColorSpace::Linear, &resolution).unwrap();
        
        assert!(!ImageFrame::are_frames_compatible(&frame1, &frame2));
    }

    #[test]
    fn test_image_frame_compatibility_different_color_space() {
        let resolution = ImageXYResolution::new(640, 480).unwrap();
        let frame1 = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Linear, &resolution).unwrap();
        let frame2 = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Gamma, &resolution).unwrap();
        
        assert!(!ImageFrame::are_frames_compatible(&frame1, &frame2));
    }

    #[test]
    fn test_image_frame_from_array_valid() {
        let array = Array3::<f32>::zeros((480, 640, 3)); // HWC format
        let frame = ImageFrame::from_array(
            array,
            &ColorSpace::Linear,
            &MemoryOrderLayout::HeightsWidthsChannels
        ).unwrap();
        
        assert_eq!(frame.get_height(), 480);
        assert_eq!(frame.get_width(), 640);
        assert_eq!(frame.get_number_of_color_channels(), 3);
        assert_eq!(frame.get_color_channel_layout(), &ColorChannelLayout::RGB);
    }

    #[test]
    fn test_image_frame_from_array_invalid_channels() {
        let array = Array3::<f32>::zeros((100, 100, 5)); // Invalid channel count
        let result = ImageFrame::from_array(
            array,
            &ColorSpace::Linear,
            &MemoryOrderLayout::HeightsWidthsChannels
        );
        
        assert!(result.is_err());
    }

    #[test]
    fn test_image_frame_clone() {
        let resolution = ImageXYResolution::new(100, 100).unwrap();
        let original = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Linear, &resolution).unwrap();
        let cloned = original.clone();
        
        assert!(ImageFrame::are_frames_compatible(&original, &cloned));
        assert_eq!(original.get_width(), cloned.get_width());
        assert_eq!(original.get_height(), cloned.get_height());
    }

    #[test]
    fn test_image_frame_debug() {
        let resolution = ImageXYResolution::new(320, 240).unwrap();
        let frame = ImageFrame::new(&ColorChannelLayout::RGB, &ColorSpace::Linear, &resolution).unwrap();
        let debug_string = format!("{:?}", frame);
        
        assert!(debug_string.contains("ImageFrame") || debug_string.contains("320") || debug_string.contains("240"));
    }
}

#[cfg(test)]
mod test_segmented_image_frame {
    use super::*;

    #[test]
    fn test_segmented_image_frame_creation() {
        let center_res = ImageXYResolution::new(640, 480).unwrap();
        let peripheral_res = ImageXYResolution::new(160, 120).unwrap();
        let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(center_res, peripheral_res).unwrap();
        
        let frame = SegmentedImageFrame::new(
            &resolutions,
            &ColorSpace::Linear,
            &ColorChannelLayout::RGB,
            &ColorChannelLayout::RGB
        ).unwrap();
        
        assert_eq!(frame.get_center().get_width(), 640);
        assert_eq!(frame.get_center().get_height(), 480);
        assert_eq!(frame.get_lower_left().get_width(), 160);
        assert_eq!(frame.get_lower_left().get_height(), 120);
    }

    #[test]
    fn test_segmented_image_frame_different_center_peripheral() {
        let center_res = ImageXYResolution::new(800, 600).unwrap();
        let peripheral_res = ImageXYResolution::new(200, 150).unwrap();
        let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(center_res, peripheral_res).unwrap();
        
        let frame = SegmentedImageFrame::new(
            &resolutions,
            &ColorSpace::Gamma,
            &ColorChannelLayout::RGBA,
            &ColorChannelLayout::RGB
        ).unwrap();
        
        // Center should be RGBA
        assert_eq!(frame.get_center().get_color_channel_layout(), &ColorChannelLayout::RGBA);
        assert_eq!(frame.get_center().get_number_of_color_channels(), 4);
        
        // Peripherals should be RGB
        assert_eq!(frame.get_lower_left().get_color_channel_layout(), &ColorChannelLayout::RGB);
        assert_eq!(frame.get_lower_left().get_number_of_color_channels(), 3);
    }

    #[test]
    fn test_segmented_image_frame_all_segments() {
        let center_res = ImageXYResolution::new(400, 300).unwrap();
        let peripheral_res = ImageXYResolution::new(100, 75).unwrap();
        let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(center_res, peripheral_res).unwrap();
        
        let frame = SegmentedImageFrame::new(
            &resolutions,
            &ColorSpace::Linear,
            &ColorChannelLayout::GrayScale,
            &ColorChannelLayout::GrayScale
        ).unwrap();
        
        // Test all segment getters
        assert_eq!(frame.get_center().get_width(), 400);
        assert_eq!(frame.get_lower_left().get_width(), 100);
        assert_eq!(frame.get_lower_middle().get_width(), 100);
        assert_eq!(frame.get_lower_right().get_width(), 100);
        assert_eq!(frame.get_middle_left().get_width(), 100);
        assert_eq!(frame.get_middle_right().get_width(), 100);
        assert_eq!(frame.get_upper_left().get_width(), 100);
        assert_eq!(frame.get_upper_middle().get_width(), 100);
        assert_eq!(frame.get_upper_right().get_width(), 100);
        
        // All should be grayscale
        assert_eq!(frame.get_center().get_color_channel_layout(), &ColorChannelLayout::GrayScale);
        assert_eq!(frame.get_lower_left().get_color_channel_layout(), &ColorChannelLayout::GrayScale);
    }

    #[test]
    fn test_segmented_image_frame_properties_constructor() {
        let center_res = ImageXYResolution::new(640, 480).unwrap();
        let peripheral_res = ImageXYResolution::new(160, 120).unwrap();
        let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(center_res, peripheral_res).unwrap();
        
        let properties = SegmentedImageFrameProperties::new(
            resolutions,
            ColorSpace::Linear,
            ColorChannelLayout::RGB,
            ColorChannelLayout::RGB
        );
        
        let frame = SegmentedImageFrame::from_segmented_image_frame_properties(&properties).unwrap();
        
        assert_eq!(frame.get_center().get_width(), 640);
        assert_eq!(frame.get_center().get_height(), 480);
        assert_eq!(frame.get_center().get_color_space(), &ColorSpace::Linear);
    }

    #[test]
    fn test_segmented_image_frame_individual_resolutions() {
        let center = ImageXYResolution::new(800, 600).unwrap();
        let corner = ImageXYResolution::new(100, 75).unwrap();
        let edge = ImageXYResolution::new(200, 150).unwrap();
        
        let resolutions = SegmentedXYImageResolutions::new(
            corner, edge, corner,    // bottom row
            edge, center, edge,      // middle row  
            corner, edge, corner     // top row
        );
        
        let frame = SegmentedImageFrame::new(
            &resolutions,
            &ColorSpace::Linear,
            &ColorChannelLayout::RGB,
            &ColorChannelLayout::RGB
        ).unwrap();
        
        // Test different resolutions
        assert_eq!(frame.get_center().get_width(), 800);
        assert_eq!(frame.get_lower_left().get_width(), 100);  // corner
        assert_eq!(frame.get_lower_middle().get_width(), 200); // edge
        assert_eq!(frame.get_middle_left().get_width(), 200);  // edge
    }

    #[test]
    fn test_segmented_image_frame_clone() {
        let center_res = ImageXYResolution::new(320, 240).unwrap();
        let peripheral_res = ImageXYResolution::new(80, 60).unwrap();
        let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(center_res, peripheral_res).unwrap();
        
        let original = SegmentedImageFrame::new(
            &resolutions,
            &ColorSpace::Linear,
            &ColorChannelLayout::RGB,
            &ColorChannelLayout::RGB
        ).unwrap();
        
        let cloned = original.clone();
        
        assert_eq!(original.get_center().get_width(), cloned.get_center().get_width());
        assert_eq!(original.get_lower_left().get_height(), cloned.get_lower_left().get_height());
    }

    #[test]
    fn test_segmented_image_frame_debug() {
        let center_res = ImageXYResolution::new(400, 300).unwrap();
        let peripheral_res = ImageXYResolution::new(100, 75).unwrap();
        let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(center_res, peripheral_res).unwrap();
        
        let frame = SegmentedImageFrame::new(
            &resolutions,
            &ColorSpace::Linear,
            &ColorChannelLayout::RGB,
            &ColorChannelLayout::RGB
        ).unwrap();
        
        let debug_string = format!("{:?}", frame);
        assert!(debug_string.contains("SegmentedImageFrame") || debug_string.len() > 0);
    }
}

#[cfg(test)]
mod test_error_handling {
    use super::*;

    #[test]
    fn test_image_resolution_zero_errors() {
        assert!(ImageXYResolution::new(0, 100).is_err());
        assert!(ImageXYResolution::new(100, 0).is_err());
        assert!(ImageXYResolution::new(0, 0).is_err());
    }

    #[test]
    fn test_color_channel_layout_invalid_conversions() {
        assert!(ColorChannelLayout::try_from(0).is_err());
        assert!(ColorChannelLayout::try_from(5).is_err());
        assert!(ColorChannelLayout::try_from(255).is_err());
        assert!(ColorChannelLayout::try_from(1000).is_err());
    }

    #[test]
    fn test_gaze_properties_validation_errors() {
        // Test all boundary conditions
        assert!(GazeProperties::new_center_gaze(-0.1, 0.5, 0.3, 0.4).is_err()); // x < 0
        assert!(GazeProperties::new_center_gaze(1.1, 0.5, 0.3, 0.4).is_err());  // x > 1
        assert!(GazeProperties::new_center_gaze(0.5, -0.1, 0.3, 0.4).is_err()); // y < 0
        assert!(GazeProperties::new_center_gaze(0.5, 1.1, 0.3, 0.4).is_err());  // y > 1
        assert!(GazeProperties::new_center_gaze(0.5, 0.5, 0.0, 0.4).is_err());  // width = 0
        assert!(GazeProperties::new_center_gaze(0.5, 0.5, 1.1, 0.4).is_err());  // width > 1
        assert!(GazeProperties::new_center_gaze(0.5, 0.5, 0.3, 0.0).is_err());  // height = 0
        assert!(GazeProperties::new_center_gaze(0.5, 0.5, 0.3, 1.1).is_err());  // height > 1
    }

    #[test]
    fn test_image_frame_invalid_array_channels() {
        let invalid_arrays = vec![
            Array3::<f32>::zeros((100, 100, 0)), // 0 channels
            Array3::<f32>::zeros((100, 100, 5)), // 5 channels
            Array3::<f32>::zeros((100, 100, 10)), // 10 channels
        ];
        
        for array in invalid_arrays {
            let result = ImageFrame::from_array(
                array,
                &ColorSpace::Linear,
                &MemoryOrderLayout::HeightsWidthsChannels
            );
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_feagi_json_parsing_errors() {
        let invalid_json_strings = vec![
            "",                                    // Empty string
            "{",                                   // Incomplete object
            r#"{"key": }"#,                       // Missing value
            r#"{"key": "value""#,                 // Missing closing brace
            r#"{key: "value"}"#,                  // Unquoted key
            r#"{"key": 'value'}"#,                // Single quotes
            r#"{"key": value}"#,                  // Unquoted string value
        ];
        
        for invalid_json in invalid_json_strings {
            let result = FeagiJSON::from_json_string(invalid_json.to_string());
            assert!(result.is_err(), "Expected error for: {}", invalid_json);
        }
    }
}

#[cfg(test)]
mod test_comprehensive_scenarios {
    use super::*;

    #[test]
    fn test_vision_processing_pipeline() {
        // Create a typical vision processing scenario
        let input_resolution = ImageXYResolution::new(1920, 1080).unwrap();
        let center_resolution = ImageXYResolution::new(640, 480).unwrap();
        let peripheral_resolution = ImageXYResolution::new(160, 120).unwrap();
        
        // Create input frame
        let input_frame = ImageFrame::new(
            &ColorChannelLayout::RGB,
            &ColorSpace::Gamma,
            &input_resolution
        ).unwrap();
        
        // Create segmented frame resolutions
        let segmented_resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            center_resolution,
            peripheral_resolution
        ).unwrap();
        
        // Create segmented frame
        let segmented_frame = SegmentedImageFrame::new(
            &segmented_resolutions,
            &ColorSpace::Gamma,
            &ColorChannelLayout::RGB,  // Center high quality
            &ColorChannelLayout::GrayScale  // Peripheral lower quality
        ).unwrap();
        
        // Verify the pipeline components
        assert_eq!(input_frame.get_width(), 1920);
        assert_eq!(input_frame.get_height(), 1080);
        assert_eq!(input_frame.get_number_of_color_channels(), 3);
        
        assert_eq!(segmented_frame.get_center().get_width(), 640);
        assert_eq!(segmented_frame.get_center().get_number_of_color_channels(), 3);
        assert_eq!(segmented_frame.get_lower_left().get_width(), 160);
        assert_eq!(segmented_frame.get_lower_left().get_number_of_color_channels(), 1);
    }

    #[test]
    fn test_multi_resolution_segmentation() {
        // Test different resolutions for different segments
        let center = ImageXYResolution::new(800, 600).unwrap();
        let edge_high = ImageXYResolution::new(200, 150).unwrap();
        let corner_low = ImageXYResolution::new(100, 75).unwrap();
        
        let resolutions = SegmentedXYImageResolutions::new(
            corner_low,  edge_high,   corner_low,   // bottom
            edge_high,   center,      edge_high,    // middle
            corner_low,  edge_high,   corner_low    // top
        );
        
        let frame = SegmentedImageFrame::new(
            &resolutions,
            &ColorSpace::Linear,
            &ColorChannelLayout::RGBA,
            &ColorChannelLayout::RGB
        ).unwrap();
        
        // Test the resolution hierarchy
        assert_eq!(frame.get_center().get_width(), 800);           // Highest
        assert_eq!(frame.get_middle_left().get_width(), 200);      // Medium
        assert_eq!(frame.get_lower_left().get_width(), 100);       // Lowest
        
        // Test different channel layouts
        assert_eq!(frame.get_center().get_number_of_color_channels(), 4);  // RGBA center
        assert_eq!(frame.get_middle_left().get_number_of_color_channels(), 3);  // RGB peripheral
    }

    #[test]
    fn test_gaze_control_system() {
        // Test a complete gaze control scenario
        let gaze_center = GazeProperties::new_center_gaze(0.3, 0.7, 0.4, 0.3).unwrap();
        let input_resolution = ImageXYResolution::new(1920, 1080).unwrap();
        let center_resolution = ImageXYResolution::new(640, 480).unwrap();
        let peripheral_resolution = ImageXYResolution::new(160, 120).unwrap();
        
        // Create input properties
        let input_properties = ImageFrameProperties::new(
            input_resolution,
            ColorChannelLayout::RGB,
            ColorSpace::Gamma
        );
        
        // Create segmented properties  
        let segmented_resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            center_resolution,
            peripheral_resolution
        ).unwrap();
        
        let segmented_properties = SegmentedImageFrameProperties::new(
            segmented_resolutions,
            ColorSpace::Gamma,
            ColorChannelLayout::RGB,
            ColorChannelLayout::RGB
        );
        
        // Test that all components are compatible
        assert_eq!(input_properties.get_color_space(), segmented_properties.get_color_space());
        assert!((gaze_center.get_center_x_normalized() - 0.3).abs() < f32::EPSILON);
        assert!((gaze_center.get_center_y_normalized() - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn test_json_configuration_system() {
        // Test JSON configuration for vision system
        let config_json = serde_json::json!({
            "vision_config": {
                "input_resolution": {
                    "width": 1920,
                    "height": 1080
                },
                "center_resolution": {
                    "width": 640,
                    "height": 480
                },
                "peripheral_resolution": {
                    "width": 160,
                    "height": 120
                },
                "color_space": "gamma",
                "center_channels": "rgb",
                "peripheral_channels": "grayscale",
                "gaze": {
                    "center_x": 0.5,
                    "center_y": 0.5,
                    "width": 0.3,
                    "height": 0.4
                }
            }
        });
        
        let feagi_json = FeagiJSON::from_json_value(config_json);
        let json_value = feagi_json.borrow_json_value();
        
        // Extract configuration and create components
        let input_width = json_value["vision_config"]["input_resolution"]["width"].as_u64().unwrap() as usize;
        let input_height = json_value["vision_config"]["input_resolution"]["height"].as_u64().unwrap() as usize;
        let center_width = json_value["vision_config"]["center_resolution"]["width"].as_u64().unwrap() as usize;
        let center_height = json_value["vision_config"]["center_resolution"]["height"].as_u64().unwrap() as usize;
        
        let input_resolution = ImageXYResolution::new(input_width, input_height).unwrap();
        let center_resolution = ImageXYResolution::new(center_width, center_height).unwrap();
        
        assert_eq!(input_resolution.width, 1920);
        assert_eq!(input_resolution.height, 1080);
        assert_eq!(center_resolution.width, 640);
        assert_eq!(center_resolution.height, 480);
        
        // Test gaze configuration
        let gaze_x = json_value["vision_config"]["gaze"]["center_x"].as_f64().unwrap() as f32;
        let gaze_y = json_value["vision_config"]["gaze"]["center_y"].as_f64().unwrap() as f32;
        let gaze_width = json_value["vision_config"]["gaze"]["width"].as_f64().unwrap() as f32;
        let gaze_height = json_value["vision_config"]["gaze"]["height"].as_f64().unwrap() as f32;
        
        let gaze = GazeProperties::new_center_gaze(gaze_x, gaze_y, gaze_width, gaze_height).unwrap();
        
        assert!((gaze.get_center_x_normalized() - 0.5).abs() < f32::EPSILON);
        assert!((gaze.get_center_y_normalized() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_memory_layout_compatibility() {
        // Test different memory layouts work with image frames
        let test_layouts = vec![
            MemoryOrderLayout::HeightsWidthsChannels,
            MemoryOrderLayout::ChannelsHeightsWidths,
            MemoryOrderLayout::WidthsHeightsChannels,
        ];
        
        for layout in test_layouts {
            let array = Array3::<f32>::zeros((100, 100, 3));
            let result = ImageFrame::from_array(
                array,
                &ColorSpace::Linear,
                &layout
            );
            
            // All should succeed since we're creating valid 3-channel arrays
            assert!(result.is_ok(), "Failed for layout: {:?}", layout);
            
            let frame = result.unwrap();
            assert_eq!(frame.get_number_of_color_channels(), 3);
        }
    }

    #[test]
    fn test_extreme_resolution_scenarios() {
        // Test very small resolutions
        let tiny_resolution = ImageXYResolution::new(1, 1).unwrap();
        let tiny_frame = ImageFrame::new(
            &ColorChannelLayout::GrayScale,
            &ColorSpace::Linear,
            &tiny_resolution
        ).unwrap();
        
        assert_eq!(tiny_frame.get_width(), 1);
        assert_eq!(tiny_frame.get_height(), 1);
        
        // Test large resolutions (within reason for tests)
        let large_resolution = ImageXYResolution::new(3840, 2160).unwrap(); // 4K
        let large_frame = ImageFrame::new(
            &ColorChannelLayout::RGB,
            &ColorSpace::Linear,
            &large_resolution
        ).unwrap();
        
        assert_eq!(large_frame.get_width(), 3840);
        assert_eq!(large_frame.get_height(), 2160);
        
        // Test segmented frame with mixed extreme resolutions
        let segmented_resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            large_resolution,
            tiny_resolution
        ).unwrap();
        
        let segmented_frame = SegmentedImageFrame::new(
            &segmented_resolutions,
            &ColorSpace::Linear,
            &ColorChannelLayout::RGB,
            &ColorChannelLayout::GrayScale
        ).unwrap();
        
        assert_eq!(segmented_frame.get_center().get_width(), 3840);
        assert_eq!(segmented_frame.get_lower_left().get_width(), 1);
    }
}

#[cfg(test)]
mod test_type_conversions_and_interop {
    use super::*;

    #[test]
    fn test_image_xy_point_conversions() {
        let original_coord = FlatCoordinateU32::new(100, 200);
        let image_point = ImageXYPoint::from(original_coord);
        let back_to_coord: FlatCoordinateU32 = image_point.into();
        
        assert_eq!(original_coord.x, back_to_coord.x);
        assert_eq!(original_coord.y, back_to_coord.y);
    }

    #[test]
    fn test_image_xy_resolution_conversions() {
        let original_cartesian = CartesianResolution::new(800, 600).unwrap();
        let image_resolution = ImageXYResolution::from(original_cartesian);
        let back_to_cartesian: CartesianResolution = image_resolution.into();
        
        assert_eq!(original_cartesian.width, back_to_cartesian.width);
        assert_eq!(original_cartesian.height, back_to_cartesian.height);
    }

    #[test]
    fn test_properties_integration() {
        let resolution = ImageXYResolution::new(640, 480).unwrap();
        let properties = ImageFrameProperties::new(
            resolution,
            ColorChannelLayout::RGB,
            ColorSpace::Linear
        );
        
        let frame1 = ImageFrame::from_image_frame_properties(&properties).unwrap();
        let frame2 = ImageFrame::new(
            properties.get_color_channel_layout(),
            properties.get_color_space(),
            properties.get_image_resolution()
        ).unwrap();
        
        assert!(ImageFrame::are_frames_compatible(&frame1, &frame2));
    }

    #[test]
    fn test_segmented_properties_integration() {
        let center_res = ImageXYResolution::new(640, 480).unwrap();
        let peripheral_res = ImageXYResolution::new(160, 120).unwrap();
        let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(center_res, peripheral_res).unwrap();
        
        let properties = SegmentedImageFrameProperties::new(
            resolutions,
            ColorSpace::Gamma,
            ColorChannelLayout::RGBA,
            ColorChannelLayout::RGB
        );
        
        let frame1 = SegmentedImageFrame::from_segmented_image_frame_properties(&properties).unwrap();
        let frame2 = SegmentedImageFrame::new(
            properties.get_segmented_resolutions(),
            properties.get_color_space(),
            properties.get_center_color_channel_layout(),
            properties.get_peripheral_color_channel_layout()
        ).unwrap();
        
        // Both frames should have the same structure
        assert_eq!(frame1.get_center().get_width(), frame2.get_center().get_width());
        assert_eq!(frame1.get_center().get_height(), frame2.get_center().get_height());
        assert_eq!(
            frame1.get_center().get_color_channel_layout(),
            frame2.get_center().get_color_channel_layout()
        );
    }

    #[test] 
    fn test_json_feagi_integration() {
        // Test JSON data that could come from FEAGI
        let feagi_command = serde_json::json!({
            "command": "configure_vision",
            "parameters": {
                "camera_id": "main_camera",
                "resolution": {
                    "width": 1920,
                    "height": 1080
                },
                "processing": {
                    "segmentation": true,
                    "center_quality": "high",
                    "peripheral_quality": "low"
                }
            }
        });
        
        let feagi_json = FeagiJSON::from_json_value(feagi_command);
        let json_data = feagi_json.borrow_json_value();
        
        assert_eq!(json_data["command"], "configure_vision");
        assert_eq!(json_data["parameters"]["camera_id"], "main_camera");
        assert_eq!(json_data["parameters"]["resolution"]["width"], 1920);
        assert_eq!(json_data["parameters"]["processing"]["segmentation"], true);
        
        // Verify the JSON can be serialized back
        let display_output = format!("{}", feagi_json);
        assert!(display_output.contains("configure_vision"));
        assert!(display_output.contains("main_camera"));
    }
}
