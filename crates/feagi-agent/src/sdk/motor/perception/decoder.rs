//! Perception decoder implementation.

use std::path::PathBuf;
use std::time::Instant;

use serde::Serialize;

use crate::core::SdkError;
use crate::sdk::base::{CorticalTopology, TopologyCache};
use crate::sdk::motor::perception::config::PerceptionDecoderConfig;
use crate::sdk::motor::traits::MotorDecoder;
use crate::sdk::types::{
    ColorChannelLayout, ColorSpace, CorticalChannelCount, CorticalChannelIndex, CorticalUnitIndex,
    FrameChangeHandling, ImageFrame, ImageFrameProperties, ImageXYResolution, MiscData,
    MiscDataDimensions, MotorCorticalUnit,
};
use feagi_sensorimotor::data_types::text_token::decode_token_id_from_misc_data_with_depth;
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use std::sync::Mutex;
use feagi_sensorimotor::caching::MotorDeviceCache;

/// Semantic segmentation frame decoded from OSEG motor output.
#[derive(Debug, Clone, Serialize)]
pub struct OsegFrame {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub channels: u32,
    pub x: Vec<u32>,
    pub y: Vec<u32>,
    pub z: Vec<u32>,
    pub p: Vec<f32>,
}

/// RGB image frame decoded from OIMG motor output.
#[derive(Debug, Clone, Serialize)]
pub struct OimgFrame {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub channels: u32,
    pub x: Vec<u32>,
    pub y: Vec<u32>,
    pub z: Vec<u32>,
    pub p: Vec<u32>,
}

/// High-level perception frame returned to controllers/UI.
#[derive(Debug, Clone, Serialize)]
pub struct PerceptionFrame {
    pub timestamp_ms: u64,
    pub source_width: Option<u32>,
    pub source_height: Option<u32>,
    pub oseg: Option<OsegFrame>,
    pub oimg: Option<OimgFrame>,
    pub oten_text: Option<String>,
    pub oten_token_id: Option<u32>,
}

/// Decoder for FEAGI motor outputs into higher-level perception frames.
pub struct PerceptionDecoder {
    config: PerceptionDecoderConfig,
    motor_cache: Mutex<MotorDeviceCache>,
    oseg_topology: CorticalTopology,
    oimg_topology: CorticalTopology,
    oten_topology: CorticalTopology,
    #[cfg(feature = "sdk-text")]
    tokenizer: Option<tokenizers::Tokenizer>,
}

impl PerceptionDecoder {
    /// Create a new perception decoder (fetches required topologies).
    pub async fn new(
        config: PerceptionDecoderConfig,
        topology_cache: &TopologyCache,
        tokenizer_path: Option<PathBuf>,
    ) -> Result<Self, SdkError> {
        let unit = CorticalUnitIndex::from(config.cortical_unit_id);
        let frame = FrameChangeHandling::Absolute;

        let oseg_id =
            MotorCorticalUnit::get_cortical_ids_array_for_object_segmentation_with_parameters(
                frame, unit,
            )[0];
        let oimg_id =
            MotorCorticalUnit::get_cortical_ids_array_for_simple_vision_output_with_parameters(
                frame, unit,
            )[0];
        let oten_id =
            MotorCorticalUnit::get_cortical_ids_array_for_text_english_output_with_parameters(
                frame, unit,
            )[0];

        let oseg_topology = topology_cache.get_topology(&oseg_id).await?;
        let oimg_topology = topology_cache.get_topology(&oimg_id).await?;
        let oten_topology = topology_cache.get_topology(&oten_id).await?;

        let mut motor_cache = MotorDeviceCache::new();
        register_oseg(
            &mut motor_cache,
            unit,
            oseg_topology,
            FrameChangeHandling::Absolute,
        )?;
        register_oimg(
            &mut motor_cache,
            unit,
            oimg_topology,
            FrameChangeHandling::Absolute,
        )?;
        register_oten(
            &mut motor_cache,
            unit,
            oten_topology,
            FrameChangeHandling::Absolute,
        )?;

        #[cfg(feature = "sdk-text")]
        let tokenizer = match tokenizer_path {
            Some(path) => Some(
                tokenizers::Tokenizer::from_file(path)
                    .map_err(|e| SdkError::Other(format!("Tokenizer load failed: {e}")))?,
            ),
            None => None,
        };

        Ok(Self {
            config,
            motor_cache: Mutex::new(motor_cache),
            oseg_topology,
            oimg_topology,
            oten_topology,
            #[cfg(feature = "sdk-text")]
            tokenizer,
        })
    }

