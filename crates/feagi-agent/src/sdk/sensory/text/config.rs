// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Text encoder configuration

use crate::core::{AgentConfig, AgentType};
use crate::sdk::error::{Result, SdkError};
use feagi_structures::genomic::cortical_area::descriptors::CorticalGroupIndex;
use feagi_structures::genomic::cortical_area::io_cortical_area_data_type::FrameChangeHandling;
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::genomic::SensoryCorticalUnit;
use std::collections::HashMap;

/// Text encoder configuration
///
/// # Example
/// ```ignore
/// use feagi_agent::sdk::sensory::text::TextEncoderConfig;
///
/// let config = TextEncoderConfig {
///     agent_id: "text-input-01".to_string(),
///     cortical_group_id: 0,
///     feagi_host: "localhost".to_string(),
///     feagi_api_port: 8080,
///     feagi_zmq_registration_port: 30001,
///     feagi_zmq_sensory_port: 5555,
///     feagi_tick_hz: 60,
///     feagi_heartbeat_interval_s: 5.0,
///     feagi_connection_timeout_ms: 5000,
///     feagi_registration_retries: 3,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct TextEncoderConfig {
    // Identity
    pub agent_id: String,
    pub cortical_group_id: u8,

    // FEAGI network configuration
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
    /// Get cortical ID for text input (iten)
    pub fn cortical_id(&self) -> CorticalID {
        let group_index = CorticalGroupIndex::from(self.cortical_group_id);
        SensoryCorticalUnit::get_cortical_ids_array_for_text_english_input(
            FrameChangeHandling::Absolute,
            group_index,
        )[0]
    }

    /// Convert to AgentConfig for core client
    pub fn to_agent_config(&self) -> Result<AgentConfig> {
        let cortical_id = self.cortical_id();

        // Build cortical mappings for registration
        let mut cortical_mappings = HashMap::new();
        cortical_mappings.insert(cortical_id.as_base_64(), 0);

        let config = AgentConfig::new(self.agent_id.clone(), AgentType::Sensory)
            .with_registration_endpoint(format!(
                "tcp://{}:{}",
                self.feagi_host, self.feagi_zmq_registration_port
            ))
            .with_sensory_endpoint(format!(
                "tcp://{}:{}",
                self.feagi_host, self.feagi_zmq_sensory_port
            ))
            .with_heartbeat_interval(self.feagi_heartbeat_interval_s)
            .with_connection_timeout_ms(self.feagi_connection_timeout_ms)
            .with_registration_retries(self.feagi_registration_retries)
            .with_sensory_capability(self.feagi_tick_hz as f64, None, cortical_mappings);

        Ok(config)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.agent_id.is_empty() {
            return Err(SdkError::InvalidConfiguration(
                "agent_id cannot be empty".to_string(),
            ));
        }

        if self.feagi_tick_hz == 0 {
            return Err(SdkError::InvalidConfiguration(
                "feagi_tick_hz must be > 0".to_string(),
            ));
        }

        Ok(())
    }
}

