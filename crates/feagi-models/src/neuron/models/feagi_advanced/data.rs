use crate::neuron::common_structs::model_and_quantization::NestedNeuronModelTypeAndQuantization;
use crate::neuron::models::feagi_advanced::quantization::{FeagiAdvancedModelQuantization, FeagiAdvancedModelQuantizationLevel};
use crate::neuron::models_shared_traits::data::{NeuronModelCorticalData, NeuronModelNeuronData};
use feagi_data::neurons::NeuronMembranePotential;
use feagi_data::values::quantizable::PercentageUnsigned;
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
#[derive(Debug, Clone)]
pub struct FeagiAdvancedModelCorticalData<CPQ>
where
    CPQ: FeagiAdvancedModelQuantization,
{
    pub excitability: PercentageUnsigned<CPQ::PercentageQuant>,

    pub refractory_period_limit: RefractoryPeriodLimit<CPQ::CorticalLimitAndSnoozeQuants>,

    /// Upper limit of fire threshold, over this and we wont fire
    pub fire_threshold_limit: NeuronMembranePotential<CPQ::MembranePotentialQuant>,

    pub consecutive_fire_limit: ConsecutiveFireLimit<CPQ::CorticalLimitAndSnoozeQuants>,

    pub snooze_period: SnoozePeriod<CPQ::CorticalLimitAndSnoozeQuants>,

    pub degeneracy_constant: DegeneracyConstant<CPQ::DegeneracyConstantQuant>,
}

impl<CPQ> NeuronModelCorticalData<CPQ> for FeagiAdvancedModelCorticalData<CPQ>
where
    CPQ: FeagiAdvancedModelQuantization,
{
    const LEVEL: NestedNeuronModelTypeAndQuantization = NestedNeuronModelTypeAndQuantization::FeagiAdvanced(FeagiAdvancedModelQuantizationLevel::Standard32bit);
}

impl<CPQ> FeagiAdvancedModelCorticalData<CPQ>
where
    CPQ: FeagiAdvancedModelQuantization,
{
    pub fn new(
        excitability: PercentageUnsigned<CPQ::PercentageQuant>,
        refractory_period_limit: RefractoryPeriodLimit<CPQ::CorticalLimitAndSnoozeQuants>,
        fire_threshold_limit: NeuronMembranePotential<CPQ::MembranePotentialQuant>,
        consecutive_fire_limit: ConsecutiveFireLimit<CPQ::CorticalLimitAndSnoozeQuants>,
        snooze_period: SnoozePeriod<CPQ::CorticalLimitAndSnoozeQuants>,
        degeneracy_constant: DegeneracyConstant<CPQ::DegeneracyConstantQuant>,
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

#[derive(Debug, Clone)]
pub struct FeagiAdvancedModelNeuronData<CPQ>
where
    CPQ: FeagiAdvancedModelQuantization,
{
    pub neuron_fire_threshold: NeuronMembranePotential<CPQ::MembranePotentialQuant>,
    pub neuron_leak_coefficient: LeakCoefficient<CPQ::DegeneracyConstantQuant>, // TODO is this correct quant?
    pub neuron_refractory_countdown: RefractoryCountdown<CPQ::NeuronCountdownQuants>,
    pub neuron_consecutive_fire_countdown: ConsecutiveFireCountdown<CPQ::NeuronCountdownQuants>,
}

impl<CPQ> NeuronModelNeuronData<CPQ> for FeagiAdvancedModelNeuronData<CPQ>
where
    CPQ: FeagiAdvancedModelQuantization,
{
    const LEVEL: NestedNeuronModelTypeAndQuantization = NestedNeuronModelTypeAndQuantization::FeagiAdvanced(FeagiAdvancedModelQuantizationLevel::Standard32bit);
}

impl<CPQ> FeagiAdvancedModelNeuronData<CPQ>
where
    CPQ: FeagiAdvancedModelQuantization,
{
    pub fn new(
        neuron_fire_threshold: NeuronMembranePotential<CPQ::MembranePotentialQuant>,
        neuron_leak_coefficient: LeakCoefficient<CPQ::DegeneracyConstantQuant>,
        neuron_refractory_countdown: RefractoryCountdown<CPQ::NeuronCountdownQuants>,
        neuron_consecutive_fire_countdown: ConsecutiveFireCountdown<CPQ::NeuronCountdownQuants>,
    ) -> Self {
        Self {
            neuron_fire_threshold,
            neuron_leak_coefficient,
            neuron_refractory_countdown,
            neuron_consecutive_fire_countdown,
        }
    }
}
