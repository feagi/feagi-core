//! Unified decoder for GazeProperties (linear or exponential).

use crate::data_pipeline::per_channel_stream_caches::MotorPipelineStageRunner;
use crate::data_types::GazeProperties;
use crate::neuron_voxel_coding::xyzp::coder_shared_functions::{
    decode_unsigned_percentage_from_fractional_exponential_neurons,
    decode_unsigned_percentage_from_linear_neurons,
};
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPDecoder;
use crate::wrapped_io_data::WrappedIOType;
use crate::configuration::jsonable::DecoderProperties;
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
                *number_channels as usize * ECCENTRICITY_CHANNEL_WIDTH as usize
            ],
            z_depth_modularity_scratch_space: vec![
                Vec::new();
                *number_channels as usize * MODULARITY_CHANNEL_WIDTH as usize
            ],
        };
        Ok(Box::new(decoder))
    }
}

impl NeuronVoxelXYZPDecoder for GazePropertiesNeuronVoxelXYZPDecoder {
    fn get_decodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::GazeProperties
    }

    fn get_as_properties(&self) -> DecoderProperties {
        DecoderProperties::GazeProperties(
            NeuronDepth::new(self.channel_eccentricity_dimensions.depth).unwrap(),
            NeuronDepth::new(self.channel_modularity_dimensions.depth).unwrap(),
            self.interpolation.into(),
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

        if eccentricity_neuron_array.is_none() && modularity_neuron_array.is_none() {
            return Ok(());
        }

        let eccentricity_neuron_array = eccentricity_neuron_array.unwrap();
        let modularity_neuron_array = modularity_neuron_array.unwrap();
        if eccentricity_neuron_array.is_empty() && modularity_neuron_array.is_empty() {
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
        for neuron in eccentricity_neuron_array.iter() {
            if neuron.neuron_voxel_coordinate.y != ONLY_ALLOWED_Y || neuron.potential == 0.0 {
                continue;
            }

            if neuron.neuron_voxel_coordinate.x >= (number_of_channels * ECCENTRICITY_CHANNEL_WIDTH)
                || neuron.neuron_voxel_coordinate.z >= eccentricity_z_depth
            {
                continue;
            }

            let z_row_vector = self
                .z_depth_eccentricity_scratch_space
                .get_mut(neuron.neuron_voxel_coordinate.x as usize)
                .unwrap();
            z_row_vector.push(neuron.neuron_voxel_coordinate.z);
        }

        // Collect modularity neuron data
        for neuron in modularity_neuron_array.iter() {
            if neuron.neuron_voxel_coordinate.y != ONLY_ALLOWED_Y || neuron.potential == 0.0 {
                continue;
            }

            if neuron.neuron_voxel_coordinate.x >= (number_of_channels * MODULARITY_CHANNEL_WIDTH)
                || neuron.neuron_voxel_coordinate.z >= modularity_z_depth
            {
                continue;
            }

            let z_row_vector = self
                .z_depth_modularity_scratch_space
                .get_mut(neuron.neuron_voxel_coordinate.x as usize)
                .unwrap();
            z_row_vector.push(neuron.neuron_voxel_coordinate.z);
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
                .unwrap();
            let eccentricity_z_b_vector = self
                .z_depth_eccentricity_scratch_space
                .get(eccentricity_z_row_b_index)
                .unwrap();
            let modularity_z_vector = self
                .z_depth_modularity_scratch_space
                .get(modularity_z_row_index)
                .unwrap();

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

