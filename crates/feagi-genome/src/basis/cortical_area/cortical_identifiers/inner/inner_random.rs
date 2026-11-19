use crate::basis::cortical_area::cortical_identifiers::inner::CorticalIdentifierCollisionPolicy;
use crate::basis::cortical_area::cortical_identifiers::inner::{
    CorticalIdentifierInner, CorticalIdentifierInnerEncoder, CorticalIdentifierInnerParseError,
};


// TODO take a external custom seed / rng for compatibility with no-std!

/// Encodes to the 6 bytes at random. Retries up to eight times on collision. Used by many areas.
/// Does not contain any relevant data
#[derive(Clone, Copy, PartialEq, Hash, Debug, Eq)]
pub struct CorticalIdentifierInnerEncoderRandom;

impl CorticalIdentifierInnerEncoder for CorticalIdentifierInnerEncoderRandom {
    type ContainedData = ();
    type ContextRequiredToEncode = ();
    const COLLISION_RETRY_POLICY: CorticalIdentifierCollisionPolicy =
        CorticalIdentifierCollisionPolicy::RetryNumberTimes(8); // Did you get the broom closet ending? The broom closet ending was my favorite!

    fn encode_data(
        _data: Self::ContainedData,
        _context: &Self::ContextRequiredToEncode,
    ) -> Result<CorticalIdentifierInner<Self>, CorticalIdentifierInnerParseError> {
        let bytes: [u8; 6] = rand::random();
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

