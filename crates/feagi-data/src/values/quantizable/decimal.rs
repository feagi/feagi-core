//! Values that hold some sort of decimal (float) value. Note that yes, different hardwares support
//! different values types

// TODO Equal check with epsilon

use crate::values::quantizable::custom_data_types::StorageF8;
use crate::values::quantizable::quantization_level_packing::QuantizationLevelPacking;
use crate::values::quantizable::{PercentageUnsigned, QuantizedElementBase};
use half::{bf16, f16};

/// Represents a value that is represented as a decimal number, main backbone for computations
#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum DecimalQuantizationLevel {
    F16 = 0,
    BF16 = 1,
    F32 = 2,
    F64 = 3,
    StorageF8 = 4,
    // We can support a max of 16 quants
}

impl Into<u8> for DecimalQuantizationLevel {
    fn into(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for DecimalQuantizationLevel {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(DecimalQuantizationLevel::F16),
            1 => Ok(DecimalQuantizationLevel::BF16),
            2 => Ok(DecimalQuantizationLevel::F32),
            3 => Ok(DecimalQuantizationLevel::F64),
            4 => Ok(DecimalQuantizationLevel::StorageF8),
            _ => Err(()),
        }
    }
}

impl QuantizationLevelPacking for DecimalQuantizationLevel {
    const NUMBER_BITS: usize = 4; // 16 quants ( we will need more custom types later)

    unsafe fn from_unpacked_byte(byte: u8) -> Self {
        core::mem::transmute(byte)
    }
}

/// Quantizable data for some decimal value (float)
pub trait QuantizedDecimalTrait:
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
    + Sized
    + 'static
    + QuantizedElementBase
{
    const LEVEL: DecimalQuantizationLevel;

    fn quant_clamp(&self, min: Self, max: Self) -> Self;

    /// Will wrap whatever quant this is to a `DecimalEnum`
    fn quant_to_enum(value: Self) -> DecimalEnum;

    fn quant_to_storage_f8(self) -> StorageF8;

    fn quant_to_f16(self) -> f16;

    fn quant_to_bf16(self) -> bf16;

    fn quant_to_f32(self) -> f32;

    fn quant_to_f64(self) -> f64;

    /// Creates from a decimal of another quantization
    fn from_quantization<FromQuant: QuantizedDecimalTrait>(value: FromQuant) -> Self;

    /// Converts to a decimal of another quantization
    fn to_quantization<ToQuant: QuantizedDecimalTrait>(self) -> ToQuant {
        ToQuant::from_quantization(self)
    }

    fn scale_by_unsigned_percentage<OTHER: QuantizedDecimalTrait>(self, p: PercentageUnsigned<OTHER>) -> Self {
        // percentages will always be in valid range, we dont need a checked conversion
        self * Self::from_quantization::<OTHER>(p.get_decimal())
    }

    fn scale_by_same_quant_unsigned_percentage(self, p: &PercentageUnsigned<Self>) -> Self {
        self * p.get_decimal()
    }
}

impl QuantizedDecimalTrait for StorageF8 {
    const LEVEL: DecimalQuantizationLevel = DecimalQuantizationLevel::StorageF8;

    fn quant_clamp(&self, min: Self, max: Self) -> Self {
        todo!()
    }

    fn quant_to_enum(value: Self) -> DecimalEnum {
        DecimalEnum::StorageF8(value)
    }

    fn quant_to_storage_f8(self) -> StorageF8 {
        todo!()
    }

    fn quant_to_f16(self) -> f16 {
        todo!()
    }

    fn quant_to_bf16(self) -> bf16 {
        todo!()
    }

    fn quant_to_f32(self) -> f32 {
        todo!()
    }

    fn quant_to_f64(self) -> f64 {
        todo!()
    }

    fn from_quantization<FromQuant: QuantizedDecimalTrait>(value: FromQuant) -> Self {
        todo!()
    }
}

impl QuantizedDecimalTrait for f16 {
    const LEVEL: DecimalQuantizationLevel = DecimalQuantizationLevel::F16;

    fn quant_clamp(&self, min: Self, max: Self) -> Self {
        self.clamp(min, max)
    }

    fn quant_to_enum(value: Self) -> DecimalEnum {
        DecimalEnum::F16(value)
    }

    fn quant_to_storage_f8(self) -> StorageF8 {
        todo!()
    }

    fn quant_to_f16(self) -> f16 {
        self
    }

    fn quant_to_bf16(self) -> bf16 {
        bf16::from_f32(self.to_f32())
    }

