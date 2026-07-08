use std::marker::PhantomData;
use feagi_data::{create_wrapped_quantized_decimal, create_wrapped_quantized_index};
use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use feagi_npu_common::wrapped_indexes::NeuronMembranePotential;
use crate::neuron_models::feagi_standard::quantization::FeagiStandardModelQuantization;
use crate::neuron_models::neuron_model_traits::neuron_model_data::{NeuronModelCorticalData, NeuronModelNeuronData};

// TODO percentages should be their own types

create_wrapped_quantized_decimal!(pub Excitation);
create_wrapped_quantized_index!(pub RefractoryPeriodLimit);
// Fire threshold is just membrane potential
create_wrapped_quantized_index!(pub ConsecutiveFireLimit);
create_wrapped_quantized_index!(pub SnoozePeriod);
create_wrapped_quantized_decimal!(pub DegeneracyConstant);

create_wrapped_quantized_decimal!(pub LeakCoefficient);
create_wrapped_quantized_index!(pub RefractoryCountdown);
create_wrapped_quantized_index!(pub ConsecutiveFireCountdown);
#[derive(Debug, Copy, Clone)]
pub struct FeagiStandardModelCorticalData<NMQ>
where
    NMQ: FeagiStandardModelQuantization,
{
    pub excitability: Excitation<NMQ::CorticalExcitabilityQuant>,

    pub refractory_period_limit: RefractoryPeriodLimit<NMQ::CorticalRefractoryPeriodLimitQuant>,

    /// Upper limit of fire threshold, over this and we wont fire
    pub fire_threshold_limit: NeuronMembranePotential<NMQ::CorticalFireThreshold>,

    pub consecutive_fire_limit: ConsecutiveFireLimit<NMQ::CorticalConsecutiveFireLimit>,

    pub snooze_period: SnoozePeriod<NMQ::CorticalSnoozePeriod>,

    pub degeneracy_constant: DegeneracyConstant<NMQ::CorticalDegeneracyConstant>,

    _p: PhantomData<NMQ>,
}


impl<NMQ> NeuronModelCorticalData<NMQ> for FeagiStandardModelCorticalData<NMQ>
where
    NMQ: FeagiStandardModelQuantization,
{
    const MODEL_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER: bool = true;
    const MODEL_SUPPORTS_CORTICAL_LAYOUT_DIMENSIONAL: bool = true;
}

impl<NMQ> FeagiStandardModelCorticalData<NMQ>
where
    NMQ: FeagiStandardModelQuantization,
{
    pub fn new(
        excitability: Excitation<NMQ::CorticalExcitabilityQuant>,
        refractory_period_limit: RefractoryPeriodLimit<NMQ::CorticalRefractoryPeriodLimitQuant>,
        fire_threshold_limit: NeuronMembranePotential<NMQ::CorticalFireThreshold>,
        consecutive_fire_limit: ConsecutiveFireLimit<NMQ::CorticalConsecutiveFireLimit>,
        snooze_period: SnoozePeriod<NMQ::CorticalSnoozePeriod>,
        degeneracy_constant: DegeneracyConstant<NMQ::CorticalDegeneracyConstant>,
    ) -> Self {
        Self {
            excitability,
            refractory_period_limit,
            fire_threshold_limit,
            consecutive_fire_limit,
            snooze_period,
            degeneracy_constant,
            _p: PhantomData,
        }
    }
}


#[derive(Debug, Copy, Clone)]
pub struct FeagiStandardModelNeuronData<NMQ>
where
    NMQ: FeagiStandardModelQuantization,
{
    pub neuron_fire_threshold: NeuronMembranePotential<<NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant>,
    pub neuron_leak_coefficient: LeakCoefficient<NMQ::NeuronLeakCoefficientQuant>,
    pub neuron_refractory_countdown: RefractoryCountdown<NMQ::NeuronRefractoryCountdownQuant>,
    pub neuron_consecutive_fire_countdown: ConsecutiveFireCountdown<NMQ::NeuronConsecutiveFireCountdownQuant>,
    _p: PhantomData<(NMQ)>,
}

impl<NMQ> NeuronModelNeuronData<NMQ> for FeagiStandardModelNeuronData<NMQ>
where
    NMQ: FeagiStandardModelQuantization,
{
    
}

impl<NMQ> FeagiStandardModelNeuronData<NMQ>
where
    NMQ: FeagiStandardModelQuantization,
{
    pub fn new(
        neuron_fire_threshold: NeuronMembranePotential<<NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant>,
        neuron_leak_coefficient: LeakCoefficient<NMQ::NeuronLeakCoefficientQuant>,
        neuron_refractory_countdown: RefractoryCountdown<NMQ::NeuronRefractoryCountdownQuant>,
        neuron_consecutive_fire_countdown: ConsecutiveFireCountdown<NMQ::NeuronConsecutiveFireCountdownQuant>,
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
