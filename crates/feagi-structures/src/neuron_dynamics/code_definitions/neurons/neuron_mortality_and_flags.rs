use crate::{define_ref_immut_access_trait_methods, define_ref_immut_mut_access_trait_methods};
use crate::neuron_dynamics::code_definitions::neurons::base_neuron_model_fields::{NeuronModelNeuronTrait, NeuronModelNeuronMutRefTrait, NeuronModelNeuronMutSliceRef, NeuronModelNeuronRefTrait, NeuronModelNeuronSliceRef};
use crate::quantization_level::CorticalAreaNeuronQuantization;
// TODO any way to speeden this with const functions? low priority

// TODO macro Define a generic neuron flag with 8 properties

// TODO macro Define a mortal neuron flag with 7 spare properties

/// Trait applied to all Neuron Flags
pub trait NeuronFlag: Sized{
    /// Use "new_from_raw" to create a flag with all fields false
    const FLAG_ALL_FALSE: u8 = 0;
    /// Use "new_from_raw" to create a flag with all fields true
    const FLAG_ALL_TRUE: u8 = 255;

    /// Use this to create directly from a u8 (8 bits)
    fn new_from_raw(bits: u8) -> Self;
}

/// Trait applied to neuron flags
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

// TODO macro that takes in a MortalNeuronFlag struct and a target neuron model name and
// automatically implements all these traits

/// These traits
//region Neuron Trait Extensions


//region Individual Neurons

pub trait MortalNeuronModelNeuron<CANQ: CorticalAreaNeuronQuantization, MNF: MortalNeuronFlag>: NeuronModelNeuronTrait<CANQ> {
    define_ref_immut_mut_access_trait_methods!(flags_with_mortality, MNF);

    /// Returns true if a neuron is alive
    fn is_neuron_alive(&self) -> bool {
        self.get_flags_with_mortality().is_neuron_alive()
    }
    /// Allows setting alive state of a neuron
    fn set_neuron_alive(&mut self, set_alive: bool) {
        self.get_flags_with_mortality_mut().set_neuron_alive(set_alive);
    }
    /// Toggles if a neuron is alive, slightly faster when setting it when used with flags
    fn toggle_neuron_alive(&mut self) {
        self.get_flags_with_mortality_mut().toggle_neuron_alive();
    }
}

pub trait MortalNeuronModelNeuronRef<'a, CANQ: CorticalAreaNeuronQuantization, MNF: MortalNeuronFlag>: NeuronModelNeuronRefTrait<'a, CANQ> {
    define_ref_immut_access_trait_methods!(flags_with_mortality, &'a MNF);

    /// Returns true if a neuron is alive
    fn is_neuron_alive(&self) -> bool {
        self.get_flags_with_mortality().is_neuron_alive()
    }
}

pub trait MortalNeuronModelNeuronMutRef<'a, CANQ: CorticalAreaNeuronQuantization, MNF: MortalNeuronFlag>: NeuronModelNeuronMutRefTrait<'a, CANQ> {
    define_ref_immut_mut_access_trait_methods!(flags_with_mortality, &'a mut MNF);

    /// Returns true if a neuron is alive
    fn is_neuron_alive(&self) -> bool {
        self.get_flags_with_mortality().is_neuron_alive()
    }
    /// Allows setting alive state of a neuron
    fn set_neuron_alive(&mut self, set_alive: bool) {
        self.get_flags_with_mortality_mut().set_neuron_alive(set_alive);
    }
    /// Toggles if a neuron is alive, slightly faster when setting it when used with flags
    fn toggle_neuron_alive(&mut self) {
        self.get_flags_with_mortality_mut().toggle_neuron_alive();
    }
}


//endregion

//region Neuron Slices

/// Defines all the fields for a slice of all neurons as an immutable reference. Required for all model
/// implementations. Used to generate Individual Neuron Ref Slice Struct
pub trait MortalNeuronModelNeuronSliceRef<'a, CANQ: CorticalAreaNeuronQuantization, MNF: MortalNeuronFlag>: NeuronModelNeuronSliceRef<'a, CANQ> {
    define_ref_immut_access_trait_methods!(flags_with_mortality, &'a [MNF]);

    /// Returns true if a neuron is alive
    fn is_neuron_alive(&self) -> bool {
        self.get_flags_with_mortality().is_neuron_alive()
    }
}

/// Defines all the fields for a slice of all neurons as a mutable reference. Required for all model
/// implementations. Used to generate Individual Neuron Mut Ref Slice Struct
pub trait MortalNeuronModelNeuronMutSliceRef<'a, CANQ: CorticalAreaNeuronQuantization, MNF: MortalNeuronFlag>: NeuronModelNeuronMutSliceRef<'a, CANQ> {
    define_ref_immut_mut_access_trait_methods!(flags_with_mortality, &'a mut [MNF]);

    /// Returns true if a neuron is alive
    fn is_neuron_alive(&self) -> bool {
        self.get_flags_with_mortality().is_neuron_alive()
    }
    /// Allows setting alive state of a neuron
    fn set_neuron_alive(&mut self, set_alive: bool) {
        self.get_flags_with_mortality_mut().set_neuron_alive(set_alive);
    }
    /// Toggles if a neuron is alive, slightly faster when setting it when used with flags
    fn toggle_neuron_alive(&mut self) {
        self.get_flags_with_mortality_mut().toggle_neuron_alive();
    }
}

//endregion


// TODO Neuron Collections!





//endregion