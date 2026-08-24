use crate::cortical_area::cortical_area_model_quantization::CorticalAreaModelQuantization;
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use crate::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;

/// Any per neuron level data that is internal, not to be exposed to genome developers
pub trait NeuronDataWork<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaModelQuantization,
{
    /// Denotes that there is data that needs to be allocated. 
    /// Only the null implementation has this false
    const HAS_DATA_TO_ALLOCATE: bool = true;

    // extend per neuron firing implementation
}