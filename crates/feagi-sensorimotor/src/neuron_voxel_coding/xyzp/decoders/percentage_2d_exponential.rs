use crate::data_pipeline::per_channel_stream_caches::MotorPipelineStageRunner;
use crate::data_types::Percentage2D;
use crate::neuron_voxel_coding::xyzp::coder_shared_functions::decode_unsigned_percentage_from_fractional_exponential_neurons;
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPDecoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelDimensions, NeuronDepth,
};
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use feagi_structures::FeagiDataError;
use std::time::Instant;

#[allow(dead_code)]
const WIDTH_GIVEN_POSITIVE_Z_ROW: u32 = 1; // One row of neuron voxels along the Z represents 0 -> +1
#[allow(dead_code)]
const NUMBER_PAIRS_PER_CHANNEL: u32 = 2; // How many numbers are encoded per channel?
#[allow(dead_code)]
const CHANNEL_WIDTH: u32 = WIDTH_GIVEN_POSITIVE_Z_ROW * NUMBER_PAIRS_PER_CHANNEL;

#[derive(Debug)]
pub struct Percentage2DExponentialNeuronVoxelXYZPDecoder {
    channel_dimensions: CorticalChannelDimensions,
    cortical_read_target: CorticalID,
    z_depth_scratch_space: Vec<Vec<u32>>, // # channels * NUMBER_PAIRS_PER_CHANNEL long, basically 1 vector per 1 z rows
}

// NOTE: we need to be cautious of multiple neuron_voxels coming in affecting the result (we should average them)

impl NeuronVoxelXYZPDecoder for Percentage2DExponentialNeuronVoxelXYZPDecoder {
    fn get_decoded_data_type(&self) -> WrappedIOType {
        WrappedIOType::Percentage_2D
    }

    fn read_neuron_data_multi_channel_into_pipeline_input_cache(
        &mut self,
        neurons_to_read: &CorticalMappedXYZPNeuronVoxels,
        __time_of_read: Instant,
        pipelines_with_data_to_update: &mut Vec<MotorPipelineStageRunner>,
        channel_changed: &mut Vec<bool>,
    ) -> Result<(), FeagiDataError> {
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

        for scratch_per_z_depth in self.z_depth_scratch_space.iter_mut() {
            // Not worth making parallel
            scratch_per_z_depth.clear()
        }

        let number_of_channels = pipelines_with_data_to_update.len() as u32;
        let max_possible_x_index = CHANNEL_WIDTH * number_of_channels; // Something is wrong if we reach here
        let z_depth: u32 = self.channel_dimensions.depth;

        for neuron in neuron_array.iter() {
            // Ignoring any neuron_voxels that have no potential (if sent for some reason).
            if neuron.neuron_voxel_coordinate.y != ONLY_ALLOWED_Y || neuron.potential == 0.0 {
                continue; // Something is wrong, but currently we will just skip these
            }

            if neuron.neuron_voxel_coordinate.x >= max_possible_x_index
                || neuron.neuron_voxel_coordinate.z >= z_depth
            {
                continue; // Something is wrong, but currently we will just skip these
            }

            let z_row_vector = self
                .z_depth_scratch_space
                .get_mut(neuron.neuron_voxel_coordinate.x as usize)
                .unwrap();
            z_row_vector.push(neuron.neuron_voxel_coordinate.z)
        }

        let _z_depth_float = self.channel_dimensions.depth as f32;

        // At this point, we have numbers in scratch space to average out
        for (channel_index, (pipeline, changed_flag)) in pipelines_with_data_to_update
            .iter_mut()
            .zip(channel_changed.iter_mut())
            .enumerate()
            .take(number_of_channels as usize)
        {
            // Literally not worth making parallel... right?
            let z_row_a_index = channel_index * NUMBER_PAIRS_PER_CHANNEL as usize;

            // We need to ensure if ANY of the numbers changed (as in they added anything to the vector for that row that only originally had 0), we update it and label it as such

            let z_a_row_vector = self.z_depth_scratch_space.get(z_row_a_index).unwrap();
            let z_b_row_vector = self.z_depth_scratch_space.get(z_row_a_index + 1).unwrap();

            if z_a_row_vector.is_empty() && z_b_row_vector.is_empty() {
                continue; // No data collected for this channel. Do not emit
            }
            *changed_flag = true;
            let percentage_2d: &mut Percentage2D =
                pipeline.get_preprocessed_cached_value_mut().try_into()?;

            if !z_a_row_vector.is_empty() {
                decode_unsigned_percentage_from_fractional_exponential_neurons(
                    z_a_row_vector,
                    &mut percentage_2d.a,
                );
            }
            if !z_b_row_vector.is_empty() {
                decode_unsigned_percentage_from_fractional_exponential_neurons(
                    z_b_row_vector,
                    &mut percentage_2d.b,
                )
            }
        }

        Ok(())
    }
}

impl Percentage2DExponentialNeuronVoxelXYZPDecoder {
    #[allow(dead_code)]
    pub fn new_box(
        cortical_read_target: CorticalID,
        z_resolution: NeuronDepth,
        number_channels: CorticalChannelCount,
    ) -> Result<Box<dyn NeuronVoxelXYZPDecoder + Sync + Send>, FeagiDataError> {
        const CHANNEL_Y_HEIGHT: u32 = 1;

        let decoder = Percentage2DExponentialNeuronVoxelXYZPDecoder {
            channel_dimensions: CorticalChannelDimensions::new(
                CHANNEL_WIDTH,
                CHANNEL_Y_HEIGHT,
                *z_resolution,
            )?,
            cortical_read_target,
            z_depth_scratch_space: vec![
                Vec::new();
                *number_channels as usize
                    * NUMBER_PAIRS_PER_CHANNEL as usize
            ],
        };
        Ok(Box::new(decoder))
    }
}
