// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! `memory.capture` recording: states, episode links, boundary pieces, boost, protection.
//!
//! Same rig style as `memory_regression_golden.rs`: a real `DynamicNPU` built through a real
//! `ConnectomeManager` and a real `PlasticityService`, driven burst by burst on one thread
//! (stimulus, `process_burst`, `PlasticityService::run_burst_sync`, drain, the production runtime's post-burst step). One memory
//! area is fed by an upstream area where neuron `i` firing alone is "state i"; a driver area
//! with one neuron, held on, makes the `memory.capture` modulator active.

use ahash::AHashMap;
use feagi_brain_development::ConnectomeManager;
use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::modulator_engine::ModulatorBinding;
use feagi_npu_burst_engine::{
    DynamicNPU, TracingMutex, FIRE_KIND_CHAIN_RECALL, FIRE_KIND_EPISODIC_MEMORY,
    FIRE_KIND_STDP_ELIGIBLE,
};
use feagi_npu_neural::synapse::{SYNAPSE_EDGE_ASSOCIATIVE_MEMORY, SYNAPSE_EDGE_SEQUENTIAL_MEMORY};
use feagi_npu_neural::types::NeuronId;
use feagi_npu_plasticity::{
    AsyncPlasticityExecutor, LtmEviction, MemoryMpMode, MemoryNeuronLifecycleConfig,
    MpChangeEncoding, PlasticityCommand, PlasticityConfig, PlasticityService,
};
use feagi_npu_runtime::StdRuntime;
use feagi_services::plasticity_runtime::PlasticityRuntime;
use feagi_structures::genomic::cortical_area::{
    CorticalArea, CorticalAreaDimensions, CorticalAreaType, CorticalID,
    IOCorticalAreaConfigurationFlag, MemoryCorticalType,
};
use feagi_structures::genomic::ModulatorKind;
use parking_lot::RwLock;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

type SharedNpu = Arc<TracingMutex<DynamicNPU>>;

const SEQ: u8 = SYNAPSE_EDGE_SEQUENTIAL_MEMORY;

fn life(initial: u32, growth: f32, longterm: u32) -> MemoryNeuronLifecycleConfig {
    MemoryNeuronLifecycleConfig {
        initial_lifespan: initial,
        lifespan_growth_rate: growth,
        longterm_threshold: longterm,
        max_reactivations: 1000,
        memory_neuron_cap: 0,
        ltm_eviction: LtmEviction::Lru,
    }
}

/// The area properties production reads the lifecycle and MP mode from
/// (`memory_lifecycle_config` and `memory_mp_mode` in feagi-brain-development).
fn lifecycle_properties(
    life: &MemoryNeuronLifecycleConfig,
    mp: &MemoryMpMode,
) -> Vec<(&'static str, Value)> {
    let mut props = vec![
        ("init_lifespan", json!(life.initial_lifespan)),
        ("lifespan_growth_rate", json!(life.lifespan_growth_rate)),
        ("longterm_mem_threshold", json!(life.longterm_threshold)),
        ("memory_neuron_cap", json!(life.memory_neuron_cap)),
        (
            "ltm_eviction",
            json!(match life.ltm_eviction {
                LtmEviction::Lru => "lru",
                LtmEviction::None => "none",
            }),
        ),
    ];
    match mp {
        MemoryMpMode::PatternOnly => {}
        MemoryMpMode::MpLearning => props.push(("mp_learning_enabled", json!(true))),
        MemoryMpMode::Change(MpChangeEncoding::Differential { quantization }) => {
            props.push(("mp_change_mode", json!("mp_differential")));
            props.push(("mp_delta_quantization", json!(quantization)));
        }
        MemoryMpMode::Change(MpChangeEncoding::Ratio {
            quantization_percent,
        }) => {
            props.push(("mp_change_mode", json!("mp_ratio")));
            props.push(("mp_ratio_quantization", json!(quantization_percent)));
        }
    }
    props
}

fn regular_area(
    manager: &mut ConnectomeManager,
    id: &[u8; 8],
    name: &str,
    neurons: u32,
) -> (CorticalID, u32) {
    let cortical_id = CorticalID::try_from_bytes(id).unwrap();
    let area = CorticalArea::new(
        cortical_id,
        0,
        name.to_string(),
        CorticalAreaDimensions::new(neurons, 1, 1).unwrap(),
        (0, 0, 0).into(),
        CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean),
    )
    .unwrap();
    manager.add_cortical_area(area).unwrap();
    manager.create_neurons_for_area(&cortical_id).unwrap();
    let idx = manager.get_cortical_idx(&cortical_id).unwrap();
    (cortical_id, idx)
}

/// One captured link as the NPU reports it.
#[derive(Debug, Clone, PartialEq)]
struct Link {
    source: u32,
    target: u32,
    weight: f32,
    psp: f32,
    group: u16,
}

struct Rig {
    npu: SharedNpu,
    manager: Arc<RwLock<ConnectomeManager>>,
    service: PlasticityService,
    runtime: PlasticityRuntime,
    mem_idx: u32,
    mem_id: CorticalID,
    up: Vec<u32>,
    /// Neuron of the driver area that turns `memory.capture` on while it is stimulated.
    capture_drive: (u32, f32),
    /// Neuron of the second driver area (formation / freeze / gate).
    aux_drive: (u32, f32),
    capture_idx: u32,
    aux_idx: u32,
    capture_on: bool,
    aux_on: bool,
    burst: u64,
    /// Encoded neuron ids, one entry per burst that encoded (None for a silent burst).
    encoded: Vec<Option<u32>>,
    /// Memory neurons that fired, per burst.
    fired_memory: Vec<Vec<u32>>,
    /// Sorted ids of every area that fired, per burst (for equality checks).
    fired_all: Vec<Vec<u32>>,
    /// `(memory neurons, synapses)` after every burst.
    counts: Vec<(usize, usize)>,
    /// The twin of the upstream area: where replayed frames land.
    twin_idx: u32,
    /// Twin neurons that fired, per burst.
    fired_twin: Vec<usize>,
    /// Replay frames each encoded state stores, `(offset, coordinates)`.
    frames: AHashMap<u32, Vec<(u32, usize)>>,
    /// `(burst, episode neuron id)` of every episode inject the service queued.
    episode_injects: Vec<(u64, u32)>,
}

impl Rig {
    /// `states` is the number of upstream neurons: state `i` is neuron `i` firing alone.
    fn new(
        states: u32,
        lifecycle: MemoryNeuronLifecycleConfig,
        mem_props: Vec<(&str, Value)>,
    ) -> Rig {
        let npu: SharedNpu = Arc::new(TracingMutex::new(
            DynamicNPU::new_f32(StdRuntime::new(), CPUBackend::new(), 4096, 8192, 8)
                .expect("npu creation"),
            "capture-rig",
        ));
        let mut manager = ConnectomeManager::new_for_testing_with_npu(Arc::clone(&npu));
        manager.setup_core_morphologies_for_testing();
        let executor = Arc::new(Mutex::new(AsyncPlasticityExecutor::new(
            PlasticityConfig::default(),
            feagi_npu_plasticity::create_memory_stats_cache(),
            Arc::clone(&npu),
        )));
        let service = executor
            .lock()
            .unwrap()
            .get_service()
            .expect("plasticity service");
        manager.set_plasticity_executor(Arc::clone(&executor));
        let (up_id, up_idx) = regular_area(&mut manager, b"csrc0001", "up", states);

        let mem_id = CorticalID::try_from_bytes(b"mmem0001").unwrap();
        let mut mem_area = CorticalArea::new(
            mem_id,
            0,
            "mem".to_string(),
            CorticalAreaDimensions::new(2, 2, 1).unwrap(),
            (0, 0, 0).into(),
            CorticalAreaType::Memory(MemoryCorticalType::Memory),
        )
        .unwrap();
        mem_area
            .properties
            .insert("is_mem_type".to_string(), json!(true));
        mem_area
            .properties
            .insert("temporal_depth".to_string(), json!(1));
        for (key, value) in lifecycle_properties(&lifecycle, &MemoryMpMode::PatternOnly) {
            mem_area.properties.insert(key.to_string(), value);
        }
        for (key, value) in &mem_props {
            mem_area
                .properties
                .insert((*key).to_string(), value.clone());
        }
        manager.add_cortical_area(mem_area).unwrap();
        manager
            .update_cortical_mapping(
                &up_id,
                &mem_id,
                vec![json!({
                    "morphology_id": "episodic_memory",
                    "morphology_scalar": 1,
                    "postSynapticCurrent_multiplier": 1.0,
                })],
            )
            .unwrap();
        manager
            .regenerate_synapses_for_mapping(&up_id, &mem_id)
            .unwrap();
        let mem_idx = manager.get_cortical_idx(&mem_id).unwrap();
        let twin_idx = manager
            .get_memory_twin_for_upstream_idx(mem_idx, up_idx)
            .and_then(|twin| manager.get_cortical_idx(&twin))
            .expect("the memory area has a twin of its upstream area");

        // Registered by the manager through the linked executor, from the area's properties,
        // as in production (the mapping above is the registration trigger).
        assert!(service.registered_memory_area_indexes().contains(&mem_idx));
        // The area's own properties (capture_max_bursts) reach the NPU through the same sync a
        // genome load and an area update use.
        manager.sync_memory_associative_lif_params(&mem_id);

        let (_, capture_idx) = regular_area(&mut manager, b"cdrv0001", "capdrv", 1);
        let (_, aux_idx) = regular_area(&mut manager, b"cdrv0002", "auxdrv", 1);
        let up = npu.lock().unwrap().get_neurons_in_cortical_area(up_idx);
        let capture_neuron = npu
            .lock()
            .unwrap()
            .get_neurons_in_cortical_area(capture_idx)[0];
        let aux_neuron = npu.lock().unwrap().get_neurons_in_cortical_area(aux_idx)[0];
        let manager = Arc::new(RwLock::new(manager));
        let runtime = PlasticityRuntime::new(Arc::clone(&npu), Arc::clone(&manager), executor);
        Rig {
            npu,
            manager,
            service,
            runtime,
            mem_idx,
            mem_id,
            up,
            capture_drive: (capture_neuron, 10.0),
            aux_drive: (aux_neuron, 10.0),
            capture_idx,
            aux_idx,
            capture_on: false,
            aux_on: false,
            burst: 0,
            encoded: Vec::new(),
            fired_memory: Vec::new(),
            fired_all: Vec::new(),
            counts: Vec::new(),
            twin_idx,
            fired_twin: Vec::new(),
            frames: AHashMap::new(),
            episode_injects: Vec::new(),
        }
    }

