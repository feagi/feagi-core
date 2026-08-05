use crate::cortical_area::implementations::feagi_advanced::data::{FeagiAdvancedModelCorticalData, FeagiAdvancedModelNeuronData};
use crate::cortical_area::implementations::feagi_advanced::quantization::FeagiAdvancedModelQuantization;
use crate::cortical_area::neuron::neuron_history::implementations::full::NeuronModelFullNeuronHistory;
use crate::cortical_area::neuron::neuron_model::layout_specific::dimensional::DimensionalNeuronModel;
use crate::cortical_area::neuron::neuron_model::neuron_burst_index_rollover_handling::NeuronModelNoSpecialBurstIndexRolloverHandling;
use crate::cortical_area::neuron::neuron_model::neuron_model::NeuronModel;
use crate::wrapped_indexes::BurstIndex;
use feagi_data::neurons::{DimensionalCorticalArea4DDimensions, NeuronCorticalLocalIndex, NeuronMembranePotential};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

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
        _neuron_linear_index: &NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
        burst_index: &BurstIndex<FIQ::GlobalBurstIndexQuant>,
        _dimensional_cortical_dimensions: &DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexQuant>,
        neuron_history: &Self::NeuronHistoryType,
        cortical_area_data: &Self::CorticalData,
        neuron_model_data: &mut Self::NeuronData,
        this_neuron_potential: &mut NeuronMembranePotential<NMQ::MembranePotentialQuant>,
    ) -> bool {




        *this_neuron_potential += *incoming_potential; // - QuantizedDecimalTrait::QUANT_ZERO )); // TODO right now subtracting 0, but this is the resting potential



        false
    }
}
