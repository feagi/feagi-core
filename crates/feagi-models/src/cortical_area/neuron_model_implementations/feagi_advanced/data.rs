use feagi_data::{create_wrapped_quantized_decimal, create_wrapped_quantized_index};
use feagi_data::neurons::NeuronMembranePotential;
use feagi_data::values::quantizable::PercentageUnsigned;
use crate::cortical_area::neuron_model_implementations::feagi_advanced::quantization::FeagiAdvancedModelQuantization;
use crate::cortical_area::neuron::cortical_data::NeuronModelCorticalData;
use crate::cortical_area::neuron::neuron_data::NeuronModelNeuronData;

//create_wrapped_quantized_decimal!(pub Excitation);
create_wrapped_quantized_index!(pub RefractoryPeriodLimit);
// Fire threshold is just membrane potential
create_wrapped_quantized_index!(pub ConsecutiveFireLimit);
create_wrapped_quantized_index!(pub SnoozePeriod);
create_wrapped_quantized_decimal!(pub DegeneracyConstant);

create_wrapped_quantized_decimal!(pub LeakCoefficient);
create_wrapped_quantized_index!(pub RefractoryCountdown);
create_wrapped_quantized_index!(pub ConsecutiveFireCountdown);

#[derive(Debug, Clone, Copy, Default)]
pub struct FeagiAdvancedModelCorticalData<NMQ>
where
    NMQ: FeagiAdvancedModelQuantization,
{
    pub excitability: PercentageUnsigned<NMQ::PercentageQuant>,

    pub refractory_period_limit: RefractoryPeriodLimit<NMQ::CorticalLimitAndSnoozeQuants>,

    /// Upper limit of fire threshold, over this and we wont fire
    pub fire_threshold_limit: NeuronMembranePotential<NMQ::MembranePotentialQuant>,

    pub consecutive_fire_limit: ConsecutiveFireLimit<NMQ::CorticalLimitAndSnoozeQuants>,

    pub snooze_period: SnoozePeriod<NMQ::CorticalLimitAndSnoozeQuants>,

    pub degeneracy_constant: DegeneracyConstant<NMQ::DegeneracyConstantQuant>,
}

impl<NMQ> NeuronModelCorticalData<NMQ> for FeagiAdvancedModelCorticalData<NMQ> where NMQ: FeagiAdvancedModelQuantization {}

impl<NMQ> FeagiAdvancedModelCorticalData<NMQ>
where
    NMQ: FeagiAdvancedModelQuantization,
{
    pub fn new(
        excitability: PercentageUnsigned<NMQ::PercentageQuant>,
        refractory_period_limit: RefractoryPeriodLimit<NMQ::CorticalLimitAndSnoozeQuants>,
        fire_threshold_limit: NeuronMembranePotential<NMQ::MembranePotentialQuant>,
        consecutive_fire_limit: ConsecutiveFireLimit<NMQ::CorticalLimitAndSnoozeQuants>,
        snooze_period: SnoozePeriod<NMQ::CorticalLimitAndSnoozeQuants>,
        degeneracy_constant: DegeneracyConstant<NMQ::DegeneracyConstantQuant>,
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

#[derive(Debug, Clone, Copy, Default)]
pub struct FeagiAdvancedModelNeuronData<NMQ>
where
    NMQ: FeagiAdvancedModelQuantization,
{
    pub neuron_fire_threshold: NeuronMembranePotential<NMQ::MembranePotentialQuant>,
    pub neuron_leak_coefficient: LeakCoefficient<NMQ::DegeneracyConstantQuant>, // TODO is this correct quant?
    pub neuron_refractory_countdown: RefractoryCountdown<NMQ::NeuronCountdownQuants>,
    pub neuron_consecutive_fire_countdown: ConsecutiveFireCountdown<NMQ::NeuronCountdownQuants>,
}

impl<NMQ> NeuronModelNeuronData<NMQ> for FeagiAdvancedModelNeuronData<NMQ> where NMQ: FeagiAdvancedModelQuantization {}

impl<NMQ> FeagiAdvancedModelNeuronData<NMQ>
where
    NMQ: FeagiAdvancedModelQuantization,
{
    pub fn new(
        neuron_fire_threshold: NeuronMembranePotential<NMQ::MembranePotentialQuant>,
        neuron_leak_coefficient: LeakCoefficient<NMQ::DegeneracyConstantQuant>,
        neuron_refractory_countdown: RefractoryCountdown<NMQ::NeuronCountdownQuants>,
        neuron_consecutive_fire_countdown: ConsecutiveFireCountdown<NMQ::NeuronCountdownQuants>,
    ) -> Self {
        Self {
            neuron_fire_threshold,
            neuron_leak_coefficient,
            neuron_refractory_countdown,
            neuron_consecutive_fire_countdown,
        }
    }
}
