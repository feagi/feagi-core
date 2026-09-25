use std::marker::PhantomData;
use crate::cortical_area::data_structs::InverseOutgoingConnectionCount;
use crate::cortical_area::extendable_components::cortical_area_quantization::CorticalAreaQuantization;
use crate::cortical_area::extendable_components::cortical_model_data_field::CorticalModelDataField;
use crate::cortical_area::data_structs::per_neuron_flags::PerNeuronFlags;

pub struct CorticalAreaModelNeuronData<
    CAQ: CorticalAreaQuantization,
    NeuronDataInternal: CorticalModelDataField<CAQ>,
    NeuronDataScratch: CorticalModelDataField<CAQ>
>
{
    /// The per neuron level data that should be saved to the connectome
    pub neuron_data_internal: NeuronDataInternal,
    /// The per neuron level data that should not be saved to the connectome
    /// (starts clean with every init)
    pub neuron_data_scratch: NeuronDataScratch,
    /// Universal flags specifying various neuron firing attributes
    pub neuron_flags: PerNeuronFlags,
    /// The number of outgoing synapse connections from this neuron, inverted
    pub inverse_outgoing_connection_count: InverseOutgoingConnectionCount<CAQ::MembranePotentialQuant>
}