    fn decode_oseg(&self, unit: CorticalUnitIndex) -> Result<Option<OsegFrame>, SdkError> {
        // TODO: Support multiple channels and channel-wise aggregation.
        let channel = CorticalChannelIndex::from(0u32);
        let wrapped = self
            .motor_cache
            .lock()
            .map_err(|_| SdkError::Other("OSEG cache lock poisoned".to_string()))?
            .object_segmentation_read_postprocessed_cache_value(unit, channel)
            .map_err(|e| SdkError::Other(format!("OSEG read failed: {e}")))?;
        let misc: MiscData = wrapped
            .try_into()
            .map_err(|e| SdkError::Other(format!("OSEG decode failed: {e}")))?;
        let dims = misc.get_dimensions();

        let mut x = Vec::new();
        let mut y = Vec::new();
        let mut z = Vec::new();
        let mut p = Vec::new();
        for ((xi, yi, zi), val) in misc.get_internal_data().indexed_iter() {
            if val.abs() <= f32::EPSILON {
                continue;
            }
            x.push(xi as u32);
            y.push(yi as u32);
            z.push(zi as u32);
            p.push(*val);
        }

        Ok(Some(OsegFrame {
            width: dims.width,
            height: dims.height,
            depth: dims.depth,
            channels: self.oseg_topology.channels.max(1),
            x,
            y,
            z,
            p,
        }))
    }

    fn decode_oimg(&self, unit: CorticalUnitIndex) -> Result<Option<OimgFrame>, SdkError> {
        // TODO: Support multiple channels and channel-wise aggregation.
        let channel = CorticalChannelIndex::from(0u32);
        let wrapped = self
            .motor_cache
            .lock()
            .map_err(|_| SdkError::Other("OIMG cache lock poisoned".to_string()))?
            .simple_vision_output_read_postprocessed_cache_value(unit, channel)
            .map_err(|e| SdkError::Other(format!("OIMG read failed: {e}")))?;
        let image: ImageFrame = wrapped
            .try_into()
            .map_err(|e| SdkError::Other(format!("OIMG decode failed: {e}")))?;

        let resolution = image.get_image_frame_properties().get_image_resolution();
        let mut x = Vec::new();
        let mut y = Vec::new();
        let mut z = Vec::new();
        let mut p = Vec::new();

        for ((row, col, channel), val) in image.get_internal_data().indexed_iter() {
            if *val == 0 {
                continue;
            }
            x.push(col as u32);
            y.push(row as u32);
            z.push(channel as u32);
            p.push(*val as u32);
        }

        Ok(Some(OimgFrame {
            width: resolution.width,
            height: resolution.height,
            depth: image.get_image_frame_properties().get_color_channel_layout() as u32,
            channels: self.oimg_topology.channels.max(1),
            x,
            y,
            z,
            p,
        }))
    }

    fn decode_oten(&self, unit: CorticalUnitIndex) -> Result<(Option<u32>, Option<String>), SdkError> {
        let channel = CorticalChannelIndex::from(0u32);
        let wrapped = self
            .motor_cache
            .lock()
            .map_err(|_| SdkError::Other("OTEN cache lock poisoned".to_string()))?
            .text_english_output_read_postprocessed_cache_value(unit, channel)
            .map_err(|e| SdkError::Other(format!("OTEN read failed: {e}")))?;
        let misc: MiscData = wrapped
            .try_into()
            .map_err(|e| SdkError::Other(format!("OTEN decode failed: {e}")))?;
        let token_id = decode_token_id_from_misc_data_with_depth(&misc, self.oten_topology.depth)
            .map_err(|e| SdkError::Other(format!("OTEN token decode failed: {e}")))?;

        #[cfg(feature = "sdk-text")]
        let text = if let (Some(token_id), Some(tokenizer)) = (token_id, &self.tokenizer) {
            tokenizer.decode(&[token_id], true).ok()
        } else {
            None
        };

        #[cfg(not(feature = "sdk-text"))]
        let text: Option<String> = None;

        Ok((token_id, text))
    }
}

