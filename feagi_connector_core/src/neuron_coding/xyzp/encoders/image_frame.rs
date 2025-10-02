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
        pipelines.par_iter()
            .zip(scratch_space.par_iter_mut())
            .try_for_each(|(pipeline, scratch)| -> Result<(), FeagiDataError> {
                let channel_updated = pipeline.get_last_processed_instant();
                if channel_updated < time_of_previous_burst {
                    return Ok(()); // We haven't updated, do nothing
                }
                let updated_data = pipeline.get_most_recent_output();
                let updated_image: &ImageFrame = updated_data.into();
                updated_image.overwrite_neuron_data(scratch, ())?


                Ok(())
            })?;
        
        // After parallel processing, combine results into write_target if needed
        // TODO: Implement final aggregation
        
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