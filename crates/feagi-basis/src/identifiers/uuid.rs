use crate::identifiers::byte_id::ByteID;
use delegate::delegate;
use thiserror::Error;
// TODO common macro block

#[macro_export]
macro_rules! create_wrapped_uuid {
    ($struct_name:ident, v4) => {
        #[derive(Copy, Clone, Debug, Hash, serde::Serialize, serde::Deserialize)]
        pub struct $struct_name(uuid::Uuid);

        impl $struct_name {

            #[cfg(feature = "std")]
            pub fn new_std() -> Self {
                Self(uuid::Uuid::new_v4())
            }

            pub fn new_with_rng<E: core::error::Error>
            (mut fill_random: impl FnMut(&mut [u8]) -> Result<(), E>)
             -> Result<Self, E> {
                let mut bytes = [0u8; 16];
                fill_random(&mut bytes)?;
                Ok(Self(
                    uuid::Builder::from_random_bytes(bytes).into_uuid()
                ))
            }
        }

        impl WrappedUuid for $struct_name {
            fn as_uuid(&self) -> &uuid::Uuid {
                &self.0
            }

            fn into_uuid(&elf) -> uuid::Uuid {
                self.0
            }
        }

        impl ByteID<16> for $struct_name {
            type DeserializationError: core::error::Error;

            fn get_id_bytes(&self) -> &[u8; NUMBER_OF_BYTES] {
                self.as_uuid().as_bytes();
            }

            fn try_from_id_bytes(bytes: [u8; NUMBER_OF_BYTES]) -> Result<Self, Self::DeserializationError> {
                todo!()
            }
        }

        base_64_id!($struct_name, 16);
    };

    ($struct_name:ident, v7) => {
        #[derive(Copy, Clone, Debug, Hash, serde::Serialize, serde::Deserialize)]
        pub struct $struct_name(uuid::Uuid);

        impl $struct_name {
            #[cfg(feature = "std")]
            pub fn new_now() -> Self {
                Self(uuid::Uuid::now_v7())
            }

            // TODO no-std safe block

        }

        impl WrappedUuid for $struct_name {
            fn as_uuid(&self) -> &uuid::Uuid {
                &self.0
            }

            fn into_uuid(&elf) -> uuid::Uuid {
                self.0
            }
        }

        impl ByteID<16> for $struct_name {
            type DeserializationError: core::error::Error;

            fn get_id_bytes(&self) -> &[u8; NUMBER_OF_BYTES] {
                self.as_uuid().as_bytes();
            }

            fn try_from_id_bytes(bytes: [u8; NUMBER_OF_BYTES]) -> Result<Self, Self::DeserializationError> {
                todo!()
            }
        }

        base_64_id!($struct_name, 16);
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