use crate::values::quantizable::QuantizedDecimalTrait;

/// Encoding of the Cortical Potential Quantization as a byte for use on all backends
#[repr(u8)]
#[derive(Default, Clone, Copy, Debug)]
pub enum CorticalMembranePotentialQuantizationLevel {
    // Unless otherwise noted, a quantization level is for CPU
    #[default]
    Float32 = 0,
    Float64 = 1,
    // TODO f16, f8, uint8, sint8, f64
    
}


impl CorticalMembranePotentialQuantizationLevel {
    /// Gets the enum from a byte without any checks. Invalid bytes will cause undefined behavior!
    pub unsafe fn unsafe_from_byte(byte: u8) -> Self {
        unsafe { core::mem::transmute(byte) }
    }
}

/// Defines the quantization of the neuron potential for a specific cortical area for the CPU. All cortical
/// areas must have the neuron potential. This quantization is set per cortical area, and
/// is controlled by the Neuron Model quantization state, although this cortical level neuron
/// potential quantization has discrete steps that must be followed. Since other devices have their
/// own quantization requirements, they will use a different trait and system
pub trait CorticalPotentialCPUQuantization: Clone + Copy {
    const QUANTIZATION_LEVEL: CorticalMembranePotentialQuantizationLevel;

    /// Defines the quantization of the membrane potential of a neuron, which all models must
    /// include. This may vary between cortical areas, even of the same model. This also impacts
    /// the FCL as well
    type MembranePotentialQuant: QuantizedDecimalTrait;
}

//region Discrete Levels

#[derive(Debug, Clone, Copy)]
pub struct CorticalPotentialQuantizationCPUFloat32;

impl CorticalPotentialCPUQuantization for CorticalPotentialQuantizationCPUFloat32 {
    const QUANTIZATION_LEVEL: CorticalMembranePotentialQuantizationLevel =
        CorticalMembranePotentialQuantizationLevel::Float32;
    type MembranePotentialQuant = f32;
}

#[derive(Debug, Clone, Copy)]
pub struct CorticalPotentialQuantizationCPUFloat64;

impl CorticalPotentialCPUQuantization for CorticalPotentialQuantizationCPUFloat64 {
    const QUANTIZATION_LEVEL: CorticalMembranePotentialQuantizationLevel =
        CorticalMembranePotentialQuantizationLevel::Float64;
    type MembranePotentialQuant = f64;
}

//endregion
