use std::fmt;
use serde::{Deserialize, Serialize};
use feagi_structures::FeagiDataError;

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
    ReceiveSystemMessages
}

//endregion

//region API Version

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FeagiApiVersion {
    version: u64
}

impl FeagiApiVersion {
    pub const fn get_current_api_version() -> Self {
        Self { version: 1 } // TODO actual logic here
    }
}

//endregion

//region Agent Descriptor
/// Describes an agent connecting to FEAGI.
///
/// Contains identification information including manufacturer, agent name,
/// version, and a unique instance ID.
///
/// All deserialization (JSON, etc.) goes through validation automatically.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentDescriptor {
    instance_id: u32,
    manufacturer: String,
    agent_name: String,
    agent_version: u32,
}

impl AgentDescriptor {

    /// Maximum length in bytes for the manufacturer field
    pub const MAX_MANUFACTURER_NAME_BYTE_COUNT: usize = 128;
    /// Maximum length in bytes for the agent name field
    pub const MAX_AGENT_NAME_BYTE_COUNT: usize = 64;

    /// Total size in bytes when serialized to binary format
    pub const SIZE_BYTES: usize = 4 + Self::MAX_MANUFACTURER_NAME_BYTE_COUNT + Self::MAX_AGENT_NAME_BYTE_COUNT + 4;

    /// Create a new AgentDescriptor with validation.
    ///
    /// # Arguments
    /// * `instance_id` - Unique instance identifier
    /// * `manufacturer` - Manufacturer name (ASCII only, max 20 bytes)
    /// * `agent_name` - Agent name (ASCII only, max 20 bytes)
    /// * `agent_version` - Version number (must be non-zero)
    ///
    /// # Errors
    /// Returns an error if:
    /// - `manufacturer` or `agent_name` contain non-ASCII characters
    /// - `manufacturer` exceeds 20 bytes
    /// - `agent_name` exceeds 20 bytes
    /// - `agent_version` is zero
    pub fn new(
        instance_id: u32,
        manufacturer: &str,
        agent_name: &str,
        agent_version: u32,
    ) -> Result<Self, FeagiDataError> {
        Self::validate(manufacturer, agent_name, agent_version)?;

        Ok(AgentDescriptor {
            instance_id,
            manufacturer: manufacturer.to_string(),
            agent_name: agent_name.to_string(),
            agent_version,
        })
    }

    /// Get the instance ID
    pub fn instance_id(&self) -> u32 {
        self.instance_id
    }

    /// Get the manufacturer name
    pub fn manufacturer(&self) -> &str {
        &self.manufacturer
    }

    /// Get the agent name
    pub fn agent_name(&self) -> &str {
        &self.agent_name
    }

    /// Get the agent version
    pub fn agent_version(&self) -> u32 {
        self.agent_version
    }

