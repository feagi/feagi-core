// Note this right now is very similar to IndexCount, but this will differ with time

use crate::values::quantizable::feagi_data_value_quantization_error::FeagiFailQuantizationOutOfRange;
use crate::values::quantizable::{FeagiDataValueQuantizationError, QuantizationLevelPacking, QuantizedElementBase};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum UnsignedIntegerQuantizationLevel {
    U8 = 0,
    U16 = 1,
    U32 = 2,
    U64 = 3,
    Usize = 4,
    // We can support a max of 8
}

impl Into<u8> for UnsignedIntegerQuantizationLevel {
    fn into(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for UnsignedIntegerQuantizationLevel {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(UnsignedIntegerQuantizationLevel::U8),
            1 => Ok(UnsignedIntegerQuantizationLevel::U16),
            2 => Ok(UnsignedIntegerQuantizationLevel::U32),
            3 => Ok(UnsignedIntegerQuantizationLevel::U64),
            4 => Ok(UnsignedIntegerQuantizationLevel::Usize),
            _ => Err(()),
        }
    }
}

impl QuantizationLevelPacking for UnsignedIntegerQuantizationLevel {
    const NUMBER_BITS: usize = 3;

    unsafe fn from_unpacked_byte(byte: u8) -> Self {
        core::mem::transmute(byte)
    }
}

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
    + core::cmp::Ord
    + core::iter::Sum
    + core::fmt::Debug
    + core::fmt::Display
    + core::ops::Rem<Output = Self>
    + core::ops::RemAssign
    + core::cmp::Eq
    + core::hash::Hash
    + Sized
    + 'static
    + QuantizedElementBase
{
    const LEVEL: UnsignedIntegerQuantizationLevel;

    const QUANT_MAX: Self;

    const QUANT_MAX_U8: Self;
    const QUANT_MAX_U16: Self;
    const QUANT_MAX_U32: Self;
    const QUANT_MAX_U64: Self;
    const QUANT_MAX_USIZE: usize;

    const QUANT_CLAMPED_U8: u8;
    const QUANT_CLAMPED_U16: u16;
    const QUANT_CLAMPED_U32: u32;
    const QUANT_CLAMPED_U64: u64;
    const QUANT_CLAMPED_USIZE: usize;

    /// Tries to convert from usize, does NOT check bounds!
    fn quant_from_usize(value: usize) -> Self;

    /// Will wrap whatever quant this is to an `UnsignedIntegerEnum`
    fn quant_to_enum(value: Self) -> UnsignedIntegerEnum;

    fn is_negative(&self) -> bool;
    fn is_zero_or_negative(&self) -> bool;
}

impl QuantizedUnsignedIntegerTrait for u8 {
    const LEVEL: UnsignedIntegerQuantizationLevel = UnsignedIntegerQuantizationLevel::U8;
    const QUANT_MAX: Self = u8::MAX;
    const QUANT_MAX_U8: Self = u8::MAX;
    const QUANT_MAX_U16: Self = u8::MAX;
    const QUANT_MAX_U32: Self = u8::MAX;
    const QUANT_MAX_U64: Self = u8::MAX;
    const QUANT_MAX_USIZE: usize = u8::MAX as usize;

    const QUANT_CLAMPED_U8: u8 = u8::MAX;
    const QUANT_CLAMPED_U16: u16 = u8::MAX as u16;
    const QUANT_CLAMPED_U32: u32 = u8::MAX as u32;
    const QUANT_CLAMPED_U64: u64 = u8::MAX as u64;
    const QUANT_CLAMPED_USIZE: usize = u8::MAX as usize;

    fn quant_from_usize(value: usize) -> Self {
        value as u8
    }

    fn quant_to_enum(value: Self) -> UnsignedIntegerEnum {
        UnsignedIntegerEnum::U8(value)
    }

    #[inline(always)]
    fn is_negative(&self) -> bool {
        false
    }

