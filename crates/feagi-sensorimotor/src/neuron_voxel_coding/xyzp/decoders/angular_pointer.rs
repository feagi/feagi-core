use crate::configuration::jsonable::JSONDecoderProperties;
use crate::data_pipeline::per_channel_stream_caches::MotorPipelineStageRunner;
use crate::data_types::descriptors::{
    AngularPointerProperties, ANGULAR_POINTER_INCREMENTAL_CHANNEL_WIDTH,
};
use crate::data_types::{Percentage, SignedPercentage, SignedPercentage3D};
use crate::neuron_voxel_coding::xyzp::coder_shared_functions::{
    decode_signed_percentage_from_linear_neurons_along_z,
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

/// Decodes one cortical area's activity into a signed yaw/pitch/roll pointer.
///
/// Voxel layout:
///
/// - `Absolute`: `3×1×depth` (yaw at X=0, pitch at X=1, roll at X=2). Linear Z
///   encoding maps low Z to `+1` and high Z to `-1`. All three axes must fire
///   in the same burst; a partial attitude is not a pose.
/// - `Incremental`: `6×1×depth` — yaw+/yaw−, pitch+/pitch−, roll+/roll−
///   (even X = positive, odd X = negative). One column is enough. Silent
///   axes emit `0.0`.
///
/// Both modes emit `SignedPercentage3D` (each axis in `[-1, 1]`, `0` = center
/// / no motion). Controllers map `[-1, 1]` to a declared angle range.
#[derive(Debug)]
pub struct AngularPointerNeuronVoxelXYZPDecoder {
    cortical_read_target: CorticalID,
    properties: AngularPointerProperties,
    frame_change_handling: FrameChangeHandling,
    percentage_neuron_positioning: PercentageNeuronPositioning,
    /// Per-channel scratch: Z indexes for up to six incremental polarity columns.
    z_depth_scratch: Vec<[Vec<u32>; 6]>,
}

const ONLY_ALLOWED_Y: u32 = 0;
const INCREMENTAL_COLUMNS: u32 = ANGULAR_POINTER_INCREMENTAL_CHANNEL_WIDTH;

impl AngularPointerNeuronVoxelXYZPDecoder {
    pub fn new_box(
        cortical_read_target: CorticalID,
        properties: AngularPointerProperties,
        number_of_channels: CorticalChannelCount,
    ) -> Result<Box<dyn NeuronVoxelXYZPDecoder + Sync + Send>, FeagiDataError> {
        let (frame_change_handling, percentage_neuron_positioning) =
            match cortical_read_target.extract_io_data_flag()? {
                IOCorticalAreaConfigurationFlag::SignedPercentage3D(frame, positioning) => {
                    (frame, positioning)
                }
                other => {
                    return Err(FeagiDataError::InternalError(format!(
                        "AngularPointer decoder expected a SignedPercentage3D \
                         cortical area flag, got {}",
                        other
                    )));
                }
            };

        let channel_count = *number_of_channels as usize;
        if frame_change_handling == FrameChangeHandling::Incremental {
            properties.require_incremental_parameters()?;
        }

        let decoder = AngularPointerNeuronVoxelXYZPDecoder {
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

    fn decode_unsigned_axis_percentage(&self, z_indexes: &Vec<u32>) -> Option<f32> {
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

    fn decode_signed_axis(&self, z_indexes: &Vec<u32>) -> Option<f32> {
        if z_indexes.is_empty() {
            return None;
        }
        let mut signed = SignedPercentage::new_from_m1_1_unchecked(0.0);
        match self.percentage_neuron_positioning {
            PercentageNeuronPositioning::Linear => {
                decode_signed_percentage_from_linear_neurons_along_z(
                    z_indexes,
                    self.properties.depth,
                    &mut signed,
                );
            }
            PercentageNeuronPositioning::Fractional => {
                // Single-column Absolute cannot use pos/neg fractional lobes.
                // Map the unsigned fractional magnitude onto [-1, 1] around center.
                let unsigned = self.decode_unsigned_axis_percentage(z_indexes)?;
                signed.inplace_update_unchecked((unsigned * 2.0 - 1.0).clamp(-1.0, 1.0));
            }
        }
        Some(signed.get_as_m1_1())
    }

    fn channel_attitude(&self, channel_index: usize) -> Option<[f32; 3]> {
        let scratch = self.z_depth_scratch.get(channel_index)?;
        let yaw = self.decode_signed_axis(&scratch[0])?;
        let pitch = self.decode_signed_axis(&scratch[1])?;
        let roll = self.decode_signed_axis(&scratch[2])?;
        Some([yaw, pitch, roll])
    }

    /// Incremental angular rate for one channel.
    ///
    /// Each axis is two polarity columns (even +, odd −). Magnitude is the
    /// unsigned linear Z decode (low Z = large step). Silent axes stay at `0.0`.
    fn channel_incremental_rate(&self, channel_index: usize) -> Option<[f32; 3]> {
        let scratch = self.z_depth_scratch.get(channel_index)?;
        let mut rate = [0.0_f32; 3];
        let mut any = false;
        for axis in 0..3 {
            let positive = self.decode_unsigned_axis_percentage(&scratch[axis * 2]);
            let negative = self.decode_unsigned_axis_percentage(&scratch[axis * 2 + 1]);
            if positive.is_none() && negative.is_none() {
                continue;
            }
            any = true;
            rate[axis] = (positive.unwrap_or(0.0) - negative.unwrap_or(0.0)).clamp(-1.0, 1.0);
        }
        if any {
            Some(rate)
        } else {
            None
        }
    }

    fn write_signed(
        pipeline: &mut MotorPipelineStageRunner,
        values: [f32; 3],
    ) -> Result<(), FeagiDataError> {
        let vector: &mut SignedPercentage3D =
            pipeline.get_preprocessed_cached_value_mut().try_into()?;
        vector.a = SignedPercentage::new_from_m1_1(values[0].clamp(-1.0, 1.0))
            .expect("yaw is clamped to [-1, 1]");
        vector.b = SignedPercentage::new_from_m1_1(values[1].clamp(-1.0, 1.0))
            .expect("pitch is clamped to [-1, 1]");
        vector.c = SignedPercentage::new_from_m1_1(values[2].clamp(-1.0, 1.0))
            .expect("roll is clamped to [-1, 1]");
        Ok(())
    }
}

impl NeuronVoxelXYZPDecoder for AngularPointerNeuronVoxelXYZPDecoder {
    fn get_decodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::SignedPercentage_3D
    }

    fn get_as_properties(&self) -> JSONDecoderProperties {
        JSONDecoderProperties::AngularPointer(self.properties)
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
            let values = match self.frame_change_handling {
                FrameChangeHandling::Absolute => self.channel_attitude(channel_index),
                FrameChangeHandling::Incremental => self.channel_incremental_rate(channel_index),
            };
            let Some(values) = values else {
                continue;
            };
            let pipeline = pipelines_with_data_to_update
                .get_mut(channel_index)
                .expect("channel_index is within pipeline bounds");
            Self::write_signed(pipeline, values)?;
            channel_changed[channel_index] = true;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::AngularPointerNeuronVoxelXYZPDecoder;
    use crate::data_pipeline::per_channel_stream_caches::MotorPipelineStageRunner;
    use crate::data_types::descriptors::AngularPointerProperties;
    use crate::data_types::SignedPercentage3D;
    use crate::wrapped_io_data::WrappedIOData;
    use feagi_structures::genomic::cortical_area::descriptors::{
        CorticalChannelCount, CorticalSubUnitIndex, CorticalUnitIndex,
    };
    use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
        angular_pointer_io_flag, FrameChangeHandling, PercentageNeuronPositioning,
    };
    use feagi_structures::genomic::cortical_area::CorticalID;
    use feagi_structures::neuron_voxels::xyzp::{
        CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZP, NeuronVoxelXYZPArrays,
    };
    use std::time::Instant;

    fn pointer_cortical_id(frame: FrameChangeHandling) -> CorticalID {
        angular_pointer_io_flag(frame, PercentageNeuronPositioning::Linear).as_io_cortical_id(
            false,
            *b"ang",
            CorticalUnitIndex::from(0u8),
            CorticalSubUnitIndex::from(0u8),
        )
    }

    fn three_axis_voxel_map(
        id: CorticalID,
        yaw_z: u32,
        pitch_z: u32,
        roll_z: u32,
    ) -> CorticalMappedXYZPNeuronVoxels {
        let mut arrays = NeuronVoxelXYZPArrays::new();
        arrays.push(&NeuronVoxelXYZP::new(0, 0, yaw_z, 1.0));
        arrays.push(&NeuronVoxelXYZP::new(1, 0, pitch_z, 1.0));
        arrays.push(&NeuronVoxelXYZP::new(2, 0, roll_z, 1.0));
        let mut map = CorticalMappedXYZPNeuronVoxels::new();
        map.insert(id, arrays);
        map
    }

    fn one_axis_voxel_map(id: CorticalID, axis: u32, z: u32) -> CorticalMappedXYZPNeuronVoxels {
        let mut arrays = NeuronVoxelXYZPArrays::new();
        arrays.push(&NeuronVoxelXYZP::new(axis, 0, z, 1.0));
        let mut map = CorticalMappedXYZPNeuronVoxels::new();
        map.insert(id, arrays);
        map
    }

    fn read_signed(pipelines: &[MotorPipelineStageRunner]) -> [f32; 3] {
        match pipelines[0].get_postprocessed_motor_value() {
            WrappedIOData::SignedPercentage_3D(p) => {
                [p.a.get_as_m1_1(), p.b.get_as_m1_1(), p.c.get_as_m1_1()]
            }
            other => panic!("expected SignedPercentage_3D output, got {:?}", other),
        }
    }

    fn one_channel_pipeline() -> Vec<MotorPipelineStageRunner> {
        vec![
            MotorPipelineStageRunner::new(WrappedIOData::SignedPercentage_3D(
                SignedPercentage3D::new_zero(),
            ))
            .unwrap(),
        ]
    }

    #[test]
    fn rejects_non_attitude_layout_dimensions() {
        assert!(AngularPointerProperties::new_absolute(64, 64, 10).is_err());
        assert!(AngularPointerProperties::new_absolute(3, 2, 10).is_err());
        assert!(AngularPointerProperties::new_absolute(1, 1, 10).is_err());
        assert!(AngularPointerProperties::new_absolute(3, 1, 10).is_ok());
        assert!(AngularPointerProperties::new_incremental(3, 1, 10, 1000).is_err());
        assert!(AngularPointerProperties::new_incremental(6, 1, 10, 1000).is_ok());
    }

    #[test]
    fn absolute_mode_decodes_signed_along_z() {
        let id = pointer_cortical_id(FrameChangeHandling::Absolute);
        let mut decoder = AngularPointerNeuronVoxelXYZPDecoder::new_box(
            id,
            AngularPointerProperties::new_absolute(3, 1, 10).unwrap(),
            CorticalChannelCount::new(1).unwrap(),
        )
        .unwrap();

        // Linear along-Z: z=0 -> +1.0, z=max -> -1.0. Depth 10 has max index 9,
        // so z=4 maps to 1 - 2*(4/9), not workspace center.
        let neurons = three_axis_voxel_map(id, 0, 4, 9);
        let mut pipelines = one_channel_pipeline();
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
        let out = read_signed(&pipelines);
        let pitch_z4 = 1.0 - 2.0 * (4.0 / 9.0);
        assert!((out[0] - 1.0).abs() < 1e-4, "yaw {}", out[0]);
        assert!((out[1] - pitch_z4).abs() < 1e-4, "pitch {}", out[1]);
        assert!((out[2] + 1.0).abs() < 1e-4, "roll {}", out[2]);
    }

    #[test]
    fn absolute_mode_requires_all_three_axes() {
        let id = pointer_cortical_id(FrameChangeHandling::Absolute);
        let mut decoder = AngularPointerNeuronVoxelXYZPDecoder::new_box(
            id,
            AngularPointerProperties::new_absolute(3, 1, 10).unwrap(),
            CorticalChannelCount::new(1).unwrap(),
        )
        .unwrap();
        let mut pipelines = one_channel_pipeline();
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
        let out = read_signed(&pipelines);
        assert!(out[0].abs() < 1e-4);
        assert!(out[1].abs() < 1e-4);
        assert!(out[2].abs() < 1e-4);
    }

    #[test]
    fn incremental_mode_requires_window_parameters() {
        let id = pointer_cortical_id(FrameChangeHandling::Incremental);
        let result = AngularPointerNeuronVoxelXYZPDecoder::new_box(
            id,
            AngularPointerProperties::new_absolute(3, 1, 10).unwrap(),
            CorticalChannelCount::new(1).unwrap(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn incremental_mode_maps_polarity_columns() {
        let id = pointer_cortical_id(FrameChangeHandling::Incremental);
        let mut decoder = AngularPointerNeuronVoxelXYZPDecoder::new_box(
            id,
            AngularPointerProperties::new_incremental(6, 1, 10, 1000).unwrap(),
            CorticalChannelCount::new(1).unwrap(),
        )
        .unwrap();

        // yaw+ at z=0 (large +), pitch- at x=3 z=0 (large -), roll+ at x=4 z=9 (small +).
        let mut arrays = NeuronVoxelXYZPArrays::new();
        arrays.push(&NeuronVoxelXYZP::new(0, 0, 0, 1.0));
        arrays.push(&NeuronVoxelXYZP::new(3, 0, 0, 1.0));
        arrays.push(&NeuronVoxelXYZP::new(4, 0, 9, 1.0));
        let mut neurons = CorticalMappedXYZPNeuronVoxels::new();
        neurons.insert(id, arrays);
        let mut pipelines = one_channel_pipeline();
        let mut changed = vec![false];
        decoder
            .read_neuron_data_multi_channel_into_pipeline_input_cache(
                &neurons,
                Instant::now(),
                &mut pipelines,
                &mut changed,
            )
            .unwrap();

        let out = read_signed(&pipelines);
        assert!((out[0] - 1.0).abs() < 1e-4, "yaw+ {}", out[0]);
        assert!((out[1] + 1.0).abs() < 1e-4, "pitch- {}", out[1]);
        assert!(out[2].abs() < 1e-4, "roll small {}", out[2]);
    }

    #[test]
    fn incremental_mode_accepts_single_axis_activation() {
        let id = pointer_cortical_id(FrameChangeHandling::Incremental);
        let mut decoder = AngularPointerNeuronVoxelXYZPDecoder::new_box(
            id,
            AngularPointerProperties::new_incremental(6, 1, 10, 1000).unwrap(),
            CorticalChannelCount::new(1).unwrap(),
        )
        .unwrap();
        let mut pipelines = one_channel_pipeline();
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
        let out = read_signed(&pipelines);
        assert!((out[0] - 1.0).abs() < 1e-4, "yaw {}", out[0]);
        assert!(out[1].abs() < 1e-4, "pitch {}", out[1]);
        assert!(out[2].abs() < 1e-4, "roll {}", out[2]);
    }
}
