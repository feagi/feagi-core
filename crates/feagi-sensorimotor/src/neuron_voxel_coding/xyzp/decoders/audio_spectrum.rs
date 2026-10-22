use crate::configuration::jsonable::JSONDecoderProperties;
use crate::data_pipeline::per_channel_stream_caches::MotorPipelineStageRunner;
use crate::data_types::{AudioSpectrumFrame, AudioSpectrumProperties};
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPDecoder;
use crate::wrapped_io_data::WrappedIOType;
use feagi_structures::genomic::cortical_area::descriptors::CorticalChannelCount;
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use feagi_structures::FeagiDataError;
use std::time::Instant;

/// Reads x = column, y = phase step, potential = magnitude.
///
/// Every decode is one tick of audio, so every channel is silenced first and
/// marked changed even when the area has no active neurons. When several phase
/// rows fire in one column, the row with the highest potential wins.
#[derive(Debug)]
pub struct AudioSpectrumNeuronVoxelXYZPDecoder {
    cortical_read_target: CorticalID,
    properties: AudioSpectrumProperties,
}

impl NeuronVoxelXYZPDecoder for AudioSpectrumNeuronVoxelXYZPDecoder {
    fn get_decodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::AudioSpectrumFrame(Some(self.properties))
    }

    fn get_as_properties(&self) -> JSONDecoderProperties {
        JSONDecoderProperties::AudioSpectrum(self.properties)
    }

    fn read_neuron_data_multi_channel_into_pipeline_input_cache(
        &mut self,
        neurons_to_read: &CorticalMappedXYZPNeuronVoxels,
        _time_of_read: Instant,
        pipelines_with_data_to_update: &mut Vec<MotorPipelineStageRunner>,
        channel_changed: &mut Vec<bool>,
    ) -> Result<(), FeagiDataError> {
        for (pipeline, changed) in pipelines_with_data_to_update
            .iter_mut()
            .zip(channel_changed.iter_mut())
        {
            let frame: &mut AudioSpectrumFrame =
                pipeline.get_preprocessed_cached_value_mut().try_into()?;
            frame.silence();
            *changed = true;
        }

        let Some(neuron_array) = neurons_to_read.get_neurons_of(&self.cortical_read_target) else {
            return Ok(());
        };

        let columns = self.properties.bin_count;
        let channel_count = pipelines_with_data_to_update.len() as u32;
        for neuron in neuron_array.iter() {
            let coordinate = neuron.neuron_voxel_coordinate;
            if coordinate.x >= columns * channel_count
                || coordinate.y >= self.properties.phase_steps
                || coordinate.z != 0
            {
                continue;
            }
            let potential = neuron.potential.clamp(0.0, 1.0);
            if potential <= 0.0 {
                continue;
            }
            let channel = (coordinate.x / columns) as usize;
            let column = (coordinate.x % columns) as usize;
            let frame: &mut AudioSpectrumFrame = pipelines_with_data_to_update[channel]
                .get_preprocessed_cached_value_mut()
                .try_into()?;
            if potential > frame.get_magnitudes()[column] {
                frame.set_column(column, potential, coordinate.y)?;
            }
        }
        Ok(())
    }
}

impl AudioSpectrumNeuronVoxelXYZPDecoder {
    pub fn new_box(
        cortical_read_target: CorticalID,
        properties: AudioSpectrumProperties,
        _number_of_channels: CorticalChannelCount,
    ) -> Result<Box<dyn NeuronVoxelXYZPDecoder + Sync + Send>, FeagiDataError> {
        properties.validate()?;
        Ok(Box::new(AudioSpectrumNeuronVoxelXYZPDecoder {
            cortical_read_target,
            properties,
        }))
    }
}
