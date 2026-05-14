//! This file defines the traits used to define the fields that every neuron in a given neuron
//! model should use.

use core::ops::Range;
use crate::{define_ref_immut_access_trait_methods, define_ref_immut_mut_access_trait_methods, };
use crate::quantization_level::CorticalAreaNeuronQuantization;
use crate::neuron_dynamics::code_definitions::neurons::common_neuron_structs::{LinearNeuronIndexCount, NeuronMembranePotential};

// TODO macro that builds all the neuron traits from a given list of properties, the collection,
// and auto implements the functions in here

// TODO sub macros that implemnents the internal functions

//region Individual Neurons

/// Defines all the fields for a single neuron as independent values. Required for all model
/// implementations. Used to generate Individual Neuron Struct
pub trait NeuronModelNeuron<CANQ: CorticalAreaNeuronQuantization> {
    /// Membrane potential is required for all neuron models
    define_ref_immut_mut_access_trait_methods!(membrane_potential, NeuronMembranePotential<CANQ::NeuronValueQuant>);

    // Define other fields here. Make sure all implementations use inline
}

/// Defines all the fields for a single neuron as an immutable reference. Required for all model
/// implementations. Used to generate Individual Neuron Ref Struct
pub trait NeuronModelNeuronRef<'a, CANQ: CorticalAreaNeuronQuantization> {

    type NeuronStruct: NeuronModelNeuron<CANQ>;

    fn clone_as_neuron(&self) -> Self::NeuronStruct; // TODO macro

    /// Membrane potential is required for all neuron models
    define_ref_immut_access_trait_methods!(membrane_potential, &'a NeuronMembranePotential<CANQ::NeuronValueQuant>);

    // Define other fields here. Make sure all implementations use inline
}

/// Defines all the fields for a single neuron as a mutable reference. Required for all model
/// implementations. Used to generate Individual Neuron Mut Ref Struct
pub trait NeuronModelNeuronMutRef<'a, CANQ: CorticalAreaNeuronQuantization> {

    type NeuronStruct: NeuronModelNeuron<CANQ>;

    fn clone_as_neuron(&self) -> Self::NeuronStruct; // TODO macro

    /// Membrane potential is required for all neuron models
    define_ref_immut_mut_access_trait_methods!(membrane_potential, &'a mut NeuronMembranePotential<CANQ::NeuronValueQuant>);

    // Define other fields here. Make sure all implementations use inline
}


//endregion

//region Neuron Slices

/// Defines all the fields for a slice of all neurons as an immutable reference. Required for all model
/// implementations. Used to generate Individual Neuron Ref Slice Struct
pub trait NeuronModelNeuronSliceRef<'a, CANQ: CorticalAreaNeuronQuantization> {
    /// Membrane potential is required for all neuron models
    define_ref_immut_access_trait_methods!(membrane_potential, &'a [NeuronMembranePotential<CANQ::NeuronValueQuant>]);

    // Define other fields here. Make sure all implementations use inline
}

/// Defines all the fields for a slice of all neurons as a mutable reference. Required for all model
/// implementations. Used to generate Individual Neuron Mut Ref Slice Struct
pub trait NeuronModelNeuronMutSliceRef<'a, CANQ: CorticalAreaNeuronQuantization> {
    /// Membrane potential is required for all neuron models
    define_ref_immut_mut_access_trait_methods!(membrane_potential, &'a mut [NeuronMembranePotential<CANQ::NeuronValueQuant>]);

    // Define other fields here. Make sure all implementations use inline
}

//endregion


//region Neuron Collections

/// Enum for defining what kind of collection a neuron collection is
#[derive(Clone, PartialEq)]
pub enum NeuronCollectionType {
    DenseFixedArray,
    DenseResizableVector,
    // TODO Indexed Fixed Array?
    IndexedResizableVector,
    // TODO indexed fixed hashmap?
    IndexedResizableHashmap
    // TODO others? Device support specifics?
}

/// Properties shared by all Neuron Collections,
/// which is a way of simply Neurons in a linear fashion in memory
pub trait NeuronModelCollectionBaseShared<CANQ: CorticalAreaNeuronQuantization>
{
    // TODO these parameters should be generatable by macro

