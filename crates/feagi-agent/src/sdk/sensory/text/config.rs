//! Text encoder configuration.

use crate::core::{AgentConfig, AgentType, SdkError};
use crate::sdk::types::{CorticalID, CorticalUnitIndex, FrameChangeHandling, SensoryCorticalUnit};

/// Configuration for the text encoder.
#[derive(Debug, Clone)]
pub struct TextEncoderConfig {
    pub agent_id: String,
    pub cortical_unit_id: u8,
    pub feagi_host: String,
    pub feagi_api_port: u16,
    pub feagi_zmq_registration_port: u16,
    pub feagi_zmq_sensory_port: u16,
    pub feagi_tick_hz: u32,
    pub feagi_heartbeat_interval_s: f64,
    pub feagi_connection_timeout_ms: u64,
    pub feagi_registration_retries: u32,
}

impl TextEncoderConfig {
    /// Build an AgentConfig for this text encoder.
    pub fn to_agent_config(&self) -> Result<AgentConfig, SdkError> {
        let registration_endpoint = format!(
            "tcp://{}:{}",
            self.feagi_host, self.feagi_zmq_registration_port
        );
        let sensory_endpoint = format!(
            "tcp://{}:{}",
            self.feagi_host, self.feagi_zmq_sensory_port
        );

        let agent_type = AgentType::Sensory;
        Ok(AgentConfig::new(self.agent_id.clone(), agent_type)
            .with_sensory_capability(self.feagi_tick_hz as f64, None)
            .with_registration_endpoint(registration_endpoint)
            .with_sensory_endpoint(sensory_endpoint)
            .with_heartbeat_interval(self.feagi_heartbeat_interval_s)
            .with_connection_timeout_ms(self.feagi_connection_timeout_ms)
            .with_registration_retries(self.feagi_registration_retries))
    }

    /// Cortical ID used for text input.
    pub fn cortical_id(&self) -> CorticalID {
        SensoryCorticalUnit::get_cortical_ids_array_for_text_english_input_with_parameters(
            FrameChangeHandling::Absolute,
            CorticalUnitIndex::from(self.cortical_unit_id),
        )[0]
    }
}
