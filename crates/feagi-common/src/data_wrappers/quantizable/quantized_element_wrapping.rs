//! A Quantized Wrapper is effectively a struct that wraps around a Quantized Element Rust value.
//! This makes it easy to make custom implementations, extensions, or even alter base behavior.
//! Best to use the macros to generate these


#[macro_export]
/// Creates a Wrapped Index Count Wrapper struct of given name
macro_rules! create_quantized_index_count_wrapper  {
    ( $struct_name:ident ) => {
        $crate::create_quantized_index_count_wrapper!(@impl [pub] $struct_name);
    };
    ( $struct_name:ident, $quant_element:ty ) => {
        $crate::create_quantized_index_count_wrapper!(@impl_concrete [pub] $struct_name, $quant_element);
    };
    ( private $struct_name:ident ) => {
        $crate::create_quantized_index_count_wrapper!(@impl [] $struct_name);
    };
    ( private $struct_name:ident, $quant_element:ty ) => {
        $crate::create_quantized_index_count_wrapper!(@impl_concrete [] $struct_name, $quant_element);
    };
    ( $visibility:vis $struct_name:ident ) => {
        $crate::create_quantized_index_count_wrapper!(@impl [$visibility] $struct_name);
    };
    ( $visibility:vis $struct_name:ident, $quant_element:ty ) => {
        $crate::create_quantized_index_count_wrapper!(@impl_concrete [$visibility] $struct_name, $quant_element);
    };
    ( @impl [$($visibility:tt)*] $struct_name:ident ) => {

        #[repr(transparent)]
        #[derive(Copy, Clone, Default)]
        $($visibility)* struct $struct_name<QE: $crate::feagi_common_quantizable::QuantizedIndexCountTrait>(QE);

        impl<QE: $crate::feagi_common_quantizable::QuantizedIndexCountTrait>
            $crate::data_wrappers::quantizable::wrapper_traits::QuantizedElementWrapperIndexCount<QE>
            for $struct_name<QE>
        {

        }

        $crate::__impl_common_quantized_wrapper!(
            $struct_name,
            QE,
            $crate::feagi_common_quantizable::QuantizedIndexCountTrait
        );
        $crate::__impl_supports_uint_ops!(
            $struct_name,
            QE,
            $crate::feagi_common_quantizable::QuantizedIndexCountTrait
        );
        
        impl<QE: $crate::feagi_common_quantizable::QuantizedIndexCountTrait>
            $crate::feagi_common_quantizable::QuantizedIndexCountTrait
            for $struct_name<QE>
        {
            #[inline(always)]
            fn to_u32(self) -> u32 {
                self.0.to_u32()
            }

            #[inline(always)]
            fn from_u32(value: u32) -> Self {
                Self(QE::from_u32(value))
            }

            #[inline(always)]
            fn from_u32_clamped(value: u32) -> Self {
                Self(QE::from_u32_clamped(value))
            }
        }

        impl<QE: $crate::feagi_common_quantizable::QuantizedIndexCountTrait> $struct_name<QE>
        {
            #[inline(always)]
            pub const fn const_unwrap(self) -> QE {
                self.0
            }

            #[inline(always)]
            pub const fn const_wrap(e: QE) -> Self {
                Self(e)
            }
        }

    };
    ( @impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty ) => {

        #[repr(transparent)]
        #[derive(Copy, Clone, Default)]
        $($visibility)* struct $struct_name($quant_element);

        impl $crate::data_wrappers::quantizable::wrapper_traits::QuantizedElementWrapperIndexCount<$quant_element>
            for $struct_name
        where
            $quant_element: $crate::feagi_common_quantizable::QuantizedIndexCountTrait,
        {

        }

        $crate::__impl_common_quantized_wrapper_concrete!(
            $struct_name,
            $quant_element,
            $crate::feagi_common_quantizable::QuantizedIndexCountTrait
        );
        $crate::__impl_supports_uint_ops_concrete!(
            $struct_name,
            $quant_element,
            $crate::feagi_common_quantizable::QuantizedIndexCountTrait
        );
        
        impl $crate::feagi_common_quantizable::QuantizedIndexCountTrait
            for $struct_name
        where
            $quant_element: $crate::feagi_common_quantizable::QuantizedIndexCountTrait,
        {
            #[inline(always)]
            fn to_u32(self) -> u32 {
                self.0.to_u32()
            }

            #[inline(always)]
            fn from_u32(value: u32) -> Self {
                Self(<$quant_element as $crate::feagi_common_quantizable::QuantizedIndexCountTrait>::from_u32(value))
            }

            #[inline(always)]
            fn from_u32_clamped(value: u32) -> Self {
                Self(<$quant_element as $crate::feagi_common_quantizable::QuantizedIndexCountTrait>::from_u32_clamped(value))
            }
        }

        impl $struct_name
        {
            #[inline(always)]
            pub const fn const_unwrap(self) -> $quant_element {
                self.0
            }

            #[inline(always)]
            pub const fn const_wrap(e: $quant_element) -> Self {
                Self(e)
            }
        }

    };
}



