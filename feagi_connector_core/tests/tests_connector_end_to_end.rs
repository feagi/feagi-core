//! Tests for the data pipeline module - focusing on end -> end tests

use feagi_connector_core::data_types::descriptors::ColorSpace;
use feagi_connector_core::data_types::ImageFrame;

//region Helpers


fn load_bird_image() -> ImageFrame {
    let bird_bytes = std::fs::read("tests/images/bird.jpg").expect("Bird image should exist");
    ImageFrame::new_from_jpeg_bytes(&bird_bytes, &ColorSpace::Gamma).expect("Bird image should load correctly")
}



//endregion

#[cfg(test)]
mod test_connector_cache_sensor_load_image {
    use std::ops::Deref;
    use std::time::Instant;
    use feagi_connector_core::data_types::descriptors::{ColorChannelLayout, ColorSpace, GazeProperties, ImageXYResolution, SegmentedImageFrameProperties, SegmentedXYImageResolutions};
    use feagi_connector_core::data_types::MiscData;
    use feagi_connector_core::wrapped_io_data::WrappedIOData;
    use feagi_data_serialization::{FeagiByteContainer, FeagiSerializable};
    use feagi_data_structures::genomic::{CorticalID, MotorCorticalType, SensorCorticalType};
    use feagi_data_structures::genomic::descriptors::{CorticalChannelCount, CorticalChannelIndex, CorticalGroupIndex};
    use feagi_data_structures::neuron_voxels::xyzp::{CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZPArrays};
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


        let mut connector_cache = feagi_connector_core::IOCache::new();
        connector_cache.sensor_segmented_vision_absolute_try_register(cortical_group, number_channels, bird_image_properties, segmented_bird_properties, initial_gaze).unwrap();
        connector_cache.sensor_segmented_vision_absolute_try_write(cortical_group, channel_index, &bird_image.into()).unwrap();
        let bytes = connector_cache.sensor_copy_feagi_byte_container();
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

        let mut connector_cache = feagi_connector_core::IOCache::new();
        connector_cache.sensor_segmented_vision_absolute_try_register(cortical_group, number_channels, bird_image_properties, segmented_bird_properties, initial_gaze).unwrap();

        let wrapped: WrappedIOData = bird_image.into();

        connector_cache.sensor_segmented_vision_absolute_try_write(cortical_group, channel_index, &wrapped).unwrap();
        connector_cache.sensor_encode_data_to_bytes(0);
        let bytes = connector_cache.sensor_copy_feagi_byte_container();

        connector_cache.sensor_segmented_vision_absolute_try_write(cortical_group, channel_index, &wrapped).unwrap();
        connector_cache.sensor_encode_data_to_bytes(0);
        let bytes = connector_cache.sensor_copy_feagi_byte_container();
    }

    #[test]
    fn test_segment_bird_image_with_moving_gaze() {
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


        let mut connector_cache = feagi_connector_core::IOCache::new();
        connector_cache.sensor_segmented_vision_absolute_try_register(cortical_group, number_channels, bird_image_properties, segmented_bird_properties, initial_gaze).unwrap();
        connector_cache.motor_gaze_absolute_linear_try_register(cortical_group, number_channels, 10.try_into().unwrap());

        connector_cache.sensor_segmented_vision_absolute_try_write(cortical_group, channel_index, &bird_image.into()).unwrap();

        // TODO motor bytes sending

        connector_cache.sensor_encode_data_to_bytes(0);
        let sensor_bytes = connector_cache.sensor_copy_feagi_byte_container();
        let motor_data = connector_cache.motor_gaze_absolute_linear_try_read_postprocessed_cached_value(cortical_group, channel_index).unwrap();
    }

    #[test]
    fn test_encode_of_misc_then_reencode() {
        let time_of_previous_burst: Instant = Instant::now();

        let cortical_group: CorticalGroupIndex = 0.into();
        let number_channels: CorticalChannelCount = 1.try_into().unwrap();
        let channel_index: CorticalChannelIndex = 0.into();

        let bird_image = load_bird_image();
        let misc_data = MiscData::new_from_image_frame(&bird_image).unwrap();

        let mut connector_cache = feagi_connector_core::IOCache::new();
        connector_cache.sensor_miscellaneous_absolute_try_register(cortical_group, number_channels, misc_data.get_dimensions()).unwrap();
        connector_cache.motor_miscellaneous_absolute_try_register(cortical_group, number_channels, misc_data.get_dimensions()).unwrap();

        connector_cache.sensor_miscellaneous_absolute_try_write(cortical_group, channel_index, misc_data.clone()).unwrap();
        connector_cache.sensor_encode_data_to_bytes(0);
        let bytes = connector_cache.sensor_copy_feagi_byte_container();

        let neuron_struct_box = bytes.try_create_new_struct_from_index(0).unwrap();
        let sensor_neuron_struct: CorticalMappedXYZPNeuronVoxels = neuron_struct_box.try_into().unwrap();

        // Since the output of the sensor is under cortical ID imis00, to read it to the motor, we need to assign it to omis00,
        let neuron_data = sensor_neuron_struct.get_neurons_of(&CorticalID::new_sensor_cortical_area_id(SensorCorticalType::MiscellaneousAbsolute, cortical_group).unwrap()).unwrap();
        let mut motor_neuron_struct = CorticalMappedXYZPNeuronVoxels::new();
        motor_neuron_struct.insert(CorticalID::new_motor_cortical_area_id(MotorCorticalType::MiscellaneousAbsolute, cortical_group).unwrap(), neuron_data.clone());



        let mut new_bytes: FeagiByteContainer = FeagiByteContainer::new_empty();
        new_bytes.overwrite_byte_data_with_single_struct_data(&motor_neuron_struct, 0).unwrap();

        connector_cache.motor_replace_feagi_byte_container(new_bytes);
        connector_cache.motor_update_data_from_bytes();
        let new_misc_data = connector_cache.motor_miscellaneous_absolute_try_read_postprocessed_cached_value(cortical_group, channel_index).unwrap();
        assert_eq!(misc_data, new_misc_data);
    }



}