pub mod cortical_identifiers;
pub mod foundation;
pub mod processing;
pub mod cortical_types;

use crate::basis::cortical_area::foundation::power::CTFoundationPower;
use crate::basis::cortical_area::foundation::{CTFoundationSubtypedGenerator, CTFoundationSubtypedKind};
#[cfg(feature = "alloc")]
use crate::basis::cortical_area::foundation::{
    CTFoundationSubtypedSpawningParameters, CTFoundationSubtypedStorage, CTFoundationSubtypedStorageOrGenerator,
};
use crate::basis::cortical_area::processing::interconnect::CTProcessingInterconnect;
use crate::basis::cortical_area::processing::{CTProcessingSubtypedGenerator, CTProcessingSubtypedKind};
#[cfg(feature = "alloc")]
use crate::basis::cortical_area::processing::{
    CTProcessingSubtypedSpawningParameters, CTProcessingSubtypedStorage, CTProcessingSubtypedStorageOrGenerator,
};
use cortical_identifiers::inner::CorticalIdentifierInner;
use cortical_identifiers::inner::CorticalIdentifierInnerEncoder;
use feagi_connectome::connectome_basis::neuron_model::neuron_layout::{NeuronLayoutRestrictor, NeuronLayoutTag};
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::connectome_models::neuron_models::models::kind::NMKind;
#[cfg(feature = "alloc")]
use feagi_connectome::connectome_models::neuron_models::models::NMNeurogenesisStorageOrGenerator;
use feagi_connectome::connectome_models::neuron_models::models::{NMModeled, NMNeurogenesisGenerator};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use crate::basis::cortical_area::cortical_identifiers::{CorticalIdentifier, CorticalIdentifierParseError};
use crate::basis::cortical_area::cortical_identifiers::inner::CorticalIdentifierInnerParseError;

//region Cortical Type

/// A cortical type is a higher level description of a cortical area. It has a set layout, but
/// could be a variety of neuron layouts (of that given layout). This is first extended by another
/// trait for the specific cortical type, then implemented by an enum for the cortical subtype,
/// where each variant is a neuron model / quant that subtype supports
pub trait CorticalType: Sized + Copy + Into<NMKind> {
    /// What neuron layout is this cortical type restricted to?
    type NeuronLayout: NeuronLayoutTag;

    /// What restrictions, if any, should be on the neuron layout? Layout types must match
    type NeuronLayoutRestrictor<FIQ: FeagiIndexQuantization>
    : NeuronLayoutRestrictor<FIQ, NeuronLayout = <Self::NeuronLayout as NeuronLayoutTag>::Layout<FIQ>, >;

    /// What method of inner ID encoding is used for this cortical type
    type InnerCorticalIDEncoder: CorticalIdentifierInnerEncoder;

    /// What regions is this cortical type allowed within the genome?
    const ALLOWED_REGIONS: CorticalAllowedRegions;

    /// The default generator for this given cortical type, enumed for the neuron model and quant
    type NeuronModelNeurogenesisGenerator<FIQ: FeagiIndexQuantization>: Into<NMModeled<NMNeurogenesisGenerator<FIQ>>>;
    
    /// The default storage OR generator for this given cortical type, enumed for the neuron model and quant
    #[cfg(feature = "alloc")]
    type NeuronModelNeuroGenesisStorageOrGenerator<FIQ: FeagiIndexQuantization>: Into<NMModeled<NMNeurogenesisStorageOrGenerator<FIQ>>>
        + Serialize
        + DeserializeOwned;

    /// Given a relevant cortical ID Inner, wraps it for this cortical type
    fn wrap_cortical_identifier_inner(inner: CorticalIdentifierInner<Self::InnerCorticalIDEncoder>) -> CorticalIdentifier;

    /// Given a storage or generator, copies it and returns a generator (converts first if its a storage)
    #[cfg(feature = "alloc")]
    fn copy_storage_or_generator_as_generator<FIQ: FeagiIndexQuantization>(
        sog: &Self::NeuronModelNeuroGenesisStorageOrGenerator<FIQ>
    ) -> Self::NeuronModelNeurogenesisGenerator<FIQ>;
    
    //region Default impls
    
