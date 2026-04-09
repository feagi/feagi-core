
// NOTE: I suspect in common use, nonplastic connections also tend to be made in bundles. Lets capitalize on that

// TODO some things should be moved to a higher level trait as we understand other synapse types more

use feagi_structures::base_quantizable::{QuantizablePercentType, QuantizableUIntType, QuantizableValueType};
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use crate::executors::cortical_mapping_definition_executors::NonPlasticCorticalMappingDefinitionExecutor;
use crate::quantizables::{NPUNeuronIndex, PSPMultiplier, SynapseCount, SynapticWeight};
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
    fn get_nonplastic_synapse_data_from_source_neuron_index(&self, source_neuron_index: NPUNeuronIndex<NeuronIndexQuant>) -> Result<&NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>, FeagiNPUSynapseError>;

    fn get_nonplastic_synapse_data_from_destination_neuron_index(&self, destination_neuron_index: NPUNeuronIndex<NeuronIndexQuant>) -> Result<&NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>, FeagiNPUSynapseError>;


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
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType

{
    fn add_synapses_mapping_between_cortical_areas(&mut self,
                                                   source_area_index: CorticalAreaIndex<CorticalIndexQuant>,
                                                   destination_area_index: CorticalAreaIndex<CorticalIndexQuant>,
                                                   neuron_mapping_executor: &impl NonPlasticCorticalMappingDefinitionExecutor<NeuronIndexQuant, SynapseIndexQuant, CoordQuant, CorticalIndexQuant, BurstDeltaQuant, ValueQuant>);



}