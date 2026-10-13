//! Common traits and extensions used throughout IDs in the Feagi Crates

use heapless;

// TODO General ID trait? We need to be cautious of byte IDs vs UUIDs (should we?)





//region ByteID

/// An ID in feagi that is internally a fixed array of bytes
pub trait ByteID<const NUMBER_OF_BYTES: usize>: Sized
{
    type DeserializationError;

    /// Get the bytes of the encoding ID
    fn get_id_bytes(&self) -> &[u8; NUMBER_OF_BYTES];

    /// Try to parse the given bytes into the ID here
    fn try_from_id_bytes(bytes: &[u8; NUMBER_OF_BYTES]) -> Result<Self, Self::DeserializationError>;
}

//region Base64 encode decode

pub struct Base64EncodedByteID<
    ID: ByteID<NUMBER_OF_SOURCE_BYTES>,
    const NUMBER_OF_SOURCE_BYTES: usize,
    const NUMBER_BASE_64_BYTES: usize, // Have to do this BS due to limitations with rust currently
>
{
    base_64_encoded_str: heapless::String<{ NUMBER_OF_SOURCE_BYTES / 3 }>,
    _p: core::marker::PhantomData<ID>
}

impl<ID: ByteID<NUMBER_OF_SOURCE_BYTES>, const NUMBER_OF_SOURCE_BYTES: usize>
Base64EncodedByteID<ID, NUMBER_OF_SOURCE_BYTES>
{
    pub const NUMBER_BASE_64_BYTES: usize = (NUMBER_OF_SOURCE_BYTES / 3) * 4; // TODO we need to account for padding!
}

impl<ID: ByteID<NUMBER_OF_SOURCE_BYTES>, const NUMBER_OF_SOURCE_BYTES: usize> From<ID> for Base64EncodedByteID<ID, NUMBER_OF_SOURCE_BYTES> {
    fn from(value: ID) -> Self {
        todo!()
    }
}

//endregion

//endregion







