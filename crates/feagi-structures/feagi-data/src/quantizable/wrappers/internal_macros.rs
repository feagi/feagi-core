// These are internal, generated because they are annoying to write
// Users need not interact with these

#[doc(hidden)]
#[macro_export]
/// Internal use, for generating quantized wrappers
macro_rules! __impl_quantized_element_wrapper_base {
    ($wrapper_type:ident, $quant_element:ident, $($quant_bound:tt)+) => {
        impl<$quant_element> $crate::quantizable::wrappers::QuantizedElementWrapperBase<$quant_element> for $wrapper_type<$quant_element>
        where
            $quant_element: $($quant_bound)+,
        {
            #[inline(always)]
            fn wrap(quantizable: $quant_element) -> Self {
                Self(quantizable)
            }

            #[inline(always)]
            fn unwrap(self) -> $quant_element {
                self.0
            }

            #[inline(always)]
            fn quant_ref(&self) -> &$quant_element {
                &self.0
            }

            #[inline(always)]
            fn quant_ref_mut(&mut self) -> &mut $quant_element {
                &mut self.0
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
/// Internal use, for generating quantized wrappers
macro_rules! __impl_quantized_element_base {
    ($wrapper_type:ident, $quant_element:ident, $($quant_bound:tt)+) => {
        impl<$quant_element> $crate::quantizable::base_types::QuantizedElementBase for $wrapper_type<$quant_element>
        where
            $quant_element: $($quant_bound)+,
        {
            const QUANTIZATION_LEVEL: $crate::quantizable::QuantizationLevel =
                <$quant_element as $crate::quantizable::base_types::QuantizedElementBase>::QUANTIZATION_LEVEL;
            const QUANT_ZERO: Self =
                Self(<$quant_element as $crate::quantizable::base_types::QuantizedElementBase>::QUANT_ZERO);
        }
    };
}

#[doc(hidden)]
#[macro_export]
/// Internal use, for generating quantized wrappers
macro_rules! __impl_supports_basic_core_math_ops {
    ($wrapper_type:ident, $quant_element:ident, $($quant_bound:tt)+) => {
        impl<$quant_element> core::ops::Add for $wrapper_type<$quant_element>
        where
            $quant_element: $($quant_bound)+,
        {
            type Output = Self;

            #[inline(always)]
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<$quant_element> core::ops::Sub for $wrapper_type<$quant_element>
        where
            $quant_element: $($quant_bound)+,
        {
            type Output = Self;

            #[inline(always)]
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl<$quant_element> core::ops::Mul for $wrapper_type<$quant_element>
        where
            $quant_element: $($quant_bound)+,
        {
            type Output = Self;

            #[inline(always)]
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl<$quant_element> core::ops::Div for $wrapper_type<$quant_element>
        where
            $quant_element: $($quant_bound)+,
        {
            type Output = Self;

            #[inline(always)]
            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }

        impl<$quant_element> core::ops::AddAssign for $wrapper_type<$quant_element>
        where
            $quant_element: $($quant_bound)+,
        {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<$quant_element> core::ops::SubAssign for $wrapper_type<$quant_element>
        where
            $quant_element: $($quant_bound)+,
        {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl<$quant_element> core::ops::MulAssign for $wrapper_type<$quant_element>
        where
            $quant_element: $($quant_bound)+,
        {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        impl<$quant_element> core::ops::DivAssign for $wrapper_type<$quant_element>
        where
            $quant_element: $($quant_bound)+,
        {
            #[inline(always)]
            fn div_assign(&mut self, rhs: Self) {
                self.0 /= rhs.0;
            }
        }

        impl<$quant_element> core::cmp::PartialEq for $wrapper_type<$quant_element>
        where
            $quant_element: $($quant_bound)+,
        {
            #[inline(always)]
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl<$quant_element> core::cmp::PartialOrd for $wrapper_type<$quant_element>
        where
            $quant_element: $($quant_bound)+,
        {
            #[inline(always)]
            fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                self.0.partial_cmp(&other.0)
            }
        }

        impl<$quant_element> $crate::SupportsBasicCoreMathOps for $wrapper_type<$quant_element>
        where
            $quant_element: $($quant_bound)+,
        {
        }
    };
}

#[doc(hidden)]
#[macro_export]
/// Internal use, for generating quantized wrappers
macro_rules! __impl_alloc_formatting {
    ($wrapper_type:ident, $quant_element:ident, $($quant_bound:tt)+) => {
        impl<$quant_element> core::fmt::Debug for $wrapper_type<$quant_element>
        where
            $quant_element: $($quant_bound)+ + core::fmt::Debug,
        {
            #[inline(always)]
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.debug_tuple(stringify!($wrapper_type)).field(&self.0).finish()
            }
        }

        impl<$quant_element> core::fmt::Display for $wrapper_type<$quant_element>
        where
            $quant_element: $($quant_bound)+ + core::fmt::Display,
        {
            #[inline(always)]
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::Display::fmt(&self.0, formatter)
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
/// Internal use, for generating quantized wrappers
macro_rules! __impl_supports_uint_ops {
    ($wrapper_type:ident, $quant_element:ident, $($quant_bound:tt)+) => {
        impl<$quant_element> core::ops::Rem for $wrapper_type<$quant_element>
        where
            $quant_element: $($quant_bound)+,
        {
            type Output = Self;

            #[inline(always)]
            fn rem(self, rhs: Self) -> Self::Output {
                Self(self.0 % rhs.0)
            }
        }

        impl<$quant_element> core::ops::RemAssign for $wrapper_type<$quant_element>
        where
            $quant_element: $($quant_bound)+,
        {
            #[inline(always)]
            fn rem_assign(&mut self, rhs: Self) {
                self.0 %= rhs.0;
            }
        }

        impl<$quant_element> $crate::SupportsUintOps for $wrapper_type<$quant_element>
        where
            $quant_element: $($quant_bound)+,
        {
            const QUANT_MAX_AS_USIZE: usize =
                <$quant_element as $crate::SupportsUintOps>::QUANT_MAX_AS_USIZE;
            const QUANT_ZERO: Self =
                Self(<$quant_element as $crate::SupportsUintOps>::QUANT_ZERO);
            const QUANT_ONE: Self =
                Self(<$quant_element as $crate::SupportsUintOps>::QUANT_ONE);

            #[inline(always)]
            fn from_usize_unchecked(u: usize) -> Self {
                Self(<$quant_element as $crate::SupportsUintOps>::from_usize_unchecked(u))
            }

            #[inline(always)]
            fn from_usize_clamped(u: usize) -> Self {
                Self(<$quant_element as $crate::SupportsUintOps>::from_usize_clamped(u))
            }

            #[inline(always)]
            fn to_usize(self) -> usize {
                <$quant_element as $crate::SupportsUintOps>::to_usize(self.0)
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
/// Internal use, for generating quantized wrappers
macro_rules! __impl_common_quantized_wrapper {
    ($wrapper_type:ident, $quant_element:ident, $($quant_bound:tt)+) => {
        $crate::__impl_quantized_element_base!($wrapper_type, $quant_element, $($quant_bound)+);
        $crate::__impl_supports_basic_core_math_ops!($wrapper_type, $quant_element, $($quant_bound)+);
        $crate::__impl_alloc_formatting!($wrapper_type, $quant_element, $($quant_bound)+);
        $crate::__impl_quantized_element_wrapper_base!($wrapper_type, $quant_element, $($quant_bound)+);
    };
}

