


pub trait FireCandidateList<NeuronIndexQuant> where
    NeuronIndexQuant: QuantizableUInt,
{

    fn get_interneuron_indexes_slice(&self) -> &[NeuronNPUIndex<NeuronIndexQuant>];

    fn get_interneuron_indexes_slice_mut(&mut self) -> &[NeuronNPUIndex<NeuronIndexQuant>];

    fn get_interneuron_indexes_mut(&mut self) -> &mut Vec<NeuronNPUIndex<NeuronIndexQuant>>;

    fn add_interneuron_index(&mut self, neuron_index_quant: NeuronIndexQuant);
}

