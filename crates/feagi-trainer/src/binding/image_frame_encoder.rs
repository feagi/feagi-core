//! Image-frame encoder selector over `feagi-sensorimotor` vision coders.
//!
//! Selects FEAGI's Cartesian-plane vision coder (`Vision` template) and writes a resized RGB
//! [`ImageFrame`] into the sensory cache before encoding to neuron voxels. When
//! `encoder_profile.segmentation_teacher` is set, also writes the sample mask onto the
//! Object Segmentation Input IPU (`iseg`, W×H×C, Z = class). Ignore-label pixels are omitted.

use std::time::Instant;

use feagi_sensorimotor::data_pipeline::PipelineStageProperties;
use feagi_sensorimotor::data_types::descriptors::{
    ColorChannelLayout, ColorSpace, ImageFrameProperties, ImageXYResolution,
};
use feagi_sensorimotor::data_types::processing::ImageFrameProcessor;
use feagi_sensorimotor::data_types::ImageFrame;
use feagi_sensorimotor::wrapped_io_data::WrappedIOData;
use feagi_sensorimotor::ConnectorCache;
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelIndex, CorticalUnitIndex,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::FrameChangeHandling;
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::genomic::SensoryCorticalUnit;
use feagi_structures::neuron_voxels::xyzp::{
    CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZPArrays,
};

use crate::binding::encoder::EncoderPlugin;
use crate::binding::profile::EncoderBindingProfile;
use crate::contracts::common::{PluginId, PluginRef};
use crate::contracts::ir_sample::{IRSample, Payload, TypedTarget};
use crate::error::TrainerError;

/// Sensory cortical unit index this encoder writes to for the vision slice.
const VISION_SENSORY_UNIT: u16 = 0;
/// Sensory unit index for the iseg mask teacher. Distinct type from vision, same group 0.
const ISEG_SENSORY_UNIT: u16 = 0;

/// Stateless selector that encodes PNG/JPEG image bytes via FEAGI vision coding.
#[derive(Debug, Clone, Copy, Default)]
pub struct ImageFrameEncoder;

impl ImageFrameEncoder {
    /// Creates a new selector.
    pub fn new() -> Self {
        Self
    }

    /// Cortical ID the mask teacher writes. Must match ensure_io Object Segmentation Input unit 0.
    pub fn iseg_cortical_id() -> CorticalID {
        SensoryCorticalUnit::get_cortical_ids_array_for_object_segmentation_input_with_parameters(
            FrameChangeHandling::Absolute,
            CorticalUnitIndex::from(ISEG_SENSORY_UNIT),
        )[0]
    }

