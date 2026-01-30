// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Synapse count tests for cortical areas.
//!
//! These tests validate incoming/outgoing synapse counts per area.

use feagi_brain_development::ConnectomeManager;
use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::{DynamicNPU, RustNPU, TracingMutex};
use feagi_npu_runtime::StdRuntime;
use feagi_structures::genomic::cortical_area::{
    CorticalArea, CorticalAreaDimensions, CorticalAreaType, CorticalID,
    IOCorticalAreaConfigurationFlag,
};
use feagi_services::impls::ConnectomeServiceImpl;
use feagi_services::traits::ConnectomeService;
use feagi_structures::genomic::brain_regions::{BrainRegion, RegionID, RegionType};
use parking_lot::RwLock;
use serde_json::json;
use std::sync::Arc;

fn create_test_manager(
) -> (
    ConnectomeManager,
    Arc<TracingMutex<DynamicNPU>>,
) {
    let runtime = StdRuntime;
    let backend = CPUBackend::new();
    let npu = RustNPU::new(runtime, backend, 10_000, 10_000, 10)
        .expect("Failed to create NPU");
    let dyn_npu = Arc::new(TracingMutex::new(DynamicNPU::F32(npu), "TestNPU"));
    let mut manager = ConnectomeManager::new_for_testing_with_npu(dyn_npu.clone());
    manager.setup_core_morphologies_for_testing();
    (manager, dyn_npu)
}

fn create_test_manager_arc(
) -> (
    Arc<RwLock<ConnectomeManager>>,
    Arc<TracingMutex<DynamicNPU>>,
) {
    let (mut manager, dyn_npu) = create_test_manager();
    manager.setup_core_morphologies_for_testing();
    (Arc::new(RwLock::new(manager)), dyn_npu)
}

fn create_area(
    cortical_id: CorticalID,
    cortical_idx: u32,
    name: &str,
    area_type: CorticalAreaType,
) -> CorticalArea {
    create_area_with_dimensions(
        cortical_id,
        cortical_idx,
        name,
        area_type,
        (2, 2, 1),
    )
}

fn create_area_with_dimensions(
    cortical_id: CorticalID,
    cortical_idx: u32,
    name: &str,
    area_type: CorticalAreaType,
    dimensions: (u32, u32, u32),
) -> CorticalArea {
    CorticalArea::new(
        cortical_id,
        cortical_idx,
        name.to_string(),
        CorticalAreaDimensions::new(dimensions.0, dimensions.1, dimensions.2)
            .expect("Invalid dimensions"),
        (0, 0, 0).into(),
        area_type,
    )
    .expect("Failed to create cortical area")
}

fn add_area_neurons(manager: &mut ConnectomeManager, cortical_id: &CorticalID) {
    for x in 0..2 {
        for y in 0..2 {
            manager
                .add_neuron(
                    cortical_id,
                    x,
                    y,
                    0,
                    1.0,
                    0.0,
                    0.1,
                    0.0,
                    0,
                    1,
                    1.0,
                    3,
                    1,
                    false,
                )
                .unwrap();
        }
    }
}

#[test]
fn test_incoming_outgoing_synapse_counts_by_area() {
    let (mut manager, _dyn_npu) = create_test_manager();

    let src_id = CorticalID::try_from_bytes(b"cst_s001").unwrap();
    let dst_id = CorticalID::try_from_bytes(b"cst_d001").unwrap();

    let src_area = create_area(
        src_id,
        0,
        "src",
        CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean),
    );
    let dst_area = create_area(
        dst_id,
        1,
        "dst",
        CorticalAreaType::BrainOutput(IOCorticalAreaConfigurationFlag::Boolean),
    );

    manager.add_cortical_area(src_area).unwrap();
    manager.add_cortical_area(dst_area).unwrap();

    let s0 = manager
        .add_neuron(&src_id, 0, 0, 0, 1.0, 0.0, 0.1, 0.0, 0, 1, 1.0, 3, 1, false)
        .unwrap();
    let s1 = manager
        .add_neuron(&src_id, 1, 0, 0, 1.0, 0.0, 0.1, 0.0, 0, 1, 1.0, 3, 1, false)
        .unwrap();
    let t0 = manager
        .add_neuron(&dst_id, 0, 0, 0, 1.0, 0.0, 0.1, 0.0, 0, 1, 1.0, 3, 1, false)
        .unwrap();
    let t1 = manager
        .add_neuron(&dst_id, 1, 0, 0, 1.0, 0.0, 0.1, 0.0, 0, 1, 1.0, 3, 1, false)
        .unwrap();

    // Create synapses: src -> dst (2) and dst -> src (1)
    manager.create_synapse(s0, t0, 128, 200, 0).unwrap();
    manager.create_synapse(s1, t1, 128, 200, 0).unwrap();
    manager.create_synapse(t0, s1, 128, 200, 0).unwrap();

    assert_eq!(
        manager.get_outgoing_synapse_count_in_area(&src_id),
        2
    );
    assert_eq!(
        manager.get_incoming_synapse_count_in_area(&src_id),
        1
    );
    assert_eq!(
        manager.get_outgoing_synapse_count_in_area(&dst_id),
        1
    );
    assert_eq!(
        manager.get_incoming_synapse_count_in_area(&dst_id),
        2
    );
}

