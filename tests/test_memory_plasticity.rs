// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Memory + plasticity integration tests.

These tests validate:
- Memory area registration configures FireLedger tracking
- Upstream firing is archived and available for pattern detection
- Memory neuron allocation behavior is correct and deterministic
*/

use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::{DynamicNPU, TracingMutex};
use feagi_npu_neural::types::NeuronId;
use feagi_npu_plasticity::{
    MemoryNeuronArray, MemoryNeuronLifecycleConfig, PatternConfig, PatternDetector,
    PlasticityCommand, PlasticityConfig, PlasticityService,
};
use feagi_npu_runtime::StdRuntime;
use std::collections::HashSet;
use std::sync::Arc;

fn build_npu(tag: &'static str) -> Arc<TracingMutex<DynamicNPU>> {
    Arc::new(TracingMutex::new(
        DynamicNPU::new_f32(StdRuntime::new(), CPUBackend::new(), 16, 16, 8)
            .expect("Failed to create NPU"),
        tag,
    ))
}

fn create_single_neuron_area(npu: &Arc<TracingMutex<DynamicNPU>>, cortical_idx: u32, name: &str) {
    let mut npu_lock = npu.lock().unwrap();
    npu_lock.register_cortical_area(cortical_idx, name.to_string());
    npu_lock
        .create_cortical_area_neurons(
            cortical_idx,
            1,
            1,
            1,
            1,
            0.1,
            0.0,
            0.0,
            0.0,
            f32::MAX,
            0.0,
            0.0,
            0,
            0,
            1.0,
            0,
            0,
            false,
        )
        .expect("Failed to create neurons");
}

fn inject_and_burst(
    npu: &Arc<TracingMutex<DynamicNPU>>,
    neuron_id: u32,
    potential: f32,
) -> u64 {
    let mut npu_lock = npu.lock().unwrap();
    npu_lock.inject_sensory_with_potentials(&[(NeuronId(neuron_id), potential)]);
    npu_lock.process_burst().expect("Burst failed").burst
}

fn drain_plasticity_commands(service: &PlasticityService) -> Vec<PlasticityCommand> {
    let mut commands = Vec::new();
    for _ in 0..500 {
        let drained = service.drain_commands();
        if !drained.is_empty() {
            commands.extend(drained);
            break;
        }
    }
    commands
}

fn apply_plasticity_commands(npu: &Arc<TracingMutex<DynamicNPU>>, commands: &[PlasticityCommand]) {
    if commands.is_empty() {
        return;
    }
    let mut npu_lock = npu.lock().unwrap();
    for command in commands {
        match command {
            PlasticityCommand::RegisterMemoryNeuron { .. } => {}
            PlasticityCommand::InjectMemoryNeuronToFCL {
                neuron_id,
                area_idx,
                membrane_potential,
                ..
            } => {
                npu_lock.inject_memory_neuron_to_fcl(*neuron_id, *area_idx, *membrane_potential);
            }
            PlasticityCommand::UpdateWeightsDelta { .. } => {}
            PlasticityCommand::UpdateStateCounters { .. } => {}
        }
    }
}

#[test]
fn test_memory_area_registration_tracks_fire_ledger() {
    let npu = build_npu("memory-fire-ledger-test-npu");
    let service = PlasticityService::new(
        PlasticityConfig::default(),
        feagi_npu_plasticity::create_memory_stats_cache(),
        npu.clone(),
    );

    let upstream_idx = 7u32;
    let temporal_depth = 3u32;
    service.register_memory_area(
        100,
        "mem_00".to_string(),
        temporal_depth,
        vec![upstream_idx],
        None,
    );

    let configs = npu.lock().unwrap().get_all_fire_ledger_configs();
    let window = configs
        .iter()
        .find(|(idx, _)| *idx == upstream_idx)
        .map(|(_, w)| *w)
        .expect("FireLedger window should be configured for upstream area");
    assert_eq!(window, temporal_depth as usize);
}

#[test]
fn test_upstream_firing_is_available_for_pattern_detection() {
    let npu = build_npu("memory-pattern-test-npu");
    create_single_neuron_area(&npu, 7, "upstream");

    let service = PlasticityService::new(
        PlasticityConfig::default(),
        feagi_npu_plasticity::create_memory_stats_cache(),
        npu.clone(),
    );
    service.register_memory_area(100, "mem_00".to_string(), 1, vec![7], None);

    let upstream_neuron_id = {
        let npu_lock = npu.lock().unwrap();
        let ids = npu_lock.get_neurons_in_cortical_area(7);
        assert_eq!(ids.len(), 1);
        ids[0]
    };

    let burst = inject_and_burst(&npu, upstream_neuron_id, 10.0);

    let window = npu
        .lock()
        .unwrap()
        .get_fire_ledger_dense_window_bitmaps(7, burst, 1)
        .expect("FireLedger window unavailable");
    assert_eq!(window.len(), 1);

    let fired_ids: HashSet<u32> = window[0].1.iter().collect();
    assert!(fired_ids.contains(&upstream_neuron_id));

    let detector = PatternDetector::new(PatternConfig::default());
    let pattern = detector.detect_pattern(100, &[7], burst, vec![fired_ids], Some(1));
    assert!(pattern.is_some());
}