    fn quant_to_f32(self) -> f32 {
        f16::to_f32(self)
    }

    fn quant_to_f64(self) -> f64 {
        f16::to_f64(self)
    }

    fn from_quantization<FromQuant: QuantizedDecimalTrait>(value: FromQuant) -> Self {
        value.quant_to_f16()
    }
}

impl QuantizedDecimalTrait for bf16 {
    const LEVEL: DecimalQuantizationLevel = DecimalQuantizationLevel::BF16;

    fn quant_clamp(&self, min: Self, max: Self) -> Self {
        self.clamp(min, max)
    }

    fn quant_to_enum(value: Self) -> DecimalEnum {
        DecimalEnum::BF16(value)
    }

    fn quant_to_storage_f8(self) -> StorageF8 {
        todo!()
    }

    fn quant_to_f16(self) -> f16 {
        f16::from_f32(self.to_f32())
    }

    fn quant_to_bf16(self) -> bf16 {
        self
    }

    fn quant_to_f32(self) -> f32 {
        bf16::to_f32(self)
    }

    fn quant_to_f64(self) -> f64 {
        bf16::to_f64(self)
    }

    fn from_quantization<FromQuant: QuantizedDecimalTrait>(value: FromQuant) -> Self {
        value.quant_to_bf16()
    }
}

impl QuantizedDecimalTrait for f32 {
    const LEVEL: DecimalQuantizationLevel = DecimalQuantizationLevel::F32;

    fn quant_clamp(&self, min: Self, max: Self) -> Self {
        self.clamp(min, max)
    }

    fn quant_to_enum(value: Self) -> DecimalEnum {
        DecimalEnum::F32(value)
    }

    fn quant_to_storage_f8(self) -> StorageF8 {
        todo!()
    }

    fn quant_to_f16(self) -> f16 {
        f16::from_f32(self)
    }

    fn quant_to_bf16(self) -> bf16 {
        bf16::from_f32(self)
    }

    fn quant_to_f32(self) -> f32 {
        self
    }

    fn quant_to_f64(self) -> f64 {
        self as f64
    }

    fn from_quantization<FromQuant: QuantizedDecimalTrait>(value: FromQuant) -> Self {
        value.quant_to_f32()
    }
}

impl QuantizedDecimalTrait for f64 {
    const LEVEL: DecimalQuantizationLevel = DecimalQuantizationLevel::F64;

    fn quant_clamp(&self, min: Self, max: Self) -> Self {
        self.clamp(min, max)
    }

    fn quant_to_enum(value: Self) -> DecimalEnum {
        DecimalEnum::F64(value)
    }

    fn quant_to_storage_f8(self) -> StorageF8 {
        todo!()
    }

    fn quant_to_f16(self) -> f16 {
        f16::from_f64(self)
    }

    fn quant_to_bf16(self) -> bf16 {
        bf16::from_f64(self)
    }

    fn quant_to_f32(self) -> f32 {
        self as f32
    }

    fn quant_to_f64(self) -> f64 {
        self
    }

    fn from_quantization<FromQuant: QuantizedDecimalTrait>(value: FromQuant) -> Self {
        value.quant_to_f64()
    }
}

/// Allows storing all quantized decimal types under a single enum
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DecimalEnum {
    F16(f16),
    BF16(bf16),
    F32(f32),
    F64(f64),
    StorageF8(StorageF8),
}

impl DecimalEnum {
    pub fn new_from_quantized<FromQuant: QuantizedDecimalTrait>(value: FromQuant) -> Self {
        FromQuant::quant_to_enum(value)
    }

    pub fn get_level(&self) -> DecimalQuantizationLevel {
        match self {
            DecimalEnum::F16(_) => DecimalQuantizationLevel::F16,
            DecimalEnum::BF16(_) => DecimalQuantizationLevel::BF16,
            DecimalEnum::F32(_) => DecimalQuantizationLevel::F32,
            DecimalEnum::F64(_) => DecimalQuantizationLevel::F64,
            DecimalEnum::StorageF8(_) => DecimalQuantizationLevel::StorageF8,
        }
    }

    pub fn into_quant<Quant: QuantizedDecimalTrait>(self) -> Quant {
        match self {
            DecimalEnum::F16(value) => value.to_quantization(),
            DecimalEnum::BF16(value) => value.to_quantization(),
            DecimalEnum::F32(value) => value.to_quantization(),
            DecimalEnum::F64(value) => value.to_quantization(),
            DecimalEnum::StorageF8(value) => value.to_quantization(),
        }
    }
}

