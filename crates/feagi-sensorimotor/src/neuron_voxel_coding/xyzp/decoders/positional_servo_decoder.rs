//! Decoder for PositionalServo with both absolute and incremental cortical areas.

use crate::configuration::jsonable::JSONDecoderProperties;
use crate::data_pipeline::per_channel_stream_caches::MotorPipelineStageRunner;
use crate::data_types::{Percentage, Percentage2D};
use crate::neuron_voxel_coding::xyzp::coder_shared_functions::decode_unsigned_percentage_from_linear_neurons;
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPDecoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelDimensions, NeuronDepth,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::PercentageNeuronPositioning;
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use feagi_structures::FeagiDataError;
use std::time::Instant;

/// Decoder for PositionalServo with absolute and incremental cortical areas.
///
/// PositionalServo has two cortical areas:
/// - Area 0 (absolute): 1x1xZ - each channel has 1 neuron width for absolute position
/// - Area 1 (incremental): 2x1xZ - each channel has 2 neuron widths (forward/backward)
///
/// Incremental commands are integrated into the current cached position so the
/// output is always an absolute target percentage. The step magnitude per tick
/// is controlled by `incremental_step_size_0_1`.
#[derive(Debug)]
pub struct PositionalServoNeuronVoxelXYZPDecoder {
    channel_absolute_dimensions: CorticalChannelDimensions,
    /// Incremental cortical area uses its own z depth (often matches absolute).
    channel_incremental_dimensions: CorticalChannelDimensions,
    cortical_absolute_read_target: CorticalID,
    cortical_incremental_read_target: CorticalID,
    interpolation: PercentageNeuronPositioning,
    /// Scratch space for absolute area (1 per channel)
    z_depth_absolute_scratch_space: Vec<Vec<u32>>,
    /// Scratch space for incremental area (2 per channel: forward and backward)
    z_depth_incremental_forward_scratch_space: Vec<Vec<u32>>,
    z_depth_incremental_backward_scratch_space: Vec<Vec<u32>>,
    /// `None` retains legacy positional-servo semantics. When present, each
    /// channel emits `(target_position, speed_limit)` percentages.
    default_speed_0_1_per_channel: Option<Vec<f32>>,
    /// Normalized incremental step applied per decode tick.
    incremental_step_size_0_1: f32,
}

/// Maximum position change per tick at full deflection, expressed as a fraction
/// of the [0, 1] range.  At `0.002`, a 360-degree servo ticking at 25 Hz sees
/// a worst-case target velocity of ~18 deg/s, which stays within typical
/// physical speed limits (20 deg/s on Lite6, for example).
const INCREMENTAL_STEP_SIZE: f32 = 0.002;

