//! Integration tests for the `prepopulate_from_byte_slice` helper on
//! [`CorticalMappedNeuronVoxelCoordVectors`] combined with the v2 wire-format
//! deserializer.
//!
//! These tests validate the "external consumer" flow (e.g. Brain Visualizer):
//!   1. Producer serializes a populated `CorticalMappedNeuronVoxelCoordVectors`.
//!   2. Consumer receives the byte slice but does NOT know the list of cortical
//!      ids inside. The consumer DOES know authoritative dimensions for every
//!      cortical area in its current genome snapshot.
//!   3. Consumer creates an empty `CorticalMappedNeuronVoxelCoordVectors`,
//!      calls `prepopulate_from_byte_slice` with its dims map, then runs the
//!      standard `try_deserialize_and_update_self_from_byte_slice` pass.
//!   4. Round-trip equality holds on voxel counts per cortical id.

use ahash::AHashMap;
use feagi_serialization::FeagiSerializable;
use feagi_structures::genomic::cortical_area::{CoreCorticalType, CorticalID};
use feagi_structures::neuron_voxels::coord_potential::{
    CorticalMappedNeuronVoxelCoordVectors, NeuronVoxelCoordVector,
};
use feagi_structures::neuron_voxels::descriptors::{
    NeuronVoxelCoordinate, NeuronVoxelDimensions, NeuronVoxelPotential,
};
use feagi_structures::neuron_voxels::traits::SingleCorticalNeuronVoxelCollectionAlloc;

type Vectors = CorticalMappedNeuronVoxelCoordVectors<f32, u32, u32, u16>;

fn build_producer_snapshot() -> (Vectors, AHashMap<CorticalID, NeuronVoxelDimensions<u32>>) {
    let mut snapshot: Vectors = CorticalMappedNeuronVoxelCoordVectors::new();
    let mut dims_by_id: AHashMap<CorticalID, NeuronVoxelDimensions<u32>> = AHashMap::new();

    let id_a = CoreCorticalType::Power.to_cortical_id();
    let dims_a = NeuronVoxelDimensions::<u32>::new(4, 5, 6).unwrap();
    let mut vec_a: NeuronVoxelCoordVector<f32, u32, u32> =
        NeuronVoxelCoordVector::new(dims_a, 0u32);
    vec_a.push_neuron_voxel_unchecked(
        NeuronVoxelCoordinate::<u32>::new(0, 0, 0),
        NeuronVoxelPotential::from(0.25f32),
    );
    vec_a.push_neuron_voxel_unchecked(
        NeuronVoxelCoordinate::<u32>::new(1, 2, 3),
        NeuronVoxelPotential::from(0.75f32),
    );
    snapshot.insert(id_a, vec_a);
    dims_by_id.insert(id_a, dims_a);

    let id_b = CoreCorticalType::Death.to_cortical_id();
    let dims_b = NeuronVoxelDimensions::<u32>::new(2, 2, 2).unwrap();
    let mut vec_b: NeuronVoxelCoordVector<f32, u32, u32> =
        NeuronVoxelCoordVector::new(dims_b, 0u32);
    vec_b.push_neuron_voxel_unchecked(
        NeuronVoxelCoordinate::<u32>::new(1, 1, 1),
        NeuronVoxelPotential::from(1.0f32),
    );
    snapshot.insert(id_b, vec_b);
    dims_by_id.insert(id_b, dims_b);

    (snapshot, dims_by_id)
}

fn serialize(snapshot: &Vectors) -> Vec<u8> {
    let mut buf = vec![0u8; snapshot.get_number_of_bytes_needed()];
    snapshot
        .try_serialize_struct_to_byte_slice(&mut buf)
        .unwrap();
    buf
}

#[test]
fn prepopulate_then_deserialize_round_trip_preserves_content() {
    let (producer, dims_by_id) = build_producer_snapshot();
    let bytes = serialize(&producer);

    let mut consumer: Vectors = CorticalMappedNeuronVoxelCoordVectors::new();
    consumer
        .prepopulate_from_byte_slice(&bytes, &dims_by_id)
        .unwrap();

    assert_eq!(
        consumer.len(),
        producer.len(),
        "prepopulate must install one entry per cortical id in the wire header"
    );

    consumer
        .try_deserialize_and_update_self_from_byte_slice(&bytes)
        .unwrap();

    for (id, producer_vec) in producer.iter() {
        let consumer_vec = consumer
            .get(id)
            .expect("deserialized consumer should contain every producer id");
        assert_eq!(
            consumer_vec.get_number_neuron_voxel_contained_count(),
            producer_vec.get_number_neuron_voxel_contained_count(),
            "voxel count mismatch for cortical id {:?}",
            id
        );
    }
}

#[test]
fn prepopulate_fails_when_dims_missing_for_incoming_id() {
    let (producer, _full_dims) = build_producer_snapshot();
    let bytes = serialize(&producer);

    let id_a = CoreCorticalType::Power.to_cortical_id();
    let mut partial_dims: AHashMap<CorticalID, NeuronVoxelDimensions<u32>> = AHashMap::new();
    partial_dims.insert(id_a, NeuronVoxelDimensions::<u32>::new(4, 5, 6).unwrap());

    let mut consumer: Vectors = CorticalMappedNeuronVoxelCoordVectors::new();
    let err = consumer
        .prepopulate_from_byte_slice(&bytes, &partial_dims)
        .expect_err("missing dim entry must be a hard error");
    let msg = format!("{:?}", err);
    assert!(
        msg.contains("missing from dims_by_cortical_id"),
        "error should name the contract violation, got: {}",
        msg
    );
}

#[test]
fn prepopulate_fails_on_truncated_header() {
    let (producer, dims_by_id) = build_producer_snapshot();
    let bytes = serialize(&producer);

    let truncated = &bytes[..bytes.len().min(3)];
    let mut consumer: Vectors = CorticalMappedNeuronVoxelCoordVectors::new();
    consumer
        .prepopulate_from_byte_slice(truncated, &dims_by_id)
        .expect_err("truncated header must fail prepopulation");
}