    #[inline(always)]
    fn is_zero_or_negative(&self) -> bool {
        *self == 0
    }
}

impl QuantizedUnsignedIntegerTrait for u16 {
    const LEVEL: UnsignedIntegerQuantizationLevel = UnsignedIntegerQuantizationLevel::U16;
    const QUANT_MAX: Self = u16::MAX;
    const QUANT_MAX_U8: Self = u8::MAX as u16;
    const QUANT_MAX_U16: Self = u16::MAX;
    const QUANT_MAX_U32: Self = u16::MAX;
    const QUANT_MAX_U64: Self = u16::MAX;
    const QUANT_MAX_USIZE: usize = u16::MAX as usize;

    const QUANT_CLAMPED_U8: u8 = u8::MAX;
    const QUANT_CLAMPED_U16: u16 = u16::MAX;
    const QUANT_CLAMPED_U32: u32 = u16::MAX as u32;
    const QUANT_CLAMPED_U64: u64 = u16::MAX as u64;
    const QUANT_CLAMPED_USIZE: usize = u16::MAX as usize;

    fn quant_from_usize(value: usize) -> Self {
        value as u16
    }

    fn quant_to_enum(value: Self) -> UnsignedIntegerEnum {
        UnsignedIntegerEnum::U16(value)
    }

    #[inline(always)]
    fn is_negative(&self) -> bool {
        false
    }

    #[inline(always)]
    fn is_zero_or_negative(&self) -> bool {
        *self == 0
    }
}

// lol, lmao even
impl QuantizedUnsignedIntegerTrait for u32 {
    const LEVEL: UnsignedIntegerQuantizationLevel = UnsignedIntegerQuantizationLevel::U32;
    const QUANT_MAX: Self = u32::MAX;
    const QUANT_MAX_U8: Self = u8::MAX as u32;
    const QUANT_MAX_U16: Self = u16::MAX as u32;
    const QUANT_MAX_U32: Self = u32::MAX;
    const QUANT_MAX_U64: Self = u32::MAX;
    const QUANT_MAX_USIZE: usize = u32::MAX as usize;

    const QUANT_CLAMPED_U8: u8 = u8::MAX;
    const QUANT_CLAMPED_U16: u16 = u16::MAX;
    const QUANT_CLAMPED_U32: u32 = u32::MAX;
    const QUANT_CLAMPED_U64: u64 = u32::MAX as u64;
    const QUANT_CLAMPED_USIZE: usize = u32::MAX as usize;

    fn quant_from_usize(value: usize) -> Self {
        value as u32
    }

    fn quant_to_enum(value: Self) -> UnsignedIntegerEnum {
        UnsignedIntegerEnum::U32(value)
    }

    #[inline(always)]
    fn is_negative(&self) -> bool {
        false
    }

    #[inline(always)]
    fn is_zero_or_negative(&self) -> bool {
        *self == 0
    }
}

impl QuantizedUnsignedIntegerTrait for u64 {
    const LEVEL: UnsignedIntegerQuantizationLevel = UnsignedIntegerQuantizationLevel::U64;
    const QUANT_MAX: Self = u64::MAX;
    const QUANT_MAX_U8: Self = u8::MAX as u64;
    const QUANT_MAX_U16: Self = u16::MAX as u64;
    const QUANT_MAX_U32: Self = u32::MAX as u64;
    const QUANT_MAX_U64: Self = u64::MAX;
    const QUANT_MAX_USIZE: usize = usize::MAX;

    const QUANT_CLAMPED_U8: u8 = u8::MAX;
    const QUANT_CLAMPED_U16: u16 = u16::MAX;
    const QUANT_CLAMPED_U32: u32 = u32::MAX;
    const QUANT_CLAMPED_U64: u64 = u64::MAX;
    const QUANT_CLAMPED_USIZE: usize = usize::MAX;

    fn quant_from_usize(value: usize) -> Self {
        value as u64
    }

