use feagi_structures::base_quantizable::{QuantizableUIntType, QuantizableValueType};
use crate::CorticalTypedNeuronIndex;
use crate::quantizables::{BurstDelta, NPUGlobalQuantization, NPUSynapseQuantization, PSPMultiplier, SynapticWeight};
use crate::synapse::SynapseFlag;

#[derive(Debug, Copy, Clone)]
pub struct NonPlasticSynapseFull<Q: NPUGlobalQuantization, S: NPUSynapseQuantization>
{
    pub synapse_flag: SynapseFlag,
    pub synapse_weight: SynapticWeight<S::SynapseIndexCountQuant>,
    pub postsynaptic_potential_multiplier: PSPMultiplier<ValueQuant>,
    pub synaptic_delay: BurstDelta<BurstDeltaQuant>
    pub source_neuron_index: CorticalTypedNeuronIndex<NeuronIndexQuant>,
    pub destination_neuron_index: CorticalTypedNeuronIndex<NeuronIndexQuant>,
}

impl<NeuronIndexQuant, BurstDeltaQuant, ValueQuant> crate::synapse::non_plastic_dimensional::NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType,
{
    pub const NUMBER_OF_BYTES: usize =
        NeuronIndexQuant::NUMBER_OF_BYTES +
            NeuronIndexQuant::NUMBER_OF_BYTES +
            crate::synapse::non_plastic_dimensional::NonplasticSynapseProperties::<ValueQuant, BurstDeltaQuant>::NUMBER_OF_BYTES;


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

impl<ValueQuant, BurstDeltaQuant> crate::synapse::non_plastic_dimensional::NonplasticSynapseProperties<ValueQuant, BurstDeltaQuant>
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