use feagi_structures::base_quantizable::{QuantizablePercentType, QuantizableUIntType, QuantizableValueType};
use crate::quantizables::{NPUSynapseIndex, SynapseCount};

// NOTE: we cannot add most properties to the base trait since synapse types vary wildly in implementation

pub trait BaseSynapseStaticStorageTrait<SynapseIndexQuant, NeuronIndexQuant, PercentageQuant, PotentialQuant >
where
    SynapseIndexQuant: QuantizableUIntType,
    NeuronIndexQuant: QuantizableUIntType,
    PercentageQuant: QuantizablePercentType,
    PotentialQuant: QuantizableValueType,
{
    const NUMBER_BYTES_PER_NEURON: usize;

    /// Gets the maximum possible synapse index achievable by current quantization (or in the case
    /// of static implementations, the size of the synapse array).
    fn get_max_possible_synapse_index(&self) -> NPUSynapseIndex<SynapseIndexQuant>;

    /// Returns the count of valid synapses in the structure. NOT THE SAME AS TOTAL NUMBER OF
    /// SYNAPSES STORED!
    fn get_total_number_of_valid_synapses(&self) -> &SynapseCount<SynapseIndexQuant>;

    /// Returns the count of invalid synapses in the structure. NOT THE SAME AS TOTAL FREE CAPACITY!
    fn get_total_number_of_invalid_synapses(&self) -> &SynapseCount<SynapseIndexQuant>;

    // TODO should I have a function that takes a source neuron and returns an iterator of destinations, or that takes a iterator of source neurons and returns a destination iterator iterator (cursed) ?
    // why not both?
}

pub trait BaseSynapseAllocStorageTrait<SynapseIndexQuant, NeuronIndexQuant, PercentageQuant, PotentialQuant >:
BaseSynapseStaticStorageTrait<SynapseIndexQuant, NeuronIndexQuant, PercentageQuant, PotentialQuant>
where
    SynapseIndexQuant: QuantizableUIntType,
    NeuronIndexQuant: QuantizableUIntType,
    PercentageQuant: QuantizablePercentType,
    PotentialQuant: QuantizableValueType,
{
    fn free_unused_synapse_capacity(&mut self, spare_capacity_to_maintain: SynapseCount<SynapseIndexQuant>) -> SynapseCount<SynapseIndexQuant>;
}