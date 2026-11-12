use serde::{Deserialize, Serialize};
use feagi_basis::neurons::CorticalAreaNeuronPotential;
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::by_generator::NeurogenesisGenerator;
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{CorticalLayoutContextOf, NeuronLayout, NeuronLayoutOf, NeuronLayoutError, NeuronLayoutRestrictor};
use crate::basis::cortical_area::{CTGeneratorTyped, CorticalAllowedRegions, CorticalSubTypeUsesDefaultCorticalGenerator, CorticalType, CorticalTypeKind};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::{NeuronLayoutRestrictorVoxelBound, NeuronLayoutTagVoxel};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::neuron_models::models::foundation_power::{NMFoundationPowerDataCorticalInternal, NMFoundationPowerDataCorticalProperties, NMFoundationPowerDataCorticalVariance, NMFoundationPowerDataNeuronInternal};
use feagi_connectome::neuron_models::models::foundation_power::quantizations::{NMFoundationPowerNeurogenesisGeneratorQuanted, NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted, NMFoundationPowerNeurogenesisStorageQuanted, NMFoundationPowerQuantBFloat16, NMFoundationPowerQuantFloat16, NMFoundationPowerQuantFloat32, NMFoundationPowerQuantFloat64};
use feagi_connectome::neuron_models::neurogenesis_generator_uniform_defaults::{NeurogenesisGeneratorUniformDefault, NeurogenesisGeneratorUniformDefaultInitParameters};
use crate::basis::cortical_area::cortical_identifiers::CorticalIdentifierParseError;
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
{

}

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
pub(crate) enum CorticalIdentifierFoundation {
    Power([u8; 6]) = 0
}

impl CorticalIdentifierFoundation {

