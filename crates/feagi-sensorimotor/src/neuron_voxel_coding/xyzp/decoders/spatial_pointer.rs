use crate::configuration::jsonable::JSONDecoderProperties;
use crate::data_pipeline::per_channel_stream_caches::MotorPipelineStageRunner;
use crate::data_types::descriptors::{
    SpatialPointerProperties, SPATIAL_POINTER_INCREMENTAL_CHANNEL_WIDTH,
};
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
use std::time::Instant;

/// Decodes one cortical area's activity into a normalized XYZ spatial pointer.
///
/// Voxel layout:
///
/// - `Absolute`: `3×1×depth` (x at X=0, y at X=1, z at X=2), unsigned [0, 1].
/// - `Incremental`: `6×1×depth` like PositionalServo incremental — X+/X−, Y+/Y−,
///   Z+/Z− (even X = positive, odd X = negative). Linear Z encoding uses low Z
///   as the large increment and high Z as the small increment.
///
/// The decoder supports two mechanisms, selected by the owning area's
/// `FrameChangeHandling` (derived from the cortical ID, not duplicated in properties):
///
/// - `Absolute`: decodes each axis via the percentage neuron layout and emits one unsigned
///   `Percentage3D` tuple (x/y/z in [0, 1]). All three axes must fire in the same burst;
///   a partial XYZ is not a pose.
///
/// - `Incremental`: decodes each firing polarity column as an unsigned magnitude and
///   emits a signed `SignedPercentage3D` velocity (x/y/z in [-1, 1]). One column is
///   enough. Silent axes emit `0.0`.
#[derive(Debug)]
pub struct SpatialPointerNeuronVoxelXYZPDecoder {
    cortical_read_target: CorticalID,
    properties: SpatialPointerProperties,
    frame_change_handling: FrameChangeHandling,
    percentage_neuron_positioning: PercentageNeuronPositioning,
    /// Per-channel scratch: Z indexes for up to six incremental polarity columns.
    z_depth_scratch: Vec<[Vec<u32>; 6]>,
}

