//! The three wrapper families built on top of [`QuantizedUnsignedIntegerTrait`].
//!
//! Every quantized unsigned integer in FEAGI is stored in one of three roles, and the role is
//! carried by the wrapper type rather than by the quantization:
//!
//! - **Index** ([`create_wrapped_quantized_index`]) addresses a position inside some collection.
//! - **Count** ([`create_wrapped_quantized_count`]) is a size, length or population.
//! - **Unsigned** ([`create_wrapped_quantized_unsigned`]) is a plain unsigned datum that is
//!   neither of the above, such as a limit or a countdown.
//!
//! Each family gets its own trait so a function can accept "any index" without also accepting any
//! count. The three traits are deliberately *not* unified under a common supertrait: instead each
//! wrapper derefs to the quantized integer it holds, so the shared value behaviour is reached
//! through [`core::ops::Deref`] rather than through a fourth trait that all three would have to
//! agree on.
//!
//! Only the operations that produce a `Self` live on the family traits, because an associated
//! function has no receiver to deref through and because a clamp that returned a bare integer
//! would silently discard the role. Everything that reads the value out
//! (`quant_to_usize`, `to_quantization`, and friends) is reached by deref and requires
//! [`QuantizedUnsignedIntegerTrait`] to be in scope at the call site.

use crate::values::quantizable::{FeagiDataValueQuantizationError, QuantizedUnsignedIntegerTrait, UnsignedIntegerQuantizationLevel};

