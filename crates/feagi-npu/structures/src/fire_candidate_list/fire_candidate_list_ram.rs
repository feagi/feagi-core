
// TODO I dont think this is a good implementation. By using a vector, its very easy to add a neuron
// multiple times. Now while this can be addressedwith some sorting / searching, I dont like this.
// This can also be addressed with a hashset, but thats not all compatible with embedded and can be
// a bit heavy. We should think about this.

use feagi_structures::base_quantizable::QuantizableUIntType;
use crate::fire_candidate_list::FireCandidateListTrait;
use crate::quantizables::NPUNeuronIndex;

pub struct FireCandidateListRam<NeuronIndexQuant> where
    NeuronIndexQuant: QuantizableUIntType,
{
    core_neuron_indexes: Vec<NPUNeuronIndex<NeuronIndexQuant>>,
    sensory_neuron_indexes: Vec<NPUNeuronIndex<NeuronIndexQuant>>,
    motor_neuron_indexes: Vec<NPUNeuronIndex<NeuronIndexQuant>>,
    inter_neuron_indexes: Vec<NPUNeuronIndex<NeuronIndexQuant>>,
}

impl<NeuronIndexQuant> FireCandidateListRam<NeuronIndexQuant> where
    NeuronIndexQuant: QuantizableUIntType,
{
    pub fn new(preallocated_index_count: usize) -> Self {
        Self {
            core_neuron_indexes: Vec::with_capacity(preallocated_index_count),
            sensory_neuron_indexes: Vec::with_capacity(preallocated_index_count),
            motor_neuron_indexes: Vec::with_capacity(preallocated_index_count),
            inter_neuron_indexes: Vec::with_capacity(preallocated_index_count),
        }
    }
}

impl<NeuronIndexQuant> FireCandidateListTrait<NeuronIndexQuant> for FireCandidateListRam<NeuronIndexQuant> where
    NeuronIndexQuant: QuantizableUIntType,
{
    fn get_core_neuron_indexes_slice(&self) -> &[NPUNeuronIndex<NeuronIndexQuant>] {
        self.core_neuron_indexes.as_slice()
    }

    fn get_core_neuron_indexes_slice_mut(&mut self) -> &[NPUNeuronIndex<NeuronIndexQuant>] {
        self.core_neuron_indexes.as_mut_slice()
    }

    fn get_core_neuron_indexes_mut(&mut self) -> &mut Vec<NPUNeuronIndex<NeuronIndexQuant>> {
        &mut self.core_neuron_indexes
    }

    fn add_core_neuron_index(&mut self, neuron_index_quant: NPUNeuronIndex<NeuronIndexQuant>) {
        self.core_neuron_indexes.push(neuron_index_quant);
    }

    fn get_sensory_neuron_indexes_slice(&self) -> &[NPUNeuronIndex<NeuronIndexQuant>] {
        self.sensory_neuron_indexes.as_slice()
    }

    fn get_sensory_neuron_indexes_slice_mut(&mut self) -> &[NPUNeuronIndex<NeuronIndexQuant>] {
        self.sensory_neuron_indexes.as_mut_slice()
    }

    fn get_sensory_neuron_indexes_mut(&mut self) -> &mut Vec<NPUNeuronIndex<NeuronIndexQuant>> {
        &mut self.sensory_neuron_indexes
    }

    fn add_sensory_neuron_index(&mut self, neuron_index_quant: NPUNeuronIndex<NeuronIndexQuant>) {
        self.sensory_neuron_indexes.push(neuron_index_quant);
    }

    fn get_motor_neuron_indexes_slice(&self) -> &[NPUNeuronIndex<NeuronIndexQuant>] {
        self.motor_neuron_indexes.as_slice()
    }

    fn get_motor_neuron_indexes_slice_mut(&mut self) -> &[NPUNeuronIndex<NeuronIndexQuant>] {
        self.motor_neuron_indexes.as_mut_slice()
    }

    fn get_motor_neuron_indexes_mut(&mut self) -> &mut Vec<NPUNeuronIndex<NeuronIndexQuant>> {
        &mut self.motor_neuron_indexes
    }

    fn add_motor_neuron_index(&mut self, neuron_index_quant: NPUNeuronIndex<NeuronIndexQuant>) {
        self.motor_neuron_indexes.push(neuron_index_quant);
    }

    fn get_inter_neuron_indexes_slice(&self) -> &[NPUNeuronIndex<NeuronIndexQuant>] {
        self.inter_neuron_indexes.as_slice()
    }

    fn get_inter_neuron_indexes_slice_mut(&mut self) -> &[NPUNeuronIndex<NeuronIndexQuant>] {
        self.inter_neuron_indexes.as_mut_slice()
    }

    fn get_inter_neuron_indexes_mut(&mut self) -> &mut Vec<NPUNeuronIndex<NeuronIndexQuant>> {
        &mut self.inter_neuron_indexes
    }

    fn add_inter_neuron_index(&mut self, neuron_index_quant: NPUNeuronIndex<NeuronIndexQuant>) {
        self.inter_neuron_indexes.push(neuron_index_quant);
    }

    fn clear(&mut self) {
        self.core_neuron_indexes.clear();
        self.sensory_neuron_indexes.clear();
        self.motor_neuron_indexes.clear();
        self.inter_neuron_indexes.clear();
    }
}