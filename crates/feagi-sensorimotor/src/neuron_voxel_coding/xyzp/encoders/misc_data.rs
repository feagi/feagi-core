use crate::data_pipeline::per_channel_stream_caches::{
    PipelineStageRunner, SensoryPipelineStageRunner,
};
use crate::data_types::descriptors::MiscDataDimensions;
use crate::data_types::MiscData;
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPEncoder;
use crate::wrapped_io_data::WrappedIOType;
use feagi_structures::genomic::cortical_area::descriptors::{CorticalChannelCount, CorticalChannelIndex};
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::neuron_voxels::xyzp::{
    CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZPArrays,
};
use feagi_structures::FeagiDataError;
use rayon::prelude::*;
use std::time::Instant;
use crate::configuration::jsonable::JSONEncoderProperties;

#[derive(Debug)]
#[allow(dead_code)]
pub struct MiscDataNeuronVoxelXYZPEncoder {
    misc_data_dimensions: MiscDataDimensions,
    cortical_write_target: CorticalID,
    scratch_space: Vec<NeuronVoxelXYZPArrays>,
}

impl NeuronVoxelXYZPEncoder for MiscDataNeuronVoxelXYZPEncoder {
    fn get_encodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::MiscData(Some(self.misc_data_dimensions))
    }

    fn get_as_properties(&self) -> JSONEncoderProperties {
        JSONEncoderProperties::MiscData(self.misc_data_dimensions)
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
                |(current_channel_index, (pipeline, scratch))| -> Result<(), FeagiDataError> {
                    let channel_updated = pipeline.get_last_processed_instant();
                    if channel_updated < time_of_previous_burst {
                        return Ok(()); // We haven't updated, do nothing
                    }
                    let channel_write_target = pipeline.get_channel_index_override()
                        .unwrap_or_else(|| CorticalChannelIndex::from(current_channel_index as u32)); // Get override if available
                    let updated_data = pipeline.get_postprocessed_sensor_value();
                    let updated_misc: &MiscData = updated_data.try_into()?;

                    updated_misc.overwrite_neuron_data(scratch, (*channel_write_target).into())?;
                    Ok(())
                },
            )?;

        let total_neurons: usize = self.scratch_space.iter().map(|scratch| scratch.len()).sum();

        neuron_array_target.ensure_capacity(total_neurons);

        // TODO could this possibly be done in a parallel way? Probably not worth it
        neuron_array_target.update_vectors_from_external(
            |target_x, target_y, target_z, target_p| {
                for scratch in self.scratch_space.iter() {
                    let (scratch_x, scratch_y, scratch_z, scratch_p) =
                        scratch.borrow_xyzp_vectors();
                    target_x.extend_from_slice(scratch_x);
                    target_y.extend_from_slice(scratch_y);
                    target_z.extend_from_slice(scratch_z);
                    target_p.extend_from_slice(scratch_p);
                }
                Ok(())
            },
        )?;
        Ok(())
    }
}

impl MiscDataNeuronVoxelXYZPEncoder {
    #[allow(dead_code)]
    pub fn new_box(
        cortical_write_target: CorticalID,
        misc_data_dimensions: MiscDataDimensions,
        number_channels: CorticalChannelCount,
    ) -> Result<Box<dyn NeuronVoxelXYZPEncoder + Sync + Send>, FeagiDataError> {
        let encoder = MiscDataNeuronVoxelXYZPEncoder {
            misc_data_dimensions,
            cortical_write_target,
            scratch_space: vec![NeuronVoxelXYZPArrays::new(); *number_channels as usize],
        };
        Ok(Box::new(encoder))
    }
}
