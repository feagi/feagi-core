// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "plasticity")]

use std::sync::{Arc, Mutex};
use std::thread::yield_now;

use feagi_services::plasticity_runtime::PlasticityRuntime;
use feagi_brain_development::models::CorticalAreaExt;
use feagi_brain_development::ConnectomeManager;
use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::{
    BurstLoopRunner, DynamicNPU, MemoryReplayFrame, MotorPublisher, RawFireQueueSnapshot,
    TracingMutex, VisualizationPublisher,
};
use feagi_npu_plasticity::{
    create_memory_stats_cache, AsyncPlasticityExecutor, PlasticityConfig, PlasticityExecutor,
};
use feagi_npu_runtime::StdRuntime;
use feagi_structures::genomic::cortical_area::{
    CorticalArea, CorticalAreaDimensions, CorticalAreaType, CorticalID,
    IOCorticalAreaConfigurationFlag, MemoryCorticalType,
};
use parking_lot::RwLock;

static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Stop the burst loop when the guard drops.
struct BurstRunnerGuard {
    burst_runner: Arc<RwLock<BurstLoopRunner>>,
}

impl Drop for BurstRunnerGuard {
    fn drop(&mut self) {
        self.burst_runner.write().stop();
    }
}

/// Wait until the burst count reaches the target.
fn wait_for_burst_count(
    burst_runner: &Arc<RwLock<BurstLoopRunner>>,
    target: u64,
    max_iters: usize,
) -> bool {
    for _ in 0..max_iters {
        if burst_runner.read().get_burst_count() >= target {
            return true;
        }
        yield_now();
    }
    false
}

/// Check if the twin area fired via FireLedger.
fn wait_for_twin_fire_ledger(
    burst_runner: &Arc<RwLock<BurstLoopRunner>>,
    npu: &Arc<TracingMutex<DynamicNPU>>,
    twin_idx: u32,
    max_iters: usize,
) -> bool {
    for _ in 0..max_iters {
        let burst = burst_runner.read().get_burst_count();
        if burst == 0 {
            yield_now();
            continue;
        }
        if let Ok(npu_lock) = npu.lock() {
            if let Ok(window) = npu_lock.get_fire_ledger_dense_window_bitmaps(twin_idx, burst, 1) {
                if window.iter().any(|(_, bm)| !bm.is_empty()) {
                    return true;
                }
            }
        }
        yield_now();
    }
    false
}

/// Wire the burst loop the way the production binary does: notify the plasticity executor
/// each burst and run the runtime's post-burst step.
fn wire_plasticity_callbacks(
    burst_runner: &Arc<RwLock<BurstLoopRunner>>,
    executor: Arc<Mutex<AsyncPlasticityExecutor>>,
    npu: Arc<TracingMutex<DynamicNPU>>,
    manager: Arc<RwLock<ConnectomeManager>>,
) {
    let executor_for_notify = Arc::clone(&executor);
    burst_runner
        .write()
        .set_plasticity_notify_callback(move |timestep: u64| {
            if let Ok(exec) = executor_for_notify.lock() {
                exec.notify_burst(timestep);
            }
        });
    let runtime = Arc::new(PlasticityRuntime::new(npu, manager, executor));
    burst_runner
        .write()
        .set_post_burst_callback(move |timestep: u64| runtime.apply_post_burst(timestep));
}

struct NoopViz;

impl VisualizationPublisher for NoopViz {
    fn publish_raw_fire_queue_for_agent(
        &self,
        _agent_id: &str,
        _fire_data: RawFireQueueSnapshot,
    ) -> Result<(), String> {
        Ok(())
    }
}

struct NoopMotor;

impl MotorPublisher for NoopMotor {
    fn publish_motor(&self, _agent_id: &str, _data: &[u8]) -> Result<(), String> {
        Ok(())
    }
}

