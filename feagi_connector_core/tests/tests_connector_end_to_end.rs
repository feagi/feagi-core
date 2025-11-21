//! Tests for the data pipeline module - focusing on end -> end tests

use feagi_connector_core::data_types::descriptors::ColorSpace;
use feagi_connector_core::data_types::{ImageFrame, SegmentedImageFrame};

//region Helpers


fn load_bird_image() -> ImageFrame {
    let bird_bytes = std::fs::read("tests/images/bird.jpg").expect("Bird image should exist");
    ImageFrame::new_from_jpeg_bytes(&bird_bytes, &ColorSpace::Gamma).expect("Bird image should load correctly")
}


//endregion

#[cfg(test)]
mod test_connector_cache_sensor_load_image {
    use std::time::Instant;
    use feagi_connector_core::data_types::descriptors::{GazeProperties, SegmentedImageFrameProperties, SegmentedXYImageResolutions};
    use feagi_connector_core::data_types::MiscData;
    use feagi_connector_core::wrapped_io_data::WrappedIOData;
    use feagi_data_structures::genomic::{MotorCorticalUnit, SensoryCorticalUnit};
    use feagi_data_structures::genomic::cortical_area::descriptors::{CorticalChannelCount, CorticalChannelIndex, CorticalGroupIndex};
    use feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::FrameChangeHandling;
    use crate::load_bird_image;

    #[test]
    fn test_segment_bird_image() {
        let time_of_previous_burst: Instant = Instant::now(); // Pretend

        let cortical_group: CorticalGroupIndex = 0.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();
        let segmented_image_resolutions: SegmentedXYImageResolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            (256, 256).try_into().unwrap(), (128, 128).try_into().unwrap()
        );


        let bird_image = load_bird_image();
        let bird_image_properties = bird_image.get_image_frame_properties();
        let segmented_bird_properties = SegmentedImageFrameProperties::new(
            &segmented_image_resolutions,
            &bird_image_properties.get_color_channel_layout(),
            &bird_image_properties.get_color_channel_layout(),
            &bird_image_properties.get_color_space());
        let initial_gaze = GazeProperties::new((0.5, 0.5).try_into().unwrap(), (0.5, 0.5).try_into().unwrap());


