use crate::base_quantizable::unsigned_integer::QuantizableUInt;
use crate::base_quantizable::value::QuantizableValue;
use crate::neuron::data::SynapseFlag;

pub trait SynapseData<SynapseIndexAndSize, NeuronIndex, Weight, Potential>
where
    SynapseIndexAndSize: QuantizableUInt,
    NeuronIndex: QuantizableUInt,
    Weight: QuantizableValue, // TODO float??
    Potential: QuantizableUInt,
{
    fn current_synapse_capacity(&self) -> SynapseIndexAndSize;
    fn set_current_synapse_capacity(&mut self, capacity: SynapseIndexAndSize);

    fn get_source_neuron(&self, synapse_index: SynapseIndexAndSize) -> NeuronIndex;
    fn set_source_neuron(&mut self, synapse_index: SynapseIndexAndSize, source_neuron: NeuronIndex);

    fn get_destination_neuron(&self, synapse_index: SynapseIndexAndSize) -> NeuronIndex;
    fn set_destination_neuron(&mut self, synapse_index: SynapseIndexAndSize, destination_neuron: NeuronIndex);

    fn get_weight(&self, synapse_index: SynapseIndexAndSize) -> Weight;
    fn set_weight(&mut self, synapse_index: SynapseIndexAndSize, weight: Weight);

    fn get_post_synaptic_potential(&self, synapse_index: SynapseIndexAndSize) -> Potential;
    fn set_post_synaptic_potential(&mut self, synapse_index: SynapseIndexAndSize, post_synaptic_potential: Potential);

    fn get_synapse_flag(&self, synapse_index: SynapseIndexAndSize) -> SynapseFlag;
    fn set_synapse_flag(&mut self, synapse_index: SynapseIndexAndSize, synapse_flag: SynapseFlag);
}