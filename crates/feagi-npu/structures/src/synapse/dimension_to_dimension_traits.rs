//! Traits for synapses describing connections going from dimensional cortical areas to dimensional cortical areas


// TODO some things should be moved to a higher level trait as we understand other synapse types more

use feagi_structures::base_quantizable::{QuantizablePercentType, QuantizableUIntType, QuantizableValueType};
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use crate::quantizables::{NPUNeuronIndex, NPUSynapseIndex, PSPMultiplier, SynapseCount, SynapticWeight};
use crate::synapse::base_traits::{BaseSynapseAllocStorageTrait, BaseSynapseStaticStorageTrait, BaseSynapseStorageTrait};
use crate::synapse::feagi_npu_synapse_error::FeagiNPUSynapseError;
use crate::synapse::non_plastic_dimensional::NonPlasticSynapseFull;

// NOTE: We know the type of synapse that will be created will always be a dimension to a dimensional neuron

pub trait Dim2DimSynapseBaseStorageTrait<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>:
BaseSynapseStorageTrait<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    SynapseIndexQuant: QuantizableUIntType,
    SynapseBundleIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{

    //region Get Connections
    fn get_destination_neuron_indexes_from_source_neuron_index(&self, source_neuron_index: NPUNeuronIndex<NeuronIndexQuant>) -> Result<&[NPUNeuronIndex<NeuronIndexQuant>], FeagiNPUSynapseError>;

    fn get_source_neuron_indexes_from_destination_neuron_index(&self, destination_neuron_index: NPUNeuronIndex<NeuronIndexQuant>) -> Result<&[NPUNeuronIndex<NeuronIndexQuant>], FeagiNPUSynapseError>;


    //endregion


    //region Synapse Invalidation
    /// Invalidates (but does not remove) a single synapse
    fn invalidate_synapse_by_synapse_index(&mut self, synapse_index: NPUSynapseIndex<SynapseIndexQuant>)-> Result<(), FeagiNPUSynapseError>;

    /// Invalidates (but does not remove) a multiple synapses
    fn invalidate_synapses_by_synapse_indexes(&mut self, synapse_index: NPUSynapseIndex<SynapseIndexQuant>)-> Result<(), FeagiNPUSynapseError>;

    /// Invalidates all synapses with the given source neuron index, and returns the number of synapses invalidated
    fn invalidate_synapses_with_source_neuron_index(&mut self, source_neurons_index: NPUNeuronIndex<NeuronIndexQuant>) -> Result<SynapseCount<SynapseIndexQuant>, FeagiNPUSynapseError>;

    /// Invalidates all synapses that come from one of the given source neuron indexes, and returns the total number of synapses invalidated
    fn invalidate_synapses_with_source_neuron_indexes(&mut self, source_neurons_indexes: &[NPUNeuronIndex<NeuronIndexQuant>]) -> Result<SynapseCount<SynapseIndexQuant>, FeagiNPUSynapseError>;

    /// Invalidates all synapses with the given destination neuron index, and returns the number of synapses invalidated
    fn invalidate_synapses_with_destination_neuron_index(&mut self, destination_neurons_index: NPUNeuronIndex<NeuronIndexQuant>) -> Result<SynapseCount<SynapseIndexQuant>, FeagiNPUSynapseError>;

    /// Invalidates all synapses that come from one of the given destination neuron indexes, and returns the total number of synapses invalidated
    fn invalidate_synapses_with_destination_neuron_indexes(&mut self, destination_neurons_indexes: &[NPUNeuronIndex<NeuronIndexQuant>]) -> Result<SynapseCount<SynapseIndexQuant>, FeagiNPUSynapseError>;

    //endregion

}


pub trait Dim2DimSynapseStaticStorageTrait<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>:
Dim2DimSynapseBaseStorageTrait<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant> +
BaseSynapseStaticStorageTrait<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    SynapseIndexQuant: QuantizableUIntType,
    SynapseBundleIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{


}

pub trait Dim2DimSynapseAllocStorageTrait<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>:
Dim2DimSynapseBaseStorageTrait<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant> +
BaseSynapseAllocStorageTrait<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    SynapseIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{

    fn remove_all_synapses_mappings_to_and_from_cortical_area(&mut self, area_index: CorticalAreaIndex<CorticalIndexQuant>);

    fn remove_all_synaptic_mappings_between_cortical_areas(&mut self, source_area_index: CorticalAreaIndex<CorticalIndexQuant>, destination_area_index: CorticalAreaIndex<CorticalIndexQuant>);

    // TODO
    //fn remove_specific_synaptic_mapping_between_cortical_areas(&mut self, source_area_index: CorticalAreaIndex<CorticalIndexQuant>, destination_area_index: CorticalAreaIndex<CorticalIndexQuant>, mapping_index: usize);



}