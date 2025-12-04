// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
BrainRegion data model.

Represents a hierarchical grouping of cortical areas with functional significance.
Moved from feagi-core/crates/feagi-bdu/src/models/brain_region.rs
*/

mod region_id;
pub use region_id::RegionID;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use crate::FeagiDataError;
use crate::genomic::cortical_area::CorticalID;
use crate::genomic::descriptors::{GenomeCoordinate2D, GenomeCoordinate3D};

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

/// Properties and metadata for a brain region
///
/// Contains hierarchical relationships, visualization coordinates, and I/O tracking
/// for a brain region. All fields are optional to allow for partial specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrainRegionProperties {
    /// Human-readable description of the region's purpose
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Parent region ID for hierarchical organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_region_id: Option<RegionID>,

    /// Child region IDs for hierarchical organization
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub child_regions: Vec<RegionID>,

    /// 2D visualization coordinates in genome space
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coordinate_2d: Option<GenomeCoordinate2D>,

    /// 3D visualization coordinates in genome space
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coordinate_3d: Option<GenomeCoordinate3D>,

    /// Input cortical area IDs (areas that provide input to this region)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<CorticalID>,

    /// Output cortical area IDs (areas that receive output from this region)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<CorticalID>,

    /// Optional signature for region identification
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl Default for BrainRegionProperties {
    fn default() -> Self {
        Self {
            description: None,
            parent_region_id: None,
            child_regions: Vec::new(),
            coordinate_2d: None,
            coordinate_3d: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            signature: None,
        }
    }
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

    /// Region properties and metadata
    #[serde(default, flatten)]
    pub properties: BrainRegionProperties,
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
    pub fn new(region_id: RegionID, name: String, region_type: RegionType) -> Result<Self, FeagiDataError> {
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
            properties: BrainRegionProperties::default(),
        })
    }

    /// Create a region with initial cortical areas
    pub fn with_areas(mut self, areas: impl IntoIterator<Item = CorticalID>) -> Self {
        self.cortical_areas.extend(areas);
        self
    }

    /// Create a region with custom properties
    pub fn with_properties(mut self, properties: BrainRegionProperties) -> Self {
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

    /// Set the description for this region
    pub fn set_description(&mut self, description: String) {
        self.properties.description = Some(description);
    }

    /// Set the parent region ID
    pub fn set_parent_region(&mut self, parent_id: RegionID) {
        self.properties.parent_region_id = Some(parent_id);
    }

    /// Add a child region ID
    pub fn add_child_region(&mut self, child_id: RegionID) {
        if !self.properties.child_regions.contains(&child_id) {
            self.properties.child_regions.push(child_id);
        }
    }

    /// Set 2D visualization coordinates in genome space
    pub fn set_coordinate_2d(&mut self, coord: GenomeCoordinate2D) {
        self.properties.coordinate_2d = Some(coord);
    }

    /// Set 3D visualization coordinates in genome space
    pub fn set_coordinate_3d(&mut self, coord: GenomeCoordinate3D) {
        self.properties.coordinate_3d = Some(coord);
    }

    /// Add an input area
    pub fn add_input(&mut self, area_id: CorticalID) {
        if !self.properties.inputs.contains(&area_id) {
            self.properties.inputs.push(area_id);
        }
    }

    /// Add an output area
    pub fn add_output(&mut self, area_id: CorticalID) {
        if !self.properties.outputs.contains(&area_id) {
            self.properties.outputs.push(area_id);
        }
    }

    /// Convert to dictionary representation (for serialization)
    pub fn to_dict(&self) -> serde_json::Value {
        // Convert CorticalIDs to their base64 string representation for JSON
        let area_ids: Vec<String> = self.cortical_areas.iter()
            .map(|id| id.as_base_64())
            .collect();
        
        let inputs: Vec<String> = self.properties.inputs.iter()
            .map(|id| id.as_base_64())
            .collect();
        
        let outputs: Vec<String> = self.properties.outputs.iter()
            .map(|id| id.as_base_64())
            .collect();
        
        let mut dict = serde_json::json!({
            "id": self.region_id.to_string(),
            "name": self.name,
            "region_type": self.region_type.to_string(),
            "cortical_areas": area_ids,
        });
        
        // Add optional properties if present
        if let Some(ref desc) = self.properties.description {
            dict["description"] = serde_json::json!(desc);
        }
        if let Some(ref parent) = self.properties.parent_region_id {
            dict["parent_region_id"] = serde_json::json!(parent.to_string());
        }
        if !self.properties.child_regions.is_empty() {
            let child_ids: Vec<String> = self.properties.child_regions.iter()
                .map(|id| id.to_string())
                .collect();
            dict["child_regions"] = serde_json::json!(child_ids);
        }
        if let Some(ref coord) = self.properties.coordinate_2d {
            dict["coordinate_2d"] = serde_json::json!([coord.x, coord.y]);
        }
        if let Some(ref coord) = self.properties.coordinate_3d {
            dict["coordinate_3d"] = serde_json::json!([coord.x, coord.y, coord.z]);
        }
        if !inputs.is_empty() {
            dict["inputs"] = serde_json::json!(inputs);
        }
        if !outputs.is_empty() {
            dict["outputs"] = serde_json::json!(outputs);
        }
        if let Some(ref sig) = self.properties.signature {
            dict["signature"] = serde_json::json!(sig);
        }
        
        dict
    }
}
