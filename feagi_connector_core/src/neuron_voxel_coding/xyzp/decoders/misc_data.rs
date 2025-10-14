use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::CorticalID;
use feagi_data_structures::genomic::descriptors::CorticalChannelCount;
use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use crate::data_types::descriptors::MiscDataDimensions;
use crate::data_types::MiscData;
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPDecoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

#[derive(Debug)]
pub struct MiscDataNeuronVoxelXYZPDecoder {
    cortical_read_target: CorticalID,
    misc_dimensions: MiscDataDimensions
}

impl NeuronVoxelXYZPDecoder for MiscDataNeuronVoxelXYZPDecoder {
    fn get_decoded_data_type(&self) -> WrappedIOType {
        WrappedIOType::MiscData(Some(self.misc_dimensions))
    }

    fn read_neuron_data_multi_channel(&mut self, read_target: &CorticalMappedXYZPNeuronVoxels, _time_of_read: Instant, write_target: &mut Vec<WrappedIOData>, channel_changed: &mut Vec<bool>) -> Result<(), FeagiDataError> {
        let neuron_array = read_target.get_neurons_of(&self.cortical_read_target);

        if neuron_array.is_none() {
            return Ok(());
        }

        let neuron_array = neuron_array.unwrap();
        if neuron_array.is_empty() {
            return Ok(());
        }

        let number_of_channels = write_target.len() as u32;
        let max_possible_x_index = self.misc_dimensions.width * number_of_channels; // Something is wrong if we reach here
        let max_possible_y_index = self.misc_dimensions.height;
        let max_possible_z_index = self.misc_dimensions.depth;

        for neuron in neuron_array.iter() {
            if neuron.cortical_coordinate.x >= max_possible_x_index || neuron.cortical_coordinate.y >= max_possible_y_index || neuron.cortical_coordinate.z >= max_possible_z_index {
                continue;
            }

            let channel_index: u32  = neuron.cortical_coordinate.x / self.misc_dimensions.width;
            let in_channel_x_index: u32  = neuron.cortical_coordinate.x % self.misc_dimensions.width;
            let misc_data: &mut MiscData = write_target.get_mut(channel_index as usize).unwrap().try_into()?;
            if !channel_changed[channel_index as usize] {
                misc_data.blank_data();
                channel_changed[channel_index as usize] = true;
            }
            let internal_data = misc_data.get_internal_data_mut();
            internal_data[(in_channel_x_index as usize, neuron.cortical_coordinate.y as usize, neuron.cortical_coordinate.z as usize)] = neuron.potential.clamp(-1.0, 1.0);



        };

        Ok(())

    }
}

impl MiscDataNeuronVoxelXYZPDecoder {
    pub fn new_box(cortical_read_target: CorticalID, misc_dimensions: MiscDataDimensions, _number_of_channels: CorticalChannelCount) -> Result<Box<dyn NeuronVoxelXYZPDecoder + Sync + Send>, FeagiDataError> {
        // Yeah, we aren't using channel count right now but every other coder needs this param, lets keep up the pattern eh
        let decoder = MiscDataNeuronVoxelXYZPDecoder {
            cortical_read_target,
            misc_dimensions
        };
        Ok(Box::new(decoder))
    }
}

