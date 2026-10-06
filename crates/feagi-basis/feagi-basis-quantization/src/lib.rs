

/// Easy Import
pub mod prelude {
    pub use crate::quantizable::{
        QuantizedDecimalTrait, QuantizedDecimalUnwrappedTrait,
        QuantizedElementBase, QuantizedSignedIntegerTrait,
        QuantizedSignedIntegerUnwrappedTrait, QuantizedUnsignedIntegerTrait, QuantizedUnsignedIntegerUnwrappedTrait
    };
    
    pub use super::{create_wrapped_percentage_unsigned, create_wrapped_quantized_decimal,
                    create_wrapped_quantized_signed_integer, create_wrapped_quantized_unsigned_integer
    };
}

/// Quantizable base elements
pub mod quantizable;

pub use feagi_basis_quantization_error::FeagiBasisQuantizationError;

mod feagi_basis_quantization_error;
