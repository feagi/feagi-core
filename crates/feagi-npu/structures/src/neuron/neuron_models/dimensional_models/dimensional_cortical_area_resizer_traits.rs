use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NumberNeuronsPerVoxel};
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::neuron_models::dimensional_models::dimensional_neuron_data_traits::DimensionalNeuronModelDataSharedTrait;
use crate::quantizables::{NPUDimensionalNeuronQuantization, NPUGlobalQuantization};

#[cfg(feature = "alloc")]
pub trait DimensionalCorticalAreaResizerTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> {

    /// The type of neuron model / cortical data that is generated
    type DimensionNeuronModelType: DimensionalNeuronModelDataSharedTrait<Q, DNQ>;

    /// New intended dimensions
    fn new_dimensions(&self) -> NeuronVoxelDimensions<DNQ::CoordQuant>;

    /// New intended voxel neuron density
    fn new_neuron_per_voxel_density(&self) -> NeuronCount<NumberNeuronsPerVoxel>;

    /// Function to resize cortical area. Be sure to call
    /// "resize_neuron_data_vectors_for_new_dimensions" to actually resize the data structures
    /// Inside the given type
    fn resize_cortical_area_ram(&self, current_dimensions: NeuronVoxelDimensions<DNQ::CoordQuant>,
                            current_per_voxel_density: NeuronCount<NumberNeuronsPerVoxel>,
                            cortical_area_to_resize: &mut Self::DimensionNeuronModelType
    ) -> Result<(), FeagiNPUNeuronError>;

    /// Returns the number of neurons that the cortical area will have following the resize
    fn number_of_neurons_following_resize(&self) -> NeuronCount<Q::NeuronIndexCountQuant> {
        self.new_dimensions().get_number_neurons(&self.new_neuron_per_voxel_density())
    }

    // TODO feature gated cortical area resizing for other hardware implementations!
}