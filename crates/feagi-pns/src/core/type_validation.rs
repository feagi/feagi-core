// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Type-Aware Validation for PNS (Phase 4)

Provides optional validation and routing optimization based on detailed
cortical type information from feagi-data-processing.

This module enhances agent registration without breaking existing functionality:
- Validates agent capabilities against cortical area types
- Provides routing hints based on data encoding
- Enables future optimizations (compression, buffering strategies)

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use feagi_data_structures::genomic::cortical_area::{
    CorticalAreaType, IOCorticalAreaDataType,
};
use feagi_types::{CorticalArea, CorticalTypeAdapter};

/// Validation result for agent-area compatibility
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_compatible: bool,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
}

impl ValidationResult {
    pub fn compatible() -> Self {
        Self {
            is_compatible: true,
            warnings: Vec::new(),
            recommendations: Vec::new(),
        }
    }

    pub fn incompatible(reason: String) -> Self {
        Self {
            is_compatible: false,
            warnings: vec![reason],
            recommendations: Vec::new(),
        }
    }

    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }

    pub fn with_recommendation(mut self, recommendation: String) -> Self {
        self.recommendations.push(recommendation);
        self
    }
}

/// Validate that an agent's sensory capability is compatible with a cortical area
///
/// Phase 4: Basic compatibility checks
/// Future: Add detailed validation based on IOCorticalAreaDataType
pub fn validate_sensory_compatibility(
    agent_id: &str,
    agent_modality: &str,
    area: &CorticalArea,
) -> ValidationResult {
    // Check if area is an input area
    if let Some(ref cortical_type) = area.cortical_type_new {
        if !CorticalTypeAdapter::is_input(cortical_type) {
            return ValidationResult::incompatible(format!(
                "Agent {} (modality: {}) trying to connect to non-IPU area {}",
                agent_id, agent_modality, area.cortical_id
            ));
        }

        // Phase 4: Provide recommendations based on IOCorticalAreaDataType
        if let CorticalAreaType::BrainInput(io_type) = cortical_type {
            let mut result = ValidationResult::compatible();

            match io_type {
                IOCorticalAreaDataType::CartesianPlane(_) => {
                    if !agent_modality.to_lowercase().contains("vision")
                        && !agent_modality.to_lowercase().contains("camera")
                    {
                        result = result.with_warning(format!(
                            "Area {} expects CartesianPlane data (vision), but agent modality is '{}'",
                            area.cortical_id, agent_modality
                        ));
                    }
                }
                IOCorticalAreaDataType::Percentage(_, _)
                | IOCorticalAreaDataType::SignedPercentage(_, _) => {
                    result = result.with_recommendation(format!(
                        "Area {} uses percentage encoding - ensure data is normalized 0-100%",
                        area.cortical_id
                    ));
                }
                _ => {}
            }

            return result;
        }
    }

    // Fallback: No detailed type info available, assume compatible
    ValidationResult::compatible()
}

/// Validate that an agent's motor capability is compatible with a cortical area
///
/// Phase 4: Basic compatibility checks
/// Future: Add detailed validation based on IOCorticalAreaDataType
pub fn validate_motor_compatibility(
    agent_id: &str,
    agent_modality: &str,
    area: &CorticalArea,
) -> ValidationResult {
    // Check if area is an output area
    if let Some(ref cortical_type) = area.cortical_type_new {
        if !CorticalTypeAdapter::is_output(cortical_type) {
            return ValidationResult::incompatible(format!(
                "Agent {} (modality: {}) trying to connect to non-OPU area {}",
                agent_id, agent_modality, area.cortical_id
            ));
        }

        // Phase 4: Provide recommendations based on IOCorticalAreaDataType
        if let CorticalAreaType::BrainOutput(io_type) = cortical_type {
            let mut result = ValidationResult::compatible();

            match io_type {
                IOCorticalAreaDataType::Percentage(_, _)
                | IOCorticalAreaDataType::SignedPercentage(_, _) => {
                    result = result.with_recommendation(format!(
                        "Area {} outputs percentage values (0-100%) - ensure actuators are calibrated accordingly",
                        area.cortical_id
                    ));
                }
                _ => {}
            }

            return result;
        }
    }

    // Fallback: No detailed type info available, assume compatible
    ValidationResult::compatible()
}

