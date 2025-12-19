// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Cortical Type Utilities for BDU

Phase 3: Type-aware helpers for leveraging detailed CorticalAreaType information
from feagi-data-processing.

These utilities allow BDU to make informed decisions based on:
- Sensor modality and data encoding (IOCorticalAreaDataType)
- Frame change handling (Absolute vs Differential)
- Neuron positioning strategies
- Core type classifications

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use feagi_data_structures::genomic::cortical_area::{
    CorticalAreaType, IOCorticalAreaDataType,
};
#[cfg(test)]
use feagi_data_structures::genomic::cortical_area::CorticalID;
use feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::FrameChangeHandling;
use feagi_types::CorticalArea;

/// Extract detailed IOCorticalAreaDataType from a cortical area
///
/// Returns None if:
/// - cortical_type_new is not populated
/// - Area is not an IPU or OPU
pub fn get_io_data_type(area: &CorticalArea) -> Option<&IOCorticalAreaDataType> {
    match area.cortical_type_new.as_ref()? {
        CorticalAreaType::BrainInput(io_type) => Some(io_type),
        CorticalAreaType::BrainOutput(io_type) => Some(io_type),
        _ => None,
    }
}

/// Check if an area uses absolute frame encoding
///
/// Returns true if the area is an IPU/OPU with Absolute frame change handling.
/// Returns false for Differential encoding or non-IO areas.
pub fn uses_absolute_frames(area: &CorticalArea) -> bool {
    if let Some(io_type) = get_io_data_type(area) {
        matches!(
            io_type,
            IOCorticalAreaDataType::Percentage(FrameChangeHandling::Absolute, _)
            | IOCorticalAreaDataType::Percentage2D(FrameChangeHandling::Absolute, _)
            | IOCorticalAreaDataType::Percentage3D(FrameChangeHandling::Absolute, _)
            | IOCorticalAreaDataType::Percentage4D(FrameChangeHandling::Absolute, _)
            | IOCorticalAreaDataType::SignedPercentage(FrameChangeHandling::Absolute, _)
            | IOCorticalAreaDataType::SignedPercentage2D(FrameChangeHandling::Absolute, _)
            | IOCorticalAreaDataType::SignedPercentage3D(FrameChangeHandling::Absolute, _)
            | IOCorticalAreaDataType::SignedPercentage4D(FrameChangeHandling::Absolute, _)
            | IOCorticalAreaDataType::CartesianPlane(FrameChangeHandling::Absolute)
            | IOCorticalAreaDataType::Misc(FrameChangeHandling::Absolute)
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
            IOCorticalAreaDataType::Percentage(FrameChangeHandling::Incremental, _)
            | IOCorticalAreaDataType::Percentage2D(FrameChangeHandling::Incremental, _)
            | IOCorticalAreaDataType::Percentage3D(FrameChangeHandling::Incremental, _)
            | IOCorticalAreaDataType::Percentage4D(FrameChangeHandling::Incremental, _)
            | IOCorticalAreaDataType::SignedPercentage(FrameChangeHandling::Incremental, _)
            | IOCorticalAreaDataType::SignedPercentage2D(FrameChangeHandling::Incremental, _)
            | IOCorticalAreaDataType::SignedPercentage3D(FrameChangeHandling::Incremental, _)
            | IOCorticalAreaDataType::SignedPercentage4D(FrameChangeHandling::Incremental, _)
            | IOCorticalAreaDataType::CartesianPlane(FrameChangeHandling::Incremental)
            | IOCorticalAreaDataType::Misc(FrameChangeHandling::Incremental)
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
            IOCorticalAreaDataType::Percentage(_, _)
            | IOCorticalAreaDataType::Percentage2D(_, _)
            | IOCorticalAreaDataType::Percentage3D(_, _)
            | IOCorticalAreaDataType::Percentage4D(_, _)
            | IOCorticalAreaDataType::SignedPercentage(_, _)
            | IOCorticalAreaDataType::SignedPercentage2D(_, _)
            | IOCorticalAreaDataType::SignedPercentage3D(_, _)
            | IOCorticalAreaDataType::SignedPercentage4D(_, _)
        )
    } else {
        false
    }
}

/// Check if an area uses cartesian plane encoding (e.g., vision)
pub fn uses_cartesian_encoding(area: &CorticalArea) -> bool {
    if let Some(io_type) = get_io_data_type(area) {
        matches!(io_type, IOCorticalAreaDataType::CartesianPlane(_))
    } else {
        false
    }
}