    /// Define the data structure holding the actual neural data
    const NEURON_COLLECTION_TYPE: NeuronCollectionType;

    type SingleNeuron: NeuronModelNeuron<CANQ>;
    type SingleNeuronRef: NeuronModelNeuronRef<'static, CANQ>;
    type SingleNeuronMutRef: NeuronModelNeuronMutRef<'static,CANQ>;
    type NeuronSlice: NeuronModelNeuronSliceRef<'static, CANQ>;
    type NeuronMutSlice: NeuronModelNeuronMutSliceRef<'static, CANQ>;


    // NOTE: The below are filled by implementations

    /// What is the upper bound (exclusive) neuron  index allowed?
    fn get_neuron_max_linear_index(&self) -> LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>;

    /// In sparse implementations, then number of neurons contained may be less than the max possible index
    fn get_number_contained_neurons(&self) -> LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>;

    fn try_get_neuron_data_ref(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<Self::SingleNeuronRef, FeagiNeuronError>;

    fn try_get_neuron_data_ref_mut(&mut self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<&Self::SingleNeuronMutRef, FeagiNeuronError>;

    fn enumerated_linear_neuron_iter(&self) -> impl Iterator<Item = EnumeratedNeuronLinearReference<CANQ,Self::SingleNeuronReference>>;

    fn enumerated_linear_neuron_iter_mut(&self) -> impl Iterator<Item = EnumeratedNeuronLinearReferenceMut<CANQ,Self::SingleNeuronReference>>;

    // TODO RAYON iterators
}

/// Shared Neuron Collection Traits of dense (nonsparse) neuron collections
pub trait NeuronModelCollectionBaseDense<CANQ: CorticalAreaNeuronQuantization>:
NeuronModelCollectionBaseShared<CANQ>
{
    // TODO these parameters should be generatable by macro

    fn try_get_neuron_data_slice_ref(&self, index_range: Range<LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>>) -> Result<Self::NeuronSlice, FeagiNeuronError>;

    fn try_get_neuron_data_slice_ref_mut(&mut self, index_range: Range<LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>>) -> Result<Self::NeuronMutSlice, FeagiNeuronError>;

    fn linear_neuron_iter(&self) -> impl Iterator<Item = &Self::SingleNeuronRef> {
        todo!()
    }

    fn linear_neuron_iter_mut(&self) -> impl Iterator<Item = &mut Self::SingleNeuronMutRef> {
        todo!()
    }

    // TODO RAYON iterators
}

/*
/// Dense neuron array that doesnt grow. Useful for embedded
pub trait NeuronModelCollectionDenseFixed<CANQ: CorticalAreaNeuronQuantization>:
NeuronModelCollectionBaseDense<CANQ>
{
    const NEURON_COLLECTION_TYPE: NeuronCollectionType = NeuronCollectionType::DenseFixedArray;
 // TODO
}
 */


/// Dense neuron array that can change size
pub trait NeuronModelCollectionDenseResizable<CANQ: CorticalAreaNeuronQuantization>:
NeuronModelCollectionBaseDense<CANQ>
{
    const NEURON_COLLECTION_TYPE: NeuronCollectionType = NeuronCollectionType::DenseResizableVector;
    // TODO
}

/// Indexed (Sparse) Neuron Array that uses vectors
pub trait NeuronModelCollectionIndexedVector<CANQ: CorticalAreaNeuronQuantization>:
NeuronModelCollectionBaseShared<CANQ>
{
    const NEURON_COLLECTION_TYPE: NeuronCollectionType = NeuronCollectionType::IndexedResizableVector;

    // TODO
}

/*
/// Indexed (Sparse) Neuron Array that uses hashmaps
pub trait NeuronModelCollectionIndexedHashmap<CANQ: CorticalAreaNeuronQuantization>:
NeuronModelCollectionBaseShared<CANQ>
{
    const NEURON_COLLECTION_TYPE: NeuronCollectionType = NeuronCollectionType::IndexedResizableHashmap;

    // TODO
}


 */
//endregion