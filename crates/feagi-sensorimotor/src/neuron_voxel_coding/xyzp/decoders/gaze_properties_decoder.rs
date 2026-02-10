//! Unified decoder for GazeProperties (linear or exponential).

use crate::configuration::jsonable::JSONDecoderProperties;
use crate::data_pipeline::per_channel_stream_caches::MotorPipelineStageRunner;
use crate::data_types::GazeProperties;
use crate::neuron_voxel_coding::xyzp::coder_shared_functions::{
    decode_unsigned_percentage_from_fractional_exponential_neurons,
    decode_unsigned_percentage_from_linear_neurons,
};
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPDecoder;
use crate::wrapped_io_data::WrappedIOType;
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelDimensions, NeuronDepth,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::PercentageNeuronPositioning;
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use feagi_structures::FeagiDataError;
use std::time::Instant;

const ECCENTRICITY_CHANNEL_WIDTH: u32 = 2;
const MODULARITY_CHANNEL_WIDTH: u32 = 1;

/// Decoder for GazeProperties (supports both linear and exponential interpolation).
#[derive(Debug)]
pub struct GazePropertiesNeuronVoxelXYZPDecoder {
    channel_eccentricity_dimensions: CorticalChannelDimensions,
    channel_modularity_dimensions: CorticalChannelDimensions,
    cortical_eccentricity_read_target: CorticalID,
    cortical_modularity_read_target: CorticalID,
    interpolation: PercentageNeuronPositioning,
    z_depth_eccentricity_scratch_space: Vec<Vec<u32>>,
    z_depth_modularity_scratch_space: Vec<Vec<u32>>,
}

impl GazePropertiesNeuronVoxelXYZPDecoder {
    #[allow(dead_code)]
    pub fn new_box(
        eccentricity_cortical_id: CorticalID,
        modularity_cortical_id: CorticalID,
        eccentricity_z_depth: NeuronDepth,
        modularity_z_depth: NeuronDepth,
        number_channels: CorticalChannelCount,
        interpolation: PercentageNeuronPositioning,
    ) -> Result<Box<dyn NeuronVoxelXYZPDecoder + Sync + Send>, FeagiDataError> {
        const CHANNEL_Y_HEIGHT: u32 = 1;

        let decoder = GazePropertiesNeuronVoxelXYZPDecoder {
            channel_eccentricity_dimensions: CorticalChannelDimensions::new(
                ECCENTRICITY_CHANNEL_WIDTH,
                CHANNEL_Y_HEIGHT,
                *eccentricity_z_depth,
            )?,
            channel_modularity_dimensions: CorticalChannelDimensions::new(
                MODULARITY_CHANNEL_WIDTH,
                CHANNEL_Y_HEIGHT,
                *modularity_z_depth,
            )?,
            cortical_eccentricity_read_target: eccentricity_cortical_id,
            cortical_modularity_read_target: modularity_cortical_id,
            interpolation,
            z_depth_eccentricity_scratch_space: vec![
                Vec::new();
                *number_channels as usize
                    * ECCENTRICITY_CHANNEL_WIDTH as usize
            ],
            z_depth_modularity_scratch_space: vec![
                Vec::new();
                *number_channels as usize
                    * MODULARITY_CHANNEL_WIDTH as usize
            ],
        };
        Ok(Box::new(decoder))
    }
}

