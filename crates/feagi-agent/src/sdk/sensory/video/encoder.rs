// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Video encoder implementation

use crate::sdk::base::CorticalTopology;
use crate::sdk::error::{Result, SdkError};
use crate::sdk::sensory::traits::SensoryEncoder;
use crate::sdk::sensory::video::config::{VideoEncoderConfig, VideoEncodingStrategy};
use feagi_sensorimotor::data_types::descriptors::{
    ColorChannelLayout, ColorSpace, ImageFrameProperties, ImageXYResolution,
    SegmentedImageFrameProperties, SegmentedXYImageResolutions,
};
use feagi_sensorimotor::data_types::processing::{ImageFrameProcessor, ImageFrameSegmentator};
use feagi_sensorimotor::data_types::{GazeProperties, ImageFrame, Percentage, Percentage2D, SegmentedImageFrame};
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::neuron_voxels::xyzp::{
    CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZPArrays,
};

/// Video encoder for FEAGI sensory data
///
/// Encodes video frames into FEAGI's XYZP voxel format. Supports both simple
/// full-frame vision and segmented vision with gaze modulation.
///
/// # Example
/// ```ignore
/// use feagi_agent::sdk::sensory::video::{VideoEncoder, VideoEncoderConfig, VideoEncodingStrategy};
/// use feagi_agent::sdk::base::TopologyCache;
/// use feagi_sensorimotor::data_types::ImageFrame;
///
/// // Create encoder
/// let config = VideoEncoderConfig { /* ... */ };
/// let topology_cache = TopologyCache::new("localhost", 8080, 5.0)?;
/// let mut encoder = VideoEncoder::new(config, &topology_cache).await?;
///
/// // Encode frames
/// let frame: ImageFrame = /* ... */;
/// let encoded = encoder.encode(&frame)?;
/// ```
pub struct VideoEncoder {
    config: VideoEncoderConfig,
    cortical_ids: Vec<CorticalID>,
    _topologies: Vec<CorticalTopology>,
    mode: EncoderMode,
}

enum EncoderMode {
    Simple {
        processor: ImageFrameProcessor,
        prev_frame: ImageFrame,
        _input_props: ImageFrameProperties,
        output_props: ImageFrameProperties,
    },
    Segmented {
        segmentator: ImageFrameSegmentator,
        brightness_contrast: ImageFrameProcessor,
        prev_frame: SegmentedImageFrame,
        input_props: ImageFrameProperties,
        output_props: SegmentedImageFrameProperties,
        gaze: GazeProperties,
    },
}

impl VideoEncoder {
    /// Create a new video encoder
    ///
    /// This fetches topologies from FEAGI and configures the encoder.
    ///
    /// # Arguments
    /// * `config` - Encoder configuration
    /// * `topology_cache` - Topology cache for fetching cortical dimensions
    pub async fn new(
        config: VideoEncoderConfig,
        topology_cache: &crate::sdk::base::TopologyCache,
    ) -> Result<Self> {
        config.validate()?;

        let unit_index = feagi_structures::genomic::cortical_area::descriptors::CorticalUnitIndex::from(config.cortical_unit_id);
        let cortical_ids = config.encoding_strategy.cortical_ids(unit_index);

        // Fetch topologies
        let topologies = topology_cache.get_topologies(&cortical_ids).await?;

        // Validate topology count
        let expected_count = config.encoding_strategy.cortical_id_count();
        if topologies.len() != expected_count {
            return Err(SdkError::InvalidConfiguration(format!(
                "Expected {} topologies, got {}",
                expected_count,
                topologies.len()
            )));
        }

        // Build encoder mode
        let mode = match config.encoding_strategy {
            VideoEncodingStrategy::SimpleVision => {
                Self::build_simple_mode(&config, &topologies[0])?
            }
            VideoEncodingStrategy::SegmentedVision => {
                Self::build_segmented_mode(&config, &topologies)?
            }
        };

        Ok(Self {
            config,
            cortical_ids,
            _topologies: topologies,
            mode,
        })
    }

