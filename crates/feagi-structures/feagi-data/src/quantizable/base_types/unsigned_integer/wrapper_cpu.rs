

#[macro_export]
macro_rules! define_quantized_unsigned_integer_wrapper_cpu {
    (
        $(#[$struct_meta:meta])*
        $vis:vis struct $struct_name:ident
    ) => {
        $(#[$struct_meta])*
        #[repr(transparent)]
        #[derive(Copy, Clone, Default)]
        $vis struct $struct_name<QuantIndex: $crate::quantizable_types::QuantizedUnsignedIntegerTrait>(QuantIndex);

        impl<QuantIndex: $crate::quantizable_types::QuantizedUnsignedIntegerTrait>
            $crate::quantizable_types::QuantizedUnsignedIntegerWrapperTrait<QuantIndex>
            for $struct_name<QuantIndex>
        {
            #[inline(always)]
            fn wrap_quant(quant: QuantIndex) -> Self {
                Self(quant)
            }

            #[inline(always)]
            fn quant(self) -> QuantIndex {
                self.0
            }

            #[inline(always)]
            fn quant_ref(&self) -> &QuantIndex {
                &self.0
            }

            #[inline(always)]
            fn quant_mut(&mut self) -> &mut QuantIndex {
                &mut self.0
            }
        }

        //region Math
        impl<QuantIndex: $crate::quantizable_types::QuantizedUnsignedIntegerTrait> core::ops::Add
            for $struct_name<QuantIndex>
        {
            type Output = Self;

            #[inline(always)]
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<QuantIndex: $crate::quantizable_types::QuantizedUnsignedIntegerTrait> core::ops::Sub
            for $struct_name<QuantIndex>
        {
            type Output = Self;

            #[inline(always)]
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl<QuantIndex: $crate::quantizable_types::QuantizedUnsignedIntegerTrait> core::ops::Mul
            for $struct_name<QuantIndex>
        {
            type Output = Self;

            #[inline(always)]
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl<QuantIndex: $crate::quantizable_types::QuantizedUnsignedIntegerTrait> core::ops::Div
            for $struct_name<QuantIndex>
        {
            type Output = Self;

            #[inline(always)]
            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }

        impl<QuantIndex: $crate::quantizable_types::QuantizedUnsignedIntegerTrait> core::ops::Rem
            for $struct_name<QuantIndex>
        {
            type Output = Self;

            #[inline(always)]
            fn rem(self, rhs: Self) -> Self::Output {
                Self(self.0 % rhs.0)
            }
        }

        impl<QuantIndex: $crate::quantizable_types::QuantizedUnsignedIntegerTrait> core::ops::AddAssign
            for $struct_name<QuantIndex>
        {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<QuantIndex: $crate::quantizable_types::QuantizedUnsignedIntegerTrait> core::ops::SubAssign
            for $struct_name<QuantIndex>
        {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl<QuantIndex: $crate::quantizable_types::QuantizedUnsignedIntegerTrait> core::ops::MulAssign
            for $struct_name<QuantIndex>
        {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        impl<QuantIndex: $crate::quantizable_types::QuantizedUnsignedIntegerTrait> core::ops::DivAssign
            for $struct_name<QuantIndex>
        {
            #[inline(always)]
            fn div_assign(&mut self, rhs: Self) {
                self.0 /= rhs.0;
            }
        }

        impl<QuantIndex: $crate::quantizable_types::QuantizedUnsignedIntegerTrait> core::ops::RemAssign
            for $struct_name<QuantIndex>
        {
            #[inline(always)]
            fn rem_assign(&mut self, rhs: Self) {
                self.0 %= rhs.0;
            }
        }

        impl<QuantIndex: $crate::quantizable_types::QuantizedUnsignedIntegerTrait> core::cmp::PartialEq
            for $struct_name<QuantIndex>
        {
            #[inline(always)]
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl<QuantIndex: $crate::quantizable_types::QuantizedUnsignedIntegerTrait> core::cmp::PartialOrd
            for $struct_name<QuantIndex>
        {
            #[inline(always)]
            fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                self.0.partial_cmp(&other.0)
            }
        }
        //endregion

        #[cfg(feature = "alloc")]
        impl<QuantIndex: $crate::quantizable_types::QuantizedUnsignedIntegerTrait> core::fmt::Debug
            for $struct_name<QuantIndex>
        where
            QuantIndex: core::fmt::Debug,
        {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.debug_tuple(stringify!($struct_name)).field(&self.0).finish()
            }
        }

        #[cfg(feature = "alloc")]
        impl<QuantIndex: $crate::quantizable_types::QuantizedUnsignedIntegerTrait> core::fmt::Display
            for $struct_name<QuantIndex>
        where
            QuantIndex: core::fmt::Display,
        {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::Display::fmt(&self.0, formatter)
            }
        }
    };
}