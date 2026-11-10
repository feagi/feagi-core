use feagi_basis::neurons::{CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
use feagi_basis::prelude::*;
use feagi_connectome::neuron_models::models::feagi_advanced::quantizations::NMFeagiAdvancedQuantStandard32Bit;
use feagi_connectome::neuron_models::models::feagi_advanced::{
    NMFeagiAdvancedDataCorticalInternal, NMFeagiAdvancedDataCorticalProperties, NMFeagiAdvancedDataCorticalVariance,
    NMFeagiAdvancedDataNeuronInternal, NMFeagiAdvancedDataNeuronScratch, NMFeagiAdvancedDataNeuronVariance, NMFeagiAdvancedTrait, Period,
};
use feagi_connectome::neuron_models::models::foundation_power::quantizations::NMFoundationPowerQuantFloat32;
use feagi_connectome::neuron_models::models::foundation_power::{
    NMFoundationPowerDataCorticalInternal, NMFoundationPowerDataCorticalProperties, NMFoundationPowerDataCorticalVariance,
    NMFoundationPowerDataNeuronInternal, NMFoundationPowerDataNeuronVariance,
};
use feagi_connectome::neuron_models::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;
use feagi_connectome_neural::neural_basis::neuron_model::model_data::{CorticalDataVarianceSpecification, NeuronModelUniversalCorticalProperties};
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::by_generator::NeurogenesisByGenerator;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage::{NeurogenesisFromStorage, NeurogenesisStorageError};
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::NeurogenesisMethod;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutVoxel;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutData, NeuronLayoutOf};
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, FeagiIndexQuantizationStandard};

type Fiq = FeagiIndexQuantizationStandard;
type Advanced = NMFeagiAdvancedQuantStandard32Bit;
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

//region Variance

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

//endregion

//region Storage

fn advanced_properties() -> NMFeagiAdvancedDataCorticalProperties<Advanced> {
    // refractory_period, firing_threshold, leak_coefficient, leak_variability,
    // postsynaptic_current, postsynaptic_current_max, degeneration
    NMFeagiAdvancedDataCorticalProperties::new(Period::QUANT_ONE, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0)
}

