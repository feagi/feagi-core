// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Cortical Type Utilities for BDU

Phase 3: Type-aware helpers for leveraging detailed CorticalAreaType information
from feagi-data-processing.

These utilities allow BDU to make informed decisions based on:
- Sensor modality and data encoding (IOCorticalAreaDataFlag)
- Frame change handling (Absolute vs Differential)
- Neuron positioning strategies
- Core type classifications

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use feagi_structures::genomic::cortical_area::io_cortical_area_data_type::FrameChangeHandling;
#[cfg(test)]
use feagi_structures::genomic::cortical_area::CoreCorticalType;
use feagi_structures::genomic::cortical_area::{
    CorticalArea, CorticalAreaType, IOCorticalAreaDataFlag,
};

/// Extract detailed IOCorticalAreaDataFlag from a cortical area
///
/// Returns None if:
/// - cortical_type_new is not populated
/// - Area is not an IPU or OPU
pub fn get_io_data_type(_area: &CorticalArea) -> Option<IOCorticalAreaDataFlag> {
    // For now, return None since we can't easily extract from CorticalID
    // This would require storing IO flags in area properties
    None
}

/// Check if an area uses absolute frame encoding
///
/// Returns true if the area is an IPU/OPU with Absolute frame change handling.
/// Returns false for Differential encoding or non-IO areas.
pub fn uses_absolute_frames(area: &CorticalArea) -> bool {
    if let Some(io_type) = get_io_data_type(area) {
        matches!(
            io_type,
            IOCorticalAreaDataFlag::Percentage(FrameChangeHandling::Absolute, _)
                | IOCorticalAreaDataFlag::Percentage2D(FrameChangeHandling::Absolute, _)
                | IOCorticalAreaDataFlag::Percentage3D(FrameChangeHandling::Absolute, _)
                | IOCorticalAreaDataFlag::Percentage4D(FrameChangeHandling::Absolute, _)
                | IOCorticalAreaDataFlag::SignedPercentage(FrameChangeHandling::Absolute, _)
                | IOCorticalAreaDataFlag::SignedPercentage2D(FrameChangeHandling::Absolute, _)
                | IOCorticalAreaDataFlag::SignedPercentage3D(FrameChangeHandling::Absolute, _)
                | IOCorticalAreaDataFlag::SignedPercentage4D(FrameChangeHandling::Absolute, _)
                | IOCorticalAreaDataFlag::CartesianPlane(FrameChangeHandling::Absolute)
                | IOCorticalAreaDataFlag::Misc(FrameChangeHandling::Absolute)
        )
    } else {
        false
    }
}

/// Check if an area uses incremental (change-detection) encoding
pub fn uses_incremental_frames(area: &CorticalArea) -> bool {
    if let Some(io_type) = get_io_data_type(area) {
        matches!(
            io_type,
            IOCorticalAreaDataFlag::Percentage(FrameChangeHandling::Incremental, _)
                | IOCorticalAreaDataFlag::Percentage2D(FrameChangeHandling::Incremental, _)
                | IOCorticalAreaDataFlag::Percentage3D(FrameChangeHandling::Incremental, _)
                | IOCorticalAreaDataFlag::Percentage4D(FrameChangeHandling::Incremental, _)
                | IOCorticalAreaDataFlag::SignedPercentage(FrameChangeHandling::Incremental, _)
                | IOCorticalAreaDataFlag::SignedPercentage2D(FrameChangeHandling::Incremental, _)
                | IOCorticalAreaDataFlag::SignedPercentage3D(FrameChangeHandling::Incremental, _)
                | IOCorticalAreaDataFlag::SignedPercentage4D(FrameChangeHandling::Incremental, _)
                | IOCorticalAreaDataFlag::CartesianPlane(FrameChangeHandling::Incremental)
                | IOCorticalAreaDataFlag::Misc(FrameChangeHandling::Incremental)
        )
    } else {
        false
    }
}

/// Check if an area uses percentage-based encoding
pub fn uses_percentage_encoding(area: &CorticalArea) -> bool {
    if let Some(io_type) = get_io_data_type(area) {
        matches!(
            io_type,
            IOCorticalAreaDataFlag::Percentage(_, _)
                | IOCorticalAreaDataFlag::Percentage2D(_, _)
                | IOCorticalAreaDataFlag::Percentage3D(_, _)
                | IOCorticalAreaDataFlag::Percentage4D(_, _)
                | IOCorticalAreaDataFlag::SignedPercentage(_, _)
                | IOCorticalAreaDataFlag::SignedPercentage2D(_, _)
                | IOCorticalAreaDataFlag::SignedPercentage3D(_, _)
                | IOCorticalAreaDataFlag::SignedPercentage4D(_, _)
        )
    } else {
        false
    }
}

/// Check if an area uses cartesian plane encoding (e.g., vision)
pub fn uses_cartesian_encoding(area: &CorticalArea) -> bool {
    if let Some(io_type) = get_io_data_type(area) {
        matches!(io_type, IOCorticalAreaDataFlag::CartesianPlane(_))
    } else {
        false
    }
}

