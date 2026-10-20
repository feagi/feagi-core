//! Common traits and extensions used throughout IDs in the Feagi Crates

#[cfg(feature = "alloc")]
extern crate alloc;

use base64::Engine;
use thiserror::Error;

// TODO use more optimized base64 encoding / decoding engines via cfg flags

/// Defines a fixed-byte ID tuple struct and its standard surface.
///
/// The generated struct is `Name([u8; N])` with `Debug`, `Clone`, `Copy`, `PartialEq`,
/// `Eq`, `Hash`, `serde::Serialize`, and `serde::Deserialize`. The macro also adds:
///
/// - `from_bytes_const`, which wraps the bytes and does not run a deserialization check
/// - [`ByteID`], including `try_from_id_bytes`
/// - `Base64EncodedName`, the padded URL-safe base64 alias
/// - [`core::fmt::Display`] as that same base64 text
///
/// ```text
/// impl_byte_array_id!(
///     /// Docs for the generated struct.
///     pub MyId,
///     8
/// );
/// impl_byte_array_id!(pub MyId, 8, MyIdError, try_my_id_from_bytes);
/// ```
///
/// Without a check function, `try_from_id_bytes` returns `Ok(Self(bytes))` and
/// `DeserializationError` is [`core::convert::Infallible`], which cannot be constructed.
/// Pass an error type only together with `try_my_id_from_bytes`:
/// `fn([u8; N]) -> Result<MyId, MyIdError>`.
#[macro_export]
macro_rules! impl_byte_array_id {
    (
        $(#[$meta:meta])*
        $vis:vis $id_name:ident,
        $number_of_bytes:expr
    ) => {
        $crate::impl_byte_array_id! {
            @body
            $(#[$meta])*
            $vis $id_name,
            $number_of_bytes,
            ::core::convert::Infallible
        }
    };

    (
        $(#[$meta:meta])*
        $vis:vis $id_name:ident,
        $number_of_bytes:expr,
        $error_ty:ty,
        $try_from_id_bytes:path
    ) => {
        $crate::impl_byte_array_id! {
            @body
            $(#[$meta])*
            $vis $id_name,
            $number_of_bytes,
            $error_ty,
            $try_from_id_bytes
        }
    };

    (
        @body
        $(#[$meta:meta])*
        $vis:vis $id_name:ident,
        $number_of_bytes:expr,
        $error_ty:ty
        $(, $try_from_id_bytes:path)?
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ::serde::Serialize, ::serde::Deserialize)]
        $vis struct $id_name([u8; $number_of_bytes]);

        impl $id_name {
            /// Wraps `bytes` as this ID.
            ///
            /// This does not run deserialization checks, so it can be called from const context.
            /// Checks belong in `ByteID::try_from_id_bytes`, which base64 parsing also uses.
            pub const fn from_bytes_const(bytes: [u8; $number_of_bytes]) -> Self {
                Self(bytes)
            }
        }

        impl $crate::identifiers::byte_id::ByteID<$number_of_bytes> for $id_name {
            type DeserializationError = $error_ty;

            fn get_id_bytes(&self) -> &[u8; $number_of_bytes] {
                &self.0
            }

            fn into_id_bytes(self) -> [u8; $number_of_bytes] {
                self.0
            }

            fn try_from_id_bytes(
                bytes: [u8; $number_of_bytes],
            ) -> Result<Self, Self::DeserializationError> {
                $crate::impl_byte_array_id!(@try_from $id_name, bytes $(, $try_from_id_bytes)?)
            }
        }

        paste::paste! {
            pub type [<Base64Encoded $id_name>] = $crate::identifiers::byte_id::Base64EncodedByteID<
                $id_name,
                $number_of_bytes,
                { ($number_of_bytes + 2) / 3 * 4 },
            >;

            impl core::fmt::Display for $id_name {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    [<Base64Encoded $id_name>]::fmt_id(self, f)
                }
            }
        }
    };

    (@try_from $id_name:ident, $bytes:ident) => {
        Ok($id_name($bytes))
    };

    (@try_from $id_name:ident, $bytes:ident, $try_from_id_bytes:path) => {
        $try_from_id_bytes($bytes)
    };
}

/// Adds a `std`-only `new_random` constructor to a fixed-byte ID.
///
/// The struct must already have `from_bytes_const`. [`impl_byte_array_id!`] generates that.
///
/// ```text
/// impl_byte_array_id_random!(MyId, 8);
/// ```
///
/// `new_random` is compiled only when the crate's `std` feature is enabled. Random bytes
/// come from [`rand`] and are wrapped with `from_bytes_const`.
#[macro_export]
macro_rules! impl_byte_array_id_random {
    ($id_name:ident, $number_of_bytes:expr) => {
        impl $id_name {
            /// Builds an ID from random bytes.
            ///
            /// The bytes are not passed through `ByteID::try_from_id_bytes`.
            #[cfg(feature = "std")]
            pub fn new_random() -> Self {
                let mut bytes = [0u8; $number_of_bytes];
                rand::fill(&mut bytes[..]);
                Self::from_bytes_const(bytes)
            }
        }
    };
}

/// An ID in feagi that is internally a fixed array of bytes
pub trait ByteID<const NUMBER_OF_BYTES: usize>:
    Sized + Clone + core::hash::Hash + core::fmt::Debug + core::fmt::Display + serde::Serialize + serde::de::DeserializeOwned
{
    type DeserializationError: core::error::Error;

    /// Get the bytes of the encoding ID
    fn get_id_bytes(&self) -> &[u8; NUMBER_OF_BYTES];

    /// Return the inner byte array, consuming self
    fn into_id_bytes(self) -> [u8; NUMBER_OF_BYTES];

    /// Try to parse the given bytes into the ID here
    fn try_from_id_bytes(bytes: [u8; NUMBER_OF_BYTES]) -> Result<Self, Self::DeserializationError>;
}

//region Base64ByteId

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
    /// Fails when the two length consts disagree.
    ///
    /// Padded URL-safe base64 uses 4 characters for every 3 source bytes, rounded up.
    const BASE64_LEN_MATCHES_SOURCE: () = assert!(
        NUMBER_BASE_64_BYTES == (NUMBER_OF_SOURCE_BYTES + 2) / 3 * 4,
        "NUMBER_BASE_64_BYTES must be the padded URL-safe base64 length of NUMBER_OF_SOURCE_BYTES"
    );

    /// Borrow the padded URL-safe base64 text.
    pub fn as_str(&self) -> &str {
        self.base_64_encoded_str.as_str()
    }

    /// Write the padded URL-safe base64 form of `id` into `f`.
    ///
    /// A concrete ID knows both lengths, so `Display` can call `Base64EncodedMyId::fmt_id(self, f)`.
    pub fn fmt_id(id: &ID, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let _ = Self::BASE64_LEN_MATCHES_SOURCE;
        let mut arr = [0u8; NUMBER_BASE_64_BYTES];
        // NUMBER_BASE_64_BYTES is the padded length, so this write fills `arr` with ASCII.
        _ = base64::prelude::BASE64_URL_SAFE
            .encode_slice(id.get_id_bytes(), &mut arr)
            .unwrap_or_else(|_| panic!("Failed to encode base64 ID into NUMBER_BASE_64_BYTES"));
        let text = core::str::from_utf8(&arr).expect("base64 alphabet is ASCII");
        f.write_str(text)
    }

    /// Parse a padded URL-safe base64 ID.
    ///
    /// The text must be the canonical encoding of exactly `NUMBER_OF_SOURCE_BYTES` bytes,
    /// and those bytes must be a valid `ID`.
    pub fn try_from_str(text: &str) -> Result<Self, Base64EncodedByteIdError<ID::DeserializationError>> {
        let _ = Self::BASE64_LEN_MATCHES_SOURCE;
        if text.len() != NUMBER_BASE_64_BYTES {
            return Err(Base64EncodedByteIdError::WrongLength {
                actual: text.len(),
                expected: NUMBER_BASE_64_BYTES,
            });
        }

        let mut decoded = [0u8; NUMBER_OF_SOURCE_BYTES];
        let decoded_len = base64::prelude::BASE64_URL_SAFE
            .decode_slice(text.as_bytes(), &mut decoded)
            .map_err(|_| Base64EncodedByteIdError::InvalidEncoding)?;
        // A string of the padded length can still decode to fewer bytes (for example "AA=="
        // is 4 characters, the same width as a 3-byte ID). Only an exact decode is canonical.
        if decoded_len != NUMBER_OF_SOURCE_BYTES {
            return Err(Base64EncodedByteIdError::InvalidEncoding);
        }

        // Validate that the bytes are fine for that struct
        ID::try_from_id_bytes(decoded)?;

        let base_64_encoded_str = heapless::String::try_from(text).unwrap_or_else(|_| panic!("text length was checked against NUMBER_BASE_64_BYTES"));
        Ok(Self {
            base_64_encoded_str,
            _p: core::marker::PhantomData,
        })
    }

    /// Parse a [`heapless::String`] holding a padded URL-safe base64 ID.
    pub fn try_from_heapless<const CAP: usize>(text: &heapless::String<CAP>) -> Result<Self, Base64EncodedByteIdError<ID::DeserializationError>> {
        Self::try_from_str(text.as_str())
    }

    /// Decode this URL-safe base64 ID back into `ID`
    pub fn to_id(&self) -> ID {
        let _ = Self::BASE64_LEN_MATCHES_SOURCE;

        let mut arr = [0u8; NUMBER_OF_SOURCE_BYTES];
        base64::prelude::BASE64_URL_SAFE
            .decode_slice(self.base_64_encoded_str.as_bytes(), &mut arr)
            .expect("stored base64 ID is canonical");
        ID::try_from_id_bytes(arr).expect("stored base64 ID is a valid ID")
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

impl<ID: ByteID<NUMBER_OF_SOURCE_BYTES>, const NUMBER_OF_SOURCE_BYTES: usize, const NUMBER_BASE_64_BYTES: usize> core::str::FromStr
    for Base64EncodedByteID<ID, NUMBER_OF_SOURCE_BYTES, NUMBER_BASE_64_BYTES>
{
    type Err = Base64EncodedByteIdError<ID::DeserializationError>;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::try_from_str(text)
    }
}

impl<ID: ByteID<NUMBER_OF_SOURCE_BYTES>, const NUMBER_OF_SOURCE_BYTES: usize, const NUMBER_BASE_64_BYTES: usize> TryFrom<&str>
    for Base64EncodedByteID<ID, NUMBER_OF_SOURCE_BYTES, NUMBER_BASE_64_BYTES>
{
    type Error = Base64EncodedByteIdError<ID::DeserializationError>;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        Self::try_from_str(text)
    }
}

impl<ID: ByteID<NUMBER_OF_SOURCE_BYTES>, const NUMBER_OF_SOURCE_BYTES: usize, const NUMBER_BASE_64_BYTES: usize, const CAP: usize>
    TryFrom<&heapless::String<CAP>> for Base64EncodedByteID<ID, NUMBER_OF_SOURCE_BYTES, NUMBER_BASE_64_BYTES>
{
    type Error = Base64EncodedByteIdError<ID::DeserializationError>;

    fn try_from(text: &heapless::String<CAP>) -> Result<Self, Self::Error> {
        Self::try_from_heapless(text)
    }
}

impl<ID: ByteID<NUMBER_OF_SOURCE_BYTES>, const NUMBER_OF_SOURCE_BYTES: usize, const NUMBER_BASE_64_BYTES: usize> core::convert::AsRef<str>
    for Base64EncodedByteID<ID, NUMBER_OF_SOURCE_BYTES, NUMBER_BASE_64_BYTES>
{
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<ID: ByteID<NUMBER_OF_SOURCE_BYTES>, const NUMBER_OF_SOURCE_BYTES: usize, const NUMBER_BASE_64_BYTES: usize> core::fmt::Display
    for Base64EncodedByteID<ID, NUMBER_OF_SOURCE_BYTES, NUMBER_BASE_64_BYTES>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<ID: ByteID<NUMBER_OF_SOURCE_BYTES>, const NUMBER_OF_SOURCE_BYTES: usize, const NUMBER_BASE_64_BYTES: usize>
    crate::feagi_message::feagi_message_path::parameters::FeagiMessagePathParameterElement
    for Base64EncodedByteID<ID, NUMBER_OF_SOURCE_BYTES, NUMBER_BASE_64_BYTES>
{
    fn write_path_segment<W: core::fmt::Write>(&self, out: &mut W) -> core::fmt::Result {
        out.write_str(self.as_str())
    }
}

impl<ID: ByteID<NUMBER_OF_SOURCE_BYTES>, const NUMBER_OF_SOURCE_BYTES: usize, const NUMBER_BASE_64_BYTES: usize> serde::Serialize
    for Base64EncodedByteID<ID, NUMBER_OF_SOURCE_BYTES, NUMBER_BASE_64_BYTES>
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de, ID: ByteID<NUMBER_OF_SOURCE_BYTES>, const NUMBER_OF_SOURCE_BYTES: usize, const NUMBER_BASE_64_BYTES: usize> serde::Deserialize<'de>
    for Base64EncodedByteID<ID, NUMBER_OF_SOURCE_BYTES, NUMBER_BASE_64_BYTES>
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct EncodedIdVisitor<ID, const NUMBER_OF_SOURCE_BYTES: usize, const NUMBER_BASE_64_BYTES: usize>(core::marker::PhantomData<ID>);

        impl<'de, ID: ByteID<NUMBER_OF_SOURCE_BYTES>, const NUMBER_OF_SOURCE_BYTES: usize, const NUMBER_BASE_64_BYTES: usize> serde::de::Visitor<'de>
            for EncodedIdVisitor<ID, NUMBER_OF_SOURCE_BYTES, NUMBER_BASE_64_BYTES>
        {
            type Value = Base64EncodedByteID<ID, NUMBER_OF_SOURCE_BYTES, NUMBER_BASE_64_BYTES>;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("a padded URL-safe base64 ID")
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Base64EncodedByteID::try_from_str(value).map_err(E::custom)
            }

            // Owned input cannot be borrowed. Copying it needs an allocator.
            #[cfg(feature = "alloc")]
            fn visit_string<E: serde::de::Error>(self, value: alloc::string::String) -> Result<Self::Value, E> {
                self.visit_str(&value)
            }
        }

        deserializer.deserialize_str(EncodedIdVisitor(core::marker::PhantomData))
    }
}

/// Failure to parse a [`Base64EncodedByteID`] from text.
#[derive(Debug, Error)]
pub enum Base64EncodedByteIdError<E: core::error::Error> {
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

//endregion
