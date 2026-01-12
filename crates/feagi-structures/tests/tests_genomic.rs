//! Tests for the genomic module
//!
//! This module contains comprehensive tests for genomic data structures
//! including cortical types, cortical IDs, descriptors, and sensory cortical units.

use feagi_structures::genomic::cortical_area::descriptors::*;
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::*;
use feagi_structures::genomic::cortical_area::*;
use feagi_structures::genomic::*;
use feagi_structures::FeagiDataError;

// TODO tests for Brain_Regions

/// Tests for genomic/cortical_area/ subdirectory
#[cfg(test)]
mod test_cortical_area {
    use super::*;

    /// Tests for cortical_area/cortical_area.rs
    mod cortical_area_tests {
        use super::*;
        use feagi_structures::genomic::cortical_area::CorticalArea;
        use feagi_structures::genomic::descriptors::GenomeCoordinate3D;

        #[test]
        fn test_cortical_area_creation() {
            let dims = CorticalAreaDimensions::new(128, 128, 20).unwrap();
            let cortical_id = CoreCorticalType::Power.to_cortical_id();
            let position = GenomeCoordinate3D::new(0, 0, 0);

            let cortical_type = CorticalAreaType::Core(CoreCorticalType::Power);

            let area = CorticalArea::new(
                cortical_id,
                0,
                "Power Area".to_string(),
                dims,
                position,
                cortical_type,
            )
            .unwrap();

            assert_eq!(area.cortical_id, cortical_id);
            assert_eq!(area.name, "Power Area");
            assert_eq!(area.total_voxels(), 128 * 128 * 20);
            assert_eq!(area.position, position);
        }

        #[test]
        fn test_properties() {
            let dims = CorticalAreaDimensions::new(10, 10, 10).unwrap();
            let cortical_id = CoreCorticalType::Death.to_cortical_id();
            let position = GenomeCoordinate3D::new(0, 0, 0);

            let cortical_type = CorticalAreaType::Core(CoreCorticalType::Death);

            let mut area = CorticalArea::new(
                cortical_id,
                0,
                "Test".to_string(),
                dims,
                position,
                cortical_type,
            )
            .unwrap();

            area.properties
                .insert("neurons_per_voxel".to_string(), serde_json::json!(1));
            area.properties.insert(
                "description".to_string(),
                serde_json::json!("Test area for death signal"),
            );

            assert_eq!(
                area.get_property("neurons_per_voxel"),
                Some(&serde_json::json!(1))
            );
            assert_eq!(
                area.get_property("description"),
                Some(&serde_json::json!("Test area for death signal"))
            );
        }

        #[test]
        fn test_empty_name_rejected() {
            let dims = CorticalAreaDimensions::new(10, 10, 10).unwrap();
            let cortical_id = CoreCorticalType::Power.to_cortical_id();
            let position = GenomeCoordinate3D::new(0, 0, 0);

            let cortical_type = CorticalAreaType::Custom(CustomCorticalType::LeakyIntegrateFire);

            let result = CorticalArea::new(
                cortical_id,
                0,
                "".to_string(),
                dims,
                position,
                cortical_type,
            );

            assert!(result.is_err());
        }

        #[test]
        fn test_whitespace_only_name_rejected() {
            let dims = CorticalAreaDimensions::new(10, 10, 10).unwrap();
            let cortical_id = CoreCorticalType::Power.to_cortical_id();
            let position = GenomeCoordinate3D::new(0, 0, 0);

            let cortical_type = CorticalAreaType::Core(CoreCorticalType::Power);

            let result = CorticalArea::new(
                cortical_id,
                0,
                "   ".to_string(),
                dims,
                position,
                cortical_type,
            );

            assert!(result.is_err());
        }

        #[test]
        fn test_get_property_returns_none_for_missing() {
            let dims = CorticalAreaDimensions::new(10, 10, 10).unwrap();
            let cortical_id = CoreCorticalType::Death.to_cortical_id();
            let position = GenomeCoordinate3D::new(0, 0, 0);
            let cortical_type = CorticalAreaType::Core(CoreCorticalType::Death);

            let area = CorticalArea::new(
                cortical_id,
                0,
                "Test".to_string(),
                dims,
                position,
                cortical_type,
            )
            .unwrap();

            assert!(area.get_property("nonexistent_key").is_none());
        }

        #[test]
        fn test_cortical_area_dimensions_accessor() {
            let dims = CorticalAreaDimensions::new(128, 256, 64).unwrap();
            let cortical_id = CoreCorticalType::Power.to_cortical_id();
            let position = GenomeCoordinate3D::new(10, 20, 30);
            let cortical_type = CorticalAreaType::Core(CoreCorticalType::Power);

            let area = CorticalArea::new(
                cortical_id,
                0,
                "Test Area".to_string(),
                dims,
                position,
                cortical_type,
            )
            .unwrap();

            assert_eq!(area.dimensions.width, 128);
            assert_eq!(area.dimensions.height, 256);
            assert_eq!(area.dimensions.depth, 64);
        }

        #[test]
        fn test_cortical_area_cortical_idx() {
            let dims = CorticalAreaDimensions::new(10, 10, 10).unwrap();
            let cortical_id = CoreCorticalType::Power.to_cortical_id();
            let position = GenomeCoordinate3D::new(0, 0, 0);
            let cortical_type = CorticalAreaType::Core(CoreCorticalType::Power);

            let area = CorticalArea::new(
                cortical_id,
                42,
                "Test".to_string(),
                dims,
                position,
                cortical_type,
            )
            .unwrap();

            assert_eq!(area.cortical_idx, 42);
        }

