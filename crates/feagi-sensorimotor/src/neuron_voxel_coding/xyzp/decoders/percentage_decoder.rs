//! Unified decoder for all percentage types (unsigned/signed, 1D-4D, linear/exponential).

use crate::data_pipeline::per_channel_stream_caches::MotorPipelineStageRunner;
use crate::data_types::{
    Percentage, Percentage2D, Percentage3D, Percentage4D, SignedPercentage, SignedPercentage2D,
    SignedPercentage3D, SignedPercentage4D,
};
use crate::neuron_voxel_coding::xyzp::coder_shared_functions::{
    decode_signed_percentage_from_fractional_exponential_neurons,
    decode_signed_percentage_from_linear_neurons,
    decode_unsigned_percentage_from_fractional_exponential_neurons,
    decode_unsigned_percentage_from_linear_neurons,
};
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPDecoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelDimensions, NeuronDepth,
};
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use feagi_structures::FeagiDataError;
use std::time::Instant;
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::PercentageNeuronPositioning;
use crate::configuration::jsonable::JSONDecoderProperties;
use crate::data_types::descriptors::PercentageChannelDimensionality;

const WIDTH_GIVEN_POSITIVE_Z_ROW: u32 = 1;

/// Decoder for all percentage types.
#[derive(Debug)]
pub struct PercentageNeuronVoxelXYZPDecoder {
    channel_dimensions: CorticalChannelDimensions,
    cortical_read_target: CorticalID,
    interpolation: PercentageNeuronPositioning,
    is_signed: bool,
    number_percentages: PercentageChannelDimensionality,
    /// For unsigned: single scratch space. For signed: used as positive scratch space.
    z_depth_scratch_space: Vec<Vec<u32>>,
    /// Only used for signed decoders
    z_depth_scratch_space_negative: Vec<Vec<u32>>,
}

impl PercentageNeuronVoxelXYZPDecoder {
    /// Create a new boxed decoder with the specified configuration.
    #[allow(dead_code)]
    pub fn new_box(
        cortical_read_target: CorticalID,
        z_resolution: NeuronDepth,
        number_channels: CorticalChannelCount,
        interpolation: PercentageNeuronPositioning,
        is_signed: bool,
        number_percentages: PercentageChannelDimensionality,
    ) -> Result<Box<dyn NeuronVoxelXYZPDecoder + Sync + Send>, FeagiDataError> {
        const CHANNEL_Y_HEIGHT: u32 = 1;
        let number_pairs_per_channel = number_percentages.as_u32();
        let channel_width = WIDTH_GIVEN_POSITIVE_Z_ROW * number_pairs_per_channel;
        let scratch_size = *number_channels as usize * number_pairs_per_channel as usize;

        let z_depth_scratch_space_negative = if is_signed {
            vec![Vec::new(); scratch_size]
        } else {
            Vec::new() // empty
        };

        let decoder = PercentageNeuronVoxelXYZPDecoder {
            channel_dimensions: CorticalChannelDimensions::new(
                channel_width,
                CHANNEL_Y_HEIGHT,
                *z_resolution,
            )?,
            cortical_read_target,
            interpolation,
            is_signed,
            number_percentages: number_percentages,
            z_depth_scratch_space: vec![Vec::new(); scratch_size],
            z_depth_scratch_space_negative,
        };
        Ok(Box::new(decoder))
    }

    /// Clear all scratch spaces
    fn clear_scratch_spaces(&mut self) {
        for scratch in self.z_depth_scratch_space.iter_mut() {
            scratch.clear();
        }
        
        for scratch in self.z_depth_scratch_space_negative.iter_mut() {
            scratch.clear();
        }
    }

    /// Decode a single unsigned percentage using the configured interpolation
    fn decode_unsigned(&self, z_vector: &Vec<u32>, target: &mut Percentage) {
        match self.interpolation {
            PercentageNeuronPositioning::Linear => {
                decode_unsigned_percentage_from_linear_neurons(
                    z_vector,
                    self.channel_dimensions.depth,
                    target,
                );
            }
            PercentageNeuronPositioning::Fractional => {
                decode_unsigned_percentage_from_fractional_exponential_neurons(z_vector, target);
            }
        }
    }

