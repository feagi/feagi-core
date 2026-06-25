use crate::shared_traits::QuantizedElementBase;

/// Trait designed to hold Sint data values in a quantized form
pub trait QuantizedSignedIntegerTrait: QuantizedElementBase
{
    fn is_negative(&self) -> bool;
    fn is_zero_or_negative(&self) -> bool;
}

impl QuantizedSignedIntegerTrait for isize {
    #[inline(always)]
    fn is_negative(&self) -> bool {
        *self < 0
    }

    #[inline(always)]
    fn is_zero_or_negative(&self) -> bool {
        *self <= 0
    }
}

impl QuantizedSignedIntegerTrait for i8 {
    #[inline(always)]
    fn is_negative(&self) -> bool {
        *self < 0
    }

    #[inline(always)]
    fn is_zero_or_negative(&self) -> bool {
        *self <= 0
    }
}

impl QuantizedSignedIntegerTrait for i16 {
    #[inline(always)]
    fn is_negative(&self) -> bool {
        *self < 0
    }

    #[inline(always)]
    fn is_zero_or_negative(&self) -> bool {
        *self <= 0
    }
}

// lol, lmao even
impl QuantizedSignedIntegerTrait for i32 {
    #[inline(always)]
    fn is_negative(&self) -> bool {
        *self < 0
    }

    #[inline(always)]
    fn is_zero_or_negative(&self) -> bool {
        *self <= 0
    }
}

#[cfg(feature = "support_64bit_indexing")]
impl QuantizedSignedIntegerTrait for i64 {
    #[inline(always)]
    fn is_negative(&self) -> bool {
        *self < 0
    }

    #[inline(always)]
    fn is_zero_or_negative(&self) -> bool {
        *self <= 0
    }
}