/// Shared behaviour implemented by every strongly-typed wrapper generated by
/// [`create_wrapped_quantized_decimal`].
///
/// Each wrapper produced by the macro is a distinct `#[repr(transparent)]` newtype, so that
/// logically different decimal values cannot be accidentally mixed at compile time. This trait
/// exposes the common surface those wrappers share, allowing functions to generically accept
/// "some wrapped decimal" (e.g. `fn foo<D: WrappedQuantizedDecimal>(value: D)`) while still
/// preserving the compile-time distinctness of the concrete wrapper types.
///
/// The bulk of the behaviour is provided here as default methods that delegate to the underlying
/// [`QuantizedDecimalTrait`] value; the macro only needs to supply [`Self::new`],
/// [`Self::deref`] and the `Self`-typed constants.
///
/// Note: because the underlying types are floats, `Eq`, `Ord` and `Hash` are intentionally *not*
/// required (nor is `Rem`), matching the interface the macro actually generates.
pub trait WrappedQuantizedDecimal:
    Copy
    + Clone
    + Send
    + Sync
    + Default
    + core::fmt::Debug
    + core::cmp::PartialEq
    + core::cmp::PartialOrd
    + core::ops::Add<Output = Self>
    + core::ops::Sub<Output = Self>
    + core::ops::Mul<Output = Self>
    + core::ops::Div<Output = Self>
    + core::ops::AddAssign
    + core::ops::SubAssign
    + core::ops::MulAssign
    + core::ops::DivAssign
    + From<Self::Quant>
    + AsRef<Self::Quant>
    + AsMut<Self::Quant>
    + Sized
    + 'static
{
    /// The underlying quantized decimal value this wrapper stores.
    type Quant: QuantizedDecimalTrait;

    /// The quantization level of the underlying value.
    const LEVEL: DecimalQuantizationLevel = <Self::Quant as QuantizedDecimalTrait>::LEVEL;

    /// Zero, expressed in the wrapper's own type.
    const QUANT_ZERO: Self;
    /// One, expressed in the wrapper's own type.
    const QUANT_ONE: Self;

    /// Wraps a raw quantized value into this wrapper type.
    fn new(value: Self::Quant) -> Self;

    /// Extracts the inner quantized decimal.
    fn deref(self) -> Self::Quant;

    fn quant_to_storage_f8(self) -> StorageF8 {
        self.deref().quant_to_storage_f8()
    }

    fn quant_to_f16(self) -> f16 {
        self.deref().quant_to_f16()
    }

    fn quant_to_bf16(self) -> bf16 {
        self.deref().quant_to_bf16()
    }

    fn quant_to_f32(self) -> f32 {
        self.deref().quant_to_f32()
    }

    fn quant_to_f64(self) -> f64 {
        self.deref().quant_to_f64()
    }

    /// Creates from a decimal of another quantization
    fn from_quantization<FromQuant: QuantizedDecimalTrait>(value: FromQuant) -> Self {
        Self::new(Self::Quant::from_quantization(value))
    }

    /// Converts to a decimal of another quantization
    fn to_quantization<ToQuant: QuantizedDecimalTrait>(self) -> ToQuant {
        self.deref().to_quantization()
    }

    fn scale_by_unsigned_percentage<OTHER: QuantizedDecimalTrait>(self, p: PercentageUnsigned<OTHER>) -> Self {
        Self::new(self.deref().scale_by_unsigned_percentage(p))
    }

    fn scale_by_same_quant_unsigned_percentage(self, p: &PercentageUnsigned<Self::Quant>) -> Self {
        Self::new(self.deref().scale_by_same_quant_unsigned_percentage(p))
    }
}

/// Shared behaviour implemented by every wrapped enum generated by
/// [`create_wrapped_quantized_decimal`].
///
/// These enums hide the generic wrapped decimal type behind concrete variants
/// (`F16`, `BF16`, `F32`, `F64`, `StorageF8`) while preserving wrapper-family semantics.
pub trait WrappedQuantizedDecimalEnum: Copy + Clone + Send + Sync + core::fmt::Debug + core::cmp::PartialEq + Sized + 'static {
    fn get_level(&self) -> DecimalQuantizationLevel;

    fn into_quant<Quant: QuantizedDecimalTrait>(self) -> Quant;

    fn to_f64(self) -> f64;
}

