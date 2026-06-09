use std::marker::PhantomData;
use feagi_structures::feagi_data::feagi_pdi::PDIElement;
use feagi_structures::feagi_data::feagi_pdi::tag_device::{PDITagCPU, PDITagGenericDevice};
use feagi_structures::feagi_data::shared_quantization_sets::FeagiGlobalQuantization;
use crate::neuron_models::neuron_models::feagi_standard::quantization::FeagiStandardModelQuantization;

#[repr(C)]
pub struct FeagiStandardModelCorticalDataCPU<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
{
    pub excitability: NMQ::CorticalExcitabilityQuant,
    pub refractory_period_limit: NMQ::CorticalRefractoryPeriodLimitQuant,
    pub fire_threshold_limit: NMQ::CorticalFireThresholdLimit,
    pub consecutive_fire_limit: NMQ::CorticalConsecutiveFireLimit,
    _p: PhantomData<(FGQ, NMQ)>,
}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> PDITagGenericDevice for FeagiStandardModelCorticalDataCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> PDITagCPU for FeagiStandardModelCorticalDataCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> PDIElement for FeagiStandardModelCorticalDataCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> FeagiStandardModelCorticalDataCPU<FGQ, NMQ> {
    pub fn new(
        excitability: NMQ::CorticalExcitabilityQuant,
        refractory_period_limit: NMQ::CorticalRefractoryPeriodLimitQuant,
        fire_threshold_limit: NMQ::CorticalFireThresholdLimit,
        consecutive_fire_limit: NMQ::CorticalConsecutiveFireLimit,
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

// TODO needs padding!
#[repr(C)]
pub struct FeagiStandardModelNeuronDataCPU<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
{
    pub neuron_fire_threshold: NMQ::CorticalPotentialQuant::NeuronPotentialQuant,
    pub neuron_leak_coefficient: NMQ::NeuronLeakCoefficientQuant,
    pub neuron_refractory_countdown: NMQ::NeuronRefractoryCountdownQuant,
    pub neuron_consecutive_fire_countdown: NMQ::NeuronConsecutiveFireCountdownQuant,
    pub neuron_burst_index_of_last_fire: FGQ::GlobalBurstIndexQuant,
    _p: PhantomData<(FGQ, NMQ)>,
}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> PDITagGenericDevice for FeagiStandardModelNeuronDataCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> PDITagCPU for FeagiStandardModelNeuronDataCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> PDIElement for FeagiStandardModelNeuronDataCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> FeagiStandardModelNeuronDataCPU<FGQ, NMQ> {
    pub fn new(
        neuron_burst_index_of_last_fire: FGQ::GlobalBurstIndexQuant,
        neuron_fire_threshold: NMQ::CorticalPotentialQuant::NeuronPotentialQuant,
        neuron_leak_coefficient: NMQ::NeuronLeakCoefficientQuant,
        neuron_refractory_countdown: NMQ::NeuronRefractoryCountdownQuant,
        neuron_consecutive_fire_countdown: NMQ::NeuronConsecutiveFireCountdownQuant
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