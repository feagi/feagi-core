//! This file defines the traits used to define the fields that every neuron in a given neuron
//! model should use.
use core::ops::Range;
use crate::define_ref_immut_mut_access_trait_methods;
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

/// Impliments
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

            pub struct [<$model_neuron_name Neuron>]<CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization> {
                membrane_potential: $crate::neuron_dynamics::code_definitions::neurons::common_linear_neuron_structs::NeuronMembranePotential<CANQ::NeuronValueQuant>,
                $(
                    $field : $ty,
                )*
            }

            pub struct [<$model_neuron_name NeuronRef>]<'a, CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization + 'a> {
                membrane_potential: &'a $crate::neuron_dynamics::code_definitions::neurons::common_linear_neuron_structs::NeuronMembranePotential<CANQ::NeuronValueQuant>,
                $(
                    $field : &'a $ty,
                )*
            }

            pub struct [<$model_neuron_name NeuronRefMut>]<'a, CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization + 'a> {
                membrane_potential: &'a mut $crate::neuron_dynamics::code_definitions::neurons::common_linear_neuron_structs::NeuronMembranePotential<CANQ::NeuronValueQuant>,
                $(
                    $field : &'a mut $ty,
                )*
            }



            pub trait [<$model_neuron_name NeuronTrait>]<CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization>:
            $crate::neuron_dynamics::code_definitions::neurons::base_neuron_model_fields::NeuronModelNeuron<CANQ>
            {
                $(
                    fn [<get_ $field>](&self) -> &$ty;
                    fn [<get_ $field _mut>](&mut self) -> &mut $ty;
                )*
            }

            pub trait [<$model_neuron_name NeuronRefTrait>]<'a, CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization>:
            $crate::neuron_dynamics::code_definitions::neurons::base_neuron_model_fields::NeuronModelNeuronRefTrait<'a, CANQ>
            {
                $(
                    fn [<get_ $field>](&self) -> &'a $ty;
                )*
            }

            pub trait [<$model_neuron_name NeuronMutRefTrait>]<'a, CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization>:
            $crate::neuron_dynamics::code_definitions::neurons::base_neuron_model_fields::NeuronModelNeuronMutRefTrait<'a, CANQ>
            {
                $(
                    fn [<get_ $field _mut>](&mut self) -> &'a mut $ty;
                )*
            }


            impl<CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization> [<$model_neuron_name Neuron>]<CANQ> {
                #[inline(always)]
                pub fn new(
                    membrane_potential: $crate::neuron_dynamics::code_definitions::neurons::common_linear_neuron_structs::NeuronMembranePotential<CANQ::NeuronValueQuant>,
                    $(
                        $field: $ty,
                    )*
                ) -> Self {
                    Self {
                        membrane_potential,
                        $(
                            $field,
                        )*
                    }
                }
            }

            impl<CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization> $crate::neuron_dynamics::code_definitions::neurons::base_neuron_model_fields::NeuronModelNeuron<CANQ> for [<$model_neuron_name Neuron>]<CANQ> {
                $crate::define_ref_immut_mut_access_concrete_methods!(
                    membrane_potential,
                    $crate::neuron_dynamics::code_definitions::neurons::common_linear_neuron_structs::NeuronMembranePotential<CANQ::NeuronValueQuant>,
                    membrane_potential
                );
            }

            impl<'a, CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization> $crate::neuron_dynamics::code_definitions::neurons::base_neuron_model_fields::NeuronModelNeuronRef<'a, CANQ> for [<$model_neuron_name NeuronRef>]<'a, CANQ> {
                #[inline(always)]
                fn get_membrane_potential(&self) -> &'a $crate::neuron_dynamics::code_definitions::neurons::common_linear_neuron_structs::NeuronMembranePotential<CANQ::NeuronValueQuant> {
                    self.membrane_potential
                }
            }

            impl<'a, CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization> $crate::neuron_dynamics::code_definitions::neurons::base_neuron_model_fields::NeuronModelNeuronRef<'a, CANQ> for [<$model_neuron_name NeuronRefMut>]<'a, CANQ> {
                #[inline(always)]
                fn get_membrane_potential(&self) -> &'a $crate::neuron_dynamics::code_definitions::neurons::common_linear_neuron_structs::NeuronMembranePotential<CANQ::NeuronValueQuant> {
                    self.membrane_potential
                }
            }

            impl<'a, CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization> $crate::neuron_dynamics::code_definitions::neurons::base_neuron_model_fields::NeuronModelNeuronMutRef<'a, CANQ> for [<$model_neuron_name NeuronRefMut>]<'a, CANQ> {
                #[inline(always)]
                fn get_membrane_potential_mut(&mut self) -> &mut $crate::neuron_dynamics::code_definitions::neurons::common_linear_neuron_structs::NeuronMembranePotential<CANQ::NeuronValueQuant> {
                    self.membrane_potential
                }
            }
        }
    };
}

