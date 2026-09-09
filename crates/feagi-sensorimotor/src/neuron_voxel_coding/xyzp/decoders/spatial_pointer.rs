use crate::configuration::jsonable::JSONDecoderProperties;
use crate::data_pipeline::per_channel_stream_caches::MotorPipelineStageRunner;
use crate::data_types::descriptors::{SpatialPointerProperties, SPATIAL_POINTER_CHANNEL_WIDTH};
use crate::data_types::{Percentage, Percentage3D, SignedPercentage, SignedPercentage3D};
use crate::neuron_voxel_coding::xyzp::coder_shared_functions::{
    decode_unsigned_percentage_from_fractional_exponential_neurons,
    decode_unsigned_percentage_from_linear_neurons,
};
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPDecoder;
use crate::wrapped_io_data::WrappedIOType;
use feagi_structures::genomic::cortical_area::descriptors::CorticalChannelCount;
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, IOCorticalAreaConfigurationFlag, PercentageNeuronPositioning,
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
/// Voxel layout matches [`CartesianPosition`](feagi_structures::genomic::SensoryCorticalUnit::CartesianPosition):
/// per channel, `3×1×depth` with one X column per axis (x at X=0, y at X=1, z at X=2),
/// `Y=0`, and percentage encoded along Z.
///
/// The decoder supports two mechanisms, selected by the owning area's
/// `FrameChangeHandling` (derived from the cortical ID, not duplicated in properties):
///
/// - `Absolute`: decodes each axis via the percentage neuron layout and emits one unsigned
///   `Percentage3D` tuple (x/y/z in [0, 1]).
///
/// - `Incremental`: decodes the same unsigned position each read, maintains a per-channel
///   rolling window spanning `window_ms`, and emits the average motion (per-axis least-squares
///   velocity) as a signed `SignedPercentage3D` tuple (x/y/z in [-1, 1], `0` = no motion).
///   The velocity is scaled by `max_axis_velocity`.
#[derive(Debug)]
pub struct SpatialPointerNeuronVoxelXYZPDecoder {
    cortical_read_target: CorticalID,
    properties: SpatialPointerProperties,
    frame_change_handling: FrameChangeHandling,
    percentage_neuron_positioning: PercentageNeuronPositioning,
    /// Incremental-only configuration: rolling-window length.
    window: Duration,
    /// Incremental-only configuration: per-axis velocity that maps to encoding full scale.
    max_axis_velocity: f32,
    /// Incremental-only state: per-channel timestamped history of normalized positions.
    position_history: Vec<VecDeque<(Instant, [f32; 3])>>,
    /// Per-channel scratch: Z indexes collected for each of the three axis columns.
    z_depth_scratch: Vec<[Vec<u32>; 3]>,
}

/// Neutral signed output for incremental mode (no motion on any axis).
const INCREMENTAL_NEUTRAL: f32 = 0.0;
const ONLY_ALLOWED_Y: u32 = 0;
const AXES_PER_CHANNEL: u32 = SPATIAL_POINTER_CHANNEL_WIDTH;

