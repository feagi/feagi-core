// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Text encoder configuration

use crate::core::{AgentConfig, AgentType};
use crate::sdk::error::{Result, SdkError};
use feagi_structures::genomic::cortical_area::descriptors::{CorticalUnitIndex, CorticalSubUnitIndex};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{FrameChangeHandling, IOCorticalAreaConfigurationFlag};
use feagi_structures::genomic::cortical_area::CorticalID;

/// Text encoder configuration
///
/// # Example
/// ```ignore
/// use feagi_agent::sdk::sensory::text::TextEncoderConfig;
///
/// let config = TextEncoderConfig {
///     agent_id: "text-input-01".to_string(),
///     cortical_unit_id: 0,
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
    pub cortical_unit_id: u8,  // Cortical unit index (which unit of this type)

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
    /// 
    /// Uses the proper feagi-structures method to generate cortical ID
    pub fn cortical_id(&self) -> CorticalID {
        // Text input uses Misc data type with Absolute frame handling
        let data_flag = IOCorticalAreaConfigurationFlag::Misc(FrameChangeHandling::Absolute);
        
        data_flag.as_io_cortical_id(
            true,                                          // is_input
            [b't', b'e', b'n'],                           // "ten" in iten
            CorticalUnitIndex::from(self.cortical_unit_id), // unit index
            CorticalSubUnitIndex::from(0),                // subunit index (always 0 for text)
        )
    }

    /// Convert to AgentConfig for core client
    pub fn to_agent_config(&self) -> Result<AgentConfig> {
        // Device registrations are handled separately via ConnectorAgent and
        // device_registrations in capabilities.
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
            .with_sensory_capability(self.feagi_tick_hz as f64, None);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cortical_id_generation() {
        // Test group 0
        let config0 = TextEncoderConfig {
            agent_id: "test".to_string(),
            cortical_unit_id: 0,
            feagi_host: "localhost".to_string(),
            feagi_api_port: 8080,
            feagi_zmq_registration_port: 30001,
            feagi_zmq_sensory_port: 5555,
            feagi_tick_hz: 60,
            feagi_heartbeat_interval_s: 5.0,
            feagi_connection_timeout_ms: 5000,
            feagi_registration_retries: 3,
        };
        let id0 = config0.cortical_id();
        println!("Group 0: {}", id0.as_base_64());
        
        // Test group 1 - should match genome
        let config1 = TextEncoderConfig {
            agent_id: "test".to_string(),
            cortical_unit_id: 1,
            feagi_host: "localhost".to_string(),
            feagi_api_port: 8080,
            feagi_zmq_registration_port: 30001,
            feagi_zmq_sensory_port: 5555,
            feagi_tick_hz: 60,
            feagi_heartbeat_interval_s: 5.0,
            feagi_connection_timeout_ms: 5000,
            feagi_registration_retries: 3,
        };
        let id1 = config1.cortical_id();
        println!("Group 1: {} (expected: aXRlbgEAAAA=)", id1.as_base_64());
        assert_eq!(id1.as_base_64(), "aXRlbgEAAAA=", "Group 1 cortical ID should match genome");
    }
}

