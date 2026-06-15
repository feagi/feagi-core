use crate::neural_processing_unit_data_structures::burst_engine::npu_data::neuron_indexes_condensed_burst_engine_table::NPUNeuronIndexesCondensedBurstEngineTableCPU;
use crate::neural_processing_unit_data_structures::burst_engine::npu_data::neuron_membrane_potentials_mp_grouped_table::NPUNeuronMembranePotentialsMPGroupedTableCPU;
use crate::neural_processing_unit_data_structures::neuron_models::neuron_models::feagi_standard::quantization::FeagiStandardModelQuantization;




pub trait NPUExecutorBurstCondenseFCLToFCLCompact<FGQ: FeagiStandardModelQuantization>: {}

//region CPU Implementation

pub struct NPUExecutorBurstCondenseFCLToFCLCompactCPU<FGQ: FeagiStandardModelQuantization>
{
    _p: core::marker::PhantomData<FGQ>,
}

impl<FGQ: FeagiStandardModelQuantization> NPUExecutorBurstCondenseFCLToFCLCompactCPU<FGQ>
{
    pub fn condense_fcl(
        fcl_by_quants_table: &mut NPUNeuronMembranePotentialsMPGroupedTableCPU<FGQ>,
        fcl_compact_global_table: &mut NPUNeuronIndexesCondensedBurstEngineTableCPU<FGQ>, 
        neuron_activity_bits_global_table: &mut Z) 
    {
        todo!()
    }
}
//endregion