#[test]
fn test_memory_neuron_allocation_is_deterministic() {
    let mut memory_array = MemoryNeuronArray::new(100);
    let lifecycle_config = MemoryNeuronLifecycleConfig::default();

    let neuron_a = memory_array
        .create_memory_neuron(0xAAAA, 10, 1, &lifecycle_config)
        .expect("Failed to create first memory neuron");
    let neuron_b = memory_array
        .create_memory_neuron(0xBBBB, 10, 2, &lifecycle_config)
        .expect("Failed to create second memory neuron");

    let id_a = memory_array
        .get_neuron_id(neuron_a)
        .expect("Missing memory neuron ID");
    let id_b = memory_array
        .get_neuron_id(neuron_b)
        .expect("Missing memory neuron ID");

    assert_ne!(id_a, id_b);
    assert!(feagi_npu_plasticity::NeuronIdManager::is_memory_neuron_id(id_a));
    assert!(feagi_npu_plasticity::NeuronIdManager::is_memory_neuron_id(id_b));
}

#[test]
fn test_memory_command_flow_creates_and_reactivates() {
    let npu = build_npu("memory-command-flow-npu");
    create_single_neuron_area(&npu, 7, "upstream");

    let service = PlasticityService::new(
        PlasticityConfig::default(),
        feagi_npu_plasticity::create_memory_stats_cache(),
        npu.clone(),
    );
    service.start();
    service.register_memory_area(100, "mem_00".to_string(), 1, vec![7], None);

    let upstream_neuron_id = {
        let npu_lock = npu.lock().unwrap();
        let ids = npu_lock.get_neurons_in_cortical_area(7);
        ids[0]
    };

    let burst = inject_and_burst(&npu, upstream_neuron_id, 10.0);
    service.notify_burst(burst);

    let commands = drain_plasticity_commands(&service);
    assert!(
        commands
            .iter()
            .any(|c| matches!(c, PlasticityCommand::InjectMemoryNeuronToFCL { .. })),
        "Expected InjectMemoryNeuronToFCL command"
    );

    apply_plasticity_commands(&npu, &commands);
    let result = {
        let npu_lock = npu.lock().unwrap();
        npu_lock.process_burst().expect("Burst failed")
    };
    let memory_fired = result
        .fired_neurons
        .iter()
        .any(|id| feagi_npu_plasticity::NeuronIdManager::is_memory_neuron_id(id.0));
    assert!(memory_fired, "Expected memory neuron to fire after injection");

    let burst_again = inject_and_burst(&npu, upstream_neuron_id, 10.0);
    service.notify_burst(burst_again);
    let commands_again = drain_plasticity_commands(&service);
    let reactivation = commands_again.iter().any(|c| match c {
        PlasticityCommand::InjectMemoryNeuronToFCL { is_reactivation, .. } => *is_reactivation,
        _ => false,
    });
    assert!(reactivation, "Expected reactivation command");
}

#[test]
fn test_insufficient_history_blocks_pattern_detection_until_ready() {
    let npu = build_npu("memory-history-test-npu");
    create_single_neuron_area(&npu, 7, "upstream");

    let service = PlasticityService::new(
        PlasticityConfig::default(),
        feagi_npu_plasticity::create_memory_stats_cache(),
        npu.clone(),
    );
    service.start();
    service.register_memory_area(100, "mem_00".to_string(), 2, vec![7], None);

    let upstream_neuron_id = {
        let npu_lock = npu.lock().unwrap();
        npu_lock.get_neurons_in_cortical_area(7)[0]
    };

    let burst = inject_and_burst(&npu, upstream_neuron_id, 10.0);
    service.notify_burst(burst);
    let commands = drain_plasticity_commands(&service);
    assert!(
        commands.is_empty(),
        "No commands should be emitted with insufficient history"
    );

    let burst2 = inject_and_burst(&npu, upstream_neuron_id, 10.0);
    service.notify_burst(burst2);
    let commands_ready = drain_plasticity_commands(&service);
    assert!(
        commands_ready
            .iter()
            .any(|c| matches!(c, PlasticityCommand::InjectMemoryNeuronToFCL { .. })),
        "Expected commands once history window is complete"
    );
}
