use crate::feagi_id::base_64::FeagiIDBase64;

/// An ID in feagi that is internally a fixed array of bytes
pub trait FeagiID<const ID_BYTE_COUNT: usize>: Copy + Eq + core::hash::Hash + core::fmt::Debug
+ serde::Serialize + serde::de::DeserializeOwned + core::fmt::Display {
    type DeserializationError: core::error::Error;

    /// Get the bytes of the encoding ID
    fn get_id_bytes(&self) -> &[u8; ID_BYTE_COUNT];

    /// Return the inner byte array, consuming self
    fn into_id_bytes(self) -> [u8; ID_BYTE_COUNT];

    /// Try to parse the given bytes into the ID here
    fn try_from_id_bytes(bytes: [u8; ID_BYTE_COUNT]) -> Result<Self, Self::DeserializationError>;
}


// region Extensions

pub trait HasBase64<const ID_BYTE_COUNT: usize, const BASE_64_BYTE_COUNT: usize>: FeagiID<ID_BYTE_COUNT>
{
    type Base64Type: FeagiIDBase64<ID_BYTE_COUNT, BASE_64_BYTE_COUNT,Original=Self>;

    fn from_base_64(b64: &Self::Base64Type) -> Self {
        b64.to_original()
    }
    
    /// Turns self into the base 64 representation
    fn to_base_64(&self) -> Self::Base64Type {
        Self::Base64Type::from_original(&self)
    }
}



//endregion