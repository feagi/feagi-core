use crate::configuration::jsonable::JSONDecoderProperties;
use crate::data_pipeline::per_channel_stream_caches::MotorPipelineStageRunner;
use crate::data_types::descriptors::SpatialPointerProperties;
use crate::data_types::{Percentage, Percentage3D, SignedPercentage, SignedPercentage3D};
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPDecoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};
use feagi_structures::genomic::cortical_area::descriptors::CorticalChannelCount;
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, IOCorticalAreaConfigurationFlag,
};
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::neuron_voxels::xyzp::{
    CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZPArrays,
};
use feagi_structures::FeagiDataError;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Decodes one cortical area's activity into a normalized XYZ spatial pointer.
///
/// The decoder supports two mechanisms, selected by the owning area's
/// `FrameChangeHandling` (derived from the cortical ID, not duplicated in properties):
///
/// - `Absolute`: emits the PSP-weighted centroid of all active voxels as one unsigned
///   `Percentage3D` tuple (x/y/z in [0, 1] over the channel-local grid). For depth=1, z=0.
///
/// - `Incremental`: maintains a per-channel rolling window of recent centroids spanning
///   `window_ms` and emits the average motion (per-axis least-squares velocity) of the
///   centroid over that window as a signed `SignedPercentage3D` tuple (x/y/z in [-1, 1],
///   `0` = no motion). The velocity is scaled by `max_axis_velocity` (the velocity that
///   maps to full scale +/-1.0). This drives a robot arm in the direction the activity is
///   moving rather than to an absolute position.
///
/// The decoded output type therefore depends on the mode: `Percentage3D` for Absolute and
/// `SignedPercentage3D` for Incremental, matching the area's declared configuration flag
/// (`Percentage3D` vs `SignedPercentage3D`).
#[derive(Debug)]
pub struct SpatialPointerNeuronVoxelXYZPDecoder {
    cortical_read_target: CorticalID,
    properties: SpatialPointerProperties,
    frame_change_handling: FrameChangeHandling,
    /// Incremental-only configuration: rolling-window length.
    window: Duration,
    /// Incremental-only configuration: per-axis velocity that maps to encoding full scale.
    max_axis_velocity: f32,
    /// Incremental-only state: per-channel timestamped history of normalized centroids.
    centroid_history: Vec<VecDeque<(Instant, [f32; 3])>>,
}

/// Neutral signed output for incremental mode (no motion on any axis).
const INCREMENTAL_NEUTRAL: f32 = 0.0;

#[inline]
fn normalize_weighted_axis(weighted_sum: f32, total_weight: f32, axis_size: u32) -> f32 {
    if axis_size <= 1 || total_weight <= 0.0 {
        return 0.0;
    }
    let max_index = (axis_size - 1) as f32;
    ((weighted_sum / total_weight) / max_index).clamp(0.0, 1.0)
}

/// Computes the per-axis least-squares velocity (units per second) of a series of
/// timestamped 3D points.
///
/// This fits an independent straight line to each axis against time and returns the
/// slopes. A least-squares fit is used instead of an endpoint difference so that
/// per-frame jitter in the source signal (e.g. a camera tracker) is averaged out and the
/// resulting direction is stable across the window.
///
/// `times_s` holds sample times in seconds relative to an arbitrary origin; `points` is
/// the aligned series of `[x, y, z]` values. Returns `[0.0; 3]` when fewer than two
/// samples are present or when all timestamps coincide (no resolvable time base).
fn regression_velocity_per_axis(times_s: &[f32], points: &[[f32; 3]]) -> [f32; 3] {
    let n = times_s.len();
    if n < 2 || n != points.len() {
        return [0.0; 3];
    }

    let count = n as f32;
    let sum_t: f32 = times_s.iter().sum();
    let sum_tt: f32 = times_s.iter().map(|t| t * t).sum();
    let denominator = count * sum_tt - sum_t * sum_t;
    if denominator.abs() <= f32::EPSILON {
        return [0.0; 3];
    }

    let mut velocity = [0.0f32; 3];
    for axis in 0..3 {
        let sum_v: f32 = points.iter().map(|p| p[axis]).sum();
        let sum_tv: f32 = times_s
            .iter()
            .zip(points.iter())
            .map(|(t, p)| t * p[axis])
            .sum();
        velocity[axis] = (count * sum_tv - sum_t * sum_v) / denominator;
    }
    velocity
}

