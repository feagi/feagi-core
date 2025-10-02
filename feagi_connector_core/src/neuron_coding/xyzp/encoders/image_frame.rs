use std::time::Instant;
use rayon::prelude::*;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::CorticalID;
use feagi_data_structures::neurons::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZPArrays};
use crate::data_pipeline::PipelineStageRunner;
use crate::data_types::descriptors::ImageFrameProperties;
use crate::data_types::ImageFrame;
use crate::neuron_coding::xyzp::NeuronXYZPEncoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

#[derive(Debug)]
pub struct ImageFrameNeuronXYZPEncoder {
    image_properties: ImageFrameProperties,
    cortical_write_target: CorticalID,
}

impl NeuronXYZPEncoder for ImageFrameNeuronXYZPEncoder {
    fn get_encodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::ImageFrame(Some(self.image_properties))
    }

    fn write_neuron_data_multi_channel(&self, pipelines: &Vec<PipelineStageRunner>, time_of_previous_burst: Instant, write_target: &mut NeuronXYZPArrays, scratch_space: &mut Vec<NeuronXYZPArrays>) -> Result<(), FeagiDataError> {
        // If this is called, then at least one channel has had something updated

        write_target.clear();
        pipelines.par_iter()
            .zip(scratch_space.par_iter_mut())
            .enumerate()
            .try_for_each(|(index, (pipeline, scratch))| -> Result<(), FeagiDataError> {
                let channel_updated = pipeline.get_last_processed_instant();
                if channel_updated < time_of_previous_burst {
                    return Ok(()); // We haven't updated, do nothing
                }
                let updated_data = pipeline.get_most_recent_output();
                let updated_image: &ImageFrame = updated_data.into();
                let x_offset = index as u32 * self.image_properties.get_image_resolution().width;
                updated_image.overwrite_neuron_data(scratch, x_offset.into())?;
                Ok(())
            })?;
        
        // After parallel processing, combine all scratch spaces into write_target
        // First, calculate total neurons needed and ensure capacity
        let total_neurons: usize = scratch_space.iter()
            .map(|scratch| scratch.len())
            .sum();
        
        write_target.ensure_capacity(total_neurons);
        
        // Collect all scratch data into write_target using direct vector access
        write_target.update_vectors_from_external(|target_x, target_y, target_z, target_p| {
            for scratch in scratch_space.iter() {
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

impl ImageFrameNeuronXYZPEncoder {
    pub fn new(cortical_write_target: CorticalID, image_properties: &ImageFrameProperties) -> Result<Self, FeagiDataError> {
        Ok(ImageFrameNeuronXYZPEncoder{
            image_properties: image_properties.clone(),
            cortical_write_target,
        })
    }
}