impl MotorDecoder for PerceptionDecoder {
    type Input = CorticalMappedXYZPNeuronVoxels;
    type Output = PerceptionFrame;

    fn decode(&self, input: &Self::Input) -> Result<Self::Output, SdkError> {
        {
            let mut cache = self
                .motor_cache
                .lock()
                .map_err(|_| SdkError::Other("Motor cache lock poisoned".to_string()))?;
            cache
                .ingest_neuron_data_and_run_callbacks(input.clone(), Instant::now())
                .map_err(|e| SdkError::Other(format!("Motor cache ingest failed: {e}")))?;
        }

        let unit = CorticalUnitIndex::from(self.config.cortical_unit_id);
        let oseg = self.decode_oseg(unit)?;
        let oimg = self.decode_oimg(unit)?;
        let (oten_token_id, oten_text) = self.decode_oten(unit)?;

        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| SdkError::Other(format!("Timestamp error: {e}")))?
            .as_millis() as u64;

        Ok(PerceptionFrame {
            timestamp_ms,
            source_width: oimg.as_ref().map(|f| f.width),
            source_height: oimg.as_ref().map(|f| f.height),
            oseg,
            oimg,
            oten_text,
            oten_token_id,
        })
    }
}

fn register_oseg(
    motor_cache: &mut MotorDeviceCache,
    unit: CorticalUnitIndex,
    topology: CorticalTopology,
    frame: FrameChangeHandling,
) -> Result<(), SdkError> {
    let channels = CorticalChannelCount::new(topology.channels).map_err(|e| {
        SdkError::Other(format!("Invalid OSEG channel count: {e}"))
    })?;
    let dims =
        MiscDataDimensions::new(topology.width, topology.height, topology.depth).map_err(|e| {
            SdkError::Other(format!("Invalid OSEG dimensions: {e}"))
        })?;
    motor_cache
        .object_segmentation_register(unit, channels, frame, dims)
        .map_err(|e| SdkError::Other(format!("OSEG register failed: {e}")))
}

fn register_oimg(
    motor_cache: &mut MotorDeviceCache,
    unit: CorticalUnitIndex,
    topology: CorticalTopology,
    frame: FrameChangeHandling,
) -> Result<(), SdkError> {
    let channels = CorticalChannelCount::new(topology.channels).map_err(|e| {
        SdkError::Other(format!("Invalid OIMG channel count: {e}"))
    })?;
    let resolution = ImageXYResolution::new(topology.width, topology.height).map_err(|e| {
        SdkError::Other(format!("Invalid OIMG resolution: {e}"))
    })?;
    let layout = match topology.depth {
        1 => ColorChannelLayout::GrayScale,
        3 => ColorChannelLayout::RGB,
        _ => {
            return Err(SdkError::Other(format!(
                "Unsupported OIMG depth: {}",
                topology.depth
            )))
        }
    };
    // TODO: allow caller-configurable ColorSpace.
    let props = ImageFrameProperties::new(resolution, ColorSpace::Gamma, layout)
        .map_err(|e| SdkError::Other(format!("OIMG properties error: {e}")))?;
    motor_cache
        .simple_vision_output_register(unit, channels, frame, props)
        .map_err(|e| SdkError::Other(format!("OIMG register failed: {e}")))
}

fn register_oten(
    motor_cache: &mut MotorDeviceCache,
    unit: CorticalUnitIndex,
    topology: CorticalTopology,
    frame: FrameChangeHandling,
) -> Result<(), SdkError> {
    let channels = CorticalChannelCount::new(topology.channels).map_err(|e| {
        SdkError::Other(format!("Invalid OTEN channel count: {e}"))
    })?;
    let dims =
        MiscDataDimensions::new(topology.width, topology.height, topology.depth).map_err(|e| {
            SdkError::Other(format!("Invalid OTEN dimensions: {e}"))
        })?;
    motor_cache
        .text_english_output_register(unit, channels, frame, dims)
        .map_err(|e| SdkError::Other(format!("OTEN register failed: {e}")))
}