    fn image_properties(
        profile: &EncoderBindingProfile,
    ) -> Result<ImageFrameProperties, TrainerError> {
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

    /// One voxel per kept pixel at `(x, y, class)` with P = 1.0. Ignore pixels are omitted.
    fn write_iseg_mask(
        width: u32,
        height: u32,
        depth: u32,
        labels: &[u8],
        ignore_label: Option<u8>,
    ) -> Result<NeuronVoxelXYZPArrays, TrainerError> {
        let expected_len = (width as usize)
            .checked_mul(height as usize)
            .ok_or_else(|| TrainerError::Config("iseg mask dimensions overflow".to_string()))?;
        if labels.len() != expected_len {
            return Err(TrainerError::Config(format!(
                "iseg mask length {} does not match {width}x{height}",
                labels.len()
            )));
        }
        let mut arrays = NeuronVoxelXYZPArrays::new();
        for y in 0..height {
            for x in 0..width {
                let label = labels[(y as usize) * (width as usize) + (x as usize)];
                if ignore_label == Some(label) {
                    continue;
                }
                let class_z = u32::from(label);
                if class_z >= depth {
                    return Err(TrainerError::Config(format!(
                        "iseg class {label} is outside mask depth {depth}"
                    )));
                }
                arrays.push_raw(x, y, class_z, 1.0);
            }
        }
        Ok(arrays)
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
        let mut neurons = sensor_cache.get_neurons().clone();
        if let Some(teacher) = &profile.segmentation_teacher {
            teacher.validate()?;
            let Some(TypedTarget::SegmentationMask {
                width,
                height,
                labels,
                ignore_label,
            }) = &sample.target
            else {
                return Err(TrainerError::Config(
                    "image encoder segmentation_teacher requires a SegmentationMask target"
                        .to_string(),
                ));
            };
            if *width != teacher.mask_width || *height != teacher.mask_height {
                return Err(TrainerError::Config(format!(
                    "iseg mask {}x{} does not match teacher {}x{}",
                    width, height, teacher.mask_width, teacher.mask_height
                )));
            }
            let arrays = Self::write_iseg_mask(
                teacher.mask_width,
                teacher.mask_height,
                teacher.mask_depth,
                labels,
                *ignore_label,
            )?;
            neurons.insert(Self::iseg_cortical_id(), arrays);
        }
        Ok(neurons)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::encoding_scheme::EncodingScheme;
    use crate::binding::profile::SegmentationTeacherBinding;
    use crate::contracts::common::{DatasetVersionId, Modality, OutputType, SampleId, Split};
    use crate::contracts::ir_sample::SCHEMA_VERSION;
    use image::{ImageFormat, Rgb, RgbImage};
    use std::collections::BTreeMap;
    use std::io::Cursor;

    fn rgb_png(width: u32, height: u32) -> Vec<u8> {
        let img = RgbImage::from_fn(width, height, |x, y| {
            Rgb([(x % 256) as u8, (y % 256) as u8, 80])
        });
        let mut bytes = Vec::new();
        image::DynamicImage::ImageRgb8(img)
            .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .expect("png");
        bytes
    }

    fn vision_profile(width: u32, height: u32) -> EncoderBindingProfile {
        EncoderBindingProfile {
            cortical_area_id: "simple_vision".to_string(),
            cortical_name: Some("Image Vision IPU".to_string()),
            channels: 1,
            scheme: EncodingScheme::PopulationSingleSpike {
                bins: 1,
                spacing: crate::binding::encoding_scheme::BinSpacing::Linear,
            },
            image_width: Some(width),
            image_height: Some(height),
            stream: None,
            teacher: None,
            segmentation_teacher: None,
        }
    }

    fn sample(png: Vec<u8>, target: Option<TypedTarget>) -> IRSample {
        IRSample {
            schema_version: SCHEMA_VERSION,
            sample_id: SampleId("s".to_string()),
            dataset_version_id: DatasetVersionId("d".to_string()),
            split: Split::Train,
            modality: Modality::Image,
            payload: Payload::Bytes(png),
            target,
            output_type: OutputType::SegmentationMask,
            coordinate_frame: None,
            timestamp: None,
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn vision_only_encode_omits_iseg() {
        let profile = vision_profile(2, 2);
        let mut encoder = ImageFrameEncoder::new();
        let frame = encoder
            .encode(&sample(rgb_png(2, 2), None), &profile)
            .expect("encode");
        assert!(
            frame
                .get_neurons_of(&ImageFrameEncoder::iseg_cortical_id())
                .is_none(),
            "vision-only encode must not write iseg"
        );
    }

    #[test]
    fn segmentation_teacher_writes_class_voxels_and_skips_ignore() {
        let mut profile = vision_profile(2, 2);
        profile.segmentation_teacher = Some(SegmentationTeacherBinding {
            cortical_area_id: "object_segmentation_input".to_string(),
            cortical_name: Some("Image Segmentation Input IPU".to_string()),
            mask_width: 2,
            mask_height: 2,
            mask_depth: 3,
        });
        let target = TypedTarget::SegmentationMask {
            width: 2,
            height: 2,
            // (0,0)=class 0, (1,0)=class 2, (0,1)=ignore, (1,1)=class 1
            labels: vec![0, 2, 255, 1],
            ignore_label: Some(255),
        };
        let mut encoder = ImageFrameEncoder::new();
        let frame = encoder
            .encode(&sample(rgb_png(2, 2), Some(target)), &profile)
            .expect("encode");
        let teacher = frame
            .get_neurons_of(&ImageFrameEncoder::iseg_cortical_id())
            .expect("iseg");
        let mut voxels: Vec<_> = teacher
            .iter()
            .map(|n| {
                (
                    n.neuron_voxel_coordinate.x,
                    n.neuron_voxel_coordinate.y,
                    n.neuron_voxel_coordinate.z,
                    n.potential,
                )
            })
            .collect();
        voxels.sort_by_key(|v| (v.1, v.0, v.2));
        assert_eq!(voxels, vec![(0, 0, 0, 1.0), (1, 0, 2, 1.0), (1, 1, 1, 1.0)]);
    }

    #[test]
    fn segmentation_teacher_without_mask_is_an_error() {
        let mut profile = vision_profile(2, 2);
        profile.segmentation_teacher = Some(SegmentationTeacherBinding {
            cortical_area_id: "object_segmentation_input".to_string(),
            cortical_name: None,
            mask_width: 2,
            mask_height: 2,
            mask_depth: 2,
        });
        let mut encoder = ImageFrameEncoder::new();
        let err = encoder
            .encode(&sample(rgb_png(2, 2), None), &profile)
            .expect_err("missing mask");
        assert!(err.to_string().contains("SegmentationMask"));
    }
}
