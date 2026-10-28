use crate::feagi_id::feagi_id::{FeagiID, HasBase64};
use base64::Engine;
use thiserror::Error;

// TODO use more optimized base64 encoding / decoding engines via cfg flags

/// The padded URL-safe base64 form of a [`HasBase64`] ID.
///
/// # Safety
///
/// The ID bytes of an implementor must always be the canonical padded URL-safe base64 encoding
/// of exactly `ID_BYTE_COUNT` bytes, and those bytes must pass the
/// [`FeagiIDBase64::Original`]'s `try_from_id_bytes`. The only ways to build one must be
/// [`FeagiIDBase64::from_original`] and `try_from_id_bytes`, and the original itself must have no
/// constructor that skips its own check. Only the `feagi_id!` macro implements this.
pub unsafe trait FeagiIDBase64<const ID_BYTE_COUNT: usize, const BASE_64_BYTE_COUNT: usize>:
FeagiID<BASE_64_BYTE_COUNT, DeserializationError = FeagiIDBase64Error<<Self::Original as FeagiID<ID_BYTE_COUNT>>::DeserializationError>>
+ core::str::FromStr + for<'a> TryFrom<&'a str> + core::convert::AsRef<str> + core::fmt::Display
+ crate::feagi_message::feagi_message_path::parameters::FeagiMessagePathParameterElement
+ crate::feagi_message::feagi_message_path::configurables::FeagiMessagePathStaticElement
{

    /// Fails when the two length consts disagree.
    ///
    /// Padded URL-safe base64 uses 4 characters for every 3 source bytes, rounded up.
    const BASE64_LEN_MATCHES_SOURCE: () = assert!(
        BASE_64_BYTE_COUNT == ID_BYTE_COUNT.div_ceil(3) * 4,
        "BASE_64_BYTE_COUNT must be the padded URL-safe base64 length of ID_BYTE_COUNT"
    );

    type Original: HasBase64<ID_BYTE_COUNT, BASE_64_BYTE_COUNT, Base64Type=Self>;

    /// Decode this base64 ID back into the original ID
    fn to_original(&self) -> Self::Original {
        let _: () = Self::BASE64_LEN_MATCHES_SOURCE;
        let mut decoded = [0u8; ID_BYTE_COUNT];
        // SAFETY: the trait contract says the text is the canonical encoding of a valid Original,
        // so it decodes to exactly ID_BYTE_COUNT bytes and those bytes pass the Original's check.
        unsafe {
            base64::prelude::BASE64_URL_SAFE
                .decode_slice_unchecked(self.get_id_bytes(), &mut decoded)
                .unwrap_unchecked();
            Self::Original::try_from_id_bytes(decoded).unwrap_unchecked()
        }
    }

    /// Build this from the original ID. [`FeagiIDBase64::encode_original`] gives the text bytes
    fn from_original(original: &Self::Original) -> Self;

    /// Borrow the padded URL-safe base64 text.
    fn as_str(&self) -> &str {
        // SAFETY: the trait contract says the ID bytes are base64 text, and that alphabet is
        // ASCII (`A-Z`, `a-z`, `0-9`, `-`, `_`, `=`).
        unsafe { core::str::from_utf8_unchecked(self.get_id_bytes()) }
    }

    /// Parse a padded URL-safe base64 ID.
    ///
    /// The text must be the canonical encoding of exactly `ID_BYTE_COUNT` bytes, and those bytes
    /// must be a valid [`FeagiIDBase64::Original`].
    fn try_from_str(text: &str) -> Result<Self, FeagiIDBase64Error<<Self::Original as FeagiID<ID_BYTE_COUNT>>::DeserializationError>> {
        let original = Self::decode_original(text.as_bytes())?;
        Ok(Self::from_original(&original))
    }

    /// Parse a [`heapless::String`] holding a padded URL-safe base64 ID.
    fn try_from_heapless<const CAP: usize>(text: &heapless::String<CAP>) -> Result<Self, FeagiIDBase64Error<<Self::Original as FeagiID<ID_BYTE_COUNT>>::DeserializationError>> {
        Self::try_from_str(text.as_str())
    }

    /// Encode `original` as padded URL-safe base64 text bytes.
    fn encode_original(original: &Self::Original) -> [u8; BASE_64_BYTE_COUNT] {
        let _: () = Self::BASE64_LEN_MATCHES_SOURCE;
        let mut text = [0u8; BASE_64_BYTE_COUNT];
        // BASE_64_BYTE_COUNT is the padded length, so this write fills `text` with ASCII.
        _ = base64::prelude::BASE64_URL_SAFE
            .encode_slice(original.get_id_bytes(), &mut text)
            .unwrap_or_else(|_| panic!("Failed to encode base64 ID into BASE_64_BYTE_COUNT"));
        text
    }

    /// Decode padded URL-safe base64 text into the original ID.
    ///
    /// Implementors can use this as the check in their `try_from_id_bytes`.
    fn decode_original(text: &[u8]) -> Result<Self::Original, FeagiIDBase64Error<<Self::Original as FeagiID<ID_BYTE_COUNT>>::DeserializationError>> {
        let _: () = Self::BASE64_LEN_MATCHES_SOURCE;
        if text.len() != BASE_64_BYTE_COUNT {
            return Err(FeagiIDBase64Error::WrongLength {
                actual: text.len(),
                expected: BASE_64_BYTE_COUNT,
            });
        }

        let mut decoded = [0u8; ID_BYTE_COUNT];
        let decoded_len = base64::prelude::BASE64_URL_SAFE
            .decode_slice(text, &mut decoded)
            .map_err(|_| FeagiIDBase64Error::InvalidEncoding)?;
        // A string of the padded length can still decode to fewer bytes (for example "AA=="
        // is 4 characters, the same width as a 3-byte ID). Only an exact decode is canonical.
        if decoded_len != ID_BYTE_COUNT {
            return Err(FeagiIDBase64Error::InvalidEncoding);
        }

        Ok(Self::Original::try_from_id_bytes(decoded)?)
    }
}

