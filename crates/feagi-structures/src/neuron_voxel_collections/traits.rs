use crate::genomic::cortical_area::CorticalID;
use crate::neuron_voxel_collections::voxel_structs::{NeuronVoxelCoordinate, NeuronVoxelDimensions, NeuronVoxelIndexCount, NeuronVoxelPotential, SingleCorticalNeuronVoxelCollectionType};
use crate::base_feagi_types::quantizable_types::{FeagiBaseQuantizationType, QuantizableUIntType, QuantizableValueType};
use crate::neuron_voxel_collections::FeagiStructuresNeuronVoxelError;

#[cfg(feature = "alloc")]
use ahash::AHashMap;
use crate::quantization_level::CorticalAreaNeuronQuantization;

//region NeuronVoxel
/// Represents the potential of a single voxel (which may contain one or more neuron_collections)
pub trait NeuronVoxel<CANQ: CorticalAreaNeuronQuantization>
{
    const NUMBER_OF_BYTES: usize = CANQ::NeuronValueQuant::NUMBER_OF_BYTES;

    fn get_voxel_potential(&self) -> NeuronVoxelPotential<CANQ::NeuronValueQuant>;

    fn get_voxel_potential_ref(&self) -> &NeuronVoxelPotential<CANQ::NeuronValueQuant>;

    fn set_voxel_potential_ref_mut(&mut self) -> &mut NeuronVoxelPotential<CANQ::NeuronValueQuant>;

    fn set_voxel_potential(&mut self, potential: NeuronVoxelPotential<CANQ::NeuronValueQuant>);

}

//endregion

//region SingleCACollection

/// Defines any collection of neuron_collections sparsely from a single cortical area
pub trait SingleCorticalNeuronVoxelCollectionBase<CANQ: CorticalAreaNeuronQuantization>
{
    // NOTE since neuron collections may be stored in different ways, I see no good way to
    // expose a common interface for getting out potential data efficiently

    const COLLECTION_TYPE: SingleCorticalNeuronVoxelCollectionType;

    /// Returns the dimensions of the cortical area this collection is storing neuron voxel data for
    fn get_representing_cortical_area_dimensions(&self) -> &NeuronVoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>;

    /// What is the upper bound (exclusive) neuron voxel index allowed?
    fn get_neuron_voxel_max_index(&self) -> NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>;

    fn iter_index(&self) -> impl Iterator<Item=(NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)>;

    #[cfg(feature = "rayon")]
    fn iter_index_par(&self) -> impl Iterator<Item=(NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)>;

    /// Iterate over non-zero potential values by neuron index
    fn iter_nonzero_potential_index(&self) -> impl Iterator<Item=(NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)>;

    // TODO par iterators for nonzero potentials?
}


// TODO should we have a function to count the number of nonzero potentials specifically?
/// Defines a collection of neuron_collections of a single cortical area backed by dynamic data structures
/// (Vector)
#[cfg(feature = "alloc")]
pub trait SingleCorticalNeuronVoxelCollectionAlloc<CANQ: CorticalAreaNeuronQuantization>:
SingleCorticalNeuronVoxelCollectionBase<CANQ>
{

    /// Returns the number of neuron voxels stored in the structure
    fn get_number_neuron_voxel_contained_count(&self) -> NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>;

    fn get_neuron_voxel_count_allocated_capacity(&self) -> NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>;

    fn reserve(&mut self, number_of_additional_voxels_to_reserve_for: NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>);

    /// Clears / zeros all stored neuron voxels (without deallocating) and changes cortical area size
    fn empty_and_change_cortical_area_dimensions(&mut self, new_dimensions: NeuronVoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>);

    fn shrink_to_fit(&mut self);
}

// NOTE: Dont need a "static" trait variant since only 1 struct is static

