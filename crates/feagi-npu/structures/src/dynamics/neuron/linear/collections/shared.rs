use feagi_structures::CorticalAreaNeuronQuantization;
use feagi_structures::neuron::{FeagiNeuronError, LinearNeuronIndexCount, NeuronMembranePotential};
use crate::dynamics::neuron::linear::neurons::{NeuronModelNeuronMutRefTrait, NeuronModelNeuronRefTrait, NeuronModelNeuronTrait};


//region Shared Enums

/// Defines if neurons are expected to be grouped together or not
#[derive(Clone, PartialEq)]
pub enum NeuronGroupingType {
    Single,
    Multiple,
}

#[derive(Clone, PartialEq)]
pub enum NeuronPackingType {
    Serial,
    Parallel
}

/// Enum for defining what kind of collection a neuron collection is
#[derive(Clone, PartialEq)]
pub enum NeuronCollectionType {
    PackedFixedArray,
    PackedResizableVector,
    IndexedResizableVector,
    IndexedResizableHashmap,

    // TODO Array, fixed Hashmaps?
    // TODO others? Device support specifics?
}

//endregion

//region Universally Shared (Linear Base)

/// Base trait shared by all neuron model collections, which establishes a fallback universal way
/// to reference, modify, and iterate through any neuron collection structure
pub trait NeuronModelCollectionBaseLinearTrait<CANQ: CorticalAreaNeuronQuantization>
{
    const NEURON_COLLECTION_TYPE: NeuronCollectionType;

    type SingleNeuron: NeuronModelNeuronTrait<CANQ>;
    type SingleNeuronRef: NeuronModelNeuronRefTrait<'static, CANQ>;
    type SingleNeuronMutRef: NeuronModelNeuronMutRefTrait<'static,CANQ>;

    fn is_sorted_in_increasing_index_order(&self) -> bool;

    /// What is the upper bound (exclusive) neuron  index allowed?
    fn get_neuron_max_linear_index(&self) -> LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>;

    /// In sparse implementations, then number of neurons contained may be less than the max possible index
    fn get_number_contained_neurons(&self) -> LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>;

    fn try_get_neuron_data_ref(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<Self::SingleNeuronRef, FeagiNeuronError>;

    fn try_get_neuron_data_ref_mut(&mut self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<&Self::SingleNeuronMutRef, FeagiNeuronError>;

    fn enumerated_linear_neuron_iter(&self) -> impl Iterator<Item = EnumeratedNeuronLinearReference<CANQ,Self::SingleNeuronRef>>;

    fn enumerated_linear_neuron_iter_mut(&self) -> impl Iterator<Item = EnumeratedNeuronLinearReferenceMut<CANQ,Self::SingleNeuronMutRef>>;

    // TODO RAYON iterators

}

//region Iterators
pub struct EnumeratedNeuronLinearReference<'a, CANQ: CorticalAreaNeuronQuantization, NMF: NeuronModelNeuronRefTrait<'a, CANQ>> {
    linear_neuron_index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
    neuron_collection_ref: &'a NMF
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMF: NeuronModelNeuronRefTrait<'a, CANQ>> EnumeratedNeuronLinearReference<'a, CANQ, NMF> {
    pub fn get_linear_index(&self) -> &LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        &self.linear_neuron_index
    }

    pub fn neuron_ref(&self) -> &'a NMF {
        self.neuron_collection_ref
    }
}

pub struct EnumeratedNeuronLinearReferenceMut<'a, CANQ: CorticalAreaNeuronQuantization, NMF: NeuronModelNeuronRefTrait<'a, CANQ>> {
    linear_neuron_index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
    neuron_collection_ref: &'a mut NMF
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMF: NeuronModelNeuronRefTrait<'a, CANQ>> EnumeratedNeuronLinearReferenceMut<'a, CANQ, NMF> {
    pub fn get_linear_index(&self) -> &LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        &self.linear_neuron_index
    }

    pub fn neuron_ref(&self) -> &'a NMF {
        self.neuron_collection_ref
    }