    fn build_simple_mode(
        config: &VideoEncoderConfig,
        topology: &CorticalTopology,
    ) -> Result<EncoderMode> {
        let input_res = ImageXYResolution::new(config.source_width, config.source_height)?;
        let input_props =
            ImageFrameProperties::new(input_res, ColorSpace::Gamma, ColorChannelLayout::RGB)?;

        let out_res = ImageXYResolution::new(topology.width, topology.height)?;
        let out_layout = match topology.depth {
            1 => ColorChannelLayout::GrayScale,
            3 => ColorChannelLayout::RGB,
            other => {
                return Err(SdkError::InvalidConfiguration(format!(
                    "Unsupported simple-vision depth={}",
                    other
                )))
            }
        };
        let output_props = ImageFrameProperties::new(out_res, ColorSpace::Gamma, out_layout)?;

        let mut processor = ImageFrameProcessor::new(input_props);
        processor.set_resizing_to(out_res)?;
        processor.set_brightness_offset(config.brightness)?;
        processor.set_contrast_change(config.contrast)?;

        let prev_frame = ImageFrame::new_from_image_frame_properties(&output_props)?;

        Ok(EncoderMode::Simple {
            processor,
            prev_frame,
            _input_props: input_props,
            output_props,
        })
    }

    fn build_segmented_mode(
        config: &VideoEncoderConfig,
        topologies: &[CorticalTopology],
    ) -> Result<EncoderMode> {
        if topologies.len() != 9 {
            return Err(SdkError::InvalidConfiguration(format!(
                "Segmented vision requires 9 topologies, got {}",
                topologies.len()
            )));
        }

        let input_res = ImageXYResolution::new(config.source_width, config.source_height)?;
        let input_props =
            ImageFrameProperties::new(input_res, ColorSpace::Gamma, ColorChannelLayout::RGB)?;

        // Build resolutions for all 9 segments
        let res = SegmentedXYImageResolutions::new(
            ImageXYResolution::new(topologies[0].width, topologies[0].height)?,
            ImageXYResolution::new(topologies[1].width, topologies[1].height)?,
            ImageXYResolution::new(topologies[2].width, topologies[2].height)?,
            ImageXYResolution::new(topologies[3].width, topologies[3].height)?,
            ImageXYResolution::new(topologies[4].width, topologies[4].height)?,
            ImageXYResolution::new(topologies[5].width, topologies[5].height)?,
            ImageXYResolution::new(topologies[6].width, topologies[6].height)?,
            ImageXYResolution::new(topologies[7].width, topologies[7].height)?,
            ImageXYResolution::new(topologies[8].width, topologies[8].height)?,
        );

        let center_layout = match topologies[4].depth {
            1 => ColorChannelLayout::GrayScale,
            3 => ColorChannelLayout::RGB,
            other => {
                return Err(SdkError::InvalidConfiguration(format!(
                    "Unsupported segmented center depth={}",
                    other
                )))
            }
        };

        let peripheral_layout = match topologies[0].depth {
            1 => ColorChannelLayout::GrayScale,
            3 => ColorChannelLayout::RGB,
            other => {
                return Err(SdkError::InvalidConfiguration(format!(
                    "Unsupported segmented peripheral depth={}",
                    other
                )))
            }
        };

        let output_props =
            SegmentedImageFrameProperties::new(res, center_layout, peripheral_layout, ColorSpace::Gamma);

        // Default gaze (center, 75% modulation)
        let gaze = GazeProperties::new(
            Percentage2D::try_from((0.5_f32, 0.5_f32))?,
            Percentage::new_from_0_1(0.75)?,
        );

        let segmentator = ImageFrameSegmentator::new(input_props, output_props, gaze)?;
        let brightness_contrast = ImageFrameProcessor::new(input_props);

        let prev_frame = SegmentedImageFrame::from_segmented_image_frame_properties(&output_props)?;

        Ok(EncoderMode::Segmented {
            segmentator,
            brightness_contrast,
            prev_frame,
            input_props,
            output_props,
            gaze,
        })
    }

