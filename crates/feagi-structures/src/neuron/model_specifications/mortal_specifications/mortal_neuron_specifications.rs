use crate::define_ref_immut_mut_access_trait_methods;
use crate::neuron::model_specifications::base_specifications::{BaseNeuronCollectionSharedTrait, BaseNeuronModelDataRefTrait};
use crate::quantization_level::CorticalAreaNeuronQuantization;



pub trait MortalNeuronModelCollectionSharedTrait<CANQ: CorticalAreaNeuronQuantization>:
BaseNeuronCollectionSharedTrait<CANQ> {
    type SingleNeuronReference: MortalNeuronModelDataRefTrait<'static, CANQ>;

    // TODO ???
}

/// Represents a single mortal neuron, as a reference data slice
pub trait MortalNeuronModelDataRefTrait<'a, CANQ: CorticalAreaNeuronQuantization>:
BaseNeuronModelDataRefTrait<'a, CANQ> +
NeuronAliveTrait
{
    type NeuronModelCollectionType: BaseNeuronCollectionSharedTrait<CANQ>;
}


// NOTE: No difference for Dense vs Sparse

/// Unifies definition for if a neuron is alive or not. Can be used on neurons and neuron flag structs
pub trait NeuronAliveTrait {
    /// Returns true if a neuron is alive
    fn is_neuron_alive(&self) -> bool;
    /// Allows setting alive state of a neuron
    fn set_neuron_alive(&mut self, set_alive: bool);
    /// Toggles if a neuron is alive, slightly faster when setting it when used with flags
    fn toggle_neuron_alive(&mut self);
}








// TODO cortical area?

/// Unifies definition for if a cortical area is alive or not
pub trait CorticalFlagAliveTrait {
    /// Returns true if a cortical area is alive
    fn is_cortical_area_alive(&self) -> bool;
    /// Allows setting alive state of a cortical area
    fn set_cortical_area_alive(&mut self, set_alive: bool);
    /// Toggles if a cortical area is alive, slightly faster when setting it when used with flags
    fn toggle_cortical_area_alive(&mut self);
}