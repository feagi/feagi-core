use core::marker::PhantomData;
use feagi_structures::feagi_data::quantizable_linear::base_types::{QuantizedDecimalTrait, QuantizedIndexCountTrait};
use feagi_structures::feagi_data::shared_quantization_sets::{CorticalAreaModelQuantizationBase, CorticalAreasIndexQuantization};
use crate::shared_traits_and_structs::neuron_model_common::cortical_data_traits::{NeuronModelCorticalDataCommonCPU, NeuronModelCorticalDataCommonDevice, NeuronModelCorticalDataDimensionalCPUTemplate, NeuronModelCorticalDataDimensionalDevice};


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

impl<CAIQ, FSMQ> NeuronModelCorticalDataCommonDevice<CAIQ, FSMQ> for FeagiStandardModelCorticalDataCPUGeneric<CAIQ, FSMQ> where CAIQ: CorticalAreasIndexQuantization, FSMQ: FeagiStandardModelQuantizationTrait, {}

impl<CAIQ, FSMQ> NeuronModelCorticalDataDimensionalDevice<CAIQ, FSMQ> for FeagiStandardModelCorticalDataCPUGeneric<CAIQ, FSMQ> where CAIQ: CorticalAreasIndexQuantization, FSMQ: FeagiStandardModelQuantizationTrait, {}

impl<CAIQ, FSMQ> NeuronModelCorticalDataCommonCPU<CAIQ, FSMQ> for FeagiStandardModelCorticalDataCPUGeneric<CAIQ, FSMQ> where CAIQ: CorticalAreasIndexQuantization, FSMQ: FeagiStandardModelQuantizationTrait, {}

impl<CAIQ, FSMQ> NeuronModelCorticalDataDimensionalCPUTemplate<CAIQ, FSMQ> for FeagiStandardModelCorticalDataCPUGeneric<CAIQ, FSMQ>
where
    CAIQ: CorticalAreasIndexQuantization,
    FSMQ: FeagiStandardModelQuantizationTrait,
{

}



pub struct FeagiStandardModelNeuronDataCPUGeneric<CAIQ, FSMQ, CSD, CNMCD>
where
    CAIQ: CorticalAreasIndexQuantization,
    FSMQ: FeagiStandardModelQuantizationTrait,
{
    neuron_burst_index_of_last_fire: CAIQ::GlobalBurstIndexQuant,
    neuron_fire_threshold: FSMQ::NeuronPotentialQuant,
    neuron_leak_coefficient: FSMQ::LeakCoefficientQuant,
    neuron_refractory_countdown: FSMQ::RefractoryCountdownQuant,
    neuron_consecutive_fire_countdown: FSMQ::ConsecutiveFireCountdownQuant,
    _p: PhantomData<(CAIQ, FSMQ, CSD, CNMCD)>,
}


