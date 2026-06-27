use crate::quantization_levels::cortical_potential_quantization::{CorticalPotentialQuantization, CorticalPotentialQuantizationLevel};


/// Defines the quantization used in a cortical area for the calculation of neuron dynamics.
/// All are required to support neuron potentials, hence this is the shared base of each model's
/// implementation. Each cortical area within an NPU may have different quantization levels.
/// DO NOT IMPLEMENT THIS IN ACTUAL DATA STRUCTURES! THIS IS ONLY INTENDED TO CARRY QUANTIZATION
/// CONTEXTS
pub trait NeuronModelQuantization
{
    const CORTICAL_POTENTIAL_QUANTIZATION_LEVEL: CorticalPotentialQuantizationLevel = Self::CorticalPotentialQuant::QUANTIZATION_LEVEL;

    /// Defines the quantization of the membrane potential of a neuron, which all models must
    /// include. This may vary between cortical areas, even of the same model
    type CorticalPotentialQuant: CorticalPotentialQuantization;
}


/// Defines the quantization used in an axon bundle for synapse dynamics. As synapses do not have
/// any required fields, there are not required fields.
/// DO NOT IMPLEMENT THIS IN ACTUAL DATA STRUCTURES! THIS IS ONLY INTENDED TO CARRY QUANTIZATION
/// CONTEXTS
pub trait SynapseModelQuantization { }