    /// Wrap a Cortical Identifier for this Cortical Type, after ensuring validity
    fn wrap_cortical_identifier_from_data_and_context(
        data: <Self::InnerCorticalIDEncoder as CorticalIdentifierInnerEncoder>::ContainedData,
        context: &<Self::InnerCorticalIDEncoder as CorticalIdentifierInnerEncoder>::ContextRequiredToEncode,
    ) -> Result<CorticalIdentifier, CorticalIdentifierParseError> {
        CorticalIdentifier::try_new_id_from_data_and_context::<Self>(data, context)
    }

    /// Try to init from bytes, after ensuring validity
    fn wrap_cortical_identifier_from_bytes(
        bytes: [u8; 6]
    ) -> Result<CorticalIdentifier, CorticalIdentifierParseError>
    {
        CorticalIdentifier::try_new_id_from_bytes::<Self>(bytes)
    }
    
    /// Creates the spawning parameters specific to this cortical type
    #[cfg(feature = "alloc")]
    fn build_spawning_parameters<FIQ: FeagiIndexQuantization>(
        cortical_id_generation_params: <Self::InnerCorticalIDEncoder as CorticalIdentifierInnerEncoder>::ContainedData,
        neurogenesis: Self::NeuronModelNeuroGenesisStorageOrGenerator<FIQ>,
    ) -> CorticalTypeAreaSpawningParameters<FIQ, Self> {
        CorticalTypeAreaSpawningParameters {
            cortical_id_generation_params,
            neurogenesis,
        }
    }

    /// Get what neuron model / quant is being used here but using the generic NMKind enum
    fn into_neuron_model_enum_generic(self) -> NMKind {
        self.into()
    }
    //endregion
}


/// Logic for spawning Cortical areas by their Cortical Type
#[cfg(feature = "alloc")]
pub mod spawning {
    use crate::basis::cortical_area::cortical_identifiers::inner::{CorticalIdentifierCollisionPolicy, CorticalIdentifierInner, CorticalIdentifierInnerEncoder, CorticalIdentifierInnerParseError};
    use crate::basis::cortical_area::cortical_identifiers::{CorticalIdentifier, CorticalIdentifierParseError, FauxCorticalIdentifier};
    use crate::basis::cortical_area::CorticalType;
    use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
    use serde::de::DeserializeOwned;
    use serde::{Deserialize, Serialize};
    use std::fmt::Debug;



    /// Spawning Parameters (of some context) specific to a given cortical type
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(bound = "")]
    pub struct CorticalTypeAreaSpawningParameters<FIQ, CT, CTSC>
    where
        FIQ: FeagiIndexQuantization,
        CT: CorticalType,
        CTSC: CorticalTypeSpawningContext<FIQ, CT>
    {
        pub context: CTSC,
        pub neurogenesis: CT::NeuronModelNeuroGenesisStorageOrGenerator<FIQ>,
    }

    impl<FIQ, CT, CTSC> CorticalTypeAreaSpawningParameters<FIQ, CT, CTSC>
    where
        FIQ: FeagiIndexQuantization,
        CT: CorticalType,
        CTSC: CorticalTypeSpawningContext<FIQ, CT>
    {
        /// Copies the contained neurogenesis and converts it into a generator (if it isnt already)
        pub fn copy_out_generator_for_genome(&self) -> CT::NeuronModelNeurogenesisGenerator<FIQ> {
            CT::copy_storage_or_generator_as_generator(&self.neurogenesis)
        }
    }

    
    /// Unites the different data types of different types / stages of cortical area spawning
    pub trait CorticalTypeSpawningContext<FIQ, CT>: Clone + Debug + Serialize + DeserializeOwned {}
    
    /// Cortical typed spawning parameters at the request stage
    pub struct CorticalTypeSpawningContextRequest<CT: CorticalType> {
        pub faux_cortical_identifier: FauxCorticalIdentifier,
        pub cortical_id_encoder_parameters: <CT::InnerCorticalIDEncoder as CorticalIdentifierInnerEncoder>::ContainedData
    }
    
    impl<CT: CorticalType> CorticalTypeSpawningContextRequest<CT> {

        /// What should we do if we have a collision with an existing ID in the genome
        pub fn get_collision_policy() -> CorticalIdentifierCollisionPolicy {
            <CT::InnerCorticalIDEncoder as CorticalIdentifierInnerEncoder>::COLLISION_RETRY_POLICY
        }
        
        /// Generates the inner so we can use it to check for collisions with a clone and then easily wrap it for validation
        pub fn generate_cortical_identifier_inner(
            &self,
            context: &<CT::InnerCorticalIDEncoder as CorticalIdentifierInnerEncoder>::ContextRequiredToEncode
        ) -> Result<CorticalIdentifierInner<CT::InnerCorticalIDEncoder>, CorticalIdentifierInnerParseError> {
            CorticalIdentifierInner::try_new_from_data_and_context(self.cortical_id_encoder_parameters.clone(), context)
        }
        

    }
    
    /// Once we know a cortical identifier is valid (no collision), we use this to store it for the next step
    pub(crate) struct CorticalTypeSpawningContextValidated<CT: CorticalType> {
        pub verified_cortical_identifier_inner: CorticalIdentifierInner<CT::InnerCorticalIDEncoder>
    }
    
    impl <CT: CorticalType> CorticalTypeSpawningContextValidated<CT> {
        
        /// Consume self and export the cortical Identifier
        pub fn export_cortical_identifier(self) -> CorticalIdentifier {
            CT::wrap_cortical_identifier_inner(self.verified_cortical_identifier_inner)
        }
    }
    
    
}


