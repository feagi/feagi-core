//! Tests for the genomic module (cortical area types, IDs, I/O flags, descriptors).

use feagi_structures::genomic::cortical_area::descriptors::*;
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::*;
use feagi_structures::genomic::cortical_area::*;
use feagi_structures::genomic::{FeagiStructuresGenomicError, GenomeCoordinate3DI32, SensoryCorticalUnit};
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelCoordinate;
use feagi_structures::FeagiStructuresError;

// TODO: CorticalArea lives in cortical_area.rs but is not wired into `genomic::cortical_area` module exports yet; restore integration tests when `CorticalArea` is public again.

#[cfg(test)]
mod cortical_id_tests {
    use super::*;

    #[test]
    fn core_cortical_roundtrip_u64() {
        let id = CoreCorticalType::Power.to_cortical_id();
        let u = id.as_u64();
        let id2 = CorticalID::try_from_u64(u).unwrap();
        assert_eq!(id, id2);
    }

    #[test]
    fn core_cortical_base64_roundtrip() {
        let id = CoreCorticalType::Death.to_cortical_id();
        let s = id.as_base_64();
        let id2 = CorticalID::try_from_base_64(&s).unwrap();
        assert_eq!(id, id2);
    }

    #[test]
    fn invalid_first_byte_errors() {
        let bytes = *b"\xff_______";
        let err = CorticalID::try_from_bytes(&bytes).unwrap_err();
        assert!(matches!(err, FeagiStructuresGenomicError::CorticalIdError { .. }));
    }

    #[test]
    fn as_cortical_type_core() {
        let id = CoreCorticalType::Fatigue.to_cortical_id();
        let t = id.as_cortical_type().unwrap();
        assert_eq!(t, CorticalAreaType::Core(CoreCorticalType::Fatigue));
    }
}

#[cfg(test)]
mod io_config_flag_tests {
    use super::*;

    #[test]
    fn boolean_roundtrip_bitmask() {
        let flag = IOCorticalAreaConfigurationFlag::Boolean;
        let packed = flag.to_data_type_configuration_flag();
        let back =
            IOCorticalAreaConfigurationFlag::try_from_data_type_configuration_flag(packed).unwrap();
        assert_eq!(back, flag);
    }

    #[test]
    fn percentage_roundtrip() {
        let flag = IOCorticalAreaConfigurationFlag::Percentage(
            FrameChangeHandling::Incremental,
            PercentageNeuronPositioning::Fractional,
        );
        let packed = flag.to_data_type_configuration_flag();
        let back =
            IOCorticalAreaConfigurationFlag::try_from_data_type_configuration_flag(packed).unwrap();
        assert_eq!(back, flag);
    }

    #[test]
    fn try_from_trait() {
        let packed: IOCorticalAreaConfigurationFlagBitmask = 0;
        let flag: IOCorticalAreaConfigurationFlag = packed.try_into().unwrap();
        assert_eq!(flag, IOCorticalAreaConfigurationFlag::Boolean);
    }
}

#[cfg(test)]
mod descriptor_tests {
    use super::*;

    #[test]
    fn cortical_unit_index_inner() {
        let idx = CorticalUnitIndex::from(7u8);
        assert_eq!(idx.0, 7u8);
    }

    #[test]
    fn cortical_channel_count_ok_and_zero_err() {
        let count = CorticalChannelCount::new(5u32).unwrap();
        assert_eq!(count.get(), 5u32);
        let err = CorticalChannelCount::new(0u32).unwrap_err();
        assert!(matches!(err, FeagiStructuresError::InvalidValue { .. }));
    }

    #[test]
    fn neuron_depth_nonzero() {
        let depth = CorticalChannelNeuronDepth::new(3u32).unwrap();
        assert_eq!(depth.get(), 3u32);
        assert!(CorticalChannelNeuronDepth::new(0u32).is_err());
    }

    #[test]
    fn cortical_channel_dimensions_components() {
        let d = CorticalChannelDimensions::new(8u32, 16u32, 32u32).unwrap();
        assert_eq!(d.x.get(), 8);
        assert_eq!(d.y.get(), 16);
        assert_eq!(d.z.get(), 32);
    }

    #[test]
    fn neuron_voxel_coordinate_smoke() {
        let c = NeuronVoxelCoordinate::new(10u32, 20u32, 30u32);
        assert_eq!(c.x, 10);
        assert_eq!(c.y, 20);
        assert_eq!(c.z, 30);
    }
}

#[cfg(test)]
mod genome_coordinate_tests {
    use super::GenomeCoordinate3DI32;

    #[test]
    fn new_and_fields() {
        let g = GenomeCoordinate3DI32::new(1, -2, 3);
        assert_eq!(g.x, 1);
        assert_eq!(g.y, -2);
        assert_eq!(g.z, 3);
    }
}

#[cfg(test)]
mod sensory_unit_smoke {
    use super::*;

    #[test]
    fn list_all_contains_infrared() {
        let all = SensoryCorticalUnit::list_all();
        assert!(all.iter().any(|u| matches!(u, SensoryCorticalUnit::Infrared)));
    }

    #[test]
    fn default_cortical_id_for_group_is_deterministic() {
        let id1 = SensoryCorticalUnit::Infrared
            .get_default_cortical_id_for_group(CorticalUnitIndex::from(0u8));
        let id2 = SensoryCorticalUnit::Infrared
            .get_default_cortical_id_for_group(CorticalUnitIndex::from(0u8));
        assert_eq!(id1, id2);
    }
}

// TODO: Serialization-heavy and CorticalMapped / sparse voxel collection tests were removed when those types moved or were de-featured; restore once public API stabilizes.