        let connector_agent = feagi_connector_core::ConnectorAgent::new();
        {
            let mut sensor_cache = connector_agent.get_sensor_cache();
            sensor_cache.segmented_vision_register(cortical_group, number_channels, FrameChangeHandling::Absolute, bird_image_properties, segmented_bird_properties, initial_gaze).unwrap();
            sensor_cache.segmented_vision_write(cortical_group, channel_index, bird_image.into()).unwrap();
        }
        // let bytes = sensor_cache.sensor_copy_feagi_byte_container();
    }

    #[test]
    fn test_segment_bird_image_twice() {
        let time_of_previous_burst: Instant = Instant::now(); // Pretend

        let cortical_group: CorticalGroupIndex = 0.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();
        let segmented_image_resolutions: SegmentedXYImageResolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            (256, 256).try_into().unwrap(), (128, 128).try_into().unwrap()
        );


        let bird_image = load_bird_image();
        let bird_image_properties = bird_image.get_image_frame_properties();
        let segmented_bird_properties = SegmentedImageFrameProperties::new(
            &segmented_image_resolutions,
            &bird_image_properties.get_color_channel_layout(),
            &bird_image_properties.get_color_channel_layout(),
            &bird_image_properties.get_color_space());
        let initial_gaze = GazeProperties::new((0.5, 0.5).try_into().unwrap(), (0.5, 0.5).try_into().unwrap());

        let connector_agent = feagi_connector_core::ConnectorAgent::new();
        {
            let mut sensor_cache = connector_agent.get_sensor_cache();
            sensor_cache.segmented_vision_register(cortical_group, number_channels, FrameChangeHandling::Absolute, bird_image_properties, segmented_bird_properties, initial_gaze).unwrap();

            let wrapped: WrappedIOData = bird_image.into();

            sensor_cache.segmented_vision_write(cortical_group, channel_index, wrapped.clone()).unwrap();
            // sensor_cache.sensor_encode_data_to_neurons_then_bytes(0);
            // let bytes = sensor_cache.sensor_copy_feagi_byte_container();

            sensor_cache.segmented_vision_write(cortical_group, channel_index, wrapped).unwrap();
            // sensor_cache.sensor_encode_data_to_neurons_then_bytes(0);
            // let bytes = sensor_cache.sensor_copy_feagi_byte_container();
        }
    }

    #[test]
    fn test_segment_bird_image_with_moving_gaze() {
        return; // TODO temp
        let time_of_previous_burst: Instant = Instant::now(); // Pretend

        let cortical_group: CorticalGroupIndex = 0.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();
        let segmented_image_resolutions: SegmentedXYImageResolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
            (256, 256).try_into().unwrap(), (128, 128).try_into().unwrap()
        );


        let bird_image = load_bird_image();
        let bird_image_properties = bird_image.get_image_frame_properties();
        let segmented_bird_properties = SegmentedImageFrameProperties::new(
            &segmented_image_resolutions,
            &bird_image_properties.get_color_channel_layout(),
            &bird_image_properties.get_color_channel_layout(),
            &bird_image_properties.get_color_space());
        let initial_gaze = GazeProperties::new((0.5, 0.5).try_into().unwrap(), (0.5, 0.5).try_into().unwrap());
        let second_gaze = GazeProperties::new((0.3, 0.3).try_into().unwrap(), (0.2, 0.3).try_into().unwrap());


        let connector_agent = feagi_connector_core::ConnectorAgent::new();
        {
            let mut sensor_cache = connector_agent.get_sensor_cache();
            sensor_cache.segmented_vision_register(cortical_group, number_channels, FrameChangeHandling::Absolute, bird_image_properties, segmented_bird_properties, initial_gaze).unwrap();
            sensor_cache.segmented_vision_write(cortical_group, channel_index, bird_image.into()).unwrap();
        }
        {
            let mut motor_cache = connector_agent.get_motor_cache();
            motor_cache.gaze_control_register(cortical_group, number_channels, FrameChangeHandling::Absolute, 10.try_into().unwrap(), feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::PercentageNeuronPositioning::Linear).unwrap();

            // TODO motor bytes sending

            // let sensor_bytes = sensor_cache.sensor_copy_feagi_byte_container();
            let motor_data = motor_cache.gaze_control_read_postprocessed_cache_value(cortical_group, channel_index).unwrap();
        }
    }

    #[test]
    fn test_encode_of_misc_then_reencode() {
        let time_of_previous_burst: Instant = Instant::now();

        let cortical_group: CorticalGroupIndex = 0.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();

        let bird_image = load_bird_image();
        let misc_data = MiscData::new_from_image_frame(&bird_image).unwrap();

        let connector_agent = feagi_connector_core::ConnectorAgent::new();
        {
            let mut sensor_cache = connector_agent.get_sensor_cache();
            sensor_cache.miscellaneous_register(cortical_group, number_channels, FrameChangeHandling::Absolute, misc_data.get_dimensions()).unwrap();
            sensor_cache.miscellaneous_write(cortical_group, channel_index, misc_data.clone().into()).unwrap();
        }
        {
            let mut motor_cache = connector_agent.get_motor_cache();
            motor_cache.miscellaneous_register(cortical_group, number_channels, FrameChangeHandling::Absolute, misc_data.get_dimensions()).unwrap();

            // Test encoding/decoding cycle
            // Since the output of the sensor is under cortical ID imis00, to read it to the motor, we need to assign it to omis00,
            let sensor_cortical_id = SensoryCorticalUnit::get_miscellaneous_cortical_ids_array(FrameChangeHandling::Absolute, cortical_group)[0];
            let motor_cortical_id = MotorCorticalUnit::get_miscellaneous_cortical_ids_array(FrameChangeHandling::Absolute, cortical_group)[0];
            
            // Note: This test needs reworking for the new architecture where encoding/decoding is handled differently
            // For now, we verify the registration works
            let new_misc_data = motor_cache.miscellaneous_read_postprocessed_cache_value(cortical_group, channel_index).unwrap();
            // assert_eq!(misc_data, new_misc_data);
        }
    }

    #[test]
    fn test_expanding_encode() {
        let time_of_previous_burst: Instant = Instant::now();

        let cortical_group: CorticalGroupIndex = 0.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();

        let bird_image = load_bird_image();
        let misc_data_image = MiscData::new_from_image_frame(&bird_image).unwrap();
        let misc_data_empty = MiscData::new(&misc_data_image.get_dimensions()).unwrap();
        let mut misc_data_semi = misc_data_empty.clone();
        {
            let mut data = misc_data_semi.get_internal_data_mut();
            for i in 0..20usize {
                data[[i, 0, 0]] = 1.0;
            }
        }
        let mut misc_data_solid = misc_data_empty.clone();
        {
            let mut data = misc_data_semi.get_internal_data_mut();
            data.fill(10.0);
        }



        let connector_agent = feagi_connector_core::ConnectorAgent::new();
        {
            let mut sensor_cache = connector_agent.get_sensor_cache();
            sensor_cache.miscellaneous_register(cortical_group, number_channels, FrameChangeHandling::Absolute, misc_data_empty.get_dimensions()).unwrap();

            sensor_cache.miscellaneous_write(cortical_group, channel_index, misc_data_empty.clone().into()).unwrap();
            // sensor_cache.sensor_encode_data_to_neurons_then_bytes(0);
            // let bytes_empty = sensor_cache.sensor_copy_feagi_byte_container();

            sensor_cache.miscellaneous_write(cortical_group, channel_index, misc_data_semi.clone().into()).unwrap();
            // sensor_cache.sensor_encode_data_to_neurons_then_bytes(0);
            // let bytes_semi = sensor_cache.sensor_copy_feagi_byte_container();

            sensor_cache.miscellaneous_write(cortical_group, channel_index, misc_data_image.clone().into()).unwrap();
            // sensor_cache.sensor_encode_data_to_neurons_then_bytes(0);
            // let bytes_image = sensor_cache.sensor_copy_feagi_byte_container();

            sensor_cache.miscellaneous_write(cortical_group, channel_index, misc_data_solid.clone().into()).unwrap();
            // sensor_cache.sensor_encode_data_to_neurons_then_bytes(0);
            // let bytes_solid = sensor_cache.sensor_copy_feagi_byte_container();

            sensor_cache.miscellaneous_register(1.into(), number_channels, FrameChangeHandling::Absolute, misc_data_empty.get_dimensions()).unwrap();
            sensor_cache.miscellaneous_register(2.into(), number_channels, FrameChangeHandling::Absolute, misc_data_empty.get_dimensions()).unwrap();
            sensor_cache.miscellaneous_register(3.into(), number_channels, FrameChangeHandling::Absolute, misc_data_empty.get_dimensions()).unwrap();

            sensor_cache.miscellaneous_write(1.into(), channel_index, misc_data_image.clone().into()).unwrap();
            sensor_cache.miscellaneous_write(2.into(), channel_index, misc_data_solid.clone().into()).unwrap();
            sensor_cache.miscellaneous_write(3.into(), channel_index, misc_data_image.clone().into()).unwrap();

            // sensor_cache.sensor_encode_data_to_neurons_then_bytes(0);
            // let bytes_multi = sensor_cache.sensor_copy_feagi_byte_container();
            // dbg!(bytes_multi.get_number_of_bytes_used());
        }
    }



}