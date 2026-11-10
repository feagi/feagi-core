use serde::{Deserialize, Serialize};
use crate::basis::cortical_area::{CorticalAllowedRegions, CorticalIdentifier, CorticalIdentifierParseError, CorticalType, CorticalTypeKind};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::{NeuronLayoutRestrictorVoxelUnbound, NeuronLayoutTagVoxel};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::neuron_models::models::feagi_advanced::quantizations::{NMFeagiAdvancedNeurogenesisGeneratorQuanted, NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted, NMFeagiAdvancedNeurogenesisStorageQuanted, NMFeagiAdvancedQuantStandard32Bit};
use feagi_connectome::neuron_models::models::feagi_simple::quantizations::{NMFeagiSimpleNeurogenesisGeneratorQuanted, NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted, NMFeagiSimpleNeurogenesisStorageQuanted, NMFeagiSimpleQuantStandard32Bit};

//region Cortical Subtype

pub trait CorticalTypeProcessing<NM>: CorticalType<
    NM,
    IdentifierInner=[u8; 6] // uses 6 bytes as cortical ID inner
>
where
    NM: NeuronModel<NeuronLayoutTag=Self::NeuronLayout>
{}


#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash)]
pub enum CorticalTypeProcessingSubtypeKind {
    Interconnect = 0,
}

//endregion

//region Cortical Subtype Enums

pub enum CTGeneratorProcessingSubTyped<FIQ: FeagiIndexQuantization>
{
    Interconnect(CTGeneratorProcessingInterconnectModeled<FIQ>)
}

pub enum CTStorageProcessingSubTyped<FIQ: FeagiIndexQuantization>
{
    Interconnect(CTStorageProcessingInterconnectModeled<FIQ>)
}

pub enum CTStorageOrGeneratorProcessingSubTyped<FIQ: FeagiIndexQuantization>
{
    Interconnect(CTStorageOrGeneratorProcessingInterconnectModeled<FIQ>)
}

//endregion

//region CorticalIdentifier

#[repr(C, u8)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorticalIdentifierProcessing {
    Interconnect([u8; 6])
}

impl CorticalIdentifierProcessing {
    /// Must be called from `CorticalIdentifier::try_from_bytes` as it assumes it called the
    /// correct inner!
    pub(crate) fn try_from_cortical_identifier_bytes(bytes: [u8; 8]) -> Result<CorticalIdentifier, CorticalIdentifierParseError>
    {
        debug_assert_eq!(bytes[0], 1);
        Ok(CorticalIdentifier::Processing(
            match bytes[1] {
                0 => {
                    let [_, _, last_six @ ..] = bytes;
                    CorticalIdentifierProcessing::Interconnect(last_six)
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

//region Interconnect

/// General Middle of the line processing neurons
pub struct CorticalTypeProcessingInterconnect;

impl<NM> CorticalTypeProcessing<NM> for CorticalTypeProcessingInterconnect
where
    NM: NeuronModel<NeuronLayoutTag=NeuronLayoutTagVoxel> + CorticalTypeProcessingInterconnectSupportedNeuronModels
{}

impl<NM> CorticalType<NM> for CorticalTypeProcessingInterconnect
where
    NM: NeuronModel<NeuronLayoutTag=NeuronLayoutTagVoxel> + CorticalTypeProcessingInterconnectSupportedNeuronModels
{
    type NeuronLayout = NeuronLayoutTagVoxel;
    type NeuronLayoutRestrictor<FIQ: FeagiIndexQuantization> = NeuronLayoutRestrictorVoxelUnbound<FIQ>;
    type IdentifierInner = [u8; 6];
    type CorticalSubTypeKindEnum = CorticalTypeProcessingSubtypeKind;
    const CORTICAL_AREA_TYPE_KIND: CorticalTypeKind = CorticalTypeKind::Processing;
    const CORTICAL_AREA_SUBTYPE_KIND: Self::CorticalSubTypeKindEnum = CorticalTypeProcessingSubtypeKind::Interconnect;
    const ALLOWED_REGIONS: CorticalAllowedRegions = CorticalAllowedRegions::InnerOnly;
}

//region lock neuron model support
/// Locks Cortical Type Processing Subtype Interconnect to the Neuron Models: FeagiAdvanced, FeagiSimple,
trait CorticalTypeProcessingInterconnectSupportedNeuronModels {}
impl CorticalTypeProcessingInterconnectSupportedNeuronModels for NMFeagiAdvancedQuantStandard32Bit {}
impl CorticalTypeProcessingInterconnectSupportedNeuronModels for NMFeagiSimpleQuantStandard32Bit {}

//endregion

#[derive(Debug, Serialize, Deserialize)]
pub enum CTGeneratorProcessingInterconnectModeled<FIQ: FeagiIndexQuantization> {
    FeagiSimple(NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ>),
    FeagiAdvanced(NMFeagiAdvancedNeurogenesisGeneratorQuanted<FIQ>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CTStorageProcessingInterconnectModeled<FIQ: FeagiIndexQuantization> {
    FeagiSimple(NMFeagiSimpleNeurogenesisStorageQuanted<FIQ>),
    FeagiAdvanced(NMFeagiAdvancedNeurogenesisStorageQuanted<FIQ>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CTStorageOrGeneratorProcessingInterconnectModeled<FIQ: FeagiIndexQuantization> {
    FeagiSimple(NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted<FIQ>),
    FeagiAdvanced(NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted<FIQ>),
}

//endregion

//endregion

