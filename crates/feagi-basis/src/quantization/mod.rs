

/// Easy Import
pub mod prelude {
    pub use crate::quantization::quantizable::{
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

pub use crate::{
    create_wrapped_percentage_signed, create_wrapped_percentage_unsigned,
    create_wrapped_quantized_decimal, create_wrapped_quantized_signed_integer,
    create_wrapped_quantized_unsigned_integer,
};

pub use quantization_error::QuantizationError;

mod quantization_error;