#[test]
fn test_runtime_replay_fires_twin_area() {
    let _test_lock = TEST_LOCK.lock().unwrap();
    let runtime = StdRuntime;
    let backend = CPUBackend::new();
    let npu = Arc::new(TracingMutex::new(
        DynamicNPU::INT8(
            feagi_npu_burst_engine::RustNPU::new(runtime, backend, 100_000, 100_000, 10)
                .expect("Failed to create NPU"),
        ),
        "TestNPU",
    ));

    let manager = ConnectomeManager::instance();
    {
        let mut mgr = manager.write();
        mgr.set_npu(Arc::clone(&npu));
        mgr.setup_core_morphologies_for_testing();
    }

    let src_id = CorticalID::try_from_bytes(b"csrc0101").unwrap();
    let mem_id = CorticalID::try_from_bytes(b"mmem0101").unwrap();
    let src_area = CorticalArea::new(
        src_id,
        0,
        "Replay Source".to_string(),
        CorticalAreaDimensions::new(1, 1, 1).unwrap(),
        (0, 0, 0).into(),
        CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean),
    )
    .unwrap();
    let mut mem_area = CorticalArea::new(
        mem_id,
        0,
        "Replay Memory".to_string(),
        CorticalAreaDimensions::new(1, 1, 1).unwrap(),
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

    {
        let mut mgr = manager.write();
        mgr.add_cortical_area(src_area).unwrap();
        mgr.create_neurons_for_area(&src_id).unwrap();
        mgr.add_cortical_area(mem_area).unwrap();
    }

    let burst_runner = Arc::new(RwLock::new(BurstLoopRunner::new::<NoopViz, NoopMotor>(
        Arc::clone(&npu),
        None,
        None,
        200.0,
    )));
    let _runner_guard = BurstRunnerGuard {
        burst_runner: Arc::clone(&burst_runner),
    };

    let cache = create_memory_stats_cache();
    let executor = Arc::new(Mutex::new(AsyncPlasticityExecutor::new(
        PlasticityConfig::default(),
        cache,
        Arc::clone(&npu),
    )));
    {
        let mut exec = executor.lock().unwrap();
        PlasticityExecutor::start(&mut *exec);
    }
    {
        let mut mgr = manager.write();
        mgr.set_plasticity_executor(Arc::clone(&executor));
    }
    wire_plasticity_callbacks(
        &burst_runner,
        Arc::clone(&executor),
        Arc::clone(&npu),
        Arc::clone(&manager),
    );

    let mapping_data = vec![serde_json::json!({
        "morphology_id": "episodic_memory",
        "morphology_scalar": 1,
        "postSynapticCurrent_multiplier": 1.0,
    })];
    {
        let mut mgr = manager.write();
        mgr.update_cortical_mapping(&src_id, &mem_id, mapping_data)
            .expect("Failed to update memory mapping");
        mgr.regenerate_synapses_for_mapping(&src_id, &mem_id)
            .expect("Failed to regenerate memory mapping");
    }

    let (memory_idx, upstream_idx, twin_id, twin_idx) = {
        let mgr = manager.read();
        let memory_idx = mgr.get_cortical_idx(&mem_id).unwrap();
        let upstream_idx = mgr.get_cortical_idx(&src_id).unwrap();
        let twin_id = mgr
            .get_memory_twin_for_upstream_idx(memory_idx, upstream_idx)
            .expect("Expected twin area");
        let twin_idx = mgr.get_cortical_idx(&twin_id).unwrap();
        (memory_idx, upstream_idx, twin_id, twin_idx)
    };
    {
        let mut mgr = manager.write();
        mgr.create_neurons_for_area(&twin_id)
            .expect("Failed to create neurons for twin area");
    }
    {
        let mut npu_lock = npu.lock().unwrap();
        npu_lock
            .configure_fire_ledger_window(twin_idx, 2)
            .expect("Failed to configure fire ledger window for twin area");
    }
    assert_ne!(memory_idx, upstream_idx);
    assert_ne!(twin_idx, upstream_idx);
    assert_ne!(twin_idx, memory_idx);
    let _ = twin_id;

    burst_runner
        .write()
        .start()
        .expect("Failed to start burst loop");
    assert!(
        wait_for_burst_count(&burst_runner, 1, 200_000),
        "Burst loop did not advance"
    );

    let source_potential = {
        let mgr = manager.read();
        let area = mgr.get_cortical_area(&src_id).unwrap();
        area.firing_threshold() + area.firing_threshold_increment()
    };
    const MEMORY_NEURON_ID_START: u32 = 50_000_000;
    {
        let mut npu_lock = npu.lock().unwrap();
        npu_lock.register_dynamic_neuron_mapping(MEMORY_NEURON_ID_START, mem_id);
        npu_lock.register_memory_replay_frames(
            MEMORY_NEURON_ID_START,
            vec![MemoryReplayFrame {
                offset: 0,
                upstream_area_idx: upstream_idx,
                coords: vec![(0, 0, 0)],
                membrane_potentials: None,
            }],
        );
        npu_lock.register_memory_twin_mapping(memory_idx, upstream_idx, twin_idx, source_potential);
    }
    let mut burst_target = burst_runner.read().get_burst_count();
    for _ in 0..3 {
        burst_target += 1;
        assert!(
            wait_for_burst_count(&burst_runner, burst_target, 200_000),
            "Burst loop stalled before replay stimulus"
        );
        let mut npu_lock = npu.lock().unwrap();
        npu_lock.inject_memory_neuron_to_fcl(MEMORY_NEURON_ID_START, memory_idx, source_potential);
    }

    assert!(
        wait_for_twin_fire_ledger(&burst_runner, &npu, twin_idx, 500_000),
        "Expected replay to drive twin-area firing via runtime scheduler"
    );
}

