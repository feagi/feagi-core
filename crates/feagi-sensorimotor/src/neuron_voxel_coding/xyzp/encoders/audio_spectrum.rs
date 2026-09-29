use crate::configuration::jsonable::JSONEncoderProperties;
use crate::data_pipeline::per_channel_stream_caches::{
    PipelineStageRunner, SensoryPipelineStageRunner,
};
use crate::data_types::{AudioSpectrumFrame, AudioSpectrumProperties};
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPEncoder;
use crate::wrapped_io_data::WrappedIOType;
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelIndex,
};
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::neuron_voxels::xyzp::{
    CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZPArrays,
};
use feagi_structures::FeagiDataError;
use std::time::Instant;

/// Writes one neuron per active column: x = column, y = phase step, z = 0,
/// potential = magnitude. Channels are laid side by side along X.
#[derive(Debug)]
pub struct AudioSpectrumNeuronVoxelXYZPEncoder {
    properties: AudioSpectrumProperties,
    cortical_write_target: CorticalID,
    scratch_space: Vec<NeuronVoxelXYZPArrays>,
}

impl NeuronVoxelXYZPEncoder for AudioSpectrumNeuronVoxelXYZPEncoder {
    fn get_encodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::AudioSpectrumFrame(Some(self.properties))
    }

    fn get_as_properties(&self) -> JSONEncoderProperties {
        JSONEncoderProperties::AudioSpectrum(self.properties)
    }

    fn write_neuron_data_multi_channel_from_processed_cache(
        &mut self,
        pipelines: &[SensoryPipelineStageRunner],
        time_of_previous_burst: Instant,
        write_target: &mut CorticalMappedXYZPNeuronVoxels,
    ) -> Result<(), FeagiDataError> {
        let neuron_array_target =
            write_target.ensure_clear_and_borrow_mut(&self.cortical_write_target);

        for (channel_index, (pipeline, scratch)) in pipelines
            .iter()
            .zip(self.scratch_space.iter_mut())
            .enumerate()
        {
            if pipeline.get_last_processed_instant() < time_of_previous_burst {
                continue;
            }
            let channel = pipeline
                .get_channel_index_override()
                .unwrap_or_else(|| CorticalChannelIndex::from(channel_index as u32));
            let data = pipeline.get_postprocessed_sensor_value();
            let frame: &AudioSpectrumFrame = data.try_into()?;
            write_frame(frame, *channel * self.properties.bin_count, scratch)?;
        }

        let total_neurons: usize = self.scratch_space.iter().map(|s| s.len()).sum();
        neuron_array_target.ensure_capacity(total_neurons);
        neuron_array_target.update_vectors_from_external(
            |target_x, target_y, target_z, target_p| {
                for scratch in self.scratch_space.iter() {
                    let (x, y, z, p) = scratch.borrow_xyzp_vectors();
                    target_x.extend_from_slice(x);
                    target_y.extend_from_slice(y);
                    target_z.extend_from_slice(z);
                    target_p.extend_from_slice(p);
                }
                Ok(())
            },
        )?;
        Ok(())
    }
}

fn write_frame(
    frame: &AudioSpectrumFrame,
    x_offset: u32,
    scratch: &mut NeuronVoxelXYZPArrays,
) -> Result<(), FeagiDataError> {
    scratch.clear();
    scratch.ensure_capacity(frame.active_column_count());
    scratch.update_vectors_from_external(|x_vec, y_vec, z_vec, p_vec| {
        for (column, (magnitude, phase_step)) in frame
            .get_magnitudes()
            .iter()
            .zip(frame.get_phase_steps())
            .enumerate()
        {
            if *magnitude <= 0.0 {
                continue;
            }
            x_vec.push(x_offset + column as u32);
            y_vec.push(*phase_step);
            z_vec.push(0);
            p_vec.push(*magnitude);
        }
        Ok(())
    })
}

impl AudioSpectrumNeuronVoxelXYZPEncoder {
    pub fn new_box(
        cortical_write_target: CorticalID,
        properties: AudioSpectrumProperties,
        number_channels: CorticalChannelCount,
    ) -> Result<Box<dyn NeuronVoxelXYZPEncoder + Sync + Send>, FeagiDataError> {
        properties.validate()?;
        Ok(Box::new(AudioSpectrumNeuronVoxelXYZPEncoder {
            properties,
            cortical_write_target,
            scratch_space: vec![NeuronVoxelXYZPArrays::new(); *number_channels as usize],
        }))
    }
}
