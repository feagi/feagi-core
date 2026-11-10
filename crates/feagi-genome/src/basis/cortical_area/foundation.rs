use serde::{Deserialize, Serialize};
use crate::basis::cortical_area::{CorticalAllowedRegions, CorticalIdentifier, CorticalIdentifierParseError, CorticalType, CorticalTypeKind};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::{NeuronLayoutRestrictorVoxelBound, NeuronLayoutTagVoxel};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::neuron_models::models::foundation_power::quantizations::{NMFoundationPowerNeurogenesisGeneratorQuanted, NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted, NMFoundationPowerNeurogenesisStorageQuanted, NMFoundationPowerQuantBFloat16, NMFoundationPowerQuantFloat16, NMFoundationPowerQuantFloat32, NMFoundationPowerQuantFloat64};

//region Cortical Subtype

/// These are the core cortical areas of FEAGI, that typically have some unique functionality
/// that are highly specific and possibly integrate with underlying systems. This Cortical Type
/// is unique in that pretty much every subtype has a specific single neuron model for it.
pub trait CorticalTypeFoundation<NM>: CorticalType<
    NM,
    NeuronLayout=NeuronLayoutTagVoxel, // Locked to voxel layout
    IdentifierInner=[u8; 6] // uses 6 bytes as cortical ID inner
>
where
    NM: NeuronModel<NeuronLayoutTag=NeuronLayoutTagVoxel>
{}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash)]
pub enum CorticalTypeFoundationSubtypeKind {
    Power = 0,
}

//endregion

//region Cortical Subtype Enums

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTGeneratorFoundationSubTyped<FIQ: FeagiIndexQuantization>
{
    Power(CTGeneratorFoundationPowerModeled<FIQ>)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTStorageFoundationSubTyped<FIQ: FeagiIndexQuantization>
{
    Power(CTStorageFoundationPowerModeled<FIQ>)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTStorageOrGeneratorFoundationSubTyped<FIQ: FeagiIndexQuantization>
{
    Power(CTStorageOrGeneratorFoundationPowerModeled<FIQ>)
}

//endregion

//region CorticalIdentifier

#[repr(C, u8)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub(in crate::basis) enum CorticalIdentifierFoundation {
    Power([u8; 6]) = 0
}

impl CorticalIdentifierFoundation {

    /// Must be called from `CorticalIdentifier::try_from_bytes` as it assumes it called the
    /// correct inner!
    pub(in crate::basis) fn try_from_cortical_identifier_bytes(bytes: [u8; 8]) -> Result<CorticalIdentifier, CorticalIdentifierParseError>
    {
        debug_assert_eq!(bytes[0], 0);
        Ok(CorticalIdentifier::Foundation(
            match bytes[1] {
                0 => {
                    let [_, _, last_six @ ..] = bytes;
                    CorticalIdentifierFoundation::Power(last_six)
                },
                err => {
                    return Err(CorticalIdentifierParseError::InvalidCorticalSubtype(err))
                }
            }
        ))
    }

}

//endregion

//region Subtypes

//region Power

/// Fires every burst
pub struct CorticalTypeFoundationPower;

impl<NM> CorticalTypeFoundation<NM> for CorticalTypeFoundationPower
where
    NM: NeuronModel<NeuronLayoutTag=NeuronLayoutTagVoxel> + CorticalTypeFoundationPowerSupportedNeuronModels
{

}

impl<NM> CorticalType<NM> for CorticalTypeFoundationPower
where
    NM: NeuronModel<NeuronLayoutTag=NeuronLayoutTagVoxel> + CorticalTypeFoundationPowerSupportedNeuronModels,
{
    type NeuronLayout = NeuronLayoutTagVoxel;
    type NeuronLayoutRestrictor<FIQ: FeagiIndexQuantization> = NeuronLayoutRestrictorVoxelBound<FIQ,
        0, 0, 0, 0, 1, 1, 1, 1 // XYZD min XYZD max
    >;
    type IdentifierInner = [u8; 6];
    type CorticalSubTypeKindEnum = CorticalTypeFoundationSubtypeKind;
    const CORTICAL_AREA_TYPE_KIND: CorticalTypeKind = CorticalTypeKind::Foundation;
    const CORTICAL_AREA_SUBTYPE_KIND: Self::CorticalSubTypeKindEnum = CorticalTypeFoundationSubtypeKind::Power;
    const ALLOWED_REGIONS: CorticalAllowedRegions = CorticalAllowedRegions::RootOrInner;
}

//region lock neuron model support
/// Locks Cortical Type Foundation Subtype Power to the Neuron Models: FoundationPower
trait CorticalTypeFoundationPowerSupportedNeuronModels {}
impl CorticalTypeFoundationPowerSupportedNeuronModels for NMFoundationPowerQuantBFloat16 {}
impl CorticalTypeFoundationPowerSupportedNeuronModels for NMFoundationPowerQuantFloat16 {}
impl CorticalTypeFoundationPowerSupportedNeuronModels for NMFoundationPowerQuantFloat32 {}
impl CorticalTypeFoundationPowerSupportedNeuronModels for NMFoundationPowerQuantFloat64 {}

//endregion

// NOTE: these only have 1 neuron model

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTGeneratorFoundationPowerModeled<FIQ: FeagiIndexQuantization> {
    FoundationPower(NMFoundationPowerNeurogenesisGeneratorQuanted<FIQ>)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTStorageFoundationPowerModeled<FIQ: FeagiIndexQuantization> {
    FoundationPower(NMFoundationPowerNeurogenesisStorageQuanted<FIQ>)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTStorageOrGeneratorFoundationPowerModeled<FIQ: FeagiIndexQuantization> {
    FoundationPower(NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted<FIQ>)
}


//endregion

//endregion
