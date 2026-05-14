use crate::neuron::neuron_collections::common_neuron_structs::{IndividualNeuronIndexCount, NeuronCollectionType, NeuronDensityPerVoxel, NeuronPotentialType, NeuronVoxelDimensions, NeuronVoxelIndexCount, NeuronVoxelMultiPotentialCalculationMethod, NeuronVoxelPotential};
use crate::quantization_level::CorticalAreaNeuronQuantization;

pub trait NeuronCollectionBase<CANQ: CorticalAreaNeuronQuantization> {
    const COLLECTION_TYPE: NeuronCollectionType;
    const NEURON_DATA_TYPE: NeuronPotentialType;

    /// Returns the dimensions of the cortical area this collection is storing neuron voxel data for
    fn get_representing_cortical_area_voxel_dimensions(&self) -> &NeuronVoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>;

    fn is_single_neuron_per_voxel(&self) -> bool;

    fn get_neuron_voxel_density(&self) -> NeuronDensityPerVoxel;

    /// What is the upper bound (exclusive) neuron  index allowed?
    fn get_neuron_value_max_index(&self) -> IndividualNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>;
    
    /// What is the upper bound (exclusive) neuron voxel index allowed?
    fn get_neuron_voxel_max_index(&self) -> NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>;

    /// In sparse implementations, then number of neurons contained will be less than the max possible index
    fn get_number_contained_neuron_values(&self) -> IndividualNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>;

    /// In sparse implementations, then number of neurons contained will be less than the max possible index
    fn get_number_contained_neuron_voxels(&self) -> NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>;

    fn iter_voxel_index(&self, voxel_potential_method: NeuronVoxelMultiPotentialCalculationMethod) 
        -> impl Iterator<Item=(NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)>;

    #[cfg(feature = "rayon")]
    fn iter_voxel_index_par(&self, voxel_potential_method: NeuronVoxelMultiPotentialCalculationMethod) 
        -> impl rayon::iter::ParallelIterator<Item=(NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)>;

    fn iter_voxel_index_nonzero_potential(&self, voxel_potential_method: NeuronVoxelMultiPotentialCalculationMethod) 
        -> impl Iterator<Item=(NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)>;

    #[cfg(feature = "rayon")]
    fn iter_voxel_index_nonzero_potential_par(&self, voxel_potential_method: NeuronVoxelMultiPotentialCalculationMethod) 
        -> impl rayon::iter::ParallelIterator<Item=(NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)>;

    fn iter_voxel_coordinate(&self, voxel_potential_method: NeuronVoxelMultiPotentialCalculationMethod) 
        -> impl Iterator<Item=( NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)>;

    #[cfg(feature = "rayon")]
    fn iter_voxel_coordinate_par(&self, voxel_potential_method: NeuronVoxelMultiPotentialCalculationMethod) 
        -> impl rayon::iter::ParallelIterator<Item=(NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)>;

    fn iter_voxel_coordinate_nonzero_potential(&self, voxel_potential_method: NeuronVoxelMultiPotentialCalculationMethod) 
        -> impl Iterator<Item=(NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)>;

    #[cfg(feature = "rayon")]
    fn iter_voxel_coordinate_nonzero_potential_par(&self, voxel_potential_method: NeuronVoxelMultiPotentialCalculationMethod) 
        -> impl rayon::iter::ParallelIterator<Item=(NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)>;


    // TODO should we have a function to count the number of nonzero potentials specifically?

}