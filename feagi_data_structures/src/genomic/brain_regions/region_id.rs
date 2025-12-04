// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
RegionID - UUID-based unique identifier for brain regions.

Provides type safety and global uniqueness for brain region identifiers.
*/

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use uuid::Uuid;
use crate::FeagiDataError;

/// Unique identifier for a brain region, based on UUID v7.
///
/// This struct provides type safety and ensures global uniqueness for brain region IDs.
/// UUID v7 is time-ordered, which provides better database indexing and sortability.
/// It handles serialization to and deserialization from string representations of UUIDs.
///
/// # Examples
///
/// ```
/// use feagi_data_structures::genomic::brain_regions::RegionID;
///
/// // Generate a new time-ordered RegionID
/// let region_id = RegionID::new();
///
/// // Convert to string for storage/display
/// let id_string = region_id.to_string();
///
/// // Parse from string
/// let parsed_id = RegionID::from_string(&id_string).unwrap();
/// assert_eq!(region_id, parsed_id);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegionID {
    uuid: Uuid,
}

impl RegionID {
    /// Generates a new, time-ordered RegionID (UUID v7).
    ///
    /// UUID v7 uses a timestamp-based approach, providing natural sorting
    /// and better database performance compared to random UUIDs.
    ///
    /// # Examples
    ///
    /// ```
    /// use feagi_data_structures::genomic::brain_regions::RegionID;
    ///
    /// let region_id = RegionID::new();
    /// assert_ne!(region_id.to_string(), "");
    /// ```
    pub fn new() -> Self {
        Self { uuid: Uuid::now_v7() }
    }

    /// Creates a RegionID from a UUID.
    ///
    /// # Examples
    ///
    /// ```
    /// use feagi_data_structures::genomic::brain_regions::RegionID;
    /// use uuid::Uuid;
    ///
    /// let uuid = Uuid::now_v7();
    /// let region_id = RegionID::from_uuid(uuid);
    /// assert_eq!(region_id.as_uuid(), uuid);
    /// ```
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self { uuid }
    }

    /// Tries to create a RegionID from a string.
    ///
    /// Returns an error if the string is not a valid UUID.
    ///
    /// # Examples
    ///
    /// ```
    /// use feagi_data_structures::genomic::brain_regions::RegionID;
    ///
    /// let region_id = RegionID::from_string("550e8400-e29b-41d4-a716-446655440000").unwrap();
    /// assert_eq!(region_id.to_string(), "550e8400-e29b-41d4-a716-446655440000");
    /// ```
    pub fn from_string(s: &str) -> Result<Self, FeagiDataError> {
        Uuid::parse_str(s)
            .map(RegionID::from_uuid)
            .map_err(|e| FeagiDataError::BadParameters(format!("Invalid RegionID string: {}", e)))
    }

    /// Returns the underlying UUID.
    ///
    /// # Examples
    ///
    /// ```
    /// use feagi_data_structures::genomic::brain_regions::RegionID;
    ///
    /// let region_id = RegionID::new();
    /// let uuid = region_id.as_uuid();
    /// ```
    pub fn as_uuid(&self) -> Uuid {
        self.uuid
    }

    /// Returns the byte representation of the UUID.
    ///
    /// # Examples
    ///
    /// ```
    /// use feagi_data_structures::genomic::brain_regions::RegionID;
    ///
    /// let region_id = RegionID::new();
    /// let bytes = region_id.as_bytes();
    /// assert_eq!(bytes.len(), 16);
    /// ```
    pub fn as_bytes(&self) -> &[u8; 16] {
        self.uuid.as_bytes()
    }
}

impl Default for RegionID {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for RegionID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.uuid)
    }
}

impl FromStr for RegionID {
    type Err = FeagiDataError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s)
            .map(RegionID::from_uuid)
            .map_err(|e| FeagiDataError::BadParameters(format!("Invalid RegionID string: {}", e)))
    }
}

// Implement From<Uuid> for RegionID
impl From<Uuid> for RegionID {
    fn from(uuid: Uuid) -> Self {
        RegionID::from_uuid(uuid)
    }
}

// Implement From<RegionID> for Uuid
impl From<RegionID> for Uuid {
    fn from(region_id: RegionID) -> Self {
        region_id.uuid
    }
}

// Implement Serialize for RegionID
impl Serialize for RegionID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.uuid.to_string())
    }
}

// Implement Deserialize for RegionID
impl<'de> Deserialize<'de> for RegionID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        RegionID::from_string(&s)
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_id_new() {
        let id1 = RegionID::new();
        let id2 = RegionID::new();
        
        // Each new ID should be unique
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_region_id_from_uuid() {
        let uuid = Uuid::now_v7();
        let region_id = RegionID::from_uuid(uuid);
        
        assert_eq!(region_id.as_uuid(), uuid);
    }

    #[test]
    fn test_region_id_from_string() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let region_id = RegionID::from_string(uuid_str).unwrap();
        
        assert_eq!(region_id.to_string(), uuid_str);
    }

    #[test]
    fn test_region_id_from_string_invalid() {
        let result = RegionID::from_string("not-a-uuid");
        assert!(result.is_err());
    }

    #[test]
    fn test_region_id_display() {
        let region_id = RegionID::new();
        let display_str = region_id.to_string();
        
        // Should be a valid UUID string (36 characters with dashes)
        assert_eq!(display_str.len(), 36);
        assert!(display_str.contains('-'));
    }

    #[test]
    fn test_region_id_from_str() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let region_id: RegionID = uuid_str.parse().unwrap();
        
        assert_eq!(region_id.to_string(), uuid_str);
    }

    #[test]
    fn test_region_id_serialization() {
        let region_id = RegionID::new();
        
        // Serialize to JSON
        let json = serde_json::to_string(&region_id).unwrap();
        
        // Should be a quoted UUID string
        assert!(json.starts_with('"'));
        assert!(json.ends_with('"'));
        assert_eq!(json.len(), 38); // 36 + 2 quotes
    }

    #[test]
    fn test_region_id_deserialization() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let json = format!("\"{}\"", uuid_str);
        
        let region_id: RegionID = serde_json::from_str(&json).unwrap();
        
        assert_eq!(region_id.to_string(), uuid_str);
    }

    #[test]
    fn test_region_id_roundtrip() {
        let original = RegionID::new();
        
        // Serialize
        let json = serde_json::to_string(&original).unwrap();
        
        // Deserialize
        let deserialized: RegionID = serde_json::from_str(&json).unwrap();
        
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_region_id_as_bytes() {
        let region_id = RegionID::new();
        let bytes = region_id.as_bytes();
        
        assert_eq!(bytes.len(), 16);
    }

    #[test]
    fn test_region_id_default() {
        let id1 = RegionID::default();
        let id2 = RegionID::default();
        
        // Each default should generate a new unique ID
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_region_id_equality() {
        let uuid = Uuid::now_v7();
        let id1 = RegionID::from_uuid(uuid);
        let id2 = RegionID::from_uuid(uuid);
        
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_region_id_hash() {
        use std::collections::HashSet;
        
        let id1 = RegionID::new();
        let id2 = RegionID::new();
        
        let mut set = HashSet::new();
        set.insert(id1);
        set.insert(id2);
        
        assert_eq!(set.len(), 2);
        assert!(set.contains(&id1));
        assert!(set.contains(&id2));
    }
}
