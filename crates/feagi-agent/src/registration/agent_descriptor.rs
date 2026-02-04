use base64::Engine;
use feagi_structures::FeagiDataError;
use serde::{Deserialize, Serialize};

/// Maximum length in bytes for the manufacturer field
pub const MAX_MANUFACTURER_NAME_BYTE_COUNT: usize = 32;
/// Maximum length in bytes for the agent name field
pub const MAX_AGENT_NAME_BYTE_COUNT: usize = 32;

/// Raw intermediate struct for deserialization (no validation)
#[derive(Deserialize)]
struct AgentDescriptorRaw {
    instance_id: u32,
    manufacturer: String,
    agent_name: String,
    agent_version: u32,
}

/// Describes an agent connecting to FEAGI.
///
/// Contains identification information including manufacturer, agent name,
/// version, and a unique instance ID.
///
/// All deserialization (JSON, etc.) goes through validation automatically.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "AgentDescriptorRaw")]
pub struct AgentDescriptor {
    instance_id: u32,
    manufacturer: String,
    agent_name: String,
    agent_version: u32,
}

impl TryFrom<AgentDescriptorRaw> for AgentDescriptor {
    type Error = String;

    fn try_from(raw: AgentDescriptorRaw) -> Result<Self, Self::Error> {
        AgentDescriptor::validate(&raw.manufacturer, &raw.agent_name, raw.agent_version)
            .map_err(|e| e.to_string())?;

        Ok(AgentDescriptor {
            instance_id: raw.instance_id,
            manufacturer: raw.manufacturer,
            agent_name: raw.agent_name,
            agent_version: raw.agent_version,
        })
    }
}

impl AgentDescriptor {
    /// Total size in bytes when serialized to binary format
    pub const SIZE_BYTES: usize = 4 + MAX_MANUFACTURER_NAME_BYTE_COUNT + MAX_AGENT_NAME_BYTE_COUNT + 4;

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

    /// Encode the descriptor into a fixed-width base64 string.
    ///
    /// Format: u32 instance_id + manufacturer (padded) + agent_name (padded) + u32 agent_version.
    pub fn to_base64(&self) -> String {
        let mut bytes = vec![0u8; Self::SIZE_BYTES];
        let mut offset = 0;

        bytes[offset..offset + 4].copy_from_slice(&self.instance_id.to_le_bytes());
        offset += 4;

        let manufacturer_bytes = self.manufacturer.as_bytes();
        let manufacturer_len = manufacturer_bytes.len().min(MAX_MANUFACTURER_NAME_BYTE_COUNT);
        bytes[offset..offset + manufacturer_len].copy_from_slice(&manufacturer_bytes[..manufacturer_len]);
        offset += MAX_MANUFACTURER_NAME_BYTE_COUNT;

        let agent_name_bytes = self.agent_name.as_bytes();
        let agent_name_len = agent_name_bytes.len().min(MAX_AGENT_NAME_BYTE_COUNT);
        bytes[offset..offset + agent_name_len].copy_from_slice(&agent_name_bytes[..agent_name_len]);
        offset += MAX_AGENT_NAME_BYTE_COUNT;

        bytes[offset..offset + 4].copy_from_slice(&self.agent_version.to_le_bytes());

        base64::engine::general_purpose::STANDARD.encode(bytes)
    }

    /// Decode a base64 AgentDescriptor.
    pub fn try_from_base64(encoded: &str) -> Result<Self, FeagiDataError> {
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|e| FeagiDataError::BadParameters(format!("Invalid base64: {e}")))?;

        if decoded.len() != Self::SIZE_BYTES {
            return Err(FeagiDataError::BadParameters(format!(
                "Invalid AgentDescriptor payload size: expected {}, got {}",
                Self::SIZE_BYTES,
                decoded.len()
            )));
        }

        let mut offset = 0;
        let instance_id = u32::from_le_bytes(decoded[offset..offset + 4].try_into().unwrap());
        offset += 4;

        let manufacturer_raw = &decoded[offset..offset + MAX_MANUFACTURER_NAME_BYTE_COUNT];
        offset += MAX_MANUFACTURER_NAME_BYTE_COUNT;
        let agent_name_raw = &decoded[offset..offset + MAX_AGENT_NAME_BYTE_COUNT];
        offset += MAX_AGENT_NAME_BYTE_COUNT;

        let agent_version = u32::from_le_bytes(decoded[offset..offset + 4].try_into().unwrap());

        let manufacturer = extract_padded_ascii(manufacturer_raw)?;
        let agent_name = extract_padded_ascii(agent_name_raw)?;

        Self::validate(&manufacturer, &agent_name, agent_version)?;

        Ok(Self {
            instance_id,
            manufacturer,
            agent_name,
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
        if manufacturer.len() > MAX_MANUFACTURER_NAME_BYTE_COUNT {
            return Err(FeagiDataError::BadParameters(format!(
                "Manufacturer is too long! Max length is {} bytes, got {}",
                MAX_MANUFACTURER_NAME_BYTE_COUNT,
                manufacturer.len()
            )));
        }
        if agent_name.len() > MAX_AGENT_NAME_BYTE_COUNT {
            return Err(FeagiDataError::BadParameters(format!(
                "Agent name is too long! Max length is {} bytes, got {}",
                MAX_AGENT_NAME_BYTE_COUNT,
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

fn extract_padded_ascii(bytes: &[u8]) -> Result<String, FeagiDataError> {
    let trimmed = bytes
        .iter()
        .copied()
        .take_while(|b| *b != 0)
        .collect::<Vec<u8>>();
    let value = String::from_utf8(trimmed).map_err(|e| {
        FeagiDataError::BadParameters(format!("Invalid UTF-8 in AgentDescriptor: {e}"))
    })?;
    if !value.is_ascii() {
        return Err(FeagiDataError::BadParameters(
            "AgentDescriptor fields must be ASCII".to_string(),
        ));
    }
    Ok(value)
}
