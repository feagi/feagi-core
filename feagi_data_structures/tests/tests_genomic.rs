//! Tests for the genomic module
//!
//! This module contains comprehensive tests for genomic data structures
//! including cortical types, cortical IDs, descriptors, and sensory cortical units.

use feagi_data_structures::genomic::*;
use feagi_data_structures::genomic::cortical_area::*;
use feagi_data_structures::genomic::cortical_area::descriptors::*;
use feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::*;
use feagi_data_structures::FeagiDataError;

#[cfg(test)]
mod test_cortical_area_descriptors {
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
        fn test_cortical_unit_index_creation() {
            let index = CorticalUnitIndex::from(5u8);
            assert_eq!(*index, 5u8);
        }

        #[test]
        fn test_cortical_unit_index_max_value() {
            let index = CorticalUnitIndex::from(u8::MAX);
            assert_eq!(*index, u8::MAX);
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
                assert!(msg.contains("cannot be zero"));
            } else {
                panic!("Expected BadParameters error");
            }
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
                assert!(msg.contains("cannot be zero"));
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
        use feagi_data_structures::genomic::descriptors::GenomeCoordinate;
        use super::*;

        #[test]
        fn test_neuron_voxel_coordinate_creation() {
            let coord = NeuronVoxelCoordinate::new(10, 20, 30);
            assert_eq!(coord.x, 10);
            assert_eq!(coord.y, 20);
            assert_eq!(coord.z, 30);
        }

        #[test]
        fn test_neuron_voxel_coordinate_zero_values() {
            let coord = NeuronVoxelCoordinate::new(0, 0, 0);
            assert_eq!(coord.x, 0);
            assert_eq!(coord.y, 0);
            assert_eq!(coord.z, 0);
        }

        #[test]
        fn test_neuron_voxel_coordinate_max_values() {
            let coord = NeuronVoxelCoordinate::new(u32::MAX, u32::MAX, u32::MAX);
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
        fn test_cortical_area_dimensions_valid() {
            let dims = CorticalAreaDimensions::new(100, 200, 300).unwrap();
            assert_eq!(dims.width, 100);
            assert_eq!(dims.height, 200);
            assert_eq!(dims.depth, 300);
        }
    }
}

#[cfg(test)]
mod test_genomic_descriptors {
    use feagi_data_structures::genomic::descriptors::AgentDeviceIndex;
    use super::*;

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

    #[test]
    fn test_agent_device_index_max_value() {
        let index = AgentDeviceIndex::from(u32::MAX);
        assert_eq!(*index, u32::MAX);
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
            assert_eq!(format!("{}", cortical_id), "___death");
        }

        #[test]
        fn test_core_cortical_type_power() {
            let core_type = CoreCorticalType::Power;
            let cortical_id = core_type.to_cortical_id();
            assert_eq!(format!("{}", cortical_id), "___power");
        }

