use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use crate::values::quantizable::feagi_data_value_quantization_error::{FeagiDataValueQuantizationError, FeagiFailPercentageOutOfRange};
use crate::values::quantizable::{QuantizedDecimalTrait, QuantizedDecimalUnwrappedTrait};

/// Shared signed-percentage semantics for both [`PercentageSigned`] and wrapped newtypes.
///
/// Use this as a generic bound when a function should accept either
/// [`PercentageSigned`] or a wrapped newtype implementing
/// [`QuantizedSignedPercentageWrappedTrait`].
pub trait QuantizedSignedPercentageTrait:
    Copy
    + Clone
    + Send
    + Sync
    + Default
    + core::fmt::Debug
    + core::fmt::Display
    + core::cmp::PartialEq
    + core::cmp::PartialOrd
    + core::ops::Mul<Output = Self>
    + core::ops::Div<Output = Self>
    + core::ops::MulAssign
    + core::ops::DivAssign
    + Sized
    + Serialize
    + DeserializeOwned
    + 'static
{
    /// The underlying decimal quantization type this percentage stores.
    type DecimalQuant: QuantizedDecimalTrait;

    const NEGATIVE_HUNDRED_PERCENT: Self;
    const ZERO_PERCENT: Self;
    const HUNDRED_PERCENT: Self;

    /// Checks value is between -1.0 - 1.0 before creating itself as such.
    fn new_checked(value: Self::DecimalQuant) -> Result<Self, FeagiDataValueQuantizationError>;

    /// Enforces value is within range before returning.
    fn new_clamped(value: Self::DecimalQuant) -> Self;

    /// Creates percentage without checking if the value is within -1.0 - 1.0. Faster, but risks
    /// undefined behavior if used incorrectly!
    fn new_unchecked(value: Self::DecimalQuant) -> Self;

    /// Creates from a percentage using another decimal quantization.
    fn from_quantization<FromQuant: QuantizedDecimalTrait>(value: PercentageSigned<FromQuant>) -> Self;

    /// Converts to a percentage using another decimal quantization.
    fn to_quantization<ToQuant: QuantizedDecimalTrait>(self) -> PercentageSigned<ToQuant>;

    /// Returns the inner 0.0 - 1.0 decimal contained.
    fn get_decimal(self) -> Self::DecimalQuant;
}

/// Marker trait for unwrapped [`QuantizedSignedPercentageTrait`] values.
pub trait QuantizedSignedPercentageUnwrappedTrait: QuantizedSignedPercentageTrait {}


/// Internally uses a quantized decimal, but exposes methods to treat the value as a percentage
/// from 0–100% (0.0–1.0).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Serialize, Deserialize)]
#[serde(bound(deserialize = "D: QuantizedDecimalTrait"))]
pub struct PercentageSigned<D: QuantizedDecimalTrait>(D);

impl<D: QuantizedDecimalTrait> QuantizedSignedPercentageTrait for PercentageSigned<D> {
    type DecimalQuant = D;

    const NEGATIVE_HUNDRED_PERCENT: Self = Self(D::QUANT_NEGATIVE_ONE);
    const ZERO_PERCENT: Self = Self(D::QUANT_ZERO);
    const HUNDRED_PERCENT: Self = Self(D::QUANT_ONE);

    fn new_checked(value: D) -> Result<Self, FeagiDataValueQuantizationError> {
        if value < D::QUANT_NEGATIVE_ONE || value > D::QUANT_ONE {
            return Err(FeagiFailPercentageOutOfRange::new("Attempted to store out of range percentage!", value.quant_to_f32()).into());
        }
        Ok(Self(value))
    }

    fn new_clamped(value: D) -> Self {
        Self(value.quant_clamp(D::QUANT_ZERO, D::QUANT_ONE))
    }

    fn new_unchecked(value: D) -> Self {
        debug_assert!(
            value >= D::QUANT_ZERO && value <= D::QUANT_ONE,
            "Attempted to store out of range percentage!"
        );
        Self(value)
    }

    fn from_quantization<FromQuant: QuantizedDecimalTrait>(value: PercentageSigned<FromQuant>) -> Self {
        Self(value.get_decimal().to_quantization::<D>())
    }