    pub fn neuron_ref_mut(&mut self) -> &'a mut NMF {
        self.neuron_collection_ref
    }
}
//endregion

//endregion

//region Shared Packed

//region Shared Parallel Packed

/// Trait for structs that are densely packed (not sparse)
pub trait NeuronModelCollectionParallelPackedLinearTrait<CANQ: CorticalAreaNeuronQuantization>:
NeuronModelCollectionBaseLinearTrait<CANQ> +
PackedLinearIterationMut<'static, CANQ>
{
    type ParallelSlice: NeuronModelNeuronParallelPackedCollectionSliceTrait<'static, CANQ>;
    type ParallelSliceMut: NeuronModelNeuronPackedParallelCollectionSliceMutTrait<'static, CANQ>;

    fn get_neurons_as_parallel_slice(&self) -> &Self::ParallelSlice;
    fn get_neurons_as_parallel_slice_mut(&mut self) -> &mut Self::ParallelSliceMut;

    fn is_sorted_in_increasing_index_order(&self) -> bool {
        true // All packed instances are always sorted
    }
}

//region Packed Slices

/// Defines all the fields for a slice of all neurons as an immutable reference. Required for all model
/// implementations. Used to generate Individual Neuron Ref Slice Struct
pub trait NeuronModelNeuronParallelPackedCollectionSliceTrait<'a, CANQ: CorticalAreaNeuronQuantization>:
PackedLinearIteration<'a, CANQ>
{
    /// Membrane potential is required for all neuron models
    fn get_membrane_potential(&self) -> &'a [NeuronMembranePotential<CANQ::NeuronValueQuant>];

    // Define other fields here. Make sure all implementations use inline
}

/// Defines all the fields for a slice of all neurons as a mutable reference. Required for all model
/// implementations. Used to generate Individual Neuron Mut Ref Slice Struct
pub trait NeuronModelNeuronPackedParallelCollectionSliceMutTrait<'a, CANQ: CorticalAreaNeuronQuantization>:
NeuronModelNeuronParallelPackedCollectionSliceTrait<'a, CANQ> +
PackedLinearIterationMut<'a, CANQ>
{
    /// Membrane potential is required for all neuron models
    fn get_membrane_potential_mut(&mut self) -> &mut [NeuronMembranePotential<CANQ::NeuronValueQuant>];

    // Define other fields here. Make sure all implementations use inline
}

//endregion

//endregion

//region Shared Serial Packed

/// Trait for structs that are sparse
pub trait NeuronModelCollectionSerialPackedLinearTrait<CANQ: CorticalAreaNeuronQuantization>:
NeuronModelCollectionBaseLinearTrait<CANQ> +
PackedLinearIterationMut<'static, CANQ>
{
    fn get_neurons_as_serial_slice(&self) -> &[Self::SingleNeuron];
    fn get_neurons_as_serial_slice_mut(&mut self) -> &mut [Self::SingleNeuron];

    fn is_sorted_in_increasing_index_order(&self) -> bool {
        true // All packed instances are always sorted
    }
}

//endregion

//region Packed Iteration
pub trait PackedLinearIteration<'a, CANQ: CorticalAreaNeuronQuantization> {

    type SinglePackedNeuronRef: NeuronModelNeuronRefTrait<'static, CANQ>;

    fn linear_neuron_iter(&self) -> impl Iterator<Item = &'a Self::SinglePackedNeuronRef>;
}

pub trait PackedLinearIterationMut<'a, CANQ: CorticalAreaNeuronQuantization>:
PackedLinearIteration<'a, CANQ>
{
    type SinglePackedNeuronRefMut: NeuronModelNeuronMutRefTrait<'static,CANQ>;

    fn linear_neuron_iter_mut(&mut self) -> impl Iterator<Item = &'a Self::SinglePackedNeuronRefMut>;
}

//endregion

//endregion

//region Shared Sparse

/// Trait for structs that are sparse
pub trait NeuronModelCollectionSparseLinearTrait<CANQ: CorticalAreaNeuronQuantization>:
NeuronModelCollectionBaseLinearTrait<CANQ>
{

}



//endregion