/// Maps a signed velocity (units/sec) to the signed [-1, 1] output range: `0.0` means no
/// motion, `+max_axis_velocity` maps to `+1.0` and `-max_axis_velocity` maps to `-1.0`.
/// Values beyond full scale are clamped.
#[inline]
fn encode_signed_velocity(velocity: f32, max_axis_velocity: f32) -> f32 {
    (velocity / max_axis_velocity).clamp(-1.0, 1.0)
}

impl SpatialPointerNeuronVoxelXYZPDecoder {
    pub fn new_box(
        cortical_read_target: CorticalID,
        properties: SpatialPointerProperties,
        number_of_channels: CorticalChannelCount,
    ) -> Result<Box<dyn NeuronVoxelXYZPDecoder + Sync + Send>, FeagiDataError> {
        // The decode mechanism is encoded in the cortical ID rather than duplicated in
        // the decoder properties, keeping a single source of truth for the area's mode.
        // Absolute areas are flagged Percentage3D (unsigned position); Incremental areas
        // are flagged SignedPercentage3D (signed motion).
        let frame_change_handling = match cortical_read_target.extract_io_data_flag()? {
            IOCorticalAreaConfigurationFlag::Percentage3D(frame, _) => frame,
            IOCorticalAreaConfigurationFlag::SignedPercentage3D(frame, _) => frame,
            other => {
                return Err(FeagiDataError::InternalError(format!(
                    "SpatialPointer decoder expected a Percentage3D or SignedPercentage3D \
                     cortical area flag, got {}",
                    other
                )));
            }
        };

        let channel_count = *number_of_channels as usize;
        let (window, max_axis_velocity, centroid_history) = match frame_change_handling {
            FrameChangeHandling::Absolute => {
                // Absolute mode keeps no temporal state.
                (Duration::ZERO, 0.0, Vec::new())
            }
            FrameChangeHandling::Incremental => {
                let (window_ms, max_axis_velocity) = properties.require_incremental_parameters()?;
                (
                    Duration::from_millis(window_ms as u64),
                    max_axis_velocity,
                    vec![VecDeque::new(); channel_count],
                )
            }
        };

        let decoder = SpatialPointerNeuronVoxelXYZPDecoder {
            cortical_read_target,
            properties,
            frame_change_handling,
            window,
            max_axis_velocity,
            centroid_history,
        };
        Ok(Box::new(decoder))
    }

    /// Computes the PSP-weighted centroid of a single channel as normalized [0, 1] per
    /// axis, returning `None` when the channel has no active voxels this read.
    fn channel_centroid(
        &self,
        neuron_array: &NeuronVoxelXYZPArrays,
        channel_index: usize,
        number_of_channels: u32,
    ) -> Option<[f32; 3]> {
        let per_channel_width = self.properties.width;
        let max_possible_x = per_channel_width * number_of_channels;
        let max_possible_y = self.properties.height;
        let max_possible_z = self.properties.depth;

        let x_offset = (channel_index as u32) * per_channel_width;
        let x_end = x_offset + per_channel_width;

        let mut weighted_x = 0.0f32;
        let mut weighted_y = 0.0f32;
        let mut weighted_z = 0.0f32;
        let mut total_weight = 0.0f32;

        for neuron in neuron_array.iter() {
            let nx = neuron.neuron_voxel_coordinate.x;
            let ny = neuron.neuron_voxel_coordinate.y;
            let nz = neuron.neuron_voxel_coordinate.z;

            if nx >= max_possible_x || ny >= max_possible_y || nz >= max_possible_z {
                continue;
            }
            if nx < x_offset || nx >= x_end {
                continue;
            }

            let weight = neuron.potential.abs();
            if weight <= 0.0 {
                continue;
            }

            weighted_x += (nx - x_offset) as f32 * weight;
            weighted_y += ny as f32 * weight;
            weighted_z += nz as f32 * weight;
            total_weight += weight;
        }

        if total_weight <= 0.0 {
            return None;
        }

        Some([
            normalize_weighted_axis(weighted_x, total_weight, self.properties.width),
            normalize_weighted_axis(weighted_y, total_weight, self.properties.height),
            normalize_weighted_axis(weighted_z, total_weight, self.properties.depth),
        ])
    }

