pub mod foundation;
pub mod processing;
pub mod cortical_identifiers;

use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use crate::basis::cortical_area::foundation::{CTFoundationSubtypedGenerator, CTFoundationSubtypedKind};
#[cfg(feature = "alloc")]
use crate::basis::cortical_area::foundation::{CTFoundationSubtypedSpawningParameters, CTFoundationSubtypedStorage, CTFoundationSubtypedStorageOrGenerator};
use feagi_connectome::connectome_basis::neuron_model::neuron_layout::{NeuronLayoutRestrictor, NeuronLayoutTag};
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::connectome_models::neuron_models::models::kind::{NMKind};
use feagi_connectome::connectome_models::neuron_models::models::{NMModeled, NMNeurogenesisGenerator};
#[cfg(feature = "alloc")]
use feagi_connectome::connectome_models::neuron_models::models::NMNeurogenesisStorageOrGenerator;
use crate::basis::cortical_area::cortical_identifiers::{ CorticalIdentifierInnerEncoder};
use crate::basis::cortical_area::foundation::power::CTFoundationPower;
use crate::basis::cortical_area::processing::{CTProcessingSubtypedGenerator, CTProcessingSubtypedKind};
#[cfg(feature = "alloc")]
use crate::basis::cortical_area::processing::{CTProcessingSubtypedSpawningParameters, CTProcessingSubtypedStorage, CTProcessingSubtypedStorageOrGenerator};
use crate::basis::cortical_area::processing::interconnect::CTProcessingInterconnect;
//region Cortical Type

/// A cortical type is a higher level description of a cortical area. It has a set layout, but
/// could be a variety of neuron layouts (of that given layout). This is first extended by another
/// trait for the specific cortical type, then implemented by an enum for the cortical subtype,
/// where each variant is a neuron model / quant that subtype supports
pub trait CorticalType: Sized + Copy + Into<NMKind> {

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


    /// The default generator for this given cortical type, enumed for the neuron model and quant
    type PossibleNeuronModelNeurogenesisGenerator<FIQ: FeagiIndexQuantization>: Into<NMModeled<NMNeurogenesisGenerator<FIQ>>>;

    /// The default storage OR generator for this given cortical type, enumed for the neuron model and quant
    #[cfg(feature = "alloc")]
    type PossibleNeuronModelNeuroGenesisStorageOrGenerator<FIQ: FeagiIndexQuantization>: Into<NMModeled<NMNeurogenesisStorageOrGenerator<FIQ>>> + Serialize + DeserializeOwned;

    //region Default impls

    /// Creates the spawning parameters specific to this cortical type
    #[cfg(feature = "alloc")]
    fn build_spawning_parameters<FIQ: FeagiIndexQuantization>(
        cortical_id_generation_params: <Self::InnerCorticalIDGenerator as CorticalIdentifierInnerEncoder>::DecodableData,
        neurogenesis: Self::PossibleNeuronModelNeuroGenesisStorageOrGenerator<FIQ>
    ) -> CorticalTypeAreaSpawningParameters<FIQ, Self> {
        CorticalTypeAreaSpawningParameters {
            cortical_id_generation_params,
            neurogenesis,
        }
    }
    
    /// Get what neuron model / quant is being used here but using the generic NMKind enum
    fn into_neuron_model_enum_generic(self) -> NMKind {self.into()}
    //endregion
}


/// Spawning Parameters specific to a given cortical type
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct CorticalTypeAreaSpawningParameters<FIQ: FeagiIndexQuantization, CT: CorticalType> {
    pub(crate) cortical_id_generation_params: <CT::InnerCorticalIDGenerator as CorticalIdentifierInnerEncoder>::DecodableData,
    pub(crate) neurogenesis: CT::PossibleNeuronModelNeuroGenesisStorageOrGenerator<FIQ>
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization, CT: CorticalType> CorticalTypeAreaSpawningParameters<FIQ, CT> {
    pub const REGION_ALLOWABILITY: CorticalAllowedRegions = CT::ALLOWED_REGIONS;
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
pub enum CorticalTypedGenerator<FIQ: FeagiIndexQuantization>
{
    Foundation(CTFoundationSubtypedGenerator<FIQ>),
    Processing(CTProcessingSubtypedGenerator<FIQ>)
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CorticalTypedStorage<FIQ: FeagiIndexQuantization>
{
    Foundation(CTFoundationSubtypedStorage<FIQ>),
    Processing(CTProcessingSubtypedStorage<FIQ>)
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CorticalTypedStorageOrGenerator<FIQ: FeagiIndexQuantization>
{
    Foundation(CTFoundationSubtypedStorageOrGenerator<FIQ>),
    Processing(CTProcessingSubtypedStorageOrGenerator<FIQ>)
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CorticalTypedSpawningParameters<FIQ: FeagiIndexQuantization> {
    Foundation(CTFoundationSubtypedSpawningParameters<FIQ>),
    Processing(CTProcessingSubtypedSpawningParameters<FIQ>)
}

impl<FIQ: FeagiIndexQuantization> CorticalTypedSpawningParameters<FIQ> {
    pub fn get_allowed_regions(&self) -> CorticalAllowedRegions {
        match &self {
            CorticalTypedSpawningParameters::Foundation(_) => {CTFoundationPower::ALLOWED_REGIONS}
            CorticalTypedSpawningParameters::Processing(_) => {CTProcessingInterconnect::ALLOWED_REGIONS}
        }
    }
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

impl CorticalAllowedRegions {
    pub fn is_root_allowed(&self) -> bool {
        match &self {
            CorticalAllowedRegions::InnerOnly => false,
            _ => true
        }
    }

    pub fn is_inner_allowed(&self) -> bool {
        match &self {
            CorticalAllowedRegions::RootOnly => false,
            _ => true
        }
    }
}