    fn to_quantization<ToQuant: QuantizedDecimalTrait>(self) -> PercentageSigned<ToQuant> {
        PercentageSigned(self.0.to_quantization::<ToQuant>())
    }

    fn get_decimal(self) -> D {
        self.0
    }
}

impl<D: QuantizedDecimalUnwrappedTrait> QuantizedSignedPercentageUnwrappedTrait for PercentageSigned<D> {}

impl<D: QuantizedDecimalTrait> Default for PercentageSigned<D> {
    fn default() -> Self {
        Self::ZERO_PERCENT
    }
}

impl<D: QuantizedDecimalTrait> core::ops::Mul for PercentageSigned<D> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new_clamped(self.0 * rhs.0)
    }
}

impl<D: QuantizedDecimalTrait> core::ops::Div for PercentageSigned<D> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::new_clamped(self.0 / rhs.0)
    }
}

impl<D: QuantizedDecimalTrait> core::ops::MulAssign for PercentageSigned<D> {
    fn mul_assign(&mut self, rhs: Self) {
        *self = Self::new_clamped(self.0 * rhs.0);
    }
}

impl<D: QuantizedDecimalTrait> core::ops::DivAssign for PercentageSigned<D> {
    fn div_assign(&mut self, rhs: Self) {
        *self = Self::new_clamped(self.0 / rhs.0);
    }
}

impl<D: QuantizedDecimalTrait> core::fmt::Display for PercentageSigned<D> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}%", &self.0)
    }
}

/// Shared behaviour implemented by every strongly-typed wrapper generated by
/// [`create_wrapped_percentage_signed`].
///
/// Wrapper-specific behaviour is limited to [`Self::new`] and [`Self::dewrap`]; arithmetic and
/// conversion semantics come from [`QuantizedSignedPercentageTrait`].
pub trait QuantizedSignedPercentageWrappedTrait:
    QuantizedSignedPercentageTrait
    + From<PercentageSigned<Self::Quant>>
    + AsRef<PercentageSigned<Self::Quant>>
    + AsMut<PercentageSigned<Self::Quant>>
{
    /// The underlying unwrapped decimal quantization type.
    type Quant: QuantizedDecimalUnwrappedTrait;

    /// Wraps a raw percentage value into this wrapper type.
    fn new(value: PercentageSigned<Self::Quant>) -> Self;

    /// Extracts the inner percentage value.
    fn dewrap(self) -> PercentageSigned<Self::Quant>;
}

