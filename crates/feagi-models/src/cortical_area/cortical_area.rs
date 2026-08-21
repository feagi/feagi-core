use crate::cortical_area::components::axon_model::AxonModelTrait;
use crate::cortical_area::components::dendrite::{DendriteConfigTrait, DendriteModelTrait};
use crate::cortical_area::cortical_area_model_quantization::CorticalAreaModelQuantization;
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use crate::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;



/// Describes a cortical model 
pub trait CorticalAreaModel<NPUIQ, BEIQ, CAMQ>: 
DendriteModelTrait<NPUIQ, BEIQ, CAMQ>

+ AxonModelTrait<NPUIQ, BEIQ, CAMQ>


where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaModelQuantization
{
    type Soma;
    
    type Axon;
    
    type NeuronFiringModel;
}



pub trait CorticalAreaConfig<NPUIQ, BEIQ, CAMQ>:
DendriteConfigTrait<NPUIQ, BEIQ, CAMQ>

where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaModelQuantization

{
    
}