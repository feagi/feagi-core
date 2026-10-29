use crate::cortical_area::cortical_dynamics_quantization::CorticalDynamicsQuantization;
use crate::cortical_area::data::neuron_layout::NeuronLayout;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
// NOTE: How this works is a sub trait is made FIRST per cortical area model, then
// there are implementations under that

/// Describes a specific cortical area model for some usecase, as well as its dynamics
pub trait CorticalDynamicsModel<FIQ, NL, CDQ>
where
    FIQ: FeagiIndexQuantization,
    NL: NeuronLayout<FIQ>, // Likely restricted per model
    CDQ: CorticalDynamicsQuantization, // More restricted per model
{
    // Is extended per cortical model, with restraints added to the generic bounds as needed

    // TODO Neuron layout restrictor -> employ restrictions on the neuron layout directly

}