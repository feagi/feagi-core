use feagi_genome::cortical_area::foundation::{Death, Power};
use feagi_genome::cortical_area::motor::Led;
use feagi_genome::cortical_area::processing::{Interconnect, Memory};
use feagi_genome::cortical_area::sensor::Camera;
use feagi_genome::cortical_area::tunable::BrightnessAdjust;
use feagi_genome::cortical_area::{
    CorticalAreaTypeKind, CorticalAreaTypeMember, CorticalIdentifier, CorticalIdentifierParseError, CorticalIdentifierPath, FoundationCorticalIdentifierPath,
    MotorCorticalIdentifierPath, ProcessingCorticalIdentifierPath, SensorCorticalIdentifierPath, TunableCorticalIdentifierPath,
};
use std::collections::HashSet;

const INNER: [u8; 6] = *b"abcdef";

/// Every valid path, with the cortical type it is
fn all_paths() -> [(CorticalIdentifierPath, CorticalAreaTypeKind); 7] {
    [
        (CorticalIdentifierPath::Foundation(FoundationCorticalIdentifierPath::Power(INNER)), Power::KIND),
        (CorticalIdentifierPath::Foundation(FoundationCorticalIdentifierPath::Death(INNER)), Death::KIND),
        (CorticalIdentifierPath::Processing(ProcessingCorticalIdentifierPath::Interconnect(INNER)), Interconnect::KIND),
        (CorticalIdentifierPath::Processing(ProcessingCorticalIdentifierPath::Memory(INNER)), Memory::KIND),
        (CorticalIdentifierPath::Sensor(SensorCorticalIdentifierPath::Camera(INNER)), Camera::KIND),
        (CorticalIdentifierPath::Motor(MotorCorticalIdentifierPath::Led(INNER)), Led::KIND),
        (CorticalIdentifierPath::Tunable(TunableCorticalIdentifierPath::BrightnessAdjust(INNER)), BrightnessAdjust::KIND),
    ]
}

#[test]
fn identifier_is_exactly_8_bytes() {
    assert_eq!(size_of::<CorticalIdentifier>(), 8);
    assert_eq!(size_of::<Option<CorticalIdentifier>>(), 8);
}

#[test]
fn every_path_round_trips() {
    for (path, kind) in all_paths() {
        let id = CorticalIdentifier::from_path(path);
        assert_eq!(id.path(), path);
        assert_eq!(id.kind(), kind);
        assert_eq!(path.kind(), kind);
        assert_eq!(id.inner_bytes(), INNER);
        assert_eq!(CorticalIdentifier::try_from_bytes(id.to_bytes()), Ok(id));
    }
}

#[test]
fn new_matches_the_path_of_the_same_cortical_type() {
    let from_new = CorticalIdentifier::new::<Memory>(INNER);
    let from_path = CorticalIdentifier::from_path(CorticalIdentifierPath::Processing(ProcessingCorticalIdentifierPath::Memory(INNER)));
    assert_eq!(from_new, from_path);
}

/// These bytes are what saved data will hold, so a change here should be deliberate
#[test]
fn layout_is_super_type_then_cortical_type_then_inner() {
    assert_eq!(CorticalIdentifier::new::<Power>(INNER).to_bytes(), [1, 1, b'a', b'b', b'c', b'd', b'e', b'f']);
    assert_eq!(CorticalIdentifier::new::<Memory>(INNER).to_bytes(), [2, 2, b'a', b'b', b'c', b'd', b'e', b'f']);
    assert_eq!(CorticalIdentifier::new::<BrightnessAdjust>(INNER).to_bytes(), [5, 1, b'a', b'b', b'c', b'd', b'e', b'f']);
}

#[test]
fn invalid_bytes_name_the_layer_that_rejected_them() {
    let parse = |super_type, cortical_type| CorticalIdentifier::try_from_bytes([super_type, cortical_type, 0, 0, 0, 0, 0, 0]);

    assert_eq!(parse(0, 0), Err(CorticalIdentifierParseError::SuperType(0)));
    assert_eq!(parse(99, 1), Err(CorticalIdentifierParseError::SuperType(99)));
    assert_eq!(parse(1, 0), Err(CorticalIdentifierParseError::CorticalType { super_type: 1, cortical_type: 0 }));
    // Foundation has a type 2, Sensor does not
    assert_eq!(parse(3, 2), Err(CorticalIdentifierParseError::CorticalType { super_type: 3, cortical_type: 2 }));
}

/// Tries all 2^16 possible headers. Exactly the cortical types must be valid, and each must return
/// the same bytes, so no two byte patterns can mean the same area
#[test]
fn every_possible_header_is_valid_only_if_a_cortical_type_and_is_canonical() {
    let mut valid = HashSet::new();
    for super_type in 0..=255u8 {
        for cortical_type in 0..=255u8 {
            let bytes = [super_type, cortical_type, b'a', b'b', b'c', b'd', b'e', b'f'];
            if let Ok(id) = CorticalIdentifier::try_from_bytes(bytes) {
                assert_eq!(id.to_bytes(), bytes);
                assert_eq!(id.path().to_bytes(), bytes);
                assert!(valid.insert(id));
            }
        }
    }
    assert_eq!(valid.len(), CorticalAreaTypeKind::ALL.len());
}