impl NeuronVoxelXYZPDecoder for GazePropertiesNeuronVoxelXYZPDecoder {
    fn get_decodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::GazeProperties
    }

    fn get_as_properties(&self) -> JSONDecoderProperties {
        JSONDecoderProperties::GazeProperties(
            NeuronDepth::new(self.channel_eccentricity_dimensions.depth).unwrap(),
            NeuronDepth::new(self.channel_modularity_dimensions.depth).unwrap(),
            self.interpolation,
        )
    }

    fn read_neuron_data_multi_channel_into_pipeline_input_cache(
        &mut self,
        neurons_to_read: &CorticalMappedXYZPNeuronVoxels,
        _time_of_read: Instant,
        pipelines_with_data_to_update: &mut Vec<MotorPipelineStageRunner>,
        channel_changed: &mut Vec<bool>,
    ) -> Result<(), FeagiDataError> {
        const ONLY_ALLOWED_Y: u32 = 0;

        let eccentricity_neuron_array =
            neurons_to_read.get_neurons_of(&self.cortical_eccentricity_read_target);
        let modularity_neuron_array =
            neurons_to_read.get_neurons_of(&self.cortical_modularity_read_target);

        // IMPORTANT:
        // FEAGI motor packets for gaze may arrive partially (e.g. only eccentricity OR only modulation),
        // especially during area activation / warm-up. Treat missing cortical IDs as "no update"
        // instead of panicking.
        if eccentricity_neuron_array.is_none() && modularity_neuron_array.is_none() {
            return Ok(());
        }

        let has_any_data = eccentricity_neuron_array.is_some_and(|a| !a.is_empty())
            || modularity_neuron_array.is_some_and(|a| !a.is_empty());
        if !has_any_data {
            return Ok(());
        }

        // Clear scratch spaces
        for scratch in self.z_depth_eccentricity_scratch_space.iter_mut() {
            scratch.clear();
        }
        for scratch in self.z_depth_modularity_scratch_space.iter_mut() {
            scratch.clear();
        }

        let number_of_channels = pipelines_with_data_to_update.len() as u32;
        let eccentricity_z_depth: u32 = self.channel_eccentricity_dimensions.depth;
        let modularity_z_depth: u32 = self.channel_modularity_dimensions.depth;

        // Collect eccentricity neuron data
        if let Some(eccentricity_neuron_array) = eccentricity_neuron_array {
            for neuron in eccentricity_neuron_array.iter() {
                if neuron.neuron_voxel_coordinate.y != ONLY_ALLOWED_Y || neuron.potential == 0.0 {
                    continue;
                }

                if neuron.neuron_voxel_coordinate.x
                    >= (number_of_channels * ECCENTRICITY_CHANNEL_WIDTH)
                    || neuron.neuron_voxel_coordinate.z >= eccentricity_z_depth
                {
                    continue;
                }

                let z_row_vector = self
                    .z_depth_eccentricity_scratch_space
                    .get_mut(neuron.neuron_voxel_coordinate.x as usize)
                    .ok_or_else(|| {
                        FeagiDataError::InternalError(
                            "Eccentricity scratch space indexing error".into(),
                        )
                    })?;
                z_row_vector.push(neuron.neuron_voxel_coordinate.z);
            }
        }

        // Collect modularity neuron data
        if let Some(modularity_neuron_array) = modularity_neuron_array {
            for neuron in modularity_neuron_array.iter() {
                if neuron.neuron_voxel_coordinate.y != ONLY_ALLOWED_Y || neuron.potential == 0.0 {
                    continue;
                }

                if neuron.neuron_voxel_coordinate.x
                    >= (number_of_channels * MODULARITY_CHANNEL_WIDTH)
                    || neuron.neuron_voxel_coordinate.z >= modularity_z_depth
                {
                    continue;
                }

                let z_row_vector = self
                    .z_depth_modularity_scratch_space
                    .get_mut(neuron.neuron_voxel_coordinate.x as usize)
                    .ok_or_else(|| {
                        FeagiDataError::InternalError(
                            "Modularity scratch space indexing error".into(),
                        )
                    })?;
                z_row_vector.push(neuron.neuron_voxel_coordinate.z);
            }
        }

        // Decode into pipeline caches
        for (channel_index, (pipeline, changed_flag)) in pipelines_with_data_to_update
            .iter_mut()
            .zip(channel_changed.iter_mut())
            .enumerate()
            .take(number_of_channels as usize)
        {
            let eccentricity_z_row_a_index = channel_index * ECCENTRICITY_CHANNEL_WIDTH as usize;
            let eccentricity_z_row_b_index = eccentricity_z_row_a_index + 1;
            let modularity_z_row_index = channel_index;

            let eccentricity_z_a_vector = self
                .z_depth_eccentricity_scratch_space
                .get(eccentricity_z_row_a_index)
                .ok_or_else(|| {
                    FeagiDataError::InternalError("Eccentricity scratch space read error".into())
                })?;
            let eccentricity_z_b_vector = self
                .z_depth_eccentricity_scratch_space
                .get(eccentricity_z_row_b_index)
                .ok_or_else(|| {
                    FeagiDataError::InternalError("Eccentricity scratch space read error".into())
                })?;
            let modularity_z_vector = self
                .z_depth_modularity_scratch_space
                .get(modularity_z_row_index)
                .ok_or_else(|| {
                    FeagiDataError::InternalError("Modularity scratch space read error".into())
                })?;

            if eccentricity_z_a_vector.is_empty()
                && eccentricity_z_b_vector.is_empty()
                && modularity_z_vector.is_empty()
            {
                continue;
            }

            *changed_flag = true;
            let prev_gaze: &mut GazeProperties =
                pipeline.get_preprocessed_cached_value_mut().try_into()?;

            match self.interpolation {
                PercentageNeuronPositioning::Linear => {
                    if !eccentricity_z_a_vector.is_empty() {
                        decode_unsigned_percentage_from_linear_neurons(
                            eccentricity_z_a_vector,
                            self.channel_eccentricity_dimensions.depth,
                            &mut prev_gaze.eccentricity_location_xy.a,
                        );
                    }
                    if !eccentricity_z_b_vector.is_empty() {
                        decode_unsigned_percentage_from_linear_neurons(
                            eccentricity_z_b_vector,
                            self.channel_eccentricity_dimensions.depth,
                            &mut prev_gaze.eccentricity_location_xy.b,
                        );
                    }
                    if !modularity_z_vector.is_empty() {
                        decode_unsigned_percentage_from_linear_neurons(
                            modularity_z_vector,
                            self.channel_modularity_dimensions.depth,
                            &mut prev_gaze.modulation_size,
                        );
                    }
                }
                PercentageNeuronPositioning::Fractional => {
                    if !eccentricity_z_a_vector.is_empty() {
                        decode_unsigned_percentage_from_fractional_exponential_neurons(
                            eccentricity_z_a_vector,
                            &mut prev_gaze.eccentricity_location_xy.a,
                        );
                    }
                    if !eccentricity_z_b_vector.is_empty() {
                        decode_unsigned_percentage_from_fractional_exponential_neurons(
                            eccentricity_z_b_vector,
                            &mut prev_gaze.eccentricity_location_xy.b,
                        );
                    }
                    if !modularity_z_vector.is_empty() {
                        decode_unsigned_percentage_from_fractional_exponential_neurons(
                            modularity_z_vector,
                            &mut prev_gaze.modulation_size,
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use feagi_structures::genomic::cortical_area::CoreCorticalType;
    use feagi_structures::neuron_voxels::xyzp::{
        CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZPArrays,
    };

    /// Ensures partial gaze packets do not panic.
    #[test]
    fn gaze_decoder_does_not_panic_on_partial_gaze_packet() {
        // This regression test ensures that when only ONE of the gaze cortical IDs
        // is present in the motor packet (common during activation/warm-up),
        // the decoder returns Ok(()) instead of panicking via unwrap().

        // Minimal decoder: 1 channel, 1-depth each, linear interpolation.
        let eccentricity_id = CoreCorticalType::Power.to_cortical_id();
        let modularity_id = CoreCorticalType::Death.to_cortical_id();

        let mut decoder = GazePropertiesNeuronVoxelXYZPDecoder::new_box(
            eccentricity_id,
            modularity_id,
            NeuronDepth::new(1).unwrap(),
            NeuronDepth::new(1).unwrap(),
            CorticalChannelCount::new(1).unwrap(),
            PercentageNeuronPositioning::Linear,
        )
        .unwrap();

        // Motor packet contains ONLY eccentricity array, modularity missing.
        let mut voxels = CorticalMappedXYZPNeuronVoxels::new();
        let _ = voxels.insert(eccentricity_id, NeuronVoxelXYZPArrays::new());

        let mut pipelines: Vec<MotorPipelineStageRunner> = Vec::new();
        let mut changed: Vec<bool> = Vec::new();

        // Should not panic; should return Ok.
        let result = decoder.read_neuron_data_multi_channel_into_pipeline_input_cache(
            &voxels,
            Instant::now(),
            &mut pipelines,
            &mut changed,
        );
        assert!(result.is_ok());
    }
}