        #[test]
        fn test_cortical_area_type_variants() {
            let dims = CorticalAreaDimensions::new(10, 10, 10).unwrap();
            let position = GenomeCoordinate3D::new(0, 0, 0);

            // Test Core type
            let core_id = CoreCorticalType::Power.to_cortical_id();
            let core_type = CorticalAreaType::Core(CoreCorticalType::Power);
            let core_area = CorticalArea::new(
                core_id,
                0,
                "Core Area".to_string(),
                dims,
                position,
                core_type,
            )
            .unwrap();
            assert_eq!(core_area.cortical_type, core_type);

            // Test Custom type
            let custom_id = CoreCorticalType::Death.to_cortical_id();
            let custom_type = CorticalAreaType::Custom(CustomCorticalType::LeakyIntegrateFire);
            let custom_area = CorticalArea::new(
                custom_id,
                1,
                "Custom Area".to_string(),
                dims,
                position,
                custom_type,
            )
            .unwrap();
            assert_eq!(custom_area.cortical_type, custom_type);

            // Test Memory type
            let memory_id = CoreCorticalType::Power.to_cortical_id();
            let memory_type = CorticalAreaType::Memory(MemoryCorticalType::Memory);
            let memory_area = CorticalArea::new(
                memory_id,
                2,
                "Memory Area".to_string(),
                dims,
                position,
                memory_type,
            )
            .unwrap();
            assert_eq!(memory_area.cortical_type, memory_type);
        }

        #[test]
        fn test_cortical_area_serialization() {
            let dims = CorticalAreaDimensions::new(64, 64, 10).unwrap();
            let cortical_id = CoreCorticalType::Power.to_cortical_id();
            let position = GenomeCoordinate3D::new(10, 20, 30);

            let cortical_type = CorticalAreaType::Memory(MemoryCorticalType::Memory);

            let mut area = CorticalArea::new(
                cortical_id,
                5,
                "Test Area".to_string(),
                dims,
                position,
                cortical_type,
            )
            .unwrap();

            area.properties
                .insert("neurons_per_voxel".to_string(), serde_json::json!(4));
            area.properties.insert(
                "description".to_string(),
                serde_json::json!("Memory storage area"),
            );

            // Serialize to JSON
            let json = serde_json::to_string(&area).unwrap();

            // Deserialize back
            let deserialized: CorticalArea = serde_json::from_str(&json).unwrap();

            assert_eq!(deserialized.cortical_id, area.cortical_id);
            assert_eq!(deserialized.name, "Test Area");
            assert_eq!(deserialized.cortical_idx, 5);
            assert_eq!(deserialized.cortical_type, cortical_type);
            assert_eq!(deserialized.position, position);
            assert_eq!(
                deserialized.get_property("neurons_per_voxel"),
                Some(&serde_json::json!(4))
            );
            assert_eq!(
                deserialized.get_property("description"),
                Some(&serde_json::json!("Memory storage area"))
            );
        }
    }

    /// Tests for cortical_area/cortical_area_type.rs
    mod test_cortical_area_type {
        use super::*;

        mod test_core_cortical_type {
            use super::*;

            #[test]
            fn test_core_cortical_type_death() {
                let core_type = CoreCorticalType::Death;
                let cortical_id = core_type.to_cortical_id();
                assert_eq!(format!("{}", cortical_id), cortical_id.as_base_64());
            }

            #[test]
            fn test_core_cortical_type_power() {
                let core_type = CoreCorticalType::Power;
                let cortical_id = core_type.to_cortical_id();
                assert_eq!(format!("{}", cortical_id), cortical_id.as_base_64());
            }

            #[test]
            fn test_core_cortical_type_display() {
                let death_type = CoreCorticalType::Death;
                let power_type = CoreCorticalType::Power;
                let fatigue_type = CoreCorticalType::Fatigue;

                assert_eq!(format!("{}", death_type), "CoreCorticalType(Death)");
                assert_eq!(format!("{}", power_type), "CoreCorticalType(Power)");
                assert_eq!(format!("{}", fatigue_type), "CoreCorticalType(Fatigue)");
            }

            #[test]
            fn test_core_cortical_type_fatigue() {
                let core_type = CoreCorticalType::Fatigue;
                let cortical_id = core_type.to_cortical_id();
                assert_eq!(cortical_id.as_bytes(), b"___fatig");
            }

            #[test]
            fn test_core_cortical_type_roundtrip() {
                for core_type in [
                    CoreCorticalType::Death,
                    CoreCorticalType::Power,
                    CoreCorticalType::Fatigue,
                ] {
                    let cortical_id = core_type.to_cortical_id();
                    let recovered = cortical_id.as_cortical_type().unwrap();
                    match recovered {
                        CorticalAreaType::Core(recovered_core) => {
                            assert_eq!(core_type, recovered_core);
                        }
                        _ => panic!("Expected Core variant"),
                    }
                }
            }
        }

        mod test_cortical_area_type_display {
            use super::*;

            #[test]
            fn test_cortical_area_type_display_core() {
                let area_type = CorticalAreaType::Core(CoreCorticalType::Power);
                assert!(format!("{}", area_type).contains("Core"));
            }

            #[test]
            fn test_cortical_area_type_display_custom() {
                let area_type = CorticalAreaType::Custom(CustomCorticalType::LeakyIntegrateFire);
                assert!(format!("{}", area_type).contains("Custom"));
            }

            #[test]
            fn test_cortical_area_type_display_memory() {
                let area_type = CorticalAreaType::Memory(MemoryCorticalType::Memory);
                assert!(format!("{}", area_type).contains("Memory"));
            }

            #[test]
            fn test_cortical_area_type_display_brain_input() {
                let io_type = IOCorticalAreaConfigurationFlag::Percentage(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Linear,
                );
                let area_type = CorticalAreaType::BrainInput(io_type);
                assert!(format!("{}", area_type).contains("BrainInput"));
            }

            #[test]
            fn test_cortical_area_type_display_brain_output() {
                let io_type = IOCorticalAreaConfigurationFlag::Boolean;
                let area_type = CorticalAreaType::BrainOutput(io_type);
                assert!(format!("{}", area_type).contains("BrainOutput"));
            }

            #[test]
            fn test_custom_cortical_type_display() {
                let custom_type = CustomCorticalType::LeakyIntegrateFire;
                assert_eq!(format!("{}", custom_type), "Leaky IntegrateFire");
            }

            #[test]
            fn test_memory_cortical_type_display() {
                let memory_type = MemoryCorticalType::Memory;
                assert_eq!(format!("{}", memory_type), "Memory");
            }
        }

        mod test_cortical_area_type_variants {
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
                let io_type = IOCorticalAreaConfigurationFlag::Percentage(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Linear,
                );
                let area_type = CorticalAreaType::BrainInput(io_type);

                match area_type {
                    CorticalAreaType::BrainInput(_) => (),
                    _ => panic!("Expected BrainInput variant"),
                }
            }

