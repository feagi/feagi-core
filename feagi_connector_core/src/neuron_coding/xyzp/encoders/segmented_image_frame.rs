use std::time::Instant;
use rayon::prelude::*;
use std::collections::HashSet;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::CorticalID;
use feagi_data_structures::genomic::descriptors::{CorticalChannelCount, CorticalChannelIndex, CorticalGroupIndex};
use feagi_data_structures::neurons::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZPArrays};
use crate::data_pipeline::PipelineStageRunner;
use crate::data_types::descriptors::{ImageFrameProperties, SegmentedImageFrameProperties};
use crate::data_types::{ImageFrame, SegmentedImageFrame};
use crate::neuron_coding::xyzp::NeuronXYZPEncoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

#[derive(Debug)]
pub struct SegmentedImageFrameNeuronXYZPEncoder {
    segmented_image_properties: SegmentedImageFrameProperties,
    cortical_write_targets: [CorticalID; 9],
    scratch_spaces: Vec<[NeuronXYZPArrays; 9]>,
}

impl NeuronXYZPEncoder for SegmentedImageFrameNeuronXYZPEncoder {
    fn get_encodable_data_type(&self) -> WrappedIOType {
        // Since changing Image Frame Properties often mean changing channel size, we shouldn't allow doing that
        WrappedIOType::SegmentedImageFrame(Some(self.segmented_image_properties))
    }


    fn write_neuron_data_multi_channel(&mut self, pipelines: &Vec<PipelineStageRunner>, time_of_previous_burst: Instant, write_target: &mut CorticalMappedXYZPNeuronData) -> Result<(), FeagiDataError> {
        // If this is called, then at least one channel has had something updated

        for cortical_id in self.cortical_write_targets {
            _ = write_target.ensure_clear_and_borrow_mut(&cortical_id); // Make sure we clear all data
        }


        pipelines.par_iter()
            .zip(self.scratch_spaces.par_iter_mut())
            .enumerate()
            .try_for_each(|(channel_index, (pipeline, scratches))| -> Result<(), FeagiDataError> {

                let channel_updated = pipeline.get_last_processed_instant();
                if channel_updated < time_of_previous_burst {
                    return Ok(()); // We haven't updated, do nothing
                }

                let updated_data = pipeline.get_most_recent_output();
                let updated_segmented_image: &SegmentedImageFrame = updated_data.into();
                updated_segmented_image.overwrite_neuron_data(scratches, channel_index.into())?;
                Ok(())
            })?;


        // At this point, the 9 scratch vectors have the data written to them
        // TODO performance note: We know that the keys will never collide. If we play it smart, we may be able to do this in parallel with the use of unsafe{}

        for segmented_index in 0..9 {
            let scratch = &self.scratch_spaces[segmented_index];
            let cortical_id = &self.cortical_write_targets[segmented_index];

            let total_neurons: usize = scratch.iter()
                .map(|scratch| scratch.len())
                .sum();

            let neuron_array_target = write_target.ensure_clear_and_borrow_mut(&cortical_id);

            neuron_array_target.ensure_capacity(total_neurons);

            // TODO could this possibly be done in a parallel way? Probably not worth it
            neuron_array_target.update_vectors_from_external(|target_x, target_y, target_z, target_p| {
                for scratch in scratch.iter() {
                    let (scratch_x, scratch_y, scratch_z, scratch_p) = scratch.borrow_xyzp_vectors();
                    target_x.extend_from_slice(scratch_x);
                    target_y.extend_from_slice(scratch_y);
                    target_z.extend_from_slice(scratch_z);
                    target_p.extend_from_slice(scratch_p);
                }
                Ok(())
            })?;
        };

        Ok(())
    }
}

impl SegmentedImageFrameNeuronXYZPEncoder {
    pub fn new(cortical_ids: [CorticalID; 9], segmented_image_properties: SegmentedImageFrameProperties, number_channels: CorticalChannelCount) -> Result<Self, FeagiDataError> {

        Ok(SegmentedImageFrameNeuronXYZPEncoder{
            segmented_image_properties: segmented_image_properties,
            cortical_write_targets: cortical_ids,
            scratch_spaces: vec![vec![NeuronXYZPArrays::new(); 9]; *number_channels as usize].into()
        })
    }
}