        #[test]
        fn test_core_cortical_type_display() {
            let death_type = CoreCorticalType::Death;
            let power_type = CoreCorticalType::Power;
            
            assert_eq!(format!("{}", death_type), "CoreCorticalType(Death)");
            assert_eq!(format!("{}", power_type), "CoreCorticalType(Power)");
        }
    }

    mod test_cortical_area_type {
        use super::*;

        #[test]
        fn test_cortical_area_type_core_variant() {
            let area_type = CorticalAreaType::Core(CoreCorticalType::Death);
            
            match area_type {
                CorticalAreaType::Core(CoreCorticalType::Death) => (),
                _ => panic!("Expected Core(Death) variant"),
            }
        }

        #[test]
        fn test_cortical_area_type_custom_variant() {
            let area_type = CorticalAreaType::Custom(CustomCorticalType::LeakyIntegrateFire);
            
            match area_type {
                CorticalAreaType::Custom(_) => (),
                _ => panic!("Expected Custom variant"),
            }
        }

        #[test]
        fn test_cortical_area_type_memory_variant() {
            let area_type = CorticalAreaType::Memory(MemoryCorticalType::Memory);
            
            match area_type {
                CorticalAreaType::Memory(_) => (),
                _ => panic!("Expected Memory variant"),
            }
        }

        #[test]
        fn test_cortical_area_type_brain_input_variant() {
            let io_type = IOCorticalAreaDataType::Percentage(
                FrameChangeHandling::Absolute,
                PercentageNeuronPositioning::Linear
            );
            let area_type = CorticalAreaType::BrainInput(io_type);
            
            match area_type {
                CorticalAreaType::BrainInput(_) => (),
                _ => panic!("Expected BrainInput variant"),
            }
        }

        #[test]
        fn test_cortical_area_type_brain_output_variant() {
            let io_type = IOCorticalAreaDataType::Percentage(
                FrameChangeHandling::Incremental,
                PercentageNeuronPositioning::Fractional
            );
            let area_type = CorticalAreaType::BrainOutput(io_type);
            
            match area_type {
                CorticalAreaType::BrainOutput(_) => (),
                _ => panic!("Expected BrainOutput variant"),
            }
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
            assert_eq!(CorticalID::CORTICAL_ID_LENGTH, 8);
            assert_eq!(CorticalID::NUMBER_OF_BYTES, 8);
            assert_eq!(CorticalID::CORTICAL_ID_LENGTH, CorticalID::NUMBER_OF_BYTES);
        }

        #[test]
        fn test_cortical_id_from_core_death() {
            let cortical_id = CoreCorticalType::Death.to_cortical_id();
            assert_eq!(format!("{}", cortical_id), "___death");
        }

        #[test]
        fn test_cortical_id_from_core_power() {
            let cortical_id = CoreCorticalType::Power.to_cortical_id();
            assert_eq!(format!("{}", cortical_id), "___power");
        }
    }

    mod test_cortical_id_properties {
        use super::*;

        #[test]
        fn test_cortical_id_as_bytes() {
            let cortical_id = CoreCorticalType::Death.to_cortical_id();
            let bytes = cortical_id.as_bytes();
            assert_eq!(bytes, b"___death");
        }

        #[test]
        fn test_cortical_id_display() {
            let cortical_id = CoreCorticalType::Power.to_cortical_id();
            let display_string = format!("{}", cortical_id);
            assert_eq!(display_string, "___power");
        }

        #[test]
        fn test_cortical_id_as_cortical_type_core_death() {
            let cortical_id = CoreCorticalType::Death.to_cortical_id();
            let cortical_type = cortical_id.as_cortical_type().unwrap();
            
            match cortical_type {
                CorticalAreaType::Core(CoreCorticalType::Death) => (),
                _ => panic!("Expected Core(Death) cortical type"),
            }
        }

        #[test]
        fn test_cortical_id_as_cortical_type_core_power() {
            let cortical_id = CoreCorticalType::Power.to_cortical_id();
            let cortical_type = cortical_id.as_cortical_type().unwrap();
            
            match cortical_type {
                CorticalAreaType::Core(CoreCorticalType::Power) => (),
                _ => panic!("Expected Core(Power) cortical type"),
            }
        }

        #[test]
        fn test_cortical_id_write_to_bytes() {
            let cortical_id = CoreCorticalType::Death.to_cortical_id();
            let mut buffer = [0u8; 8];
            cortical_id.write_id_to_bytes(&mut buffer);
            assert_eq!(&buffer, b"___death");
        }

        #[test]
        fn test_cortical_id_clone() {
            let cortical_id1 = CoreCorticalType::Power.to_cortical_id();
            let cortical_id2 = cortical_id1.clone();
            
            assert_eq!(cortical_id1, cortical_id2);
            assert_eq!(format!("{}", cortical_id1), format!("{}", cortical_id2));
        }

        #[test]
        fn test_cortical_id_copy() {
            let cortical_id1 = CoreCorticalType::Death.to_cortical_id();
            let cortical_id2 = cortical_id1; // Copy, not move
            
            assert_eq!(cortical_id1, cortical_id2);
        }
    }
}

#[cfg(test)]
mod test_io_cortical_area_data_type {
    use super::*;

    mod test_frame_change_handling {
        use super::*;

        #[test]
        fn test_frame_change_handling_variants() {
            let absolute = FrameChangeHandling::Absolute;
            let incremental = FrameChangeHandling::Incremental;
            
            assert_ne!(absolute, incremental);
        }

        #[test]
        fn test_frame_change_handling_default() {
            let default = FrameChangeHandling::default();
            assert_eq!(default, FrameChangeHandling::Absolute);
        }
    }

    mod test_percentage_neuron_positioning {
        use super::*;

        #[test]
        fn test_percentage_neuron_positioning_variants() {
            let linear = PercentageNeuronPositioning::Linear;
            let fractional = PercentageNeuronPositioning::Fractional;
            
            assert_ne!(linear, fractional);
        }

        #[test]
        fn test_percentage_neuron_positioning_default() {
            let default = PercentageNeuronPositioning::default();
            assert_eq!(default, PercentageNeuronPositioning::Fractional);
        }
    }

