use feagi_basis::feagi_quantization::prelude::*;
use crate::cortical_area::components::cortical_area_dynamics::components::cortical_area_quantization::CorticalAreaQuantization;
use crate::cortical_area::components::cortical_area_dynamics::CorticalAreaDynamics;
use crate::cortical_area::components::neuron_layout::NeuronLayout;

/// Describes a cortical area model with all its dynamics
pub trait CorticalAreaModel<FIQ, CAQ>:
where
    FIQ: FeagiIndexQuantization,
    CAQ: CorticalAreaQuantization,
{
    /// How are the neurons laid out in the cortical area?
    type NeuronLayout: NeuronLayout<FIQ>;

    /// The implementation of the dynamics, and the data it requires
    type CorticalAreaDynamics: CorticalAreaDynamics<FIQ, Self::NeuronLayout>; // TODO fix CAQ to CorticalAreaDynamics::CorticalAreaQuantization
    
    type CorticalAreaWriter; // TODO
    
    
}