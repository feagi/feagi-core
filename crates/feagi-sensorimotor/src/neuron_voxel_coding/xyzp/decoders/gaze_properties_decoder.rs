//! Unified decoder for GazeProperties (linear or exponential).
//!
//! A depth-1 eccentricity plane wider or taller than the two-column layout averages every
//! firing neuron into one eccentricity. X and Y are `index / (span - 1)`, so the minimum
//! corner is 0 and the opposite corner is 1. A two-column eccentricity area still reads
//! each column as a Z percentage. Modulation averages every accepted Z index into one
//! percentage, with Z 0 as 1.0 and the last index as 0.0. A silent area keeps its previous value.

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
use feagi_structures::neuron_voxels::xyzp::{
    CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZPArrays,
};
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
        eccentricity_dimensions: CorticalChannelDimensions,
        modularity_dimensions: CorticalChannelDimensions,
        number_channels: CorticalChannelCount,
        interpolation: PercentageNeuronPositioning,
    ) -> Result<Box<dyn NeuronVoxelXYZPDecoder + Sync + Send>, FeagiDataError> {
        let decoder = GazePropertiesNeuronVoxelXYZPDecoder {
            channel_eccentricity_dimensions: eccentricity_dimensions,
            channel_modularity_dimensions: modularity_dimensions,
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

    /// Eccentricity stored as neuron XY rather than two Z-percentage columns.
    fn eccentricity_is_spatial_plane(&self) -> bool {
        let dimensions = self.channel_eccentricity_dimensions;
        dimensions.depth == 1
            && (dimensions.width > ECCENTRICITY_CHANNEL_WIDTH || dimensions.height > 1)
    }
}

/// Mean normalized position of every firing neuron inside an eccentricity plane.
fn average_spatial_eccentricity(
    neurons: &NeuronVoxelXYZPArrays,
    dimensions: CorticalChannelDimensions,
) -> Option<(f32, f32)> {
    let mut x_sum = 0.0_f32;
    let mut y_sum = 0.0_f32;
    let mut count = 0.0_f32;
    for neuron in neurons.iter() {
        if neuron.potential == 0.0 {
            continue;
        }
        let x = neuron.neuron_voxel_coordinate.x;
        let y = neuron.neuron_voxel_coordinate.y;
        let z = neuron.neuron_voxel_coordinate.z;
        if x >= dimensions.width || y >= dimensions.height || z >= dimensions.depth {
            continue;
        }
        let x_pos = if dimensions.width <= 1 {
            0.0
        } else {
            x as f32 / (dimensions.width - 1) as f32
        };
        let y_pos = if dimensions.height <= 1 {
            0.0
        } else {
            y as f32 / (dimensions.height - 1) as f32
        };
        x_sum += x_pos;
        y_sum += y_pos;
        count += 1.0;
    }
    (count > 0.0).then_some((x_sum / count, y_sum / count))
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

        let spatial_eccentricity = if self.eccentricity_is_spatial_plane() {
            eccentricity_neuron_array.and_then(|neurons| {
                average_spatial_eccentricity(neurons, self.channel_eccentricity_dimensions)
            })
        } else {
            None
        };

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

        // Two-column eccentricity stores X and Y as separate Z percentages.
        // A spatial plane is averaged above and does not use these columns.
        if !self.eccentricity_is_spatial_plane() {
            if let Some(eccentricity_neuron_array) = eccentricity_neuron_array {
                for neuron in eccentricity_neuron_array.iter() {
                    if neuron.neuron_voxel_coordinate.y != ONLY_ALLOWED_Y || neuron.potential == 0.0
                    {
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

            if spatial_eccentricity.is_none()
                && eccentricity_z_a_vector.is_empty()
                && eccentricity_z_b_vector.is_empty()
                && modularity_z_vector.is_empty()
            {
                continue;
            }

            *changed_flag = true;
            let prev_gaze: &mut GazeProperties =
                pipeline.get_preprocessed_cached_value_mut().try_into()?;

            if let Some((x, y)) = spatial_eccentricity {
                prev_gaze
                    .eccentricity_location_xy
                    .a
                    .inplace_update_unchecked(x);
                prev_gaze
                    .eccentricity_location_xy
                    .b
                    .inplace_update_unchecked(y);
            }

            match self.interpolation {
                PercentageNeuronPositioning::Linear => {
                    if spatial_eccentricity.is_none() && !eccentricity_z_a_vector.is_empty() {
                        decode_unsigned_percentage_from_linear_neurons(
                            eccentricity_z_a_vector,
                            self.channel_eccentricity_dimensions.depth,
                            &mut prev_gaze.eccentricity_location_xy.a,
                        );
                    }
                    if spatial_eccentricity.is_none() && !eccentricity_z_b_vector.is_empty() {
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
                    if spatial_eccentricity.is_none() && !eccentricity_z_a_vector.is_empty() {
                        decode_unsigned_percentage_from_fractional_exponential_neurons(
                            eccentricity_z_a_vector,
                            &mut prev_gaze.eccentricity_location_xy.a,
                        );
                    }
                    if spatial_eccentricity.is_none() && !eccentricity_z_b_vector.is_empty() {
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
    use crate::data_pipeline::per_channel_stream_caches::PipelineStageRunner;
    use crate::data_types::{Percentage, Percentage2D};
    use crate::wrapped_io_data::WrappedIOData;
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
            CorticalChannelDimensions::new(2, 1, 1).unwrap(),
            CorticalChannelDimensions::new(1, 1, 1).unwrap(),
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

    /// Many firing neurons collapse to one eccentricity and one modulation.
    #[test]
    fn many_gaze_activations_average_to_one_eccentricity_and_modulation() {
        let eccentricity_id = CoreCorticalType::Power.to_cortical_id();
        let modularity_id = CoreCorticalType::Death.to_cortical_id();
        let mut decoder = GazePropertiesNeuronVoxelXYZPDecoder::new_box(
            eccentricity_id,
            modularity_id,
            CorticalChannelDimensions::new(8, 8, 1).unwrap(),
            CorticalChannelDimensions::new(1, 1, 10).unwrap(),
            CorticalChannelCount::new(1).unwrap(),
            PercentageNeuronPositioning::Linear,
        )
        .unwrap();

        let initial = GazeProperties::new(
            Percentage2D::new(
                Percentage::new_from_0_1_unchecked(0.25),
                Percentage::new_from_0_1_unchecked(0.25),
            ),
            Percentage::new_from_0_1_unchecked(0.8),
        );
        let mut pipelines =
            vec![
                MotorPipelineStageRunner::new(WrappedIOData::GazeProperties(initial))
                    .expect("pipeline"),
            ];
        let mut changed = vec![false];

        let mut eccentricity = NeuronVoxelXYZPArrays::new();
        eccentricity.push_raw(0, 0, 0, 1.0);
        eccentricity.push_raw(7, 7, 0, 1.0);
        // Outside the 8x8 plane. Must not move the average.
        eccentricity.push_raw(8, 0, 0, 1.0);
        let mut modulation = NeuronVoxelXYZPArrays::new();
        modulation.push_raw(0, 0, 0, 1.0);
        modulation.push_raw(0, 0, 9, 1.0);
        let mut voxels = CorticalMappedXYZPNeuronVoxels::new();
        let _ = voxels.insert(eccentricity_id, eccentricity);
        let _ = voxels.insert(modularity_id, modulation);

        decoder
            .read_neuron_data_multi_channel_into_pipeline_input_cache(
                &voxels,
                Instant::now(),
                &mut pipelines,
                &mut changed,
            )
            .expect("decode");

        assert!(changed[0]);
        let gaze: GazeProperties = pipelines[0]
            .get_preprocessed_cached_value()
            .try_into()
            .expect("gaze");
        assert_eq!(gaze.eccentricity_location_xy.a.get_as_0_1(), 0.5);
        assert_eq!(gaze.eccentricity_location_xy.b.get_as_0_1(), 0.5);
        assert_eq!(gaze.modulation_size.get_as_0_1(), 0.5);
    }
}
