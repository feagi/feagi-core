use std::time::Instant;
use rayon::prelude::*;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::cortical_area::CorticalID;
use feagi_data_structures::genomic::cortical_area::descriptors::{CorticalChannelCount, CorticalChannelDimensions, NeuronDepth};
use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use crate::data_pipeline::PipelineStageRunner;
use crate::data_types::SignedPercentage;
use crate::neuron_voxel_coding::xyzp::coder_shared_functions::encode_signed_percentage_to_fractional_exponential_neuron_z_indexes;
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPEncoder;
use crate::wrapped_io_data::WrappedIOType;

const NUMBER_PAIRS_PER_CHANNEL: u32 = 1; // How many numbers are encoded per channel?
const WIDTH_GIVEN_POSITIVE_Z_ROW: u32 = 1; // One row of neuron voxels along the Z represents 0 -> +1
const WIDTH_GIVEN_NEGATIVE_Z_ROW: u32 = 1; // One row of neuron voxels along the Z represents 0 -> -1
const CHANNEL_WIDTH: u32 = NUMBER_PAIRS_PER_CHANNEL * (WIDTH_GIVEN_POSITIVE_Z_ROW + WIDTH_GIVEN_NEGATIVE_Z_ROW);

#[derive(Debug)]
pub struct SignedPercentage1DExponentialNeuronVoxelXYZPEncoder {
    channel_dimensions: CorticalChannelDimensions,
    cortical_write_target: CorticalID,
    scratch_space: Vec<(Vec<u32>, Vec<u32>)>, // # channels long - (positive, negative)
}

impl NeuronVoxelXYZPEncoder for SignedPercentage1DExponentialNeuronVoxelXYZPEncoder {
    fn get_encodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::SignedPercentage
    }

    fn write_neuron_data_multi_channel_from_processed_cache(&mut self, pipelines: &Vec<PipelineStageRunner>, time_of_previous_burst: Instant, write_target: &mut CorticalMappedXYZPNeuronVoxels) -> Result<(), FeagiDataError> {
        // If this is called, then at least one channel has had something updated

        let neuron_array_target = write_target.ensure_clear_and_borrow_mut(&self.cortical_write_target);

        pipelines.par_iter()
            .zip(self.scratch_space.par_iter_mut())
            .enumerate()
            .try_for_each(|(channel_index, (pipeline, scratch))| -> Result<(), FeagiDataError> {
                let channel_updated = pipeline.get_last_processed_instant();
                if channel_updated < time_of_previous_burst {
                    return Ok(()); // We haven't updated, do nothing
                }
                let updated_data = pipeline.get_most_recent_postprocessed_output();
                let updated_signed_percentage: SignedPercentage = updated_data.try_into()?;

                // scratch arrays get cleared
                encode_signed_percentage_to_fractional_exponential_neuron_z_indexes(&updated_signed_percentage, self.channel_dimensions.depth, &mut scratch.0, &mut scratch.1);

                Ok(())
            })?;

        // Cannot parallelize due to data writing of various lengths
        for c in 0..self.scratch_space.len() as u32 {
            const Y: u32 = 0;
            let channel_scratch = &self.scratch_space[c as usize];
            
            // Write positive values
            for pos in &channel_scratch.0 {
                neuron_array_target.push_raw(c * CHANNEL_WIDTH, Y, *pos, 1.0);
            }
            // Write negative values
            for neg in &channel_scratch.1 {
                neuron_array_target.push_raw((c * CHANNEL_WIDTH) + 1, Y, *neg, 1.0);
            }
        }

        Ok(())

    }
}

impl SignedPercentage1DExponentialNeuronVoxelXYZPEncoder {
    pub fn new_box(cortical_write_target: CorticalID, z_resolution: NeuronDepth, number_channels: CorticalChannelCount) -> Result<Box<dyn NeuronVoxelXYZPEncoder + Sync + Send>, FeagiDataError> {
        const CHANNEL_Y_HEIGHT: u32 = 1;

        let encoder = SignedPercentage1DExponentialNeuronVoxelXYZPEncoder {
            channel_dimensions: CorticalChannelDimensions::new(*number_channels * CHANNEL_WIDTH, CHANNEL_Y_HEIGHT, *z_resolution)?,
            cortical_write_target,
            scratch_space: vec![(Vec::new(), Vec::new()); *number_channels as usize],
        };
        Ok(Box::new(encoder))
    }
}







