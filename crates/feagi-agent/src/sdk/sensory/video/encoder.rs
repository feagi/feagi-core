//! Video encoder implementation.

use crate::core::SdkError;
use crate::sdk::base::TopologyCache;
use crate::sdk::sensory::traits::SensoryEncoder;
use crate::sdk::sensory::video::config::{VideoEncoderConfig, VideoEncodingStrategy};
use crate::sdk::types::{
    ColorChannelLayout, ColorSpace, CorticalChannelCount, CorticalChannelIndex, CorticalUnitIndex,
    FrameChangeHandling, GazeProperties, ImageFrame, ImageFrameProperties, ImageXYResolution,
    SegmentedImageFrameProperties, SensorDeviceCache, SensoryCorticalUnit, WrappedIOData,
};
use feagi_sensorimotor::data_types::descriptors::SegmentedXYImageResolutions;

/// Video encoder backed by a sensor cache.
pub struct VideoEncoder {
    config: VideoEncoderConfig,
    cache: SensorDeviceCache,
    cortical_ids: Vec<crate::sdk::types::CorticalID>,
    channel_count: CorticalChannelCount,
    segmented_props: Option<SegmentedImageFrameProperties>,
    gaze_properties: GazeProperties,
    input_properties: Option<ImageFrameProperties>,
}

impl VideoEncoder {
    /// Create a new video encoder with topology-aware configuration.
    pub async fn new(
        config: VideoEncoderConfig,
        topology_cache: &TopologyCache,
    ) -> Result<Self, SdkError> {
        let unit = CorticalUnitIndex::from(config.cortical_unit_id);
        let frame = FrameChangeHandling::Absolute;

        let cortical_ids = match config.encoding_strategy {
            VideoEncodingStrategy::SimpleVision => SensoryCorticalUnit::Vision
                .get_cortical_ids_array_for_vision_with_parameters(frame, unit)
                .to_vec(),
            VideoEncodingStrategy::SegmentedVision => SensoryCorticalUnit::SegmentedVision
                .get_cortical_ids_array_for_segmented_vision_with_parameters(frame, unit)
                .to_vec(),
        };

        let (channel_count, segmented_props) =
            if config.encoding_strategy == VideoEncodingStrategy::SegmentedVision {
                let topologies = topology_cache.get_topologies(&cortical_ids).await?;
                let channels = topologies
                    .first()
                    .map(|topo| topo.channels)
                    .unwrap_or(1);
                let channel_count = CorticalChannelCount::new(channels).map_err(|e| {
                    SdkError::Other(format!("Segmented channel count invalid: {e}"))
                })?;
            let center_topo = topologies.get(4).ok_or_else(|| {
                SdkError::Other("Segmented vision center topology missing".to_string())
            })?;
            let peripheral_topo = topologies.first().ok_or_else(|| {
                SdkError::Other("Segmented vision peripheral topology missing".to_string())
            })?;
            let center_res = ImageXYResolution::new(center_topo.width, center_topo.height)
                .map_err(|e| SdkError::Other(format!("Segmented center resolution: {e}")))?;
            let peripheral_res =
                ImageXYResolution::new(peripheral_topo.width, peripheral_topo.height)
                    .map_err(|e| SdkError::Other(format!("Segmented peripheral resolution: {e}")))?;
            let resolutions =
                SegmentedXYImageResolutions::create_with_same_sized_peripheral(center_res, peripheral_res);

            let center_layout = layout_from_depth(center_topo.depth)?;
            let peripheral_layout = layout_from_depth(peripheral_topo.depth)?;
            // TODO: allow caller-configurable ColorSpace and layouts.
            let segmented_props = SegmentedImageFrameProperties::new(
                resolutions,
                center_layout,
                peripheral_layout,
                ColorSpace::Gamma,
            );
            (channel_count, Some(segmented_props))
        } else {
            let topo = topology_cache.get_topology(&cortical_ids[0]).await?;
            let channel_count = CorticalChannelCount::new(topo.channels).map_err(|e| {
                SdkError::Other(format!("Vision channel count invalid: {e}"))
            })?;
            (channel_count, None)
        };

        Ok(Self {
            config,
            cache: SensorDeviceCache::new(),
            cortical_ids,
            channel_count,
            segmented_props,
            gaze_properties: GazeProperties::create_default_centered(),
            input_properties: None,
        })
    }

