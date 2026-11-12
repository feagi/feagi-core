pub mod foundation;
pub mod processing;
pub mod cortical_identifiers;

use serde::{Deserialize, Serialize};
use crate::basis::cortical_area::foundation::{CTFoundationSubtypedGeneratorPrecursor, CTFoundationSubtypedKind, CTFoundationSubtypedStorage, CTFoundationSubtypedStorageOrGeneratorPrecursor};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{CorticalLayoutContextOf, NeuronLayout, NeuronLayoutOf, NeuronLayoutError, NeuronLayoutRestrictor, NeuronLayoutTag};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::neuron_models::neurogenesis_generator_uniform_defaults::{NeurogenesisGeneratorUniformDefault};
use crate::basis::cortical_area::cortical_identifiers::CorticalAreaInnerID;
use crate::basis::cortical_area::processing::{CTProcessingSubtypedGeneratorPrecursor, CTProcessingSubtypedKind, CTProcessingSubtypedStorage, CTProcessingSubtypedStorageOrGeneratorPrecursor};

//region Cortical Type

/// A cortical type is a higher level description of a cortical area. It has a set layout, but
/// could be a variety of neuron layouts (of that given layout). This is first extended by another
/// trait for the specific cortical type, then implemented bya  struct for the cortical subtype
pub trait CorticalType<NM: NeuronModel<NeuronLayoutTag=Self::NeuronLayout>> {

    /// What neuron layout is this cortical type restricted to?
    type NeuronLayout: NeuronLayoutTag;

    /// What restrictions, if any, should be on the neuron layout?
    type NeuronLayoutRestrictor<FIQ: FeagiIndexQuantization>:
        NeuronLayoutRestrictor< // layouts must match
            FIQ, NeuronLayout=<Self::NeuronLayout as NeuronLayoutTag>::Layout<FIQ>
    >;

    /// What method of inner ID is used for this cortical type
    type IdentifierInner: CorticalAreaInnerID;

    /// Any additional parameters the generator needs to do
    type CorticalAreaGeneratorParameters<FIQ: FeagiIndexQuantization>;

    /// The repr(u8) enum that represents the subtype under a given type
    type CorticalSubTypeKindEnum;

    /// Each cortical subtype has a unique byte identifier (amongst that type). This is used for 
    /// cortical Identification
    const CORTICAL_AREA_SUBTYPE_KIND: Self::CorticalSubTypeKindEnum;

    /// What regions is this cortical type allowed within the genome?
    const ALLOWED_REGIONS: CorticalAllowedRegions;

    /// Get the precursor to the neurogenesis generator for this cortical type / model. This
    /// effectively is the default values for cortical generation for the given neuron model.
    /// some precursors allow changing of the parameters before submitting it for generation
    fn get_neurogenesis_generator_precursor<FIQ: FeagiIndexQuantization>()
        -> NM::NeurogenesisGeneratorPrecursor<FIQ>;

    //region Default impls



    // TODO Cortical Identifier Conversions

    //endregion
}

//endregion


//region Cortical Type Enums



/// A nested enum representing the type and the subtype of a cortical area
#[repr(C, u8)]
#[derive(Copy, Clone, Debug)]
pub enum CorticalTypedKind{
    Foundation(CTFoundationSubtypedKind) = 0,
    Processing(CTProcessingSubtypedKind) = 1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CorticalTypedGeneratorPrecursor<FIQ: FeagiIndexQuantization>
{
    Foundation(CTFoundationSubtypedGeneratorPrecursor<FIQ>),
    Processing(CTProcessingSubtypedGeneratorPrecursor<FIQ>)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CorticalTypedStorage<FIQ: FeagiIndexQuantization>
{
    Foundation(CTFoundationSubtypedStorage<FIQ>),
    Processing(CTProcessingSubtypedStorage<FIQ>)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CorticalTypedStorageOrGeneratorPrecursor<FIQ: FeagiIndexQuantization>
{
    Foundation(CTFoundationSubtypedStorageOrGeneratorPrecursor<FIQ>),
    Processing(CTProcessingSubtypedStorageOrGeneratorPrecursor<FIQ>)
}

//region Impls

impl From<CorticalTypeAndSubtype> for CorticalTypeKind {
    fn from(value: CorticalTypeAndSubtype) -> Self {
        match value {
            CorticalTypeAndSubtype::Foundation(_) => CorticalTypeKind::Foundation,
            CorticalTypeAndSubtype::Processing(_) => CorticalTypeKind::Processing,
        }
    }
}

//endregion

//endregion

//region Subtype Common Traits

// TODO think of how we can specify requantization!

/// Extend any subtype with this trait to automatically add functions for using the Uniform Default
/// neuron generator with it for its neurogenesis. Only makes sense for implementations that
/// use this generator
pub trait CorticalSubTypeUsesDefaultCorticalGenerator<NM>: CorticalType<NM>
where
    NM: NeuronModel<
        NeuronLayoutTag=Self::NeuronLayout,
    >
{
    /// Gets the default values to be used in the generator. Fields can be overridden. TODO Requantization?
    fn uniform_default_generator_defaults<
        FIQ: FeagiIndexQuantization,
    >
    () -> NeurogenesisGeneratorUniformDefault<FIQ, NM>
    where
        NM: NeuronModel<NeurogenesisGenerator<FIQ>=NeurogenesisGeneratorUniformDefault<FIQ, NM>>
    ;
}


//endregion


/// Allows specifying where a cortical area of the given type can reside in the genome
#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash)]
pub enum CorticalAllowedRegions {
    RootOnly,
    InnerOnly,
    RootOrInner, // look, its a bitset!
}