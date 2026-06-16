use std::marker::PhantomData;
use feagi_structures::feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use crate::neural_processing_unit_data_structures::burst_engine::common_traits::neuron_model_traits::neuron_model_cortical_data::{NeuronModelCorticalData, NeuronModelCorticalDataCPU};
use crate::neural_processing_unit_data_structures::burst_engine::common_traits::neuron_model_traits::neuron_model_neuron_data::{NeuronModelNeuronData, NeuronModelNeuronDataCPU};
use crate::neural_processing_unit_data_structures::burst_engine::model_implementations::neuron_models::feagi_standard::quantization::FeagiStandardModelQuantization;

#[derive(Debug, Copy, Clone)]
pub struct FeagiStandardModelCorticalDataCPU<NMQ>
where
    NMQ: FeagiStandardModelQuantization,
{
    pub excitability: NMQ::CorticalExcitabilityQuant,
    pub refractory_period_limit: NMQ::CorticalRefractoryPeriodLimitQuant,
    pub fire_threshold_limit: NMQ::CorticalFireThresholdLimit,
    pub consecutive_fire_limit: NMQ::CorticalConsecutiveFireLimit,
    _p: PhantomData<( NMQ)>,
}


impl<NMQ> NeuronModelCorticalData<NMQ> for FeagiStandardModelCorticalDataCPU<NMQ>
where
    NMQ: FeagiStandardModelQuantization,
{
    const MODEL_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER: bool = true;
    const MODEL_SUPPORTS_CORTICAL_LAYOUT_DIMENSIONAL: bool = true;
}

impl<NMQ> NeuronModelCorticalDataCPU<NMQ> for FeagiStandardModelCorticalDataCPU<NMQ>
where
    NMQ: FeagiStandardModelQuantization,
{

}

impl<NMQ> FeagiStandardModelCorticalDataCPU<NMQ>
where
    NMQ: FeagiStandardModelQuantization,
{
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


#[derive(Debug, Copy, Clone)]
pub struct FeagiStandardModelNeuronDataCPU<NMQ>
where
    NMQ: FeagiStandardModelQuantization,
{
    pub neuron_fire_threshold: <NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant,
    pub neuron_leak_coefficient: NMQ::NeuronLeakCoefficientQuant,
    pub neuron_refractory_countdown: NMQ::NeuronRefractoryCountdownQuant,
    pub neuron_consecutive_fire_countdown: NMQ::NeuronConsecutiveFireCountdownQuant,
    _p: PhantomData<(NMQ)>,
}

impl<NMQ> FeagiStandardModelNeuronDataCPU<NMQ>
where
    NMQ: FeagiStandardModelQuantization,
{
    pub fn new(
        neuron_fire_threshold: <NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant,
        neuron_leak_coefficient: NMQ::NeuronLeakCoefficientQuant,
        neuron_refractory_countdown: NMQ::NeuronRefractoryCountdownQuant,
        neuron_consecutive_fire_countdown: NMQ::NeuronConsecutiveFireCountdownQuant,
    ) -> Self
    {
        Self {
            neuron_fire_threshold,
            neuron_leak_coefficient,
            neuron_refractory_countdown,
            neuron_consecutive_fire_countdown,
            _p: PhantomData,
        }
    }
}

impl<NMQ> NeuronModelNeuronData<
    NMQ,
> for FeagiStandardModelNeuronDataCPU<NMQ>
where
    NMQ: FeagiStandardModelQuantization,
{
}

impl<NMQ> NeuronModelNeuronDataCPU<
    NMQ,
> for FeagiStandardModelNeuronDataCPU<NMQ>
where
    NMQ: FeagiStandardModelQuantization,
{

}