    fn quant_to_enum(value: Self) -> UnsignedIntegerEnum {
        UnsignedIntegerEnum::U64(value)
    }

    #[inline(always)]
    fn is_negative(&self) -> bool {
        false
    }

    #[inline(always)]
    fn is_zero_or_negative(&self) -> bool {
        *self == 0
    }
}

// Note: Specifically we will not support usize directly since it can vary in size depending on
// backend, which could cause some issues with device interoperability
impl QuantizedUnsignedIntegerTrait for usize {
    const LEVEL: UnsignedIntegerQuantizationLevel = UnsignedIntegerQuantizationLevel::Usize;
    const QUANT_MAX: Self = usize::MAX;
    const QUANT_MAX_U8: Self = u8::MAX as usize;
    const QUANT_MAX_U16: Self = u16::MAX as usize;
    const QUANT_MAX_U32: Self = u32::MAX as usize;
    const QUANT_MAX_U64: Self = usize::MAX;
    const QUANT_MAX_USIZE: usize = usize::MAX;

    const QUANT_CLAMPED_U8: u8 = u8::MAX;
    const QUANT_CLAMPED_U16: u16 = u16::MAX;
    const QUANT_CLAMPED_U32: u32 = u32::MAX;
    const QUANT_CLAMPED_U64: u64 = u64::MAX;
    const QUANT_CLAMPED_USIZE: usize = usize::MAX;

    fn quant_from_usize(value: usize) -> Self {
        value
    }

    fn quant_to_enum(value: Self) -> UnsignedIntegerEnum {
        UnsignedIntegerEnum::U64(value as u64)
    }

    #[inline(always)]
    fn is_negative(&self) -> bool {
        false
    }

    #[inline(always)]
    fn is_zero_or_negative(&self) -> bool {
        *self == 0
    }
}

/// Allows storing all quantized unsigned integer types under a single enum
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum UnsignedIntegerEnum {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
}

impl UnsignedIntegerEnum {
    pub fn new_from_quantized<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self {
        FromQuant::quant_to_enum(value)
    }

    pub fn get_level(&self) -> UnsignedIntegerQuantizationLevel {
        match self {
            UnsignedIntegerEnum::U8(_) => UnsignedIntegerQuantizationLevel::U8,
            UnsignedIntegerEnum::U16(_) => UnsignedIntegerQuantizationLevel::U16,
            UnsignedIntegerEnum::U32(_) => UnsignedIntegerQuantizationLevel::U32,
            UnsignedIntegerEnum::U64(_) => UnsignedIntegerQuantizationLevel::U64,
        }
    }

    pub fn try_into_quant<Quant: QuantizedUnsignedIntegerTrait>(self) -> Result<Quant, FeagiDataValueQuantizationError> {
        let value = self.to_usize();
        if !unsigned_value_fits_quant::<Quant>(value) {
            return Err(FeagiFailQuantizationOutOfRange::new("Quantized unsigned integer exceeds target quantization!", value).into());
        }
        Ok(Quant::quant_from_usize(value))
    }

    pub fn into_quant<Quant: QuantizedUnsignedIntegerTrait>(self) -> Quant {
        Quant::quant_from_usize(self.to_usize())
    }

    pub fn to_usize(self) -> usize {
        match self {
            UnsignedIntegerEnum::U8(value) => value as usize,
            UnsignedIntegerEnum::U16(value) => value as usize,
            UnsignedIntegerEnum::U32(value) => value as usize,
            UnsignedIntegerEnum::U64(value) => value as usize,
        }
    }

    // TODO from usize that is CPU dependent to be either 32 bit or 64 bit
}

