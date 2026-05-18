//! This file defines the traits used to define the fields that every neuron in a given neuron
//! model should use.

use core::ops::Range;
use crate::{define_ref_immut_access_trait_methods, define_ref_immut_mut_access_trait_methods, define_ref_mut_access_trait_methods};
use crate::neuron::FeagiNeuronError;
use crate::quantization_level::CorticalAreaNeuronQuantization;
use crate::neuron_dynamics::code_definitions::neurons::common_linear_neuron_structs::{LinearNeuronIndexCount, NeuronMembranePotential};
use crate::neuron_dynamics::code_definitions::neurons::iterators::{EnumeratedNeuronLinearReference, EnumeratedNeuronLinearReferenceMut};
// TODO macro that builds all the neuron traits from a given list of properties, the collection,
// and auto implements the functions in here

// TODO sub macros that implemnents the internal functions

//region high level macros

macro_rules! __internal_neuron_generate_base_collection_struct_vec_and_base_traits{
    (
        $model_name:ident,
        {
            $( $field:ident : $ty:ty ),* $(,)?
        }
    ) => {
        ::paste::paste! {

        }
    };
}

macro_rules! __internal_neuron_generate_base_collection_struct_arr_and_base_traits{
    (
        $model_name:ident,
        {
            $( $field:ident : $ty:ty ),* $(,)?
        }
    ) => {
        ::paste::paste! {

        }
    };
}

macro_rules! __internal_neuron_generate_base_collection_struct_hash_and_base_traits{
    (
        $model_name:ident,
        {
            $( $field:ident : $ty:ty ),* $(,)?
        }
    ) => {
        ::paste::paste! {

        }
    };
}

//endregion



//region Individual Neurons

#[macro_export]
macro_rules! __internal_neuron_generate_base_neuron_structs_and_traits{
    (
        $model_neuron_name:ident,
        $model_collection_name:ident,
        {
            $( $field:ident : $ty:ty ),* $(,)?
        }
    ) => {
        ::paste::paste! {

            pub struct [<$model_neuron_name> Neuron]<CANQ: CorticalAreaNeuronQuantization> {
                membrane_potential: NeuronMembranePotential<CANQ::NeuronValueQuant>,
                $(
                    $field : $ty,
                )*
            }

            pub struct [<$model_neuron_name> NeuronRef]<'a, CANQ: CorticalAreaNeuronQuantization> {
                collection: &'a $model_neuron_name,
            }

            pub struct [<$model_neuron_name> NeuronRefMut]<'a, CANQ: CorticalAreaNeuronQuantization> {
                collection: &'a mut $model_neuron_name,
            }

            // impl NeuronModelNeuronRef for Neuron NeuronRef NeuronRefMut

            // impl NeuronModelNeuronMutRef for Neuron NeuronRefMut

            // impl NeuronModelNeuron for Neuron

            // impl NeuronModelNeuronRefClonable for NeuronRef NeuronRefMut
            impl<'a, CANQ: CorticalAreaNeuronQuantization> NeuronModelNeuronRefClonable<CANQ> for [<$model_neuron_name> NeuronRef]<'a, CANQ> {
                type NeuronStruct: NeuronModelNeuron<CANQ> = [<$model_neuron_name> Neuron];

                #[inline(always)]
                fn clone_as_neuron(&self) -> Self::NeuronStruct
                {

                }
            }

        }
    };
}

/// Defines all the fields for a single neuron as an immutable reference. Required for all model
/// implementations. Used to generate Individual Neuron Ref Struct
pub trait NeuronModelNeuronRef<'a, CANQ: CorticalAreaNeuronQuantization> {

    /// Membrane potential is required for all neuron models
    define_ref_immut_access_trait_methods!(membrane_potential, &'a NeuronMembranePotential<CANQ::NeuronValueQuant>);

    // Define other fields here. Make sure all implementations use inline
}

/// Defines all the fields for a single neuron as a mutable reference. Required for all model
/// implementations. Used to generate Individual Neuron Mut Ref Struct
pub trait NeuronModelNeuronMutRef<'a, CANQ: CorticalAreaNeuronQuantization>:
NeuronModelNeuronRef<'a, CANQ>
{
    /// Membrane potential is required for all neuron models
    define_ref_mut_access_trait_methods!(membrane_potential, &'a mut NeuronMembranePotential<CANQ::NeuronValueQuant>);

    // Define other fields here. Make sure all implementations use inline
}


/// Defines all the fields for a single neuron as independent values. Required for all model
/// implementations. Used to generate Individual Neuron Struct
pub trait NeuronModelNeuron<CANQ: CorticalAreaNeuronQuantization>:
{
    /// Membrane potential is required for all neuron models
    define_ref_immut_mut_access_trait_methods!(membrane_potential, &mut NeuronMembranePotential<CANQ::NeuronValueQuant>);
}

/// Won't be a struct itself, but rather extends the ref traits to allow cloning the ref into a
/// "NeuronModelNeuron"
pub trait NeuronModelNeuronRefClonable<CANQ: CorticalAreaNeuronQuantization>:
{
    type NeuronStruct: NeuronModelNeuron<CANQ>;
    fn clone_as_neuron(&self) -> Self::NeuronStruct; // TODO macro
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


    // NOTE: The below are filled by implementations

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

//region Dense

macro_rules! __internal_neuron_generate_linear_dense_traits{
    () => {};
}

/// Shared Neuron Collection Traits of dense (nonsparse) neuron collections
pub trait NeuronModelCollectionBaseDense<CANQ: CorticalAreaNeuronQuantization>:
NeuronModelCollectionBaseShared<CANQ>
{
    type NeuronSlice: NeuronModelNeuronSliceRef<'static, CANQ>;
    type NeuronMutSlice: NeuronModelNeuronMutSliceRef<'static, CANQ>;

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

//endregion


//region Sparse

macro_rules! __internal_neuron_generate_linear_sparse_traits{
    () => {};
}

pub trait NeuronModelCollectionBaseSparse<CANQ: CorticalAreaNeuronQuantization>:
NeuronModelCollectionBaseShared<CANQ>
{
    // TODO macros can build all these implementations

    /// How much space is allocated for neurons in memory
    fn get_neuron_capacity(&self) -> LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>;

    ///
    fn insert_or_overwrite_neuron_value_unordered(&mut self, at_index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>, neuron_data: Self::SingleNeuron) -> Result<(), FeagiNeuronError>;

    fn delete_neuron_value_at(&mut self, at_index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<(), FeagiNeuronError>;
}

/// Indexed (Sparse) Neuron Array that uses vectors
pub trait NeuronModelCollectionIndexedVector<CANQ: CorticalAreaNeuronQuantization>:
NeuronModelCollectionBaseSparse<CANQ>
{
    const NEURON_COLLECTION_TYPE: NeuronCollectionType = NeuronCollectionType::IndexedResizableVector;

    fn are_indexes_sorted_in_increasing_order(&self) -> bool;

    fn insert_or_overwrite_neuron_value_sorted(&mut self, at_index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>, neuron_data: Self::SingleNeuron) -> Result<(), FeagiNeuronError>;

    fn sort_indexes_in_increasing_order(&mut self);

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


//endregion