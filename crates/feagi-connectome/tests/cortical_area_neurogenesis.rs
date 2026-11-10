use core::marker::PhantomData;
use feagi_basis::neurons::{CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
use feagi_connectome::connectome_basis::neuron_model::{
    NeurogenesisByGenerator, NeurogenesisFromStorage, NeurogenesisGeneratorUniformDefault, NeurogenesisMethod, NeurogenesisScratchVarianceGenerator,
    NeurogenesisStorageError,
};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::{NeuronLayoutTagVoxel, NeuronLayoutVoxel};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutData, NeuronLayoutOf};
use feagi_connectome::neuron_models::models::::{
    NMFeagiAdvancedDataCorticalInternal, NMFeagiAdvancedDataCorticalProperties, NMFeagiAdvancedDataCorticalVariance,
    NMFeagiAdvancedDataNeuronInternal, NMFeagiAdvancedDataNeuronScratch, NMFeagiAdvancedDataNeuronVariance, NMFeagiAdvancedQuantDefaultQuant,
    Period,
};
use feagi_connectome::neuron_models::models::::{
    NMFoundationPowerDataCorticalInternal, NMFoundationPowerDataCorticalProperties, NMFoundationPowerDataCorticalVariance,
    NMFoundationPowerDataNeuronInternal, NMFoundationPowerDataNeuronVariance, NMFoundationPowerQuantFloat32,
};
use feagi_connectome::connectome_basis::quantization_levels::{FeagiIndexQuantization, FeagiIndexQuantizationStandard};
use feagi_connectome_neural::neural_basis::neuron_model::model_data::{CorticalDataInternalSpecification, CorticalDataPropertiesSpecification, CorticalDataScratchSpecification, CorticalDataVarianceSpecification, NeuronDataInternalSpecification, NeuronDataScratchSpecification, NeuronDataVarianceSpecification, NeuronModelUniversalCorticalProperties};
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::MembranePotentialQuantization;
use feagi_connectome_proc::{define_cortical_dynamics_quantization_template, expand_cortical_dynamics_quantization};

type Fiq = FeagiIndexQuantizationStandard;
type Advanced = NMFeagiAdvancedQuantDefaultQuant;
type Power = NMFoundationPowerQuantFloat32;

fn voxel_layout(dimensions: [usize; 4]) -> NeuronLayoutVoxel<Fiq> {
    NeuronLayoutVoxel::new(DimensionalCorticalAreaDimensions::new_dimensions_from_usize(dimensions).unwrap())
}

fn universal<NM: NeuronModel>() -> NeuronModelUniversalCorticalProperties<NM> {
    NeuronModelUniversalCorticalProperties {
        cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
        is_psp_uniform: false,
        is_psp_using_neuron_potential: false,
    }
}

fn advanced_variance(x: f32, y: f32, z: f32) -> NMFeagiAdvancedDataCorticalVariance<Advanced> {
    NMFeagiAdvancedDataCorticalVariance::new(x, y, z)
}

fn written_offsets(variance: &NMFeagiAdvancedDataCorticalVariance<Advanced>, layout: &NeuronLayoutVoxel<Fiq>, into: &mut [NMFeagiAdvancedDataNeuronVariance<Advanced>]) -> Vec<f32> {
    variance.write_neuron_variance_into_slice::<Fiq>(layout, into);
    into.iter().map(|neuron| neuron.firing_threshold_offset).collect()
}

/// With these increments each digit of the offset is one axis of the coordinate, so every coordinate
/// of the area must show up exactly once whichever order the layout walks them in
#[test]
fn variance_offset_comes_from_the_neuron_coordinates() {
    let layout = voxel_layout([2, 3, 1, 1]);
    let mut neurons = vec![NMFeagiAdvancedDataNeuronVariance::new(0.0); 6];
    let mut offsets = written_offsets(&advanced_variance(1.0, 10.0, 100.0), &layout, &mut neurons);
    offsets.sort_by(f32::total_cmp);
    assert_eq!(offsets, [0.0, 1.0, 10.0, 11.0, 20.0, 21.0]);
}

