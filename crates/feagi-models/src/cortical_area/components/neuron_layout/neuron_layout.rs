use feagi_data::neurons::neuron::indexing::NeuronCount;
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use crate::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;

/// Defines how neurons are laid out within a cortical area (the pattern / structure)
pub trait NeuronLayoutModelTrait<BEIQ>
where
    BEIQ: BurstEngineIndexQuantization,
{
    
}

/// Defines how neurons are laid out within a cortical area (the pattern / structure)
pub trait NeuronLayoutConfigTrait<BEIQ>
where
    BEIQ: BurstEngineIndexQuantization,
{
    /// (Up to) How many neurons are in this area?
    fn get_number_of_area_neurons(&self) -> NeuronCount<BEIQ::NeuronIndexQuant>;
}

