use feagi_basis::base_64_id;
use feagi_basis::feagi_message::feagi_message_element::MessageElementSerializable;
use feagi_basis::identifiers::byte_id::{padded_url_safe_base64_len, Base64EncodedByteID, Base64EncodedByteIdError, ByteID};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RawId<const N: usize>([u8; N]);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reject;

impl core::fmt::Display for Reject {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("rejected id bytes")
    }
}

impl core::error::Error for Reject {}

impl<const N: usize> core::fmt::Display for RawId<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl<const N: usize> serde::Serialize for RawId<N> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeTuple;
        let mut tuple = serializer.serialize_tuple(N)?;
        for byte in &self.0 {
            tuple.serialize_element(byte)?;
        }
        tuple.end()
    }
}

impl<'de, const N: usize> serde::Deserialize<'de> for RawId<N> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct RawIdVisitor<const N: usize>;

        impl<'de, const N: usize> serde::de::Visitor<'de> for RawIdVisitor<N> {
            type Value = RawId<N>;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(formatter, "an array of {N} bytes")
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut bytes = [0u8; N];
                for (index, byte) in bytes.iter_mut().enumerate() {
                    *byte = seq.next_element()?.ok_or_else(|| serde::de::Error::invalid_length(index, &self))?;
                }
                Ok(RawId(bytes))
            }
        }

        deserializer.deserialize_tuple(N, RawIdVisitor::<N>)
    }
}

impl<const N: usize> ByteID<N> for RawId<N> {
    type DeserializationError = Reject;

    fn get_id_bytes(&self) -> &[u8; N] {
        &self.0
    }

    fn try_from_id_bytes(bytes: [u8; N]) -> Result<Self, Self::DeserializationError> {
        if bytes.first() == Some(&0xA5) {
            return Err(Reject);
        }
        Ok(Self(bytes))
    }
}

/// `$n` has to be a literal. Stable Rust cannot compute `{ padded_url_safe_base64_len(N) }`
/// from a const generic, which is why the type carries both lengths.
macro_rules! assert_roundtrip {
    ($n:literal, $bytes:expr) => {{
        let bytes: [u8; $n] = $bytes;
        let encoded = Base64EncodedByteID::<RawId<$n>, $n, { padded_url_safe_base64_len($n) }>::from(RawId(bytes));
        let parsed =
            Base64EncodedByteID::<RawId<$n>, $n, { padded_url_safe_base64_len($n) }>::try_from_str(encoded.as_str()).expect("canonical text parses");
        assert_eq!(parsed.as_str(), encoded.as_str());
        assert_eq!(parsed.to_string(), encoded.as_str());
        assert_eq!(encoded.to_id().0, bytes);
        assert_eq!(parsed.to_id().0, bytes);
        let from_str: Base64EncodedByteID<RawId<$n>, $n, { padded_url_safe_base64_len($n) }> = encoded.as_str().parse().expect("FromStr");
        assert_eq!(from_str.as_str(), encoded.as_str());
        require_message_element::<Base64EncodedByteID<RawId<$n>, $n, { padded_url_safe_base64_len($n) }>>();
    }};
}

fn require_message_element<T: MessageElementSerializable>() {}

fn patterned<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    let mut index = 0;
    while index < N {
        bytes[index] = (index as u8).wrapping_mul(17).wrapping_add(3);
        index += 1;
    }
    if bytes.first() == Some(&0xA5) {
        bytes[0] = 0xA6;
    }
    bytes
}

#[test]
fn padded_lengths_cover_every_remainder() {
    assert_eq!(padded_url_safe_base64_len(0), 0);
    assert_eq!(padded_url_safe_base64_len(1), 4);
    assert_eq!(padded_url_safe_base64_len(2), 4);
    assert_eq!(padded_url_safe_base64_len(3), 4);
    assert_eq!(padded_url_safe_base64_len(4), 8);
    assert_eq!(padded_url_safe_base64_len(16), 24);
}

