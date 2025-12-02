// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Cortical Type Conversion Utilities (Phase 5)

Converts internal CorticalAreaType representations to API DTOs.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use super::cortical_area_dtos::CorticalTypeInfo;
use feagi_data_structures::genomic::cortical_area::{
    CorticalAreaType, IOCorticalAreaDataType,
};
use feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::FrameChangeHandling;
use feagi_types::{CorticalArea, CorticalTypeAdapter};

/// Convert internal CorticalArea to API CorticalTypeInfo
///
/// Returns None if cortical_type_new is not populated (legacy areas)
pub fn to_cortical_type_info(area: &CorticalArea) -> Option<CorticalTypeInfo> {
    let cortical_type = area.cortical_type_new.as_ref()?;
    
    let category = CorticalTypeAdapter::to_cortical_group(cortical_type).to_string();
    
    // Extract data_type and frame_handling for IPU/OPU
    let (data_type, frame_handling, encoding_details) = match cortical_type {
        CorticalAreaType::BrainInput(io_type) | CorticalAreaType::BrainOutput(io_type) => {
            extract_io_type_details(io_type)
        }
        _ => (None, None, None),
    };
    
    Some(CorticalTypeInfo {
        category,
        data_type,
        frame_handling,
        encoding_details,
    })
}

/// Extract detailed information from IOCorticalAreaDataType
fn extract_io_type_details(
    io_type: &IOCorticalAreaDataType,
) -> (Option<String>, Option<String>, Option<serde_json::Value>) {
    use IOCorticalAreaDataType::*;
    
    match io_type {
        CartesianPlane(frame_handling) => (
            Some("CartesianPlane".to_string()),
            Some(frame_handling_to_string(frame_handling)),
            None,
        ),
        Percentage(frame_handling, positioning) => (
            Some("Percentage".to_string()),
            Some(frame_handling_to_string(frame_handling)),
            Some(serde_json::json!({
                "positioning": positioning_to_string(positioning),
                "signed": false
            })),
        ),
        Percentage2D(frame_handling, positioning) => (
            Some("Percentage2D".to_string()),
            Some(frame_handling_to_string(frame_handling)),
            Some(serde_json::json!({
                "positioning": positioning_to_string(positioning),
                "signed": false,
                "dimensions": 2
            })),
        ),
        Percentage3D(frame_handling, positioning) => (
            Some("Percentage3D".to_string()),
            Some(frame_handling_to_string(frame_handling)),
            Some(serde_json::json!({
                "positioning": positioning_to_string(positioning),
                "signed": false,
                "dimensions": 3
            })),
        ),
        Percentage4D(frame_handling, positioning) => (
            Some("Percentage4D".to_string()),
            Some(frame_handling_to_string(frame_handling)),
            Some(serde_json::json!({
                "positioning": positioning_to_string(positioning),
                "signed": false,
                "dimensions": 4
            })),
        ),
        SignedPercentage(frame_handling, positioning) => (
            Some("SignedPercentage".to_string()),
            Some(frame_handling_to_string(frame_handling)),
            Some(serde_json::json!({
                "positioning": positioning_to_string(positioning),
                "signed": true
            })),
        ),
        SignedPercentage2D(frame_handling, positioning) => (
            Some("SignedPercentage2D".to_string()),
            Some(frame_handling_to_string(frame_handling)),
            Some(serde_json::json!({
                "positioning": positioning_to_string(positioning),
                "signed": true,
                "dimensions": 2
            })),
        ),
        SignedPercentage3D(frame_handling, positioning) => (
            Some("SignedPercentage3D".to_string()),
            Some(frame_handling_to_string(frame_handling)),
            Some(serde_json::json!({
                "positioning": positioning_to_string(positioning),
                "signed": true,
                "dimensions": 3
            })),
        ),
        SignedPercentage4D(frame_handling, positioning) => (
            Some("SignedPercentage4D".to_string()),
            Some(frame_handling_to_string(frame_handling)),
            Some(serde_json::json!({
                "positioning": positioning_to_string(positioning),
                "signed": true,
                "dimensions": 4
            })),
        ),
        Misc(frame_handling) => (
            Some("Misc".to_string()),
            Some(frame_handling_to_string(frame_handling)),
            None,
        ),
    }
}

/// Convert FrameChangeHandling enum to string
fn frame_handling_to_string(frame_handling: &FrameChangeHandling) -> String {
    match frame_handling {
        FrameChangeHandling::Absolute => "Absolute".to_string(),
        FrameChangeHandling::Incremental => "Incremental".to_string(),
    }
}

/// Convert PercentageNeuronPositioning enum to string
fn positioning_to_string(
    positioning: &feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::PercentageNeuronPositioning,
) -> String {
    use feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::PercentageNeuronPositioning;
    
    match positioning {
        PercentageNeuronPositioning::Linear => "Linear".to_string(),
        PercentageNeuronPositioning::Fractional => "Fractional".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use feagi_types::{AreaType, Dimensions};

    #[test]
    fn test_to_cortical_type_info_cartesian_plane() {
        let mut area = CorticalArea::new(
            "aWljMDAwX18=".to_string(),
            0,
            "Vision Input".to_string(),
            Dimensions::new(128, 128, 3),
            (0, 0, 0),
            AreaType::Sensory,
        )
        .unwrap();

        area = area.with_cortical_type_new(CorticalAreaType::BrainInput(
            IOCorticalAreaDataType::CartesianPlane(FrameChangeHandling::Absolute),
        ));

        let type_info = to_cortical_type_info(&area).unwrap();
        assert_eq!(type_info.category, "IPU");
        assert_eq!(type_info.data_type, Some("CartesianPlane".to_string()));
        assert_eq!(type_info.frame_handling, Some("Absolute".to_string()));
    }

    #[test]
    fn test_to_cortical_type_info_percentage() {
        let mut area = CorticalArea::new(
            "b21vdDAwX18=".to_string(),
            0,
            "Motor Output".to_string(),
            Dimensions::new(10, 10, 1),
            (0, 0, 0),
            AreaType::Motor,
        )
        .unwrap();

        area = area.with_cortical_type_new(CorticalAreaType::BrainOutput(
            IOCorticalAreaDataType::Percentage(
                FrameChangeHandling::Absolute,
                feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::PercentageNeuronPositioning::Linear,
            ),
        ));

        let type_info = to_cortical_type_info(&area).unwrap();
        assert_eq!(type_info.category, "OPU");
        assert_eq!(type_info.data_type, Some("Percentage".to_string()));
        assert_eq!(type_info.frame_handling, Some("Absolute".to_string()));
        assert!(type_info.encoding_details.is_some());
    }

    #[test]
    fn test_to_cortical_type_info_none_for_legacy() {
        let area = CorticalArea::new(
            "b21vdDAwX18=".to_string(),
            0,
            "Legacy Area".to_string(),
            Dimensions::new(10, 10, 1),
            (0, 0, 0),
            AreaType::Motor,
        )
        .unwrap();

        // Legacy area without cortical_type_new
        let type_info = to_cortical_type_info(&area);
        assert!(type_info.is_none());
    }
}

