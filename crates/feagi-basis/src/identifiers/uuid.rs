use crate::identifiers::byte_id::ByteID;
use thiserror::Error;

#[doc(hidden)]
pub use uuid::{Builder, Uuid, Version};

/// Generates a version-checked UUID wrapper.
///
/// `create_wrapped_uuid!(Name, v4)` or `create_wrapped_uuid!(Name, v7)` defines
/// `Name` around a `uuid::Uuid`, with `Copy`, `Clone`, `Debug`, `Hash`, `Serialize`,
/// and `Deserialize`. The expansion implements [`WrappedUuid`](crate::identifiers::uuid::WrappedUuid),
/// [`ByteID`](crate::identifiers::byte_id::ByteID) for 16 bytes,
/// [`Display`](core::fmt::Display) as padded URL-safe base64, and the `Base64EncodedName` alias.
///
/// Doc comments and other attributes written before the name are copied onto the struct:
///
/// ```text
/// create_wrapped_uuid!(
///     /// Docs for the generated struct.
///     Name,
///     v4
/// );
/// ```
///
/// `Name::new` returns `Some` only when the UUID version matches. v4 also adds
/// `new_random` (std) and `new_with_rng`. v7 adds `new_now` (std).
#[macro_export]
macro_rules! create_wrapped_uuid {
    (
        $(#[$meta:meta])*
        $struct_name:ident, v4
    ) => {
        $crate::create_wrapped_uuid! {
            @common
            $(#[$meta])*
            $struct_name, $crate::identifiers::uuid::Version::Random,
            {
                /// Builds an ID from a new random v4 UUID.
                #[cfg(feature = "std")]
                pub fn new_random() -> Self {
                    Self($crate::identifiers::uuid::Uuid::new_v4())
                }

                /// Builds an ID from 16 bytes filled by `fill_random`.
                pub fn new_with_rng<E: ::core::error::Error>(
                    mut fill_random: impl FnMut(&mut [u8]) -> Result<(), E>,
                ) -> Result<Self, E> {
                    let mut bytes = [0u8; 16];
                    fill_random(&mut bytes)?;
                    Ok(Self(
                        $crate::identifiers::uuid::Builder::from_random_bytes(bytes).into_uuid()
                    ))
                }
            }
        }
    };

    (
        $(#[$meta:meta])*
        $struct_name:ident, v7
    ) => {
        $crate::create_wrapped_uuid! {
            @common
            $(#[$meta])*
            $struct_name, $crate::identifiers::uuid::Version::SortRand,
            {
                /// Builds an ID from a new time-ordered v7 UUID.
                #[cfg(feature = "std")]
                pub fn new_now() -> Self {
                    Self($crate::identifiers::uuid::Uuid::now_v7())
                }

                // TODO no-std safe block
            }
        }
    };

    (
        @common
        $(#[$meta:meta])*
        $struct_name:ident, $version:path, { $($constructors:tt)* }
    ) => {
        $(#[$meta])*
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
        pub struct $struct_name($crate::identifiers::uuid::Uuid);

        impl $struct_name {
            /// Wraps `uuid` when its version matches this ID.
            pub const fn new(uuid: $crate::identifiers::uuid::Uuid) -> Option<Self> {
                match uuid.get_version() {
                    Some($version) => Some(Self(uuid)),
                    _ => None,
                }
            }

            $($constructors)*
        }

        impl $crate::identifiers::uuid::WrappedUuid for $struct_name {
            fn as_uuid(&self) -> &$crate::identifiers::uuid::Uuid {
                &self.0
            }

            fn into_uuid(self) -> $crate::identifiers::uuid::Uuid {
                self.0
            }
        }

        impl $crate::identifiers::byte_id::ByteID<16> for $struct_name {
            type DeserializationError = $crate::identifiers::uuid::UuidDeserializationError;

            fn get_id_bytes(&self) -> &[u8; 16] {
                self.0.as_bytes()
            }

            fn into_id_bytes(self) -> [u8; 16] {
                self.0.into_bytes()
            }

            fn try_from_id_bytes(
                _bytes: [u8; 16],
            ) -> Result<Self, Self::DeserializationError> {
                todo!()
            }
        }

        paste::paste! {
            pub type [<Base64Encoded $struct_name>] = $crate::identifiers::byte_id::Base64EncodedByteID<
                $struct_name,
                16,
                { (16 + 2) / 3 * 4 },
            >;

            impl ::core::fmt::Display for $struct_name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    [<Base64Encoded $struct_name>]::fmt_id(self, f)
                }
            }
        }
    };
}

// TODO expand

/// Common UUID wrapper type
pub trait WrappedUuid: ByteID<16> {
    fn as_uuid(&self) -> &uuid::Uuid;

    fn into_uuid(self) -> uuid::Uuid;

    // TODO try froms?
}

#[derive(Error, Debug)]
pub enum UuidDeserializationError {}
