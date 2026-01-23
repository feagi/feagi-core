//! Connection identifier for tracking registered agents.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Fixed length for connection identifiers (32 bytes = 256 bits)
pub const CONNECTION_ID_LENGTH: usize = 32;

/// A unique identifier for a registered connection.
///
/// This is currently derived from the auth token, but will be expanded
/// in future to include additional metadata for connection tracking.
///
/// The ID value is masked in `Debug` output for security.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ConnectionId {
    value: [u8; CONNECTION_ID_LENGTH],
}

impl ConnectionId {
    /// Create a new connection ID from a fixed-length byte array.
    pub fn new(value: [u8; CONNECTION_ID_LENGTH]) -> Self {
        Self { value }
    }

    /// Generate a random connection ID.
    pub fn generate() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let mut value = [0u8; CONNECTION_ID_LENGTH];
        
        // Use timestamp for first 8 bytes
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        value[0..8].copy_from_slice(&timestamp.to_le_bytes());
        
        // Fill rest with pseudo-random data based on memory addresses and timestamp
        let ptr = &value as *const _ as u64;
        value[8..16].copy_from_slice(&ptr.to_le_bytes());
        
        // XOR with additional entropy
        let entropy = timestamp.wrapping_mul(ptr).wrapping_add(0x517cc1b727220a95);
        for (i, chunk) in value[16..].chunks_mut(8).enumerate() {
            let mixed = entropy.wrapping_mul((i + 1) as u64);
            let bytes = mixed.to_le_bytes();
            for (j, byte) in chunk.iter_mut().enumerate() {
                if j < bytes.len() {
                    *byte = bytes[j];
                }
            }
        }
        
        Self { value }
    }

    /// Create from a hex string (64 characters for 32 bytes).
    ///
    /// # Errors
    /// Returns `None` if the string is not valid hex or wrong length.
    pub fn from_hex(hex: &str) -> Option<Self> {
        if hex.len() != CONNECTION_ID_LENGTH * 2 {
            return None;
        }
        
        let mut value = [0u8; CONNECTION_ID_LENGTH];
        for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
            let hex_byte = std::str::from_utf8(chunk).ok()?;
            value[i] = u8::from_str_radix(hex_byte, 16).ok()?;
        }
        Some(Self { value })
    }

    /// Create from a base64 string.
    ///
    /// # Errors
    /// Returns `None` if the string is not valid base64 or wrong length.
    pub fn from_base64(b64: &str) -> Option<Self> {
        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD.decode(b64).ok()?;
        if decoded.len() != CONNECTION_ID_LENGTH {
            return None;
        }
        let mut value = [0u8; CONNECTION_ID_LENGTH];
        value.copy_from_slice(&decoded);
        Some(Self { value })
    }

    /// Get the raw ID bytes.
    pub fn as_bytes(&self) -> &[u8; CONNECTION_ID_LENGTH] {
        &self.value
    }

    /// Convert to hex string (64 characters).
    pub fn to_hex(&self) -> String {
        self.value.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Convert to base64 string.
    pub fn to_base64(&self) -> String {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(&self.value)
    }
}

// Custom Debug impl that masks the ID value
impl fmt::Debug for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectionId")
            .field("value", &self.to_string())
            .finish()
    }
}

// Display shows a shortened representation
impl fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex = self.to_hex();
        write!(f, "{}...{}", &hex[..4], &hex[hex.len() - 4..])
    }
}
