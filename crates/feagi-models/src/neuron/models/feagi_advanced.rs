use half::bf16;
use feagi_data::{create_wrapped_quantized_decimal, create_wrapped_quantized_index};
use feagi_data::neurons::{DimensionalCorticalArea4DDimensions, NeuronCorticalLocalIndex, NeuronMembranePotential};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use feagi_data::values::quantizable::{DecimalQuantizationLevel, PercentageUnsigned, QuantizedDecimalTrait, QuantizedIndexCountTrait};
use crate::wrapped_indexes::BurstIndex;
use crate::neuron::model_and_quantization::{NestedNeuronModelTypeAndQuantization, NeuronModelType};
use crate::neuron::model_extensions::neuron_burst_index_rollover_handling::NeuronModelNoSpecialBurstIndexRolloverHandling;
use crate::neuron::model_extensions::neuron_history::NeuronModelFullNeuronHistory;
use crate::neuron::model_extensions::neuron_layout_implementations::DimensionalNeuronModel;
use crate::neuron::neuron_model::NeuronModel;
use crate::neuron::neuron_model_data::{NeuronModelCorticalData, NeuronModelNeuronData};
use crate::neuron::neuron_model_quantization::{NeuronModelQuantization, NeuronModelQuantizationLevel};

// TODO a lot of this is honestly proc macro work
//region Quantization

pub trait FeagiAdvancedModelQuantization: NeuronModelQuantization {
    const MODEL_QUANTIZATION_LEVEL: FeagiAdvancedModelQuantizationLevel;

    type NeuronCountdownQuants: QuantizedIndexCountTrait;
    type CorticalLimitAndSnoozeQuants: QuantizedIndexCountTrait;
    type PercentageQuant: QuantizedDecimalTrait;
    type DegeneracyConstantQuant: QuantizedDecimalTrait;
}

//region Discrete Levels

/// The default quantization level for Feagi Advanced
#[derive(Default, Clone, Copy)]
pub struct FeagiAdvancedModelStandardQuant;

impl FeagiAdvancedModelQuantization for FeagiAdvancedModelStandardQuant {
    const MODEL_QUANTIZATION_LEVEL: FeagiAdvancedModelQuantizationLevel = FeagiAdvancedModelQuantizationLevel::Standard;

    type NeuronCountdownQuants = u16;
    type CorticalLimitAndSnoozeQuants = u16;
    type PercentageQuant = bf16;
    type DegeneracyConstantQuant = f32;
}

impl NeuronModelQuantization for FeagiAdvancedModelStandardQuant {
    const NEURON_MODEL: NeuronModelType = NeuronModelType::FeagiAdvanced;
    type QuantLevelType = FeagiAdvancedModelQuantizationLevel;
    const NEURON_QUANTIZATION: Self::QuantLevelType = FeagiAdvancedModelQuantizationLevel::Standard;
    const NESTED_NEURON_MODEL_AND_QUANTIZATION: NestedNeuronModelTypeAndQuantization = NestedNeuronModelTypeAndQuantization::FeagiAdvanced(Self::NEURON_QUANTIZATION) ;
    const USED_DECIMAL_QUANTIZATION_LEVELS: &'static [DecimalQuantizationLevel] = &[DecimalQuantizationLevel::BF16, DecimalQuantizationLevel::F32];
}

impl MembranePotentialQuantization for FeagiAdvancedModelStandardQuant {
    type MembranePotentialQuant = f32;
}

//endregion

// TODO macro for implementing NeuronModelQuantizationLevel on FeagiAdvancedModelQuantizationLevel

#[repr(u8)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub enum FeagiAdvancedModelQuantizationLevel {
    #[default]
    Standard = 0,
}

impl NeuronModelQuantizationLevel for FeagiAdvancedModelQuantizationLevel
{
    fn get_membrane_potential_level(&self) -> DecimalQuantizationLevel {
        match self {
            FeagiAdvancedModelQuantizationLevel::Standard => DecimalQuantizationLevel::F32
        }
    }

    // TODO copy some properties here
}