#[macro_export]
/// Creates a Wrapped Unsigned Integer Wrapper struct of given name
macro_rules! create_quantized_unsigned_integer_wrapper {
    ( $struct_name:ident ) => {
        $crate::create_quantized_unsigned_integer_wrapper!(@impl [pub] $struct_name);
    };
    ( $struct_name:ident, $quant_element:ty ) => {
        $crate::create_quantized_unsigned_integer_wrapper!(@impl_concrete [pub] $struct_name, $quant_element);
    };
    ( private $struct_name:ident ) => {
        $crate::create_quantized_unsigned_integer_wrapper!(@impl [] $struct_name);
    };
    ( private $struct_name:ident, $quant_element:ty ) => {
        $crate::create_quantized_unsigned_integer_wrapper!(@impl_concrete [] $struct_name, $quant_element);
    };
    ( $visibility:vis $struct_name:ident ) => {
        $crate::create_quantized_unsigned_integer_wrapper!(@impl [$visibility] $struct_name);
    };
    ( $visibility:vis $struct_name:ident, $quant_element:ty ) => {
        $crate::create_quantized_unsigned_integer_wrapper!(@impl_concrete [$visibility] $struct_name, $quant_element);
    };
    ( @impl [$($visibility:tt)*] $struct_name:ident ) => {
        
        #[repr(transparent)]
        #[derive(Copy, Clone, Default)]
        $($visibility)* struct $struct_name<QE: $crate::feagi_common_quantizable::QuantizedUnsignedIntegerTrait>(QE);
        
        impl<QE> $crate::data_wrappers::quantizable::wrapper_traits::QuantizedElementWrapperUnsignedInteger<QE> for $struct_name<QE>
        where
            QE: $crate::feagi_common_quantizable::QuantizedUnsignedIntegerTrait,
        {
        }
        
        $crate::__impl_common_quantized_wrapper!(
            $struct_name,
            QE,
            $crate::feagi_common_quantizable::QuantizedUnsignedIntegerTrait
                + $crate::feagi_common_quantizable::shared_traits::SupportsUintOps
        );
        $crate::__impl_supports_uint_ops!(
            $struct_name,
            QE,
            $crate::feagi_common_quantizable::QuantizedUnsignedIntegerTrait
                + $crate::feagi_common_quantizable::shared_traits::SupportsUintOps
        );
        
        impl<QE> $crate::feagi_common_quantizable::QuantizedUnsignedIntegerTrait for $struct_name<QE>
        where
            QE: $crate::feagi_common_quantizable::QuantizedUnsignedIntegerTrait,
        {
        }

        impl<QE: $crate::feagi_common_quantizable::QuantizedUnsignedIntegerTrait> $struct_name<QE>
        {
            #[inline(always)]
            pub const fn const_unwrap(self) -> QE {
                self.0
            }

            #[inline(always)]
            pub const fn const_wrap(e: QE) -> Self {
                Self(e)
            }
        }
        
    };
    ( @impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty ) => {
        
        #[repr(transparent)]
        #[derive(Copy, Clone, Default)]
        $($visibility)* struct $struct_name($quant_element);
        
        impl $crate::data_wrappers::quantizable::wrapper_traits::QuantizedElementWrapperUnsignedInteger<$quant_element> for $struct_name
        where
            $quant_element: $crate::feagi_common_quantizable::QuantizedUnsignedIntegerTrait,
        {
        }
        
        $crate::__impl_common_quantized_wrapper_concrete!(
            $struct_name,
            $quant_element,
            $crate::feagi_common_quantizable::QuantizedUnsignedIntegerTrait
                + $crate::feagi_common_quantizable::shared_traits::SupportsUintOps
        );
        $crate::__impl_supports_uint_ops_concrete!(
            $struct_name,
            $quant_element,
            $crate::feagi_common_quantizable::QuantizedUnsignedIntegerTrait
                + $crate::feagi_common_quantizable::shared_traits::SupportsUintOps
        );
        
        impl $crate::feagi_common_quantizable::QuantizedUnsignedIntegerTrait for $struct_name
        where
            $quant_element: $crate::feagi_common_quantizable::QuantizedUnsignedIntegerTrait,
        {
        }

        impl $struct_name
        {
            #[inline(always)]
            pub const fn const_unwrap(self) -> $quant_element {
                self.0
            }

            #[inline(always)]
            pub const fn const_wrap(e: $quant_element) -> Self {
                Self(e)
            }
        }
        
    };
}



