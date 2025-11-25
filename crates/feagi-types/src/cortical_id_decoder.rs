// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Cortical ID Decoder
//! 
//! Decodes 8-byte cortical IDs (base64-encoded) into structured information
//! for IPU (Input Processing Unit) and OPU (Output Processing Unit) cortical areas.
//! 
//! Format (8 bytes total):
//! - Bytes 0-3: cortical_subtype (4 ASCII chars, e.g., "isvi", "imot", "ibat")
//! - Byte 4: encoding_flags
//!   - 0x10 bit: Incremental (if set) vs Absolute (if clear)
//!   - 0x20 bit: Fractional (if set) vs Linear (if clear)
//! - Byte 5: reserved/flags (typically 0x00)
//! - Byte 6: unit_id (0, 1, 2, ...)
//! - Byte 7: group_id (0, 1, 2, ...)

use serde::{Deserialize, Serialize};
use base64::Engine;

/// Decoded cortical ID information (only applicable to IPU/OPU areas)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DecodedCorticalId {
    /// 4-character cortical subtype (e.g., "isvi", "imot", "ibat")
    pub cortical_subtype: String,
    
    /// Encoding type: "Absolute" or "Incremental"
    pub encoding_type: String,
    
    /// Encoding format: "Linear" or "Fractional"
    pub encoding_format: String,
    
    /// Unit ID (0, 1, 2, ...)
    pub unit_id: u8,
    
    /// Group ID (0, 1, 2, ...)
    pub group_id: u8,
}

/// Decode a base64-encoded cortical ID into structured information
/// 
/// # Arguments
/// * `cortical_id_base64` - Base64-encoded cortical ID (8 bytes when decoded)
/// 
/// # Returns
/// * `Some(DecodedCorticalId)` if the ID is a valid IPU/OPU ID
/// * `None` if the ID is not IPU/OPU, is malformed, or cannot be decoded
/// 
/// # Examples
/// ```
/// use feagi_types::cortical_id_decoder::decode_cortical_id;
/// 
/// let decoded = decode_cortical_id("aXN2aQgAAAA=").unwrap();
/// assert_eq!(decoded.cortical_subtype, "isvi");
/// assert_eq!(decoded.encoding_type, "Absolute");
/// assert_eq!(decoded.encoding_format, "Linear");
/// assert_eq!(decoded.unit_id, 0);
/// assert_eq!(decoded.group_id, 0);
/// ```
pub fn decode_cortical_id(cortical_id_base64: &str) -> Option<DecodedCorticalId> {
    // Decode from base64
    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(cortical_id_base64)
        .ok()?;
    
    // Must be exactly 8 bytes
    if decoded_bytes.len() != 8 {
        return None;
    }
    
    // Extract cortical subtype (bytes 0-3)
    let cortical_subtype = String::from_utf8(decoded_bytes[0..4].to_vec()).ok()?;
    
    // Check if this is an IPU or OPU area (first char must be 'i', 'I', 'o', or 'O')
    let first_char = cortical_subtype.chars().next()?;
    if !matches!(first_char, 'i' | 'I' | 'o' | 'O') {
        return None;
    }
    
    // Decode encoding flags (byte 4)
    let encoding_flags = decoded_bytes[4];
    
    // Bit 0x10: Incremental (if set) vs Absolute (if clear)
    let encoding_type = if (encoding_flags & 0x10) != 0 {
        "Incremental"
    } else {
        "Absolute"
    };
    
    // Bit 0x20: Fractional (if set) vs Linear (if clear)
    let encoding_format = if (encoding_flags & 0x20) != 0 {
        "Fractional"
    } else {
        "Linear"
    };
    
    // Extract unit and group IDs (bytes 6-7)
    let unit_id = decoded_bytes[6];
    let group_id = decoded_bytes[7];
    
    Some(DecodedCorticalId {
        cortical_subtype,
        encoding_type: encoding_type.to_string(),
        encoding_format: encoding_format.to_string(),
        unit_id,
        group_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_decode_ipu_absolute_linear() {
        let decoded = decode_cortical_id("aW1pcwkAAAA=").unwrap();
        assert_eq!(decoded.cortical_subtype, "imis");
        assert_eq!(decoded.encoding_type, "Absolute");
        assert_eq!(decoded.encoding_format, "Linear");
        assert_eq!(decoded.unit_id, 0);
        assert_eq!(decoded.group_id, 0);
    }
    
    #[test]
    fn test_decode_ipu_incremental() {
        let decoded = decode_cortical_id("aW1vdBQAAAA=").unwrap();
        assert_eq!(decoded.cortical_subtype, "imot");
        assert_eq!(decoded.encoding_type, "Incremental");
        assert_eq!(decoded.encoding_format, "Linear");
        assert_eq!(decoded.unit_id, 0);
        assert_eq!(decoded.group_id, 0);
    }
    
    #[test]
    fn test_decode_with_unit_id() {
        let decoded = decode_cortical_id("aXN2aQgAAQA=").unwrap();
        assert_eq!(decoded.cortical_subtype, "isvi");
        assert_eq!(decoded.encoding_type, "Absolute");
        assert_eq!(decoded.unit_id, 1);
        assert_eq!(decoded.group_id, 0);
    }
    
    #[test]
    fn test_decode_multiple_units() {
        let decoded2 = decode_cortical_id("aXN2aQgAAgA=").unwrap();
        assert_eq!(decoded2.unit_id, 2);
        
        let decoded8 = decode_cortical_id("aXN2aQgACAA=").unwrap();
        assert_eq!(decoded8.unit_id, 8);
    }
    
    #[test]
    fn test_decode_non_ipu_opu_returns_none() {
        // Custom area starting with 'c'
        assert!(decode_cortical_id("Y19sbWFvAA==").is_none());
        
        // Core area starting with '_'
        assert!(decode_cortical_id("X19fZGVhdGg=").is_none());
    }
    
    #[test]
    fn test_decode_invalid_base64_returns_none() {
        assert!(decode_cortical_id("invalid!!!").is_none());
    }
    
    #[test]
    fn test_decode_wrong_length_returns_none() {
        // Only 4 bytes when decoded
        assert!(decode_cortical_id("aXN2aQ==").is_none());
    }
}