//endregion

//region Data

//create_wrapped_quantized_decimal!(pub Excitation);
create_wrapped_quantized_index!(pub RefractoryPeriodLimit);
// Fire threshold is just membrane potential
create_wrapped_quantized_index!(pub ConsecutiveFireLimit);
create_wrapped_quantized_index!(pub SnoozePeriod);
create_wrapped_quantized_decimal!(pub DegeneracyConstant);

create_wrapped_quantized_decimal!(pub LeakCoefficient);
create_wrapped_quantized_index!(pub RefractoryCountdown);
create_wrapped_quantized_index!(pub ConsecutiveFireCountdown);


#[derive(Debug, Clone, Default)]
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


impl<NMQ> NeuronModelCorticalData<NMQ> for FeagiAdvancedModelCorticalData<NMQ>
where
    NMQ: FeagiAdvancedModelQuantization,
{}

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


#[derive(Debug, Clone, Default)]
pub struct FeagiAdvancedModelNeuronData<NMQ>
where
    NMQ: FeagiAdvancedModelQuantization,
{
    pub neuron_fire_threshold: NeuronMembranePotential<NMQ::MembranePotentialQuant>,
    pub neuron_leak_coefficient: LeakCoefficient<NMQ::DegeneracyConstantQuant>, // TODO is this correct quant?
    pub neuron_refractory_countdown: RefractoryCountdown<NMQ::NeuronCountdownQuants>,
    pub neuron_consecutive_fire_countdown: ConsecutiveFireCountdown<NMQ::NeuronCountdownQuants>,
}

impl<NMQ> NeuronModelNeuronData<NMQ> for FeagiAdvancedModelNeuronData<NMQ>
where
    NMQ: FeagiAdvancedModelQuantization,
{}

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

//endregion

//region Model
pub struct FeagiAdvancedModel<FIQ, NMQ>
where
    FIQ: FeagiIndexQuantization,
    NMQ: FeagiAdvancedModelQuantization, // fsm quant impl
{
    // No actual members
    _p: core::marker::PhantomData<(FIQ, NMQ)>,
}

impl<FIQ, NMQ> NeuronModel<FIQ, NMQ> for FeagiAdvancedModel<FIQ, NMQ>
where
    FIQ: FeagiIndexQuantization,
    NMQ: FeagiAdvancedModelQuantization,
{
    type CorticalData = FeagiAdvancedModelCorticalData<NMQ>;
    type NeuronData = FeagiAdvancedModelNeuronData<NMQ>;
    type NeuronHistoryType = NeuronModelFullNeuronHistory<FIQ>;
    type BurstIndexRolloverHandling = NeuronModelNoSpecialBurstIndexRolloverHandling;
}

// Support Dimensional Cortical Areas

impl<FIQ, NMQ> DimensionalNeuronModel<FIQ, NMQ> for FeagiAdvancedModel<FIQ, NMQ>
where
    FIQ: FeagiIndexQuantization,
    NMQ: FeagiAdvancedModelQuantization,
{
    fn process_incoming_potential_for_dimensional_area(incoming_potential: &NeuronMembranePotential<NMQ::MembranePotentialQuant>, neuron_linear_index: &NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>, burst_index: &BurstIndex<FIQ::GlobalBurstIndexQuant>, dimensional_cortical_dimensions: &DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>, neuron_history: &Self::NeuronHistoryType, cortical_area_data: &Self::CorticalData, neuron_model_data: &mut Self::NeuronData, this_neuron_potential: &mut NeuronMembranePotential<NMQ::MembranePotentialQuant>) -> bool {
        *this_neuron_potential += *incoming_potential; // - QuantizedDecimalTrait::QUANT_ZERO )); // TODO right now subtracting 0, but this is the resting potential
        false
    }

    fn default_dimensional_area_cortical_data() -> Self::CorticalData {
        Self::CorticalData::default()
    }

    //fn default_dimensional_area_spawner() -> impl DimensionalCorticalAreaSpawner {
    //    todo!()
    //}
}


//endregion