
//region Flag Traits

use feagi_structures::CorticalAreaNeuronQuantization;
use feagi_structures::neuron::{FeagiNeuronError, LinearNeuronIndexCount};
use crate::dynamics::neuron::linear::collections::NeuronModelCollectionBaseLinearTrait;
use crate::dynamics::neuron::linear::neurons::NeuronModelNeuronTrait;

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
    fn set_neuron_alive(&mut self, set_alive: bool);
    /// Toggles if a neuron is alive, slightly faster when setting it when used with flags
    fn toggle_neuron_alive(&mut self);
}

//endregion

//region Neuron

pub trait MortalNeuron<CANQ: CorticalAreaNeuronQuantization>:
NeuronModelNeuronTrait<CANQ>
{
    /// Returns true if a neuron is alive
    fn is_neuron_alive(&self) -> bool;
}

pub trait MortalNeuronMut<CANQ: CorticalAreaNeuronQuantization>:
MortalNeuron<CANQ>
{
    /// Allows setting alive state of a neuron
    fn set_neuron_alive(&mut self, set_alive: bool);
    /// Toggles if a neuron is alive, slightly faster when setting it when used with flags
    fn toggle_neuron_alive(&mut self);
}

pub trait CountableDeadNeuronCollection<CANQ: CorticalAreaNeuronQuantization>:
NeuronModelCollectionBaseLinearTrait<CANQ>
{
    fn get_number_dead_neurons(&self) -> LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>;

    fn get_number_live_neurons(&self) -> LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        self.get_number_contained_neurons() - self.number_dead_neurons()
    }
}

pub trait CountableDeadNeuronCollectionMut<CANQ: CorticalAreaNeuronQuantization>:
CountableDeadNeuronCollection<CANQ>
{
    fn mark_neuron_as_dead(&mut self, dead_neuron: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<(), FeagiNeuronError>;
}

// TODO Vector / Array / Hashset reference to dead neurons!

//endregion
