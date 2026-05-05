

use crate::genomic::cortical_area::CorticalID;
use crate::neuron_voxels::descriptors::{NeuronVoxelCoordinate, NeuronVoxelDimensions, NeuronVoxelPotential, SingleCorticalNeuronVoxelCollectionType};
use crate::quantization::{QuantizableUIntType, QuantizableValueType};
use crate::neuron_voxels::FeagiStructuresNeuronVoxelError;

#[cfg(feature = "alloc")]
use ahash::AHashMap;


//region NeuronVoxel
/// Represents the potential of a single voxel (which may contain one or more neurons)
pub trait NeuronVoxel<VoxelPotentialQuant> where
    VoxelPotentialQuant: QuantizableValueType
{
    const NUMBER_OF_BYTES: usize;

    fn get_voxel_potential(&self) -> NeuronVoxelPotential<VoxelPotentialQuant>;

    fn get_voxel_potential_ref(&self) -> &NeuronVoxelPotential<VoxelPotentialQuant>;

    fn set_voxel_potential_ref_mut(&mut self) -> &mut NeuronVoxelPotential<VoxelPotentialQuant>;

    fn set_voxel_potential(&mut self, potential: NeuronVoxelPotential<VoxelPotentialQuant>);

}

//endregion

//region SingleCACollection

/// Defines any collection of neurons sparsely from a single cortical area
pub trait SingleCorticalNeuronVoxelCollectionBase<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant> where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    // NOTE since neuron collections may be stored in different ways, I see no good way to
    // expose a common interface for getting out potential data efficiently

    const COLLECTION_TYPE: SingleCorticalNeuronVoxelCollectionType;

    /// Returns the dimensions of the cortical area this collection is storing neuron voxel data for
    fn get_representing_cortical_area_dimensions(&self) -> &NeuronVoxelDimensions<CoordQuant>;

    /// What is the upper bound (exclusive) neuron voxel index allowed?
    fn neuron_index_max_limit(&self) -> NeuronVoxelIndexQuant;


    //region Neurons

    // TODO these functions
    /*
    fn try_get_neuron_voxel_index(&mut self, index: &NeuronVoxelIndexQuant) -> Option<NeuronVoxelPotential<VoxelPotentialQuant>>;

    fn try_get_neuron_voxel_coordinate(&mut self, coordinate: &NeuronVoxelDimensions<CoordQuant>) -> Option<&NeuronVoxelDimensions<CoordQuant>>;

    fn write_neuron_voxel_index(&mut self, voxel: NeuronVoxelIP<NeuronVoxelPotential<VoxelPotentialQuant>, NeuronVoxelIndexQuant>) -> Result<(), crate::neuron_voxels::FeagiNeuronVoxelError>;

    fn write_neuron_voxel_coordinate(&mut self, voxel: NeuronVoxelXYZP<NeuronVoxelPotential<VoxelPotentialQuant>, CoordQuant>) -> Result<(), crate::neuron_voxels::FeagiNeuronVoxelError>;

    fn write_neuron_voxel_index_raw(&mut self, index: &NeuronVoxelIndexQuant, voxel_potential: NeuronVoxelPotential<Potential>) -> Result<(), crate::neuron_voxels::FeagiNeuronVoxelError>;

    fn write_neuron_voxel_coordinate_raw(&mut self, x: CoordQuant, y: CoordQuant, z: CoordQuant, voxel_potential: NeuronVoxelPotential<Potential>) -> Result<(), crate::neuron_voxels::FeagiNeuronVoxelError>;

    fn write_neuron_voxel_index_unchecked(&mut self, voxel: NeuronVoxelIP<NeuronVoxelPotential<VoxelPotentialQuant>, NeuronVoxelIndexQuant>) -> Result<(), crate::neuron_voxels::FeagiNeuronVoxelError>;

    fn write_neuron_voxel_coordinate_unchecked(&mut self, voxel: NeuronVoxelXYZP<NeuronVoxelPotential<VoxelPotentialQuant>, CoordQuant>) -> Result<(), crate::neuron_voxels::FeagiNeuronVoxelError>;

    fn write_neuron_voxel_index_raw_unchecked(&mut self, index: &NeuronVoxelIndexQuant, voxel_potential: NeuronVoxelPotential<Potential>) -> Result<(), crate::neuron_voxels::FeagiNeuronVoxelError>;

    fn write_neuron_voxel_coordinate_raw_unchecked(&mut self, x: CoordQuant, y: CoordQuant, z: CoordQuant, voxel_potential: NeuronVoxelPotential<Potential>) -> Result<(), crate::neuron_voxels::FeagiNeuronVoxelError>;

     */
    //endregion

}


// TODO should we have a function to count the number of nonzero potentials specifically?
/// Defines a collection of neurons of a single cortical area backed by dynamic data structures
/// (Vector)
#[cfg(feature = "alloc")]
pub trait SingleCorticalNeuronVoxelCollectionAlloc<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>:
SingleCorticalNeuronVoxelCollectionBase<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant> where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{

    /// Returns the number of neuron voxels stored in the structure
    fn get_number_neuron_voxel_contained_count(&self) -> NeuronVoxelIndexQuant;

    fn get_neuron_voxel_count_allocated_capacity(&self) -> NeuronVoxelIndexQuant;

    fn reserve(&mut self, number_of_neuron_voxels_to_reserve_for: NeuronVoxelIndexQuant);

    /// Clears / zeros all stored neurons (without deallocating) and changes cortical area size
    fn empty_and_change_cortical_area_dimensions(&mut self, new_dimensions: NeuronVoxelDimensions<CoordQuant>);

    fn shrink_to_fit(&mut self);
}