#[test]
fn runtime_variance_edits_do_not_compound() {
    let layout = voxel_layout([2, 3, 1, 1]);
    let first = advanced_variance(1.0, 10.0, 0.0);
    let second = advanced_variance(5.0, 0.0, 0.0);

    let mut neurons = vec![NMFeagiAdvancedDataNeuronVariance::new(0.0); 6];
    let original = written_offsets(&first, &layout, &mut neurons);

    let edited = written_offsets(&second, &layout, &mut neurons);
    let mut fresh = vec![NMFeagiAdvancedDataNeuronVariance::new(0.0); 6];
    assert_eq!(edited, written_offsets(&second, &layout, &mut fresh));

    // Editing back lands exactly on where it started
    assert_eq!(original, written_offsets(&first, &layout, &mut neurons));
}

#[test]
#[ignore = "needs the FeagiAdvanced neurogenesis defaults filled in (neuron scratch is todo!())"]
fn storage_writes_variance_alongside_neuron_data() {
    let storage = NeurogenesisFromStorage::<Fiq, Advanced>::new(
        voxel_layout([2, 1, 1, 1]),
        universal(),
        NMFeagiAdvancedDataCorticalProperties::new(Period::QUANT_ONE, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0),
        NMFeagiAdvancedDataCorticalInternal::new(),
        advanced_variance(3.0, 0.0, 0.0),
        vec![NMFeagiAdvancedDataNeuronInternal::new(Period::QUANT_ZERO); 2],
    )
    .unwrap();
    let mut internal = vec![NMFeagiAdvancedDataNeuronInternal::new(Period::QUANT_ZERO); 2];
    let mut scratch = vec![NMFeagiAdvancedDataNeuronScratch::new(); 2];
    let mut variance = vec![NMFeagiAdvancedDataNeuronVariance::new(-1.0); 2];
    storage.write_into_slices(&mut internal, &mut scratch, &mut variance);

    let mut offsets: Vec<f32> = variance.iter().map(|neuron| neuron.firing_threshold_offset).collect();
    offsets.sort_by(f32::total_cmp);
    assert_eq!(offsets, [0.0, 3.0]);
    assert_eq!(storage.iter().len(), 2);
    assert_eq!(storage.iter().map(|(_, _, neuron)| neuron).collect::<Vec<_>>(), variance);
}

fn power_storage(neuron_count: usize) -> Result<NeurogenesisFromStorage<Fiq, Power>, NeurogenesisStorageError> {
    NeurogenesisFromStorage::new(
        voxel_layout([2, 2, 1, 1]),
        universal(),
        NMFoundationPowerDataCorticalProperties::new(),
        NMFoundationPowerDataCorticalInternal::new(),
        NMFoundationPowerDataCorticalVariance::new(),
        vec![NMFoundationPowerDataNeuronInternal::new(); neuron_count],
    )
}

#[test]
fn stored_neuron_data_must_match_the_layout() {
    assert!(power_storage(4).is_ok());
    let error = power_storage(3).unwrap_err();
    assert!(matches!(error, NeurogenesisStorageError::InvalidSize));
    assert_eq!(error.to_string(), "Given collection size of NeurogenesisFromStorage does not meet size defined by layout!");
}

/// A model without variance fields gets its variance generated, and holds no per neuron memory
#[test]
fn model_without_variance_generates_nothing() {
    assert_eq!(size_of::<NMFoundationPowerDataNeuronVariance<Power>>(), 0);
    let storage = power_storage(4).unwrap();
    assert_eq!(storage.iter().len(), 4);
}

//region Custom Neurogenesis

/// Generates the scratch and variance data of `TestCustom` neurons, which hold nothing
#[derive(Clone, Debug)]
pub struct TestCustomDefaults<FIQ, NM>(PhantomData<fn() -> (FIQ, NM)>);