#[cfg(feature = "alloc")]
pub trait SingleCorticalNeuronVoxelCollectionSparse<CANQ: CorticalAreaNeuronQuantization>:
SingleCorticalNeuronVoxelCollectionBase<CANQ> +
SingleCorticalNeuronVoxelCollectionAlloc<CANQ>
{
    /// Returns true if the array is sorted by increasing index / xyz coordinate
    fn is_sorted(&self) -> bool;
    
    /// Clears all stored neuron_collections (without deallocating)
    fn clear_all_neurons(&mut self);
    

    /// Sort by increasing index / xyz coordinate
    fn sort(&mut self);
}

pub trait SingleCorticalNeuronVoxelCollectionDense<CANQ: CorticalAreaNeuronQuantization>:
SingleCorticalNeuronVoxelCollectionBase<CANQ>
{
    /// Returns all neuron voxel potentials as a slice
    fn get_all_neuron_voxel_potentials(&self) -> &[NeuronVoxelPotential<CANQ::NeuronValueQuant>];

    /// Returns all neuron voxel potentials as a mutable slice
    fn get_all_neuron_voxel_potentials_mut(&mut self) -> &mut [NeuronVoxelPotential<CANQ::NeuronValueQuant>];

    
    fn zero_all_neuron_voxel_potentials(&mut self);

    #[cfg(feature = "alloc")]
    fn inplace_overwrite_data_from_sparse(&mut self, sparse_neurons: &impl SingleCorticalNeuronVoxelCollectionSparse<CANQ>, zero_out_first: bool);
}


//endregion

//region MultiCACollection
pub trait MultiCorticalNeuronVoxelCollectionBase<CANQ: CorticalAreaNeuronQuantization>
{
    fn get_contained_cortical_collection_type(&self, cortical_id: &CorticalID) -> Result<&SingleCorticalNeuronVoxelCollectionType, FeagiStructuresNeuronVoxelError>;

    fn get_contained_cortical_area_ids(&self) -> &[CorticalID];

    /// Only gets the base implementation, you probably should NOT use this as it doesn't allow
    /// access to more specialized performant functions
    fn get_base_collection_implementation(&self, cortical_id: &CorticalID) ->
                                                                           Result<&impl SingleCorticalNeuronVoxelCollectionBase<CANQ>, FeagiStructuresNeuronVoxelError>;

    fn get_base_collection_implementation_mut(&mut self, cortical_id: &CorticalID) ->
                                                                               Result<&mut impl SingleCorticalNeuronVoxelCollectionBase<CANQ>, FeagiStructuresNeuronVoxelError>;

}

pub trait MultiCorticalNeuronVoxelCollectionDense<CANQ: CorticalAreaNeuronQuantization>:
MultiCorticalNeuronVoxelCollectionBase<CANQ>
{
    fn get_dense_collection_implementation(&self, cortical_id: &CorticalID) -> Result<&impl SingleCorticalNeuronVoxelCollectionDense<CANQ>, FeagiStructuresNeuronVoxelError>;

    fn get_dense_collection_implementation_mut(&mut self, cortical_id: &CorticalID) -> Result<&mut impl SingleCorticalNeuronVoxelCollectionDense<CANQ>, FeagiStructuresNeuronVoxelError>;
}

#[cfg(feature = "alloc")]
pub trait MultiCorticalNeuronVoxelCollectionAlloc<CANQ: CorticalAreaNeuronQuantization>:
MultiCorticalNeuronVoxelCollectionBase<CANQ>
{
    // NOTE: Not practical to do any sort of data retrieval functions here, but we can do housekeeping

    fn get_contained_cortical_collection_types(&self) -> &AHashMap<CorticalID, SingleCorticalNeuronVoxelCollectionType>;

    // NOTE: Adding must be handled by specific implementations

    fn remove_by_cortical_id(&mut self, cortical_id: &CorticalID) -> Result<(), FeagiStructuresNeuronVoxelError>;

}

//endregion


// NOTE: The mixed type is also alone so it doesn't need a trait either

