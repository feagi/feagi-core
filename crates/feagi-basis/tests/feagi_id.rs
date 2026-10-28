use feagi_basis::feagi_id::base_64::{FeagiIDBase64, FeagiIDBase64Error};
use feagi_basis::feagi_id::feagi_id::{FeagiID, HasBase64};
use feagi_basis::feagi_id::feagi_id;

/// Rejected ID bytes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reject;

impl core::fmt::Display for Reject {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("rejected id bytes")
    }
}

impl core::error::Error for Reject {}

fn reject_leading_a5(bytes: [u8; 4]) -> Result<CheckedId, Reject> {
    if bytes[0] == 0xA5 {
        return Err(Reject);
    }
    Ok(CheckedId(bytes))
}

const PLAIN_ID_BYTES: usize = 2;

feagi_id! {
    /// A 2-byte ID with no check, sized by a const
    pub PlainId: PLAIN_ID_BYTES,
}

feagi_id! {
    /// A 4-byte ID that rejects a leading `0xA5`
    pub CheckedId: 4,
    try_from = (Reject, reject_leading_a5),
    random,
    base64,
}

feagi_id! {
    /// An 8-byte ID with a custom base64 name
    pub RenamedId: 8,
    base64 = RenamedIdText,
    random,
}

//region Plain ID

#[test]
fn plain_id_accepts_every_byte_array() {
    const ID: PlainId = PlainId::from_id_bytes([1, 2]);
    assert_eq!(ID.get_id_bytes(), &[1, 2]);
    assert_eq!(PlainId::try_from_id_bytes([1, 2]), Ok(ID));
    assert_eq!(ID.into_id_bytes(), [1, 2]);
}

#[test]
fn plain_id_serializes_as_the_byte_array() {
    let id = PlainId::from_id_bytes([7, 255]);
    let json = serde_json::to_string(&id).expect("serialize");
    assert_eq!(json, "[7,255]");
    assert_eq!(serde_json::from_str::<PlainId>(&json).expect("deserialize"), id);
}

//endregion

//region Checked ID

#[test]
fn checked_id_uses_the_check_function() {
    assert_eq!(CheckedId::try_from_id_bytes([0xA5, 0, 0, 0]), Err(Reject));
    assert_eq!(CheckedId::try_from_id_bytes([1, 2, 3, 4]).expect("valid").get_id_bytes(), &[1, 2, 3, 4]);
}

#[test]
fn checked_id_deserialize_runs_the_check() {
    let json = serde_json::to_string(&CheckedId::try_from_id_bytes([1, 2, 3, 4]).expect("valid")).expect("serialize");
    assert_eq!(json, "[1,2,3,4]");
    assert!(serde_json::from_str::<CheckedId>(&json).is_ok());
    assert!(serde_json::from_str::<CheckedId>("[165,0,0,0]").is_err());
}

#[test]
fn new_random_only_returns_valid_ids() {
    for _ in 0..1000 {
        let id = CheckedId::new_random();
        assert_ne!(id.get_id_bytes()[0], 0xA5);
    }
    assert_ne!(RenamedId::new_random(), RenamedId::new_random());
}

//endregion

//region Base64

#[test]
fn base_64_roundtrip() {
    let id = CheckedId::try_from_id_bytes([9, 8, 7, 6]).expect("valid");
    let encoded: Base64CheckedId = id.to_base_64();
    assert_eq!(encoded.as_str().len(), 8);
    assert_eq!(encoded.to_string(), encoded.as_str());
    assert_eq!(encoded.as_ref(), encoded.as_str());
    assert_eq!(encoded.to_original(), id);
    assert_eq!(CheckedId::from_base_64(&encoded), id);
    assert_eq!(Base64CheckedId::try_from_str(encoded.as_str()).expect("canonical text"), encoded);
    assert_eq!(encoded.as_str().parse::<Base64CheckedId>().expect("from_str"), encoded);
    assert_eq!(Base64CheckedId::try_from(encoded.as_str()).expect("try_from str"), encoded);
    let stored: heapless::String<16> = heapless::String::try_from(encoded.as_str()).expect("8 chars fit in 16");
    assert_eq!(Base64CheckedId::try_from(&stored).expect("heapless text"), encoded);
}

#[test]
fn custom_base_64_name_and_length() {
    let id = RenamedId::from_id_bytes([0; 8]);
    let encoded: RenamedIdText = id.to_base_64();
    // 8 bytes pad up to 3 groups of 4 characters
    assert_eq!(encoded.as_str(), "AAAAAAAAAAA=");
    assert_eq!(encoded.to_original(), id);
}

