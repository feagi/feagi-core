//! MIDI frames through the sensor and motor caches.
//!
//! The brain is replaced by a 1:1 copy from the MIDI Input areas to the MIDI
//! Output areas, so these tests cover the neuron coding contract (the encoder is
//! the exact inverse of the decoder), not network dynamics.

use feagi_sensorimotor::caching::{MotorDeviceCache, SensorDeviceCache};
use feagi_sensorimotor::data_types::{MidiControl, MidiFrame, MidiProperties};
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelIndex, CorticalUnitIndex,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, PercentageNeuronPositioning,
};
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::genomic::{MotorCorticalUnit, SensoryCorticalUnit};
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use std::time::Instant;

const HANDLING: FrameChangeHandling = FrameChangeHandling::Absolute;
const DEPTH: u32 = 10;

fn unit(index: u16) -> CorticalUnitIndex {
    CorticalUnitIndex::from(index)
}

fn channel() -> CorticalChannelIndex {
    CorticalChannelIndex::from(0u32)
}

fn channels(n: u32) -> CorticalChannelCount {
    CorticalChannelCount::new(n).unwrap()
}

fn properties() -> MidiProperties {
    MidiProperties::new(128, DEPTH).unwrap()
}

fn input_ids(group: u16, positioning: PercentageNeuronPositioning) -> [CorticalID; 2] {
    SensoryCorticalUnit::get_cortical_ids_array_for_midi_input_with_parameters(
        HANDLING,
        positioning,
        unit(group),
    )
}

fn output_ids(group: u16, positioning: PercentageNeuronPositioning) -> [CorticalID; 2] {
    MotorCorticalUnit::get_cortical_ids_array_for_midi_output_with_parameters(
        HANDLING,
        positioning,
        unit(group),
    )
}

/// Copies one tick of Input-area neurons into the Output areas, as a 1:1 genome mapping would.
fn relay(
    sensors: &SensorDeviceCache,
    positioning: PercentageNeuronPositioning,
) -> CorticalMappedXYZPNeuronVoxels {
    let from = input_ids(0, positioning);
    let to = output_ids(0, positioning);
    let mut motor_neurons = CorticalMappedXYZPNeuronVoxels::new();
    for (f, t) in from.iter().zip(to.iter()) {
        if let Some(neurons) = sensors.get_neurons().get_neurons_of(f) {
            motor_neurons.insert(*t, neurons.clone());
        }
    }
    motor_neurons
}

fn caches(positioning: PercentageNeuronPositioning) -> (SensorDeviceCache, MotorDeviceCache) {
    let mut sensors = SensorDeviceCache::new();
    sensors
        .midi_input_register(unit(0), channels(1), HANDLING, properties(), positioning)
        .unwrap();
    let mut motors = MotorDeviceCache::new();
    motors
        .midi_output_register(unit(0), channels(1), HANDLING, properties(), positioning)
        .unwrap();
    (sensors, motors)
}

/// Encode one frame, relay it, decode it.
fn round_trip(
    sensors: &mut SensorDeviceCache,
    motors: &mut MotorDeviceCache,
    positioning: PercentageNeuronPositioning,
    frame: MidiFrame,
) -> MidiFrame {
    sensors
        .midi_input_write(unit(0), channel(), frame.into())
        .unwrap();
    sensors
        .encode_all_sensors_to_neurons(Instant::now())
        .unwrap();
    let tick = relay(sensors, positioning);
    motors
        .ingest_neuron_data_and_run_callbacks(tick, Instant::now())
        .unwrap();
    motors
        .midi_output_read_postprocessed_cache_value(unit(0), channel())
        .unwrap()
}

/// Strength for a MIDI velocity (1..=127) on the 0..1 frame scale.
fn velocity_strength(velocity: u32) -> f32 {
    velocity as f32 / 127.0
}

