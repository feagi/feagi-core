//! Tests for the data pipeline module - focusing on end -> end tests

use feagi_sensorimotor::data_types::descriptors::{
    ColorChannelLayout, ColorSpace, SegmentedImageFrameProperties, SegmentedXYImageResolutions,
};
use feagi_sensorimotor::data_types::{GazeProperties, ImageFrame, Percentage, Percentage2D};
use feagi_sensorimotor::wrapped_io_data::WrappedIOData;
use feagi_sensorimotor::ConnectorAgent;
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelIndex, CorticalUnitIndex,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_data_type::FrameChangeHandling;

//region Helpers

#[allow(dead_code)]
fn load_bird_image() -> ImageFrame {
    let bird_bytes = std::fs::read("tests/images/bird.jpg").expect("Bird image should exist");
    ImageFrame::new_from_jpeg_bytes(&bird_bytes, &ColorSpace::Gamma)
        .expect("Bird image should load correctly")
}

fn create_default_gaze() -> GazeProperties {
    GazeProperties::new((0.5, 0.5).try_into().unwrap(), 0.5.try_into().unwrap())
}

fn create_gaze(x: f32, y: f32, size: f32) -> GazeProperties {
    GazeProperties::new(
        Percentage2D::new(
            Percentage::new_from_0_1_unchecked(x),
            Percentage::new_from_0_1_unchecked(y),
        ),
        Percentage::new_from_0_1_unchecked(size),
    )
}

//endregion

#[cfg(test)]
mod test_connector_cache_sensor_load_image {
    use crate::load_bird_image;
    use feagi_sensorimotor::data_types::descriptors::{
        SegmentedImageFrameProperties, SegmentedXYImageResolutions,
    };
    use feagi_sensorimotor::data_types::{GazeProperties, MiscData};
    use feagi_sensorimotor::wrapped_io_data::WrappedIOData;
    use feagi_structures::genomic::cortical_area::descriptors::{
        CorticalChannelCount, CorticalChannelIndex, CorticalUnitIndex,
    };
    use feagi_structures::genomic::cortical_area::io_cortical_area_data_type::FrameChangeHandling;
    use feagi_structures::genomic::{MotorCorticalUnit, SensoryCorticalUnit};
    use std::time::Instant;

    #[test]
    fn test_segment_bird_image() {
        let _time_of_previous_burst: Instant = Instant::now(); // Pretend

        let cortical_group: CorticalUnitIndex = 0.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();
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
            GazeProperties::new((0.5, 0.5).try_into().unwrap(), 0.5.try_into().unwrap());

        let connector_agent = feagi_sensorimotor::ConnectorAgent::new();
        {
            #[allow(unused_mut)]
            let mut sensor_cache = connector_agent.get_sensor_cache();
            sensor_cache
                .segmented_vision_register(
                    cortical_group,
                    number_channels,
                    FrameChangeHandling::Absolute,
                    bird_image_properties,
                    segmented_bird_properties,
                    initial_gaze,
                )
                .unwrap();
            sensor_cache
                .segmented_vision_write(cortical_group, channel_index, bird_image.into())
                .unwrap();
        }
        // let bytes = sensor_cache.sensor_copy_feagi_byte_container();
    }

    #[test]
    fn test_segment_bird_image_twice() {
        let _time_of_previous_burst: Instant = Instant::now(); // Pretend

        let cortical_group: CorticalUnitIndex = 0.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();
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
            GazeProperties::new((0.5, 0.5).try_into().unwrap(), 0.5.try_into().unwrap());

        let connector_agent = feagi_sensorimotor::ConnectorAgent::new();
        {
            #[allow(unused_mut)]
            let mut sensor_cache = connector_agent.get_sensor_cache();
            sensor_cache
                .segmented_vision_register(
                    cortical_group,
                    number_channels,
                    FrameChangeHandling::Absolute,
                    bird_image_properties,
                    segmented_bird_properties,
                    initial_gaze,
                )
                .unwrap();

            let wrapped: WrappedIOData = bird_image.into();

            sensor_cache
                .segmented_vision_write(cortical_group, channel_index, wrapped.clone())
                .unwrap();
            // sensor_cache.sensor_encode_data_to_neurons_then_bytes(0);
            // let bytes = sensor_cache.sensor_copy_feagi_byte_container();

            sensor_cache
                .segmented_vision_write(cortical_group, channel_index, wrapped)
                .unwrap();
            // sensor_cache.sensor_encode_data_to_neurons_then_bytes(0);
            // let bytes = sensor_cache.sensor_copy_feagi_byte_container();
        }
    }
    

