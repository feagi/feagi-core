//! Integration tests for the population encoder + class decoder selectors.
//!
//! These exercise the real `feagi-sensorimotor` coders (no mocking of the subject under
//! test). The decoder test transplants encoder-produced neurons into the OPU area so the
//! decode runs against genuine coder output rather than hand-laid voxels.

use std::collections::BTreeMap;
use std::time::Instant;

use feagi_trainer::binding::{
    BinSpacing, ClassDecoder, DecoderBindingProfile, DecoderPlugin, EncoderBindingProfile,
    EncoderPlugin, EncodingScheme, PopulationEncoder,
};
use feagi_trainer::contracts::common::{DatasetVersionId, Modality, OutputType, SampleId, Split};
use feagi_trainer::contracts::ir_sample::{IRSample, Payload};
use feagi_trainer::contracts::prediction_record::TypedPrediction;

use feagi_sensorimotor::data_types::Percentage;
use feagi_sensorimotor::wrapped_io_data::WrappedIOData;
use feagi_sensorimotor::ConnectorCache;
use feagi_genomic_context::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelIndex, CorticalUnitIndex, NeuronDepth,
};
use feagi_genomic_context::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, PercentageNeuronPositioning,
};
use feagi_genomic_context::cortical_unit::{MotorCorticalUnit, SensoryCorticalUnit};
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;

const BINS: u32 = 10;

fn tabular_sample(features: Vec<f64>) -> IRSample {
    IRSample {
        schema_version: 1,
        sample_id: SampleId("s".to_string()),
        dataset_version_id: DatasetVersionId("d".to_string()),
        split: Split::Test,
        modality: Modality::Tabular,
        payload: Payload::Tabular(features),
        target: None,
        output_type: OutputType::Class,
        coordinate_frame: None,
        timestamp: None,
        metadata: BTreeMap::new(),
    }
}

fn population_profile(channels: u32) -> EncoderBindingProfile {
    EncoderBindingProfile {
        cortical_area_id: "iris_ipu".to_string(),
        channels,
        scheme: EncodingScheme::PopulationSingleSpike {
            bins: BINS,
            spacing: BinSpacing::Linear,
        },
    }
}

fn count_input_id() -> feagi_genomic_context::cortical_area::CorticalID {
    SensoryCorticalUnit::get_cortical_ids_array_for_count_input_with_parameters(
        FrameChangeHandling::Absolute,
        PercentageNeuronPositioning::Linear,
        CorticalUnitIndex::from(0u8),
    )[0]
}

fn count_output_id() -> feagi_genomic_context::cortical_area::CorticalID {
    MotorCorticalUnit::get_cortical_ids_array_for_count_output_with_parameters(
        FrameChangeHandling::Absolute,
        PercentageNeuronPositioning::Linear,
        CorticalUnitIndex::from(0u8),
    )[0]
}

#[test]
fn encoder_produces_neurons_for_all_features() {
    let mut encoder = PopulationEncoder::new();
    let frame = encoder
        .encode(
            &tabular_sample(vec![0.1, 0.2, 0.3, 0.4]),
            &population_profile(4),
        )
        .expect("encode");
    let neurons = frame
        .get_neurons_of(&count_input_id())
        .expect("count_input area present");
    let (xs, ..) = neurons.borrow_xyzp_vectors();
    assert!(!xs.is_empty(), "expected spikes for the encoded features");
}

/// Encodes a single normalized scalar and decodes it back through the OPU path, returning
/// the recovered activation. Exercises both selectors end-to-end, independent of the
/// internal voxel layout.
fn roundtrip_decode_single(value: f64) -> f64 {
    let mut encoder = PopulationEncoder::new();
    let encoded = encoder
        .encode(&tabular_sample(vec![value]), &population_profile(1))
        .expect("encode");

    let mut motor = CorticalMappedXYZPNeuronVoxels::new();
    {
        let source = encoded
            .get_neurons_of(&count_input_id())
            .expect("encoded neurons present");
        let (xs, ys, zs, ps) = source.borrow_xyzp_vectors();
        let destination = motor.ensure_clear_and_borrow_mut(&count_output_id());
        for index in 0..xs.len() {
            destination.push_raw(xs[index], ys[index], zs[index], ps[index]);
        }
    }

    let mut decoder = ClassDecoder::new();
    let profile = DecoderBindingProfile {
        cortical_area_id: "iris_opu".to_string(),
        class_count: 1,
        bins: BINS,
    };
    match decoder.decode(motor, &profile).expect("decode") {
        TypedPrediction::Class { scores, .. } => scores[0],
        other => panic!("expected a class prediction, got {other:?}"),
    }
}

#[test]
fn encoder_is_monotonic_in_value() {
    let low = roundtrip_decode_single(0.05);
    let high = roundtrip_decode_single(0.95);
    assert!(
        high > low,
        "higher value should decode to a higher activation (low={low}, high={high})"
    );
}

#[test]
fn encoder_rejects_feature_count_mismatch() {
    let mut encoder = PopulationEncoder::new();
    let result = encoder.encode(&tabular_sample(vec![0.1, 0.2]), &population_profile(4));
    assert!(result.is_err());
}

#[test]
fn encoder_rejects_out_of_range_feature() {
    let mut encoder = PopulationEncoder::new();
    let result = encoder.encode(&tabular_sample(vec![1.5]), &population_profile(1));
    assert!(result.is_err());
}

#[test]
fn decoder_argmaxes_strongest_class_channel() {
    // Produce genuine coder neurons for three per-class activations, highest on class 1.
    let cache = ConnectorCache::new();
    {
        let mut sensor_cache = cache.get_sensor_cache();
        sensor_cache
            .count_input_register(
                CorticalUnitIndex::from(0u8),
                CorticalChannelCount::new(3).unwrap(),
                FrameChangeHandling::Absolute,
                NeuronDepth::new(BINS).unwrap(),
                PercentageNeuronPositioning::Linear,
            )
            .unwrap();
        for (channel, value) in [0.1_f32, 0.9, 0.2].into_iter().enumerate() {
            sensor_cache
                .count_input_write(
                    CorticalUnitIndex::from(0u8),
                    CorticalChannelIndex::from(channel as u32),
                    WrappedIOData::Percentage(Percentage::new_from_0_1(value).unwrap()),
                )
                .unwrap();
        }
        sensor_cache
            .encode_all_sensors_to_neurons(Instant::now())
            .unwrap();
    }
    let encoded = cache.get_sensor_cache().get_neurons().clone();

    // Transplant the encoder's neurons into the OPU cortical_area area the decoder reads.
    let mut motor = CorticalMappedXYZPNeuronVoxels::new();
    {
        let source = encoded
            .get_neurons_of(&count_input_id())
            .expect("encoded neurons present");
        let (xs, ys, zs, ps) = source.borrow_xyzp_vectors();
        let destination = motor.ensure_clear_and_borrow_mut(&count_output_id());
        for index in 0..xs.len() {
            destination.push_raw(xs[index], ys[index], zs[index], ps[index]);
        }
    }

    let mut decoder = ClassDecoder::new();
    let profile = DecoderBindingProfile {
        cortical_area_id: "iris_opu".to_string(),
        class_count: 3,
        bins: BINS,
    };
    let prediction = decoder.decode(motor, &profile).expect("decode");
    match prediction {
        TypedPrediction::Class { class_id, scores } => {
            assert_eq!(class_id, 1, "class 1 had the strongest activation");
            assert_eq!(scores.len(), 3);
            assert!(scores[1] > scores[0] && scores[1] > scores[2]);
        }
        other => panic!("expected a class prediction, got {other:?}"),
    }
}
