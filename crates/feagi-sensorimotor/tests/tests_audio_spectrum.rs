//! Audio spectrum encoding through the sensor and motor caches.
//!
//! The brain is replaced by a 1:1 copy from the input area to the output area,
//! so these tests cover the neuron coding contract, not network dynamics.

use feagi_sensorimotor::caching::{MotorDeviceCache, SensorDeviceCache};
use feagi_sensorimotor::data_types::processing::{AudioSpectrumAnalyzer, AudioSpectrumSynthesizer};
use feagi_sensorimotor::data_types::{AudioSpectrumFrame, AudioSpectrumProperties};
use feagi_sensorimotor::ConnectorCache;
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelIndex, CorticalUnitIndex,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::FrameChangeHandling;
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::genomic::{MotorCorticalUnit, SensoryCorticalUnit};
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use std::time::Instant;

const SAMPLE_RATE_HZ: u32 = 16000;

fn properties(phase_steps: u32) -> AudioSpectrumProperties {
    AudioSpectrumProperties::new_linear(SAMPLE_RATE_HZ, 1024, 256, phase_steps, -100, 0).unwrap()
}

fn unit(index: u16) -> CorticalUnitIndex {
    CorticalUnitIndex::from(index)
}

fn channel() -> CorticalChannelIndex {
    CorticalChannelIndex::from(0u32)
}

fn one_channel() -> CorticalChannelCount {
    CorticalChannelCount::new(1).unwrap()
}

fn input_id(group: u16) -> CorticalID {
    SensoryCorticalUnit::get_cortical_ids_array_for_audio_input_with_parameters(
        FrameChangeHandling::Absolute,
        unit(group),
    )[0]
}

fn output_id(group: u16) -> CorticalID {
    MotorCorticalUnit::get_cortical_ids_array_for_audio_output_with_parameters(
        FrameChangeHandling::Absolute,
        unit(group),
    )[0]
}

fn tone(frequency: f32, amplitude: f32, count: usize) -> Vec<f32> {
    (0..count)
        .map(|n| {
            amplitude
                * (2.0 * std::f32::consts::PI * frequency * n as f32 / SAMPLE_RATE_HZ as f32).sin()
        })
        .collect()
}

/// Copies one tick of input-area neurons into the output area, as a 1:1 genome mapping would.
fn relay(
    sensors: &SensorDeviceCache,
    from: CorticalID,
    to: CorticalID,
) -> CorticalMappedXYZPNeuronVoxels {
    let mut motor_neurons = CorticalMappedXYZPNeuronVoxels::new();
    if let Some(neurons) = sensors.get_neurons().get_neurons_of(&from) {
        motor_neurons.insert(to, neurons.clone());
    }
    motor_neurons
}

fn snr_db(input: &[f32], output: &[f32], latency: usize) -> f32 {
    let (mut signal, mut error) = (0.0f64, 0.0f64);
    for (index, out) in output.iter().enumerate().skip(latency) {
        let reference = input[index - latency] as f64;
        signal += reference * reference;
        error += (reference - *out as f64).powi(2);
    }
    (10.0 * (signal / error).log10()) as f32
}

#[test]
fn waveform_survives_encode_relay_decode() {
    let p = properties(256);
    let mut sensors = SensorDeviceCache::new();
    sensors
        .audio_input_register(unit(0), one_channel(), FrameChangeHandling::Absolute, p)
        .unwrap();
    let mut motors = MotorDeviceCache::new();
    motors
        .audio_output_register(unit(0), one_channel(), FrameChangeHandling::Absolute, p)
        .unwrap();

    let input: Vec<f32> = tone(440.0, 0.4, 8000)
        .iter()
        .zip(tone(1500.0, 0.2, 8000))
        .map(|(a, b)| a + b)
        .collect();
    let mut analyzer = AudioSpectrumAnalyzer::new(p).unwrap();
    let mut synthesizer = AudioSpectrumSynthesizer::new(p).unwrap();
    let mut output = Vec::new();

    for frame in analyzer.push_samples(&input).unwrap() {
        sensors
            .audio_input_write(unit(0), channel(), frame.into())
            .unwrap();
        sensors
            .encode_all_sensors_to_neurons(Instant::now())
            .unwrap();
        let tick = relay(&sensors, input_id(0), output_id(0));
        motors
            .ingest_neuron_data_and_run_callbacks(tick, Instant::now())
            .unwrap();
        let decoded: AudioSpectrumFrame = motors
            .audio_output_read_postprocessed_cache_value(unit(0), channel())
            .unwrap();
        output.extend(synthesizer.push_frame(&decoded).unwrap());
    }

    let snr = snr_db(&input, &output, synthesizer.latency_samples());
    assert!(snr > 25.0, "round trip through neurons snr {snr} dB");
}

#[test]
fn encoder_writes_one_neuron_per_active_column_on_its_phase_row() {
    let p = properties(16);
    let mut sensors = SensorDeviceCache::new();
    sensors
        .audio_input_register(unit(0), one_channel(), FrameChangeHandling::Absolute, p)
        .unwrap();
    let mut frame = AudioSpectrumFrame::new(&p).unwrap();
    frame.set_column(10, 0.75, 3).unwrap();
    frame.set_column(400, 0.25, 15).unwrap();
    sensors
        .audio_input_write(unit(0), channel(), frame.into())
        .unwrap();
    sensors
        .encode_all_sensors_to_neurons(Instant::now())
        .unwrap();

    let neurons: Vec<_> = sensors
        .get_neurons()
        .get_neurons_of(&input_id(0))
        .unwrap()
        .iter()
        .map(|n| {
            let c = n.neuron_voxel_coordinate;
            (c.x, c.y, c.z, n.potential)
        })
        .collect();
    assert_eq!(neurons, vec![(10, 3, 0, 0.75), (400, 15, 0, 0.25)]);
}

