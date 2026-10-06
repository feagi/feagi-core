use core::fmt::Debug;
use serde::{Deserialize, Serialize};
use feagi_basis_quantization::prelude::FeagiIndexQuantization;

/// What version of serialization does this data represent
const PROTOCOL_VERSION: u8 = 4;

/// Implements functions needed for a data struct (real time data)
/// to be serialized and deserialized
pub trait FeagiDataSerializableQuantized<'de, FIQ: FeagiIndexQuantization>: Debug + Serialize + Deserialize<'de> {
    /// What version of serialization does this data represent. Do not override! MUST be included
    /// as a version field in the given struct!
    const PROTOCOL_VERSION: u8 = PROTOCOL_VERSION;

    // TODO at this time we dont want to spent time creating new structs to keep this
    fn get_protocol_version(&self) -> u8 {
        Self::PROTOCOL_VERSION
    }
    
    
    // TODO error handling

    /// with zero allocation, get a view of self as a byte slice. Buffer must be big enough!
    fn as_slice<'a>(&'a self, buffer: &'a mut [u8]) -> &'a [u8] {
        postcard::to_slice(self, buffer).unwrap()
    }

    /// Without allocation, deserialize bytes directly into this existing value.
    fn inplace_load_from_bytes(&mut self, bytes: &'de [u8]) -> Result<(), postcard::Error> {
        let mut deserializer = postcard::Deserializer::from_bytes(bytes);
        <Self as Deserialize<'de>>::deserialize_in_place(&mut deserializer, self)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    fn as_new_bytes(&self) -> Vec<u8> {
        postcard::to_stdvec(self).unwrap()
    }
}













