
// NOTE: I suspect in common use, nonplastic connections also tend to be made in bundles. Lets capitalize on that

// TODO some things should be moved to a higher level trait as we understand other synapse types more

use feagi_structures::quantization::QuantizableValueType;
use feagi_structures::FeagiStructuresError;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::genomic::cortical_area::DimensionCorticalAreaType;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NumberNeuronsPerVoxel};
use crate::{CorticalTypedCorticalIndex, CorticalTypedNeuronIndex};
use crate::executors::cortical_mapping_definition_executors::NonPlasticCorticalMappingDefinitionExecutor;
use crate::neuron::flags::NeuronFlag;
use crate::quantizables::{NPUGlobalQuantization, NPUNeuronIndex, NPUSynapseQuantization, PSPMultiplier, SynapseBundleIndex, SynapticWeight};
use crate::synapse::base_traits::{BaseSynapseAllocStorageTrait, BaseSynapseStorageTrait};
use crate::synapse::dimension_to_dimension_traits::{Dim2DimSynapseBaseStorageTrait, Dim2DimSynapseStaticStorageTrait};
use crate::synapse::feagi_npu_synapse_error::FeagiNPUSynapseError;
use crate::synapse::non_plastic_dimensional::non_plastic_synapse::NonPlasticSynapseFull;

pub trait NonplasticSynapseBaseStorageTrait<Q: NPUGlobalQuantization, S: NPUSynapseQuantization>:
Dim2DimSynapseBaseStorageTrait<Q, S>
{

    //region Get Connections
    
    fn get_nonplastic_synapse_data_from_source_neuron_index(&self, source_neuron_index: CorticalTypedNeuronIndex<Q::NeuronIndexCountQuant>)
        -> Result<(impl Iterator<Item=&NonPlasticSynapseFull<Q, S>>, NeuronCount<Q::NeuronIndexCountQuant>), FeagiNPUSynapseError>;

    fn get_nonplastic_synapse_data_from_destination_neuron_index(&self, destination_neuron_index: CorticalTypedNeuronIndex<Q::NeuronIndexCountQuant>)
        -> Result<(impl Iterator<Item=&NonPlasticSynapseFull<Q, S>>, NeuronCount<Q::NeuronIndexCountQuant>), FeagiNPUSynapseError>;


    //endregion

}


pub trait NonplasticSynapseStaticStorageTrait<Q: NPUGlobalQuantization, S: NPUSynapseQuantization> :
Dim2DimSynapseStaticStorageTrait<Q, S>
{

    
}

pub trait NonplasticSynapseAllocStorageTrait<Q: NPUGlobalQuantization, S: NPUSynapseQuantization> :
BaseSynapseAllocStorageTrait<Q, S>
{
    fn add_synapses_mapping_between_cortical_areas(&mut self,
                                                   source_area_index: CorticalTypedCorticalIndex<Q::CorticalIndexCountQuant>,
                                                   source_cortical_dimensions: &NeuronVoxelDimensions<usize>,
                                                   source_cortical_neurons_per_voxel: &NeuronCount<NumberNeuronsPerVoxel>,
                                                   source_neuron_flags: &[NeuronFlag],
                                                   destination_area_index: CorticalTypedCorticalIndex<Q::CorticalIndexCountQuant>,
                                                   destination_cortical_dimensions: &NeuronVoxelDimensions<usize>,
                                                   destination_cortical_neurons_per_voxel: &NeuronCount<NumberNeuronsPerVoxel>,
                                                   destination_neuron_flags: &[NeuronFlag],
                                                   neuron_mapping_executor: &impl NonPlasticCorticalMappingDefinitionExecutor<Q>)
                                                   -> Result<SynapseBundleIndex<S::SynapseBundleIndexCountQuant>, FeagiStructuresError>;



}
