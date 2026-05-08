use crate::neuron_voxel_collections::voxel_structs::{NeuronVoxelCoordinate, NeuronVoxelDimensions, NeuronVoxelIndexCount};
use crate::neuron_collections::neuron_structs::{NeuronDensityPerVoxel, NeuronIndexCount, NeuronMembranePotential, SingleCorticalNeuronCollectionType};
use crate::quantization_level::CorticalAreaNeuronQuantization;

pub trait SingleCorticalNeuronCollectionBase<CANQ: CorticalAreaNeuronQuantization>
{
    const COLLECTION_TYPE: SingleCorticalNeuronCollectionType;
    
    fn get_neuron_voxel_density(&self) -> NeuronDensityPerVoxel;
    
    fn is_single_neuron_per_voxel(&self) -> bool;

    fn get_representing_cortical_area_dimensions(&self) -> &NeuronVoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>;

    /// What is the upper bound (exclusive) neuron  index allowed?
    fn get_neuron_max_index(&self) -> NeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>;
    fn number_neurons(&self) -> NeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>;

    fn number_voxels(&self) -> NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>;

    fn iter_index(&self) -> impl Iterator<Item=(NeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronMembranePotential<CANQ::NeuronValueQuant>)>;

    #[cfg(feature = "rayon")]
    fn iter_index_par(&self) -> impl Iterator<Item=(NeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronMembranePotential<CANQ::NeuronValueQuant>)>;

    /// Iterate over non-zero potential values by neuron index
    fn iter_nonzero_potential_index(&self) -> impl Iterator<Item=(NeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronMembranePotential<CANQ::NeuronValueQuant>)>;

    #[cfg(feature = "rayon")]
    fn iter_nonzero_potential_index_par(&self) -> impl Iterator<Item=(NeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronMembranePotential<CANQ::NeuronValueQuant>)>;


    fn iter_coordinate(&self) -> impl Iterator<Item=(NeuronVoxelCoordinate<CANQ::NeuronIndexVoxelCountQuant>, NeuronMembranePotential<CANQ::NeuronValueQuant>)>;

    #[cfg(feature = "rayon")]
    fn iter_coordinate_par(&self) -> impl Iterator<Item=(NeuronVoxelCoordinate<CANQ::NeuronIndexVoxelCountQuant>, NeuronMembranePotential<CANQ::NeuronValueQuant>)>;

    /// Iterate over non-zero potential values by neuron index
    fn iter_nonzero_potential_coordinate(&self) -> impl Iterator<Item=(NeuronVoxelCoordinate<CANQ::NeuronIndexVoxelCountQuant>, NeuronMembranePotential<CANQ::NeuronValueQuant>)>;

    #[cfg(feature = "rayon")]
    fn iter_nonzero_potential_coordinate_par(&self) -> impl Iterator<Item=(NeuronVoxelCoordinate<CANQ::NeuronIndexVoxelCountQuant>, NeuronMembranePotential<CANQ::NeuronValueQuant>)>;
}

pub trait SingleCorticalNeuronCollectionDense<CANQ: CorticalAreaNeuronQuantization>:
SingleCorticalNeuronCollectionBase<CANQ>
{
    fn get_all_neuron_potentials(&self) -> &[NeuronMembranePotential<CANQ::NeuronValueQuant>];

    fn get_all_neuron_potentials_mut(&mut self) -> &mut [NeuronMembranePotential<CANQ::NeuronValueQuant>];

    /// Iterate over slices of neurons that would compose a voxel (slice len = density)
    fn iter_voxel_neuron_slice(&self) -> impl Iterator<Item=(&[NeuronMembranePotential<CANQ::NeuronValueQuant>])>;

    #[cfg(feature = "rayon")]
    fn iter_voxel_neuron_slice_par(&self) -> impl Iterator<Item=(&[NeuronMembranePotential<CANQ::NeuronValueQuant>])>;
}