            #[test]
            fn test_cortical_area_type_brain_output_variant() {
                let io_type = IOCorticalAreaConfigurationFlag::Percentage(
                    FrameChangeHandling::Incremental,
                    PercentageNeuronPositioning::Fractional,
                );
                let area_type = CorticalAreaType::BrainOutput(io_type);

                match area_type {
                    CorticalAreaType::BrainOutput(_) => (),
                    _ => panic!("Expected BrainOutput variant"),
                }
            }
        }
    }

    /// Tests for cortical_area/cortical_id.rs
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
                assert_eq!(format!("{}", cortical_id), cortical_id.as_base_64());
            }

            #[test]
            fn test_cortical_id_from_core_power() {
                let cortical_id = CoreCorticalType::Power.to_cortical_id();
                assert_eq!(format!("{}", cortical_id), cortical_id.as_base_64());
            }

            #[test]
            fn test_invalid_cortical_id() {
                let cortical_id_result = CorticalID::try_from_base_64("short");
                assert!(cortical_id_result.is_err());
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
                assert_eq!(display_string, cortical_id.as_base_64());
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
                let cortical_id2 = cortical_id1;

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

        mod test_cortical_id_conversions {
            use super::*;

            #[test]
            fn test_cortical_id_try_from_bytes_core() {
                let bytes: [u8; 8] = *b"___death";
                let cortical_id = CorticalID::try_from_bytes(&bytes).unwrap();
                assert_eq!(cortical_id.as_bytes(), &bytes);
            }

            #[test]
            fn test_cortical_id_try_from_bytes_invalid() {
                let bytes: [u8; 8] = *b"xxxxxxxx";
                let result = CorticalID::try_from_bytes(&bytes);
                assert!(result.is_err());
            }

            #[test]
            fn test_cortical_id_try_from_u64_roundtrip() {
                let original = CoreCorticalType::Power.to_cortical_id();
                let as_u64 = original.as_u64();
                let restored = CorticalID::try_from_u64(as_u64).unwrap();
                assert_eq!(original, restored);
            }

            #[test]
            fn test_cortical_id_try_from_base64_roundtrip() {
                let original = CoreCorticalType::Death.to_cortical_id();
                let as_base64 = original.as_base_64();
                let restored = CorticalID::try_from_base_64(&as_base64).unwrap();
                assert_eq!(original, restored);
            }

            #[test]
            fn test_cortical_id_try_from_base64_invalid() {
                let result = CorticalID::try_from_base_64("not valid base64!!!");
                assert!(result.is_err());
            }

            #[test]
            fn test_cortical_id_try_from_base64_wrong_length() {
                // Valid base64 but only 4 bytes when decoded
                let short = base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    [1u8, 2, 3, 4],
                );
                let result = CorticalID::try_from_base_64(&short);
                assert!(result.is_err());
            }

            #[test]
            fn test_cortical_id_serialization_roundtrip() {
                let original = CoreCorticalType::Power.to_cortical_id();
                let serialized = serde_json::to_string(&original).unwrap();
                let deserialized: CorticalID = serde_json::from_str(&serialized).unwrap();
                assert_eq!(original, deserialized);
            }
        }

        mod test_cortical_id_extraction {
            use super::*;

            #[test]
            fn test_extract_subtype_for_input() {
                // Create an IPU cortical ID
                let io_type = IOCorticalAreaConfigurationFlag::Percentage(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Linear,
                );
                let cortical_id = io_type.as_io_cortical_id(
                    true,
                    *b"tst",
                    CorticalUnitIndex::from(0u8),
                    CorticalSubUnitIndex::from(0u8),
                );

                let subtype = cortical_id.extract_subtype();
                assert_eq!(subtype, Some("tst".to_string()));
            }

            #[test]
            fn test_extract_subtype_for_output() {
                let io_type = IOCorticalAreaConfigurationFlag::Boolean;
                let cortical_id = io_type.as_io_cortical_id(
                    false,
                    *b"mot",
                    CorticalUnitIndex::from(0u8),
                    CorticalSubUnitIndex::from(0u8),
                );

                let subtype = cortical_id.extract_subtype();
                assert_eq!(subtype, Some("mot".to_string()));
            }

            #[test]
            fn test_extract_subtype_for_core_returns_none() {
                let cortical_id = CoreCorticalType::Death.to_cortical_id();
                let subtype = cortical_id.extract_subtype();
                assert!(subtype.is_none());
            }

            #[test]
            fn test_extract_unit_id_for_core_returns_none() {
                let cortical_id = CoreCorticalType::Power.to_cortical_id();
                let unit_id = cortical_id.extract_unit_id();
                assert!(unit_id.is_none());
            }

            #[test]
            fn test_extract_group_id_for_core_returns_none() {
                let cortical_id = CoreCorticalType::Fatigue.to_cortical_id();
                let group_id = cortical_id.extract_group_id();
                assert!(group_id.is_none());
            }
        }
    }

    /// Tests for cortical_area/descriptors.rs
    mod test_descriptors {
        use super::*;

        mod test_indices {
            use super::*;

            #[test]
            fn test_cortical_group_index_creation() {
                let index = CorticalUnitIndex::from(42u8);
                assert_eq!(*index, 42u8);
            }

            #[test]
            fn test_cortical_group_index_deref() {
                let index = CorticalUnitIndex::from(100u8);
                assert_eq!(*index, 100u8);
            }

            #[test]
            fn test_cortical_group_index_max_value() {
                let index = CorticalUnitIndex::from(u8::MAX);
                assert_eq!(*index, u8::MAX);
            }

            #[test]
            fn test_cortical_group_index_zero() {
                let index = CorticalUnitIndex::from(0u8);
                assert_eq!(*index, 0u8);
            }

            #[test]
            fn test_cortical_group_index_equality() {
                let index1 = CorticalUnitIndex::from(50u8);
                let index2 = CorticalUnitIndex::from(50u8);
                let index3 = CorticalUnitIndex::from(51u8);

                assert_eq!(index1, index2);
                assert_ne!(index1, index3);
            }

            #[test]
            fn test_cortical_group_index_ordering() {
                let index1 = CorticalUnitIndex::from(10u8);
                let index2 = CorticalUnitIndex::from(20u8);

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
                let index = CorticalSubUnitIndex::from(5u8);
                assert_eq!(*index, 5u8);
            }

            #[test]
            fn test_cortical_unit_index_max_value() {
                let index = CorticalSubUnitIndex::from(u8::MAX);
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

            #[test]
            fn test_zero_values_in_dimensions() {
                assert!(CorticalChannelDimensions::new(0, 10, 10).is_err());
                assert!(CorticalChannelDimensions::new(10, 0, 10).is_err());
                assert!(CorticalChannelDimensions::new(10, 10, 0).is_err());

                assert!(CorticalAreaDimensions::new(0, 10, 10).is_err());
                assert!(CorticalAreaDimensions::new(10, 0, 10).is_err());
                assert!(CorticalAreaDimensions::new(10, 10, 0).is_err());
            }

            #[test]
            fn test_cortical_area_dimensions_total_voxels() {
                let dims = CorticalAreaDimensions::new(10, 20, 30).unwrap();
                assert_eq!(dims.total_voxels(), 10 * 20 * 30);
            }

            #[test]
            fn test_cortical_area_dimensions_total_voxels_single() {
                let dims = CorticalAreaDimensions::new(1, 1, 1).unwrap();
                assert_eq!(dims.total_voxels(), 1);
            }

            #[test]
            fn test_cortical_channel_dimensions_total_voxels() {
                let dims = CorticalChannelDimensions::new(5, 10, 15).unwrap();
                assert_eq!(dims.total_voxels(), 5 * 10 * 15);
            }
        }

        mod test_index_get_methods {
            use super::*;

            #[test]
            fn test_cortical_unit_index_get() {
                let index = CorticalUnitIndex::from(42u8);
                assert_eq!(index.get(), 42u8);
            }

            #[test]
            fn test_cortical_sub_unit_index_get() {
                let index = CorticalSubUnitIndex::from(17u8);
                assert_eq!(index.get(), 17u8);
            }

            #[test]
            fn test_cortical_channel_index_get() {
                let index = CorticalChannelIndex::from(12345u32);
                assert_eq!(index.get(), 12345u32);
            }
        }
    }

    /// Tests for cortical_area/io_cortical_area_configuration_flag.rs
    mod test_io_cortical_area_configuration_flag {
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

            #[test]
            fn test_frame_change_handling_display() {
                assert_eq!(format!("{}", FrameChangeHandling::Absolute), "Absolute");
                assert_eq!(
                    format!("{}", FrameChangeHandling::Incremental),
                    "Incremental"
                );
            }

            #[test]
            fn test_frame_change_handling_try_from_serde_map() {
                let mut map = serde_json::Map::new();
                map.insert(
                    "frame_change_handling".to_string(),
                    serde_json::json!("Absolute"),
                );
                let result = FrameChangeHandling::try_from_serde_map(&map).unwrap();
                assert_eq!(result, FrameChangeHandling::Absolute);

                map.insert(
                    "frame_change_handling".to_string(),
                    serde_json::json!("Incremental"),
                );
                let result = FrameChangeHandling::try_from_serde_map(&map).unwrap();
                assert_eq!(result, FrameChangeHandling::Incremental);
            }

            #[test]
            fn test_frame_change_handling_try_from_serde_map_missing() {
                let map = serde_json::Map::new();
                let result = FrameChangeHandling::try_from_serde_map(&map);
                assert!(result.is_err());
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

            #[test]
            fn test_percentage_neuron_positioning_display() {
                assert_eq!(format!("{}", PercentageNeuronPositioning::Linear), "Linear");
                assert_eq!(
                    format!("{}", PercentageNeuronPositioning::Fractional),
                    "Fractional"
                );
            }

            #[test]
            fn test_percentage_neuron_positioning_try_from_serde_map() {
                let mut map = serde_json::Map::new();
                map.insert(
                    "percentage_neuron_positioning".to_string(),
                    serde_json::json!("Linear"),
                );
                let result = PercentageNeuronPositioning::try_from_serde_map(&map).unwrap();
                assert_eq!(result, PercentageNeuronPositioning::Linear);

                map.insert(
                    "percentage_neuron_positioning".to_string(),
                    serde_json::json!("Fractional"),
                );
                let result = PercentageNeuronPositioning::try_from_serde_map(&map).unwrap();
                assert_eq!(result, PercentageNeuronPositioning::Fractional);
            }

            #[test]
            fn test_percentage_neuron_positioning_try_from_serde_map_missing() {
                let map = serde_json::Map::new();
                let result = PercentageNeuronPositioning::try_from_serde_map(&map);
                assert!(result.is_err());
            }
        }

        mod test_io_cortical_area_configuration_flag_variants {
            use super::*;

            #[test]
            fn test_percentage_variant() {
                let io_type = IOCorticalAreaConfigurationFlag::Percentage(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Linear,
                );

                match io_type {
                    IOCorticalAreaConfigurationFlag::Percentage(_, _) => (),
                    _ => panic!("Expected Percentage variant"),
                }
            }

            #[test]
            fn test_signed_percentage_variant() {
                let io_type = IOCorticalAreaConfigurationFlag::SignedPercentage(
                    FrameChangeHandling::Incremental,
                    PercentageNeuronPositioning::Fractional,
                );

                match io_type {
                    IOCorticalAreaConfigurationFlag::SignedPercentage(_, _) => (),
                    _ => panic!("Expected SignedPercentage variant"),
                }
            }

            #[test]
            fn test_cartesian_plane_variant() {
                let io_type =
                    IOCorticalAreaConfigurationFlag::CartesianPlane(FrameChangeHandling::Absolute);

                match io_type {
                    IOCorticalAreaConfigurationFlag::CartesianPlane(_) => (),
                    _ => panic!("Expected CartesianPlane variant"),
                }
            }

            #[test]
            fn test_percentage_4d_variant() {
                let io_type = IOCorticalAreaConfigurationFlag::SignedPercentage4D(
                    FrameChangeHandling::Incremental,
                    PercentageNeuronPositioning::Fractional,
                );

                match io_type {
                    IOCorticalAreaConfigurationFlag::SignedPercentage4D(_, _) => (),
                    _ => panic!("Expected SignedPercentage4D variant"),
                }
            }

            #[test]
            fn test_boolean_variant() {
                let io_type = IOCorticalAreaConfigurationFlag::Boolean;
                match io_type {
                    IOCorticalAreaConfigurationFlag::Boolean => (),
                    _ => panic!("Expected Boolean variant"),
                }
            }

            #[test]
            fn test_percentage_2d_variant() {
                let io_type = IOCorticalAreaConfigurationFlag::Percentage2D(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Linear,
                );
                match io_type {
                    IOCorticalAreaConfigurationFlag::Percentage2D(_, _) => (),
                    _ => panic!("Expected Percentage2D variant"),
                }
            }

            #[test]
            fn test_percentage_3d_variant() {
                let io_type = IOCorticalAreaConfigurationFlag::Percentage3D(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Linear,
                );
                match io_type {
                    IOCorticalAreaConfigurationFlag::Percentage3D(_, _) => (),
                    _ => panic!("Expected Percentage3D variant"),
                }
            }

            #[test]
            fn test_signed_percentage_2d_variant() {
                let io_type = IOCorticalAreaConfigurationFlag::SignedPercentage2D(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Fractional,
                );
                match io_type {
                    IOCorticalAreaConfigurationFlag::SignedPercentage2D(_, _) => (),
                    _ => panic!("Expected SignedPercentage2D variant"),
                }
            }

            #[test]
            fn test_misc_variant() {
                let io_type =
                    IOCorticalAreaConfigurationFlag::Misc(FrameChangeHandling::Incremental);
                match io_type {
                    IOCorticalAreaConfigurationFlag::Misc(_) => (),
                    _ => panic!("Expected Misc variant"),
                }
            }
        }

        mod test_data_type_configuration_flag {
            use super::*;

            #[test]
            fn test_to_configuration_flag_percentage() {
                let io_type = IOCorticalAreaConfigurationFlag::Percentage(
                    FrameChangeHandling::Incremental,
                    PercentageNeuronPositioning::Linear,
                );

                let flag = io_type.to_data_type_configuration_flag();
                assert_ne!(flag, 0);
            }

            #[test]
            fn test_configuration_flag_roundtrip_percentage() {
                let original = IOCorticalAreaConfigurationFlag::Percentage(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Linear,
                );

                let flag = original.to_data_type_configuration_flag();
                let recovered =
                    IOCorticalAreaConfigurationFlag::try_from_data_type_configuration_flag(flag)
                        .unwrap();

                assert_eq!(original, recovered);
            }

            #[test]
            fn test_configuration_flag_roundtrip_cartesian_plane() {
                let original = IOCorticalAreaConfigurationFlag::CartesianPlane(
                    FrameChangeHandling::Incremental,
                );

                let flag = original.to_data_type_configuration_flag();
                let recovered =
                    IOCorticalAreaConfigurationFlag::try_from_data_type_configuration_flag(flag)
                        .unwrap();

                assert_eq!(original, recovered);
            }

            #[test]
            fn test_different_types_produce_different_flags() {
                let type1 = IOCorticalAreaConfigurationFlag::Percentage(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Linear,
                );

                let type2 = IOCorticalAreaConfigurationFlag::SignedPercentage(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Linear,
                );

                let flag1 = type1.to_data_type_configuration_flag();
                let flag2 = type2.to_data_type_configuration_flag();

                assert_ne!(flag1, flag2);
            }

            #[test]
            fn test_configuration_flag_roundtrip_boolean() {
                let original = IOCorticalAreaConfigurationFlag::Boolean;
                let flag = original.to_data_type_configuration_flag();
                let recovered =
                    IOCorticalAreaConfigurationFlag::try_from_data_type_configuration_flag(flag)
                        .unwrap();
                assert_eq!(original, recovered);
            }

            #[test]
            fn test_configuration_flag_roundtrip_misc() {
                let original = IOCorticalAreaConfigurationFlag::Misc(FrameChangeHandling::Absolute);
                let flag = original.to_data_type_configuration_flag();
                let recovered =
                    IOCorticalAreaConfigurationFlag::try_from_data_type_configuration_flag(flag)
                        .unwrap();
                assert_eq!(original, recovered);
            }

            #[test]
            fn test_configuration_flag_roundtrip_all_variants() {
                let variants = [
                    IOCorticalAreaConfigurationFlag::Boolean,
                    IOCorticalAreaConfigurationFlag::Percentage(
                        FrameChangeHandling::Absolute,
                        PercentageNeuronPositioning::Linear,
                    ),
                    IOCorticalAreaConfigurationFlag::Percentage2D(
                        FrameChangeHandling::Incremental,
                        PercentageNeuronPositioning::Fractional,
                    ),
                    IOCorticalAreaConfigurationFlag::Percentage3D(
                        FrameChangeHandling::Absolute,
                        PercentageNeuronPositioning::Fractional,
                    ),
                    IOCorticalAreaConfigurationFlag::Percentage4D(
                        FrameChangeHandling::Incremental,
                        PercentageNeuronPositioning::Linear,
                    ),
                    IOCorticalAreaConfigurationFlag::SignedPercentage(
                        FrameChangeHandling::Absolute,
                        PercentageNeuronPositioning::Linear,
                    ),
                    IOCorticalAreaConfigurationFlag::SignedPercentage2D(
                        FrameChangeHandling::Incremental,
                        PercentageNeuronPositioning::Fractional,
                    ),
                    IOCorticalAreaConfigurationFlag::SignedPercentage3D(
                        FrameChangeHandling::Absolute,
                        PercentageNeuronPositioning::Fractional,
                    ),
                    IOCorticalAreaConfigurationFlag::SignedPercentage4D(
                        FrameChangeHandling::Incremental,
                        PercentageNeuronPositioning::Linear,
                    ),
                    IOCorticalAreaConfigurationFlag::CartesianPlane(FrameChangeHandling::Absolute),
                    IOCorticalAreaConfigurationFlag::Misc(FrameChangeHandling::Incremental),
                ];

                for original in variants {
                    let flag = original.to_data_type_configuration_flag();
                    let recovered =
                        IOCorticalAreaConfigurationFlag::try_from_data_type_configuration_flag(
                            flag,
                        )
                        .unwrap();
                    assert_eq!(original, recovered, "Roundtrip failed for {:?}", original);
                }
            }

            #[test]
            fn test_configuration_flag_invalid_variant() {
                // Variant 255 is invalid
                let invalid_flag: u16 = 255;
                let result = IOCorticalAreaConfigurationFlag::try_from_data_type_configuration_flag(
                    invalid_flag,
                );
                assert!(result.is_err());
            }

            #[test]
            fn test_from_trait_implementations() {
                let io_type = IOCorticalAreaConfigurationFlag::Percentage(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Linear,
                );

                // Test From<&IOCorticalAreaConfigurationFlag>
                let flag_from_ref: u16 = (&io_type).into();
                assert_eq!(flag_from_ref, io_type.to_data_type_configuration_flag());

                // Test From<IOCorticalAreaConfigurationFlag>
                let flag_from_owned: u16 = io_type.into();
                assert_eq!(flag_from_owned, flag_from_ref);
            }

            #[test]
            fn test_try_from_trait_implementation() {
                let original =
                    IOCorticalAreaConfigurationFlag::CartesianPlane(FrameChangeHandling::Absolute);
                let flag = original.to_data_type_configuration_flag();

                let recovered: IOCorticalAreaConfigurationFlag = flag.try_into().unwrap();
                assert_eq!(original, recovered);
            }
        }

        mod test_as_io_cortical_id {
            use super::*;

            #[test]
            fn test_as_io_cortical_id_creates_valid_id() {
                let io_type = IOCorticalAreaConfigurationFlag::Percentage(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Linear,
                );

                let unit_id = *b"tst";
                let unit_index = CorticalSubUnitIndex::from(0u8);
                let group_index = CorticalUnitIndex::from(5u8);

                let cortical_id = io_type.as_io_cortical_id(true, unit_id, group_index, unit_index);

                let bytes = cortical_id.as_bytes();
                assert_eq!(bytes[0], b'i');
            }

            #[test]
            fn test_as_io_cortical_id_input_vs_output() {
                let io_type = IOCorticalAreaConfigurationFlag::Percentage(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Linear,
                );

                let unit_id = *b"tst";
                let unit_index = CorticalSubUnitIndex::from(0u8);
                let group_index = CorticalUnitIndex::from(0u8);

                let input_id = io_type.as_io_cortical_id(true, unit_id, group_index, unit_index);
                let output_id = io_type.as_io_cortical_id(false, unit_id, group_index, unit_index);

                assert_ne!(input_id, output_id);

                let input_bytes = input_id.as_bytes();
                let output_bytes = output_id.as_bytes();

                assert_eq!(input_bytes[0], b'i');
                assert_eq!(output_bytes[0], b'o');
            }

            #[test]
            fn test_as_io_cortical_id_preserves_unit_identifier() {
                let io_type = IOCorticalAreaConfigurationFlag::Boolean;
                let unit_id = *b"abc";
                let cortical_id = io_type.as_io_cortical_id(
                    true,
                    unit_id,
                    CorticalUnitIndex::from(0u8),
                    CorticalSubUnitIndex::from(0u8),
                );

                let bytes = cortical_id.as_bytes();
                assert_eq!(&bytes[1..4], b"abc");
            }

            #[test]
            fn test_as_io_cortical_id_encodes_indices() {
                let io_type = IOCorticalAreaConfigurationFlag::Boolean;
                let unit_index = CorticalUnitIndex::from(5u8);
                let sub_unit_index = CorticalSubUnitIndex::from(3u8);

                let cortical_id =
                    io_type.as_io_cortical_id(true, *b"tst", unit_index, sub_unit_index);

                let bytes = cortical_id.as_bytes();
                // Byte 7 is cortical_unit_index, byte 6 is cortical_sub_unit_index
                assert_eq!(bytes[7], 5u8);
                assert_eq!(bytes[6], 3u8);
            }
        }

        mod test_io_cortical_area_configuration_flag_display {
            use super::*;

            #[test]
            fn test_boolean_display() {
                let io_type = IOCorticalAreaConfigurationFlag::Boolean;
                assert_eq!(format!("{}", io_type), "Boolean()");
            }

            #[test]
            fn test_percentage_display() {
                let io_type = IOCorticalAreaConfigurationFlag::Percentage(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Linear,
                );
                let display = format!("{}", io_type);
                assert!(display.contains("Percentage"));
                assert!(display.contains("Absolute"));
                assert!(display.contains("Linear"));
            }

            #[test]
            fn test_cartesian_plane_display() {
                let io_type = IOCorticalAreaConfigurationFlag::CartesianPlane(
                    FrameChangeHandling::Incremental,
                );
                let display = format!("{}", io_type);
                assert!(display.contains("CartesianPlane"));
                assert!(display.contains("Incremental"));
            }

            #[test]
            fn test_misc_display() {
                let io_type = IOCorticalAreaConfigurationFlag::Misc(FrameChangeHandling::Absolute);
                let display = format!("{}", io_type);
                assert!(display.contains("Misc"));
                assert!(display.contains("Absolute"));
            }

            #[test]
            fn test_all_percentage_variants_display() {
                let variants = [
                    (
                        "Percentage2D",
                        IOCorticalAreaConfigurationFlag::Percentage2D(
                            FrameChangeHandling::Absolute,
                            PercentageNeuronPositioning::Linear,
                        ),
                    ),
                    (
                        "Percentage3D",
                        IOCorticalAreaConfigurationFlag::Percentage3D(
                            FrameChangeHandling::Incremental,
                            PercentageNeuronPositioning::Fractional,
                        ),
                    ),
                    (
                        "Percentage4D",
                        IOCorticalAreaConfigurationFlag::Percentage4D(
                            FrameChangeHandling::Absolute,
                            PercentageNeuronPositioning::Linear,
                        ),
                    ),
                    (
                        "SignedPercentage",
                        IOCorticalAreaConfigurationFlag::SignedPercentage(
                            FrameChangeHandling::Incremental,
                            PercentageNeuronPositioning::Fractional,
                        ),
                    ),
                    (
                        "SignedPercentage2D",
                        IOCorticalAreaConfigurationFlag::SignedPercentage2D(
                            FrameChangeHandling::Absolute,
                            PercentageNeuronPositioning::Linear,
                        ),
                    ),
                    (
                        "SignedPercentage3D",
                        IOCorticalAreaConfigurationFlag::SignedPercentage3D(
                            FrameChangeHandling::Incremental,
                            PercentageNeuronPositioning::Fractional,
                        ),
                    ),
                    (
                        "SignedPercentage4D",
                        IOCorticalAreaConfigurationFlag::SignedPercentage4D(
                            FrameChangeHandling::Absolute,
                            PercentageNeuronPositioning::Linear,
                        ),
                    ),
                ];

                for (expected_name, variant) in variants {
                    let display = format!("{}", variant);
                    assert!(
                        display.contains(expected_name),
                        "Expected '{}' in display '{}' for {:?}",
                        expected_name,
                        display,
                        variant
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod test_descriptors {
    use feagi_structures::genomic::descriptors::GenomeCoordinate;

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

/// Tests for genomic/motor_cortical_unit.rs
#[cfg(test)]
mod test_motor_cortical_unit {
    use super::*;

    #[test]
    fn test_object_segmentation_cortical_id_and_default_topology() {
        let group = CorticalUnitIndex::from(0u8);
        let ids = MotorCorticalUnit::get_cortical_ids_array_for_object_segmentation_with_parameters(
            FrameChangeHandling::Absolute,
            group,
        );

        let bytes = ids[0].as_bytes();
        assert_eq!(bytes[0], b'o', "Expected OPU cortical id prefix 'o'");
        assert_eq!(
            &bytes[1..4],
            b"seg",
            "Expected subtype 'seg' for object segmentation"
        );

        // IOCorticalAreaDataFlag::Misc(Absolute) => variant=10 (0x0A), frame bit=0, positioning bit=0
        assert_eq!(
            bytes[4], 10,
            "Expected data type variant 10 (Misc) in low config byte"
        );
        assert_eq!(
            bytes[5], 0,
            "Expected Absolute frame handling in high config byte"
        );

        let topology = MotorCorticalUnit::ObjectSegmentation.get_unit_default_topology();
        let unit = topology
            .get(&0.into())
            .expect("Missing topology entry for area 0");
        assert_eq!(unit.channel_dimensions_default, [32, 32, 8]);
    }

    #[test]
    fn test_text_english_output_cortical_id_and_default_topology() {
        let group = CorticalUnitIndex::from(0u8);
        let ids = MotorCorticalUnit::get_cortical_ids_array_for_text_english_output_with_parameters(
            FrameChangeHandling::Absolute,
            group,
        );

        let bytes = ids[0].as_bytes();
        assert_eq!(bytes[0], b'o', "Expected OPU cortical id prefix 'o'");
        assert_eq!(
            &bytes[1..4],
            b"ten",
            "Expected subtype 'ten' for text encoding"
        );

        // IOCorticalAreaDataFlag::Misc(Absolute) => variant=10 (0x0A), frame bit=0, positioning bit=0
        assert_eq!(
            bytes[4], 10,
            "Expected data type variant 10 (Misc) in low config byte"
        );
        assert_eq!(
            bytes[5], 0,
            "Expected Absolute frame handling in high config byte"
        );

        let topology = MotorCorticalUnit::TextEnglishOutput.get_unit_default_topology();
        let unit = topology
            .get(&0.into())
            .expect("Missing topology entry for area 0");
        assert_eq!(unit.channel_dimensions_default, [1, 1, 16]);
    }
}

/// Tests for genomic/sensory_cortical_unit.rs
#[cfg(test)]
mod test_sensory_cortical_unit {
    use super::*;

    mod test_basic_properties {
        use super::*;

        #[test]
        fn test_sensory_cortical_unit_display() {
            assert_eq!(
                format!("{}", SensoryCorticalUnit::Infrared),
                "Infrared Sensor"
            );
            assert_eq!(
                format!("{}", SensoryCorticalUnit::SegmentedVision),
                "Segmented Vision"
            );
        }

        #[test]
        fn test_sensory_cortical_unit_snake_case_name() {
            assert_eq!(
                SensoryCorticalUnit::Infrared.get_snake_case_name(),
                "infrared"
            );
            assert_eq!(
                SensoryCorticalUnit::SegmentedVision.get_snake_case_name(),
                "segmented_vision"
            );
        }

        #[test]
        fn test_text_english_input_cortical_id_and_default_topology() {
            let group = CorticalUnitIndex::from(0u8);
            let ids =
                SensoryCorticalUnit::get_cortical_ids_array_for_text_english_input_with_parameters(
                    FrameChangeHandling::Absolute,
                    group,
                );

            let bytes = ids[0].as_bytes();
            assert_eq!(bytes[0], b'i', "Expected IPU cortical id prefix 'i'");
            assert_eq!(
                &bytes[1..4],
                b"ten",
                "Expected subtype 'ten' for text encoding"
            );

            // IOCorticalAreaDataFlag::Misc(Absolute) => variant=10 (0x0A), frame bit=0, positioning bit=0
            assert_eq!(
                bytes[4], 10,
                "Expected data type variant 10 (Misc) in low config byte"
            );
            assert_eq!(
                bytes[5], 0,
                "Expected Absolute frame handling in high config byte"
            );

            let topology = SensoryCorticalUnit::TextEnglishInput.get_unit_default_topology();
            let unit = topology
                .get(&0.into())
                .expect("Missing topology entry for area 0");
            assert_eq!(unit.channel_dimensions_default, [1, 1, 16]);
        }
    }

    mod test_infrared {
        use super::*;

        #[test]
        fn test_infrared_cortical_area_types_array() {
            let frame_handling = FrameChangeHandling::Absolute;
            let positioning = PercentageNeuronPositioning::Linear;

            let types =
                SensoryCorticalUnit::get_cortical_area_types_array_for_infrared_with_parameters(
                    frame_handling,
                    positioning,
                );

            assert_eq!(types.len(), 1);
            match types[0] {
                CorticalAreaType::BrainInput(_) => (),
                _ => panic!("Expected BrainInput type"),
            }
        }

        #[test]
        fn test_infrared_cortical_ids_array() {
            let frame_handling = FrameChangeHandling::Absolute;
            let positioning = PercentageNeuronPositioning::Linear;
            let group = CorticalUnitIndex::from(5u8);

            let ids = SensoryCorticalUnit::get_cortical_ids_array_for_infrared_with_parameters(
                frame_handling,
                positioning,
                group,
            );

            assert_eq!(ids.len(), 1);

            let bytes = ids[0].as_bytes();
            assert_eq!(bytes[0], b'i', "Sensor ID should start with 'i'");
            assert_eq!(&bytes[1..4], b"inf", "Infrared ID should contain 'inf'");
        }
    }

    mod test_segmented_vision {
        use super::*;

        #[test]
        fn test_segmented_vision_cortical_area_types_array() {
            let frame_handling = FrameChangeHandling::Incremental;

            let types =
                SensoryCorticalUnit::get_cortical_area_types_array_for_segmented_vision_with_parameters(
                    frame_handling,
                );

            assert_eq!(types.len(), 9);

            for area_type in types.iter() {
                match area_type {
                    CorticalAreaType::BrainInput(_) => (),
                    _ => panic!("Expected BrainInput type"),
                }
            }
        }

        #[test]
        fn test_segmented_vision_cortical_ids_array() {
            let frame_handling = FrameChangeHandling::Incremental;
            let group = CorticalUnitIndex::from(3u8);

            let ids =
                SensoryCorticalUnit::get_cortical_ids_array_for_segmented_vision_with_parameters(
                    frame_handling,
                    group,
                );

            assert_eq!(ids.len(), 9);

            for (i, id) in ids.iter().enumerate() {
                let bytes = id.as_bytes();
                assert_eq!(bytes[0], b'i', "Sensor ID should start with 'i'");
                assert_eq!(
                    &bytes[1..4],
                    b"svi",
                    "Segmented vision ID should contain 'svi'"
                );

                // Check that IDs are unique
                for (j, other_id) in ids.iter().enumerate() {
                    if i != j {
                        assert_ne!(
                            id, other_id,
                            "IDs at index {} and {} should be different",
                            i, j
                        );
                    }
                }
            }
        }
    }

    mod test_parameter_variations {
        use super::*;

        #[test]
        fn test_different_frame_handling_produces_different_ids() {
            let positioning = PercentageNeuronPositioning::Linear;
            let group = CorticalUnitIndex::from(0u8);

            let absolute_ids =
                SensoryCorticalUnit::get_cortical_ids_array_for_infrared_with_parameters(
                    FrameChangeHandling::Absolute,
                    positioning,
                    group,
                );

            let incremental_ids =
                SensoryCorticalUnit::get_cortical_ids_array_for_infrared_with_parameters(
                    FrameChangeHandling::Incremental,
                    positioning,
                    group,
                );

            assert_ne!(
                absolute_ids[0], incremental_ids[0],
                "Different frame handling should produce different IDs"
            );
        }

        #[test]
        fn test_different_positioning_produces_different_ids() {
            let frame_handling = FrameChangeHandling::Absolute;
            let group = CorticalUnitIndex::from(0u8);

            let linear_ids =
                SensoryCorticalUnit::get_cortical_ids_array_for_infrared_with_parameters(
                    frame_handling,
                    PercentageNeuronPositioning::Linear,
                    group,
                );

            let fractional_ids =
                SensoryCorticalUnit::get_cortical_ids_array_for_infrared_with_parameters(
                    frame_handling,
                    PercentageNeuronPositioning::Fractional,
                    group,
                );

            assert_ne!(
                linear_ids[0], fractional_ids[0],
                "Different positioning should produce different IDs"
            );
        }

        #[test]
        fn test_different_groups_produce_different_ids() {
            let frame_handling = FrameChangeHandling::Absolute;
            let positioning = PercentageNeuronPositioning::Linear;

            let group0_ids =
                SensoryCorticalUnit::get_cortical_ids_array_for_infrared_with_parameters(
                    frame_handling,
                    positioning,
                    CorticalUnitIndex::from(0u8),
                );

            let group1_ids =
                SensoryCorticalUnit::get_cortical_ids_array_for_infrared_with_parameters(
                    frame_handling,
                    positioning,
                    CorticalUnitIndex::from(1u8),
                );

            assert_ne!(
                group0_ids[0], group1_ids[0],
                "Different groups should produce different IDs"
            );
        }

        #[test]
        fn test_sensory_unit_with_multiple_parameters() {
            let params = [
                (
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Linear,
                ),
                (
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Fractional,
                ),
                (
                    FrameChangeHandling::Incremental,
                    PercentageNeuronPositioning::Linear,
                ),
                (
                    FrameChangeHandling::Incremental,
                    PercentageNeuronPositioning::Fractional,
                ),
            ];

            let group = CorticalUnitIndex::from(0u8);
            let mut all_ids = Vec::new();

            for (frame, pos) in params.iter() {
                let ids = SensoryCorticalUnit::get_cortical_ids_array_for_infrared_with_parameters(
                    *frame, *pos, group,
                );
                all_ids.push(ids[0]);
            }

            // All IDs should be unique
            for (i, id1) in all_ids.iter().enumerate() {
                for (j, id2) in all_ids.iter().enumerate() {
                    if i != j {
                        assert_ne!(
                            id1, id2,
                            "IDs at positions {} and {} should be different",
                            i, j
                        );
                    }
                }
            }
        }
    }
}

/// Comprehensive integration tests spanning multiple modules
#[cfg(test)]
mod test_comprehensive_scenarios {
    use super::*;
    use feagi_structures::genomic::descriptors::GenomeCoordinate;

    #[test]
    fn test_coordinate_extreme_values() {
        let max_unsigned = NeuronVoxelCoordinate::new(u32::MAX, u32::MAX, u32::MAX);
        let min_signed = GenomeCoordinate::new(i32::MIN, i32::MIN, i32::MIN);
        let max_signed = GenomeCoordinate::new(i32::MAX, i32::MAX, i32::MAX);

        assert_eq!(max_unsigned.x, u32::MAX);
        assert_eq!(min_signed.x, i32::MIN);
        assert_eq!(max_signed.x, i32::MAX);
    }

    #[test]
    fn test_neuron_depth_with_coordinates() {
        let depth = NeuronDepth::new(100).unwrap();
        let coord = NeuronVoxelCoordinate::new(10, 20, *depth - 1);

        assert_eq!(*depth, 100);
        assert_eq!(coord.z, 99); // Within depth bounds
    }
}

/// Error handling tests
#[cfg(test)]
mod test_error_handling {
    use super::*;

    #[test]
    fn test_zero_values_in_counts() {
        assert!(CorticalChannelCount::new(0).is_err());
        assert!(NeuronDepth::new(0).is_err());
    }
}
