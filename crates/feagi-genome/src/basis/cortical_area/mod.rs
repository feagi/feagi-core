pub mod foundation;
pub mod processing;
pub mod cortical_identifiers;

use serde::{Deserialize, Serialize};
use crate::basis::cortical_area::foundation::{CTFoundationSubtypedGeneratorPrecursor, CTFoundationSubtypedKind, CTFoundationSubtypedStorage, CTFoundationSubtypedStorageOrGeneratorPrecursor};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{CorticalLayoutContextOf, NeuronLayout, NeuronLayoutOf, NeuronLayoutError, NeuronLayoutRestrictor, NeuronLayoutTag};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::neuron_models::models::kind::NMQuantedKind;
use feagi_connectome::neuron_models::models::{NMNeurogenesisGeneratorModeled, NMNeurogenesisGeneratorPrecursorModeled, NMNeurogenesisStorageModeled, NMNeurogenesisStorageOrGeneratorModeled};
use feagi_connectome::neuron_models::neurogenesis_generator_uniform_defaults::{NeurogenesisGeneratorUniformDefault};
use crate::basis::cortical_area::cortical_identifiers::{CorticalAreaInnerID, CorticalIdentifier, CorticalIdentifierGenerator};
use crate::basis::cortical_area::processing::{CTProcessingSubtypedGeneratorPrecursor, CTProcessingSubtypedKind, CTProcessingSubtypedStorage, CTProcessingSubtypedStorageOrGeneratorPrecursor};

//region Cortical Type

/// A cortical type is a higher level description of a cortical area. It has a set layout, but
/// could be a variety of neuron layouts (of that given layout). This is first extended by another
/// trait for the specific cortical type, then implemented bya  struct for the cortical subtype
pub trait CorticalType {

    /// The enums that represents the kinds of neuron models data that this cortical type can hold
    type PossibleNeuronModelKindsEnum: Into<NMQuantedKind>;
    type PossibleNeuronModelGeneratorEnum<FIQ: FeagiIndexQuantization>: Into<NMNeurogenesisGeneratorModeled<FIQ>>;
    type PossibleNeuronModelGeneratorPrecursorEnum<FIQ: FeagiIndexQuantization>: Into<NMNeurogenesisGeneratorPrecursorModeled<FIQ>>;
    type PossibleNeuronModelStorageEnum<FIQ: FeagiIndexQuantization>: Into<NMNeurogenesisStorageModeled<FIQ>>;
    type PossibleNeuronModelStorageOrGeneratorEnum<FIQ: FeagiIndexQuantization>: Into<NMNeurogenesisStorageOr<FIQ>>;

    /// What neuron layout is this cortical type restricted to?
    type NeuronLayout: NeuronLayoutTag;

    /// What restrictions, if any, should be on the neuron layout?
    type NeuronLayoutRestrictor<FIQ: FeagiIndexQuantization>:
        NeuronLayoutRestrictor< // layouts must match
            FIQ, NeuronLayout=<Self::NeuronLayout as NeuronLayoutTag>::Layout<FIQ>
    >;

    /// What method of inner ID is used for this cortical type
    type CorticalIdentifierInner: CorticalAreaInnerID;

    /// How is the Inner Cortical Identifier Generated?
    type CorticalIdentifierGenerator: CorticalIdentifierGenerator<InnerID=Self::CorticalIdentifierInner>;

    /// What regions is this cortical type allowed within the genome?
    const ALLOWED_REGIONS: CorticalAllowedRegions;

    /// Get what neuron model / quant is being used here
    fn get_neuron_model_enum(&self) ->Self::PossibleNeuronModelKindsEnum;

    /// Generates the precursor structure for the generator. Note that the NM will be more restricted
    /// in the implementation
    fn get_neurogenesis_generator_precursor<FIQ>() -> NM::NeurogenesisGeneratorPrecursor<FIQ>
    where
        FIQ: FeagiIndexQuantization,

    /// Generates a cortical identifier with the inbuilt rules
    fn generate_cortical_identifier(
        context: &mut <Self::CorticalIdentifierGenerator as CorticalIdentifierGenerator>::ContextRequiredToEncode
    ) -> CorticalIdentifier;

    //region Default impls




    //endregion
}


//region Cortical Type Enums

/// A nested enum representing the type and the subtype of a cortical area
#[repr(C, u8)]
#[derive(Copy, Clone, Debug)]
pub enum CorticalTypedKind{
    Foundation(CTFoundationSubtypedKind) = 0,
    Processing(CTProcessingSubtypedKind) = 1,
}

impl CorticalTypedKind {

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

//endregion

//endregion


/// Allows specifying where a cortical area of the given type can reside in the genome
#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash)]
pub enum CorticalAllowedRegions {
    RootOnly,
    InnerOnly,
    RootOrInner, // look, its a bitset!
}