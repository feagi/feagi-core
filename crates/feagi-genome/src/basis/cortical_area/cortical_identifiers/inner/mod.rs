pub mod inner_random;

use crate::basis::cortical_area::cortical_identifiers::{CorticalIDType, CorticalIdentifier, CorticalIdentifierParseError, FauxCorticalIdentifier};
use crate::basis::cortical_area::CorticalType;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use core::fmt::Debug;
use core::hash::Hash;
use crate::basis::cortical_area::cortical_identifiers::inner::random_byte_generator::CorticalIDInnerRNG;

/// Defines how the CorticalIdentifierInner is encoded to bytes
pub trait CorticalIdentifierInnerEncoder: Sized + Clone + Debug + Hash{

    /// What data itself can be encoded and decoded
    type ContainedData: Clone + Debug + Serialize + DeserializeOwned + PartialEq;
    
    /// Encode the encodable data of the inner ID
    fn encode_data<RNG: CorticalIDInnerRNG>(
        data: Self::ContainedData,
        rng: &RNG,
    ) -> Result<CorticalIdentifierInner<Self>, CorticalIdentifierInnerParseError>;

    /// Extract the decodable data. Must never fail
    fn decode_data(inner: CorticalIdentifierInner<Self>) -> Self::ContainedData;

    /// Validates that a set of bytes is of this generated inner type
    fn validate_from_bytes(bytes: &[u8; 6]) -> Result<(), CorticalIdentifierInnerParseError>;
}

/// Represents the inner 6 bytes of a `CorrticalIdentifier` for a given type
#[derive(Clone, Copy, PartialEq, Hash, Debug, Eq)]
pub struct CorticalIdentifierInner<Encoder: CorticalIdentifierInnerEncoder> {
    bytes: [u8; 6],
    _p: core::marker::PhantomData<Encoder>,
}

impl<Encoder: CorticalIdentifierInnerEncoder> CorticalIdentifierInner<Encoder> {
    /// Try to init from internal data and any given context (systems, not data)
    pub fn try_new_from_data<RNG: CorticalIDInnerRNG>(
        data: Encoder::ContainedData,
        rng: &RNG,
    ) -> Result<Self, CorticalIdentifierInnerParseError> {
        Encoder::encode_data(data, rng)
    }

    /// Unwrap into whatever data this was encoding
    pub fn unwrap_as_data(self) -> Encoder::ContainedData {
        Encoder::decode_data(self)
    }

    /// Try to init from bytes, after ensuring validity
    pub fn try_new_from_bytes(bytes: [u8; 6]) -> Result<Self, CorticalIdentifierInnerParseError> {
        Encoder::validate_from_bytes(&bytes)?;
        Ok(Self {
            bytes,
            _p: core::marker::PhantomData,
        })
    }

    /// Unwrap self exposing inner bytes
    pub fn unwrap_as_bytes(self) -> [u8; 6] {
        self.bytes
    }

    /// Copy internal bytes to the given byte slice
    pub fn copy_to_bytes(&self, target: &mut [u8; 6]) {
        *target = self.bytes.clone()
    }

    /// Rerolls the RNG portion
    pub fn reroll_inner<RNG: CorticalIDInnerRNG>(self, rng: &RNG) -> Self {
        let preserved = self.unwrap_as_data();
        Self::try_new_from_data(preserved, rng).unwrap() // always safe
    }

    //region Identifiers

    /// Try to init cortical identifier from internal data and any given context (systems, not data)
    pub fn try_new_cortical_identifier_from_data<CT, RNG>(
        data: Encoder::ContainedData,
        rng: &RNG,
    ) -> Result<CorticalIdentifier, CorticalIdentifierParseError>
    where
        CT: CorticalType<InnerCorticalIDEncoder=Encoder>,
        RNG: CorticalIDInnerRNG
    {
        let inner = Self::try_new_from_data(data, rng)?;
        Ok(CT::wrap_cortical_identifier_inner(inner))
    }

    /// Try to init cortical identifier from bytes, after ensuring validity
    pub fn try_new_cortical_identifier_from_bytes<CT>(
        bytes: [u8; 6]
    ) -> Result<CorticalIdentifier, CorticalIdentifierParseError>
    where
        CT: CorticalType<InnerCorticalIDEncoder=Encoder>
    {
        let inner = Self::try_new_from_bytes(bytes)?;
        Ok(CT::wrap_cortical_identifier_inner(inner))
    }


    /// Try to init faux cortical identifier from internal data and any given context (systems, not data)
    fn try_new_faux_cortical_identifier_from_data<CT, RNG>(
        data: Encoder::ContainedData,
        rng: &RNG,
    ) -> Result<FauxCorticalIdentifier, CorticalIdentifierParseError>
    where
        CT: CorticalType<InnerCorticalIDEncoder=Encoder>,
        RNG: CorticalIDInnerRNG
    {
        let inner = Self::try_new_from_data(data, rng)?;
        Ok(CT::wrap_faux_cortical_identifier_inner(inner))
    }

    /// Try to init faux cortical identifier from bytes, after ensuring validity
    fn try_new_faux_cortical_identifier_from_bytes<CT>(
        bytes: [u8; 6]
    ) -> Result<FauxCorticalIdentifier, CorticalIdentifierParseError>
    where
        CT: CorticalType<InnerCorticalIDEncoder=Encoder>
    {
        let inner = Self::try_new_from_bytes(bytes)?;
        Ok(CT::wrap_faux_cortical_identifier_inner(inner))
    }

    //endregion
}

impl<Encoder: CorticalIdentifierInnerEncoder> Serialize for CorticalIdentifierInner<Encoder> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.bytes.serialize(serializer)
    }
}

/// Deserializes through the encoder so the bytes are verified
impl<'de, Encoder: CorticalIdentifierInnerEncoder> Deserialize<'de> for CorticalIdentifierInner<Encoder> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bytes = <[u8; 6]>::deserialize(deserializer)?;
        CorticalIdentifierInner::try_new_from_bytes(bytes).map_err(serde::de::Error::custom)
    }
}


/// If there is a collision with an ID in the genome, what should we do?
pub enum CorticalIdentifierCollisionPolicy {
    /// Fail Immediately
    Reject,
    /// Retry up to this many times. Errors if we fail this many times
    RetryNumberTimes(u8),
}

#[derive(Debug, thiserror::Error)]
pub enum CorticalIdentifierInnerParseError {
    #[error("Invalid inner parameters to encode! {0}")]
    InvalidEncodingParameters(&'static str),
}

/// How RNG is handled
pub mod random_byte_generator {

    /// How RNG is produced
    pub trait CorticalIDInnerRNG {
        fn random_on_slice<const N: usize>(&self, bytes: &mut [u8; N]);
    }

    #[cfg(feature = "std")]
    /// RNG using standard STD Rand
    pub struct CorticalIDInnerRNGSTD;

    #[cfg(feature = "std")]
    impl CorticalIDInnerRNG for CorticalIDInnerRNGSTD {
        fn random_on_slice<const N: usize>(&self, bytes: &mut [u8; N]) {
            *bytes = rand::random()
        }
    }

    // TODO embedded?


}