#[macro_export]
/// Creates a Wrapped Signed Integer Wrapper struct of given name
macro_rules! create_quantized_signed_integer_wrapper {
    ( $struct_name:ident ) => {
        $crate::create_quantized_signed_integer_wrapper!(@impl [pub] $struct_name);
    };
    ( $struct_name:ident, $quant_element:ty ) => {
        $crate::create_quantized_signed_integer_wrapper!(@impl_concrete [pub] $struct_name, $quant_element);
    };
    ( private $struct_name:ident ) => {
        $crate::create_quantized_signed_integer_wrapper!(@impl [] $struct_name);
    };
    ( private $struct_name:ident, $quant_element:ty ) => {
        $crate::create_quantized_signed_integer_wrapper!(@impl_concrete [] $struct_name, $quant_element);
    };
    ( $visibility:vis $struct_name:ident ) => {
        $crate::create_quantized_signed_integer_wrapper!(@impl [$visibility] $struct_name);
    };
    ( $visibility:vis $struct_name:ident, $quant_element:ty ) => {
        $crate::create_quantized_signed_integer_wrapper!(@impl_concrete [$visibility] $struct_name, $quant_element);
    };
    ( @impl [$($visibility:tt)*] $struct_name:ident ) => {
        
        #[repr(transparent)]
        #[derive(Copy, Clone, Default)]
        $($visibility)* struct $struct_name<QE: $crate::feagi_common_quantizable::QuantizedSignedIntegerTrait>(QE);
        
        impl<QE> $crate::data_wrappers::quantizable::wrapper_traits::QuantizedElementWrapperSignedInteger<QE> for $struct_name<QE>
        where
            QE: $crate::feagi_common_quantizable::QuantizedSignedIntegerTrait
        {
        }
        
        $crate::__impl_common_quantized_wrapper!(
            $struct_name,
            QE,
            $crate::feagi_common_quantizable::QuantizedSignedIntegerTrait
        );
        
        impl<QE> $crate::feagi_common_quantizable::QuantizedSignedIntegerTrait for $struct_name<QE>
        where
            QE: $crate::feagi_common_quantizable::QuantizedSignedIntegerTrait
        {
            #[inline(always)]
            fn is_negative(&self) -> bool {
                self.0.is_negative()
            }

            #[inline(always)]
            fn is_zero_or_negative(&self) -> bool {
                self.0.is_zero_or_negative()
            }
        }

        impl<QE: $crate::feagi_common_quantizable::QuantizedSignedIntegerTrait> $struct_name<QE>
        {
            #[inline(always)]
            pub const fn const_unwrap(self) -> QE {
                self.0
            }

            #[inline(always)]
            pub const fn const_wrap(e: QE) -> Self {
                Self(e)
            }
        }
        
    };
    ( @impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty ) => {
        
        #[repr(transparent)]
        #[derive(Copy, Clone, Default)]
        $($visibility)* struct $struct_name($quant_element);
        
        impl $crate::data_wrappers::quantizable::wrapper_traits::QuantizedElementWrapperSignedInteger<$quant_element> for $struct_name
        where
            $quant_element: $crate::feagi_common_quantizable::QuantizedSignedIntegerTrait
        {
        }
        
        $crate::__impl_common_quantized_wrapper_concrete!(
            $struct_name,
            $quant_element,
            $crate::feagi_common_quantizable::QuantizedSignedIntegerTrait
        );
        
        impl $crate::feagi_common_quantizable::QuantizedSignedIntegerTrait for $struct_name
        where
            $quant_element: $crate::feagi_common_quantizable::QuantizedSignedIntegerTrait
        {
            #[inline(always)]
            fn is_negative(&self) -> bool {
                self.0.is_negative()
            }

            #[inline(always)]
            fn is_zero_or_negative(&self) -> bool {
                self.0.is_zero_or_negative()
            }
        }

        impl $struct_name
        {
            #[inline(always)]
            pub const fn const_unwrap(self) -> $quant_element {
                self.0
            }

            #[inline(always)]
            pub const fn const_wrap(e: $quant_element) -> Self {
                Self(e)
            }
        }
        
    };
}