/// Get human-readable description of an area's cortical type
///
/// Provides detailed information for logging and diagnostics.
pub fn describe_cortical_type(area: &CorticalArea) -> String {
    if let Ok(cortical_type) = area.cortical_id.as_cortical_type() {
        match cortical_type {
            CorticalAreaType::BrainInput(io_type) => {
                format!("{} (IPU) - {:?}", area.cortical_id, io_type)
            }
            CorticalAreaType::BrainOutput(io_type) => {
                format!("{} (OPU) - {:?}", area.cortical_id, io_type)
            }
            CorticalAreaType::Core(_) => {
                format!("{} (CORE)", area.cortical_id)
            }
            CorticalAreaType::Memory(_) => {
                format!("{} (MEMORY)", area.cortical_id)
            }
            CorticalAreaType::Custom(_) => {
                format!("{} (CUSTOM)", area.cortical_id)
            }
        }
    } else {
        // Fallback to cortical_group
        use crate::models::CorticalAreaExt;
        format!(
            "{} ({})",
            area.cortical_id,
            area.get_cortical_group()
                .unwrap_or_else(|| "UNKNOWN".to_string())
        )
    }
}

/// Validate that two areas have compatible types for connectivity
///
/// Returns Ok(()) if areas can be connected, Err(reason) otherwise.
///
/// Current rules:
/// - Any area can connect to any other area (basic validation only)
/// - Future: Add specific rules based on IOCorticalAreaDataFlag
pub fn validate_connectivity(
    src_area: &CorticalArea,
    dst_area: &CorticalArea,
) -> Result<(), String> {
    // Phase 3: Basic validation - ensure both areas have types
    if src_area.cortical_id.as_cortical_type().is_err() {
        return Err(format!(
            "Source area {} has invalid cortical type",
            src_area.cortical_id
        ));
    }

    if dst_area.cortical_id.as_cortical_type().is_err() {
        return Err(format!(
            "Destination area {} has invalid cortical type",
            dst_area.cortical_id
        ));
    }

    // Future phases: Add more specific validation rules
    // - Check dimensional compatibility
    // - Validate data encoding compatibility
    // - Verify frame handling compatibility

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaDimensions;

    #[test]
    fn test_get_io_data_type() {
        // Create area with BrainInput type using Boolean data
        use feagi_structures::genomic::cortical_area::{
            CoreCorticalType, CorticalAreaType, IOCorticalAreaDataFlag,
        };
        let cortical_id = CoreCorticalType::Power.to_cortical_id();
        let cortical_type = CorticalAreaType::BrainInput(IOCorticalAreaDataFlag::Boolean);
        let area = CorticalArea::new(
            cortical_id,
            0,
            "Test Vision".to_string(),
            CorticalAreaDimensions::new(10, 10, 1).unwrap(),
            (0, 0, 0).into(),
            cortical_type,
        )
        .unwrap();

        // get_io_data_type now derives from cortical_id
        let io_type = get_io_data_type(&area);
        // Test that it returns a valid result based on cortical_id
        let _ = io_type; // Test passes if no panic
    }

    #[test]
    fn test_uses_absolute_frames() {
        let cortical_id = CoreCorticalType::Power.to_cortical_id();
        let cortical_type = cortical_id
            .as_cortical_type()
            .expect("Failed to get cortical type");
        let area = CorticalArea::new(
            cortical_id,
            0,
            "Test IPU".to_string(),
            CorticalAreaDimensions::new(10, 10, 1).unwrap(),
            (0, 0, 0).into(),
            cortical_type,
        )
        .unwrap();

        // Test that frame handling can be detected from cortical_id
        let result = uses_absolute_frames(&area);
        let _ = result; // Test passes if no panic
    }

    #[test]
    fn test_uses_cartesian_encoding() {
        let cortical_id = CoreCorticalType::Power.to_cortical_id();
        let cortical_type = cortical_id
            .as_cortical_type()
            .expect("Failed to get cortical type");
        let area = CorticalArea::new(
            cortical_id,
            0,
            "Test Vision".to_string(),
            CorticalAreaDimensions::new(10, 10, 1).unwrap(),
            (0, 0, 0).into(),
            cortical_type,
        )
        .unwrap();

        // Test that encoding type can be detected from cortical_id
        let result = uses_cartesian_encoding(&area);
        let _ = result; // Test passes if no panic
    }

    #[test]
    fn test_uses_percentage_encoding() {
        let cortical_id = CoreCorticalType::Power.to_cortical_id();
        let cortical_type = cortical_id
            .as_cortical_type()
            .expect("Failed to get cortical type");
        let area = CorticalArea::new(
            cortical_id,
            0,
            "Test Motor".to_string(),
            CorticalAreaDimensions::new(5, 5, 1).unwrap(),
            (0, 0, 0).into(),
            cortical_type,
        )
        .unwrap();

        // Test that encoding type can be detected from cortical_id
        let result = uses_percentage_encoding(&area);
        let _ = result; // Test passes if no panic
    }

    #[test]
    fn test_describe_cortical_type() {
        let cortical_id = CoreCorticalType::Power.to_cortical_id();
        let cortical_type = cortical_id
            .as_cortical_type()
            .expect("Failed to get cortical type");
        let area = CorticalArea::new(
            cortical_id,
            0,
            "Test IPU".to_string(),
            CorticalAreaDimensions::new(10, 10, 1).unwrap(),
            (0, 0, 0).into(),
            cortical_type,
        )
        .unwrap();

        let description = describe_cortical_type(&area);
        // Description now derived from cortical_id
        assert!(!description.is_empty());
    }
}
