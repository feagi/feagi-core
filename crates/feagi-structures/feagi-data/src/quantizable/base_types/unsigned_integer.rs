use crate::core_numerical_types::{SupportsUintOps};
use crate::quantizable::base_types::QuantizedElementBase;

// Note this right now is very similar to IndexCount, but this will differ with time

/// Trait designed to hold uint data values in a quantized form
pub trait QuantizedUnsignedIntegerTrait: QuantizedElementBase
+ SupportsUintOps
{
}

impl QuantizedUnsignedIntegerTrait for usize {
}

impl QuantizedUnsignedIntegerTrait for u8 {
}

impl QuantizedUnsignedIntegerTrait for u16 {
}

// lol, lmao even
impl QuantizedUnsignedIntegerTrait for u32 {
}

#[cfg(feature = "support_64bit_indexing")]
impl QuantizedUnsignedIntegerTrait for u64 {
}