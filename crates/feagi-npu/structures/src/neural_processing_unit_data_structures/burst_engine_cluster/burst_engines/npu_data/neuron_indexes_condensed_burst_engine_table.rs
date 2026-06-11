use core::marker::PhantomData;
use feagi_structures::feagi_data::shared_quantization_sets::{CorticalPotentialQuantization, CorticalPotentialQuantizationFloat32, FeagiGlobalQuantization};
use crate::neural_processing_unit_data_structures::calculate_struct_padding::calculate_byte_alignment_padding;
use crate::neural_processing_unit_data_structures::cpu_wrappers::{NPUWrappedNeuronIndexBurstEngineIndex, NPUWrappedNeuronMembranePotential};

/// is a dense list of global neuron indexes, typically used to filter for active neurons, that
/// is scoped to the current burst engine
pub trait NPUNeuronIndexesCondensedBurstEngineTable<FGQ: FeagiGlobalQuantization> {}


// TODO other quants!

/// Neuron Potentials Grouped by MP Quantization. This struct is used for holding the neuron
/// membrane potentials and for the FCL. Make sure to use NPUWrappedNeuronMPQuantIndex indexing
#[repr(C)]
pub struct NPUNeuronIndexesCondensedBurstEngineTableCPU<FGQ: FeagiGlobalQuantization>
{
    pub neuron_indexes: Vec<NPUWrappedNeuronIndexBurstEngineIndex<FGQ::NeuronIndexCountQuant>>,
    _padding: [u8; calculate_byte_alignment_padding(size_of::<Vec<u8>>())], // data type irrelevant.
    _p: PhantomData<FGQ>,

}

impl<FGQ: FeagiGlobalQuantization> NPUNeuronIndexesCondensedBurstEngineTable<FGQ> for NPUNeuronIndexesCondensedBurstEngineTableCPU<FGQ> {}