    /// Decode a single signed percentage using the configured interpolation
    fn decode_signed(
        &self,
        z_vector_positive: &Vec<u32>,
        z_vector_negative: &Vec<u32>,
        target: &mut SignedPercentage,
    ) {
        match self.interpolation {
            PercentageNeuronPositioning::Linear => {
                decode_signed_percentage_from_linear_neurons(
                    z_vector_positive,
                    z_vector_negative,
                    self.channel_dimensions.depth,
                    target,
                );
            }
            PercentageNeuronPositioning::Fractional => {
                decode_signed_percentage_from_fractional_exponential_neurons(
                    z_vector_positive,
                    z_vector_negative,
                    target,
                );
            }
        }
    }

    fn process_unsigned_channel(
        &self,
        base_index: usize,
        number_pairs: usize,
        pipeline: &mut MotorPipelineStageRunner,
        changed_flag: &mut bool,
    ) -> Result<(), FeagiDataError> {
        // Check if any data was collected
        let has_data = (0..number_pairs)
            .any(|i| !self.z_depth_scratch_space.get(base_index + i).unwrap().is_empty());

        if !has_data {
            return Ok(());
        }

        *changed_flag = true;

        match self.number_percentages {
            PercentageChannelDimensionality::D1 => {
                let percentage: &mut Percentage =
                    pipeline.get_preprocessed_cached_value_mut().try_into()?;
                let z_vector = self.z_depth_scratch_space.get(base_index).unwrap();
                if !z_vector.is_empty() {
                    self.decode_unsigned(z_vector, percentage);
                }
            }
            PercentageChannelDimensionality::D2 => {
                let percentage_2d: &mut Percentage2D =
                    pipeline.get_preprocessed_cached_value_mut().try_into()?;
                let z_a = self.z_depth_scratch_space.get(base_index).unwrap();
                let z_b = self.z_depth_scratch_space.get(base_index + 1).unwrap();
                if !z_a.is_empty() {
                    self.decode_unsigned(z_a, &mut percentage_2d.a);
                }
                if !z_b.is_empty() {
                    self.decode_unsigned(z_b, &mut percentage_2d.b);
                }
            }
            PercentageChannelDimensionality::D3 => {
                let percentage_3d: &mut Percentage3D =
                    pipeline.get_preprocessed_cached_value_mut().try_into()?;
                let z_a = self.z_depth_scratch_space.get(base_index).unwrap();
                let z_b = self.z_depth_scratch_space.get(base_index + 1).unwrap();
                let z_c = self.z_depth_scratch_space.get(base_index + 2).unwrap();
                if !z_a.is_empty() {
                    self.decode_unsigned(z_a, &mut percentage_3d.a);
                }
                if !z_b.is_empty() {
                    self.decode_unsigned(z_b, &mut percentage_3d.b);
                }
                if !z_c.is_empty() {
                    self.decode_unsigned(z_c, &mut percentage_3d.c);
                }
            }
            PercentageChannelDimensionality::D4 => {
                let percentage_4d: &mut Percentage4D =
                    pipeline.get_preprocessed_cached_value_mut().try_into()?;
                let z_a = self.z_depth_scratch_space.get(base_index).unwrap();
                let z_b = self.z_depth_scratch_space.get(base_index + 1).unwrap();
                let z_c = self.z_depth_scratch_space.get(base_index + 2).unwrap();
                let z_d = self.z_depth_scratch_space.get(base_index + 3).unwrap();
                if !z_a.is_empty() {
                    self.decode_unsigned(z_a, &mut percentage_4d.a);
                }
                if !z_b.is_empty() {
                    self.decode_unsigned(z_b, &mut percentage_4d.b);
                }
                if !z_c.is_empty() {
                    self.decode_unsigned(z_c, &mut percentage_4d.c);
                }
                if !z_d.is_empty() {
                    self.decode_unsigned(z_d, &mut percentage_4d.d);
                }
            }
        }
        Ok(())
    }