    /// Create AgentDescriptor from base64-encoded agent_id (REST API compatibility)
    /// Supports both old format (72 bytes) and new format (200 bytes) for backward compatibility
    pub fn try_from_base64(agent_id_b64: &str) -> Result<Self, FeagiDataError> {
        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(agent_id_b64)
            .map_err(|e| FeagiDataError::DeserializationError(format!("Invalid base64: {}", e)))?;
        
        // Support both old (72 bytes) and new (200 bytes) formats
        const OLD_FORMAT_SIZE: usize = 72; // 4 + 32 + 32 + 4
        const OLD_MANUFACTURER_SIZE: usize = 32;
        const OLD_AGENT_NAME_SIZE: usize = 32;
        
        let (instance_id, manufacturer, agent_name, agent_version) = if decoded.len() == OLD_FORMAT_SIZE {
            // Old format: 4 + 32 + 32 + 4 = 72 bytes
            let instance_id = u32::from_le_bytes([decoded[0], decoded[1], decoded[2], decoded[3]]);
            
            let manufacturer_bytes = &decoded[4..4 + OLD_MANUFACTURER_SIZE];
            let manufacturer = String::from_utf8_lossy(manufacturer_bytes)
                .trim_end_matches('\0')
                .to_string();
            
            let agent_name_bytes = &decoded[4 + OLD_MANUFACTURER_SIZE..4 + OLD_MANUFACTURER_SIZE + OLD_AGENT_NAME_SIZE];
            let agent_name = String::from_utf8_lossy(agent_name_bytes)
                .trim_end_matches('\0')
                .to_string();
            
            let version_offset = 4 + OLD_MANUFACTURER_SIZE + OLD_AGENT_NAME_SIZE;
            let agent_version = u32::from_le_bytes([
                decoded[version_offset],
                decoded[version_offset + 1],
                decoded[version_offset + 2],
                decoded[version_offset + 3],
            ]);
            
            (instance_id, manufacturer, agent_name, agent_version)
        } else if decoded.len() == Self::SIZE_BYTES {
            // New format: 4 + 128 + 64 + 4 = 200 bytes
            let instance_id = u32::from_le_bytes([decoded[0], decoded[1], decoded[2], decoded[3]]);
            
            let manufacturer_bytes = &decoded[4..4 + Self::MAX_MANUFACTURER_NAME_BYTE_COUNT];
            let manufacturer = String::from_utf8_lossy(manufacturer_bytes)
                .trim_end_matches('\0')
                .to_string();
            
            let agent_name_bytes = &decoded[4 + Self::MAX_MANUFACTURER_NAME_BYTE_COUNT..4 + Self::MAX_MANUFACTURER_NAME_BYTE_COUNT + Self::MAX_AGENT_NAME_BYTE_COUNT];
            let agent_name = String::from_utf8_lossy(agent_name_bytes)
                .trim_end_matches('\0')
                .to_string();
            
            let version_offset = 4 + Self::MAX_MANUFACTURER_NAME_BYTE_COUNT + Self::MAX_AGENT_NAME_BYTE_COUNT;
            let agent_version = u32::from_le_bytes([
                decoded[version_offset],
                decoded[version_offset + 1],
                decoded[version_offset + 2],
                decoded[version_offset + 3],
            ]);
            
            (instance_id, manufacturer, agent_name, agent_version)
        } else {
            return Err(FeagiDataError::DeserializationError(format!(
                "Invalid agent_id length: expected {} (new) or {} (old) bytes, got {}",
                Self::SIZE_BYTES,
                OLD_FORMAT_SIZE,
                decoded.len()
            )));
        };
        
        Self::new(instance_id, &manufacturer, &agent_name, agent_version)
    }

    /// Validate the fields without creating a new instance.
    fn validate(
        manufacturer: &str,
        agent_name: &str,
        agent_version: u32,
    ) -> Result<(), FeagiDataError> {
        if !manufacturer.is_ascii() {
            return Err(FeagiDataError::BadParameters(
                "Manufacturer must contain ASCII characters only!".to_string(),
            ));
        }
        if !agent_name.is_ascii() {
            return Err(FeagiDataError::BadParameters(
                "Agent name must contain ASCII characters only!".to_string(),
            ));
        }
        if manufacturer.len() > Self::MAX_MANUFACTURER_NAME_BYTE_COUNT {
            return Err(FeagiDataError::BadParameters(format!(
                "Manufacturer is too long! Max length is {} bytes, got {}",
                Self::MAX_MANUFACTURER_NAME_BYTE_COUNT,
                manufacturer.len()
            )));
        }
        if agent_name.len() > Self::MAX_AGENT_NAME_BYTE_COUNT {
            return Err(FeagiDataError::BadParameters(format!(
                "Agent name is too long! Max length is {} bytes, got {}",
                Self::MAX_AGENT_NAME_BYTE_COUNT,
                agent_name.len()
            )));
        }
        if agent_version == 0 {
            return Err(FeagiDataError::BadParameters(
                "Agent version cannot be zero!".to_string(),
            ));
        }
        Ok(())
    }
}

//endregion