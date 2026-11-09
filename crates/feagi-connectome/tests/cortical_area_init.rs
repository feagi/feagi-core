use core::marker::PhantomData;
use feagi_basis::neurons::{CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::{NeuronLayoutTagVoxel, NeuronLayoutVoxel};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutData, NeuronLayoutOf};
use feagi_connectome::connectome_basis::neuron_model::{
    NeurogenesisByGenerator, NeurogenesisDefaultsGenerator, NeurogenesisFromStorage, NeurogenesisMethod, NeurogenesisScratchVarianceGenerator,
    NeurogenesisStorageError,
};
use feagi_connectome::connectome_basis::neuron_model::feagi_advanced::{
    FeagiAdvancedCorticalDataInternal, FeagiAdvancedCorticalDataProperties, FeagiAdvancedCorticalDataVariance,
    FeagiAdvancedNeuronDataInternal, FeagiAdvancedNeuronDataScratch, FeagiAdvancedNeuronDataVariance, NeuronModelFeagiAdvancedDefaultQuant,
    Period,
};
use feagi_connectome::connectome_basis::neuron_model::foundation_power::{
    FoundationPowerCorticalDataInternal, FoundationPowerCorticalDataProperties, FoundationPowerCorticalDataVariance,
    FoundationPowerNeuronDataInternal, FoundationPowerNeuronDataVariance, NeuronModelFoundationPowerFloat32,
};
use feagi_connectome::connectome_basis::quantization_levels::{FeagiIndexQuantization, FeagiIndexQuantizationStandard};
use feagi_connectome_neural::neural_basis::neuron_model::model_data::{CorticalDataInternalSpecification, CorticalDataPropertiesSpecification, CorticalDataScratchSpecification, CorticalDataVarianceSpecification, NeuronDataInternalSpecification, NeuronDataScratchSpecification, NeuronDataVarianceSpecification, NeuronModelUniversalCorticalProperties};
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::MembranePotentialQuantization;
use feagi_connectome_proc::{define_cortical_dynamics_quantization_template, expand_cortical_dynamics_quantization};

type Fiq = FeagiIndexQuantizationStandard;
type Advanced = NeuronModelFeagiAdvancedDefaultQuant;
type Power = NeuronModelFoundationPowerFloat32;

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

fn advanced_variance(x: f32, y: f32, z: f32) -> FeagiAdvancedCorticalDataVariance<Advanced> {
    FeagiAdvancedCorticalDataVariance::new(x, y, z)
}

fn written_offsets(variance: &FeagiAdvancedCorticalDataVariance<Advanced>, layout: &NeuronLayoutVoxel<Fiq>, into: &mut [FeagiAdvancedNeuronDataVariance<Advanced>]) -> Vec<f32> {
    variance.write_neuron_variance_into_slice::<Fiq>(layout, into);
    into.iter().map(|neuron| neuron.firing_threshold_offset).collect()
}

/// With these increments each digit of the offset is one axis of the coordinate, so every coordinate
/// of the area must show up exactly once whichever order the layout walks them in
#[test]
fn variance_offset_comes_from_the_neuron_coordinates() {
    let layout = voxel_layout([2, 3, 1, 1]);
    let mut neurons = vec![FeagiAdvancedNeuronDataVariance::new(0.0); 6];
    let mut offsets = written_offsets(&advanced_variance(1.0, 10.0, 100.0), &layout, &mut neurons);
    offsets.sort_by(f32::total_cmp);
    assert_eq!(offsets, [0.0, 1.0, 10.0, 11.0, 20.0, 21.0]);
}