//region Cortical Type Spawning



/// Spawning Parameters specific to a given cortical type
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct CorticalTypeAreaSpawningParameters<FIQ: FeagiIndexQuantization, CT: CorticalType> {
    pub(crate) cortical_id_generation_params: <CT::InnerCorticalIDEncoder as CorticalIdentifierInnerEncoder>::ContainedData,
    pub(crate) neurogenesis: CT::NeuronModelNeuroGenesisStorageOrGenerator<FIQ>,
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization, CT: CorticalType> CorticalTypeAreaSpawningParameters<FIQ, CT> {
    pub const REGION_ALLOWABILITY: CorticalAllowedRegions = CT::ALLOWED_REGIONS;
}


//endregion

//region Cortical Type Enums


/// A nested enum representing the type and the subtype of a cortical area
#[repr(C, u8)]
#[derive(Copy, Clone, Debug)]
pub enum CorticalTypedKind {
    Foundation(CTFoundationSubtypedKind) = 0,
    Processing(CTProcessingSubtypedKind) = 1,
}

impl CorticalTypedKind {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CorticalTypedGenerator<FIQ: FeagiIndexQuantization> {
    Foundation(CTFoundationSubtypedGenerator<FIQ>),
    Processing(CTProcessingSubtypedGenerator<FIQ>),
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CorticalTypedStorage<FIQ: FeagiIndexQuantization> {
    Foundation(CTFoundationSubtypedStorage<FIQ>),
    Processing(CTProcessingSubtypedStorage<FIQ>),
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CorticalTypedStorageOrGenerator<FIQ: FeagiIndexQuantization> {
    Foundation(CTFoundationSubtypedStorageOrGenerator<FIQ>),
    Processing(CTProcessingSubtypedStorageOrGenerator<FIQ>),
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CorticalTypedSpawningParameters<FIQ: FeagiIndexQuantization> {
    Foundation(CTFoundationSubtypedSpawningParameters<FIQ>),
    Processing(CTProcessingSubtypedSpawningParameters<FIQ>),
}

impl<FIQ: FeagiIndexQuantization> CorticalTypedSpawningParameters<FIQ> {
    pub fn get_allowed_regions(&self) -> CorticalAllowedRegions {
        match &self {
            CorticalTypedSpawningParameters::Foundation(_) => CTFoundationPower::ALLOWED_REGIONS,
            CorticalTypedSpawningParameters::Processing(_) => CTProcessingInterconnect::ALLOWED_REGIONS,
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
            _ => true,
        }
    }

    pub fn is_inner_allowed(&self) -> bool {
        match &self {
            CorticalAllowedRegions::RootOnly => false,
            _ => true,
        }
    }
}
