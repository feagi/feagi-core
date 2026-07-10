use crate::neuron_models::feagi_standard::data::{
    FeagiStandardModelCorticalData, FeagiStandardModelNeuronData,
};
use crate::neuron_models::feagi_standard::quantization::FeagiStandardModelQuantization;
use crate::neuron_models::neuron_model_traits::neuron_model_processor::{
    NeuronModelProcessorBase, NeuronModelProcessorWithHistory,
};
use core::marker::PhantomData;
use feagi_data::neurons::{NeuronCorticalLocalIndex, NeuronMembranePotential};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::QuantizedElementBase;
use feagi_npu_common::descriptors::cortical_area_descriptors::CorticalAreaLayoutDataDimensional;
use feagi_npu_common::wrapped_indexes::BurstIndex;

pub struct FeagiStandardModelProcessor<FIQ, CPQ>
where
    FIQ: FeagiIndexQuantization,
    CPQ: FeagiStandardModelQuantization, // fsm quant impl
{
    // No actual members
    _p: PhantomData<(FIQ, CPQ)>,
}

impl<FIQ, CPQ>
    NeuronModelProcessorBase<
        FIQ,
        CPQ,
        FeagiStandardModelCorticalData<CPQ>,
        FeagiStandardModelNeuronData<CPQ>,
    > for FeagiStandardModelProcessor<FIQ, CPQ>
where
    FIQ: FeagiIndexQuantization,
    CPQ: FeagiStandardModelQuantization,
{
}

impl<FIQ, CPQ>
    NeuronModelProcessorWithHistory<
        FIQ,
        CPQ,
        FeagiStandardModelCorticalData<CPQ>,
        FeagiStandardModelNeuronData<CPQ>,
    > for FeagiStandardModelProcessor<FIQ, CPQ>
where
    FIQ: FeagiIndexQuantization,
    CPQ: FeagiStandardModelQuantization,
{
    fn process_neuron_potential_for_dimensional_cortical_configuration(
        incoming_potential: &NeuronMembranePotential<CPQ::MembranePotentialQuant>,
        neuron_linear_index: &NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
        burst_index: &BurstIndex<FIQ::GlobalBurstIndexQuant>,
        burst_index_of_last_activity: &BurstIndex<FIQ::GlobalBurstIndexQuant>,
        burst_index_of_last_firing: &BurstIndex<FIQ::GlobalBurstIndexQuant>,
        cortical_layout_dimensional: &CorticalAreaLayoutDataDimensional<FIQ>,
        cortical_area_data: &FeagiStandardModelCorticalData<CPQ>,
        neuron_model_data: &mut FeagiStandardModelNeuronData<CPQ>,
        this_neuron_potential: &mut NeuronMembranePotential<CPQ::MembranePotentialQuant>,
    ) -> bool {
        // update neuron potential
        *this_neuron_potential += *incoming_potential; // - QuantizedDecimalTrait::QUANT_ZERO )); // TODO right now subtracting 0, but this is the resting potential

        // If consecutive fire is disabled, it is set to 0
        if cortical_area_data.consecutive_fire_limit
            == CPQ::CorticalLimitAndSnoozeQuants::QUANT_ZERO
        {
            if *this_neuron_potential.as_ref() > neuron_model_data.neuron_fire_threshold {
                return true;
            }
        }

        false
    }
}
