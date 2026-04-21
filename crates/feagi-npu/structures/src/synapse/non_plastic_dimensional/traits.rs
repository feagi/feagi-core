
// NOTE: I suspect in common use, nonplastic connections also tend to be made in bundles. Lets capitalize on that

// TODO some things should be moved to a higher level trait as we understand other synapse types more

use feagi_structures::base_quantizable::QuantizableValueType;
use feagi_structures::FeagiStructuresError;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NumberNeuronsPerVoxel};
use crate::executors::cortical_mapping_definition_executors::NonPlasticCorticalMappingDefinitionExecutor;
use crate::neuron::dimensional_neurons::shared_structs::{DimensionalTypedCorticalIndex, DimensionalTypedNeuronIndex, NPUDimensionalAreaType};
use crate::neuron::flags::NeuronFlag;
use crate::quantizables::{NPUQuantization, NPUNeuronIndex, PSPMultiplier, SynapseBundleIndex, SynapticWeight};
use crate::synapse::base_traits::{BaseSynapseAllocStorageTrait, BaseSynapseStorageTrait};
use crate::synapse::dimension_to_dimension_traits::Dim2DimSynapseStaticStorageTrait;
use crate::synapse::feagi_npu_synapse_error::FeagiNPUSynapseError;
use crate::synapse::non_plastic_dimensional::NonPlasticSynapseFull;

pub trait NonplasticSynapseBaseStorageTrait<Q: NPUQuantization> :
BaseSynapseStorageTrait<Q>
{
    const DEFAULT_SYNAPSE_WEIGHT: SynapticWeight<Q::Value> = SynapticWeight::ONE;
    const DEFAULT_SYNAPSE_PSP: PSPMultiplier<Q::Value> = PSPMultiplier::ONE;

    //region Get Connections


    fn get_nonplastic_synapse_data_from_source_neuron_index(&self, source_neuron_index: DimensionalTypedNeuronIndex<Q::NeuronIndex>, )
        -> Result<(impl Iterator<Item=&NonPlasticSynapseFull<Q::NeuronIndex, Q::BurstDelta, Q::Value>>, NeuronCount<Q::NeuronIndex>), FeagiNPUSynapseError>;

    fn get_nonplastic_synapse_data_from_source_neuron_index_mut(&mut self, source_neuron_index: DimensionalTypedNeuronIndex<Q::NeuronIndex>)
        -> Result<(impl Iterator<Item=&mut NonPlasticSynapseFull<Q::NeuronIndex, Q::BurstDelta, Q::Value>>, NeuronCount<Q::NeuronIndex>), FeagiNPUSynapseError>;

    fn get_nonplastic_synapse_data_from_destination_neuron_index(&self, destination_neuron_index: DimensionalTypedNeuronIndex<Q::NeuronIndex>)
        -> Result<(impl Iterator<Item=&NonPlasticSynapseFull<Q::NeuronIndex, Q::BurstDelta, Q::Value>>, NeuronCount<Q::NeuronIndex>), FeagiNPUSynapseError>;

    fn get_nonplastic_synapse_data_from_destination_neuron_index_mut(&mut self, destination_neuron_index: DimensionalTypedNeuronIndex<Q::NeuronIndex>)
        -> Result<(impl Iterator<Item=&mut NonPlasticSynapseFull<Q::NeuronIndex, Q::BurstDelta, Q::Value>>, NeuronCount<Q::NeuronIndex>), FeagiNPUSynapseError>;


    //endregion




}


pub trait NonplasticSynapseStaticStorageTrait<Q: NPUQuantization> :
Dim2DimSynapseStaticStorageTrait<Q>
{




}

pub trait NonplasticSynapseAllocStorageTrait<Q: NPUQuantization> :
BaseSynapseAllocStorageTrait<Q>
{
    fn add_synapses_mapping_between_cortical_areas(&mut self,
                                                   source_area_index: DimensionalTypedCorticalIndex<Q::CorticalIndex>,
                                                   source_neuron_indexes: core::ops::Range<NPUNeuronIndex<Q::NeuronIndex>>,
                                                   source_neuron_flags: &[NeuronFlag],
                                                   source_cortical_dimensions: &NeuronVoxelDimensions<Q::Coord>,
                                                   source_neuron_density: NumberNeuronsPerVoxel,
                                                   destination_area_index: DimensionalTypedCorticalIndex<Q::CorticalIndex>,
                                                   destination_neuron_indexes: core::ops::Range<NPUNeuronIndex<Q::NeuronIndex>>,
                                                   destination_neuron_flags: &[NeuronFlag],
                                                   destination_cortical_dimensions: &NeuronVoxelDimensions<Q::Coord>,
                                                   destination_neuron_density: NumberNeuronsPerVoxel,
                                                   neuron_mapping_executor: &impl NonPlasticCorticalMappingDefinitionExecutor<Q::NeuronIndex, Q::SynapseIndex, Q::Coord, Q::CorticalIndex, Q::BurstDelta, Q::Value>)
        -> Result<SynapseBundleIndex<Q::SynapseBundleIndex>, FeagiStructuresError>;



}
