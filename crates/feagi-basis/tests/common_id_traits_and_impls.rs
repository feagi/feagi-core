use feagi_basis::base_64_id;
use feagi_basis::common_id_traits_and_impls::{padded_url_safe_base64_len, Base64EncodedByteID, ByteID};

#[derive(Debug, PartialEq)]
struct RawId<const N: usize>([u8; N]);

#[derive(Debug, PartialEq)]
pub struct Reject;

impl core::fmt::Display for Reject {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("rejected id bytes")
    }
}

impl core::error::Error for Reject {}

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
        assert_eq!(encoded.try_into_id().expect("canonical encoding").0, bytes);
    }};
}

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
    // `From` only reads the bytes. Validation happens in `try_into_id`.
    let encoded = Base64EncodedByteID::<RawId<1>, 1, 4>::from(RawId([0xA5]));
    assert_eq!(encoded.try_into_id(), Err(Reject));
}

#[derive(Debug, PartialEq)]
pub struct MacroId([u8; 4]);

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
    assert_eq!(encoded.try_into_id().expect("macro alias roundtrip").0, bytes);
}
