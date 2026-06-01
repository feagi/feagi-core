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


// TODO obviously temporary
pub struct TempFeagiStandardQuant32;

impl NeuronModelQuantization for TempFeagiStandardQuant32 {
    type NeuronPotentialQuant = f32;
}

impl FeagiStandardModelQuantization for TempFeagiStandardQuant32 {
    type LeakCoefficientQuant = f32;
    type ConsecutiveFireCountdownQuant = u32;
    type RefractoryCountdownQuant = u32;
    type ExcitabilityQuant = f32;
    type RefractoryPeriodLimitQuant = u32;
    type FireThresholdLimit = f32;
    type ConsecutiveFireLimit = u32;
}


// TODO lets think about how we tackle this. Expecting something dynamic in nature, but can
// we try avoiding box anyways? Or maybe we shouldnt in this case and as_any will get us through
// this issue
/*
pub enum TempFeagiStandardQuantizations {
    Bit32(TempFeagiStandardQuant32)
}

impl TempFeagiStandardQuantizations {
    fn get_quantization_impl(self) -> dyn FeagiStandardModelQuantizationTrait
    {
        match self {
            TempFeagiStandardQuantizations::Bit32(x) => {

            }
        }
    }
}

 */



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

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> PDITagGenericDevice for FeagiStandardModelCorticalDataCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> PDITagCPU for FeagiStandardModelCorticalDataCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> PDIElement for FeagiStandardModelCorticalDataCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> CorticalModelData<FGQ, NMQ> for FeagiStandardModelCorticalDataCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> CorticalModelDataCPU<FGQ, NMQ> for FeagiStandardModelCorticalDataCPU<FGQ, NMQ> {}

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



pub struct FeagiStandardModelProcessorCPU<FGQ, NMQ, CCC, CMD, NMD>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    CCC: CorticalConfigurationDimensional<FGQ>,
    CMD: CorticalModelDataCPU<FGQ, NMQ>,
    NMD: NeuronModelDataCPU<FGQ, NMQ>
{
    // No actual members
    _p: PhantomData<(FGQ, NMQ, CCC, CMD, NMD)>,
}

impl<FGQ, NMQ, CCC, CMD, NMD> PDITagGenericDevice for FeagiStandardModelProcessorCPU<FGQ, NMQ, CCC, CMD, NMD>
where CCC: CorticalConfigurationDimensional<FGQ>, CMD: CorticalModelDataCPU<FGQ, NMQ>, FGQ: FeagiGlobalQuantization, NMD: NeuronModelDataCPU<FGQ, NMQ>, NMQ: NeuronModelQuantization, {}

impl<FGQ, NMQ, CCC, CMD, NMD> PDITagCPU for FeagiStandardModelProcessorCPU<FGQ, NMQ, CCC, CMD, NMD>
where CCC: CorticalConfigurationDimensional<FGQ>, CMD: CorticalModelDataCPU<FGQ, NMQ>, FGQ: FeagiGlobalQuantization, NMD: NeuronModelDataCPU<FGQ, NMQ>, NMQ: NeuronModelQuantization, {}

impl<FGQ, NMQ, CCC, CMD, NMD> PDIElement for FeagiStandardModelProcessorCPU<FGQ, NMQ, CCC, CMD, NMD>
where CCC: CorticalConfigurationDimensional<FGQ>, CMD: CorticalModelDataCPU<FGQ, NMQ>, FGQ: FeagiGlobalQuantization, NMD: NeuronModelDataCPU<FGQ, NMQ>, NMQ: NeuronModelQuantization, {}

impl<FGQ, NMQ, CCC, CMD, NMD> NeuronModelProcessor<FGQ, NMQ, CCC, CMD, NMD> for FeagiStandardModelProcessorCPU<FGQ, NMQ, CCC, CMD, NMD>
where CCC: CorticalConfigurationDimensional<FGQ>, CMD: CorticalModelDataCPU<FGQ, NMQ>, FGQ: FeagiGlobalQuantization, NMD: NeuronModelDataCPU<FGQ, NMQ>, NMQ: NeuronModelQuantization,
{
    const MODEL_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER: bool = true;
}

impl<FGQ, NMQ, CCC, CMD, NMD> NeuronModelProcessorCPU<FGQ, NMQ, CCC, CMD, NMD> for FeagiStandardModelProcessorCPU<FGQ, NMQ, CCC, CMD, NMD>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    CCC: CorticalConfigurationDimensional<FGQ>,
    CMD: CorticalModelDataCPU<FGQ, NMQ>,
    NMD: NeuronModelDataCPU<FGQ, NMQ>
{
    fn process_neuron_potential<IPQuant: QuantizedDecimalTrait>(
        &self,
        incoming_neuron_potential: &IPQuant,
        neuron_linear_index: &FGQ::NeuronIndexCountQuant,
        burst_index: &FGQ::GlobalBurstIndexQuant,
        cortical_area_configuration: &CCC,
        cortical_area_data: &CMD,
        neuron_model_data: &mut NMD,
        this_neuron_potential: &mut NMQ::NeuronPotentialQuant)
        -> bool
    {
        todo!()
    }

    fn prepare_cortical_data_for_burst_index_rollover(
        &self,
        cortical_area_data: &mut CMD)
    {
        todo!()
    }

    fn prepare_neuron_data_for_burst_index_rollover(
        &self,
        neuron_linear_index: &FGQ::NeuronIndexCountQuant,
        neuron_model_data: &mut NMD)
    {
        todo!()
    }
}