#[test]
fn postcard_is_the_8_bytes_with_nothing_added() {
    let id = CorticalIdentifier::new::<Camera>(INNER);
    let bytes = postcard::to_allocvec(&id).unwrap();
    assert_eq!(bytes, [3, 1, b'a', b'b', b'c', b'd', b'e', b'f']);
    assert_eq!(postcard::from_bytes::<CorticalIdentifier>(&bytes), Ok(id));

    assert!(postcard::from_bytes::<CorticalIdentifier>(&[0, 0, 0, 0, 0, 0, 0, 0]).is_err());
    assert!(postcard::from_bytes::<CorticalIdentifier>(&[3, 2, 0, 0, 0, 0, 0, 0]).is_err());
    assert_eq!(postcard::to_allocvec(&Some(id)).unwrap().len(), 9); // Postcard's own tag, not memory
}

#[test]
fn json_round_trips_and_rejects_invalid_bytes() {
    let id = CorticalIdentifier::new::<Led>(INNER);
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, "[4,1,97,98,99,100,101,102]");
    assert_eq!(serde_json::from_str::<CorticalIdentifier>(&json).unwrap(), id);
    assert!(serde_json::from_str::<CorticalIdentifier>("[3,2,0,0,0,0,0,0]").is_err());
}

/// A family of its own, as every real cortical type has plain bytes as its inner ID for now
mod custom_inner {
    use feagi_basis::quantization::quantizable::DataValueQuantizationError;
    use feagi_connectome::cortical_area::neuron_layout::voxel::NeuronLayoutTagVoxel;
    use feagi_connectome::cortical_area::{CorticalAreaSpec, NeuronLayoutTag};
    use feagi_connectome::family_macros::{define_cortical_type_family_template, expand_cortical_identifier, expand_cortical_type_family};
    use feagi_connectome::neuron_models::neuron_model;
    use feagi_connectome::neuron_models::neuron_model::feagi_simple::NeuronModelFeagiSimple;
    use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
    use feagi_genome::cortical_area::CorticalAreaInnerID;

    pub trait TestType {
        type NeuronLayout: NeuronLayoutTag;
        type Inner: CorticalAreaInnerID;
    }

    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct Channel {
        pub unit: u8,
        pub rest: [u8; 5],
    }

    impl From<Channel> for [u8; 6] {
        fn from(channel: Channel) -> Self {
            let [a, b, c, d, e] = channel.rest;
            [channel.unit, a, b, c, d, e]
        }
    }

    impl TryFrom<[u8; 6]> for Channel {
        type Error = ();

        fn try_from(bytes: [u8; 6]) -> Result<Self, ()> {
            let [unit, a, b, c, d, e] = bytes;
            if unit == 0xFF {
                return Err(());
            }
            Ok(Self { unit, rest: [a, b, c, d, e] })
        }
    }

    impl CorticalAreaInnerID for Channel {}

    pub mod device {
        use super::*;

        pub struct Thing;

        impl TestType for Thing {
            type NeuronLayout = NeuronLayoutTagVoxel;
            type Inner = Channel;
        }

        impl UsesFeagiSimple for Thing {
            fn default_spec<FIQ: FeagiIndexQuantization, NM: NeuronModelFeagiSimple>() -> Result<CorticalAreaSpec<FIQ, NM>, DataValueQuantizationError> {
                unimplemented!()
            }
        }
    }

    define_cortical_type_family_template! {
        Family: TestType,
        Error: TestTypeSelectionError,
        ModelFamily: NeuronModel,
        ModelFamilyError: NeuronModelSelectionError,
        Identifier: TestIdentifier,
        IdentifierError: TestIdentifierParseError,

        Types: {
            Device = 7: {
                Thing = 9: [FeagiSimple],
            }
        }
    }

    test_type_family_template!(expand_cortical_type_family);
    test_type_family_template!(expand_cortical_identifier);

    #[test]
    fn a_cortical_type_decides_what_its_inner_id_holds() {
        let channel = Channel { unit: 3, rest: *b"abcde" };
        let id = TestIdentifier::new::<device::Thing>(channel);
        assert_eq!(id.to_bytes(), [7, 9, 3, b'a', b'b', b'c', b'd', b'e']);
        assert_eq!(id.path(), TestIdentifierPath::Device(DeviceTestIdentifierPath::Thing(channel)));
    }

    #[test]
    fn a_cortical_type_may_reject_its_inner_id() {
        assert_eq!(
            TestIdentifier::try_from_bytes([7, 9, 0xFF, 0, 0, 0, 0, 0]),
            Err(TestIdentifierParseError::Inner(TestTypeKind::Device(DeviceSuperTypeKind::Thing)))
        );
    }
}
