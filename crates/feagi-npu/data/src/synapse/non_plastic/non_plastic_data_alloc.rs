


pub struct NonplasticSynapseAllocRAMStorage<SynapseIndexAndSize, NeuronIndex, Weight, Potential> {
    // Data
    synapse_data: Vec<NonplasticSynapseProperties>,

    // Cached Data
    cache_from_source_neuron_synapse_mappings: AHashMap<(NPUNeuronType, NeuronIndex), Vec<SynapseIndexAndSize>>,
    cache_to_destination_neuron_synapse_mappings: AHashMap<(NPUNeuronType, NeuronIndex), Vec<SynapseIndexAndSize>>,
    cache_number_valid_synapses: SynapseIndexAndSize,
    cache_number_invalid_synapses: SynapseIndexAndSize,
}

impl NonplasticSynapseAllocRAMStorage<SynapseIndexAndSize, NeuronIndex, Weight, Potential> {

}

impl NonplasticSynapseStaticStorageTrait<SynapseIndexAndSize, NeuronIndex, Weight, Potential> for NonplasticSynapseAllocRAMStorage<SynapseIndexAndSize, NeuronIndex, Weight, Potential> {

}

impl BaseSynapseStaticStorageTrait<SynapseIndexQuant, NeuronIndexQuant, PercentageQuant, PotentialQuant> for NonplasticSynapseAllocRAMStorage<SynapseIndexAndSize, NeuronIndex, Weight, Potential> {
    /// Gets the maximum possible synapse index achievable by current quantization (or in the case
    /// of static implementations, the size of the synapse array).
    fn get_max_possible_synapse_index(&self) -> SynapseIndexQuant {
        SynapseIndexAndSize::MAX
    }

    /// Returns the count of valid synapses in the structure. NOT THE SAME AS TOTAL NUMBER OF
    /// SYNAPSES STORED!
    fn get_total_number_of_valid_synapses(&self) -> &SynapseIndexQuant {
        &self.cache_number_valid_synapsesq
    }

    /// Returns the count of invalid synapses in the structure. NOT THE SAME AS TOTAL FREE CAPACITY!
    fn get_total_number_of_invalid_synapses(&self) -> &SynapseIndexQuant {
        &self.cache_number_invalid_synapsesq
    }
}