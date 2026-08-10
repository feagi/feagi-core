// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Cortical area allocation and the visualization readout built on top of it.

Allocation and bursting are tested together because they are only meaningful together: the burst
kernels iterate the per-neuron vectors that allocation fills, so an area that allocates
incompletely shows up as a burst that panics or silently processes nothing.
*/

use core::marker::PhantomData;

use feagi_data::neurons::DimensionalCorticalArea4DDimensions;
use feagi_data::quantization_levels::feagi_index_quantization::{FeagiIndexQuantization, FeagiIndexQuantizationGenomic};
use feagi_genomic_context::cortical_area::CorticalID;
use feagi_models::connectome_requests::connectome_request::ConnectomeRequest;
use feagi_models::cortical_area::neuron_model_implementations::feagi_advanced::composers::FeagiAdvancedModelCorticalWriter;
use feagi_models::cortical_area::neuron_model_implementations::feagi_advanced::quantization::FeagiAdvancedModelStandardQuant;
use feagi_npu::dynamic_npu::DynamicNPU;

type NeuronQuant = <FeagiIndexQuantizationGenomic as FeagiIndexQuantization>::NeuronIndexQuant;

/// Builds an area-add request for a `x * y * z * d` dimensional area.
fn add_area_request(cortical_id: CorticalID, x: usize, y: usize, z: usize, d: usize) -> ConnectomeRequest {
    let dimensions = DimensionalCorticalArea4DDimensions::<NeuronQuant>::try_new_from_usizes(x, y, z, d).expect("dimensions should be representable");

    let writer = FeagiAdvancedModelCorticalWriter::DefaultNewDimensional {
        dimensions,
        _p: PhantomData::<FeagiAdvancedModelStandardQuant>,
    };

    ConnectomeRequest::CorticalAreaAdd {
        TEMP_adding_id: cortical_id,
        writer: writer.into(),
    }
}

/// Two distinct cortical IDs. Content does not matter, only that they differ.
fn two_cortical_ids() -> (CorticalID, CorticalID) {
    let first = CorticalID::try_from_bytes(b"_power__").expect("valid cortical id");
    let second = CorticalID::try_from_bytes(b"_death__").expect("valid cortical id");
    (first, second)
}

#[test]
fn bursting_with_allocated_areas_does_not_panic() {
    let (first, _) = two_cortical_ids();
    let mut npu = DynamicNPU::new();

    npu.request(add_area_request(first, 2, 2, 2, 1));

    // Before allocation was implemented the per-neuron vectors stayed empty, so the burst kernel
    // iterated nothing and this passed vacuously. With neurons present it exercises the kernel.
    npu.execute_single_burst();
}

#[test]
fn areas_are_registered_in_engine_index_order() {
    let (first, second) = two_cortical_ids();
    let mut npu = DynamicNPU::new();

    npu.request(add_area_request(first, 1, 1, 1, 1));
    npu.request(add_area_request(second, 1, 1, 1, 1));

    assert_eq!(npu.cortical_areas(), &[first, second]);
}

/// The current `FeagiAdvancedModel` dynamics are a placeholder that fires every neuron on every
/// sixteenth burst and nothing in between. These tests pin the readout against that rhythm; when
/// real dynamics land they will need new stimulus, not a new readout.
const PLACEHOLDER_FIRING_PERIOD: usize = 16;

#[test]
fn quiet_bursts_report_no_areas() {
    let (first, _) = two_cortical_ids();
    let mut npu = DynamicNPU::new();

    npu.request(add_area_request(first, 2, 3, 4, 1));

    // Burst zero fires under the placeholder dynamics, so step past it before sampling.
    npu.execute_single_burst();
    npu.execute_single_burst();

    // Quiet areas are omitted entirely rather than reported with empty coordinate vectors.
    assert!(npu.fire_queue_snapshot().is_empty());
}

#[test]
fn firing_burst_reports_every_neuron_once_at_its_own_coordinate() {
    let (first, _) = two_cortical_ids();
    let (x, y, z) = (2usize, 3usize, 4usize);
    let mut npu = DynamicNPU::new();

    npu.request(add_area_request(first, x, y, z, 1));
    npu.execute_single_burst();

    let snapshot = npu.fire_queue_snapshot();
    assert_eq!(snapshot.len(), 1, "the single area should be reported");

    let (reported_id, area) = &snapshot[0];
    assert_eq!(*reported_id, first, "snapshot should be keyed by cortical id");
    assert_eq!(area.len(), x * y * z, "every neuron in the area fires");

    // Each neuron should map to a distinct coordinate covering the whole volume, which is what
    // proves the linear-index-to-coordinate walk lines up with the area's dimensions.
    let mut reported: Vec<(u32, u32, u32)> = (0..area.len()).map(|i| (area.coords_x[i], area.coords_y[i], area.coords_z[i])).collect();
    reported.sort_unstable();

    let mut expected: Vec<(u32, u32, u32)> = Vec::new();
    for zi in 0..z as u32 {
        for yi in 0..y as u32 {
            for xi in 0..x as u32 {
                expected.push((xi, yi, zi));
            }
        }
    }
    expected.sort_unstable();

    assert_eq!(reported, expected);
}

