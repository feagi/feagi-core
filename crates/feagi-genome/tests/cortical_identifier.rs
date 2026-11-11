use feagi_genome::basis::cortical_area::processing::CorticalIdentifierProcessing;
use feagi_genome::basis::cortical_area::{CorticalIdentifier, CorticalIdentifierBytes, CorticalIdentifierParseError};
use std::collections::HashSet;

const INNER: [u8; 6] = *b"abcdef";

/// The bytes of every valid cortical type, with `INNER` as the inner ID
const ALL_VALID: [[u8; 8]; 2] = [
    [0, 0, b'a', b'b', b'c', b'd', b'e', b'f'], // Foundation Power
    [1, 0, b'a', b'b', b'c', b'd', b'e', b'f'], // Processing Interconnect
];

#[test]
fn identifier_is_exactly_8_bytes() {
    assert_eq!(size_of::<CorticalIdentifier>(), 8);
    assert_eq!(size_of::<Option<CorticalIdentifier>>(), 8);
    assert_eq!(size_of::<CorticalIdentifierBytes>(), 8);
}

#[test]
fn every_cortical_type_round_trips() {
    for bytes in ALL_VALID {
        assert_eq!(CorticalIdentifierBytes::try_from_bytes(bytes).unwrap().to_bytes(), bytes);
    }
}

#[test]
fn invalid_bytes_name_the_layer_that_rejected_them() {
    let parse = |cortical_type, subtype| CorticalIdentifierBytes::try_from_bytes([cortical_type, subtype, 0, 0, 0, 0, 0, 0]);

    assert!(matches!(parse(2, 0), Err(CorticalIdentifierParseError::InvalidCorticalType(2))));
    assert!(matches!(parse(99, 0), Err(CorticalIdentifierParseError::InvalidCorticalType(99))));
    assert!(matches!(parse(0, 1), Err(CorticalIdentifierParseError::InvalidCorticalSubtype(1))));
    assert!(matches!(parse(1, 5), Err(CorticalIdentifierParseError::InvalidCorticalSubtype(5))));
}

/// Tries all 2^16 possible headers. Exactly the cortical types must be valid, and each must return
/// the same bytes, so no two byte patterns can mean the same area
#[test]
fn every_possible_header_is_valid_only_if_a_cortical_type_and_is_canonical() {
    let mut valid = HashSet::new();
    for cortical_type in 0..=255u8 {
        for subtype in 0..=255u8 {
            let bytes = [cortical_type, subtype, b'a', b'b', b'c', b'd', b'e', b'f'];
            if let Ok(id) = CorticalIdentifierBytes::try_from_bytes(bytes) {
                assert_eq!(id.to_bytes(), bytes);
                assert!(valid.insert(id));
            }
        }
    }
    assert_eq!(valid.len(), ALL_VALID.len());
}

#[test]
fn bytes_serialize_as_the_8_bytes() {
    let id = CorticalIdentifierBytes::try_from_bytes(ALL_VALID[1]).unwrap();

    let bytes = postcard::to_allocvec(&id).unwrap();
    assert_eq!(bytes, ALL_VALID[1]);
    assert_eq!(postcard::from_bytes::<CorticalIdentifierBytes>(&bytes).unwrap(), id);

    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, "[1,0,97,98,99,100,101,102]");
    assert_eq!(serde_json::from_str::<CorticalIdentifierBytes>(&json).unwrap(), id);
}

/// The identifier itself serializes as its nested enum, by cortical type then subtype
#[test]
fn identifier_serializes_by_type_then_subtype() {
    let id = CorticalIdentifier::Processing(CorticalIdentifierProcessing::Interconnect(INNER));
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, r#"{"Processing":{"Interconnect":[97,98,99,100,101,102]}}"#);
    assert_eq!(serde_json::from_str::<CorticalIdentifier>(&json).unwrap(), id);

    let bytes = postcard::to_allocvec(&id).unwrap();
    assert_eq!(postcard::from_bytes::<CorticalIdentifier>(&bytes).unwrap(), id);
}