/// Each neuron starts with its own index
#[derive(Clone, Debug)]
pub struct IndexAsValueGenerator<FIQ: FeagiIndexQuantization, NM: NeuronModel> {
    layout: NeuronLayoutOf<NM, FIQ>,
    universal_properties: NeuronModelUniversalCorticalProperties<NM>,
    cortical_properties: NM::CorticalDataProperties,
    cortical_internal: NM::CorticalDataInternal,
    cortical_variance: NM::CorticalDataVariance,
}

define_cortical_dynamics_quantization_template! {
    /// Only exists to test custom neurogenesis
    TestCustom

    NeuronLayout: NeuronLayoutTagVoxel

    NeurogenesisDefaults: TestCustomDefaults<FIQ, Self>

    NeurogenesisGenerator: IndexAsValueGenerator<FIQ, Self>

    QuantizationLevels: {
        Float32: {
            MembranePotentialQuant: f32,
        }
    }

    DataSpecifications: {
        NeuronDataInternal: {
            value: MembranePotentialQuant,
        }
    }
}

cortical_dynamics_quantization_test_custom_template!(expand_cortical_dynamics_quantization);

impl<FIQ: FeagiIndexQuantization, NM: NMTestCustomTrait> NeurogenesisScratchVarianceGenerator<FIQ, NM> for TestCustomDefaults<FIQ, NM> {
    fn static_get_cortical_scratch(_layout: &NeuronLayoutOf<NM, FIQ>, _cortical_properties: &NM::CorticalDataProperties, _cortical_internal: &NM::CorticalDataInternal) -> NM::CorticalDataScratch {
        NMTestCustomDataCorticalScratch::new()
    }

    fn static_get_neuron_scratch(_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>, _layout: &NeuronLayoutOf<NM, FIQ>, _internal_neuron_data: &NM::NeuronDataInternal) -> NM::NeuronDataScratch {
        NMTestCustomDataNeuronScratch::new()
    }

    fn static_get_neuron_variance(index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>, layout: &NeuronLayoutOf<NM, FIQ>, variance: &NM::CorticalDataVariance) -> NM::NeuronDataVariance {
        variance.neuron_variance_at::<FIQ>(layout, index)
    }
}

/// Generic over every quantization level of the model, converting the index to the level's
/// membrane potential type
impl<FIQ: FeagiIndexQuantization, NM: NMTestCustomTrait> NeurogenesisMethod<FIQ, NM> for IndexAsValueGenerator<FIQ, NM> {
    fn get_layout(&self) -> &NeuronLayoutOf<NM, FIQ> {
        &self.layout
    }

    fn get_universal_cortical_properties(&self) -> NeuronModelUniversalCorticalProperties<NM> {
        self.universal_properties
    }

    fn into_model_cortical_data(self) -> (NM::CorticalDataProperties, NM::CorticalDataInternal) {
        (self.cortical_properties, self.cortical_internal)
    }

    fn clone_model_cortical_data(&self) -> (NM::CorticalDataProperties, NM::CorticalDataInternal) {
        (self.cortical_properties.clone(), self.cortical_internal.clone())
    }

    fn get_model_cortical_variance(&self) -> &NM::CorticalDataVariance {
        &self.cortical_variance
    }

    fn spawn_neuron_data_at(&self, index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>) -> (NM::NeuronDataInternal, NM::NeuronDataScratch) {
        use feagi_basis::prelude::{QuantizedDecimalTrait, QuantizedUnsignedIntegerTrait};
        let value = QuantizedDecimalTrait::from_quantization(index.quant_to_usize() as f32);
        let internal = NMTestCustomDataNeuronInternal::new(value);
        let scratch = NM::NeurogenesisDefaults::<FIQ>::static_get_neuron_scratch(index, &self.layout, &internal);
        (internal, scratch)
    }
}