#[test]
fn notes_round_trip_for_several_velocities_including_the_extremes() {
    for positioning in [
        PercentageNeuronPositioning::Linear,
        PercentageNeuronPositioning::Fractional,
    ] {
        let (mut sensors, mut motors) = caches(positioning);
        let mut frame = MidiFrame::new(&properties()).unwrap();
        let notes = [(0usize, 1u32), (36, 20), (60, 64), (72, 100), (127, 127)];
        for (column, velocity) in notes {
            frame
                .notes
                .raise_strength(column, velocity_strength(velocity) * 255.0);
        }
        let decoded = round_trip(&mut sensors, &mut motors, positioning, frame.clone());
        assert_eq!(decoded.notes.sounding_count(), notes.len());
        for (column, _) in notes {
            let sent = frame.notes.strength(column);
            let got = decoded.notes.strength(column);
            assert!(
                (sent - got).abs() < 1e-6,
                "column {column}: sent {sent} got {got}"
            );
        }
        // Silent columns stay silent.
        assert_eq!(decoded.notes.strength(1), 0.0);
        // Velocity 1 and 127 keep their order on the potential scale.
        assert!(decoded.notes.strength(0) < decoded.notes.strength(127));
        assert!((decoded.notes.strength(127) - velocity_strength(127)).abs() < 1e-6);
    }
}

#[test]
fn quietest_velocity_writes_a_potential_above_the_unit_firing_threshold() {
    let (mut sensors, _motors) = caches(PercentageNeuronPositioning::Linear);
    let mut frame = MidiFrame::new(&properties()).unwrap();
    frame.notes.raise_strength(9, velocity_strength(1) * 255.0);
    sensors
        .midi_input_write(unit(0), channel(), frame.into())
        .unwrap();
    sensors
        .encode_all_sensors_to_neurons(Instant::now())
        .unwrap();
    let notes_id = input_ids(0, PercentageNeuronPositioning::Linear)[0];
    let voxels: Vec<_> = sensors
        .get_neurons()
        .get_neurons_of(&notes_id)
        .unwrap()
        .iter()
        .map(|n| {
            let c = n.neuron_voxel_coordinate;
            (c.x, c.y, c.z, n.potential)
        })
        .collect();
    assert_eq!(voxels.len(), 1);
    let (x, y, z, p) = voxels[0];
    assert_eq!((x, y, z), (9, 0, 0));
    let threshold = SensoryCorticalUnit::MidiInput
        .get_default_firing_threshold()
        .unwrap() as f32;
    assert!(p > threshold, "potential {p} vs threshold {threshold}");
    assert!((p - 255.0 / 127.0).abs() < 1e-4);
}

#[test]
fn every_control_round_trips_within_one_level_step() {
    for positioning in [PercentageNeuronPositioning::Linear] {
        for control in MidiControl::ALL {
            for z_step in 0..DEPTH {
                // Levels the decoder can return: exact bins of the linear coding.
                let level = 1.0 - z_step as f32 / (DEPTH - 1) as f32;
                let (mut sensors, mut motors) = caches(positioning);
                let mut frame = MidiFrame::new(&properties()).unwrap();
                frame.expression.set_level(control, level);
                let decoded = round_trip(&mut sensors, &mut motors, positioning, frame);
                assert_eq!(decoded.expression.driven_count(), 1, "{control:?}");
                let got = decoded.expression.level(control).expect("driven");
                assert!(
                    (got - level).abs() <= 1.0 / (DEPTH - 1) as f32 + 1e-6,
                    "{control:?} level {level} got {got}"
                );
                for other in MidiControl::ALL {
                    if other != control {
                        assert_eq!(decoded.expression.level(other), None);
                    }
                }
            }
        }
    }
}

#[test]
fn fractional_positioning_round_trips_every_control() {
    let positioning = PercentageNeuronPositioning::Fractional;
    for control in MidiControl::ALL {
        let (mut sensors, mut motors) = caches(positioning);
        let mut frame = MidiFrame::new(&properties()).unwrap();
        frame.expression.set_level(control, 0.5);
        let decoded = round_trip(&mut sensors, &mut motors, positioning, frame);
        let got = decoded.expression.level(control).expect("driven");
        assert!((got - 0.5).abs() < 0.01, "{control:?} got {got}");
        assert_eq!(decoded.expression.driven_count(), 1);
    }
}

