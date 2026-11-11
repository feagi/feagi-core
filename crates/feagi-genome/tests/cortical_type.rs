use feagi_basis::neurons::{CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
use feagi_basis::prelude::*;
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutVoxel;
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutData, NeuronLayoutRestrictor};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, FeagiIndexQuantizationStandard};
use feagi_connectome::neuron_models::models::feagi_advanced::quantizations::NMFeagiAdvancedQuantStandard32Bit;
use feagi_connectome::neuron_models::models::feagi_simple::quantizations::NMFeagiSimpleQuantStandard32Bit;
use feagi_connectome::neuron_models::models::foundation_power::quantizations::{NMFoundationPowerNeurogenesisGeneratorQuanted, NMFoundationPowerQuantFloat16, NMFoundationPowerQuantFloat32};
use feagi_connectome::neuron_models::models::foundation_power::{
    NMFoundationPowerDataCorticalInternal, NMFoundationPowerDataCorticalProperties, NMFoundationPowerDataCorticalVariance, NMFoundationPowerDataNeuronInternal,
};
use feagi_connectome::neuron_models::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;
use feagi_genome::basis::cortical_area::foundation::{CTGeneratorFoundationPowerModeled, CTGeneratorFoundationSubTyped, CorticalTypeFoundationPower, CorticalTypeFoundationSubtypeKind};
use feagi_genome::basis::cortical_area::processing::{CorticalTypeProcessingInterconnect, CorticalTypeProcessingSubtypeKind};
use feagi_genome::basis::cortical_area::{CorticalAllowedRegions, CorticalType, CorticalTypeAndSubtype, CorticalTypeKind};

type Fiq = FeagiIndexQuantizationStandard;
type Power32 = NMFoundationPowerQuantFloat32;
type Simple = NMFeagiSimpleQuantStandard32Bit;
type Advanced = NMFeagiAdvancedQuantStandard32Bit;

/// These are the bytes of the cortical identifier, which saved data depends on, so a change here
/// should be deliberate
#[test]
fn kinds_are_the_identifier_bytes() {
    assert_eq!(CorticalTypeKind::Foundation as u8, 0);
    assert_eq!(CorticalTypeKind::Processing as u8, 1);
    assert_eq!(CorticalTypeFoundationSubtypeKind::Power as u8, 0);
    assert_eq!(CorticalTypeProcessingSubtypeKind::Interconnect as u8, 0);
}

#[test]
fn subtype_narrows_to_its_type() {
    let foundation = CorticalTypeKind::from(CorticalTypeAndSubtype::Foundation(CorticalTypeFoundationSubtypeKind::Power));
    assert!(matches!(foundation, CorticalTypeKind::Foundation));
    let processing = CorticalTypeKind::from(CorticalTypeAndSubtype::Processing(CorticalTypeProcessingSubtypeKind::Interconnect));
    assert!(matches!(processing, CorticalTypeKind::Processing));
}

#[test]
fn power_describes_itself() {
    type T = CorticalTypeFoundationPower;
    assert!(matches!(<T as CorticalType<Power32>>::CORTICAL_AREA_TYPE_KIND, CorticalTypeKind::Foundation));
    assert!(matches!(<T as CorticalType<Power32>>::CORTICAL_AREA_SUBTYPE_KIND, CorticalTypeFoundationSubtypeKind::Power));
    assert!(matches!(<T as CorticalType<Power32>>::ALLOWED_REGIONS, CorticalAllowedRegions::RootOrInner));
    // Every quantization of the model is allowed, and describes the type the same way
    assert!(matches!(<T as CorticalType<NMFoundationPowerQuantFloat16>>::CORTICAL_AREA_TYPE_KIND, CorticalTypeKind::Foundation));
}

