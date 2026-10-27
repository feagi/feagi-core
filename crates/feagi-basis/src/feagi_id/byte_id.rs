/// Defines a fixed-byte ID tuple struct and its standard surface.
///
/// The generated struct is `Name([u8; N])` with `Debug`, `Clone`, `Copy`, `PartialEq`,
/// `Eq`, `Hash`, `serde::Serialize`, and `serde::Deserialize`. The macro also adds:
///
/// - `from_bytes_const`, which wraps the bytes and does not run a deserialization check
/// - [`Identifier`] and [`ByteID`], including `try_from_id_bytes`
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

        impl $crate::feagi_id::ByteID<$number_of_bytes> for $id_name {

            type Base64Type = paste::paste! {[<Base64Encoded $id_name>]};
            type DeserializationError = $error_ty;

            fn get_id_bytes(&self) -> &[u8; $number_of_bytes] {
                &self.0
            }

            fn into_id_bytes(self) -> [u8; $number_of_bytes] {
                self.0
            }

            fn to_base_64(self) -> Self::Base64Type {
                self.into()
            }

            fn try_from_id_bytes(
                bytes: [u8; $number_of_bytes],
            ) -> Result<Self, Self::DeserializationError> {
                $crate::impl_byte_array_id!(@try_from $id_name, bytes $(, $try_from_id_bytes)?)
            }
        }

        paste::paste! {
            #[allow(clippy::manual_div_ceil)]
            pub type [<Base64Encoded $id_name>] = $crate::feagi_id::Base64EncodedByteID<
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
pub trait ByteID<const NUMBER_OF_BYTES: usize>: Copy + Eq + core::hash::Hash + core::fmt::Debug
+ serde::Serialize + serde::de::DeserializeOwned + core::fmt::Display {
    type Base64Type;
    type DeserializationError: core::error::Error;

    /// Get the bytes of the encoding ID
    fn get_id_bytes(&self) -> &[u8; NUMBER_OF_BYTES];

    /// Return the inner byte array, consuming self
    fn into_id_bytes(self) -> [u8; NUMBER_OF_BYTES];

    /// Turns self into the base 64 representation
    fn to_base_64(self) -> Self::Base64Type;

    /// Try to parse the given bytes into the ID here
    fn try_from_id_bytes(bytes: [u8; NUMBER_OF_BYTES]) -> Result<Self, Self::DeserializationError>;
}
