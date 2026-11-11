#![cfg(feature = "burst-engine-cpu")]

//! Cortical areas added to the CPU burst engine fire as their neuron model says, and the engine
//! sends which of their neurons fired out through its visualization channel

use feagi_basis::neurons::{CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
use feagi_basis::parallelism::block_on_thread;
use feagi_basis::parallelism::feagi_channel::base_channels::implementations::alloc_channels::FeagiChannelAllocBuilder;
use feagi_basis::prelude::*;
use feagi_connectome::neuron_models::cpu_burst_engine::{CpuBurstEngine, CpuConnectomeDispatch};
use feagi_connectome::neuron_models::models::foundation_death::quantizations::{
    NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted, NMFoundationDeathQuantFloat32,
};
use feagi_connectome::neuron_models::models::foundation_death::{
    NMFoundationDeathDataCorticalInternal, NMFoundationDeathDataCorticalProperties, NMFoundationDeathDataCorticalVariance,
    NMFoundationDeathDataNeuronInternal,
};
use feagi_connectome::neuron_models::models::foundation_power::quantizations::{
    NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted, NMFoundationPowerQuantFloat16, NMFoundationPowerQuantFloat32, NMFoundationPowerQuantFloat64,
};
use feagi_connectome::neuron_models::models::foundation_power::{
    NMFoundationPowerDataCorticalInternal, NMFoundationPowerDataCorticalProperties, NMFoundationPowerDataCorticalVariance,
    NMFoundationPowerDataNeuronInternal, NMFoundationPowerTrait,
};
use feagi_connectome::neuron_models::models::NMNeurogenesisStorageOrGeneratorModeled;
use feagi_connectome::neuron_models::neurogenesis_generator_uniform_defaults::{NeurogenesisGeneratorUniformDefault, NeurogenesisGeneratorUniformDefaultInitParameters};
use feagi_connectome_neural::neural_basis::connectome_editing::{
    ConnectomeEditResultantCorticalAreaAdd, ConnectomeEditResultantEnum, ConnectomeEditStepCorticalAreaAdd,
};
use feagi_connectome_neural::neural_basis::engine::{BurstEngine, BurstEngineIndex};
use feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::by_generator::NeurogenesisGenerator;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage_or_by_generator::NeurogenesisFromStorageOrByGenerator;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutVoxel;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::NeuronLayout;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantizationStandard;
use feagi_connectome_neural::neural_basis::visualization::{BurstVisualizationData, NeuronVisualizationVoxelIndex};

type Fiq = FeagiIndexQuantizationStandard;
type Visualization = BurstVisualizationData<Fiq>;
type Transmitter = thingbuf_sender::Sender;
type Engine = CpuBurstEngine<Fiq, Transmitter>;
type Area = NMNeurogenesisStorageOrGeneratorModeled<Fiq>;

/// Names the halves of the visualization channel without depending on thingbuf directly
mod thingbuf_sender {
    use super::Visualization;
    use feagi_basis::parallelism::feagi_channel::base_channels::base_channels::FeagiChannelBuilder;
    use feagi_basis::parallelism::feagi_channel::base_channels::implementations::alloc_channels::FeagiChannelAllocBuilder;

    pub type Sender = <FeagiChannelAllocBuilder<Visualization> as FeagiChannelBuilder<Visualization>>::Transmitter;
    pub type Receiver = <FeagiChannelAllocBuilder<Visualization> as FeagiChannelBuilder<Visualization>>::Receiver;
}

//region Helpers

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

/// The generator of an area whose neurons always fire
fn power_generator<NM>(dimensions: [usize; 4]) -> NeurogenesisFromStorageOrByGenerator<Fiq, NM, NeurogenesisGeneratorUniformDefault<Fiq, NM>>
where
    NM: NMFoundationPowerTrait + NeuronModel<NeurogenesisGenerator<Fiq> = NeurogenesisGeneratorUniformDefault<Fiq, NM>>,
{
    NeurogenesisFromStorageOrByGenerator::ByGenerator(NeurogenesisGeneratorUniformDefault::new(
        NeurogenesisGeneratorUniformDefaultInitParameters {
            universal_properties: universal(),
            cortical_properties: NMFoundationPowerDataCorticalProperties::new(),
            cortical_internal: NMFoundationPowerDataCorticalInternal::new(),
            cortical_variance: NMFoundationPowerDataCorticalVariance::new(),
            neuron_internal: NMFoundationPowerDataNeuronInternal::new(),
        },
        voxel_layout(dimensions),
    ))
}

fn power_area_float_16(dimensions: [usize; 4]) -> Area {
    Area::FoundationPower(NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted::Float16(power_generator::<NMFoundationPowerQuantFloat16>(dimensions)))
}

fn power_area_float_32(dimensions: [usize; 4]) -> Area {
    Area::FoundationPower(NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted::Float32(power_generator::<NMFoundationPowerQuantFloat32>(dimensions)))
}

fn power_area_float_64(dimensions: [usize; 4]) -> Area {
    Area::FoundationPower(NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted::Float64(power_generator::<NMFoundationPowerQuantFloat64>(dimensions)))
}

/// An area whose neurons never fire
fn death_area_float_32(dimensions: [usize; 4]) -> Area {
    Area::FoundationDeath(NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted::Float32(NeurogenesisFromStorageOrByGenerator::ByGenerator(
        NeurogenesisGeneratorUniformDefault::<Fiq, NMFoundationDeathQuantFloat32>::new(
            NeurogenesisGeneratorUniformDefaultInitParameters {
                universal_properties: universal(),
                cortical_properties: NMFoundationDeathDataCorticalProperties::new(),
                cortical_internal: NMFoundationDeathDataCorticalInternal::new(),
                cortical_variance: NMFoundationDeathDataCorticalVariance::new(),
                neuron_internal: NMFoundationDeathDataNeuronInternal::new(),
            },
            voxel_layout(dimensions),
        ),
    )))
}

/// A CPU burst engine without cortical areas, and the receiver of its visualization
fn new_engine() -> (Engine, thingbuf_sender::Receiver) {
    // One slot for the listener to read while the engine writes the other, as the worker makes
    let (transmitter, receiver) = FeagiChannelAllocBuilder::<Visualization>::create_channels(2);
    let engine = block_on_thread(<Engine as BurstEngine<Fiq, CpuConnectomeDispatch, ()>>::init_burst_engine((), BurstEngineIndex(0), transmitter))
        .expect("the CPU burst engine always initializes");
    (engine, receiver)
}

/// Adds the area to the engine, giving back where it ended up
fn add_area(engine: &mut Engine, area: Area) -> ConnectomeEditResultantCorticalAreaAdd<Fiq> {
    let step = ConnectomeEditStepCorticalAreaAdd::<Fiq, CpuConnectomeDispatch>::new(area).into();
    match block_on_thread(BurstEngine::<Fiq, CpuConnectomeDispatch, ()>::process_connectome_edit(engine, step)) {
        Ok(ConnectomeEditResultantEnum::CorticalAreaAdd(Ok(added))) => added,
        Ok(ConnectomeEditResultantEnum::CorticalAreaAdd(Err(error))) => panic!("the area was refused: {error}"),
        Ok(ConnectomeEditResultantEnum::Nothing) => panic!("adding an area was answered as nothing"),
        Err(error) => panic!("the engine failed to add the area: {error}"),
    }
}

fn run_burst(engine: &mut Engine, send_visualization: bool) {
    block_on_thread(BurstEngine::<Fiq, CpuConnectomeDispatch, ()>::run_burst(engine, send_visualization));
}

/// The visualization words of an area of `neuron_count` neurons
fn words_of(visualization: &Visualization, area: &ConnectomeEditResultantCorticalAreaAdd<Fiq>, neuron_count: usize) -> Vec<u8> {
    let first_word = area.visualization_start_index.quant_to_usize();
    visualization.bit_data.store()[first_word..first_word + neuron_count.div_ceil(8)].to_vec()
}

//endregion

/// The neurons of a power area fire, those of a death area do not, and the visualization sent out
/// has one bit per neuron saying so, each area in its own words
#[test]
fn firing_neurons_are_sent_out_as_visualization() {
    let (mut engine, receiver) = new_engine();
    // 6 neurons, in 1 word with 2 bits of padding
    let power = add_area(&mut engine, power_area_float_32([3, 2, 1, 1]));
    // 10 neurons, in 2 words with 6 bits of padding
    let death = add_area(&mut engine, death_area_float_32([5, 2, 1, 1]));
    // 16 neurons, in exactly 2 words
    let second_power = add_area(&mut engine, power_area_float_32([4, 4, 1, 1]));

    assert_eq!(power.new_cortical_index.quant_to_usize(), 0);
    assert_eq!(death.new_cortical_index.quant_to_usize(), 1);
    assert_eq!(second_power.new_cortical_index.quant_to_usize(), 2);
    assert_eq!(power.visualization_start_index.quant_to_usize(), 0);
    assert_eq!(death.visualization_start_index.quant_to_usize(), 1);
    assert_eq!(second_power.visualization_start_index.quant_to_usize(), 3);

    run_burst(&mut engine, true);

    let visualization = receiver.try_recv_ref().expect("the burst sent its visualization");
    assert_eq!(visualization.bit_data.store().len(), 5);
    assert_eq!(words_of(&visualization, &power, 6), [0b0011_1111], "every neuron of the power area fired, its padding did not");
    assert_eq!(words_of(&visualization, &death, 10), [0, 0], "no neuron of the death area fired");
    assert_eq!(words_of(&visualization, &second_power, 16), [0xFF, 0xFF]);

    // The same through the bit accessor listeners would use
    for neuron in 0..6 {
        assert_eq!(visualization.bit_data.get_bool(NeuronVisualizationVoxelIndex::quant_from_usize_unchecked(neuron)), Some(true));
    }
    for neuron in 8..24 {
        assert_eq!(visualization.bit_data.get_bool(NeuronVisualizationVoxelIndex::quant_from_usize_unchecked(neuron)), Some(false));
    }
}

/// Neuron models of different membrane potential quantizations are held apart, and all burst
#[test]
fn every_membrane_potential_quantization_bursts() {
    let (mut engine, receiver) = new_engine();
    let float_16 = add_area(&mut engine, power_area_float_16([2, 1, 1, 1]));
    let silent = add_area(&mut engine, death_area_float_32([3, 1, 1, 1]));
    let float_64 = add_area(&mut engine, power_area_float_64([4, 1, 1, 1]));
    let float_32 = add_area(&mut engine, power_area_float_32([5, 1, 1, 1]));

    run_burst(&mut engine, true);

    let visualization = receiver.try_recv_ref().expect("the burst sent its visualization");
    assert_eq!(words_of(&visualization, &float_16, 2), [0b0000_0011]);
    assert_eq!(words_of(&visualization, &silent, 3), [0]);
    assert_eq!(words_of(&visualization, &float_64, 4), [0b0000_1111]);
    assert_eq!(words_of(&visualization, &float_32, 5), [0b0001_1111]);
}

/// Visualization is only sent by the bursts asked to, and every one sent is of its own burst
#[test]
fn visualization_is_only_sent_when_asked() {
    let (mut engine, receiver) = new_engine();
    let power = add_area(&mut engine, power_area_float_32([3, 1, 1, 1]));

    run_burst(&mut engine, false);
    assert!(receiver.try_recv_ref().is_err(), "a burst not asked to send visualization sent it");
    assert_eq!(engine.burst_index().quant_to_usize(), 1);

    // Power neurons fire every burst, so each visualization shows them firing
    for _ in 0..3 {
        run_burst(&mut engine, true);
        let visualization = receiver.try_recv_ref().expect("the burst sent its visualization");
        assert_eq!(words_of(&visualization, &power, 3), [0b0000_0111]);
    }
    assert_eq!(engine.burst_index().quant_to_usize(), 4);
}

/// Adding an area changes where areas are in the visualization, which listeners are told through
/// the change counter. A slot written before the area was added is sized again
#[test]
fn adding_an_area_changes_the_visualization_structure() {
    let (mut engine, receiver) = new_engine();
    let first = add_area(&mut engine, power_area_float_32([3, 1, 1, 1]));

    run_burst(&mut engine, true);
    let counter_before = {
        let visualization = receiver.try_recv_ref().expect("the burst sent its visualization");
        assert_eq!(visualization.bit_data.store().len(), 1);
        visualization.change_counter.0
    };

    let second = add_area(&mut engine, power_area_float_32([9, 1, 1, 1]));
    // Both slots of the channel are written, so both are sized for the second area
    for _ in 0..2 {
        run_burst(&mut engine, true);
        let visualization = receiver.try_recv_ref().expect("the burst sent its visualization");
        assert_ne!(visualization.change_counter.0, counter_before);
        assert_eq!(visualization.bit_data.store().len(), 3);
        assert_eq!(words_of(&visualization, &first, 3), [0b0000_0111]);
        assert_eq!(words_of(&visualization, &second, 9), [0xFF, 0b0000_0001]);
    }
}

/// Without a listener taking visualization out of the channel it fills, which skips visualization
/// rather than holding the burst up
#[test]
fn full_visualization_channel_does_not_stop_bursts() {
    let (mut engine, _unread_receiver) = new_engine();
    add_area(&mut engine, power_area_float_32([3, 1, 1, 1]));
    for _ in 0..10 {
        run_burst(&mut engine, true);
    }
    assert_eq!(engine.burst_index().quant_to_usize(), 10);
}
