use crate::feagi_id::feagi_id::{FeagiID, HasBase64};
use base64::Engine;
use thiserror::Error;

// TODO use more optimized base64 encoding / decoding engines via cfg flags

/// The padded URL-safe base64 form of a [`HasBase64`] ID.
///
/// The ID bytes of this type are the base64 text itself, so they are always ASCII. Rust does not
/// allow blanket impls of foreign traits over this trait, so each implementor writes its own
/// `FromStr`, `TryFrom<&str>`, `TryFrom<&heapless::String<CAP>>`, `AsRef<str>`, `Display`,
/// `Serialize`, `Deserialize`, and FEAGI message path impls by forwarding to the methods here
/// and to [`deserialize_base_64`].
pub trait FeagiIDBase64<const ID_BYTE_COUNT: usize, const BASE_64_BYTE_COUNT: usize>: FeagiID<BASE_64_BYTE_COUNT> {

    /// Fails when the two length consts disagree.
    ///
    /// Padded URL-safe base64 uses 4 characters for every 3 source bytes, rounded up.
    const BASE64_LEN_MATCHES_SOURCE: () = assert!(
        BASE_64_BYTE_COUNT == (ID_BYTE_COUNT + 2) / 3 * 4,
        "BASE_64_BYTE_COUNT must be the padded URL-safe base64 length of ID_BYTE_COUNT"
    );

    type Original: HasBase64<ID_BYTE_COUNT, BASE_64_BYTE_COUNT, Base64Type=Self>;

    /// Decode this base64 ID back into the original ID
    fn to_original(&self) -> Self::Original {
        let _: () = Self::BASE64_LEN_MATCHES_SOURCE;
        let mut decoded = [0u8; ID_BYTE_COUNT];
        // SAFETY: a FeagiIDBase64 only holds the canonical encoding of a valid Original, so the
        // text decodes to exactly ID_BYTE_COUNT bytes and those bytes pass the Original's check.
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
        // SAFETY: the ID bytes of a FeagiIDBase64 are base64 text, and that alphabet is ASCII
        // (`A-Z`, `a-z`, `0-9`, `-`, `_`, `=`).
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

    /// Write the padded URL-safe base64 form of `original` into `f`.
    ///
    /// This lets the original ID implement `Display` without building this type.
    fn fmt_original(original: &Self::Original, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let text = Self::encode_original(original);
        f.write_str(core::str::from_utf8(&text).expect("base64 alphabet is ASCII"))
    }
}

// impl core::str::FromStr, TryFrom<&str>, TryFrom<&heapless::String, core::convert::AsRef<str>
// core::fmt::Display, crate::feagi_message::feagi_message_path::parameters::FeagiMessagePathParameterElement,
// crate::feagi_message::feagi_message_path::configurables::FeagiMessagePathStaticElement

/// Deserialize a [`FeagiIDBase64`] from padded URL-safe base64 text.
///
/// Implementors call this from `serde::Deserialize::deserialize`. `Serialize` is
/// `serializer.serialize_str(self.as_str())`.
pub fn deserialize_base_64<'de, D, T, const ID_BYTE_COUNT: usize, const BASE_64_BYTE_COUNT: usize>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: FeagiIDBase64<ID_BYTE_COUNT, BASE_64_BYTE_COUNT>,
{
    struct Base64Visitor<T, const ID_BYTE_COUNT: usize, const BASE_64_BYTE_COUNT: usize>(core::marker::PhantomData<T>);

    impl<'de, T: FeagiIDBase64<ID_BYTE_COUNT, BASE_64_BYTE_COUNT>, const ID_BYTE_COUNT: usize, const BASE_64_BYTE_COUNT: usize> serde::de::Visitor<'de>
        for Base64Visitor<T, ID_BYTE_COUNT, BASE_64_BYTE_COUNT>
    {
        type Value = T;

        fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            formatter.write_str("a padded URL-safe base64 ID")
        }

        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
            T::try_from_str(value).map_err(E::custom)
        }

        // Owned input cannot be borrowed. Copying it needs an allocator.
        #[cfg(feature = "alloc")]
        fn visit_string<E: serde::de::Error>(self, value: alloc::string::String) -> Result<Self::Value, E> {
            self.visit_str(&value)
        }
    }

    deserializer.deserialize_str(Base64Visitor::<T, ID_BYTE_COUNT, BASE_64_BYTE_COUNT>(core::marker::PhantomData))
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
