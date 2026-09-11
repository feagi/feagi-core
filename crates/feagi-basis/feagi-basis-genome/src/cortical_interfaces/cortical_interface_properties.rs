use feagi_basis_neuron::wrapped_types::CorticalChannelVoxelDimensions;
use feagi_basis_quantization::prelude::GenomeDimensionsQuant;
use crate::spatial::GenomeCoordinate;

// TODO migrate away from this, instead each unit should have a generator struct 

pub struct CorticalInterfaceAreaTopology {
    /// The offset of this area in relation to the cortical interface as a whole
    pub relative_position: GenomeCoordinate,
    /// What starting dimensions should this 
    pub channel_dimensions_default: CorticalChannelVoxelDimensions<GenomeDimensionsQuant>,
    pub channel_dimensions_minimum: CorticalChannelVoxelDimensions<GenomeDimensionsQuant>,
    pub channel_dimensions_maximum: CorticalChannelVoxelDimensions<GenomeDimensionsQuant>
}