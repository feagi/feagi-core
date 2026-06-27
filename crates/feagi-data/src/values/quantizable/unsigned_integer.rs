

// Note this right now is very similar to IndexCount, but this will differ with time

use crate::values::quantizable::shared_traits::{QuantizedElementBase, SupportsUintOps};

/// Trait designed to hold uint data values in a quantized form
pub trait QuantizedUnsignedIntegerTrait: QuantizedElementBase
+ SupportsUintOps
{}

impl QuantizedUnsignedIntegerTrait for usize {
}

impl QuantizedUnsignedIntegerTrait for u8 {
}

impl QuantizedUnsignedIntegerTrait for u16 {
}

// lol, lmao even
impl QuantizedUnsignedIntegerTrait for u32 {
}

impl QuantizedUnsignedIntegerTrait for u64 {
}