use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::npu_data::cortical_area_data_mapping_table::CorticalAreaDataMappingTableCPU;
use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::npu_data::cortical_descriptor_lookup_table::CorticalDescriptorLookupTableCPU;
use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::npu_data::neuron_membrane_potentials_mp_grouped_table::NPUNeuronMembranePotentialsMPGroupedTableCPU;
use crate::neural_processing_unit_data_structures::neuron_models::neuron_models::feagi_standard::quantization::FeagiStandardModelQuantization;

pub trait NPUExecutorBurstRunNeuronDynamics<FGQ: FeagiStandardModelQuantization>: {}



pub struct NPUExecutorBurstRunNeuronDynamicsCPU<FGQ: FeagiStandardModelQuantization>
{
    _p: core::marker::PhantomData<FGQ>,
}

impl<FGQ: FeagiStandardModelQuantization> NPUExecutorBurstRunNeuronDynamicsCPU<FGQ>
{
    pub fn run_neuron_dynamics(
        condensed_triggered_neuron_indexes: &NPUNeuronMembranePotentialsMPGroupedTableCPU<FGQ>,
        neuron_cortical_descriptor_lookup_table: &CorticalDescriptorLookupTableCPU<FGQ, 0>,
        cortical_area_data_mapping_table: &CorticalAreaDataMappingTableCPU<FGQ, 2>,

        mp_quant_typed_fcl: &mut NPUNeuronMembranePotentialsMPGroupedTableCPU<FGQ>, // we zero out after
        mp_quant_model_typed_neuron_data: &mut E,
        mp_quant_typed_neuron_membrane_potentials: &mut F,
    )
    {
        todo!()
    }

}