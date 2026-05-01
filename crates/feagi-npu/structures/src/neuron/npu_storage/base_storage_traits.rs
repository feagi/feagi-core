// I am deliberately adding "trait" to their names to make things less confusing. I don't care about
// the convention right npw
// Only keep the parts that all neurons share in this base trait
// NOTE: I recommend parallel arrays for data storage instead an array of structs due to
// how data may be retrieved

use core::ops::Range;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::neurons::descriptors::{NeuronCount};
use crate::neuron::FeagiNPUNeuronError;
use crate::{CorticalTypedCorticalIndex, CorticalTypedNeuronIndex, NPUCorticalAreaIdentifierFlag};
use crate::quantizables::{NPUBaseNeuronQuantization, NPUGlobalQuantization, NPUNeuronIndex};

pub trait BaseNeuronCommonStorageTrait<Q: NPUGlobalQuantization, BNQ: NPUBaseNeuronQuantization> {

    const TYPE_CORTICAL_AREA: NPUCorticalAreaIdentifierFlag;

    fn get_cortical_typed_cortical_area_index(cortical_area_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>) 
        -> CorticalTypedCorticalIndex<Q::CorticalIndexCountQuant> {
        CorticalTypedCorticalIndex {
            index: cortical_area_index,
            cortical_type: Self::TYPE_CORTICAL_AREA,
        }
    }

    fn get_cortical_typed_neuron_index(neuron_index: NPUNeuronIndex<BNQ::NeuronIndexCountQuant>) -> CorticalTypedNeuronIndex<BNQ::NeuronIndexCountQuant> {
        CorticalTypedNeuronIndex {
            index: neuron_index,
            cortical_type: Self::TYPE_CORTICAL_AREA,
        }
    }

    /// Gets the maximum possible neuron index achievable by current quantization (or in the case
    /// of static implementations, the size of the array).
    fn get_max_possible_neuron_index(&self) -> NPUNeuronIndex<BNQ::NeuronIndexCountQuant>;



    /// Returns the count of valid neurons in the structure. NOT THE SAME AS TOTAL NUMBER OF
    /// NEURONS STORED!
    fn get_total_number_of_valid_neurons(&self) -> NeuronCount<BNQ::NeuronIndexCountQuant>;

    /// Returns the count of invalid neurons in the structure. NOT THE SAME AS TOTAL FREE CAPACITY!
    fn get_total_number_of_invalid_neurons(&self) -> NeuronCount<BNQ::NeuronIndexCountQuant>;

    fn get_number_cortical_areas(&self) -> CorticalAreaIndex<Q::CorticalIndexCountQuant>;
    
}

pub trait BaseNeuronFixedStorageTrait<Q: NPUGlobalQuantization, BNQ: NPUBaseNeuronQuantization>:
BaseNeuronCommonStorageTrait<Q, BNQ>
{
    // TODO?
}

#[cfg(feature = "alloc")]
pub trait BaseNeuronResizableStorageTrait<Q: NPUGlobalQuantization, BNQ: NPUBaseNeuronQuantization>:
BaseNeuronCommonStorageTrait<Q, BNQ>
{
    /// Frees unused neuron vector capacity and invalid neurons (assuming they were sorted to the back first!)
    /// albeit allowing a buffer of free space. Returns the number of neurons that were freed.
    /// Returns 0 if no neurons were freed (nothing to free or spare capacity is at or less than
    /// what was requested). Note that invalid neurons not sorted to the back will not be freed.
    fn free_unused_neuron_capacity(&mut self, spare_capacity_to_maintain: NeuronCount<BNQ::NeuronIndexCountQuant>) -> NeuronCount<BNQ::NeuronIndexCountQuant>;


    /// Deletes a cortical area by invalidating all of its neurons. Returns the neuron indexes
    /// of the disabled neurons
    /// WARNING: BE SURE TO REMOVE ASSOCIATED SYNAPSE MAPPINGS!
    fn delete_cortical_area(&mut self, cortical_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>)
                            -> Result<Range<NPUNeuronIndex<BNQ::NeuronIndexCountQuant>>, FeagiNPUNeuronError>;
    
    // TODO Duplicate Cortical Areas? With and without mappings?

}