const ONLY_ALLOWED_Y: u32 = 0;
const INCREMENTAL_COLUMNS: u32 = SPATIAL_POINTER_INCREMENTAL_CHANNEL_WIDTH;

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
        if frame_change_handling == FrameChangeHandling::Incremental {
            properties.require_incremental_parameters()?;
        }

        let decoder = SpatialPointerNeuronVoxelXYZPDecoder {
            cortical_read_target,
            properties,
            frame_change_handling,
            percentage_neuron_positioning,
            z_depth_scratch: vec![
                [
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ];
                channel_count
            ],
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
        let columns = self.column_count();
        let max_x = columns * number_of_channels;

        for neuron in neuron_array.iter() {
            let nx = neuron.neuron_voxel_coordinate.x;
            let ny = neuron.neuron_voxel_coordinate.y;
            let nz = neuron.neuron_voxel_coordinate.z;

            if ny != ONLY_ALLOWED_Y || neuron.potential == 0.0 || nx >= max_x || nz >= z_depth {
                continue;
            }

            let channel_index = (nx / columns) as usize;
            let column_index = (nx % columns) as usize;
            if channel_index >= self.z_depth_scratch.len() {
                continue;
            }

            self.z_depth_scratch[channel_index][column_index].push(nz);
        }
    }

    fn column_count(&self) -> u32 {
        match self.frame_change_handling {
            FrameChangeHandling::Absolute => self.properties.width,
            FrameChangeHandling::Incremental => INCREMENTAL_COLUMNS,
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
        if self.properties.width == 1 {
            return Some([x, 0.0, 0.0]);
        }
        let y = self.decode_axis_percentage(&scratch[1])?;
        let z = self.decode_axis_percentage(&scratch[2])?;
        Some([x, y, z])
    }

    /// Incremental velocity for one channel.
    ///
    /// Each axis is two polarity columns (even +, odd −). Magnitude is the
    /// unsigned linear Z decode (low Z = large step). Silent axes stay at `0.0`.
    fn channel_incremental_motion(&self, channel_index: usize) -> Option<[f32; 3]> {
        let scratch = self.z_depth_scratch.get(channel_index)?;
        let mut motion = [0.0_f32; 3];
        let mut any = false;
        for axis in 0..3 {
            let positive = self.decode_axis_percentage(&scratch[axis * 2]);
            let negative = self.decode_axis_percentage(&scratch[axis * 2 + 1]);
            if positive.is_none() && negative.is_none() {
                continue;
            }
            any = true;
            motion[axis] = (positive.unwrap_or(0.0) - negative.unwrap_or(0.0)).clamp(-1.0, 1.0);
        }
        if any {
            Some(motion)
        } else {
            None
        }
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
        _time_of_read: Instant,
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
            match self.frame_change_handling {
                FrameChangeHandling::Absolute => {
                    let Some(position) = self.channel_position(channel_index) else {
                        continue;
                    };
                    let pipeline = pipelines_with_data_to_update
                        .get_mut(channel_index)
                        .expect("channel_index is within pipeline bounds");
                    Self::write_position(pipeline, position)?;
                }
                FrameChangeHandling::Incremental => {
                    let Some(motion) = self.channel_incremental_motion(channel_index) else {
                        continue;
                    };
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
    use std::time::Instant;

    fn pointer_cortical_id(frame: FrameChangeHandling) -> CorticalID {
        spatial_pointer_io_flag(frame, PercentageNeuronPositioning::Linear).as_io_cortical_id(
            false,
            *b"ptr",
            CorticalUnitIndex::from(0u8),
            CorticalSubUnitIndex::from(0u8),
        )
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
    fn rejects_non_cartesian_layout_dimensions() {
        assert!(SpatialPointerProperties::new_absolute(64, 64, 10).is_err());
        assert!(SpatialPointerProperties::new_absolute(3, 2, 10).is_err());
        assert!(SpatialPointerProperties::new_absolute(3, 1, 10).is_ok());
        assert!(SpatialPointerProperties::new_absolute(1, 1, 10).is_ok());
        assert!(SpatialPointerProperties::new_incremental(3, 1, 10, 1000, 4.0).is_err());
        assert!(SpatialPointerProperties::new_incremental(6, 1, 10, 1000, 4.0).is_ok());
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
    fn speed_mode_decodes_single_unsigned_column() {
        let id = pointer_cortical_id(FrameChangeHandling::Absolute);
        let mut decoder = SpatialPointerNeuronVoxelXYZPDecoder::new_box(
            id,
            SpatialPointerProperties::new_absolute(1, 1, 10).unwrap(),
            CorticalChannelCount::new(1).unwrap(),
        )
        .unwrap();
        let mut arrays = NeuronVoxelXYZPArrays::new();
        arrays.push(&NeuronVoxelXYZP::new(0, 0, 0, 1.0));
        let mut neurons = CorticalMappedXYZPNeuronVoxels::new();
        neurons.insert(id, arrays);
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
        assert!((out[0] - 1.0).abs() < 1e-4, "speed {}", out[0]);
        assert!(out[1].abs() < 1e-4);
        assert!(out[2].abs() < 1e-4);
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
    fn incremental_mode_maps_polarity_columns_and_inverted_z() {
        let id = pointer_cortical_id(FrameChangeHandling::Incremental);
        let mut decoder = SpatialPointerNeuronVoxelXYZPDecoder::new_box(
            id,
            SpatialPointerProperties::new_incremental(6, 1, 10, 1000, 4.0).unwrap(),
            CorticalChannelCount::new(1).unwrap(),
        )
        .unwrap();

        // X+ at z=0 (large +), Y- at x=3 z=0 (large -), Z+ at x=4 z=9 (small +).
        let mut arrays = NeuronVoxelXYZPArrays::new();
        arrays.push(&NeuronVoxelXYZP::new(0, 0, 0, 1.0));
        arrays.push(&NeuronVoxelXYZP::new(3, 0, 0, 1.0));
        arrays.push(&NeuronVoxelXYZP::new(4, 0, 9, 1.0));
        let mut neurons = CorticalMappedXYZPNeuronVoxels::new();
        neurons.insert(id, arrays);
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
        assert!((out[0] - 1.0).abs() < 1e-4, "x+ {}", out[0]);
        assert!((out[1] + 1.0).abs() < 1e-4, "y- {}", out[1]);
        assert!(out[2].abs() < 1e-4, "z small {}", out[2]);
    }

    #[test]
    fn incremental_mode_low_z_is_larger_than_high_z() {
        let id = pointer_cortical_id(FrameChangeHandling::Incremental);
        let mut decoder = SpatialPointerNeuronVoxelXYZPDecoder::new_box(
            id,
            SpatialPointerProperties::new_incremental(6, 1, 10, 1000, 4.0).unwrap(),
            CorticalChannelCount::new(1).unwrap(),
        )
        .unwrap();

        let mut pipelines = one_channel_motion_pipeline();
        let mut changed = vec![false];
        decoder
            .read_neuron_data_multi_channel_into_pipeline_input_cache(
                &one_axis_voxel_map(id, 0, 8),
                Instant::now(),
                &mut pipelines,
                &mut changed,
            )
            .unwrap();
        let high_z = read_motion(&pipelines)[0];

        decoder
            .read_neuron_data_multi_channel_into_pipeline_input_cache(
                &one_axis_voxel_map(id, 0, 0),
                Instant::now(),
                &mut pipelines,
                &mut changed,
            )
            .unwrap();
        let low_z = read_motion(&pipelines)[0];

        assert!(low_z > high_z);
        assert!((low_z - 1.0).abs() < 1e-4);
    }

    fn one_axis_voxel_map(id: CorticalID, axis: u32, z: u32) -> CorticalMappedXYZPNeuronVoxels {
        let mut arrays = NeuronVoxelXYZPArrays::new();
        arrays.push(&NeuronVoxelXYZP::new(axis, 0, z, 1.0));
        let mut map = CorticalMappedXYZPNeuronVoxels::new();
        map.insert(id, arrays);
        map
    }

    #[test]
    fn absolute_mode_requires_all_three_axes() {
        let id = pointer_cortical_id(FrameChangeHandling::Absolute);
        let mut decoder = SpatialPointerNeuronVoxelXYZPDecoder::new_box(
            id,
            SpatialPointerProperties::new_absolute(3, 1, 10).unwrap(),
            CorticalChannelCount::new(1).unwrap(),
        )
        .unwrap();
        let mut pipelines = one_channel_position_pipeline();
        let mut changed = vec![false];
        decoder
            .read_neuron_data_multi_channel_into_pipeline_input_cache(
                &one_axis_voxel_map(id, 0, 0),
                Instant::now(),
                &mut pipelines,
                &mut changed,
            )
            .unwrap();
        assert!(!changed[0]);
        let out = read_position(&pipelines);
        assert!(out[0].abs() < 1e-4);
        assert!(out[1].abs() < 1e-4);
        assert!(out[2].abs() < 1e-4);
    }

    #[test]
    fn incremental_mode_accepts_single_axis_activation() {
        let id = pointer_cortical_id(FrameChangeHandling::Incremental);
        let mut decoder = SpatialPointerNeuronVoxelXYZPDecoder::new_box(
            id,
            SpatialPointerProperties::new_incremental(6, 1, 10, 1000, 4.0).unwrap(),
            CorticalChannelCount::new(1).unwrap(),
        )
        .unwrap();
        let mut pipelines = one_channel_motion_pipeline();
        let mut changed = vec![false];
        decoder
            .read_neuron_data_multi_channel_into_pipeline_input_cache(
                &one_axis_voxel_map(id, 0, 0),
                Instant::now(),
                &mut pipelines,
                &mut changed,
            )
            .unwrap();
        assert!(changed[0]);
        let out = read_motion(&pipelines);
        assert!((out[0] - 1.0).abs() < 1e-4, "x {}", out[0]);
        assert!(out[1].abs() < 1e-4, "y {}", out[1]);
        assert!(out[2].abs() < 1e-4, "z {}", out[2]);
    }
}
