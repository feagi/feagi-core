
// TODO I dont think this is a good implementation. By using a vector, its very easy to add a neuron
// multiple times. Now while this can be addressedwith some sorting / searching, I dont like this.
// This can also be addressed with a hashset, but thats not all compatible with embedded and can be
// a bit heavy. We should think about this.

pub struct FireCandidateListRam<NeuronIndexQuant> where
    NeuronIndexQuant: QuantizableUInt,
{
    dimensional_neuron_indexes: Vec<NeuronNPUIndex<NeuronIndexQuant>>,
}

impl<NeuronIndexQuant> FireCandidateListRam<NeuronIndexQuant> where
    NeuronIndexQuant: QuantizableUInt,
{
    pub fn new(preallocated_index_count: usize) -> Self {
        Self {
            dimensional_neuron_indexes: Vec::with_capacity(preallocated_index_count),
        }
    }
}

impl<NeuronIndexQuant> FireCandidateList<NeuronIndexQuant> for FireCandidateListRam<NeuronIndexQuant> where
    NeuronIndexQuant: QuantizableUInt,
{
    fn get_dimensional_neuron_indexes_slice(&self) -> &[NeuronNPUIndex<NeuronIndexQuant>] {
        self.dimensional_neuron_indexes.as_slice()
    }

    fn get_dimensional_neuron_indexes_slice_mut(&mut self) -> &[NeuronNPUIndex<NeuronIndexQuant>] {
        self.dimensional_neuron_indexes.as_mut_slice()
    }

    fn get_dimensional_neuron_indexes_mut(&self) -> &mut Vec<NeuronNPUIndex<NeuronIndexQuant>> {
        &mut self.dimensional_neuron_indexes
    }

    fn add_dimensional_neuron_index(&mut self, neuron_index_quant: NeuronIndexQuant) {
        self.dimensional_neuron_indexes.push(neuron_index_quant);
    }
}