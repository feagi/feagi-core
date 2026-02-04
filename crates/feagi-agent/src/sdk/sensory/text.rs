//! Text encoder config. SDK surface for desktop text controller.

use crate::core::AgentConfig;

/// Configuration for the text encoder.
#[derive(Debug, Clone)]
pub struct TextEncoderConfig {
    pub agent_id: String,
    pub cortical_unit_id: u8,
    pub feagi_host: String,
    pub feagi_api_port: u16,
    pub feagi_connection_timeout_ms: u64,
    pub feagi_zmq_registration_port: u16,
    pub feagi_zmq_sensory_port: u16,
    pub feagi_zmq_motor_port: u16,
    pub feagi_tick_hz: f64,
    pub feagi_heartbeat_interval_s: f64,
    pub feagi_registration_retries: u32,
}

impl TextEncoderConfig {
    pub fn to_agent_config(&self) -> Result<AgentConfig, crate::FeagiAgentClientError> {
        Ok(AgentConfig::new(
            self.agent_id.clone(),
            crate::core::AgentType::Sensory,
        ))
    }
}
