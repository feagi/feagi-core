use serde::{Deserialize, Serialize};
use feagi_basis::neurons::CorticalAreaNeuronPotential;
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::by_generator::NeurogenesisByGenerator;
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{CorticalLayoutContextOf, NeuronLayoutData, NeuronLayoutOf, NeuronLayoutRestrictionError, NeuronLayoutRestrictor};
use crate::basis::cortical_area::{CTGeneratorTyped, CorticalAllowedRegions, CorticalSubTypeUsesDefaultCorticalGenerator, CorticalType, CorticalTypeKind};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::{NeuronLayoutRestrictorVoxelUnbound, NeuronLayoutTagVoxel};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::neuron_models::models::feagi_advanced::{NMFeagiAdvancedDataCorticalInternal, NMFeagiAdvancedDataCorticalProperties, NMFeagiAdvancedDataCorticalVariance, NMFeagiAdvancedDataNeuronInternal, NMFeagiAdvancedTrait};
use feagi_connectome::neuron_models::models::feagi_advanced::quantizations::{NMFeagiAdvancedNeurogenesisGeneratorQuanted, NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted, NMFeagiAdvancedNeurogenesisStorageQuanted, NMFeagiAdvancedQuantStandard32Bit};
use feagi_connectome::neuron_models::models::feagi_simple::{NMFeagiSimpleDataCorticalInternal, NMFeagiSimpleDataCorticalProperties, NMFeagiSimpleDataCorticalVariance, NMFeagiSimpleDataNeuronInternal, NMFeagiSimpleTrait};
use feagi_connectome::neuron_models::models::feagi_simple::quantizations::{NMFeagiSimpleNeurogenesisGeneratorQuanted, NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted, NMFeagiSimpleNeurogenesisStorageQuanted, NMFeagiSimpleQuantStandard32Bit};
use feagi_connectome::neuron_models::models::{feagi_advanced, feagi_simple, ProxyNMNeuroGenesisFromGenerator};
use feagi_connectome::neuron_models::neurogenesis_generator_uniform_defaults::{NeurogenesisGeneratorUniformDefault, NeurogenesisGeneratorUniformDefaultInitParameters};
use crate::basis::cortical_area::cortical_identifiers::CorticalIdentifierParseError;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTGeneratorProcessingSubTyped<FIQ: FeagiIndexQuantization>
{
    Interconnect(CTGeneratorProcessingInterconnectModeled<FIQ>)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTStorageProcessingSubTyped<FIQ: FeagiIndexQuantization>
{
    Interconnect(CTStorageProcessingInterconnectModeled<FIQ>)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
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
    pub(crate) fn try_from_cortical_identifier_bytes(bytes: [u8; 8]) -> Result<Self, CorticalIdentifierParseError>
    {
        debug_assert_eq!(bytes[0] % 128, 1);
        Ok(
            match bytes[1] {
                0 => {
                    let [_, _, last_six @ ..] = bytes;
                    CorticalIdentifierProcessing::Interconnect(last_six)
                },
                err => {
                    return Err(CorticalIdentifierParseError::InvalidCorticalSubtype(err))
                }
            }
        )
    }
}

//endregion

//region Subtypes

//region Interconnect

/// General Middle of the line processing neurons
pub struct CorticalTypeProcessingInterconnect;

//region Feagi Advanced

impl CorticalTypeProcessing<NMFeagiAdvancedQuantStandard32Bit> for CorticalTypeProcessingInterconnect
{}

impl CorticalType<NMFeagiAdvancedQuantStandard32Bit> for CorticalTypeProcessingInterconnect
{
    type NeuronLayout = NeuronLayoutTagVoxel;
    type NeuronLayoutRestrictor<FIQ: FeagiIndexQuantization> = NeuronLayoutRestrictorVoxelUnbound<FIQ>;
    type IdentifierInner = [u8; 6];
    type CorticalAreaGeneratorParameters<FIQ: FeagiIndexQuantization> = (
        NeuronModelUniversalCorticalProperties<NMFeagiAdvancedQuantStandard32Bit>,
        <NMFeagiAdvancedQuantStandard32Bit as NeuronModel>::CorticalDataProperties
    );
    type CorticalSubTypeKindEnum = CorticalTypeProcessingSubtypeKind;
    const CORTICAL_AREA_TYPE_KIND: CorticalTypeKind = CorticalTypeKind::Processing;
    const CORTICAL_AREA_SUBTYPE_KIND: Self::CorticalSubTypeKindEnum = CorticalTypeProcessingSubtypeKind::Interconnect;

    const ALLOWED_REGIONS: CorticalAllowedRegions = CorticalAllowedRegions::InnerOnly;

    fn create_cortical_area_generator<FIQ: FeagiIndexQuantization>(
        parameters: Self::CorticalAreaGeneratorParameters<FIQ>,
        cortical_layout: CorticalLayoutContextOf<NMFeagiAdvancedQuantStandard32Bit,
            FIQ>) -> Result<CTGeneratorTyped<FIQ>, NeuronLayoutRestrictionError> {

        Self::NeuronLayoutRestrictor::<FIQ>::validate_cortical_context(&cortical_layout)?;
        let mut generator_defaults = Self::uniform_default_generator_defaults::<FIQ>();
        generator_defaults.universal_properties = parameters.0;
        generator_defaults.cortical_properties = parameters.1;

        let layout = NeuronLayoutOf::<NMFeagiAdvancedQuantStandard32Bit, FIQ>::new(cortical_layout);

        let generator = NeurogenesisGeneratorUniformDefault::new(
            generator_defaults,
            layout
        );
        Ok (
            CTGeneratorTyped::Processing(
                CTGeneratorProcessingSubTyped::Interconnect(
                    CTGeneratorProcessingInterconnectModeled::FeagiAdvanced(
                        NMFeagiAdvancedNeurogenesisGeneratorQuanted::Standard32Bit(
                            generator
                        )
                    )
                )
            )
        )
    }
}

impl CorticalSubTypeUsesDefaultCorticalGenerator<NMFeagiAdvancedQuantStandard32Bit> for CorticalTypeProcessingInterconnect

{
    fn uniform_default_generator_defaults<FIQ: FeagiIndexQuantization>() -> NeurogenesisGeneratorUniformDefaultInitParameters<NMFeagiAdvancedQuantStandard32Bit> {
        NeurogenesisGeneratorUniformDefaultInitParameters {
            universal_properties: NeuronModelUniversalCorticalProperties {
                cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
                is_psp_uniform: false,
                is_psp_using_neuron_potential: false,
            },
            cortical_properties: NMFeagiAdvancedDataCorticalProperties{
                refractory_period: feagi_advanced::Period::QUANT_ONE,
                firing_threshold: CorticalAreaNeuronPotential::QUANT_ONE,
                leak_coefficient: CorticalAreaNeuronPotential::QUANT_ZERO,
                leak_variability: CorticalAreaNeuronPotential::QUANT_ZERO,
                postsynaptic_current: CorticalAreaNeuronPotential::QUANT_ZERO,
                postsynaptic_current_max: CorticalAreaNeuronPotential::QUANT_ONE,
                degeneration: CorticalAreaNeuronPotential::QUANT_ZERO,
            },
            cortical_internal: NMFeagiAdvancedDataCorticalInternal::new(),
            cortical_variance: NMFeagiAdvancedDataCorticalVariance::new(
                CorticalAreaNeuronPotential::QUANT_ZERO,
                CorticalAreaNeuronPotential::QUANT_ZERO,
                CorticalAreaNeuronPotential::QUANT_ZERO
            ),
            neuron_internal: NMFeagiAdvancedDataNeuronInternal {
                refractory_countdown: feagi_advanced::Period::QUANT_ONE,
            },
        }
    }
}


//endregion


//region FeagiSimple

impl CorticalTypeProcessing<NMFeagiSimpleQuantStandard32Bit> for CorticalTypeProcessingInterconnect
{}

impl CorticalType<NMFeagiSimpleQuantStandard32Bit> for CorticalTypeProcessingInterconnect
{
    type NeuronLayout = NeuronLayoutTagVoxel;
    type NeuronLayoutRestrictor<FIQ: FeagiIndexQuantization> = NeuronLayoutRestrictorVoxelUnbound<FIQ>;
    type IdentifierInner = [u8; 6];
    type CorticalAreaGeneratorParameters<FIQ: FeagiIndexQuantization> = (
        NeuronModelUniversalCorticalProperties<NMFeagiSimpleQuantStandard32Bit>,
        <NMFeagiSimpleQuantStandard32Bit as NeuronModel>::CorticalDataProperties
    );
    type CorticalSubTypeKindEnum = CorticalTypeProcessingSubtypeKind;
    const CORTICAL_AREA_TYPE_KIND: CorticalTypeKind = CorticalTypeKind::Processing;
    const CORTICAL_AREA_SUBTYPE_KIND: Self::CorticalSubTypeKindEnum = CorticalTypeProcessingSubtypeKind::Interconnect;
    const ALLOWED_REGIONS: CorticalAllowedRegions = CorticalAllowedRegions::InnerOnly;

    fn create_cortical_area_generator<FIQ: FeagiIndexQuantization>(
        parameters: Self::CorticalAreaGeneratorParameters<FIQ>,
        cortical_layout: CorticalLayoutContextOf<NMFeagiSimpleQuantStandard32Bit,
            FIQ>) -> Result<CTGeneratorTyped<FIQ>, NeuronLayoutRestrictionError> {

        Self::NeuronLayoutRestrictor::<FIQ>::validate_cortical_context(&cortical_layout)?;
        let mut generator_defaults = Self::uniform_default_generator_defaults::<FIQ>();
        generator_defaults.universal_properties = parameters.0;
        generator_defaults.cortical_properties = parameters.1;

        let layout = NeuronLayoutOf::<NMFeagiAdvancedQuantStandard32Bit, FIQ>::new(cortical_layout);

        let generator = NeurogenesisGeneratorUniformDefault::new(
            generator_defaults,
            layout
        );
        Ok (
            CTGeneratorTyped::Processing(
                CTGeneratorProcessingSubTyped::Interconnect(
                    CTGeneratorProcessingInterconnectModeled::FeagiSimple(
                        NMFeagiSimpleNeurogenesisGeneratorQuanted::Standard32Bit(
                            generator
                        )
                    )
                )
            )
        )
    }
}

impl CorticalSubTypeUsesDefaultCorticalGenerator<NMFeagiSimpleQuantStandard32Bit> for CorticalTypeProcessingInterconnect

{
    fn uniform_default_generator_defaults<FIQ: FeagiIndexQuantization>() -> NeurogenesisGeneratorUniformDefaultInitParameters<NMFeagiSimpleQuantStandard32Bit> {
        NeurogenesisGeneratorUniformDefaultInitParameters {
            universal_properties: NeuronModelUniversalCorticalProperties {
                cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
                is_psp_uniform: false,
                is_psp_using_neuron_potential: false,
            },
            cortical_properties: NMFeagiSimpleDataCorticalProperties {
                refractory_period: feagi_simple::Period::QUANT_ONE,
                firing_threshold: CorticalAreaNeuronPotential::QUANT_ONE,
                postsynaptic_current: CorticalAreaNeuronPotential::QUANT_ZERO,
                postsynaptic_current_max: CorticalAreaNeuronPotential::QUANT_ZERO,
            },
            cortical_internal: NMFeagiSimpleDataCorticalInternal::new(),
            cortical_variance: NMFeagiSimpleDataCorticalVariance::new(),
            neuron_internal: NMFeagiSimpleDataNeuronInternal {
                refractory_countdown: feagi_simple::Period::QUANT_ONE,
            },
        }
    }
}

//endregion

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTGeneratorProcessingInterconnectModeled<FIQ: FeagiIndexQuantization> {
    FeagiSimple(NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ>),
    FeagiAdvanced(NMFeagiAdvancedNeurogenesisGeneratorQuanted<FIQ>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTStorageProcessingInterconnectModeled<FIQ: FeagiIndexQuantization> {
    FeagiSimple(NMFeagiSimpleNeurogenesisStorageQuanted<FIQ>),
    FeagiAdvanced(NMFeagiAdvancedNeurogenesisStorageQuanted<FIQ>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTStorageOrGeneratorProcessingInterconnectModeled<FIQ: FeagiIndexQuantization> {
    FeagiSimple(NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted<FIQ>),
    FeagiAdvanced(NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted<FIQ>),
}

//endregion

//endregion

