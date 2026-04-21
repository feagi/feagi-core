//! Decoder for PositionalServo with both absolute and incremental cortical areas.

use crate::_compat::prelude::*;

use crate::configuration::jsonable::JSONDecoderProperties;
use crate::data_pipeline::per_channel_stream_caches::MotorPipelineStageRunner;
use crate::data_types::Percentage;
use crate::neuron_voxel_coding::xyzp::coder_shared_functions::decode_unsigned_percentage_from_linear_neurons;
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPDecoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};
use feagi_structures::genomic::cortical_area::descriptors::{CorticalChannelCount, CorticalChannelDimensions};
use crate::_compat::NeuronDepth;
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::PercentageNeuronPositioning;
use feagi_structures::genomic::cortical_area::CorticalID;
use crate::_compat::CorticalMappedXYZPNeuronVoxels;
use crate::_compat::FeagiDataError;
use std::time::Instant;

/// Decoder for PositionalServo with absolute and incremental cortical areas.
///
/// PositionalServo has two cortical areas:
/// - Area 0 (absolute): 1x1xZ - each channel has 1 neuron width for absolute position
/// - Area 1 (incremental): 2x1xZ - each channel has 2 neuron widths (forward/backward)
#[derive(Debug)]
pub struct PositionalServoNeuronVoxelXYZPDecoder {
    channel_absolute_dimensions: CorticalChannelDimensions,
    cortical_absolute_read_target: CorticalID,
    cortical_incremental_read_target: CorticalID,
    interpolation: PercentageNeuronPositioning,
    /// Scratch space for absolute area (1 per channel)
    z_depth_absolute_scratch_space: Vec<Vec<u32>>,
    /// Scratch space for incremental area (2 per channel: forward and backward)
    z_depth_incremental_forward_scratch_space: Vec<Vec<u32>>,
    z_depth_incremental_backward_scratch_space: Vec<Vec<u32>>,
}

impl PositionalServoNeuronVoxelXYZPDecoder {
    #[allow(dead_code)]
    pub fn new_box(
        absolute_cortical_id: CorticalID,
        incremental_cortical_id: CorticalID,
        z_depth: NeuronDepth,
        number_channels: CorticalChannelCount,
        interpolation: PercentageNeuronPositioning,
    ) -> Result<Box<dyn NeuronVoxelXYZPDecoder + Sync + Send>, FeagiDataError> {
        const CHANNEL_Y_HEIGHT: u32 = 1;
        const ABSOLUTE_WIDTH_PER_CHANNEL: u32 = 1;
        let absolute_total_width = number_channels.value() * ABSOLUTE_WIDTH_PER_CHANNEL;

        let decoder = PositionalServoNeuronVoxelXYZPDecoder {
            channel_absolute_dimensions: CorticalChannelDimensions::new(
                absolute_total_width,
                CHANNEL_Y_HEIGHT,
                z_depth.get(),
            )?,
            cortical_absolute_read_target: absolute_cortical_id,
            cortical_incremental_read_target: incremental_cortical_id,
            interpolation,
            z_depth_absolute_scratch_space: vec![Vec::new(); number_channels.get() as usize],
            z_depth_incremental_forward_scratch_space: vec![Vec::new(); number_channels.get() as usize],
            z_depth_incremental_backward_scratch_space: vec![Vec::new(); number_channels.get() as usize],
        };
        Ok(Box::new(decoder))
    }

    fn clear_scratch_spaces(&mut self) {
        for scratch in self.z_depth_absolute_scratch_space.iter_mut() {
            scratch.clear();
        }
        for scratch in self.z_depth_incremental_forward_scratch_space.iter_mut() {
            scratch.clear();
        }
        for scratch in self.z_depth_incremental_backward_scratch_space.iter_mut() {
            scratch.clear();
        }
    }

    fn decode_percentage(&self, z_vector: &[u32], target: &mut Percentage) {
        match self.interpolation {
            PercentageNeuronPositioning::Linear => {
                decode_unsigned_percentage_from_linear_neurons(
                    z_vector,
                    self.channel_absolute_dimensions.depth(),
                    target,
                );
            }
            PercentageNeuronPositioning::Fractional => {
                // For fractional, use the linear decoder as fallback
                // (PositionalServo typically uses Linear positioning)
                decode_unsigned_percentage_from_linear_neurons(
                    z_vector,
                    self.channel_absolute_dimensions.depth(),
                    target,
                );
            }
        }
    }
}

