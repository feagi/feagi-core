// TODO a macro would be nice to standardize mp grouping

use feagi_structures::feagi_data::quantization_levels::feagi_global_quantization::FeagiGlobalQuantization;
use crate::neural_processing_unit_data_structures::wrappers::{NPUWrappedNeuronIndexBurstEngineIndex, NPUWrappedNeuronMembranePotential};


/// Converts from BurstEngine neuron index to a known quant type to the mp quant index value by
/// simply subtracting the given offset. O(0) time but you need to have the quant flat already
pub struct EngineNeuronIndexOffsetsToMPQuantNeuronIndex<FIQ: FeagiGlobalQuantization>
{
    // NOTE: f32 will ALWAYS be zero!
    pub float_32: NPUWrappedNeuronIndexBurstEngineIndex<FIQ::NeuronIndexCountQuant>,
    //float_8_down_offset: NPUWrappedNeuronIndexBurstEngineIndex<FIQ::NeuronIndexCountQuant>,
    // TODO other quants!
}


pub struct MPQuantNeuronFCLValues
{
    // TODO we should be using CBQ!
    pub float_32: Vec<NPUWrappedNeuronMembranePotential<f32>> 
}


pub struct MPQuantNeuronMembranePotentialValues
{
    // TODO we should be using CBQ!
    pub float_32: Vec<NPUWrappedNeuronMembranePotential<f32>>
}


pub struct MPQuantBasePostSynapticPotentials
{
    // TODO we should be using CBQ!
    pub float_32: Vec<NPUWrappedNeuronMembranePotential<f32>>
}