    /// Writes an unsigned position centroid (Absolute mode) into the channel's
    /// `Percentage3D` output slot.
    fn write_position(
        pipeline: &mut MotorPipelineStageRunner,
        position: [f32; 3],
    ) -> Result<(), FeagiDataError> {
        let pointer: &mut Percentage3D = pipeline.get_preprocessed_cached_value_mut().try_into()?;
        pointer.a = Percentage::new_from_0_1(position[0].clamp(0.0, 1.0))
            .expect("position x is clamped to [0, 1]");
        pointer.b = Percentage::new_from_0_1(position[1].clamp(0.0, 1.0))
            .expect("position y is clamped to [0, 1]");
        pointer.c = Percentage::new_from_0_1(position[2].clamp(0.0, 1.0))
            .expect("position z is clamped to [0, 1]");
        Ok(())
    }

    /// Writes a signed motion vector (Incremental mode) into the channel's
    /// `SignedPercentage3D` output slot.
    fn write_motion(
        pipeline: &mut MotorPipelineStageRunner,
        motion: [f32; 3],
    ) -> Result<(), FeagiDataError> {
        let vector: &mut SignedPercentage3D =
            pipeline.get_preprocessed_cached_value_mut().try_into()?;
        vector.a = SignedPercentage::new_from_m1_1(motion[0].clamp(-1.0, 1.0))
            .expect("motion x is clamped to [-1, 1]");
        vector.b = SignedPercentage::new_from_m1_1(motion[1].clamp(-1.0, 1.0))
            .expect("motion y is clamped to [-1, 1]");
        vector.c = SignedPercentage::new_from_m1_1(motion[2].clamp(-1.0, 1.0))
            .expect("motion z is clamped to [-1, 1]");
        Ok(())
    }

    /// Pushes the latest centroid, evicts samples older than the window, and returns the
    /// signed motion vector ([-1, 1] per axis, 0 = no motion) for the channel.
    fn incremental_motion_for_channel(
        &mut self,
        channel_index: usize,
        time_of_read: Instant,
        centroid: [f32; 3],
    ) -> [f32; 3] {
        let history = &mut self.centroid_history[channel_index];
        history.push_back((time_of_read, centroid));

        // Evict samples that have aged out of the rolling window.
        while let Some((oldest_time, _)) = history.front() {
            if time_of_read.duration_since(*oldest_time) > self.window {
                history.pop_front();
            } else {
                break;
            }
        }

        if history.len() < 2 {
            // Not enough temporal evidence to resolve motion: emit the neutral command.
            return [INCREMENTAL_NEUTRAL; 3];
        }

        // Use the oldest retained sample as the time origin for numerical stability.
        let origin = history.front().expect("history has >= 2 samples").0;
        let mut times_s = Vec::with_capacity(history.len());
        let mut points = Vec::with_capacity(history.len());
        for (sample_time, sample_centroid) in history.iter() {
            times_s.push(sample_time.duration_since(origin).as_secs_f32());
            points.push(*sample_centroid);
        }

        let velocity = regression_velocity_per_axis(&times_s, &points);
        [
            encode_signed_velocity(velocity[0], self.max_axis_velocity),
            encode_signed_velocity(velocity[1], self.max_axis_velocity),
            encode_signed_velocity(velocity[2], self.max_axis_velocity),
        ]
    }
}

