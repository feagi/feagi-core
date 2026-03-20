

pub trait BaseSynapseStaticStorageTrait<SynapseIndexQuant, NeuronIndexQuant, PercentageQuant, PotentialQuant >
where
    SynapseIndexQuant: SynapseIndex,
    NeuronIndexQuant: QuantizableUInt,
    PercentageQuant: PercentageScale,
    PotentialQuant: PotentialUnit,
{
    const NUMBER_BYTES_PER_NEURON: usize;

    /// Gets the maximum possible synapse index achievable by current quantization (or in the case
    /// of static implementations, the size of the synapse array).
    fn get_max_possible_synapse_index(&self) -> SynapseIndexQuant;

    /// Returns the count of valid synapses in the structure. NOT THE SAME AS TOTAL NUMBER OF
    /// SYNAPSES STORED!
    fn get_total_number_of_valid_synapses(&self) -> &SynapseIndexQuant;

    /// Returns the count of invalid synapses in the structure. NOT THE SAME AS TOTAL FREE CAPACITY!
    fn get_total_number_of_invalid_synapses(&self) -> &SynapseIndexQuant;

    // TODO should I have a function that takes a source neuron and returns an iterator of destinations, or that takes a iterator of source neurons and returns a destination iterator iterator (cursed) ?
    // why not both?
}