use feagi_structures::base_quantizable::QuantizableUIntType;
use crate::quantizables::NPUNeuronIndex;

pub trait FireQueueTrait<NeuronIndexQuant> where
    NeuronIndexQuant: QuantizableUIntType,
{

    fn get_dimensional_neuron_indexes_slice(&self) -> &[NPUNeuronIndex<NeuronIndexQuant>];

    fn get_dimensional_neuron_indexes_slice_mut(&mut self) -> &[NPUNeuronIndex<NeuronIndexQuant>];

    fn get_dimensional_neuron_indexes_mut(&mut self) -> &mut Vec<NPUNeuronIndex<NeuronIndexQuant>>;

    fn add_dimensional_neuron_index(&mut self, neuron_index_quant: NPUNeuronIndex<NeuronIndexQuant>);
}

