pub mod foundation;
pub mod processing;
pub mod cortical_identifiers;

use serde::{Deserialize, Serialize};
use crate::basis::cortical_area::foundation::{CTGeneratorFoundationSubTyped, CTStorageFoundationSubTyped, CTStorageOrGeneratorFoundationSubTyped, CorticalTypeFoundationSubtypeKind};
use crate::basis::cortical_area::processing::{CTGeneratorProcessingSubTyped, CTStorageOrGeneratorProcessingSubTyped, CTStorageProcessingSubTyped, CorticalTypeProcessingSubtypeKind};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{CorticalLayoutContextOf, NeuronLayout, NeuronLayoutOf, NeuronLayoutError, NeuronLayoutRestrictor, NeuronLayoutTag};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::neuron_models::neurogenesis_generator_uniform_defaults::{NeurogenesisGeneratorUniformDefault};
use crate::basis::cortical_area::cortical_identifiers::CorticalAreaInnerID;

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

    /// Each cortical type has a unique byte identifier. This is used for cortical Identification
    const CORTICAL_AREA_TYPE_KIND: CorticalTypeKind;

    /// Each cortical subtype has a unique byte identifier (amongst that type). This is used for 
    /// cortical Identification
    const CORTICAL_AREA_SUBTYPE_KIND: Self::CorticalSubTypeKindEnum;

    /// What regions is this cortical type allowed within the genome?
    const ALLOWED_REGIONS: CorticalAllowedRegions;

    /// Creates a generator enum given the parameters and layout. Be sure the implementation
    /// verifys the cortical layout!
    fn create_cortical_area_generator<FIQ: FeagiIndexQuantization>(
        parameters: Self::CorticalAreaGeneratorParameters<FIQ>,
        cortical_layout: CorticalLayoutContextOf<FIQ, NM>
    ) -> Result<CTGeneratorTyped<FIQ>, NeuronLayoutError>;

    //region Default impls

    // TODO Cortical Identifier Conversions

    //endregion
}

//endregion

//region Cortical Type Enums

/// A no-nested enum representing the type of cortical area
#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash)]
pub enum CorticalTypeKind{
    Foundation = 0,
    Processing = 1,
}

/// A nested enum representing the type and the subtype of a cortical area
#[derive(Copy, Clone, Debug, Hash)]
pub enum CorticalTypeAndSubtype{
    Foundation(CorticalTypeFoundationSubtypeKind),
    Processing(CorticalTypeProcessingSubtypeKind),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTGeneratorTyped<FIQ: FeagiIndexQuantization>
{
    Foundation(CTGeneratorFoundationSubTyped<FIQ>),
    Processing(CTGeneratorProcessingSubTyped<FIQ>)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTStorageTyped<FIQ: FeagiIndexQuantization>
{
    Foundation(CTStorageFoundationSubTyped<FIQ>),
    Processing(CTStorageProcessingSubTyped<FIQ>)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTStorageOrGeneratorTyped<FIQ: FeagiIndexQuantization>
{
    Foundation(CTStorageOrGeneratorFoundationSubTyped<FIQ>),
    Processing(CTStorageOrGeneratorProcessingSubTyped<FIQ>)
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