fn unsigned_value_fits_quant<Quant: QuantizedUnsignedIntegerTrait>(value: usize) -> bool {
    match Quant::LEVEL {
        UnsignedIntegerQuantizationLevel::U8 => value <= u8::MAX as usize,
        UnsignedIntegerQuantizationLevel::U16 => value <= u16::MAX as usize,
        UnsignedIntegerQuantizationLevel::U32 => value <= u32::MAX as usize,
        UnsignedIntegerQuantizationLevel::U64 => true,
        UnsignedIntegerQuantizationLevel::Usize => true,
    }
}

/// Shared behaviour implemented by every strongly-typed wrapper generated by
/// [`create_wrapped_quantized_unsigned_integer`].
///
/// Each wrapper produced by the macro is a distinct `#[repr(transparent)]` newtype, so that
/// logically different unsigned integer values cannot be accidentally mixed at compile time. This
/// trait exposes the common surface those wrappers share, allowing functions to generically accept
/// "some wrapped unsigned integer" (e.g. `fn foo<I: WrappedQuantizedUnsignedInteger>(value: I)`) while
/// still preserving the compile-time distinctness of the concrete wrapper types.
///
/// The bulk of the behaviour is provided here as default methods that delegate to the underlying
/// [`QuantizedUnsignedIntegerTrait`] value; the macro only needs to supply [`Self::new`],
/// [`Self::deref`] and the `Self`-typed constants.
pub trait WrappedQuantizedUnsignedInteger:
    Copy
    + Clone
    + Send
    + Sync
    + Default
    + core::fmt::Debug
    + core::cmp::PartialEq
    + core::cmp::Eq
    + core::cmp::PartialOrd
    + core::cmp::Ord
    + core::hash::Hash
    + core::ops::Add<Output = Self>
    + core::ops::Sub<Output = Self>
    + core::ops::Mul<Output = Self>
    + core::ops::Div<Output = Self>
    + core::ops::Rem<Output = Self>
    + core::ops::AddAssign
    + core::ops::SubAssign
    + core::ops::MulAssign
    + core::ops::DivAssign
    + core::ops::RemAssign
    + From<Self::Quant>
    + AsRef<Self::Quant>
    + AsMut<Self::Quant>
    + Sized
    + 'static
{
    /// The underlying quantized unsigned integer value this wrapper stores.
    type Quant: QuantizedUnsignedIntegerTrait;

    /// The quantization level of the underlying value.
    const LEVEL: UnsignedIntegerQuantizationLevel = <Self::Quant as QuantizedUnsignedIntegerTrait>::LEVEL;

    /// Zero, expressed in the wrapper's own type.
    const QUANT_ZERO: Self;
    /// One, expressed in the wrapper's own type.
    const QUANT_ONE: Self;
    /// The maximum representable value, expressed in the wrapper's own type.
    const QUANT_MAX: Self;

    const QUANT_MAX_U8: Self;
    const QUANT_MAX_U16: Self;
    const QUANT_MAX_U32: Self;
    const QUANT_MAX_U64: Self;
    const QUANT_MAX_USIZE: usize = <Self::Quant as QuantizedUnsignedIntegerTrait>::QUANT_MAX_USIZE;

    const QUANT_CLAMPED_U8: u8 = <Self::Quant as QuantizedUnsignedIntegerTrait>::QUANT_CLAMPED_U8;
    const QUANT_CLAMPED_U16: u16 = <Self::Quant as QuantizedUnsignedIntegerTrait>::QUANT_CLAMPED_U16;
    const QUANT_CLAMPED_U32: u32 = <Self::Quant as QuantizedUnsignedIntegerTrait>::QUANT_CLAMPED_U32;
    const QUANT_CLAMPED_U64: u64 = <Self::Quant as QuantizedUnsignedIntegerTrait>::QUANT_CLAMPED_U64;
    const QUANT_CLAMPED_USIZE: usize = <Self::Quant as QuantizedUnsignedIntegerTrait>::QUANT_CLAMPED_USIZE;

    /// Wraps a raw quantized value into this wrapper type.
    fn new(value: Self::Quant) -> Self;

    /// Extracts the inner quantized unsigned integer.
    fn deref(self) -> Self::Quant;

    /// Tries to convert from usize, does NOT check bounds!
    fn quant_from_usize(value: usize) -> Self {
        Self::new(Self::Quant::quant_from_usize(value))
    }

    /// Will wrap whatever quant this is to an [`UnsignedIntegerEnum`]
    fn quant_to_enum(self) -> UnsignedIntegerEnum {
        Self::Quant::quant_to_enum(self.deref())
    }

    fn is_negative(&self) -> bool {
        self.deref().is_negative()
    }

    fn is_zero_or_negative(&self) -> bool {
        self.deref().is_zero_or_negative()
    }
}

