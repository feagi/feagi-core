/*!
BrainRegion data model.

Represents a hierarchical grouping of cortical areas with functional significance.
*/

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::types::{BduError, BduResult};

/// Type of brain region (functional/anatomical classification)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RegionType {
    /// Sensory processing regions
    Sensory,
    /// Motor control regions
    Motor,
    /// Memory and association regions
    Memory,
    /// Custom/user-defined regions
    Custom,
}

impl Default for RegionType {
    fn default() -> Self {
        Self::Custom
    }
}

impl std::fmt::Display for RegionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sensory => write!(f, "sensory"),
            Self::Motor => write!(f, "motor"),
            Self::Memory => write!(f, "memory"),
            Self::Custom => write!(f, "custom"),
        }
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
    pub region_id: String,

    /// Human-readable name
    pub name: String,

    /// Functional/anatomical type
    pub region_type: RegionType,

    /// Set of cortical area IDs contained in this region
    #[serde(default)]
    pub cortical_areas: HashSet<String>,

    /// Additional user-defined properties
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

impl BrainRegion {
    /// Create a new brain region
    ///
    /// # Arguments
    ///
    /// * `region_id` - Unique identifier
    /// * `name` - Human-readable name
    /// * `region_type` - Functional type
    ///
    /// # Errors
    ///
    /// Returns error if region_id or name is empty
    ///
    pub fn new(region_id: String, name: String, region_type: RegionType) -> BduResult<Self> {
        if region_id.is_empty() {
            return Err(BduError::InvalidArea(
                "region_id cannot be empty".to_string(),
            ));
        }

        if name.is_empty() {
            return Err(BduError::InvalidArea(
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
    pub fn with_areas(mut self, areas: impl IntoIterator<Item = String>) -> Self {
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
    pub fn add_area(&mut self, area_id: String) -> bool {
        self.cortical_areas.insert(area_id)
    }

    /// Remove a cortical area from this region
    ///
    /// Returns `true` if the area was present and removed, `false` if it wasn't present
    ///
    pub fn remove_area(&mut self, area_id: &str) -> bool {
        self.cortical_areas.remove(area_id)
    }

    /// Check if this region contains a specific cortical area
    pub fn contains_area(&self, area_id: &str) -> bool {
        self.cortical_areas.contains(area_id)
    }

    /// Get all cortical area IDs in this region
    pub fn get_all_areas(&self) -> Vec<&String> {
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

    /// Add a property
    pub fn add_property(&mut self, key: String, value: serde_json::Value) {
        self.properties.insert(key, value);
    }

    /// Get a property value by key
    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }

    /// Update multiple properties
    pub fn update_properties(&mut self, updates: HashMap<String, serde_json::Value>) {
        self.properties.extend(updates);
    }

    /// Convert to dictionary representation (for serialization)
    pub fn to_dict(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.region_id,
            "name": self.name,
            "region_type": self.region_type.to_string(),
            "cortical_areas": self.cortical_areas.iter().collect::<Vec<_>>(),
            "properties": self.properties,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brain_region_creation() {
        let region = BrainRegion::new(
            "visual_cortex".to_string(),
            "Visual Cortex".to_string(),
            RegionType::Sensory,
        )
        .unwrap();

        assert_eq!(region.region_id, "visual_cortex");
        assert_eq!(region.name, "Visual Cortex");
        assert_eq!(region.region_type, RegionType::Sensory);
        assert_eq!(region.area_count(), 0);
    }

    #[test]
    fn test_add_remove_areas() {
        let mut region = BrainRegion::new(
            "test_region".to_string(),
            "Test".to_string(),
            RegionType::Custom,
        )
        .unwrap();

        // Add areas
        assert!(region.add_area("area1".to_string()));
        assert!(region.add_area("area2".to_string()));
        assert!(!region.add_area("area1".to_string())); // Already exists

        assert_eq!(region.area_count(), 2);
        assert!(region.contains_area("area1"));
        assert!(region.contains_area("area2"));

        // Remove area
        assert!(region.remove_area("area1"));
        assert!(!region.remove_area("area1")); // Already removed

        assert_eq!(region.area_count(), 1);
        assert!(!region.contains_area("area1"));
        assert!(region.contains_area("area2"));
    }

    #[test]
    fn test_with_areas() {
        let region = BrainRegion::new(
            "test".to_string(),
            "Test".to_string(),
            RegionType::Custom,
        )
        .unwrap()
        .with_areas(vec!["area1".to_string(), "area2".to_string(), "area3".to_string()]);

        assert_eq!(region.area_count(), 3);
        assert!(region.contains_area("area1"));
        assert!(region.contains_area("area2"));
        assert!(region.contains_area("area3"));
    }

    #[test]
    fn test_properties() {
        let mut region = BrainRegion::new(
            "test".to_string(),
            "Test".to_string(),
            RegionType::Sensory,
        )
        .unwrap();

        region.add_property("modality".to_string(), serde_json::json!("visual"));
        region.add_property("layer".to_string(), serde_json::json!(4));

        assert_eq!(
            region.get_property("modality"),
            Some(&serde_json::json!("visual"))
        );
        assert_eq!(region.get_property("layer"), Some(&serde_json::json!(4)));
    }

    #[test]
    fn test_clear_areas() {
        let mut region = BrainRegion::new(
            "test".to_string(),
            "Test".to_string(),
            RegionType::Custom,
        )
        .unwrap()
        .with_areas(vec!["area1".to_string(), "area2".to_string()]);

        assert_eq!(region.area_count(), 2);

        region.clear_areas();
        assert_eq!(region.area_count(), 0);
    }

    #[test]
    fn test_serialization() {
        let region = BrainRegion::new(
            "test".to_string(),
            "Test Region".to_string(),
            RegionType::Memory,
        )
        .unwrap()
        .with_areas(vec!["area1".to_string()]);

        // Serialize to JSON
        let json = serde_json::to_string(&region).unwrap();

        // Deserialize back
        let deserialized: BrainRegion = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.region_id, "test");
        assert_eq!(deserialized.name, "Test Region");
        assert_eq!(deserialized.region_type, RegionType::Memory);
        assert!(deserialized.contains_area("area1"));
    }
}