    mod test_io_cortical_area_data_type_variants {
        use super::*;

        #[test]
        fn test_percentage_variant() {
            let io_type = IOCorticalAreaDataType::Percentage(
                FrameChangeHandling::Absolute,
                PercentageNeuronPositioning::Linear
            );
            
            match io_type {
                IOCorticalAreaDataType::Percentage(_, _) => (),
                _ => panic!("Expected Percentage variant"),
            }
        }

        #[test]
        fn test_signed_percentage_variant() {
            let io_type = IOCorticalAreaDataType::SignedPercentage(
                FrameChangeHandling::Incremental,
                PercentageNeuronPositioning::Fractional
            );
            
            match io_type {
                IOCorticalAreaDataType::SignedPercentage(_, _) => (),
                _ => panic!("Expected SignedPercentage variant"),
            }
        }

        #[test]
        fn test_cartesian_plane_variant() {
            let io_type = IOCorticalAreaDataType::CartesianPlane(FrameChangeHandling::Absolute);
            
            match io_type {
                IOCorticalAreaDataType::CartesianPlane(_) => (),
                _ => panic!("Expected CartesianPlane variant"),
            }
        }

        #[test]
        fn test_percentage_3d_variant() {
            let io_type = IOCorticalAreaDataType::SignedPercentage3D(
                FrameChangeHandling::Absolute,
                PercentageNeuronPositioning::Linear
            );
            
            match io_type {
                IOCorticalAreaDataType::SignedPercentage3D(_, _) => (),
                _ => panic!("Expected SignedPercentage3D variant"),
            }
        }

        #[test]
        fn test_percentage_4d_variant() {
            let io_type = IOCorticalAreaDataType::SignedPercentage4D(
                FrameChangeHandling::Incremental,
                PercentageNeuronPositioning::Fractional
            );
            
            match io_type {
                IOCorticalAreaDataType::SignedPercentage4D(_, _) => (),
                _ => panic!("Expected SignedPercentage4D variant"),
            }
        }
    }

    mod test_data_type_configuration_flag {
        use super::*;

        #[test]
        fn test_to_configuration_flag_percentage() {
            let io_type = IOCorticalAreaDataType::Percentage(
                FrameChangeHandling::Incremental,
                PercentageNeuronPositioning::Linear
            );
            
            let flag = io_type.to_data_type_configuration_flag();
            
            // Verify the flag is non-zero
            assert_ne!(flag, 0);
        }

        #[test]
        fn test_configuration_flag_roundtrip_percentage() {
            let original = IOCorticalAreaDataType::Percentage(
                FrameChangeHandling::Absolute,
                PercentageNeuronPositioning::Linear
            );
            
            let flag = original.to_data_type_configuration_flag();
            let recovered = IOCorticalAreaDataType::try_from_data_type_configuration_flag(flag).unwrap();
            
            assert_eq!(original, recovered);
        }

        #[test]
        fn test_configuration_flag_roundtrip_cartesian_plane() {
            let original = IOCorticalAreaDataType::CartesianPlane(FrameChangeHandling::Incremental);
            
            let flag = original.to_data_type_configuration_flag();
            let recovered = IOCorticalAreaDataType::try_from_data_type_configuration_flag(flag).unwrap();
            
            assert_eq!(original, recovered);
        }

        #[test]
        fn test_different_types_produce_different_flags() {
            let type1 = IOCorticalAreaDataType::Percentage(
                FrameChangeHandling::Absolute,
                PercentageNeuronPositioning::Linear
            );
            
            let type2 = IOCorticalAreaDataType::SignedPercentage(
                FrameChangeHandling::Absolute,
                PercentageNeuronPositioning::Linear
            );
            
            let flag1 = type1.to_data_type_configuration_flag();
            let flag2 = type2.to_data_type_configuration_flag();
            
            assert_ne!(flag1, flag2);
        }
    }

    mod test_as_io_cortical_id {
        use super::*;

        #[test]
        fn test_as_io_cortical_id_creates_valid_id() {
            let io_type = IOCorticalAreaDataType::Percentage(
                FrameChangeHandling::Absolute,
                PercentageNeuronPositioning::Linear
            );
            
            let unit_id = *b"tst";
            let unit_index = CorticalUnitIndex::from(0u8);
            let group_index = CorticalGroupIndex::from(5u8);
            
            let cortical_id = io_type.as_io_cortical_id(true, unit_id, unit_index, group_index);
            
            // Verify the ID starts with 'i' for input
            let bytes = cortical_id.as_bytes();
            assert_eq!(bytes[0], b'i');
        }

