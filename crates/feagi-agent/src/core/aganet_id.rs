use base64::{engine::general_purpose, Engine as _};
use feagi_structures::FeagiDataError;

const MAX_MANUFACTURER_LENGTH: usize = 20;
const MAX_AGENT_NAME_LENGTH: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentID {
    instance_id: u32,
    manufacturer: [u8; MAX_MANUFACTURER_LENGTH], //ASCII
    agent_name: [u8; MAX_AGENT_NAME_LENGTH], //ASCII
    agent_version: u32,
}

impl AgentID {
    /// Total size in bytes of the AgentID structure
    pub const SIZE_BYTES: usize =
        4 + MAX_MANUFACTURER_LENGTH + MAX_AGENT_NAME_LENGTH + 4; // instance_id + manufacturer + agent_name + agent_version

    pub fn new(
        instance_id: u32,
        manufacturer: &str,
        agent_name: &str,
        agent_version: u32,
    ) -> Result<Self, FeagiDataError> {
        if !manufacturer.is_ascii() || !agent_name.is_ascii() {
            return Err(FeagiDataError::BadParameters(
                "ASCII characters only!".to_string(),
            ));
        }

        if manufacturer.len() > MAX_MANUFACTURER_LENGTH {
            return Err(FeagiDataError::BadParameters(format!(
                "Manufacturer is too long! Max length is {} characters!",
                MAX_MANUFACTURER_LENGTH
            )));
        }
        if agent_name.len() > MAX_AGENT_NAME_LENGTH {
            return Err(FeagiDataError::BadParameters(format!(
                "Agent name is too long! Max length is {} characters!",
                MAX_AGENT_NAME_LENGTH
            )));
        }
        if agent_version == 0 {
            return Err(FeagiDataError::BadParameters(
                "Agent Version cannot be zero!".to_string(),
            ));
        }

        // Create fixed-size arrays padded with null bytes
        let mut manufacturer_bytes = [0u8; MAX_MANUFACTURER_LENGTH];
        manufacturer_bytes[..manufacturer.len()].copy_from_slice(manufacturer.as_bytes());

        let mut agent_name_bytes = [0u8; MAX_AGENT_NAME_LENGTH];
        agent_name_bytes[..agent_name.len()].copy_from_slice(agent_name.as_bytes());

        Ok(AgentID {
            instance_id,
            manufacturer: manufacturer_bytes,
            agent_name: agent_name_bytes,
            agent_version,
        })
    }

    /// Get the instance ID
    pub fn instance_id(&self) -> u32 {
        self.instance_id
    }

    /// Get the manufacturer name as a string slice (without null padding)
    pub fn manufacturer(&self) -> &str {
        let end = self
            .manufacturer
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(MAX_MANUFACTURER_LENGTH);
        // Safe: we validated ASCII in constructor
        std::str::from_utf8(&self.manufacturer[..end]).unwrap_or("")
    }

    /// Get the agent name as a string slice (without null padding)
    pub fn agent_name(&self) -> &str {
        let end = self
            .agent_name
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(MAX_AGENT_NAME_LENGTH);
        // Safe: we validated ASCII in constructor
        std::str::from_utf8(&self.agent_name[..end]).unwrap_or("")
    }

    /// Get the agent version
    pub fn agent_version(&self) -> u32 {
        self.agent_version
    }

    /// Get the raw manufacturer bytes (including null padding)
    pub fn manufacturer_bytes(&self) -> &[u8; MAX_MANUFACTURER_LENGTH] {
        &self.manufacturer
    }

    /// Get the raw agent name bytes (including null padding)
    pub fn agent_name_bytes(&self) -> &[u8; MAX_AGENT_NAME_LENGTH] {
        &self.agent_name
    }

    /// Serialize to a fixed-size byte array
    pub fn to_bytes(&self) -> [u8; Self::SIZE_BYTES] {
        let mut bytes = [0u8; Self::SIZE_BYTES];
        let mut offset = 0;

        // instance_id (4 bytes, little-endian)
        bytes[offset..offset + 4].copy_from_slice(&self.instance_id.to_le_bytes());
        offset += 4;

        // manufacturer (MAX_MANUFACTURER_LENGTH bytes)
        bytes[offset..offset + MAX_MANUFACTURER_LENGTH].copy_from_slice(&self.manufacturer);
        offset += MAX_MANUFACTURER_LENGTH;

        // agent_name (MAX_AGENT_NAME_LENGTH bytes)
        bytes[offset..offset + MAX_AGENT_NAME_LENGTH].copy_from_slice(&self.agent_name);
        offset += MAX_AGENT_NAME_LENGTH;

        // agent_version (4 bytes, little-endian)
        bytes[offset..offset + 4].copy_from_slice(&self.agent_version.to_le_bytes());

        bytes
    }

    /// Deserialize from a fixed-size byte array
    pub fn from_bytes(bytes: &[u8; Self::SIZE_BYTES]) -> Result<Self, FeagiDataError> {
        let mut offset = 0;

        // instance_id (4 bytes, little-endian)
        let instance_id = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset += 4;

        // manufacturer (MAX_MANUFACTURER_LENGTH bytes)
        let mut manufacturer = [0u8; MAX_MANUFACTURER_LENGTH];
        manufacturer.copy_from_slice(&bytes[offset..offset + MAX_MANUFACTURER_LENGTH]);
        offset += MAX_MANUFACTURER_LENGTH;

        // Validate manufacturer is ASCII
        for &b in &manufacturer {
            if b != 0 && !b.is_ascii() {
                return Err(FeagiDataError::DeserializationError(
                    "Manufacturer contains non-ASCII characters".to_string(),
                ));
            }
        }

        // agent_name (MAX_AGENT_NAME_LENGTH bytes)
        let mut agent_name = [0u8; MAX_AGENT_NAME_LENGTH];
        agent_name.copy_from_slice(&bytes[offset..offset + MAX_AGENT_NAME_LENGTH]);
        offset += MAX_AGENT_NAME_LENGTH;

        // Validate agent_name is ASCII
        for &b in &agent_name {
            if b != 0 && !b.is_ascii() {
                return Err(FeagiDataError::DeserializationError(
                    "Agent name contains non-ASCII characters".to_string(),
                ));
            }
        }

        // agent_version (4 bytes, little-endian)
        let agent_version = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());

        if agent_version == 0 {
            return Err(FeagiDataError::DeserializationError(
                "Agent Version cannot be zero!".to_string(),
            ));
        }

        Ok(AgentID {
            instance_id,
            manufacturer,
            agent_name,
            agent_version,
        })
    }

    /// Encode the AgentID to a base64 string
    pub fn to_base64(&self) -> String {
        general_purpose::STANDARD.encode(self.to_bytes())
    }

    /// Try to decode an AgentID from a base64 string
    pub fn try_from_base64(encoded: &str) -> Result<Self, FeagiDataError> {
        let decoded = general_purpose::STANDARD.decode(encoded).map_err(|e| {
            FeagiDataError::DeserializationError(format!(
                "Failed to decode base64 string: {}",
                e
            ))
        })?;

        if decoded.len() != Self::SIZE_BYTES {
            return Err(FeagiDataError::DeserializationError(format!(
                "Invalid AgentID length: expected {} bytes, got {}",
                Self::SIZE_BYTES,
                decoded.len()
            )));
        }

        let mut bytes = [0u8; Self::SIZE_BYTES];
        bytes.copy_from_slice(&decoded);
        Self::from_bytes(&bytes)
    }
}
