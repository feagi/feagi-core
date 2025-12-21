use crate::data_pipeline::per_channel_stream_caches::{
    PipelineStageRunner, SensoryPipelineStageRunner,
};
use crate::data_types::SignedPercentage4D;
use crate::neuron_voxel_coding::xyzp::coder_shared_functions::encode_signed_percentage_to_fractional_exponential_neuron_z_indexes;
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPEncoder;
use crate::wrapped_io_data::WrappedIOType;
use feagi_data_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelDimensions, NeuronDepth,
};
use feagi_data_structures::genomic::cortical_area::CorticalID;
use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use feagi_data_structures::FeagiDataError;
use rayon::prelude::*;
use std::time::Instant;

#[allow(dead_code)]
const NUMBER_PAIRS_PER_CHANNEL: u32 = 4; // How many numbers are encoded per channel?
#[allow(dead_code)]
const WIDTH_GIVEN_POSITIVE_Z_ROW: u32 = 1; // One row of neuron voxels along the Z represents 0 -> +1
#[allow(dead_code)]
const WIDTH_GIVEN_NEGATIVE_Z_ROW: u32 = 1; // One row of neuron voxels along the Z represents 0 -> -1
#[allow(dead_code)]
const CHANNEL_WIDTH: u32 =
    NUMBER_PAIRS_PER_CHANNEL * (WIDTH_GIVEN_POSITIVE_Z_ROW + WIDTH_GIVEN_NEGATIVE_Z_ROW);

type ScratchSpaceTuple4DSigned = (
    (Vec<u32>, Vec<u32>),
    (Vec<u32>, Vec<u32>),
    (Vec<u32>, Vec<u32>),
    (Vec<u32>, Vec<u32>),
);

#[derive(Debug)]
#[allow(dead_code)]
pub struct SignedPercentage4DExponentialNeuronVoxelXYZPEncoder {
    channel_dimensions: CorticalChannelDimensions,
    cortical_write_target: CorticalID,
    scratch_space: Vec<ScratchSpaceTuple4DSigned>, // # channels long - (a_pos, a_neg), (b_pos, b_neg), (c_pos, c_neg), (d_pos, d_neg)
}

impl NeuronVoxelXYZPEncoder for SignedPercentage4DExponentialNeuronVoxelXYZPEncoder {
    fn get_encodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::SignedPercentage_4D
    }

    fn write_neuron_data_multi_channel_from_processed_cache(
        &mut self,
        pipelines: &[SensoryPipelineStageRunner],
        time_of_previous_burst: Instant,
        write_target: &mut CorticalMappedXYZPNeuronVoxels,
    ) -> Result<(), FeagiDataError> {
        // If this is called, then at least one channel has had something updated

        let neuron_array_target =
            write_target.ensure_clear_and_borrow_mut(&self.cortical_write_target);

        pipelines
            .par_iter()
            .zip(self.scratch_space.par_iter_mut())
            .enumerate()
            .try_for_each(
                |(_channel_index, (pipeline, scratch))| -> Result<(), FeagiDataError> {
                    let channel_updated = pipeline.get_last_processed_instant();
                    if channel_updated < time_of_previous_burst {
                        return Ok(()); // We haven't updated, do nothing
                    }
                    let updated_data = pipeline.get_postprocessed_sensor_value();
                    let updated_signed_percentage_4d: SignedPercentage4D =
                        updated_data.try_into()?;

                    // scratch arrays get cleared
                    encode_signed_percentage_to_fractional_exponential_neuron_z_indexes(
                        &updated_signed_percentage_4d.a,
                        self.channel_dimensions.depth,
                        &mut scratch.0 .0,
                        &mut scratch.0 .1,
                    );
                    encode_signed_percentage_to_fractional_exponential_neuron_z_indexes(
                        &updated_signed_percentage_4d.b,
                        self.channel_dimensions.depth,
                        &mut scratch.1 .0,
                        &mut scratch.1 .1,
                    );
                    encode_signed_percentage_to_fractional_exponential_neuron_z_indexes(
                        &updated_signed_percentage_4d.c,
                        self.channel_dimensions.depth,
                        &mut scratch.2 .0,
                        &mut scratch.2 .1,
                    );
                    encode_signed_percentage_to_fractional_exponential_neuron_z_indexes(
                        &updated_signed_percentage_4d.d,
                        self.channel_dimensions.depth,
                        &mut scratch.3 .0,
                        &mut scratch.3 .1,
                    );

                    Ok(())
                },
            )?;

        // Cannot parallelize due to data writing of various lengths
        for c in 0..self.scratch_space.len() as u32 {
            const Y: u32 = 0;
            let channel_scratch = &self.scratch_space[c as usize];

            // Write 'a' positive values
            for a_pos in &channel_scratch.0 .0 {
                neuron_array_target.push_raw(c * CHANNEL_WIDTH, Y, *a_pos, 1.0);
            }
            // Write 'a' negative values
            for a_neg in &channel_scratch.0 .1 {
                neuron_array_target.push_raw((c * CHANNEL_WIDTH) + 1, Y, *a_neg, 1.0);
            }
            // Write 'b' positive values
            for b_pos in &channel_scratch.1 .0 {
                neuron_array_target.push_raw((c * CHANNEL_WIDTH) + 2, Y, *b_pos, 1.0);
            }
            // Write 'b' negative values
            for b_neg in &channel_scratch.1 .1 {
                neuron_array_target.push_raw((c * CHANNEL_WIDTH) + 3, Y, *b_neg, 1.0);
            }
            // Write 'c' positive values
            for c_pos in &channel_scratch.2 .0 {
                neuron_array_target.push_raw((c * CHANNEL_WIDTH) + 4, Y, *c_pos, 1.0);
            }
            // Write 'c' negative values
            for c_neg in &channel_scratch.2 .1 {
                neuron_array_target.push_raw((c * CHANNEL_WIDTH) + 5, Y, *c_neg, 1.0);
            }
            // Write 'd' positive values
            for d_pos in &channel_scratch.3 .0 {
                neuron_array_target.push_raw((c * CHANNEL_WIDTH) + 6, Y, *d_pos, 1.0);
            }
            // Write 'd' negative values
            for d_neg in &channel_scratch.3 .1 {
                neuron_array_target.push_raw((c * CHANNEL_WIDTH) + 7, Y, *d_neg, 1.0);
            }
        }

        Ok(())
    }
}

impl SignedPercentage4DExponentialNeuronVoxelXYZPEncoder {
    #[allow(dead_code)]
    pub fn new_box(
        cortical_write_target: CorticalID,
        z_resolution: NeuronDepth,
        number_channels: CorticalChannelCount,
    ) -> Result<Box<dyn NeuronVoxelXYZPEncoder + Sync + Send>, FeagiDataError> {
        const CHANNEL_Y_HEIGHT: u32 = 1;

        let encoder = SignedPercentage4DExponentialNeuronVoxelXYZPEncoder {
            channel_dimensions: CorticalChannelDimensions::new(
                *number_channels * CHANNEL_WIDTH,
                CHANNEL_Y_HEIGHT,
                *z_resolution,
            )?,
            cortical_write_target,
            scratch_space: vec![
                (
                    (Vec::new(), Vec::new()),
                    (Vec::new(), Vec::new()),
                    (Vec::new(), Vec::new()),
                    (Vec::new(), Vec::new())
                );
                *number_channels as usize
            ],
        };
        Ok(Box::new(encoder))
    }
}