/// Creates a wrapper for signed percentage values.
#[macro_export]
macro_rules! create_wrapped_percentage_signed {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident
    ) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, ::serde::Serialize, ::serde::Deserialize)]
        #[serde(bound(deserialize = "Q: ::serde::de::DeserializeOwned"))]
        $vis struct $struct_name<Q: $crate::values::quantizable::QuantizedDecimalUnwrappedTrait>(
            $crate::values::quantizable::PercentageSigned<Q>
        );

        impl<Q: $crate::values::quantizable::QuantizedDecimalUnwrappedTrait> $struct_name<Q> {
            pub const NEGATIVE_HUNDRED_PERCENT: Self =
                Self::const_new($crate::values::quantizable::PercentageSigned::<Q>::NEGATIVE_HUNDRED_PERCENT);
            pub const ZERO_PERCENT: Self =
                Self::const_new($crate::values::quantizable::PercentageSigned::<Q>::ZERO_PERCENT);
            pub const HUNDRED_PERCENT: Self =
                Self::const_new($crate::values::quantizable::PercentageSigned::<Q>::HUNDRED_PERCENT);

            pub const fn const_new(value: $crate::values::quantizable::PercentageSigned<Q>) -> Self {
                Self(value)
            }

            pub const fn const_dewrap(self) -> $crate::values::quantizable::PercentageSigned<Q> {
                self.0
            }

            pub fn new(v: $crate::values::quantizable::PercentageSigned<Q>) -> Self {
                Self(v)
            }

            /// Extracts the inner percentage.
            pub fn dewrap(self) -> $crate::values::quantizable::PercentageSigned<Q> {
                self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalUnwrappedTrait>
            $crate::values::quantizable::QuantizedSignedPercentageTrait for $struct_name<Q>
        {
            type DecimalQuant = Q;

            const NEGATIVE_HUNDRED_PERCENT: Self =
                Self::const_new($crate::values::quantizable::PercentageSigned::<Q>::NEGATIVE_HUNDRED_PERCENT);
            const ZERO_PERCENT: Self =
                Self::const_new($crate::values::quantizable::PercentageSigned::<Q>::ZERO_PERCENT);
            const HUNDRED_PERCENT: Self =
                Self::const_new($crate::values::quantizable::PercentageSigned::<Q>::HUNDRED_PERCENT);

            fn new_checked(value: Q) -> Result<Self, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                Ok(Self::const_new(
                    $crate::values::quantizable::PercentageSigned::new_checked(value)?,
                ))
            }

            fn new_clamped(value: Q) -> Self {
                Self::const_new($crate::values::quantizable::PercentageSigned::new_clamped(value))
            }

            fn new_unchecked(value: Q) -> Self {
                Self::const_new($crate::values::quantizable::PercentageSigned::new_unchecked(value))
            }

            fn from_quantization<FromQuant: $crate::values::quantizable::QuantizedDecimalTrait>(
                value: $crate::values::quantizable::PercentageSigned<FromQuant>,
            ) -> Self {
                Self::const_new($crate::values::quantizable::PercentageSigned::from_quantization(value))
            }

            fn to_quantization<ToQuant: $crate::values::quantizable::QuantizedDecimalTrait>(
                self,
            ) -> $crate::values::quantizable::PercentageSigned<ToQuant> {
                self.0.to_quantization()
            }

            fn get_decimal(self) -> Q {
                self.0.get_decimal()
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalUnwrappedTrait>
            $crate::values::quantizable::QuantizedSignedPercentageWrappedTrait for $struct_name<Q>
        {
            type Quant = Q;

            fn new(value: $crate::values::quantizable::PercentageSigned<Q>) -> Self {
                Self(value)
            }

            fn dewrap(self) -> $crate::values::quantizable::PercentageSigned<Q> {
                self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalUnwrappedTrait>
            From<$crate::values::quantizable::PercentageSigned<Q>> for $struct_name<Q>
        {
            fn from(value: $crate::values::quantizable::PercentageSigned<Q>) -> Self {
                Self(value)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalUnwrappedTrait>
            From<&$crate::values::quantizable::PercentageSigned<Q>> for &$struct_name<Q>
        {
            fn from(value: &$crate::values::quantizable::PercentageSigned<Q>) -> Self {
                unsafe {
                    &*(value as *const $crate::values::quantizable::PercentageSigned<Q> as *const $struct_name<Q>)
                }
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalUnwrappedTrait>
            AsRef<$crate::values::quantizable::PercentageSigned<Q>> for $struct_name<Q>
        {
            fn as_ref(&self) -> &$crate::values::quantizable::PercentageSigned<Q> {
                &self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalUnwrappedTrait>
            AsMut<$crate::values::quantizable::PercentageSigned<Q>> for $struct_name<Q>
        {
            fn as_mut(&mut self) -> &mut $crate::values::quantizable::PercentageSigned<Q> {
                &mut self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalUnwrappedTrait> core::fmt::Display for $struct_name<Q> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::Display::fmt(&self.0, f)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalUnwrappedTrait> core::ops::Mul for $struct_name<Q> {
            type Output = Self;
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalUnwrappedTrait> core::ops::Div for $struct_name<Q> {
            type Output = Self;
            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalUnwrappedTrait> core::ops::MulAssign for $struct_name<Q> {
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalUnwrappedTrait> core::ops::DivAssign for $struct_name<Q> {
            fn div_assign(&mut self, rhs: Self) {
                self.0 /= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalUnwrappedTrait> Default for $struct_name<Q> {
            fn default() -> Self {
                Self($crate::values::quantizable::PercentageSigned::<Q>::default())
            }
        }
    };
}
