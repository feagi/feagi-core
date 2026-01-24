//! Perception decoder configuration.

use crate::core::{AgentConfig, AgentType, SdkError};
use crate::sdk::types::{
    CorticalID, CorticalSubUnitIndex, CorticalUnitIndex, FrameChangeHandling,
    IOCorticalAreaConfigurationFlag, MotorCorticalUnit,
};
use feagi_io::{MotorUnit, MotorUnitSpec};

/// Configuration for the perception decoder.
#[derive(Debug, Clone)]
pub struct PerceptionDecoderConfig {
    pub agent_id: String,
    pub cortical_unit_id: u8,
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
    /// Build an AgentConfig for this perception decoder.
    pub fn to_agent_config(&self) -> Result<AgentConfig, SdkError> {
        let registration_endpoint = format!(
            "tcp://{}:{}",
            self.feagi_host, self.feagi_zmq_registration_port
        );
        let sensory_endpoint = format!(
            "tcp://{}:{}",
            self.feagi_host, self.feagi_zmq_agent_sensory_port
        );
        let motor_endpoint = format!("tcp://{}:{}", self.feagi_host, self.feagi_zmq_motor_port);

        let agent_type = AgentType::Motor;
        let group = self.cortical_unit_id;
        let motor_units = vec![
            MotorUnitSpec {
                unit: MotorUnit::ObjectSegmentation,
                group,
            },
            MotorUnitSpec {
                unit: MotorUnit::SimpleVisionOutput,
                group,
            },
            MotorUnitSpec {
                unit: MotorUnit::TextEnglishOutput,
                group,
            },
        ];
        let output_count = 1;

        Ok(AgentConfig::new(self.agent_id.clone(), agent_type)
            .with_motor_units("perception", output_count, motor_units)
            .with_registration_endpoint(registration_endpoint)
            .with_sensory_endpoint(sensory_endpoint)
            .with_motor_endpoint(motor_endpoint)
            .with_heartbeat_interval(self.feagi_heartbeat_interval_s)
            .with_connection_timeout_ms(self.feagi_connection_timeout_ms)
            .with_registration_retries(self.feagi_registration_retries))
    }

    /// Returns the cortical IDs used by this decoder (oseg, oimg, oten).
    pub fn cortical_ids(&self) -> Vec<CorticalID> {
        let unit = CorticalUnitIndex::from(self.cortical_unit_id);
        let frame = FrameChangeHandling::Absolute;

        let oseg =
            MotorCorticalUnit::get_cortical_ids_array_for_object_segmentation_with_parameters(
                frame, unit,
            )
            .first()
            .copied();

        let oimg =
            MotorCorticalUnit::get_cortical_ids_array_for_simple_vision_output_with_parameters(
                frame, unit,
            )
            .first()
            .copied();

        let oten =
            MotorCorticalUnit::get_cortical_ids_array_for_text_english_output_with_parameters(
                frame, unit,
            )
            .first()
            .copied();

        [oseg, oimg, oten].into_iter().flatten().collect()
    }

    /// Returns the cortical ID for OTEN output.
    pub fn oten_cortical_id(&self) -> CorticalID {
        IOCorticalAreaConfigurationFlag::Misc(FrameChangeHandling::Absolute).as_io_cortical_id(
            false,
            *b"ten",
            CorticalUnitIndex::from(self.cortical_unit_id),
            CorticalSubUnitIndex::from(0u8),
        )
    }
}