/// Computes the per-axis least-squares velocity (units per second) of a series of
/// timestamped 3D points.
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
        let (frame_change_handling, percentage_neuron_positioning) =
            match cortical_read_target.extract_io_data_flag()? {
                IOCorticalAreaConfigurationFlag::Percentage3D(frame, positioning) => {
                    (frame, positioning)
                }
                IOCorticalAreaConfigurationFlag::SignedPercentage3D(frame, positioning) => {
                    (frame, positioning)
                }
                other => {
                    return Err(FeagiDataError::InternalError(format!(
                        "SpatialPointer decoder expected a Percentage3D or SignedPercentage3D \
                         cortical area flag, got {}",
                        other
                    )));
                }
            };

        let channel_count = *number_of_channels as usize;
        let (window, max_axis_velocity, position_history) = match frame_change_handling {
            FrameChangeHandling::Absolute => (Duration::ZERO, 0.0, Vec::new()),
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
            percentage_neuron_positioning,
            window,
            max_axis_velocity,
            position_history,
            z_depth_scratch: vec![[Vec::new(), Vec::new(), Vec::new()]; channel_count],
        };
        Ok(Box::new(decoder))
    }

    fn clear_scratch(&mut self) {
        for channel_scratch in self.z_depth_scratch.iter_mut() {
            for axis_scratch in channel_scratch.iter_mut() {
                axis_scratch.clear();
            }
        }
    }

    fn collect_axis_z_indexes(
        &mut self,
        neuron_array: &NeuronVoxelXYZPArrays,
        number_of_channels: u32,
    ) {
        let z_depth = self.properties.depth;
        let max_x = AXES_PER_CHANNEL * number_of_channels;

        for neuron in neuron_array.iter() {
            let nx = neuron.neuron_voxel_coordinate.x;
            let ny = neuron.neuron_voxel_coordinate.y;
            let nz = neuron.neuron_voxel_coordinate.z;

            if ny != ONLY_ALLOWED_Y || neuron.potential == 0.0 || nx >= max_x || nz >= z_depth {
                continue;
            }

            let channel_index = (nx / AXES_PER_CHANNEL) as usize;
            let axis_index = (nx % AXES_PER_CHANNEL) as usize;
            if channel_index >= self.z_depth_scratch.len() {
                continue;
            }

            self.z_depth_scratch[channel_index][axis_index].push(nz);
        }
    }

    fn decode_axis_percentage(&self, z_indexes: &Vec<u32>) -> Option<f32> {
        if z_indexes.is_empty() {
            return None;
        }

        let mut percentage = Percentage::new_zero();
        match self.percentage_neuron_positioning {
            PercentageNeuronPositioning::Linear => {
                decode_unsigned_percentage_from_linear_neurons(
                    z_indexes,
                    self.properties.depth,
                    &mut percentage,
                );
            }
            PercentageNeuronPositioning::Fractional => {
                decode_unsigned_percentage_from_fractional_exponential_neurons(
                    z_indexes,
                    &mut percentage,
                );
            }
        }
        Some(percentage.get_as_0_1())
    }

    fn channel_position(&self, channel_index: usize) -> Option<[f32; 3]> {
        let scratch = self.z_depth_scratch.get(channel_index)?;
        let x = self.decode_axis_percentage(&scratch[0])?;
        let y = self.decode_axis_percentage(&scratch[1])?;
        let z = self.decode_axis_percentage(&scratch[2])?;
        Some([x, y, z])
    }

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

    fn incremental_motion_for_channel(
        &mut self,
        channel_index: usize,
        time_of_read: Instant,
        position: [f32; 3],
    ) -> [f32; 3] {
        let history = &mut self.position_history[channel_index];
        history.push_back((time_of_read, position));

        while let Some((oldest_time, _)) = history.front() {
            if time_of_read.duration_since(*oldest_time) > self.window {
                history.pop_front();
            } else {
                break;
            }
        }

        if history.len() < 2 {
            return [INCREMENTAL_NEUTRAL; 3];
        }

        let origin = history.front().expect("history has >= 2 samples").0;
        let mut times_s = Vec::with_capacity(history.len());
        let mut points = Vec::with_capacity(history.len());
        for (sample_time, sample_position) in history.iter() {
            times_s.push(sample_time.duration_since(origin).as_secs_f32());
            points.push(*sample_position);
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
        self.clear_scratch();
        self.collect_axis_z_indexes(neuron_array, number_of_channels);

        #[allow(clippy::needless_range_loop)]
        for channel_index in 0..(number_of_channels as usize) {
            let Some(position) = self.channel_position(channel_index) else {
                continue;
            };

            match self.frame_change_handling {
                FrameChangeHandling::Absolute => {
                    let pipeline = pipelines_with_data_to_update
                        .get_mut(channel_index)
                        .expect("channel_index is within pipeline bounds");
                    Self::write_position(pipeline, position)?;
                }
                FrameChangeHandling::Incremental => {
                    let motion =
                        self.incremental_motion_for_channel(channel_index, time_of_read, position);
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
        encode_signed_velocity, regression_velocity_per_axis, SpatialPointerNeuronVoxelXYZPDecoder,
        INCREMENTAL_NEUTRAL,
    };
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

    fn pointer_cortical_id(frame: FrameChangeHandling) -> CorticalID {
        spatial_pointer_io_flag(frame, PercentageNeuronPositioning::Linear).as_io_cortical_id(
            false,
            *b"ptr",
            CorticalUnitIndex::from(0u8),
            CorticalSubUnitIndex::from(0u8),
        )
    }

    fn axis_voxel_map(id: CorticalID, axis_x: u32, z: u32) -> CorticalMappedXYZPNeuronVoxels {
        let mut arrays = NeuronVoxelXYZPArrays::new();
        arrays.push(&NeuronVoxelXYZP::new(axis_x, 0, z, 1.0));
        let mut map = CorticalMappedXYZPNeuronVoxels::new();
        map.insert(id, arrays);
        map
    }

    fn three_axis_voxel_map(
        id: CorticalID,
        x_z: u32,
        y_z: u32,
        z_z: u32,
    ) -> CorticalMappedXYZPNeuronVoxels {
        let mut arrays = NeuronVoxelXYZPArrays::new();
        arrays.push(&NeuronVoxelXYZP::new(0, 0, x_z, 1.0));
        arrays.push(&NeuronVoxelXYZP::new(1, 0, y_z, 1.0));
        arrays.push(&NeuronVoxelXYZP::new(2, 0, z_z, 1.0));
        let mut map = CorticalMappedXYZPNeuronVoxels::new();
        map.insert(id, arrays);
        map
    }

    fn read_position(pipelines: &[MotorPipelineStageRunner]) -> [f32; 3] {
        match pipelines[0].get_postprocessed_motor_value() {
            WrappedIOData::Percentage_3D(p) => {
                [p.a.get_as_0_1(), p.b.get_as_0_1(), p.c.get_as_0_1()]
            }
            other => panic!("expected Percentage_3D output, got {:?}", other),
        }
    }

    fn read_motion(pipelines: &[MotorPipelineStageRunner]) -> [f32; 3] {
        match pipelines[0].get_postprocessed_motor_value() {
            WrappedIOData::SignedPercentage_3D(p) => {
                [p.a.get_as_m1_1(), p.b.get_as_m1_1(), p.c.get_as_m1_1()]
            }
            other => panic!("expected SignedPercentage_3D output, got {:?}", other),
        }
    }

    fn one_channel_position_pipeline() -> Vec<MotorPipelineStageRunner> {
        vec![
            MotorPipelineStageRunner::new(WrappedIOData::Percentage_3D(Percentage3D::new_zero()))
                .unwrap(),
        ]
    }

    fn one_channel_motion_pipeline() -> Vec<MotorPipelineStageRunner> {
        vec![
            MotorPipelineStageRunner::new(WrappedIOData::SignedPercentage_3D(
                SignedPercentage3D::new_zero(),
            ))
            .unwrap(),
        ]
    }

    #[test]
    fn regression_returns_zero_for_insufficient_samples() {
        assert_eq!(regression_velocity_per_axis(&[], &[]), [0.0; 3]);
    }

    #[test]
    fn encode_signed_velocity_maps_neutral_and_extremes() {
        assert_eq!(encode_signed_velocity(0.0, 2.0), INCREMENTAL_NEUTRAL);
        assert_eq!(encode_signed_velocity(2.0, 2.0), 1.0);
        assert_eq!(encode_signed_velocity(-2.0, 2.0), -1.0);
    }

    #[test]
    fn rejects_non_cartesian_layout_dimensions() {
        assert!(SpatialPointerProperties::new_absolute(64, 64, 10).is_err());
        assert!(SpatialPointerProperties::new_absolute(3, 2, 10).is_err());
        assert!(SpatialPointerProperties::new_absolute(3, 1, 10).is_ok());
    }

    #[test]
    fn absolute_mode_decodes_percentage_axes_from_3x1x10_layout() {
        let id = pointer_cortical_id(FrameChangeHandling::Absolute);
        let mut decoder = SpatialPointerNeuronVoxelXYZPDecoder::new_box(
            id,
            SpatialPointerProperties::new_absolute(3, 1, 10).unwrap(),
            CorticalChannelCount::new(1).unwrap(),
        )
        .unwrap();

        // Linear decode: z=0 -> 1.0, z=9 -> 0.0 on depth 10.
        let neurons = three_axis_voxel_map(id, 0, 5, 9);
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

        assert!(changed[0]);
        let out = read_position(&pipelines);
        assert!((out[0] - 1.0).abs() < 1e-4, "x {}", out[0]);
        assert!((out[1] - (4.0 / 9.0)).abs() < 1e-4, "y {}", out[1]);
        assert!(out[2].abs() < 1e-4, "z {}", out[2]);
    }

    #[test]
    fn absolute_mode_decodes_paired_central_voxels_at_workspace_center() {
        let id = pointer_cortical_id(FrameChangeHandling::Absolute);
        let mut decoder = SpatialPointerNeuronVoxelXYZPDecoder::new_box(
            id,
            SpatialPointerProperties::new_absolute(3, 1, 10).unwrap(),
            CorticalChannelCount::new(1).unwrap(),
        )
        .unwrap();
        let mut voxels = NeuronVoxelXYZPArrays::new();
        for axis in 0..3 {
            voxels.push(&NeuronVoxelXYZP::new(axis, 0, 4, 1.0));
            voxels.push(&NeuronVoxelXYZP::new(axis, 0, 5, 1.0));
        }
        let mut neurons = CorticalMappedXYZPNeuronVoxels::new();
        neurons.insert(id, voxels);
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

        assert!(changed[0]);
        let out = read_position(&pipelines);
        assert!((out[0] - 0.5).abs() < 1e-4, "x {}", out[0]);
        assert!((out[1] - 0.5).abs() < 1e-4, "y {}", out[1]);
        assert!((out[2] - 0.5).abs() < 1e-4, "z {}", out[2]);
    }

    #[test]
    fn incremental_mode_requires_window_parameters() {
        let id = pointer_cortical_id(FrameChangeHandling::Incremental);
        let result = SpatialPointerNeuronVoxelXYZPDecoder::new_box(
            id,
            SpatialPointerProperties::new_absolute(3, 1, 10).unwrap(),
            CorticalChannelCount::new(1).unwrap(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn incremental_mode_first_read_is_neutral() {
        let id = pointer_cortical_id(FrameChangeHandling::Incremental);
        let mut decoder = SpatialPointerNeuronVoxelXYZPDecoder::new_box(
            id,
            SpatialPointerProperties::new_incremental(3, 1, 10, 1000, 4.0).unwrap(),
            CorticalChannelCount::new(1).unwrap(),
        )
        .unwrap();

        let neurons = axis_voxel_map(id, 0, 5);
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
        for axis in out {
            assert!(
                axis.abs() < 1e-6,
                "first read must be neutral, got {}",
                axis
            );
        }
    }

    #[test]
    fn incremental_mode_encodes_increasing_x_axis_as_positive_motion() {
        let id = pointer_cortical_id(FrameChangeHandling::Incremental);
        let mut decoder = SpatialPointerNeuronVoxelXYZPDecoder::new_box(
            id,
            SpatialPointerProperties::new_incremental(3, 1, 10, 1000, 4.0).unwrap(),
            CorticalChannelCount::new(1).unwrap(),
        )
        .unwrap();

        let base = Instant::now();
        let mut pipelines = one_channel_motion_pipeline();
        // X axis column (X=0): decreasing z means increasing normalized x over time.
        let x_z_values = [9u32, 8, 7, 6, 5];
        let mut out = [0.0f32; 3];
        for (frame, &x_z) in x_z_values.iter().enumerate() {
            let neurons = three_axis_voxel_map(id, x_z, 5, 5);
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
}