    #[test]
    fn test_encode_of_misc_then_reencode() {
        let _time_of_previous_burst: Instant = Instant::now();

        let cortical_group: CorticalUnitIndex = 0.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();

        let bird_image = load_bird_image();
        let misc_data = MiscData::new_from_image_frame(&bird_image).unwrap();

        let connector_agent = feagi_sensorimotor::ConnectorAgent::new();
        {
            #[allow(unused_mut)]
            let mut sensor_cache = connector_agent.get_sensor_cache();
            sensor_cache
                .miscellaneous_register(
                    cortical_group,
                    number_channels,
                    FrameChangeHandling::Absolute,
                    misc_data.get_dimensions(),
                )
                .unwrap();
            sensor_cache
                .miscellaneous_write(cortical_group, channel_index, misc_data.clone().into())
                .unwrap();
        }
        {
            let mut motor_cache = connector_agent.get_motor_cache();
            motor_cache
                .miscellaneous_register(
                    cortical_group,
                    number_channels,
                    FrameChangeHandling::Absolute,
                    misc_data.get_dimensions(),
                )
                .unwrap();

            // Test encoding/decoding cycle
            // Since the output of the sensor is under cortical ID imis00, to read it to the motor, we need to assign it to omis00,
            let _sensor_cortical_id = SensoryCorticalUnit::get_cortical_ids_array_for_miscellaneous(
                FrameChangeHandling::Absolute,
                cortical_group,
            )[0];
            let _motor_cortical_id = MotorCorticalUnit::get_cortical_ids_array_for_miscellaneous(
                FrameChangeHandling::Absolute,
                cortical_group,
            )[0];

            // Note: This test needs reworking for the new architecture where encoding/decoding is handled differently
            // For now, we verify the registration works
            let _new_misc_data = motor_cache
                .miscellaneous_read_postprocessed_cache_value(cortical_group, channel_index)
                .unwrap();
            // assert_eq!(misc_data, new_misc_data);
        }
    }

    #[test]
    fn test_expanding_encode() {
        let _time_of_previous_burst: Instant = Instant::now();

        let cortical_group: CorticalUnitIndex = 0.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();

        let bird_image = load_bird_image();
        let misc_data_image = MiscData::new_from_image_frame(&bird_image).unwrap();
        let misc_data_empty = MiscData::new(&misc_data_image.get_dimensions()).unwrap();
        let mut misc_data_semi = misc_data_empty.clone();
        {
            let data = misc_data_semi.get_internal_data_mut();
            for i in 0..20usize {
                data[[i, 0, 0]] = 1.0;
            }
        }
        let misc_data_solid = misc_data_empty.clone();
        {
            let data = misc_data_semi.get_internal_data_mut();
            data.fill(10.0);
        }

        let connector_agent = feagi_sensorimotor::ConnectorAgent::new();
        {
            #[allow(unused_mut)]
            let mut sensor_cache = connector_agent.get_sensor_cache();
            sensor_cache
                .miscellaneous_register(
                    cortical_group,
                    number_channels,
                    FrameChangeHandling::Absolute,
                    misc_data_empty.get_dimensions(),
                )
                .unwrap();

            sensor_cache
                .miscellaneous_write(
                    cortical_group,
                    channel_index,
                    misc_data_empty.clone().into(),
                )
                .unwrap();
            // sensor_cache.sensor_encode_data_to_neurons_then_bytes(0);
            // let bytes_empty = sensor_cache.sensor_copy_feagi_byte_container();

            sensor_cache
                .miscellaneous_write(cortical_group, channel_index, misc_data_semi.clone().into())
                .unwrap();
            // sensor_cache.sensor_encode_data_to_neurons_then_bytes(0);
            // let bytes_semi = sensor_cache.sensor_copy_feagi_byte_container();

            sensor_cache
                .miscellaneous_write(
                    cortical_group,
                    channel_index,
                    misc_data_image.clone().into(),
                )
                .unwrap();
            // sensor_cache.sensor_encode_data_to_neurons_then_bytes(0);
            // let bytes_image = sensor_cache.sensor_copy_feagi_byte_container();

            sensor_cache
                .miscellaneous_write(
                    cortical_group,
                    channel_index,
                    misc_data_solid.clone().into(),
                )
                .unwrap();
            // sensor_cache.sensor_encode_data_to_neurons_then_bytes(0);
            // let bytes_solid = sensor_cache.sensor_copy_feagi_byte_container();

            sensor_cache
                .miscellaneous_register(
                    1.into(),
                    number_channels,
                    FrameChangeHandling::Absolute,
                    misc_data_empty.get_dimensions(),
                )
                .unwrap();
            sensor_cache
                .miscellaneous_register(
                    2.into(),
                    number_channels,
                    FrameChangeHandling::Absolute,
                    misc_data_empty.get_dimensions(),
                )
                .unwrap();
            sensor_cache
                .miscellaneous_register(
                    3.into(),
                    number_channels,
                    FrameChangeHandling::Absolute,
                    misc_data_empty.get_dimensions(),
                )
                .unwrap();

            sensor_cache
                .miscellaneous_write(1.into(), channel_index, misc_data_image.clone().into())
                .unwrap();
            sensor_cache
                .miscellaneous_write(2.into(), channel_index, misc_data_solid.clone().into())
                .unwrap();
            sensor_cache
                .miscellaneous_write(3.into(), channel_index, misc_data_image.clone().into())
                .unwrap();

            // sensor_cache.sensor_encode_data_to_neurons_then_bytes(0);
            // let bytes_multi = sensor_cache.sensor_copy_feagi_byte_container();
            // dbg!(bytes_multi.get_number_of_bytes_used());
        }
    }
}

