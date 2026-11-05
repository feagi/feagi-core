use feagi_basis::neurons::{CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
use feagi_connectome::cortical_area::neuron_layout::voxel::NeuronLayoutVoxel;
use feagi_connectome::cortical_area::neuron_layout::NeuronLayout;
use feagi_connectome::cortical_area::{CorticalAreaSpec, CorticalAreaSpecError, NeuronInit, NeuronInitializer, NeuronModelUniversalCorticalProperties};
use feagi_connectome::neuron_models::neuron_model::feagi_advanced::{
    FeagiAdvancedCorticalDataInternal, FeagiAdvancedCorticalDataProperties, FeagiAdvancedCorticalDataScratch, FeagiAdvancedCorticalDataVariance,
    FeagiAdvancedNeuronDataInternal, FeagiAdvancedNeuronDataScratch, FeagiAdvancedNeuronDataVariance, NeuronModelFeagiAdvancedDefaultQuant, Period,
};
use feagi_connectome::neuron_models::neuron_model::foundation_power::{
    FoundationPowerCorticalDataInternal, FoundationPowerCorticalDataProperties, FoundationPowerCorticalDataScratch, FoundationPowerCorticalDataVariance,
    FoundationPowerNeuronDataInternal, FoundationPowerNeuronDataScratch, FoundationPowerNeuronDataVariance,
    NeuronModelFoundationPowerFloat32,
};
use feagi_connectome::quantization_levels::feagi_index_quantization::{FeagiIndexQuantization, FeagiIndexQuantizationStandard};
use feagi_connectome_neural::neural_basis::neuron_model::specifications::{CorticalDataInternalSpecification, CorticalDataPropertiesSpecification, CorticalDataScratchSpecification, CorticalDataVarianceSpecification, NeuronDataInternalSpecification, NeuronDataScratchSpecification, NeuronDataVarianceSpecification};
use feagi_connectome_neural::neural_basis::neuron_model::{NeuronLayoutOf, NeuronModel, NeuronModelTag};
use feagi_connectome_neural::neural_basis::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
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
fn spec_writes_variance_alongside_neuron_data() {
    let spec = CorticalAreaSpec::<Fiq, Advanced>::new_uniform(
        voxel_layout([2, 1, 1, 1]),
        universal(),
        FeagiAdvancedCorticalDataProperties::new(Period::QUANT_ONE, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0),
        FeagiAdvancedCorticalDataInternal::new(),
        FeagiAdvancedCorticalDataScratch::new(),
        advanced_variance(3.0, 0.0, 0.0),
        FeagiAdvancedNeuronDataInternal::new(Period::QUANT_ZERO),
        FeagiAdvancedNeuronDataScratch::new(),
    );
    let mut internal = vec![FeagiAdvancedNeuronDataInternal::new(Period::QUANT_ZERO); 2];
    let mut scratch = vec![FeagiAdvancedNeuronDataScratch::new(); 2];
    let mut variance = vec![FeagiAdvancedNeuronDataVariance::new(-1.0); 2];
    spec.write_into_slices(&mut internal, &mut scratch, &mut variance);

    let mut offsets: Vec<f32> = variance.iter().map(|neuron| neuron.firing_threshold_offset).collect();
    offsets.sort_by(f32::total_cmp);
    assert_eq!(offsets, [0.0, 3.0]);
    assert_eq!(spec.iter().len(), 2);
    assert_eq!(spec.iter().map(|(_, _, neuron)| neuron).collect::<Vec<_>>(), variance);
}

fn power_spec(neurons: NeuronInit<Power>) -> Result<CorticalAreaSpec<Fiq, Power>, CorticalAreaSpecError> {
    CorticalAreaSpec::new(
        voxel_layout([2, 2, 1, 1]),
        universal(),
        FoundationPowerCorticalDataProperties::new(),
        FoundationPowerCorticalDataInternal::new(),
        FoundationPowerCorticalDataScratch::new(),
        FoundationPowerCorticalDataVariance::new(),
        neurons,
    )
}

#[test]
fn stored_neuron_data_must_match_the_layout() {
    let stored = |count| NeuronInit::Stored {
        internal: vec![FoundationPowerNeuronDataInternal::new(); count],
        scratch: vec![FoundationPowerNeuronDataScratch::new(); 4],
    };
    assert!(power_spec(stored(4)).is_ok());
    assert!(matches!(power_spec(stored(3)), Err(CorticalAreaSpecError::StoredNeuronCountMismatch)));
}

/// A model without variance fields gets its variance generated, and holds no per neuron memory
#[test]
fn model_without_variance_generates_nothing() {
    assert_eq!(size_of::<FoundationPowerNeuronDataVariance<Power>>(), 0);
    let spec = power_spec(NeuronInit::Uniform {
        internal: FoundationPowerNeuronDataInternal::new(),
        scratch: FoundationPowerNeuronDataScratch::new(),
    }).unwrap();
    assert_eq!(spec.iter().len(), 4);
}

//region Custom Init

/// Each neuron starts with its own index
#[derive(Debug, Clone, PartialEq)]
pub struct IndexAsValueInit;

define_cortical_dynamics_quantization_template! {
    /// Only exists to test custom neuron initialization
    TestCustom

    NeuronLayout: NeuronLayoutTypeVoxel

    CustomNeuronInit: IndexAsValueInit

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

impl NeuronInitializer<NeuronModelTestCustomFloat32> for IndexAsValueInit {
    fn neuron_data_at<FIQ: FeagiIndexQuantization>(
        &self,
        _layout: &NeuronLayoutOf<NeuronModelTestCustomFloat32, FIQ>,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> (TestCustomNeuronDataInternal<NeuronModelTestCustomFloat32>, TestCustomNeuronDataScratch<NeuronModelTestCustomFloat32>) {
        use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;
        (TestCustomNeuronDataInternal::new(index.quant_to_usize() as f32), TestCustomNeuronDataScratch::new())
    }
}

#[test]
fn custom_neuron_init_runs_through_the_spec() {
    let spec = CorticalAreaSpec::<Fiq, NeuronModelTestCustomFloat32>::new(
        voxel_layout([3, 1, 1, 1]),
        universal(),
        TestCustomCorticalDataProperties::new(),
        TestCustomCorticalDataInternal::new(),
        TestCustomCorticalDataScratch::new(),
        TestCustomCorticalDataVariance::new(),
        NeuronInit::Custom(IndexAsValueInit),
    ).unwrap();
    let values: Vec<f32> = spec.iter().map(|(internal, _, _)| internal.value).collect();
    assert_eq!(values, [0.0, 1.0, 2.0]);
}

//endregion

//region Dispatch

#[cfg(feature = "burst-engine-cpu")]
mod dispatch {
    use super::*;
    use feagi_basis::BurstIndex;
    use feagi_connectome::burst_engine_cpu::{CorticalCPUParallelDynamics, CpuCorticalAreaSpecDispatch, CpuCorticalAreaSpecVisitor};
    use feagi_connectome::neuron_models::neuron_model::feagi_advanced::BurstEngineDynamics;
    use feagi_connectome::neuron_models::neuron_model::NeuronModelEnum;
    use feagi_connectome::neuron_models::spec_dispatch::{CorticalAreaSpecContainer, CorticalAreaSpecEnum};

    /// Builds the neurons of the spec, then runs the first through the dynamics the dispatch chose
    struct FireFirstNeuron;

    impl<FIQ: FeagiIndexQuantization> CpuCorticalAreaSpecVisitor<FIQ> for FireFirstNeuron {
        type Output = (usize, bool);

        fn visit<NM, D>(self, spec: CorticalAreaSpec<FIQ, NM>) -> Self::Output
        where
            NM: NeuronModel,
            D: CorticalCPUParallelDynamics<FIQ, NM>,
        {
            let mut neurons: Vec<_> = spec.iter().collect();
            let (internal, scratch, variance) = &mut neurons[0];
            let (properties, cortical_internal, cortical_scratch) = spec.get_cortical_level_data();
            let mut processing_buffer = CorticalAreaNeuronPotential::QUANT_ZERO;
            let is_firing = D::process_model_neuron_dynamics(
                &mut processing_buffer,
                &BurstIndex::QUANT_ZERO,
                spec.get_neuron_layout(),
                &CorticalAreaNeuronLocalIndex::QUANT_ZERO,
                (properties, cortical_internal, cortical_scratch),
                (internal, scratch, variance),
            );
            (neurons.len(), is_firing)
        }
    }

    #[test]
    fn spec_enum_dispatches_to_the_dynamics_of_its_model() {
        let spec: CorticalAreaSpecEnum<Fiq> = NeuronModelEnum::<CorticalAreaSpecContainer<Fiq>>::from_data::<Power>(
            power_spec(NeuronInit::Uniform {
                internal: FoundationPowerNeuronDataInternal::new(),
                scratch: FoundationPowerNeuronDataScratch::new(),
            }).unwrap(),
        );
        // The power model always fires
        assert_eq!(spec.dispatch_cpu(FireFirstNeuron), (4, true));
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
            let is_firing = <BurstEngineDynamics<Fiq, Advanced> as CorticalCPUParallelDynamics<Fiq, Advanced>>::process_model_neuron_dynamics(
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
