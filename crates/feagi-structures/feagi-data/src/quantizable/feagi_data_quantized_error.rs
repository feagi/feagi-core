use crate::feagi_error::FeagiErrorBase;
use crate::quantizable::base_types::QuantizedIndexCountTrait;
use crate::quantizable::quantization_levels::QuantizationLevel;

#[derive(Debug)]
pub enum FeagiDataQuantizedError {
    QuantizationOverflowError{
        context: &'static str,
        limiting_quantization: QuantizationLevel,
        required_quantization: QuantizationLevel,
    },
    CollectionInvalidIndexError{
        context: &'static str,
        invalid_index: u32,
    },
    QuantizationEtcError{context: &'static str},
}

impl FeagiDataQuantizedError {
    /// Verify that the loading in data will not exceed the given space, if it does, error
    pub const fn verify_quantization_index<QuantIndexCount: QuantizedIndexCountTrait>(loading_data: usize,
                                     error_message: &'static str) -> Result<(), FeagiDataQuantizedError>
    {
        if loading_data < QuantIndexCount::MAX_AS_USIZE {
            return Ok(())
        }
        Err(FeagiDataQuantizedError::QuantizationOverflowError{
            context: error_message,
            limiting_quantization: QuantIndexCount::QUANTIZATION_LEVEL,
            required_quantization: QuantizationLevel::minimum_quantization_needed_for_usize(loading_data),
        })
        
    }
}

impl FeagiErrorBase for FeagiDataQuantizedError {}