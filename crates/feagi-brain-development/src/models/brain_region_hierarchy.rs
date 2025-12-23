// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
BrainRegionHierarchy - Tree structure for organizing brain regions.

Manages parent-child relationships between brain regions, enabling hierarchical
organization for genome editing and visualization.
*/

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::types::{BduError, BduResult};
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::genomic::BrainRegion;

/// Hierarchical tree structure for brain regions
///
/// Maintains parent-child relationships between regions and provides methods
/// for tree traversal, validation, and manipulation.
///
/// # Design Notes
///
/// - Root region has parent_id = None
/// - Each region can have multiple children
/// - Cycles are prevented by validation
/// - Fast lookups via HashMap
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainRegionHierarchy {
    /// Map of region_id -> BrainRegion
    regions: HashMap<String, BrainRegion>,

    /// Map of region_id -> parent_region_id
    #[serde(default)]
    parent_map: HashMap<String, String>,

    /// Map of region_id -> child_region_ids
    #[serde(default)]
    children_map: HashMap<String, HashSet<String>>,

    /// ID of the root region (typically "root")
    root_id: Option<String>,
}

impl BrainRegionHierarchy {
    /// Create a new empty hierarchy
    pub fn new() -> Self {
        Self {
            regions: HashMap::new(),
            parent_map: HashMap::new(),
            children_map: HashMap::new(),
            root_id: None,
        }
    }

    /// Create a hierarchy with a root region
    pub fn with_root(root: BrainRegion) -> Self {
        let mut hierarchy = Self::new();
        let root_id = root.region_id;
        hierarchy.regions.insert(root_id.to_string(), root);
        hierarchy.root_id = Some(root_id.to_string());
        hierarchy
    }

    /// Add a region to the hierarchy
    ///
    /// # Arguments
    ///
    /// * `region` - The region to add
    /// * `parent_id` - Optional parent region ID (None for root)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Region ID already exists
    /// - Parent ID doesn't exist
    /// - Adding would create a cycle
    ///
    pub fn add_region(&mut self, region: BrainRegion, parent_id: Option<String>) -> BduResult<()> {
        let region_id = region.region_id;

        // Check if region already exists
        if self.regions.contains_key(&region_id.to_string()) {
            return Err(BduError::InvalidArea(format!(
                "Region {} already exists",
                region_id
            )));
        }

        // Validate parent exists (if specified)
        if let Some(ref parent) = parent_id {
            if !self.regions.contains_key(parent) {
                return Err(BduError::InvalidArea(format!(
                    "Parent region {} does not exist",
                    parent
                )));
            }
        }

        // Add region
        let region_id_str = region_id.to_string();
        self.regions.insert(region_id_str.clone(), region);

        // Update parent/child maps
        if let Some(parent) = parent_id {
            self.parent_map
                .insert(region_id_str.clone(), parent.clone());
            self.children_map
                .entry(parent)
                .or_default()
                .insert(region_id_str.clone());
        } else if self.root_id.is_none() {
            // First region without parent becomes root
            self.root_id = Some(region_id_str);
        }

        Ok(())
    }

    /// Remove a region and reassign its children to its parent
    ///
    /// # Arguments
    ///
    /// * `region_id` - ID of the region to remove
    ///
    /// # Errors
    ///
    /// Returns error if region doesn't exist or is the root
    ///
    pub fn remove_region(&mut self, region_id: &str) -> BduResult<()> {
        // Cannot remove root
        if self.root_id.as_deref() == Some(region_id) {
            return Err(BduError::InvalidArea(
                "Cannot remove root region".to_string(),
            ));
        }

        // Check if region exists
        if !self.regions.contains_key(region_id) {
            return Err(BduError::InvalidArea(format!(
                "Region {} does not exist",
                region_id
            )));
        }

        // Get parent and children
        let parent_id = self.parent_map.get(region_id).cloned();
        let children = self
            .children_map
            .get(region_id)
            .cloned()
            .unwrap_or_default();

        // Reassign children to parent
        if let Some(parent) = &parent_id {
            for child in &children {
                self.parent_map.insert(child.clone(), parent.clone());
                self.children_map
                    .entry(parent.clone())
                    .or_default()
                    .insert(child.clone());
            }
        }

        // Remove region from parent's children
        if let Some(parent) = &parent_id {
            if let Some(parent_children) = self.children_map.get_mut(parent) {
                parent_children.remove(region_id);
            }
        }

        // Remove region
        self.regions.remove(region_id);
        self.parent_map.remove(region_id);
        self.children_map.remove(region_id);

        Ok(())
    }

