use crate::configuration::jsonable::JSONDecoderProperties;
use crate::data_pipeline::per_channel_stream_caches::MotorPipelineStageRunner;
use crate::data_types::descriptors::MiscDataDimensions;
use crate::data_types::MiscData;
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPDecoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};
use feagi_structures::genomic::cortical_area::descriptors::{CorticalChannelCount, NeuronDepth};
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use feagi_structures::FeagiDataError;
use std::time::Instant;

#[derive(Debug)]
pub struct MiscDataNeuronVoxelXYZPDecoder {
    cortical_read_target: CorticalID,
    misc_dimensions: MiscDataDimensions,
}

impl NeuronVoxelXYZPDecoder for MiscDataNeuronVoxelXYZPDecoder {
    fn get_decodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::MiscData(Some(self.misc_dimensions))
    }

    fn get_as_properties(&self) -> JSONDecoderProperties {
        JSONDecoderProperties::MiscData(self.misc_dimensions)
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
        let max_possible_x_index = self.misc_dimensions.width * number_of_channels; // Something is wrong if we reach here
        let max_possible_y_index = self.misc_dimensions.height;
        let max_possible_z_index = self.misc_dimensions.depth;

        for neuron in neuron_array.iter() {
            if neuron.neuron_voxel_coordinate.x >= max_possible_x_index
                || neuron.neuron_voxel_coordinate.y >= max_possible_y_index
                || neuron.neuron_voxel_coordinate.z >= max_possible_z_index
            {
                continue;
            }

            let channel_index: u32 = neuron.neuron_voxel_coordinate.x / self.misc_dimensions.width;
            let in_channel_x_index: u32 =
                neuron.neuron_voxel_coordinate.x % self.misc_dimensions.width;
            let misc_data: &mut MiscData = pipelines_with_data_to_update
                .get_mut(channel_index as usize)
                .unwrap()
                .get_preprocessed_cached_value_mut()
                .try_into()?;
            if !channel_changed[channel_index as usize] {
                misc_data.blank_data();
                channel_changed[channel_index as usize] = true;
            }
            let internal_data = misc_data.get_internal_data_mut(); // TODO should we possibly allocate these references outside this loop?
            internal_data[(
                in_channel_x_index as usize,
                neuron.neuron_voxel_coordinate.y as usize,
                neuron.neuron_voxel_coordinate.z as usize,
            )] = neuron.potential.clamp(-1.0, 1.0);
        }

        Ok(())
    }
}

impl MiscDataNeuronVoxelXYZPDecoder {
    #[allow(dead_code)]
    pub fn new_box(
        cortical_read_target: CorticalID,
        misc_dimensions: MiscDataDimensions,
        _number_of_channels: CorticalChannelCount,
    ) -> Result<Box<dyn NeuronVoxelXYZPDecoder + Sync + Send>, FeagiDataError> {
        // Yeah, we aren't using channel count right now but every other coder needs this param, lets keep up the pattern eh
        let decoder = MiscDataNeuronVoxelXYZPDecoder {
            cortical_read_target,
            misc_dimensions,
        };
        Ok(Box::new(decoder))
    }
}