        #[test]
        fn test_as_io_cortical_id_input_vs_output() {
            let io_type = IOCorticalAreaDataType::Percentage(
                FrameChangeHandling::Absolute,
                PercentageNeuronPositioning::Linear
            );
            
            let unit_id = *b"tst";
            let unit_index = CorticalUnitIndex::from(0u8);
            let group_index = CorticalGroupIndex::from(0u8);
            
            let input_id = io_type.as_io_cortical_id(true, unit_id, unit_index, group_index);
            let output_id = io_type.as_io_cortical_id(false, unit_id, unit_index, group_index);
            
            assert_ne!(input_id, output_id);
            
            let input_bytes = input_id.as_bytes();
            let output_bytes = output_id.as_bytes();
            
            assert_eq!(input_bytes[0], b'i');
            assert_eq!(output_bytes[0], b'o');
        }
    }
}

#[cfg(test)]
mod test_sensory_cortical_unit {
    use super::*;

    #[test]
    fn test_sensory_cortical_unit_display() {
        // Test the Display implementation for sensor types
        assert_eq!(format!("{}", SensoryCorticalUnit::Infrared), "Infrared Sensor");
        assert_eq!(format!("{}", SensoryCorticalUnit::SegmentedVision), "Segmented Vision");
    }

    #[test]
    fn test_sensory_cortical_unit_snake_case_name() {
        // Test the snake_case_name method
        assert_eq!(SensoryCorticalUnit::Infrared.get_snake_case_name(), "infrared");
        assert_eq!(SensoryCorticalUnit::SegmentedVision.get_snake_case_name(), "segmented_vision");
    }

    #[test]
    fn test_infrared_cortical_area_types_array() {
        // Test that infrared generates 1 cortical area type
        let frame_handling = FrameChangeHandling::Absolute;
        let positioning = PercentageNeuronPositioning::Linear;
        
        let types = SensoryCorticalUnit::get_infrared_cortical_area_types_array(frame_handling, positioning);
        
        assert_eq!(types.len(), 1);
        match types[0] {
            CorticalAreaType::BrainInput(_) => (),
            _ => panic!("Expected BrainInput type"),
        }
    }

    #[test]
    fn test_infrared_cortical_ids_array() {
        // Test that infrared generates correct cortical IDs
        let frame_handling = FrameChangeHandling::Absolute;
        let positioning = PercentageNeuronPositioning::Linear;
        let group = CorticalGroupIndex::from(5u8);
        
        let ids = SensoryCorticalUnit::get_infrared_cortical_ids_array(frame_handling, positioning, group);
        
        assert_eq!(ids.len(), 1);
        
        // Verify the cortical ID has the correct structure
        let bytes = ids[0].as_bytes();
        assert_eq!(bytes[0], b'i', "Sensor ID should start with 'i'");
        assert_eq!(&bytes[1..4], b"inf", "Infrared ID should contain 'inf'");
    }

    #[test]
    fn test_segmented_vision_cortical_area_types_array() {
        // Test that segmented vision generates 9 cortical area types
        let frame_handling = FrameChangeHandling::Incremental;
        
        let types = SensoryCorticalUnit::get_segmented_vision_cortical_area_types_array(frame_handling);
        
        assert_eq!(types.len(), 9);
        
        // All should be BrainInput types
        for area_type in types.iter() {
            match area_type {
                CorticalAreaType::BrainInput(_) => (),
                _ => panic!("Expected BrainInput type"),
            }
        }
    }

    #[test]
    fn test_segmented_vision_cortical_ids_array() {
        // Test that segmented vision generates 9 distinct cortical IDs
        let frame_handling = FrameChangeHandling::Incremental;
        let group = CorticalGroupIndex::from(3u8);
        
        let ids = SensoryCorticalUnit::get_segmented_vision_cortical_ids_array(frame_handling, group);
        
        assert_eq!(ids.len(), 9);
        
        // Verify all IDs are unique and properly formatted
        for (i, id) in ids.iter().enumerate() {
            let bytes = id.as_bytes();
            assert_eq!(bytes[0], b'i', "Sensor ID should start with 'i'");
            assert_eq!(&bytes[1..4], b"svi", "Segmented vision ID should contain 'svi'");
            
            // Check that IDs are unique
            for (j, other_id) in ids.iter().enumerate() {
                if i != j {
                    assert_ne!(id, other_id, "IDs at index {} and {} should be different", i, j);
                }
            }
        }
    }

