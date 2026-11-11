use core::str::FromStr;
use feagi_basis::feagi_id::base_64::FeagiIDBase64Error;
use feagi_basis::feagi_message::feagi_message_description::FeagiFieldKind;
use feagi_basis::feagi_message::feagi_message_path::configurables::FeagiMessagePathStaticElement;
use feagi_basis::feagi_message::feagi_message_path::parameters::FeagiMessagePathParameterElement;
use std::collections::HashMap;
use feagi_genome::basis::cortical_area::cortical_identifiers::{CorticalIdentifierBase64, CorticalIdentifierBytes, CorticalIdentifierParseError};

/// Processing Interconnect, with the inner ID "abcdef"
fn interconnect() -> CorticalIdentifierBytes {
    CorticalIdentifierBytes::try_from_bytes([1, 0, b'a', b'b', b'c', b'd', b'e', b'f']).unwrap()
}

/// This text is what paths and saved requests will hold, so a change here should be deliberate
#[test]
fn text_is_padded_url_safe_base64_of_the_8_bytes() {
    let base64 = CorticalIdentifierBase64::from(interconnect());
    assert_eq!(base64.as_str(), "AQBhYmNkZWY=");
    assert_eq!(base64.to_string(), "AQBhYmNkZWY=");
    assert_eq!(size_of::<CorticalIdentifierBase64>(), 12);
}

#[test]
fn every_cortical_type_round_trips() {
    let identifiers = [
        CorticalIdentifierBytes::try_from_bytes([0, 0, b'a', b'b', b'c', b'd', b'e', b'f']).unwrap(), // Foundation Power
        interconnect(),
    ];
    for identifier in identifiers {
        let base64 = CorticalIdentifierBase64::from(identifier);
        assert_eq!(CorticalIdentifierBytes::try_from(base64).unwrap(), identifier);
        assert_eq!(CorticalIdentifierBase64::from_str(base64.as_str()).unwrap(), base64);
    }
}

/// These bytes become `-` and `_`, where standard base64 would write `+` and `/`
#[test]
fn text_uses_the_url_safe_alphabet() {
    let identifier = CorticalIdentifierBytes::try_from_bytes([1, 0, 0xFB, 0xFB, 0xEF, 0xBE, 0xFF, 0xFF]).unwrap();
    let base64 = CorticalIdentifierBase64::from(identifier);
    assert!(base64.as_str().contains('-') && base64.as_str().contains('_'), "{base64}");
    assert!(!base64.as_str().contains('+') && !base64.as_str().contains('/'));
    assert_eq!(CorticalIdentifierBase64::from_str(base64.as_str()).unwrap(), base64);
}

#[test]
fn text_of_another_length_is_rejected() {
    assert!(matches!(
        CorticalIdentifierBase64::from_str("AQBh"),
        Err(FeagiIDBase64Error::WrongLength { actual: 4, expected: 12 })
    ));
    assert!(matches!(
        CorticalIdentifierBase64::from_str("AQBhYmNkZWY"),
        Err(FeagiIDBase64Error::WrongLength { actual: 11, expected: 12 })
    ));
}

#[test]
fn text_that_is_not_canonical_base64_of_8_bytes_is_rejected() {
    let invalid = [
        "!!!!!!!!!!!!", // Not base64
        "AQBhYmNkZWY+", // The standard alphabet instead of the URL safe one
        "AQBhYmNkZWZ=", // Bits set past the end of the last byte
        "AQBhYmNkZQ==", // Only 7 bytes
        "AQBhYmNkZWZn", // 9 bytes
    ];
    for text in invalid {
        assert!(matches!(CorticalIdentifierBase64::from_str(text), Err(FeagiIDBase64Error::InvalidEncoding)), "{text}");
    }
}

#[test]
fn text_of_an_invalid_identifier_is_rejected() {
    // The legacy ID "___power"
    assert!(matches!(
        CorticalIdentifierBase64::from_str("X19fcG93ZXI="),
        Err(FeagiIDBase64Error::Id(CorticalIdentifierParseError::InvalidCorticalType(95)))
    ));
    // [2, 0, 0, 0, 0, 0, 0, 0], as there is no cortical type 2
    assert!(matches!(
        CorticalIdentifierBase64::from_str("AgAAAAAAAAA="),
        Err(FeagiIDBase64Error::Id(CorticalIdentifierParseError::InvalidCorticalType(2)))
    ));
    // [1, 5, 0, 0, 0, 0, 0, 0], as Processing has no subtype 5
    assert!(matches!(
        CorticalIdentifierBase64::from_str("AQUAAAAAAAA="),
        Err(FeagiIDBase64Error::Id(CorticalIdentifierParseError::InvalidCorticalSubtype(5)))
    ));
}

#[test]
fn is_a_string_path_parameter_that_parses_back() {
    assert_eq!(<CorticalIdentifierBase64 as FeagiMessagePathParameterElement>::KIND, FeagiFieldKind::String);
    let base64 = CorticalIdentifierBase64::from(interconnect());
    let mut segment = String::new();
    base64.write_path_segment(&mut segment).unwrap();
    assert_eq!(segment, "AQBhYmNkZWY=");
    assert_eq!(segment.parse::<CorticalIdentifierBase64>().unwrap(), base64);
}

#[test]
fn is_a_static_path_element() {
    let base64 = CorticalIdentifierBase64::from(interconnect());
    assert_eq!(base64.to_str_slice(), "AQBhYmNkZWY=");
    assert_eq!(base64.as_ref(), "AQBhYmNkZWY=");
    assert_eq!(CorticalIdentifierBase64::try_from("AQBhYmNkZWY=").unwrap(), base64);
}

#[cfg(feature = "alloc")]
#[test]
fn serializes_as_its_text() {
    let base64 = CorticalIdentifierBase64::from(interconnect());
    let json = serde_json::to_string(&base64).unwrap();
    assert_eq!(json, "\"AQBhYmNkZWY=\"");
    assert_eq!(serde_json::from_str::<CorticalIdentifierBase64>(&json).unwrap(), base64);

    assert!(serde_json::from_str::<CorticalIdentifierBase64>("\"X19fcG93ZXI=\"").is_err());
    assert!(serde_json::from_str::<CorticalIdentifierBase64>("[1,0,97,98,99,100,101,102]").is_err());

    let bytes = postcard::to_allocvec(&base64).unwrap();
    assert_eq!(postcard::from_bytes::<CorticalIdentifierBase64>(&bytes).unwrap(), base64);
}


/// Requests hold these as the keys of JSON objects
#[cfg(feature = "alloc")]
#[test]
fn may_be_a_json_object_key() {
    let base64 = CorticalIdentifierBase64::from(interconnect());
    let map = HashMap::from([(base64, 7)]);
    let json = serde_json::to_string(&map).unwrap();
    assert_eq!(json, r#"{"AQBhYmNkZWY=":7}"#);
    assert_eq!(serde_json::from_str::<HashMap<CorticalIdentifierBase64, i32>>(&json).unwrap(), map);
}
