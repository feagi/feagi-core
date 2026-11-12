use feagi_basis::neurons::{CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutVoxel;
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayout, NeuronLayoutRestrictor};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, FeagiIndexQuantizationStandard};
use feagi_connectome::neuron_models::models::feagi_advanced::quantizations::NMFeagiAdvancedQuantStandard32Bit;
use feagi_connectome::neuron_models::models::feagi_simple::quantizations::NMFeagiSimpleQuantStandard32Bit;
use feagi_connectome::neuron_models::models::exclusive_power::quantizations::{NMFoundationPowerNeurogenesisGeneratorQuanted, NMFoundationPowerQuantFloat16, NMFoundationPowerQuantFloat32};
use feagi_connectome::neuron_models::models::exclusive_power::{
    NMFoundationPowerDataCorticalInternal, NMFoundationPowerDataCorticalProperties, NMFoundationPowerDataCorticalVariance, NMFoundationPowerDataNeuronInternal,
};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::by_generator::NeurogenesisGenerator;
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::NeurogenesisMethod;
use feagi_connectome::neuron_models::models::feagi_advanced::quantizations::NMFeagiAdvancedNeurogenesisGeneratorQuanted;
use feagi_connectome::neuron_models::neurogenesis_generator_uniform_defaults::{NeurogenesisGeneratorUniformDefault, NeurogenesisGeneratorUniformDefaultInitParameters};
use feagi_genome::basis::cortical_area::foundation::{CTGeneratorFoundationPowerModeled, CTGeneratorFoundationSubTyped, CorticalTypeFoundationPower, CorticalTypeFoundationSubtypeKind};
use feagi_genome::basis::cortical_area::processing::{CTGeneratorProcessingInterconnectModeled, CTGeneratorProcessingSubTyped, CorticalTypeProcessingInterconnect, CorticalTypeProcessingSubtypeKind};
use feagi_genome::basis::cortical_area::{CTGeneratorTyped, CorticalAllowedRegions, CorticalSubTypeUsesDefaultCorticalGenerator, CorticalType, CorticalTypeAndSubtype, CorticalTypeKind};

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

type PowerRestrictor = <CorticalTypeFoundationPower as CorticalType<Power32>>::NeuronLayoutRestrictor<Fiq>;
type InterconnectRestrictor = <CorticalTypeProcessingInterconnect as CorticalType<Simple>>::NeuronLayoutRestrictor<Fiq>;

/// Power is a single neuron, so its restrictor only allows dimensions of 1
#[test]
fn power_restricts_its_layout_to_one_neuron() {
    assert!(PowerRestrictor::validate_cortical_context(&dimensions([1, 1, 1, 1])).is_ok());
    assert!(PowerRestrictor::validate_cortical_context(&dimensions([2, 1, 1, 1])).is_err());
    assert!(PowerRestrictor::validate_cortical_context(&dimensions([1, 1, 1, 3])).is_err());

    assert!(NeuronLayoutVoxel::<Fiq>::validate_with_restrictor_and_new::<PowerRestrictor>(dimensions([1, 1, 1, 1])).is_ok());
    assert!(NeuronLayoutVoxel::<Fiq>::validate_with_restrictor_and_new::<PowerRestrictor>(dimensions([4, 4, 1, 1])).is_err());
}

#[test]
fn interconnect_does_not_restrict_its_layout() {
    assert!(InterconnectRestrictor::validate_cortical_context(&dimensions([1, 1, 1, 1])).is_ok());
    assert!(InterconnectRestrictor::validate_cortical_context(&dimensions([8, 4, 2, 3])).is_ok());
}

fn power_generator() -> NeurogenesisGeneratorUniformDefault<Fiq, Power32> {
    NeurogenesisGeneratorUniformDefault::new(
        NeurogenesisGeneratorUniformDefaultInitParameters {
            universal_properties: NeuronModelUniversalCorticalProperties::<Power32> {
                cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
                is_psp_uniform: false,
                is_psp_using_neuron_potential: false,
            },
            cortical_properties: NMFoundationPowerDataCorticalProperties::new(),
            cortical_internal: NMFoundationPowerDataCorticalInternal::new(),
            cortical_variance: NMFoundationPowerDataCorticalVariance::new(),
            neuron_internal: NMFoundationPowerDataNeuronInternal::new(),
        },
        NeuronLayoutVoxel::new(dimensions([1, 1, 1, 1])),
    )
}

//region Generator creation

/// Power has nothing to configure, so the generator of an area is the cortical type's defaults
#[test]
fn power_creates_its_generator_from_its_defaults() {
    let generator = <CorticalTypeFoundationPower as CorticalType<Power32>>::create_cortical_area_generator::<Fiq>((), dimensions([1, 1, 1, 1])).unwrap();
    let CTGeneratorTyped::Foundation(CTGeneratorFoundationSubTyped::Power(CTGeneratorFoundationPowerModeled::FoundationPower(
        NMFoundationPowerNeurogenesisGeneratorQuanted::Float32(inner),
    ))) = generator
    else {
        panic!("created as another cortical type or quantization: {generator:?}")
    };
    assert_eq!(inner, power_generator());
    assert_eq!(inner.iter().len(), 1);
}

#[test]
fn creating_a_generator_checks_the_layout() {
    assert!(<CorticalTypeFoundationPower as CorticalType<Power32>>::create_cortical_area_generator::<Fiq>((), dimensions([2, 1, 1, 1])).is_err());
}

/// The parameters given replace the defaults of the cortical type, and the model chosen decides the
/// variant of the generator
#[test]
fn interconnect_creates_its_generator_from_the_given_parameters() {
    let defaults = <CorticalTypeProcessingInterconnect as CorticalSubTypeUsesDefaultCorticalGenerator<Advanced>>::uniform_default_generator_defaults::<Fiq>();
    let mut properties = defaults.cortical_properties;
    properties.firing_threshold = CorticalAreaNeuronPotential::new(7.0);

    let generator =
        <CorticalTypeProcessingInterconnect as CorticalType<Advanced>>::create_cortical_area_generator::<Fiq>((defaults.universal_properties, properties), dimensions([2, 3, 1, 1])).unwrap();
    let CTGeneratorTyped::Processing(CTGeneratorProcessingSubTyped::Interconnect(CTGeneratorProcessingInterconnectModeled::FeagiAdvanced(
        NMFeagiAdvancedNeurogenesisGeneratorQuanted::Standard32Bit(inner),
    ))) = generator
    else {
        panic!("created as another cortical type or model: {generator:?}")
    };
    assert_eq!(inner.cortical_properties.firing_threshold.deref(), 7.0);
    assert_eq!(inner.iter().len(), 6);
}

#[test]
fn interconnect_creates_a_generator_of_each_model_it_allows() {
    let defaults = <CorticalTypeProcessingInterconnect as CorticalSubTypeUsesDefaultCorticalGenerator<Simple>>::uniform_default_generator_defaults::<Fiq>();
    let generator =
        <CorticalTypeProcessingInterconnect as CorticalType<Simple>>::create_cortical_area_generator::<Fiq>((defaults.universal_properties, defaults.cortical_properties), dimensions([1, 1, 1, 1])).unwrap();
    assert!(matches!(
        generator,
        CTGeneratorTyped::Processing(CTGeneratorProcessingSubTyped::Interconnect(CTGeneratorProcessingInterconnectModeled::FeagiSimple(_)))
    ));
}

//endregion

/// Every level is present even with a single model, so that allowing a second model later does not
/// change how existing data is saved
#[cfg(feature = "alloc")]
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