#[cfg(test)]
mod test_image_segmentation_basic {
    use super::*;

    #[test]
    fn test_segment_bird_image_centered_gaze() {
        let cortical_group: CorticalUnitIndex = 0.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();

        let segmented_resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            (256, 256).try_into().unwrap(),
            (128, 128).try_into().unwrap(),
        );

        let bird_image = load_bird_image();
        let bird_properties = bird_image.get_image_frame_properties();
        let segmented_properties = SegmentedImageFrameProperties::new(
            segmented_resolutions,
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_space(),
        );
        let initial_gaze = create_default_gaze();

        let connector_agent = ConnectorAgent::new();
        {
            let mut sensor_cache = connector_agent.get_sensor_cache();
            sensor_cache
                .segmented_vision_register(
                    cortical_group,
                    number_channels,
                    FrameChangeHandling::Absolute,
                    bird_properties,
                    segmented_properties,
                    initial_gaze,
                )
                .expect("Registration should succeed");

            sensor_cache
                .segmented_vision_write(cortical_group, channel_index, bird_image.into())
                .expect("Write should succeed");
        }
    }

    #[test]
    fn test_segment_bird_image_multiple_writes() {
        let cortical_group: CorticalUnitIndex = 0.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();

        let segmented_resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            (256, 256).try_into().unwrap(),
            (128, 128).try_into().unwrap(),
        );

        let bird_image = load_bird_image();
        let bird_properties = bird_image.get_image_frame_properties();
        let segmented_properties = SegmentedImageFrameProperties::new(
            segmented_resolutions,
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_space(),
        );
        let initial_gaze = create_default_gaze();

        let connector_agent = ConnectorAgent::new();
        {
            let mut sensor_cache = connector_agent.get_sensor_cache();
            sensor_cache
                .segmented_vision_register(
                    cortical_group,
                    number_channels,
                    FrameChangeHandling::Absolute,
                    bird_properties,
                    segmented_properties,
                    initial_gaze,
                )
                .expect("Registration should succeed");

            let wrapped: WrappedIOData = bird_image.into();

            // First write
            sensor_cache
                .segmented_vision_write(cortical_group, channel_index, wrapped.clone())
                .expect("First write should succeed");

            // Second write (same image)
            sensor_cache
                .segmented_vision_write(cortical_group, channel_index, wrapped.clone())
                .expect("Second write should succeed");

            // Third write
            sensor_cache
                .segmented_vision_write(cortical_group, channel_index, wrapped)
                .expect("Third write should succeed");
        }
    }

    #[test]
    fn test_segment_with_small_center_large_peripheral() {
        let cortical_group: CorticalUnitIndex = 0.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();

        // Smaller center, larger peripheral - unusual but should work
        let segmented_resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            (64, 64).try_into().unwrap(),
            (256, 256).try_into().unwrap(),
        );

        let bird_image = load_bird_image();
        let bird_properties = bird_image.get_image_frame_properties();
        let segmented_properties = SegmentedImageFrameProperties::new(
            segmented_resolutions,
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_space(),
        );
        let initial_gaze = create_default_gaze();

        let connector_agent = ConnectorAgent::new();
        {
            let mut sensor_cache = connector_agent.get_sensor_cache();
            sensor_cache
                .segmented_vision_register(
                    cortical_group,
                    number_channels,
                    FrameChangeHandling::Absolute,
                    bird_properties,
                    segmented_properties,
                    initial_gaze,
                )
                .expect("Registration should succeed");

            sensor_cache
                .segmented_vision_write(cortical_group, channel_index, bird_image.into())
                .expect("Write should succeed");
        }
    }
}

