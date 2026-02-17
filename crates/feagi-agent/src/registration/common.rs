use serde::{Deserialize, Serialize};
use std::fmt;

//region Auth Token
/// Fixed length for authentication tokens (32 bytes = 256 bits)
pub const AUTH_TOKEN_LENGTH: usize = 32;

/// A secure authentication token of fixed length.
///
/// The token value is masked in `Debug` output to prevent accidental exposure in logs.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthToken {
    value: [u8; AUTH_TOKEN_LENGTH],
}

impl AuthToken {
    /// Create a new auth token from a fixed-length byte array.
    pub fn new(value: [u8; AUTH_TOKEN_LENGTH]) -> Self {
        Self { value }
    }

    /// Create a token from a base64 string.
    ///
    /// # Errors
    /// Returns `None` if the string is not valid base64 or wrong length.
    pub fn from_base64(b64: &str) -> Option<Self> {
        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD.decode(b64).ok()?;
        if decoded.len() != AUTH_TOKEN_LENGTH {
            return None;
        }
        let mut value = [0u8; AUTH_TOKEN_LENGTH];
        value.copy_from_slice(&decoded);
        Some(Self { value })
    }

    /// Get the raw token bytes.
    ///
    /// **Warning**: This exposes the actual token. Use carefully and avoid logging.
    pub fn as_bytes(&self) -> &[u8; AUTH_TOKEN_LENGTH] {
        &self.value
    }

    /// Convert to base64 string.
    pub fn to_base64(&self) -> String {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(self.value)
    }
}

// Custom Debug impl that masks the token value
impl fmt::Debug for AuthToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthToken")
            .field("value", &"[REDACTED]")
            .finish()
    }
}

// Display shows a masked representation
impl fmt::Display for AuthToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let base64 = self.to_base64();
        write!(f, "{}...{}", &base64[..4], &base64[base64.len() - 4..])
    }
}

//endregion

//region Agent Capabilities

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentCapabilities {
    SendSensorData,
    ReceiveMotorData,
    ReceiveNeuronVisualizations,
}

//endregion
