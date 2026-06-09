use core::marker::PhantomData;
use feagi_structures::feagi_data::feagi_pdi::PDIElement;
use feagi_structures::feagi_data::feagi_pdi::tag_device::{PDITagCPU, PDITagGenericDevice};
use feagi_structures::feagi_data::quantizable_linear::base_types::{QuantizedDecimalTrait, QuantizedIndexCountTrait};
use feagi_structures::feagi_data::shared_quantization_sets::{NeuronModelQuantization, FeagiGlobalQuantization};
use crate::shared_traits_and_structs::base_traits_all_devices::{NeuronModelProcessor, CorticalModelData, NeuronModelData};
use crate::shared_traits_and_structs::base_traits_cpu::{NeuronModelProcessorCPU, CorticalModelDataCPU, NeuronModelDataCPU};
use crate::shared_traits_and_structs::cortical_configuration::{CorticalConfiguration, CorticalConfigurationDimensional, CorticalConfigurationDimensionalCPU};
// TODO derive macro for cortical data, neuron data trait impls! (take class like CPU)



/// The quantization parameters for this neuron model
pub trait FeagiStandardModelQuantization:
NeuronModelQuantization
{
    type LeakCoefficientQuant: QuantizedDecimalTrait;
    type ConsecutiveFireCountdownQuant: QuantizedIndexCountTrait;
    type RefractoryCountdownQuant: QuantizedIndexCountTrait;

    type ExcitabilityQuant: QuantizedDecimalTrait;
    type RefractoryPeriodLimitQuant: QuantizedIndexCountTrait;
    type FireThresholdLimit: QuantizedDecimalTrait;
    type ConsecutiveFireLimit: QuantizedIndexCountTrait;
}

//region Quantization Instantiation
#[derive(Default)]
pub enum FeagiStandardModelQuantizationAndDeviceMode {
    
    #[default]
    CPU32Bit(FeagiStandardModelCPU32Bit)
}

#[derive(Default)]
pub(crate) struct FeagiStandardModelCPU32Bit;

impl NeuronModelQuantization for FeagiStandardModelCPU32Bit {
    type NeuronPotentialQuant = f32;
}

impl FeagiStandardModelQuantization for FeagiStandardModelCPU32Bit {
    type LeakCoefficientQuant = f32;
    type ConsecutiveFireCountdownQuant = u32;
    type RefractoryCountdownQuant = u32;
    type ExcitabilityQuant = f32;
    type RefractoryPeriodLimitQuant = u32;
    type FireThresholdLimit = f32;
    type ConsecutiveFireLimit = u32;
}



//endregion


// TODO obviously temporary







pub struct FeagiStandardModelCorticalDataCPU<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
{
    pub excitability: NMQ::ExcitabilityQuant,
    pub refractory_period_limit: NMQ::RefractoryPeriodLimitQuant,
    pub fire_threshold_limit: NMQ::FireThresholdLimit,
    pub consecutive_fire_limit: NMQ::ConsecutiveFireLimit,
    _p: PhantomData<(FGQ, NMQ)>,
}

//region Tag Traits

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> PDITagGenericDevice for FeagiStandardModelCorticalDataCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> PDITagCPU for FeagiStandardModelCorticalDataCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> PDIElement for FeagiStandardModelCorticalDataCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> CorticalModelData<FGQ, NMQ> for FeagiStandardModelCorticalDataCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> CorticalModelDataCPU<FGQ, NMQ> for FeagiStandardModelCorticalDataCPU<FGQ, NMQ> {

}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> FeagiStandardModelCorticalDataCPU<FGQ, NMQ> {
    pub fn new(
        excitability: NMQ::ExcitabilityQuant,
        refractory_period_limit: NMQ::RefractoryPeriodLimitQuant,
        fire_threshold_limit: NMQ::FireThresholdLimit,
        consecutive_fire_limit: NMQ::ConsecutiveFireLimit,
    ) -> Self {
        Self {
            excitability,
            refractory_period_limit,
            fire_threshold_limit,
            consecutive_fire_limit,
            _p: PhantomData,
        }
    }
}

//endregion