    fn process_signed_channel(
        &self,
        base_index: usize,
        number_pairs: usize,
        pipeline: &mut MotorPipelineStageRunner,
        changed_flag: &mut bool,
    ) -> Result<(), FeagiDataError> {

        // Check if any data was collected
        let has_data = (0..number_pairs).any(|i| {
            !self.z_depth_scratch_space.get(base_index + i).unwrap().is_empty()
                || !self.z_depth_scratch_space_negative.get(base_index + i).unwrap().is_empty()
        });

        if !has_data {
            return Ok(());
        }

        *changed_flag = true;

        match self.number_percentages {
            PercentageChannelDimensionality::D1 => {
                let signed_percentage: &mut SignedPercentage =
                    pipeline.get_preprocessed_cached_value_mut().try_into()?;
                let z_pos = self.z_depth_scratch_space.get(base_index).unwrap();
                let z_neg = self.z_depth_scratch_space_negative.get(base_index).unwrap();
                self.decode_signed(z_pos, z_neg, signed_percentage);
            }
            PercentageChannelDimensionality::D2 => {
                let signed_2d: &mut SignedPercentage2D =
                    pipeline.get_preprocessed_cached_value_mut().try_into()?;
                let z_a_pos = self.z_depth_scratch_space.get(base_index).unwrap();
                let z_b_pos = self.z_depth_scratch_space.get(base_index + 1).unwrap();
                let z_a_neg = self.z_depth_scratch_space_negative.get(base_index).unwrap();
                let z_b_neg = self.z_depth_scratch_space_negative.get(base_index + 1).unwrap();

                if !z_a_pos.is_empty() || !z_a_neg.is_empty() {
                    self.decode_signed(z_a_pos, z_a_neg, &mut signed_2d.a);
                }
                if !z_b_pos.is_empty() || !z_b_neg.is_empty() {
                    self.decode_signed(z_b_pos, z_b_neg, &mut signed_2d.b);
                }
            }
            PercentageChannelDimensionality::D3 => {
                let signed_3d: &mut SignedPercentage3D =
                    pipeline.get_preprocessed_cached_value_mut().try_into()?;
                let z_a_pos = self.z_depth_scratch_space.get(base_index).unwrap();
                let z_b_pos = self.z_depth_scratch_space.get(base_index + 1).unwrap();
                let z_c_pos = self.z_depth_scratch_space.get(base_index + 2).unwrap();
                let z_a_neg = self.z_depth_scratch_space_negative.get(base_index).unwrap();
                let z_b_neg = self.z_depth_scratch_space_negative.get(base_index + 1).unwrap();
                let z_c_neg = self.z_depth_scratch_space_negative.get(base_index + 2).unwrap();

                if !z_a_pos.is_empty() || !z_a_neg.is_empty() {
                    self.decode_signed(z_a_pos, z_a_neg, &mut signed_3d.a);
                }
                if !z_b_pos.is_empty() || !z_b_neg.is_empty() {
                    self.decode_signed(z_b_pos, z_b_neg, &mut signed_3d.b);
                }
                if !z_c_pos.is_empty() || !z_c_neg.is_empty() {
                    self.decode_signed(z_c_pos, z_c_neg, &mut signed_3d.c);
                }
            }
            PercentageChannelDimensionality::D4 => {
                let signed_4d: &mut SignedPercentage4D =
                    pipeline.get_preprocessed_cached_value_mut().try_into()?;
                let z_a_pos = self.z_depth_scratch_space.get(base_index).unwrap();
                let z_b_pos = self.z_depth_scratch_space.get(base_index + 1).unwrap();
                let z_c_pos = self.z_depth_scratch_space.get(base_index + 2).unwrap();
                let z_d_pos = self.z_depth_scratch_space.get(base_index + 3).unwrap();
                let z_a_neg = self.z_depth_scratch_space_negative.get(base_index).unwrap();
                let z_b_neg = self.z_depth_scratch_space_negative.get(base_index + 1).unwrap();
                let z_c_neg = self.z_depth_scratch_space_negative.get(base_index + 2).unwrap();
                let z_d_neg = self.z_depth_scratch_space_negative.get(base_index + 3).unwrap();

                if !z_a_pos.is_empty() || !z_a_neg.is_empty() {
                    self.decode_signed(z_a_pos, z_a_neg, &mut signed_4d.a);
                }
                if !z_b_pos.is_empty() || !z_b_neg.is_empty() {
                    self.decode_signed(z_b_pos, z_b_neg, &mut signed_4d.b);
                }
                if !z_c_pos.is_empty() || !z_c_neg.is_empty() {
                    self.decode_signed(z_c_pos, z_c_neg, &mut signed_4d.c);
                }
                if !z_d_pos.is_empty() || !z_d_neg.is_empty() {
                    self.decode_signed(z_d_pos, z_d_neg, &mut signed_4d.d);
                }
            }
        }
        Ok(())
    }
}