/// Deserialize a [`FeagiIDBase64`] from its padded URL-safe base64 text. Used by the code
/// `feagi_id!` generates
#[doc(hidden)]
pub fn deserialize_base_64<'de, D, B, const ID_BYTE_COUNT: usize, const BASE_64_BYTE_COUNT: usize>(
    deserializer: D,
) -> Result<B, D::Error>
where
    D: serde::Deserializer<'de>,
    B: FeagiIDBase64<ID_BYTE_COUNT, BASE_64_BYTE_COUNT>,
{
    struct Base64Visitor<B, const ID_BYTE_COUNT: usize, const BASE_64_BYTE_COUNT: usize>(core::marker::PhantomData<B>);

    impl<'de, B: FeagiIDBase64<ID_BYTE_COUNT, BASE_64_BYTE_COUNT>, const ID_BYTE_COUNT: usize, const BASE_64_BYTE_COUNT: usize>
        serde::de::Visitor<'de> for Base64Visitor<B, ID_BYTE_COUNT, BASE_64_BYTE_COUNT>
    {
        type Value = B;

        fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            formatter.write_str("a padded URL-safe base64 ID")
        }

        // Borrowed and owned strings both forward here by default
        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
            B::try_from_str(value).map_err(E::custom)
        }
    }

    deserializer.deserialize_str(Base64Visitor::<B, ID_BYTE_COUNT, BASE_64_BYTE_COUNT>(core::marker::PhantomData))
}

/// Failure to parse a [`FeagiIDBase64`] from text.
#[derive(Debug, Error)]
pub enum FeagiIDBase64Error<E: core::error::Error> {
    /// The text was not the padded URL-safe length for this ID.
    #[error("base64 ID text length {actual} did not match padded length {expected}")]
    WrongLength { actual: usize, expected: usize }, // TODO make this a u8 (u16?)

    /// The text was not a canonical padded URL-safe encoding of this ID's byte length.
    #[error("base64 ID text was not a canonical padded URL-safe encoding of this ID")]
    InvalidEncoding,

    /// The decoded bytes were not a valid ID and failed the ID's internal check
    #[error(transparent)]
    Id(#[from] E),
}
