// I am deliberately adding "trait" to their names to make things less confusing. I don't care about
// the convention right npw
// Only keep the parts that all neurons share in this base trait
// NOTE: I recommend parallel arrays for data storage instead an array of structs due to
// how data may be retrieved

use core::ops::Range;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::neurons::descriptors::NeuronCount;
use crate::neuron::FeagiNPUNeuronError;
use crate::quantizables::{NPUQuantization, NPUNeuronIndex};

pub trait BaseNeuronStaticStorageTrait<Q: NPUQuantization>
{
    // NOTE: Due to varying internal implementations, memory fragmentation may occur in various
    // ways, ergo be cautious of calculating internal state. Instead, use the below helper functions.

    const NUMBER_BYTES_PER_NEURON: usize;

    /// Gets the maximum possible neuron index achievable by current quantization (or in the case
    /// of static implementations, the size of the array).
    fn get_max_possible_neuron_index(&self) -> NPUNeuronIndex<Q::NeuronIndex>;

    /// Returns the count of valid neurons in the structure. NOT THE SAME AS TOTAL NUMBER OF
    /// NEURONS STORED!
    fn get_total_number_of_valid_neurons(&self) -> NeuronCount<Q::NeuronIndex>;

    /// Returns the count of invalid neurons in the structure. NOT THE SAME AS TOTAL FREE CAPACITY!
    fn get_total_number_of_invalid_neurons(&self) -> NeuronCount<Q::NeuronIndex>;

    /// Gets the maximum possible cortical area index achievable by current quantization (or in the
    /// case of static implementations, the size of the array).
    fn get_max_possible_cortical_area_index(&self) -> CorticalAreaIndex<Q::CorticalIndex>;



    // /// Brings all invalid neurons to the back of the internal arrays and returns the number of
    // /// them. Note that depending on the implementation, neuron order may not be preserved
    //fn sort_invalid_neurons_to_the_back(&self) -> NeuronIndexQuant; // TODO we need to be more careful here, bind this with synapses

}

#[cfg(feature = "alloc")]
pub trait BaseNeuronAllocStorageTrait<Q: NPUQuantization>:
BaseNeuronStaticStorageTrait<Q>
{
    /// Frees unused neuron vector capacity and invalid neurons (assuming they were sorted to the back first!)
    /// albeit allowing a buffer of free space. Returns the number of neurons that were freed.
    /// Returns 0 if no neurons were freed (nothing to free or spare capacity is at or less than
    /// what was requested). Note that invalid neurons not sorted to the back will not be freed.
    fn free_unused_neuron_capacity(&mut self, spare_capacity_to_maintain: NeuronCount<Q::NeuronIndex>) -> NeuronCount<Q::NeuronIndex>;


    /// Deletes a cortical area by invalidating all of its neurons. Returns the neuron indexes
    /// of the disabled neurons
    /// WARNING: BE SURE TO REMOVE ASSOCIATED SYNAPSE MAPPINGS!
    fn delete_cortical_area(&mut self, cortical_index: CorticalAreaIndex<Q::CorticalIndex>)
                            -> Result<Range<NPUNeuronIndex<Q::NeuronIndex>>, FeagiNPUNeuronError>;
    
    // TODO Duplicate Cortical Area

}
