use feagi_structures::quantization::{QuantizableUIntType, QuantizableValueType};
use crate::CorticalTypedNeuronIndex;
use crate::quantizables::{BurstDelta, NPUGlobalQuantization, NPUSynapseQuantization, PSPMultiplier, SynapticWeight};
use crate::synapse::non_plastic_dimensional::non_plastic_synapse_flag::NonPlasticSynapseFlag;

#[derive(Debug, Copy, Clone)]
pub struct NonPlasticSynapseFull<Q: NPUGlobalQuantization, S: NPUSynapseQuantization>
{
    pub synapse_flag: NonPlasticSynapseFlag,
    pub synapse_weight: SynapticWeight<S::SynapseValueType>,
    pub postsynaptic_potential_multiplier: PSPMultiplier<S::SynapseValueType>,
    pub synaptic_delay: BurstDelta<S::BurstDelay>,
    pub source_neuron_index: CorticalTypedNeuronIndex<Q::NeuronIndexCountQuant>,
    pub destination_neuron_index: CorticalTypedNeuronIndex<Q::NeuronIndexCountQuant>,
}

impl<Q: NPUGlobalQuantization, S: NPUSynapseQuantization> NonPlasticSynapseFull<Q, S>
{
    pub const NUMBER_OF_BYTES: usize = 0; // TODO

    pub fn is_valid(&self) -> bool {
        self.synapse_flag.is_valid()
    }
}