// NOTE: Dont need a "static" trait variant since only 1 struct is static

#[cfg(feature = "alloc")]
pub trait SingleCorticalNeuronVoxelCollectionSparse<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>:
SingleCorticalNeuronVoxelCollectionBase<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant> +
SingleCorticalNeuronVoxelCollectionAlloc<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant> where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    /// Clears all stored neurons (without deallocating)
    fn clear_all_neurons(&mut self);

    fn iter_index(&self) -> impl Iterator<Item=(NeuronVoxelIndexQuant, NeuronVoxelPotential<VoxelPotentialQuant>)>;

    fn iter_coordinate(&self) -> impl Iterator<Item=(NeuronVoxelCoordinate<CoordQuant>, NeuronVoxelPotential<VoxelPotentialQuant>)>;

    /// Sort by increasing index / xyz coordinate
    fn sort(&mut self);

    #[cfg(feature = "rayon")]
    fn iter_index_par(&self) -> impl Iterator<Item=(NeuronVoxelIndexQuant, NeuronVoxelPotential<VoxelPotentialQuant>)>;

    #[cfg(feature = "rayon")]
    fn iter_coordinate_par(&self) -> impl Iterator<Item=(NeuronVoxelCoordinate<CoordQuant>, NeuronVoxelPotential<VoxelPotentialQuant>)>;
}

pub trait SingleCorticalNeuronVoxelCollectionDense<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>:
SingleCorticalNeuronVoxelCollectionBase<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant> where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    fn get_all_neuron_voxel_potentials(&self) -> &[NeuronVoxelPotential<VoxelPotentialQuant>];

    fn get_all_neuron_voxel_potentials_mut(&mut self) -> &mut [NeuronVoxelPotential<VoxelPotentialQuant>];

    /// Iterate over non-zero potential values by neuron index
    fn iter_nonzero_index(&self) -> impl Iterator<Item=(NeuronVoxelIndexQuant, NeuronVoxelPotential<VoxelPotentialQuant>)>;

    /// Iterate over non-zero potential values by neuron coordinate
    fn iter_nonzero_coordinate(&self) -> impl Iterator<Item=(NeuronVoxelCoordinate<CoordQuant>, NeuronVoxelPotential<VoxelPotentialQuant>)>;

    // TODO par iterators for nonzero potentials?

    fn zero_all_neuron_voxel_potentials(&mut self);

    #[cfg(feature = "alloc")]
    fn inplace_overwrite_data_from_sparse(&mut self, sparse_neurons: &impl SingleCorticalNeuronVoxelCollectionSparse<NeuronVoxelPotential<VoxelPotentialQuant>, CoordQuant, NeuronVoxelIndexQuant>);
}


//endregion

//region MultiCACollection
pub trait MultiCorticalNeuronVoxelCollectionBase<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, CorticalAreaIndexQuant> where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType,
    CorticalAreaIndexQuant: QuantizableUIntType
{
    fn get_contained_cortical_collection_type(&self, cortical_id: &CorticalID) -> Result<&SingleCorticalNeuronVoxelCollectionType, FeagiStructuresNeuronVoxelError>;

    fn get_contained_cortical_area_ids(&self) -> &[CorticalID];

    /// Only gets the base implementation, you probably should NOT use this as it doesn't allow
    /// access to more specialized performant functions
    fn get_base_collection_implementation(&self, cortical_id: &CorticalID) ->
                                                                           Result<&impl SingleCorticalNeuronVoxelCollectionBase<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>, FeagiStructuresNeuronVoxelError>;

    fn get_base_collection_implementation_mut(&mut self, cortical_id: &CorticalID) ->
                                                                               Result<&mut impl SingleCorticalNeuronVoxelCollectionBase<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>, FeagiStructuresNeuronVoxelError>;

}

pub trait MultiCorticalNeuronVoxelCollectionDense<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, CorticalAreaIndexQuant>:
MultiCorticalNeuronVoxelCollectionBase<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, CorticalAreaIndexQuant> where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType,
    CorticalAreaIndexQuant: QuantizableUIntType
{
    fn get_dense_collection_implementation(&self, cortical_id: &CorticalID) -> Result<&impl SingleCorticalNeuronVoxelCollectionDense<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>, FeagiStructuresNeuronVoxelError>;

    fn get_dense_collection_implementation_mut(&mut self, cortical_id: &CorticalID) -> Result<&mut impl SingleCorticalNeuronVoxelCollectionDense<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>, FeagiStructuresNeuronVoxelError>;
}

#[cfg(feature = "alloc")]
pub trait MultiCorticalNeuronVoxelCollectionAlloc<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, CorticalAreaIndexQuant>:
MultiCorticalNeuronVoxelCollectionBase<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, CorticalAreaIndexQuant> where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType,
    CorticalAreaIndexQuant: QuantizableUIntType
{
    // NOTE: Not practical to do any sort of data retrieval functions here, but we can do housekeeping

    fn get_contained_cortical_collection_types(&self) -> &AHashMap<CorticalID, SingleCorticalNeuronVoxelCollectionType>;

    // NOTE: Adding must be handled by specific implementations

    fn remove_by_cortical_id(&mut self, cortical_id: &CorticalID) -> Result<(), FeagiStructuresNeuronVoxelError>;

}

//endregion
// NOTE: The mixed type is also alone so it doesn't need a trait either