#[test]
fn storage_writes_variance_alongside_neuron_data() {
    let storage = NeurogenesisFromStorage::<Fiq, Advanced>::new(
        voxel_layout([2, 1, 1, 1]),
        universal(),
        advanced_properties(),
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

/// A model without variance fields holds no per neuron variance memory
#[test]
fn model_without_variance_generates_nothing() {
    assert_eq!(size_of::<NMFoundationPowerDataNeuronVariance<Power>>(), 0);
    let storage = power_storage(4).unwrap();
    assert_eq!(storage.iter().len(), 4);
}

//endregion

//region Custom Generator

/// A generator other than the model's own: each neuron starts refractory for as many bursts as its
/// index
#[derive(Clone, Debug)]
pub struct IndexAsRefractoryGenerator<FIQ: FeagiIndexQuantization, NM: NeuronModel> {
    layout: NeuronLayoutOf<NM, FIQ>,
    universal_properties: NeuronModelUniversalCorticalProperties<NM>,
    cortical_properties: NM::CorticalDataProperties,
    cortical_internal: NM::CorticalDataInternal,
    cortical_variance: NM::CorticalDataVariance,
}

/// Generic over every quantization level of the model, converting the index to the level's
/// period type
impl<FIQ: FeagiIndexQuantization, NM: NMFeagiAdvancedTrait> NeurogenesisMethod<FIQ, NM> for IndexAsRefractoryGenerator<FIQ, NM> {
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
        let countdown = Period::new(QuantizedDecimalTrait::from_quantization(index.quant_to_usize() as f32));
        let internal = NMFeagiAdvancedDataNeuronInternal::new(countdown);
        let scratch = NM::generate_neuron_scratch(index, &self.layout, &internal);
        (internal, scratch)
    }
}

impl<FIQ: FeagiIndexQuantization, NM: NMFeagiAdvancedTrait> NeurogenesisByGenerator<FIQ, NM> for IndexAsRefractoryGenerator<FIQ, NM> {
    type OtherDataNeededForFromStorage = ();

    /// Regenerates from the stored cortical data, every neuron starting at its index again
    fn new_generator_from_storage(storage: &NeurogenesisFromStorage<FIQ, NM>, _other_data: ()) -> Self {
        let (cortical_properties, cortical_internal) = storage.clone_model_cortical_data();
        Self {
            layout: *storage.get_layout(),
            universal_properties: storage.get_universal_cortical_properties(),
            cortical_properties,
            cortical_internal,
            cortical_variance: storage.get_model_cortical_variance().clone(),
        }
    }
}

fn refractory_countdowns<G: NeurogenesisMethod<Fiq, Advanced>>(neurogenesis: &G) -> Vec<f32> {
    neurogenesis.iter().map(|(internal, _, _)| internal.refractory_countdown.deref()).collect()
}

#[test]
fn custom_generator_runs_through_the_neurogenesis_method() {
    let generator = IndexAsRefractoryGenerator::<Fiq, Advanced> {
        layout: voxel_layout([3, 1, 1, 1]),
        universal_properties: universal(),
        cortical_properties: advanced_properties(),
        cortical_internal: NMFeagiAdvancedDataCorticalInternal::new(),
        cortical_variance: advanced_variance(0.0, 0.0, 0.0),
    };
    assert_eq!(refractory_countdowns(&generator), [0.0, 1.0, 2.0]);
}

/// Stored neuron data is used as is, while the generator made from that storage generates its own
#[test]
fn custom_generator_from_storage_regenerates_neurons() {
    let storage = NeurogenesisFromStorage::<Fiq, Advanced>::new(
        voxel_layout([3, 1, 1, 1]),
        universal(),
        advanced_properties(),
        NMFeagiAdvancedDataCorticalInternal::new(),
        advanced_variance(0.0, 0.0, 0.0),
        vec![NMFeagiAdvancedDataNeuronInternal::new(Period::new(7.0)); 3],
    )
    .unwrap();
    assert_eq!(refractory_countdowns(&storage), [7.0, 7.0, 7.0]);

    let generator = IndexAsRefractoryGenerator::new_generator_from_storage(&storage, ());
    assert_eq!(refractory_countdowns(&generator), [0.0, 1.0, 2.0]);
}

//endregion

//region Uniform Defaults Generator

fn uniform_generator(dimensions: [usize; 4], countdown: f32) -> NeurogenesisGeneratorUniformDefault<Fiq, Advanced> {
    NeurogenesisGeneratorUniformDefault::new(
        voxel_layout(dimensions),
        universal(),
        advanced_properties(),
        NMFeagiAdvancedDataCorticalInternal::new(),
        advanced_variance(0.0, 0.0, 0.0),
        NMFeagiAdvancedDataNeuronInternal::new(Period::new(countdown)),
    )
}

/// Every neuron starts with the same internal data
#[test]
fn uniform_generator_starts_every_neuron_the_same() {
    let generator = uniform_generator([2, 2, 1, 1], 5.0);
    assert_eq!(refractory_countdowns(&generator), [5.0; 4]);

    let mut internal = vec![NMFeagiAdvancedDataNeuronInternal::new(Period::new(-1.0)); 4];
    let mut scratch = vec![NMFeagiAdvancedDataNeuronScratch::new(); 4];
    let mut variance = vec![NMFeagiAdvancedDataNeuronVariance::new(0.0); 4];
    generator.write_into_slices(&mut internal, &mut scratch, &mut variance);
    assert!(internal.iter().all(|neuron| neuron.refractory_countdown.deref() == 5.0));
}

/// Made from storage, the cortical data is kept while every neuron starts from the given value
#[test]
fn uniform_generator_from_storage_keeps_cortical_data_and_resets_neurons() {
    let storage = NeurogenesisFromStorage::<Fiq, Advanced>::new(
        voxel_layout([3, 1, 1, 1]),
        universal(),
        advanced_properties(),
        NMFeagiAdvancedDataCorticalInternal::new(),
        advanced_variance(0.0, 0.0, 0.0),
        [1.0, 2.0, 3.0].map(|countdown| NMFeagiAdvancedDataNeuronInternal::new(Period::new(countdown))).to_vec(),
    )
    .unwrap();
    let generator = NeurogenesisGeneratorUniformDefault::new_generator_from_storage(&storage, NMFeagiAdvancedDataNeuronInternal::new(Period::new(9.0)));
    assert_eq!(generator, uniform_generator([3, 1, 1, 1], 9.0));
    assert_eq!(refractory_countdowns(&generator), [9.0; 3]);
}

/// The uniform generator is FeagiAdvanced's own generator, so it converts into the model's
/// generator type without wrapping
#[test]
fn uniform_generator_is_the_model_generator() {
    let generator: <Advanced as NeuronModel>::NeurogenesisByGeneratorType<Fiq> = uniform_generator([3, 1, 1, 1], 4.0);
    assert_eq!(refractory_countdowns(&generator), [4.0; 3]);
}

//endregion

//region CPU Dynamics

#[cfg(feature = "burst-engine-cpu")]
mod cpu_dynamics {
    use super::*;
    use feagi_basis::BurstIndex;
    use feagi_connectome::neuron_models::models::feagi_advanced::{BurstEngineDynamics, NMFeagiAdvancedDataCorticalScratch};
    use feagi_connectome_neural::burst_engine_cpu::NeuronCPUParallelDynamics;

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
            self.processing_buffer = CorticalAreaNeuronPotential::new(self.processing_buffer.deref() + incoming);
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
            (is_firing, self.processing_buffer.deref())
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
