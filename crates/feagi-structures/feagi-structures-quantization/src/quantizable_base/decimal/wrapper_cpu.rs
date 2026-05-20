#[macro_export]
macro_rules! define_quantized_decimal_wrapper_cpu {
    (
        $(#[$struct_meta:meta])*
        $vis:vis struct $struct_name:ident
    ) => {
        $(#[$struct_meta])*
        #[repr(transparent)]
        #[derive(Copy, Clone, Default)]
        $vis struct $struct_name<QuantDeci: $crate::quantizable_base::decimal::QuantizedDecimalTrait>(QuantDeci);

        impl<QuantDeci: $crate::quantizable_base::decimal::QuantizedDecimalTrait>
            $crate::quantizable_base::decimal::QuantizedDecimalWrapperTrait<QuantDeci>
            for $struct_name<QuantDeci>
        {
            #[inline(always)]
            fn wrap_quant(quant: QuantDeci) -> Self {
                Self(quant)
            }

            #[inline(always)]
            fn quant(self) -> QuantDeci {
                self.0
            }

            #[inline(always)]
            fn quant_ref(&self) -> &QuantDeci {
                &self.0
            }

            #[inline(always)]
            fn quant_mut(&mut self) -> &mut QuantDeci {
                &mut self.0
            }
        }

        //region Math
        impl<QuantDeci: $crate::quantizable_base::decimal::QuantizedDecimalTrait> core::ops::Add
            for $struct_name<QuantDeci>
        {
            type Output = Self;

            #[inline(always)]
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<QuantDeci: $crate::quantizable_base::decimal::QuantizedDecimalTrait> core::ops::Sub
            for $struct_name<QuantDeci>
        {
            type Output = Self;

            #[inline(always)]
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl<QuantDeci: $crate::quantizable_base::decimal::QuantizedDecimalTrait> core::ops::Mul
            for $struct_name<QuantDeci>
        {
            type Output = Self;

            #[inline(always)]
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl<QuantDeci: $crate::quantizable_base::decimal::QuantizedDecimalTrait> core::ops::Div
            for $struct_name<QuantDeci>
        {
            type Output = Self;

            #[inline(always)]
            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }

        impl<QuantDeci: $crate::quantizable_base::decimal::QuantizedDecimalTrait> core::ops::AddAssign
            for $struct_name<QuantDeci>
        {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<QuantDeci: $crate::quantizable_base::decimal::QuantizedDecimalTrait> core::ops::SubAssign
            for $struct_name<QuantDeci>
        {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl<QuantDeci: $crate::quantizable_base::decimal::QuantizedDecimalTrait> core::ops::MulAssign
            for $struct_name<QuantDeci>
        {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        impl<QuantDeci: $crate::quantizable_base::decimal::QuantizedDecimalTrait> core::ops::DivAssign
            for $struct_name<QuantDeci>
        {
            #[inline(always)]
            fn div_assign(&mut self, rhs: Self) {
                self.0 /= rhs.0;
            }
        }

        impl<QuantDeci: $crate::quantizable_base::decimal::QuantizedDecimalTrait> core::cmp::PartialEq
            for $struct_name<QuantDeci>
        {
            #[inline(always)]
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl<QuantDeci: $crate::quantizable_base::decimal::QuantizedDecimalTrait> core::cmp::PartialOrd
            for $struct_name<QuantDeci>
        {
            #[inline(always)]
            fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                self.0.partial_cmp(&other.0)
            }
        }
        //endregion

        #[cfg(feature = "alloc")]
        impl<QuantDeci: $crate::quantizable_base::decimal::QuantizedDecimalTrait> core::fmt::Debug
            for $struct_name<QuantDeci>
        where
            QuantDeci: core::fmt::Debug,
        {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.debug_tuple(stringify!($struct_name)).field(&self.0).finish()
            }
        }

        #[cfg(feature = "alloc")]
        impl<QuantDeci: $crate::quantizable_base::decimal::QuantizedDecimalTrait> core::fmt::Display
            for $struct_name<QuantDeci>
        where
            QuantDeci: core::fmt::Display,
        {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::Display::fmt(&self.0, formatter)
            }
        }
    };
}