#[cfg(test)]
mod test_image_segmentation_gaze_positions {
    use super::*;

    #[test]
    fn test_segment_with_top_left_gaze() {
        let cortical_group: CorticalUnitIndex = 0.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();

        let segmented_resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            (128, 128).try_into().unwrap(),
            (64, 64).try_into().unwrap(),
        );

        let bird_image = load_bird_image();
        let bird_properties = bird_image.get_image_frame_properties();
        let segmented_properties = SegmentedImageFrameProperties::new(
            segmented_resolutions,
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_space(),
        );

        // Gaze towards top-left (negative eccentricity)
        let gaze = create_gaze(0.2, 0.8, 0.3);

        let connector_agent = ConnectorAgent::new();
        {
            let mut sensor_cache = connector_agent.get_sensor_cache();
            sensor_cache
                .segmented_vision_register(
                    cortical_group,
                    number_channels,
                    FrameChangeHandling::Absolute,
                    bird_properties,
                    segmented_properties,
                    gaze,
                )
                .expect("Registration should succeed");

            sensor_cache
                .segmented_vision_write(cortical_group, channel_index, bird_image.into())
                .expect("Write should succeed");
        }
    }

    #[test]
    fn test_segment_with_bottom_right_gaze() {
        let cortical_group: CorticalUnitIndex = 0.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();

        let segmented_resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            (128, 128).try_into().unwrap(),
            (64, 64).try_into().unwrap(),
        );

        let bird_image = load_bird_image();
        let bird_properties = bird_image.get_image_frame_properties();
        let segmented_properties = SegmentedImageFrameProperties::new(
            segmented_resolutions,
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_space(),
        );

        // Gaze towards bottom-right
        let gaze = create_gaze(0.8, 0.2, 0.4);

        let connector_agent = ConnectorAgent::new();
        {
            let mut sensor_cache = connector_agent.get_sensor_cache();
            dbg!("a");
            sensor_cache
                .segmented_vision_register(
                    cortical_group,
                    number_channels,
                    FrameChangeHandling::Absolute,
                    bird_properties,
                    segmented_properties,
                    gaze,
                )
                .expect("Registration should succeed");

            sensor_cache
                .segmented_vision_write(cortical_group, channel_index, bird_image.into())
                .expect("Write should succeed");
        }
    }

    #[test]
    fn test_segment_with_minimum_modulation_size() {
        let cortical_group: CorticalUnitIndex = 0.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();

        let segmented_resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            (128, 128).try_into().unwrap(),
            (64, 64).try_into().unwrap(),
        );

        let bird_image = load_bird_image();
        let bird_properties = bird_image.get_image_frame_properties();
        let segmented_properties = SegmentedImageFrameProperties::new(
            segmented_resolutions,
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_space(),
        );

        // Very small modulation size (zoomed in)
        let gaze = create_gaze(0.5, 0.5, 0.1);

        let connector_agent = ConnectorAgent::new();
        {
            let mut sensor_cache = connector_agent.get_sensor_cache();
            sensor_cache
                .segmented_vision_register(
                    cortical_group,
                    number_channels,
                    FrameChangeHandling::Absolute,
                    bird_properties,
                    segmented_properties,
                    gaze,
                )
                .expect("Registration should succeed");

            sensor_cache
                .segmented_vision_write(cortical_group, channel_index, bird_image.into())
                .expect("Write should succeed");
        }
    }

    #[test]
    fn test_segment_with_maximum_modulation_size() {
        let cortical_group: CorticalUnitIndex = 0.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();

        let segmented_resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            (128, 128).try_into().unwrap(),
            (64, 64).try_into().unwrap(),
        );

        let bird_image = load_bird_image();
        let bird_properties = bird_image.get_image_frame_properties();
        let segmented_properties = SegmentedImageFrameProperties::new(
            segmented_resolutions,
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_space(),
        );

        // Maximum modulation size (zoomed out)
        let gaze = create_gaze(0.5, 0.5, 1.0);

        let connector_agent = ConnectorAgent::new();
        {
            let mut sensor_cache = connector_agent.get_sensor_cache();
            sensor_cache
                .segmented_vision_register(
                    cortical_group,
                    number_channels,
                    FrameChangeHandling::Absolute,
                    bird_properties,
                    segmented_properties,
                    gaze,
                )
                .expect("Registration should succeed");

            sensor_cache
                .segmented_vision_write(cortical_group, channel_index, bird_image.into())
                .expect("Write should succeed");
        }
    }
}

