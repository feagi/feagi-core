use feagi_data::neurons::{DimensionalCorticalArea4DDimensions, NeuronCorticalLocalIndex, NeuronMembranePotential};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::cortical_area::neuron::neuron_model::NeuronModel;
use crate::cortical_area::neuron::neuron_model_quantization::NeuronModelQuantization;
use crate::wrapped_indexes::BurstIndex;

/// Extend `NeuronModel` to denote that the model can function on dimensional cortical areas
pub trait DimensionalNeuronModel<FIQ, NMQ>: NeuronModel<FIQ, NMQ>
where // NOTE: These all should be extended for the given neuron model!
    FIQ: FeagiIndexQuantization,
    NMQ: NeuronModelQuantization,
{

    /// Dimensional Neuron received input potential. Process it, updating any internal states and
    /// update this neurons potential. Return true if it results in this neuron firing, otherwise
    /// return false.
    fn process_incoming_potential_for_dimensional_area(
        incoming_potential: &NeuronMembranePotential<NMQ::MembranePotentialQuant>,
        neuron_linear_index: &NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
        burst_index: &BurstIndex<FIQ::GlobalBurstIndexQuant>,
        dimensional_cortical_dimensions: &DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexQuant>,
        neuron_history: &Self::NeuronHistoryType,
        cortical_area_data: &Self::CorticalData,
        neuron_model_data: &mut Self::NeuronData,
        this_neuron_potential: &mut NeuronMembranePotential<NMQ::MembranePotentialQuant>,
    ) -> bool;
}