//! Phase 2 registration response (capability endpoints).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::sdk::common::AgentCapabilities;

/// Response sent back after successful phase 2 registration.
/// Contains endpoint addresses for each requested capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase2Response {
    /// Map of capability to endpoint address.
    pub endpoints: HashMap<AgentCapabilities, String>,
}

impl Phase2Response {
    /// Create a new phase 2 response with placeholder endpoints.
    pub fn new(capabilities: &[AgentCapabilities]) -> Self {
        let mut endpoints = HashMap::new();
        for cap in capabilities {
            // TODO: Replace placeholders with actual endpoint addresses
            let placeholder = match cap {
                AgentCapabilities::SendSensorData => "tcp://placeholder:5001",
                AgentCapabilities::ReceiveMotorData => "tcp://placeholder:5002",
                AgentCapabilities::ReceiveNeuronVisualizations => "tcp://placeholder:5003",
            };
            endpoints.insert(*cap, placeholder.to_string());
        }
        Self { endpoints }
    }

    /// Serialize to JSON bytes.
    pub fn to_json_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Phase2Response serialization should never fail")
    }
}