pub struct FeagiStandardModelNeuronDataCPU<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
{
    pub neuron_burst_index_of_last_fire: FGQ::GlobalBurstIndexQuant,
    pub neuron_fire_threshold: NMQ::NeuronPotentialQuant,
    pub neuron_leak_coefficient: NMQ::LeakCoefficientQuant,
    pub neuron_refractory_countdown: NMQ::RefractoryCountdownQuant,
    pub neuron_consecutive_fire_countdown: NMQ::ConsecutiveFireCountdownQuant,
    _p: PhantomData<(FGQ, NMQ)>,
}

//region Tag Traits

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> PDITagGenericDevice for FeagiStandardModelNeuronDataCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> PDITagCPU for FeagiStandardModelNeuronDataCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> PDIElement for FeagiStandardModelNeuronDataCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> NeuronModelData<FGQ, NMQ> for FeagiStandardModelNeuronDataCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> NeuronModelDataCPU<FGQ, NMQ> for FeagiStandardModelNeuronDataCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> FeagiStandardModelNeuronDataCPU<FGQ, NMQ> {
    pub fn new(
        neuron_burst_index_of_last_fire: FGQ::GlobalBurstIndexQuant,
        neuron_fire_threshold: NMQ::NeuronPotentialQuant,
        neuron_leak_coefficient: NMQ::LeakCoefficientQuant,
        neuron_refractory_countdown: NMQ::RefractoryCountdownQuant,
        neuron_consecutive_fire_countdown: NMQ::ConsecutiveFireCountdownQuant
    ) -> Self {
        Self {
            neuron_burst_index_of_last_fire,
            neuron_fire_threshold,
            neuron_leak_coefficient,
            neuron_refractory_countdown,
            neuron_consecutive_fire_countdown,
            _p: PhantomData,
        }
    }


}

//endregion

pub struct FeagiStandardModelProcessorCPU<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization, // fsm quant impl
{
    // No actual members
    _p: PhantomData<(FGQ, NMQ)>,
}

//region Tag Traits

impl<FGQ, NMQ> PDITagGenericDevice for FeagiStandardModelProcessorCPU<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization, {}

impl<FGQ, NMQ> PDITagCPU for FeagiStandardModelProcessorCPU<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization, {}

impl<FGQ, NMQ> PDIElement for FeagiStandardModelProcessorCPU<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization, {}

impl<FGQ, NMQ, CCC, CMD, NMD> NeuronModelProcessor<FGQ, NMQ, CCC, CMD, NMD> for FeagiStandardModelProcessorCPU<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
    CCC: CorticalConfigurationDimensional<FGQ>,
    CMD: CorticalModelDataCPU<FGQ, NMQ>,
    NMD: NeuronModelDataCPU<FGQ, NMQ>,
{
    const MODEL_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER: bool = true;
}

//endregion

impl<FGQ, NMQ> NeuronModelProcessorCPU<
    FGQ,
    NMQ,
    CorticalConfigurationDimensionalCPU<FGQ>,
    FeagiStandardModelCorticalDataCPU<FGQ, NMQ>,
    FeagiStandardModelNeuronDataCPU<FGQ, NMQ>> for FeagiStandardModelProcessorCPU<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
{
    fn process_neuron_potential<IPQuant: QuantizedDecimalTrait>(
        &self,
        incoming_neuron_potential: &IPQuant,
        neuron_linear_index: &FGQ::NeuronIndexCountQuant,
        burst_index: &FGQ::GlobalBurstIndexQuant,
        cortical_area_configuration: &CorticalConfigurationDimensionalCPU<FGQ>,
        cortical_area_data: &FeagiStandardModelCorticalDataCPU<FGQ, NMQ>,
        neuron_model_data: &mut FeagiStandardModelNeuronDataCPU<FGQ, NMQ>,
        this_neuron_potential: &mut NMQ::NeuronPotentialQuant)
        -> bool
    {
        todo!()
    }

    fn prepare_cortical_data_for_burst_index_rollover(
        &self,
        cortical_area_data: &mut FeagiStandardModelCorticalDataCPU<FGQ, NMQ>)
    {
        todo!()
    }

    fn prepare_neuron_data_for_burst_index_rollover(
        &self,
        neuron_linear_index: &FGQ::NeuronIndexCountQuant,
        neuron_model_data: &mut FeagiStandardModelNeuronDataCPU<FGQ, NMQ>)
    {
        todo!()
    }
}

