//! Segmentation-mask decoder selector over `feagi-sensorimotor` misc-data coders.
//!
//! Selects FEAGI's object-segmentation motor template (`ObjectSegmentation` + `MiscData`,
//! `W×H×1`) and decodes each pixel's class from its potential,
//! `(class_id + 1) / class_count`. Pixels the brain left silent are reported as
//! [`UNPREDICTED_PIXEL`] and declared as the prediction's `unpredicted_label`.

use std::time::Instant;

use feagi_sensorimotor::data_types::descriptors::MiscDataDimensions;
use feagi_sensorimotor::ConnectorCache;
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelIndex, CorticalUnitIndex,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::FrameChangeHandling;
use feagi_structures::neuron_voxels::class_potential::decode_class_potential;
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;

use crate::binding::decoder::DecoderPlugin;
use crate::binding::profile::{validate_segmentation_class_count, DecoderBindingProfile};
use crate::contracts::common::{PluginId, PluginRef};
use crate::contracts::prediction_record::{TypedPrediction, UNPREDICTED_PIXEL};
use crate::error::TrainerError;

/// Motor cortical unit index this decoder reads from for the segmentation slice.
const SEGMENTATION_MOTOR_UNIT: u16 = 0;

/// Stateless selector that decodes a dense class-id mask from misc-data motor output.
#[derive(Debug, Clone, Copy, Default)]
pub struct SegmentationMaskDecoder;

impl SegmentationMaskDecoder {
    /// Creates a new selector.
    pub fn new() -> Self {
        Self
    }

    fn misc_dimensions(
        profile: &DecoderBindingProfile,
    ) -> Result<MiscDataDimensions, TrainerError> {
        let width = profile.mask_width.ok_or_else(|| {
            TrainerError::Config(
                "segmentation decoder requires decoder_profile.mask_width".to_string(),
            )
        })?;
        let height = profile.mask_height.ok_or_else(|| {
            TrainerError::Config(
                "segmentation decoder requires decoder_profile.mask_height".to_string(),
            )
        })?;
        MiscDataDimensions::new(width, height, 1).map_err(map_err)
    }
}

fn map_err<E: std::fmt::Display>(e: E) -> TrainerError {
    TrainerError::Config(e.to_string())
}

impl DecoderPlugin for SegmentationMaskDecoder {
    type Frame = CorticalMappedXYZPNeuronVoxels;

    fn plugin_ref(&self) -> PluginRef {
        PluginRef {
            id: PluginId("decoder.segmentation_mask".to_string()),
            version: "2.0.0".to_string(),
        }
    }

    fn decode(
        &mut self,
        motor: Self::Frame,
        profile: &DecoderBindingProfile,
    ) -> Result<TypedPrediction, TrainerError> {
        validate_segmentation_class_count(profile.class_count)?;
        let misc_dims = Self::misc_dimensions(profile)?;
        let width = misc_dims.width as usize;
        let height = misc_dims.height as usize;

        let cache = ConnectorCache::new();
        let unit = CorticalUnitIndex::from(SEGMENTATION_MOTOR_UNIT);
        let segmentation = {
            let mut motor_cache = cache.get_motor_cache();
            motor_cache
                .object_segmentation_register(
                    unit,
                    CorticalChannelCount::new(1).map_err(map_err)?,
                    FrameChangeHandling::Absolute,
                    misc_dims,
                )
                .map_err(map_err)?;
            motor_cache
                .ingest_neuron_data_and_run_callbacks(motor, Instant::now())
                .map_err(map_err)?;
            motor_cache
                .object_segmentation_read_postprocessed_cache_value(
                    unit,
                    CorticalChannelIndex::from(0u32),
                )
                .map_err(map_err)?
        };

        let data = segmentation.get_internal_data();
        let mut labels = vec![UNPREDICTED_PIXEL; width * height];
        for xi in 0..width {
            for yi in 0..height {
                if let Some(class_id) =
                    decode_class_potential(data[(xi, yi, 0)], profile.class_count)
                {
                    labels[yi * width + xi] = class_id as u8;
                }
            }
        }

        Ok(TypedPrediction::SegmentationMask {
            width: misc_dims.width,
            height: misc_dims.height,
            labels,
            unpredicted_label: Some(UNPREDICTED_PIXEL),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use feagi_structures::genomic::MotorCorticalUnit;
    use feagi_structures::neuron_voxels::class_potential::encode_class_potential;
    use feagi_structures::neuron_voxels::xyzp::NeuronVoxelXYZPArrays;

    fn profile(class_count: u32) -> DecoderBindingProfile {
        DecoderBindingProfile {
            cortical_area_id: "object_segmentation".to_string(),
            cortical_name: None,
            class_count,
            bins: 1,
            mask_width: Some(3),
            mask_height: Some(1),
        }
    }

    fn motor_frame(pixels: &[(u32, f32)]) -> CorticalMappedXYZPNeuronVoxels {
        let cortical_id =
            MotorCorticalUnit::get_cortical_ids_array_for_object_segmentation_with_parameters(
                FrameChangeHandling::Absolute,
                CorticalUnitIndex::from(SEGMENTATION_MOTOR_UNIT),
            )[0];
        let mut arrays = NeuronVoxelXYZPArrays::new();
        for (x, potential) in pixels {
            arrays.push_raw(*x, 0, 0, *potential);
        }
        let mut frame = CorticalMappedXYZPNeuronVoxels::new();
        frame.insert(cortical_id, arrays);
        frame
    }

    #[test]
    fn decodes_class_from_potential_and_marks_silent_pixels() {
        let class_count = 19;
        let frame = motor_frame(&[
            (0, encode_class_potential(13, class_count).unwrap()),
            (2, encode_class_potential(0, class_count).unwrap()),
        ]);
        let prediction = SegmentationMaskDecoder::new()
            .decode(frame, &profile(class_count))
            .expect("decode");
        assert_eq!(
            prediction,
            TypedPrediction::SegmentationMask {
                width: 3,
                height: 1,
                labels: vec![13, UNPREDICTED_PIXEL, 0],
                unpredicted_label: Some(UNPREDICTED_PIXEL),
            }
        );
    }

    #[test]
    fn rejects_class_counts_that_collide_with_the_silent_label() {
        let frame = motor_frame(&[]);
        assert!(SegmentationMaskDecoder::new()
            .decode(frame, &profile(256))
            .is_err());
    }
}
