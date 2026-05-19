use feagi_structures::CorticalAreaNeuronQuantization;
use feagi_structures::neuron::{FeagiNeuronError, LinearNeuronIndexCount, NeuronMembranePotential};
use crate::dynamics::neuron::linear::neurons::{NeuronData, NeuronDataRef, NeuronDataRefMut, NeuronModelParametersTrait};

//region Shared Enums

/// Defines if neurons are expected to be grouped together or not
#[derive(Clone, PartialEq)]
pub enum NeuronGroupingType {
    Single,
    Multiple,
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
pub trait NeuronModelCollectionBaseLinearTrait<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>
{
    const NEURON_GROUPING_TYPE: NeuronGroupingType;
    const NEURON_COLLECTION_TYPE: NeuronCollectionType;

    fn is_sorted_in_increasing_index_order(&self) -> bool;

    /// What is the upper bound (exclusive) neuron  index allowed?
    fn get_neuron_max_linear_index(&self) -> LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>;

    /// In sparse implementations, then number of neurons contained may be less than the max possible index
    fn get_number_contained_neurons(&self) -> LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>;

    fn try_get_neuron_data(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<NeuronData<CANQ, NMP>, FeagiNeuronError>;

    fn try_get_neuron_data_ref(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<NeuronDataRef<CANQ, NMP>, FeagiNeuronError>;

    fn try_get_neuron_data_ref_mut(&mut self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<NeuronDataRefMut<CANQ, NMP>, FeagiNeuronError>;

    fn try_get_membrane_potential_data(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<NeuronMembranePotential<CANQ::NeuronValueQuant>, FeagiNeuronError>;

    fn try_get_membrane_potential_data_ref(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<&NeuronMembranePotential<CANQ::NeuronValueQuant>, FeagiNeuronError>;

    fn try_get_membrane_potential_data_ref_mut(&mut self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<&mut NeuronMembranePotential<CANQ::NeuronValueQuant>, FeagiNeuronError>;

    fn try_get_neuron_model_data(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<NMP, FeagiNeuronError>;

    fn try_get_neuron_model_data_ref(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<&NMP, FeagiNeuronError>;

    fn try_get_neuron_model_data_ref_mut(&mut self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<&mut NMP, FeagiNeuronError>;

    fn enumerated_linear_neuron_iter(&self) -> impl Iterator<Item = EnumeratedNeuronLinearReference<CANQ, NeuronDataRef<CANQ, NMP>>>;

    fn enumerated_linear_neuron_iter_mut(&self) -> impl Iterator<Item = EnumeratedNeuronLinearReferenceMut<CANQ, NeuronDataRefMut<CANQ, NMP>>>;

    // TODO RAYON iterators
}

//region Iterators
pub struct EnumeratedNeuronLinearReference<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> {
    linear_neuron_index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
    potential: &'a NeuronMembranePotential<CANQ::NeuronValueQuant>,
    model_parameters: &'a NMP,
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> EnumeratedNeuronLinearReference<'a, CANQ, NMP> {
    pub fn get_linear_index(&self) -> &LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        &self.linear_neuron_index
    }

    pub fn neuron_ref(&self) -> NeuronDataRef<'a, CANQ, NMP> {
        NeuronDataRef::new(self.potential, self.model_parameters)
    }
}

pub struct EnumeratedNeuronLinearReferenceMut<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> {
    linear_neuron_index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
    potential: &'a mut NeuronMembranePotential<CANQ::NeuronValueQuant>,
    model_parameters: &'a mut NMP,
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> EnumeratedNeuronLinearReferenceMut<'a, CANQ, NMP> {
    pub fn get_linear_index(&self) -> &LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        &self.linear_neuron_index
    }

    pub fn neuron_ref(&self) -> NeuronDataRef<'a, CANQ, NMP> {
        NeuronDataRef::new(self.potential, self.model_parameters)
    }

    pub fn neuron_ref_mut(&self) -> NeuronDataRefMut<'a, CANQ, NMP> {
        NeuronDataRefMut::new(self.potential, self.model_parameters)
    }
}
//endregion

//endregion

//region Shared Packed

/// Trait for structs that are densely packed (not sparse)
pub trait NeuronModelCollectionPackedLinearTrait<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>:
NeuronModelCollectionBaseLinearTrait<CANQ, NMP> +
PackedLinearIterationMut<'static, CANQ, NMP>
{
    fn get_membrane_potentials_as_slice(&self) -> &[NeuronMembranePotential<CANQ::NeuronValueQuant>];
    fn get_membrane_potentials_as_slice_mut(&mut self) -> &mut [NeuronMembranePotential<CANQ::NeuronValueQuant>];
    fn get_neuron_model_data_as_slice(&self) -> &[NMP];
    fn get_neuron_model_data_as_slice_mut(&mut self) -> &mut [NMP];

    fn get_neuron_data_as_slice(&self) -> NeuronModelSlice<'static, CANQ, NMP> {
        NeuronModelSlice {
            neuron_potentials: self.get_membrane_potentials_as_slice(),
            get_model_parameters: self.get_neuron_model_data_as_slice(),
        }
    }
    fn get_neuron_data_as_slice_mut(&mut self) -> NeuronModelMutSlice<'static, CANQ, NMP> {
        NeuronModelMutSlice {
            neuron_potentials: self.get_membrane_potentials_as_slice_mut(),
            get_model_parameters: self.get_neuron_model_data_as_slice_mut(),
        }
    }

    fn is_sorted_in_increasing_index_order(&self) -> bool {
        true // All packed instances are always sorted
    }
}

//region Packed Slices

/// Defines all the fields for a slice of all neurons as an immutable reference
pub struct NeuronModelSlice<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>
{
    /// Potential of neuron
    pub neuron_potentials: &'a [NeuronMembranePotential<CANQ::NeuronValueQuant>],

    /// All other parameters of neurons
    pub get_model_parameters: &'a [NMP]
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> PackedLinearIteration<'a, CANQ, NMP> for NeuronModelSlice<'a, CANQ, NMP> {
    fn linear_neuron_iter(&self) -> impl Iterator<Item=NeuronDataRef<'a, CANQ, NMP>> {
        todo!()
    }
}

/// Defines all the fields for a slice of all neurons as a mutable reference
pub struct NeuronModelMutSlice<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>
{
    /// Potential of neuron
    pub neuron_potentials: &'a mut [NeuronMembranePotential<CANQ::NeuronValueQuant>],

    /// All other parameters of neurons
    pub get_model_parameters: &'a mut [NMP]
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> PackedLinearIteration<'a, CANQ, NMP> for NeuronModelMutSlice<'a, CANQ, NMP> {
    fn linear_neuron_iter(&self) -> impl Iterator<Item=NeuronDataRef<'a, CANQ, NMP>> {
        todo!()
    }
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> PackedLinearIterationMut<'a, CANQ, NMP> for NeuronModelMutSlice<'a, CANQ, NMP> {

    fn linear_neuron_iter_mut(&mut self) -> impl Iterator<Item=NeuronDataRefMut<'a, CANQ, NMP>> {
        todo!()
    }
}



//endregion


//region Packed Iteration
pub trait PackedLinearIteration<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> {
    fn linear_neuron_iter(&self) -> impl Iterator<Item = NeuronDataRef<'a, CANQ, NMP>>;
}

pub trait PackedLinearIterationMut<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>:
PackedLinearIteration<'a, CANQ, NMP>
{
    fn linear_neuron_iter_mut(&mut self) -> impl Iterator<Item = NeuronDataRefMut<'a, CANQ, NMP>>;
}

//endregion

//endregion

//region Shared Sparse

/// Trait for structs that are sparse
pub trait NeuronModelCollectionSparseLinearTrait<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>:
NeuronModelCollectionBaseLinearTrait<CANQ, NMP>
{
    // idk?
}



//endregion