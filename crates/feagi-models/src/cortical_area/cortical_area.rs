use crate::cortical_area::components::axon_model::AxonModelTrait;
use crate::cortical_area::components::dendrite::{DendriteConfigTrait, DendriteModelTrait};
use crate::cortical_area::components::neuron_layout::neuron_layout_model::{NeuronLayout};
use crate::cortical_area::parameters::dynamics::components::quantization::quantization::CorticalAreaQuantization;
use crate::cortical_area::parameters::dynamics::cortical_area_dynamics::CorticalAreaDynamics;
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use crate::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;

// TODO Neuron Membrane Type should be configurable

/// Describes a cortical area model with all its dynamics
pub trait CorticalAreaModel<NPUIQ, BEIQ, CAD, CAMQ>:

where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
    type NeuronLayout: NeuronLayout<BEIQ>;

    type CorticalAreaDynamics: CorticalAreaDynamics<NPUIQ, BEIQ, Self::NeuronLayout, CAMQ>;
}