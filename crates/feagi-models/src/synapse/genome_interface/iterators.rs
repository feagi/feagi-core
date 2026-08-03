use feagi_data::neurons::NeuronCorticalLocalIndex;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

pub trait SynapseNeuronPairIterator<FIQ: FeagiIndexQuantization> {
    fn iterate_over_neuron_pairs(
        &self,
        number_source_neurons: &NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
        number_destination_neurons: &NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> impl Iterator<Item= (NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>, NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>)>;
}