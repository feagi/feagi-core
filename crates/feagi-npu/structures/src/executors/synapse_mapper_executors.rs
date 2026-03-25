


// TODO we may need a shared neuron flag, not neuron specific ones

///
pub trait SynapseMapperNonplasticExecutor<NeuronIndex, CoordQuant> {

    // Neuron order to be incrementing x->y->z
    fn map_dimensional_neurons(source_neuron_indexes: Range<NeuronIndex>,
                          destination_neuron_indexes: Range<NeuronIndex>,
                          neuron_flags: &[InterneuronFlag],
                          cortical_dimensions: &NeuronVoxelDimensions<CoordQuant>,
                          synapse_destination: &mut Impl<NonplasticSynapseAllocStorageTrait>)
                          -> Result<(), FeagiNPUError>;
}


// EXAMPLE
struct last_to_first<NeuronIndex, CoordQuant> {
    offset_from_start: NeuronIndex
}

impl last_to_first {

    pub fn new(offset_from_start: NeuronIndex) -> last_to_first {
        last_to_first{offset_from_start}
    }

}

impl SynapseMapperNonplasticExecutor for last_to_first {
    fn map_dimensional_neurons(source_neuron_indexes: Range<NeuronIndex>,
                               destination_neuron_indexes: Range<NeuronIndex>,
                               neuron_flags: &[InterneuronFlag],
                               cortical_dimensions: &NeuronVoxelDimensions<CoordQuant>,
                               synapse_destination: &mut Impl<NonplasticSynapseAllocStorageTrait>)
                               -> Result<(), FeagiNPUError> {

        let source_area_last_index: NeuronIndex = source_neuron_indexes.end;
        let destination_area_first_index: NeuronIndex = destination_neuron_indexes.start + self.offset_from_start;


        synapse_destination.create_spanned_synapse_connections(
            &[source_area_last_index],
            FeagiNPUType::Interneuron,
            &[destination_area_first_index],
            FeagiNPUType::Interneuron,
        );
        Ok(())
    }
}