    /// Must be called from `CorticalIdentifier::try_from_bytes` as it assumes it called the
    /// correct inner!
    pub(crate) fn try_from_cortical_identifier_bytes(bytes: [u8; 8]) -> Result<Self, CorticalIdentifierParseError>
    {
        debug_assert_eq!(bytes[0] % 128, 0);
        Ok(
            match bytes[1] {
                0 => {
                    let [_, _, last_six @ ..] = bytes;
                    CorticalIdentifierFoundation::Power(last_six)
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

//region Power

pub enum

pub struct CorticalTypeFoundationPower

/*
/// Fires every burst
pub struct CorticalTypeFoundationPower;

//region Float16

impl CorticalTypeFoundation<NMFoundationPowerQuantFloat16> for CorticalTypeFoundationPower
{}

impl CorticalType<NMFoundationPowerQuantFloat16> for CorticalTypeFoundationPower
{
    type NeuronLayout = NeuronLayoutTagVoxel;
    type NeuronLayoutRestrictor<FIQ: FeagiIndexQuantization> = NeuronLayoutRestrictorVoxelBound<FIQ,
        1, 1, 1, 1, 2, 2, 2, 2 // XYZD min (inclusive) XYZD max (exclusive), so exactly 1x1x1x1
    >;
    type IdentifierInner = [u8; 6];
    type CorticalAreaGeneratorParameters<FIQ: FeagiIndexQuantization> = ();
    type CorticalSubTypeKindEnum = CorticalTypeFoundationSubtypeKind;
    const CORTICAL_AREA_TYPE_KIND: CorticalTypeKind = CorticalTypeKind::Foundation;
    const CORTICAL_AREA_SUBTYPE_KIND: Self::CorticalSubTypeKindEnum = CorticalTypeFoundationSubtypeKind::Power;
    const ALLOWED_REGIONS: CorticalAllowedRegions = CorticalAllowedRegions::RootOrInner;

    fn create_cortical_area_generator<FIQ: FeagiIndexQuantization>(
        _parameters: Self::CorticalAreaGeneratorParameters<FIQ>,
        cortical_layout: CorticalLayoutContextOf<NMFoundationPowerQuantFloat16,
            FIQ>) -> Result<CTGeneratorTyped<FIQ>, NeuronLayoutError> {

        Self::NeuronLayoutRestrictor::<FIQ>::validate_cortical_context(&cortical_layout)?;
        // Power has nothing to configure, so the generator is entirely its defaults
        let generator_defaults = Self::uniform_default_generator_defaults::<FIQ>();

        let layout = NeuronLayoutOf::<NMFoundationPowerQuantFloat16, FIQ>::new(cortical_layout);

        let generator = NeurogenesisGeneratorUniformDefault::new(
            generator_defaults,
            layout
        );
        Ok (
            CTGeneratorTyped::Foundation(
                CTGeneratorFoundationSubTyped::Power(
                    CTGeneratorFoundationPowerModeled::FoundationPower(
                        NMFoundationPowerNeurogenesisGeneratorQuanted::Float16(
                            generator
                        )
                    )
                )
            )
        )
    }
}

impl CorticalSubTypeUsesDefaultCorticalGenerator<NMFoundationPowerQuantFloat16> for CorticalTypeFoundationPower
{
    fn uniform_default_generator_defaults<FIQ: FeagiIndexQuantization>() -> NeurogenesisGeneratorUniformDefaultInitParameters<NMFoundationPowerQuantFloat16> {
        NeurogenesisGeneratorUniformDefaultInitParameters {
            universal_properties: NeuronModelUniversalCorticalProperties {
                cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
                is_psp_uniform: false,
                is_psp_using_neuron_potential: false,
            },
            cortical_properties: NMFoundationPowerDataCorticalProperties::new(),
            cortical_internal: NMFoundationPowerDataCorticalInternal::new(),
            cortical_variance: NMFoundationPowerDataCorticalVariance::new(),
            neuron_internal: NMFoundationPowerDataNeuronInternal::new(),
        }
    }
}

//endregion


//region BFloat16

impl CorticalTypeFoundation<NMFoundationPowerQuantBFloat16> for CorticalTypeFoundationPower
{}

impl CorticalType<NMFoundationPowerQuantBFloat16> for CorticalTypeFoundationPower
{
    type NeuronLayout = NeuronLayoutTagVoxel;
    type NeuronLayoutRestrictor<FIQ: FeagiIndexQuantization> = NeuronLayoutRestrictorVoxelBound<FIQ,
        1, 1, 1, 1, 2, 2, 2, 2 // XYZD min (inclusive) XYZD max (exclusive), so exactly 1x1x1x1
    >;
    type IdentifierInner = [u8; 6];
    type CorticalAreaGeneratorParameters<FIQ: FeagiIndexQuantization> = ();
    type CorticalSubTypeKindEnum = CorticalTypeFoundationSubtypeKind;
    const CORTICAL_AREA_TYPE_KIND: CorticalTypeKind = CorticalTypeKind::Foundation;
    const CORTICAL_AREA_SUBTYPE_KIND: Self::CorticalSubTypeKindEnum = CorticalTypeFoundationSubtypeKind::Power;
    const ALLOWED_REGIONS: CorticalAllowedRegions = CorticalAllowedRegions::RootOrInner;

    fn create_cortical_area_generator<FIQ: FeagiIndexQuantization>(
        _parameters: Self::CorticalAreaGeneratorParameters<FIQ>,
        cortical_layout: CorticalLayoutContextOf<NMFoundationPowerQuantBFloat16,
            FIQ>) -> Result<CTGeneratorTyped<FIQ>, NeuronLayoutError> {

        Self::NeuronLayoutRestrictor::<FIQ>::validate_cortical_context(&cortical_layout)?;
        // Power has nothing to configure, so the generator is entirely its defaults
        let generator_defaults = Self::uniform_default_generator_defaults::<FIQ>();

        let layout = NeuronLayoutOf::<NMFoundationPowerQuantBFloat16, FIQ>::new(cortical_layout);

        let generator = NeurogenesisGeneratorUniformDefault::new(
            generator_defaults,
            layout
        );
        Ok (
            CTGeneratorTyped::Foundation(
                CTGeneratorFoundationSubTyped::Power(
                    CTGeneratorFoundationPowerModeled::FoundationPower(
                        NMFoundationPowerNeurogenesisGeneratorQuanted::BFloat16(
                            generator
                        )
                    )
                )
            )
        )
    }
}

impl CorticalSubTypeUsesDefaultCorticalGenerator<NMFoundationPowerQuantBFloat16> for CorticalTypeFoundationPower
{
    fn uniform_default_generator_defaults<FIQ: FeagiIndexQuantization>() -> NeurogenesisGeneratorUniformDefaultInitParameters<NMFoundationPowerQuantBFloat16> {
        NeurogenesisGeneratorUniformDefaultInitParameters {
            universal_properties: NeuronModelUniversalCorticalProperties {
                cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
                is_psp_uniform: false,
                is_psp_using_neuron_potential: false,
            },
            cortical_properties: NMFoundationPowerDataCorticalProperties::new(),
            cortical_internal: NMFoundationPowerDataCorticalInternal::new(),
            cortical_variance: NMFoundationPowerDataCorticalVariance::new(),
            neuron_internal: NMFoundationPowerDataNeuronInternal::new(),
        }
    }
}

//endregion


//region Float32

impl CorticalTypeFoundation<NMFoundationPowerQuantFloat32> for CorticalTypeFoundationPower
{}

impl CorticalType<NMFoundationPowerQuantFloat32> for CorticalTypeFoundationPower
{
    type NeuronLayout = NeuronLayoutTagVoxel;
    type NeuronLayoutRestrictor<FIQ: FeagiIndexQuantization> = NeuronLayoutRestrictorVoxelBound<FIQ,
        1, 1, 1, 1, 2, 2, 2, 2 // XYZD min (inclusive) XYZD max (exclusive), so exactly 1x1x1x1
    >;
    type IdentifierInner = [u8; 6];
    type CorticalAreaGeneratorParameters<FIQ: FeagiIndexQuantization> = ();
    type CorticalSubTypeKindEnum = CorticalTypeFoundationSubtypeKind;
    const CORTICAL_AREA_TYPE_KIND: CorticalTypeKind = CorticalTypeKind::Foundation;
    const CORTICAL_AREA_SUBTYPE_KIND: Self::CorticalSubTypeKindEnum = CorticalTypeFoundationSubtypeKind::Power;
    const ALLOWED_REGIONS: CorticalAllowedRegions = CorticalAllowedRegions::RootOrInner;

    fn create_cortical_area_generator<FIQ: FeagiIndexQuantization>(
        _parameters: Self::CorticalAreaGeneratorParameters<FIQ>,
        cortical_layout: CorticalLayoutContextOf<NMFoundationPowerQuantFloat32,
            FIQ>) -> Result<CTGeneratorTyped<FIQ>, NeuronLayoutError> {

        Self::NeuronLayoutRestrictor::<FIQ>::validate_cortical_context(&cortical_layout)?;
        // Power has nothing to configure, so the generator is entirely its defaults
        let generator_defaults = Self::uniform_default_generator_defaults::<FIQ>();

        let layout = NeuronLayoutOf::<NMFoundationPowerQuantFloat32, FIQ>::new(cortical_layout);

        let generator = NeurogenesisGeneratorUniformDefault::new(
            generator_defaults,
            layout
        );
        Ok (
            CTGeneratorTyped::Foundation(
                CTGeneratorFoundationSubTyped::Power(
                    CTGeneratorFoundationPowerModeled::FoundationPower(
                        NMFoundationPowerNeurogenesisGeneratorQuanted::Float32(
                            generator
                        )
                    )
                )
            )
        )
    }
}

impl CorticalSubTypeUsesDefaultCorticalGenerator<NMFoundationPowerQuantFloat32> for CorticalTypeFoundationPower
{
    fn uniform_default_generator_defaults<FIQ: FeagiIndexQuantization>() -> NeurogenesisGeneratorUniformDefaultInitParameters<NMFoundationPowerQuantFloat32> {
        NeurogenesisGeneratorUniformDefaultInitParameters {
            universal_properties: NeuronModelUniversalCorticalProperties {
                cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
                is_psp_uniform: false,
                is_psp_using_neuron_potential: false,
            },
            cortical_properties: NMFoundationPowerDataCorticalProperties::new(),
            cortical_internal: NMFoundationPowerDataCorticalInternal::new(),
            cortical_variance: NMFoundationPowerDataCorticalVariance::new(),
            neuron_internal: NMFoundationPowerDataNeuronInternal::new(),
        }
    }
}

//endregion


//region Float64

impl CorticalTypeFoundation<NMFoundationPowerQuantFloat64> for CorticalTypeFoundationPower
{}

impl CorticalType<NMFoundationPowerQuantFloat64> for CorticalTypeFoundationPower
{
    type NeuronLayout = NeuronLayoutTagVoxel;
    type NeuronLayoutRestrictor<FIQ: FeagiIndexQuantization> = NeuronLayoutRestrictorVoxelBound<FIQ,
        1, 1, 1, 1, 2, 2, 2, 2 // XYZD min (inclusive) XYZD max (exclusive), so exactly 1x1x1x1
    >;
    type IdentifierInner = [u8; 6];
    type CorticalAreaGeneratorParameters<FIQ: FeagiIndexQuantization> = ();
    type CorticalSubTypeKindEnum = CorticalTypeFoundationSubtypeKind;
    const CORTICAL_AREA_TYPE_KIND: CorticalTypeKind = CorticalTypeKind::Foundation;
    const CORTICAL_AREA_SUBTYPE_KIND: Self::CorticalSubTypeKindEnum = CorticalTypeFoundationSubtypeKind::Power;
    const ALLOWED_REGIONS: CorticalAllowedRegions = CorticalAllowedRegions::RootOrInner;

    fn create_cortical_area_generator<FIQ: FeagiIndexQuantization>(
        _parameters: Self::CorticalAreaGeneratorParameters<FIQ>,
        cortical_layout: CorticalLayoutContextOf<NMFoundationPowerQuantFloat64,
            FIQ>) -> Result<CTGeneratorTyped<FIQ>, NeuronLayoutError> {

        Self::NeuronLayoutRestrictor::<FIQ>::validate_cortical_context(&cortical_layout)?;
        // Power has nothing to configure, so the generator is entirely its defaults
        let generator_defaults = Self::uniform_default_generator_defaults::<FIQ>();

        let layout = NeuronLayoutOf::<NMFoundationPowerQuantFloat64, FIQ>::new(cortical_layout);

        let generator = NeurogenesisGeneratorUniformDefault::new(
            generator_defaults,
            layout
        );
        Ok (
            CTGeneratorTyped::Foundation(
                CTGeneratorFoundationSubTyped::Power(
                    CTGeneratorFoundationPowerModeled::FoundationPower(
                        NMFoundationPowerNeurogenesisGeneratorQuanted::Float64(
                            generator
                        )
                    )
                )
            )
        )
    }
}

impl CorticalSubTypeUsesDefaultCorticalGenerator<NMFoundationPowerQuantFloat64> for CorticalTypeFoundationPower
{
    fn uniform_default_generator_defaults<FIQ: FeagiIndexQuantization>() -> NeurogenesisGeneratorUniformDefaultInitParameters<NMFoundationPowerQuantFloat64> {
        NeurogenesisGeneratorUniformDefaultInitParameters {
            universal_properties: NeuronModelUniversalCorticalProperties {
                cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
                is_psp_uniform: false,
                is_psp_using_neuron_potential: false,
            },
            cortical_properties: NMFoundationPowerDataCorticalProperties::new(),
            cortical_internal: NMFoundationPowerDataCorticalInternal::new(),
            cortical_variance: NMFoundationPowerDataCorticalVariance::new(),
            neuron_internal: NMFoundationPowerDataNeuronInternal::new(),
        }
    }
}

//endregion




 */
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
