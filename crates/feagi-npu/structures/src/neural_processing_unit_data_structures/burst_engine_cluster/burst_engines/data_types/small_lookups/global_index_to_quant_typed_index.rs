use feagi_structures::feagi_data::quantizable_linear::wrappers::QuantizedElementWrapperBase;
use feagi_structures::feagi_data::shared_quantization_sets::{CorticalPotentialQuantizationLevel, FeagiGlobalQuantization};
use crate::neural_processing_unit_data_structures::cpu_wrappers::{NPUWrappedNeuronIndexBurstEngineIndex, NPUWrappedNeuronMPQuantIndex};

/// Uses binary search in an internal table to convert the global index to a quant local one,
/// assuming the indexes are ordered. It also returns the type of the quant as a flag.
/// This trait is generic, there are several implementations that can be made
pub trait NPUGlobalIndexToQuantTypedIndex<FGQ: FeagiGlobalQuantization>:
{}


pub struct NPUGlobalNeuronIndexToQuantTypedNeuronIndexTableCPU<FGQ: FeagiGlobalQuantization>
{
    pub bound_1: NPUWrappedNeuronIndexBurstEngineIndex<FGQ::NeuronIndexCountQuant>,
    pub bound_2: NPUWrappedNeuronIndexBurstEngineIndex<FGQ::NeuronIndexCountQuant>,
    pub bound_3: NPUWrappedNeuronIndexBurstEngineIndex<FGQ::NeuronIndexCountQuant>,
}

impl<FGQ: FeagiGlobalQuantization> NPUGlobalNeuronIndexToQuantTypedNeuronIndexTableCPU<FGQ> {
    pub fn new( bound_1: NPUWrappedNeuronIndexBurstEngineIndex<FGQ::NeuronIndexCountQuant>,
                bound_2: NPUWrappedNeuronIndexBurstEngineIndex<FGQ::NeuronIndexCountQuant>,
                bound_3: NPUWrappedNeuronIndexBurstEngineIndex<FGQ::NeuronIndexCountQuant>)
        -> Self
    {
        Self { bound_1, bound_2, bound_3, }
    }

    pub fn down_convert_global_neuron_index_to_quant(&self, global: NPUWrappedNeuronIndexBurstEngineIndex<FGQ::NeuronIndexCountQuant>)
        -> (NPUWrappedNeuronMPQuantIndex<FGQ::NeuronIndexCountQuant>,  CorticalPotentialQuantizationLevel)
    {
        if global < self.bound_2
        {
            if global < self.bound_1 { (NPUWrappedNeuronMPQuantIndex::wrap(global.unwrap()), CorticalPotentialQuantizationLevel::Float32) } // TODO correct to f8!
            else { (NPUWrappedNeuronMPQuantIndex::wrap((global - self.bound_1).unwrap()), CorticalPotentialQuantizationLevel::Float32)  } // TODO correct to f16!
        }
        else
        {
            if global < self.bound_3 { (NPUWrappedNeuronMPQuantIndex::wrap((global - self.bound_2).unwrap()), CorticalPotentialQuantizationLevel::Float32) } 
            else { (NPUWrappedNeuronMPQuantIndex::wrap((global - self.bound_3).unwrap()), CorticalPotentialQuantizationLevel::Float32)  } // TODO correct to f64!
        }
    }
}

impl<FGQ: FeagiGlobalQuantization> NPUGlobalIndexToQuantTypedIndex<FGQ> for NPUGlobalNeuronIndexToQuantTypedNeuronIndexTableCPU<FGQ> {}




pub struct NPUGlobalSynapseIndexToQuantTypedSynapseIndexTableCPU<FGQ: FeagiGlobalQuantization>
{
    pub bound_1: NPUWrappedNeuronIndexBurstEngineIndex<FGQ::NeuronIndexCountQuant>,
    pub bound_2: NPUWrappedNeuronIndexBurstEngineIndex<FGQ::NeuronIndexCountQuant>,
    pub bound_3: NPUWrappedNeuronIndexBurstEngineIndex<FGQ::NeuronIndexCountQuant>,
}

impl<FGQ: FeagiGlobalQuantization> NPUGlobalIndexToQuantTypedIndex<FGQ> for NPUGlobalSynapseIndexToQuantTypedSynapseIndexTableCPU<FGQ> {}