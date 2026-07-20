use crate::burst_index::BurstIndex;
use crate::neuron::neuron_history::NeuronHistoryFull;
use crate::neuron::models::feagi_advanced::data::{FeagiAdvancedModelCorticalData, FeagiAdvancedModelNeuronData};
use crate::neuron::models::feagi_advanced::quantization::FeagiAdvancedModelQuantization;
use crate::neuron::models_shared::model::NeuronModel;
use core::marker::PhantomData;
use feagi_data::neurons::{DimensionalCorticalArea4DDimensions, NeuronCorticalLocalIndex, NeuronMembranePotential};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::QuantizedElementBase;

pub struct FeagiAdvancedModelProcessor<FIQ, CPQ>
where
    FIQ: FeagiIndexQuantization,
    CPQ: FeagiAdvancedModelQuantization, // fsm quant impl
{
    // No actual members
    _p: PhantomData<(FIQ, CPQ)>,
}

impl<FIQ, CPQ> NeuronModel<FIQ, CPQ, FeagiAdvancedModelCorticalData<CPQ>, FeagiAdvancedModelNeuronData<CPQ>>
    for FeagiAdvancedModelProcessor<FIQ, CPQ>
where
    FIQ: FeagiIndexQuantization,
    CPQ: FeagiAdvancedModelQuantization,
{
    type UsedNeuronHistory = NeuronHistoryFull<FIQ>;

    fn process_neuron_potential_for_dimensional_layout_cortical_area(
        incoming_potential: &NeuronMembranePotential<CPQ::MembranePotentialQuant>,
        neuron_linear_index: &NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
        burst_index: &BurstIndex<FIQ::GlobalBurstIndexQuant>,
        dimensional_cortical_dimensions: DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
        neuron_history: &Self::UsedNeuronHistory,
        cortical_area_data: &FeagiAdvancedModelCorticalData<CPQ>,
        neuron_model_data: &mut FeagiAdvancedModelNeuronData<CPQ>,
        this_neuron_potential: &mut NeuronMembranePotential<CPQ::MembranePotentialQuant>,
    ) -> bool {
        // update neuron potential
        *this_neuron_potential += *incoming_potential; // - QuantizedDecimalTrait::QUANT_ZERO )); // TODO right now subtracting 0, but this is the resting potential

        // If consecutive fire is disabled, it is set to 0
        if cortical_area_data.consecutive_fire_limit == CPQ::CorticalLimitAndSnoozeQuants::QUANT_ZERO {
            if *this_neuron_potential.as_ref() > neuron_model_data.neuron_fire_threshold {
                return true;
            }
        }

        false
    }

    fn process_neuron_potential_for_none_layout_cortical_area(
        incoming_potential: &NeuronMembranePotential<CPQ::MembranePotentialQuant>,
        neuron_linear_index: &NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
        burst_index: &BurstIndex<FIQ::GlobalBurstIndexQuant>,
        memory_cortical_number_neurons: NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
        neuron_history: &Self::UsedNeuronHistory,
        cortical_area_data: &FeagiAdvancedModelCorticalData<CPQ>,
        neuron_model_data: &mut FeagiAdvancedModelNeuronData<CPQ>,
        this_neuron_potential: &mut NeuronMembranePotential<CPQ::MembranePotentialQuant>,
    ) -> bool {
        panic!("not implemented yet");
    }
}