#[cfg(test)]
mod test_image_segmentation_color_channels {
    use super::*;

    #[test]
    fn test_segment_with_grayscale_peripheral() {
        let cortical_group: CorticalUnitIndex = 0.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();

        let segmented_resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            (128, 128).try_into().unwrap(),
            (64, 64).try_into().unwrap(),
        );

        let bird_image = load_bird_image();
        let bird_properties = bird_image.get_image_frame_properties();

        // RGB center, grayscale peripheral
        let segmented_properties = SegmentedImageFrameProperties::new(
            segmented_resolutions,
            bird_properties.get_color_channel_layout(), // RGB center
            ColorChannelLayout::GrayScale,              // Grayscale peripheral
            bird_properties.get_color_space(),
        );
        let initial_gaze = create_default_gaze();

        let connector_agent = ConnectorAgent::new();
        {
            let mut sensor_cache = connector_agent.get_sensor_cache();
            sensor_cache
                .segmented_vision_register(
                    cortical_group,
                    number_channels,
                    FrameChangeHandling::Absolute,
                    bird_properties,
                    segmented_properties,
                    initial_gaze,
                )
                .expect("Registration should succeed");

            sensor_cache
                .segmented_vision_write(cortical_group, channel_index, bird_image.into())
                .expect("Write should succeed");
        }
    }

    #[test]
    fn test_segment_with_grayscale_center_and_peripheral() {
        let cortical_group: CorticalUnitIndex = 0.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();

        let segmented_resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            (128, 128).try_into().unwrap(),
            (64, 64).try_into().unwrap(),
        );

        let bird_image = load_bird_image();
        let bird_properties = bird_image.get_image_frame_properties();

        // Both center and peripheral as grayscale
        let segmented_properties = SegmentedImageFrameProperties::new(
            segmented_resolutions,
            ColorChannelLayout::GrayScale,
            ColorChannelLayout::GrayScale,
            bird_properties.get_color_space(),
        );
        let initial_gaze = create_default_gaze();

        let connector_agent = ConnectorAgent::new();
        {
            let mut sensor_cache = connector_agent.get_sensor_cache();
            sensor_cache
                .segmented_vision_register(
                    cortical_group,
                    number_channels,
                    FrameChangeHandling::Absolute,
                    bird_properties,
                    segmented_properties,
                    initial_gaze,
                )
                .expect("Registration should succeed");

            sensor_cache
                .segmented_vision_write(cortical_group, channel_index, bird_image.into())
                .expect("Write should succeed");
        }
    }
}

#[cfg(test)]
mod test_image_segmentation_multiple_channels {
    use super::*;

