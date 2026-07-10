use crate::neuron_models::feagi_standard::quantization::FeagiStandardModelQuantization;
use crate::neuron_models::neuron_model_traits::neuron_model_data::{
    NeuronModelCorticalData, NeuronModelNeuronData,
};
use feagi_data::{create_wrapped_quantized_decimal, create_wrapped_quantized_index};
// TODO percentages should be their own types

//create_wrapped_quantized_decimal!(pub Excitation);
create_wrapped_quantized_index!(pub RefractoryPeriodLimit);
// Fire threshold is just membrane potential
create_wrapped_quantized_index!(pub ConsecutiveFireLimit);
create_wrapped_quantized_index!(pub SnoozePeriod);
create_wrapped_quantized_decimal!(pub DegeneracyConstant);

create_wrapped_quantized_decimal!(pub LeakCoefficient);
create_wrapped_quantized_index!(pub RefractoryCountdown);
create_wrapped_quantized_index!(pub ConsecutiveFireCountdown);
#[derive(Debug, Copy, Clone)]
pub struct FeagiStandardModelCorticalData<CPQ>
where
    CPQ: FeagiStandardModelQuantization,
{
    pub excitability: CPQ::PercentageQuant,

    pub refractory_period_limit: CPQ::CorticalLimitAndSnoozeQuants,

    /// Upper limit of fire threshold, over this and we wont fire
    pub fire_threshold_limit: CPQ::MembranePotentialQuant,

    pub consecutive_fire_limit: CPQ::CorticalLimitAndSnoozeQuants,

    pub snooze_period: CPQ::CorticalLimitAndSnoozeQuants,

    pub degeneracy_constant: CPQ::DegeneracyConstantQuant,
}

impl<CPQ> NeuronModelCorticalData<CPQ> for FeagiStandardModelCorticalData<CPQ>
where
    CPQ: FeagiStandardModelQuantization,
{
    const MODEL_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER: bool = true;
    const MODEL_SUPPORTS_CORTICAL_LAYOUT_DIMENSIONAL: bool = true;
}

impl<CPQ> FeagiStandardModelCorticalData<CPQ>
where
    CPQ: FeagiStandardModelQuantization,
{
    pub fn new(
        excitability: CPQ::PercentageQuant,
        refractory_period_limit: CPQ::CorticalLimitAndSnoozeQuants,
        fire_threshold_limit: CPQ::MembranePotentialQuant,
        consecutive_fire_limit: CPQ::CorticalLimitAndSnoozeQuants,
        snooze_period: CPQ::CorticalLimitAndSnoozeQuants,
        degeneracy_constant: CPQ::DegeneracyConstantQuant,
    ) -> Self {
        Self {
            excitability,
            refractory_period_limit,
            fire_threshold_limit,
            consecutive_fire_limit,
            snooze_period,
            degeneracy_constant,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct FeagiStandardModelNeuronData<CPQ>
where
    CPQ: FeagiStandardModelQuantization,
{
    // TODO this is arguable a cortical level property except that fire threshold increment may be used. This is a prime example os something to move to an advanced model
    pub neuron_fire_threshold: CPQ::MembranePotentialQuant,
    pub neuron_leak_coefficient: CPQ::CorticalLimitAndSnoozeQuants,
    pub neuron_refractory_countdown: CPQ::NeuronCountdownQuants,
    pub neuron_consecutive_fire_countdown: CPQ::NeuronCountdownQuants,
}

impl<CPQ> NeuronModelNeuronData<CPQ> for FeagiStandardModelNeuronData<CPQ> where
    CPQ: FeagiStandardModelQuantization
{
}

impl<CPQ> FeagiStandardModelNeuronData<CPQ>
where
    CPQ: FeagiStandardModelQuantization,
{
    pub fn new(
        neuron_fire_threshold: CPQ::MembranePotentialQuant,
        neuron_leak_coefficient: CPQ::CorticalLimitAndSnoozeQuants,
        neuron_refractory_countdown: CPQ::NeuronCountdownQuants,
        neuron_consecutive_fire_countdown: CPQ::NeuronCountdownQuants,
    ) -> Self {
        Self {
            neuron_fire_threshold,
            neuron_leak_coefficient,
            neuron_refractory_countdown,
            neuron_consecutive_fire_countdown,
        }
    }
}