impl NeuronVoxelXYZPDecoder for PositionalServoNeuronVoxelXYZPDecoder {
    fn get_decodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::Percentage
    }

    fn get_as_properties(&self) -> JSONDecoderProperties {
        JSONDecoderProperties::PositionalServo(
            NeuronDepth::new(self.channel_absolute_dimensions.depth()).unwrap(),
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

        let absolute_neuron_array =
            neurons_to_read.get_neurons_of(&self.cortical_absolute_read_target);
        let incremental_neuron_array =
            neurons_to_read.get_neurons_of(&self.cortical_incremental_read_target);

        // Both arrays may be None during startup or if only one area is active
        if absolute_neuron_array.is_none() && incremental_neuron_array.is_none() {
            return Ok(());
        }

        self.clear_scratch_spaces();

        let number_of_channels = pipelines_with_data_to_update.len();
        let z_depth = self.channel_absolute_dimensions.depth();

        // Collect neurons from absolute area (1 neuron width per channel)
        if let Some(neurons) = absolute_neuron_array {
            for neuron in neurons.iter() {
                if neuron.coordinate.y != ONLY_ALLOWED_Y || neuron.potential == NeuronVoxelPotential::from(0.0f32) {
                    continue;
                }
                if neuron.coordinate.z >= z_depth {
                    continue;
                }

                let channel_index = neuron.coordinate.x as usize;
                if channel_index >= number_of_channels {
                    continue;
                }

                if let Some(scratch) = self.z_depth_absolute_scratch_space.get_mut(channel_index) {
                    scratch.push(neuron.coordinate.z);
                }
            }
        }

        // Collect neurons from incremental area (2 neuron widths per channel: even=forward, odd=backward)
        if let Some(neurons) = incremental_neuron_array {
            for neuron in neurons.iter() {
                if neuron.coordinate.y != ONLY_ALLOWED_Y || neuron.potential == NeuronVoxelPotential::from(0.0f32) {
                    continue;
                }
                if neuron.coordinate.z >= z_depth {
                    continue;
                }

                let neuron_x = neuron.coordinate.x;
                let channel_index = (neuron_x / 2) as usize;

                // DEBUG: Log first few neurons
                if channel_index < 3 {
                    eprintln!(
                        "[SERVO_DECODER] Incremental neuron: X={}, Z={}, channel={}, forward={}",
                        neuron_x,
                        neuron.coordinate.z,
                        channel_index,
                        neuron_x % 2 == 0
                    );
                }

                if channel_index >= number_of_channels {
                    continue;
                }

                let is_forward = neuron_x % 2 == 0;
                let scratch = if is_forward {
                    self.z_depth_incremental_forward_scratch_space
                        .get_mut(channel_index)
                } else {
                    self.z_depth_incremental_backward_scratch_space
                        .get_mut(channel_index)
                };

                if let Some(s) = scratch {
                    s.push(neuron.coordinate.z);
                }
            }
        }

        // Process each channel: prioritize absolute if present, otherwise use incremental
        for (channel_index, (pipeline, changed_flag)) in pipelines_with_data_to_update
            .iter_mut()
            .zip(channel_changed.iter_mut())
            .enumerate()
            .take(number_of_channels)
        {
            let forward_scratch = &self.z_depth_incremental_forward_scratch_space[channel_index];
            let backward_scratch = &self.z_depth_incremental_backward_scratch_space[channel_index];
            let absolute_scratch = &self.z_depth_absolute_scratch_space[channel_index];

            let has_incremental = !forward_scratch.is_empty() || !backward_scratch.is_empty();
            let has_absolute = !absolute_scratch.is_empty();

            if !has_incremental && !has_absolute {
                continue;
            }

            *changed_flag = true;

            let percentage: &mut Percentage =
                pipeline.get_preprocessed_cached_value_mut().try_into()?;

            // Absolute takes priority over incremental
            if has_absolute {
                eprintln!("[SERVO_DECODER] Channel {}: Using ABSOLUTE", channel_index);
                // Use absolute value directly
                self.decode_percentage(absolute_scratch, percentage);
            } else if has_incremental {
                eprintln!(
                    "[SERVO_DECODER] Channel {}: Using INCREMENTAL (fwd={}, bwd={})",
                    channel_index,
                    !forward_scratch.is_empty(),
                    !backward_scratch.is_empty()
                );
                let mut forward_value = Percentage::new_zero();
                let mut backward_value = Percentage::new_zero();

                if !forward_scratch.is_empty() {
                    self.decode_percentage(forward_scratch, &mut forward_value);
                }
                if !backward_scratch.is_empty() {
                    self.decode_percentage(backward_scratch, &mut backward_value);
                }

                // Net incremental command: forward - backward, normalized to 0-1 range
                // 0.0 = full backward, 0.5 = neutral/no movement, 1.0 = full forward
                let forward_f = forward_value.get_as_0_1();
                let backward_f = backward_value.get_as_0_1();
                let net_direction = forward_f - backward_f; // Range: -1.0 to +1.0

                // Convert to 0-1 range where 0.5 is neutral
                let output_value = ((net_direction + 1.0) / 2.0).clamp(0.0, 1.0);
                eprintln!(
                    "[SERVO_DECODER] Channel {}: fwd={:.3}, bwd={:.3}, net={:.3}, output={:.3}",
                    channel_index, forward_f, backward_f, net_direction, output_value
                );
                *percentage = Percentage::new_from_0_1(output_value)
                    .unwrap_or_else(|_| Percentage::new_from_0_1_unchecked(output_value));
            }
        }

        Ok(())
    }
}
