// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
BrainRegion data model.

Represents a hierarchical grouping of cortical areas with functional significance.
Moved from feagi-core/crates/feagi-bdu/src/models/brain_region.rs
*/

mod region_id;
pub use region_id::RegionID;

use crate::genomic::cortical_area::CorticalID;
use crate::FeagiDataError;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Type of brain region (placeholder for future functional/anatomical classification)
///
/// Currently, no specific region types are defined. This enum serves as a placeholder
/// for future extensions when functional or anatomical classification is implemented.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RegionType {
    /// Generic/undefined region type (placeholder)
    Undefined,
}

impl Default for RegionType {
    fn default() -> Self {
        Self::Undefined
    }
}

impl std::fmt::Display for RegionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "undefined")
    }
}

/// Brain region metadata (genome representation)
///
/// A brain region is a hierarchical grouping of cortical areas that share
/// functional or anatomical characteristics. Regions form a tree structure
/// where each region can contain multiple cortical areas and sub-regions.
///
/// # Design Notes
///
/// - Regions are organizational constructs (not physical entities)
/// - Used for genome editing, visualization, and bulk operations
/// - Serializable for genome persistence
/// - Properties stored as HashMap for maximum flexibility
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainRegion {
    /// Unique identifier for this region
    pub region_id: RegionID,

    /// Human-readable name (mapped to "title" in genome JSON)
    pub name: String,

    /// Functional/anatomical type
    pub region_type: RegionType,

    /// Set of cortical area IDs contained in this region
    #[serde(default)]
    pub cortical_areas: HashSet<CorticalID>,

    /// Additional user-defined properties
    /// Commonly used keys: description, coordinate_2d, coordinate_3d, inputs, outputs, signature
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

impl BrainRegion {
    /// Create a new brain region
    ///
    /// # Arguments
    ///
    /// * `region_id` - Unique identifier (validated RegionID)
    /// * `name` - Human-readable name
    /// * `region_type` - Functional type
    ///
    /// # Errors
    ///
    /// Returns error if name is empty
    ///
    pub fn new(
        region_id: RegionID,
        name: String,
        region_type: RegionType,
    ) -> Result<Self, FeagiDataError> {
        if name.trim().is_empty() {
            return Err(FeagiDataError::BadParameters(
                "name cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            region_id,
            name,
            region_type,
            cortical_areas: HashSet::new(),
            properties: HashMap::new(),
        })
    }

    /// Create a region with initial cortical areas
    pub fn with_areas(mut self, areas: impl IntoIterator<Item = CorticalID>) -> Self {
        self.cortical_areas.extend(areas);
        self
    }

    /// Create a region with custom properties
    pub fn with_properties(mut self, properties: HashMap<String, serde_json::Value>) -> Self {
        self.properties = properties;
        self
    }

    /// Add a cortical area to this region
    ///
    /// Returns `true` if the area was newly added, `false` if it was already present
    ///
    pub fn add_area(&mut self, area_id: CorticalID) -> bool {
        self.cortical_areas.insert(area_id)
    }

    /// Remove a cortical area from this region
    ///
    /// Returns `true` if the area was present and removed, `false` if it wasn't present
    ///
    pub fn remove_area(&mut self, area_id: &CorticalID) -> bool {
        self.cortical_areas.remove(area_id)
    }

    /// Check if this region contains a specific cortical area
    pub fn contains_area(&self, area_id: &CorticalID) -> bool {
        self.cortical_areas.contains(area_id)
    }

    /// Get all cortical area IDs in this region
    pub fn get_all_areas(&self) -> Vec<&CorticalID> {
        self.cortical_areas.iter().collect()
    }

    /// Get the number of cortical areas in this region
    pub fn area_count(&self) -> usize {
        self.cortical_areas.len()
    }

    /// Clear all cortical areas from this region
    pub fn clear_areas(&mut self) {
        self.cortical_areas.clear();
    }

    /// Get a property value by key
    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }

    /// Add a property to the region
    pub fn add_property(&mut self, key: String, value: serde_json::Value) {
        self.properties.insert(key, value);
    }

    /// Convert to dictionary representation (for serialization)
    pub fn to_dict(&self) -> serde_json::Value {
        // Convert CorticalIDs to their base64 string representation for JSON
        let area_ids: Vec<String> = self
            .cortical_areas
            .iter()
            .map(|id| id.as_base_64())
            .collect();

        let mut dict = serde_json::json!({
            "id": self.region_id.to_string(),
            "name": self.name,
            "region_type": self.region_type.to_string(),
            "cortical_areas": area_ids,
        });

        // Add all properties from the HashMap
        for (key, value) in &self.properties {
            dict[key] = value.clone();
        }

        dict
    }
}
