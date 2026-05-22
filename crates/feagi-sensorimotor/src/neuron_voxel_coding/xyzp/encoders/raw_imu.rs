//! Encoder for [`crate::data_types::RawIMU`] composite values.
//!
//! `RawIMU` is the composite wrapped data type backing
//! [`feagi_structures::genomic::SensoryCorticalUnit::RawIMU`]. A single Raw IMU
//! cortical unit owns THREE sub-cortical-areas (accelerometer, gyroscope,
//! magnetometer); this encoder is the analog of
//! [`SegmentedImageFrameNeuronVoxelXYZPEncoder`](super::SegmentedImageFrameNeuronVoxelXYZPEncoder)
//! for IMU data: one composite read from the per-channel pipeline cache, three
//! voxel-array writes (one per sub-area) at burst time.
//!
//! Per sub-area, encoding logic matches the D3 signed branch of
//! [`super::PercentageNeuronVoxelXYZPEncoder`]. The X-axis layout per channel
//! `c` packs each sub-component's three signed axes into 6 X-slots:
//! `[c*6+0]=a_pos, [c*6+1]=a_neg, [c*6+2]=b_pos, [c*6+3]=b_neg,
//! [c*6+4]=c_pos, [c*6+5]=c_neg`.

use crate::configuration::jsonable::JSONEncoderProperties;
use crate::data_pipeline::per_channel_stream_caches::{
    PipelineStageRunner, SensoryPipelineStageRunner,
};
use crate::data_types::{RawIMU, SignedPercentage3D, RAW_IMU_SUBUNIT_COUNT};
use crate::neuron_voxel_coding::xyzp::coder_shared_functions::{
    encode_signed_percentage_to_fractional_exponential_neuron_z_indexes,
    encode_signed_percentage_to_linear_neuron_z_index,
};
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPEncoder;
use crate::wrapped_io_data::WrappedIOType;
use feagi_genome_definitions::::descriptors::{
    CorticalChannelCount, CorticalChannelIndex, NeuronDepth,
};
use feagi_genome_definitions::::io_cortical_area_configuration_flag::PercentageNeuronPositioning;
use feagi_genome_definitions::::CorticalID;
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use feagi_structures::FeagiDataError;
use rayon::prelude::*;
use std::time::Instant;

/// Per-channel positive/negative z-index scratch buffers for a single
/// 3-axis sub-component (accel, gyro, or mag).
#[derive(Debug, Default, Clone)]
struct AxisTripletScratch {
    a_pos: Vec<u32>,
    a_neg: Vec<u32>,
    b_pos: Vec<u32>,
    b_neg: Vec<u32>,
    c_pos: Vec<u32>,
    c_neg: Vec<u32>,
}

