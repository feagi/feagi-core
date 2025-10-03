//! Tests for the genomic module
//! 
//! This module contains comprehensive tests for genomic data structures
//! including cortical types, cortical IDs, descriptors, and their relationships.

use feagi_data_structures::genomic::*;
use feagi_data_structures::genomic::descriptors::*;
use feagi_data_structures::common_macros::*;
use feagi_data_structures::FeagiDataError;
use std::ops::Range;

#[cfg(test)]
mod test_descriptors {
    use super::*;

    mod test_indices {
        use super::*;

        #[test]
        fn test_cortical_group_index_creation() {
            let index = CorticalGroupIndex::from(42u8);
            assert_eq!(*index, 42u8);
        }

        #[test]
        fn test_cortical_group_index_deref() {
            let index = CorticalGroupIndex::from(100u8);
            assert_eq!(*index, 100u8);
        }

        #[test]
        fn test_cortical_group_index_max_value() {
            let index = CorticalGroupIndex::from(u8::MAX);
            assert_eq!(*index, u8::MAX);
        }

        #[test]
        fn test_cortical_group_index_zero() {
            let index = CorticalGroupIndex::from(0u8);
            assert_eq!(*index, 0u8);
        }

        #[test]
        fn test_cortical_group_index_equality() {
            let index1 = CorticalGroupIndex::from(50u8);
            let index2 = CorticalGroupIndex::from(50u8);
            let index3 = CorticalGroupIndex::from(51u8);
            
            assert_eq!(index1, index2);
            assert_ne!(index1, index3);
        }

        #[test]
        fn test_cortical_group_index_ordering() {
            let index1 = CorticalGroupIndex::from(10u8);
            let index2 = CorticalGroupIndex::from(20u8);
            
            assert!(index1 < index2);
            assert!(index2 > index1);
        }

        #[test]
        fn test_cortical_channel_index_creation() {
            let index = CorticalChannelIndex::from(1000u32);
            assert_eq!(*index, 1000u32);
        }

        #[test]
        fn test_cortical_channel_index_max_value() {
            let index = CorticalChannelIndex::from(u32::MAX);
            assert_eq!(*index, u32::MAX);
        }

        #[test]
        fn test_agent_device_index_creation() {
            let index = AgentDeviceIndex::from(500u32);
            assert_eq!(*index, 500u32);
        }

        #[test]
        fn test_agent_device_index_zero() {
            let index = AgentDeviceIndex::from(0u32);
            assert_eq!(*index, 0u32);
        }
    }

    mod test_counts {
        use super::*;

        #[test]
        fn test_cortical_channel_count_valid() {
            let count = CorticalChannelCount::new(5).unwrap();
            assert_eq!(*count, 5u32);
        }

        #[test]
        fn test_cortical_channel_count_minimum() {
            let count = CorticalChannelCount::new(1).unwrap();
            assert_eq!(*count, 1u32);
        }

        #[test]
        fn test_cortical_channel_count_large() {
            let count = CorticalChannelCount::new(u32::MAX).unwrap();
            assert_eq!(*count, u32::MAX);
        }

        #[test]
        fn test_cortical_channel_count_zero_error() {
            let result = CorticalChannelCount::new(0);
            assert!(result.is_err());
            if let Err(FeagiDataError::BadParameters(msg)) = result {
                assert_eq!(msg, "Count cannot be zero!");
            } else {
                panic!("Expected BadParameters error");
            }
        }

        #[test]
        fn test_cortical_channel_count_display() {
            let count = CorticalChannelCount::new(42).unwrap();
            let display_string = format!("{}", count);
            assert_eq!(display_string, "42");
        }

        #[test]
        fn test_cortical_channel_count_from_conversion() {
            let count = CorticalChannelCount::from(10u32);
            assert_eq!(*count, 10u32);
        }

        #[test]
        fn test_neuron_depth_valid() {
            let depth = NeuronDepth::new(3).unwrap();
            assert_eq!(*depth, 3u32);
        }

