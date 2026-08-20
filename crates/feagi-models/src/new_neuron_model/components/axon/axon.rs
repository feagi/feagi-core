use crate::new_neuron_model::neuron_model_quantization::NeuronModelQuantizationLevel;
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use crate::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;

/// The direct output of a cortical area, this is what mappings will interact with for their incoming data
pub trait Axon<NPUIQ, BEIQ, NMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    NMQ: NeuronModelQuantizationLevel,
{
    
    
}





