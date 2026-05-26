use crate::core_numerical_types::{SupportsBasicCoreMathOps, SupportsUintOps};
use crate::quantizable::base_types::QuantizedElementBase;

// Note this right now is very similar to IndexCount, but this will differ with time

/// Trait designed to hold uint data values in a quantized form
pub trait QuantizedUnsignedIntegerTrait: QuantizedElementBase
+ SupportsUintOps
{
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

/// Something all wrappers share, for easy data access
pub trait QuantizedUnsignedIntegerWrapperTrait<QuantIndex: QuantizedUnsignedIntegerTrait>:
Copy
+ Clone
+ core::ops::Add<Output = Self>
+ core::ops::Sub<Output = Self>
+ core::ops::Mul<Output = Self>
+ core::ops::Div<Output = Self>
+ core::ops::AddAssign
+ core::ops::SubAssign
+ core::ops::MulAssign
+ core::ops::DivAssign
+ core::cmp::PartialOrd
+ Send
+ Sync
+ 'static
{
    fn wrap_quant(quant: QuantIndex) -> Self;
    fn quant(self) -> QuantIndex;
    fn quant_ref(&self) -> &QuantIndex;
    fn quant_mut(&mut self) -> &mut QuantIndex;
}