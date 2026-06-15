use core::marker::PhantomData;
use feagi_structures::feagi_data::quantization_levels::feagi_global_quantization::FeagiGlobalQuantization;
use feagi_structures::feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use crate::neural_processing_unit_data_structures::burst_engine::descriptor_flags::cortical_area_layout::CorticalLayoutDimensionalCPU;
use crate::neural_processing_unit_data_structures::wrappers::{NPUWrappedNeuronCorticalLocalIndex, NPUWrappedNeuronMembranePotential};
use crate::neural_processing_unit_data_structures::burst_engine::common_traits::neuron_model_traits::neuron_model_processor::{NeuronModelProcessor, NeuronModelProcessorCPU};
use crate::neural_processing_unit_data_structures::neuron_models::neuron_models::feagi_standard::data::{FeagiStandardModelCorticalDataCPU, FeagiStandardModelNeuronDataCPU};
use crate::neural_processing_unit_data_structures::neuron_models::neuron_models::feagi_standard::quantization::FeagiStandardModelQuantization;
use crate::npu_descriptors::NPUGlobalBurstCounter;

pub struct FeagiStandardModelProcessorCPU<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization, // fsm quant impl
{
    // No actual members
    _p: PhantomData<(FGQ, NMQ)>,
}

//region Tag Traits



//endregion

impl<FGQ, NMQ> NeuronModelProcessor<
    FGQ,
    NMQ,
    FeagiStandardModelCorticalDataCPU<NMQ>,
    FeagiStandardModelNeuronDataCPU<NMQ>,
> for FeagiStandardModelProcessorCPU<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
{
}

impl<FGQ, NMQ> NeuronModelProcessorCPU<
    FGQ,
    NMQ,
    FeagiStandardModelCorticalDataCPU<NMQ>,
    FeagiStandardModelNeuronDataCPU<NMQ>
> for FeagiStandardModelProcessorCPU<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
{
    fn create_blank_cortical_area_of_cortical_configuration_dimensional(cortical_area_layout: &CorticalLayoutDimensionalCPU<FGQ, NMQ::CorticalPotentialQuant>) -> FeagiStandardModelCorticalDataCPU<NMQ> {
        todo!()
    }

    fn create_blank_neuron_of_cortical_configuration_dimensional(neuron_linear_index: &NPUWrappedNeuronCorticalLocalIndex<FGQ::NeuronIndexCountQuant>, burst_index: &NPUGlobalBurstCounter<FGQ::GlobalBurstIndexQuant>, cortical_area_layout: &CorticalLayoutDimensionalCPU<FGQ, NMQ::CorticalPotentialQuant>, cortical_area_data: &FeagiStandardModelCorticalDataCPU<NMQ>) -> FeagiStandardModelNeuronDataCPU<NMQ> {
        todo!()
    }

    fn process_neuron_potential_for_dimensional_cortical_configuration(
        incoming_potential: &NPUWrappedNeuronMembranePotential<<NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant>,
        neuron_linear_index: &NPUWrappedNeuronCorticalLocalIndex<FGQ::NeuronIndexCountQuant>,
        burst_index: &NPUGlobalBurstCounter<FGQ::GlobalBurstIndexQuant>,
        burst_index_of_last_activity: &NPUGlobalBurstCounter<FGQ::GlobalBurstIndexQuant>,
        cortical_layout_dimensional: &CorticalLayoutDimensionalCPU<FGQ, NMQ::CorticalPotentialQuant>,
        cortical_area_data: &FeagiStandardModelCorticalDataCPU<NMQ>,
        neuron_model_data: &mut FeagiStandardModelNeuronDataCPU<NMQ>,
        this_neuron_potential: &mut NPUWrappedNeuronMembranePotential<<NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant>)
        -> bool {
        todo!()
    }
}

