pub mod foundation;
pub mod processing;

use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutData, NeuronLayoutRestrictor, NeuronLayoutTag};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
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



/// Allows specifying where a cortical area of the given type can reside in the genome
#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash, Debug)]
pub enum CorticalAllowedRegions {
    RootOnly,
    InnerOnly,
    RootOrInner, // look, its a bitmask!
}