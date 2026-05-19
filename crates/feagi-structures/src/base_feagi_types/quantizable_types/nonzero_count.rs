use crate::base_feagi_types::quantizable_types::QuantizableUIntType;

/// Add constructors for a quantizable uint that block the use of zero
#[macro_export]
macro_rules! add_non_zero_constructors_to_quant_uint{
    ($struct_name:ident) => {

        // TODO custom error message?

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> $struct_name<T> {

            /// Creates a "nonzero" struct without checking the value actually isnt zero
            pub const fn new_unchecked(value: T) -> Self {
                Self(value)
            }
            
            /// Verifies a value is nonzero before creating the struct
            pub fn new_non_zero(value: T) -> Result<Self, $crate::FeagiStructuresError>  {
                Self::verify_not_zero(value)?;
                Ok(Self(value))
            }

            /// Verifies a value is nonzero before creating the struct
            pub fn new(value: T) -> Result<Self, $crate::FeagiStructuresError>  {
                Self::new_non_zero(value)
            }
            
            /// Only verifies value is nonzero during debug
            pub fn new_verify_nonzero_if_debug(value: T) -> Result<Self, $crate::FeagiStructuresError>  {
                // TODO debug only
                Self::verify_not_zero(value)?;

                Ok(Self(value))
            }
            
            /// Verifies a value is nonzero before updating the struct
            pub fn update_non_zero(&mut self, value: T) -> Result<(), $crate::FeagiStructuresError>  {
                Self::verify_not_zero(value)?;
                self.0 = value;
                Ok(())
            }
            
            /// Only verifies value is nonzero during debug
            pub fn update_verify_nonzero_if_debug(&mut self, value: T) -> Result<(), $crate::FeagiStructuresError>  {
                // TODO debug only
                Self::verify_not_zero(value)?;
                self.0 = value;
                Ok(())
            }

            pub const fn get(self) -> T {
                self.0
            }

            fn verify_not_zero(value: T) -> Result<(), $crate::FeagiStructuresError>
            {
                if value == T::ZERO {
                    return Err($crate::FeagiStructuresError::InvalidValue{
                        context: "Given unsigned integer cannot be zero!",
                    })
                }
                Ok(())
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> $crate::base_feagi_types::quantizable_types::NonZeroQuantizableUIntType for $struct_name<T> {
            // nothing lol. Enjoy your trait!
        }

    };
}

/// Trait added to QuantizableUIntType Structs (not the quantization itself) that
/// is added to signify that a value should not be zero. No actual enforcement
pub trait NonZeroQuantizableUIntType:
QuantizableUIntType
{
    // The trait doesnt actually do anything. its just a glorified tag
}