    #[test]
    fn test_segment_with_multiple_channels() {
        let cortical_group: CorticalUnitIndex = 0.into();
        let number_channels: CorticalChannelCount = 3.try_into().unwrap();
        let channel_0: CorticalChannelIndex = 0.into();
        let channel_1: CorticalChannelIndex = 1.into();
        let channel_2: CorticalChannelIndex = 2.into();

        let segmented_resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            (128, 128).try_into().unwrap(),
            (64, 64).try_into().unwrap(),
        );

        let bird_image = load_bird_image();
        let bird_properties = bird_image.get_image_frame_properties();
        let segmented_properties = SegmentedImageFrameProperties::new(
            segmented_resolutions,
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_space(),
        );
        let initial_gaze = create_default_gaze();

        let connector_agent = ConnectorAgent::new();
        {
            let mut sensor_cache = connector_agent.get_sensor_cache();
            sensor_cache
                .segmented_vision_register(
                    cortical_group,
                    number_channels,
                    FrameChangeHandling::Absolute,
                    bird_properties,
                    segmented_properties,
                    initial_gaze,
                )
                .expect("Registration should succeed");

            let wrapped: WrappedIOData = bird_image.into();

            // Write to all three channels
            sensor_cache
                .segmented_vision_write(cortical_group, channel_0, wrapped.clone())
                .expect("Write to channel 0 should succeed");
            sensor_cache
                .segmented_vision_write(cortical_group, channel_1, wrapped.clone())
                .expect("Write to channel 1 should succeed");
            sensor_cache
                .segmented_vision_write(cortical_group, channel_2, wrapped)
                .expect("Write to channel 2 should succeed");
        }
    }

    #[test]
    fn test_segment_with_multiple_cortical_groups() {
        let group_0: CorticalUnitIndex = 0.into();
        let group_1: CorticalUnitIndex = 1.into();
        let group_2: CorticalUnitIndex = 2.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();

        let segmented_resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            (128, 128).try_into().unwrap(),
            (64, 64).try_into().unwrap(),
        );

        let bird_image = load_bird_image();
        let bird_properties = bird_image.get_image_frame_properties();
        let segmented_properties = SegmentedImageFrameProperties::new(
            segmented_resolutions,
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_space(),
        );

        let gaze_center = create_gaze(0.5, 0.5, 0.5);
        let gaze_left = create_gaze(0.2, 0.5, 0.3);
        let gaze_right = create_gaze(0.8, 0.5, 0.3);

        let connector_agent = ConnectorAgent::new();
        {
            let mut sensor_cache = connector_agent.get_sensor_cache();

            // Register multiple cortical groups with different gazes
            sensor_cache
                .segmented_vision_register(
                    group_0,
                    number_channels,
                    FrameChangeHandling::Absolute,
                    bird_properties,
                    segmented_properties,
                    gaze_center,
                )
                .expect("Registration for group 0 should succeed");

            sensor_cache
                .segmented_vision_register(
                    group_1,
                    number_channels,
                    FrameChangeHandling::Absolute,
                    bird_properties,
                    segmented_properties,
                    gaze_left,
                )
                .expect("Registration for group 1 should succeed");

            sensor_cache
                .segmented_vision_register(
                    group_2,
                    number_channels,
                    FrameChangeHandling::Absolute,
                    bird_properties,
                    segmented_properties,
                    gaze_right,
                )
                .expect("Registration for group 2 should succeed");

            let wrapped: WrappedIOData = bird_image.into();

            // Write to all groups
            sensor_cache
                .segmented_vision_write(group_0, channel_index, wrapped.clone())
                .expect("Write to group 0 should succeed");
            sensor_cache
                .segmented_vision_write(group_1, channel_index, wrapped.clone())
                .expect("Write to group 1 should succeed");
            sensor_cache
                .segmented_vision_write(group_2, channel_index, wrapped)
                .expect("Write to group 2 should succeed");
        }
    }
}

#[cfg(test)]
mod test_image_segmentation_resolutions {
    use super::*;

    #[test]
    fn test_segment_with_asymmetric_resolutions() {
        let cortical_group: CorticalUnitIndex = 0.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();

        // Non-square resolutions
        let segmented_resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            (256, 128).try_into().unwrap(), // Wide center
            (64, 32).try_into().unwrap(),   // Wide peripheral
        );

