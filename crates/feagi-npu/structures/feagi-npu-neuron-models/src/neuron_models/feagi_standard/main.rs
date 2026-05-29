use core::marker::PhantomData;
use feagi_structures::feagi_data::feagi_ecs::element::{FeagiECSElementOnCPU, FeagiECSElementOnDevice};
use feagi_structures::feagi_data::quantizable_linear::base_types::{QuantizedDecimalTrait, QuantizedIndexCountTrait};
use feagi_structures::feagi_data::shared_quantization_sets::{CorticalAreaModelQuantizationBase, CorticalAreasIndexQuantization};
use crate::shared_traits_and_structs::neuron_model_common::cortical_configuration::CorticalConfiguration;
use crate::shared_traits_and_structs::neuron_model_common::cortical_data_traits::CorticalModelData;
use crate::shared_traits_and_structs::neuron_model_common::neuron_data_traits::{NeuronDataCommon, NeuronDataCommonCPU};

/// The quantization parameters for this neuron model
pub trait FeagiStandardModelQuantizationTrait:
CorticalAreaModelQuantizationBase
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

impl CorticalAreaModelQuantizationBase for TempFeagiStandardQuant32 {
    type NeuronPotentialQuant = f32;
}

impl FeagiStandardModelQuantizationTrait for TempFeagiStandardQuant32 {
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



pub struct FeagiStandardModelCorticalDataCPUGeneric<CAIQ, FSMQ>
where
    CAIQ: CorticalAreasIndexQuantization,
    FSMQ: FeagiStandardModelQuantizationTrait,
{
    pub excitability: FSMQ::ExcitabilityQuant,
    pub refractory_period_limit: FSMQ::RefractoryPeriodLimitQuant,
    pub fire_threshold_limit: FSMQ::FireThresholdLimit,
    pub consecutive_fire_limit: FSMQ::ConsecutiveFireLimit,
    _p: PhantomData<(CAIQ, FSMQ)>,
}

impl<CAIQ, FSMQ> FeagiECSElementOnDevice for FeagiStandardModelCorticalDataCPUGeneric<CAIQ, FSMQ> where CAIQ: CorticalAreasIndexQuantization, FSMQ: FeagiStandardModelQuantizationTrait, {}

impl<CAIQ, FSMQ> FeagiECSElementOnCPU for FeagiStandardModelCorticalDataCPUGeneric<CAIQ, FSMQ> where CAIQ: CorticalAreasIndexQuantization, FSMQ: FeagiStandardModelQuantizationTrait, {}

impl<CAIQ, FSMQ> CorticalModelData<CAIQ, FSMQ> for FeagiStandardModelCorticalDataCPUGeneric<CAIQ, FSMQ> where CAIQ: CorticalAreasIndexQuantization, FSMQ: FeagiStandardModelQuantizationTrait, {}



pub struct FeagiStandardModelNeuronDataCPUGeneric<CAIQ, FSMQ, CC, CMC>
where
    CAIQ: CorticalAreasIndexQuantization,
    FSMQ: FeagiStandardModelQuantizationTrait,
    CC: CorticalConfiguration<CAIQ>,
    CMC: CorticalModelData<CAIQ, FSMQ>
{
    neuron_burst_index_of_last_fire: CAIQ::GlobalBurstIndexQuant,
    neuron_fire_threshold: FSMQ::NeuronPotentialQuant,
    neuron_leak_coefficient: FSMQ::LeakCoefficientQuant,
    neuron_refractory_countdown: FSMQ::RefractoryCountdownQuant,
    neuron_consecutive_fire_countdown: FSMQ::ConsecutiveFireCountdownQuant,
    _p: PhantomData<(CAIQ, FSMQ, CC, CMC)>,
}

impl<CAIQ, FSMQ, CC, CMC> NeuronDataCommon<CAIQ, FSMQ, CC, CMC> for FeagiStandardModelNeuronDataCPUGeneric<CAIQ, FSMQ, CC, CMC>
where
    CAIQ: CorticalAreasIndexQuantization,
    FSMQ: FeagiStandardModelQuantizationTrait,
    CC: CorticalConfiguration<CAIQ>,
    CMC: CorticalModelData<CAIQ, FSMQ>
{
    const NEURON_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER: bool = false;
}

impl<CAIQ, FSMQ, CC, CMC> FeagiECSElementOnDevice for FeagiStandardModelNeuronDataCPUGeneric<CAIQ, FSMQ, CC, CMC>
where
    CAIQ: CorticalAreasIndexQuantization,
    FSMQ: FeagiStandardModelQuantizationTrait,
    CC: CorticalConfiguration<CAIQ>,
    CMC: CorticalModelData<CAIQ, FSMQ>
{}

impl<CAIQ, FSMQ, CC, CMC> FeagiECSElementOnCPU for FeagiStandardModelNeuronDataCPUGeneric<CAIQ, FSMQ, CC, CMC>
where
    CAIQ: CorticalAreasIndexQuantization,
    FSMQ: FeagiStandardModelQuantizationTrait,
    CC: CorticalConfiguration<CAIQ>,
    CMC: CorticalModelData<CAIQ, FSMQ>
{}

impl<CAIQ, FSMQ, CC, CMC> NeuronDataCommonCPU<CAIQ, FSMQ, CC, CMC> for FeagiStandardModelNeuronDataCPUGeneric<CAIQ, FSMQ, CC, CMC>
where
    CAIQ: CorticalAreasIndexQuantization,
    FSMQ: FeagiStandardModelQuantizationTrait,
    CC: CorticalConfiguration<CAIQ>,
    CMC: CorticalModelData<CAIQ, FSMQ>
{
    fn process_neuron_potential<IPQuant: QuantizedDecimalTrait>(
        &mut self, incoming_neuron_potential: &IPQuant,
        this_neuron_linear_index: &CAIQ::NeuronIndexCountQuant,
        cortical_configuration: &CC,
        cortical_model_data: &CMC,
        self_neuron_potential: &mut FSMQ::NeuronPotentialQuant) -> bool {



        // TODO obviously a temp setup
        self_neuron_potential.load_f32_inplace(incoming_neuron_potential.to_f32() * 1.05);
        false
    }
}