    /// Change a region's parent
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Region doesn't exist
    /// - New parent doesn't exist
    /// - Would create a cycle
    ///
    pub fn change_parent(&mut self, region_id: &str, new_parent_id: &str) -> BduResult<()> {
        // Validate both exist
        if !self.regions.contains_key(region_id) {
            return Err(BduError::InvalidArea(format!(
                "Region {} does not exist",
                region_id
            )));
        }

        if !self.regions.contains_key(new_parent_id) {
            return Err(BduError::InvalidArea(format!(
                "Parent region {} does not exist",
                new_parent_id
            )));
        }

        // Check for cycle (new parent cannot be a descendant)
        if self.is_descendant(new_parent_id, region_id) {
            return Err(BduError::InvalidArea(
                "Cannot create cycle in hierarchy".to_string(),
            ));
        }

        // Remove from old parent's children
        if let Some(old_parent) = self.parent_map.get(region_id) {
            if let Some(old_parent_children) = self.children_map.get_mut(old_parent) {
                old_parent_children.remove(region_id);
            }
        }

        // Update parent map
        self.parent_map
            .insert(region_id.to_string(), new_parent_id.to_string());

        // Add to new parent's children
        self.children_map
            .entry(new_parent_id.to_string())
            .or_default()
            .insert(region_id.to_string());

        Ok(())
    }

    /// Check if one region is a descendant of another
    fn is_descendant(&self, potential_descendant: &str, ancestor: &str) -> bool {
        let mut current = potential_descendant;

        while let Some(parent) = self.parent_map.get(current) {
            if parent == ancestor {
                return true;
            }
            current = parent;
        }

        false
    }

    /// Get a region by ID
    pub fn get_region(&self, region_id: &str) -> Option<&BrainRegion> {
        self.regions.get(region_id)
    }

    /// Get a mutable reference to a region
    pub fn get_region_mut(&mut self, region_id: &str) -> Option<&mut BrainRegion> {
        self.regions.get_mut(region_id)
    }

    /// Get the parent of a region
    pub fn get_parent(&self, region_id: &str) -> Option<&String> {
        self.parent_map.get(region_id)
    }

    /// Find which brain region contains a given cortical area
    ///
    /// Searches all brain regions to find which one contains the specified cortical area.
    /// This is used to populate `parent_region_id` in API responses for Brain Visualizer.
    ///
    /// # Arguments
    /// * `cortical_id` - Cortical area to search for
    ///
    /// # Returns
    /// * `Option<String>` - Region ID (UUID string) if found, None if area not in any region
    ///
    pub fn find_region_containing_area(&self, cortical_id: &CorticalID) -> Option<String> {
        for (region_id, region) in &self.regions {
            if region.cortical_areas.contains(cortical_id) {
                return Some(region_id.clone());
            }
        }
        None
    }

    /// Get the root brain region ID (region with no parent)
    ///
    /// Searches for the region that has no parent in the parent_map.
    /// This provides O(n) lookup but is cached by ConnectomeManager for O(1) access.
    ///
    /// # Returns
    /// * `Option<String>` - Root region ID (UUID string) if found
    ///
    pub fn get_root_region_id(&self) -> Option<String> {
        for region_id in self.regions.keys() {
            if !self.parent_map.contains_key(region_id) {
                return Some(region_id.clone());
            }
        }
        None
    }

