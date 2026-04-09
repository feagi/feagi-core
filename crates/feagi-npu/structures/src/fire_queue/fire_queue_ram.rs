

use feagi_structures::base_quantizable::QuantizableUIntType;
use crate::fire_queue::traits::FireQueueTrait;
use crate::quantizables::NPUNeuronIndex;

pub struct FireQueueRam<NeuronIndexQuant> where
    NeuronIndexQuant: QuantizableUIntType,
{
    dimensional_neuron_indexes: Vec<NPUNeuronIndex<NeuronIndexQuant>>,
}

impl<NeuronIndexQuant> FireQueueRam<NeuronIndexQuant> where
    NeuronIndexQuant: QuantizableUIntType,
{
    pub fn new(preallocated_index_count: usize) -> Self {
        Self {
            dimensional_neuron_indexes: Vec::with_capacity(preallocated_index_count),
        }
    }
}

impl<NeuronIndexQuant> FireQueueTrait<NeuronIndexQuant> for FireQueueRam<NeuronIndexQuant> where
    NeuronIndexQuant: QuantizableUIntType,
{
    fn get_dimensional_neuron_indexes_slice(&self) -> &[NPUNeuronIndex<NeuronIndexQuant>] {
        self.dimensional_neuron_indexes.as_slice()
    }

    fn get_dimensional_neuron_indexes_slice_mut(&mut self) -> &[NPUNeuronIndex<NeuronIndexQuant>] {
        self.dimensional_neuron_indexes.as_mut_slice()
    }

    fn get_dimensional_neuron_indexes_mut(&mut self) -> &mut Vec<NPUNeuronIndex<NeuronIndexQuant>> {
        &mut self.dimensional_neuron_indexes
    }

    fn add_dimensional_neuron_index(&mut self, neuron_index_quant: NPUNeuronIndex<NeuronIndexQuant>) {
        self.dimensional_neuron_indexes.push(neuron_index_quant);
    }
}