#[test]
fn runtime_variance_edits_do_not_compound() {
    let layout = voxel_layout([2, 3, 1, 1]);
    let first = advanced_variance(1.0, 10.0, 0.0);
    let second = advanced_variance(5.0, 0.0, 0.0);

    let mut neurons = vec![FeagiAdvancedNeuronDataVariance::new(0.0); 6];
    let original = written_offsets(&first, &layout, &mut neurons);

    let edited = written_offsets(&second, &layout, &mut neurons);
    let mut fresh = vec![FeagiAdvancedNeuronDataVariance::new(0.0); 6];
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
        FeagiAdvancedCorticalDataProperties::new(Period::QUANT_ONE, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0),
        FeagiAdvancedCorticalDataInternal::new(),
        advanced_variance(3.0, 0.0, 0.0),
        vec![FeagiAdvancedNeuronDataInternal::new(Period::QUANT_ZERO); 2],
    )
    .unwrap();
    let mut internal = vec![FeagiAdvancedNeuronDataInternal::new(Period::QUANT_ZERO); 2];
    let mut scratch = vec![FeagiAdvancedNeuronDataScratch::new(); 2];
    let mut variance = vec![FeagiAdvancedNeuronDataVariance::new(-1.0); 2];
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
        FoundationPowerCorticalDataProperties::new(),
        FoundationPowerCorticalDataInternal::new(),
        FoundationPowerCorticalDataVariance::new(),
        vec![FoundationPowerNeuronDataInternal::new(); neuron_count],
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
    assert_eq!(size_of::<FoundationPowerNeuronDataVariance<Power>>(), 0);
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

impl<FIQ: FeagiIndexQuantization, NM: NeuronModelTestCustom> NeurogenesisScratchVarianceGenerator<FIQ, NM> for TestCustomDefaults<FIQ, NM> {
    fn static_get_cortical_scratch(_layout: &NeuronLayoutOf<NM, FIQ>, _cortical_properties: &NM::CorticalDataProperties, _cortical_internal: &NM::CorticalDataInternal) -> NM::CorticalDataScratch {
        TestCustomCorticalDataScratch::new()
    }

    fn static_get_neuron_scratch(_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>, _layout: &NeuronLayoutOf<NM, FIQ>, _internal_neuron_data: &NM::NeuronDataInternal) -> NM::NeuronDataScratch {
        TestCustomNeuronDataScratch::new()
    }

    fn static_get_neuron_variance(index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>, layout: &NeuronLayoutOf<NM, FIQ>, variance: &NM::CorticalDataVariance) -> NM::NeuronDataVariance {
        variance.neuron_variance_at::<FIQ>(layout, index)
    }
}

/// Generic over every quantization level of the model, converting the index to the level's
/// membrane potential type
impl<FIQ: FeagiIndexQuantization, NM: NeuronModelTestCustom> NeurogenesisMethod<FIQ, NM> for IndexAsValueGenerator<FIQ, NM> {
    fn get_layout(&self) -> &NeuronLayoutOf<NM, FIQ> {
        &self.layout
    }

    fn get_universal_cortical_properties(&self) -> NeuronModelUniversalCorticalProperties<NM> {
        self.universal_properties
    }

    fn into_model_cortical_data(self) -> (NM::CorticalDataProperties, NM::CorticalDataInternal) {
        (self.cortical_properties, self.cortical_internal)
    }

    fn get_model_cortical_variance(&self) -> &NM::CorticalDataVariance {
        &self.cortical_variance
    }

    fn spawn_neuron_data_at(&self, index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>) -> (NM::NeuronDataInternal, NM::NeuronDataScratch) {
        use feagi_basis::prelude::{QuantizedDecimalTrait, QuantizedUnsignedIntegerTrait};
        let value = QuantizedDecimalTrait::from_quantization(index.quant_to_usize() as f32);
        let internal = TestCustomNeuronDataInternal::new(value);
        let scratch = NM::NeurogenesisDefaults::<FIQ>::static_get_neuron_scratch(index, &self.layout, &internal);
        (internal, scratch)
    }
}