/// Get human-readable description of an area's cortical type
///
/// Provides detailed information for logging and diagnostics.
pub fn describe_cortical_type(area: &CorticalArea) -> String {
    if let Some(ref cortical_type) = area.cortical_type_new {
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
        format!("{} ({})", area.cortical_id, area.get_cortical_group())
    }
}

/// Validate that two areas have compatible types for connectivity
///
/// Returns Ok(()) if areas can be connected, Err(reason) otherwise.
///
/// Current rules:
/// - Any area can connect to any other area (basic validation only)
/// - Future: Add specific rules based on IOCorticalAreaDataType
pub fn validate_connectivity(
    src_area: &CorticalArea,
    dst_area: &CorticalArea,
) -> Result<(), String> {
    // Phase 3: Basic validation - ensure both areas have types
    if src_area.cortical_type_new.is_none() {
        return Err(format!(
            "Source area {} missing cortical_type_new",
            src_area.cortical_id
        ));
    }
    
    if dst_area.cortical_type_new.is_none() {
        return Err(format!(
            "Destination area {} missing cortical_type_new",
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
    use feagi_types::{Dimensions};
    use feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::PercentageNeuronPositioning;

    #[test]
    fn test_get_io_data_type() {
        // Create area with IOCorticalAreaDataType
        let mut area = CorticalArea::new(
            CorticalID::try_from_base_64("aWljMDAwX18=").unwrap(),
            0,
            "Test IPU".to_string(),
            Dimensions::new(10, 10, 1),
            (0, 0, 0),
        ).unwrap();
        
        // Initially None
        assert!(get_io_data_type(&area).is_none());
        
        // Add cortical_type_new
        area = area.with_cortical_type_new(CorticalAreaType::BrainInput(
            IOCorticalAreaDataType::CartesianPlane(FrameChangeHandling::Absolute)
        ));
        
        // Now should return the IO type
        assert!(get_io_data_type(&area).is_some());
    }

    #[test]
    fn test_uses_absolute_frames() {
        let mut area = CorticalArea::new(
            CorticalID::try_from_base_64("aWljMDAwX18=").unwrap(),
            0,
            "Test IPU".to_string(),
            Dimensions::new(10, 10, 1),
            (0, 0, 0),
        ).unwrap();
        
        area = area.with_cortical_type_new(CorticalAreaType::BrainInput(
            IOCorticalAreaDataType::CartesianPlane(FrameChangeHandling::Absolute)
        ));
        
        assert!(uses_absolute_frames(&area));
        assert!(!uses_incremental_frames(&area));
    }

    #[test]
    fn test_uses_cartesian_encoding() {
        let mut area = CorticalArea::new(
            CorticalID::try_from_base_64("aWljMDAwX18=").unwrap(),
            0,
            "Test Vision".to_string(),
            Dimensions::new(10, 10, 1),
            (0, 0, 0),
        ).unwrap();
        
        area = area.with_cortical_type_new(CorticalAreaType::BrainInput(
            IOCorticalAreaDataType::CartesianPlane(FrameChangeHandling::Absolute)
        ));
        
        assert!(uses_cartesian_encoding(&area));
        assert!(!uses_percentage_encoding(&area));
    }

    #[test]
    fn test_uses_percentage_encoding() {
        let mut area = CorticalArea::new(
            CorticalID::try_from_base_64("b21vdDAwX18=").unwrap(),
            0,
            "Test Motor".to_string(),
            Dimensions::new(5, 5, 1),
            (0, 0, 0),
        ).unwrap();
        
        area = area.with_cortical_type_new(CorticalAreaType::BrainOutput(
            IOCorticalAreaDataType::Percentage(
                FrameChangeHandling::Absolute,
                PercentageNeuronPositioning::Linear
            )
        ));
        
        assert!(uses_percentage_encoding(&area));
        assert!(!uses_cartesian_encoding(&area));
    }

    #[test]
    fn test_describe_cortical_type() {
        let mut area = CorticalArea::new(
            CorticalID::try_from_base_64("aWljMDAwX18=").unwrap(),
            0,
            "Test IPU".to_string(),
            Dimensions::new(10, 10, 1),
            (0, 0, 0),
        ).unwrap();
        
        area = area.with_cortical_type_new(CorticalAreaType::BrainInput(
            IOCorticalAreaDataType::CartesianPlane(FrameChangeHandling::Absolute)
        ));
        
        let description = describe_cortical_type(&area);
        assert!(description.contains("IPU"));
        assert!(description.contains("CartesianPlane"));
    }
}

