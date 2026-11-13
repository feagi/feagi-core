//! The generic pieces of the neuron model enum tree. Every model has a single enum generic over an
//! [`NMContainer`], holding one variant per quantization (such as `NMFeagiSimpleQuanted<C>`), and
//! [`NMModeled`] holds one variant per model. Which data the tree holds is chosen by the container,
//! so one tree serves the kinds, precursors, generators and storages alike.
//!
//! [`NMMember`] (implemented by every marker struct) and the visitors ([`NMVisitor`] and its by
//! reference variants) reach the data of any member directly, without walking the tree a level at
//! a time

use core::fmt::Debug;
use core::marker::PhantomData;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage::NeurogenesisStorage;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage_or_by_generator::{NeurogenesisFromStorageOrByGeneratorOf, NeurogenesisFromStorageOrByGeneratorPrecursor};
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::neuron_models::models::kind::NMKind;
use crate::neuron_models::models::NMModeled;

//region Containers

/// A kind of data held for any member of the neuron model enum tree. Implemented on a zero sized
/// struct per kind of data, `Of` being that data for member `NM`
pub trait NMContainer: Clone + Copy + Debug + Send + Sync + 'static {
    /// The data held for member `NM`
    type Of<NM: NeuronModel>: Clone + Debug;
}

/// Holds the marker struct of the member itself, describing a model and its quantization
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NMContainerMarker;

impl NMContainer for NMContainerMarker {
    type Of<NM: NeuronModel> = NM;
}

/// Holds the data needed to build the generator of the member
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NMContainerGeneratorPrecursor<FIQ: FeagiIndexQuantization>(PhantomData<FIQ>);

impl<FIQ: FeagiIndexQuantization> NMContainer for NMContainerGeneratorPrecursor<FIQ> {
    type Of<NM: NeuronModel> = NM::NeurogenesisGeneratorPrecursor<FIQ>;
}

/// Holds the generator of the member
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NMContainerGenerator<FIQ: FeagiIndexQuantization>(PhantomData<FIQ>);

impl<FIQ: FeagiIndexQuantization> NMContainer for NMContainerGenerator<FIQ> {
    type Of<NM: NeuronModel> = NM::NeurogenesisGenerator<FIQ>;
}

/// Holds the stored neuron data of the member
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NMContainerStorage<FIQ: FeagiIndexQuantization>(PhantomData<FIQ>);

impl<FIQ: FeagiIndexQuantization> NMContainer for NMContainerStorage<FIQ> {
    type Of<NM: NeuronModel> = NeurogenesisStorage<FIQ, NM>;
}

/// Holds either the stored neuron data or the generator of the member
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NMContainerStorageOrGenerator<FIQ: FeagiIndexQuantization>(PhantomData<FIQ>);

impl<FIQ: FeagiIndexQuantization> NMContainer for NMContainerStorageOrGenerator<FIQ> {
    type Of<NM: NeuronModel> = NeurogenesisFromStorageOrByGeneratorOf<FIQ, NM>;
}

/// Holds either the stored neuron data or the generator precursor of the member
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NMContainerStorageOrGeneratorPrecursor<FIQ: FeagiIndexQuantization>(PhantomData<FIQ>);

impl<FIQ: FeagiIndexQuantization> NMContainer for NMContainerStorageOrGeneratorPrecursor<FIQ> {
    type Of<NM: NeuronModel> = NeurogenesisFromStorageOrByGeneratorPrecursor<FIQ, NM>;
}

//endregion

//region Member

/// A member of the neuron model enum tree: a model at one of its quantizations. Implemented by every
/// marker struct, it wraps and unwraps the data of any container straight to and from
/// [`NMModeled`]
pub trait NMMember: NeuronModel + Sized {
    /// The model and quantization this member is
    const KIND: NMKind;

    /// The marker struct itself, for building the kind of this member without an instance at hand
    const MARKER: Self;

    /// Wraps data of this member
    fn into_modeled<C: NMContainer>(data: C::Of<Self>) -> NMModeled<C>;

    /// Unwraps the data held, if held for this member
    fn try_from_modeled<C: NMContainer>(value: NMModeled<C>) -> Result<C::Of<Self>, NMAccessError>;

    /// The data held, if held for this member
    fn try_ref_modeled<C: NMContainer>(value: &NMModeled<C>) -> Result<&C::Of<Self>, NMAccessError>;

    /// The data held, if held for this member
    fn try_mut_modeled<C: NMContainer>(value: &mut NMModeled<C>) -> Result<&mut C::Of<Self>, NMAccessError>;
}

/// Data of the neuron model enum tree was accessed as a member or model it does not hold
#[derive(Clone, Copy, PartialEq, Debug, thiserror::Error)]
pub enum NMAccessError {
    /// Accessed as a member other than the one held
    #[error("Tried to access the neuron model {expected:?}, but {actual:?} is held")]
    IncorrectMember { expected: NMKind, actual: NMKind },
    /// Accessed as a model other than the one held
    #[error("Tried to access a neuron model of model {expected}, but {actual:?} is held")]
    IncorrectModel { expected: &'static str, actual: NMKind },
}

//endregion

//region Visitors

/// Runs generic code on the data of whichever member is held, see `NMModeled::dispatch`
pub trait NMVisitor<C: NMContainer> {
    type Output;
    fn visit<NMM: NMMember>(self, data: C::Of<NMM>) -> Self::Output;
}

/// Runs generic code on a reference to the data of whichever member is held, see
/// `NMModeled::dispatch_ref`
pub trait NMVisitorRef<'a, C: NMContainer> {
    type Output;
    fn visit<NMM: NMMember>(self, data: &'a C::Of<NMM>) -> Self::Output;
}

/// Runs generic code on a mutable reference to the data of whichever member is held, see
/// `NMModeled::dispatch_mut`
pub trait NMVisitorMut<'a, C: NMContainer> {
    type Output;
    fn visit<NMM: NMMember>(self, data: &'a mut C::Of<NMM>) -> Self::Output;
}

//endregion
