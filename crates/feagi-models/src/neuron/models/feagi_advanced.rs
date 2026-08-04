use std::marker::PhantomData;
use crate::neuron::model_capabilities::neuron_burst_index_rollover_handling::NeuronModelNoSpecialBurstIndexRolloverHandling;
use crate::neuron::model_capabilities::neuron_history::NeuronModelFullNeuronHistory;
use crate::neuron::model_capabilities::neuron_layout_implementations::DimensionalNeuronModel;
use crate::neuron::neuron_model::NeuronModel;
use crate::neuron::neuron_model_data::{NeuronModelCorticalData, NeuronModelNeuronData};
use crate::neuron::neuron_model_quantization::{NeuronModelQuantization, NeuronModelQuantizationLevel};
use crate::wrapped_indexes::BurstIndex;
use feagi_data::neurons::{DimensionalCorticalArea4DDimensions, NeuronCorticalLocalIndex, NeuronMembranePotential};
use feagi_data::quantization_levels::feagi_index_quantization::{FeagiIndexQuantization, FeagiIndexQuantizationGenomic};
use feagi_data::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use feagi_data::values::quantizable::{DecimalQuantizationLevel, PercentageUnsigned, QuantizedDecimalTrait, QuantizedIndexCountTrait, WrappedQuantizedIndexCount};
use feagi_data::{create_wrapped_quantized_decimal, create_wrapped_quantized_index};
use half::bf16;
use crate::neuron::cortical_area_layout::CorticalAreaLayoutDimensional;
use crate::neuron::cortical_writer::NeuronModelCorticalWriter;
use crate::neuron::model_generated::cortical_layout::CorticalAreaLayoutNested;
use crate::neuron::model_generated::model_type_and_quantization::{NeuronModelType, NeuronModelTypeAndQuantizationNested};
use crate::neuron::properties::{CorticalAreaProperties, NeuronProperties};
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
    const NESTED_NEURON_MODEL_AND_QUANTIZATION: NeuronModelTypeAndQuantizationNested =
        NeuronModelTypeAndQuantizationNested::FeagiAdvanced(Self::NEURON_QUANTIZATION);
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

impl NeuronModelQuantizationLevel for FeagiAdvancedModelQuantizationLevel {
    fn get_membrane_potential_level(&self) -> DecimalQuantizationLevel {
        match self {
            FeagiAdvancedModelQuantizationLevel::Standard => DecimalQuantizationLevel::F32,
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

//endregion

//region Cortical Writer

#[derive(Debug, Clone, Copy)]
pub enum FeagiAdvancedModelCorticalWriter<NMQ>
where
    NMQ: FeagiAdvancedModelQuantization,
{
    DefaultNewDimensional {dimensions: DimensionalCorticalArea4DDimensions<<FeagiIndexQuantizationGenomic as FeagiIndexQuantization>::NeuronIndexQuant>, _p: PhantomData<NMQ>},
}

impl<NMQ> NeuronModelCorticalWriter<NMQ, FeagiAdvancedModelCorticalData<NMQ>, FeagiAdvancedModelNeuronData<NMQ>> for FeagiAdvancedModelCorticalWriter<NMQ>
where
    NMQ: FeagiAdvancedModelQuantization,
{
    fn number_neurons_needed<FIQ: FeagiIndexQuantization>(&self) -> Result<FIQ::NeuronIndexQuant, ()> {
        match self {
            FeagiAdvancedModelCorticalWriter::DefaultNewDimensional { dimensions, _p: _ } => {
                let u = dimensions.number_contained_elements();
                let r: FIQ::NeuronIndexQuant = u.try_to_quantization().unwrap(); // TODO error handling!
                Ok(r)
            }
        }
    }

    fn write_to_cortical_area<FIQ: FeagiIndexQuantization>(self, cortical_data: &mut FeagiAdvancedModelCorticalData<NMQ>, neuron_data: &mut [FeagiAdvancedModelNeuronData<NMQ>]) -> Result<(CorticalAreaLayoutNested<FeagiIndexQuantizationGenomic>, CorticalAreaProperties, impl Iterator<Item=NeuronProperties>), ()> {

        match self {
            FeagiAdvancedModelCorticalWriter::DefaultNewDimensional { dimensions, _p } => {

                // TODO check dimensions

                // Uniform
                let new_cortical: FeagiAdvancedModelCorticalData<NMQ> = FeagiAdvancedModelCorticalData {
                    excitability: PercentageUnsigned::ZERO_PERCENT,
                    refractory_period_limit: RefractoryPeriodLimit::QUANT_ONE,
                    fire_threshold_limit: NeuronMembranePotential::QUANT_ONE,
                    consecutive_fire_limit: ConsecutiveFireLimit::QUANT_ONE,
                    snooze_period: SnoozePeriod::QUANT_ONE,
                    degeneracy_constant: DegeneracyConstant::QUANT_ONE,
                };

                let new_cortical_properties = CorticalAreaProperties {
                    non_mp_psp: 0.0,
                    probe_cortical_area_input_disabled: false,
                    probe_cortical_area_output_disabled: false,
                    is_psp_uniform: false,
                    is_psp_mp_driven: false,
                };

                let new_uniform_neuron: FeagiAdvancedModelNeuronData<NMQ> = FeagiAdvancedModelNeuronData {
                    neuron_fire_threshold: NeuronMembranePotential::QUANT_ONE,
                    neuron_leak_coefficient: LeakCoefficient::QUANT_ONE,
                    neuron_refractory_countdown: RefractoryCountdown::QUANT_ONE,
                    neuron_consecutive_fire_countdown: ConsecutiveFireCountdown::QUANT_ONE,
                };

                let new_uniform_neuron_properties = NeuronProperties {
                    probe_force_disabled: false,
                    probe_force_firing: false,
                };

                let dimensions: DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexQuant> = dimensions.try_to_quantization().unwrap(); // TODO ERROR CHECKING
                let number_neurons = dimensions.number_contained_elements().quant_to_usize();
                let layout = CorticalAreaLayoutNested::Dimensional(CorticalAreaLayoutDimensional{dimensions});

                *cortical_data = new_cortical;
                neuron_data.fill(new_uniform_neuron);
                Ok((layout, new_cortical_properties, core::iter::repeat(new_uniform_neuron_properties).take(number_neurons)))
            }
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
    fn process_incoming_potential_for_dimensional_area(
        incoming_potential: &NeuronMembranePotential<NMQ::MembranePotentialQuant>,
        neuron_linear_index: &NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
        burst_index: &BurstIndex<FIQ::GlobalBurstIndexQuant>,
        dimensional_cortical_dimensions: &DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexQuant>,
        neuron_history: &Self::NeuronHistoryType,
        cortical_area_data: &Self::CorticalData,
        neuron_model_data: &mut Self::NeuronData,
        this_neuron_potential: &mut NeuronMembranePotential<NMQ::MembranePotentialQuant>,
    ) -> bool {
        *this_neuron_potential += *incoming_potential; // - QuantizedDecimalTrait::QUANT_ZERO )); // TODO right now subtracting 0, but this is the resting potential
        false
    }
    
}

//endregion
