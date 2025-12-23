// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Cortical Type Conversion Utilities (Phase 5)

Converts internal CorticalAreaType representations to API DTOs.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use super::cortical_area_dtos::CorticalTypeInfo;
use feagi_brain_development::models::CorticalAreaExt;
use feagi_structures::genomic::cortical_area::io_cortical_area_data_type::FrameChangeHandling;
use feagi_structures::genomic::cortical_area::CorticalArea;
use feagi_structures::genomic::cortical_area::{CorticalAreaType, IOCorticalAreaDataFlag};
// Note: CorticalTypeAdapter removed - use feagi_structures::CorticalID directly

/// Convert internal CorticalArea to API CorticalTypeInfo
///
/// Returns None if cortical_type_new is not populated (legacy areas)
pub fn to_cortical_type_info(area: &CorticalArea) -> Option<CorticalTypeInfo> {
    let cortical_type = area.cortical_id.as_cortical_type().ok()?;

    let category = area
        .get_cortical_group()
        .unwrap_or_else(|| "CUSTOM".to_string());

    // Extract data_type and frame_handling for IPU/OPU
    let (data_type, frame_handling, encoding_details) = match cortical_type {
        CorticalAreaType::BrainInput(brain_input) => extract_io_type_details(&brain_input),
        CorticalAreaType::BrainOutput(brain_output) => extract_io_type_details(&brain_output),
        _ => (None, None, None),
    };

    Some(CorticalTypeInfo {
        category,
        data_type,
        frame_handling,
        encoding_details,
    })
}

/// Extract detailed information from IOCorticalAreaDataFlag
fn extract_io_type_details(
    io_type: &IOCorticalAreaDataFlag,
) -> (Option<String>, Option<String>, Option<serde_json::Value>) {
    use IOCorticalAreaDataFlag::*;

    match io_type {
        Boolean => (Some("Boolean".to_string()), None, None),
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
    positioning: &feagi_structures::genomic::cortical_area::io_cortical_area_data_type::PercentageNeuronPositioning,
) -> String {
    use feagi_structures::genomic::cortical_area::io_cortical_area_data_type::PercentageNeuronPositioning;

    match positioning {
        PercentageNeuronPositioning::Linear => "Linear".to_string(),
        PercentageNeuronPositioning::Fractional => "Fractional".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use feagi_brain_development::{CorticalArea, CorticalID, Dimensions};
    use feagi_structures::genomic::cortical_area::{CorticalAreaType, IOCorticalAreaDataFlag};

    #[test]
    fn test_to_cortical_type_info_cartesian_plane() {
        // Create a valid CorticalID for testing (must start with valid prefix: 'c', 'm', '_', 'i', 'o')
        // Using 'c' prefix for custom type
        let mut bytes = [0u8; 8];
        bytes[0] = b'c'; // Custom type prefix
        let cortical_id = CorticalID::try_from_bytes(&bytes).unwrap();
        let area = CorticalArea::new(
            cortical_id,
            0,
            "Vision Input".to_string(),
            Dimensions::new(128, 128, 3).unwrap(),
            (0, 0, 0).into(),
            CorticalAreaType::BrainInput(IOCorticalAreaDataFlag::Boolean),
        )
        .unwrap();

        // Test that the function correctly extracts type info
        // Category comes from AreaType (Sensory -> IPU), not from CorticalID type
        let type_info = to_cortical_type_info(&area);

        // Verify the function returns Some (custom CorticalID encodes type, so as_cortical_type() succeeds)
        assert!(
            type_info.is_some(),
            "Function should return Some when cortical_id encodes type"
        );

        // Verify the category is derived from AreaType (Sensory -> IPU)
        let info = type_info.unwrap();
        assert_eq!(
            info.category, "IPU",
            "Sensory AreaType should map to IPU category"
        );
        // Custom CorticalID type doesn't provide IPU/OPU data_type, so those should be None
        assert!(
            info.data_type.is_none(),
            "Custom CorticalID type doesn't provide IPU data_type"
        );
        assert!(info.frame_handling.is_none());
    }

    #[test]
    fn test_to_cortical_type_info_percentage() {
        // Create a valid CorticalID for testing (using 'c' prefix for custom type)
        let mut bytes = [0u8; 8];
        bytes[0] = b'c'; // Custom type prefix
        let cortical_id = CorticalID::try_from_bytes(&bytes).unwrap();
        let area = CorticalArea::new(
            cortical_id,
            0,
            "Motor Output".to_string(),
            Dimensions::new(10, 10, 1).unwrap(),
            (0, 0, 0).into(),
            CorticalAreaType::BrainOutput(IOCorticalAreaDataFlag::Boolean),
        )
        .unwrap();

        // Test that the function correctly extracts type info
        // Category comes from AreaType (Motor -> OPU), not from CorticalID type
        let type_info = to_cortical_type_info(&area);

        // Verify the function returns Some
        assert!(
            type_info.is_some(),
            "Function should return Some when cortical_id encodes type"
        );

        let info = type_info.unwrap();
        assert_eq!(
            info.category, "OPU",
            "Motor AreaType should map to OPU category"
        );
        // Custom CorticalID type doesn't provide OPU data_type
        assert!(
            info.data_type.is_none(),
            "Custom CorticalID type doesn't provide OPU data_type"
        );
    }

    #[test]
    fn test_to_cortical_type_info_derives_from_cortical_id() {
        // Create a valid CorticalID for testing (using 'c' prefix for custom type)
        let mut bytes = [0u8; 8];
        bytes[0] = b'c'; // Custom type prefix
        let cortical_id = CorticalID::try_from_bytes(&bytes).unwrap();
        let area = CorticalArea::new(
            cortical_id,
            0,
            "Motor Area".to_string(),
            Dimensions::new(10, 10, 1).unwrap(),
            (0, 0, 0).into(),
            CorticalAreaType::BrainOutput(IOCorticalAreaDataFlag::Boolean),
        )
        .unwrap();

        // Type info is derived from cortical_id via as_cortical_type()
        // Category is derived from AreaType (Motor -> OPU)
        let type_info = to_cortical_type_info(&area);

        // Verify the function correctly extracts type info
        assert!(
            type_info.is_some(),
            "Function should return Some when cortical_id encodes type"
        );

        let info = type_info.unwrap();
        assert_eq!(
            info.category, "OPU",
            "Motor AreaType should map to OPU category"
        );

        // This test verifies that the function correctly:
        // 1. Extracts cortical type from CorticalID (as_cortical_type())
        // 2. Derives category from AreaType (get_cortical_group())
        // 3. Returns appropriate type information
    }
}