        #[test]
        fn test_neuron_depth_zero_error() {
            let result = NeuronDepth::new(0);
            assert!(result.is_err());
            if let Err(FeagiDataError::BadParameters(msg)) = result {
                assert_eq!(msg, "Count cannot be zero!");
            } else {
                panic!("Expected BadParameters error");
            }
        }

        #[test]
        fn test_neuron_depth_max_value() {
            let depth = NeuronDepth::new(u32::MAX).unwrap();
            assert_eq!(*depth, u32::MAX);
        }
    }

    mod test_coordinates {
        use super::*;

        #[test]
        fn test_cortical_coordinate_creation() {
            let coord = CorticalCoordinate::new(10, 20, 30);
            assert_eq!(coord.x, 10);
            assert_eq!(coord.y, 20);
            assert_eq!(coord.z, 30);
        }

        #[test]
        fn test_cortical_coordinate_zero_values() {
            let coord = CorticalCoordinate::new(0, 0, 0);
            assert_eq!(coord.x, 0);
            assert_eq!(coord.y, 0);
            assert_eq!(coord.z, 0);
        }

        #[test]
        fn test_cortical_coordinate_max_values() {
            let coord = CorticalCoordinate::new(u32::MAX, u32::MAX, u32::MAX);
            assert_eq!(coord.x, u32::MAX);
            assert_eq!(coord.y, u32::MAX);
            assert_eq!(coord.z, u32::MAX);
        }

        #[test]
        fn test_genome_coordinate_creation() {
            let coord = GenomeCoordinate::new(-10, 0, 20);
            assert_eq!(coord.x, -10);
            assert_eq!(coord.y, 0);
            assert_eq!(coord.z, 20);
        }

        #[test]
        fn test_genome_coordinate_negative_values() {
            let coord = GenomeCoordinate::new(-100, -200, -300);
            assert_eq!(coord.x, -100);
            assert_eq!(coord.y, -200);
            assert_eq!(coord.z, -300);
        }

        #[test]
        fn test_genome_coordinate_extreme_values() {
            let coord = GenomeCoordinate::new(i32::MAX, i32::MIN, 0);
            assert_eq!(coord.x, i32::MAX);
            assert_eq!(coord.y, i32::MIN);
            assert_eq!(coord.z, 0);
        }

    }

    mod test_dimensions {
        use super::*;

        #[test]
        fn test_cortical_channel_dimensions_valid() {
            let dims = CorticalChannelDimensions::new(10, 20, 30).unwrap();
            assert_eq!(dims.width, 10);
            assert_eq!(dims.height, 20);
            assert_eq!(dims.depth, 30);
        }

        #[test]
        fn test_cortical_channel_dimensions_minimum() {
            let dims = CorticalChannelDimensions::new(1, 1, 1).unwrap();
            assert_eq!(dims.width, 1);
            assert_eq!(dims.height, 1);
            assert_eq!(dims.depth, 1);
        }

        #[test]
        fn test_cortical_channel_dimensions_zero_error() {
            let result = CorticalChannelDimensions::new(0, 10, 10);
            assert!(result.is_err());
        }

        #[test]
        fn test_cortical_dimensions_valid() {
            let dims = CorticalDimensions::new(100, 200, 300).unwrap();
            assert_eq!(dims.width, 100);
            assert_eq!(dims.height, 200);
            assert_eq!(dims.depth, 300);
        }
    }

    mod test_dimension_ranges {
        use super::*;

        #[test]
        fn test_cortical_channel_dimension_range_valid() {
            let range = CorticalChannelDimensionRange::new(0..10, 5..15, 10..20).unwrap();
            assert_eq!(range.width, 0..10);
            assert_eq!(range.height, 5..15);
            assert_eq!(range.depth, 10..20);
        }
    }
}

#[cfg(test)]
mod test_cortical_types {
    use super::*;

    mod test_core_cortical_type {
        use super::*;