    /// Get all children of a region
    pub fn get_children(&self, region_id: &str) -> Vec<&String> {
        self.children_map
            .get(region_id)
            .map(|children| children.iter().collect())
            .unwrap_or_default()
    }

    /// Get all descendant regions (recursive)
    pub fn get_all_descendants(&self, region_id: &str) -> Vec<&String> {
        let mut descendants = Vec::new();
        let mut to_visit = vec![region_id];

        while let Some(current) = to_visit.pop() {
            if let Some(children) = self.children_map.get(current) {
                for child in children {
                    descendants.push(child);
                    to_visit.push(child);
                }
            }
        }

        descendants
    }

    /// Get all cortical areas in a region and its descendants
    pub fn get_all_areas_recursive(&self, region_id: &str) -> HashSet<String> {
        let mut areas = HashSet::new();

        // Add areas from this region
        if let Some(region) = self.regions.get(region_id) {
            // Convert CorticalID to String
            areas.extend(region.cortical_areas.iter().map(|id| id.to_string()));
        }

        // Add areas from descendants
        for descendant_id in self.get_all_descendants(region_id) {
            if let Some(region) = self.regions.get(descendant_id) {
                // Convert CorticalID to String
                areas.extend(region.cortical_areas.iter().map(|id| id.to_string()));
            }
        }

        areas
    }

    /// Get the root region ID
    pub fn get_root_id(&self) -> Option<&String> {
        self.root_id.as_ref()
    }

    /// Get all region IDs
    pub fn get_all_region_ids(&self) -> Vec<&String> {
        self.regions.keys().collect()
    }

    /// Get the total number of regions
    pub fn region_count(&self) -> usize {
        self.regions.len()
    }

    /// Get all regions as a cloned HashMap
    ///
    /// This is useful for extracting all brain regions to sync with RuntimeGenome
    pub fn get_all_regions(&self) -> HashMap<String, BrainRegion> {
        self.regions.clone()
    }
}