impl AxisTripletScratch {
    fn clear(&mut self) {
        self.a_pos.clear();
        self.a_neg.clear();
        self.b_pos.clear();
        self.b_neg.clear();
        self.c_pos.clear();
        self.c_neg.clear();
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct RawIMUNeuronVoxelXYZPEncoder {
    /// Authoritative cortical-id targets in the canonical Raw IMU sub-area
    /// order: index 0 = accelerometer, 1 = gyroscope, 2 = magnetometer.
    cortical_write_targets: [CorticalID; RAW_IMU_SUBUNIT_COUNT],
    z_neuron_resolution: NeuronDepth,
    interpolation: PercentageNeuronPositioning,
    /// Per-channel × per-sub-area scratch space.
    /// Outer index = channel, inner index = sub-area in canonical order.
    scratch_spaces: Vec<[AxisTripletScratch; RAW_IMU_SUBUNIT_COUNT]>,
}

impl RawIMUNeuronVoxelXYZPEncoder {
    /// Number of X-slots used per channel for a signed 3-axis sub-component
    /// (a_pos, a_neg, b_pos, b_neg, c_pos, c_neg).
    const CHANNEL_X_WIDTH: u32 = 6;

    #[allow(dead_code)]
    pub fn new_box(
        cortical_ids: [CorticalID; RAW_IMU_SUBUNIT_COUNT],
        z_neuron_resolution: NeuronDepth,
        number_channels: CorticalChannelCount,
        interpolation: PercentageNeuronPositioning,
    ) -> Result<Box<dyn NeuronVoxelXYZPEncoder + Sync + Send>, FeagiDataError> {
        let num_channels = *number_channels as usize;
        let scratch_spaces = (0..num_channels)
            .map(|_| std::array::from_fn(|_| AxisTripletScratch::default()))
            .collect();

        let encoder = RawIMUNeuronVoxelXYZPEncoder {
            cortical_write_targets: cortical_ids,
            z_neuron_resolution,
            interpolation,
            scratch_spaces,
        };
        Ok(Box::new(encoder))
    }

    /// Encode one signed 3-axis sub-component into the provided scratch slot.
    /// Mirrors the D3 signed branch of `PercentageNeuronVoxelXYZPEncoder`.
    #[inline]
    fn encode_sub_component(
        interpolation: PercentageNeuronPositioning,
        sub_component: &SignedPercentage3D,
        z_depth: u32,
        z_depth_float: f32,
        scratch: &mut AxisTripletScratch,
    ) {
        scratch.clear();
        match interpolation {
            PercentageNeuronPositioning::Linear => {
                encode_signed_percentage_to_linear_neuron_z_index(
                    &sub_component.a,
                    z_depth_float,
                    &mut scratch.a_pos,
                    &mut scratch.a_neg,
                );
                encode_signed_percentage_to_linear_neuron_z_index(
                    &sub_component.b,
                    z_depth_float,
                    &mut scratch.b_pos,
                    &mut scratch.b_neg,
                );
                encode_signed_percentage_to_linear_neuron_z_index(
                    &sub_component.c,
                    z_depth_float,
                    &mut scratch.c_pos,
                    &mut scratch.c_neg,
                );
            }
            PercentageNeuronPositioning::Fractional => {
                encode_signed_percentage_to_fractional_exponential_neuron_z_indexes(
                    &sub_component.a,
                    z_depth,
                    &mut scratch.a_pos,
                    &mut scratch.a_neg,
                );
                encode_signed_percentage_to_fractional_exponential_neuron_z_indexes(
                    &sub_component.b,
                    z_depth,
                    &mut scratch.b_pos,
                    &mut scratch.b_neg,
                );
                encode_signed_percentage_to_fractional_exponential_neuron_z_indexes(
                    &sub_component.c,
                    z_depth,
                    &mut scratch.c_pos,
                    &mut scratch.c_neg,
                );
            }
        }
    }
}

impl NeuronVoxelXYZPEncoder for RawIMUNeuronVoxelXYZPEncoder {
    fn get_encodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::RawIMU
    }

    fn get_as_properties(&self) -> JSONEncoderProperties {
        JSONEncoderProperties::Percentage(
            self.z_neuron_resolution,
            self.interpolation,
            true, // is_signed
            crate::data_types::descriptors::PercentageChannelDimensionality::D3,
        )
    }

    fn write_neuron_data_multi_channel_from_processed_cache(
        &mut self,
        pipelines: &[SensoryPipelineStageRunner],
        time_of_previous_burst: Instant,
        write_target: &mut CorticalMappedXYZPNeuronVoxels,
    ) -> Result<(), FeagiDataError> {
        let z_depth = *self.z_neuron_resolution;
        let z_depth_float = z_depth as f32;
        let interpolation = self.interpolation;

        // Phase 1 — per channel, decompose composite and fill scratch buffers
        // for the three sub-components in parallel across channels.
        pipelines
            .par_iter()
            .zip(self.scratch_spaces.par_iter_mut())
            .try_for_each(|(pipeline, sub_scratches)| -> Result<(), FeagiDataError> {
                if pipeline.get_last_processed_instant() < time_of_previous_burst {
                    // Channel was not updated this burst; clear scratches so no
                    // stale neurons are emitted for it on the aggregation pass.
                    for s in sub_scratches.iter_mut() {
                        s.clear();
                    }
                    return Ok(());
                }
                let data = pipeline.get_postprocessed_sensor_value();
                let composite: &RawIMU = data.try_into()?;
                let ordered = composite.get_ordered_sub_components();
                for (sub_index, sub_value) in ordered.iter().enumerate() {
                    Self::encode_sub_component(
                        interpolation,
                        sub_value,
                        z_depth,
                        z_depth_float,
                        &mut sub_scratches[sub_index],
                    );
                }
                Ok(())
            })?;

        // Phase 2 — for each sub-area, ensure its cortical neuron array is
        // present-and-cleared, then fold every channel's scratch into it.
        const Y: u32 = 0;
        for sub_index in 0..RAW_IMU_SUBUNIT_COUNT {
            let cortical_id = &self.cortical_write_targets[sub_index];
            let neuron_array_target = write_target.ensure_clear_and_borrow_mut(cortical_id);

            for (current_channel_index, sub_scratches) in self.scratch_spaces.iter().enumerate() {
                let pipeline = &pipelines[current_channel_index];
                if pipeline.get_last_processed_instant() < time_of_previous_burst {
                    continue;
                }
                let channel_write_target = pipeline
                    .get_channel_index_override()
                    .unwrap_or_else(|| CorticalChannelIndex::from(current_channel_index as u32));
                let c = *channel_write_target;
                let scratch = &sub_scratches[sub_index];

                for z in &scratch.a_pos {
                    neuron_array_target.push_raw(c * Self::CHANNEL_X_WIDTH, Y, *z, 1.0);
                }
                for z in &scratch.a_neg {
                    neuron_array_target.push_raw(c * Self::CHANNEL_X_WIDTH + 1, Y, *z, 1.0);
                }
                for z in &scratch.b_pos {
                    neuron_array_target.push_raw(c * Self::CHANNEL_X_WIDTH + 2, Y, *z, 1.0);
                }
                for z in &scratch.b_neg {
                    neuron_array_target.push_raw(c * Self::CHANNEL_X_WIDTH + 3, Y, *z, 1.0);
                }
                for z in &scratch.c_pos {
                    neuron_array_target.push_raw(c * Self::CHANNEL_X_WIDTH + 4, Y, *z, 1.0);
                }
                for z in &scratch.c_neg {
                    neuron_array_target.push_raw(c * Self::CHANNEL_X_WIDTH + 5, Y, *z, 1.0);
                }
            }
        }

        Ok(())
    }
}
