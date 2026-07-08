use core::marker::PhantomData;
use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiGlobalQuantization;
use feagi_data::values::quantizable::{QuantizedDecimalTrait, QuantizedElementBase};
use feagi_npu_common::descriptors::cortical_area_descriptors::CorticalAreaLayoutDataDimensional;
use feagi_npu_common::wrapped_indexes::{BurstIndex, NeuronCorticalIndex, NeuronMembranePotential};
use crate::neuron_models::feagi_standard::data::{ConsecutiveFireLimit, FeagiStandardModelCorticalData, FeagiStandardModelNeuronData};
use crate::neuron_models::feagi_standard::quantization::FeagiStandardModelQuantization;
use crate::neuron_models::neuron_model_traits::neuron_model_processor::{NeuronModelProcessorBase, NeuronModelProcessorWithHistory};

pub struct FeagiStandardModelProcessor<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization, // fsm quant impl
{
    // No actual members
    _p: PhantomData<(FGQ, NMQ)>,
}

impl<FGQ, NMQ> NeuronModelProcessorBase<FGQ, NMQ, FeagiStandardModelCorticalData<NMQ>, FeagiStandardModelNeuronData<NMQ>> for FeagiStandardModelProcessor<FGQ, NMQ> where FGQ: FeagiGlobalQuantization, NMQ: FeagiStandardModelQuantization, {}

impl<FGQ, NMQ> NeuronModelProcessorWithHistory<
    FGQ,
    NMQ,
    FeagiStandardModelCorticalData<NMQ>,
    FeagiStandardModelNeuronData<NMQ>,
> for FeagiStandardModelProcessor<FGQ, NMQ>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: FeagiStandardModelQuantization,
{
    fn process_neuron_potential_for_dimensional_cortical_configuration(
        incoming_potential: &NeuronMembranePotential<<NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant>,
        neuron_linear_index: &NeuronCorticalIndex<FGQ::NeuronIndexCountQuant>,
        burst_index: &BurstIndex<FGQ::GlobalBurstIndexQuant>,
        burst_index_of_last_activity: &BurstIndex<FGQ::GlobalBurstIndexQuant>,
        burst_index_of_last_firing: &BurstIndex<FGQ::GlobalBurstIndexQuant>,
        cortical_layout_dimensional: &CorticalAreaLayoutDataDimensional<FGQ>,
        cortical_area_data: &FeagiStandardModelCorticalData<NMQ>,
        neuron_model_data: &mut FeagiStandardModelNeuronData<NMQ>,
        this_neuron_potential: &mut NeuronMembranePotential<<NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant>
    ) -> bool {

        let prev_membrane_potential = *this_neuron_potential;

        // update neuron potential
        *this_neuron_potential = prev_membrane_potential + *incoming_potential;// - QuantizedDecimalTrait::QUANT_ZERO )); // TODO right now subtracting 0, but this is the resting potential


        /// If consecutive fire is disabled, its set to 0
        if cortical_area_data.consecutive_fire_limit == ConsecutiveFireLimit::from(NMQ::CorticalConsecutiveFireLimit::QUANT_ZERO) {
            if *this_neuron_potential > neuron_model_data.neuron_fire_threshold
            {
                return true;
            }

        }

        return false;
    }
}

