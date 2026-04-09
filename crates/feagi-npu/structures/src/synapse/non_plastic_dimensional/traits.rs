
// NOTE: I suspect in common use, nonplastic connections also tend to be made in bundles. Lets capitalize on that

// TODO some things should be moved to a higher level trait as we understand other synapse types more

use feagi_structures::base_quantizable::{QuantizablePercentType, QuantizableUIntType, QuantizableValueType};
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NumberNeuronsPerVoxel};
use crate::executors::cortical_mapping_definition_executors::NonPlasticCorticalMappingDefinitionExecutor;
use crate::neuron::flags::NeuronFlag;
use crate::quantizables::{NPUNeuronIndex, PSPMultiplier, SynapseBundleIndex, SynapseCount, SynapticWeight};
use crate::synapse::base_traits::{BaseSynapseAllocStorageTrait, BaseSynapseStorageTrait};
use crate::synapse::dimension_to_dimension_traits::Dim2DimSynapseStaticStorageTrait;
use crate::synapse::feagi_npu_synapse_error::FeagiNPUSynapseError;
use crate::synapse::non_plastic_dimensional::NonPlasticSynapseFull;

pub trait NonplasticSynapseBaseStorageTrait<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant> :
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
    const DEFAULT_SYNAPSE_WEIGHT: SynapticWeight<ValueQuant> = SynapticWeight::ONE;
    const DEFAULT_SYNAPSE_PSP: PSPMultiplier<ValueQuant> = PSPMultiplier::ONE;

    //region Get Connections


    fn get_nonplastic_synapse_data_from_source_neuron_index(&self, source_neuron_index: NPUNeuronIndex<NeuronIndexQuant>) -> Result<(impl Iterator<Item=&NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>>, NeuronCount<NeuronIndexQuant>), FeagiNPUSynapseError>;

    fn get_nonplastic_synapse_data_from_source_neuron_index_mut(&mut self, source_neuron_index: NPUNeuronIndex<NeuronIndexQuant>) -> Result<(impl Iterator<Item=&mut NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>>, NeuronCount<NeuronIndexQuant>), FeagiNPUSynapseError>;

    fn get_nonplastic_synapse_data_from_destination_neuron_index(&self, destination_neuron_index: NPUNeuronIndex<NeuronIndexQuant>) -> Result<(impl Iterator<Item=&NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>>, NeuronCount<NeuronIndexQuant>), FeagiNPUSynapseError>;

    fn get_nonplastic_synapse_data_from_destination_neuron_index_mut(&mut self, destination_neuron_index: NPUNeuronIndex<NeuronIndexQuant>) -> Result<(impl Iterator<Item=&mut NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>>, NeuronCount<NeuronIndexQuant>), FeagiNPUSynapseError>;


    //endregion




}


pub trait NonplasticSynapseStaticStorageTrait<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant> :
Dim2DimSynapseStaticStorageTrait<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
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

pub trait NonplasticSynapseAllocStorageTrait<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant> :
BaseSynapseAllocStorageTrait<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
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
    fn add_synapses_mapping_between_cortical_areas(&mut self,
                                                   source_area_index: CorticalAreaIndex<CorticalIndexQuant>,
                                                   source_neuron_indexes: core::ops::Range<NPUNeuronIndex<NeuronIndexQuant>>,
                                                   source_neuron_flags: &[NeuronFlag],
                                                   source_cortical_dimensions: &NeuronVoxelDimensions<CoordQuant>,
                                                   source_neuron_density: NumberNeuronsPerVoxel,
                                                   destination_area_index: CorticalAreaIndex<CorticalIndexQuant>,
                                                   destination_neuron_indexes: core::ops::Range<NPUNeuronIndex<NeuronIndexQuant>>,
                                                   destination_neuron_flags: &[NeuronFlag],
                                                   destination_cortical_dimensions: &NeuronVoxelDimensions<CoordQuant>,
                                                   destination_neuron_density: NumberNeuronsPerVoxel,
                                                   neuron_mapping_executor: &impl NonPlasticCorticalMappingDefinitionExecutor<NeuronIndexQuant, SynapseIndexQuant, CoordQuant, CorticalIndexQuant, BurstDeltaQuant, ValueQuant>) 
        -> Result<SynapseBundleIndex<SynapseBundleIndexQuant>, FeagiNPUSynapseError>;



}