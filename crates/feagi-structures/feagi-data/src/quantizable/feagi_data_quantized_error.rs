use feagi_logging_and_errors::{FeagiErrorKey, FeagiError, generate_feagi_error};
use crate::quantizable::base_types::QuantizedIndexCountTrait;
use crate::quantizable::quantization_levels::QuantizationLevel;

/// A value overflowed due to an insufficient quantization level
#[derive(FeagiErrorKey)]
pub struct QuantizationOverflowFeagiErrKey {
    context: &'static str,
    limiting_quantization: QuantizationLevel,
    required_quantization: QuantizationLevel,
}

/// Some other quantization related error
#[derive(FeagiErrorKey)]
pub struct QuantizationEtcFeagiErrKey {
    context: &'static str,
}


generate_feagi_error!{
    FeagiDataQuantizedError,
    keys: {
        QuantizationOverflowError: QuantizationOverflowFeagiErrKey,
        QuantizationEtcError: QuantizationEtcFeagiErrKey
    },
    sub_errors: {

    },
}



impl FeagiDataQuantizedError {
    /// Verify that the loading in data will not exceed the given space, if it does, error
    pub const fn verify_quantization_index<QuantIndexCount: QuantizedIndexCountTrait>(loading_data: usize,
                                     error_message: &'static str) -> Result<(), FeagiDataQuantizedError>
    {
        if loading_data < QuantIndexCount::QUANT_MAX_AS_USIZE {
            return Ok(())
        }
        Err(FeagiDataQuantizedError::QuantizationOverflowError(
                QuantizationOverflowFeagiErrKey::new(
                    error_message,
                    QuantIndexCount::QUANTIZATION_LEVEL,
                    QuantizationLevel::minimum_quantization_needed_for_usize(loading_data)
                )
            )
        )
    }
}
