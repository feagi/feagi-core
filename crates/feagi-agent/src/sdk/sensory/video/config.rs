//! Video encoder configuration.

use crate::core::{AgentConfig, AgentType, SdkError};
use feagi_io::{MotorUnit, SensoryUnit};

/// Video encoding strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoEncodingStrategy {
    /// Single-frame RGB vision stream.
    SimpleVision,
    /// Segmented vision stream (central + peripheral).
    SegmentedVision,
}

/// Configuration for the video encoder.
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
    pub feagi_tick_hz: u32,
    pub feagi_heartbeat_interval_s: f64,
    pub feagi_connection_timeout_ms: u64,
    pub feagi_registration_retries: u32,
    pub diff_threshold: u8,
    pub brightness: i32,
    pub contrast: f32,
}

impl VideoEncoderConfig {
    /// Build an AgentConfig for this video encoder.
    pub fn to_agent_config(&self) -> Result<AgentConfig, SdkError> {
        let registration_endpoint = format!(
            "tcp://{}:{}",
            self.feagi_host, self.feagi_zmq_registration_port
        );
        let sensory_endpoint = format!(
            "tcp://{}:{}",
            self.feagi_host, self.feagi_zmq_sensory_port
        );
        let motor_endpoint = format!("tcp://{}:{}", self.feagi_host, self.feagi_zmq_motor_port);

        let agent_type = AgentType::Sensory;
        Ok(AgentConfig::new(self.agent_id.clone(), agent_type)
            .with_vision_unit(
                "vision",
                (self.source_width as usize, self.source_height as usize),
                3,
                SensoryUnit::Vision,
                self.cortical_unit_id,
            )
            .with_registration_endpoint(registration_endpoint)
            .with_sensory_endpoint(sensory_endpoint)
            .with_motor_endpoint(motor_endpoint)
            .with_heartbeat_interval(self.feagi_heartbeat_interval_s)
            .with_connection_timeout_ms(self.feagi_connection_timeout_ms)
            .with_registration_retries(self.feagi_registration_retries))
    }

    /// Build an AgentConfig that enables motor feedback for segmented vision.
    pub fn to_agent_config_with_motor_feedback(&self) -> Result<AgentConfig, SdkError> {
        let registration_endpoint = format!(
            "tcp://{}:{}",
            self.feagi_host, self.feagi_zmq_registration_port
        );
        let sensory_endpoint = format!(
            "tcp://{}:{}",
            self.feagi_host, self.feagi_zmq_sensory_port
        );
        let motor_endpoint = format!("tcp://{}:{}", self.feagi_host, self.feagi_zmq_motor_port);

        let agent_type = AgentType::Both;
        Ok(AgentConfig::new(self.agent_id.clone(), agent_type)
            .with_vision_unit(
                "segmented-vision",
                (self.source_width as usize, self.source_height as usize),
                3,
                SensoryUnit::SegmentedVision,
                self.cortical_unit_id,
            )
            .with_motor_unit("gaze", 2, MotorUnit::Gaze, self.cortical_unit_id)
            .with_registration_endpoint(registration_endpoint)
            .with_sensory_endpoint(sensory_endpoint)
            .with_motor_endpoint(motor_endpoint)
            .with_heartbeat_interval(self.feagi_heartbeat_interval_s)
            .with_connection_timeout_ms(self.feagi_connection_timeout_ms)
            .with_registration_retries(self.feagi_registration_retries))
    }
}
