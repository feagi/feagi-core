use core::hash::Hash;
use crate::values::quantizable::custom_data_types::StorageF8;
use crate::values::quantizable::{DecimalQuantizationLevel, QuantizedDecimalTrait, QuantizedDecimalUnwrappedTrait};
use half::{bf16, f16};

/// Defines the quantization of the neuron potential for a specific cortical area. All cortical
/// areas must have this property. This quantization is set per cortical area, and
/// is controlled by the Neuron Model Quantization state, although this cortical level neuron
/// potential quantization has discrete steps that must be followed.
pub trait MembranePotentialQuantization: Clone + Copy + Hash {
    /// Defines the quantization of the membrane potential of a neuron within a cortical area.
    type MembranePotentialQuant: QuantizedDecimalUnwrappedTrait; // TODO wrap this actually

    const MEMBRANE_POTENTIAL_QUANT_LEVEL: DecimalQuantizationLevel = Self::MembranePotentialQuant::LEVEL;
}

//region Discrete Levels

#[derive(Debug, Clone, Copy, Hash)]
pub struct CorticalMembranePotentialQuantizationStorageF8;

impl MembranePotentialQuantization for CorticalMembranePotentialQuantizationStorageF8 {
    type MembranePotentialQuant = StorageF8;
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct CorticalMembranePotentialQuantizationFloat16;

impl MembranePotentialQuantization for CorticalMembranePotentialQuantizationFloat16 {
    type MembranePotentialQuant = f16;
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct CorticalMembranePotentialQuantizationFloatB16;

impl MembranePotentialQuantization for CorticalMembranePotentialQuantizationFloatB16 {
    type MembranePotentialQuant = bf16;
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct CorticalMembranePotentialQuantizationFloat32;

impl MembranePotentialQuantization for CorticalMembranePotentialQuantizationFloat32 {
    type MembranePotentialQuant = f32;
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct CorticalMembranePotentialQuantizationFloat64;

impl MembranePotentialQuantization for CorticalMembranePotentialQuantizationFloat64 {
    type MembranePotentialQuant = f64;
}

pub type CorticalMembranePotentialQuantizationGenomic = CorticalMembranePotentialQuantizationFloat64;

//endregion
