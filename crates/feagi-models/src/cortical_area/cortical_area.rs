use crate::cortical_area::components::neuron_layout::neuron_layout_model::{NeuronLayout};
use crate::cortical_area::parameters::body::dynamics::components::quantization::quantization::CorticalAreaQuantization;
use crate::cortical_area::parameters::body::dynamics::cortical_area_dynamics::CorticalAreaDynamics;
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use crate::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;

// TODO Neuron Membrane Type should be configurable

/// Describes a cortical area model with all its dynamics
pub trait CorticalAreaModel<NPUIQ, BEIQ, CAMQ, PPPP>:

where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
    /// How are the neurons laid out in the cortical area?
    type NeuronLayout: NeuronLayout<BEIQ>;

    /// The implementation of the dynamics, and the data it requires
    type CorticalAreaDynamics: CorticalAreaDynamics<NPUIQ, BEIQ, Self::NeuronLayout, CAMQ>;
}