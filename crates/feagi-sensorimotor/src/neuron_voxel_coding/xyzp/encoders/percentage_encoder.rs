//! Unified encoder for all percentage types (unsigned/signed, 1D-4D, linear/exponential).

use crate::data_pipeline::per_channel_stream_caches::{
    PipelineStageRunner, SensoryPipelineStageRunner,
};
use crate::data_types::{
    Percentage, Percentage2D, Percentage3D, Percentage4D, SignedPercentage, SignedPercentage2D,
    SignedPercentage3D, SignedPercentage4D,
};
use crate::data_types::descriptors::PercentageChannelDimensionality;
use crate::neuron_voxel_coding::xyzp::coder_shared_functions::{
    encode_signed_percentage_to_fractional_exponential_neuron_z_indexes,
    encode_signed_percentage_to_linear_neuron_z_index,
    encode_unsigned_percentage_to_fractional_exponential_neuron_z_indexes,
    encode_unsigned_percentage_to_linear_neuron_z_index,
};
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPEncoder;
use crate::wrapped_io_data::WrappedIOType;
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelDimensions, NeuronDepth,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_data_type::PercentageNeuronPositioning;
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use feagi_structures::FeagiDataError;
use rayon::prelude::*;
use std::time::Instant;

/// Scratch space sized appropriately for dimension count
#[derive(Debug)]
enum ScratchSpace {
    D1(Vec<Vec<u32>>),
    D2(Vec<(Vec<u32>, Vec<u32>)>),
    D3(Vec<(Vec<u32>, Vec<u32>, Vec<u32>)>),
    D4(Vec<(Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>)>),
}

impl ScratchSpace {
    fn new(dims: PercentageChannelDimensionality, num_channels: usize) -> Self {
        match dims {
            PercentageChannelDimensionality::D1 => ScratchSpace::D1(vec![Vec::new(); num_channels]),
            PercentageChannelDimensionality::D2 => ScratchSpace::D2(vec![(Vec::new(), Vec::new()); num_channels]),
            PercentageChannelDimensionality::D3 => ScratchSpace::D3(vec![(Vec::new(), Vec::new(), Vec::new()); num_channels]),
            PercentageChannelDimensionality::D4 => ScratchSpace::D4(vec![(Vec::new(), Vec::new(), Vec::new(), Vec::new()); num_channels]),
        }
    }
}

/// Encoder for all percentage types.
#[derive(Debug)]
pub struct PercentageNeuronVoxelXYZPEncoder {
    channel_dimensions: CorticalChannelDimensions,
    cortical_write_target: CorticalID,
    interpolation: PercentageNeuronPositioning,
    is_signed: bool,
    number_percentages: PercentageChannelDimensionality,
    number_channels: u32,
    scratch_space: ScratchSpace,
    scratch_space_negative: ScratchSpace,
}

impl PercentageNeuronVoxelXYZPEncoder {
    #[allow(dead_code)]
    pub fn new_box(
        cortical_write_target: CorticalID,
        z_resolution: NeuronDepth,
        number_channels: CorticalChannelCount,
        interpolation: PercentageNeuronPositioning,
        is_signed: bool,
        number_percentages: PercentageChannelDimensionality,
    ) -> Result<Box<dyn NeuronVoxelXYZPEncoder + Sync + Send>, FeagiDataError> {
        const CHANNEL_Y_HEIGHT: u32 = 1;
        
        let num_dims = number_percentages.as_u32();
        let channel_width = if is_signed { num_dims * 2 } else { num_dims };
        let num_channels = *number_channels as usize;

        let encoder = PercentageNeuronVoxelXYZPEncoder {
            channel_dimensions: CorticalChannelDimensions::new(
                *number_channels * channel_width,
                CHANNEL_Y_HEIGHT,
                *z_resolution,
            )?,
            cortical_write_target,
            interpolation,
            is_signed,
            number_percentages,
            number_channels: *number_channels,
            scratch_space: ScratchSpace::new(number_percentages, num_channels),
            scratch_space_negative: ScratchSpace::new(number_percentages, num_channels),
        };
        Ok(Box::new(encoder))
    }