impl<FIQ: FeagiIndexQuantization, NM: NMTestCustomTrait> NeurogenesisByGenerator<FIQ, NM> for IndexAsValueGenerator<FIQ, NM> {
    type OtherDataNeededForFromStorage = ();

    /// Regenerates from the stored cortical data, every neuron starting at its index again
    fn new_generator_from_storage(storage: &NeurogenesisFromStorage<FIQ, NM>, _other_data: ()) -> Self {
        let (cortical_properties, cortical_internal) = storage.clone().into_model_cortical_data();
        Self {
            layout: *storage.get_layout(),
            universal_properties: storage.get_universal_cortical_properties(),
            cortical_properties,
            cortical_internal,
            cortical_variance: storage.get_model_cortical_variance().clone(),
        }
    }
}

fn index_as_value_generator(neuron_count: usize) -> IndexAsValueGenerator<Fiq, NMTestCustomQuantFloat32> {
    IndexAsValueGenerator {
        layout: voxel_layout([neuron_count, 1, 1, 1]),
        universal_properties: universal(),
        cortical_properties: NMTestCustomDataCorticalProperties::new(),
        cortical_internal: NMTestCustomDataCorticalInternal::new(),
        cortical_variance: NMTestCustomDataCorticalVariance::new(),
    }
}

#[test]
fn custom_neurogenesis_runs_through_the_spec() {
    let generator = index_as_value_generator(3);
    let values: Vec<f32> = generator.iter().map(|(internal, _, _)| internal.value).collect();
    assert_eq!(values, [0.0, 1.0, 2.0]);
}

/// Stored neuron data is used as is, while the generator made from that storage generates its own
#[test]
fn custom_generator_from_storage_regenerates_neurons() {
    let storage = NeurogenesisFromStorage::<Fiq, NMTestCustomQuantFloat32>::new(
        voxel_layout([3, 1, 1, 1]),
        universal(),
        NMTestCustomDataCorticalProperties::new(),
        NMTestCustomDataCorticalInternal::new(),
        NMTestCustomDataCorticalVariance::new(),
        vec![NMTestCustomDataNeuronInternal::new(7.0); 3],
    )
    .unwrap();
    let stored: Vec<f32> = storage.iter().map(|(internal, _, _)| internal.value).collect();
    assert_eq!(stored, [7.0, 7.0, 7.0]);

    let generator = IndexAsValueGenerator::new_generator_from_storage(&storage, ());
    let generated: Vec<f32> = generator.iter().map(|(internal, _, _)| internal.value).collect();
    assert_eq!(generated, [0.0, 1.0, 2.0]);
}

//endregion

//region Defaults Generator

/// Generates the scratch and variance data of `TestUniform` neurons, which hold nothing
#[derive(Clone, Debug)]
pub struct TestUniformDefaults<FIQ, NM>(PhantomData<fn() -> (FIQ, NM)>);

define_cortical_dynamics_quantization_template! {
    /// Only exists to test the defaults generator, which it uses as its generator
    TestUniform

    NeuronLayout: NeuronLayoutTagVoxel

    NeurogenesisDefaults: TestUniformDefaults<FIQ, Self>

    NeurogenesisGenerator: NeurogenesisDefaultsGenerator<FIQ, Self>

    QuantizationLevels: {
        Float32: {
            MembranePotentialQuant: f32,
        }
    }

    DataSpecifications: {
        NeuronDataInternal: {
            value: MembranePotentialQuant,
        }
    }
}

cortical_dynamics_quantization_test_uniform_template!(expand_cortical_dynamics_quantization);

impl<FIQ: FeagiIndexQuantization, NM: NMTestUniformTrait> NeurogenesisScratchVarianceGenerator<FIQ, NM> for TestUniformDefaults<FIQ, NM> {
    fn static_get_cortical_scratch(_layout: &NeuronLayoutOf<NM, FIQ>, _cortical_properties: &NM::CorticalDataProperties, _cortical_internal: &NM::CorticalDataInternal) -> NM::CorticalDataScratch {
        NMTestUniformDataCorticalScratch::new()
    }

