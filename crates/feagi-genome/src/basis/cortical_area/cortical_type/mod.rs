pub mod foundation;
pub mod processing;

use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutData, NeuronLayoutRestrictor, NeuronLayoutTag};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::cortical_area::cortical_type::foundation::CorticalIdentifierFoundation;
use crate::basis::cortical_area::cortical_type::processing::CorticalIdentifierProcessing;
use crate::basis::cortical_area::identifier::CorticalAreaInnerID;

/// A cortical type is a higher level description of a cortical area. It has a set layout, but
/// could be a variety of neuron layouts (of that given layout). This is first extended by another
/// trait for the specific cortical type, then implemented bya  struct for the cortical subtype
pub trait CorticalType<NM: NeuronModel<NeuronLayoutTag=Self::NeuronLayout>> {

    /// What neuron layout is this cortical type restricted to?
    type NeuronLayout: NeuronLayoutTag;

    /// What restrictions, if any, should be on the neuron layout?
    type NeuronLayoutRestrictor<FIQ: FeagiIndexQuantization>:
        NeuronLayoutRestrictor< // layouts must match
            FIQ, CorticalLevelContext=<<Self::NeuronLayout as NeuronLayoutTag>::Layout<FIQ> as NeuronLayoutData<FIQ>>::CorticalLevelContext
    >;

    /// What method of inner ID is used for this cortical type
    type IdentifierInner: CorticalAreaInnerID;

    /// The repr(u8) enum that represents the subtype under a given type
    type CorticalSubTypeKindEnum;

    /// Each cortical type has a unique byte identifier. This is used for cortical Identification
    const CORTICAL_AREA_TYPE_KIND: CorticalTypeKind;

    /// Each cortical subtype has a unique byte identifier (amongst that type). This is used for 
    /// cortical Identification
    const CORTICAL_AREA_SUBTYPE_KIND: Self::CorticalSubTypeKindEnum;

    /// What regions is this cortical type allowed within the genome?
    const ALLOWED_REGIONS: CorticalAllowedRegions;

    //region Default impls

    // TODO Cortical Identifier Conversions

    //endregion
}

/// A no-nested enum representing the type of cortical area
#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash, Debug)]
pub enum CorticalTypeKind{
    Foundation = 0,
    Processing = 1,
}

/// A nested enum representing the type and the subtype of a cortical area
#[derive(Copy, Clone, Debug, Hash, Debug)]
pub enum CorticalTypeAndSubtype{
    Foundation(),
    Processing(),
}

impl From<CorticalTypeAndSubtype> for CorticalTypeKind {
    fn from(value: CorticalTypeAndSubtype) -> Self {
        match value {
            CorticalTypeAndSubtype::Foundation() => CorticalTypeKind::Foundation,
            CorticalTypeAndSubtype::Processing() => CorticalTypeKind::Processing,
        }
    }
}

pub(crate) trait CorticalIdentifierInner {
    /// The repr(u8) enum that represents the subtype under a given type
    type CorticalSubTypeKindEnum;

    /// Each cortical subtype has a unique byte identifier (amongst that type). This is used for
    /// cortical Identification
    const CORTICAL_AREA_SUBTYPE_KIND: Self::CorticalSubTypeKindEnum;

    /// Each cortical type has a unique byte identifier. This is used for cortical Identification
    const CORTICAL_AREA_TYPE_KIND: CorticalTypeKind;
}

#[repr(C, u8)]
#[derive(Copy, Clone, Debug)]
pub enum CorticalIdentifier {
    Foundation(CorticalIdentifierFoundation) = 0,
    Processing(CorticalIdentifierProcessing) = 1,
}

impl CorticalIdentifier {

    fn try_from_bytes(bytes: [u8; 8]) -> Result<Self, ()> {
        match bytes[0] {
            0 => CorticalIdentifierFoundation::try_from_cortical_identifier_bytes(bytes),
            1 => CorticalIdentifierProcessing::try_from_cortical_identifier_bytes(bytes),
            _ => Err(())
        }

    }

    /// Directly cast from bytes to this. Does NOT check validity!
    unsafe fn from_bytes_unsafe(bytes: [u8; 8]) -> Self {
        core::mem::transmute(bytes)
    }

    fn into_bytes(self) -> CorticalIdentifierBytes {
        const _: () = assert!(core::mem::size_of::<CorticalIdentifier>() == 8); // compile time check
        let arr: [u8; 8] = unsafe {core::mem::transmute(self)};
        CorticalIdentifierBytes(arr)
    }
}

pub struct CorticalIdentifierBytes(pub(crate) [u8; 8]);




/// Allows specifying where a cortical area of the given type can reside in the genome
#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash, Debug)]
pub enum CorticalAllowedRegions {
    RootOnly,
    InnerOnly,
    RootOrInner, // look, its a bitset!
}