use feagi_structures::base_quantizable::{QuantizableUIntType, QuantizableValueType};
use crate::quantizables::{BurstDelta, NPUNeuronIndex, PSPMultiplier, SynapticWeight};
use crate::synapse::synapse_flags::SynapseFlag;


#[derive(Debug, Copy, Clone)]
pub struct NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType,
{
    pub source_neuron_index: NPUNeuronIndex<NeuronIndexQuant>,
    pub destination_neuron_index: NPUNeuronIndex<NeuronIndexQuant>,
    pub synapse_properties: NonplasticSynapseProperties<ValueQuant, BurstDeltaQuant>,
}

impl<NeuronIndexQuant, BurstDeltaQuant, ValueQuant> NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType,
{
    pub const NUMBER_OF_BYTES: usize = 
        NeuronIndexQuant::NUMBER_OF_BYTES + 
        NeuronIndexQuant::NUMBER_OF_BYTES +
            NonplasticSynapseProperties::<ValueQuant, BurstDeltaQuant>::NUMBER_OF_BYTES;

    
    pub fn is_valid(&self) -> bool {
        self.synapse_properties.is_valid()
    }
    
    
}


/// Defines the properties of a nonplastic synapse
#[derive(Debug, Copy, Clone)]
pub struct NonplasticSynapseProperties<ValueQuant, BurstDeltaQuant> 
where
    BurstDeltaQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType
{
    pub synapse_flag: SynapseFlag,
    pub synapse_weight: SynapticWeight<ValueQuant>,
    pub postsynaptic_potential_multiplier: PSPMultiplier<ValueQuant>,
    pub synaptic_delay: BurstDelta<BurstDeltaQuant>
}

impl<ValueQuant, BurstDeltaQuant> NonplasticSynapseProperties<ValueQuant, BurstDeltaQuant>
where
    BurstDeltaQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType
{
    pub const NUMBER_OF_BYTES: usize = 1 + ValueQuant::NUMBER_OF_BYTES + ValueQuant::NUMBER_OF_BYTES + BurstDeltaQuant::NUMBER_OF_BYTES;
    
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        self.synapse_flag.is_valid()
    }
}