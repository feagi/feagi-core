use crate::data_pipeline::per_channel_stream_caches::MotorPipelineStageRunner;
use crate::data_types::descriptors::ImageFrameProperties;
use crate::data_types::ImageFrame;
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPDecoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};
use feagi_structures::genomic::cortical_area::descriptors::CorticalChannelCount;
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use feagi_structures::FeagiDataError;
use std::time::Instant;
use crate::configuration::jsonable::DecoderProperties;

#[derive(Debug)]
pub struct CartesianPlaneNeuronVoxelXYZPDecoder {
    cortical_read_target: CorticalID,
    image_properties: ImageFrameProperties,
}

impl NeuronVoxelXYZPDecoder for CartesianPlaneNeuronVoxelXYZPDecoder {
    fn get_decodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::ImageFrame(Some(self.image_properties))
    }

    fn get_as_properties(&self) -> DecoderProperties {
        todo!()
    }

    fn read_neuron_data_multi_channel_into_pipeline_input_cache(
        &mut self,
        neurons_to_read: &CorticalMappedXYZPNeuronVoxels,
        __time_of_read: Instant,
        pipelines_with_data_to_update: &mut Vec<MotorPipelineStageRunner>,
        channel_changed: &mut Vec<bool>,
    ) -> Result<(), FeagiDataError> {
        let neuron_array = neurons_to_read.get_neurons_of(&self.cortical_read_target);

        if neuron_array.is_none() {
            return Ok(());
        }

        let neuron_array = neuron_array.unwrap();
        if neuron_array.is_empty() {
            return Ok(());
        }

        let number_of_channels = pipelines_with_data_to_update.len() as u32;
        let resolution = self.image_properties.get_image_resolution();
        let width = resolution.width;
        let height = resolution.height;
        let max_possible_x_index = width * number_of_channels;

        for neuron in neuron_array.iter() {
            // z should be 0 (R), 1 (G), or 2 (B)
            if neuron.neuron_voxel_coordinate.x >= max_possible_x_index
                || neuron.neuron_voxel_coordinate.y >= height
                || neuron.neuron_voxel_coordinate.z >= 3
            {
                continue;
            }

            let channel_index: u32 = neuron.neuron_voxel_coordinate.x / width;
            let in_channel_x_index: u32 = neuron.neuron_voxel_coordinate.x % width;

            let image_frame: &mut ImageFrame = pipelines_with_data_to_update
                .get_mut(channel_index as usize)
                .unwrap()
                .get_preprocessed_cached_value_mut()
                .try_into()?;

            if !channel_changed[channel_index as usize] {
                image_frame.blink_image();
                channel_changed[channel_index as usize] = true;
            }

            let pixels = image_frame.get_internal_data_mut();

            // Convert from FEAGI cartesian (bottom-left origin) to image coordinates (top-left origin)
            // FEAGI: y=0 is bottom, y increases upward
            // Image: row=0 is top, row increases downward
            let row = (height - 1 - neuron.neuron_voxel_coordinate.y) as usize;
            let col = in_channel_x_index as usize;
            let color_channel = neuron.neuron_voxel_coordinate.z as usize;

            // Canonical image decoding (absolute intensity):
            // - Neuron potential (p) carries raw pixel intensity in 0..255.
            // - Clamp defensively to the representable u8 range.
            let color_val = neuron.potential.clamp(0.0, 255.0).round() as u8;

            pixels[[row, col, color_channel]] = color_val;
        }

        Ok(())
    }
}

impl CartesianPlaneNeuronVoxelXYZPDecoder {
    #[allow(dead_code)]
    pub fn new_box(
        cortical_read_target: CorticalID,
        image_properties: &ImageFrameProperties,
        _number_of_channels: CorticalChannelCount,
    ) -> Result<Box<dyn NeuronVoxelXYZPDecoder + Sync + Send>, FeagiDataError> {
        let decoder = CartesianPlaneNeuronVoxelXYZPDecoder {
            cortical_read_target,
            image_properties: *image_properties,
        };
        Ok(Box::new(decoder))
    }
}
