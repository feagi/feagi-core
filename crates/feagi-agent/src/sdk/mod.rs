use serde_json::Value;

use crate::FeagiAgentClientError;

/// Minimal ConnectorAgent for device registration storage in REST flows.
#[derive(Debug, Clone)]
pub struct ConnectorAgent {
    agent_descriptor: AgentDescriptor,
    device_registrations: Value,
}

impl ConnectorAgent {
    pub fn new_from_device_registration_json(
        agent_descriptor: AgentDescriptor,
        device_registrations: Value,
    ) -> Result<Self, FeagiAgentClientError> {
        if !device_registrations.is_object() {
            return Err(FeagiAgentClientError::UnableToDecodeReceivedData(
                "device_registrations must be a JSON object".to_string(),
            ));
        }
        Ok(Self {
            agent_descriptor,
            device_registrations,
        })
    }

    pub fn new_empty(agent_descriptor: AgentDescriptor) -> Self {
        Self {
            agent_descriptor,
            device_registrations: serde_json::json!({}),
        }
    }

    pub fn set_device_registrations_from_json(
        &mut self,
        device_registrations: Value,
    ) -> Result<(), FeagiAgentClientError> {
        if !device_registrations.is_object() {
            return Err(FeagiAgentClientError::UnableToDecodeReceivedData(
                "device_registrations must be a JSON object".to_string(),
            ));
        }
        self.device_registrations = device_registrations;
        Ok(())
    }

    pub fn get_device_registration_json(&self) -> Result<Value, FeagiAgentClientError> {
        Ok(self.device_registrations.clone())
    }

    pub fn agent_descriptor(&self) -> &AgentDescriptor {
        &self.agent_descriptor
    }
}

pub use crate::registration::AuthToken;
pub use crate::registration::AgentDescriptor;