        #[test]
        fn test_core_cortical_type_death() {
            let core_type = CoreCorticalType::Death;
            let cortical_id = core_type.to_cortical_id();
            assert_eq!(cortical_id.as_ascii_string(), "_death");
        }

        #[test]
        fn test_core_cortical_type_power() {
            let core_type = CoreCorticalType::Power;
            let cortical_id = core_type.to_cortical_id();
            assert_eq!(cortical_id.as_ascii_string(), "_power");
        }

        #[test]
        fn test_core_cortical_type_display() {
            let death_type = CoreCorticalType::Death;
            let power_type = CoreCorticalType::Power;
            
            assert_eq!(format!("{}", death_type), "CoreCorticalType(Death)");
            assert_eq!(format!("{}", power_type), "CoreCorticalType(Power)");
        }

        #[test]
        fn test_core_cortical_type_conversion_to_cortical_type() {
            let core_type = CoreCorticalType::Death;
            let cortical_type: CorticalType = core_type.into();
            
            match cortical_type {
                CorticalType::Core(CoreCorticalType::Death) => (),
                _ => panic!("Expected Core(Death) cortical type"),
            }
        }
    }

    mod test_cortical_type {
        use super::*;

        #[test]
        fn test_cortical_type_is_type_checks() {
            let custom = CorticalType::Custom;
            let memory = CorticalType::Memory;
            let core = CorticalType::Core(CoreCorticalType::Death);
            
            assert!(custom.is_type_custom());
            assert!(!custom.is_type_memory());
            assert!(!custom.is_type_core());
            assert!(!custom.is_type_sensor());
            assert!(!custom.is_type_motor());
            
            assert!(memory.is_type_memory());
            assert!(!memory.is_type_custom());
            
            assert!(core.is_type_core());
            assert!(!core.is_type_custom());
        }

        #[test]
        fn test_cortical_type_verify_type_success() {
            let custom = CorticalType::Custom;
            let memory = CorticalType::Memory;
            let core = CorticalType::Core(CoreCorticalType::Power);
            
            assert!(custom.verify_is_custom().is_ok());
            assert!(memory.verify_is_memory().is_ok());
            assert!(core.verify_is_core().is_ok());
        }

        #[test]
        fn test_cortical_type_verify_type_failure() {
            let custom = CorticalType::Custom;
            
            assert!(custom.verify_is_memory().is_err());
            assert!(custom.verify_is_core().is_err());
            assert!(custom.verify_is_sensor().is_err());
            assert!(custom.verify_is_motor().is_err());
        }

        #[test]
        fn test_cortical_type_display() {
            let custom = CorticalType::Custom;
            let memory = CorticalType::Memory;
            let core = CorticalType::Core(CoreCorticalType::Death);
            
            assert_eq!(format!("{}", custom), "'Custom'");
            assert_eq!(format!("{}", memory), "'Memory'");
            assert!(format!("{}", core).contains("'Core("));
        }

        #[test]
        fn test_cortical_type_to_cortical_id_errors() {
            let custom = CorticalType::Custom;
            let memory = CorticalType::Memory;
            let index = CorticalGroupIndex::from(0u8);
            
            assert!(custom.to_cortical_id(index).is_err());
            assert!(memory.to_cortical_id(index).is_err());
        }

        #[test]
        fn test_cortical_type_to_cortical_id_success() {
            let core = CorticalType::Core(CoreCorticalType::Death);
            let index = CorticalGroupIndex::from(0u8);
            
            let result = core.to_cortical_id(index);
            assert!(result.is_ok());
            assert_eq!(result.unwrap().as_ascii_string(), "_death");
        }

        #[test]
        fn test_cortical_type_channel_size_boundaries_errors() {
            let custom = CorticalType::Custom;
            let memory = CorticalType::Memory;
            let core = CorticalType::Core(CoreCorticalType::Death);
            
            assert!(custom.try_get_channel_size_boundaries().is_err());
            assert!(memory.try_get_channel_size_boundaries().is_err());
            assert!(core.try_get_channel_size_boundaries().is_err());
        }

