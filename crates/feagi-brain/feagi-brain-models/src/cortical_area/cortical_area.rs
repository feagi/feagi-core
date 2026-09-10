use feagi_data::feagi_data_neuron::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::models::cortical_area::components::neuron_layout::neuron_layout_model::NeuronLayout;
use crate::models::cortical_area::parameters::body::dynamics::components::quantization::quantization::CorticalAreaQuantization;
use crate::models::cortical_area::parameters::body::dynamics::cortical_area_dynamics::CorticalAreaDynamics;

/// Describes a cortical area model with all its dynamics
pub trait CorticalAreaModel<FIQ, CAMQ>:

where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
    /// How are the neurons laid out in the cortical area?
    type NeuronLayout: NeuronLayout<FIQ>;

    /// The implementation of the dynamics, and the data it requires
    type CorticalAreaDynamics: CorticalAreaDynamics<FIQ, Self::NeuronLayout, CAMQ>;
    
    type CorticalAreaWriter; // TODO
    
    
}