    /// Set gaze properties for segmented vision encoding.
    pub fn set_gaze_properties(&mut self, gaze: &GazeProperties) -> Result<(), SdkError> {
        self.gaze_properties = gaze.clone();
        // TODO: propagate gaze changes into segmented vision pipeline stage.
        Ok(())
    }

    /// Set brightness adjustment applied before encoding.
    pub fn set_brightness(&mut self, brightness: i32) -> Result<(), SdkError> {
        self.config.brightness = brightness;
        // TODO: apply brightness in encode().
        Ok(())
    }

    /// Set contrast adjustment applied before encoding.
    pub fn set_contrast(&mut self, contrast: f32) -> Result<(), SdkError> {
        self.config.contrast = contrast;
        // TODO: apply contrast in encode().
        Ok(())
    }
}

impl SensoryEncoder for VideoEncoder {
    type Input = ImageFrame;

    fn encode(&mut self, input: &Self::Input) -> Result<Vec<u8>, SdkError> {
        let unit = CorticalUnitIndex::from(self.config.cortical_unit_id);
        let channel = CorticalChannelIndex::from(0u32);

        if self.input_properties.is_none() {
            let props = input.get_image_frame_properties();
            self.input_properties = Some(props);
            match self.config.encoding_strategy {
                VideoEncodingStrategy::SimpleVision => {
                    self.cache
                        .vision_register(
                            unit,
                            self.channel_count,
                            FrameChangeHandling::Absolute,
                            props,
                        )
                        .map_err(|e| SdkError::Other(format!("Vision register failed: {e}")))?;
                }
                VideoEncodingStrategy::SegmentedVision => {
                    let segmented_props = self.segmented_props.ok_or_else(|| {
                        SdkError::Other("Segmented vision properties missing".to_string())
                    })?;
                    self.cache
                        .segmented_vision_register(
                            unit,
                            self.channel_count,
                            FrameChangeHandling::Absolute,
                            props,
                            segmented_props,
                            self.gaze_properties.clone(),
                        )
                        .map_err(|e| {
                            SdkError::Other(format!("Segmented vision register failed: {e}"))
                        })?;
                }
            }
        }

        match self.config.encoding_strategy {
            VideoEncodingStrategy::SimpleVision => {
                // TODO: apply brightness/contrast/diff threshold preprocessing here.
                self.cache
                    .vision_write(unit, channel, WrappedIOData::ImageFrame(input.clone()))
                    .map_err(|e| SdkError::Other(format!("Vision write failed: {e}")))?;
            }
            VideoEncodingStrategy::SegmentedVision => {
                // TODO: apply brightness/contrast/diff threshold preprocessing here.
                self.cache
                    .segmented_vision_write(unit, channel, input.clone().into())
                    .map_err(|e| SdkError::Other(format!("Segmented write failed: {e}")))?;
            }
        }

        self.cache
            .encode_neurons_to_bytes()
            .map_err(|e| SdkError::Other(format!("Video byte encode failed: {e}")))?;

        Ok(self.cache.get_feagi_byte_container().get_byte_ref().to_vec())
    }

    fn cortical_ids(&self) -> &[crate::sdk::types::CorticalID] {
        &self.cortical_ids
    }
}

fn layout_from_depth(depth: u32) -> Result<ColorChannelLayout, SdkError> {
    match depth {
        1 => Ok(ColorChannelLayout::GrayScale),
        3 => Ok(ColorChannelLayout::RGB),
        _ => Err(SdkError::Other(format!(
            "Unsupported channel depth: {depth}"
        ))),
    }
}
