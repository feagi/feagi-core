use std::time::Instant;
use rayon::prelude::*;
use std::collections::HashSet;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::CorticalID;
use feagi_data_structures::genomic::descriptors::{CorticalChannelCount, CorticalChannelIndex, CorticalGroupIndex};
use feagi_data_structures::neuron_voxels::xyzp::{CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZPArrays};
use crate::data_pipeline::PipelineStageRunner;
use crate::data_types::descriptors::{ImageFrameProperties, SegmentedImageFrameProperties};
use crate::data_types::{ImageFrame, SegmentedImageFrame};
use crate::neuron_coding::xyzp::NeuronVoxelXYZPEncoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

#[derive(Debug)]
pub struct SegmentedImageFrameNeuronVoxelXYZPEncoder {
    segmented_image_properties: SegmentedImageFrameProperties,
    cortical_write_targets: [CorticalID; 9],
    neuron_scratch_spaces: Vec<[NeuronVoxelXYZPArrays; 9]>, //channel index {segment index }
}

impl NeuronVoxelXYZPEncoder for SegmentedImageFrameNeuronVoxelXYZPEncoder {
    fn get_encodable_data_type(&self) -> WrappedIOType {
        // Since changing Image Frame Properties often mean changing channel size, we shouldn't allow doing that
        WrappedIOType::SegmentedImageFrame(Some(self.segmented_image_properties))
    }


    fn write_neuron_data_multi_channel(&mut self, pipelines: &Vec<PipelineStageRunner>, time_of_previous_burst: Instant, write_target: &mut CorticalMappedXYZPNeuronVoxels) -> Result<(), FeagiDataError> {
        // If this is called, then at least one channel has had something updated


        // Parallel iterate over channels
        pipelines.par_iter()
            .zip(self.neuron_scratch_spaces.par_iter_mut())
            .enumerate()
            .try_for_each(|(channel_index, (pipeline, scratches))| -> Result<(), FeagiDataError> {



                let channel_updated = pipeline.get_last_processed_instant();
                if channel_updated < time_of_previous_burst {
                    return Ok(()); // We haven't updated, do nothing
                }

                let updated_data = pipeline.get_most_recent_output();
                let updated_segmented_image: &SegmentedImageFrame = updated_data.try_into()?;

                updated_segmented_image.overwrite_neuron_data(scratches, (channel_index as u32).into())?;
                Ok(())
            })?;

        // At this point, each channels set of scratch vectors have their 9 scratch vectors with data written to them
        // Lets count the number of neuron_voxels in each segment across all their channels for proper allocation, before moving the data
        for segmented_index in 0..9 {
            let mut neuron_count_in_segment: usize = 0;
            for channel_index in 0..pipelines.len() {
                neuron_count_in_segment += self.neuron_scratch_spaces[channel_index][segmented_index].len();
            }

            let cortical_id = &self.cortical_write_targets[segmented_index];
            let neuron_array_target = write_target.ensure_clear_and_borrow_mut(&cortical_id);
            neuron_array_target.ensure_capacity(neuron_count_in_segment);

            for channel_index in 0..pipelines.len() {
                let scratch_for_channel_and_segment = &self.neuron_scratch_spaces[channel_index][segmented_index];
                let (scratch_x, scratch_y, scratch_z, scratch_p) = scratch_for_channel_and_segment.borrow_xyzp_vectors();
                neuron_array_target.update_vectors_from_external(|target_x, target_y, target_z, target_p| {
                    target_x.extend_from_slice(scratch_x);
                    target_y.extend_from_slice(scratch_y);
                    target_z.extend_from_slice(scratch_z);
                    target_p.extend_from_slice(scratch_p);
                    Ok(())
                })?;
            }
        }
        Ok(())
    }
}

impl SegmentedImageFrameNeuronVoxelXYZPEncoder {
    pub fn new_box(cortical_ids: [CorticalID; 9], segmented_image_properties: SegmentedImageFrameProperties, number_channels: CorticalChannelCount) -> Result<Box<dyn NeuronVoxelXYZPEncoder + Sync + Send>, FeagiDataError> {
        let encoder = SegmentedImageFrameNeuronVoxelXYZPEncoder{
            segmented_image_properties: segmented_image_properties,
            cortical_write_targets: cortical_ids,
            neuron_scratch_spaces: vec![[NeuronVoxelXYZPArrays::new(),
                                     NeuronVoxelXYZPArrays::new(),
                                     NeuronVoxelXYZPArrays::new(),
                                     NeuronVoxelXYZPArrays::new(),
                                     NeuronVoxelXYZPArrays::new(),
                                     NeuronVoxelXYZPArrays::new(),
                                     NeuronVoxelXYZPArrays::new(),
                                     NeuronVoxelXYZPArrays::new(),
                                     NeuronVoxelXYZPArrays::new()];
                                        *number_channels as usize]
        };
        Ok(Box::new(encoder))
    }
}