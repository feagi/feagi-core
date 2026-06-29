

// Note this right now is very similar to IndexCount, but this will differ with time

use crate::values::quantizable::QuantizedElementBase;

/// Trait designed to hold uint data values in a quantized form
pub trait QuantizedUnsignedIntegerTrait:
Copy
+ Clone
+ Send
+ Sync
+ Default
+ core::ops::Add<Output = Self>
+ core::ops::Sub<Output = Self>
+ core::ops::Mul<Output = Self>
+ core::ops::Div<Output = Self>
+ core::ops::AddAssign
+ core::ops::SubAssign
+ core::ops::MulAssign
+ core::ops::DivAssign
+ core::cmp::PartialOrd
+ core::fmt::Debug
+ core::fmt::Display
+ core::ops::Rem<Output = Self>
+ core::ops::RemAssign
+ core::cmp::Eq
+ core::hash::Hash
+ Sized
+ 'static
+ QuantizedElementBase
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