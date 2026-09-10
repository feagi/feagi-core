//! Segmentation-mask decoder selector over `feagi-sensorimotor` misc-data coders.
//!
//! Selects FEAGI's object-segmentation motor template (`ObjectSegmentation` + `MiscData`) and
//! decodes per-pixel class ids by argmax over the Z (class) dimension.

use std::time::Instant;

use feagi_sensorimotor::data_types::descriptors::MiscDataDimensions;
use feagi_sensorimotor::ConnectorCache;
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelIndex, CorticalUnitIndex,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::FrameChangeHandling;
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;

use crate::binding::decoder::DecoderPlugin;
use crate::binding::profile::DecoderBindingProfile;
use crate::contracts::common::{PluginId, PluginRef};
use crate::contracts::prediction_record::TypedPrediction;
use crate::error::TrainerError;

/// Motor cortical unit index this decoder reads from for the segmentation slice.
const SEGMENTATION_MOTOR_UNIT: u8 = 0;

/// Stateless selector that decodes a dense class-id mask from misc-data motor output.
#[derive(Debug, Clone, Copy, Default)]
pub struct SegmentationMaskDecoder;

impl SegmentationMaskDecoder {
    /// Creates a new selector.
    pub fn new() -> Self {
        Self
    }

    fn misc_dimensions(profile: &DecoderBindingProfile) -> Result<MiscDataDimensions, TrainerError> {
        let width = profile.mask_width.ok_or_else(|| {
            TrainerError::Config("segmentation decoder requires decoder_profile.mask_width".to_string())
        })?;
        let height = profile.mask_height.ok_or_else(|| {
            TrainerError::Config(
                "segmentation decoder requires decoder_profile.mask_height".to_string(),
            )
        })?;
        let depth = profile.mask_depth.ok_or_else(|| {
            TrainerError::Config("segmentation decoder requires decoder_profile.mask_depth".to_string())
        })?;
        MiscDataDimensions::new(width, height, depth).map_err(map_err)
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
            version: "1.0.0".to_string(),
        }
    }

    fn decode(
        &mut self,
        motor: Self::Frame,
        profile: &DecoderBindingProfile,
    ) -> Result<TypedPrediction, TrainerError> {
        let misc_dims = Self::misc_dimensions(profile)?;
        let width = misc_dims.width as usize;
        let height = misc_dims.height as usize;
        let depth = misc_dims.depth as usize;

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
        let mut labels = vec![0_u8; width * height];
        for xi in 0..width {
            for yi in 0..height {
                let mut best_z = 0_usize;
                let mut best_val = f32::NEG_INFINITY;
                for zi in 0..depth {
                    let value = data[(xi, yi, zi)];
                    if value > best_val {
                        best_val = value;
                        best_z = zi;
                    }
                }
                labels[yi * width + xi] = best_z as u8;
            }
        }

        Ok(TypedPrediction::SegmentationMask {
            width: misc_dims.width,
            height: misc_dims.height,
            labels,
        })
    }
}
