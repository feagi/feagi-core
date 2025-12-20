use crate::data_pipeline::per_channel_stream_caches::{
    PipelineStageRunner, SensoryPipelineStageRunner,
};
use crate::data_types::Percentage;
use crate::neuron_voxel_coding::xyzp::coder_shared_functions::encode_unsigned_percentage_to_fractional_exponential_neuron_z_indexes;
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

const NUMBER_PAIRS_PER_CHANNEL: u32 = 1; // How many numbers are encoded per channel?
const CHANNEL_WIDTH: u32 = NUMBER_PAIRS_PER_CHANNEL * 1;

#[derive(Debug)]
pub struct PercentageExponentialNeuronVoxelXYZPEncoder {
    channel_dimensions: CorticalChannelDimensions,
    cortical_write_target: CorticalID,
    scratch_space: Vec<Vec<u32>>, // # channels long
}

impl NeuronVoxelXYZPEncoder for PercentageExponentialNeuronVoxelXYZPEncoder {
    fn get_encodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::Percentage
    }

    fn write_neuron_data_multi_channel_from_processed_cache(
        &mut self,
        pipelines: &Vec<SensoryPipelineStageRunner>,
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
                |(__channel_index, (pipeline, scratch))| -> Result<(), FeagiDataError> {
                    let channel_updated = pipeline.get_last_processed_instant();
                    if channel_updated < time_of_previous_burst {
                        return Ok(()); // We haven't updated, do nothing
                    }
                    let updated_data = pipeline.get_postprocessed_sensor_value();
                    let updated_percentage: Percentage = updated_data.try_into()?;

                    // scratch array gets cleared
                    encode_unsigned_percentage_to_fractional_exponential_neuron_z_indexes(
                        &updated_percentage,
                        self.channel_dimensions.depth,
                        scratch,
                    );

                    Ok(())
                },
            )?;

        // Cannot parallelize due to data writing of various lengths
        for c in 0..self.scratch_space.len() as u32 {
            const Y: u32 = 0;
            let channel_scratch = &self.scratch_space[c as usize];
            for z in channel_scratch {
                neuron_array_target.push_raw(c * CHANNEL_WIDTH, Y, *z, 1.0);
            }
        }

        Ok(())
    }
}

impl PercentageExponentialNeuronVoxelXYZPEncoder {
    pub fn new_box(
        cortical_write_target: CorticalID,
        z_resolution: NeuronDepth,
        number_channels: CorticalChannelCount,
    ) -> Result<Box<dyn NeuronVoxelXYZPEncoder + Sync + Send>, FeagiDataError> {
        const CHANNEL_Y_HEIGHT: u32 = 1;

        let encoder = PercentageExponentialNeuronVoxelXYZPEncoder {
            channel_dimensions: CorticalChannelDimensions::new(
                *number_channels * CHANNEL_WIDTH,
                CHANNEL_Y_HEIGHT,
                *z_resolution,
            )?,
            cortical_write_target,
            scratch_space: vec![Vec::new(); *number_channels as usize],
        };
        Ok(Box::new(encoder))
    }
}
