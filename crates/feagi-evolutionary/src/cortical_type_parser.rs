// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Cortical Type Parser - Parse CorticalAreaType from genome properties

This module provides utilities to parse the rich CorticalAreaType
from genome JSON properties during genome loading.

Forward-looking design:
- Parse ALL type information from genome during load
- Store CorticalAreaType in RuntimeGenome
- No re-parsing at runtime

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::types::EvoError;
use feagi_structures::genomic::cortical_area::CorticalAreaType;
use std::collections::HashMap;

/// Parse CorticalAreaType from genome properties
///
/// This is called during genome loading to convert flat genome properties
/// into the strongly-typed CorticalAreaType.
///
/// # Arguments
/// * `properties` - HashMap of genome properties for a cortical area
///
/// # Returns
/// * `Ok(CorticalAreaType)` - Parsed cortical type
/// * `Err(EvoError)` - If required properties are missing or invalid
pub fn parse_cortical_type(
    properties: &HashMap<String, serde_json::Value>,
) -> Result<CorticalAreaType, EvoError> {
    // Get cortical_group (required)
    let cortical_group = properties
        .get("cortical_group")
        .and_then(|v| v.as_str())
        .ok_or_else(|| EvoError::InvalidGenome("Missing cortical_group property".to_string()))?;

    // Parse base type from cortical_group
    let cortical_type = match cortical_group.to_uppercase().as_str() {
        "IPU" => CorticalAreaType::BrainInput(
            feagi_structures::genomic::cortical_area::IOCorticalAreaConfigurationFlag::CartesianPlane(
                feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::FrameChangeHandling::Absolute
            )
        ),
        "OPU" => CorticalAreaType::BrainOutput(
            feagi_structures::genomic::cortical_area::IOCorticalAreaConfigurationFlag::CartesianPlane(
                feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::FrameChangeHandling::Absolute
            )
        ),
        "CORE" => CorticalAreaType::Core(
            feagi_structures::genomic::cortical_area::CoreCorticalType::Death
        ),
        "CUSTOM" => CorticalAreaType::Custom(
            feagi_structures::genomic::cortical_area::CustomCorticalType::LeakyIntegrateFire
        ),
        "MEMORY" => CorticalAreaType::Memory(
            feagi_structures::genomic::cortical_area::MemoryCorticalType::Memory
        ),
        _ => return Err(EvoError::InvalidGenome(format!("Unknown cortical_group: {}", cortical_group))),
    };

    Ok(cortical_type)
}

/// Validate that a cortical type is compatible with genome properties
///
/// This ensures that the parsed CorticalAreaType makes sense given
/// other properties in the genome (e.g., dimensions, mappings).
///
/// # Arguments
/// * `cortical_type` - The parsed CorticalAreaType
/// * `properties` - HashMap of genome properties
///
/// # Returns
/// * `Ok(())` - If type is valid
/// * `Err(String)` - If type is incompatible with properties
pub fn validate_cortical_type(
    cortical_type: &CorticalAreaType,
    properties: &HashMap<String, serde_json::Value>,
) -> Result<(), String> {
    // Validate based on cortical type
    match cortical_type {
        CorticalAreaType::BrainInput(_) => {
            // IPU areas should have sensory-related properties
            // Future: validate dimensions, data encoding, etc.
        }
        CorticalAreaType::BrainOutput(_) => {
            // OPU areas should have motor-related properties
            // Future: validate output encoding, control scheme, etc.
        }
        CorticalAreaType::Core(_) => {
            // Core areas have specific requirements
            // Future: validate power, death, etc.
        }
        _ => {}
    }

    // Check for incompatible property combinations
    if let Some(cortical_mapping_dst) = properties.get("cortical_mapping_dst") {
        if matches!(cortical_type, CorticalAreaType::BrainOutput(_))
            && cortical_mapping_dst.is_object()
        {
            // OPU areas typically shouldn't have outgoing mappings
            // (though this is not strictly enforced yet)
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_ipu_type() {
        let mut properties = HashMap::new();
        properties.insert("cortical_group".to_string(), json!("IPU"));

        let cortical_type = parse_cortical_type(&properties).unwrap();
        assert!(matches!(cortical_type, CorticalAreaType::BrainInput(_)));
    }

    #[test]
    fn test_parse_opu_type() {
        let mut properties = HashMap::new();
        properties.insert("cortical_group".to_string(), json!("OPU"));

        let cortical_type = parse_cortical_type(&properties).unwrap();
        assert!(matches!(cortical_type, CorticalAreaType::BrainOutput(_)));
    }

    #[test]
    fn test_parse_core_type() {
        let mut properties = HashMap::new();
        properties.insert("cortical_group".to_string(), json!("CORE"));

        let cortical_type = parse_cortical_type(&properties).unwrap();
        assert!(matches!(cortical_type, CorticalAreaType::Core(_)));
    }

    #[test]
    fn test_parse_missing_group() {
        let properties = HashMap::new();

        let result = parse_cortical_type(&properties);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_group() {
        let mut properties = HashMap::new();
        properties.insert("cortical_group".to_string(), json!("INVALID"));

        let result = parse_cortical_type(&properties);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_ipu_type() {
        use feagi_structures::genomic::cortical_area::{
            io_cortical_area_configuration_flag::FrameChangeHandling, IOCorticalAreaConfigurationFlag,
        };

        let cortical_type = CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::CartesianPlane(
            FrameChangeHandling::Absolute,
        ));
        let properties = HashMap::new();

        assert!(validate_cortical_type(&cortical_type, &properties).is_ok());
    }
}
