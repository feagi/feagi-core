use core::marker::PhantomData;
use feagi_structures::feagi_data::feagi_pdi::PDIElement;
use feagi_structures::feagi_data::feagi_pdi::tag_device::{PDITagCPU, PDITagGenericDevice};
use feagi_structures::feagi_data::shared_quantization_sets::{CorticalPotentialQuantization, FeagiGlobalQuantization};
use crate::neural_processing_unit_data_structures::cpu_wrappers::indexes_global::NPUNeuronMembranePotential;
use crate::neural_processing_unit_data_structures::cpu_wrappers::cortical_spatial::NPUNeuronIndexCorticalLocal;
use crate::neuron_models::base_traits_cpu::{NeuronModelProcessorCPU};
use crate::neural_processing_unit_data_structures::cortical_structure_configuration::{CorticalConfigurationBase};
use crate::neuron_models::base_traits_all_devices::NeuronModelProcessor;
use crate::neuron_models::neuron_models::feagi_standard::data::{FeagiStandardModelCorticalDataCPU, FeagiStandardModelNeuronDataCPU};
use crate::neuron_models::neuron_models::feagi_standard::quantization::FeagiStandardModelQuantization;
use crate::npu_descriptors::NPUGlobalBurstCounter;

pub struct FeagiStandardModelProcessorCPU<FGQ, NMQ, CCB>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization, // fsm quant impl
    CCB: CorticalConfigurationBase<FGQ, NMQ::CorticalPotentialQuant>
{
    // No actual members
    _p: PhantomData<(FGQ, NMQ, CCB)>,
}

//region Tag Traits

impl<FGQ, NMQ, CCB> PDIElement for FeagiStandardModelProcessorCPU<FGQ, NMQ, CCB>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
    CCB: CorticalConfigurationBase<FGQ, NMQ::CorticalPotentialQuant> {}

impl<FGQ, NMQ, CCB> PDITagGenericDevice for FeagiStandardModelProcessorCPU<FGQ, NMQ, CCB>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
    CCB: CorticalConfigurationBase<FGQ, NMQ::CorticalPotentialQuant> {}

impl<FGQ, NMQ, CCB> PDITagCPU for FeagiStandardModelProcessorCPU<FGQ, NMQ, CCB>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
    CCB: CorticalConfigurationBase<FGQ, NMQ::CorticalPotentialQuant> {}

//endregion

impl<FGQ, NMQ, CCB> NeuronModelProcessor<
    FGQ,
    NMQ,
    CCB,
    FeagiStandardModelCorticalDataCPU<FGQ, NMQ, CCB>,
    FeagiStandardModelNeuronDataCPU<FGQ, NMQ>,
> for FeagiStandardModelProcessorCPU<FGQ, NMQ, CCB>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
    CCB: CorticalConfigurationBase<FGQ, NMQ::CorticalPotentialQuant>,
{
    const MODEL_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER: bool = false;
}

impl<FGQ, NMQ, CCB> NeuronModelProcessorCPU<
    FGQ,
    NMQ,
    CCB,
    FeagiStandardModelCorticalDataCPU<FGQ, NMQ, CCB>,
    FeagiStandardModelNeuronDataCPU<FGQ, NMQ>> for FeagiStandardModelProcessorCPU<FGQ, NMQ, CCB>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
    CCB: CorticalConfigurationBase<FGQ, NMQ::CorticalPotentialQuant> // TODO CPU trait!
{
    fn process_neuron_potential
    (
        &self,
        incoming_neuron_potential:  &NPUNeuronMembranePotential<<NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant>,
        neuron_linear_index: &NPUNeuronIndexCorticalLocal<FGQ::NeuronIndexCountQuant>,
        burst_index:  &NPUGlobalBurstCounter<FGQ::GlobalBurstIndexQuant>,
        cortical_area_configuration:  &CCB,
        cortical_area_data: &FeagiStandardModelCorticalDataCPU<FGQ, NMQ, CCB>,
        neuron_model_data: &mut FeagiStandardModelNeuronDataCPU<FGQ, NMQ>,
        this_neuron_potential: &mut NPUNeuronMembranePotential<<NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant>
    )
        -> bool
    {
        todo!()
    }

    /// If enabled via the const, this method will be called on all neurons of that
    /// neuron model type right before the global burst index overflows and resets to 0. Use this
    /// method to update any values that need to be updated in that case
    fn prepare_cortical_data_for_burst_index_rollover(
        &self,
        cortical_area_data: &mut FeagiStandardModelCorticalDataCPU<FGQ, NMQ, CCB>)
    {
        todo!()
    }

    fn prepare_neuron_data_for_burst_index_rollover(
        &self,
        neuron_linear_index: &NPUNeuronIndexCorticalLocal<FGQ::NeuronIndexCountQuant>,
        neuron_model_data: &mut FeagiStandardModelNeuronDataCPU<FGQ, NMQ>)
    {
        todo!()
    }
}

