

pub trait SynapseData<SynapseIndexQuant, NeuronIndexQuant, PercentageQuant, PotentialQuant >
where
    SynapseIndexQuant: SynapseIndex,
    NeuronIndexQuant: QuantizableUInt, // TODO we need to address the multiple types of neuron indexes
    PercentageQuant: PercentageScale,
    PotentialQuant: PotentialUnit,
{
    fn number_of_active_synapses(&self) -> SynapseIndexAndSize;

    fn get_synapse_indexes(&self) -> &[SynapseIndexQuant];
    fn get_synapse_indexes_mut(&mut self) -> &mut [SynapseIndexQuant];

    fn get_source_neurons(&self) -> &[NeuronIndexQuant];
    fn get_source_neurons_mut(&mut self) -> &mut [NeuronIndexQuant];

    fn get_destination_neurons(&self) -> &[NeuronIndexQuant];
    fn get_destination_neurons_mut(&mut self) -> &mut [NeuronIndexQuant];

    fn get_synapse_weights(&self) -> &[PotentialQuant];
    fn set_synapse_weights(&mut self) -> &mut [PotentialQuant];

    fn get_post_synaptic_potentials(&self) -> &[PotentialQuant];
    fn set_post_synaptic_potentials(&mut self) -> &mut [PotentialQuant];

    fn get_synapse_flags(&self) -> &[SynapseFlag];
    fn set_synapse_flags(&mut self) -> &mut [SynapseFlag];

    // TODO
}