/// Shared behaviour implemented by every wrapped enum generated by
/// [`create_wrapped_quantized_unsigned_integer`].
///
/// These enums hide the generic quantized wrapper type behind concrete variants
/// (`U8`, `U16`, `U32`, `U64`) while preserving the wrapper family semantics.
pub trait WrappedQuantizedUnsignedIntegerEnum:
    Copy + Clone + Send + Sync + core::fmt::Debug + core::cmp::PartialEq + core::cmp::Eq + core::hash::Hash + Sized + 'static
{
    fn get_level(&self) -> UnsignedIntegerQuantizationLevel;

    fn try_into_quant<Quant: QuantizedUnsignedIntegerTrait>(self) -> Result<Quant, FeagiDataValueQuantizationError>;

    fn into_quant<Quant: QuantizedUnsignedIntegerTrait>(self) -> Quant;

    fn to_usize(self) -> usize;
}

/// Creates a wrapper for quantized unsigned integers
#[macro_export]
macro_rules! create_wrapped_quantized_unsigned_integer {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident
    ) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        $vis struct $struct_name<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(Q);

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> $struct_name<Q> {
            pub const LEVEL: $crate::values::quantizable::UnsignedIntegerQuantizationLevel = Q::LEVEL;
            pub const QUANT_ZERO: Self = Self::const_new(Q::QUANT_ZERO);
            pub const QUANT_ONE: Self = Self::const_new(Q::QUANT_ONE);
            pub const QUANT_MAX: Self = Self::const_new(Q::QUANT_MAX);

            pub const QUANT_MAX_U8: Self = Self::const_new(Q::QUANT_MAX_U8);
            pub const QUANT_MAX_U16: Self = Self::const_new(Q::QUANT_MAX_U16);
            pub const QUANT_MAX_U32: Self = Self::const_new(Q::QUANT_MAX_U32);
            pub const QUANT_MAX_U64: Self = Self::const_new(Q::QUANT_MAX_U64);
            pub const QUANT_MAX_USIZE: usize = Q::QUANT_MAX_USIZE;

            pub const QUANT_CLAMPED_U8: u8 = Q::QUANT_CLAMPED_U8;
            pub const QUANT_CLAMPED_U16: u16 = Q::QUANT_CLAMPED_U16;
            pub const QUANT_CLAMPED_U32: u32 = Q::QUANT_CLAMPED_U32;
            pub const QUANT_CLAMPED_U64: u64 = Q::QUANT_CLAMPED_U64;
            pub const QUANT_CLAMPED_USIZE: usize = Q::QUANT_CLAMPED_USIZE;

            pub const fn const_new(value: Q) -> Self {
                Self(value)
            }

            pub const fn const_deref(self) -> Q {
                self.0
            }

            pub fn new(v: Q) -> Self {
                Self(v)
            }

            /// Extracts the inner quantized unsigned integer
            pub fn deref(self) -> Q {
                self.0
            }

            pub fn is_negative(&self) -> bool {
                self.0.is_negative()
            }

            pub fn is_zero_or_negative(&self) -> bool {
                self.0.is_zero_or_negative()
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>
            $crate::values::quantizable::WrappedQuantizedUnsignedInteger for $struct_name<Q>
        {
            type Quant = Q;

            const QUANT_ZERO: Self = Self::const_new(Q::QUANT_ZERO);
            const QUANT_ONE: Self = Self::const_new(Q::QUANT_ONE);
            const QUANT_MAX: Self = Self::const_new(Q::QUANT_MAX);
            const QUANT_MAX_U8: Self = Self::const_new(Q::QUANT_MAX_U8);
            const QUANT_MAX_U16: Self = Self::const_new(Q::QUANT_MAX_U16);
            const QUANT_MAX_U32: Self = Self::const_new(Q::QUANT_MAX_U32);
            const QUANT_MAX_U64: Self = Self::const_new(Q::QUANT_MAX_U64);

            fn new(value: Q) -> Self {
                Self(value)
            }

            fn deref(self) -> Q {
                self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> From<Q> for $struct_name<Q> {
            fn from(value: Q) -> Self {
                Self(value)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> From<&Q> for &$struct_name<Q> {
            fn from(value: &Q) -> Self {
                unsafe { &*(value as *const Q as *const $struct_name<Q>) }
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> From<&mut Q> for &mut $struct_name<Q> {
            fn from(value: &mut Q) -> Self {
                unsafe { &mut *(value as *mut Q as *mut $struct_name<Q>) }
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> AsRef<Q> for $struct_name<Q> {
            fn as_ref(&self) -> &Q {
                &self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> AsMut<Q> for $struct_name<Q> {
            fn as_mut(&mut self) -> &mut Q {
                &mut self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> core::ops::Add for $struct_name<Q> {
            type Output = Self;
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> core::ops::Sub for $struct_name<Q> {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> core::ops::Mul for $struct_name<Q> {
            type Output = Self;
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> core::ops::Div for $struct_name<Q> {
            type Output = Self;
            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> core::ops::Rem for $struct_name<Q> {
            type Output = Self;
            fn rem(self, rhs: Self) -> Self::Output {
                Self(self.0 % rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> core::ops::AddAssign for $struct_name<Q> {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> core::ops::SubAssign for $struct_name<Q> {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> core::ops::MulAssign for $struct_name<Q> {
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> core::ops::DivAssign for $struct_name<Q> {
            fn div_assign(&mut self, rhs: Self) {
                self.0 /= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> core::ops::RemAssign for $struct_name<Q> {
            fn rem_assign(&mut self, rhs: Self) {
                self.0 %= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> Default for $struct_name<Q> {
            fn default() -> Self {
                Self(Q::default())
            }
        }

        ::paste::paste! {
            #[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
            $vis enum [<$struct_name Enum>] {
                U8($struct_name<u8>),
                U16($struct_name<u16>),
                U32($struct_name<u32>),
                U64($struct_name<u64>),
            }

            impl [<$struct_name Enum>] {
                pub fn new_from_quantized<FromQuant: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    value: $struct_name<FromQuant>
                ) -> Self {
                    let as_usize = $crate::values::quantizable::UnsignedIntegerEnum::new_from_quantized(value.deref()).to_usize();
                    match FromQuant::LEVEL {
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U8 => {
                            Self::U8($struct_name::<u8>::new(<u8 as $crate::values::quantizable::QuantizedUnsignedIntegerTrait>::quant_from_usize(as_usize)))
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U16 => {
                            Self::U16($struct_name::<u16>::new(<u16 as $crate::values::quantizable::QuantizedUnsignedIntegerTrait>::quant_from_usize(as_usize)))
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U32 => {
                            Self::U32($struct_name::<u32>::new(<u32 as $crate::values::quantizable::QuantizedUnsignedIntegerTrait>::quant_from_usize(as_usize)))
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U64 => {
                            Self::U64($struct_name::<u64>::new(<u64 as $crate::values::quantizable::QuantizedUnsignedIntegerTrait>::quant_from_usize(as_usize)))
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::Usize => {
                            Self::U64($struct_name::<u64>::new(<u64 as $crate::values::quantizable::QuantizedUnsignedIntegerTrait>::quant_from_usize(as_usize)))
                        }
                    }
                }

                pub fn from_unsigned_integer_enum(value: $crate::values::quantizable::UnsignedIntegerEnum) -> Self {
                    match value {
                        $crate::values::quantizable::UnsignedIntegerEnum::U8(v) => {
                            Self::U8($struct_name::<u8>::new(v))
                        }
                        $crate::values::quantizable::UnsignedIntegerEnum::U16(v) => {
                            Self::U16($struct_name::<u16>::new(v))
                        }
                        $crate::values::quantizable::UnsignedIntegerEnum::U32(v) => {
                            Self::U32($struct_name::<u32>::new(v))
                        }
                        $crate::values::quantizable::UnsignedIntegerEnum::U64(v) => {
                            Self::U64($struct_name::<u64>::new(v))
                        }
                    }
                }

                pub fn into_unsigned_integer_enum(self) -> $crate::values::quantizable::UnsignedIntegerEnum {
                    match self {
                        Self::U8(v) => $crate::values::quantizable::UnsignedIntegerEnum::U8(v.deref()),
                        Self::U16(v) => $crate::values::quantizable::UnsignedIntegerEnum::U16(v.deref()),
                        Self::U32(v) => $crate::values::quantizable::UnsignedIntegerEnum::U32(v.deref()),
                        Self::U64(v) => $crate::values::quantizable::UnsignedIntegerEnum::U64(v.deref()),
                    }
                }

                pub fn try_into_wrapped_quant<Quant: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> Result<$struct_name<Quant>, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                    Ok($struct_name::<Quant>::new(
                        <Self as $crate::values::quantizable::WrappedQuantizedUnsignedIntegerEnum>::try_into_quant::<Quant>(self)?
                    ))
                }

                pub fn into_wrapped_quant<Quant: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $struct_name<Quant> {
                    $struct_name::<Quant>::new(
                        <Self as $crate::values::quantizable::WrappedQuantizedUnsignedIntegerEnum>::into_quant::<Quant>(self)
                    )
                }
            }

            impl $crate::values::quantizable::WrappedQuantizedUnsignedIntegerEnum for [<$struct_name Enum>] {
                fn get_level(&self) -> $crate::values::quantizable::UnsignedIntegerQuantizationLevel {
                    match self {
                        Self::U8(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U8,
                        Self::U16(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U16,
                        Self::U32(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U32,
                        Self::U64(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U64,
                    }
                }

                fn try_into_quant<Quant: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> Result<Quant, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                    match self {
                        Self::U8(value) => $crate::values::quantizable::UnsignedIntegerEnum::U8(value.deref()).try_into_quant(),
                        Self::U16(value) => $crate::values::quantizable::UnsignedIntegerEnum::U16(value.deref()).try_into_quant(),
                        Self::U32(value) => $crate::values::quantizable::UnsignedIntegerEnum::U32(value.deref()).try_into_quant(),
                        Self::U64(value) => $crate::values::quantizable::UnsignedIntegerEnum::U64(value.deref()).try_into_quant(),
                    }
                }

                fn into_quant<Quant: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(self) -> Quant {
                    match self {
                        Self::U8(value) => Quant::quant_from_usize(value.deref() as usize),
                        Self::U16(value) => Quant::quant_from_usize(value.deref() as usize),
                        Self::U32(value) => Quant::quant_from_usize(value.deref() as usize),
                        Self::U64(value) => Quant::quant_from_usize(value.deref() as usize),
                    }
                }

                fn to_usize(self) -> usize {
                    match self {
                        Self::U8(value) => value.deref() as usize,
                        Self::U16(value) => value.deref() as usize,
                        Self::U32(value) => value.deref() as usize,
                        Self::U64(value) => value.deref() as usize,
                    }
                }
            }
        }
    };
}
