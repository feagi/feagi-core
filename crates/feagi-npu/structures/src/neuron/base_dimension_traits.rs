
use core::ops::Range;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::NumberNeuronsPerVoxel;
use crate::neuron::base_traits::{BaseNeuronAllocStorageTrait, BaseNeuronStaticStorageTrait};
use crate::neuron::FeagiNPUNeuronError;
use crate::quantizables::{NPUQuantization, NPUNeuronIndex};

pub trait DimensionalStaticStorageTrait<Q: NPUQuantization>:
BaseNeuronStaticStorageTrait<Q>
{

}




#[cfg(feature = "alloc")]
pub trait DimensionalAllocStorageTrait<Q: NPUQuantization>:
BaseNeuronAllocStorageTrait<Q> +
DimensionalStaticStorageTrait<Q>
// % synaptic attractivity
{
    /// Creates a cortical area of given dimensions and neuron density,
    /// and returns its cortical area index and range of neuron indexes it covers
    fn create_cortical_area_with_default_neurons(&mut self,
                                                 cortical_area_dimensions: NeuronVoxelDimensions<Q::Coord>,
                                                 neurons_per_voxel: NumberNeuronsPerVoxel)
                                                 -> Result<(CorticalAreaIndex<Q::CorticalIndex>), FeagiNPUNeuronError>;


    /// Effectively deletes a cortical area (by invalidating their neurons), then rebuilds it to the
    /// new given dimensions and density. While cortical properties are preserved, neuron data is
    /// reset to default. Returns a tuple of the old invalid neuron index range, and the new
    /// created neuron index range.
    fn resize_cortical_area_with_default_neurons(&mut self,
                                                 cortical_area_dimensions: NeuronVoxelDimensions<Q::Coord>,
                                                 neurons_per_voxel: NumberNeuronsPerVoxel,
                                                 cortical_index: CorticalAreaIndex<Q::CorticalIndex>)
                                                 -> Result<(Range<NPUNeuronIndex<Q::NeuronIndex>>), FeagiNPUNeuronError>;

}