impl NeuronVoxelXYZPDecoder for SpatialPointerNeuronVoxelXYZPDecoder {
    fn get_decodable_data_type(&self) -> WrappedIOType {
        match self.frame_change_handling {
            FrameChangeHandling::Absolute => WrappedIOType::Percentage_3D,
            FrameChangeHandling::Incremental => WrappedIOType::SignedPercentage_3D,
        }
    }

    fn get_as_properties(&self) -> JSONDecoderProperties {
        JSONDecoderProperties::SpatialPointer(self.properties)
    }

    fn read_neuron_data_multi_channel_into_pipeline_input_cache(
        &mut self,
        neurons_to_read: &CorticalMappedXYZPNeuronVoxels,
        time_of_read: Instant,
        pipelines_with_data_to_update: &mut Vec<MotorPipelineStageRunner>,
        channel_changed: &mut Vec<bool>,
    ) -> Result<(), FeagiDataError> {
        let neuron_array = neurons_to_read.get_neurons_of(&self.cortical_read_target);
        let Some(neuron_array) = neuron_array else {
            return Ok(());
        };
        if neuron_array.is_empty() {
            return Ok(());
        }

        let number_of_channels = pipelines_with_data_to_update.len() as u32;

        // Index-based loop is intentional: the body interleaves an immutable borrow of
        // `self` (centroid), a mutable borrow of `self` (incremental window), and
        // independent indexed access into both `pipelines_with_data_to_update` and
        // `channel_changed`, which an iterator-with-enumerate cannot express cleanly.
        #[allow(clippy::needless_range_loop)]
        for channel_index in 0..(number_of_channels as usize) {
            let Some(centroid) =
                self.channel_centroid(neuron_array, channel_index, number_of_channels)
            else {
                continue;
            };

            match self.frame_change_handling {
                FrameChangeHandling::Absolute => {
                    let pipeline = pipelines_with_data_to_update
                        .get_mut(channel_index)
                        .expect("channel_index is within pipeline bounds");
                    Self::write_position(pipeline, centroid)?;
                }
                FrameChangeHandling::Incremental => {
                    // Compute motion first (mutably borrows self) before borrowing the pipeline.
                    let motion =
                        self.incremental_motion_for_channel(channel_index, time_of_read, centroid);
                    let pipeline = pipelines_with_data_to_update
                        .get_mut(channel_index)
                        .expect("channel_index is within pipeline bounds");
                    Self::write_motion(pipeline, motion)?;
                }
            }
            channel_changed[channel_index] = true;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        encode_signed_velocity, normalize_weighted_axis, regression_velocity_per_axis,
        INCREMENTAL_NEUTRAL,
    };

    #[test]
    fn normalize_axis_returns_zero_for_single_dimension() {
        let val = normalize_weighted_axis(42.0, 2.0, 1);
        assert_eq!(val, 0.0);
    }

    #[test]
    fn normalize_axis_maps_to_expected_fraction() {
        // 64-length axis means max index is 63. Index 12.6 should normalize to 0.2.
        let weighted_sum = 126.0;
        let total_weight = 10.0;
        let val = normalize_weighted_axis(weighted_sum, total_weight, 64);
        assert!((val - 0.2).abs() < 1e-6);
    }

    #[test]
    fn normalize_axis_clamps_upper_bound() {
        let val = normalize_weighted_axis(9999.0, 1.0, 64);
        assert_eq!(val, 1.0);
    }

    #[test]
    fn regression_returns_zero_for_insufficient_samples() {
        assert_eq!(regression_velocity_per_axis(&[], &[]), [0.0; 3]);
        assert_eq!(
            regression_velocity_per_axis(&[0.0], &[[0.1, 0.2, 0.3]]),
            [0.0; 3]
        );
    }

    #[test]
    fn regression_returns_zero_when_all_times_equal() {
        let times = [1.0, 1.0, 1.0];
        let points = [[0.0, 0.0, 0.0], [0.5, 0.5, 0.5], [1.0, 1.0, 1.0]];
        assert_eq!(regression_velocity_per_axis(&times, &points), [0.0; 3]);
    }

    #[test]
    fn regression_recovers_known_slope_and_sign() {
        // x increases at +0.2/s (left-to-right sweep), y decreases at -0.1/s, z is flat.
        let times = [0.0, 1.0, 2.0, 3.0];
        let points = [
            [0.10, 0.40, 0.50],
            [0.30, 0.30, 0.50],
            [0.50, 0.20, 0.50],
            [0.70, 0.10, 0.50],
        ];
        let velocity = regression_velocity_per_axis(&times, &points);
        assert!(
            (velocity[0] - 0.2).abs() < 1e-5,
            "x velocity {}",
            velocity[0]
        );
        assert!(
            (velocity[1] + 0.1).abs() < 1e-5,
            "y velocity {}",
            velocity[1]
        );
        assert!(velocity[2].abs() < 1e-5, "z velocity {}", velocity[2]);
    }

    #[test]
    fn regression_is_robust_to_jitter_on_monotonic_trend() {
        // A clean +0.2/s trend with alternating per-sample jitter still yields a
        // positive, roughly correct slope (endpoint methods would be far noisier).
        let times = [0.0, 1.0, 2.0, 3.0, 4.0];
        let points = [
            [0.10, 0.0, 0.0],
            [0.34, 0.0, 0.0],
            [0.48, 0.0, 0.0],
            [0.72, 0.0, 0.0],
            [0.88, 0.0, 0.0],
        ];
        let velocity = regression_velocity_per_axis(&times, &points);
        assert!(
            velocity[0] > 0.15 && velocity[0] < 0.25,
            "x velocity {}",
            velocity[0]
        );
    }

    #[test]
    fn encode_signed_velocity_maps_neutral_and_extremes() {
        // No motion maps to the signed neutral (0.0).
        assert_eq!(encode_signed_velocity(0.0, 2.0), INCREMENTAL_NEUTRAL);
        // Full scale maps to +/-1.0.
        assert_eq!(encode_signed_velocity(2.0, 2.0), 1.0);
        assert_eq!(encode_signed_velocity(-2.0, 2.0), -1.0);
        // Beyond full scale clamps.
        assert_eq!(encode_signed_velocity(10.0, 2.0), 1.0);
        assert_eq!(encode_signed_velocity(-10.0, 2.0), -1.0);
        // Half scale maps linearly to +/-0.5.
        assert!((encode_signed_velocity(1.0, 2.0) - 0.5).abs() < 1e-6);
        assert!((encode_signed_velocity(-1.0, 2.0) + 0.5).abs() < 1e-6);
    }

    //region Full decode-path tests

    use super::SpatialPointerNeuronVoxelXYZPDecoder;
    use crate::data_pipeline::per_channel_stream_caches::MotorPipelineStageRunner;
    use crate::data_types::descriptors::SpatialPointerProperties;
    use crate::data_types::{Percentage3D, SignedPercentage3D};
    use crate::wrapped_io_data::WrappedIOData;
    use feagi_structures::genomic::cortical_area::descriptors::{
        CorticalChannelCount, CorticalSubUnitIndex, CorticalUnitIndex,
    };
    use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
        spatial_pointer_io_flag, FrameChangeHandling, PercentageNeuronPositioning,
    };
    use feagi_structures::genomic::cortical_area::CorticalID;
    use feagi_structures::neuron_voxels::xyzp::{
        CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZP, NeuronVoxelXYZPArrays,
    };
    use std::time::{Duration, Instant};

