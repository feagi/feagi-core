use std::time::Instant;
use rayon::prelude::*;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::CorticalID;
use feagi_data_structures::genomic::descriptors::CorticalChannelCount;
use feagi_data_structures::neuron_voxels::xyzp::{CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZPArrays};
use crate::data_pipeline::PipelineStageRunner;
use crate::data_types::descriptors::ImageFrameProperties;
use crate::data_types::ImageFrame;
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPEncoder;
use crate::wrapped_io_data::{ WrappedIOType};

#[derive(Debug)]
pub struct ImageFrameNeuronVoxelXYZPEncoder {
    image_properties: ImageFrameProperties,
    cortical_write_target: CorticalID,
    scratch_space: Vec<NeuronVoxelXYZPArrays>,
}

impl NeuronVoxelXYZPEncoder for ImageFrameNeuronVoxelXYZPEncoder {
    fn get_encodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::ImageFrame(Some(self.image_properties))
    }

    fn write_neuron_data_multi_channel(&mut self, pipelines: &Vec<PipelineStageRunner>, time_of_previous_burst: Instant, write_target: &mut CorticalMappedXYZPNeuronVoxels) -> Result<(), FeagiDataError> {
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
                let updated_data = pipeline.get_most_recent_output();
                let updated_image: &ImageFrame = updated_data.try_into()?;
                updated_image.overwrite_neuron_data(scratch, (channel_index as u32).into())?;
                Ok(())
            })?;

        let total_neurons: usize = self.scratch_space.iter()
            .map(|scratch| scratch.len())
            .sum();

        neuron_array_target.ensure_capacity(total_neurons);

        // TODO could this possibly be done in a parallel way? Probably not worth it
        neuron_array_target.update_vectors_from_external(|target_x, target_y, target_z, target_p| {
            for scratch in self.scratch_space.iter() {
                let (scratch_x, scratch_y, scratch_z, scratch_p) = scratch.borrow_xyzp_vectors();
                target_x.extend_from_slice(scratch_x);
                target_y.extend_from_slice(scratch_y);
                target_z.extend_from_slice(scratch_z);
                target_p.extend_from_slice(scratch_p);
            }
            Ok(())
        })?;
        Ok(())
    }
}

impl ImageFrameNeuronVoxelXYZPEncoder {
    pub fn new_box(cortical_write_target: CorticalID, image_properties: &ImageFrameProperties, number_channels: CorticalChannelCount) -> Result<Box<dyn NeuronVoxelXYZPEncoder + Sync + Send>, FeagiDataError> {
        let encoder = ImageFrameNeuronVoxelXYZPEncoder{
            image_properties: image_properties.clone(),
            cortical_write_target,
            scratch_space: vec![NeuronVoxelXYZPArrays::new(); *number_channels as usize],
        };
        Ok(Box::new(encoder))
    }
}