#[test]
fn test_post_burst_callback_does_not_stall_on_memory_replay() {
    let _test_lock = TEST_LOCK.lock().unwrap();
    let runtime = StdRuntime;
    let backend = CPUBackend::new();
    let npu = Arc::new(TracingMutex::new(
        DynamicNPU::INT8(
            feagi_npu_burst_engine::RustNPU::new(runtime, backend, 100_000, 100_000, 10)
                .expect("Failed to create NPU"),
        ),
        "TestNPU",
    ));

    let manager = ConnectomeManager::instance();
    {
        let mut mgr = manager.write();
        mgr.set_npu(Arc::clone(&npu));
        mgr.setup_core_morphologies_for_testing();
    }

    let a1_id = CorticalID::try_from_bytes(b"csrc0201").unwrap();
    let a2_id = CorticalID::try_from_bytes(b"csrc0202").unwrap();
    let m1_id = CorticalID::try_from_bytes(b"mmem0201").unwrap();
    let m2_id = CorticalID::try_from_bytes(b"mmem0202").unwrap();

    let a1_area = CorticalArea::new(
        a1_id,
        0,
        "Upstream A1".to_string(),
        CorticalAreaDimensions::new(1, 1, 1).unwrap(),
        (0, 0, 0).into(),
        CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean),
    )
    .unwrap();
    let a2_area = CorticalArea::new(
        a2_id,
        0,
        "Upstream A2".to_string(),
        CorticalAreaDimensions::new(1, 1, 1).unwrap(),
        (0, 0, 0).into(),
        CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean),
    )
    .unwrap();
    let mut m1_area = CorticalArea::new(
        m1_id,
        0,
        "Memory M1".to_string(),
        CorticalAreaDimensions::new(1, 1, 1).unwrap(),
        (0, 0, 0).into(),
        CorticalAreaType::Memory(MemoryCorticalType::Memory),
    )
    .unwrap();
    m1_area
        .properties
        .insert("is_mem_type".to_string(), serde_json::json!(true));
    m1_area
        .properties
        .insert("temporal_depth".to_string(), serde_json::json!(1));
    let mut m2_area = CorticalArea::new(
        m2_id,
        0,
        "Memory M2".to_string(),
        CorticalAreaDimensions::new(1, 1, 1).unwrap(),
        (0, 0, 0).into(),
        CorticalAreaType::Memory(MemoryCorticalType::Memory),
    )
    .unwrap();
    m2_area
        .properties
        .insert("is_mem_type".to_string(), serde_json::json!(true));
    m2_area
        .properties
        .insert("temporal_depth".to_string(), serde_json::json!(1));

    {
        let mut mgr = manager.write();
        mgr.add_cortical_area(a1_area).unwrap();
        mgr.add_cortical_area(a2_area).unwrap();
        mgr.create_neurons_for_area(&a1_id).unwrap();
        mgr.create_neurons_for_area(&a2_id).unwrap();
        mgr.add_cortical_area(m1_area).unwrap();
        mgr.add_cortical_area(m2_area).unwrap();
    }

    let burst_runner = Arc::new(RwLock::new(BurstLoopRunner::new::<NoopViz, NoopMotor>(
        Arc::clone(&npu),
        None,
        None,
        200.0,
    )));
    let _runner_guard = BurstRunnerGuard {
        burst_runner: Arc::clone(&burst_runner),
    };

    let cache = create_memory_stats_cache();
    let executor = Arc::new(Mutex::new(AsyncPlasticityExecutor::new(
        PlasticityConfig::default(),
        cache,
        Arc::clone(&npu),
    )));
    {
        let mut exec = executor.lock().unwrap();
        PlasticityExecutor::start(&mut *exec);
    }
    {
        let mut mgr = manager.write();
        mgr.set_plasticity_executor(Arc::clone(&executor));
    }
    wire_plasticity_callbacks(
        &burst_runner,
        Arc::clone(&executor),
        Arc::clone(&npu),
        Arc::clone(&manager),
    );

    let episodic_mapping = vec![serde_json::json!({
        "morphology_id": "episodic_memory",
        "morphology_scalar": 1,
        "postSynapticCurrent_multiplier": 1.0,
    })];
    let associative_mapping = vec![serde_json::json!({
        "morphology_id": "associative_memory",
        "morphology_scalar": 1,
        "postSynapticCurrent_multiplier": 1.0,
        "plasticity_flag": true,
        "plasticity_constant": 1,
        "ltp_multiplier": 1,
        "ltd_multiplier": 1,
        "plasticity_window": 5,
    })];
    {
        let mut mgr = manager.write();
        mgr.update_cortical_mapping(&a1_id, &m1_id, episodic_mapping.clone())
            .expect("Failed to map a1->m1");
        mgr.update_cortical_mapping(&a2_id, &m2_id, episodic_mapping)
            .expect("Failed to map a2->m2");
        mgr.update_cortical_mapping(&m1_id, &m2_id, associative_mapping)
            .expect("Failed to map m1->m2 associative");
        mgr.regenerate_synapses_for_mapping(&a1_id, &m1_id)
            .expect("Failed to regen a1->m1");
        mgr.regenerate_synapses_for_mapping(&a2_id, &m2_id)
            .expect("Failed to regen a2->m2");
        mgr.regenerate_synapses_for_mapping(&m1_id, &m2_id)
            .expect("Failed to regen m1->m2");
    }

    burst_runner
        .write()
        .start()
        .expect("Failed to start burst loop");
    assert!(
        wait_for_burst_count(&burst_runner, 1, 200_000),
        "Burst loop did not advance"
    );

    let source_potential = {
        let mgr = manager.read();
        let area = mgr.get_cortical_area(&a1_id).unwrap();
        area.firing_threshold() + area.firing_threshold_increment()
    };

    let mut last_burst = burst_runner.read().get_burst_count();
    for _ in 0..10 {
        let mut npu_lock = npu.lock().unwrap();
        npu_lock.inject_sensory_xyzp_by_id(&a1_id, &[(0, 0, 0, source_potential)]);
        npu_lock.inject_sensory_xyzp_by_id(&a2_id, &[(0, 0, 0, source_potential)]);
        drop(npu_lock);

        let target = last_burst + 2;
        assert!(
            wait_for_burst_count(&burst_runner, target, 200_000),
            "Burst loop stalled after memory replay injections"
        );
        last_burst = burst_runner.read().get_burst_count();
    }
}

