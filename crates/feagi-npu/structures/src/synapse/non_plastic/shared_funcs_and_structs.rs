use feagi_structures::base_quantizable::{QuantizableUIntType, QuantizableValueType};
use crate::quantizables::{BurstDelta, NPUNeuronIndex, PSPMultiplier, SynapticWeight};
use crate::synapse::synapse_flags::SynapseFlag;

/// Defines the properties and mapping of a nonplastic synapse. Mostly meant for iterators
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


/// Defines the properties of a nonplastic synapse
pub struct NonplasticSynapseProperties<ValueQuant, BurstDeltaQuant> where
    BurstDeltaQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType
{
    pub synapse_flag: SynapseFlag,
    pub synapse_weight: SynapticWeight<ValueQuant>,
    pub postsynaptic_potential_multiplier: PSPMultiplier<ValueQuant>,
    pub synaptic_delay: BurstDelta<BurstDeltaQuant>

}