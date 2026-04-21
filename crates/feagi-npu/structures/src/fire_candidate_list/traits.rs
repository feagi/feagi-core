use feagi_structures::base_quantizable::QuantizableUIntType;
use crate::quantizables::NPUNeuronIndex;

pub trait FireCandidateListTrait<NeuronIndexQuant> where
    NeuronIndexQuant: QuantizableUIntType,
{
    fn get_core_neuron_indexes_slice(&self) -> &[NPUNeuronIndex<NeuronIndexQuant>];

    fn get_core_neuron_indexes_slice_mut(&mut self) -> &[NPUNeuronIndex<NeuronIndexQuant>];

    fn get_core_neuron_indexes_mut(&mut self) -> &mut Vec<NPUNeuronIndex<NeuronIndexQuant>>;

    fn add_core_neuron_index(&mut self, neuron_index_quant: NPUNeuronIndex<NeuronIndexQuant>);

    fn get_sensory_neuron_indexes_slice(&self) -> &[NPUNeuronIndex<NeuronIndexQuant>];

    fn get_sensory_neuron_indexes_slice_mut(&mut self) -> &[NPUNeuronIndex<NeuronIndexQuant>];

    fn get_sensory_neuron_indexes_mut(&mut self) -> &mut Vec<NPUNeuronIndex<NeuronIndexQuant>>;

    fn add_sensory_neuron_index(&mut self, neuron_index_quant: NPUNeuronIndex<NeuronIndexQuant>);

    fn get_motor_neuron_indexes_slice(&self) -> &[NPUNeuronIndex<NeuronIndexQuant>];

    fn get_motor_neuron_indexes_slice_mut(&mut self) -> &[NPUNeuronIndex<NeuronIndexQuant>];

    fn get_motor_neuron_indexes_mut(&mut self) -> &mut Vec<NPUNeuronIndex<NeuronIndexQuant>>;

    fn add_motor_neuron_index(&mut self, neuron_index_quant: NPUNeuronIndex<NeuronIndexQuant>);

    fn get_inter_neuron_indexes_slice(&self) -> &[NPUNeuronIndex<NeuronIndexQuant>];

    fn get_inter_neuron_indexes_slice_mut(&mut self) -> &[NPUNeuronIndex<NeuronIndexQuant>];

    fn get_inter_neuron_indexes_mut(&mut self) -> &mut Vec<NPUNeuronIndex<NeuronIndexQuant>>;

    fn add_inter_neuron_index(&mut self, neuron_index_quant: NPUNeuronIndex<NeuronIndexQuant>);

    fn clear(&mut self);
}

