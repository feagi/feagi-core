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
pub use feagi_structures::genomic::cortical_area::{
    CoreCorticalType, CorticalArea, CorticalAreaDimensions, CorticalID,
};

/// Extension trait providing business logic methods for CorticalArea
pub trait CorticalAreaExt {
    /// Create a cortical area with custom properties
    fn with_properties(self, properties: HashMap<String, serde_json::Value>) -> Self;

    /// Add a single property (builder pattern)
    fn add_property(self, key: String, value: serde_json::Value) -> Self;

    /// Add a single property in-place
    fn add_property_mut(&mut self, key: String, value: serde_json::Value);

    /// Check if a 3D position is within this area's bounds
    fn contains_position(&self, pos: (i32, i32, i32)) -> bool;

    /// Convert absolute brain position to relative position within this area
    fn to_relative_position(&self, pos: (i32, i32, i32)) -> BduResult<Position>;

    /// Convert relative position within area to absolute brain position
    fn to_absolute_position(&self, rel_pos: Position) -> BduResult<(i32, i32, i32)>;

    /// Get neurons_per_voxel from properties (defaults to 1)
    fn neurons_per_voxel(&self) -> u32;

    /// Get refractory_period from properties (defaults to 0)
    fn refractory_period(&self) -> u16;

    /// Get snooze_period from properties (defaults to 0)
    fn snooze_period(&self) -> u16;

    /// Get leak_coefficient from properties (defaults to 0.0)
    fn leak_coefficient(&self) -> f32;

    /// Get firing_threshold from properties (defaults to 1.0)
    fn firing_threshold(&self) -> f32;

    /// Get firing_threshold_limit from properties (defaults to 0.0 = no limit)
    fn firing_threshold_limit(&self) -> f32;

    /// Get property as u32 with default
    fn get_u32_property(&self, key: &str, default: u32) -> u32;

    /// Get property as u16 with default
    fn get_u16_property(&self, key: &str, default: u16) -> u16;

    /// Get property as f32 with default
    fn get_f32_property(&self, key: &str, default: f32) -> f32;

    /// Get property as bool with default
    fn get_bool_property(&self, key: &str, default: bool) -> bool;

    /// Check if this is an input area
    fn is_input_area(&self) -> bool;

    /// Check if this is an output area
    fn is_output_area(&self) -> bool;

    /// Get cortical group classification
    fn get_cortical_group(&self) -> Option<String>;

    /// Get visible flag from properties (defaults to true)
    fn visible(&self) -> bool;

    /// Get sub_group from properties
    fn sub_group(&self) -> Option<String>;

    /// Get plasticity_constant from properties
    fn plasticity_constant(&self) -> f32;

    /// Get postsynaptic_current from properties
    fn postsynaptic_current(&self) -> f32;

    /// Get psp_uniform_distribution from properties
    fn psp_uniform_distribution(&self) -> bool;

    /// Get degeneration from properties
    fn degeneration(&self) -> f32;

    /// Get burst_engine_active from properties
    fn burst_engine_active(&self) -> bool;

    /// Get firing_threshold_increment from properties
    fn firing_threshold_increment(&self) -> f32;

    /// Get firing_threshold_increment_x from properties
    fn firing_threshold_increment_x(&self) -> f32;

    /// Get firing_threshold_increment_y from properties
    fn firing_threshold_increment_y(&self) -> f32;

    /// Get firing_threshold_increment_z from properties
    fn firing_threshold_increment_z(&self) -> f32;

    /// Get consecutive_fire_count from properties
    fn consecutive_fire_count(&self) -> u32;

    /// Get leak_variability from properties
    fn leak_variability(&self) -> f32;

    /// Get neuron_excitability from properties
    fn neuron_excitability(&self) -> f32;

    /// Get postsynaptic_current_max from properties
    fn postsynaptic_current_max(&self) -> f32;

    /// Get mp_charge_accumulation from properties
    fn mp_charge_accumulation(&self) -> bool;

    /// Get mp_driven_psp from properties
    fn mp_driven_psp(&self) -> bool;

    /// Get init_lifespan from properties (memory parameter)
    fn init_lifespan(&self) -> u32;