/// A cortical type may allow several neuron models, which all describe the type the same way
#[test]
fn interconnect_describes_itself_for_every_model_it_allows() {
    type T = CorticalTypeProcessingInterconnect;
    assert!(matches!(<T as CorticalType<Simple>>::CORTICAL_AREA_TYPE_KIND, CorticalTypeKind::Processing));
    assert!(matches!(<T as CorticalType<Advanced>>::CORTICAL_AREA_TYPE_KIND, CorticalTypeKind::Processing));
    assert!(matches!(<T as CorticalType<Simple>>::CORTICAL_AREA_SUBTYPE_KIND, CorticalTypeProcessingSubtypeKind::Interconnect));
    assert!(matches!(<T as CorticalType<Advanced>>::ALLOWED_REGIONS, CorticalAllowedRegions::InnerOnly));
}

fn dimensions(xyzd: [usize; 4]) -> DimensionalCorticalAreaDimensions<<Fiq as FeagiIndexQuantization>::NeuronIndexQuant> {
    DimensionalCorticalAreaDimensions::new_dimensions_from_usize(xyzd).unwrap()
}

/// Power is a single neuron, so its restrictor only allows dimensions of 1
#[test]
fn power_restricts_its_layout_to_one_neuron() {
    let restrictor = <<CorticalTypeFoundationPower as CorticalType<Power32>>::NeuronLayoutRestrictor<Fiq>>::new();
    assert!(restrictor.validate_cortical_context(&dimensions([1, 1, 1, 1])).is_ok());
    assert!(restrictor.validate_cortical_context(&dimensions([2, 1, 1, 1])).is_err());
    assert!(restrictor.validate_cortical_context(&dimensions([1, 1, 1, 3])).is_err());

    assert!(NeuronLayoutVoxel::<Fiq>::validate_with_restrictor_and_new(dimensions([1, 1, 1, 1]), &restrictor).is_ok());
    assert!(NeuronLayoutVoxel::<Fiq>::validate_with_restrictor_and_new(dimensions([4, 4, 1, 1]), &restrictor).is_err());
}

#[test]
fn interconnect_does_not_restrict_its_layout() {
    let restrictor = <<CorticalTypeProcessingInterconnect as CorticalType<Simple>>::NeuronLayoutRestrictor<Fiq>>::new();
    assert!(restrictor.validate_cortical_context(&dimensions([1, 1, 1, 1])).is_ok());
    assert!(restrictor.validate_cortical_context(&dimensions([8, 4, 2, 3])).is_ok());
}

fn power_generator() -> NeurogenesisGeneratorUniformDefault<Fiq, Power32> {
    NeurogenesisGeneratorUniformDefault::new(
        NeuronLayoutVoxel::new(dimensions([1, 1, 1, 1])),
        NeuronModelUniversalCorticalProperties::<Power32> {
            cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
            is_psp_uniform: false,
            is_psp_using_neuron_potential: false,
        },
        NMFoundationPowerDataCorticalProperties::new(),
        NMFoundationPowerDataCorticalInternal::new(),
        NMFoundationPowerDataCorticalVariance::new(),
        NMFoundationPowerDataNeuronInternal::new(),
    )
}

/// Every level is present even with a single model, so that allowing a second model later does not
/// change how existing data is saved
#[test]
fn generator_serializes_by_subtype_then_model_then_quantization() {
    let generator = CTGeneratorFoundationSubTyped::Power(CTGeneratorFoundationPowerModeled::FoundationPower(
        NMFoundationPowerNeurogenesisGeneratorQuanted::Float32(power_generator()),
    ));
    let json = serde_json::to_string(&generator).unwrap();
    assert!(json.starts_with(r#"{"Power":{"FoundationPower":{"Float32":{"layout":"#), "{json}");

    let back: CTGeneratorFoundationSubTyped<Fiq> = serde_json::from_str(&json).unwrap();
    let CTGeneratorFoundationSubTyped::Power(CTGeneratorFoundationPowerModeled::FoundationPower(NMFoundationPowerNeurogenesisGeneratorQuanted::Float32(inner))) = &back else {
        panic!("deserialized as another quantization: {back:?}")
    };
    assert_eq!(inner, &power_generator());
    assert_eq!(<Power32 as NeuronModel>::QUANT_KIND, NMFoundationPowerNeurogenesisGeneratorQuanted::Float32(power_generator()).quant_kind());
}
