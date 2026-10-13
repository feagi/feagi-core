//! Common traits and extensions used throughout IDs in the Feagi Crates

use base64::{DecodeError, DecodeSliceError, Engine};
use heapless;
use thiserror::Error;

// TODO use more optimized base64 encoding / decoding engines via cfg flags

// TODO General ID trait? We need to be cautious of byte IDs vs UUIDs (should we?)



//region ByteID

/// An ID in feagi that is internally a fixed array of bytes
pub trait ByteID<const NUMBER_OF_BYTES: usize>: Sized
{
    type DeserializationError: core::error::Error;

    /// Get the bytes of the encoding ID
    fn get_id_bytes(&self) -> &[u8; NUMBER_OF_BYTES];

    /// Try to parse the given bytes into the ID here
    fn try_from_id_bytes(bytes: [u8; NUMBER_OF_BYTES]) -> Result<Self, Self::DeserializationError>;
}



//region Base64 encode decode

#[macro_export]
macro_rules! base_64_id {
    ($id_name:ident, $number_of_bytes:expr) => {
        paste::paste!{
            pub type [<Base64Encoded $id_name>] = Base64EncodedByteID<$id_name, $number_of_bytes, {{$number_of_bytes * 4} / 3}>;
        }
    };
}


pub struct Base64EncodedByteID<
    ID: ByteID<NUMBER_OF_SOURCE_BYTES>,
    const NUMBER_OF_SOURCE_BYTES: usize,
    const NUMBER_BASE_64_BYTES: usize, // Have to do this BS due to limitations with rust currently
>
{
    base_64_encoded_str: heapless::String<{ NUMBER_BASE_64_BYTES }>,
    _p: core::marker::PhantomData<ID>
}

impl<ID: ByteID<NUMBER_OF_SOURCE_BYTES>, const NUMBER_OF_SOURCE_BYTES: usize, const NUMBER_BASE_64_BYTES: usize>
Base64EncodedByteID<ID, NUMBER_OF_SOURCE_BYTES, NUMBER_BASE_64_BYTES>
{

}

impl<ID: ByteID<NUMBER_OF_SOURCE_BYTES>, const NUMBER_OF_SOURCE_BYTES: usize, const NUMBER_BASE_64_BYTES: usize>
From<ID> for
Base64EncodedByteID<ID, NUMBER_OF_SOURCE_BYTES, NUMBER_BASE_64_BYTES>
{
    fn from(value: ID) -> Self {
        let mut arr = [0u8; NUMBER_BASE_64_BYTES];
        _ = base64::prelude::BASE64_URL_SAFE
            .encode_slice(value.get_id_bytes(), &mut arr)
            .map_err(|e| { // This should be impossible
                panic!("Failed to decode base 64 ID as slice was too small!")
            });
        let vec = heapless::Vec::from_array(arr);
        unsafe {
            Self {
                base_64_encoded_str: heapless::String::from_utf8_unchecked(vec),
                _p: core::marker::PhantomData
            }
        }
    }
}

impl<ID: ByteID<NUMBER_OF_SOURCE_BYTES>, const NUMBER_OF_SOURCE_BYTES: usize, const NUMBER_BASE_64_BYTES: usize>
TryFrom<Base64EncodedByteID<ID, NUMBER_OF_SOURCE_BYTES, NUMBER_BASE_64_BYTES>> for
ID
{
    type Error = ID::DeserializationError;

    fn try_from(value: Base64EncodedByteID<ID, NUMBER_OF_SOURCE_BYTES, NUMBER_BASE_64_BYTES>) -> Result<Self, Self::Error> {
        let bytes = value.base_64_encoded_str.as_bytes();
        let mut arr = [0u8; NUMBER_OF_SOURCE_BYTES];
        _ = base64::prelude::BASE64_URL_SAFE.decode_slice(bytes, &mut arr)
            .map_err(|_| {
                // Since we control the creation of this struct, this should never be possible
                panic!("Base64EncodedByteID Could not be decoded as it had invalid data or was outputting to invalid length!")
            });
        ID::try_from_id_bytes(arr)
    }
}


//endregion

//endregion