#[macro_export]
/// Creates a Wrapped Decimal Wrapper struct of given name
macro_rules! create_quantized_decimal_wrapper {
    ( $struct_name:ident ) => {
        $crate::create_quantized_decimal_wrapper!(@impl [pub] $struct_name);
    };
    ( $struct_name:ident, $quant_element:ty ) => {
        $crate::create_quantized_decimal_wrapper!(@impl_concrete [pub] $struct_name, $quant_element);
    };
    ( private $struct_name:ident ) => {
        $crate::create_quantized_decimal_wrapper!(@impl [] $struct_name);
    };
    ( private $struct_name:ident, $quant_element:ty ) => {
        $crate::create_quantized_decimal_wrapper!(@impl_concrete [] $struct_name, $quant_element);
    };
    ( $visibility:vis $struct_name:ident ) => {
        $crate::create_quantized_decimal_wrapper!(@impl [$visibility] $struct_name);
    };
    ( $visibility:vis $struct_name:ident, $quant_element:ty ) => {
        $crate::create_quantized_decimal_wrapper!(@impl_concrete [$visibility] $struct_name, $quant_element);
    };
    ( @impl [$($visibility:tt)*] $struct_name:ident ) => {
        
        #[repr(transparent)]
        #[derive(Copy, Clone, Default)]
        $($visibility)* struct $struct_name<QE: $crate::feagi_common_quantizable::QuantizedDecimalTrait>(QE);
        
        impl<QE: $crate::feagi_common_quantizable::QuantizedDecimalTrait>
            $crate::data_wrappers::quantizable::wrapper_traits::QuantizedElementWrapperDecimal<QE>
            for $struct_name<QE>
        {
        }
        
        $crate::__impl_common_quantized_wrapper!(
            $struct_name,
            QE,
            $crate::feagi_common_quantizable::QuantizedDecimalTrait
        );
        
        impl<QE: $crate::feagi_common_quantizable::QuantizedDecimalTrait>
            $crate::feagi_common_quantizable::QuantizedDecimalTrait
            for $struct_name<QE>
        {
            #[inline(always)]
            fn to_f32(self) -> f32 {
                self.0.to_f32()
            }
        
            #[inline(always)]
            fn from_f32(value: f32) -> Self {
                Self(QE::from_f32(value))
            }
        
            #[inline(always)]
            fn load_f32_inplace(&mut self, value: f32) {
                self.0.load_f32_inplace(value);
            }
        }

        impl<QE: $crate::feagi_common_quantizable::QuantizedDecimalTrait> $struct_name<QE>
        {
            #[inline(always)]
            pub const fn const_unwrap(self) -> QE {
                self.0
            }

            #[inline(always)]
            pub const fn const_wrap(e: QE) -> Self {
                Self(e)
            }
        }
                
    };
    ( @impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty ) => {
        
        #[repr(transparent)]
        #[derive(Copy, Clone, Default)]
        $($visibility)* struct $struct_name($quant_element);
        
        impl $crate::data_wrappers::quantizable::wrapper_traits::QuantizedElementWrapperDecimal<$quant_element>
            for $struct_name
        where
            $quant_element: $crate::feagi_common_quantizable::QuantizedDecimalTrait,
        {
        }
        
        $crate::__impl_common_quantized_wrapper_concrete!(
            $struct_name,
            $quant_element,
            $crate::feagi_common_quantizable::QuantizedDecimalTrait
        );
        
        impl $crate::feagi_common_quantizable::QuantizedDecimalTrait
            for $struct_name
        where
            $quant_element: $crate::feagi_common_quantizable::QuantizedDecimalTrait,
        {
            #[inline(always)]
            fn to_f32(self) -> f32 {
                self.0.to_f32()
            }
        
            #[inline(always)]
            fn from_f32(value: f32) -> Self {
                Self(<$quant_element as $crate::feagi_common_quantizable::QuantizedDecimalTrait>::from_f32(value))
            }
        
            #[inline(always)]
            fn load_f32_inplace(&mut self, value: f32) {
                self.0.load_f32_inplace(value);
            }
        }

        impl $struct_name
        {
            #[inline(always)]
            pub const fn const_unwrap(self) -> $quant_element {
                self.0
            }

            #[inline(always)]
            pub const fn const_wrap(e: $quant_element) -> Self {
                Self(e)
            }
        }
                
    };
}


