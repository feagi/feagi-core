//! Traits for all base (linear) neurons, neuron refs, and neuron mutable refs

use feagi_structures::CorticalAreaNeuronQuantization;
use feagi_structures::neuron::NeuronMembranePotential;

/// Defines all the fields for a single neuron as independent values. Required for all model
/// implementations. Used to generate Individual Neuron Struct
pub trait NeuronModelNeuronTrait<CANQ: CorticalAreaNeuronQuantization>
{
    /// Membrane potential is required for all neuron models
    fn get_membrane_potential(&self) -> &NeuronMembranePotential<CANQ::NeuronValueQuant>;
    fn get_membrane_potential_mut(&mut self) -> &mut NeuronMembranePotential<CANQ::NeuronValueQuant>;
}

/// Defines all the fields for a single neuron as an immutable reference. Required for all model
/// implementations. Used to generate Individual Neuron Ref Struct
pub trait NeuronModelNeuronRefTrait<'a, CANQ: CorticalAreaNeuronQuantization> {

    /// Membrane potential is required for all neuron models
    fn get_membrane_potential(&self) -> &'a NeuronMembranePotential<CANQ::NeuronValueQuant>;

    // Define other fields here. Make sure all implementations use inline
}

/// Defines all the fields for a single neuron as a mutable reference. Required for all model
/// implementations. Used to generate Individual Neuron Mut Ref Struct
pub trait NeuronModelNeuronMutRefTrait<'a, CANQ: CorticalAreaNeuronQuantization>:
NeuronModelNeuronRefTrait<'a, CANQ>
{
    /// Membrane potential is required for all neuron models
    fn get_membrane_potential_mut(&mut self) -> &mut NeuronMembranePotential<CANQ::NeuronValueQuant>;

    // Define other fields here. Make sure all implementations use inline
}

/// Won't be a struct itself, but rather extends the ref traits to allow cloning the ref into a
/// "NeuronModelNeuron"
pub trait NeuronModelNeuronRefClonableTrait<CANQ: CorticalAreaNeuronQuantization>:
{
    type NeuronStruct: NeuronModelNeuronTrait<CANQ>;
    fn clone_as_neuron(&self) -> Self::NeuronStruct;
}