    fn static_get_neuron_scratch(_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>, _layout: &NeuronLayoutOf<NM, FIQ>, _internal_neuron_data: &NM::NeuronDataInternal) -> NM::NeuronDataScratch {
        NMTestUniformDataNeuronScratch::new()
    }

    fn static_get_neuron_variance(index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>, layout: &NeuronLayoutOf<NM, FIQ>, variance: &NM::CorticalDataVariance) -> NM::NeuronDataVariance {
        variance.neuron_variance_at::<FIQ>(layout, index)
    }
}

type Uniform = NMTestUniformQuantFloat32;

fn uniform_generator(dimensions: [usize; 4], value: f32) -> NeurogenesisGeneratorUniformDefault<Fiq, Uniform> {
    NeurogenesisGeneratorUniformDefault::new(
        voxel_layout(dimensions),
        universal(),
        NMTestUniformDataCorticalProperties::new(),
        NMTestUniformDataCorticalInternal::new(),
        NMTestUniformDataCorticalVariance::new(),
        NMTestUniformDataNeuronInternal::new(value),
    )
}

/// Every neuron starts with the same internal data
#[test]
fn defaults_generator_starts_every_neuron_the_same() {
    let generator = uniform_generator([2, 2, 1, 1], 5.0);
    let values: Vec<f32> = generator.iter().map(|(internal, _, _)| internal.value).collect();
    assert_eq!(values, [5.0; 4]);

    let mut internal = vec![NMTestUniformDataNeuronInternal::new(-1.0); 4];
    let mut scratch = vec![NMTestUniformDataNeuronScratch::new(); 4];
    let mut variance = vec![NMTestUniformDataNeuronVariance::new(); 4];
    generator.write_into_slices(&mut internal, &mut scratch, &mut variance);
    assert!(internal.iter().all(|neuron| neuron.value == 5.0));
}

/// Made from storage, the cortical data is kept while every neuron starts from the given value
#[test]
fn defaults_generator_from_storage_keeps_cortical_data_and_resets_neurons() {
    let storage = NeurogenesisFromStorage::<Fiq, Uniform>::new(
        voxel_layout([3, 1, 1, 1]),
        universal(),
        NMTestUniformDataCorticalProperties::new(),
        NMTestUniformDataCorticalInternal::new(),
        NMTestUniformDataCorticalVariance::new(),
        [1.0, 2.0, 3.0].map(NMTestUniformDataNeuronInternal::new).to_vec(),
    )
    .unwrap();
    let generator = NeurogenesisGeneratorUniformDefault::new_generator_from_storage(&storage, NMTestUniformDataNeuronInternal::new(9.0));
    assert_eq!(generator, uniform_generator([3, 1, 1, 1], 9.0));
    let values: Vec<f32> = generator.iter().map(|(internal, _, _)| internal.value).collect();
    assert_eq!(values, [9.0; 3]);
}

/// What a cortical type does: wraps the defaults generator to fill in its own defaults, which a
/// client may edit, then hands it out as the generator of the model
struct ExampleCorticalTypeGenerator<FIQ: FeagiIndexQuantization, NM: NMTestUniformTrait>(pub NeurogenesisGeneratorUniformDefault<FIQ, NM>);

impl<FIQ: FeagiIndexQuantization, NM: NMTestUniformTrait> ExampleCorticalTypeGenerator<FIQ, NM> {
    /// Every neuron of this cortical type starts at 2
    fn new(layout: NeuronLayoutOf<NM, FIQ>) -> Self {
        use feagi_basis::prelude::QuantizedDecimalTrait;
        Self(NeurogenesisGeneratorUniformDefault::new(
            layout,
            universal(),
            NMTestUniformDataCorticalProperties::new(),
            NMTestUniformDataCorticalInternal::new(),
            NMTestUniformDataCorticalVariance::new(),
            NMTestUniformDataNeuronInternal::new(QuantizedDecimalTrait::from_quantization(2.0)),
        ))
    }