impl NeuronVoxelXYZPDecoder for PercentageNeuronVoxelXYZPDecoder {
    fn get_decodable_data_type(&self) -> WrappedIOType {
        match (self.is_signed, self.number_percentages) {
            (false, PercentageChannelDimensionality::D1) => WrappedIOType::Percentage,
            (false, PercentageChannelDimensionality::D2) => WrappedIOType::Percentage_2D,
            (false, PercentageChannelDimensionality::D3) => WrappedIOType::Percentage_3D,
            (false, PercentageChannelDimensionality::D4) => WrappedIOType::Percentage_4D,
            (true, PercentageChannelDimensionality::D1) => WrappedIOType::SignedPercentage,
            (true, PercentageChannelDimensionality::D2) => WrappedIOType::SignedPercentage_2D,
            (true, PercentageChannelDimensionality::D3) => WrappedIOType::SignedPercentage_3D,
            (true, PercentageChannelDimensionality::D4) => WrappedIOType::SignedPercentage_4D,
        }
    }

    fn get_as_properties(&self) -> JSONDecoderProperties {
        JSONDecoderProperties::Percentage(
            NeuronDepth::new(self.channel_dimensions.depth).unwrap(),
            self.interpolation,
            self.is_signed,
            self.number_percentages,
        )
    }
    
    fn read_neuron_data_multi_channel_into_pipeline_input_cache(
        &mut self,
        neurons_to_read: &CorticalMappedXYZPNeuronVoxels,
        __time_of_read: Instant,
        pipelines_with_data_to_update: &mut Vec<MotorPipelineStageRunner>,
        channel_changed: &mut Vec<bool>,
    ) -> Result<(), FeagiDataError> {
        const ONLY_ALLOWED_Y: u32 = 0;

        let neuron_array = neurons_to_read.get_neurons_of(&self.cortical_read_target);
        if neuron_array.is_none() {
            return Ok(());
        }

        let neuron_array = neuron_array.unwrap();
        if neuron_array.is_empty() {
            return Ok(());
        }

        self.clear_scratch_spaces();

        let number_of_channels = pipelines_with_data_to_update.len() as u32;
        let max_possible_x_index = self.number_percentages.as_u32() * number_of_channels;
        let z_depth = self.channel_dimensions.depth;

        // Collect neuron data into scratch spaces
        match self.is_signed {
            false => {
                for neuron in neuron_array.iter() {
                    if neuron.neuron_voxel_coordinate.y != ONLY_ALLOWED_Y
                        || neuron.potential == 0.0
                    {
                        continue;
                    }
                    if neuron.neuron_voxel_coordinate.x >= max_possible_x_index
                        || neuron.neuron_voxel_coordinate.z >= z_depth
                    {
                        continue;
                    }

                    if let Some(z_row_vector) = self
                        .z_depth_scratch_space
                        .get_mut(neuron.neuron_voxel_coordinate.x as usize)
                    {
                        z_row_vector.push(neuron.neuron_voxel_coordinate.z);
                    }
                }
            }
            true => {
                for neuron in neuron_array.iter() {
                    if neuron.neuron_voxel_coordinate.y != ONLY_ALLOWED_Y
                        || neuron.potential == 0.0
                    {
                        continue;
                    }
                    if neuron.neuron_voxel_coordinate.z >= z_depth {
                        continue;
                    }

                    // For signed: even X = positive, odd X = negative
                    // Map X coordinate to channel index
                    let channel_index = (neuron.neuron_voxel_coordinate.x / 2) as usize;
                    if channel_index >= number_of_channels as usize {
                        continue;
                    }

                    let z_row_vector = if neuron.neuron_voxel_coordinate.x % 2 == 0 {
                        self.z_depth_scratch_space.get_mut(channel_index)
                    } else {
                        self.z_depth_scratch_space_negative.get_mut(channel_index)
                    };

                    if let Some(v) = z_row_vector {
                        v.push(neuron.neuron_voxel_coordinate.z);
                    }
                }
            }
        }

        // Process each channel based on sign and dimensions
        let number_pairs = self.number_percentages.as_u32() as usize;

        for (channel_index, (pipeline, changed_flag)) in pipelines_with_data_to_update
            .iter_mut()
            .zip(channel_changed.iter_mut())
            .enumerate()
            .take(number_of_channels as usize)
        {
            match self.is_signed {
                false => {
                    let base_index = channel_index * number_pairs;
                    self.process_unsigned_channel(
                        base_index,
                        number_pairs,
                        pipeline,
                        changed_flag,
                    )?;
                }
                true => {
                    let base_index = channel_index * number_pairs;
                    self.process_signed_channel(
                        base_index,
                        number_pairs,
                        pipeline,
                        changed_flag,
                    )?;
                }
            }
        }

        Ok(())
    }
}