/// Get recommended buffer size based on cortical area type
///
/// Phase 4: Returns sensible defaults based on data type
/// Future: More sophisticated sizing based on dimensions and encoding
pub fn get_recommended_buffer_size(area: &CorticalArea) -> usize {
    if let Some(ref cortical_type) = area.cortical_type_new {
        if let Some(io_type) = match cortical_type {
            CorticalAreaType::BrainInput(t) => Some(t),
            CorticalAreaType::BrainOutput(t) => Some(t),
            _ => None,
        } {
            return match io_type {
                IOCorticalAreaDataType::CartesianPlane(_) => {
                    // Vision typically needs larger buffers
                    area.dimensions.volume() * 4 // 4 bytes per voxel
                }
                IOCorticalAreaDataType::Percentage(_, _)
                | IOCorticalAreaDataType::SignedPercentage(_, _) => {
                    // Percentage encoding is compact
                    area.dimensions.volume() * 2 // 2 bytes per voxel
                }
                _ => {
                    // Default sizing
                    area.dimensions.volume() * 2
                }
            };
        }
    }

    // Fallback: Default buffer size
    area.dimensions.volume() * 2
}

/// Check if compression is recommended for this area type
///
/// Phase 4: Basic heuristics
/// Future: More sophisticated compression strategies
pub fn should_use_compression(area: &CorticalArea) -> bool {
    if let Some(ref cortical_type) = area.cortical_type_new {
        if let Some(io_type) = match cortical_type {
            CorticalAreaType::BrainInput(t) => Some(t),
            CorticalAreaType::BrainOutput(t) => Some(t),
            _ => None,
        } {
            return match io_type {
                IOCorticalAreaDataType::CartesianPlane(_) => {
                    // Vision data benefits from compression
                    area.dimensions.volume() > 1000
                }
                _ => {
                    // Small data doesn't benefit from compression
                    area.dimensions.volume() > 500
                }
            };
        }
    }

    // Fallback: Compress if large
    area.dimensions.volume() > 1000
}

#[cfg(test)]
mod tests {
    use super::*;
    use feagi_types::{AreaType, Dimensions};
use feagi_data_structures::genomic::cortical_area::CorticalID;

    #[test]
    fn test_validate_sensory_compatibility() {
        // Create IPU area with CartesianPlane
        let mut area = CorticalArea::new(
            CorticalID::try_from_base_64("aWljMDAwX18=").unwrap(),
            0,
            "Vision Input".to_string(),
            Dimensions::new(128, 128, 3),
            (0, 0, 0),
        )
        .unwrap();

        area = area.with_cortical_type_new(CorticalAreaType::BrainInput(
            IOCorticalAreaDataType::CartesianPlane(
                feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::FrameChangeHandling::Absolute,
            ),
        ));

        // Compatible: vision modality with CartesianPlane area
        let result = validate_sensory_compatibility("agent1", "vision", &area);
        assert!(result.is_compatible);

        // Warning: non-vision modality with CartesianPlane area
        let result = validate_sensory_compatibility("agent1", "audio", &area);
        assert!(result.is_compatible); // Still compatible, but has warning
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_validate_motor_compatibility() {
        // Create OPU area
        let mut area = CorticalArea::new(
            CorticalID::try_from_base_64("b21vdDAwX18=").unwrap(),
            0,
            "Motor Output".to_string(),
            Dimensions::new(10, 10, 1),
            (0, 0, 0),
        )
        .unwrap();

        area = area.with_cortical_type_new(CorticalAreaType::BrainOutput(
            IOCorticalAreaDataType::Percentage(
                feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::FrameChangeHandling::Absolute,
                feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::PercentageNeuronPositioning::Linear,
            ),
        ));

        // Compatible: motor modality with OPU area
        let result = validate_motor_compatibility("agent1", "servo", &area);
        assert!(result.is_compatible);
    }

    #[test]
    fn test_get_recommended_buffer_size() {
        let mut area = CorticalArea::new(
            CorticalID::try_from_base_64("aWljMDAwX18=").unwrap(),
            0,
            "Vision Input".to_string(),
            Dimensions::new(128, 128, 3),
            (0, 0, 0),
        )
        .unwrap();

        area = area.with_cortical_type_new(CorticalAreaType::BrainInput(
            IOCorticalAreaDataType::CartesianPlane(
                feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::FrameChangeHandling::Absolute,
            ),
        ));

        // CartesianPlane should get 4 bytes per voxel
        let buffer_size = get_recommended_buffer_size(&area);
        assert_eq!(buffer_size, 128 * 128 * 3 * 4);
    }

    #[test]
    fn test_should_use_compression() {
        let mut large_area = CorticalArea::new(
            CorticalID::try_from_base_64("aWljMDAwX18=").unwrap(),
            0,
            "Large Vision".to_string(),
            Dimensions::new(128, 128, 3),
            (0, 0, 0),
        )
        .unwrap();

        large_area = large_area.with_cortical_type_new(CorticalAreaType::BrainInput(
            IOCorticalAreaDataType::CartesianPlane(
                feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::FrameChangeHandling::Absolute,
            ),
        ));

        // Large vision area should use compression
        assert!(should_use_compression(&large_area));
    }
}