/// Defines the value trait and the companion enum trait for one wrapper family.
///
/// The three families share a shape, so the traits are generated rather than written out three
/// times. See the module documentation for what distinguishes them.
macro_rules! define_unsigned_wrapper_family_traits {
    (
        $(#[$value_meta:meta])*
        $value_trait:ident,
        $(#[$enum_meta:meta])*
        $enum_trait:ident $(,)?
    ) => {
        $(#[$value_meta])*
        pub trait $value_trait:
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
            + core::ops::Deref<Target = Self::Quant>
            + From<Self::Quant>
            + AsRef<Self::Quant>
            + AsMut<Self::Quant>
            + Sized
            + 'static
        {
            /// The quantized unsigned integer this wrapper stores.
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

            /// Extracts the inner quantized value by copy.
            fn deref(self) -> Self::Quant;

            /// Converts from usize, does NOT check bounds!
            fn quant_from_usize(value: usize) -> Self {
                Self::new(Self::Quant::quant_from_usize(value))
            }

            /// Tries converting from usize, returns an error if out of bounds
            fn quant_try_from_usize(value: usize) -> Result<Self, FeagiDataValueQuantizationError> {
                Ok(Self::new(Self::Quant::quant_try_from_usize(value)?))
            }

            /// Creates from another quantization. Does not check for validity of ranges!
            fn from_quantization<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self {
                Self::new(Self::Quant::from_quantization(value))
            }

            /// Creates from another quantization, clamping the value to ensure it fits
            fn from_quantization_clamped<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self {
                Self::new(Self::Quant::from_quantization_clamped(value))
            }

            /// Tries to create from another quantization, returns an error if it would break the bounds
            fn try_from_quantization<FromQuant: QuantizedUnsignedIntegerTrait>(
                value: FromQuant,
            ) -> Result<Self, FeagiDataValueQuantizationError> {
                Ok(Self::new(Self::Quant::try_from_quantization(value)?))
            }

            /// Clamps the value for another quantization, but does not actually change the
            /// quantization itself
            fn clamp_for_quantization<ClampFor: QuantizedUnsignedIntegerTrait>(self) -> Self {
                Self::new(self.deref().clamp_for_quantization::<ClampFor>())
            }

            /// Clamps the value for a runtime-provided quantization level, but does not actually
            /// change the quantization itself
            fn clamp_for_quantization_level_runtime(self, level: UnsignedIntegerQuantizationLevel) -> Self {
                Self::new(self.deref().clamp_for_quantization_level_runtime(level))
            }
        }

        $(#[$enum_meta])*
        pub trait $enum_trait:
            Copy + Clone + Send + Sync + core::fmt::Debug + core::cmp::PartialEq + core::cmp::Eq + core::hash::Hash + Sized + 'static
        {
            fn get_level(&self) -> UnsignedIntegerQuantizationLevel;

            fn try_into_quant<Quant: QuantizedUnsignedIntegerTrait>(self) -> Result<Quant, FeagiDataValueQuantizationError>;

            fn into_quant<Quant: QuantizedUnsignedIntegerTrait>(self) -> Quant;

            fn to_usize(self) -> usize;
        }
    };
}

define_unsigned_wrapper_family_traits!(
    /// A position addressing an element of some collection. Generated by
    /// [`create_wrapped_quantized_index`].
    WrappedQuantizedIndex,
    /// Hides the quantization generic of a wrapped index behind concrete variants.
    WrappedQuantizedIndexEnum,
);

define_unsigned_wrapper_family_traits!(
    /// A size, length or population. Generated by [`create_wrapped_quantized_count`].
    WrappedQuantizedCount,
    /// Hides the quantization generic of a wrapped count behind concrete variants.
    WrappedQuantizedCountEnum,
);

define_unsigned_wrapper_family_traits!(
    /// A plain unsigned datum that is neither an index nor a count. Generated by
    /// [`create_wrapped_quantized_unsigned`].
    WrappedQuantizedUnsigned,
    /// Hides the quantization generic of a wrapped unsigned datum behind concrete variants.
    WrappedQuantizedUnsignedEnum,
);

/// Emits one wrapper type, its arithmetic and conversion surface, and its companion enum.
///
/// Not called directly: use [`create_wrapped_quantized_index`],
/// [`create_wrapped_quantized_count`] or [`create_wrapped_quantized_unsigned`], which pick the
/// family traits for you.
#[doc(hidden)]
#[macro_export]
macro_rules! create_wrapped_quantized_unsigned_family {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident,
        $value_trait:ident,
        $enum_trait:ident $(,)?
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

            /// Extracts the inner quantized value by copy.
            pub fn deref(self) -> Q {
                self.0
            }
        }

        // The rest of the wrapper's behaviour lives on the family trait, and everything that only
        // reads the value out is reached by deref to `Q`.
        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>
            $crate::values::quantizable::$value_trait for $struct_name<Q>
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

        // Read-only on purpose: a `DerefMut` would let a bare integer be assigned straight into
        // the wrapper, which is exactly the confusion these types exist to prevent.
        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> core::ops::Deref for $struct_name<Q> {
            type Target = Q;
            fn deref(&self) -> &Q {
                &self.0
            }
        }

        // NOTE: Into<Q> for $struct_name<Q> is not needed!

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> From<Q> for $struct_name<Q> {
            fn from(value: Q) -> Self {
                Self(value)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> From<&Q> for &$struct_name<Q> {
            fn from(value: &Q) -> Self {
                // tRust me bro
                unsafe { &*(value as *const Q as *const $struct_name<Q>) }
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> From<&mut Q> for &mut $struct_name<Q> {
            fn from(value: &mut Q) -> Self {
                // tRust me bro
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
                    match FromQuant::LEVEL {
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U8 => {
                            Self::U8($struct_name::<u8>::new(<u8 as $crate::values::quantizable::QuantizedUnsignedIntegerTrait>::from_quantization(value.deref())))
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U16 => {
                            Self::U16($struct_name::<u16>::new(<u16 as $crate::values::quantizable::QuantizedUnsignedIntegerTrait>::from_quantization(value.deref())))
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U32 => {
                            Self::U32($struct_name::<u32>::new(<u32 as $crate::values::quantizable::QuantizedUnsignedIntegerTrait>::from_quantization(value.deref())))
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U64
                        | $crate::values::quantizable::UnsignedIntegerQuantizationLevel::Usize => {
                            Self::U64($struct_name::<u64>::new(<u64 as $crate::values::quantizable::QuantizedUnsignedIntegerTrait>::from_quantization(value.deref())))
                        }
                    }
                }

                pub fn from_unsigned_integer_enum(value: $crate::values::quantizable::UnsignedIntegerEnum) -> Self {
                    match value {
                        $crate::values::quantizable::UnsignedIntegerEnum::U8(v) => Self::U8($struct_name::<u8>::new(v)),
                        $crate::values::quantizable::UnsignedIntegerEnum::U16(v) => Self::U16($struct_name::<u16>::new(v)),
                        $crate::values::quantizable::UnsignedIntegerEnum::U32(v) => Self::U32($struct_name::<u32>::new(v)),
                        $crate::values::quantizable::UnsignedIntegerEnum::U64(v) => Self::U64($struct_name::<u64>::new(v)),
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
                        <Self as $crate::values::quantizable::$enum_trait>::try_into_quant::<Quant>(self)?
                    ))
                }

                pub fn into_wrapped_quant<Quant: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $struct_name<Quant> {
                    $struct_name::<Quant>::new(
                        <Self as $crate::values::quantizable::$enum_trait>::into_quant::<Quant>(self)
                    )
                }
            }

            impl $crate::values::quantizable::$enum_trait for [<$struct_name Enum>] {
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
                        Self::U8(value) => Quant::try_from_quantization(value.deref()),
                        Self::U16(value) => Quant::try_from_quantization(value.deref()),
                        Self::U32(value) => Quant::try_from_quantization(value.deref()),
                        Self::U64(value) => Quant::try_from_quantization(value.deref()),
                    }
                }

                fn into_quant<Quant: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(self) -> Quant {
                    match self {
                        Self::U8(value) => Quant::from_quantization(value.deref()),
                        Self::U16(value) => Quant::from_quantization(value.deref()),
                        Self::U32(value) => Quant::from_quantization(value.deref()),
                        Self::U64(value) => Quant::from_quantization(value.deref()),
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

/// Creates a wrapper for a value that addresses a position inside a collection.
#[macro_export]
macro_rules! create_wrapped_quantized_index {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident
    ) => {
        $crate::create_wrapped_quantized_unsigned_family!(
            $(#[$meta])*
            $vis $struct_name,
            WrappedQuantizedIndex,
            WrappedQuantizedIndexEnum,
        );
    };
}

/// Creates a wrapper for a size, length or population.
#[macro_export]
macro_rules! create_wrapped_quantized_count {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident
    ) => {
        $crate::create_wrapped_quantized_unsigned_family!(
            $(#[$meta])*
            $vis $struct_name,
            WrappedQuantizedCount,
            WrappedQuantizedCountEnum,
        );
    };
}

/// Creates a wrapper for a plain unsigned datum that is neither an index nor a count.
#[macro_export]
macro_rules! create_wrapped_quantized_unsigned {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident
    ) => {
        $crate::create_wrapped_quantized_unsigned_family!(
            $(#[$meta])*
            $vis $struct_name,
            WrappedQuantizedUnsigned,
            WrappedQuantizedUnsignedEnum,
        );
    };
}
