//! Authentication request containing agent descriptor and auth token.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::sdk::common::AgentDescriptor;
use crate::sdk::common::AuthToken;
use crate::sdk::common::FeagiAgentError;

/// An authentication request containing agent identification and credentials.
///
/// This struct is used for agent authentication with FEAGI services.
/// It can be serialized to/from JSON for network transmission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    /// The agent descriptor encoded as base64.
    agent_descriptor: String,
    /// The authentication token encoded as base64.
    auth_token: String,
}

impl AuthRequest {
    /// Create a new authentication request.
    ///
    /// # Arguments
    /// * `agent_descriptor` - The agent's descriptor
    /// * `auth_token` - The authentication token
    pub fn new(agent_descriptor: &AgentDescriptor, auth_token: &AuthToken) -> Self {
        Self {
            agent_descriptor: agent_descriptor.to_base64(),
            auth_token: auth_token.to_base64(),
        }
    }

    /// Get the agent descriptor.
    ///
    /// # Errors
    /// Returns an error if the stored base64 is invalid.
    pub fn agent_descriptor(&self) -> Result<AgentDescriptor, FeagiAgentError> {
        AgentDescriptor::try_from_base64(&self.agent_descriptor)
            .map_err(|e| FeagiAgentError::GeneralFailure(format!("Invalid agent descriptor: {}", e)))
    }

    /// Get the auth token.
    ///
    /// # Errors
    /// Returns an error if the stored base64 is invalid.
    pub fn auth_token(&self) -> Result<AuthToken, FeagiAgentError> {
        AuthToken::from_base64(&self.auth_token)
            .ok_or_else(|| FeagiAgentError::GeneralFailure("Invalid auth token: bad base64 or wrong length".to_string()))
    }

    /// Parse an AuthRequest from a JSON value.
    ///
    /// # Arguments
    /// * `json` - A JSON value to parse
    ///
    /// # Errors
    /// Returns an error if required fields are missing or invalid.
    pub fn from_json(json: &Value) -> Result<Self, FeagiAgentError> {
        let obj = json.as_object().ok_or_else(|| {
            FeagiAgentError::GeneralFailure("Expected JSON object".to_string())
        })?;

        // Extract agent_descriptor
        let agent_descriptor = obj
            .get("agent_descriptor")
            .ok_or_else(|| FeagiAgentError::GeneralFailure("Missing field: 'agent_descriptor'".to_string()))?
            .as_str()
            .ok_or_else(|| FeagiAgentError::GeneralFailure("Field 'agent_descriptor' must be a string".to_string()))?
            .to_string();

        // Validate agent_descriptor is valid base64 and correct format
        AgentDescriptor::try_from_base64(&agent_descriptor)
            .map_err(|e| FeagiAgentError::GeneralFailure(format!("Invalid agent descriptor: {}", e)))?;

        // Extract auth_token
        let auth_token = obj
            .get("auth_token")
            .ok_or_else(|| FeagiAgentError::GeneralFailure("Missing field: 'auth_token'".to_string()))?
            .as_str()
            .ok_or_else(|| FeagiAgentError::GeneralFailure("Field 'auth_token' must be a string".to_string()))?
            .to_string();

        // Validate auth_token is valid base64 and correct length
        AuthToken::from_base64(&auth_token)
            .ok_or_else(|| FeagiAgentError::GeneralFailure("Invalid auth token: bad base64 or wrong length".to_string()))?;

        Ok(Self {
            agent_descriptor,
            auth_token,
        })
    }

    /// Convert to a JSON value.
    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).expect("AuthRequest serialization should never fail")
    }
}