    /// Update gaze for segmented vision
    ///
    /// Only applies to segmented vision mode. Ignored for simple vision.
    pub fn set_gaze(&mut self, x: f32, y: f32, modulation: f32) -> Result<()> {
        match &mut self.mode {
            EncoderMode::Segmented {
                ref mut segmentator,
                ref mut gaze,
                ..
            } => {
                let new_gaze = GazeProperties::new(
                    Percentage2D::try_from((x.clamp(0.0, 1.0), y.clamp(0.0, 1.0)))?,
                    Percentage::new_from_0_1(modulation.clamp(0.0, 1.0))?,
                );
                *gaze = new_gaze;
                segmentator.update_gaze(&new_gaze)?;
                Ok(())
            }
            EncoderMode::Simple { .. } => {
                Err(SdkError::InvalidConfiguration(
                    "Gaze properties only apply to SegmentedVision encoding".to_string(),
                ))
            }
        }
    }

    /// Update brightness/contrast
    pub fn set_brightness(&mut self, brightness: i32) -> Result<()> {
        match &mut self.mode {
            EncoderMode::Simple {
                ref mut processor, ..
            } => {
                processor.set_brightness_offset(brightness)?;
            }
            EncoderMode::Segmented {
                ref mut brightness_contrast,
                ..
            } => {
                brightness_contrast.set_brightness_offset(brightness)?;
            }
        }
        Ok(())
    }

    pub fn set_contrast(&mut self, contrast: f32) -> Result<()> {
        match &mut self.mode {
            EncoderMode::Simple {
                ref mut processor, ..
            } => {
                processor.set_contrast_change(contrast)?;
            }
            EncoderMode::Segmented {
                ref mut brightness_contrast,
                ..
            } => {
                brightness_contrast.set_contrast_change(contrast)?;
            }
        }
        Ok(())
    }

    /// Update diff threshold for change detection
    pub fn set_diff_threshold(&mut self, threshold: u8) -> Result<()> {
        self.config.diff_threshold = threshold;
        Ok(())
    }

    /// Check if encoder is in SegmentedVision mode
    pub fn is_segmented_vision(&self) -> bool {
        matches!(self.mode, EncoderMode::Segmented { .. })
    }
}

impl SensoryEncoder for VideoEncoder {
    type Input = ImageFrame;

    fn encode(&mut self, input: &Self::Input) -> Result<Vec<u8>> {
        match &mut self.mode {
            EncoderMode::Simple {
                processor,
                prev_frame,
                output_props,
                ..
            } => {
                let mut processed = ImageFrame::new_from_image_frame_properties(output_props)?;
                processor.process_image(input, &mut processed)?;

                // Apply diff threshold
                apply_diff_threshold_image(
                    &mut processed,
                    &mut prev_frame.clone(),
                    self.config.diff_threshold,
                );

                // Encode to XYZP
                let mut mapped = CorticalMappedXYZPNeuronVoxels::new_with_capacity(1);
                let target = mapped.ensure_clear_and_borrow_mut(&self.cortical_ids[0]);
                encode_image_frame_to_xyzp(&processed, 0, target)?;

                // Serialize
                serialize_xyzp(&mapped)
            }
            EncoderMode::Segmented {
                segmentator,
                brightness_contrast,
                prev_frame,
                input_props,
                output_props,
                ..
            } => {
                let mut adjusted = ImageFrame::new_from_image_frame_properties(input_props)?;
                brightness_contrast.process_image(input, &mut adjusted)?;

                let mut segmented = SegmentedImageFrame::from_segmented_image_frame_properties(output_props)?;
                segmentator.segment_image(&adjusted, &mut segmented)?;

                // Apply diff threshold
                apply_diff_threshold_segmented(
                    &mut segmented,
                    prev_frame,
                    self.config.diff_threshold,
                );

                // Encode to XYZP
                let mut mapped = CorticalMappedXYZPNeuronVoxels::new_with_capacity(9);
                let cortical_ids_arr: [CorticalID; 9] = [
                    self.cortical_ids[0],
                    self.cortical_ids[1],
                    self.cortical_ids[2],
                    self.cortical_ids[3],
                    self.cortical_ids[4],
                    self.cortical_ids[5],
                    self.cortical_ids[6],
                    self.cortical_ids[7],
                    self.cortical_ids[8],
                ];
                encode_segmented_frame_to_xyzp_mapped(&segmented, 0, &cortical_ids_arr, &mut mapped)?;

                // Serialize
                serialize_xyzp(&mapped)
            }
        }
    }

