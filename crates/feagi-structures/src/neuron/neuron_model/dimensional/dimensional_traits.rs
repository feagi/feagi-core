use crate::neuron::individual_neuron_structs::IndividualNeuronIndexCount;
use crate::neuron::feagi_neuron_error::FeagiNeuronCollectionError;
use crate::neuron::neuron_model::dimensional::dimensional_structs::{NeuronDensityPerVoxel, NeuronVoxelDimensions, NeuronVoxelIndexCount};
use crate::neuron::neuron_model::dimensional::smart_voxel_iterator::VoxelSmartIterator;
use crate::neuron::neuron_model::dimensional::voxel_potential_calculation_method::NeuronVoxelMultiPotentialCalculationMethod;
use crate::neuron::neuron_model::neuron_model_base_traits::NeuronModelContainerBaseTrait;
use crate::quantization_level::CorticalAreaNeuronQuantization;

/// Base trait defining all possible Dimensional (representable by voxels) neuron data collections
pub trait DimensionalNeuronContainerBaseTrait<CANQ: CorticalAreaNeuronQuantization>:
NeuronModelContainerBaseTrait<CANQ>
{
    fn get_neuro_count_voxel_density(&self) -> IndividualNeuronIndexCount<NeuronDensityPerVoxel>;

    fn get_cortical_area_voxel_dimensions(&self) -> &NeuronVoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>;

    fn is_single_neuron_per_voxel(&self) -> bool;

    /// What is the upper bound (exclusive) neuron index allowed?
    fn get_neuron_max_index(&self) -> NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>;
    
    /// What is the upper bound (exclusive) neuron voxel index allowed?
    fn get_neuron_voxel_max_index(&self) -> NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>;

    /// In sparse implementations, then number of neurons contained may be less than the max possible index
    fn get_total_number_neurons(&self) -> NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        self.get_cortical_data().get_total_number_neurons()
    }

    fn get_iterator_neuron_voxel(&self, voxel_potential_method: NeuronVoxelMultiPotentialCalculationMethod)
                        -> impl Iterator<Item=VoxelSmartIterator<CANQ>>;

    // TODO clear / zero all neurons?
}

/// A Dimensional Neuron Container that stores neurons by index in a sparse fashion
pub trait DimensionalNeuronContainerSparseBaseTrait<CANQ: CorticalAreaNeuronQuantization>:
DimensionalNeuronContainerBaseTrait<CANQ>
{
    fn is_sorted_from_smallest_to_largest(&self) -> bool;

    fn sort_self(&mut self) -> Result<(), FeagiNeuronCollectionError>;
    
    // TODO iterate neuron model enumerated
}

/// A Dimensional Neuron Container that stores all neurons in its bounds as a
/// flat ordered collection
pub trait DimensionalNeuronContainerDenseBaseTrait<CANQ: CorticalAreaNeuronQuantization>:
DimensionalNeuronContainerBaseTrait<CANQ>

{
    // TODO iterate neuron model
}

/// A Dimensional Neuron Container that cannot be resized in memory (size known at compile time)
pub trait DimensionalNeuronContainerFixedBaseTrait<CANQ: CorticalAreaNeuronQuantization>:
DimensionalNeuronContainerBaseTrait<CANQ>
{
    // TODO mainly for embedded
}

/// A Dimensional Neuron Container that can allocate additional memory to resize
pub trait DimensionalNeuronContainerResizableBaseTrait<CANQ: CorticalAreaNeuronQuantization>:
DimensionalNeuronContainerBaseTrait<CANQ>
{
    fn resize_neuron_data_vectors_for_new_dimensions(&mut self,
                                                     new_dimensions: NeuronVoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>,
                                                     neurons_per_voxel: IndividualNeuronIndexCount<NeuronDensityPerVoxel>);
}