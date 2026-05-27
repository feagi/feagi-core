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
        $($visibility)* struct $struct_name<QE: $crate::quantizable_linear::base_types::QuantizedIndexCountTrait>(QE);

        impl<QE: $crate::quantizable_linear::base_types::QuantizedIndexCountTrait>
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperIndexCount<QE>
            for $struct_name<QE>
        {

        }

        $crate::__impl_common_quantized_wrapper!(
            $struct_name,
            QE,
            $crate::quantizable_linear::base_types::QuantizedIndexCountTrait
        );
        $crate::__impl_supports_uint_ops!(
            $struct_name,
            QE,
            $crate::quantizable_linear::base_types::QuantizedIndexCountTrait
        );
        
        impl<QE: $crate::quantizable_linear::base_types::QuantizedIndexCountTrait>
            $crate::quantizable_linear::base_types::QuantizedIndexCountTrait
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

        impl<QE: $crate::quantizable_linear::base_types::QuantizedIndexCountTrait> $struct_name<QE>
        {
            #[inline(always)]
            pub const fn const_take(self) -> QE {
                self.0
            }
        }

    };
    ( @impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty ) => {

        #[repr(transparent)]
        #[derive(Copy, Clone, Default)]
        $($visibility)* struct $struct_name($quant_element);

        impl $crate::quantizable_linear::wrappers::QuantizedElementWrapperIndexCount<$quant_element>
            for $struct_name
        where
            $quant_element: $crate::quantizable_linear::base_types::QuantizedIndexCountTrait,
        {

        }

        $crate::__impl_common_quantized_wrapper_concrete!(
            $struct_name,
            $quant_element,
            $crate::quantizable_linear::base_types::QuantizedIndexCountTrait
        );
        $crate::__impl_supports_uint_ops_concrete!(
            $struct_name,
            $quant_element,
            $crate::quantizable_linear::base_types::QuantizedIndexCountTrait
        );
        
        impl $crate::quantizable_linear::base_types::QuantizedIndexCountTrait
            for $struct_name
        where
            $quant_element: $crate::quantizable_linear::base_types::QuantizedIndexCountTrait,
        {
            #[inline(always)]
            fn to_u32(self) -> u32 {
                self.0.to_u32()
            }

            #[inline(always)]
            fn from_u32(value: u32) -> Self {
                Self(<$quant_element as $crate::quantizable_linear::base_types::QuantizedIndexCountTrait>::from_u32(value))
            }

            #[inline(always)]
            fn from_u32_clamped(value: u32) -> Self {
                Self(<$quant_element as $crate::quantizable_linear::base_types::QuantizedIndexCountTrait>::from_u32_clamped(value))
            }
        }

        impl $struct_name
        {
            #[inline(always)]
            pub const fn const_take(self) -> $quant_element {
                self.0
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
        $($visibility)* struct $struct_name<QE: $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait>(QE);
        
        impl<QE> $crate::quantizable_linear::wrappers::QuantizedElementWrapperUnsignedInteger<QE> for $struct_name<QE>
        where
            QE: $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait,
        {
        }
        
        $crate::__impl_common_quantized_wrapper!(
            $struct_name,
            QE,
            $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait
                + $crate::SupportsUintOps
        );
        $crate::__impl_supports_uint_ops!(
            $struct_name,
            QE,
            $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait
                + $crate::SupportsUintOps
        );
        
        impl<QE> $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait for $struct_name<QE>
        where
            QE: $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait,
        {
        }

        impl<QE: $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait> $struct_name<QE>
        {
            #[inline(always)]
            pub const fn const_take(self) -> QE {
                self.0
            }
        }
        
    };
    ( @impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty ) => {
        
        #[repr(transparent)]
        #[derive(Copy, Clone, Default)]
        $($visibility)* struct $struct_name($quant_element);
        
        impl $crate::quantizable_linear::wrappers::QuantizedElementWrapperUnsignedInteger<$quant_element> for $struct_name
        where
            $quant_element: $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait,
        {
        }
        
        $crate::__impl_common_quantized_wrapper_concrete!(
            $struct_name,
            $quant_element,
            $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait
                + $crate::SupportsUintOps
        );
        $crate::__impl_supports_uint_ops_concrete!(
            $struct_name,
            $quant_element,
            $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait
                + $crate::SupportsUintOps
        );
        
        impl $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait for $struct_name
        where
            $quant_element: $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait,
        {
        }

        impl $struct_name
        {
            #[inline(always)]
            pub const fn const_take(self) -> $quant_element {
                self.0
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
        $($visibility)* struct $struct_name<QE: $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait>(QE);
        
        impl<QE> $crate::quantizable_linear::wrappers::QuantizedElementWrapperSignedInteger<QE> for $struct_name<QE>
        where
            QE: $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait
        {
        }
        
        $crate::__impl_common_quantized_wrapper!(
            $struct_name,
            QE,
            $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait
        );
        
        impl<QE> $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait for $struct_name<QE>
        where
            QE: $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait
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

        impl<QE: $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait> $struct_name<QE>
        {
            #[inline(always)]
            pub const fn const_take(self) -> QE {
                self.0
            }
        }
        
    };
    ( @impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty ) => {
        
        #[repr(transparent)]
        #[derive(Copy, Clone, Default)]
        $($visibility)* struct $struct_name($quant_element);
        
        impl $crate::quantizable_linear::wrappers::QuantizedElementWrapperSignedInteger<$quant_element> for $struct_name
        where
            $quant_element: $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait
        {
        }
        
        $crate::__impl_common_quantized_wrapper_concrete!(
            $struct_name,
            $quant_element,
            $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait
        );
        
        impl $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait for $struct_name
        where
            $quant_element: $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait
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
            pub const fn const_take(self) -> $quant_element {
                self.0
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
        $($visibility)* struct $struct_name<QE: $crate::quantizable_linear::base_types::QuantizedDecimalTrait>(QE);
        
        impl<QE: $crate::quantizable_linear::base_types::QuantizedDecimalTrait>
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperDecimal<QE>
            for $struct_name<QE>
        {
        }
        
        $crate::__impl_common_quantized_wrapper!(
            $struct_name,
            QE,
            $crate::quantizable_linear::base_types::QuantizedDecimalTrait
        );
        
        impl<QE: $crate::quantizable_linear::base_types::QuantizedDecimalTrait>
            $crate::quantizable_linear::base_types::QuantizedDecimalTrait
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

        impl<QE: $crate::quantizable_linear::base_types::QuantizedDecimalTrait> $struct_name<QE>
        {
            #[inline(always)]
            pub const fn const_take(self) -> QE {
                self.0
            }
        }
                
    };
    ( @impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty ) => {
        
        #[repr(transparent)]
        #[derive(Copy, Clone, Default)]
        $($visibility)* struct $struct_name($quant_element);
        
        impl $crate::quantizable_linear::wrappers::QuantizedElementWrapperDecimal<$quant_element>
            for $struct_name
        where
            $quant_element: $crate::quantizable_linear::base_types::QuantizedDecimalTrait,
        {
        }
        
        $crate::__impl_common_quantized_wrapper_concrete!(
            $struct_name,
            $quant_element,
            $crate::quantizable_linear::base_types::QuantizedDecimalTrait
        );
        
        impl $crate::quantizable_linear::base_types::QuantizedDecimalTrait
            for $struct_name
        where
            $quant_element: $crate::quantizable_linear::base_types::QuantizedDecimalTrait,
        {
            #[inline(always)]
            fn to_f32(self) -> f32 {
                self.0.to_f32()
            }
        
            #[inline(always)]
            fn from_f32(value: f32) -> Self {
                Self(<$quant_element as $crate::quantizable_linear::base_types::QuantizedDecimalTrait>::from_f32(value))
            }
        
            #[inline(always)]
            fn load_f32_inplace(&mut self, value: f32) {
                self.0.load_f32_inplace(value);
            }
        }

        impl $struct_name
        {
            #[inline(always)]
            pub const fn const_take(self) -> $quant_element {
                self.0
            }
        }
                
    };
}


