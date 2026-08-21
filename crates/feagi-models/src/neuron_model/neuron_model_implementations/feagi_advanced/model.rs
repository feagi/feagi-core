use crate::neuron_model::cortical_area::burst_index_rollover_handling::implementations::no_burst_index_rollover_handling::NeuronModelNoSpecialBurstIndexRolloverHandling;
use crate::neuron_model::cortical_area::cortical_layout::implementations::dimensional::DimensionalLayout;
use crate::neuron_model::cortical_area::neuron_history::implementations::full::NeuronModelFullNeuronHistory;
use crate::neuron_model::neuron_model::NeuronModel;
use crate::neuron_model::neuron_model_implementations::feagi_advanced::data::{FeagiAdvancedModelCorticalData, FeagiAdvancedModelNeuronData};
use crate::neuron_model::neuron_model_implementations::feagi_advanced::quantization::FeagiAdvancedModelQuantization;
use crate::wrapped_indexes::BurstIndex;
use feagi_data::neurons::neuron::neuron::{NeuronCorticalLocalIndex, NeuronMembranePotential};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::WrappedQuantizedUnsignedInteger;

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
    type CorticalLayout = DimensionalLayout<FIQ>;
    type CorticalData = FeagiAdvancedModelCorticalData<NMQ>;
    type NeuronData = FeagiAdvancedModelNeuronData<NMQ>;
    type NeuronHistoryType = NeuronModelFullNeuronHistory<FIQ>;
    type BurstIndexRolloverHandling = NeuronModelNoSpecialBurstIndexRolloverHandling;

    fn process_incoming_potential_for_dimensional_area(
        incoming_potential: &NeuronMembranePotential<NMQ::MembranePotentialQuant>,
        neuron_linear_index: &NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
        burst_index: &BurstIndex<FIQ::GlobalBurstIndexQuant>,
        cortical_layout: &Self::CorticalLayout,
        neuron_history: &Self::NeuronHistoryType,
        cortical_area_data: &Self::CorticalData,
        neuron_model_data: &mut Self::NeuronData,
        this_neuron_potential: &mut NeuronMembranePotential<NMQ::MembranePotentialQuant>,
    ) -> bool {
        if (burst_index.quant_to_usize() % 16) == 0 {
            return true;
        }
        return false;
    }
}
