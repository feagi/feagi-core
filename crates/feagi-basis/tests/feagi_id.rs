use feagi_basis::feagi_id::base_64::{deserialize_base_64, FeagiIDBase64, FeagiIDBase64Error};
use feagi_basis::feagi_id::feagi_id::{FeagiID, HasBase64};

/// Rejected ID bytes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reject;

impl core::fmt::Display for Reject {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("rejected id bytes")
    }
}

impl core::error::Error for Reject {}

/// A 4-byte ID that rejects a leading `0xA5`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
struct TestId([u8; 4]);

impl FeagiID<4> for TestId {
    type DeserializationError = Reject;

    fn get_id_bytes(&self) -> &[u8; 4] {
        &self.0
    }

    fn into_id_bytes(self) -> [u8; 4] {
        self.0
    }

    fn try_from_id_bytes(bytes: [u8; 4]) -> Result<Self, Self::DeserializationError> {
        if bytes[0] == 0xA5 {
            return Err(Reject);
        }
        Ok(Self(bytes))
    }
}

impl HasBase64<4, 8> for TestId {
    type Base64Type = Base64TestId;
}

impl core::fmt::Display for TestId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Base64TestId::fmt_original(self, f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Base64TestId([u8; 8]);

impl FeagiID<8> for Base64TestId {
    type DeserializationError = FeagiIDBase64Error<Reject>;

    fn get_id_bytes(&self) -> &[u8; 8] {
        &self.0
    }

    fn into_id_bytes(self) -> [u8; 8] {
        self.0
    }

    fn try_from_id_bytes(bytes: [u8; 8]) -> Result<Self, Self::DeserializationError> {
        Self::decode_original(&bytes)?;
        Ok(Self(bytes))
    }
}

impl FeagiIDBase64<4, 8> for Base64TestId {
    type Original = TestId;

    fn from_original(original: &Self::Original) -> Self {
        Self(Self::encode_original(original))
    }
}

impl core::fmt::Display for Base64TestId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl serde::Serialize for Base64TestId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for Base64TestId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserialize_base_64::<D, Self, 4, 8>(deserializer)
    }
}

#[test]
fn base_64_roundtrip() {
    let id = TestId([9, 8, 7, 6]);
    let encoded = id.to_base_64();
    assert_eq!(encoded.as_str().len(), 8);
    assert_eq!(encoded.as_str(), id.to_string());
    assert_eq!(encoded.to_string(), id.to_string());
    assert_eq!(encoded.to_original(), id);
    assert_eq!(TestId::from_base_64(&encoded), id);
    assert_eq!(Base64TestId::try_from_str(encoded.as_str()).expect("canonical text"), encoded);
    let stored: heapless::String<16> = heapless::String::try_from(encoded.as_str()).expect("8 chars fit in 16");
    assert_eq!(Base64TestId::try_from_heapless(&stored).expect("heapless text"), encoded);
}

#[test]
fn try_from_str_rejects_bad_text() {
    assert!(matches!(
        Base64TestId::try_from_str("AAAA"),
        Err(FeagiIDBase64Error::WrongLength { actual: 4, expected: 8 })
    ));
    assert!(matches!(Base64TestId::try_from_str("!!!!!!!!"), Err(FeagiIDBase64Error::InvalidEncoding)));
    // Both are 8 characters, but only "AAAAAA==" decodes to exactly 4 bytes.
    assert!(matches!(Base64TestId::try_from_str("AAAAAA=="), Ok(_)));
    assert!(matches!(Base64TestId::try_from_str("AAAA===="), Err(FeagiIDBase64Error::InvalidEncoding)));
    let rejected = Base64TestId::encode_original(&TestId([0xA5, 0, 0, 0]));
    let rejected = core::str::from_utf8(&rejected).expect("ascii");
    assert!(matches!(Base64TestId::try_from_str(rejected), Err(FeagiIDBase64Error::Id(Reject))));
    assert!(matches!(Base64TestId::try_from_id_bytes(*b"!!!!!!!!"), Err(FeagiIDBase64Error::InvalidEncoding)));
}

#[test]
fn serde_uses_the_base_64_text() {
    let encoded = TestId([0xFB, 0xEF, 0x01, 0x02]).to_base_64();
    let json = serde_json::to_string(&encoded).expect("serialize");
    assert_eq!(json, format!("\"{}\"", encoded.as_str()));
    let parsed: Base64TestId = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed, encoded);
    assert!(serde_json::from_str::<Base64TestId>("\"AAAA====\"").is_err());
}
