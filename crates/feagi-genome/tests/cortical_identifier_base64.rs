use core::str::FromStr;
use feagi_basis::feagi_id::base_64::FeagiIDBase64Error;
use feagi_basis::feagi_message::feagi_message_description::FeagiFieldKind;
use feagi_basis::feagi_message::feagi_message_path::configurables::FeagiMessagePathStaticElement;
use feagi_basis::feagi_message::feagi_message_path::parameters::FeagiMessagePathParameterElement;
use feagi_genome::cortical_area::cortical_identifier_base64::CorticalIdentifierBase64;
use feagi_genome::cortical_area::foundation::{Death, Power};
use feagi_genome::cortical_area::motor::Led;
use feagi_genome::cortical_area::processing::{Interconnect, Memory};
use feagi_genome::cortical_area::sensor::Camera;
use feagi_genome::cortical_area::tunable::BrightnessAdjust;
use feagi_genome::cortical_area::{CorticalIdentifier, CorticalIdentifierParseError};
use std::collections::HashMap;

const INNER: [u8; 6] = *b"abcdef";

fn camera() -> CorticalIdentifier {
    CorticalIdentifier::new::<Camera>(INNER)
}

/// This text is what paths and saved requests will hold, so a change here should be deliberate
#[test]
fn text_is_padded_url_safe_base64_of_the_8_bytes() {
    let base64 = CorticalIdentifierBase64::from(camera());
    assert_eq!(base64.as_str(), "AwFhYmNkZWY=");
    assert_eq!(base64.to_string(), "AwFhYmNkZWY=");
    assert_eq!(size_of::<CorticalIdentifierBase64>(), 12);
}

#[test]
fn every_cortical_type_round_trips() {
    let identifiers = [
        CorticalIdentifier::new::<Power>(INNER),
        CorticalIdentifier::new::<Death>(INNER),
        CorticalIdentifier::new::<Interconnect>(INNER),
        CorticalIdentifier::new::<Memory>(INNER),
        CorticalIdentifier::new::<Camera>(INNER),
        CorticalIdentifier::new::<Led>(INNER),
        CorticalIdentifier::new::<BrightnessAdjust>(INNER),
    ];
    for identifier in identifiers {
        let base64 = CorticalIdentifierBase64::from(identifier);
        assert_eq!(CorticalIdentifier::try_from(base64).unwrap(), identifier);
        let back: CorticalIdentifier = base64.try_into().unwrap();
        assert_eq!(back, identifier);
        assert_eq!(CorticalIdentifierBase64::from_str(base64.as_str()).unwrap(), base64);
    }
}

/// These bytes become `-` and `_`, where standard base64 would write `+` and `/`
#[test]
fn text_uses_the_url_safe_alphabet() {
    let base64 = CorticalIdentifierBase64::from(CorticalIdentifier::new::<Camera>([0xFB, 0xFB, 0xEF, 0xBE, 0xFF, 0xFF]));
    assert!(base64.as_str().contains('-') && base64.as_str().contains('_'), "{base64}");
    assert!(!base64.as_str().contains('+') && !base64.as_str().contains('/'));
    assert_eq!(CorticalIdentifierBase64::from_str(base64.as_str()).unwrap(), base64);
}

#[test]
fn text_of_another_length_is_rejected() {
    assert!(matches!(
        CorticalIdentifierBase64::from_str("AwFh"),
        Err(FeagiIDBase64Error::WrongLength { actual: 4, expected: 12 })
    ));
    assert!(matches!(
        CorticalIdentifierBase64::from_str("AwFhYmNkZWY"),
        Err(FeagiIDBase64Error::WrongLength { actual: 11, expected: 12 })
    ));
}

#[test]
fn text_that_is_not_canonical_base64_of_8_bytes_is_rejected() {
    let invalid = [
        "!!!!!!!!!!!!", // Not base64
        "AwFhYmNkZWY+", // The standard alphabet instead of the URL safe one
        "AwFhYmNkZWZ=", // Bits set past the end of the last byte
        "AwFhYmNkZQ==", // Only 7 bytes
        "AwFhYmNkZWZn", // 9 bytes
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
        Err(FeagiIDBase64Error::Id(CorticalIdentifierParseError::SuperType(95)))
    ));
    // [3, 2, 0, 0, 0, 0, 0, 0], as Sensor has no type 2
    assert!(matches!(
        CorticalIdentifierBase64::from_str("AwIAAAAAAAA="),
        Err(FeagiIDBase64Error::Id(CorticalIdentifierParseError::CorticalType { super_type: 3, cortical_type: 2 }))
    ));
}

#[test]
fn is_a_string_path_parameter_that_parses_back() {
    assert_eq!(<CorticalIdentifierBase64 as FeagiMessagePathParameterElement>::KIND, FeagiFieldKind::String);
    let base64 = CorticalIdentifierBase64::from(camera());
    let mut segment = String::new();
    base64.write_path_segment(&mut segment).unwrap();
    assert_eq!(segment, "AwFhYmNkZWY=");
    assert_eq!(segment.parse::<CorticalIdentifierBase64>().unwrap(), base64);
}

#[test]
fn is_a_static_path_element() {
    let base64 = CorticalIdentifierBase64::from(camera());
    assert_eq!(base64.to_str_slice(), "AwFhYmNkZWY=");
    assert_eq!(base64.as_ref(), "AwFhYmNkZWY=");
    assert_eq!(CorticalIdentifierBase64::try_from("AwFhYmNkZWY=").unwrap(), base64);
}

#[test]
fn serializes_as_its_text() {
    let base64 = CorticalIdentifierBase64::from(camera());
    let json = serde_json::to_string(&base64).unwrap();
    assert_eq!(json, "\"AwFhYmNkZWY=\"");
    assert_eq!(serde_json::from_str::<CorticalIdentifierBase64>(&json).unwrap(), base64);

    assert!(serde_json::from_str::<CorticalIdentifierBase64>("\"X19fcG93ZXI=\"").is_err());
    assert!(serde_json::from_str::<CorticalIdentifierBase64>("[3,1,97,98,99,100,101,102]").is_err());

    let bytes = postcard::to_allocvec(&base64).unwrap();
    assert_eq!(postcard::from_bytes::<CorticalIdentifierBase64>(&bytes).unwrap(), base64);
}

/// Requests hold these as the keys of JSON objects
#[test]
fn may_be_a_json_object_key() {
    let base64 = CorticalIdentifierBase64::from(camera());
    let map = HashMap::from([(base64, 7)]);
    let json = serde_json::to_string(&map).unwrap();
    assert_eq!(json, r#"{"AwFhYmNkZWY=":7}"#);
    assert_eq!(serde_json::from_str::<HashMap<CorticalIdentifierBase64, i32>>(&json).unwrap(), map);
}
