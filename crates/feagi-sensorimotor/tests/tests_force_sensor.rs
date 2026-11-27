//! Signed scalar sensor unit (Force) through the sensor cache: sign coding of written values.

use feagi_sensorimotor::caching::SensorDeviceCache;
use feagi_sensorimotor::data_types::SignedPercentage;
use feagi_sensorimotor::wrapped_io_data::WrappedIOData;
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelIndex, CorticalUnitIndex, NeuronDepth,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, PercentageNeuronPositioning,
};
use feagi_structures::genomic::SensoryCorticalUnit;
use std::time::Instant;

const DEPTH: u32 = 10;

fn unit() -> CorticalUnitIndex {
    CorticalUnitIndex::from(0u16)
}

/// Register one Force channel, write `value`, encode, and return the (x, y, z) voxels.
fn encode(positioning: PercentageNeuronPositioning, value: f32) -> Vec<(u32, u32, u32)> {
    let mut cache = SensorDeviceCache::new();
    cache
        .force_register(
            unit(),
            CorticalChannelCount::new(1).unwrap(),
            FrameChangeHandling::Absolute,
            NeuronDepth::new(DEPTH).unwrap(),
            positioning,
        )
        .expect("Force register must succeed");
    cache
        .force_write(
            unit(),
            CorticalChannelIndex::from(0u32),
            WrappedIOData::SignedPercentage(SignedPercentage::new_from_m1_1(value).unwrap()),
        )
        .expect("Force write must accept a signed percentage");
    cache.encode_all_sensors_to_neurons(Instant::now()).unwrap();
    let id = SensoryCorticalUnit::get_cortical_ids_array_for_force_with_parameters(
        FrameChangeHandling::Absolute,
        positioning,
        unit(),
    )[0];
    let mut voxels: Vec<(u32, u32, u32)> = cache
        .get_neurons()
        .get_neurons_of(&id)
        .map(|n| {
            n.iter()
                .map(|v| {
                    let c = v.neuron_voxel_coordinate;
                    (c.x, c.y, c.z)
                })
                .collect()
        })
        .unwrap_or_default();
    voxels.sort();
    voxels
}

#[test]
fn linear_codes_sign_along_z_in_one_column() {
    let linear = PercentageNeuronPositioning::Linear;
    // z = 0 is +1.0, z = depth - 1 is -1.0, zero sits in the middle.
    assert_eq!(encode(linear, 1.0), vec![(0, 0, 0)]);
    assert_eq!(encode(linear, 0.0), vec![(0, 0, 4)]);
    assert_eq!(encode(linear, -1.0), vec![(0, 0, DEPTH - 1)]);
    assert_eq!(encode(linear, 0.5), vec![(0, 0, 2)]);
    assert_eq!(encode(linear, -0.5), vec![(0, 0, 6)]);
}

#[test]
fn fractional_codes_sign_in_two_columns() {
    let fractional = PercentageNeuronPositioning::Fractional;
    // Column 0 carries the positive lobe, column 1 the negative lobe.
    assert_eq!(encode(fractional, 0.5), vec![(0, 0, 0)]);
    assert_eq!(encode(fractional, -0.5), vec![(1, 0, 0)]);
    // Zero writes the smallest bin in both lobes.
    assert_eq!(
        encode(fractional, 0.0),
        vec![(0, 0, DEPTH - 1), (1, 0, DEPTH - 1)]
    );
}

#[test]
fn force_rejects_an_unsigned_wrapper_type() {
    let mut cache = SensorDeviceCache::new();
    cache
        .force_register(
            unit(),
            CorticalChannelCount::new(1).unwrap(),
            FrameChangeHandling::Absolute,
            NeuronDepth::new(DEPTH).unwrap(),
            PercentageNeuronPositioning::Linear,
        )
        .unwrap();
    let unsigned = WrappedIOData::Percentage(
        feagi_sensorimotor::data_types::Percentage::new_from_0_1(0.5).unwrap(),
    );
    assert!(cache
        .force_write(unit(), CorticalChannelIndex::from(0u32), unsigned)
        .is_err());
}
