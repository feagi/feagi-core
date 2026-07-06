use crate::values::quantizable::QuantizedDecimalTrait;

//region Cortical Potential Quantization
/// Defines the quantization of the neuron potential for a specific cortical_area area. All cortical_area
/// areas must have the neuron potential. This quantization is set per cortical_area area, and
/// is controlled by the Neuron Model quantization state, although this cortical_area level neuron
/// potential quantization has discrete steps that must be followed.

pub trait CorticalPotentialQuantization {
    const QUANTIZATION_LEVEL: CorticalPotentialQuantizationLevel;

    /// Defines the quantization of the membrane potential of a neuron, which all models must
    /// include. This may vary between cortical_area areas, even of the same model. This also impacts
    /// the FCL as well
    type NeuronPotentialQuant: QuantizedDecimalTrait;
}

//region Discrete Levels

pub struct CorticalPotentialQuantizationFloat32;

impl CorticalPotentialQuantization for CorticalPotentialQuantizationFloat32 {
    const QUANTIZATION_LEVEL: CorticalPotentialQuantizationLevel = CorticalPotentialQuantizationLevel::Float32;
    type NeuronPotentialQuant = f32;
}

pub struct CorticalPotentialQuantizationFloat64;

impl CorticalPotentialQuantization for CorticalPotentialQuantizationFloat64 {
    const QUANTIZATION_LEVEL: CorticalPotentialQuantizationLevel = CorticalPotentialQuantizationLevel::Float64;
    type NeuronPotentialQuant = f64;
}

//endregion

#[repr(u8)]
#[derive(Default)]
pub enum CorticalPotentialQuantizationLevel {
    #[default]
    Float32 = 0,
    Float64 = 5,
    // TODO f16, f8, uint8, sint8, f64
}

