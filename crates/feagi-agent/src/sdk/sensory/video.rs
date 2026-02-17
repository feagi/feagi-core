//! Video encoder config and strategy. SDK surface for desktop video controller.

use crate::core::AgentConfig;
use crate::sdk::base::TopologyClient;

/// Strategy for vision encoding (simple or segmented).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoEncodingStrategy {
    SimpleVision,
    SegmentedVision,
}

/// Configuration for the video encoder (resolution, FEAGI endpoints, etc.).
#[derive(Debug, Clone)]
pub struct VideoEncoderConfig {
    pub agent_id: String,
    pub cortical_unit_id: u8,
    pub encoding_strategy: VideoEncodingStrategy,
    pub source_width: u32,
    pub source_height: u32,
    pub feagi_host: String,
    pub feagi_api_port: u16,
    pub feagi_zmq_registration_port: u16,
    pub feagi_zmq_sensory_port: u16,
    pub feagi_zmq_motor_port: u16,
    pub feagi_tick_hz: f64,
    pub feagi_heartbeat_interval_s: f64,
    pub feagi_connection_timeout_ms: u64,
    pub feagi_registration_retries: u32,
    pub sensory_send_hwm: u32,
    pub sensory_linger_ms: u32,
    pub sensory_immediate: bool,
    pub diff_threshold: u8,
    pub brightness: u8,
    pub contrast: f32,
}

impl VideoEncoderConfig {
    pub fn to_agent_config(&self) -> Result<AgentConfig, crate::FeagiAgentClientError> {
        Ok(AgentConfig::new(
            self.agent_id.clone(),
            crate::core::AgentType::Sensory,
        ))
    }

    pub fn to_agent_config_with_motor_feedback(
        &self,
    ) -> Result<AgentConfig, crate::FeagiAgentClientError> {
        Ok(AgentConfig::new(
            self.agent_id.clone(),
            crate::core::AgentType::Both,
        ))
    }
}

/// Video encoder. Stub for desktop; real implementation can wrap feagi_sensorimotor pipeline.
#[derive(Debug)]
pub struct VideoEncoder {
    _config: VideoEncoderConfig,
}

impl VideoEncoder {
    pub async fn new(
        config: VideoEncoderConfig,
        _topology_cache: &TopologyClient,
    ) -> Result<Self, crate::FeagiAgentClientError> {
        Ok(Self { _config: config })
    }

    pub fn apply_gaze(
        &mut self,
        _gaze_x: f32,
        _gaze_y: f32,
        _gaze_size: f32,
    ) -> Result<(), crate::FeagiAgentClientError> {
        Ok(())
    }

    pub fn is_segmented_vision(&self) -> bool {
        self._config.encoding_strategy == VideoEncodingStrategy::SegmentedVision
    }

    pub fn set_gaze_properties(
        &mut self,
        _gaze: &feagi_sensorimotor::data_types::GazeProperties,
    ) -> Result<(), crate::FeagiAgentClientError> {
        Ok(())
    }

    pub fn set_gaze(
        &mut self,
        _x: f32,
        _y: f32,
        _modulation: f32,
    ) -> Result<(), crate::FeagiAgentClientError> {
        Ok(())
    }

    pub fn set_brightness(&mut self, _brightness: u8) -> Result<(), crate::FeagiAgentClientError> {
        Ok(())
    }

    pub fn set_contrast(&mut self, _contrast: f32) -> Result<(), crate::FeagiAgentClientError> {
        Ok(())
    }

    pub fn set_diff_threshold(
        &mut self,
        _threshold: u8,
    ) -> Result<(), crate::FeagiAgentClientError> {
        Ok(())
    }
}

impl crate::sdk::sensory::traits::SensoryEncoder for VideoEncoder {
    fn encode(
        &mut self,
        _frame: &feagi_sensorimotor::data_types::ImageFrame,
    ) -> Result<Vec<u8>, crate::FeagiAgentClientError> {
        Ok(Vec::new())
    }
}