    fn into_model_generator(self) -> NM::NeurogenesisGenerator<FIQ>
    where
        NM::NeurogenesisGenerator<FIQ>: From<NeurogenesisGeneratorUniformDefault<FIQ, NM>>,
    {
        self.0.into()
    }
}

#[test]
fn wrapped_defaults_generator_becomes_the_model_generator() {
    let mut wrapped = ExampleCorticalTypeGenerator::<Fiq, Uniform>::new(voxel_layout([3, 1, 1, 1]));
    assert_eq!(wrapped.0, uniform_generator([3, 1, 1, 1], 2.0));
    // A client edits the defaults before the area is created
    wrapped.0.neuron_internal = NMTestUniformDataNeuronInternal::new(4.0);
    let generator: <Uniform as NeuronModel>::NeurogenesisGenerator<Fiq> = wrapped.into_model_generator();
    let values: Vec<f32> = generator.iter().map(|(internal, _, _)| internal.value).collect();
    assert_eq!(values, [4.0; 3]);
}

//endregion

//region Dispatch

#[cfg(feature = "burst-engine-cpu")]
mod dispatch {
    use super::*;
    use feagi_basis::BurstIndex;
    use feagi_connectome_neural::burst_engine_cpu::{CpuCorticalAreaNeurogenesisDispatch, CpuCorticalAreaNeurogenesisVisitor, NeuronCPUParallelDynamics};
    use feagi_connectome::neuron_models::models::::{BurstEngineDynamics, NMFeagiAdvancedDataCorticalScratch};
    use feagi_connectome::connectome_basis::neuron_model::NMEnum;
    use feagi_connectome::connectome_basis::neuron_model::{NeurogenesisFromStorageOrByGeneratorContainer, NeurogenesisFromStorageOrByGeneratorEnum, NeurogenesisFromStorageOrByGeneratorOf};

    /// Builds the neurons of the area, then runs the first through the dynamics the dispatch chose
    struct FireFirstNeuron;

    impl<FIQ: FeagiIndexQuantization> CpuCorticalAreaNeurogenesisVisitor<FIQ> for FireFirstNeuron {
        type Output = (usize, bool);

        fn visit<NM, D>(self, neurogenesis: NeurogenesisFromStorageOrByGeneratorOf<FIQ, NM>) -> Self::Output
        where
            NM: NeuronModel,
            D: NeuronCPUParallelDynamics<FIQ, NM>,
        {
            let mut neurons: Vec<_> = neurogenesis.iter().collect();
            let (internal, scratch, variance) = &mut neurons[0];
            let layout = *neurogenesis.get_layout();
            let (properties, cortical_internal) = neurogenesis.into_model_cortical_data();
            let cortical_scratch = NM::NeurogenesisDefaults::<FIQ>::static_get_cortical_scratch(&layout, &properties, &cortical_internal);
            let mut processing_buffer = CorticalAreaNeuronPotential::QUANT_ZERO;
            let is_firing = D::process_model_neuron_dynamics(
                &mut processing_buffer,
                &BurstIndex::QUANT_ZERO,
                &layout,
                &CorticalAreaNeuronLocalIndex::QUANT_ZERO,
                (&properties, &cortical_internal, &cortical_scratch),
                (internal, scratch, variance),
            );
            (neurons.len(), is_firing)
        }
    }

    #[test]
    #[ignore = "needs the FoundationPower neurogenesis defaults filled in (scratch is todo!())"]
    fn init_enum_dispatches_to_the_dynamics_of_its_model() {
        let neurogenesis: NeurogenesisFromStorageOrByGeneratorEnum<Fiq> = NMEnum::<NeurogenesisFromStorageOrByGeneratorContainer<Fiq>>::from_data::<Power>(
            NeurogenesisFromStorageOrByGeneratorOf::FromStorage(power_storage(4).unwrap()),
        );
        // The power model always fires
        assert_eq!(neurogenesis.dispatch_cpu(FireFirstNeuron), (4, true));
    }

