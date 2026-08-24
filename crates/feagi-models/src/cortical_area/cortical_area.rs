use crate::cortical_area::components::axon_model::AxonModelTrait;
use crate::cortical_area::components::dendrite::{DendriteConfigTrait, DendriteModelTrait};
use crate::cortical_area::components::neuron_layout::neuron_layout_config::NeuronLayoutConfigTrait;
use crate::cortical_area::components::neuron_layout::neuron_layout_model::NeuronLayoutModelTrait;
use crate::cortical_area::cortical_area_model_quantization::CorticalAreaModelQuantization;
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use crate::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;

// TODO storage backend should probably not be here

// TODO Neuron Membrane Type should be configurable

/// Describes a cortical area model with all its dynamics
pub trait CorticalAreaModel<NPUIQ, BEIQ, CAMQ>: 

where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaModelQuantization
{
    type NeuronLayout: NeuronLayoutModelTrait;
}


/// Describes a cortical area model with all its configuration
pub trait CorticalAreaConfig<NPUIQ, BEIQ, CAMQ>:
DendriteConfigTrait<NPUIQ, BEIQ, CAMQ>

where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaModelQuantization

{
    type NeuronLayout: NeuronLayoutConfigTrait<BEIQ>;
}