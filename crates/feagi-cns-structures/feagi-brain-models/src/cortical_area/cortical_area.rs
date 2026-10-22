use crate::cortical_area::components_rigid::neuron_layout::NeuronLayout;
use crate::cortical_area::components_extendable::cortical_area_dynamics::CorticalAreaDynamics;
use crate::cortical_area::components_extendable::cortical_area_quantization::CorticalAreaQuantization;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Describes a cortical area model with all its dynamics
pub trait CorticalAreaModel<FIQ, CAQ>:
where
    FIQ: FeagiIndexQuantization,
    CAQ: CorticalAreaQuantization,
{
    /// How are the neurons laid out in the cortical area?
    type NeuronLayout: NeuronLayout<FIQ>;

    /// The implementation of the dynamics, and the data it requires
    type CorticalAreaDynamics: CorticalAreaDynamics<FIQ, Self::NeuronLayout, CAQ>;

    type CorticalAreaModelWriter; // TODO
    
    fn get_neuron_layout(&self) -> &Self::NeuronLayout;
    
    fn replace_neuron_layout(&mut self, new_neuron_layout: Self::NeuronLayout);
    
    
    
}