    fn cortical_ids(&self) -> &[CorticalID] {
        &self.cortical_ids
    }
}

// Helper functions (extracted from desktop controllers)

fn apply_diff_threshold_image(current: &mut ImageFrame, prev: &mut ImageFrame, threshold: u8) {
    if threshold == 0 {
        prev.get_internal_byte_data_mut()
            .copy_from_slice(current.get_internal_byte_data());
        return;
    }

    let cur = current.get_internal_byte_data_mut();
    let prev_bytes = prev.get_internal_byte_data_mut();
    let t = threshold as i16;

    for (c, p) in cur.iter_mut().zip(prev_bytes.iter_mut()) {
        let diff = (*c as i16 - *p as i16).abs();
        if diff <= t {
            *c = 0;
        } else {
            *p = *c;
        }
    }
}

fn apply_diff_threshold_segmented(
    current: &mut SegmentedImageFrame,
    prev: &mut SegmentedImageFrame,
    threshold: u8,
) {
    let t = threshold as i16;
    let mut cur_images = current.get_mut_ordered_image_frame_references();
    let mut prev_images = prev.get_mut_ordered_image_frame_references();

    for (cur, prev) in cur_images.iter_mut().zip(prev_images.iter_mut()) {
        let cur_bytes = cur.get_internal_byte_data_mut();
        let prev_bytes = prev.get_internal_byte_data_mut();
        for (c, p) in cur_bytes.iter_mut().zip(prev_bytes.iter_mut()) {
            let diff = (*c as i16 - *p as i16).abs();
            if diff <= t {
                *c = 0;
            } else {
                *p = *c;
            }
        }
    }
}

fn encode_image_frame_to_xyzp(
    frame: &ImageFrame,
    channel_index: u32,
    write_target: &mut NeuronVoxelXYZPArrays,
) -> Result<()> {
    const EPSILON: u8 = 1;

    let res = frame.get_xy_resolution();
    let width = res.width;
    let height = res.height;
    let x_offset = channel_index * width;

    write_target.clear();
    write_target.ensure_capacity(frame.get_number_elements());

    write_target.update_vectors_from_external(|x_vec, y_vec, z_vec, p_vec| {
        for ((row, col, z), color_val) in frame.get_internal_data().indexed_iter() {
            if *color_val > EPSILON {
                x_vec.push(col as u32 + x_offset);
                y_vec.push(height - 1 - (row as u32));
                z_vec.push(z as u32);
                p_vec.push(*color_val as f32);
            }
        }
        Ok(())
    })?;

    Ok(())
}

fn encode_segmented_frame_to_xyzp_mapped(
    frame: &SegmentedImageFrame,
    channel_index: u32,
    cortical_ids: &[CorticalID; 9],
    mapped: &mut CorticalMappedXYZPNeuronVoxels,
) -> Result<()> {
    let ordered = frame.get_ordered_image_frame_references();
    for (idx, img) in ordered.iter().enumerate() {
        let target = mapped.ensure_clear_and_borrow_mut(&cortical_ids[idx]);
        encode_image_frame_to_xyzp(img, channel_index, target)?;
    }
    Ok(())
}

fn serialize_xyzp(mapped: &CorticalMappedXYZPNeuronVoxels) -> Result<Vec<u8>> {
    let mut byte_container = feagi_serialization::FeagiByteContainer::new_empty();
    byte_container
        .overwrite_byte_data_with_single_struct_data(mapped, 0)
        .map_err(|e| SdkError::EncodingFailed(format!("Failed to serialize XYZP: {:?}", e)))?;
    Ok(byte_container.get_byte_ref().to_vec())
}