    /// Get lifespan_growth_rate from properties (memory parameter)
    fn lifespan_growth_rate(&self) -> f32;

    /// Get longterm_mem_threshold from properties (memory parameter)
    fn longterm_mem_threshold(&self) -> u32;
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

    fn add_property_mut(&mut self, key: String, value: serde_json::Value) {
        self.properties.insert(key, value);
    }

    fn contains_position(&self, pos: (i32, i32, i32)) -> bool {
        let (x, y, z) = pos;
        let ox = self.position.x;
        let oy = self.position.y;
        let oz = self.position.z;

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
                dims: (
                    self.dimensions.width as usize,
                    self.dimensions.height as usize,
                    self.dimensions.depth as usize,
                ),
            });
        }

        let ox = self.position.x;
        let oy = self.position.y;
        let oz = self.position.z;
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
                dims: (
                    self.dimensions.width as usize,
                    self.dimensions.height as usize,
                    self.dimensions.depth as usize,
                ),
            });
        }

        let ox = self.position.x;
        let oy = self.position.y;
        let oz = self.position.z;
        Ok((
            ox + rel_pos.0 as i32,
            oy + rel_pos.1 as i32,
            oz + rel_pos.2 as i32,
        ))
    }

    fn neurons_per_voxel(&self) -> u32 {
        self.get_u32_property("neurons_per_voxel", 1)
    }

    fn refractory_period(&self) -> u16 {
        self.get_u16_property("refractory_period", 0)
    }

    fn snooze_period(&self) -> u16 {
        self.get_u16_property("snooze_period", 0)
    }

    fn leak_coefficient(&self) -> f32 {
        self.get_f32_property("leak_coefficient", 0.0)
    }

    fn firing_threshold(&self) -> f32 {
        self.get_f32_property("firing_threshold", 1.0)
    }

    fn get_u32_property(&self, key: &str, default: u32) -> u32 {
        self.properties
            .get(key)
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(default)
    }

    fn get_u16_property(&self, key: &str, default: u16) -> u16 {
        self.properties
            .get(key)
            .and_then(|v| v.as_u64())
            .map(|v| v as u16)
            .unwrap_or(default)
    }

    fn get_f32_property(&self, key: &str, default: f32) -> f32 {
        self.properties
            .get(key)
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(default)
    }

    fn get_bool_property(&self, key: &str, default: bool) -> bool {
        self.properties
            .get(key)
            .and_then(|v| v.as_bool())
            .unwrap_or(default)
    }

    fn is_input_area(&self) -> bool {
        matches!(
            self.cortical_type,
            feagi_structures::genomic::cortical_area::CorticalAreaType::BrainInput(_)
        )
    }

    fn is_output_area(&self) -> bool {
        matches!(
            self.cortical_type,
            feagi_structures::genomic::cortical_area::CorticalAreaType::BrainOutput(_)
        )
    }

    fn get_cortical_group(&self) -> Option<String> {
        self.properties
            .get("cortical_group")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                // Derive from cortical_type if not in properties
                use feagi_structures::genomic::cortical_area::CorticalAreaType;
                match self.cortical_type {
                    CorticalAreaType::BrainInput(_) => Some("IPU".to_string()),
                    CorticalAreaType::BrainOutput(_) => Some("OPU".to_string()),
                    CorticalAreaType::Memory(_) => Some("MEMORY".to_string()),
                    CorticalAreaType::Custom(_) => Some("CUSTOM".to_string()),
                    CorticalAreaType::Core(_) => Some("CORE".to_string()),
                }
            })
    }

    fn visible(&self) -> bool {
        self.get_bool_property("visible", true)
    }

    fn sub_group(&self) -> Option<String> {
        self.properties
            .get("sub_group")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    fn plasticity_constant(&self) -> f32 {
        self.get_f32_property("plasticity_constant", 0.0)
    }

    fn postsynaptic_current(&self) -> f32 {
        self.get_f32_property("postsynaptic_current", 1.0)
    }

    fn psp_uniform_distribution(&self) -> bool {
        self.get_bool_property("psp_uniform_distribution", false)
    }

    fn degeneration(&self) -> f32 {
        self.get_f32_property("degeneration", 0.0)
    }

    fn burst_engine_active(&self) -> bool {
        self.get_bool_property("burst_engine_active", false)
    }

    fn firing_threshold_increment(&self) -> f32 {
        self.get_f32_property("firing_threshold_increment", 0.0)
    }

    fn firing_threshold_increment_x(&self) -> f32 {
        self.get_f32_property("firing_threshold_increment_x", 0.0)
    }

    fn firing_threshold_increment_y(&self) -> f32 {
        self.get_f32_property("firing_threshold_increment_y", 0.0)
    }

    fn firing_threshold_increment_z(&self) -> f32 {
        self.get_f32_property("firing_threshold_increment_z", 0.0)
    }

    fn firing_threshold_limit(&self) -> f32 {
        self.get_f32_property("firing_threshold_limit", 0.0)
    }

    fn consecutive_fire_count(&self) -> u32 {
        self.get_u32_property("consecutive_fire_limit", 0)
    }

    fn leak_variability(&self) -> f32 {
        self.get_f32_property("leak_variability", 0.0)
    }

    fn neuron_excitability(&self) -> f32 {
        self.get_f32_property("neuron_excitability", 100.0)
    }

    fn postsynaptic_current_max(&self) -> f32 {
        self.get_f32_property("postsynaptic_current_max", 0.0)
    }

    fn mp_charge_accumulation(&self) -> bool {
        self.get_bool_property("mp_charge_accumulation", false)
    }

    fn mp_driven_psp(&self) -> bool {
        self.get_bool_property("mp_driven_psp", false)
    }

    fn init_lifespan(&self) -> u32 {
        self.get_u32_property("init_lifespan", 0)
    }

    fn lifespan_growth_rate(&self) -> f32 {
        self.get_f32_property("lifespan_growth_rate", 0.0)
    }

    fn longterm_mem_threshold(&self) -> u32 {
        self.get_u32_property("longterm_mem_threshold", 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_position() {
        let cortical_id = CoreCorticalType::Power.to_cortical_id();
        let cortical_type = cortical_id
            .as_cortical_type()
            .expect("Failed to get cortical type");
        let dims = CorticalAreaDimensions::new(10, 10, 10).unwrap();
        let area = CorticalArea::new(
            cortical_id,
            0,
            "Test Area".to_string(),
            dims,
            (5, 5, 5).into(),
            cortical_type,
        )
        .unwrap();

        assert!(area.contains_position((5, 5, 5))); // Min corner
        assert!(area.contains_position((14, 14, 14))); // Max corner
        assert!(!area.contains_position((4, 5, 5))); // Outside (x too small)
        assert!(!area.contains_position((15, 5, 5))); // Outside (x too large)
    }

    #[test]
    fn test_position_conversion() {
        let cortical_id = CoreCorticalType::Power.to_cortical_id();
        let cortical_type = cortical_id
            .as_cortical_type()
            .expect("Failed to get cortical type");
        let dims = CorticalAreaDimensions::new(10, 10, 10).unwrap();
        let area = CorticalArea::new(
            cortical_id,
            0,
            "Test Area".to_string(),
            dims,
            (100, 200, 300).into(),
            cortical_type,
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
        let cortical_id = CoreCorticalType::Power.to_cortical_id();
        let cortical_type = cortical_id
            .as_cortical_type()
            .expect("Failed to get cortical type");
        let dims = CorticalAreaDimensions::new(10, 10, 10).unwrap();
        let area = CorticalArea::new(
            cortical_id,
            0,
            "Test".to_string(),
            dims,
            (0, 0, 0).into(),
            cortical_type,
        )
        .unwrap()
        .add_property("resolution".to_string(), serde_json::json!(128))
        .add_property("modality".to_string(), serde_json::json!("visual"));

        assert_eq!(
            area.get_property("resolution"),
            Some(&serde_json::json!(128))
        );
        assert_eq!(
            area.get_property("modality"),
            Some(&serde_json::json!("visual"))
        );
        assert_eq!(area.get_property("nonexistent"), None);
    }
}
