//! Perception decoder config and frame. SDK surface for desktop perception inspector.

use crate::core::{AgentConfig, AgentType};
use feagi_structures::genomic::cortical_area::CorticalID;
use serde::Serialize;

/// Configuration for the perception decoder.
#[derive(Debug, Clone)]
pub struct PerceptionDecoderConfig {
    pub agent_id: String,
    pub cortical_unit_id: u8,
    pub feagi_host: String,
    pub feagi_api_port: u16,
    pub feagi_connection_timeout_ms: u64,
    pub feagi_motor_poll_interval_s: f64,
    pub feagi_zmq_registration_port: u16,
    pub feagi_zmq_agent_sensory_port: u16,
    pub feagi_zmq_motor_port: u16,
    pub feagi_heartbeat_interval_s: f64,
    pub feagi_registration_retries: u32,
    pub cortical_ids: Vec<CorticalID>,
}

impl PerceptionDecoderConfig {
    pub fn to_agent_config(&self) -> Result<AgentConfig, crate::FeagiAgentClientError> {
        Ok(AgentConfig::new(self.agent_id.clone(), AgentType::Motor)
            .with_registration_endpoint(format!("tcp://{}:{}", self.feagi_host, self.feagi_api_port)))
    }

    pub fn cortical_ids(&self) -> &[CorticalID] {
        &self.cortical_ids
    }
}

/// Decoded perception frame (OSEG, OIMG, OTEN, etc.).
#[derive(Debug, Clone, Default, Serialize)]
pub struct PerceptionFrame {
    pub oseg: Option<Vec<u8>>,
    pub oimg: Option<Vec<u8>>,
    pub oten: Option<Vec<u8>>,
    pub oten_token_id: Option<u32>,
    pub oten_text: Option<String>,
}

/// Perception decoder. Stub for desktop; real implementation can wrap feagi_sensorimotor.
pub struct PerceptionDecoder {
    _config: PerceptionDecoderConfig,
}

impl PerceptionDecoder {
    pub async fn new(
        config: PerceptionDecoderConfig,
        _topology_cache: &TopologyClient,
        _tokenizer_path: Option<std::path::PathBuf>,
    ) -> Result<Self, crate::FeagiAgentClientError> {
        Ok(Self { _config: config })
    }

    pub fn tick(&mut self) -> Result<Option<PerceptionFrame>, crate::FeagiAgentClientError> {
        Ok(None)
    }

    /// Returns (oseg_avail, oimg_avail, oten_avail).
    pub fn available_areas(&self) -> (bool, bool, bool) {
        let n = self._config.cortical_ids.len();
        (n > 0, n > 1, n > 2)
    }
}

/// Type alias for topology client used by perception decoder (same as base::TopologyClient).
pub use crate::sdk::base::TopologyClient;

impl crate::sdk::motor::traits::MotorDecoder for PerceptionDecoder {
    fn decode(
        &mut self,
        _motor_data: &crate::sdk::types::CorticalMappedXYZPNeuronVoxels,
    ) -> Result<Option<PerceptionFrame>, crate::FeagiAgentClientError> {
        Ok(None)
    }
}
