//! Image-frame encoder selector over `feagi-sensorimotor` vision coders.
//!
//! Selects FEAGI's Cartesian-plane vision coder (`Vision` template) and writes a resized RGB
//! [`ImageFrame`] into the sensory cache before encoding to neuron voxels.

use std::time::Instant;

use feagi_sensorimotor::data_types::descriptors::{
    ColorChannelLayout, ColorSpace, ImageFrameProperties, ImageXYResolution,
};
use feagi_sensorimotor::data_types::processing::ImageFrameProcessor;
use feagi_sensorimotor::data_types::ImageFrame;
use feagi_sensorimotor::data_pipeline::PipelineStageProperties;
use feagi_sensorimotor::wrapped_io_data::WrappedIOData;
use feagi_sensorimotor::ConnectorCache;
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelIndex, CorticalUnitIndex,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::FrameChangeHandling;
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;

use crate::binding::encoder::EncoderPlugin;
use crate::binding::profile::EncoderBindingProfile;
use crate::contracts::common::{PluginId, PluginRef};
use crate::contracts::ir_sample::{IRSample, Payload};
use crate::error::TrainerError;

/// Sensory cortical unit index this encoder writes to for the vision slice.
const VISION_SENSORY_UNIT: u8 = 0;

/// Stateless selector that encodes PNG/JPEG image bytes via FEAGI vision coding.
#[derive(Debug, Clone, Copy, Default)]
pub struct ImageFrameEncoder;

impl ImageFrameEncoder {
    /// Creates a new selector.
    pub fn new() -> Self {
        Self
    }

    fn image_properties(profile: &EncoderBindingProfile) -> Result<ImageFrameProperties, TrainerError> {
        let width = profile.image_width.ok_or_else(|| {
            TrainerError::Config("image encoder requires encoder_profile.image_width".to_string())
        })?;
        let height = profile.image_height.ok_or_else(|| {
            TrainerError::Config("image encoder requires encoder_profile.image_height".to_string())
        })?;
        let resolution = ImageXYResolution::new(width, height).map_err(map_err)?;
        ImageFrameProperties::new(resolution, ColorSpace::Gamma, ColorChannelLayout::RGB)
            .map_err(map_err)
    }
}

fn map_err<E: std::fmt::Display>(e: E) -> TrainerError {
    TrainerError::Config(e.to_string())
}

impl EncoderPlugin for ImageFrameEncoder {
    type Frame = CorticalMappedXYZPNeuronVoxels;

    fn plugin_ref(&self) -> PluginRef {
        PluginRef {
            id: PluginId("encoder.image_frame".to_string()),
            version: "1.0.0".to_string(),
        }
    }

    fn encode(
        &mut self,
        sample: &IRSample,
        profile: &EncoderBindingProfile,
    ) -> Result<Self::Frame, TrainerError> {
        let png_bytes = match &sample.payload {
            Payload::Bytes(bytes) => bytes,
            other => {
                return Err(TrainerError::Config(format!(
                    "image encoder requires a bytes payload, got {other:?}"
                )))
            }
        };

        let image_props = Self::image_properties(profile)?;
        let frame =
            ImageFrame::new_from_png_bytes(png_bytes, &ColorSpace::Gamma).map_err(map_err)?;
        image_props
            .verify_image_frame_matches_properties(&frame)
            .map_err(map_err)?;

        let cache = ConnectorCache::new();
        let unit = CorticalUnitIndex::from(VISION_SENSORY_UNIT);
        let mut sensor_cache = cache.get_sensor_cache();
        sensor_cache
            .vision_register(
                unit,
                CorticalChannelCount::new(profile.channels).map_err(map_err)?,
                FrameChangeHandling::Absolute,
                image_props,
            )
            .map_err(map_err)?;
        sensor_cache
            .vision_replace_all_stages(
                unit,
                CorticalChannelIndex::from(0u32),
                vec![PipelineStageProperties::new_image_frame_processor(
                    ImageFrameProcessor::new(image_props),
                )],
            )
            .map_err(map_err)?;
        sensor_cache
            .vision_write(
                unit,
                CorticalChannelIndex::from(0u32),
                WrappedIOData::ImageFrame(frame),
            )
            .map_err(map_err)?;
        sensor_cache
            .encode_all_sensors_to_neurons(Instant::now())
            .map_err(map_err)?;
        Ok(sensor_cache.get_neurons().clone())
    }
}
