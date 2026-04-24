
// NOTE: I suspect in common use, nonplastic connections also tend to be made in bundles. Lets capitalize on that

// TODO some things should be moved to a higher level trait as we understand other synapse types more

use feagi_structures::base_quantizable::QuantizableValueType;
use feagi_structures::FeagiStructuresError;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::genomic::cortical_area::DimensionCorticalAreaType;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NumberNeuronsPerVoxel};
use crate::executors::cortical_mapping_definition_executors::NonPlasticCorticalMappingDefinitionExecutor;
use crate::neuron::dimensional_neurons::shared_structs::{DimensionalNeuronCorticalData, DimensionalTypedCorticalIndex, DimensionalTypedNeuronIndex};
use crate::neuron::flags::NeuronFlag;
use crate::quantizables::{NPUQuantization, NPUNeuronIndex, PSPMultiplier, SynapseBundleIndex, SynapticWeight};
use crate::synapse::base_traits::{BaseSynapseAllocStorageTrait, BaseSynapseStorageTrait};
use crate::synapse::dimension_to_dimension_traits::Dim2DimSynapseStaticStorageTrait;
use crate::synapse::feagi_npu_synapse_error::FeagiNPUSynapseError;
use crate::synapse::non_plastic_dimensional::NonPlasticSynapseFull;

pub trait NonplasticSynapseBaseStorageTrait<Q: NPUQuantization> :
BaseSynapseStorageTrait<Q>
{
    const DEFAULT_SYNAPSE_WEIGHT: SynapticWeight<Q::ValueQuant> = SynapticWeight::ONE;
    const DEFAULT_SYNAPSE_PSP: PSPMultiplier<Q::ValueQuant> = PSPMultiplier::ONE;

    //region Get Connections


    fn get_nonplastic_synapse_data_from_source_neuron_index(&self, source_neuron_index: DimensionalTypedNeuronIndex<Q::NeuronIndexQuant>, )
                                                            -> Result<(impl Iterator<Item=&NonPlasticSynapseFull<Q::NeuronIndexQuant, Q::BurstDeltaQuant, Q::ValueQuant>>, NeuronCount<Q::NeuronIndexQuant>), FeagiNPUSynapseError>;

    fn get_nonplastic_synapse_data_from_destination_neuron_index(&self, destination_neuron_index: DimensionalTypedNeuronIndex<Q::NeuronIndexQuant>)
        -> Result<(impl Iterator<Item=&NonPlasticSynapseFull<Q::NeuronIndexQuant, Q::BurstDeltaQuant, Q::ValueQuant>>, NeuronCount<Q::NeuronIndexQuant>), FeagiNPUSynapseError>;


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
                                                   source_area_index: DimensionalTypedCorticalIndex<Q::CorticalIndexQuant>,
                                                   source_cortical_data: &DimensionalNeuronCorticalData<Q>,
                                                   source_neuron_flags: &[NeuronFlag],
                                                   destination_area_index: DimensionalTypedCorticalIndex<Q::CorticalIndexQuant>,
                                                   destination_cortical_data: &DimensionalNeuronCorticalData<Q>,
                                                   destination_neuron_flags: &[NeuronFlag],
                                                   neuron_mapping_executor: &impl NonPlasticCorticalMappingDefinitionExecutor<Q>)
                                                   -> Result<SynapseBundleIndex<Q::SynapseBundleIndexQuant>, FeagiStructuresError>;



}