impl Default for BrainRegionHierarchy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use feagi_structures::genomic::brain_regions::{RegionID, RegionType};

    #[test]
    fn test_hierarchy_creation() {
        let root =
            BrainRegion::new(RegionID::new(), "Root".to_string(), RegionType::Undefined).unwrap();

        let hierarchy = BrainRegionHierarchy::with_root(root);

        assert_eq!(hierarchy.region_count(), 1);
        assert!(hierarchy.get_root_id().is_some());
    }

    #[test]
    fn test_add_regions() {
        let root =
            BrainRegion::new(RegionID::new(), "Root".to_string(), RegionType::Undefined).unwrap();

        let mut hierarchy = BrainRegionHierarchy::with_root(root);
        let root_id = hierarchy.get_root_id().unwrap().clone();

        // Add child
        let visual =
            BrainRegion::new(RegionID::new(), "Visual".to_string(), RegionType::Undefined).unwrap();
        let visual_id = visual.region_id.to_string();

        hierarchy.add_region(visual, Some(root_id.clone())).unwrap();

        assert_eq!(hierarchy.region_count(), 2);
        assert_eq!(hierarchy.get_parent(&visual_id), Some(&root_id));
    }

    #[test]
    fn test_remove_region() {
        let root =
            BrainRegion::new(RegionID::new(), "Root".to_string(), RegionType::Undefined).unwrap();

        let mut hierarchy = BrainRegionHierarchy::with_root(root);
        let root_id = hierarchy.get_root_id().unwrap().clone();

        // Add regions
        let visual =
            BrainRegion::new(RegionID::new(), "Visual".to_string(), RegionType::Undefined).unwrap();
        let visual_id = visual.region_id.to_string();

        let v1 =
            BrainRegion::new(RegionID::new(), "V1".to_string(), RegionType::Undefined).unwrap();
        let v1_id = v1.region_id.to_string();

        hierarchy.add_region(visual, Some(root_id.clone())).unwrap();
        hierarchy.add_region(v1, Some(visual_id.clone())).unwrap();

        // Remove visual (v1 should be reassigned to root)
        hierarchy.remove_region(&visual_id).unwrap();

        assert_eq!(hierarchy.region_count(), 2);
        assert_eq!(hierarchy.get_parent(&v1_id), Some(&root_id));
    }

    #[test]
    fn test_change_parent() {
        let root =
            BrainRegion::new(RegionID::new(), "Root".to_string(), RegionType::Undefined).unwrap();

        let mut hierarchy = BrainRegionHierarchy::with_root(root);
        let root_id = hierarchy.get_root_id().unwrap().clone();

        // Add regions
        let visual =
            BrainRegion::new(RegionID::new(), "Visual".to_string(), RegionType::Undefined).unwrap();
        let visual_id = visual.region_id.to_string();

        let motor =
            BrainRegion::new(RegionID::new(), "Motor".to_string(), RegionType::Undefined).unwrap();
        let motor_id = motor.region_id.to_string();

        let v1 =
            BrainRegion::new(RegionID::new(), "V1".to_string(), RegionType::Undefined).unwrap();
        let v1_id = v1.region_id.to_string();

        hierarchy.add_region(visual, Some(root_id.clone())).unwrap();
        hierarchy.add_region(motor, Some(root_id.clone())).unwrap();
        hierarchy.add_region(v1, Some(visual_id.clone())).unwrap();

        // Move v1 from visual to motor
        hierarchy.change_parent(&v1_id, &motor_id).unwrap();

        assert_eq!(hierarchy.get_parent(&v1_id), Some(&motor_id));
        assert!(!hierarchy.get_children(&visual_id).contains(&&v1_id));
        assert!(hierarchy.get_children(&motor_id).contains(&&v1_id));
    }

    #[test]
    fn test_get_descendants() {
        let root =
            BrainRegion::new(RegionID::new(), "Root".to_string(), RegionType::Undefined).unwrap();

        let mut hierarchy = BrainRegionHierarchy::with_root(root);
        let root_id = hierarchy.get_root_id().unwrap().clone();

        // Create tree: root -> visual -> v1, v2
        let visual =
            BrainRegion::new(RegionID::new(), "Visual".to_string(), RegionType::Undefined).unwrap();
        let visual_id = visual.region_id.to_string();

        let v1 =
            BrainRegion::new(RegionID::new(), "V1".to_string(), RegionType::Undefined).unwrap();

        let v2 =
            BrainRegion::new(RegionID::new(), "V2".to_string(), RegionType::Undefined).unwrap();

        hierarchy.add_region(visual, Some(root_id.clone())).unwrap();
        hierarchy.add_region(v1, Some(visual_id.clone())).unwrap();
        hierarchy.add_region(v2, Some(visual_id.clone())).unwrap();

        // Get descendants of root
        let descendants = hierarchy.get_all_descendants(&root_id);
        assert_eq!(descendants.len(), 3); // visual, v1, v2

        // Get descendants of visual
        let visual_descendants = hierarchy.get_all_descendants(&visual_id);
        assert_eq!(visual_descendants.len(), 2); // v1, v2
    }

    #[test]
    fn test_cycle_prevention() {
        let root =
            BrainRegion::new(RegionID::new(), "Root".to_string(), RegionType::Undefined).unwrap();

        let mut hierarchy = BrainRegionHierarchy::with_root(root);
        let root_id = hierarchy.get_root_id().unwrap().clone();

        let visual =
            BrainRegion::new(RegionID::new(), "Visual".to_string(), RegionType::Undefined).unwrap();
        let visual_id = visual.region_id.to_string();

        let v1 =
            BrainRegion::new(RegionID::new(), "V1".to_string(), RegionType::Undefined).unwrap();
        let v1_id = v1.region_id.to_string();

        hierarchy.add_region(visual, Some(root_id.clone())).unwrap();
        hierarchy.add_region(v1, Some(visual_id.clone())).unwrap();

        // Try to make visual a child of v1 (would create cycle)
        let result = hierarchy.change_parent(&visual_id, &v1_id);
        assert!(result.is_err());
    }
}