    /// Builds the SpatialPointer cortical ID for the given frame-change mode, matching how
    /// the genome encodes the area's mechanism: Absolute -> `Percentage3D` (unsigned
    /// position), Incremental -> `SignedPercentage3D` (signed motion).
    fn pointer_cortical_id(frame: FrameChangeHandling) -> CorticalID {
        spatial_pointer_io_flag(frame, PercentageNeuronPositioning::Linear).as_io_cortical_id(
            false,
            *b"ptr",
            CorticalUnitIndex::from(0u8),
            CorticalSubUnitIndex::from(0u8),
        )
    }

    /// Wraps a single active voxel at (x, y, z) into a one-channel neuron map for `id`.
    fn single_voxel_map(id: CorticalID, x: u32, y: u32, z: u32) -> CorticalMappedXYZPNeuronVoxels {
        let mut arrays = NeuronVoxelXYZPArrays::new();
        arrays.push(&NeuronVoxelXYZP::new(x, y, z, 1.0));
        let mut map = CorticalMappedXYZPNeuronVoxels::new();
        map.insert(id, arrays);
        map
    }

    /// Reads back an Absolute-mode position tuple ([0, 1] per axis) as [x, y, z].
    fn read_position(pipelines: &[MotorPipelineStageRunner]) -> [f32; 3] {
        match pipelines[0].get_postprocessed_motor_value() {
            WrappedIOData::Percentage_3D(p) => {
                [p.a.get_as_0_1(), p.b.get_as_0_1(), p.c.get_as_0_1()]
            }
            other => panic!("expected Percentage_3D output, got {:?}", other),
        }
    }

