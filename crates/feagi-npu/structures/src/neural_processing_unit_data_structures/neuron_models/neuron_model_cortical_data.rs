use feagi_structures::feagi_data::shared_quantization_sets::{FeagiGlobalQuantization, NeuronModelQuantization};
use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::data_types::cortical_area_layout::CorticalConfigurationBase;


/// Root trait for all cortical data implementations, essentially any cortical level data shared
/// by all neurons in a cortical area of a given neuron model. This should be extended with only
/// the cortical level data
pub trait NeuronModelCorticalData<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
{
    /// Set to true if the neuron model needs to be informed if the global burst index counter is
    /// about to overflow. Otherwise, set to false
    const MODEL_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER: bool;

    /// Set to true if the neuron model can run under Dimensional type cortical areas
    const MODEL_SUPPORTS_CORTICAL_CONFIGURATION_DIMENSIONAL: bool;
    // TODO other cortical configuration types
    
    
    // Implement any cortical level data

    // No methods!
}



//region CPU Specific Trait


/// Root CPU trait for all cortical data implementations, essentially any cortical level data shared
/// by all neurons in a cortical area of a given neuron model
pub trait NeuronModelCorticalDataCPU<FGQ, NMQ>:
NeuronModelCorticalData<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
{
    // Implement any cortical level data members (or make members pub)
}



//endregion