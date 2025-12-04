// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
CorticalArea business logic and extension methods.

The core CorticalArea data structure is defined in feagi_data_structures.
This module provides business logic methods for coordinate transformations
and builder patterns.
*/

use std::collections::HashMap;

use crate::types::{BduError, BduResult, Position};

// Import core types from feagi_data_structures
pub use feagi_data_structures::genomic::cortical_area::{
    CorticalArea, AreaType, CorticalID, CorticalAreaDimensions
};

/// Extension trait providing business logic methods for CorticalArea
pub trait CorticalAreaExt {
    /// Create a cortical area with custom properties
    fn with_properties(self, properties: HashMap<String, serde_json::Value>) -> Self;
    
    /// Add a single property
    fn add_property(self, key: String, value: serde_json::Value) -> Self;
    
    /// Check if a 3D position is within this area's bounds
    fn contains_position(&self, pos: (i32, i32, i32)) -> bool;
    
    /// Convert absolute brain position to relative position within this area
    fn to_relative_position(&self, pos: (i32, i32, i32)) -> BduResult<Position>;
    
    /// Convert relative position within area to absolute brain position
    fn to_absolute_position(&self, rel_pos: Position) -> BduResult<(i32, i32, i32)>;
}

impl CorticalAreaExt for CorticalArea {

    fn with_properties(mut self, properties: HashMap<String, serde_json::Value>) -> Self {
        self.properties = properties;
        self
    }

    fn add_property(mut self, key: String, value: serde_json::Value) -> Self {
        self.properties.insert(key, value);
        self
    }

    fn contains_position(&self, pos: (i32, i32, i32)) -> bool {
        let (x, y, z) = pos;
        let (ox, oy, oz) = self.position;

        x >= ox
            && y >= oy
            && z >= oz
            && x < ox + self.dimensions.width as i32
            && y < oy + self.dimensions.height as i32
            && z < oz + self.dimensions.depth as i32
    }

    fn to_relative_position(&self, pos: (i32, i32, i32)) -> BduResult<Position> {
        if !self.contains_position(pos) {
            return Err(BduError::OutOfBounds {
                pos: (pos.0 as u32, pos.1 as u32, pos.2 as u32),
                dims: (self.dimensions.width as usize, self.dimensions.height as usize, self.dimensions.depth as usize),
            });
        }

        let (ox, oy, oz) = self.position;
        Ok((
            (pos.0 - ox) as u32,
            (pos.1 - oy) as u32,
            (pos.2 - oz) as u32,
        ))
    }

    fn to_absolute_position(&self, rel_pos: Position) -> BduResult<(i32, i32, i32)> {
        if !self.dimensions.contains(rel_pos) {
            return Err(BduError::OutOfBounds {
                pos: rel_pos,
                dims: (self.dimensions.width as usize, self.dimensions.height as usize, self.dimensions.depth as usize),
            });
        }

        let (ox, oy, oz) = self.position;
        Ok((
            ox + rel_pos.0 as i32,
            oy + rel_pos.1 as i32,
            oz + rel_pos.2 as i32,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_position() {
        let cortical_id = CorticalID::try_from_base_64("test01").unwrap();
        let dims = CorticalAreaDimensions::new(10, 10, 10).unwrap();
        let area = CorticalArea::new(
            cortical_id,
            0,
            "Test Area".to_string(),
            dims,
            (5, 5, 5),
            AreaType::Custom,
        )
        .unwrap();

        assert!(area.contains_position((5, 5, 5))); // Min corner
        assert!(area.contains_position((14, 14, 14))); // Max corner
        assert!(!area.contains_position((4, 5, 5))); // Outside (x too small)
        assert!(!area.contains_position((15, 5, 5))); // Outside (x too large)
    }

    #[test]
    fn test_position_conversion() {
        let cortical_id = CorticalID::try_from_base_64("test02").unwrap();
        let dims = CorticalAreaDimensions::new(10, 10, 10).unwrap();
        let area = CorticalArea::new(
            cortical_id,
            0,
            "Test Area".to_string(),
            dims,
            (100, 200, 300),
            AreaType::Custom,
        )
        .unwrap();

        // Area spans from (100,200,300) to (109,209,309)
        // Absolute (105, 207, 308) should map to relative (5, 7, 8)
        let rel_pos = area.to_relative_position((105, 207, 308)).unwrap();
        assert_eq!(rel_pos, (5, 7, 8));

        // Convert back
        let abs_pos = area.to_absolute_position(rel_pos).unwrap();
        assert_eq!(abs_pos, (105, 207, 308));

        // Test out of bounds
        let result = area.to_relative_position((99, 200, 300));
        assert!(result.is_err());
    }

    #[test]
    fn test_properties() {
        let cortical_id = CorticalID::try_from_base_64("test03").unwrap();
        let dims = CorticalAreaDimensions::new(10, 10, 10).unwrap();
        let area = CorticalArea::new(
            cortical_id,
            0,
            "Test".to_string(),
            dims,
            (0, 0, 0),
            AreaType::Sensory,
        )
        .unwrap()
        .add_property("resolution".to_string(), serde_json::json!(128))
        .add_property("modality".to_string(), serde_json::json!("visual"));

        assert_eq!(area.get_property("resolution"), Some(&serde_json::json!(128)));
        assert_eq!(
            area.get_property("modality"),
            Some(&serde_json::json!("visual"))
        );
        assert_eq!(area.get_property("nonexistent"), None);
    }
}
