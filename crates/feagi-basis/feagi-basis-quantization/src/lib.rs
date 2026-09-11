

/// Singular values that are quantized
pub mod values;

/// Collections that are quantized, allow parallel mutable (unsafe) access
pub mod generic_collections;

/// Common levels of quantizations that is used throughout FEAGI
pub mod quantization_levels;
pub mod feagi_quantization_value;

/// Easy Import
pub mod prelude {
    pub use super::values::quantizable::{
        QuantizedUnsignedIntegerUnwrappedTrait, QuantizedUnsignedIntegerTrait,
        QuantizedSignedIntegerUnwrappedTrait, QuantizedSignedIntegerTrait,
        QuantizedDecimalUnwrappedTrait, QuantizedDecimalTrait
    };
    pub use super::quantization_levels::genome_quantization::{GenomeCoordinatesQuant, GenomeDimensionsQuant};
    
    pub use super::quantization_levels::feagi_index_quantization::{
        FeagiIndexQuantization, FeagiIndexQuantizationLevel, FeagiIndexQuantizationMini, 
        FeagiIndexQuantizationLow, FeagiIndexQuantizationStandard
    };
}