    #[test]
    fn test_different_frame_handling_produces_different_ids() {
        // Test that different frame handling produces different IDs
        let positioning = PercentageNeuronPositioning::Linear;
        let group = CorticalGroupIndex::from(0u8);
        
        let absolute_ids = SensoryCorticalUnit::get_infrared_cortical_ids_array(
            FrameChangeHandling::Absolute,
            positioning,
            group
        );
        
        let incremental_ids = SensoryCorticalUnit::get_infrared_cortical_ids_array(
            FrameChangeHandling::Incremental,
            positioning,
            group
        );
        
        assert_ne!(absolute_ids[0], incremental_ids[0], "Different frame handling should produce different IDs");
    }

    #[test]
    fn test_different_positioning_produces_different_ids() {
        // Test that different positioning produces different IDs
        let frame_handling = FrameChangeHandling::Absolute;
        let group = CorticalGroupIndex::from(0u8);
        
        let linear_ids = SensoryCorticalUnit::get_infrared_cortical_ids_array(
            frame_handling,
            PercentageNeuronPositioning::Linear,
            group
        );
        
        let fractional_ids = SensoryCorticalUnit::get_infrared_cortical_ids_array(
            frame_handling,
            PercentageNeuronPositioning::Fractional,
            group
        );
        
        assert_ne!(linear_ids[0], fractional_ids[0], "Different positioning should produce different IDs");
    }

    #[test]
    fn test_different_groups_produce_different_ids() {
        // Test that different groups produce different IDs
        let frame_handling = FrameChangeHandling::Absolute;
        let positioning = PercentageNeuronPositioning::Linear;
        
        let group0_ids = SensoryCorticalUnit::get_infrared_cortical_ids_array(
            frame_handling,
            positioning,
            CorticalGroupIndex::from(0u8)
        );
        
        let group1_ids = SensoryCorticalUnit::get_infrared_cortical_ids_array(
            frame_handling,
            positioning,
            CorticalGroupIndex::from(1u8)
        );
        
        assert_ne!(group0_ids[0], group1_ids[0], "Different groups should produce different IDs");
    }
}

#[cfg(test)]
mod test_comprehensive_scenarios {
    use feagi_data_structures::genomic::descriptors::{AgentDeviceIndex, GenomeCoordinate};
    use super::*;

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
        let max_unsigned = NeuronVoxelCoordinate::new(u32::MAX, u32::MAX, u32::MAX);
        let min_signed = GenomeCoordinate::new(i32::MIN, i32::MIN, i32::MIN);
        let max_signed = GenomeCoordinate::new(i32::MAX, i32::MAX, i32::MAX);
        
        assert_eq!(max_unsigned.x, u32::MAX);
        assert_eq!(min_signed.x, i32::MIN);
        assert_eq!(max_signed.x, i32::MAX);
    }

    #[test]
    fn test_neuron_depth_with_coordinates() {
        // Test neuron depth in realistic scenarios
        let depth = NeuronDepth::new(100).unwrap();
        let coord = NeuronVoxelCoordinate::new(10, 20, *depth - 1);
        
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
    }

    #[test]
    fn test_sensory_unit_with_multiple_parameters() {
        // Test creating IDs with various parameter combinations
        let params = [
            (FrameChangeHandling::Absolute, PercentageNeuronPositioning::Linear),
            (FrameChangeHandling::Absolute, PercentageNeuronPositioning::Fractional),
            (FrameChangeHandling::Incremental, PercentageNeuronPositioning::Linear),
            (FrameChangeHandling::Incremental, PercentageNeuronPositioning::Fractional),
        ];
        
        let group = CorticalGroupIndex::from(0u8);
        let mut all_ids = Vec::new();
        
        for (frame, pos) in params.iter() {
            let ids = SensoryCorticalUnit::get_infrared_cortical_ids_array(*frame, *pos, group);
            all_ids.push(ids[0]);
        }
        
        // All IDs should be unique
        for (i, id1) in all_ids.iter().enumerate() {
            for (j, id2) in all_ids.iter().enumerate() {
                if i != j {
                    assert_ne!(id1, id2, "IDs at positions {} and {} should be different", i, j);
                }
            }
        }
    }
}

#[cfg(test)]
mod test_error_handling {
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
        
        assert!(CorticalAreaDimensions::new(0, 10, 10).is_err());
        assert!(CorticalAreaDimensions::new(10, 0, 10).is_err());
        assert!(CorticalAreaDimensions::new(10, 10, 0).is_err());
    }
}