impl PositionalServoNeuronVoxelXYZPDecoder {
    #[allow(dead_code)]
    pub fn new_box(
        absolute_cortical_id: CorticalID,
        incremental_cortical_id: CorticalID,
        z_depth: NeuronDepth,
        number_channels: CorticalChannelCount,
        interpolation: PercentageNeuronPositioning,
    ) -> Result<Box<dyn NeuronVoxelXYZPDecoder + Sync + Send>, FeagiDataError> {
        Self::new_with_depths_box(
            absolute_cortical_id,
            incremental_cortical_id,
            z_depth,
            z_depth,
            number_channels,
            interpolation,
            None,
            INCREMENTAL_STEP_SIZE,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_depths_box(
        absolute_cortical_id: CorticalID,
        incremental_cortical_id: CorticalID,
        absolute_z_depth: NeuronDepth,
        incremental_z_depth: NeuronDepth,
        number_channels: CorticalChannelCount,
        interpolation: PercentageNeuronPositioning,
        default_speed_0_1_per_channel: Option<Vec<f32>>,
        incremental_step_size_0_1: f32,
    ) -> Result<Box<dyn NeuronVoxelXYZPDecoder + Sync + Send>, FeagiDataError> {
        const CHANNEL_Y_HEIGHT: u32 = 1;
        const ABSOLUTE_WIDTH_PER_CHANNEL: u32 = 1;
        const INCREMENTAL_WIDTH_PER_CHANNEL: u32 = 2;
        let decoder = PositionalServoNeuronVoxelXYZPDecoder {
            channel_absolute_dimensions: CorticalChannelDimensions::new(
                *number_channels * ABSOLUTE_WIDTH_PER_CHANNEL,
                CHANNEL_Y_HEIGHT,
                *absolute_z_depth,
            )?,
            channel_incremental_dimensions: CorticalChannelDimensions::new(
                *number_channels * INCREMENTAL_WIDTH_PER_CHANNEL,
                CHANNEL_Y_HEIGHT,
                *incremental_z_depth,
            )?,
            cortical_absolute_read_target: absolute_cortical_id,
            cortical_incremental_read_target: incremental_cortical_id,
            interpolation,
            z_depth_absolute_scratch_space: vec![Vec::new(); *number_channels as usize],
            z_depth_incremental_forward_scratch_space: vec![Vec::new(); *number_channels as usize],
            z_depth_incremental_backward_scratch_space: vec![Vec::new(); *number_channels as usize],
            default_speed_0_1_per_channel,
            incremental_step_size_0_1,
        };
        Ok(Box::new(decoder))
    }

    /// Creates the explicit target-and-speed positional-servo decoder.
    ///
    /// The caller must configure one safe default speed for every channel.
    /// No implicit speed is used when an absolute target fires alone.
    #[allow(clippy::too_many_arguments)]
    pub fn new_target_speed_box(
        absolute_cortical_id: CorticalID,
        incremental_cortical_id: CorticalID,
        absolute_z_depth: NeuronDepth,
        incremental_z_depth: NeuronDepth,
        number_channels: CorticalChannelCount,
        interpolation: PercentageNeuronPositioning,
        default_speed_0_1_per_channel: Vec<f32>,
        incremental_step_size_0_1: f32,
    ) -> Result<Box<dyn NeuronVoxelXYZPDecoder + Sync + Send>, FeagiDataError> {
        if default_speed_0_1_per_channel.len() != *number_channels as usize
            || default_speed_0_1_per_channel
                .iter()
                .any(|speed| !speed.is_finite() || !(0.0..=1.0).contains(speed))
        {
            return Err(FeagiDataError::BadParameters(
                "PositionalServoTargetSpeed requires one finite [0, 1] default speed per channel."
                    .to_string(),
            ));
        }
        if !(incremental_step_size_0_1.is_finite()
            && incremental_step_size_0_1 > 0.0
            && incremental_step_size_0_1 <= 1.0)
        {
            return Err(FeagiDataError::BadParameters(
                "PositionalServoTargetSpeed requires incremental_step_size_0_1 in (0, 1]."
                    .to_string(),
            ));
        }
        Self::new_with_depths_box(
            absolute_cortical_id,
            incremental_cortical_id,
            absolute_z_depth,
            incremental_z_depth,
            number_channels,
            interpolation,
            Some(default_speed_0_1_per_channel),
            incremental_step_size_0_1,
        )
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

    fn decode_absolute_percentage(&self, z_vector: &[u32], target: &mut Percentage) {
        Self::decode_percentage_with_depth(
            z_vector,
            self.channel_absolute_dimensions.depth,
            self.interpolation,
            target,
        );
    }

    fn decode_incremental_percentage(&self, z_vector: &[u32], target: &mut Percentage) {
        Self::decode_percentage_with_depth(
            z_vector,
            self.channel_incremental_dimensions.depth,
            self.interpolation,
            target,
        );
    }

    fn decode_percentage_with_depth(
        z_vector: &[u32],
        z_depth: u32,
        interpolation: PercentageNeuronPositioning,
        target: &mut Percentage,
    ) {
        match interpolation {
            PercentageNeuronPositioning::Linear | PercentageNeuronPositioning::Fractional => {
                decode_unsigned_percentage_from_linear_neurons(z_vector, z_depth, target);
            }
        }
    }
}

impl NeuronVoxelXYZPDecoder for PositionalServoNeuronVoxelXYZPDecoder {
    fn get_decodable_data_type(&self) -> WrappedIOType {
        if self.default_speed_0_1_per_channel.is_some() {
            WrappedIOType::Percentage_2D
        } else {
            WrappedIOType::Percentage
        }
    }

    fn get_as_properties(&self) -> JSONDecoderProperties {
        let depth = NeuronDepth::new(self.channel_absolute_dimensions.depth).unwrap();
        match &self.default_speed_0_1_per_channel {
            Some(default_speeds) => JSONDecoderProperties::PositionalServoTargetSpeed(
                depth,
                self.interpolation,
                default_speeds.clone(),
                self.incremental_step_size_0_1,
            ),
            None => JSONDecoderProperties::PositionalServo(depth, self.interpolation),
        }
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
        let z_depth = self.channel_absolute_dimensions.depth;

        // Collect neurons from absolute area (1 neuron width per channel)
        if let Some(neurons) = absolute_neuron_array {
            for neuron in neurons.iter() {
                if neuron.neuron_voxel_coordinate.y != ONLY_ALLOWED_Y || neuron.potential == 0.0 {
                    continue;
                }
                if neuron.neuron_voxel_coordinate.z >= z_depth {
                    continue;
                }

                let channel_index = neuron.neuron_voxel_coordinate.x as usize;
                if channel_index >= number_of_channels {
                    continue;
                }

                if let Some(scratch) = self.z_depth_absolute_scratch_space.get_mut(channel_index) {
                    scratch.push(neuron.neuron_voxel_coordinate.z);
                }
            }
        }

        // Collect neurons from incremental area (2 neuron widths per channel: even=forward, odd=backward)
        if let Some(neurons) = incremental_neuron_array {
            for neuron in neurons.iter() {
                if neuron.neuron_voxel_coordinate.y != ONLY_ALLOWED_Y || neuron.potential == 0.0 {
                    continue;
                }
                if neuron.neuron_voxel_coordinate.z >= z_depth {
                    continue;
                }

                let neuron_x = neuron.neuron_voxel_coordinate.x;
                let channel_index = (neuron_x / 2) as usize;

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
                    s.push(neuron.neuron_voxel_coordinate.z);
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

            let mut forward_value = Percentage::new_zero();
            let mut backward_value = Percentage::new_zero();
            if has_incremental {
                if !forward_scratch.is_empty() {
                    self.decode_incremental_percentage(forward_scratch, &mut forward_value);
                }
                if !backward_scratch.is_empty() {
                    self.decode_incremental_percentage(backward_scratch, &mut backward_value);
                }
            }

            if let Some(default_speeds) = &self.default_speed_0_1_per_channel {
                let target_speed: &mut Percentage2D =
                    pipeline.get_preprocessed_cached_value_mut().try_into()?;
                if has_absolute {
                    self.decode_absolute_percentage(absolute_scratch, &mut target_speed.a);
                } else if has_incremental {
                    // Standalone incremental activation must still nudge target.
                    // Paired behavior is unchanged because absolute takes priority.
                    let net_direction = forward_value.get_as_0_1() - backward_value.get_as_0_1();
                    let current_target = target_speed.a.get_as_0_1();
                    let new_target = (current_target
                        + net_direction * self.incremental_step_size_0_1)
                        .clamp(0.0, 1.0);
                    target_speed.a = Percentage::new_from_0_1(new_target)
                        .unwrap_or_else(|_| Percentage::new_from_0_1_unchecked(new_target));
                }
                // Speed is the unsigned [0, 1] activity on the forward incremental
                // column (x even). The backward column is legacy direction semantics
                // and must not override speed when both halves carry activity from
                // broad stimulation.
                let speed = if has_incremental {
                    if !forward_scratch.is_empty() {
                        forward_value.get_as_0_1()
                    } else {
                        backward_value.get_as_0_1()
                    }
                } else {
                    let cached_speed = target_speed.b.get_as_0_1();
                    if cached_speed > 0.0 {
                        cached_speed
                    } else {
                        default_speeds[channel_index]
                    }
                };
                target_speed.b = Percentage::new_from_0_1(speed)
                    .unwrap_or_else(|_| Percentage::new_from_0_1_unchecked(speed));
            } else if has_absolute {
                let percentage: &mut Percentage =
                    pipeline.get_preprocessed_cached_value_mut().try_into()?;
                self.decode_absolute_percentage(absolute_scratch, percentage);
            } else {
                let percentage: &mut Percentage =
                    pipeline.get_preprocessed_cached_value_mut().try_into()?;
                // net_direction: -1.0 (full backward) to +1.0 (full forward)
                let net_direction = forward_value.get_as_0_1() - backward_value.get_as_0_1();

                // Integrate the delta into the current cached position so the
                // output stays an absolute target percentage.
                let current_pos = percentage.get_as_0_1();
                let new_pos =
                    (current_pos + net_direction * self.incremental_step_size_0_1).clamp(0.0, 1.0);
                *percentage = Percentage::new_from_0_1(new_pos)
                    .unwrap_or_else(|_| Percentage::new_from_0_1_unchecked(new_pos));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_pipeline::per_channel_stream_caches::MotorPipelineStageRunner;
    use feagi_structures::genomic::cortical_area::descriptors::{
        CorticalChannelCount, CorticalSubUnitIndex, CorticalUnitIndex,
    };
    use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
        FrameChangeHandling, IOCorticalAreaConfigurationFlag, PercentageNeuronPositioning,
    };
    use feagi_structures::neuron_voxels::xyzp::{
        CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZP, NeuronVoxelXYZPArrays,
    };

    const Z_DEPTH: u32 = 10;
    const NUM_CHANNELS: u32 = 1;

    fn absolute_cortical_id() -> CorticalID {
        IOCorticalAreaConfigurationFlag::Percentage(
            FrameChangeHandling::Absolute,
            PercentageNeuronPositioning::Linear,
        )
        .as_io_cortical_id(
            false,
            *b"pse",
            CorticalUnitIndex::from(0u8),
            CorticalSubUnitIndex::from(0u8),
        )
    }

    fn incremental_cortical_id() -> CorticalID {
        IOCorticalAreaConfigurationFlag::Percentage(
            FrameChangeHandling::Incremental,
            PercentageNeuronPositioning::Linear,
        )
        .as_io_cortical_id(
            false,
            *b"pse",
            CorticalUnitIndex::from(0u8),
            CorticalSubUnitIndex::from(1u8),
        )
    }

    fn make_decoder() -> Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> {
        PositionalServoNeuronVoxelXYZPDecoder::new_box(
            absolute_cortical_id(),
            incremental_cortical_id(),
            NeuronDepth::new(Z_DEPTH).unwrap(),
            CorticalChannelCount::new(NUM_CHANNELS).unwrap(),
            PercentageNeuronPositioning::Linear,
        )
        .unwrap()
    }

    fn make_target_speed_decoder() -> Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> {
        make_target_speed_decoder_with_step(INCREMENTAL_STEP_SIZE)
    }

    fn make_target_speed_decoder_with_step(
        step_size_0_1: f32,
    ) -> Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> {
        PositionalServoNeuronVoxelXYZPDecoder::new_target_speed_box(
            absolute_cortical_id(),
            incremental_cortical_id(),
            NeuronDepth::new(Z_DEPTH).unwrap(),
            NeuronDepth::new(Z_DEPTH).unwrap(),
            CorticalChannelCount::new(NUM_CHANNELS).unwrap(),
            PercentageNeuronPositioning::Linear,
            vec![0.2],
            step_size_0_1,
        )
        .unwrap()
    }

    fn one_channel_pipeline() -> Vec<MotorPipelineStageRunner> {
        vec![MotorPipelineStageRunner::new(WrappedIOData::Percentage(
            Percentage::new_from_0_1(0.5).unwrap(),
        ))
        .unwrap()]
    }

    fn one_channel_target_speed_pipeline() -> Vec<MotorPipelineStageRunner> {
        vec![
            MotorPipelineStageRunner::new(WrappedIOData::Percentage_2D(Percentage2D::new(
                Percentage::new_from_0_1(0.5).unwrap(),
                Percentage::new_zero(),
            )))
            .unwrap(),
        ]
    }

    fn read_percentage(pipelines: &[MotorPipelineStageRunner]) -> f32 {
        match pipelines[0].get_postprocessed_motor_value() {
            WrappedIOData::Percentage(p) => p.get_as_0_1(),
            other => panic!("expected Percentage, got {:?}", other),
        }
    }

    fn read_target_speed(pipelines: &[MotorPipelineStageRunner]) -> (f32, f32) {
        match pipelines[0].get_postprocessed_motor_value() {
            WrappedIOData::Percentage_2D(value) => (value.a.get_as_0_1(), value.b.get_as_0_1()),
            other => panic!("expected Percentage_2D, got {:?}", other),
        }
    }

    fn make_neuron_map(
        id: CorticalID,
        neurons: &[(u32, u32, u32)],
    ) -> CorticalMappedXYZPNeuronVoxels {
        let mut arrays = NeuronVoxelXYZPArrays::new();
        for &(x, y, z) in neurons {
            arrays.push(&NeuronVoxelXYZP::new(x, y, z, 1.0));
        }
        let mut map = CorticalMappedXYZPNeuronVoxels::new();
        map.insert(id, arrays);
        map
    }

    fn decode(
        decoder: &mut Box<dyn NeuronVoxelXYZPDecoder + Sync + Send>,
        neurons: &CorticalMappedXYZPNeuronVoxels,
        pipelines: &mut Vec<MotorPipelineStageRunner>,
    ) -> Vec<bool> {
        let mut changed = vec![false; pipelines.len()];
        decoder
            .read_neuron_data_multi_channel_into_pipeline_input_cache(
                neurons,
                Instant::now(),
                pipelines,
                &mut changed,
            )
            .unwrap();
        changed
    }

    // -----------------------------------------------------------------------
    // Absolute-area tests
    // -----------------------------------------------------------------------

    #[test]
    fn absolute_sets_position_directly() {
        let mut decoder = make_decoder();
        let mut pipelines = one_channel_pipeline();
        // z encoding: percentage = 1.0 - z/z_depth.  z=1 -> 0.9
        let neurons = make_neuron_map(absolute_cortical_id(), &[(0, 0, 1)]);
        let changed = decode(&mut decoder, &neurons, &mut pipelines);

        assert!(changed[0], "channel must be marked changed");
        let pos = read_percentage(&pipelines);
        assert!(
            (pos - 0.9).abs() < 0.02,
            "absolute neuron at z=1 must set position near 0.9, got {pos}"
        );
    }

    #[test]
    fn absolute_overrides_start_position() {
        let mut decoder = make_decoder();
        let mut pipelines = one_channel_pipeline();
        assert!(
            (read_percentage(&pipelines) - 0.5).abs() < 1e-6,
            "initial position must be 0.5"
        );

        // z=8 -> percentage = 1.0 - 8/10 = 0.2
        let neurons = make_neuron_map(absolute_cortical_id(), &[(0, 0, 8)]);
        decode(&mut decoder, &neurons, &mut pipelines);

        let pos = read_percentage(&pipelines);
        assert!(
            pos < 0.25,
            "absolute neuron at z=8 must set position near 0.2, got {pos}"
        );
    }

    // -----------------------------------------------------------------------
    // CRITICAL SAFETY TESTS: incremental output must be bounded
    // -----------------------------------------------------------------------

    #[test]
    fn incremental_forward_produces_small_positive_delta() {
        let mut decoder = make_decoder();
        let mut pipelines = one_channel_pipeline();
        let start = read_percentage(&pipelines);

        // Forward neuron (x=0) at z=0 (maximum signal: percentage = 1.0 - 0/10 = 1.0)
        let neurons = make_neuron_map(incremental_cortical_id(), &[(0, 0, 0)]);
        let changed = decode(&mut decoder, &neurons, &mut pipelines);

        assert!(changed[0]);
        let after = read_percentage(&pipelines);
        let delta = after - start;
        assert!(
            delta > 0.0,
            "forward incremental must increase position, delta={delta}"
        );
        assert!(
            delta <= INCREMENTAL_STEP_SIZE + 1e-6,
            "single-tick delta ({delta}) must not exceed INCREMENTAL_STEP_SIZE ({INCREMENTAL_STEP_SIZE})"
        );
    }

    #[test]
    fn incremental_backward_produces_small_negative_delta() {
        let mut decoder = make_decoder();
        let mut pipelines = one_channel_pipeline();
        let start = read_percentage(&pipelines);

        // Backward neuron (x=1) at z=0 (maximum signal)
        let neurons = make_neuron_map(incremental_cortical_id(), &[(1, 0, 0)]);
        let changed = decode(&mut decoder, &neurons, &mut pipelines);

        assert!(changed[0]);
        let after = read_percentage(&pipelines);
        let delta = after - start;
        assert!(
            delta < 0.0,
            "backward incremental must decrease position, delta={delta}"
        );
        assert!(
            delta.abs() <= INCREMENTAL_STEP_SIZE + 1e-6,
            "single-tick |delta| ({}) must not exceed INCREMENTAL_STEP_SIZE ({INCREMENTAL_STEP_SIZE})",
            delta.abs()
        );
    }

    /// THE KEY SAFETY TEST: even at maximum neuron activation, a single
    /// incremental tick must NEVER jump more than INCREMENTAL_STEP_SIZE.
    /// The pre-fix decoder violated this, producing jumps of up to 0.5.
    #[test]
    fn incremental_full_forward_never_exceeds_step_size() {
        let mut decoder = make_decoder();
        let mut pipelines = one_channel_pipeline();

        // Single forward neuron at z=0 gives percentage = 1.0 (maximum signal)
        let neurons = make_neuron_map(incremental_cortical_id(), &[(0, 0, 0)]);

        let before = read_percentage(&pipelines);
        decode(&mut decoder, &neurons, &mut pipelines);
        let after = read_percentage(&pipelines);
        let delta = (after - before).abs();

        assert!(
            delta <= INCREMENTAL_STEP_SIZE + 1e-6,
            "SAFETY: full-forward single-tick delta ({delta:.4}) exceeds \
             INCREMENTAL_STEP_SIZE ({INCREMENTAL_STEP_SIZE}). This was the root cause \
             of the xARM servo snapping to extremes."
        );
    }

    /// Symmetric check for full backward.
    #[test]
    fn incremental_full_backward_never_exceeds_step_size() {
        let mut decoder = make_decoder();
        let mut pipelines = one_channel_pipeline();

        // Single backward neuron at z=0 gives percentage = 1.0 (maximum signal)
        let neurons = make_neuron_map(incremental_cortical_id(), &[(1, 0, 0)]);

        let before = read_percentage(&pipelines);
        decode(&mut decoder, &neurons, &mut pipelines);
        let after = read_percentage(&pipelines);
        let delta = (after - before).abs();

        assert!(
            delta <= INCREMENTAL_STEP_SIZE + 1e-6,
            "SAFETY: full-backward single-tick delta ({delta:.4}) exceeds \
             INCREMENTAL_STEP_SIZE ({INCREMENTAL_STEP_SIZE})"
        );
    }

    // -----------------------------------------------------------------------
    // Incremental accumulation and clamping
    // -----------------------------------------------------------------------

    #[test]
    fn incremental_accumulates_over_multiple_ticks() {
        let mut decoder = make_decoder();
        let mut pipelines = one_channel_pipeline();
        let start = read_percentage(&pipelines);

        // z=0 gives maximum forward signal (percentage=1.0), delta=STEP_SIZE per tick
        let neurons = make_neuron_map(incremental_cortical_id(), &[(0, 0, 0)]);
        for _ in 0..5 {
            decode(&mut decoder, &neurons, &mut pipelines);
        }

        let after = read_percentage(&pipelines);
        let total_delta = after - start;
        assert!(
            total_delta > INCREMENTAL_STEP_SIZE,
            "5 forward ticks must accumulate beyond a single step ({total_delta})"
        );
        assert!(
            total_delta <= 5.0 * INCREMENTAL_STEP_SIZE + 1e-5,
            "5 ticks must not exceed 5 * step_size ({total_delta})"
        );
    }

    #[test]
    fn incremental_clamps_at_upper_bound() {
        let mut decoder = make_decoder();
        let mut pipelines = one_channel_pipeline();

        let neurons = make_neuron_map(incremental_cortical_id(), &[(0, 0, 0)]);
        // At step=0.002, need 250+ ticks to travel from 0.5 to 1.0
        for _ in 0..500 {
            decode(&mut decoder, &neurons, &mut pipelines);
        }

        let pos = read_percentage(&pipelines);
        assert!(pos <= 1.0, "position must clamp at 1.0, got {pos}");
        assert!(
            pos > 0.99,
            "500 forward ticks from 0.5 must reach near 1.0, got {pos}"
        );
    }

    #[test]
    fn incremental_clamps_at_lower_bound() {
        let mut decoder = make_decoder();
        let mut pipelines = one_channel_pipeline();

        let neurons = make_neuron_map(incremental_cortical_id(), &[(1, 0, 0)]);
        for _ in 0..500 {
            decode(&mut decoder, &neurons, &mut pipelines);
        }

        let pos = read_percentage(&pipelines);
        assert!(pos >= 0.0, "position must clamp at 0.0, got {pos}");
        assert!(
            pos < 0.01,
            "500 backward ticks from 0.5 must reach near 0.0, got {pos}"
        );
    }

    // -----------------------------------------------------------------------
    // Absolute takes priority when both areas fire
    // -----------------------------------------------------------------------

    #[test]
    fn absolute_takes_priority_over_incremental() {
        let mut decoder = make_decoder();
        let mut pipelines = one_channel_pipeline();

        // Fire both: absolute at z=8 (low position: 1-8/10 = 0.2) and full forward incremental z=0
        let mut map = CorticalMappedXYZPNeuronVoxels::new();
        let mut abs_arr = NeuronVoxelXYZPArrays::new();
        abs_arr.push(&NeuronVoxelXYZP::new(0, 0, 8, 1.0));
        map.insert(absolute_cortical_id(), abs_arr);
        let mut inc_arr = NeuronVoxelXYZPArrays::new();
        inc_arr.push(&NeuronVoxelXYZP::new(0, 0, 0, 1.0));
        map.insert(incremental_cortical_id(), inc_arr);

        decode(&mut decoder, &map, &mut pipelines);
        let pos = read_percentage(&pipelines);

        assert!(
            pos < 0.25,
            "absolute at z=8 (0.2) must win over forward incremental, got {pos}"
        );
    }

    #[test]
    fn target_speed_mode_uses_absolute_target_and_incremental_speed() {
        let mut decoder = make_target_speed_decoder();
        let mut pipelines = one_channel_target_speed_pipeline();
        let mut map = CorticalMappedXYZPNeuronVoxels::new();
        let mut absolute = NeuronVoxelXYZPArrays::new();
        absolute.push(&NeuronVoxelXYZP::new(0, 0, 8, 1.0));
        map.insert(absolute_cortical_id(), absolute);
        let mut incremental = NeuronVoxelXYZPArrays::new();
        incremental.push(&NeuronVoxelXYZP::new(0, 0, 0, 1.0));
        map.insert(incremental_cortical_id(), incremental);

        decode(&mut decoder, &map, &mut pipelines);

        let (target, speed) = read_target_speed(&pipelines);
        assert!(
            target < 0.25,
            "absolute target must be retained, got {target}"
        );
        assert!(
            (speed - 1.0).abs() < 1e-6,
            "full incremental activation must produce full speed, got {speed}"
        );
    }

    #[test]
    fn target_speed_mode_uses_configured_speed_for_absolute_only_command() {
        let mut decoder = make_target_speed_decoder();
        let mut pipelines = one_channel_target_speed_pipeline();
        let neurons = make_neuron_map(absolute_cortical_id(), &[(0, 0, 1)]);

        decode(&mut decoder, &neurons, &mut pipelines);

        let (target, speed) = read_target_speed(&pipelines);
        assert!(
            target > 0.85,
            "absolute target must be decoded, got {target}"
        );
        assert!(
            (speed - 0.2).abs() < 1e-6,
            "absolute-only command must use configured speed, got {speed}"
        );
    }

    #[test]
    fn target_speed_incremental_only_updates_target_and_speed() {
        let mut decoder = make_target_speed_decoder();
        let mut pipelines = one_channel_target_speed_pipeline();
        let start_target = read_target_speed(&pipelines).0;
        assert!(
            (start_target - 0.5).abs() < 1e-6,
            "initial target must be 0.5"
        );

        let neurons = make_neuron_map(incremental_cortical_id(), &[(0, 0, 0)]);
        decode(&mut decoder, &neurons, &mut pipelines);

        let (target, speed) = read_target_speed(&pipelines);
        assert!(
            target > start_target,
            "incremental-only forward activation must increase target, start={start_target}, after={target}"
        );
        assert!(
            (target - (start_target + INCREMENTAL_STEP_SIZE)).abs() < 1e-6,
            "incremental-only target delta must be one step size, got {target}"
        );
        assert!(
            (speed - 1.0).abs() < 1e-6,
            "full forward incremental must set full speed, got {speed}"
        );
    }

    #[test]
    fn target_speed_uses_configured_incremental_step_size() {
        let custom_step = 0.01_f32;
        let mut decoder = make_target_speed_decoder_with_step(custom_step);
        let mut pipelines = one_channel_target_speed_pipeline();
        let start_target = read_target_speed(&pipelines).0;
        let neurons = make_neuron_map(incremental_cortical_id(), &[(0, 0, 0)]);

        decode(&mut decoder, &neurons, &mut pipelines);
        let (target, _speed) = read_target_speed(&pipelines);
        assert!(
            (target - (start_target + custom_step)).abs() < 1e-6,
            "incremental-only target delta must match configured step size"
        );
    }

    #[test]
    fn target_speed_incremental_partial_activation_maps_to_speed_scalar() {
        let mut decoder = make_target_speed_decoder();
        let mut pipelines = one_channel_target_speed_pipeline();
        // z=7 with depth 10 -> 1.0 - 7/9 ~= 0.22
        let neurons = make_neuron_map(incremental_cortical_id(), &[(0, 0, 7)]);
        decode(&mut decoder, &neurons, &mut pipelines);

        let start_target = 0.5_f32;
        let (target, speed) = read_target_speed(&pipelines);
        assert!(
            target > start_target,
            "partial incremental activation must nudge target upward, got {target}"
        );
        assert!(
            speed > 0.15 && speed < 0.3,
            "partial incremental activation must map directly to speed, got {speed}"
        );
    }

    #[test]
    fn target_speed_incremental_z9_is_min_speed_at_depth_10() {
        let mut decoder = make_target_speed_decoder();
        let mut pipelines = one_channel_target_speed_pipeline();
        let neurons = make_neuron_map(incremental_cortical_id(), &[(0, 0, 9)]);
        decode(&mut decoder, &neurons, &mut pipelines);

        let (target, speed) = read_target_speed(&pipelines);
        assert!(
            (target - 0.5).abs() < 1e-6,
            "z=9 should decode to near-zero incremental influence on target, got {target}"
        );
        assert!(
            speed < 0.05,
            "forward z=9 at depth 10 must decode to near-zero speed, got {speed}"
        );
    }

    #[test]
    fn target_speed_forward_column_wins_over_backward_for_speed() {
        let mut decoder = make_target_speed_decoder();
        let mut pipelines = one_channel_target_speed_pipeline();
        let mut map = CorticalMappedXYZPNeuronVoxels::new();
        let mut incremental = NeuronVoxelXYZPArrays::new();
        // Forward z=7 -> ~0.22 speed; backward z=0 -> full speed.
        incremental.push(&NeuronVoxelXYZP::new(0, 0, 7, 1.0));
        incremental.push(&NeuronVoxelXYZP::new(1, 0, 0, 1.0));
        map.insert(incremental_cortical_id(), incremental);

        decode(&mut decoder, &map, &mut pipelines);

        let (target, speed) = read_target_speed(&pipelines);
        assert!(
            target < 0.5,
            "backward-heavy incremental activity must nudge target backward, got {target}"
        );
        assert!(
            speed > 0.15 && speed < 0.3,
            "forward column must set speed even when backward is fully active, got {speed}"
        );
    }

    #[test]
    fn target_speed_mode_preserves_incremental_speed_on_absolute_only_command() {
        let mut decoder = make_target_speed_decoder();
        let mut pipelines = one_channel_target_speed_pipeline();

        let incremental = make_neuron_map(incremental_cortical_id(), &[(0, 0, 7)]);
        decode(&mut decoder, &incremental, &mut pipelines);
        let established_speed = read_target_speed(&pipelines).1;
        assert!(
            established_speed > 0.15 && established_speed < 0.3,
            "incremental must establish speed first, got {established_speed}"
        );

        let absolute = make_neuron_map(absolute_cortical_id(), &[(0, 0, 1)]);
        decode(&mut decoder, &absolute, &mut pipelines);

        let (target, speed) = read_target_speed(&pipelines);
        assert!(
            target > 0.85,
            "absolute target must be decoded, got {target}"
        );
        assert!(
            (speed - established_speed).abs() < 1e-6,
            "absolute-only must preserve prior incremental speed, got {speed}"
        );
    }

    // -----------------------------------------------------------------------
    // No-activity tick must NOT change position
    // -----------------------------------------------------------------------

    #[test]
    fn no_neurons_does_not_change_position() {
        let mut decoder = make_decoder();
        let mut pipelines = one_channel_pipeline();
        let before = read_percentage(&pipelines);

        let empty_map = CorticalMappedXYZPNeuronVoxels::new();
        let changed = decode(&mut decoder, &empty_map, &mut pipelines);

        assert!(!changed[0], "no-activity tick must not flag changed");
        let after = read_percentage(&pipelines);
        assert!(
            (after - before).abs() < 1e-6,
            "no-activity must preserve position"
        );
    }
}