    /// Reads back an Incremental-mode signed motion tuple ([-1, 1] per axis) as [x, y, z].
    fn read_motion(pipelines: &[MotorPipelineStageRunner]) -> [f32; 3] {
        match pipelines[0].get_postprocessed_motor_value() {
            WrappedIOData::SignedPercentage_3D(p) => {
                [p.a.get_as_m1_1(), p.b.get_as_m1_1(), p.c.get_as_m1_1()]
            }
            other => panic!("expected SignedPercentage_3D output, got {:?}", other),
        }
    }

    /// One-channel pipeline seeded with an unsigned `Percentage3D` slot (Absolute mode).
    fn one_channel_position_pipeline() -> Vec<MotorPipelineStageRunner> {
        vec![
            MotorPipelineStageRunner::new(WrappedIOData::Percentage_3D(Percentage3D::new_zero()))
                .unwrap(),
        ]
    }

    /// One-channel pipeline seeded with a signed `SignedPercentage3D` slot (Incremental mode).
    fn one_channel_motion_pipeline() -> Vec<MotorPipelineStageRunner> {
        vec![
            MotorPipelineStageRunner::new(WrappedIOData::SignedPercentage_3D(
                SignedPercentage3D::new_zero(),
            ))
            .unwrap(),
        ]
    }

    #[test]
    fn absolute_mode_emits_centroid_of_active_voxels() {
        let id = pointer_cortical_id(FrameChangeHandling::Absolute);
        let mut decoder = SpatialPointerNeuronVoxelXYZPDecoder::new_box(
            id,
            SpatialPointerProperties::new_absolute(64, 64, 1).unwrap(),
            CorticalChannelCount::new(1).unwrap(),
        )
        .unwrap();

        // Single voxel at x=31, y=15 over a 64x64 grid (max index 63).
        let neurons = single_voxel_map(id, 31, 15, 0);
        let mut pipelines = one_channel_position_pipeline();
        let mut changed = vec![false];
        decoder
            .read_neuron_data_multi_channel_into_pipeline_input_cache(
                &neurons,
                Instant::now(),
                &mut pipelines,
                &mut changed,
            )
            .unwrap();

        assert!(
            changed[0],
            "absolute decode must mark the channel as changed"
        );
        let out = read_position(&pipelines);
        assert!((out[0] - 31.0 / 63.0).abs() < 1e-4, "x centroid {}", out[0]);
        assert!((out[1] - 15.0 / 63.0).abs() < 1e-4, "y centroid {}", out[1]);
        assert!(
            out[2].abs() < 1e-6,
            "z must be 0 for depth=1, got {}",
            out[2]
        );
    }

    #[test]
    fn incremental_mode_requires_window_parameters() {
        let id = pointer_cortical_id(FrameChangeHandling::Incremental);
        // Absolute-only properties (no window config) must be rejected for an
        // Incremental area rather than silently defaulting.
        let result = SpatialPointerNeuronVoxelXYZPDecoder::new_box(
            id,
            SpatialPointerProperties::new_absolute(64, 64, 1).unwrap(),
            CorticalChannelCount::new(1).unwrap(),
        );
        assert!(
            result.is_err(),
            "incremental area without window config must error"
        );
    }