        #[test]
        fn test_cortical_type_from_bytes_custom() {
            let bytes = b"custom";
            let result = CorticalType::try_get_type_from_bytes(bytes).unwrap();
            assert_eq!(result, CorticalType::Custom);
        }

        #[test]
        fn test_cortical_type_from_bytes_memory() {
            let bytes = b"memory";
            let result = CorticalType::try_get_type_from_bytes(bytes).unwrap();
            assert_eq!(result, CorticalType::Memory);
        }

        #[test]
        fn test_cortical_type_from_bytes_invalid_start() {
            let bytes = b"xinval";
            let result = CorticalType::try_get_type_from_bytes(bytes);
            assert!(result.is_err());
        }
    }
}

#[cfg(test)]
mod test_cortical_id {
    use super::*;

    mod test_cortical_id_creation {
        use super::*;

        #[test]
        fn test_cortical_id_constants() {
            assert_eq!(CorticalID::CORTICAL_ID_LENGTH, 6);
            assert_eq!(CorticalID::NUMBER_OF_BYTES, 6);
            assert_eq!(CorticalID::CORTICAL_ID_LENGTH, CorticalID::NUMBER_OF_BYTES);
        }

        #[test]
        fn test_cortical_id_new_custom_valid() {
            let result = CorticalID::new_custom_cortical_area_id("cust01".to_string());
            assert!(result.is_ok());
            
            let cortical_id = result.unwrap();
            assert_eq!(cortical_id.as_ascii_string(), "cust01");
        }

        #[test]
        fn test_cortical_id_new_custom_wrong_prefix() {
            let result = CorticalID::new_custom_cortical_area_id("xust01".to_string());
            assert!(result.is_err());
            
            if let Err(FeagiDataError::BadParameters(msg)) = result {
                assert!(msg.contains("A custom cortical area ID must start with 'c'"));
            } else {
                panic!("Expected BadParameters error");
            }
        }

