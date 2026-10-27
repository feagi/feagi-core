
use thiserror::Error;

/// Defines a 16-byte ID that holds a UUID of a single version.
/// ```text
/// impl_uuid_id!(
///     /// Docs for the generated struct.
///     pub MyId,
///     v7
/// );
/// ```
#[macro_export]
macro_rules! impl_uuid_id {
    (
        $(#[$meta:meta])*
        $vis:vis $id_name:ident, v4
    ) => {
        $crate::impl_uuid_id! {
            @common
            $(#[$meta])*
            $vis $id_name, ::uuid::Version::Random
        }

        impl $id_name {
            /// Builds an ID from a new random v4 UUID.
            #[cfg(feature = "std")]
            pub fn new_random() -> Self {
                let mut bytes = [0u8; 16];
                rand::fill(&mut bytes[..]);
                Self::new_from_random_bytes(bytes)
            }

            /// Builds a v4 UUID from `random_bytes`, overwriting its version and variant bits.
            pub const fn new_from_random_bytes(random_bytes: [u8; 16]) -> Self {
                Self::from_bytes_const(::uuid::Builder::from_random_bytes(random_bytes).into_uuid().into_bytes())
            }
        }
    };

    (
        $(#[$meta:meta])*
        $vis:vis $id_name:ident, v7
    ) => {
        $crate::impl_uuid_id! {
            @common
            $(#[$meta])*
            $vis $id_name, ::uuid::Version::SortRand
        }

        impl $id_name {
            /// Builds an ID from a new time-ordered v7 UUID.
            #[cfg(feature = "std")]
            pub fn new_now() -> Self {
                Self::from_bytes_const(::uuid::Uuid::now_v7().into_bytes())
            }

            /// Builds a v7 UUID from a unix timestamp in milliseconds and 10 counter / random bytes.
            pub const fn new_from_unix_millis(unix_millis: u64, counter_random_bytes: &[u8; 10]) -> Self {
                Self::from_bytes_const(
                    ::uuid::Builder::from_unix_timestamp_millis(unix_millis, counter_random_bytes)
                        .into_uuid()
                        .into_bytes(),
                )
            }
        }
    };

    (
        @common
        $(#[$meta:meta])*
        $vis:vis $id_name:ident, $version:path
    ) => {
        $crate::impl_byte_array_id!(
            $(#[$meta])*
            $vis $id_name,
            16,
            $crate::identifiers::UuidVersionError,
            $id_name::try_from_uuid_bytes
        );

        impl $id_name {
            /// Wraps `uuid` when its version matches this ID.
            pub const fn from_uuid(uuid: ::uuid::Uuid) -> Result<Self, $crate::feagi_id_prev::uuid::UuidVersionError> {
                match uuid.get_version() {
                    Some($version) => Ok(Self::from_bytes_const(uuid.into_bytes())),
                    actual => Err($crate::feagi_id_prev::uuid::UuidVersionError { expected: $version, actual }),
                }
            }

            /// Reads this ID as a UUID.
            pub const fn as_uuid(&self) -> ::uuid::Uuid {
                ::uuid::Uuid::from_bytes(self.0)
            }

            fn try_from_uuid_bytes(bytes: [u8; 16]) -> Result<Self, $crate::feagi_id_prev::uuid::UuidVersionError> {
                Self::from_uuid(::uuid::Uuid::from_bytes(bytes))
            }
        }
    };
}

/// The UUID bytes of a [`impl_uuid_id!`] ID were not the version that ID holds.
#[derive(Debug, Error)]
#[error("UUID version {actual:?} did not match expected version {expected:?}")]
pub struct UuidVersionError {
    pub expected: uuid::Version,
    pub actual: Option<uuid::Version>,
}