/// Creates a wrapper for quantized decimal values
#[macro_export]
macro_rules! create_wrapped_quantized_decimal {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident
    ) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        $vis struct $struct_name<Q: $crate::values::quantizable::QuantizedDecimalTrait>(Q);

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> $struct_name<Q> {
            pub const LEVEL: $crate::values::quantizable::DecimalQuantizationLevel = Q::LEVEL;
            pub const QUANT_ZERO: Self = Self::const_new(Q::QUANT_ZERO);
            pub const QUANT_ONE: Self = Self::const_new(Q::QUANT_ONE);

            pub const fn const_new(value: Q) -> Self
            {
                Self(value)
            }

            pub const fn const_deref(self) -> Q
            {
                self.0
            }

            pub fn new(v: Q) -> Self {
                Self(v)
            }

            /// Extracts the inner quantized decimal
            pub fn deref(self) -> Q {
                self.0
            }
        }

        // The bulk of the wrapper's behaviour lives on the shared
        // `WrappedQuantizedDecimal` trait so that functions can generically accept any wrapped
        // decimal. See its definition for the available methods.
        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait>
            $crate::values::quantizable::WrappedQuantizedDecimal for $struct_name<Q>
        {
            type Quant = Q;

            const QUANT_ZERO: Self = Self::const_new(Q::QUANT_ZERO);
            const QUANT_ONE: Self = Self::const_new(Q::QUANT_ONE);

            fn new(value: Q) -> Self {
                Self(value)
            }

            fn deref(self) -> Q {
                self.0
            }
        }

        // NOTE: Into<Q> for $struct_name<Q> is not needed!

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> From<Q> for $struct_name<Q> {
            fn from(value: Q) -> Self {
                Self(value)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> From<&Q> for &$struct_name<Q> {
            fn from(value: &Q) -> Self {
                // tRust me bro
                unsafe { &*(value as *const Q as *const $struct_name<Q>) }
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> From<&mut Q> for &mut $struct_name<Q> {
            fn from(value: &mut Q) -> Self {
                // tRust me bro
                unsafe { &mut *(value as *mut Q as *mut $struct_name<Q>) }
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> AsRef<Q> for $struct_name<Q> {
            fn as_ref(&self) -> &Q {
                &self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> AsMut<Q> for $struct_name<Q> {
            fn as_mut(&mut self) -> &mut Q {
                &mut self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> core::ops::Add for $struct_name<Q> {
            type Output = Self;
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> core::ops::Sub for $struct_name<Q> {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> core::ops::Mul for $struct_name<Q> {
            type Output = Self;
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> core::ops::Div for $struct_name<Q> {
            type Output = Self;
            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> core::ops::AddAssign for $struct_name<Q> {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> core::ops::SubAssign for $struct_name<Q> {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> core::ops::MulAssign for $struct_name<Q> {
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> core::ops::DivAssign for $struct_name<Q> {
            fn div_assign(&mut self, rhs: Self) {
                self.0 /= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> Default for $struct_name<Q> {
            fn default() -> Self {
                Self(Q::default())
            }
        }

        ::paste::paste! {
            #[derive(Clone, Copy, Debug, PartialEq)]
            $vis enum [<$struct_name Enum>] {
                F16($struct_name<half::f16>),
                BF16($struct_name<half::bf16>),
                F32($struct_name<f32>),
                F64($struct_name<f64>),
                StorageF8($struct_name<$crate::values::quantizable::custom_data_types::StorageF8>),
            }

            impl [<$struct_name Enum>] {
                pub fn new_from_quantized<FromQuant: $crate::values::quantizable::QuantizedDecimalTrait>(
                    value: $struct_name<FromQuant>
                ) -> Self {
                    match FromQuant::LEVEL {
                        $crate::values::quantizable::DecimalQuantizationLevel::F16 => {
                            Self::F16($struct_name::<half::f16>::new(<half::f16 as $crate::values::quantizable::QuantizedDecimalTrait>::from_quantization(value.deref())))
                        }
                        $crate::values::quantizable::DecimalQuantizationLevel::BF16 => {
                            Self::BF16($struct_name::<half::bf16>::new(<half::bf16 as $crate::values::quantizable::QuantizedDecimalTrait>::from_quantization(value.deref())))
                        }
                        $crate::values::quantizable::DecimalQuantizationLevel::F32 => {
                            Self::F32($struct_name::<f32>::new(<f32 as $crate::values::quantizable::QuantizedDecimalTrait>::from_quantization(value.deref())))
                        }
                        $crate::values::quantizable::DecimalQuantizationLevel::F64 => {
                            Self::F64($struct_name::<f64>::new(<f64 as $crate::values::quantizable::QuantizedDecimalTrait>::from_quantization(value.deref())))
                        }
                        $crate::values::quantizable::DecimalQuantizationLevel::StorageF8 => {
                            Self::StorageF8(
                                $struct_name::<$crate::values::quantizable::custom_data_types::StorageF8>::new(
                                    <$crate::values::quantizable::custom_data_types::StorageF8 as $crate::values::quantizable::QuantizedDecimalTrait>::from_quantization(value.deref())
                                )
                            )
                        }
                    }
                }

                pub fn from_decimal_enum(value: $crate::values::quantizable::DecimalEnum) -> Self {
                    match value {
                        $crate::values::quantizable::DecimalEnum::F16(v) => {
                            Self::F16($struct_name::<half::f16>::new(v))
                        }
                        $crate::values::quantizable::DecimalEnum::BF16(v) => {
                            Self::BF16($struct_name::<half::bf16>::new(v))
                        }
                        $crate::values::quantizable::DecimalEnum::F32(v) => {
                            Self::F32($struct_name::<f32>::new(v))
                        }
                        $crate::values::quantizable::DecimalEnum::F64(v) => {
                            Self::F64($struct_name::<f64>::new(v))
                        }
                        $crate::values::quantizable::DecimalEnum::StorageF8(v) => {
                            Self::StorageF8(
                                $struct_name::<$crate::values::quantizable::custom_data_types::StorageF8>::new(v)
                            )
                        }
                    }
                }

                pub fn into_decimal_enum(self) -> $crate::values::quantizable::DecimalEnum {
                    match self {
                        Self::F16(v) => $crate::values::quantizable::DecimalEnum::F16(v.deref()),
                        Self::BF16(v) => $crate::values::quantizable::DecimalEnum::BF16(v.deref()),
                        Self::F32(v) => $crate::values::quantizable::DecimalEnum::F32(v.deref()),
                        Self::F64(v) => $crate::values::quantizable::DecimalEnum::F64(v.deref()),
                        Self::StorageF8(v) => $crate::values::quantizable::DecimalEnum::StorageF8(v.deref()),
                    }
                }

                pub fn into_wrapped_quant<Quant: $crate::values::quantizable::QuantizedDecimalTrait>(
                    self
                ) -> $struct_name<Quant> {
                    $struct_name::<Quant>::new(
                        <Self as $crate::values::quantizable::WrappedQuantizedDecimalEnum>::into_quant::<Quant>(self)
                    )
                }
            }

            impl $crate::values::quantizable::WrappedQuantizedDecimalEnum for [<$struct_name Enum>] {
                fn get_level(&self) -> $crate::values::quantizable::DecimalQuantizationLevel {
                    match self {
                        Self::F16(_) => $crate::values::quantizable::DecimalQuantizationLevel::F16,
                        Self::BF16(_) => $crate::values::quantizable::DecimalQuantizationLevel::BF16,
                        Self::F32(_) => $crate::values::quantizable::DecimalQuantizationLevel::F32,
                        Self::F64(_) => $crate::values::quantizable::DecimalQuantizationLevel::F64,
                        Self::StorageF8(_) => $crate::values::quantizable::DecimalQuantizationLevel::StorageF8,
                    }
                }

                fn into_quant<Quant: $crate::values::quantizable::QuantizedDecimalTrait>(self) -> Quant {
                    match self {
                        Self::F16(value) => Quant::from_quantization(value.deref()),
                        Self::BF16(value) => Quant::from_quantization(value.deref()),
                        Self::F32(value) => Quant::from_quantization(value.deref()),
                        Self::F64(value) => Quant::from_quantization(value.deref()),
                        Self::StorageF8(value) => Quant::from_quantization(value.deref()),
                    }
                }

                fn to_f64(self) -> f64 {
                    match self {
                        Self::F16(value) => <half::f16 as $crate::values::quantizable::QuantizedDecimalTrait>::quant_to_f64(value.deref()),
                        Self::BF16(value) => <half::bf16 as $crate::values::quantizable::QuantizedDecimalTrait>::quant_to_f64(value.deref()),
                        Self::F32(value) => <f32 as $crate::values::quantizable::QuantizedDecimalTrait>::quant_to_f64(value.deref()),
                        Self::F64(value) => value.deref(),
                        Self::StorageF8(value) => <$crate::values::quantizable::custom_data_types::StorageF8 as $crate::values::quantizable::QuantizedDecimalTrait>::quant_to_f64(value.deref()),
                    }
                }
            }
        }
    };
}
