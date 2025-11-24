/*!
Cortical Type Adapter - Conversion utilities for CorticalAreaType

This module provides clean conversion from genome properties to the
strongly-typed CorticalAreaType system from feagi-data-processing.

Forward-looking design:
- Parse genome properties ONCE to CorticalAreaType during load
- All runtime logic uses CorticalAreaType
- No bidirectional lossy conversions
- No fallbacks (returns Result)

Eventually (Phase 6), old AreaType will be removed entirely.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use feagi_data_structures::genomic::cortical_area::{
    CorticalAreaType, CoreCorticalType, CustomCorticalType, MemoryCorticalType,
    IOCorticalAreaDataType,
};
use feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::{
    FrameChangeHandling, PercentageNeuronPositioning,
};
use thiserror::Error;

/// Type alias for the authoritative CorticalAreaType from feagi-data-processing
/// 
/// This is NOT a duplicate - it's just a convenient alias for external crates.
/// The single source of truth is feagi_data_structures::genomic::cortical_area::CorticalAreaType
pub type NewCorticalAreaType = CorticalAreaType;

#[derive(Debug, Error)]
pub enum CorticalTypeError {
    #[error("Unknown cortical_group: {0}")]
    UnknownGroup(String),
    
    #[error("Missing required cortical_group property")]
    MissingGroup,
}

/// Utilities for parsing cortical types from genome properties
pub struct CorticalTypeAdapter;

impl CorticalTypeAdapter {
    /// Parse CorticalAreaType from cortical_group property
    ///
    /// This is the PRIMARY conversion path during genome load.
    /// All genome areas MUST have a cortical_group property.
    ///
    /// # Arguments
    /// * `cortical_group` - The cortical_group string from genome (CORE, IPU, OPU, MEMORY, CUSTOM)
    ///
    /// # Returns
    /// * `Ok(CorticalAreaType)` - Parsed type
    /// * `Err(CorticalTypeError)` - If group is unknown
    ///
    /// # Note
    /// For IPU/OPU types, this returns sensible defaults for FrameChangeHandling
    /// and PercentageNeuronPositioning. Later phases will parse these from
    /// additional genome properties.
    pub fn parse_from_cortical_group(cortical_group: &str) -> Result<CorticalAreaType, CorticalTypeError> {
        match cortical_group.to_uppercase().as_str() {
            "CORE" => Ok(CorticalAreaType::Core(CoreCorticalType::Power)),
            "IPU" => Ok(CorticalAreaType::BrainInput(
                IOCorticalAreaDataType::CartesianPlane(FrameChangeHandling::Absolute)
            )),
            "OPU" => Ok(CorticalAreaType::BrainOutput(
                IOCorticalAreaDataType::Percentage(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Linear
                )
            )),
            "MEMORY" => Ok(CorticalAreaType::Memory(MemoryCorticalType::Memory)),
            "CUSTOM" => Ok(CorticalAreaType::Custom(CustomCorticalType::LeakyIntegrateFire)),
            unknown => Err(CorticalTypeError::UnknownGroup(unknown.to_string())),
        }
    }

    /// Get cortical_group string from CorticalAreaType
    ///
    /// Used when serializing genome to JSON.
    pub fn to_cortical_group(cortical_type: &CorticalAreaType) -> &'static str {
        match cortical_type {
            CorticalAreaType::Core(_) => "CORE",
            CorticalAreaType::BrainInput(_) => "IPU",
            CorticalAreaType::BrainOutput(_) => "OPU",
            CorticalAreaType::Memory(_) => "MEMORY",
            CorticalAreaType::Custom(_) => "CUSTOM",
        }
    }

    /// Check if a cortical type is an input (IPU)
    #[inline]
    pub fn is_input(cortical_type: &CorticalAreaType) -> bool {
        matches!(cortical_type, CorticalAreaType::BrainInput(_))
    }

    /// Check if a cortical type is an output (OPU)
    #[inline]
    pub fn is_output(cortical_type: &CorticalAreaType) -> bool {
        matches!(cortical_type, CorticalAreaType::BrainOutput(_))
    }

    /// Check if a cortical type is core
    #[inline]
    pub fn is_core(cortical_type: &CorticalAreaType) -> bool {
        matches!(cortical_type, CorticalAreaType::Core(_))
    }

    /// Check if a cortical type is memory
    #[inline]
    pub fn is_memory(cortical_type: &CorticalAreaType) -> bool {
        matches!(cortical_type, CorticalAreaType::Memory(_))
    }

    /// Check if a cortical type is custom
    #[inline]
    pub fn is_custom(cortical_type: &CorticalAreaType) -> bool {
        matches!(cortical_type, CorticalAreaType::Custom(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ipu() {
        let cortical_type = CorticalTypeAdapter::parse_from_cortical_group("IPU").unwrap();
        assert!(matches!(cortical_type, CorticalAreaType::BrainInput(_)));
        assert_eq!(CorticalTypeAdapter::to_cortical_group(&cortical_type), "IPU");
    }

    #[test]
    fn test_parse_opu() {
        let cortical_type = CorticalTypeAdapter::parse_from_cortical_group("OPU").unwrap();
        assert!(matches!(cortical_type, CorticalAreaType::BrainOutput(_)));
        assert_eq!(CorticalTypeAdapter::to_cortical_group(&cortical_type), "OPU");
    }

    #[test]
    fn test_parse_core() {
        let cortical_type = CorticalTypeAdapter::parse_from_cortical_group("CORE").unwrap();
        assert!(matches!(cortical_type, CorticalAreaType::Core(_)));
        assert_eq!(CorticalTypeAdapter::to_cortical_group(&cortical_type), "CORE");
    }

    #[test]
    fn test_parse_memory() {
        let cortical_type = CorticalTypeAdapter::parse_from_cortical_group("MEMORY").unwrap();
        assert!(matches!(cortical_type, CorticalAreaType::Memory(_)));
        assert_eq!(CorticalTypeAdapter::to_cortical_group(&cortical_type), "MEMORY");
    }

    #[test]
    fn test_parse_custom() {
        let cortical_type = CorticalTypeAdapter::parse_from_cortical_group("CUSTOM").unwrap();
        assert!(matches!(cortical_type, CorticalAreaType::Custom(_)));
        assert_eq!(CorticalTypeAdapter::to_cortical_group(&cortical_type), "CUSTOM");
    }

    #[test]
    fn test_parse_case_insensitive() {
        let upper = CorticalTypeAdapter::parse_from_cortical_group("IPU").unwrap();
        let lower = CorticalTypeAdapter::parse_from_cortical_group("ipu").unwrap();
        let mixed = CorticalTypeAdapter::parse_from_cortical_group("IpU").unwrap();
        
        assert!(matches!(upper, CorticalAreaType::BrainInput(_)));
        assert!(matches!(lower, CorticalAreaType::BrainInput(_)));
        assert!(matches!(mixed, CorticalAreaType::BrainInput(_)));
    }

    #[test]
    fn test_parse_unknown_group_fails() {
        let result = CorticalTypeAdapter::parse_from_cortical_group("UNKNOWN");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CorticalTypeError::UnknownGroup(_)));
    }

    #[test]
    fn test_is_input() {
        let input_type = CorticalAreaType::BrainInput(
            IOCorticalAreaDataType::CartesianPlane(FrameChangeHandling::Absolute)
        );
        assert!(CorticalTypeAdapter::is_input(&input_type));
        assert!(!CorticalTypeAdapter::is_output(&input_type));
    }

    #[test]
    fn test_is_core() {
        let core_type = CorticalAreaType::Core(CoreCorticalType::Power);
        assert!(CorticalTypeAdapter::is_core(&core_type));
        assert!(!CorticalTypeAdapter::is_input(&core_type));
    }
}

