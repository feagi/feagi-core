use feagi_quantization::create_wrapped_quantized_unsigned_integer;
use feagi_quantization::prelude::{QuantizedUnsignedIntegerUnwrappedTrait};

create_wrapped_quantized_unsigned_integer!(
    /// Defines the burst index, the current "tick" that the burst engine is on
    pub BurstIndex
);

impl<Q: QuantizedUnsignedIntegerUnwrappedTrait> BurstIndex<Q> {
    
    /// Calculates the halfway point to the max quant value. Needed to know when to rollover
    pub fn halfway_point() -> Self {
        BurstIndex::new(Q::QUANT_MAX / (Q::QUANT_ONE + Q::QUANT_ONE))
    }
    
}