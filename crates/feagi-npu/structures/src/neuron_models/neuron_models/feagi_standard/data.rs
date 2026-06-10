use std::marker::PhantomData;
use feagi_structures::feagi_data::feagi_pdi::PDIElement;
use feagi_structures::feagi_data::feagi_pdi::tag_device::{PDITagCPU, PDITagGenericDevice};
use feagi_structures::feagi_data::quantizable_linear::base_types::{QuantizedDecimalTrait, QuantizedElementBase};
use feagi_structures::feagi_data::shared_quantization_sets::{CorticalPotentialQuantization, FeagiGlobalQuantization};
use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::data_structures::tables::cortical_structure_configuration::cortical_configuration::CorticalConfigurationBase;
use crate::neuron_models::base_traits_all_devices::{NeuronModelCorticalData, NeuronModelNeuronData};
use crate::neuron_models::base_traits_cpu::{NeuronModelCorticalDataCPU, NeuronModelNeuronDataCPU};
use crate::neuron_models::neuron_models::feagi_standard::quantization::FeagiStandardModelQuantization;
use crate::npu_descriptors::NPUGlobalBurstCounter;
use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::data_structures::cpu_wrappers::cortical_spatial::NPUNeuronIndexCorticalLocal;

#[repr(C)]
pub struct FeagiStandardModelCorticalDataCPU<FGQ, NMQ, CCB>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
    CCB: CorticalConfigurationBase<FGQ, NMQ::CorticalPotentialQuant>
{
    pub excitability: NMQ::CorticalExcitabilityQuant,
    pub refractory_period_limit: NMQ::CorticalRefractoryPeriodLimitQuant,
    pub fire_threshold_limit: NMQ::CorticalFireThresholdLimit,
    pub consecutive_fire_limit: NMQ::CorticalConsecutiveFireLimit,
    _p: PhantomData<(FGQ, NMQ, CCB)>,
}

//region Tags

impl<FGQ, NMQ, CCB> PDITagGenericDevice for FeagiStandardModelCorticalDataCPU<FGQ, NMQ, CCB>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
    CCB: CorticalConfigurationBase<FGQ, NMQ::CorticalPotentialQuant>,
{
}

impl<FGQ, NMQ, CCB> PDITagCPU for FeagiStandardModelCorticalDataCPU<FGQ, NMQ, CCB>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
    CCB: CorticalConfigurationBase<FGQ, NMQ::CorticalPotentialQuant>,
{
}

impl<FGQ, NMQ, CCB> PDIElement for FeagiStandardModelCorticalDataCPU<FGQ, NMQ, CCB>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
    CCB: CorticalConfigurationBase<FGQ, NMQ::CorticalPotentialQuant>,
{
}

//endregion

impl<FGQ, NMQ, CCB> NeuronModelCorticalData<FGQ, NMQ, CCB> for FeagiStandardModelCorticalDataCPU<FGQ, NMQ, CCB>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
    CCB: CorticalConfigurationBase<FGQ, NMQ::CorticalPotentialQuant>,
{
}

impl<FGQ, NMQ, CCB> NeuronModelCorticalDataCPU<FGQ, NMQ, CCB> for FeagiStandardModelCorticalDataCPU<FGQ, NMQ, CCB>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
    CCB: CorticalConfigurationBase<FGQ, NMQ::CorticalPotentialQuant>,
{
}

impl<FGQ, NMQ, CCB> FeagiStandardModelCorticalDataCPU<FGQ, NMQ, CCB>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
    CCB: CorticalConfigurationBase<FGQ, NMQ::CorticalPotentialQuant>,
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

// TODO needs padding!
#[repr(C)]
pub struct FeagiStandardModelNeuronDataCPU<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
{
    pub neuron_fire_threshold: <NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant,
    pub neuron_leak_coefficient: NMQ::NeuronLeakCoefficientQuant,
    pub neuron_refractory_countdown: NMQ::NeuronRefractoryCountdownQuant,
    pub neuron_consecutive_fire_countdown: NMQ::NeuronConsecutiveFireCountdownQuant,
    pub neuron_burst_index_of_last_fire: FGQ::GlobalBurstIndexQuant,
    _p: PhantomData<(FGQ, NMQ)>,
}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> PDITagGenericDevice for FeagiStandardModelNeuronDataCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> PDITagCPU for FeagiStandardModelNeuronDataCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> PDIElement for FeagiStandardModelNeuronDataCPU<FGQ, NMQ> {}

impl<FGQ, NMQ, CCB> NeuronModelNeuronData<
    FGQ,
    NMQ,
    CCB,
    FeagiStandardModelCorticalDataCPU<FGQ, NMQ, CCB>,
> for FeagiStandardModelNeuronDataCPU<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
    CCB: CorticalConfigurationBase<FGQ, NMQ::CorticalPotentialQuant>,
{
}

impl<FGQ, NMQ, CCB> NeuronModelNeuronDataCPU<
    FGQ,
    NMQ,
    CCB,
    FeagiStandardModelCorticalDataCPU<FGQ, NMQ, CCB>,
> for FeagiStandardModelNeuronDataCPU<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
    CCB: CorticalConfigurationBase<FGQ, NMQ::CorticalPotentialQuant>,
{
    fn create_blank_neuron(
        _neuron_linear_index: &NPUNeuronIndexCorticalLocal<FGQ::NeuronIndexCountQuant>,
        burst_index: &NPUGlobalBurstCounter<FGQ::GlobalBurstIndexQuant>,
        _cortical_area_configuration: &CCB,
        cortical_area_data: &FeagiStandardModelCorticalDataCPU<FGQ, NMQ, CCB>,
    ) -> Self {
        Self::new(
            burst_index.const_unwrap(),
            <<NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant as QuantizedDecimalTrait>::from_f32(
                cortical_area_data.fire_threshold_limit.to_f32(),
            ),
            <NMQ::NeuronLeakCoefficientQuant as QuantizedElementBase>::QUANT_ZERO,
            <NMQ::NeuronRefractoryCountdownQuant as QuantizedElementBase>::QUANT_ZERO,
            <NMQ::NeuronConsecutiveFireCountdownQuant as QuantizedElementBase>::QUANT_ZERO,
        )
    }
}

impl<FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization> FeagiStandardModelNeuronDataCPU<FGQ, NMQ> {
    pub fn new(
        neuron_burst_index_of_last_fire: FGQ::GlobalBurstIndexQuant,
        neuron_fire_threshold: <NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant,
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