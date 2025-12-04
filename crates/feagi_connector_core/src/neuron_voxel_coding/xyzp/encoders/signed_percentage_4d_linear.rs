use std::time::Instant;
use rayon::prelude::*;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::cortical_area::CorticalID;
use feagi_data_structures::genomic::cortical_area::descriptors::{CorticalChannelCount, CorticalChannelDimensions, NeuronDepth};
use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use crate::data_pipeline::PipelineStageRunner;
use crate::data_types::SignedPercentage4D;
use crate::neuron_voxel_coding::xyzp::coder_shared_functions::encode_signed_percentage_to_linear_neuron_z_index;
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPEncoder;
use crate::wrapped_io_data::WrappedIOType;

const NUMBER_PAIRS_PER_CHANNEL: u32 = 4; // How many numbers are encoded per channel?
const WIDTH_GIVEN_POSITIVE_Z_ROW: u32 = 1; // One row of neuron voxels along the Z represents 0 -> +1
const WIDTH_GIVEN_NEGATIVE_Z_ROW: u32 = 1; // One row of neuron voxels along the Z represents 0 -> -1
const CHANNEL_WIDTH: u32 = NUMBER_PAIRS_PER_CHANNEL * (WIDTH_GIVEN_POSITIVE_Z_ROW + WIDTH_GIVEN_NEGATIVE_Z_ROW);

#[derive(Debug)]
pub struct SignedPercentage4DLinearNeuronVoxelXYZPEncoder {
    channel_dimensions: CorticalChannelDimensions,
    cortical_write_target: CorticalID,
    scratch_space: Vec<((Vec<u32>, Vec<u32>), (Vec<u32>, Vec<u32>), (Vec<u32>, Vec<u32>), (Vec<u32>, Vec<u32>))>, // # channels long - (a_pos, a_neg), (b_pos, b_neg), (c_pos, c_neg), (d_pos, d_neg)
}

impl NeuronVoxelXYZPEncoder for SignedPercentage4DLinearNeuronVoxelXYZPEncoder {
    fn get_encodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::SignedPercentage_4D
    }

    fn write_neuron_data_multi_channel_from_processed_cache(&mut self, pipelines: &Vec<PipelineStageRunner>, time_of_previous_burst: Instant, write_target: &mut CorticalMappedXYZPNeuronVoxels) -> Result<(), FeagiDataError> {
        // If this is called, then at least one channel has had something updated

        let neuron_array_target = write_target.ensure_clear_and_borrow_mut(&self.cortical_write_target);

        let z_length_as_float = self.channel_dimensions.depth as f32;

        pipelines.par_iter()
            .zip(self.scratch_space.par_iter_mut())
            .enumerate()
            .try_for_each(|(channel_index, (pipeline, scratch))| -> Result<(), FeagiDataError> {
                let channel_updated = pipeline.get_last_processed_instant();
                if channel_updated < time_of_previous_burst {
                    return Ok(()); // We haven't updated, do nothing
                }
                let updated_data = pipeline.get_most_recent_postprocessed_output();
                let updated_signed_percentage_4d: SignedPercentage4D = updated_data.try_into()?;

                // scratch arrays get cleared
                encode_signed_percentage_to_linear_neuron_z_index(&updated_signed_percentage_4d.a, z_length_as_float, &mut scratch.0.0, &mut scratch.0.1);
                encode_signed_percentage_to_linear_neuron_z_index(&updated_signed_percentage_4d.b, z_length_as_float, &mut scratch.1.0, &mut scratch.1.1);
                encode_signed_percentage_to_linear_neuron_z_index(&updated_signed_percentage_4d.c, z_length_as_float, &mut scratch.2.0, &mut scratch.2.1);
                encode_signed_percentage_to_linear_neuron_z_index(&updated_signed_percentage_4d.d, z_length_as_float, &mut scratch.3.0, &mut scratch.3.1);

                Ok(())
            })?;

        // Cannot parallelize due to data writing of various lengths
        for c in 0..self.scratch_space.len() as u32 {
            const Y: u32 = 0;
            let channel_scratch = &self.scratch_space[c as usize];
            
            // Write 'a' positive values
            for a_pos in &channel_scratch.0.0 {
                neuron_array_target.push_raw(c * CHANNEL_WIDTH, Y, *a_pos, 1.0);
            }
            // Write 'a' negative values
            for a_neg in &channel_scratch.0.1 {
                neuron_array_target.push_raw((c * CHANNEL_WIDTH) + 1, Y, *a_neg, 1.0);
            }
            // Write 'b' positive values
            for b_pos in &channel_scratch.1.0 {
                neuron_array_target.push_raw((c * CHANNEL_WIDTH) + 2, Y, *b_pos, 1.0);
            }
            // Write 'b' negative values
            for b_neg in &channel_scratch.1.1 {
                neuron_array_target.push_raw((c * CHANNEL_WIDTH) + 3, Y, *b_neg, 1.0);
            }
            // Write 'c' positive values
            for c_pos in &channel_scratch.2.0 {
                neuron_array_target.push_raw((c * CHANNEL_WIDTH) + 4, Y, *c_pos, 1.0);
            }
            // Write 'c' negative values
            for c_neg in &channel_scratch.2.1 {
                neuron_array_target.push_raw((c * CHANNEL_WIDTH) + 5, Y, *c_neg, 1.0);
            }
            // Write 'd' positive values
            for d_pos in &channel_scratch.3.0 {
                neuron_array_target.push_raw((c * CHANNEL_WIDTH) + 6, Y, *d_pos, 1.0);
            }
            // Write 'd' negative values
            for d_neg in &channel_scratch.3.1 {
                neuron_array_target.push_raw((c * CHANNEL_WIDTH) + 7, Y, *d_neg, 1.0);
            }
        }

        Ok(())

    }
}

impl SignedPercentage4DLinearNeuronVoxelXYZPEncoder {
    pub fn new_box(cortical_write_target: CorticalID, z_resolution: NeuronDepth, number_channels: CorticalChannelCount) -> Result<Box<dyn NeuronVoxelXYZPEncoder + Sync + Send>, FeagiDataError> {
        const CHANNEL_Y_HEIGHT: u32 = 1;

        let encoder = SignedPercentage4DLinearNeuronVoxelXYZPEncoder {
            channel_dimensions: CorticalChannelDimensions::new(*number_channels * CHANNEL_WIDTH, CHANNEL_Y_HEIGHT, *z_resolution)?,
            cortical_write_target,
            scratch_space: vec![((Vec::new(), Vec::new()), (Vec::new(), Vec::new()), (Vec::new(), Vec::new()), (Vec::new(), Vec::new())); *number_channels as usize],
        };
        Ok(Box::new(encoder))
    }
}