#[test]
fn firing_recurs_on_the_placeholder_period() {
    let (first, _) = two_cortical_ids();
    let mut npu = DynamicNPU::new();

    npu.request(add_area_request(first, 1, 1, 1, 1));

    let mut firing_bursts = Vec::new();
    for burst in 0..(PLACEHOLDER_FIRING_PERIOD * 2) {
        npu.execute_single_burst();
        if !npu.fire_queue_snapshot().is_empty() {
            firing_bursts.push(burst);
        }
    }

    assert_eq!(firing_bursts, vec![0, PLACEHOLDER_FIRING_PERIOD]);
}

/*
The readout is driven by the firing bitmap the burst packs, which stores one bit per neuron in
byte-aligned per-area runs. The tests below cover what that packing can get wrong and the
coordinate walk alone cannot: unused bits in a run's final byte, and one area's run disturbing
another's.
*/

#[test]
fn unused_bits_in_a_ragged_final_byte_are_not_reported_as_neurons() {
    let (first, _) = two_cortical_ids();
    let mut npu = DynamicNPU::new();

    // Three neurons occupy one byte, leaving five bits that are storage but not members.
    npu.request(add_area_request(first, 3, 1, 1, 1));
    npu.execute_single_burst();

    let snapshot = npu.fire_queue_snapshot();
    assert_eq!(snapshot.len(), 1, "the single area should be reported");
    assert_eq!(
        snapshot[0].1.len(),
        3,
        "the five unused bits of the byte must not be counted as firing neurons"
    );
}

#[test]
fn an_area_spanning_many_bytes_reports_every_neuron_exactly_once() {
    let (first, _) = two_cortical_ids();
    let (x, y) = (10usize, 10usize);
    let mut npu = DynamicNPU::new();

    // A hundred neurons span thirteen bytes, the last of which is only half used.
    npu.request(add_area_request(first, x, y, 1, 1));
    npu.execute_single_burst();

    let snapshot = npu.fire_queue_snapshot();
    let area = &snapshot[0].1;
    assert_eq!(area.len(), x * y);

    let mut reported: Vec<(u32, u32)> = (0..area.len()).map(|i| (area.coords_x[i], area.coords_y[i])).collect();
    reported.sort_unstable();
    reported.dedup();
    assert_eq!(
        reported.len(),
        x * y,
        "every neuron should appear once, at a coordinate no other neuron claims"
    );
}

/// The regression test for overlapping runs. The allocator used to derive each new run's start
/// from the *first* run rather than the last, so the third area onwards was handed bytes already
/// owned by the second. Three areas are the minimum that exposes it.
#[test]
fn neighbouring_ragged_areas_do_not_bleed_into_each_other() {
    let (first, second) = two_cortical_ids();
    let third = CorticalID::try_from_bytes(b"_third__").expect("valid cortical id");
    let mut npu = DynamicNPU::new();

    // No count is a multiple of eight, so a layout that packed runs bit-tight rather than
    // byte-aligned would also make these areas share bytes.
    npu.request(add_area_request(first, 3, 1, 1, 1));
    npu.request(add_area_request(second, 5, 1, 1, 1));
    npu.request(add_area_request(third, 7, 1, 1, 1));
    npu.execute_single_burst();

    let snapshot = npu.fire_queue_snapshot();
    let counts: Vec<(CorticalID, usize)> = snapshot.iter().map(|(id, area)| (*id, area.len())).collect();

    assert_eq!(counts, vec![(first, 3), (second, 5), (third, 7)]);
}

#[test]
fn firing_bits_do_not_survive_into_the_next_burst() {
    let (first, _) = two_cortical_ids();
    let mut npu = DynamicNPU::new();

    // Multiple bytes, so a clear that missed some of them would still leave neurons reported.
    npu.request(add_area_request(first, 10, 10, 1, 1));

    npu.execute_single_burst();
    assert!(!npu.fire_queue_snapshot().is_empty(), "burst zero fires under the placeholder dynamics");

    npu.execute_single_burst();
    assert!(
        npu.fire_queue_snapshot().is_empty(),
        "last burst's bits must be cleared, not left standing"
    );
}

#[test]
fn snapshot_survives_multiple_areas_and_bursts() {
    let (first, second) = two_cortical_ids();
    let mut npu = DynamicNPU::new();

    npu.request(add_area_request(first, 2, 2, 2, 1));
    npu.request(add_area_request(second, 3, 1, 1, 2));

    // Repeated bursts re-run the kernel over both areas' neuron ranges; an allocation that got
    // the per-area offsets wrong would index out of bounds here rather than on the first burst.
    for _ in 0..5 {
        npu.execute_single_burst();
        let snapshot = npu.fire_queue_snapshot();
        for (_, area) in snapshot {
            assert_eq!(area.coords_x.len(), area.len());
            assert_eq!(area.coords_y.len(), area.len());
            assert_eq!(area.coords_z.len(), area.len());
        }
    }
}