    fn channel_width(&self) -> u32 {
        let num_dims = self.number_percentages.as_u32();
        if self.is_signed { num_dims * 2 } else { num_dims }
    }
}

impl NeuronVoxelXYZPEncoder for PercentageNeuronVoxelXYZPEncoder {
    fn get_encodable_data_type(&self) -> WrappedIOType {
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

    fn write_neuron_data_multi_channel_from_processed_cache(
        &mut self,
        pipelines: &[SensoryPipelineStageRunner],
        time_of_previous_burst: Instant,
        write_target: &mut CorticalMappedXYZPNeuronVoxels,
    ) -> Result<(), FeagiDataError> {
        let neuron_array_target =
            write_target.ensure_clear_and_borrow_mut(&self.cortical_write_target);

        let z_depth = self.channel_dimensions.depth;
        let z_depth_float = z_depth as f32;
        let interpolation = self.interpolation;
        let is_signed = self.is_signed;
        let channel_width = self.channel_width();

        // Process based on dimensionality
        match (&mut self.scratch_space, &mut self.scratch_space_negative) {
            (ScratchSpace::D1(scratch), ScratchSpace::D1(scratch_neg)) => {
                pipelines
                    .par_iter()
                    .zip(scratch.par_iter_mut())
                    .zip(scratch_neg.par_iter_mut())
                    .try_for_each(|((pipeline, s), s_neg)| -> Result<(), FeagiDataError> {
                        if pipeline.get_last_processed_instant() < time_of_previous_burst {
                            return Ok(());
                        }
                        let data = pipeline.get_postprocessed_sensor_value();
                        if is_signed {
                            let p: SignedPercentage = data.try_into()?;
                            encode_signed(interpolation, &p, z_depth, z_depth_float, s, s_neg);
                        } else {
                            let p: Percentage = data.try_into()?;
                            encode_unsigned(interpolation, &p, z_depth, z_depth_float, s);
                        }
                        Ok(())
                    })?;

                // Write to neurons
                for (c, (s, s_neg)) in scratch.iter().zip(scratch_neg.iter()).enumerate() {
                    let c = c as u32;
                    const Y: u32 = 0;
                    for z in s { neuron_array_target.push_raw(c * channel_width, Y, *z, 1.0); }
                    if is_signed {
                        for z in s_neg { neuron_array_target.push_raw(c * channel_width + 1, Y, *z, 1.0); }
                    }
                }
            }

            (ScratchSpace::D2(scratch), ScratchSpace::D2(scratch_neg)) => {
                pipelines
                    .par_iter()
                    .zip(scratch.par_iter_mut())
                    .zip(scratch_neg.par_iter_mut())
                    .try_for_each(|((pipeline, s), s_neg)| -> Result<(), FeagiDataError> {
                        if pipeline.get_last_processed_instant() < time_of_previous_burst {
                            return Ok(());
                        }
                        let data = pipeline.get_postprocessed_sensor_value();
                        if is_signed {
                            let p: SignedPercentage2D = data.try_into()?;
                            encode_signed(interpolation, &p.a, z_depth, z_depth_float, &mut s.0, &mut s_neg.0);
                            encode_signed(interpolation, &p.b, z_depth, z_depth_float, &mut s.1, &mut s_neg.1);
                        } else {
                            let p: Percentage2D = data.try_into()?;
                            encode_unsigned(interpolation, &p.a, z_depth, z_depth_float, &mut s.0);
                            encode_unsigned(interpolation, &p.b, z_depth, z_depth_float, &mut s.1);
                        }
                        Ok(())
                    })?;

                for (c, (s, s_neg)) in scratch.iter().zip(scratch_neg.iter()).enumerate() {
                    let c = c as u32;
                    const Y: u32 = 0;
                    if is_signed {
                        for z in &s.0 { neuron_array_target.push_raw(c * channel_width, Y, *z, 1.0); }
                        for z in &s_neg.0 { neuron_array_target.push_raw(c * channel_width + 1, Y, *z, 1.0); }
                        for z in &s.1 { neuron_array_target.push_raw(c * channel_width + 2, Y, *z, 1.0); }
                        for z in &s_neg.1 { neuron_array_target.push_raw(c * channel_width + 3, Y, *z, 1.0); }
                    } else {
                        for z in &s.0 { neuron_array_target.push_raw(c * channel_width, Y, *z, 1.0); }
                        for z in &s.1 { neuron_array_target.push_raw(c * channel_width + 1, Y, *z, 1.0); }
                    }
                }
            }

            (ScratchSpace::D3(scratch), ScratchSpace::D3(scratch_neg)) => {
                pipelines
                    .par_iter()
                    .zip(scratch.par_iter_mut())
                    .zip(scratch_neg.par_iter_mut())
                    .try_for_each(|((pipeline, s), s_neg)| -> Result<(), FeagiDataError> {
                        if pipeline.get_last_processed_instant() < time_of_previous_burst {
                            return Ok(());
                        }
                        let data = pipeline.get_postprocessed_sensor_value();
                        if is_signed {
                            let p: SignedPercentage3D = data.try_into()?;
                            encode_signed(interpolation, &p.a, z_depth, z_depth_float, &mut s.0, &mut s_neg.0);
                            encode_signed(interpolation, &p.b, z_depth, z_depth_float, &mut s.1, &mut s_neg.1);
                            encode_signed(interpolation, &p.c, z_depth, z_depth_float, &mut s.2, &mut s_neg.2);
                        } else {
                            let p: Percentage3D = data.try_into()?;
                            encode_unsigned(interpolation, &p.a, z_depth, z_depth_float, &mut s.0);
                            encode_unsigned(interpolation, &p.b, z_depth, z_depth_float, &mut s.1);
                            encode_unsigned(interpolation, &p.c, z_depth, z_depth_float, &mut s.2);
                        }
                        Ok(())
                    })?;

                for (c, (s, s_neg)) in scratch.iter().zip(scratch_neg.iter()).enumerate() {
                    let c = c as u32;
                    const Y: u32 = 0;
                    if is_signed {
                        for z in &s.0 { neuron_array_target.push_raw(c * channel_width, Y, *z, 1.0); }
                        for z in &s_neg.0 { neuron_array_target.push_raw(c * channel_width + 1, Y, *z, 1.0); }
                        for z in &s.1 { neuron_array_target.push_raw(c * channel_width + 2, Y, *z, 1.0); }
                        for z in &s_neg.1 { neuron_array_target.push_raw(c * channel_width + 3, Y, *z, 1.0); }
                        for z in &s.2 { neuron_array_target.push_raw(c * channel_width + 4, Y, *z, 1.0); }
                        for z in &s_neg.2 { neuron_array_target.push_raw(c * channel_width + 5, Y, *z, 1.0); }
                    } else {
                        for z in &s.0 { neuron_array_target.push_raw(c * channel_width, Y, *z, 1.0); }
                        for z in &s.1 { neuron_array_target.push_raw(c * channel_width + 1, Y, *z, 1.0); }
                        for z in &s.2 { neuron_array_target.push_raw(c * channel_width + 2, Y, *z, 1.0); }
                    }
                }
            }

            (ScratchSpace::D4(scratch), ScratchSpace::D4(scratch_neg)) => {
                pipelines
                    .par_iter()
                    .zip(scratch.par_iter_mut())
                    .zip(scratch_neg.par_iter_mut())
                    .try_for_each(|((pipeline, s), s_neg)| -> Result<(), FeagiDataError> {
                        if pipeline.get_last_processed_instant() < time_of_previous_burst {
                            return Ok(());
                        }
                        let data = pipeline.get_postprocessed_sensor_value();
                        if is_signed {
                            let p: SignedPercentage4D = data.try_into()?;
                            encode_signed(interpolation, &p.a, z_depth, z_depth_float, &mut s.0, &mut s_neg.0);
                            encode_signed(interpolation, &p.b, z_depth, z_depth_float, &mut s.1, &mut s_neg.1);
                            encode_signed(interpolation, &p.c, z_depth, z_depth_float, &mut s.2, &mut s_neg.2);
                            encode_signed(interpolation, &p.d, z_depth, z_depth_float, &mut s.3, &mut s_neg.3);
                        } else {
                            let p: Percentage4D = data.try_into()?;
                            encode_unsigned(interpolation, &p.a, z_depth, z_depth_float, &mut s.0);
                            encode_unsigned(interpolation, &p.b, z_depth, z_depth_float, &mut s.1);
                            encode_unsigned(interpolation, &p.c, z_depth, z_depth_float, &mut s.2);
                            encode_unsigned(interpolation, &p.d, z_depth, z_depth_float, &mut s.3);
                        }
                        Ok(())
                    })?;

                for (c, (s, s_neg)) in scratch.iter().zip(scratch_neg.iter()).enumerate() {
                    let c = c as u32;
                    const Y: u32 = 0;
                    if is_signed {
                        for z in &s.0 { neuron_array_target.push_raw(c * channel_width, Y, *z, 1.0); }
                        for z in &s_neg.0 { neuron_array_target.push_raw(c * channel_width + 1, Y, *z, 1.0); }
                        for z in &s.1 { neuron_array_target.push_raw(c * channel_width + 2, Y, *z, 1.0); }
                        for z in &s_neg.1 { neuron_array_target.push_raw(c * channel_width + 3, Y, *z, 1.0); }
                        for z in &s.2 { neuron_array_target.push_raw(c * channel_width + 4, Y, *z, 1.0); }
                        for z in &s_neg.2 { neuron_array_target.push_raw(c * channel_width + 5, Y, *z, 1.0); }
                        for z in &s.3 { neuron_array_target.push_raw(c * channel_width + 6, Y, *z, 1.0); }
                        for z in &s_neg.3 { neuron_array_target.push_raw(c * channel_width + 7, Y, *z, 1.0); }
                    } else {
                        for z in &s.0 { neuron_array_target.push_raw(c * channel_width, Y, *z, 1.0); }
                        for z in &s.1 { neuron_array_target.push_raw(c * channel_width + 1, Y, *z, 1.0); }
                        for z in &s.2 { neuron_array_target.push_raw(c * channel_width + 2, Y, *z, 1.0); }
                        for z in &s.3 { neuron_array_target.push_raw(c * channel_width + 3, Y, *z, 1.0); }
                    }
                }
            }

            _ => unreachable!("Scratch space dimension mismatch"),
        }

        Ok(())
    }
}

#[inline]
fn encode_unsigned(
    interpolation: PercentageNeuronPositioning,
    value: &Percentage,
    z_depth: u32,
    z_depth_float: f32,
    scratch: &mut Vec<u32>,
) {
    match interpolation {
        PercentageNeuronPositioning::Linear => {
            encode_unsigned_percentage_to_linear_neuron_z_index(value, z_depth_float, scratch);
        }
        PercentageNeuronPositioning::Fractional => {
            encode_unsigned_percentage_to_fractional_exponential_neuron_z_indexes(value, z_depth, scratch);
        }
    }
}

#[inline]
fn encode_signed(
    interpolation: PercentageNeuronPositioning,
    value: &SignedPercentage,
    z_depth: u32,
    z_depth_float: f32,
    scratch_pos: &mut Vec<u32>,
    scratch_neg: &mut Vec<u32>,
) {
    match interpolation {
        PercentageNeuronPositioning::Linear => {
            encode_signed_percentage_to_linear_neuron_z_index(value, z_depth_float, scratch_pos, scratch_neg);
        }
        PercentageNeuronPositioning::Fractional => {
            encode_signed_percentage_to_fractional_exponential_neuron_z_indexes(value, z_depth, scratch_pos, scratch_neg);
        }
    }
}