#[test]
fn test_post_burst_callback_under_connectome_contention() {
    let _test_lock = TEST_LOCK.lock().unwrap();
    let runtime = StdRuntime;
    let backend = CPUBackend::new();
    let npu = Arc::new(TracingMutex::new(
        DynamicNPU::INT8(
            feagi_npu_burst_engine::RustNPU::new(runtime, backend, 100_000, 100_000, 10)
                .expect("Failed to create NPU"),
        ),
        "TestNPU",
    ));

    let manager = ConnectomeManager::instance();
    {
        let mut mgr = manager.write();
        mgr.set_npu(Arc::clone(&npu));
        mgr.setup_core_morphologies_for_testing();
    }

    let a1_id = CorticalID::try_from_bytes(b"csrc0301").unwrap();
    let a2_id = CorticalID::try_from_bytes(b"csrc0302").unwrap();
    let m1_id = CorticalID::try_from_bytes(b"mmem0301").unwrap();
    let m2_id = CorticalID::try_from_bytes(b"mmem0302").unwrap();

    let a1_area = CorticalArea::new(
        a1_id,
        0,
        "Upstream A1".to_string(),
        CorticalAreaDimensions::new(1, 1, 1).unwrap(),
        (0, 0, 0).into(),
        CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean),
    )
    .unwrap();
    let a2_area = CorticalArea::new(
        a2_id,
        0,
        "Upstream A2".to_string(),
        CorticalAreaDimensions::new(1, 1, 1).unwrap(),
        (0, 0, 0).into(),
        CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean),
    )
    .unwrap();
    let mut m1_area = CorticalArea::new(
        m1_id,
        0,
        "Memory M1".to_string(),
        CorticalAreaDimensions::new(1, 1, 1).unwrap(),
        (0, 0, 0).into(),
        CorticalAreaType::Memory(MemoryCorticalType::Memory),
    )
    .unwrap();
    m1_area
        .properties
        .insert("is_mem_type".to_string(), serde_json::json!(true));
    m1_area
        .properties
        .insert("temporal_depth".to_string(), serde_json::json!(1));
    let mut m2_area = CorticalArea::new(
        m2_id,
        0,
        "Memory M2".to_string(),
        CorticalAreaDimensions::new(1, 1, 1).unwrap(),
        (0, 0, 0).into(),
        CorticalAreaType::Memory(MemoryCorticalType::Memory),
    )
    .unwrap();
    m2_area
        .properties
        .insert("is_mem_type".to_string(), serde_json::json!(true));
    m2_area
        .properties
        .insert("temporal_depth".to_string(), serde_json::json!(1));

    {
        let mut mgr = manager.write();
        mgr.add_cortical_area(a1_area).unwrap();
        mgr.add_cortical_area(a2_area).unwrap();
        mgr.create_neurons_for_area(&a1_id).unwrap();
        mgr.create_neurons_for_area(&a2_id).unwrap();
        mgr.add_cortical_area(m1_area).unwrap();
        mgr.add_cortical_area(m2_area).unwrap();
    }

    let burst_runner = Arc::new(RwLock::new(BurstLoopRunner::new::<NoopViz, NoopMotor>(
        Arc::clone(&npu),
        None,
        None,
        200.0,
    )));
    let _runner_guard = BurstRunnerGuard {
        burst_runner: Arc::clone(&burst_runner),
    };

    let cache = create_memory_stats_cache();
    let executor = Arc::new(Mutex::new(AsyncPlasticityExecutor::new(
        PlasticityConfig::default(),
        cache,
        Arc::clone(&npu),
    )));
    {
        let mut exec = executor.lock().unwrap();
        PlasticityExecutor::start(&mut *exec);
    }
    {
        let mut mgr = manager.write();
        mgr.set_plasticity_executor(Arc::clone(&executor));
    }
    wire_plasticity_callbacks(
        &burst_runner,
        Arc::clone(&executor),
        Arc::clone(&npu),
        Arc::clone(&manager),
    );

    let episodic_mapping = vec![serde_json::json!({
        "morphology_id": "episodic_memory",
        "morphology_scalar": 1,
        "postSynapticCurrent_multiplier": 1.0,
    })];
    let associative_mapping = vec![serde_json::json!({
        "morphology_id": "associative_memory",
        "morphology_scalar": 1,
        "postSynapticCurrent_multiplier": 1.0,
        "plasticity_flag": true,
        "plasticity_constant": 1,
        "ltp_multiplier": 1,
        "ltd_multiplier": 1,
        "plasticity_window": 5,
    })];
    {
        let mut mgr = manager.write();
        mgr.update_cortical_mapping(&a1_id, &m1_id, episodic_mapping.clone())
            .expect("Failed to map a1->m1");
        mgr.update_cortical_mapping(&a2_id, &m2_id, episodic_mapping)
            .expect("Failed to map a2->m2");
        mgr.update_cortical_mapping(&m1_id, &m2_id, associative_mapping)
            .expect("Failed to map m1->m2 associative");
        mgr.regenerate_synapses_for_mapping(&a1_id, &m1_id)
            .expect("Failed to regen a1->m1");
        mgr.regenerate_synapses_for_mapping(&a2_id, &m2_id)
            .expect("Failed to regen a2->m2");
        mgr.regenerate_synapses_for_mapping(&m1_id, &m2_id)
            .expect("Failed to regen m1->m2");
    }

    burst_runner
        .write()
        .start()
        .expect("Failed to start burst loop");
    assert!(
        wait_for_burst_count(&burst_runner, 1, 200_000),
        "Burst loop did not advance"
    );

    let keep_running = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let keep_running_thread = Arc::clone(&keep_running);
    let contention_thread = std::thread::spawn(move || {
        while keep_running_thread.load(std::sync::atomic::Ordering::Relaxed) {
            let manager = ConnectomeManager::instance();
            let _lock = manager.write();
            for _ in 0..100 {
                yield_now();
            }
        }
    });

    let source_potential = {
        let mgr = manager.read();
        let area = mgr.get_cortical_area(&a1_id).unwrap();
        area.firing_threshold() + area.firing_threshold_increment()
    };

    let mut last_burst = burst_runner.read().get_burst_count();
    for _ in 0..10 {
        let mut npu_lock = npu.lock().unwrap();
        npu_lock.inject_sensory_xyzp_by_id(&a1_id, &[(0, 0, 0, source_potential)]);
        npu_lock.inject_sensory_xyzp_by_id(&a2_id, &[(0, 0, 0, source_potential)]);
        drop(npu_lock);

        let target = last_burst + 2;
        assert!(
            wait_for_burst_count(&burst_runner, target, 200_000),
            "Burst loop stalled under connectome contention"
        );
        last_burst = burst_runner.read().get_burst_count();
    }

    keep_running.store(false, std::sync::atomic::Ordering::Relaxed);
    let _ = contention_thread.join();
}

