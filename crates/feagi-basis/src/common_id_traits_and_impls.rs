//! Common traits and extensions used throughout IDs in the Feagi Crates

use base64::Engine;

// TODO use more optimized base64 encoding / decoding engines via cfg flags

// TODO General ID trait? We need to be cautious of byte IDs vs UUIDs (should we?)

//region ByteID

/// An ID in feagi that is internally a fixed array of bytes
pub trait ByteID<const NUMBER_OF_BYTES: usize>: Sized + Clone + core::hash::Hash
+ core::fmt::Debug + core::fmt::Display + serde::Serialize + serde::de::DeserializeOwned {
    type DeserializationError: core::error::Error;

    /// Get the bytes of the encoding ID
    fn get_id_bytes(&self) -> &[u8; NUMBER_OF_BYTES];

    /// Try to parse the given bytes into the ID here
    fn try_from_id_bytes(bytes: [u8; NUMBER_OF_BYTES]) -> Result<Self, Self::DeserializationError>;
}

//region Base64 encode decode

/// Byte length of a padded URL-safe base64 encoding of `source_bytes` bytes.
pub const fn padded_url_safe_base64_len(source_bytes: usize) -> usize {
    (source_bytes + 2) / 3 * 4
}

#[macro_export]
macro_rules! base_64_id {
    ($id_name:ident, $number_of_bytes:expr) => {
        paste::paste!{
            pub type [<Base64Encoded $id_name>] = Base64EncodedByteID<$id_name, $number_of_bytes, { $crate::common_id_traits_and_impls::padded_url_safe_base64_len($number_of_bytes) }>;
        }
    };
}

#[derive(Debug, Clone, Hash)]
pub struct Base64EncodedByteID<
    ID: ByteID<NUMBER_OF_SOURCE_BYTES>,
    const NUMBER_OF_SOURCE_BYTES: usize,
    const NUMBER_BASE_64_BYTES: usize, // Have to do this BS due to limitations with rust currently
> {
    base_64_encoded_str: heapless::String<{ NUMBER_BASE_64_BYTES }>,
    _p: core::marker::PhantomData<ID>,
}

impl<ID: ByteID<NUMBER_OF_SOURCE_BYTES>, const NUMBER_OF_SOURCE_BYTES: usize, const NUMBER_BASE_64_BYTES: usize>
    Base64EncodedByteID<ID, NUMBER_OF_SOURCE_BYTES, NUMBER_BASE_64_BYTES>
{
    /// Fails when the two length consts disagree
    const BASE64_LEN_MATCHES_SOURCE: () = assert!(
        NUMBER_BASE_64_BYTES == padded_url_safe_base64_len(NUMBER_OF_SOURCE_BYTES),
        "NUMBER_BASE_64_BYTES must be padded_url_safe_base64_len(NUMBER_OF_SOURCE_BYTES)"
    );

    /// Decode this URL-safe base64 ID back into `ID`
    pub fn try_into_id(self) -> Result<ID, ID::DeserializationError> {
        let _ = Self::BASE64_LEN_MATCHES_SOURCE;

        let bytes = self.base_64_encoded_str.as_bytes();
        let mut arr = [0u8; NUMBER_OF_SOURCE_BYTES];
        // The stored text is a canonical encoding of exactly NUMBER_OF_SOURCE_BYTES, so this fills `arr`.
        _ = base64::prelude::BASE64_URL_SAFE
            .decode_slice(bytes, &mut arr)
            .unwrap_or_else(|_| panic!("Base64EncodedByteID could not be decoded as padded URL-safe base64 of NUMBER_OF_SOURCE_BYTES"));
        ID::try_from_id_bytes(arr)
    }
}

impl<ID: ByteID<NUMBER_OF_SOURCE_BYTES>, const NUMBER_OF_SOURCE_BYTES: usize, const NUMBER_BASE_64_BYTES: usize> From<ID>
    for Base64EncodedByteID<ID, NUMBER_OF_SOURCE_BYTES, NUMBER_BASE_64_BYTES>
{
    fn from(value: ID) -> Self {
        let _ = Self::BASE64_LEN_MATCHES_SOURCE;

        let mut arr = [0u8; NUMBER_BASE_64_BYTES];
        // NUMBER_BASE_64_BYTES is the padded length, so this write fills `arr` with ASCII.
        _ = base64::prelude::BASE64_URL_SAFE
            .encode_slice(value.get_id_bytes(), &mut arr)
            .unwrap_or_else(|_| panic!("Failed to encode base64 ID into NUMBER_BASE_64_BYTES"));
        let vec = heapless::Vec::from_array(arr);
        Self {
            // SAFETY: the buffer length is the padded URL-safe encoding length, so every byte
            // was written by BASE64_URL_SAFE. That alphabet is ASCII (`A-Z`, `a-z`, `0-9`, `-`, `_`, `=`).
            base_64_encoded_str: unsafe { heapless::String::from_utf8_unchecked(vec) },
            _p: core::marker::PhantomData,
        }
    }
}

//endregion

//endregion
