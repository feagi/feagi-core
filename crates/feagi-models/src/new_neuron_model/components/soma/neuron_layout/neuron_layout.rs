
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use crate::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;

/// Defines how neurons are laid out within a cortical area (the pattern / structure)
pub trait NeuronLayout<NPUIQ, BEIQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
{
    
}