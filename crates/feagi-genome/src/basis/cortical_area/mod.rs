pub mod cortical_identifiers;
pub mod foundation;
pub mod processing;
pub mod cortical_types;

use crate::basis::cortical_area::foundation::power::CTFoundationPower;
use crate::basis::cortical_area::foundation::{CTFoundationSubtypedGenerator, CTFoundationSubtypedKind, CTFoundationSubtypedSpawningRequest, CTFoundationSubtypedSpawningValidated};
#[cfg(feature = "alloc")]
use crate::basis::cortical_area::foundation::{
    CTFoundationSubtypedStorage, CTFoundationSubtypedStorageOrGenerator,
};
use crate::basis::cortical_area::processing::interconnect::CTProcessingInterconnect;
use crate::basis::cortical_area::processing::{CTProcessingSubtypedGenerator, CTProcessingSubtypedKind, CTProcessingSubtypedSpawningRequest, CTProcessingSubtypedSpawningValidated};
#[cfg(feature = "alloc")]
use crate::basis::cortical_area::processing::{
    CTProcessingSubtypedStorage, CTProcessingSubtypedStorageOrGenerator,
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
use crate::basis::cortical_area::cortical_identifiers::{CorticalIdentifier, CorticalIdentifierParseError, FauxCorticalIdentifier};
use crate::basis::cortical_area::cortical_identifiers::inner::random_byte_generator::CorticalIDInnerRNG;
use crate::basis::cortical_area::spawning::{CorticalTypeAreaSpawningParameters};
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

    fn wrap_faux_cortical_identifier_inner(inner: CorticalIdentifierInner<Self::InnerCorticalIDEncoder>) -> FauxCorticalIdentifier;

    /// Given a storage or generator, copies it and returns a generator (converts first if its a storage)
    #[cfg(feature = "alloc")]
    fn copy_storage_or_generator_as_generator<FIQ: FeagiIndexQuantization>(
        sog: &Self::NeuronModelNeuroGenesisStorageOrGenerator<FIQ>
    ) -> Self::NeuronModelNeurogenesisGenerator<FIQ>;
    
    //region Default impls
    
    /// Wrap a Cortical Identifier for this Cortical Type, after ensuring validity
    fn wrap_cortical_identifier_from_data_and_context<RNG: CorticalIDInnerRNG>(
        data: <Self::InnerCorticalIDEncoder as CorticalIdentifierInnerEncoder>::ContainedData,
        rng: &RNG,
    ) -> Result<CorticalIdentifier, CorticalIdentifierParseError> {
        CorticalIdentifier::try_new_id_from_data_and_context::<Self, RNG>(data, rng)
    }

    /// Try to init from bytes, after ensuring validity
    fn wrap_cortical_identifier_from_bytes(
        bytes: [u8; 6]
    ) -> Result<CorticalIdentifier, CorticalIdentifierParseError>
    {
        CorticalIdentifier::try_new_id_from_bytes::<Self>(bytes)
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
    use crate::basis::cortical_area::cortical_identifiers::inner::random_byte_generator::CorticalIDInnerRNG;

    pub trait CorticalTypeAreaSpawningParameters<FIQ, CT>
    where
        FIQ: FeagiIndexQuantization,
        CT: CorticalType,
    {    }

    /// Cortical typed spawning parameters at the request stage. Contains the neurogenesis, and
    /// the previously used cortical identifier inner (which also encodes the parameters). This
    /// allows us to relate this ID (as a faux) to mappings in a request, and also allow
    /// an avenue of controlled replacement in the case of a collision
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(bound = "")]
    pub struct CorticalTypeSpawningRequest<FIQ, CT>
    where
        FIQ: FeagiIndexQuantization,
        CT: CorticalType,
    {
        neurogenesis: CT::NeuronModelNeuroGenesisStorageOrGenerator<FIQ>,
        used_cortical_id_inner: CorticalIdentifierInner<CT::InnerCorticalIDEncoder>,
    }



    impl<FIQ, CT> CorticalTypeSpawningRequest<FIQ, CT>
    where
        FIQ: FeagiIndexQuantization,
        CT: CorticalType,
    {
        pub fn new(
            neurogenesis: CT::NeuronModelNeuroGenesisStorageOrGenerator<FIQ>,
            used_cortical_id_inner: CorticalIdentifierInner<CT::InnerCorticalIDEncoder>,
        ) -> Self {
            Self {
                neurogenesis,
                used_cortical_id_inner,
            }
        }

        pub fn get_neurogenesis(&self) -> &CT::NeuronModelNeuroGenesisStorageOrGenerator<FIQ> {
            &self.neurogenesis
        }

        /// Get the used cortical identifier inner as a faux cortical ID
        pub fn copy_used_cortical_id_as_faux(&self) -> FauxCorticalIdentifier {
            CT::wrap_faux_cortical_identifier_inner(self.used_cortical_id_inner.clone())
        }

        /// Copy out the neurogenesis method as a generator, converting it if needed
        pub fn copy_neurogenesis_as_generator(&self) -> CT::NeuronModelNeurogenesisGenerator<FIQ> {
            CT::copy_storage_or_generator_as_generator(&self.neurogenesis)
        }

        /// Converts into a verified spawner once neurogenesis is confirmed valid, cortical properties
        /// are confirmed valid. Optionally, can replace the cortical ID inner with something that
        /// wont collide if required (MUST still have the same internal data!)
        pub(crate) fn into_verified_spawner(
            self, replacing_inner: Option<CorticalIdentifierInner<CT::InnerCorticalIDEncoder>>
        ) -> CorticalTypeSpawningValidated<FIQ, CT> {

            let new_inner = match replacing_inner {
                Some(new) => {
                    // Do not change the inner data state!
                    debug_assert!({
                        let prev_data = self.used_cortical_id_inner.clone().unwrap_as_data();
                        let new_data = new.clone().unwrap_as_data();
                        prev_data == new_data
                    });
                    new
                }
                None => { self.used_cortical_id_inner }
            };

            CorticalTypeSpawningValidated {
                neurogenesis: self.neurogenesis,
                verified_cortical_identifier_inner: new_inner
            }

        }
    }
    
    /// Once we know a cortical identifier is valid (no collision), we use this to store it for the next step
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(bound = "")]
    pub(crate) struct CorticalTypeSpawningValidated<FIQ, CT>
    where
        FIQ: FeagiIndexQuantization,
        CT: CorticalType,
    {
        pub(crate) neurogenesis: CT::NeuronModelNeuroGenesisStorageOrGenerator<FIQ>,
        pub(crate) verified_cortical_identifier_inner: CorticalIdentifierInner<CT::InnerCorticalIDEncoder>
    }

    impl<FIQ, CT> CorticalTypeSpawningValidated<FIQ, CT>
    where
        FIQ: FeagiIndexQuantization,
        CT: CorticalType,
    {

        /// Consume self and export the cortical Identifier
        pub fn export_cortical_identifier(self) -> CorticalIdentifier {
            CT::wrap_cortical_identifier_inner(self.verified_cortical_identifier_inner)
        }
    }



    
}

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
pub enum CorticalTypedSpawningRequest<FIQ: FeagiIndexQuantization> {
    Foundation(CTFoundationSubtypedSpawningRequest<FIQ>),
    Processing(CTProcessingSubtypedSpawningRequest<FIQ>)
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub(crate) enum CorticalTypedSpawningValidated<FIQ: FeagiIndexQuantization> {
    Foundation(CTFoundationSubtypedSpawningValidated<FIQ>),
    Processing(CTProcessingSubtypedSpawningValidated<FIQ>)
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
