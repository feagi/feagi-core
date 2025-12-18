use std::time::Instant;
use rayon::prelude::*;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::cortical_area::CorticalID;
use feagi_data_structures::genomic::cortical_area::descriptors::{CorticalChannelCount, CorticalChannelDimensions, NeuronDepth};
use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use crate::data_pipeline::per_channel_stream_caches::{PipelineStageRunner, SensoryPipelineStageRunner};
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPEncoder;
use crate::wrapped_io_data::WrappedIOType;

const NEURON_TRUE_VAL: f32 = 1.0;
const NEURON_FALSE_VAL: f32 = 0.0;

#[derive(Debug, Copy, Clone)]
enum BoolState {
    Unchanged,
    False,
    True,
}

#[derive(Debug)]
pub struct BooleanNeuronVoxelXYZPEncoder {
    cortical_write_target: CorticalID,
    scratch_space: Vec<BoolState>, // # channels long
}

impl NeuronVoxelXYZPEncoder for BooleanNeuronVoxelXYZPEncoder {
    fn get_encodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::Boolean
    }

    fn write_neuron_data_multi_channel_from_processed_cache(&mut self, pipelines: &Vec<SensoryPipelineStageRunner>, time_of_previous_burst: Instant, write_target: &mut CorticalMappedXYZPNeuronVoxels) -> Result<(), FeagiDataError> {
        // If this is called, then at least one channel has had something updated
        let neuron_array_target = write_target.ensure_clear_and_borrow_mut(&self.cortical_write_target);


        pipelines.par_iter()
            .zip(self.scratch_space.par_iter_mut())
            .enumerate()
            .try_for_each(|(channel_index, (pipeline, scratch))| -> Result<(), FeagiDataError> {
                let channel_updated = pipeline.get_last_processed_instant();
                if channel_updated < time_of_previous_burst {
                    *scratch = BoolState::Unchanged;
                    return Ok(()); // We haven't updated, do nothing
                }
                let updated_data = pipeline.get_postprocessed_sensor_value();
                let updated_bool: bool = updated_data.try_into()?;
                if updated_bool {
                    *scratch = BoolState::True;
                }
                else {
                    *scratch = BoolState::False;
                }
                Ok(())
            })?;

        // Cannot parallelize due to data writing of various lengths
        for c in 0..self.scratch_space.len() as u32 {
            const Y: u32 = 0;
            const Z: u32 = 0;
            for (channel_x, changed) in self.scratch_space.iter().enumerate() {
                match changed {
                    BoolState::Unchanged => {
                        // Not possible
                        return Err(FeagiDataError::InternalError("Unable to send unchanged boolean as a changed!".into()))
                    }
                    BoolState::True => {
                        neuron_array_target.push_raw(channel_x as u32, Y, Z, NEURON_TRUE_VAL)
                    }
                    BoolState::False => {
                        neuron_array_target.push_raw(channel_x as u32, Y, Z, NEURON_FALSE_VAL)
                    }
                }
            }
        }

        Ok(())

    }
}

impl BooleanNeuronVoxelXYZPEncoder {
    pub fn new_box(cortical_write_target: CorticalID, number_channels: CorticalChannelCount) -> Result<Box<dyn NeuronVoxelXYZPEncoder + Sync + Send>, FeagiDataError> {
        let encoder = BooleanNeuronVoxelXYZPEncoder {
            cortical_write_target,
            scratch_space: vec![BoolState::Unchanged; *number_channels as usize],
        };
        Ok(Box::new(encoder))
    }
}







