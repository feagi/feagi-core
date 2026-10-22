use crate::cortical_area::components::cortical_area_quantization::CorticalAreaQuantization;
use crate::cortical_area::components::neuron_layout::NeuronLayout;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Describes a cortical area model with all its dynamics
pub trait CorticalAreaModel<FIQ, NL, CAQ>:
where
    FIQ: FeagiIndexQuantization,
    NL: NeuronLayout<FIQ>, // This may be fixed for certain models
    CAQ: CorticalAreaQuantization, // this will be further refined per model
{
    // /// The implementation of the dynamics, and the data it requires
    //type CorticalAreaDynamics: CorticalAreaDynamics<FIQ, NL, CAQ>;

    //type CorticalAreaModelWriter; // TODO

    
}