#[test]
fn try_from_str_rejects_bad_text() {
    assert!(matches!(
        Base64CheckedId::try_from_str("AAAA"),
        Err(FeagiIDBase64Error::WrongLength { actual: 4, expected: 8 })
    ));
    assert!(matches!(Base64CheckedId::try_from_str("!!!!!!!!"), Err(FeagiIDBase64Error::InvalidEncoding)));
    // Both are 8 characters, but only "AAAAAA==" decodes to exactly 4 bytes.
    assert!(matches!(Base64CheckedId::try_from_str("AAAAAA=="), Ok(_)));
    assert!(matches!(Base64CheckedId::try_from_str("AAAA===="), Err(FeagiIDBase64Error::InvalidEncoding)));
    // 0xA5 0 0 0 encodes to "pQAAAA=="
    assert!(matches!(Base64CheckedId::try_from_str("pQAAAA=="), Err(FeagiIDBase64Error::Id(Reject))));
    assert!(matches!(Base64CheckedId::try_from_id_bytes(*b"!!!!!!!!"), Err(FeagiIDBase64Error::InvalidEncoding)));
}

#[test]
fn base_64_serde_uses_the_text() {
    let encoded = CheckedId::try_from_id_bytes([0xFB, 0xEF, 0x01, 0x02]).expect("valid").to_base_64();
    let json = serde_json::to_string(&encoded).expect("serialize");
    assert_eq!(json, format!("\"{}\"", encoded.as_str()));
    let parsed: Base64CheckedId = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed, encoded);
    assert!(serde_json::from_str::<Base64CheckedId>("\"AAAA====\"").is_err());
    assert!(serde_json::from_str::<Base64CheckedId>("\"pQAAAA==\"").is_err());
}

#[test]
fn from_converts_both_ways() {
    let id = CheckedId::try_from_id_bytes([9, 8, 7, 6]).expect("valid");
    let encoded = Base64CheckedId::from(id);
    assert_eq!(encoded, id.to_base_64());
    assert_eq!(Base64CheckedId::from(&id), encoded);
    assert_eq!(CheckedId::from(encoded), id);
    assert_eq!(CheckedId::from(&encoded), id);
    let encoded: Base64CheckedId = id.into();
    let back: CheckedId = encoded.into();
    assert_eq!(back, id);
}

#[test]
fn base_64_is_a_string_path_parameter() {
    use feagi_basis::feagi_message::feagi_message_description::FeagiFieldKind;
    use feagi_basis::feagi_message::feagi_message_path::parameters::FeagiMessagePathParameterElement;
    assert_eq!(<Base64CheckedId as FeagiMessagePathParameterElement>::KIND, FeagiFieldKind::String);
    let encoded = CheckedId::try_from_id_bytes([1, 2, 3, 4]).expect("valid").to_base_64();
    let mut segment = String::new();
    encoded.write_path_segment(&mut segment).expect("write");
    assert_eq!(segment.parse::<Base64CheckedId>().expect("parse back"), encoded);
}

//endregion

//region UUID

feagi_id! {
    /// A random v4 UUID ID
    pub RandomUuidId: 16,
    uuid = v4,
    base64,
}

feagi_id! {
    /// A time-ordered v7 UUID ID
    pub SortedUuidId: 16,
    uuid = v7,
}

#[test]
fn uuid_constructors_make_their_version() {
    use feagi_basis::feagi_id::uuid::FeagiUuidID;
    assert_eq!(RandomUuidId::new_random().as_uuid().get_version(), Some(uuid::Version::Random));
    assert_eq!(RandomUuidId::new_from_random_bytes([0xFF; 16]).as_uuid().get_version(), Some(uuid::Version::Random));
    assert_eq!(SortedUuidId::new_now().as_uuid().get_version(), Some(uuid::Version::SortRand));
    const AT_EPOCH: SortedUuidId = SortedUuidId::new_from_unix_millis(0, &[0; 10]);
    assert_eq!(AT_EPOCH.as_uuid().get_version(), Some(uuid::Version::SortRand));
    assert_eq!(<SortedUuidId as FeagiUuidID>::VERSION, uuid::Version::SortRand);
}

#[test]
fn uuid_check_rejects_other_versions() {
    use feagi_basis::feagi_id::uuid::{FeagiUuidID, UuidVersionError};
    let v7 = SortedUuidId::new_now();
    assert!(matches!(
        RandomUuidId::try_from_id_bytes(v7.into_id_bytes()),
        Err(UuidVersionError { expected: uuid::Version::Random, actual: Some(uuid::Version::SortRand) })
    ));
    assert!(matches!(SortedUuidId::try_from_id_bytes([0; 16]), Err(UuidVersionError { actual: Some(uuid::Version::Nil), .. })));
    assert_eq!(SortedUuidId::try_from_uuid(v7.as_uuid()).expect("v7"), v7);
    let json = serde_json::to_string(&RandomUuidId::new_from_random_bytes([0; 16])).expect("serialize");
    assert!(serde_json::from_str::<RandomUuidId>(&json).is_ok());
    assert!(serde_json::from_str::<SortedUuidId>(&json).is_err());
}

#[test]
fn uuid_base_64_roundtrip() {
    let id = RandomUuidId::new_random();
    let encoded: Base64RandomUuidId = id.into();
    // 16 bytes pad up to 6 groups of 4 characters
    assert_eq!(encoded.as_str().len(), 24);
    assert_eq!(RandomUuidId::from(encoded), id);
}

//endregion