        #[test]
        fn test_cortical_id_new_custom_wrong_length() {
            let result = CorticalID::new_custom_cortical_area_id("cust".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn test_cortical_id_new_custom_non_ascii() {
            let result = CorticalID::new_custom_cortical_area_id("cust🎯1".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn test_cortical_id_new_custom_invalid_characters() {
            let result = CorticalID::new_custom_cortical_area_id("cust@1".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn test_cortical_id_new_memory_valid() {
            let result = CorticalID::new_memory_cortical_area_id("mem001".to_string());
            assert!(result.is_ok());
            
            let cortical_id = result.unwrap();
            assert_eq!(cortical_id.as_ascii_string(), "mem001");
        }

        #[test]
        fn test_cortical_id_new_memory_wrong_prefix() {
            let result = CorticalID::new_memory_cortical_area_id("xem001".to_string());
            assert!(result.is_err());
            
            if let Err(FeagiDataError::BadParameters(msg)) = result {
                assert!(msg.contains("A memory cortical area ID must start with 'm'"));
            } else {
                panic!("Expected BadParameters error");
            }
        }

        #[test]
        fn test_cortical_id_new_core_death() {
            let result = CorticalID::new_core_cortical_area_id(CoreCorticalType::Death);
            assert!(result.is_ok());
            
            let cortical_id = result.unwrap();
            assert_eq!(cortical_id.as_ascii_string(), "_death");
        }

        #[test]
        fn test_cortical_id_new_core_power() {
            let result = CorticalID::new_core_cortical_area_id(CoreCorticalType::Power);
            assert!(result.is_ok());
            
            let cortical_id = result.unwrap();
            assert_eq!(cortical_id.as_ascii_string(), "_power");
        }

        #[test]
        fn test_cortical_id_from_string_valid() {
            let result = CorticalID::from_string("custom".to_string());
            assert!(result.is_ok());
            
            let cortical_id = result.unwrap();
            assert_eq!(cortical_id.as_ascii_string(), "custom");
        }

        #[test]
        fn test_cortical_id_from_string_wrong_length() {
            let result = CorticalID::from_string("short".to_string());
            assert!(result.is_err());
            
            if let Err(FeagiDataError::BadParameters(msg)) = result {
                assert!(msg.contains("A cortical ID must have a length of 6"));
            } else {
                panic!("Expected BadParameters error");
            }
        }

        #[test]
        fn test_cortical_id_from_string_non_ascii() {
            let result = CorticalID::from_string("cust🎯1".to_string());
            assert!(result.is_err());
            
            if let Err(FeagiDataError::BadParameters(msg)) = result {
            } else {
                panic!("Expected BadParameters error");
            }
        }

        #[test]
        fn test_cortical_id_from_string_invalid_characters() {
            let result = CorticalID::from_string("cust@1".to_string());
            assert!(result.is_err());
            
            if let Err(FeagiDataError::BadParameters(msg)) = result {
                assert!(msg.contains("A cortical ID must be made only of alphanumeric characters and underscores"));
            } else {
                panic!("Expected BadParameters error");
            }
        }

        #[test]
        fn test_cortical_id_from_bytes_valid() {
            let bytes = b"custom";
            let result = CorticalID::from_bytes(bytes);
            assert!(result.is_ok());
            
            let cortical_id = result.unwrap();
            assert_eq!(cortical_id.as_ascii_string(), "custom");
        }

        #[test]
        fn test_cortical_id_from_bytes_non_ascii() {
            let bytes = [0xFF, 0xFE, 0xFD, 0xFC, 0xFB, 0xFA]; // Invalid UTF-8
            let result = CorticalID::from_bytes(&bytes);
            assert!(result.is_err());
            
            if let Err(FeagiDataError::DeserializationError(msg)) = result {
                assert!(msg.contains("Unable to parse cortical ID as ASCII"));
            } else {
                panic!("Expected DeserializationError");
            }
        }

        #[test]
        fn test_cortical_id_try_from_cortical_type_core() {
            let cortical_type = CorticalType::Core(CoreCorticalType::Death);
            let index = CorticalGroupIndex::from(0u8);
            
            let result = CorticalID::try_from_cortical_type(&cortical_type, index);
            assert!(result.is_ok());
            
            let cortical_id = result.unwrap();
            assert_eq!(cortical_id.as_ascii_string(), "_death");
        }

        #[test]
        fn test_cortical_id_try_from_cortical_type_custom_error() {
            let cortical_type = CorticalType::Custom;
            let index = CorticalGroupIndex::from(0u8);
            
            let result = CorticalID::try_from_cortical_type(&cortical_type, index);
            assert!(result.is_err());
        }
    }

    mod test_cortical_id_properties {
        use super::*;

        #[test]
        fn test_cortical_id_as_bytes() {
            let cortical_id = CorticalID::from_string("custom".to_string()).unwrap();
            let bytes = cortical_id.as_bytes();
            assert_eq!(bytes, b"custom");
        }

        #[test]
        fn test_cortical_id_as_ascii_string() {
            let cortical_id = CorticalID::from_string("memory".to_string()).unwrap();
            let string = cortical_id.as_ascii_string();
            assert_eq!(string, "memory");
        }

        #[test]
        fn test_cortical_id_get_cortical_type_custom() {
            let cortical_id = CorticalID::from_string("custom".to_string()).unwrap();
            let cortical_type = cortical_id.get_cortical_type();
            assert_eq!(cortical_type, CorticalType::Custom);
        }

        #[test]
        fn test_cortical_id_get_cortical_type_memory() {
            let cortical_id = CorticalID::from_string("memory".to_string()).unwrap();
            let cortical_type = cortical_id.get_cortical_type();
            assert_eq!(cortical_type, CorticalType::Memory);
        }

        #[test]
        fn test_cortical_id_get_cortical_type_core() {
            let cortical_id = CorticalID::new_core_cortical_area_id(CoreCorticalType::Death).unwrap();
            let cortical_type = cortical_id.get_cortical_type();
            
            match cortical_type {
                CorticalType::Core(CoreCorticalType::Death) => (),
                _ => panic!("Expected Core(Death) cortical type"),
            }
        }

        #[test]
        fn test_cortical_id_clone() {
            let cortical_id1 = CorticalID::from_string("clone1".to_string()).unwrap();
            let cortical_id2 = cortical_id1.clone();
            
            assert_eq!(cortical_id1, cortical_id2);
            assert_eq!(cortical_id1.as_ascii_string(), cortical_id2.as_ascii_string());
        }
    }
}

#[cfg(test)]
mod test_type_conversions {
    use super::*;

    #[test]
    fn test_core_cortical_type_to_cortical_type() {
        let core_type = CoreCorticalType::Death;
        let cortical_type: CorticalType = core_type.into();
        
        assert!(cortical_type.is_type_core());
        match cortical_type {
            CorticalType::Core(CoreCorticalType::Death) => (),
            _ => panic!("Expected Core(Death)"),
        }
    }

    #[test]
    fn test_core_cortical_type_reference_to_cortical_type() {
        let core_type = CoreCorticalType::Power;
        let cortical_type: CorticalType = (&core_type).into();
        
        assert!(cortical_type.is_type_core());
        match cortical_type {
            CorticalType::Core(CoreCorticalType::Power) => (),
            _ => panic!("Expected Core(Power)"),
        }
    }

    #[test]
    fn test_index_conversions() {
        let base_value = 42u8;
        let group_index = CorticalGroupIndex::from(base_value);
        let back_to_base: u8 = group_index.into();
        
        assert_eq!(base_value, back_to_base);
    }

    #[test]
    fn test_count_conversions() {
        let base_value = 100u32;
        let channel_count = CorticalChannelCount::from(base_value);
        let back_to_base: u32 = channel_count.into();
        
        assert_eq!(base_value, back_to_base);
    }
}

#[cfg(test)]
mod test_comprehensive_scenarios {
    use super::*;

    #[test]
    fn test_cortical_id_type_roundtrip() {
        // Test that we can create a cortical ID and get back the correct type
        let original_type = CorticalType::Core(CoreCorticalType::Death);
        let index = CorticalGroupIndex::from(0u8);
        
        let cortical_id = CorticalID::try_from_cortical_type(&original_type, index).unwrap();
        let recovered_type = cortical_id.get_cortical_type();
        
        assert_eq!(original_type, recovered_type);
    }

    #[test]
    fn test_maximum_group_index() {
        // Test that we can handle the maximum group index value
        let max_index = CorticalGroupIndex::from(u8::MAX);
        let core_type = CoreCorticalType::Power;
        
        // Core types should work with any index (though they don't use it)
        let cortical_type = CorticalType::Core(core_type);
        let cortical_id = CorticalID::try_from_cortical_type(&cortical_type, max_index);
        
        assert!(cortical_id.is_ok());
    }

    #[test]
    fn test_neuron_depth_with_coordinates() {
        // Test neuron depth in realistic scenarios
        let depth = NeuronDepth::new(100).unwrap();
        let coord = CorticalCoordinate::new(10, 20, *depth - 1);
        
        assert_eq!(*depth, 100);
        assert_eq!(coord.z, 99); // Within depth bounds
    }

    #[test]
    fn test_agent_device_channel_mapping() {
        // Test a realistic scenario of mapping agent devices to channels
        let agent_device = AgentDeviceIndex::from(1001u32);
        let cortical_group = CorticalGroupIndex::from(5u8);
        let cortical_channel = CorticalChannelIndex::from(0u32);
        
        assert_eq!(*agent_device, 1001);
        assert_eq!(*cortical_group, 5);
        assert_eq!(*cortical_channel, 0);
        
        // These would typically be used together in mapping tables
        assert_ne!(*agent_device, *cortical_channel); // Different index types
    }
}

#[cfg(test)]
mod test_error_handling_edge_cases {
    use super::*;

    #[test]
    fn test_zero_values_in_counts() {
        // All count types should reject zero
        assert!(CorticalChannelCount::new(0).is_err());
        assert!(NeuronDepth::new(0).is_err());
    }

    #[test]
    fn test_zero_values_in_dimensions() {
        // All dimension types should reject zero in any axis
        assert!(CorticalChannelDimensions::new(0, 10, 10).is_err());
        assert!(CorticalChannelDimensions::new(10, 0, 10).is_err());
        assert!(CorticalChannelDimensions::new(10, 10, 0).is_err());
        
        assert!(CorticalDimensions::new(0, 10, 10).is_err());
        assert!(CorticalDimensions::new(10, 0, 10).is_err());
        assert!(CorticalDimensions::new(10, 10, 0).is_err());
    }

    #[test]
    fn test_cortical_id_length_boundaries() {
        // Test exactly at the boundaries of the length requirement
        assert!(CorticalID::from_string("".to_string()).is_err());
        assert!(CorticalID::from_string("short".to_string()).is_err());
        assert!(CorticalID::from_string("exactl".to_string()).is_err()); // 6 chars but invalid type
        assert!(CorticalID::from_string("toolong".to_string()).is_err());
    }

    #[test]
    fn test_cortical_id_character_restrictions() {
        // Test various invalid character combinations
        let invalid_chars = vec![
            "cust!1", "cust@1", "cust#1", "cust$1", "cust%1",
            "cust^1", "cust&1", "cust*1", "cust(1", "cust)1",
            "cust-1", "cust+1", "cust=1", "cust[1", "cust]1",
            "cust{1", "cust}1", "cust|1", "cust\\1", "cust:1",
            "cust;1", "cust\"1", "cust'1", "cust<1", "cust>1",
            "cust,1", "cust.1", "cust?1", "cust/1", "cust 1",
        ];
        
        for invalid_id in invalid_chars {
            let result = CorticalID::from_string(invalid_id.to_string());
            assert!(result.is_err(), "Expected error for invalid ID: {}", invalid_id);
        }
    }

    #[test]
    fn test_cortical_id_valid_characters() {
        // Test that all valid characters work (alphanumeric + underscore)
        let valid_chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_";
        
        // Create valid IDs using these characters
        let result1 = CorticalID::from_string("custom".to_string());
        let result2 = CorticalID::from_string("CUSTOM".to_string()); // not valid
        let result3 = CorticalID::from_string("cust01".to_string());
        let result4 = CorticalID::from_string("c_st01".to_string());
        
        assert!(result1.is_ok());
        assert!(!result2.is_ok());
        assert!(result3.is_ok());
        assert!(result4.is_ok());
    }

    #[test]
    fn test_maximum_index_values() {
        // Test that maximum values work correctly
        let max_group = CorticalGroupIndex::from(u8::MAX);
        let max_channel = CorticalChannelIndex::from(u32::MAX);
        let max_agent = AgentDeviceIndex::from(u32::MAX);
        
        assert_eq!(*max_group, u8::MAX);
        assert_eq!(*max_channel, u32::MAX);
        assert_eq!(*max_agent, u32::MAX);
    }

    #[test]
    fn test_coordinate_extreme_values() {
        // Test coordinates at extreme values
        let max_unsigned = CorticalCoordinate::new(u32::MAX, u32::MAX, u32::MAX);
        let min_signed = GenomeCoordinate::new(i32::MIN, i32::MIN, i32::MIN);
        let max_signed = GenomeCoordinate::new(i32::MAX, i32::MAX, i32::MAX);
        
        assert_eq!(max_unsigned.x, u32::MAX);
        assert_eq!(min_signed.x, i32::MIN);
        assert_eq!(max_signed.x, i32::MAX);
    }
}

