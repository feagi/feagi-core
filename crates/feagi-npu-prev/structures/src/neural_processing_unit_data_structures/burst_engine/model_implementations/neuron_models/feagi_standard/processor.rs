use core::marker::PhantomData;
use feagi_structures::feagi_data::quantization_levels::feagi_global_quantization::FeagiGlobalQuantization;
use feagi_structures::feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use crate::neural_processing_unit_data_structures::wrappers::{NPUWrappedBurstEngineBurstIndex, NPUWrappedNeuronCorticalLocalIndex, NPUWrappedNeuronMembranePotential};
use crate::neural_processing_unit_data_structures::burst_engine::common_traits::neuron_model_traits::neuron_model_processor::{NeuronModelProcessor, NeuronModelProcessorBase, NeuronModelProcessorBaseCPU, NeuronModelProcessorWithBurstHistoryCPU, NeuronModelProcessorWithHistory};
use crate::neural_processing_unit_data_structures::burst_engine::model_implementations::neuron_models::feagi_standard::data::{FeagiStandardModelCorticalDataCPU, FeagiStandardModelNeuronDataCPU};
use crate::neural_processing_unit_data_structures::burst_engine::model_implementations::neuron_models::feagi_standard::quantization::FeagiStandardModelQuantization;
use crate::neural_processing_unit_data_structures::burst_engine::engines::rayon::npu_data::npu_structured::burst_engine_global::CorticalLayoutDimensionalCPU;

pub struct FeagiStandardModelProcessorCPU<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization, // fsm quant impl
{
    // No actual members
    _p: PhantomData<(FGQ, NMQ)>,
}


impl<FGQ, NMQ> NeuronModelProcessorBase<FGQ, NMQ, FeagiStandardModelCorticalDataCPU<NMQ>, FeagiStandardModelNeuronDataCPU<NMQ>> for FeagiStandardModelProcessorCPU<FGQ, NMQ> where FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization, {}

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

impl<FGQ, NMQ> NeuronModelProcessorBaseCPU<FGQ, NMQ, FeagiStandardModelCorticalDataCPU<NMQ>, FeagiStandardModelNeuronDataCPU<NMQ>> for FeagiStandardModelProcessorCPU<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
{}

impl<FGQ, NMQ> NeuronModelProcessorWithHistory<FGQ, NMQ, FeagiStandardModelCorticalDataCPU<NMQ>, FeagiStandardModelNeuronDataCPU<NMQ>> for FeagiStandardModelProcessorCPU<FGQ, NMQ> where FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization, {}

impl<FGQ, NMQ> NeuronModelProcessorWithBurstHistoryCPU<
    FGQ,
    NMQ,
    FeagiStandardModelCorticalDataCPU<NMQ>,
    FeagiStandardModelNeuronDataCPU<NMQ>
> for FeagiStandardModelProcessorCPU<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
{
    fn process_neuron_potential_for_dimensional_cortical_configuration(
        incoming_potential: &NPUWrappedNeuronMembranePotential<<NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant>,
        neuron_linear_index: &NPUWrappedNeuronCorticalLocalIndex<FGQ::NeuronIndexCountQuant>,
        burst_index: &NPUWrappedBurstEngineBurstIndex<FGQ::GlobalBurstIndexQuant>,
        burst_index_of_last_activity: &NPUWrappedBurstEngineBurstIndex<FGQ::GlobalBurstIndexQuant>,
        burst_index_of_last_firing: &NPUWrappedBurstEngineBurstIndex<FGQ::GlobalBurstIndexQuant>,
        cortical_layout_dimensional: &CorticalLayoutDimensionalCPU<FGQ>,
        cortical_area_data: &FeagiStandardModelCorticalDataCPU<NMQ>,
        neuron_model_data: &mut FeagiStandardModelNeuronDataCPU<NMQ>,
        this_neuron_potential: &mut NPUWrappedNeuronMembranePotential<<NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant>
    ) -> bool {
        *this_neuron_potential += *incoming_potential;
        return true;
    }
}

