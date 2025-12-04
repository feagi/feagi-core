// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
CorticalArea data structure (genome representation).

Pure data definition - no business logic.
Transformation methods live in feagi-bdu.
Moved from feagi-core/crates/feagi-bdu/src/models/cortical_area.rs
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::FeagiDataError;
use super::{CorticalAreaDimensions, CorticalID};

/// Type of cortical area (functional classification)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AreaType {
    /// Sensory input areas
    Sensory,
    /// Motor output areas
    Motor,
    /// Memory/association areas
    Memory,
    /// Custom/user-defined areas
    Custom,
}

impl Default for AreaType {
    fn default() -> Self {
        Self::Custom
    }
}

impl std::fmt::Display for AreaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sensory => write!(f, "sensory"),
            Self::Motor => write!(f, "motor"),
            Self::Memory => write!(f, "memory"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

/// Cortical area metadata (genome representation)
///
/// Pure data structure containing static genome metadata.
/// Runtime operations and transformations are implemented in feagi-bdu.
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorticalArea {
    /// Unique typed cortical identifier
    pub cortical_id: CorticalID,

    /// Integer index assigned by ConnectomeManager
    pub cortical_idx: u32,

    /// Human-readable name
    pub name: String,

    /// 3D dimensions (width, height, depth in voxels)
    pub dimensions: CorticalAreaDimensions,

    /// 3D position in brain space (can be negative)
    pub position: (i32, i32, i32),

    /// Functional type of this area
    pub area_type: AreaType,

    /// Additional user-defined properties
    /// Note: neurons_per_voxel is stored here
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

impl CorticalArea {
    /// Create a new cortical area with validation
    ///
    /// # Arguments
    ///
    /// * `cortical_id` - Unique typed cortical identifier
    /// * `cortical_idx` - Integer index for fast lookups
    /// * `name` - Human-readable name
    /// * `dimensions` - 3D dimensions (width, height, depth)
    /// * `position` - 3D position in brain space
    /// * `area_type` - Functional type
    ///
    /// # Errors
    ///
    /// Returns error if name is empty
    ///
    pub fn new(
        cortical_id: CorticalID,
        cortical_idx: u32,
        name: String,
        dimensions: CorticalAreaDimensions,
        position: (i32, i32, i32),
        area_type: AreaType,
    ) -> Result<Self, FeagiDataError> {
        // Validate name
        if name.is_empty() {
            return Err(FeagiDataError::BadParameters(
                "name cannot be empty".to_string(),
            ));
        }

        // Note: CorticalID validation happens in CorticalID constructors
        // Note: dimensions validation happens in CorticalAreaDimensions::new()

        Ok(Self {
            cortical_id,
            cortical_idx,
            name,
            dimensions,
            position,
            area_type,
            properties: HashMap::new(),
        })
    }

    /// Get a property value by key
    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }

    /// Get the total number of voxels in this area
    pub fn total_voxels(&self) -> u32 {
        self.dimensions.total_voxels()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cortical_area_creation() {
        let dims = CorticalAreaDimensions::new(128, 128, 20).unwrap();
        let cortical_id = CorticalID::try_from_base_64("iav001").unwrap();
        let area = CorticalArea::new(
            cortical_id,
            0,
            "Visual Input".to_string(),
            dims,
            (0, 0, 0),
            AreaType::Sensory,
        )
        .unwrap();

        assert_eq!(area.cortical_id.as_base_64(), "iav001");
        assert_eq!(area.name, "Visual Input");
        assert_eq!(area.total_voxels(), 128 * 128 * 20);
    }

    #[test]
    fn test_invalid_cortical_id() {
        let dims = CorticalAreaDimensions::new(10, 10, 10).unwrap();
        // CorticalID validation happens in constructor, so this will fail at CorticalID creation
        let cortical_id_result = CorticalID::try_from_base_64("short");
        assert!(cortical_id_result.is_err());
    }

    #[test]
    fn test_properties() {
        let dims = CorticalAreaDimensions::new(10, 10, 10).unwrap();
        let cortical_id = CorticalID::try_from_base_64("test03").unwrap();
        let mut area = CorticalArea::new(
            cortical_id,
            0,
            "Test".to_string(),
            dims,
            (0, 0, 0),
            AreaType::Sensory,
        )
        .unwrap();

        area.properties.insert("resolution".to_string(), serde_json::json!(128));
        area.properties.insert("modality".to_string(), serde_json::json!("visual"));
        area.properties.insert("neurons_per_voxel".to_string(), serde_json::json!(1));

        assert_eq!(area.get_property("resolution"), Some(&serde_json::json!(128)));
        assert_eq!(
            area.get_property("modality"),
            Some(&serde_json::json!("visual"))
        );
        assert_eq!(area.get_property("neurons_per_voxel"), Some(&serde_json::json!(1)));
        assert_eq!(area.get_property("nonexistent"), None);
    }
}