    /// A neuron of the advanced model, with a threshold of 10, half of its charge leaking every
    /// burst, and 2 bursts of refractory period
    struct AdvancedNeuron {
        /// Carried between bursts, as the engine does
        processing_buffer: CorticalAreaNeuronPotential<f32>,
        internal: NMFeagiAdvancedDataNeuronInternal<Advanced>,
        variance: NMFeagiAdvancedDataNeuronVariance<Advanced>,
    }

    impl AdvancedNeuron {
        fn new(threshold_offset: f32) -> Self {
            Self {
                processing_buffer: CorticalAreaNeuronPotential::QUANT_ZERO,
                internal: NMFeagiAdvancedDataNeuronInternal::new(Period::QUANT_ZERO),
                variance: NMFeagiAdvancedDataNeuronVariance::new(threshold_offset),
            }
        }

        /// Integrates the input, returns if the neuron fired and the potential it output
        fn burst(&mut self, incoming: f32) -> (bool, f32) {
            self.processing_buffer = CorticalAreaNeuronPotential::new(self.processing_buffer.dewrap() + incoming);
            let is_firing = <BurstEngineDynamics<Fiq, Advanced> as NeuronCPUParallelDynamics<Fiq, Advanced>>::process_model_neuron_dynamics(
                &mut self.processing_buffer,
                &BurstIndex::QUANT_ZERO,
                &voxel_layout([1, 1, 1, 1]),
                &CorticalAreaNeuronLocalIndex::QUANT_ZERO,
                (
                    &NMFeagiAdvancedDataCorticalProperties::new(Period::new(2.0), 10.0, 0.5, 0.0, 1.0, 1.0, 0.0),
                    &NMFeagiAdvancedDataCorticalInternal::new(),
                    &NMFeagiAdvancedDataCorticalScratch::new(),
                ),
                (&mut self.internal, &mut NMFeagiAdvancedDataNeuronScratch::new(), &self.variance),
            );
            (is_firing, self.processing_buffer.dewrap())
        }
    }

    #[test]
    fn advanced_neuron_leaks_integrates_and_fires_at_threshold() {
        let mut neuron = AdvancedNeuron::new(0.0);
        assert_eq!(neuron.burst(4.0), (false, 2.0));
        assert_eq!(neuron.burst(4.0), (false, 3.0));
        // Exactly reaching the threshold fires, and outputs the firing potential unleaked
        assert_eq!(neuron.burst(7.0), (true, 10.0));
    }

    #[test]
    fn advanced_neuron_decays_without_input() {
        let mut neuron = AdvancedNeuron::new(0.0);
        assert_eq!(neuron.burst(8.0), (false, 4.0));
        assert_eq!(neuron.burst(0.0), (false, 2.0));
        assert_eq!(neuron.burst(0.0), (false, 1.0));
    }

    #[test]
    fn advanced_neuron_clears_and_ignores_input_while_refractory() {
        let mut neuron = AdvancedNeuron::new(0.0);
        assert_eq!(neuron.burst(10.0), (true, 10.0));
        assert_eq!(neuron.burst(100.0), (false, 0.0));
        assert_eq!(neuron.burst(100.0), (false, 0.0));
        // Starts charging from rest again, the charge from before firing and during the
        // refractory period is gone
        assert_eq!(neuron.burst(4.0), (false, 2.0));
    }

    #[test]
    fn advanced_neuron_threshold_includes_its_offset() {
        let mut neuron = AdvancedNeuron::new(5.0);
        assert_eq!(neuron.burst(12.0), (false, 6.0));
        assert_eq!(neuron.burst(9.0), (true, 15.0));
    }
}

//endregion
