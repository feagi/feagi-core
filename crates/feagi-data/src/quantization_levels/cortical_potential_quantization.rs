use crate::values::quantizable::{DecimalQuantizationLevel, QuantizedDecimalTrait};


/// Defines the quantization of the neuron potential for a specific cortical area. All cortical
/// areas must have the neuron potential. This quantization is set per cortical area, and
/// is controlled by the Neuron Model quantization state, although this cortical level neuron
/// potential quantization has discrete steps that must be followed.
pub trait CorticalPotentialQuantization: Clone + Copy {
    /// All quantizations used by a given neuron model quantization level. Useful for validating
    /// device compatibility. This will also be extended in extensions of this trait
    const USED_QUANTIZATION_LEVELS: &'static [DecimalQuantizationLevel]; // Don't include a default, as we then forget about it

    /// Defines the quantization of the membrane potential of a neuron, which all models must
    /// include. This may vary between cortical areas, even of the same model. This also impacts
    /// the FCL as well
    type MembranePotentialQuant: QuantizedDecimalTrait;
}

//region Discrete Levels

#[derive(Debug, Clone, Copy)]
pub struct CorticalPotentialQuantizationFloat32;

impl CorticalPotentialQuantization for CorticalPotentialQuantizationFloat32 {
    //const CORTICAL_POTENTIAL_QUANTIZATION_LEVEL: DecimalQuantizationLevel = DecimalQuantizationLevel::F32;
    const USED_QUANTIZATION_LEVELS: &'static [DecimalQuantizationLevel] = &[DecimalQuantizationLevel::F32];
    type MembranePotentialQuant = f32;
}

#[derive(Debug, Clone, Copy)]
pub struct CorticalPotentialQuantizationFloat64;

impl CorticalPotentialQuantization for CorticalPotentialQuantizationFloat64 {
    //const CORTICAL_POTENTIAL_QUANTIZATION_LEVEL: DecimalQuantizationLevel = DecimalQuantizationLevel::F64;
    const USED_QUANTIZATION_LEVELS: &'static [DecimalQuantizationLevel] = &[DecimalQuantizationLevel::F64];
    type MembranePotentialQuant = f64;
}

//endregion
