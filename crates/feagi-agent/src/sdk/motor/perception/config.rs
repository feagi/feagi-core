// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Perception decoder configuration

use crate::core::{AgentConfig, AgentType};
use crate::sdk::error::{Result, SdkError};
use feagi_structures::genomic::cortical_area::descriptors::CorticalUnitIndex;
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::FrameChangeHandling;
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::genomic::MotorCorticalUnit;

/// Perception decoder configuration
///
/// # Example
/// ```ignore
/// use feagi_agent::sdk::motor::perception::PerceptionDecoderConfig;
///
/// let config = PerceptionDecoderConfig {
///     agent_id: "perception-inspector".to_string(),
///     cortical_unit_id: 0,
///     feagi_host: "localhost".to_string(),
///     feagi_api_port: 8080,
///     feagi_zmq_registration_port: 30001,
///     feagi_zmq_agent_sensory_port: 5555,
///     feagi_zmq_motor_port: 5564,
///     feagi_heartbeat_interval_s: 5.0,
///     feagi_connection_timeout_ms: 5000,
///     feagi_registration_retries: 3,
///     feagi_motor_poll_interval_s: 0.01,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct PerceptionDecoderConfig {
    // Identity
    pub agent_id: String,
    pub cortical_unit_id: u8,  // Cortical unit index (which unit of this type)

    // FEAGI network configuration
    pub feagi_host: String,
    pub feagi_api_port: u16,
    pub feagi_zmq_registration_port: u16,
    pub feagi_zmq_agent_sensory_port: u16,
    pub feagi_zmq_motor_port: u16,
    pub feagi_heartbeat_interval_s: f64,
    pub feagi_connection_timeout_ms: u64,
    pub feagi_registration_retries: u32,
    pub feagi_motor_poll_interval_s: f64,
}

impl PerceptionDecoderConfig {
    /// Get cortical IDs for perception (oseg, oimg, oten)
    pub fn cortical_ids(&self) -> [CorticalID; 3] {
        let unit_index = CorticalUnitIndex::from(self.cortical_unit_id);
        [
            MotorCorticalUnit::get_cortical_ids_array_for_object_segmentation_with_parameters(
                FrameChangeHandling::Absolute,
                unit_index,
            )[0],
            MotorCorticalUnit::get_cortical_ids_array_for_simple_vision_output_with_parameters(
                FrameChangeHandling::Absolute,
                unit_index,
            )[0],
            MotorCorticalUnit::get_cortical_ids_array_for_text_english_output_with_parameters(
                FrameChangeHandling::Absolute,
                unit_index,
            )[0],
        ]
    }

    /// Convert to AgentConfig for core client
    pub fn to_agent_config(&self) -> Result<AgentConfig> {
        let cortical_ids = self.cortical_ids();
        let output_count = cortical_ids.len();

        let group = self.cortical_unit_id;

        let config = AgentConfig::new(self.agent_id.clone(), AgentType::Motor)
            .with_registration_endpoint(format!(
                "tcp://{}:{}",
                self.feagi_host, self.feagi_zmq_registration_port
            ))
            .with_sensory_endpoint(format!(
                "tcp://{}:{}",
                self.feagi_host, self.feagi_zmq_agent_sensory_port
            ))
            .with_motor_endpoint(format!(
                "tcp://{}:{}",
                self.feagi_host, self.feagi_zmq_motor_port
            ))
            .with_heartbeat_interval(self.feagi_heartbeat_interval_s)
            .with_connection_timeout_ms(self.feagi_connection_timeout_ms)
            .with_registration_retries(self.feagi_registration_retries)
            .with_motor_units(
                "motor",
                output_count,
                vec![
                    feagi_io::MotorUnitSpec {
                        unit: feagi_io::MotorUnit::ObjectSegmentation,
                        group,
                    },
                    feagi_io::MotorUnitSpec {
                        unit: feagi_io::MotorUnit::SimpleVisionOutput,
                        group,
                    },
                    feagi_io::MotorUnitSpec {
                        unit: feagi_io::MotorUnit::TextEnglishOutput,
                        group,
                    },
                ],
            );

        Ok(config)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.agent_id.is_empty() {
            return Err(SdkError::InvalidConfiguration(
                "agent_id cannot be empty".to_string(),
            ));
        }

        Ok(())
    }
}