impl<FIQ: FeagiIndexQuantization, NM: NeuronModelTestCustom> NeurogenesisByGenerator<FIQ, NM> for IndexAsValueGenerator<FIQ, NM> {
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

fn index_as_value_generator(neuron_count: usize) -> IndexAsValueGenerator<Fiq, NeuronModelTestCustomFloat32> {
    IndexAsValueGenerator {
        layout: voxel_layout([neuron_count, 1, 1, 1]),
        universal_properties: universal(),
        cortical_properties: TestCustomCorticalDataProperties::new(),
        cortical_internal: TestCustomCorticalDataInternal::new(),
        cortical_variance: TestCustomCorticalDataVariance::new(),
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
    let storage = NeurogenesisFromStorage::<Fiq, NeuronModelTestCustomFloat32>::new(
        voxel_layout([3, 1, 1, 1]),
        universal(),
        TestCustomCorticalDataProperties::new(),
        TestCustomCorticalDataInternal::new(),
        TestCustomCorticalDataVariance::new(),
        vec![TestCustomNeuronDataInternal::new(7.0); 3],
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

impl<FIQ: FeagiIndexQuantization, NM: NeuronModelTestUniform> NeurogenesisScratchVarianceGenerator<FIQ, NM> for TestUniformDefaults<FIQ, NM> {
    fn static_get_cortical_scratch(_layout: &NeuronLayoutOf<NM, FIQ>, _cortical_properties: &NM::CorticalDataProperties, _cortical_internal: &NM::CorticalDataInternal) -> NM::CorticalDataScratch {
        TestUniformCorticalDataScratch::new()
    }

    fn static_get_neuron_scratch(_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>, _layout: &NeuronLayoutOf<NM, FIQ>, _internal_neuron_data: &NM::NeuronDataInternal) -> NM::NeuronDataScratch {
        TestUniformNeuronDataScratch::new()
    }

    fn static_get_neuron_variance(index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>, layout: &NeuronLayoutOf<NM, FIQ>, variance: &NM::CorticalDataVariance) -> NM::NeuronDataVariance {
        variance.neuron_variance_at::<FIQ>(layout, index)
    }
}

type Uniform = NeuronModelTestUniformFloat32;

fn uniform_generator(dimensions: [usize; 4], value: f32) -> NeurogenesisDefaultsGenerator<Fiq, Uniform> {
    NeurogenesisDefaultsGenerator::new(
        voxel_layout(dimensions),
        universal(),
        TestUniformCorticalDataProperties::new(),
        TestUniformCorticalDataInternal::new(),
        TestUniformCorticalDataVariance::new(),
        TestUniformNeuronDataInternal::new(value),
    )
}

/// Every neuron starts with the same internal data
#[test]
fn defaults_generator_starts_every_neuron_the_same() {
    let generator = uniform_generator([2, 2, 1, 1], 5.0);
    let values: Vec<f32> = generator.iter().map(|(internal, _, _)| internal.value).collect();
    assert_eq!(values, [5.0; 4]);

    let mut internal = vec![TestUniformNeuronDataInternal::new(-1.0); 4];
    let mut scratch = vec![TestUniformNeuronDataScratch::new(); 4];
    let mut variance = vec![TestUniformNeuronDataVariance::new(); 4];
    generator.write_into_slices(&mut internal, &mut scratch, &mut variance);
    assert!(internal.iter().all(|neuron| neuron.value == 5.0));
}

/// Made from storage, the cortical data is kept while every neuron starts from the given value
#[test]
fn defaults_generator_from_storage_keeps_cortical_data_and_resets_neurons() {
    let storage = NeurogenesisFromStorage::<Fiq, Uniform>::new(
        voxel_layout([3, 1, 1, 1]),
        universal(),
        TestUniformCorticalDataProperties::new(),
        TestUniformCorticalDataInternal::new(),
        TestUniformCorticalDataVariance::new(),
        [1.0, 2.0, 3.0].map(TestUniformNeuronDataInternal::new).to_vec(),
    )
    .unwrap();
    let generator = NeurogenesisDefaultsGenerator::new_generator_from_storage(&storage, TestUniformNeuronDataInternal::new(9.0));
    assert_eq!(generator, uniform_generator([3, 1, 1, 1], 9.0));
    let values: Vec<f32> = generator.iter().map(|(internal, _, _)| internal.value).collect();
    assert_eq!(values, [9.0; 3]);
}

/// What a cortical type does: wraps the defaults generator to fill in its own defaults, which a
/// client may edit, then hands it out as the generator of the model
struct ExampleCorticalTypeGenerator<FIQ: FeagiIndexQuantization, NM: NeuronModelTestUniform>(pub NeurogenesisDefaultsGenerator<FIQ, NM>);

impl<FIQ: FeagiIndexQuantization, NM: NeuronModelTestUniform> ExampleCorticalTypeGenerator<FIQ, NM> {
    /// Every neuron of this cortical type starts at 2
    fn new(layout: NeuronLayoutOf<NM, FIQ>) -> Self {
        use feagi_basis::prelude::QuantizedDecimalTrait;
        Self(NeurogenesisDefaultsGenerator::new(
            layout,
            universal(),
            TestUniformCorticalDataProperties::new(),
            TestUniformCorticalDataInternal::new(),
            TestUniformCorticalDataVariance::new(),
            TestUniformNeuronDataInternal::new(QuantizedDecimalTrait::from_quantization(2.0)),
        ))
    }

    fn into_model_generator(self) -> NM::NeurogenesisGenerator<FIQ>
    where
        NM::NeurogenesisGenerator<FIQ>: From<NeurogenesisDefaultsGenerator<FIQ, NM>>,
    {
        self.0.into()
    }
}

#[test]
fn wrapped_defaults_generator_becomes_the_model_generator() {
    let mut wrapped = ExampleCorticalTypeGenerator::<Fiq, Uniform>::new(voxel_layout([3, 1, 1, 1]));
    assert_eq!(wrapped.0, uniform_generator([3, 1, 1, 1], 2.0));
    // A client edits the defaults before the area is created
    wrapped.0.neuron_internal = TestUniformNeuronDataInternal::new(4.0);
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
    use feagi_connectome_neural::burst_engine_cpu::{NeuronCPUParallelDynamics, CpuCorticalAreaInitDispatch, CpuCorticalAreaInitVisitor};
    use feagi_connectome::connectome_basis::neuron_model::feagi_advanced::{BurstEngineDynamics, FeagiAdvancedCorticalDataScratch};
    use feagi_connectome::connectome_basis::neuron_model::NeuronModelEnum;
    use feagi_connectome::connectome_basis::neuron_model::{CorticalAreaInitContainer, CorticalAreaInitEnum, CorticalAreaNeurogenesis};

    /// Builds the neurons of the area, then runs the first through the dynamics the dispatch chose
    struct FireFirstNeuron;

    impl<FIQ: FeagiIndexQuantization> CpuCorticalAreaInitVisitor<FIQ> for FireFirstNeuron {
        type Output = (usize, bool);

        fn visit<NM, D>(self, neurogenesis: CorticalAreaNeurogenesis<FIQ, NM>) -> Self::Output
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
        let init: CorticalAreaInitEnum<Fiq> = NeuronModelEnum::<CorticalAreaInitContainer<Fiq>>::from_data::<Power>(
            CorticalAreaNeurogenesis::FromStorage(power_storage(4).unwrap()),
        );
        // The power model always fires
        assert_eq!(init.dispatch_cpu(FireFirstNeuron), (4, true));
    }

    /// A neuron of the advanced model, with a threshold of 10, half of its charge leaking every
    /// burst, and 2 bursts of refractory period
    struct AdvancedNeuron {
        /// Carried between bursts, as the engine does
        processing_buffer: CorticalAreaNeuronPotential<f32>,
        internal: FeagiAdvancedNeuronDataInternal<Advanced>,
        variance: FeagiAdvancedNeuronDataVariance<Advanced>,
    }

    impl AdvancedNeuron {
        fn new(threshold_offset: f32) -> Self {
            Self {
                processing_buffer: CorticalAreaNeuronPotential::QUANT_ZERO,
                internal: FeagiAdvancedNeuronDataInternal::new(Period::QUANT_ZERO),
                variance: FeagiAdvancedNeuronDataVariance::new(threshold_offset),
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
                    &FeagiAdvancedCorticalDataProperties::new(Period::new(2.0), 10.0, 0.5, 0.0, 1.0, 1.0, 0.0),
                    &FeagiAdvancedCorticalDataInternal::new(),
                    &FeagiAdvancedCorticalDataScratch::new(),
                ),
                (&mut self.internal, &mut FeagiAdvancedNeuronDataScratch::new(), &self.variance),
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
