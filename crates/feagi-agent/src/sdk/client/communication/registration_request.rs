//! Registration request sent by agent to complete registration (phase 2).

use serde::{Deserialize, Serialize};

use crate::sdk::common::{AgentCapabilities, ConnectionId, FeagiAgentError};

/// Request sent by agent to complete registration (phase 2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationRequest {
    /// The connection ID received from phase 1 (base64 encoded).
    pub connection_id: String,
    /// Generic JSON data for future use.
    pub data: serde_json::Value,
    /// The capabilities this agent requires.
    pub capabilities: Vec<AgentCapabilities>,
}

impl RegistrationRequest {
    /// Parse from a JSON value.
    pub fn from_json(json: &serde_json::Value) -> Result<Self, FeagiAgentError> {
        serde_json::from_value(json.clone())
            .map_err(|e| FeagiAgentError::GeneralFailure(format!("Invalid RegistrationRequest: {}", e)))
    }

    /// Get the connection ID.
    pub fn connection_id(&self) -> Result<ConnectionId, FeagiAgentError> {
        ConnectionId::from_base64(&self.connection_id)
            .ok_or_else(|| FeagiAgentError::GeneralFailure("Invalid connection_id: bad base64 or wrong length".to_string()))
    }
}
