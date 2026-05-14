use core::ops::Range;
use crate::define_ref_access_trait_methods;
use crate::neuron::FeagiNeuronError;
use crate::neuron::model_specifications::base_specifications::base_iteration::{EnumeratedBaseNeuronReference, EnumeratedBaseNeuronReferenceMut};
use crate::quantization_level::CorticalAreaNeuronQuantization;
use super::base_neuron_common_structs::{LinearNeuronIndexCount, NeuronCollectionType, NeuronMembranePotential};


/// Base Neuron Model Collection (a collection of neurons of a single cortical area), and the
/// required implementations by all computer memory models
pub trait BaseNeuronCollectionSharedTrait<CANQ: CorticalAreaNeuronQuantization> {
    const NEURON_COLLECTION_TYPE: NeuronCollectionType;

    type SingleNeuronReference: BaseNeuronModelDataRefTrait<'static, CANQ>;
    type SingleNeuronReferenceMut: BaseNeuronModelDataMutRefTrait<'static, CANQ>;

    /// What is the upper bound (exclusive) neuron  index allowed?
    fn get_neuron_max_linear_index(&self) -> LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>;

    /// In sparse implementations, then number of neurons contained may be less than the max possible index
    fn get_number_contained_neurons(&self) -> LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>;

    fn try_get_neuron_data_ref(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<Self::SingleNeuronReference, FeagiNeuronError>;

    fn try_get_neuron_data_ref_mut(&mut self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<&Self::SingleNeuronReferenceMut, FeagiNeuronError>;

    fn enumerated_linear_neuron_iter(&self) -> impl Iterator<Item = EnumeratedBaseNeuronReference<CANQ,Self::SingleNeuronReference>>;

    fn enumerated_linear_neuron_iter_mut(&self) -> impl Iterator<Item = EnumeratedBaseNeuronReferenceMut<CANQ,Self::SingleNeuronReference>>;
}

/// A Neuron Model Collection that stores its neurons in a sparse format (value mapped by
/// linear index)
pub trait BaseNeuronModelCollectionSparseTrait<CANQ: CorticalAreaNeuronQuantization>:
BaseNeuronCollectionSharedTrait<CANQ>
{
    
    
    
    fn enumerated_linear_neuron_iter(&self) -> impl Iterator<Item = EnumeratedBaseNeuronReference<CANQ,Self::SingleNeuronReference>> {
        todo!()
    }

    fn enumerated_linear_neuron_iter_mut(&self) -> impl Iterator<Item = EnumeratedBaseNeuronReferenceMut<CANQ,Self::SingleNeuronReference>> {
        todo!()
    }
}

/// A Neuron Model Collection that stores all its neurons in a dense vector or array. Doesn't need
/// to store a separate index field
pub trait BaseNeuronModelCollectionDenseTrait<CANQ: CorticalAreaNeuronQuantization>:
BaseNeuronCollectionSharedTrait<CANQ>
{
    type NeuronSliceReference: BaseNeuronModelDataSliceRefTrait<'static, CANQ>;
    type NeuronSliceReferenceMut: BaseNeuronModelDataSliceMutRefTrait<'static, CANQ>;

    define_ref_access_trait_methods!(membrane_potentials, [NeuronMembranePotential<CANQ::NeuronValueQuant>]);

    fn try_get_neuron_data_slice_ref(&self, index_range: Range<LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>>) -> Result<Self::NeuronSliceReference, FeagiNeuronError>;

    fn try_get_neuron_data_slice_ref_mut(&mut self, index_range: Range<LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>>) -> Result<Self::NeuronSliceReferenceMut, FeagiNeuronError>;

    
    
    
    fn enumerated_linear_neuron_iter(&self) -> impl Iterator<Item = EnumeratedBaseNeuronReference<CANQ,Self::SingleNeuronReference>> {
        todo!()
    }

    fn enumerated_linear_neuron_iter_mut(&self) -> impl Iterator<Item = EnumeratedBaseNeuronReferenceMut<CANQ,Self::SingleNeuronReference>> {
        todo!()
    }

    fn linear_neuron_iter(&self) -> impl Iterator<Item = &Self::SingleNeuronReference> {
        todo!()
    }

    fn linear_neuron_iter_mut(&self) -> impl Iterator<Item = &mut Self::SingleNeuronReference> {
        todo!()
    }


}

/// Represents a single neuron, as a reference data reference
pub trait BaseNeuronModelDataRefTrait<'a, CANQ: CorticalAreaNeuronQuantization> {
    type NeuronModelCollectionType: BaseNeuronCollectionSharedTrait<CANQ>;

    // Model implements neuron potential, and other model specific properties
}

/// Represents several neurons, as a reference data slice
pub trait BaseNeuronModelDataSliceRefTrait<'a, CANQ: CorticalAreaNeuronQuantization> {
    type NeuronModelCollectionType: BaseNeuronCollectionSharedTrait<CANQ>;

    // Model implements neuron potential, and other model specific properties as slices
}


/// Represents a single neuron, as a mut reference data reference
pub trait BaseNeuronModelDataMutRefTrait<'a, CANQ: CorticalAreaNeuronQuantization> {
    type NeuronModelCollectionType: BaseNeuronCollectionSharedTrait<CANQ>;

    // Model implements neuron potential, and other model specific properties
}

/// Represents several neurons, as a mut reference data slice
pub trait BaseNeuronModelDataSliceMutRefTrait<'a, CANQ: CorticalAreaNeuronQuantization> {
    type NeuronModelCollectionType: BaseNeuronCollectionSharedTrait<CANQ>;

    // Model implements neuron potential, and other model specific properties as slices
}