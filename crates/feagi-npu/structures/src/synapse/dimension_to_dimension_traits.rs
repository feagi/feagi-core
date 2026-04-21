//! Traits for synapses describing connections going from dimensional cortical areas to dimensional cortical areas


// TODO some things should be moved to a higher level trait as we understand other synapse types more

use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use crate::neuron::dimensional_neurons::shared_structs::{DimensionalTypedCorticalIndex, DimensionalTypedNeuronIndex, NPUDimensionalAreaType};
use crate::quantizables::{NPUQuantization, NPUNeuronIndex, NPUSynapseIndex, SynapseCount};
use crate::synapse::base_traits::{BaseSynapseAllocStorageTrait, BaseSynapseStaticStorageTrait, BaseSynapseStorageTrait};
use crate::synapse::feagi_npu_synapse_error::FeagiNPUSynapseError;

// NOTE: We know the type of synapse that will be created will always be a dimension to a dimensional neuron

pub trait Dim2DimSynapseBaseStorageTrait<Q: NPUQuantization>:
BaseSynapseStorageTrait<Q>
{

    //region Get Connections
    fn get_destination_neuron_indexes_from_source_neuron_index(&self, source_neuron_index: DimensionalTypedNeuronIndex<Q::NeuronIndex>) -> Result<&[DimensionalTypedNeuronIndex<Q::NeuronIndex>], FeagiNPUSynapseError>;

    fn get_source_neuron_indexes_from_destination_neuron_index(&self, destination_neuron_index: DimensionalTypedNeuronIndex<Q::NeuronIndex>) -> Result<&[DimensionalTypedNeuronIndex<Q::NeuronIndex>], FeagiNPUSynapseError>;


    //endregion


    //region Sparse Synapse Invalidation
    /// Invalidates (but does not remove) a single synapse
    fn invalidate_synapse_by_synapse_index(&mut self, synapse_index: DimensionalTypedNeuronIndex<Q::NeuronIndex>)-> Result<(), FeagiNPUSynapseError>;

    /// Invalidates (but does not remove) a multiple synapses
    fn invalidate_synapses_by_synapse_indexes(&mut self, synapse_index: DimensionalTypedNeuronIndex<Q::NeuronIndex>)-> Result<(), FeagiNPUSynapseError>;

    /// Invalidates all synapses with the given source neuron index, and returns the number of synapses invalidated
    fn invalidate_synapses_with_source_neuron_index(&mut self, source_neurons_index: DimensionalTypedNeuronIndex<Q::NeuronIndex>) -> Result<SynapseCount<Q::SynapseIndex>, FeagiNPUSynapseError>;

    /// Invalidates all synapses that come from one of the given source neuron indexes, and returns the total number of synapses invalidated
    fn invalidate_synapses_with_source_neuron_indexes(&mut self, source_neurons_indexes: &[DimensionalTypedNeuronIndex<Q::NeuronIndex>]) -> Result<SynapseCount<Q::SynapseIndex>, FeagiNPUSynapseError>;

    /// Invalidates all synapses with the given destination neuron index, and returns the number of synapses invalidated
    fn invalidate_synapses_with_destination_neuron_index(&mut self, destination_neurons_index: DimensionalTypedNeuronIndex<Q::NeuronIndex>) -> Result<SynapseCount<Q::SynapseIndex>, FeagiNPUSynapseError>;

    /// Invalidates all synapses that come from one of the given destination neuron indexes, and returns the total number of synapses invalidated
    fn invalidate_synapses_with_destination_neuron_indexes(&mut self, destination_neurons_indexes: &[DimensionalTypedNeuronIndex<Q::NeuronIndex>]) -> Result<SynapseCount<Q::SynapseIndex>, FeagiNPUSynapseError>;

    //endregion

}


pub trait Dim2DimSynapseStaticStorageTrait<Q: NPUQuantization>:
Dim2DimSynapseBaseStorageTrait<Q> +
BaseSynapseStaticStorageTrait<Q>
{


}

pub trait Dim2DimSynapseAllocStorageTrait<Q: NPUQuantization>:
Dim2DimSynapseBaseStorageTrait<Q> +
BaseSynapseAllocStorageTrait<Q>
{

    fn remove_all_synapses_mappings_to_and_from_cortical_area(&mut self, area_index: DimensionalTypedCorticalIndex<Q::CorticalIndex>)
        -> Result<(), FeagiNPUSynapseError>;

    fn remove_all_synaptic_mappings_between_cortical_areas(&mut self, source_area_index: DimensionalTypedCorticalIndex<Q::CorticalIndex>,
                                                           destination_area_index: DimensionalTypedCorticalIndex<Q::CorticalIndex>)
        -> Result<(), FeagiNPUSynapseError>;

    // TODO
    //fn remove_specific_synaptic_mapping_between_cortical_areas(&mut self, source_area_index: CorticalAreaIndex<Q::CorticalIndex>, destination_area_index: CorticalAreaIndex<Q::CorticalIndex>, mapping_index: usize);



}