#[test]
fn test_post_burst_memory_conversion_command_does_not_deadlock() {
    let _test_lock = TEST_LOCK.lock().unwrap();
    let runtime = StdRuntime;
    let backend = CPUBackend::new();
    let npu = Arc::new(TracingMutex::new(
        DynamicNPU::INT8(
            feagi_npu_burst_engine::RustNPU::new(runtime, backend, 100_000, 100_000, 10)
                .expect("Failed to create NPU"),
        ),
        "TestNPU",
    ));

    let manager = ConnectomeManager::instance();
    {
        let mut mgr = manager.write();
        mgr.set_npu(Arc::clone(&npu));
        mgr.setup_core_morphologies_for_testing();
    }

    let m1_id = CorticalID::try_from_bytes(b"mmem0401").unwrap();
    let m1_area = CorticalArea::new(
        m1_id,
        0,
        "Memory M1".to_string(),
        CorticalAreaDimensions::new(1, 1, 1).unwrap(),
        (0, 0, 0).into(),
        CorticalAreaType::Memory(MemoryCorticalType::Memory),
    )
    .unwrap();
    let mut m1_area = m1_area;
    m1_area
        .properties
        .insert("is_mem_type".to_string(), serde_json::json!(true));
    m1_area
        .properties
        .insert("temporal_depth".to_string(), serde_json::json!(1));

    {
        let mut mgr = manager.write();
        mgr.add_cortical_area(m1_area).unwrap();
    }

    let burst_runner = Arc::new(RwLock::new(BurstLoopRunner::new::<NoopViz, NoopMotor>(
        Arc::clone(&npu),
        None,
        None,
        200.0,
    )));
    let _runner_guard = BurstRunnerGuard {
        burst_runner: Arc::clone(&burst_runner),
    };

    let cache = create_memory_stats_cache();
    let executor = Arc::new(Mutex::new(AsyncPlasticityExecutor::new(
        PlasticityConfig::default(),
        cache,
        Arc::clone(&npu),
    )));
    {
        let mut exec = executor.lock().unwrap();
        PlasticityExecutor::start(&mut *exec);
    }
    {
        let mut mgr = manager.write();
        mgr.set_plasticity_executor(Arc::clone(&executor));
    }
    wire_plasticity_callbacks(
        &burst_runner,
        Arc::clone(&executor),
        Arc::clone(&npu),
        Arc::clone(&manager),
    );

    burst_runner
        .write()
        .start()
        .expect("Failed to start burst loop");
    assert!(
        wait_for_burst_count(&burst_runner, 1, 200_000),
        "Burst loop did not advance"
    );

    let memory_idx = {
        let mgr = manager.read();
        mgr.get_cortical_idx(&m1_id).expect("Missing memory idx")
    };
    let command = feagi_npu_plasticity::PlasticityCommand::MemoryNeuronConvertedToLtm {
        neuron_id: 50_000_001,
        area_idx: memory_idx,
        pattern_hash: 1,
    };
    {
        let exec = executor.lock().unwrap();
        exec.enqueue_commands_for_test(vec![command]);
    }

    let burst_before = burst_runner.read().get_burst_count();
    assert!(
        wait_for_burst_count(&burst_runner, burst_before + 2, 200_000),
        "Burst loop stalled while processing MemoryNeuronConvertedToLtm"
    );
}

