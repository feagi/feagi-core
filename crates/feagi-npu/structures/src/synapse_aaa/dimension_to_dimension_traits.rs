//! Traits for synapses describing connections going from dimensional cortical areas to dimensional cortical areas


// TODO some things should be moved to a higher level trait as we understand other synapse types more

use crate::{CorticalTypedCorticalIndex, CorticalTypedNeuronIndex};
use crate::quantizables::{NPUGlobalQuantization, NPUSynapseQuantization, SynapseCount};
use crate::synapse_aaa::base_traits::{BaseSynapseAllocStorageTrait, BaseSynapseStaticStorageTrait, BaseSynapseStorageTrait};
use crate::synapse_aaa::feagi_npu_synapse_error::FeagiNPUSynapseError;

// NOTE: We know the type of synapse that will be created will always be a dimension to a dimensional neuron

pub trait Dim2DimSynapseBaseStorageTrait<Q: NPUGlobalQuantization, S: NPUSynapseQuantization>:
BaseSynapseStorageTrait<Q, S>
{

    //region Get Connections
    fn get_destination_neuron_indexes_from_source_neuron_index(&self, source_neuron_index: CorticalTypedNeuronIndex<Q::NeuronIndexCountQuant>)
        -> Result<&[CorticalTypedNeuronIndex<Q::NeuronIndexCountQuant>], FeagiNPUSynapseError>;

    fn get_source_neuron_indexes_from_destination_neuron_index(&self, destination_neuron_index: CorticalTypedNeuronIndex<Q::NeuronIndexCountQuant>)
        -> Result<&[CorticalTypedNeuronIndex<Q::NeuronIndexCountQuant>], FeagiNPUSynapseError>;


    //endregion


    //region Sparse Synapse Invalidation
    /// Invalidates (but does not remove) a single synapse
    fn kill_synapse_by_synapse_index(&mut self, synapse_index: CorticalTypedNeuronIndex<Q::NeuronIndexCountQuant>)
                                     -> Result<(), FeagiNPUSynapseError>;

    /// Invalidates (but does not remove) a multiple synapses
    fn kill_synapses_by_synapse_indexes(&mut self, synapse_index: CorticalTypedNeuronIndex<Q::NeuronIndexCountQuant>)
                                        -> Result<(), FeagiNPUSynapseError>;

    /// Invalidates all synapses with the given source neuron index, and returns the number of synapses invalidated
    fn kill_synapses_with_source_neuron_index(&mut self, source_neurons_index: CorticalTypedNeuronIndex<Q::NeuronIndexCountQuant>)
                                              -> Result<SynapseCount<S::SynapseIndexCountQuant>, FeagiNPUSynapseError>;

    /// Invalidates all synapses that come from one of the given source neuron indexes, and returns the total number of synapses invalidated
    fn kill_synapses_with_source_neuron_indexes(&mut self, source_neurons_indexes: &[CorticalTypedNeuronIndex<Q::NeuronIndexCountQuant>])
                                                -> Result<SynapseCount<S::SynapseIndexCountQuant>, FeagiNPUSynapseError>;

    /// Invalidates all synapses with the given destination neuron index, and returns the number of synapses invalidated
    fn kill_synapses_with_destination_neuron_index(&mut self, destination_neurons_index: CorticalTypedNeuronIndex<Q::NeuronIndexCountQuant>)
                                                   -> Result<SynapseCount<S::SynapseIndexCountQuant>, FeagiNPUSynapseError>;

    /// Invalidates all synapses that come from one of the given destination neuron indexes, and returns the total number of synapses invalidated
    fn kill_synapses_with_destination_neuron_indexes(&mut self, destination_neurons_indexes: &[CorticalTypedNeuronIndex<Q::NeuronIndexCountQuant>])
                                                     -> Result<SynapseCount<S::SynapseIndexCountQuant>, FeagiNPUSynapseError>;

    //endregion

}


pub trait Dim2DimSynapseStaticStorageTrait<Q: NPUGlobalQuantization, S: NPUSynapseQuantization>:
Dim2DimSynapseBaseStorageTrait<Q, S> +
BaseSynapseStaticStorageTrait<Q, S>
{


}

pub trait Dim2DimSynapseAllocStorageTrait<Q: NPUGlobalQuantization, S: NPUSynapseQuantization>:
Dim2DimSynapseBaseStorageTrait<Q, S> +
BaseSynapseAllocStorageTrait<Q, S>
{

    fn remove_all_synapses_mappings_to_and_from_cortical_area(&mut self, area_index: CorticalTypedCorticalIndex<Q::CorticalIndexCountQuant>)
        -> Result<(), FeagiNPUSynapseError>;

    fn remove_all_synaptic_mappings_between_cortical_areas(&mut self, source_area_index: CorticalTypedCorticalIndex<Q::CorticalIndexCountQuant>,
                                                           destination_area_index: CorticalTypedCorticalIndex<Q::CorticalIndexCountQuant>)
                                                           -> Result<(), FeagiNPUSynapseError>;

    // TODO
    //fn remove_specific_synaptic_mapping_between_cortical_areas(&mut self, source_area_index: CorticalAreaIndex<Q::CorticalIndex>, destination_area_index: CorticalAreaIndex<Q::CorticalIndex>, mapping_index: usize);

}