#[test]
fn encode_and_decode_roundtrip() {
    assert_roundtrip!(0, []);
    assert_roundtrip!(1, [0x00]);
    assert_roundtrip!(1, [0xFF]);
    assert_roundtrip!(2, [0xFB, 0xEF]);
    assert_roundtrip!(3, patterned::<3>());
    assert_roundtrip!(4, patterned::<4>());
    assert_roundtrip!(16, patterned::<16>());
    assert_roundtrip!(17, patterned::<17>());
}

#[test]
fn id_rejection_is_returned() {
    let rejected = Base64EncodedByteID::<RawId<1>, 1, 4>::from(RawId([0xA5]));
    assert!(matches!(
        Base64EncodedByteID::<RawId<1>, 1, 4>::try_from_str(rejected.as_str()),
        Err(Base64EncodedByteIdError::Id(Reject))
    ));
}

#[test]
fn try_from_str_rejects_wrong_length_alphabet_and_width() {
    type Encoded = Base64EncodedByteID<RawId<3>, 3, 4>;
    assert!(matches!(
        Encoded::try_from_str("AA"),
        Err(Base64EncodedByteIdError::WrongLength { actual: 2, expected: 4 })
    ));
    assert!(matches!(Encoded::try_from_str("!!!!"), Err(Base64EncodedByteIdError::InvalidEncoding)));
    // "AA==" is the 1-byte encoding of 0x00. It is 4 characters, but not a 3-byte ID.
    assert!(matches!(Encoded::try_from_str("AA=="), Err(Base64EncodedByteIdError::InvalidEncoding)));
}

#[test]
fn try_from_heapless_parses_the_same_text() {
    let encoded = Base64EncodedByteID::<RawId<1>, 1, 4>::from(RawId([0xFF]));
    let stored: heapless::String<8> = heapless::String::try_from(encoded.as_str()).expect("4 chars fit in 8");
    let parsed = Base64EncodedByteID::<RawId<1>, 1, 4>::try_from_heapless(&stored).expect("heapless text");
    assert_eq!(parsed.to_id().0, [0xFF]);
    let parsed = Base64EncodedByteID::<RawId<1>, 1, 4>::try_from(stored.as_str()).expect("str try_from");
    assert_eq!(parsed.as_str(), encoded.as_str());
}

#[test]
fn fmt_id_is_the_display_text() {
    struct Show(RawId<1>);
    impl core::fmt::Display for Show {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            Base64EncodedByteID::<RawId<1>, 1, 4>::fmt_id(&self.0, f)
        }
    }

    let id = RawId([0xFF]);
    let encoded = Base64EncodedByteID::<RawId<1>, 1, 4>::from(id.clone());
    assert_eq!(Show(id).to_string(), encoded.as_str());
}

#[test]
fn serde_roundtrip_uses_the_base64_text() {
    let encoded = Base64EncodedByteID::<RawId<2>, 2, 4>::from(RawId([0xFB, 0xEF]));
    let json = serde_json::to_string(&encoded).expect("serialize");
    assert_eq!(json, format!("\"{}\"", encoded.as_str()));
    let parsed: Base64EncodedByteID<RawId<2>, 2, 4> = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.to_id().0, [0xFB, 0xEF]);
    assert!(serde_json::from_str::<Base64EncodedByteID<RawId<2>, 2, 4>>("\"AA==\"").is_err());
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct MacroId([u8; 4]);

impl core::fmt::Display for MacroId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:02x}{:02x}{:02x}{:02x}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

impl ByteID<4> for MacroId {
    type DeserializationError = Reject;

    fn get_id_bytes(&self) -> &[u8; 4] {
        &self.0
    }

    fn try_from_id_bytes(bytes: [u8; 4]) -> Result<Self, Self::DeserializationError> {
        Ok(Self(bytes))
    }
}

base_64_id!(MacroId, 4);

#[test]
fn macro_alias_uses_the_padded_length() {
    let bytes = [9u8, 8, 7, 6];
    let encoded = Base64EncodedMacroId::from(MacroId(bytes));
    assert_eq!(encoded.to_id().0, bytes);
}