#[test]
fn test_projector_morphology_counts_by_area() {
    let (mut manager, dyn_npu) = create_test_manager();

    let src_id = CorticalID::try_from_bytes(b"cst_s002").unwrap();
    let dst_id = CorticalID::try_from_bytes(b"cst_d002").unwrap();

    let src_area = create_area(
        src_id,
        0,
        "src",
        CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean),
    );
    let dst_area = create_area(
        dst_id,
        1,
        "dst",
        CorticalAreaType::BrainOutput(IOCorticalAreaConfigurationFlag::Boolean),
    );

    manager.add_cortical_area(src_area).unwrap();
    manager.add_cortical_area(dst_area).unwrap();

    add_area_neurons(&mut manager, &src_id);
    add_area_neurons(&mut manager, &dst_id);

    let mapping_data = vec![json!({
        "morphology_id": "projector",
        "morphology_scalar": [1, 1, 1],
        "plasticity_flag": false,
        "postSynapticCurrent_multiplier": 1,
        "synapse_attractivity": 100
    })];
    manager
        .update_cortical_mapping(&src_id, &dst_id, mapping_data)
        .unwrap();
    manager
        .regenerate_synapses_for_mapping(&src_id, &dst_id)
        .unwrap();

    let (
        expected_outgoing_src,
        expected_incoming_dst,
        expected_outgoing_dst,
        expected_incoming_src,
    ) = {
        let mut npu = dyn_npu.lock().unwrap();
        npu.rebuild_synapse_index();
        let src_idx = manager.get_cortical_idx(&src_id).unwrap();
        let dst_idx = manager.get_cortical_idx(&dst_id).unwrap();
        let src_neurons = npu.get_neurons_in_cortical_area(src_idx);
        let dst_neurons = npu.get_neurons_in_cortical_area(dst_idx);

        let outgoing_src: usize = src_neurons
            .iter()
            .map(|neuron_id| npu.get_outgoing_synapses(*neuron_id).len())
            .sum();
        let incoming_dst: usize = dst_neurons
            .iter()
            .map(|neuron_id| npu.get_incoming_synapses(*neuron_id).len())
            .sum();
        let outgoing_dst: usize = dst_neurons
            .iter()
            .map(|neuron_id| npu.get_outgoing_synapses(*neuron_id).len())
            .sum();
        let incoming_src: usize = src_neurons
            .iter()
            .map(|neuron_id| npu.get_incoming_synapses(*neuron_id).len())
            .sum();

        (outgoing_src, incoming_dst, outgoing_dst, incoming_src)
    };

    assert_eq!(
        manager.get_outgoing_synapse_count_in_area(&src_id),
        expected_outgoing_src
    );
    assert_eq!(
        manager.get_incoming_synapse_count_in_area(&dst_id),
        expected_incoming_dst
    );
    assert_eq!(
        manager.get_incoming_synapse_count_in_area(&src_id),
        expected_incoming_src
    );
    assert_eq!(
        manager.get_outgoing_synapse_count_in_area(&dst_id),
        expected_outgoing_dst
    );
}