    /// True for an active episode neuron of the memory area.
    fn is_episode(&self, id: u32) -> bool {
        self.service
            .get_memory_neuron_array()
            .lock()
            .unwrap()
            .episode_members_of(id)
            .is_some()
    }

    /// Episode neuron ids, sorted.
    fn episodes(&self) -> Vec<u32> {
        let array = self.service.get_memory_neuron_array();
        let array = array.lock().unwrap();
        array
            .episode_lists()
            .into_iter()
            .filter(|(area, ..)| *area == self.mem_idx)
            .map(|(_, id, _)| id)
            .collect()
    }

    fn binding(
        &self,
        id: &str,
        kind: ModulatorKind,
        magnitude: f32,
        driver_idx: u32,
    ) -> ModulatorBinding {
        ModulatorBinding {
            instance_id: id.to_string(),
            kind,
            magnitude_percent: magnitude,
            graded: false,
            full_scale_potential: 0.0,
            driver_cortical_idx: driver_idx,
        }
    }

    /// Bind modulators and subscribe the memory area to the area-level ones.
    fn bind(&self, bindings: Vec<ModulatorBinding>, area_level: &[&str]) {
        let guard = self.npu.lock().unwrap();
        guard.set_modulator_bindings(bindings);
        let mut subs = AHashMap::new();
        subs.insert(
            self.mem_idx,
            area_level.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        );
        guard.set_modulator_area_subscriptions(subs);
    }

    /// Subscribe the area to a `memory.capture` instance of `magnitude` driven by the capture
    /// driver area.
    fn bind_capture(&self, magnitude: f32) {
        self.bind(
            vec![self.binding(
                "cap",
                ModulatorKind::MemoryCapture,
                magnitude,
                self.capture_idx,
            )],
            &["cap"],
        );
    }

    /// Add the `sequential_memory` loop to the area (optionally with modulator ids).
    fn add_loop(&mut self, modulators: &[&str]) {
        let mem_id = self.mem_id;
        let mut rule = json!({
            "morphology_id": "sequential_memory",
            "morphology_scalar": 1,
            "postSynapticCurrent_multiplier": 1.0,
        });
        if !modulators.is_empty() {
            rule["modulators"] = json!(modulators);
        }
        self.set_self_mapping(vec![rule]);
        let _ = mem_id;
    }

    fn set_self_mapping(&mut self, rules: Vec<Value>) {
        let mem_id = self.mem_id;
        self.manager
            .write()
            .update_cortical_mapping(&mem_id, &mem_id, rules)
            .unwrap();
        self.manager
            .write()
            .regenerate_synapses_for_mapping(&mem_id, &mem_id)
            .unwrap();
    }

    /// One burst: optional upstream state `state`, drivers as currently held.
    fn step(&mut self, state: Option<usize>) {
        let result = {
            let mut guard = self.npu.lock().unwrap();
            let mut stim: Vec<(NeuronId, f32)> = Vec::new();
            if let Some(i) = state {
                stim.push((NeuronId(self.up[i]), 10.0));
            }
            if self.capture_on {
                stim.push((NeuronId(self.capture_drive.0), self.capture_drive.1));
            }
            if self.aux_on {
                stim.push((NeuronId(self.aux_drive.0), self.aux_drive.1));
            }
            if !stim.is_empty() {
                guard.inject_sensory_with_potentials(&stim);
            }
            guard.process_burst().expect("burst")
        };
        self.service.run_burst_sync(result.burst);
        // Production order: the post-burst callback drains and applies the queue after the NPU
        // lock is released. The rig reads the queue first, then puts it back for the runtime.
        let commands = self.service.drain_commands();
        self.service.enqueue_commands_for_test(commands.clone());
        self.runtime.apply_post_burst(result.burst);
        // An episode neuron's own inject is not an encoded state.
        let episode_ids: Vec<u32> = commands
            .iter()
            .filter_map(|c| match c {
                PlasticityCommand::InjectMemoryNeuronToFCL { neuron_id, .. }
                    if self.is_episode(*neuron_id) =>
                {
                    Some(*neuron_id)
                }
                _ => None,
            })
            .collect();
        for id in &episode_ids {
            self.episode_injects.push((result.burst, *id));
        }
        for command in &commands {
            if let PlasticityCommand::InjectMemoryNeuronToFCL {
                neuron_id,
                replay_frames,
                ..
            } = command
            {
                if episode_ids.contains(neuron_id) {
                    continue;
                }
                self.frames.insert(
                    *neuron_id,
                    replay_frames
                        .iter()
                        .map(|f| (f.offset, f.coords.len()))
                        .collect(),
                );
            }
        }
        let encoded = commands.iter().find_map(|c| match c {
            PlasticityCommand::InjectMemoryNeuronToFCL { neuron_id, .. }
                if !episode_ids.contains(neuron_id) =>
            {
                Some(*neuron_id)
            }
            _ => None,
        });
        self.encoded.push(encoded);
        let mut memory = Vec::new();
        let mut all = Vec::new();
        if let Some(sample) = &result.fire_queue_sample {
            for (ids, ..) in sample.values() {
                for id in ids {
                    all.push(*id);
                    if *id >= 50_000_000 {
                        memory.push(*id);
                    }
                }
            }
        }
        memory.sort_unstable();
        all.sort_unstable();
        let twin_fired = result
            .fire_queue_sample
            .as_ref()
            .and_then(|sample| sample.get(&self.twin_idx))
            .map(|(ids, ..)| ids.len())
            .unwrap_or(0);
        self.fired_twin.push(twin_fired);
        self.fired_memory.push(memory);
        self.fired_all.push(all);
        self.counts
            .push((self.memory_ids().len(), self.synapse_total()));
        self.burst = result.burst;
    }

    /// Cue a state by recall: its memory neuron fires with the recall kind next burst.
    fn cue(&mut self, neuron_id: u32) {
        self.npu
            .lock()
            .unwrap()
            .inject_memory_neuron_to_fcl_with_kind(
                neuron_id,
                self.mem_idx,
                1.5,
                FIRE_KIND_STDP_ELIGIBLE,
            );
    }

    fn quiet(&mut self, n: usize) {
        for _ in 0..n {
            self.step(None);
        }
    }

    fn states(&mut self, seq: &[usize]) {
        for s in seq {
            self.step(Some(*s));
        }
    }

    fn memory_ids(&self) -> Vec<u32> {
        let array = self.service.get_memory_neuron_array();
        let array = array.lock().unwrap();
        let mut ids = array.get_active_neurons_by_area(self.mem_idx);
        ids.sort_unstable();
        ids
    }

    /// Every valid sequential link, sorted.
    fn links(&self) -> Vec<Link> {
        let guard = self.npu.lock().unwrap();
        let mut out = Vec::new();
        for id in self.memory_ids() {
            for (target, weight, psp, _ty, flags, group) in guard.get_outgoing_synapses_detailed(id)
            {
                if flags & SEQ != 0 {
                    out.push(Link {
                        source: id,
                        target,
                        weight,
                        psp,
                        group,
                    });
                }
            }
        }
        out.sort_by_key(|l| (l.source, l.target));
        out
    }

    /// Distinct consecutive encoded neuron ids, in order: the chain recording should produce.
    fn expected_chain(&self) -> Vec<u32> {
        let mut chain: Vec<u32> = Vec::new();
        for id in self.encoded.iter().flatten() {
            if chain.last() != Some(id) {
                chain.push(*id);
            }
        }
        chain
    }

    fn detail(&self, id: u32) -> feagi_npu_plasticity::MemoryNeuronDetail {
        self.service
            .get_memory_neuron_array()
            .lock()
            .unwrap()
            .get_memory_neuron_detail(id)
            .expect("detail")
    }

    fn synapse_total(&self) -> usize {
        self.npu.lock().unwrap().get_synapse_count()
    }
}

/// A rig with capture bound at `magnitude`, a loop whose rule names a gate (so it has a
/// modulation group) and the capture driver held on.
fn capturing_rig(
    states: u32,
    lifecycle: MemoryNeuronLifecycleConfig,
    magnitude: f32,
) -> (Rig, u16) {
    capturing_rig_with(states, lifecycle, magnitude, Vec::new())
}

fn capturing_rig_with(
    states: u32,
    lifecycle: MemoryNeuronLifecycleConfig,
    magnitude: f32,
    mem_props: Vec<(&str, Value)>,
) -> (Rig, u16) {
    let mut rig = Rig::new(states, lifecycle, mem_props);
    rig.bind(
        vec![
            rig.binding(
                "cap",
                ModulatorKind::MemoryCapture,
                magnitude,
                rig.capture_idx,
            ),
            rig.binding("chain_gate", ModulatorKind::Gate, 100.0, rig.aux_idx),
        ],
        &["cap"],
    );
    rig.add_loop(&["chain_gate"]);
    let group = rig
        .npu
        .lock()
        .unwrap()
        .sequential_loop_params(rig.mem_idx)
        .expect("the loop is registered")
        .modulation_group;
    assert_ne!(group, 0, "the loop's rule must own a modulation group");
    rig.capture_on = true;
    (rig, group)
}

