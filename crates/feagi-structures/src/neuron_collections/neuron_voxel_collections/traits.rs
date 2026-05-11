use crate::genomic::cortical_area::CorticalID;
use crate::base_feagi_types::quantizable_types::{FeagiBaseQuantizationType, QuantizableUIntType, QuantizableValueType};
use crate::neuron_collections::neuron_voxel_collections::FeagiStructuresNeuronVoxelError;

#[cfg(feature = "alloc")]
use ahash::AHashMap;
use crate::neuron_collections::base_neuron_collection_traits::NeuronCollectionBase;
use crate::neuron_collections::common_neuron_structs::{NeuronVoxelDimensions, NeuronVoxelIndexCount, NeuronVoxelPotential};
use crate::quantization_level::CorticalAreaNeuronQuantization;

//region NeuronVoxel
/// Represents the potential of a single voxel (which may contain one or more neuron_collections)
pub trait NeuronVoxel<CANQ: CorticalAreaNeuronQuantization>
{
    fn get_voxel_potential(&self) -> NeuronVoxelPotential<CANQ::NeuronValueQuant>;

    fn get_voxel_potential_ref(&self) -> &NeuronVoxelPotential<CANQ::NeuronValueQuant>;

    fn set_voxel_potential_ref_mut(&mut self) -> &mut NeuronVoxelPotential<CANQ::NeuronValueQuant>;

    fn set_voxel_potential(&mut self, potential: NeuronVoxelPotential<CANQ::NeuronValueQuant>);

}

//endregion



/// Defines any collection of neuron_collections sparsely from a single cortical area
pub trait NeuronVoxelCollectionBase<CANQ: CorticalAreaNeuronQuantization>:
NeuronCollectionBase<CANQ>
{

}




#[cfg(feature = "alloc")]
pub trait NeuronVoxelCollectionResizable<CANQ: CorticalAreaNeuronQuantization>:
NeuronVoxelCollectionBase<CANQ>
{
    fn get_neuron_voxel_count_allocated_capacity(&self) -> NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>;

    fn reserve(&mut self, number_of_additional_voxels_to_reserve_for: NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>);

    /// Clears / zeros all stored neuron voxels (without deallocating) and changes cortical area size
    fn empty_and_change_cortical_area_dimensions(&mut self, new_dimensions: NeuronVoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>);

    fn shrink_to_fit(&mut self);
}

// NOTE: Dont need a "static" trait variant since only 1 struct is static

#[cfg(feature = "alloc")]
pub trait NeuronVoxelCollectionSparse<CANQ: CorticalAreaNeuronQuantization>:
NeuronVoxelCollectionBase<CANQ> +
NeuronVoxelCollectionResizable<CANQ>
{
    /// Returns true if the array is sorted by increasing index / xyz coordinate
    fn is_sorted(&self) -> bool;

    /// Clears all stored neuron_collections (without deallocating)
    fn clear_all_neurons(&mut self);


    /// Sort by increasing index / xyz coordinate
    fn sort(&mut self);
}

pub trait SingleCorticalNeuronVoxelCollectionDense<CANQ: CorticalAreaNeuronQuantization>:
NeuronVoxelCollectionBase<CANQ>
{
    /// Returns all neuron voxel potentials as a slice
    fn get_all_neuron_voxel_potentials(&self) -> &[NeuronVoxelPotential<CANQ::NeuronValueQuant>];

    /// Returns all neuron voxel potentials as a mutable slice
    fn get_all_neuron_voxel_potentials_mut(&mut self) -> &mut [NeuronVoxelPotential<CANQ::NeuronValueQuant>];


    fn zero_all_neuron_voxel_potentials(&mut self);

    #[cfg(feature = "alloc")] // sparese requires alloc
    fn inplace_overwrite_data_from_sparse(&mut self, sparse_neurons: &impl NeuronVoxelCollectionSparse<CANQ>, zero_out_first: bool);
}




