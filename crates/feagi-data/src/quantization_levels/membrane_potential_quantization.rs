use half::{bf16, f16};
use crate::values::quantizable::{DecimalQuantizationLevel, QuantizedDecimalTrait};
use crate::values::quantizable::custom_data_types::StorageF8;

/// Defines the quantization of the neuron potential for a specific cortical area. All cortical
/// areas must have this property. This quantization is set per cortical area, and
/// is controlled by the Neuron Model Quantization state, although this cortical level neuron
/// potential quantization has discrete steps that must be followed.
pub trait MembranePotentialQuantization: Clone + Copy {
    /// Defines the quantization of the membrane potential of a neuron within a cortical area.
    type MembranePotentialQuant: QuantizedDecimalTrait;
}

//region Discrete Levels

#[derive(Debug, Clone, Copy)]
pub struct CorticalMembranePotentialQuantizationStorageF8;

impl MembranePotentialQuantization for CorticalMembranePotentialQuantizationStorageF8 {
    type MembranePotentialQuant = StorageF8;
}

#[derive(Debug, Clone, Copy)]
pub struct CorticalMembranePotentialQuantizationFloat16;

impl MembranePotentialQuantization for CorticalMembranePotentialQuantizationFloat16 {
    type MembranePotentialQuant = f16;
}

#[derive(Debug, Clone, Copy)]
pub struct CorticalMembranePotentialQuantizationFloatB16;

impl MembranePotentialQuantization for CorticalMembranePotentialQuantizationFloatB16 {
    type MembranePotentialQuant = bf16;
}

#[derive(Debug, Clone, Copy)]
pub struct CorticalMembranePotentialQuantizationFloat32;

impl MembranePotentialQuantization for CorticalMembranePotentialQuantizationFloat32 {
    type MembranePotentialQuant = f32;
}

#[derive(Debug, Clone, Copy)]
pub struct CorticalMembranePotentialQuantizationFloat64;

impl MembranePotentialQuantization for CorticalMembranePotentialQuantizationFloat64 {
    type MembranePotentialQuant = f64;
}

//endregion
