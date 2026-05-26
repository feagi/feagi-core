//! A Quantized Wrapper is effectively a struct that wraps around a Quantized Element Rust value.
//! This makes it easy to make custom implementations, extensions, or even alter base behavior.
//! Best to use the macros to generate these


#[macro_export]
/// Creates a Wrapped Index Count Wrapper struct of given name
macro_rules! create_quantized_index_count_wrapper  {
    ( $struct_name:ident ) => {

        #[repr(transparent)]
        #[derive(Copy, Clone, Default)]
        pub struct $struct_name<QE: $crate::quantizable_linear::base_types::QuantizedIndexCountTrait>(QE);

        impl<QE: $crate::quantizable_linear::base_types::QuantizedIndexCountTrait>
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperIndexCount<QE>
            for $struct_name<QE>
        {
        }

        $crate::__impl_common_quantized_wrapper!(
            $struct_name,
            QE,
            $crate::quantizable::base_types::QuantizedIndexCountTrait
        );
        $crate::__impl_supports_uint_ops!(
            $struct_name,
            QE,
            $crate::quantizable::base_types::QuantizedIndexCountTrait
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

    };
}



#[macro_export]
/// Creates a Wrapped Unsigned Integer Wrapper struct of given name
macro_rules! create_quantized_unsigned_integer_wrapper {
    ( $struct_name:ident ) => {
        
        #[repr(transparent)]
        #[derive(Copy, Clone, Default)]
        pub struct $struct_name<QE: $crate::quantizable_linear::base_types::QuantizedIndexCountTrait>(QE);
        
        impl<QE> $crate::quantizable_linear::wrappers::QuantizedElementWrapperUnsignedInteger<QE> for $struct_name<QE>
        where
            QE: $crate::quantizable_linear::base_types::QuantizedIndexCountTrait
                + $crate::SupportsUintOps,
        {
        }
        
        $crate::__impl_common_quantized_wrapper!(
            $struct_name,
            QE,
            $crate::quantizable::base_types::QuantizedUnsignedIntegerTrait
                + $crate::SupportsUintOps
        );
        $crate::__impl_supports_uint_ops!(
            $struct_name,
            QE,
            $crate::quantizable::base_types::QuantizedUnsignedIntegerTrait
                + $crate::SupportsUintOps
        );
        
        impl<QE> $crate::quantizable_linear::base_types::QuantizedIndexCountTrait for $struct_name<QE>
        where
            QE: $crate::quantizable_linear::base_types::QuantizedIndexCountTrait
                + $crate::SupportsUintOps,
        {
        }
        
    };
}



#[macro_export]
/// Creates a Wrapped Signed Integer Wrapper struct of given name
macro_rules! create_quantized_signed_integer_wrapper {
    ( $struct_name:ident ) => {
        
        #[repr(transparent)]
        #[derive(Copy, Clone, Default)]
        pub struct $struct_name<QE: $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait>(QE);
        
        impl<QE> $crate::quantizable_linear::wrappers::QuantizedElementWrapperSignedInteger<QE> for $struct_name<QE>
        where
            QE: $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait
        {
        }
        
        $crate::__impl_common_quantized_wrapper!(
            $struct_name,
            QE,
            $crate::quantizable::base_types::QuantizedSignedIntegerTrait
        );
        
        impl<QE> $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait for $struct_name<QE>
        where
            QE: $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait
        {
        }
        
    };
}



#[macro_export]
/// Creates a Wrapped Decimal Wrapper struct of given name
macro_rules! create_quantized_decimal_wrapper {
    ( $struct_name:ident ) => {
        
        #[repr(transparent)]
        #[derive(Copy, Clone, Default)]
        pub struct $struct_name<QE: $crate::quantizable_linear::base_types::QuantizedDecimalTrait>(QE);
        
        impl<QE: $crate::quantizable_linear::base_types::QuantizedDecimalTrait>
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperDecimal<QE>
            for $struct_name<QE>
        {
        }
        
        $crate::__impl_common_quantized_wrapper!(
            $struct_name,
            QE,
            $crate::quantizable::base_types::QuantizedDecimalTrait
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
                
            };
}


