use crate::basis::cortical_area::cortical_identifiers::inner::CorticalIdentifierCollisionPolicy;
use crate::basis::cortical_area::cortical_identifiers::inner::{
    CorticalIdentifierInner, CorticalIdentifierInnerEncoder, CorticalIdentifierInnerParseError,
};
use crate::basis::cortical_area::cortical_identifiers::inner::random_byte_generator::CorticalIDInnerRNG;

/// Encodes to the 6 bytes at random. Retries up to eight times on collision. Used by many areas.
/// Does not contain any relevant data
#[derive(Clone, Copy, PartialEq, Hash, Debug, Eq)]
pub struct CorticalIdentifierInnerEncoderRandom;

impl CorticalIdentifierInnerEncoder for CorticalIdentifierInnerEncoderRandom {
    type ContainedData = ();
    
    fn encode_data<RNG: CorticalIDInnerRNG>(
        _data: Self::ContainedData,
        rng: &RNG,
    ) -> Result<CorticalIdentifierInner<Self>, CorticalIdentifierInnerParseError> {
        let mut bytes = [0u8; 6];
        rng.random_on_slice::<6>(&mut bytes);
        Ok(CorticalIdentifierInner{bytes, _p: Default::default()}) // never fails
    }

    fn decode_data(inner: CorticalIdentifierInner<Self>) -> Self::ContainedData {
        // nothing to get
        () 
    }

    fn validate_from_bytes(bytes: &[u8; 6]) -> Result<(), CorticalIdentifierInnerParseError> {
        // Nothing to verify
        Ok(())
    }
}

