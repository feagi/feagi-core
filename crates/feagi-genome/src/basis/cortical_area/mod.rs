pub mod foundation;
pub mod processing;
pub mod cortical_identifiers;

use serde::{Deserialize, Serialize};
use crate::basis::cortical_area::foundation::{CTFoundationSubtypedGeneratorPrecursor, CTFoundationSubtypedKind, CTFoundationSubtypedStorage, CTFoundationSubtypedStorageOrGeneratorPrecursor};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{CorticalLayoutContextOf, NeuronLayout, NeuronLayoutOf, NeuronLayoutError, NeuronLayoutRestrictor, NeuronLayoutTag};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::neuron_models::models::kind::{NMKind};
use feagi_connectome::neuron_models::models::{NMNeurogenesisGenerator};
use crate::basis::cortical_area::cortical_identifiers::{ CorticalIdentifierInnerEncoder};
use crate::basis::cortical_area::processing::{CTProcessingSubtypedGeneratorPrecursor, CTProcessingSubtypedKind, CTProcessingSubtypedStorage, CTProcessingSubtypedStorageOrGeneratorPrecursor};

//region Cortical Type

/// A cortical type is a higher level description of a cortical area. It has a set layout, but
/// could be a variety of neuron layouts (of that given layout). This is first extended by another
/// trait for the specific cortical type, then implemented bya  struct for the cortical subtype
pub trait CorticalType: Sized {

    /// What neuron layout is this cortical type restricted to?
    type NeuronLayout: NeuronLayoutTag;

    /// What restrictions, if any, should be on the neuron layout?
    type NeuronLayoutRestrictor<FIQ: FeagiIndexQuantization>:
        NeuronLayoutRestrictor< // layouts must match
            FIQ, NeuronLayout=<Self::NeuronLayout as NeuronLayoutTag>::Layout<FIQ>
    >;

    /// What method of inner ID is used for this cortical type
    type InnerCorticalIDGenerator: CorticalIdentifierInnerEncoder;

    /// What regions is this cortical type allowed within the genome?
    const ALLOWED_REGIONS: CorticalAllowedRegions;


    /// The enum that represents the neuron models that this cortical type can hold
    type PossibleNeuronModelKindEnum: Into<NMKind>;

    /// The default generator for this given cortical model, enumed for the neuron model and quant
    type PossibleNeuronModelNeurogenesisGenerator<FIQ: FeagiIndexQuantization>: Into<NMNeurogenesisGenerator<FIQ>>;



    /// Get what neuron model / quant is being used here, using the enum specific to this cortical type
    fn into_neuron_model_enum(self) ->Self::PossibleNeuronModelKindEnum;

    /// Get the default neurogenesis generator parameters for the current neuron model of this cortical type
    fn get_default_neurogenesis_generator<FIQ, NM>() -> Self::PossibleNeuronModelNeurogenesisGenerator<FIQ>
    where FIQ: FeagiIndexQuantization, NM: NeuronModel
    ;

    //region Default impls

    /// Get what neuron model / quant is being used here but using the generic NMKind enum
    fn into_neuron_model_enum_generic(self) -> NMKind {self.into_neuron_model_enum().into()}
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