/// Defines all the fields for a single neuron as independent values. Required for all model
/// implementations. Used to generate Individual Neuron Struct
pub trait NeuronModelNeuronTrait<CANQ: CorticalAreaNeuronQuantization>
{
    /// Membrane potential is required for all neuron models
    fn get_membrane_potential(&self) -> &NeuronMembranePotential<CANQ::NeuronValueQuant>;
    fn get_membrane_potential_mut(&mut self) -> &mut NeuronMembranePotential<CANQ::NeuronValueQuant>;
}

/// Defines all the fields for a single neuron as an immutable reference. Required for all model
/// implementations. Used to generate Individual Neuron Ref Struct
pub trait NeuronModelNeuronRefTrait<'a, CANQ: CorticalAreaNeuronQuantization> {

    /// Membrane potential is required for all neuron models
    fn get_membrane_potential(&self) -> &'a NeuronMembranePotential<CANQ::NeuronValueQuant>;

    // Define other fields here. Make sure all implementations use inline
}

/// Defines all the fields for a single neuron as a mutable reference. Required for all model
/// implementations. Used to generate Individual Neuron Mut Ref Struct
pub trait NeuronModelNeuronMutRefTrait<'a, CANQ: CorticalAreaNeuronQuantization>:
NeuronModelNeuronRefTrait<'a, CANQ>
{
    /// Membrane potential is required for all neuron models
    fn get_membrane_potential_mut(&mut self) -> &mut NeuronMembranePotential<CANQ::NeuronValueQuant>;

    // Define other fields here. Make sure all implementations use inline
}

/// Won't be a struct itself, but rather extends the ref traits to allow cloning the ref into a
/// "NeuronModelNeuron"
pub trait NeuronModelNeuronRefClonableTrait<CANQ: CorticalAreaNeuronQuantization>:
{
    type NeuronStruct: NeuronModelNeuronTrait<CANQ>;
    fn clone_as_neuron(&self) -> Self::NeuronStruct;
}


//endregion

//region Neuron Slices

#[macro_export]
macro_rules! __internal_neuron_generate_base_neuron_slices_traits{
    (
        $model_neuron_name:ident,
        $model_collection_name:ident,
        {
            $( $field:ident : $ty:ty ),* $(,)?
        }
    ) => {
        ::paste::paste! {




        }
    };
}

/// Defines all the fields for a slice of all neurons as an immutable reference. Required for all model
/// implementations. Used to generate Individual Neuron Ref Slice Struct
pub trait NeuronModelNeuronSliceRef<'a, CANQ: CorticalAreaNeuronQuantization> {
    /// Membrane potential is required for all neuron models
    fn get_membrane_potential(&self) -> &'a [NeuronMembranePotential<CANQ::NeuronValueQuant>];

    // Define other fields here. Make sure all implementations use inline
}

/// Defines all the fields for a slice of all neurons as a mutable reference. Required for all model
/// implementations. Used to generate Individual Neuron Mut Ref Slice Struct
pub trait NeuronModelNeuronSliceMutRef<'a, CANQ: CorticalAreaNeuronQuantization>:
NeuronModelNeuronSliceRef<'a, CANQ>
{
    /// Membrane potential is required for all neuron models
    fn get_membrane_potential_mut(&mut self) -> &mut [NeuronMembranePotential<CANQ::NeuronValueQuant>];

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

    type SingleNeuron: NeuronModelNeuronTrait<CANQ>;
    type SingleNeuronRef: NeuronModelNeuronRefTrait<'static, CANQ>;
    type SingleNeuronMutRef: NeuronModelNeuronMutRefTrait<'static,CANQ>;


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
    type NeuronMutSlice: NeuronModelNeuronSliceMutRef<'static, CANQ>;

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