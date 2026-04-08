

// TODO we may need a shared neuron flag, not neuron specific ones

use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::flags::NeuronFlag;
use crate::quantizables::NPUNeuronIndex;

/// Defines source neurons and destination neurons to use for dimensional synapse mapping.
/// ITERATORS MUST BE OF THE SAME LENGTH!
pub trait SynapseNeuronMapperDim2DimExecutor<NeuronIndexQuant, CoordQuant> {

    // NOTE: Since dimensional cortical areas can return neuron slices of cortical areas,
    // we can pass through entire slices as reference!

    fn source_dimensional_neuron_iterator<'a>(source_neuron_indexes: core::ops::Range<NPUNeuronIndex<NeuronIndexQuant>>,
                                  source_neuron_flags: &[NeuronFlag],
                                  source_cortical_dimensions: &NeuronVoxelDimensions<CoordQuant>)
        -> Result<impl Iterator<Item=&'a NPUNeuronIndex<NeuronIndexQuant>>, FeagiNPUNeuronError>;

    fn destination_dimensional_neuron_iterator<'a>(destination_neuron_indexes: core::ops::Range<NPUNeuronIndex<NeuronIndexQuant>>,
                                       destination_neuron_flags: &[NeuronFlag],
                                       destination_cortical_dimensions: &NeuronVoxelDimensions<CoordQuant>)
        -> Result<impl Iterator<Item=&'a NPUNeuronIndex<NeuronIndexQuant>>, FeagiNPUNeuronError>;
    
    // TODO rayon support!

}