#[test]
fn test_block_to_block_counts_single_neuron() {
    let (mut manager, dyn_npu) = create_test_manager();

    let src_id = CorticalID::try_from_bytes(b"cst_s003").unwrap();
    let dst_id = CorticalID::try_from_bytes(b"cst_d003").unwrap();

    let src_area = create_area_with_dimensions(
        src_id,
        0,
        "src",
        CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean),
        (1, 1, 1),
    );
    let dst_area = create_area_with_dimensions(
        dst_id,
        1,
        "dst",
        CorticalAreaType::BrainOutput(IOCorticalAreaConfigurationFlag::Boolean),
        (1, 1, 1),
    );

    manager.add_cortical_area(src_area).unwrap();
    manager.add_cortical_area(dst_area).unwrap();

    let _src_neuron = manager
        .add_neuron(&src_id, 0, 0, 0, 1.0, 0.0, 0.1, 0.0, 0, 1, 1.0, 3, 1, false)
        .unwrap();
    let _dst_neuron = manager
        .add_neuron(&dst_id, 0, 0, 0, 1.0, 0.0, 0.1, 0.0, 0, 1, 1.0, 3, 1, false)
        .unwrap();

    let mapping_data = vec![json!({
        "morphology_id": "block_to_block",
        "morphology_scalar": [1, 1, 1],
        "plasticity_flag": false,
        "postSynapticCurrent_multiplier": 1,
        "synapse_attractivity": 100
    })];
    manager
        .update_cortical_mapping(&src_id, &dst_id, mapping_data)
        .unwrap();
    manager
        .regenerate_synapses_for_mapping(&src_id, &dst_id)
        .unwrap();

    let (outgoing_src, incoming_dst) = {
        let mut npu = dyn_npu.lock().unwrap();
        npu.rebuild_synapse_index();
        let src_idx = manager.get_cortical_idx(&src_id).unwrap();
        let dst_idx = manager.get_cortical_idx(&dst_id).unwrap();
        let src_neurons = npu.get_neurons_in_cortical_area(src_idx);
        let dst_neurons = npu.get_neurons_in_cortical_area(dst_idx);
        assert_eq!(src_neurons.len(), 1);
        assert_eq!(dst_neurons.len(), 1);
        let outgoing_src: usize = src_neurons
            .iter()
            .map(|neuron_id| npu.get_outgoing_synapses(*neuron_id).len())
            .sum();
        let incoming_dst: usize = dst_neurons
            .iter()
            .map(|neuron_id| npu.get_incoming_synapses(*neuron_id).len())
            .sum();
        (outgoing_src, incoming_dst)
    };

    assert_eq!(outgoing_src, 1);
    assert_eq!(incoming_dst, 1);

    assert_eq!(
        manager.get_outgoing_synapse_count_in_area(&src_id),
        1
    );
    assert_eq!(
        manager.get_incoming_synapse_count_in_area(&dst_id),
        1
    );
}

#[test]
fn test_update_cortical_mapping_is_idempotent() {
    let (manager_arc, _dyn_npu) = create_test_manager_arc();
    let current_genome = Arc::new(RwLock::new(None));
    let service = ConnectomeServiceImpl::new(manager_arc.clone(), current_genome);

    let src_id = CorticalID::try_from_bytes(b"cst_s004").unwrap();
    let dst_id = CorticalID::try_from_bytes(b"cst_d004").unwrap();

    {
        let mut manager = manager_arc.write();
        let region = BrainRegion::new(RegionID::new(), "test".to_string(), RegionType::Undefined)
            .unwrap()
            .with_areas([src_id, dst_id]);
        manager.add_brain_region(region, None).unwrap();

        let src_area = create_area_with_dimensions(
            src_id,
            0,
            "src",
            CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean),
            (1, 1, 1),
        );
        let dst_area = create_area_with_dimensions(
            dst_id,
            1,
            "dst",
            CorticalAreaType::BrainOutput(IOCorticalAreaConfigurationFlag::Boolean),
            (1, 1, 1),
        );
        manager.add_cortical_area(src_area).unwrap();
        manager.add_cortical_area(dst_area).unwrap();
        manager
            .add_neuron(&src_id, 0, 0, 0, 1.0, 0.0, 0.1, 0.0, 0, 1, 1.0, 3, 1, false)
            .unwrap();
        manager
            .add_neuron(&dst_id, 0, 0, 0, 1.0, 0.0, 0.1, 0.0, 0, 1, 1.0, 3, 1, false)
            .unwrap();
    }

    let mapping_rules = vec![json!({
        "morphology_id": "block_to_block",
        "morphology_scalar": [1, 1, 1],
        "plasticity_flag": false,
        "postSynapticCurrent_multiplier": 1,
        "synapse_attractivity": 100
    })];

    let rt = tokio::runtime::Runtime::new().unwrap();
    let first_count = rt
        .block_on(service.update_cortical_mapping(
            src_id.as_base_64(),
            dst_id.as_base_64(),
            mapping_rules.clone(),
        ))
        .unwrap();
    assert_eq!(first_count, 1);

    let second_count = rt
        .block_on(service.update_cortical_mapping(
            src_id.as_base_64(),
            dst_id.as_base_64(),
            mapping_rules,
        ))
        .unwrap();
    assert_eq!(second_count, 0);

    let manager = manager_arc.read();
    assert_eq!(manager.get_outgoing_synapse_count_in_area(&src_id), 1);
    assert_eq!(manager.get_incoming_synapse_count_in_area(&dst_id), 1);
}
