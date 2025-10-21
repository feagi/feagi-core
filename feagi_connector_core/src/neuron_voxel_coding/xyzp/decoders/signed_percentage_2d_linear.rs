use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::CorticalID;
use feagi_data_structures::genomic::descriptors::{CorticalChannelCount, CorticalChannelDimensions};
use feagi_data_structures::neuron_voxels::xyzp::{CorticalMappedXYZPNeuronVoxels};
use crate::data_pipeline::PipelineStageRunner;
use crate::data_types::{SignedPercentage2D};
use crate::neuron_voxel_coding::xyzp::coder_shared_functions::{decode_signed_percentage_from_linear_neurons};
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPDecoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

const WIDTH_GIVEN_POSITIVE_Z_ROW: u32 = 1; // One row of neuron voxels along the Z represents 0 -> +1
const NUMBER_PAIRS_PER_CHANNEL: u32 = 2; // How many numbers are encoded per channel?
const CHANNEL_WIDTH: u32 = WIDTH_GIVEN_POSITIVE_Z_ROW * NUMBER_PAIRS_PER_CHANNEL;

#[derive(Debug)]
pub struct SignedPercentage2DLinearNeuronVoxelXYZPDecoder {
    channel_dimensions: CorticalChannelDimensions,
    cortical_read_target: CorticalID,
    z_depth_scratch_space_positive: Vec<Vec<u32>>, // # channels * NUMBER_PAIRS_PER_CHANNEL long, basically 1 vector per 1 z rows
    z_depth_scratch_space_negative: Vec<Vec<u32>>, // # channels * NUMBER_PAIRS_PER_CHANNEL long, basically 1 vector per 1 z rows
}

// NOTE: we need to be cautious of multiple neuron_voxels coming in affecting the result (we should average them)


impl NeuronVoxelXYZPDecoder for SignedPercentage2DLinearNeuronVoxelXYZPDecoder {
    fn get_decoded_data_type(&self) -> WrappedIOType {
        WrappedIOType::SignedPercentage_2D
    }

    fn read_neuron_data_multi_channel_into_pipeline_input_cache(&mut self, neurons_to_read: &CorticalMappedXYZPNeuronVoxels, _time_of_read: Instant, pipelines_with_data_to_update: &mut Vec<PipelineStageRunner>, channel_changed: &mut Vec<bool>) -> Result<(), FeagiDataError> {

        // NOTE: Expecting channel_changed to be all false. Do not reset write_target, we will write to it if we got a value for the channel!
        const ONLY_ALLOWED_Y: u32 = 0; // This structure never has height

        let neuron_array = neurons_to_read.get_neurons_of(&self.cortical_read_target);

        if neuron_array.is_none() {
            return Ok(());
        }

        let neuron_array = neuron_array.unwrap();
        if neuron_array.is_empty() {
            return Ok(());
        }

        for scratch_per_z_depth in self.z_depth_scratch_space_positive.iter_mut() { // Not worth making parallel
            scratch_per_z_depth.clear()
        }
        for scratch_per_z_depth in self.z_depth_scratch_space_negative.iter_mut() { // Not worth making parallel
            scratch_per_z_depth.clear()
        }


        let number_of_channels = pipelines_with_data_to_update.len() as u32;
        let max_possible_x_index = CHANNEL_WIDTH * number_of_channels; // Something is wrong if we reach here
        let z_depth: u32 = self.channel_dimensions.depth;


        for neuron in neuron_array.iter() {

            // Ignoring any neuron_voxels that have no potential (if sent for some reason).
            if neuron.cortical_coordinate.y != ONLY_ALLOWED_Y || neuron.potential == 0.0 {
                continue; // Something is wrong, but currently we will just skip these
            }

            if neuron.cortical_coordinate.x >= max_possible_x_index || neuron.cortical_coordinate.z >= z_depth {
                continue; // Something is wrong, but currently we will just skip these
            }

            let z_row_vector;
            if neuron.cortical_coordinate.x % 2 == 0 {
                // even, positive
                z_row_vector = self.z_depth_scratch_space_positive.get_mut(neuron.cortical_coordinate.x as usize).unwrap();
            }
            else {
                // odd, negative
                z_row_vector = self.z_depth_scratch_space_negative.get_mut(neuron.cortical_coordinate.x as usize).unwrap();
            }
            z_row_vector.push(neuron.cortical_coordinate.z)
        };

        // At this point, we have numbers in scratch space to average out
        for channel_index in 0..number_of_channels as usize { // Literally not worth making parallel... right?
            let z_row_a_index = channel_index * NUMBER_PAIRS_PER_CHANNEL as usize;

            // We need to ensure if ANY of the numbers changed (as in they added anything to the vector for that row that only originally had 0), we update it and label it as such

            let z_a_row_vector_positive = self.z_depth_scratch_space_positive.get(z_row_a_index).unwrap();
            let z_b_row_vector_positive = self.z_depth_scratch_space_positive.get(z_row_a_index + 1).unwrap();
            let z_a_row_vector_negative = self.z_depth_scratch_space_positive.get(z_row_a_index).unwrap();
            let z_b_row_vector_negative = self.z_depth_scratch_space_positive.get(z_row_a_index + 1).unwrap();

            if z_a_row_vector_positive.is_empty() && z_b_row_vector_positive.is_empty() && z_a_row_vector_negative.is_empty() && z_b_row_vector_negative.is_empty() {
                continue; // No data collected for this channel. Do not emit
            }
            channel_changed[channel_index] = true;
            let signed_percentage_2d: &mut SignedPercentage2D = pipelines_with_data_to_update.get_mut(channel_index).unwrap().get_cached_input_mut().try_into()?;

            if !(z_a_row_vector_positive.is_empty() && z_a_row_vector_negative.is_empty()) {
                decode_signed_percentage_from_linear_neurons(&z_a_row_vector_positive, &z_a_row_vector_negative, self.channel_dimensions.depth, &mut signed_percentage_2d.a);
            }
            if !(z_b_row_vector_positive.is_empty() && z_b_row_vector_negative.is_empty()) {
                decode_signed_percentage_from_linear_neurons(&z_b_row_vector_positive, &z_b_row_vector_negative, self.channel_dimensions.depth, &mut signed_percentage_2d.b);
            }
        };

        Ok(())
    }
}

impl SignedPercentage2DLinearNeuronVoxelXYZPDecoder {

    pub fn new_box(cortical_read_target: CorticalID, z_resolution: u32, number_channels: CorticalChannelCount) -> Result<Box<dyn NeuronVoxelXYZPDecoder + Sync + Send>, FeagiDataError> {
        const CHANNEL_Y_HEIGHT: u32 = 1;

        let decoder = SignedPercentage2DLinearNeuronVoxelXYZPDecoder {
            channel_dimensions: CorticalChannelDimensions::new(CHANNEL_WIDTH, CHANNEL_Y_HEIGHT, z_resolution)?,
            cortical_read_target,
            z_depth_scratch_space_positive: vec![Vec::new(); *number_channels as usize * NUMBER_PAIRS_PER_CHANNEL as usize],
            z_depth_scratch_space_negative: vec![Vec::new(); *number_channels as usize * NUMBER_PAIRS_PER_CHANNEL as usize],
        };
        Ok(Box::new(decoder))
    }
}