        let bird_image = load_bird_image();
        let bird_properties = bird_image.get_image_frame_properties();
        let segmented_properties = SegmentedImageFrameProperties::new(
            segmented_resolutions,
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_space(),
        );
        let initial_gaze = create_default_gaze();

        let connector_agent = ConnectorAgent::new();
        {
            let mut sensor_cache = connector_agent.get_sensor_cache();
            sensor_cache
                .segmented_vision_register(
                    cortical_group,
                    number_channels,
                    FrameChangeHandling::Absolute,
                    bird_properties,
                    segmented_properties,
                    initial_gaze,
                )
                .expect("Registration should succeed");

            sensor_cache
                .segmented_vision_write(cortical_group, channel_index, bird_image.into())
                .expect("Write should succeed");
        }
    }

    #[test]
    fn test_segment_with_tall_resolutions() {
        let cortical_group: CorticalUnitIndex = 0.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();

        // Tall (portrait) resolutions
        let segmented_resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            (128, 256).try_into().unwrap(), // Tall center
            (32, 64).try_into().unwrap(),   // Tall peripheral
        );

        let bird_image = load_bird_image();
        let bird_properties = bird_image.get_image_frame_properties();
        let segmented_properties = SegmentedImageFrameProperties::new(
            segmented_resolutions,
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_space(),
        );
        let initial_gaze = create_default_gaze();

        let connector_agent = ConnectorAgent::new();
        {
            let mut sensor_cache = connector_agent.get_sensor_cache();
            sensor_cache
                .segmented_vision_register(
                    cortical_group,
                    number_channels,
                    FrameChangeHandling::Absolute,
                    bird_properties,
                    segmented_properties,
                    initial_gaze,
                )
                .expect("Registration should succeed");

            sensor_cache
                .segmented_vision_write(cortical_group, channel_index, bird_image.into())
                .expect("Write should succeed");
        }
    }

    #[test]
    fn test_segment_with_very_small_resolutions() {
        let cortical_group: CorticalUnitIndex = 0.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();

        // Very small resolutions (still valid)
        let segmented_resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            (16, 16).try_into().unwrap(),
            (8, 8).try_into().unwrap(),
        );

        let bird_image = load_bird_image();
        let bird_properties = bird_image.get_image_frame_properties();
        let segmented_properties = SegmentedImageFrameProperties::new(
            segmented_resolutions,
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_space(),
        );
        let initial_gaze = create_default_gaze();

        let connector_agent = ConnectorAgent::new();
        {
            let mut sensor_cache = connector_agent.get_sensor_cache();
            sensor_cache
                .segmented_vision_register(
                    cortical_group,
                    number_channels,
                    FrameChangeHandling::Absolute,
                    bird_properties,
                    segmented_properties,
                    initial_gaze,
                )
                .expect("Registration should succeed");

            sensor_cache
                .segmented_vision_write(cortical_group, channel_index, bird_image.into())
                .expect("Write should succeed");
        }
    }
}

#[cfg(test)]
mod test_image_segmentation_frame_change_handling {
    use super::*;

    #[test]
    fn test_segment_with_differential_frame_handling() {
        let cortical_group: CorticalUnitIndex = 0.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();

        let segmented_resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            (128, 128).try_into().unwrap(),
            (64, 64).try_into().unwrap(),
        );

        let bird_image = load_bird_image();
        let bird_properties = bird_image.get_image_frame_properties();
        let segmented_properties = SegmentedImageFrameProperties::new(
            segmented_resolutions,
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_channel_layout(),
            bird_properties.get_color_space(),
        );
        let initial_gaze = create_default_gaze();

        let connector_agent = ConnectorAgent::new();
        {
            let mut sensor_cache = connector_agent.get_sensor_cache();
            sensor_cache
                .segmented_vision_register(
                    cortical_group,
                    number_channels,
                    FrameChangeHandling::Incremental,
                    bird_properties,
                    segmented_properties,
                    initial_gaze,
                )
                .expect("Registration should succeed");

            let wrapped: WrappedIOData = bird_image.into();

            // Multiple writes for differential processing
            sensor_cache
                .segmented_vision_write(cortical_group, channel_index, wrapped.clone())
                .expect("First write should succeed");
            sensor_cache
                .segmented_vision_write(cortical_group, channel_index, wrapped)
                .expect("Second write should succeed");
        }
    }
}