#[test]
fn test_post_burst_command_budget_defers_large_ltm_batches() {
    let _test_lock = TEST_LOCK.lock().unwrap();
    let runtime = StdRuntime;
    let backend = CPUBackend::new();
    let npu = Arc::new(TracingMutex::new(
        DynamicNPU::INT8(
            feagi_npu_burst_engine::RustNPU::new(runtime, backend, 100_000, 100_000, 10)
                .expect("Failed to create NPU"),
        ),
        "TestNPU",
    ));

    let manager = ConnectomeManager::instance();
    {
        let mut mgr = manager.write();
        mgr.set_npu(Arc::clone(&npu));
        mgr.setup_core_morphologies_for_testing();
    }

    let m1_id = CorticalID::try_from_bytes(b"mmem0402").unwrap();
    let m1_area = CorticalArea::new(
        m1_id,
        0,
        "Memory M1".to_string(),
        CorticalAreaDimensions::new(1, 1, 1).unwrap(),
        (0, 0, 0).into(),
        CorticalAreaType::Memory(MemoryCorticalType::Memory),
    )
    .unwrap();
    let mut m1_area = m1_area;
    m1_area
        .properties
        .insert("is_mem_type".to_string(), serde_json::json!(true));
    m1_area
        .properties
        .insert("temporal_depth".to_string(), serde_json::json!(1));

    {
        let mut mgr = manager.write();
        mgr.add_cortical_area(m1_area).unwrap();
    }

    let burst_runner = Arc::new(RwLock::new(BurstLoopRunner::new::<NoopViz, NoopMotor>(
        Arc::clone(&npu),
        None,
        None,
        200.0,
    )));
    let _runner_guard = BurstRunnerGuard {
        burst_runner: Arc::clone(&burst_runner),
    };

    let cache = create_memory_stats_cache();
    let executor = Arc::new(Mutex::new(AsyncPlasticityExecutor::new(
        PlasticityConfig::default(),
        cache,
        Arc::clone(&npu),
    )));
    {
        let mut exec = executor.lock().unwrap();
        PlasticityExecutor::start(&mut *exec);
    }
    {
        let mut mgr = manager.write();
        mgr.set_plasticity_executor(Arc::clone(&executor));
    }
    wire_plasticity_callbacks(
        &burst_runner,
        Arc::clone(&executor),
        Arc::clone(&npu),
        Arc::clone(&manager),
    );

    burst_runner
        .write()
        .start()
        .expect("Failed to start burst loop");
    assert!(
        wait_for_burst_count(&burst_runner, 1, 200_000),
        "Burst loop did not advance"
    );

    let memory_idx = {
        let mgr = manager.read();
        mgr.get_cortical_idx(&m1_id).expect("Missing memory idx")
    };

    let burst_before = burst_runner.read().get_burst_count();
    let conversion_commands: Vec<feagi_npu_plasticity::PlasticityCommand> = (0..250_u32)
        .map(
            |offset| feagi_npu_plasticity::PlasticityCommand::MemoryNeuronConvertedToLtm {
                neuron_id: 51_000_000 + offset,
                area_idx: memory_idx,
                pattern_hash: 100 + offset as u64,
            },
        )
        .collect();
    {
        let exec = executor.lock().unwrap();
        exec.enqueue_commands_for_test(conversion_commands);
    }

    assert!(
        wait_for_burst_count(&burst_runner, burst_before + 1, 200_000),
        "Burst loop did not process first throttled callback cycle"
    );

    let pending_after_first_cycle = {
        let exec = executor.lock().unwrap();
        let service = exec
            .get_service()
            .expect("Plasticity service should be available");
        service.pending_command_count()
    };
    assert!(
        pending_after_first_cycle > 0,
        "Expected deferred command backlog after first callback cycle"
    );

    assert!(
        wait_for_burst_count(&burst_runner, burst_before + 4, 200_000),
        "Burst loop stalled while draining throttled LTM command backlog"
    );
}
