
// NOTE: I suspect in common use, nonplastic connections also tend to be made in bundles. Lets capitalize on that

// TODO some things should be moved to a higher level trait as we understand other synapse types more

use feagi_structures::base_quantizable::{QuantizablePercentType, QuantizableUIntType, QuantizableValueType};
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use crate::executors::cortical_mapping_definition_executors::NonPlasticCorticalMappingDefinitionExecutor;
use crate::quantizables::{PSPMultiplier, SynapseCount, SynapticWeight};
use crate::synapse::base_traits::{BaseSynapseAllocStorageTrait, BaseSynapseStorageTrait};
use crate::synapse::dimension_to_dimension_traits::Dim2DimSynapseStaticStorageTrait;

pub trait NonplasticSynapseBaseStorageTrait<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant> :
BaseSynapseStorageTrait<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
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
    const DEFAULT_SYNAPSE_WEIGHT: SynapticWeight<ValueQuant> = SynapticWeight::ONE;
    const DEFAULT_SYNAPSE_PSP: PSPMultiplier<ValueQuant> = PSPMultiplier::ONE;


    // TODO how should we iterate this?

}


pub trait NonplasticSynapseStaticStorageTrait<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant> :
Dim2DimSynapseStaticStorageTrait<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
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
    const DEFAULT_SYNAPSE_WEIGHT: SynapticWeight<ValueQuant> = SynapticWeight::ONE;
    const DEFAULT_SYNAPSE_PSP: PSPMultiplier<ValueQuant> = PSPMultiplier::ONE;


    // TODO how should we iterate this?

}

pub trait NonplasticSynapseAllocStorageTrait<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant> :
BaseSynapseAllocStorageTrait<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
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
                                                   number_of_synapses: SynapseCount<SynapseIndexQuant>, 
                                                   neuron_mapping_executor: &impl NonPlasticCorticalMappingDefinitionExecutor<NeuronIndexQuant, SynapseIndexQuant, CoordQuant, CorticalIndexQuant, BurstDeltaQuant, ValueQuant>);
    
    

}