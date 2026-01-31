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

/// Create a cortical area with multiple neurons and return their IDs.
fn create_multi_neuron_area(
    npu: &Arc<TracingMutex<DynamicNPU>>,
    cortical_idx: u32,
    name: &str,
    neuron_count: u32,
) -> Vec<u32> {
    let mut npu_lock = npu.lock().unwrap();
    npu_lock.register_cortical_area(cortical_idx, name.to_string());
    npu_lock
        .create_cortical_area_neurons(
            cortical_idx,
            neuron_count,
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
    npu_lock.get_neurons_in_cortical_area(cortical_idx)
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

fn wait_for_commands(service: &PlasticityService) -> Vec<PlasticityCommand> {
    let mut commands = Vec::new();
    for _ in 0..10_000 {
        let drained = service.drain_commands();
        if !drained.is_empty() {
            commands.extend(drained);
            break;
        }
        std::thread::yield_now();
    }
    commands
}

fn wait_for_memory_neurons(service: &PlasticityService, expected_count: usize) -> bool {
    for _ in 0..10_000 {
        let stats = service.get_stats();
        if stats.memory_neurons_created >= expected_count {
            return true;
        }
        std::thread::yield_now();
    }
    false
}

fn apply_plasticity_commands(npu: &Arc<TracingMutex<DynamicNPU>>, commands: &[PlasticityCommand]) {
    if commands.is_empty() {
        return;
    }
    let mut npu_lock = npu.lock().unwrap();
    for command in commands {
        match command {
            PlasticityCommand::RegisterMemoryNeuron { .. } => {}
            PlasticityCommand::MemoryNeuronConvertedToLtm { .. } => {}
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

    let commands = wait_for_commands(&service);
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
    let commands_again = wait_for_commands(&service);
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
    let commands = wait_for_commands(&service);
    assert!(
        commands.is_empty(),
        "No commands should be emitted with insufficient history"
    );

    let burst2 = inject_and_burst(&npu, upstream_neuron_id, 10.0);
    service.notify_burst(burst2);
    let commands_ready = wait_for_commands(&service);
    assert!(
        commands_ready
            .iter()
            .any(|c| matches!(c, PlasticityCommand::InjectMemoryNeuronToFCL { .. })),
        "Expected commands once history window is complete"
    );
}

#[test]
fn test_memory_area_temporal_depth_zero_never_detects() {
    let npu = build_npu("memory-temporal-depth-zero-npu");
    create_single_neuron_area(&npu, 7, "upstream");

    let service = PlasticityService::new(
        PlasticityConfig::default(),
        feagi_npu_plasticity::create_memory_stats_cache(),
        npu.clone(),
    );
    service.start();
    service.register_memory_area(100, "mem_00".to_string(), 0, vec![7], None);

    let upstream_neuron_id = {
        let npu_lock = npu.lock().unwrap();
        npu_lock.get_neurons_in_cortical_area(7)[0]
    };

    let burst = inject_and_burst(&npu, upstream_neuron_id, 10.0);
    service.notify_burst(burst);

    let commands = drain_plasticity_commands(&service);
    assert!(
        commands.is_empty(),
        "No commands expected when temporal_depth is zero"
    );
}

#[test]
fn test_memory_area_with_no_upstream_areas_emits_no_commands() {
    let npu = build_npu("memory-no-upstream-npu");
    create_single_neuron_area(&npu, 7, "upstream");

    let service = PlasticityService::new(
        PlasticityConfig::default(),
        feagi_npu_plasticity::create_memory_stats_cache(),
        npu.clone(),
    );
    service.start();
    service.register_memory_area(100, "mem_00".to_string(), 1, Vec::new(), None);

    let upstream_neuron_id = {
        let npu_lock = npu.lock().unwrap();
        npu_lock.get_neurons_in_cortical_area(7)[0]
    };

    let burst = inject_and_burst(&npu, upstream_neuron_id, 10.0);
    service.notify_burst(burst);

    let commands = drain_plasticity_commands(&service);
    assert!(
        commands.is_empty(),
        "No commands expected when upstream_areas is empty"
    );
}

#[test]
fn test_multi_upstream_memory_area_requires_full_history_window() {
    let npu = build_npu("memory-multi-upstream-window-npu");
    let upstream_a = create_multi_neuron_area(&npu, 7, "upstream_a", 1)[0];
    let upstream_b = create_multi_neuron_area(&npu, 8, "upstream_b", 1)[0];

    let service = PlasticityService::new(
        PlasticityConfig::default(),
        feagi_npu_plasticity::create_memory_stats_cache(),
        npu.clone(),
    );
    service.start();
    service.register_memory_area(100, "mem_00".to_string(), 2, vec![7, 8], None);

    let burst1 = {
        let mut npu_lock = npu.lock().unwrap();
        npu_lock.inject_sensory_with_potentials(&[
            (NeuronId(upstream_a), 10.0),
            (NeuronId(upstream_b), 10.0),
        ]);
        npu_lock.process_burst().expect("Burst failed").burst
    };
    service.notify_burst(burst1);

    let commands_first = drain_plasticity_commands(&service);
    assert!(
        commands_first.is_empty(),
        "No commands expected before temporal window is filled"
    );

    let burst2 = {
        let mut npu_lock = npu.lock().unwrap();
        npu_lock.inject_sensory_with_potentials(&[
            (NeuronId(upstream_a), 10.0),
            (NeuronId(upstream_b), 10.0),
        ]);
        npu_lock.process_burst().expect("Burst failed").burst
    };
    service.notify_burst(burst2);

    let commands_ready = wait_for_commands(&service);
    assert!(
        commands_ready
            .iter()
            .any(|c| matches!(c, PlasticityCommand::InjectMemoryNeuronToFCL { .. })),
        "Expected commands once temporal window is filled for all upstream areas"
    );
}

#[test]
fn test_late_notify_with_temporal_depth_one_misses_history() {
    let npu = build_npu("memory-late-notify-depth-one-npu");
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
        npu_lock.get_neurons_in_cortical_area(7)[0]
    };

    let burst1 = inject_and_burst(&npu, upstream_neuron_id, 10.0);
    let burst2 = inject_and_burst(&npu, upstream_neuron_id, 10.0);

    // Notify for burst1 after burst2 has already been archived.
    service.notify_burst(burst1);

    let commands = drain_plasticity_commands(&service);
    assert!(
        commands.is_empty(),
        "Expected no commands when notifying late with temporal_depth=1"
    );

    service.notify_burst(burst2);
    let commands_latest = wait_for_commands(&service);
    assert!(
        commands_latest
            .iter()
            .any(|c| matches!(c, PlasticityCommand::InjectMemoryNeuronToFCL { .. })),
        "Expected commands when notifying the latest burst with temporal_depth=1"
    );
}

#[test]
fn test_late_notify_with_temporal_depth_two_still_misses() {
    let npu = build_npu("memory-late-notify-depth-two-npu");
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

    let burst1 = inject_and_burst(&npu, upstream_neuron_id, 10.0);
    let _burst2 = inject_and_burst(&npu, upstream_neuron_id, 10.0);
    let _burst3 = inject_and_burst(&npu, upstream_neuron_id, 10.0);

    // Notify for burst1 after multiple newer bursts have been archived.
    service.notify_burst(burst1);

    let commands = wait_for_commands(&service);
    assert!(
        commands.is_empty(),
        "Expected no commands when notifying late with temporal_depth=2"
    );
}

#[test]
fn test_longterm_memory_converts_and_never_dies() {
    let npu = build_npu("memory-longterm-immortal-npu");
    create_single_neuron_area(&npu, 7, "upstream");

    let service = PlasticityService::new(
        PlasticityConfig::default(),
        feagi_npu_plasticity::create_memory_stats_cache(),
        npu.clone(),
    );
    service.start();

    let lifecycle_config = MemoryNeuronLifecycleConfig {
        initial_lifespan: 5,
        longterm_threshold: 5,
        lifespan_growth_rate: 1.0,
        max_reactivations: 1000,
    };
    service.register_memory_area(
        100,
        "mem_00".to_string(),
        1,
        vec![7],
        Some(lifecycle_config),
    );

    let upstream_neuron_id = {
        let npu_lock = npu.lock().unwrap();
        npu_lock.get_neurons_in_cortical_area(7)[0]
    };

    let burst = inject_and_burst(&npu, upstream_neuron_id, 10.0);
    service.notify_burst(burst);
    let _commands = wait_for_commands(&service);
    assert!(
        wait_for_memory_neurons(&service, 1),
        "Expected memory neuron creation before long-term conversion"
    );

    // Next burst converts to long-term before any aging happens.
    let burst2 = {
        let npu_lock = npu.lock().unwrap();
        npu_lock.process_burst().expect("Burst failed").burst
    };
    service.notify_burst(burst2);
    let _commands2 = wait_for_commands(&service);

    let stats_after_conversion = service
        .get_memory_neuron_array()
        .lock()
        .unwrap()
        .get_stats();
    assert_eq!(stats_after_conversion.active_neurons, 1);
    assert_eq!(stats_after_conversion.longterm_neurons, 1);

    for _ in 0..10 {
        let burst_n = {
            let npu_lock = npu.lock().unwrap();
            npu_lock.process_burst().expect("Burst failed").burst
        };
        service.notify_burst(burst_n);
        let _ = wait_for_commands(&service);
    }

    let stats_final = service
        .get_memory_neuron_array()
        .lock()
        .unwrap()
        .get_stats();
    assert_eq!(stats_final.active_neurons, 1);
    assert_eq!(stats_final.longterm_neurons, 1);
}

/// Validate memory neuron count stalls when upstream patterns are limited.
#[test]
fn test_memory_neuron_count_stalls_with_limited_unique_patterns() {
    let npu = build_npu("memory-limited-pattern-npu");
    let upstream_neuron_ids = create_multi_neuron_area(&npu, 7, "upstream", 3);

    let service = PlasticityService::new(
        PlasticityConfig::default(),
        feagi_npu_plasticity::create_memory_stats_cache(),
        npu.clone(),
    );
    service.start();
    let lifecycle_config = MemoryNeuronLifecycleConfig {
        initial_lifespan: 9,
        lifespan_growth_rate: 9.0,
        longterm_threshold: 9,
        max_reactivations: 1000,
    };
    service.register_memory_area(100, "mem_00".to_string(), 1, vec![7], Some(lifecycle_config));

    let sequence = [0usize, 1, 2, 0, 1, 2, 0, 1, 2, 0];
    for idx in sequence {
        let neuron_id = upstream_neuron_ids[idx];
        let burst = inject_and_burst(&npu, neuron_id, 10.0);
        service.notify_burst(burst);
        let commands = wait_for_commands(&service);
        apply_plasticity_commands(&npu, &commands);
    }

    let array_stats = service.get_memory_neuron_array().lock().unwrap().get_stats();
    assert_eq!(array_stats.active_neurons, 3);

    let stats = service.get_stats();
    assert_eq!(stats.memory_neurons_created, 3);
}

#[test]
fn test_memory_replay_injects_twin_area() {
    use feagi_brain_development::models::CorticalAreaExt;
    use feagi_brain_development::ConnectomeManager;
    use feagi_structures::genomic::cortical_area::{
        CorticalArea, CorticalAreaDimensions, CorticalAreaType, CorticalID,
        IOCorticalAreaConfigurationFlag, MemoryCorticalType,
    };

    let npu = build_npu("memory-replay-twin-npu");
    let mut manager = ConnectomeManager::new_for_testing_with_npu(Arc::clone(&npu));
    manager.setup_core_morphologies_for_testing();

    let src_id = CorticalID::try_from_bytes(b"csrc0002").unwrap();
    let mem_id = CorticalID::try_from_bytes(b"mmem0002").unwrap();

    let src_area = CorticalArea::new(
        src_id,
        0,
        "Source Area".to_string(),
        CorticalAreaDimensions::new(2, 2, 1).unwrap(),
        (0, 0, 0).into(),
        CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean),
    )
    .unwrap();
    manager.add_cortical_area(src_area).unwrap();
    manager.create_neurons_for_area(&src_id).unwrap();

    let mut mem_area = CorticalArea::new(
        mem_id,
        0,
        "Memory Area".to_string(),
        CorticalAreaDimensions::new(2, 2, 1).unwrap(),
        (0, 0, 0).into(),
        CorticalAreaType::Memory(MemoryCorticalType::Memory),
    )
    .unwrap();
    mem_area
        .properties
        .insert("is_mem_type".to_string(), serde_json::json!(true));
    mem_area
        .properties
        .insert("temporal_depth".to_string(), serde_json::json!(1));
    manager.add_cortical_area(mem_area).unwrap();

    let mapping_data = vec![serde_json::json!({
        "morphology_id": "memory",
        "morphology_scalar": 1,
        "postSynapticCurrent_multiplier": 1.0,
    })];
    manager
        .update_cortical_mapping(&src_id, &mem_id, mapping_data)
        .unwrap();
    manager
        .regenerate_synapses_for_mapping(&src_id, &mem_id)
        .unwrap();

    let memory_area_idx = manager.get_cortical_idx(&mem_id).unwrap();
    let upstream_idx = manager.get_cortical_idx(&src_id).unwrap();
    let twin_id = manager
        .get_memory_twin_for_upstream_idx(memory_area_idx, upstream_idx)
        .expect("Expected twin area for replay");
    let twin_idx = manager.get_cortical_idx(&twin_id).unwrap();

    let service = PlasticityService::new(
        PlasticityConfig::default(),
        feagi_npu_plasticity::create_memory_stats_cache(),
        npu.clone(),
    );
    service.start();
    service.register_memory_area(memory_area_idx, "mem_00".to_string(), 1, vec![upstream_idx], None);

    let upstream_neuron_id = {
        let npu_lock = npu.lock().unwrap();
        npu_lock.get_neurons_in_cortical_area(upstream_idx)[0]
    };
    let burst = inject_and_burst(&npu, upstream_neuron_id, 10.0);
    service.notify_burst(burst);

    let commands = wait_for_commands(&service);
    let replay_frames = commands
        .iter()
        .find_map(|cmd| match cmd {
            PlasticityCommand::InjectMemoryNeuronToFCL { replay_frames, .. } => {
                Some(replay_frames.clone())
            }
            _ => None,
        })
        .expect("Expected replay frames from memory command");
    assert!(!replay_frames.is_empty(), "Replay frames should not be empty");

    let max_offset = replay_frames
        .iter()
        .map(|frame| frame.offset)
        .max()
        .unwrap_or(0);

    let mut twin_fired = false;
    for offset in 0..=max_offset {
        let coords: Vec<(u32, u32, u32)> = replay_frames
            .iter()
            .filter(|frame| frame.offset == offset)
            .flat_map(|frame| frame.coords.iter().copied())
            .collect();

        let mut npu_lock = npu.lock().unwrap();
        if !coords.is_empty() {
            let potential = manager
                .get_cortical_area(&twin_id)
                .map(|area| area.firing_threshold() + area.firing_threshold_increment())
                .unwrap_or(1.5);
            let xyzp_data: Vec<(u32, u32, u32, f32)> = coords
                .iter()
                .map(|(x, y, z)| (*x, *y, *z, potential))
                .collect();
            npu_lock.inject_sensory_xyzp_by_id(&twin_id, &xyzp_data);
        }

        let result = npu_lock.process_burst().expect("Burst failed");
        if result
            .fired_neurons
            .iter()
            .any(|id| npu_lock.get_neuron_cortical_area(id.0) == twin_idx)
        {
            twin_fired = true;
        }
    }

    assert!(twin_fired, "Expected replay to activate the twin area");
}
