use crate::neuron::individual_neuron_structs::IndividualNeuronIndexCount;
use crate::neuron::dimensional::dimensional_structs::{NeuronDensityPerVoxel, NeuronVoxelDimensions, NeuronVoxelIndexCount};
use crate::neuron::dimensional::dimensional_enums::{DimensionalNeuronCollectionElementType, NeuronVoxelMultiPotentialCalculationMethod};
use crate::neuron::dimensional::dimensional_voxel_iterating::DimensionalNeuronVoxelSmartIterator;
use crate::neuron::feagi_neuron_error::FeagiNeuronCollectionError;
use crate::neuron::neuron_base_traits::NeuronDataContainerBaseTrait;
use crate::quantization_level::CorticalAreaNeuronQuantization;

/// Base trait defining all possible Dimensional (representable by voxels) neuron data collections
pub trait DimensionalNeuronDataContainerBaseTrait<CANQ: CorticalAreaNeuronQuantization>:
NeuronDataContainerBaseTrait<CANQ>
{
    const DIMENSIONAL_NEURON_COLLECTION_ELEMENT_TYPE: DimensionalNeuronCollectionElementType;
    
    

    fn get_neuron_voxel_density(&self) -> NeuronDensityPerVoxel;

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
                        -> impl Iterator<Item=DimensionalNeuronVoxelSmartIterator<CANQ>>;

    // TODO clear / zero all neurons?
}

/// A Dimensional Neuron Container that stores neurons by index in a sparse fashion
pub trait DimensionalNeuronDataContainerSparseBaseTrait<CANQ: CorticalAreaNeuronQuantization>:
DimensionalNeuronDataContainerBaseTrait<CANQ>
{
    
    
    fn is_sorted_from_smallest_to_largest(&self) -> bool;

    fn sort_self(&mut self) -> Result<(), FeagiNeuronCollectionError>;
    
    // TODO iterate neuron model enumerated
}

/// A Dimensional Neuron Container that stores all neurons in its bounds as a
/// flat ordered collection
pub trait DimensionalNeuronDataContainerDenseBaseTrait<CANQ: CorticalAreaNeuronQuantization>:
DimensionalNeuronDataContainerBaseTrait<CANQ>

{
    // TODO iterate neuron model
}

/// A Dimensional Neuron Container that cannot be resized in memory (size known at compile time)
pub trait DimensionalNeuronDataContainerFixedBaseTrait<CANQ: CorticalAreaNeuronQuantization>:
DimensionalNeuronDataContainerBaseTrait<CANQ>
{
    // TODO mainly for embedded
}

/// A Dimensional Neuron Container that can allocate additional memory to resize
pub trait DimensionalNeuronDataContainerResizableBaseTrait<CANQ: CorticalAreaNeuronQuantization>:
DimensionalNeuronDataContainerBaseTrait<CANQ>
{
    fn resize_neuron_data_vectors_for_new_dimensions(&mut self,
                                                     new_dimensions: NeuronVoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>,
                                                     neurons_per_voxel: NeuronCount<NumberNeuronsPerVoxel>);
}