use feagi_structures::feagi_data::shared_quantization_sets::FeagiGlobalQuantization;
use crate::neural_processing_unit_data_structures::cpu_wrappers::NPUWrappedNeuronIndexBurstEngineIndex;

/// Converts from BurstEngine neuron index to a known quant type to the mp quant index value by
/// simply subtracting the given offset
pub trait BurstEngineNeuronIndexToMPQuantIndexLookupTable<FGQ: FeagiGlobalQuantization> {}


//region CPU implementation

pub struct BurstEngineNeuronIndexToMPQuantIndexLookupTableCPU<FGQ: FeagiGlobalQuantization>
{
    // NOTE: f32 will ALWAYS be zero!
    float_32_down_offset: NPUWrappedNeuronIndexBurstEngineIndex<FGQ::NeuronIndexCountQuant>,
    float_8_down_offset: NPUWrappedNeuronIndexBurstEngineIndex<FGQ::NeuronIndexCountQuant>,
    // TODO other quants!
}




//endregion