// ---------------------------------------------------------------------------------------------
// b. L = 12 distinct states
// ---------------------------------------------------------------------------------------------

#[test]
fn twelve_distinct_states_make_eleven_ordered_links_with_the_loop_group() {
    let (mut rig, group) = capturing_rig(12, life(30, 1.0, 1000), 50.0);
    // The loop builds no synapse of its own.
    assert!(rig.links().is_empty());
    rig.states(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    rig.quiet(1);
    let chain = rig.expected_chain();
    assert_eq!(chain.len(), 12, "twelve distinct states: {chain:?}");
    let links = rig.links();
    assert_eq!(links.len(), 11, "L-1 links: {links:?}");
    for pair in chain.windows(2) {
        let link = links
            .iter()
            .find(|l| l.source == pair[0] && l.target == pair[1])
            .unwrap_or_else(|| panic!("missing link {pair:?} in {links:?}"));
        assert_eq!(link.group, group, "every link carries the loop's group");
    }
    let info = rig.service.capture_area_info(rig.mem_idx).unwrap();
    assert_eq!((info.links, info.boundary_links, info.pieces), (11, 0, 1));
    println!(
        "b: chain {} states, {} links, group {group}",
        chain.len(),
        links.len()
    );
}

// ---------------------------------------------------------------------------------------------
// c. held input and silent bursts
// ---------------------------------------------------------------------------------------------

#[test]
fn held_input_is_one_state_and_a_silent_burst_does_not_break_the_chain() {
    let (mut rig, _) = capturing_rig(8, life(30, 1.0, 1000), 50.0);
    // A held for 5 bursts, then B: one link A -> B.
    rig.states(&[0, 0, 0, 0, 0, 1]);
    assert_eq!(rig.links().len(), 1, "held input is one state");
    assert_eq!(rig.memory_ids().len(), 2);
    // A silent burst mid-sequence, then more silence before C.
    rig.quiet(1);
    rig.states(&[2]);
    rig.quiet(2);
    rig.states(&[3]);
    let chain = rig.expected_chain();
    assert_eq!(chain.len(), 4);
    let links = rig.links();
    assert_eq!(links.len(), 3, "A->B->C->D across silent bursts: {links:?}");
    for pair in chain.windows(2) {
        assert!(
            links
                .iter()
                .any(|l| l.source == pair[0] && l.target == pair[1]),
            "chain is continuous"
        );
    }
    println!(
        "c: held 5 bursts = 1 state, {} links across silent bursts",
        links.len()
    );
}

// ---------------------------------------------------------------------------------------------
// d. pieces
// ---------------------------------------------------------------------------------------------

#[test]
fn thirty_bursts_at_cap_twelve_are_three_pieces_with_two_boundary_links() {
    let mut rig = Rig::new(
        30,
        life(60, 1.0, 1000),
        vec![("capture_max_bursts", json!(12))],
    );
    rig.bind_capture(50.0);
    rig.add_loop(&[]);
    rig.capture_on = true;
    let seq: Vec<usize> = (0..30).collect();
    rig.states(&seq);
    rig.capture_on = false;
    rig.quiet(1);
    let info = rig.service.capture_area_info(rig.mem_idx).unwrap();
    assert_eq!(info.pieces, 3, "30 bursts at cap 12");
    assert_eq!(info.boundary_links, 2, "two piece boundaries");
    assert!(!info.recording, "the session ended with the driver");
    let chain = rig.expected_chain();
    let links = rig.links();
    assert_eq!(chain.len(), 30);
    assert_eq!(
        links.len(),
        29,
        "chain continuous across pieces: {}",
        links.len()
    );
    for pair in chain.windows(2) {
        assert!(links
            .iter()
            .any(|l| l.source == pair[0] && l.target == pair[1]));
    }
    println!(
        "d: pieces {}, boundary links {}, links {}",
        info.pieces,
        info.boundary_links,
        links.len()
    );
}

// ---------------------------------------------------------------------------------------------
// e. boost
// ---------------------------------------------------------------------------------------------

#[test]
fn captured_neurons_outlive_five_times_the_initial_lifespan_and_uncaptured_ones_do_not() {
    let initial = 4;
    let mut rig = Rig::new(8, life(initial, 1.0, 100), Vec::new());
    rig.bind_capture(50.0);
    rig.add_loop(&[]);
    // Uncaptured states first (driver off).
    rig.states(&[6, 7]);
    let uncaptured: Vec<u32> = rig.memory_ids();
    assert_eq!(uncaptured.len(), 2);
    rig.capture_on = true;
    rig.states(&[0, 1, 2]);
    rig.capture_on = false;
    let captured: Vec<u32> = rig
        .memory_ids()
        .into_iter()
        .filter(|id| !uncaptured.contains(id))
        .collect();
    assert_eq!(captured.len(), 3);
    rig.quiet((initial * 5) as usize);
    let alive = rig.memory_ids();
    for id in &captured {
        assert!(
            alive.contains(id),
            "captured neuron {id} survives 5x init lifespan"
        );
    }
    for id in &uncaptured {
        assert!(!alive.contains(id), "uncaptured neuron {id} aged out");
    }
    println!(
        "e: {} captured alive after {} quiet bursts, {} uncaptured gone",
        captured.len(),
        initial * 5,
        uncaptured.len()
    );
}

#[test]
fn magnitude_one_hundred_makes_a_state_long_term_at_once() {
    let (mut rig, _) = capturing_rig(4, life(4, 1.0, 100), 100.0);
    rig.states(&[0]);
    let id = rig.memory_ids()[0];
    let detail = rig.detail(id);
    assert!(
        detail.is_longterm_memory,
        "long-term in the burst it was captured"
    );
    rig.capture_on = false;
    rig.quiet(40);
    assert!(
        rig.memory_ids().contains(&id),
        "long-term neurons never expire"
    );
    println!("e: magnitude 100 -> long-term immediately, alive after 40 quiet bursts");
}

#[test]
fn a_held_state_is_boosted_once_not_per_burst() {
    let initial = 10;
    let threshold = 100;
    let growth = 1.0_f32;
    let (mut rig, _) = capturing_rig(4, life(initial, growth, threshold), 20.0);
    rig.states(&[0, 0, 0, 0, 0]);
    let id = rig.memory_ids()[0];
    let gap = f64::from(threshold - initial);
    let boost = (gap * 0.2).ceil() as u32;
    // Created at the first burst (boosted there once), then re-encoded four times: each
    // reactivation adds the ordinary growth rate and nothing else.
    let once = initial + boost + 4 * growth as u32;
    assert_eq!(
        rig.detail(id).lifespan_current,
        once,
        "one boost of 20% of the gap plus four ordinary reactivations, not one boost per burst"
    );
    assert_eq!(
        rig.service.capture_area_info(rig.mem_idx).unwrap().boosted,
        1
    );
    println!(
        "e: held 5 bursts, lifespan {once} = initial {initial} + one boost {boost} + 4 growth"
    );
}

// ---------------------------------------------------------------------------------------------
// f. silent driver
// ---------------------------------------------------------------------------------------------

/// Per-burst `(memory neurons, synapses)`, memory fires and all fires of one scripted run.
type ScriptTrace = (Vec<(usize, usize)>, Vec<Vec<u32>>, Vec<Vec<u32>>);

fn run_plain_script(subscribed: bool) -> ScriptTrace {
    let mut rig = Rig::new(8, life(14, 2.0, 1000), Vec::new());
    if subscribed {
        rig.bind_capture(50.0);
        rig.add_loop(&[]);
    } else {
        rig.add_loop(&[]);
    }
    // The driver is never stimulated.
    rig.states(&[0, 1, 0, 0, 2]);
    rig.quiet(3);
    rig.states(&[1, 3, 2]);
    rig.quiet(12);
    (
        rig.counts.clone(),
        rig.fired_memory.clone(),
        rig.fired_all.clone(),
    )
}

#[test]
fn a_subscribed_but_silent_driver_changes_nothing() {
    let with = run_plain_script(true);
    let without = run_plain_script(false);
    assert_eq!(
        with.0, without.0,
        "neuron and synapse counts after every burst"
    );
    assert_eq!(with.1, without.1, "memory fires per burst");
    assert_eq!(with.2, without.2, "all fires per burst");
    let peak = with.0.iter().map(|c| c.0).max().unwrap();
    assert!(peak >= 4, "the script forms neurons ({peak})");
    println!(
        "f: {} bursts, peak {} neurons, {} fires identical with and without the subscription",
        with.0.len(),
        peak,
        with.2.iter().map(Vec::len).sum::<usize>()
    );
}

// ---------------------------------------------------------------------------------------------
// g. no loop
// ---------------------------------------------------------------------------------------------

#[test]
fn without_a_loop_there_are_no_links_but_the_boost_applies() {
    let mut rig = Rig::new(8, life(4, 0.0, 100), Vec::new());
    rig.bind_capture(100.0);
    rig.capture_on = true;
    rig.states(&[0, 1, 2, 3]);
    assert!(rig.links().is_empty(), "no loop, no links");
    let info = rig.service.capture_area_info(rig.mem_idx).unwrap();
    assert_eq!(info.links, 0);
    assert_eq!(info.boosted, 4, "the boost still applies");
    for id in rig.memory_ids() {
        assert!(rig.detail(id).is_longterm_memory);
    }
    println!("g: 0 links, {} neurons boosted to long-term", info.boosted);
}

// ---------------------------------------------------------------------------------------------
// h. formation and freeze
// ---------------------------------------------------------------------------------------------

#[test]
fn formation_minus_one_hundred_links_only_known_states() {
    let mut rig = Rig::new(8, life(40, 1.0, 1000), Vec::new());
    rig.add_loop(&[]);
    // A and B become known before capture.
    rig.states(&[0, 1]);
    rig.quiet(1);
    let known = rig.memory_ids();
    assert_eq!(known.len(), 2);
    rig.bind(
        vec![
            rig.binding("cap", ModulatorKind::MemoryCapture, 50.0, rig.capture_idx),
            rig.binding("form", ModulatorKind::MemoryFormation, -100.0, rig.aux_idx),
        ],
        &["cap", "form"],
    );
    rig.capture_on = true;
    rig.aux_on = true;
    // B, an unknown state C (no neuron may form), A.
    rig.states(&[1, 5, 0]);
    assert_eq!(rig.memory_ids().len(), 2, "no neuron was created");
    let links = rig.links();
    assert_eq!(
        links.len(),
        1,
        "only the known states are linked: {links:?}"
    );
    assert_eq!((links[0].source, links[0].target), (known[1], known[0]));
    println!(
        "h: formation -100 -> {} link between known states, 0 neurons created",
        links.len()
    );
}

#[test]
fn freeze_records_nothing() {
    let mut rig = Rig::new(8, life(40, 0.0, 1000), Vec::new());
    rig.add_loop(&[]);
    rig.states(&[0, 1]);
    rig.quiet(1);
    let ids = rig.memory_ids();
    let before: Vec<u32> = ids
        .iter()
        .map(|id| rig.detail(*id).lifespan_current)
        .collect();
    rig.bind(
        vec![
            rig.binding("cap", ModulatorKind::MemoryCapture, 100.0, rig.capture_idx),
            rig.binding("frz", ModulatorKind::MemoryFreeze, 100.0, rig.aux_idx),
        ],
        &["cap", "frz"],
    );
    rig.capture_on = true;
    rig.aux_on = true;
    rig.states(&[0, 1, 0, 1]);
    assert!(rig.links().is_empty(), "freeze outranks capture: no links");
    let info = rig.service.capture_area_info(rig.mem_idx).unwrap();
    assert_eq!((info.links, info.boosted), (0, 0), "no links, no boost");
    let after: Vec<u32> = ids
        .iter()
        .map(|id| rig.detail(*id).lifespan_current)
        .collect();
    assert_eq!(before, after, "lifespans stand still");
    for id in &ids {
        assert!(!rig.detail(*id).is_longterm_memory);
    }
    println!("h: freeze -> 0 links, 0 boosts, lifespans {after:?} unchanged");
}

// ---------------------------------------------------------------------------------------------
// i. mapping edits protect the links
// ---------------------------------------------------------------------------------------------

fn assoc_rule() -> Value {
    json!({
        "morphology_id": "associative_memory",
        "morphology_scalar": 1,
        "postSynapticCurrent_multiplier": 1.0,
        "plasticity_mode": "stdp",
        "plasticity_constant": 20,
        "ltp_multiplier": 5,
        "ltd_multiplier": 1,
        "plasticity_window": 3,
        "max_weight": 100.0,
    })
}

fn loop_rule(modulators: &[&str]) -> Value {
    json!({
        "morphology_id": "sequential_memory",
        "morphology_scalar": 1,
        "postSynapticCurrent_multiplier": 1.0,
        "modulators": modulators,
    })
}

#[test]
fn a_mapping_edit_keeps_the_links_and_restamps_their_group() {
    let (mut rig, old_group) = capturing_rig(8, life(60, 1.0, 1000), 50.0);
    rig.states(&[0, 1, 2, 3, 4]);
    let before = rig.links();
    assert_eq!(before.len(), 4);
    // A second synaptic modulator: the loop now names both, which is a different group.
    rig.bind(
        vec![
            rig.binding("cap", ModulatorKind::MemoryCapture, 50.0, rig.capture_idx),
            rig.binding("chain_gate", ModulatorKind::Gate, 100.0, rig.aux_idx),
            rig.binding(
                "chain_gain",
                ModulatorKind::TransmissionGain,
                50.0,
                rig.aux_idx,
            ),
        ],
        &["cap"],
    );
    rig.set_self_mapping(vec![loop_rule(&["chain_gate", "chain_gain"])]);
    let new_group = rig
        .npu
        .lock()
        .unwrap()
        .sequential_loop_params(rig.mem_idx)
        .unwrap()
        .modulation_group;
    assert_ne!(new_group, old_group);
    let after = rig.links();
    assert_eq!(after.len(), 4, "the edit keeps every link");
    for (b, a) in before.iter().zip(&after) {
        assert_eq!((b.source, b.target), (a.source, a.target));
        assert_eq!(a.group, new_group, "group re-stamped");
    }
    // Recording continues on the new rule.
    rig.states(&[5]);
    let latest = rig.links();
    assert_eq!(latest.len(), 5);
    assert!(latest.iter().all(|l| l.group == new_group));
    println!("i: edit kept 4 links, group {old_group} -> {new_group}");
}

#[test]
fn an_associative_stdp_self_mapping_leaves_the_links_unchanged() {
    let (mut rig, group) = capturing_rig(8, life(80, 1.0, 1000), 50.0);
    rig.states(&[0, 1, 2, 3]);
    let before = rig.links();
    assert_eq!(before.len(), 3);
    rig.set_self_mapping(vec![loop_rule(&["chain_gate"]), assoc_rule()]);
    assert_eq!(
        rig.links(),
        before,
        "adding the STDP rule does not touch the links"
    );
    // Co-activity the STDP pass would act on: replay the same states, then quiet bursts.
    rig.capture_on = false;
    for _ in 0..3 {
        rig.states(&[0, 1, 2, 3]);
        rig.quiet(4);
    }
    let after = rig.links();
    assert_eq!(
        after, before,
        "STDP does not decay, strengthen or remove links"
    );
    assert!(after.iter().all(|l| l.group == group));
    println!(
        "i: STDP self-mapping on the same area, {} links identical",
        after.len()
    );
}

#[test]
fn deleting_the_loop_removes_the_links() {
    let (mut rig, _) = capturing_rig(8, life(60, 1.0, 1000), 50.0);
    rig.states(&[0, 1, 2]);
    assert_eq!(rig.links().len(), 2);
    rig.set_self_mapping(Vec::new());
    assert!(rig.links().is_empty(), "no loop, no links");
    assert!(rig
        .npu
        .lock()
        .unwrap()
        .sequential_loop_params(rig.mem_idx)
        .is_none());
    println!("i: loop deleted -> 0 links");
}

// ---------------------------------------------------------------------------------------------
// k. expiry
// ---------------------------------------------------------------------------------------------

#[test]
fn expiring_a_member_removes_its_links() {
    // Magnitude 1 keeps the boost tiny so the neurons still age out.
    let mut rig = Rig::new(8, life(3, 1.0, 1000), Vec::new());
    rig.bind_capture(1.0);
    rig.add_loop(&[]);
    rig.capture_on = true;
    rig.states(&[0, 1, 2]);
    assert_eq!(rig.links().len(), 2);
    let first = rig.expected_chain()[0];
    rig.capture_on = false;
    rig.quiet(30);
    assert!(!rig.memory_ids().contains(&first), "the member expired");
    let guard = rig.npu.lock().unwrap();
    let total_seq = {
        let mut n = 0;
        for id in rig.expected_chain() {
            n += guard
                .get_outgoing_synapses_detailed(id)
                .iter()
                .filter(|s| s.4 & SEQ != 0)
                .count();
            n += guard
                .get_incoming_synapses_detailed(id)
                .iter()
                .filter(|s| s.4 & SEQ != 0)
                .count();
        }
        n
    };
    assert_eq!(total_seq, 0, "links die with their neurons");
    println!("k: members expired -> 0 links left");
}

// ---------------------------------------------------------------------------------------------
// misc: an unflagged helper guards the associative flag stays distinct
// ---------------------------------------------------------------------------------------------

#[test]
fn edge_flag_bits_are_distinct_and_named() {
    assert_ne!(SYNAPSE_EDGE_ASSOCIATIVE_MEMORY, SEQ);
    let names: BTreeSet<&str> = feagi_npu_neural::synapse::SYNAPSE_EDGE_FLAG_NAMES
        .iter()
        .map(|(_, n)| *n)
        .collect();
    assert!(names.contains("sequential_memory"));
}

// =============================================================================================
// Stage 3: chain replay
// =============================================================================================

/// A capturing rig whose loop names a gate and, optionally, a transmission-gain instance that
/// is driven by the same aux driver (magnitude -100 makes the product 0 while it fires).
fn replay_rig(
    states: u32,
    magnitude: f32,
    mem_props: Vec<(&str, Value)>,
    with_zero_gain: bool,
) -> Rig {
    let mut rig = Rig::new(states, life(30, 1.0, 1000), mem_props);
    let mut bindings = vec![
        rig.binding(
            "cap",
            ModulatorKind::MemoryCapture,
            magnitude,
            rig.capture_idx,
        ),
        rig.binding("chain_gate", ModulatorKind::Gate, 100.0, rig.aux_idx),
    ];
    let mut loop_modulators = vec!["chain_gate"];
    if with_zero_gain {
        bindings.push(rig.binding(
            "chain_gain",
            ModulatorKind::TransmissionGain,
            -100.0,
            rig.aux_idx,
        ));
        loop_modulators.push("chain_gain");
    }
    rig.bind(bindings, &["cap"]);
    rig.add_loop(&loop_modulators);
    rig
}

/// Record `seq` in one session (the capture driver on), then switch the driver off.
fn record(rig: &mut Rig, seq: &[usize]) {
    rig.capture_on = true;
    rig.states(seq);
    rig.quiet(1);
    rig.capture_on = false;
    rig.quiet(2);
}

/// Twin frames and memory spikes from `from` on.
fn twin_and_memory(rig: &Rig, from: usize) -> (usize, usize) {
    (
        rig.fired_twin[from..].iter().sum(),
        rig.fired_memory[from..].iter().map(Vec::len).sum(),
    )
}

#[test]
fn replay_length_matches_the_episode_length() {
    for len in [6usize, 12, 24] {
        let mut rig = replay_rig(
            len as u32,
            50.0,
            vec![("capture_max_bursts", json!(200))],
            false,
        );
        let seq: Vec<usize> = (0..len).collect();
        record(&mut rig, &seq);
        let chain = rig.expected_chain();
        assert_eq!(chain.len(), len);
        assert_eq!(rig.links().len(), len - 1);
        // The gate opens (its commit applies from the next burst), then the cue.
        rig.aux_on = true;
        rig.quiet(2);
        let from = rig.fired_twin.len();
        rig.cue(chain[0]);
        rig.quiet(len + 6);
        let (twin, memory) = twin_and_memory(&rig, from);
        // Cued state: 1 spike (the cue); each step is a chain-recall fire.
        // Q1: the episode's E fires once more, at the last state's burst.
        assert_eq!(memory, len + 1, "memory spikes = states + E, len {len}");
        assert!(
            (twin as i64 - len as i64).abs() <= 1,
            "len {len}: twin frames {twin}"
        );
        // The steps arrive in episode order, one burst apart.
        let episodes = rig.episodes();
        let order: Vec<u32> = rig.fired_memory[from..]
            .iter()
            .flatten()
            .copied()
            .filter(|id| !episodes.contains(id))
            .collect();
        assert_eq!(order, chain, "len {len}: replay order");
        println!("b: episode {len} -> replay {twin} twin frames, {memory} memory spikes");
    }
}

#[test]
fn a_closed_gate_or_zero_gain_replays_only_the_cued_state_an_open_gate_the_episode() {
    // `memory` counts the states plus, with the gate open, the episode's E (Q1).
    for (name, gate_on, zero_gain, expected, memory_expected) in [
        ("gate closed", false, false, 1usize, 1usize),
        ("gain 0", true, true, 1, 1),
        ("gate open", true, false, 8, 9),
    ] {
        let mut rig = replay_rig(8, 50.0, vec![("capture_max_bursts", json!(200))], zero_gain);
        record(&mut rig, &[0, 1, 2, 3, 4, 5, 6, 7]);
        let chain = rig.expected_chain();
        rig.aux_on = gate_on;
        rig.quiet(2);
        let from = rig.fired_twin.len();
        rig.cue(chain[0]);
        rig.quiet(14);
        let (twin, memory) = twin_and_memory(&rig, from);
        assert_eq!(twin, expected, "{name}: twin frames");
        assert_eq!(memory, memory_expected, "{name}: memory spikes");
        println!("c: {name} -> {twin} twin frame(s)");
    }
}

#[test]
fn a_second_cue_during_a_replay_adds_no_frames_and_no_memory_spikes() {
    let run = |second_cue: bool| -> (usize, usize, u64) {
        let mut rig = replay_rig(10, 50.0, vec![("capture_max_bursts", json!(200))], false);
        record(&mut rig, &[0, 1, 2, 3, 4, 5, 6, 7]);
        // A separate session records one more state: it has no successor.
        record(&mut rig, &[8]);
        let chain: Vec<u32> = rig.expected_chain();
        let lone = *chain.last().unwrap();
        rig.aux_on = true;
        rig.quiet(2);
        let from = rig.fired_twin.len();
        rig.cue(chain[0]);
        rig.quiet(2);
        if second_cue {
            rig.cue(lone);
        }
        rig.quiet(14);
        let (twin, memory) = twin_and_memory(&rig, from);
        let dropped = rig.npu.lock().unwrap().memory_replay_blocked_counts().0;
        (twin, memory, dropped)
    };
    let (base_twin, base_memory, base_dropped) = run(false);
    let (twin, memory, dropped) = run(true);
    assert_eq!((twin, memory), (base_twin, base_memory), "second cue");
    assert_eq!(
        (base_dropped, dropped),
        (0, 1),
        "the held-back cue was dropped"
    );
    // Once the chain is over the same cue replays its own frame.
    let mut rig = replay_rig(10, 50.0, vec![("capture_max_bursts", json!(200))], false);
    record(&mut rig, &[0, 1]);
    record(&mut rig, &[8]);
    let lone = *rig.expected_chain().last().unwrap();
    let from = rig.fired_twin.len();
    rig.cue(lone);
    rig.quiet(6);
    let (after, _) = twin_and_memory(&rig, from);
    assert_eq!(after, 1, "with no chain running the cue replays its frame");
    let (lockout, recording) = rig.npu.lock().unwrap().memory_replay_blocked_counts();
    println!(
        "d: baseline {base_twin} frames / {base_memory} spikes, with a second cue {twin} / {memory}; \
         idle cue of {lone} replays {after}; blocked by lockout {lockout} recording {recording}"
    );
}

#[test]
fn an_a_b_a_c_chain_stops_at_the_visited_state() {
    let mut rig = replay_rig(4, 50.0, vec![("capture_max_bursts", json!(200))], false);
    // A B A C: links A->B, B->A, A->C.
    record(&mut rig, &[0, 1, 0, 2]);
    let a = rig.expected_chain()[0];
    let b = rig.expected_chain()[1];
    assert_eq!(rig.links().len(), 3);
    // B fires again outside the recording: B is now the most recently fired successor of A.
    rig.step(Some(1));
    rig.quiet(4);
    rig.aux_on = true;
    rig.quiet(2);
    let from = rig.fired_twin.len();
    rig.cue(a);
    rig.quiet(14);
    let (twin, _) = twin_and_memory(&rig, from);
    let fires: Vec<u32> = rig.fired_memory[from..].iter().flatten().copied().collect();
    assert_eq!(
        fires,
        vec![a, b],
        "A then B, B's only successor A already played"
    );
    assert_eq!(twin, 2);
    println!("e: A-B-A-C replay = {fires:?} ({twin} frames)");
}

/// Two episodes share state A: A-B (session 1) and A-C (session 2).
fn shared_state_rig(first_magnitude: f32, second_magnitude: f32) -> (Rig, [u32; 3]) {
    let mut rig = replay_rig(
        3,
        first_magnitude,
        vec![("capture_max_bursts", json!(200))],
        false,
    );
    record(&mut rig, &[0, 1]);
    let a = rig.expected_chain()[0];
    let b = rig.expected_chain()[1];
    rig.bind(
        vec![
            rig.binding(
                "cap",
                ModulatorKind::MemoryCapture,
                second_magnitude,
                rig.capture_idx,
            ),
            rig.binding("chain_gate", ModulatorKind::Gate, 100.0, rig.aux_idx),
        ],
        &["cap"],
    );
    let before = rig.encoded.len();
    record(&mut rig, &[0, 2]);
    let c = rig.encoded[before..]
        .iter()
        .flatten()
        .copied()
        .find(|id| *id != a)
        .unwrap();
    (rig, [a, b, c])
}

fn first_step_after_cue(rig: &mut Rig, cue: u32) -> u32 {
    rig.aux_on = true;
    rig.quiet(2);
    let from = rig.fired_memory.len();
    rig.cue(cue);
    rig.quiet(6);
    let fires: Vec<u32> = rig.fired_memory[from..].iter().flatten().copied().collect();
    assert_eq!(fires[0], cue);
    fires[1]
}

#[test]
fn a_shared_state_follows_the_stronger_branch_then_the_one_fired_most_recently() {
    // Long-term B (magnitude 100) against short-term C (magnitude 20) recorded later.
    let (mut rig, [a, b, c]) = shared_state_rig(100.0, 20.0);
    assert!(rig.detail(b).is_longterm_memory && !rig.detail(c).is_longterm_memory);
    assert_eq!(rig.links().len(), 2, "A->B and A->C");
    let followed = first_step_after_cue(&mut rig, a);
    assert_eq!(
        followed, b,
        "long-term beats short-term even though C is newer"
    );

    // Equal strength: both long-term, same count and lifespan; the one fired last wins.
    let (mut rig, [a, b, c]) = shared_state_rig(100.0, 100.0);
    let (db, dc) = (rig.detail(b), rig.detail(c));
    assert_eq!(
        (
            db.is_longterm_memory,
            db.lifespan_current,
            db.activation_count
        ),
        (
            dc.is_longterm_memory,
            dc.lifespan_current,
            dc.activation_count
        ),
        "the first three keys tie"
    );
    assert!(dc.last_activation_burst > db.last_activation_burst);
    assert_eq!(
        first_step_after_cue(&mut rig, a),
        c,
        "C fired most recently"
    );
    // B fires again, live: now it is the most recent (and also has the larger count).
    rig.step(Some(1));
    rig.quiet(2);
    assert_eq!(
        first_step_after_cue(&mut rig, a),
        b,
        "B fired most recently"
    );
    println!("f: long-term beats short-term; equal strength -> most recently fired");
}

#[test]
fn a_cue_while_recording_replays_nothing_and_a_running_replay_finishes() {
    let mut rig = replay_rig(10, 50.0, vec![("capture_max_bursts", json!(200))], false);
    record(&mut rig, &[0, 1, 2, 3, 4, 5, 6, 7]);
    let chain = rig.expected_chain();
    rig.aux_on = true;
    rig.quiet(2);
    // The driver turns on and nothing is encoded: the area records with no new states.
    rig.capture_on = true;
    rig.quiet(3);
    assert!(rig
        .npu
        .lock()
        .unwrap()
        .memory_capture_recording(rig.mem_idx));
    let from = rig.fired_twin.len();
    rig.cue(chain[0]);
    rig.quiet(14);
    let (twin, memory) = twin_and_memory(&rig, from);
    assert_eq!(
        (twin, memory),
        (0, 1),
        "cue while recording: no frame, no step"
    );
    assert!(!rig.npu.lock().unwrap().chain_replay_active(rig.mem_idx));
    let blocked = rig.npu.lock().unwrap().memory_replay_blocked_counts().1;
    assert!(blocked >= 1);

    // A replay that is already running when recording starts finishes all its frames.
    let mut rig = replay_rig(10, 50.0, vec![("capture_max_bursts", json!(200))], false);
    record(&mut rig, &[0, 1, 2, 3, 4, 5, 6, 7]);
    let chain = rig.expected_chain();
    rig.aux_on = true;
    rig.quiet(2);
    let from = rig.fired_twin.len();
    rig.cue(chain[0]);
    rig.quiet(2);
    assert!(rig.npu.lock().unwrap().chain_replay_active(rig.mem_idx));
    rig.capture_on = true;
    rig.quiet(16);
    let (twin, memory) = twin_and_memory(&rig, from);
    assert_eq!(
        (twin, memory),
        (8, 9),
        "the running replay finished (8 states + E, Q1)"
    );
    println!("g: cue while recording -> 0 twin frames; running replay finished {twin} frames");
}

#[test]
fn temporal_depth_three_chain_windows_overlap_on_the_twin() {
    let mut rig = replay_rig(
        12,
        50.0,
        vec![
            ("capture_max_bursts", json!(200)),
            ("temporal_depth", json!(3)),
        ],
        false,
    );
    // Trailing silence while still recording, so the last windows are encoded and linked.
    rig.capture_on = true;
    rig.states(&[0, 1, 2, 3, 4, 5, 6, 7]);
    rig.quiet(5);
    rig.capture_on = false;
    rig.quiet(2);
    let chain = rig.expected_chain();
    assert!(chain.len() >= 2, "depth 3 records window states: {chain:?}");
    assert_eq!(
        rig.links().len(),
        chain.len() - 1,
        "one link per consecutive window"
    );
    rig.aux_on = true;
    rig.quiet(2);
    let from = rig.fired_twin.len();
    rig.cue(chain[0]);
    rig.quiet(chain.len() + 8);
    let per_burst: Vec<usize> = rig.fired_twin[from..].to_vec();
    let episodes = rig.episodes();
    let fires: Vec<u32> = rig.fired_memory[from..]
        .iter()
        .flatten()
        .copied()
        .filter(|id| !episodes.contains(id))
        .collect();
    assert_eq!(fires, chain, "every state of the episode plays once");
    let active = per_burst.iter().filter(|n| **n > 0).count();
    // One window played alone: the gate closes, so the cued state replays only itself.
    rig.aux_on = false;
    rig.quiet(3);
    let from = rig.fired_twin.len();
    rig.cue(chain[0]);
    rig.quiet(8);
    let alone: Vec<usize> = rig.fired_twin[from..].to_vec();
    let window_frames: usize = alone.iter().sum();
    println!(
        "h: depth 3, {} states; chain twin spikes per burst {per_burst:?}; one window alone {alone:?}; \
         stored frames of the first states {:?}",
        chain.len(),
        chain.iter().take(3).map(|id| rig.frames[id].clone()).collect::<Vec<_>>()
    );
    // Each chain neuron schedules its whole window, so successive windows overlap: the chain
    // touches fewer twin bursts than the windows would take one after the other, because
    // neighbouring windows replay the same upstream frame in the same burst (merged on the
    // twin: one spike).
    assert!(window_frames >= 2, "a window has several frames: {alone:?}");
    assert!(
        active < chain.len() * window_frames,
        "overlap: {active} twin bursts for {} windows of {window_frames} frames",
        chain.len()
    );
}

#[test]
fn chain_recall_is_a_stored_fire_kind() {
    let mut rig = replay_rig(6, 50.0, vec![("capture_max_bursts", json!(200))], false);
    rig.npu
        .lock()
        .unwrap()
        .configure_fire_ledger_window(rig.mem_idx, 64)
        .expect("track the memory area on the fire ledger");
    record(&mut rig, &[0, 1, 2, 3, 4, 5]);
    let chain = rig.expected_chain();
    rig.aux_on = true;
    rig.quiet(2);
    let cue_burst = rig.burst + 1;
    rig.cue(chain[0]);
    rig.quiet(10);
    let guard = rig.npu.lock().unwrap();
    let window = guard
        .get_all_fire_ledger_configs()
        .into_iter()
        .find(|(area, _)| *area == rig.mem_idx)
        .map(|(_, w)| w)
        .expect("the memory area is on the fire ledger");
    let frames = guard
        .get_fire_ledger_recent_frames_with_kinds(rig.mem_idx, window.max(16))
        .expect("frames with kinds");
    let mut kinds: Vec<(u64, u32, u8)> = Vec::new();
    for frame in frames {
        for id in frame.fired.iter() {
            kinds.push((frame.timestep, id, frame.kind_of(id)));
        }
    }
    let chain_kind: Vec<u32> = kinds
        .iter()
        .filter(|(t, _, k)| *t >= cue_burst && *k == FIRE_KIND_CHAIN_RECALL)
        .map(|(_, id, _)| *id)
        .collect();
    let recall_kind: Vec<u32> = kinds
        .iter()
        .filter(|(t, _, k)| *t >= cue_burst && *k == FIRE_KIND_STDP_ELIGIBLE)
        .map(|(_, id, _)| *id)
        .collect();
    // Q1: the episode's E also fires with the chain-recall kind, at the last state's burst.
    let episodes = rig.episodes();
    let steps: Vec<u32> = chain_kind
        .iter()
        .copied()
        .filter(|id| !episodes.contains(id))
        .collect();
    assert_eq!(steps, chain[1..].to_vec(), "steps carry the chain kind");
    assert_eq!(chain_kind.len(), steps.len() + 1, "plus the one E fire");
    assert_eq!(recall_kind, vec![chain[0]], "the cue keeps the recall kind");
    println!(
        "i: stored kinds: cue recall, {} steps chain-recall",
        chain_kind.len()
    );
}

#[test]
fn a_loop_free_area_never_starts_or_blocks_a_replay() {
    // No loop, no capture: the same cue replays its own frame and nothing else changes.
    let mut rig = Rig::new(
        4,
        life(30, 1.0, 1000),
        vec![("capture_max_bursts", json!(200))],
    );
    rig.states(&[0, 1, 2]);
    rig.quiet(3);
    let id = rig.expected_chain()[0];
    let from = rig.fired_twin.len();
    rig.cue(id);
    rig.cue(rig.expected_chain()[1]);
    rig.quiet(6);
    let (twin, memory) = twin_and_memory(&rig, from);
    assert_eq!((twin, memory), (2, 2));
    assert_eq!(
        rig.npu.lock().unwrap().memory_replay_blocked_counts(),
        (0, 0)
    );
}

// =============================================================================================
// Stage 4: episode neuron
// =============================================================================================

/// `(burst, kind)` of every fire of `id` the memory area's ledger still holds.
fn fire_kinds_of(rig: &Rig, id: u32) -> Vec<(u64, u8)> {
    let guard = rig.npu.lock().unwrap();
    let window = guard
        .get_all_fire_ledger_configs()
        .into_iter()
        .find(|(area, _)| *area == rig.mem_idx)
        .map(|(_, w)| w)
        .expect("the memory area is on the fire ledger");
    let frames = guard
        .get_fire_ledger_recent_frames_with_kinds(rig.mem_idx, window.max(64))
        .expect("frames with kinds");
    let mut out = Vec::new();
    for frame in frames {
        if frame.fired.contains(id) {
            out.push((frame.timestep, frame.kind_of(id)));
        }
    }
    out
}

fn members_of(rig: &Rig, episode: u32) -> Vec<u32> {
    rig.detail(episode).episode_members
}

fn open_gate(rig: &mut Rig) {
    rig.aux_on = true;
    rig.quiet(2);
}

#[test]
fn one_episode_neuron_per_new_chain_and_none_for_a_re_recording() {
    let mut rig = replay_rig(6, 50.0, vec![("capture_max_bursts", json!(200))], false);
    assert!(rig.episodes().is_empty());
    record(&mut rig, &[0, 1, 2]);
    rig.quiet(3);
    let first = rig.episodes();
    assert_eq!(first.len(), 1, "one chain, one episode neuron");
    let e = first[0];
    let chain = rig.expected_chain();
    assert_eq!(
        members_of(&rig, e),
        chain,
        "the member list is the ordered states"
    );
    let before = rig.detail(e).activation_count;

    // The same chain again: no new episode neuron, the same one is reactivated.
    record(&mut rig, &[0, 1, 2]);
    rig.quiet(3);
    assert_eq!(rig.episodes(), vec![e], "a re-recorded chain adds 0");
    let after = rig.detail(e).activation_count;
    assert!(after > before, "activation_count {before} -> {after}");

    // A different chain is a new episode.
    record(&mut rig, &[3, 4, 5]);
    rig.quiet(3);
    assert_eq!(rig.episodes().len(), 2, "a new chain adds 1");
    let info = rig.service.capture_area_info(rig.mem_idx).unwrap();
    assert_eq!(
        (
            info.episodes_created,
            info.episodes_reactivated,
            info.episodes_skipped
        ),
        (2, 1, 0)
    );
    println!(
        "b: 1 chain -> 1 E, re-record -> still 1 (activation_count {before} -> {after}), new chain -> 2"
    );
}

#[test]
fn an_episode_neuron_spikes_once_at_the_recording_end_and_once_per_e_started_replay() {
    let mut rig = replay_rig(6, 50.0, vec![("capture_max_bursts", json!(200))], false);
    rig.npu
        .lock()
        .unwrap()
        .configure_fire_ledger_window(rig.mem_idx, 64)
        .expect("track the memory area on the fire ledger");
    record(&mut rig, &[0, 1, 2, 3]);
    rig.quiet(3);
    let e = rig.episodes()[0];
    let at_end = fire_kinds_of(&rig, e);
    assert_eq!(
        at_end.len(),
        1,
        "one spike at the recording end: {at_end:?}"
    );
    assert_eq!(
        at_end[0].1, FIRE_KIND_EPISODIC_MEMORY,
        "an ordinary episodic fire"
    );

    open_gate(&mut rig);
    let cue_burst = rig.burst + 1;
    rig.cue(e);
    rig.quiet(10);
    let all = fire_kinds_of(&rig, e);
    let replay: Vec<(u64, u8)> = all
        .iter()
        .copied()
        .filter(|(t, _)| *t >= cue_burst)
        .collect();
    assert_eq!(replay.len(), 1, "one spike per E-started replay: {all:?}");
    assert_eq!(replay[0].1, FIRE_KIND_CHAIN_RECALL, "the chain-recall kind");
    // No second E spike at completion, and no new neuron was created by the replay.
    assert_eq!(rig.episodes(), vec![e]);
    println!(
        "c: E spikes: recording end kind {} x1, replay kind {} x1",
        at_end[0].1, replay[0].1
    );
}

#[test]
fn an_associative_cue_of_an_episode_replays_its_frames_in_order_through_a_shared_state() {
    use feagi_npu_neural::types::{SynapticPsp, SynapticWeight};
    // Magnitude 100 makes states and episodes long-term at once: associative recall acts on
    // long-term memory neurons.
    let mut rig = replay_rig(5, 100.0, vec![("capture_max_bursts", json!(200))], false);
    // Two episodes share state 1: 0-1-2 and 3-1-4.
    record(&mut rig, &[0, 1, 2]);
    record(&mut rig, &[3, 1, 4]);
    rig.quiet(3);
    let episodes = rig.episodes();
    assert_eq!(episodes.len(), 2);
    open_gate(&mut rig);

    // A cue area outside the memory area's upstream set, with one associative synapse onto each
    // episode neuron: firing it recalls the episode as an associative cue would.
    let (cue_id, _cue_idx) = {
        let mut manager = rig.manager.write();
        regular_area(&mut manager, b"ccue0001", "cuearea", 2)
    };
    let cue_neurons = {
        let guard = rig.npu.lock().unwrap();
        let idx = rig.manager.read().get_cortical_idx(&cue_id).unwrap();
        guard.get_neurons_in_cortical_area(idx)
    };
    {
        let mut guard = rig.npu.lock().unwrap();
        for (neuron, episode) in cue_neurons.iter().zip(&episodes) {
            guard
                .add_synapse(
                    NeuronId(*neuron),
                    NeuronId(*episode),
                    SynapticWeight(255.0),
                    SynapticPsp(255.0),
                    feagi_npu_neural::types::SynapseType::Excitatory,
                    SYNAPSE_EDGE_ASSOCIATIVE_MEMORY,
                    1,
                )
                .unwrap();
        }
        guard.rebuild_synapse_index();
    }
    for (neuron, episode) in cue_neurons.iter().zip(episodes.clone()) {
        let members = members_of(&rig, episode);
        let from = rig.fired_memory.len();
        let twin_from = rig.fired_twin.len();
        rig.npu
            .lock()
            .unwrap()
            .inject_sensory_with_potentials(&[(NeuronId(*neuron), 10.0)]);
        rig.quiet(members.len() + 8);
        let fires: Vec<u32> = rig.fired_memory[from..].iter().flatten().copied().collect();
        let mut expected = vec![episode];
        expected.extend(members.iter().copied());
        assert_eq!(fires, expected, "E, then its members in list order");
        let (twin, _) = twin_and_memory(&rig, twin_from);
        assert!(
            (twin as i64 - members.len() as i64).abs() <= 1,
            "{} members, {twin} twin frames",
            members.len()
        );
        println!(
            "d: cue of E {episode}: {} members replayed in order, {twin} twin frames",
            members.len()
        );
        rig.quiet(4);
    }
}

#[test]
fn a_one_state_recording_gets_an_episode_neuron() {
    let mut rig = replay_rig(3, 50.0, vec![("capture_max_bursts", json!(200))], false);
    record(&mut rig, &[2, 2, 2]);
    rig.quiet(3);
    let episodes = rig.episodes();
    assert_eq!(
        episodes.len(),
        1,
        "a one-state recording always gets its own E"
    );
    assert_eq!(members_of(&rig, episodes[0]).len(), 1);
    assert!(rig.links().is_empty(), "one state, no links");
    println!("e: one-state recording -> E with 1 member");
}

#[test]
fn expiring_a_member_removes_the_episode_and_the_episodes_expiry_leaves_its_members() {
    // Magnitude 1 keeps the boost small so neurons still age out.
    let mut rig = Rig::new(
        8,
        life(6, 1.0, 1000),
        vec![("capture_max_bursts", json!(200))],
    );
    rig.bind_capture(1.0);
    rig.add_loop(&[]);
    rig.capture_on = true;
    rig.states(&[0, 1, 2]);
    rig.capture_on = false;
    rig.quiet(2);
    let e = rig.episodes()[0];
    let members = members_of(&rig, e);
    assert_eq!(members.len(), 3);
    // Keep the episode and two members alive by re-encoding them; the first member is left to
    // expire.
    for _ in 0..12 {
        rig.step(Some(1));
        rig.step(Some(2));
    }
    assert!(
        !rig.memory_ids().contains(&members[0]),
        "the first member expired"
    );
    assert!(
        !rig.memory_ids().contains(&e),
        "its episode went in the same pass"
    );
    assert!(
        rig.memory_ids().contains(&members[1]),
        "the other members stay"
    );

    // The other direction: the episode expires, its members stay.
    let mut rig = Rig::new(
        8,
        life(6, 1.0, 1000),
        vec![("capture_max_bursts", json!(200))],
    );
    rig.bind_capture(1.0);
    rig.add_loop(&[]);
    rig.capture_on = true;
    rig.states(&[0, 1, 2]);
    rig.capture_on = false;
    rig.quiet(2);
    let e = rig.episodes()[0];
    let members = members_of(&rig, e);
    // The members are re-encoded every third burst (net decay of two per cycle), the episode
    // is not: it expires first, and the members are still there at that moment.
    for _ in 0..30 {
        rig.step(Some(0));
        rig.step(Some(1));
        rig.step(Some(2));
        if !rig.memory_ids().contains(&e) {
            break;
        }
    }
    assert!(!rig.memory_ids().contains(&e), "the episode expired");
    for m in &members {
        assert!(rig.memory_ids().contains(m), "member {m} stays");
    }
    assert!(rig.episodes().is_empty());
    println!(
        "f: member expiry removed E in the same pass; E expiry left {} members",
        members.len()
    );
}

#[test]
fn an_episode_neuron_is_never_a_state_and_a_piece_end_does_not_link_it_into_the_next_piece() {
    // Pieces of 8 bursts while the driver stays on: each piece ends with its own E, whose
    // episodic fire lands while the area is still recording.
    let (mut rig, _) = capturing_rig_with(
        20,
        life(60, 1.0, 1000),
        50.0,
        vec![("capture_max_bursts", json!(8))],
    );
    rig.states(&(0..20).collect::<Vec<_>>());
    rig.quiet(3);
    let episodes = rig.episodes();
    assert!(episodes.len() >= 2, "an episode per piece: {episodes:?}");
    let links = rig.links();
    for link in &links {
        assert!(
            !episodes.contains(&link.source) && !episodes.contains(&link.target),
            "episode neuron in a flagged link: {link:?}"
        );
    }
    // No member list holds an episode neuron either.
    for e in &episodes {
        for m in members_of(&rig, *e) {
            assert!(!episodes.contains(&m));
        }
    }
    // The chain of states is exactly the encoded chain: no E in it.
    let chain = rig.expected_chain();
    assert!(chain.iter().all(|id| !episodes.contains(id)));
    assert_eq!(chain.len(), 20);
    // Boundary links join the last state of a piece to the first of the next, not E.
    let info = rig.service.capture_area_info(rig.mem_idx).unwrap();
    assert!(info.boundary_links >= 1);
    // E's own fire while recording is not counted as a blocked replay: only the 20 encoding
    // fires are (the stage 3 count).
    let blocked = rig
        .npu
        .lock()
        .unwrap()
        .memory_replay_blocked_counts_for_area(rig.mem_idx);
    assert_eq!(
        blocked.1, 20,
        "an episode neuron's own fire is not a blocked replay"
    );
    // The runtime info the API serves: recording state, episode count, blocked-replay counts.
    let runtime = rig.service.memory_cortical_area_runtime_info(rig.mem_idx);
    assert!(runtime.recording, "the area is recording");
    assert_eq!(runtime.episode_neuron_count, episodes.len());
    assert_eq!(runtime.replay_blocked_by_recording, 20);
    assert_eq!(runtime.replay_blocked_by_lockout, 0);
    println!(
        "g: {} episodes, {} links, none touches an episode neuron; boundary links {}",
        episodes.len(),
        links.len(),
        info.boundary_links
    );
}

#[test]
fn creating_an_episode_neuron_at_the_cap_never_evicts_its_members_and_skips_when_full() {
    // Cap 4 with four states: no room for E without evicting a member.
    let mut rig = replay_rig(
        8,
        50.0,
        vec![
            ("capture_max_bursts", json!(200)),
            ("memory_neuron_cap", json!(4)),
        ],
        false,
    );
    record(&mut rig, &[0, 1, 2, 3]);
    rig.quiet(3);
    assert!(rig.episodes().is_empty(), "no room: E skipped");
    let chain = rig.expected_chain();
    for id in &chain {
        assert!(rig.memory_ids().contains(id), "member {id} was not evicted");
    }
    let info = rig.service.capture_area_info(rig.mem_idx).unwrap();
    assert_eq!(info.episodes_skipped, 1);

    // Cap 5: there is room for E, and nothing is evicted.
    let mut rig = replay_rig(
        8,
        50.0,
        vec![
            ("capture_max_bursts", json!(200)),
            ("memory_neuron_cap", json!(5)),
        ],
        false,
    );
    record(&mut rig, &[0, 1, 2, 3]);
    rig.quiet(3);
    assert_eq!(rig.episodes().len(), 1);
    assert_eq!(rig.memory_ids().len(), 5);
    println!("i: cap 4 -> E skipped and logged, 4 members intact; cap 5 -> E made, 5 neurons");
}

#[test]
fn an_e_started_replay_follows_its_list_exactly_even_through_a_repeated_state() {
    let mut rig = replay_rig(4, 50.0, vec![("capture_max_bursts", json!(200))], false);
    // A B A C: a state-cued replay stops at the visited state; the episode's list does not.
    record(&mut rig, &[0, 1, 0, 2]);
    rig.quiet(3);
    let e = rig.episodes()[0];
    let members = members_of(&rig, e);
    assert_eq!(
        members.len(),
        4,
        "the list keeps the repeated state: {members:?}"
    );
    assert_eq!(members[0], members[2]);
    open_gate(&mut rig);
    let from = rig.fired_memory.len();
    rig.cue(e);
    rig.quiet(14);
    let fires: Vec<u32> = rig.fired_memory[from..].iter().flatten().copied().collect();
    let mut expected = vec![e];
    expected.extend(members.iter().copied());
    assert_eq!(fires, expected, "E, then A B A C exactly");
    println!(
        "c2: E-started replay through a repeated state = {} fires",
        fires.len()
    );
}

// ---- Q1 path tracking: a state-cued replay that completes an episode's chain fires its E ----

fn track_ledger(rig: &Rig) {
    rig.npu
        .lock()
        .unwrap()
        .configure_fire_ledger_window(rig.mem_idx, 128)
        .expect("track the memory area on the fire ledger");
}

/// Fires of `e` from burst `from` on, `(burst, kind)`.
fn e_fires_from(rig: &Rig, e: u32, from: u64) -> Vec<(u64, u8)> {
    fire_kinds_of(rig, e)
        .into_iter()
        .filter(|(t, _)| *t >= from)
        .collect()
}

/// Cue `state` with the gate open and let the replay run. Returns the cue burst, the bursts at
/// which each memory neuron fired (`(burst, id)`) and the twin frames it produced.
fn cue_and_run(rig: &mut Rig, state: u32, bursts: usize) -> (u64, Vec<(u64, u32)>, usize) {
    open_gate(rig);
    let cue_burst = rig.burst + 1;
    let from = rig.fired_memory.len();
    let twin_from = rig.fired_twin.len();
    rig.cue(state);
    rig.quiet(bursts);
    let mut fires = Vec::new();
    for (k, ids) in rig.fired_memory[from..].iter().enumerate() {
        for id in ids {
            fires.push((cue_burst + k as u64, *id));
        }
    }
    let twin: usize = rig.fired_twin[twin_from..].iter().sum();
    (cue_burst, fires, twin)
}

#[test]
fn q1_a_cue_at_state_one_fires_e_once_at_the_last_state_and_starts_no_chain() {
    let mut rig = replay_rig(6, 50.0, vec![("capture_max_bursts", json!(200))], false);
    track_ledger(&rig);
    record(&mut rig, &[0, 1, 2, 3]);
    rig.quiet(3);
    let e = rig.episodes()[0];
    let members = members_of(&rig, e);
    let last = *members.last().unwrap();
    let (cue_burst, fires, twin) = cue_and_run(&mut rig, members[0], 20);
    let last_burst = fires.iter().find(|(_, id)| *id == last).unwrap().0;
    let e_fires = e_fires_from(&rig, e, cue_burst);
    assert_eq!(e_fires.len(), 1, "E fires once: {e_fires:?}");
    assert_eq!(e_fires[0], (last_burst, FIRE_KIND_CHAIN_RECALL));
    // The E fire starts nothing: the states play once each, with their frames.
    let states: Vec<u32> = fires.iter().map(|(_, id)| *id).filter(|id| *id != e).collect();
    assert_eq!(states, members, "the states play once, in order");
    assert_eq!(twin, members.len(), "no extra replay frames after E");
    println!(
        "q1b: E fired at burst {} (last state burst {}), kind {}, {} twin frames",
        e_fires[0].0, last_burst, e_fires[0].1, twin
    );
}

#[test]
fn q1_a_cue_at_state_two_fires_e_zero_times() {
    let mut rig = replay_rig(6, 50.0, vec![("capture_max_bursts", json!(200))], false);
    track_ledger(&rig);
    record(&mut rig, &[0, 1, 2, 3]);
    rig.quiet(3);
    let e = rig.episodes()[0];
    let members = members_of(&rig, e);
    let (cue_burst, fires, _) = cue_and_run(&mut rig, members[1], 20);
    assert!(fires.iter().any(|(_, id)| *id == members[3]), "the tail played");
    let e_fires = e_fires_from(&rig, e, cue_burst);
    assert_eq!(e_fires.len(), 0, "a partial run does not fire E: {e_fires:?}");
    println!("q1c: cue at state 2 -> {} E fires", e_fires.len());
}

#[test]
fn q1_an_e_started_replay_fires_e_once_in_total() {
    let mut rig = replay_rig(6, 50.0, vec![("capture_max_bursts", json!(200))], false);
    track_ledger(&rig);
    record(&mut rig, &[0, 1, 2, 3]);
    rig.quiet(3);
    let e = rig.episodes()[0];
    let (cue_burst, _, _) = cue_and_run(&mut rig, e, 20);
    let e_fires = e_fires_from(&rig, e, cue_burst);
    assert_eq!(e_fires.len(), 1, "once, at the start: {e_fires:?}");
    assert_eq!(e_fires[0].1, FIRE_KIND_CHAIN_RECALL);
    println!("q1d: E-started replay -> {} E fire (burst {})", e_fires.len(), e_fires[0].0);
}

#[test]
fn q1_a_closed_loop_gate_fires_no_e_not_even_a_one_state_one() {
    let mut rig = replay_rig(6, 50.0, vec![("capture_max_bursts", json!(200))], false);
    track_ledger(&rig);
    record(&mut rig, &[0]);
    record(&mut rig, &[1, 2, 3]);
    rig.quiet(3);
    let eps = rig.episodes();
    assert_eq!(eps.len(), 2);
    let mut counts = Vec::new();
    for e in eps {
        let first = members_of(&rig, e)[0];
        // The gate stays closed: no open_gate.
        let cue_burst = rig.burst + 1;
        rig.cue(first);
        rig.quiet(20);
        counts.push(e_fires_from(&rig, e, cue_burst).len());
    }
    assert_eq!(counts, vec![0, 0], "closed gate: no E fires");
    // Control: with the gate open the one-state episode fires.
    let one = rig
        .episodes()
        .into_iter()
        .find(|e| members_of(&rig, *e).len() == 1)
        .unwrap();
    let one_state = members_of(&rig, one)[0];
    let (cue_burst, _, _) = cue_and_run(&mut rig, one_state, 6);
    let open = e_fires_from(&rig, one, cue_burst);
    assert_eq!(open.len(), 1, "open gate: the one-state E fires: {open:?}");
    println!("q1e: closed gate E fires {counts:?}; open-gate control {}", open.len());
}

#[test]
fn q1_with_a_shared_first_state_only_the_chosen_path_fires_its_e() {
    let (mut rig, [a, b, c]) = shared_state_rig(50.0, 50.0);
    track_ledger(&rig);
    let eps = rig.episodes();
    assert_eq!(eps.len(), 2);
    let e_of = |rig: &Rig, second: u32| {
        *eps.iter()
            .find(|e| members_of(rig, **e)[1] == second)
            .unwrap()
    };
    let (ex, ey) = (e_of(&rig, b), e_of(&rig, c));
    let (cue_burst, fires, _) = cue_and_run(&mut rig, a, 20);
    let picked = fires.iter().map(|(_, id)| *id).find(|id| *id == b || *id == c).unwrap();
    let (winner, loser) = if picked == b { (ex, ey) } else { (ey, ex) };
    let w = e_fires_from(&rig, winner, cue_burst);
    let l = e_fires_from(&rig, loser, cue_burst);
    assert_eq!((w.len(), l.len()), (1, 0), "winner {w:?} loser {l:?}");
    println!("q1f: D12 picked {}, winner E fired {}, other {}", if picked == b { "X" } else { "Y" }, w.len(), l.len());
}

#[test]
fn q1_the_index_is_rebuilt_after_a_service_export_and_import() {
    let mut rig = replay_rig(6, 100.0, vec![("capture_max_bursts", json!(200))], false);
    track_ledger(&rig);
    record(&mut rig, &[0, 1, 2]);
    rig.quiet(3);
    let e = rig.episodes()[0];
    let members = members_of(&rig, e);
    let exported = rig.service.export_long_term_memory_neurons();
    assert!(exported.iter().any(|d| d.neuron_id == e), "E is long term");
    // Drop the NPU-side tables, then restore: the index must come back with the lists.
    rig.npu.lock().unwrap().set_episode_members(Vec::new(), &[e]);
    assert!(rig.npu.lock().unwrap().episode_members_known().is_empty());
    rig.service
        .restore_long_term_memory_neurons(&exported)
        .expect("restore");
    assert_eq!(rig.npu.lock().unwrap().episode_members_known().len(), 1);
    rig.quiet(2);
    let (cue_burst, fires, _) = cue_and_run(&mut rig, members[0], 20);
    assert!(fires.iter().any(|(_, id)| *id == members[2]));
    let e_fires = e_fires_from(&rig, e, cue_burst);
    assert_eq!(e_fires.len(), 1, "after reload: {e_fires:?}");
    println!("q1g: after export/import E fires {} (kind {})", e_fires.len(), e_fires[0].1);
}
