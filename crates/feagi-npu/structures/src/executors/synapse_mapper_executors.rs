


// TODO we may need a shared neuron flag, not neuron specific ones

///
pub trait SynapseMapperNonplasticExecutor<NeuronIndex, CoordQuant> {

    // Neuron order to be incrementing x->y->z
    fn map_dimensional_neurons(source_neuron_indexes: Range<NeuronIndex>,

                          destination_neuron_indexes: Range<NeuronIndex>,
                          neuron_flags: &[InterneuronFlag],
                          cortical_dimensions: &NeuronVoxelDimensions<CoordQuant>)
                          -> Result<(), FeagiNPUError>;
}