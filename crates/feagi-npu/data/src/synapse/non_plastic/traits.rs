
// NOTE: I suspect in common use, nonplastic connections also tend to be made in bundles. Lets capitalize on that


pub trait NonplasticSynapseStaticStorageTrait<SynapseIndexQuant, NeuronIndexQuant, PercentageQuant, PotentialQuant >: BaseSynapseStaticStorageTrait<SynapseIndexQuant, NeuronIndexQuant, PercentageQuant, PotentialQuant >
where
    SynapseIndexQuant: SynapseIndex,
    NeuronIndexQuant: QuantizableUInt,
    PercentageQuant: PercentageScale,
    PotentialQuant: PotentialUnit,
{
    const DEFAULT_SYNAPSE_WEIGHT: PercentageQuant = PercentageQuant:ONE;
    const DEFAULT_SYNAPSE_PSP: PotentialQuant = PotentialQuant:ONE;

    fn create_spanned_synapse_connections(&mut self, source_neurons_indexes: &[NeuronIndexQuant],
                                          source_neuron_type: NPUNeuronType,
                                          destination_neuron_indexes: &[NeuronIndexQuant],
                                          destination_neuron_type: NPUNeuronType) -> Result<&[SynapseIndexQuant], FeagiNPUDataError> {}



    // TODO how should we iterate this?

}