#[test]
fn empty_frame_writes_nothing_and_decodes_to_silence() {
    let (mut sensors, mut motors) = caches(PercentageNeuronPositioning::Linear);
    // A loud frame first, so a stale value would show.
    let mut loud = MidiFrame::new(&properties()).unwrap();
    loud.notes.raise_strength(60, 200.0);
    loud.expression.set_level(MidiControl::Volume, 1.0);
    let decoded = round_trip(
        &mut sensors,
        &mut motors,
        PercentageNeuronPositioning::Linear,
        loud,
    );
    assert_eq!(decoded.notes.sounding_count(), 1);

    let empty = MidiFrame::new(&properties()).unwrap();
    let decoded = round_trip(
        &mut sensors,
        &mut motors,
        PercentageNeuronPositioning::Linear,
        empty,
    );
    assert_eq!(decoded.notes.sounding_count(), 0);
    assert_eq!(decoded.expression.driven_count(), 0);
    let ids = input_ids(0, PercentageNeuronPositioning::Linear);
    for id in ids {
        let written = sensors
            .get_neurons()
            .get_neurons_of(&id)
            .map(|n| n.len())
            .unwrap_or(0);
        assert_eq!(written, 0, "an empty frame writes no voxels");
    }
}

#[test]
fn midi_input_registers_and_writes_a_frame_through_the_sensor_cache() {
    let (mut sensors, _motors) = caches(PercentageNeuronPositioning::Linear);
    let mut frame = MidiFrame::new(&properties()).unwrap();
    frame.notes.raise_strength(48, 128.0);
    frame.expression.set_level(MidiControl::Sustain, 1.0);
    sensors
        .midi_input_write(unit(0), channel(), frame.clone().into())
        .unwrap();
    let cached = sensors
        .midi_input_read_postprocessed_cache_value(unit(0), channel())
        .unwrap();
    assert_eq!(cached, frame);
    sensors
        .encode_all_sensors_to_neurons(Instant::now())
        .unwrap();
    let ids = input_ids(0, PercentageNeuronPositioning::Linear);
    let notes: Vec<_> = sensors
        .get_neurons()
        .get_neurons_of(&ids[0])
        .unwrap()
        .iter()
        .map(|n| (n.neuron_voxel_coordinate.x, n.potential))
        .collect();
    assert_eq!(notes, vec![(48, 128.0)]);
    // Sustain is control column 3; level 1.0 is z = 0.
    let expression: Vec<_> = sensors
        .get_neurons()
        .get_neurons_of(&ids[1])
        .unwrap()
        .iter()
        .map(|n| {
            let c = n.neuron_voxel_coordinate;
            (c.x, c.y, c.z)
        })
        .collect();
    assert_eq!(
        expression,
        vec![(MidiControl::Sustain.column() as u32, 0, 0)]
    );
}

#[test]
fn two_channels_are_laid_side_by_side() {
    let mut sensors = SensorDeviceCache::new();
    sensors
        .midi_input_register(
            unit(0),
            channels(2),
            HANDLING,
            properties(),
            PercentageNeuronPositioning::Linear,
        )
        .unwrap();
    let mut frame = MidiFrame::new(&properties()).unwrap();
    frame.notes.raise_strength(3, 100.0);
    sensors
        .midi_input_write(unit(0), CorticalChannelIndex::from(1u32), frame.into())
        .unwrap();
    sensors
        .encode_all_sensors_to_neurons(Instant::now())
        .unwrap();
    let ids = input_ids(0, PercentageNeuronPositioning::Linear);
    let xs: Vec<u32> = sensors
        .get_neurons()
        .get_neurons_of(&ids[0])
        .unwrap()
        .iter()
        .map(|n| n.neuron_voxel_coordinate.x)
        .collect();
    assert_eq!(xs, vec![128 + 3]);
}