    #[test]
    fn incremental_mode_first_read_is_neutral() {
        let id = pointer_cortical_id(FrameChangeHandling::Incremental);
        let mut decoder = SpatialPointerNeuronVoxelXYZPDecoder::new_box(
            id,
            SpatialPointerProperties::new_incremental(64, 64, 1, 1000, 4.0).unwrap(),
            CorticalChannelCount::new(1).unwrap(),
        )
        .unwrap();

        let neurons = single_voxel_map(id, 10, 10, 0);
        let mut pipelines = one_channel_motion_pipeline();
        let mut changed = vec![false];
        decoder
            .read_neuron_data_multi_channel_into_pipeline_input_cache(
                &neurons,
                Instant::now(),
                &mut pipelines,
                &mut changed,
            )
            .unwrap();

        let out = read_motion(&pipelines);
        // A single sample cannot resolve motion: every axis must be neutral (0.0).
        for axis in out {
            assert!(
                axis.abs() < 1e-6,
                "first read must be neutral (no motion), got {}",
                axis
            );
        }
    }

    #[test]
    fn incremental_mode_encodes_left_to_right_sweep_as_positive_x_motion() {
        let id = pointer_cortical_id(FrameChangeHandling::Incremental);
        let mut decoder = SpatialPointerNeuronVoxelXYZPDecoder::new_box(
            id,
            SpatialPointerProperties::new_incremental(64, 64, 1, 1000, 4.0).unwrap(),
            CorticalChannelCount::new(1).unwrap(),
        )
        .unwrap();

        let base = Instant::now();
        let mut pipelines = one_channel_motion_pipeline();
        // Activity centroid sweeps +x (left to right) at a fixed y; 5 frames, 50ms apart.
        let xs = [6u32, 12, 18, 24, 30];
        let mut out = [0.0f32; 3];
        for (frame, &x) in xs.iter().enumerate() {
            let neurons = single_voxel_map(id, x, 10, 0);
            let mut changed = vec![false];
            decoder
                .read_neuron_data_multi_channel_into_pipeline_input_cache(
                    &neurons,
                    base + Duration::from_millis(50 * frame as u64),
                    &mut pipelines,
                    &mut changed,
                )
                .unwrap();
            assert!(
                changed[0],
                "active channel must be marked changed each frame"
            );
            out = read_motion(&pipelines);
        }

        // A rightward sweep must encode as positive x motion (> neutral 0.0),
        // with no net motion on the (constant) y axis or the flat z axis.
        assert!(out[0] > 0.05, "expected positive x motion, got {}", out[0]);
        assert!(
            out[1].abs() < 1e-3,
            "y motion must be neutral, got {}",
            out[1]
        );
        assert!(
            out[2].abs() < 1e-3,
            "z motion must be neutral, got {}",
            out[2]
        );
    }

    #[test]
    fn incremental_mode_encodes_right_to_left_sweep_as_negative_x_motion() {
        let id = pointer_cortical_id(FrameChangeHandling::Incremental);
        let mut decoder = SpatialPointerNeuronVoxelXYZPDecoder::new_box(
            id,
            SpatialPointerProperties::new_incremental(64, 64, 1, 1000, 4.0).unwrap(),
            CorticalChannelCount::new(1).unwrap(),
        )
        .unwrap();

        let base = Instant::now();
        let mut pipelines = one_channel_motion_pipeline();
        let xs = [30u32, 24, 18, 12, 6];
        let mut out = [0.0f32; 3];
        for (frame, &x) in xs.iter().enumerate() {
            let neurons = single_voxel_map(id, x, 10, 0);
            let mut changed = vec![false];
            decoder
                .read_neuron_data_multi_channel_into_pipeline_input_cache(
                    &neurons,
                    base + Duration::from_millis(50 * frame as u64),
                    &mut pipelines,
                    &mut changed,
                )
                .unwrap();
            out = read_motion(&pipelines);
        }

        assert!(out[0] < -0.05, "expected negative x motion, got {}", out[0]);
    }

    //endregion
}
