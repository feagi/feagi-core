
//region Flag Traits

use feagi_structures::CorticalAreaNeuronQuantization;
use feagi_structures::neuron::{FeagiNeuronError, LinearNeuronIndexCount};
use crate::dynamics::neuron::linear::collections::NeuronModelCollectionBaseLinearTrait;
use crate::dynamics::neuron::linear::neuron_firing::NeuronFiringResultTrait;
use crate::dynamics::neuron::linear::neurons::{NeuronModelParametersTrait};

// TODO macros to make flags

/// Trait applied to all Neuron Flags
pub trait NeuronFlag: Sized{
    /// Use "new_from_raw" to create a flag with all fields false
    const FLAG_ALL_FALSE: u8 = 0;
    /// Use "new_from_raw" to create a flag with all fields true
    const FLAG_ALL_TRUE: u8 = 255;

    /// Use this to create directly from a u8 (8 bits)
    fn new_from_raw(bits: u8) -> Self;
}


/// Trait applied to neuron flags that
pub trait MortalNeuronFlag: NeuronFlag
{
    /// Use "new_from_raw" to create an alive neuron but all other fields false.
    const FLAG_ALIVE_REST_FALSE: u8 = 0x01;

    /// Returns true if a neuron is alive
    fn is_neuron_alive(&self) -> bool;
    /// Allows setting alive state of a neuron
    fn set_neuron_life(&mut self, set_alive: bool);
    /// Toggles if a neuron is alive, slightly faster when setting it when used with flags
    fn toggle_neuron_alive(&mut self);
}

//endregion

//region Neuron

pub trait MortalNeuron<CANQ: CorticalAreaNeuronQuantization>:
NeuronModelParametersTrait<CANQ>
{
    /// Returns true if a neuron is alive
    fn is_neuron_alive(&self) -> bool;
    /// Allows setting alive state of a neuron
    fn set_neuron_life(&mut self, set_alive: bool);
    /// Toggles if a neuron is alive, slightly faster when setting it when used with flags
    fn toggle_neuron_alive(&mut self);

}

pub trait CountableDeadNeuronCollection<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>:
NeuronModelCollectionBaseLinearTrait<CANQ, NMP>
{
    /// Returns true if a neuron is alive
    fn get_if_neuron_alive(&self, neuron: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<bool, FeagiNeuronError>;

    fn get_number_dead_neurons(&self) -> LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>;

    fn get_number_live_neurons(&self) -> LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        self.get_number_contained_neurons() - self.number_dead_neurons()
    }
}

pub trait CountableDeadNeuronCollectionMut<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>:
CountableDeadNeuronCollection<CANQ, NMP>
{
    /// Allows setting alive state of a neuron
    fn set_neuron_life(&mut self, neuron: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>, set_alive: bool) -> Result<(), FeagiNeuronError>;

    /// Toggles if a neuron is alive, slightly faster than setting it
    fn toggle_neuron_life(&mut self, neuron: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<(), FeagiNeuronError>;
}

// TODO Vector / Array / Hashset reference to dead neurons!

//endregion

//region Neuron Firing

pub trait NeuronFiringResultMortalityTrait<CANQ: CorticalAreaNeuronQuantization>:
NeuronFiringResultTrait<CANQ>
{
    /// If this neuron is set to die
    fn is_going_to_die(&self) -> bool;
}

//endregion