#[test]
fn decoder_keeps_the_strongest_phase_row_per_column() {
    let p = properties(16);
    let mut motors = MotorDeviceCache::new();
    motors
        .audio_output_register(unit(0), one_channel(), FrameChangeHandling::Absolute, p)
        .unwrap();
    let mut arrays = feagi_structures::neuron_voxels::xyzp::NeuronVoxelXYZPArrays::new();
    arrays.push_raw(7, 2, 0, 0.3);
    arrays.push_raw(7, 9, 0, 0.8);
    arrays.push_raw(7, 20, 0, 0.9); // outside 16 phase rows
    arrays.push_raw(8, 1, 1, 0.9); // z must be 0
    let mut tick = CorticalMappedXYZPNeuronVoxels::new();
    tick.insert(output_id(0), arrays);
    motors
        .ingest_neuron_data_and_run_callbacks(tick, Instant::now())
        .unwrap();

    let frame = motors
        .audio_output_read_postprocessed_cache_value(unit(0), channel())
        .unwrap();
    assert_eq!(frame.get_magnitudes()[7], 0.8);
    assert_eq!(frame.get_phase_steps()[7], 9);
    assert_eq!(frame.get_magnitudes()[8], 0.0);
    assert_eq!(frame.active_column_count(), 1);
}

#[test]
fn a_tick_without_audio_neurons_decodes_to_silence() {
    let p = properties(16);
    let mut motors = MotorDeviceCache::new();
    motors
        .audio_output_register(unit(0), one_channel(), FrameChangeHandling::Absolute, p)
        .unwrap();
    let mut arrays = feagi_structures::neuron_voxels::xyzp::NeuronVoxelXYZPArrays::new();
    arrays.push_raw(3, 1, 0, 0.5);
    let mut loud = CorticalMappedXYZPNeuronVoxels::new();
    loud.insert(output_id(0), arrays);
    motors
        .ingest_neuron_data_and_run_callbacks(loud, Instant::now())
        .unwrap();
    assert_eq!(
        motors
            .audio_output_read_postprocessed_cache_value(unit(0), channel())
            .unwrap()
            .active_column_count(),
        1
    );

    motors
        .ingest_neuron_data_and_run_callbacks(CorticalMappedXYZPNeuronVoxels::new(), Instant::now())
        .unwrap();
    let frame = motors
        .audio_output_read_postprocessed_cache_value(unit(0), channel())
        .unwrap();
    assert_eq!(
        frame.active_column_count(),
        0,
        "stale spectrum would replay as a stuck tone"
    );
}

#[test]
fn stereo_groups_encode_into_separate_areas() {
    let p = properties(16);
    let mut sensors = SensorDeviceCache::new();
    for group in [0u16, 1] {
        sensors
            .audio_input_register(unit(group), one_channel(), FrameChangeHandling::Absolute, p)
            .unwrap();
    }
    let mut left = AudioSpectrumFrame::new(&p).unwrap();
    left.set_column(5, 0.5, 0).unwrap();
    let mut right = AudioSpectrumFrame::new(&p).unwrap();
    right.set_column(50, 0.5, 0).unwrap();
    sensors
        .audio_input_write(unit(0), channel(), left.into())
        .unwrap();
    sensors
        .audio_input_write(unit(1), channel(), right.into())
        .unwrap();
    sensors
        .encode_all_sensors_to_neurons(Instant::now())
        .unwrap();

    let first_x = |group: u16| {
        sensors
            .get_neurons()
            .get_neurons_of(&input_id(group))
            .unwrap()
            .iter()
            .next()
            .unwrap()
            .neuron_voxel_coordinate
            .x
    };
    assert_ne!(input_id(0), input_id(1));
    assert_eq!(first_x(0), 5);
    assert_eq!(first_x(1), 50);
}

#[test]
fn registration_carries_the_audio_tunables_and_round_trips() {
    let p =
        AudioSpectrumProperties::new_logarithmic(48000, 4096, 1600, 128, 8, 20, 20000, -90, -10)
            .unwrap();
    let cache = ConnectorCache::new();
    cache
        .get_sensor_cache()
        .audio_input_register(unit(2), one_channel(), FrameChangeHandling::Absolute, p)
        .unwrap();
    cache
        .get_motor_cache()
        .audio_output_register(unit(2), one_channel(), FrameChangeHandling::Absolute, p)
        .unwrap();

    let json = cache.export_device_registrations_as_config_json().unwrap();
    let encoder = &json["input_units_and_encoder_properties"]["AudioInput"][0][1]["AudioSpectrum"];
    assert_eq!(encoder["bin_count"], 128);
    assert_eq!(encoder["phase_steps"], 8);
    assert_eq!(encoder["spacing"], "Logarithmic");
    assert_eq!(encoder["min_frequency_hz"], 20);
    assert_eq!(encoder["max_frequency_hz"], 20000);
    let decoder =
        &json["output_units_and_decoder_properties"]["AudioOutput"][0][1]["AudioSpectrum"];
    assert_eq!(decoder["magnitude_floor_db"], -90);

    let mut restored = ConnectorCache::new();
    restored
        .import_device_registrations_as_config_json(json.clone())
        .unwrap();
    assert_eq!(
        restored
            .export_device_registrations_as_config_json()
            .unwrap(),
        json
    );
}

#[test]
fn registration_rejects_properties_that_cannot_round_trip() {
    let mut p = properties(16);
    p.hop_size = p.window_size;
    let mut sensors = SensorDeviceCache::new();
    assert!(sensors
        .audio_input_register(unit(0), one_channel(), FrameChangeHandling::Absolute, p)
        .is_err());
}
