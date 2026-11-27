//! Muscle activation motor unit: a channel without a spike is zero excitation on every
//! decoded frame, including a trailing frame that carries no data for the area at all.

use feagi_sensorimotor::caching::MotorDeviceCache;
use feagi_sensorimotor::configuration::jsonable::JSONInputOutputDefinition;
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelIndex, CorticalUnitIndex, NeuronDepth,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, PercentageNeuronPositioning,
};
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::genomic::MotorCorticalUnit;
use feagi_structures::neuron_voxels::xyzp::{
    CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZPArrays,
};
use std::time::Instant;

const CHANNELS: u32 = 3;
const DEPTH: u32 = 10; // z = 0 decodes to 100%

fn unit() -> CorticalUnitIndex {
    CorticalUnitIndex::from(0u16)
}

fn channel(i: u32) -> CorticalChannelIndex {
    CorticalChannelIndex::from(i)
}

fn muscle_id() -> CorticalID {
    MotorCorticalUnit::get_cortical_ids_array_for_muscle_with_parameters(
        FrameChangeHandling::Absolute,
        PercentageNeuronPositioning::Linear,
        unit(),
    )[0]
}

fn motors() -> MotorDeviceCache {
    let mut motors = MotorDeviceCache::new();
    motors
        .muscle_register(
            unit(),
            CorticalChannelCount::new(CHANNELS).unwrap(),
            FrameChangeHandling::Absolute,
            NeuronDepth::new(DEPTH).unwrap(),
            PercentageNeuronPositioning::Linear,
        )
        .unwrap();
    motors
}

/// One frame with the given (channel, z) spikes on the Muscle area.
fn frame(spikes: &[(u32, u32)]) -> CorticalMappedXYZPNeuronVoxels {
    let mut arrays = NeuronVoxelXYZPArrays::new();
    for (x, z) in spikes {
        arrays.push_raw(*x, 0, *z, 1.0);
    }
    let mut out = CorticalMappedXYZPNeuronVoxels::new();
    out.insert(muscle_id(), arrays);
    out
}

fn read(motors: &MotorDeviceCache, ch: u32) -> f32 {
    motors
        .muscle_read_postprocessed_cache_value(unit(), channel(ch))
        .unwrap()
        .get_as_0_1()
}

fn changed(motors: &MotorDeviceCache, ch: u32) -> bool {
    motors
        .motor_channel_updated_last_decode(MotorCorticalUnit::Muscle, unit(), ch)
        .unwrap()
}

#[test]
fn muscle_fire_then_silence_reads_zero_with_changed_flag() {
    let mut m = motors();
    m.ingest_neuron_data_and_run_callbacks(frame(&[(0, 0)]), Instant::now())
        .unwrap();
    assert!(
        read(&m, 0) > 0.5,
        "fired channel reads high: {}",
        read(&m, 0)
    );
    assert!(changed(&m, 0));

    // Next frame carries a spike on another channel only: channel 0 is silent.
    m.ingest_neuron_data_and_run_callbacks(frame(&[(1, 0)]), Instant::now())
        .unwrap();
    assert_eq!(read(&m, 0), 0.0);
    assert!(changed(&m, 0), "silent channel is flagged changed");
    assert!(read(&m, 1) > 0.5);
}

#[test]
fn muscle_trailing_empty_frame_zeroes_all_channels() {
    let mut m = motors();
    m.ingest_neuron_data_and_run_callbacks(frame(&[(0, 0), (1, 0), (2, 0)]), Instant::now())
        .unwrap();
    for ch in 0..CHANNELS {
        assert!(read(&m, ch) > 0.5);
    }
    // Trailing empty frame: no data for the area at all.
    m.ingest_neuron_data_and_run_callbacks(CorticalMappedXYZPNeuronVoxels::new(), Instant::now())
        .unwrap();
    for ch in 0..CHANNELS {
        assert_eq!(read(&m, ch), 0.0, "channel {ch}");
        assert!(changed(&m, ch), "channel {ch} flagged changed");
    }
    // An area present but empty behaves the same.
    m.ingest_neuron_data_and_run_callbacks(frame(&[(2, 0)]), Instant::now())
        .unwrap();
    m.ingest_neuron_data_and_run_callbacks(frame(&[]), Instant::now())
        .unwrap();
    assert_eq!(read(&m, 2), 0.0);
}

#[test]
fn muscle_decoder_round_trips_through_json_definition() {
    let m = motors();
    let mut definition = JSONInputOutputDefinition::new();
    m.export_to_output_definition(&mut definition).unwrap();
    let text = serde_json::to_string(&definition).unwrap();
    assert!(
        text.contains("\"Muscle\""),
        "decoder exports as Muscle: {text}"
    );

    // A cache rebuilt from the definition keeps the zero-when-silent behaviour.
    let mut rebuilt = MotorDeviceCache::new();
    rebuilt.import_from_output_definition(&definition).unwrap();
    rebuilt
        .ingest_neuron_data_and_run_callbacks(frame(&[(0, 0)]), Instant::now())
        .unwrap();
    assert!(read(&rebuilt, 0) > 0.5);
    rebuilt
        .ingest_neuron_data_and_run_callbacks(CorticalMappedXYZPNeuronVoxels::new(), Instant::now())
        .unwrap();
    assert_eq!(read(&rebuilt, 0), 0.0);
    assert!(changed(&rebuilt, 0));
}
