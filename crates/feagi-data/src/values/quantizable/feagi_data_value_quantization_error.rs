use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};

#[derive(FeagiErrorKey)]
/// Tried bringing a value into quantization that was not in possible range of quantization
pub struct FeagiFailQuantizationOutOfRange {
    given_index: usize,
    max_quantization_index: usize,
    context: &'static str,
}

#[derive(FeagiErrorKey)]
/// Represents some general issue with quantization
pub struct FeagiFailInvalidQuantization {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
/// Attempted to use a quantization level on a device that does not support it
pub struct FeagiFailHardwareNoLikeQuant { // :3
    context: &'static str,
    hardware_type: &'static str, // TODO maybe this should be an enum?
    attempted_quant: &'static str,
}

#[derive(FeagiErrorKey)]
/// Attempted to index an array or something and the index was out of range
pub struct FeagiFailInvalidLinearIndex {
    given_index: usize,
    max_index_accepted: usize,
    context: &'static str,
}

#[derive(FeagiErrorKey)]
/// Attempted to store a percentage value that was not in range
pub struct FeagiFailPercentageOutOfRange {
    context: &'static str,
    attempted_percentage: f32,
}

generate_feagi_error! {
    /// Error related to a quantized value
    FeagiDataValueQuantizationError,
    keys: {
        QuantizationOutOfRange: FeagiFailQuantizationOutOfRange,
        InvalidQuantization: FeagiFailInvalidQuantization,
        IncompatibleHardware: FeagiFailHardwareNoLikeQuant,
        InvalidLinearIndex: FeagiFailInvalidLinearIndex,
        PercentageOutOfRange: FeagiFailPercentageOutOfRange,

    },
    sub_errors: {

    },
}
