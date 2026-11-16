// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Regression baseline for normal (windowed and associative) memory areas.
//!
//! Each rig drives a real `DynamicNPU` (built through a real `ConnectomeManager`) and a real
//! `PlasticityService`, burst by burst and on one thread: stimulus, `process_burst`,
//! `PlasticityService::run_burst_sync`, drain the queued commands, apply them. Every burst is
//! recorded as sorted sets (never spike order). The traces are compared with the checked-in
//! fixture `tests/fixtures/memory-regression/golden-traces.json`; every later memory change
//! must leave that file byte-identical.
//!
//! Regenerate the fixture:
//!
//! ```text
//! FEAGI_CORE_COMMIT=$(git rev-parse HEAD) \
//! FEAGI_WRITE_MEMORY_REGRESSION_FIXTURES=<feagi-core>/tests/fixtures/memory-regression \
//! cargo test --test memory_regression_golden
//! ```
//!
//! `feagi_core_commit` is a note only and is ignored by the comparison.
//!
//! Rigs (one memory area each, because the service keeps its areas in a `std` `HashMap`):
//! `window_depth1` (create, reactivate, age, expire with synapse pruning, STM to LTM, mapping
//! edit), `window_depth3`, `cap_lru`, `formation_blocked` (`memory.formation` -100),
//! `learning_freeze` (`memory.freeze`), `assoc_stdp` (associative STDP, recall through the
//! sparse memory LIF, replay to the twin with both `replay_trigger` modes), `assoc_rstdp`
//! (reward-gated associative STDP), `mp_differential`, `shape_purge`, `save_reload`.
//!
//! What a trace row holds. `fired`: area label to the sorted ids that fired in that burst.
//! `memory`: per memory area, the live neuron count and each neuron's lifecycle state.
//! `commands`: the plasticity commands the burst queued, sorted. `synapses`: the full sorted
//! synapse set of the rig's mappings, present only on bursts where it changed. `events`: what
//! the test itself did before the burst. Regular neurons are labelled by area and position;
//! memory neurons are numbered `m1`, `m2`, ... by first appearance.
//!
//! The fire kind of each fired neuron lives in a separate fixture, `fire-kinds.json`: per burst
//! and per Fire-Ledger-tracked area, the sorted `id:kind` pairs the ledger stored (read through
//! `get_fire_ledger_recent_frames_with_kinds` right after the burst, before plasticity commands
//! run). While generating, the stored kind of every area on the episodic ledger is checked
//! against what the old episodic-ledger join gave. The golden traces above never hold kinds.
//! Regenerate it with `FEAGI_WRITE_FIRE_KIND_FIXTURE=<dir>` (the same way, a separate
//! variable so the golden traces are never rewritten by accident).
//!
//! Not covered, by design:
//! - The production command applier and the twin bridge are
//!   `feagi_services::plasticity_runtime::PlasticityRuntime`; the rigs run its
//!   `apply_post_burst` after each burst.

use ahash::AHashMap;
use feagi_brain_development::ConnectomeManager;
use feagi_services::plasticity_runtime::PlasticityRuntime;
use parking_lot::RwLock;
use std::sync::Mutex;
use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::modulator_engine::ModulatorBinding;
use feagi_npu_burst_engine::{
    DynamicNPU, TracingMutex, FIRE_KIND_EPISODIC_MEMORY, FIRE_KIND_STDP_ELIGIBLE,
};
use feagi_npu_neural::types::{NeuronId, SynapseType, SynapticPsp, SynapticWeight};
use feagi_npu_plasticity::{
    AsyncPlasticityExecutor, LtmEviction, MemoryAreaShape, MemoryMpMode, MemoryNeuronLifecycleConfig, MpChangeEncoding,
    PlasticityCommand, PlasticityConfig, PlasticityService,
};
use feagi_npu_runtime::StdRuntime;
use feagi_structures::genomic::cortical_area::{
    CorticalArea, CorticalAreaDimensions, CorticalAreaType, CorticalID,
    IOCorticalAreaConfigurationFlag, MemoryCorticalType,
};
use feagi_structures::genomic::ModulatorKind;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::sync::Arc;

const FILE_NAME: &str = "golden-traces.json";
const KINDS_FILE_NAME: &str = "fire-kinds.json";
const FIXTURE_DIR: &str = "tests/fixtures/memory-regression";
const MEMORY_ID_START: u32 = 50_000_000;
/// Distinct upstream patterns: neuron positions that fire together.
const PAT_A: [usize; 2] = [0, 1];
const PAT_B: [usize; 2] = [2, 3];
const PAT_C: [usize; 2] = [0, 2];
const PAT_D: [usize; 2] = [1, 3];

type SharedNpu = Arc<TracingMutex<DynamicNPU>>;

fn lifecycle(
    initial: u32,
    growth: f32,
    longterm: u32,
    cap: u32,
    eviction: LtmEviction,
) -> MemoryNeuronLifecycleConfig {
    MemoryNeuronLifecycleConfig {
        initial_lifespan: initial,
        lifespan_growth_rate: growth,
        longterm_threshold: longterm,
        max_reactivations: 1000,
        memory_neuron_cap: cap,
        ltm_eviction: eviction,
    }
}

struct Spec {
    name: &'static str,
    depth: u32,
    lifecycle: MemoryNeuronLifecycleConfig,
    mp: MemoryMpMode,
    /// Upstream neurons (one row of an `n x 1 x 1` area).
    upstream_neurons: u32,
    /// Extra memory-area properties.
    mem_props: Vec<(&'static str, Value)>,
}

struct Rig {
    spec_name: &'static str,
    npu: SharedNpu,
    manager: Arc<RwLock<ConnectomeManager>>,
    service: PlasticityService,
    runtime: PlasticityRuntime,
    mem_idx: u32,
    mem_id: CorticalID,
    up_id: CorticalID,
    up: Vec<u32>,
    area_labels: BTreeMap<u32, String>,
    /// Regular areas whose outgoing synapses are recorded.
    tracked: Vec<u32>,
    labels: HashMap<u32, String>,
    next_memory_label: usize,
    notes: Vec<String>,
    last_synapses: Option<Vec<String>>,
    bursts: Vec<Value>,
    /// Per burst, the stored fire kind of every fired id of each ledger-tracked area.
    kind_trace: Vec<Value>,
    index: u64,
    /// Stimulus added to every burst while set (a driver held on).
    hold: Vec<(u32, f32)>,
    aux: Vec<u32>,
}

fn build_npu(tag: &'static str) -> SharedNpu {
    Arc::new(TracingMutex::new(
        DynamicNPU::new_f32(StdRuntime::new(), CPUBackend::new(), 4096, 4096, 8)
            .expect("npu creation"),
        tag,
    ))
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
        MemoryMpMode::Change(MpChangeEncoding::Ratio { quantization_percent }) => {
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

impl Rig {
    fn new(spec: Spec) -> Rig {
        let npu = build_npu(spec.name);
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

        let (up_id, up_idx) = regular_area(&mut manager, b"csrc0001", "up", spec.upstream_neurons);

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
            .insert("temporal_depth".to_string(), json!(spec.depth));
        for (key, value) in lifecycle_properties(&spec.lifecycle, &spec.mp) {
            mem_area.properties.insert(key.to_string(), value);
        }
        for (key, value) in &spec.mem_props {
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
            .and_then(|twin| manager.get_cortical_idx(&twin));

        // Registered by the manager through the linked executor, from the area's properties,
        // as in production (the mapping above is the registration trigger).
        assert!(
            service.registered_memory_area_indexes().contains(&mem_idx),
            "memory area not registered for {}",
            spec.name
        );

        let (_, aux_idx) = regular_area(&mut manager, b"caux0001", "aux", 1);
        let manager = Arc::new(RwLock::new(manager));
        let runtime = PlasticityRuntime::new(Arc::clone(&npu), Arc::clone(&manager), executor);
        let up = npu.lock().unwrap().get_neurons_in_cortical_area(up_idx);
        let aux = npu.lock().unwrap().get_neurons_in_cortical_area(aux_idx);
        let mut rig = Rig {
            spec_name: spec.name,
            npu,
            manager,
            service,
            runtime,
            mem_idx,
            mem_id,
            up_id,
            up,
            area_labels: BTreeMap::new(),
            tracked: vec![up_idx],
            labels: HashMap::new(),
            next_memory_label: 1,
            notes: Vec::new(),
            last_synapses: None,
            bursts: Vec::new(),
            kind_trace: Vec::new(),
            index: 0,
            hold: Vec::new(),
            aux,
        };
        rig.name_area(up_idx, "up");
        rig.name_area(aux_idx, "aux");
        rig.tracked.push(aux_idx);
        rig.name_area(mem_idx, "mem");
        if let Some(twin) = twin_idx {
            rig.name_area(twin, "twin");
            rig.tracked.push(twin);
        }
        rig
    }

    /// Label an area and, for a regular area, its neurons (`<label><position>`).
    fn name_area(&mut self, idx: u32, label: &str) {
        self.area_labels.insert(idx, label.to_string());
        if idx == self.mem_idx {
            return;
        }
        let ids = self.npu.lock().unwrap().get_neurons_in_cortical_area(idx);
        for (position, id) in ids.iter().enumerate() {
            self.labels.insert(*id, format!("{label}{position}"));
        }
    }

    fn add_regular_area(&mut self, id: &[u8; 8], label: &str, neurons: u32) -> (CorticalID, u32) {
        let (cortical_id, idx) = regular_area(&mut self.manager.write(), id, label, neurons);
        self.name_area(idx, label);
        (cortical_id, idx)
    }

    fn note(&mut self, text: &str) {
        self.notes.push(text.to_string());
    }

    /// Stimulus helper: fire upstream neurons at `positions` with `potential`.
    fn up_stim(&self, positions: &[usize], potential: f32) -> Vec<(u32, f32)> {
        positions.iter().map(|p| (self.up[*p], potential)).collect()
    }

    /// Stimulus helper: fire the neurons of an area at `positions`.
    fn area_stim(&self, area_idx: u32, positions: &[usize], potential: f32) -> Vec<(u32, f32)> {
        let ids = self
            .npu
            .lock()
            .unwrap()
            .get_neurons_in_cortical_area(area_idx);
        positions.iter().map(|p| (ids[*p], potential)).collect()
    }

    fn memory_ids(&self) -> Vec<u32> {
        let array = self.service.get_memory_neuron_array();
        let array = array.lock().unwrap();
        let mut ids = array.get_active_neurons_by_area(self.mem_idx);
        ids.sort_unstable();
        ids
    }

    /// Give a memory neuron the footprint a consumer builds: weak synapses in and out, to an
    /// area that is not upstream of the memory area (so they never drive new patterns).
    fn attach_footprint(&mut self, memory_neuron: u32) {
        let assoc = feagi_npu_neural::synapse::SYNAPSE_EDGE_ASSOCIATIVE_MEMORY;
        let mut guard = self.npu.lock().unwrap();
        guard
            .add_synapse(
                NeuronId(self.aux[0]),
                NeuronId(memory_neuron),
                SynapticWeight(9.0),
                SynapticPsp(0.1),
                SynapseType::Excitatory,
                assoc,
                1,
            )
            .unwrap();
        guard
            .add_synapse(
                NeuronId(memory_neuron),
                NeuronId(self.aux[0]),
                SynapticWeight(7.0),
                SynapticPsp(0.1),
                SynapseType::Excitatory,
                assoc,
                1,
            )
            .unwrap();
        guard.rebuild_synapse_index();
        drop(guard);
        self.note("attach footprint synapses to the newest memory neuron");
    }

    /// Assign `m<N>` labels to unseen memory ids, ascending, so labels never depend on set order.
    fn observe(&mut self, ids: impl IntoIterator<Item = u32>) {
        let unseen: BTreeSet<u32> = ids
            .into_iter()
            .filter(|id| *id >= MEMORY_ID_START && !self.labels.contains_key(id))
            .collect();
        for id in unseen {
            let label = format!("m{}", self.next_memory_label);
            self.next_memory_label += 1;
            self.labels.insert(id, label);
        }
    }

    fn label(&self, id: u32) -> String {
        self.labels
            .get(&id)
            .cloned()
            .unwrap_or_else(|| format!("n{id}"))
    }

    fn command_ids(commands: &[PlasticityCommand]) -> Vec<u32> {
        commands
            .iter()
            .filter_map(|c| match c {
                PlasticityCommand::RegisterMemoryNeuron { neuron_id, .. }
                | PlasticityCommand::MemoryNeuronConvertedToLtm { neuron_id, .. }
                | PlasticityCommand::InjectMemoryNeuronToFCL { neuron_id, .. }
                | PlasticityCommand::MemoryNeuronRemoved { neuron_id, .. } => Some(*neuron_id),
                _ => None,
            })
            .collect()
    }

    fn describe_command(&self, command: &PlasticityCommand) -> Option<String> {
        match command {
            PlasticityCommand::RegisterMemoryNeuron { neuron_id, .. } => {
                Some(format!("register {}", self.label(*neuron_id)))
            }
            PlasticityCommand::MemoryNeuronConvertedToLtm { neuron_id, .. } => {
                Some(format!("to_ltm {}", self.label(*neuron_id)))
            }
            PlasticityCommand::InjectMemoryNeuronToFCL {
                neuron_id,
                is_reactivation,
                replay_frames,
                ..
            } => Some(format!(
                "inject {} reactivation={} replay_frames={}",
                self.label(*neuron_id),
                is_reactivation,
                replay_frames.len()
            )),
            PlasticityCommand::MemoryNeuronRemoved {
                neuron_id, reason, ..
            } => Some(format!("removed {} {:?}", self.label(*neuron_id), reason)),
            PlasticityCommand::ResetMemoryNeuronsInArea { cortical_idx } => {
                Some(format!("reset_area {cortical_idx}"))
            }
            PlasticityCommand::UpdateWeightsDelta {
                synapse_indices, ..
            } => Some(format!("weights_delta n={}", synapse_indices.len())),
            PlasticityCommand::UpdateStateCounters { .. } => None,
        }
    }

    /// Every synapse leaving a tracked area's neurons or a memory neuron, sorted.
    fn synapse_set(&self) -> Vec<String> {
        let guard = self.npu.lock().unwrap();
        let mut sources: Vec<u32> = Vec::new();
        for area in &self.tracked {
            sources.extend(guard.get_neurons_in_cortical_area(*area));
        }
        sources.extend(self.memory_ids());
        let mut rows = BTreeSet::new();
        for source in sources {
            for (target, weight, psp, kind, flags, group) in
                guard.get_outgoing_synapses_detailed(source)
            {
                rows.insert((
                    source,
                    target,
                    weight.to_bits(),
                    psp.to_bits(),
                    kind,
                    flags,
                    group,
                ));
            }
        }
        drop(guard);
        let mut out: Vec<String> = rows
            .into_iter()
            .map(|(s, t, w, p, k, f, g)| {
                format!(
                    "{}>{} w={} psp={} type={} flags={} group={}",
                    self.label(s),
                    self.label(t),
                    f32::from_bits(w),
                    f32::from_bits(p),
                    k,
                    f,
                    g
                )
            })
            .collect();
        out.sort();
        out
    }

    /// The Fire Ledger's stored fire kind of every id that fired in `burst`, per tracked area
    /// (sorted by id). Panics when an area on the episodic ledger disagrees with the old join:
    /// episodic exactly when the id is in the episodic ledger's frame for that burst.
    fn read_stored_kinds(&self, burst: u64) -> BTreeMap<u32, Vec<(u32, u8)>> {
        let guard = self.npu.lock().unwrap();
        let episodic_areas: BTreeSet<u32> = guard
            .get_all_episodic_memory_fire_ledger_configs()
            .into_iter()
            .map(|(idx, _)| idx)
            .collect();
        let mut out = BTreeMap::new();
        for (area, window) in guard.get_all_fire_ledger_configs() {
            let frames = guard
                .get_fire_ledger_recent_frames_with_kinds(area, window)
                .expect("ledger frames with kinds");
            let Some(frame) = frames.iter().find(|f| f.timestep == burst) else {
                continue;
            };
            let pairs: Vec<(u32, u8)> = frame
                .fired
                .iter()
                .map(|id| (id, frame.kind_of(id)))
                .collect();
            if episodic_areas.contains(&area) {
                let episodic: BTreeSet<u32> = guard
                    .get_episodic_memory_fire_ledger_recent_frames(area, window)
                    .expect("episodic ledger frames")
                    .into_iter()
                    .filter(|(t, _)| *t == burst)
                    .flat_map(|(_, bitmap)| bitmap.into_iter())
                    .collect();
                for (id, kind) in &pairs {
                    let joined = if episodic.contains(id) {
                        FIRE_KIND_EPISODIC_MEMORY
                    } else {
                        FIRE_KIND_STDP_ELIGIBLE
                    };
                    assert_eq!(
                        *kind, joined,
                        "rig {} burst {burst} area {area} id {id}: stored kind {kind} != old join {joined}",
                        self.spec_name
                    );
                }
            }
            out.insert(area, pairs);
        }
        out
    }

    /// One burst: stimulus, NPU burst, one synchronous plasticity pass, apply the commands.
    fn step(&mut self, note: &str, stim: &[(u32, f32)]) {
        let result = {
            let mut guard = self.npu.lock().unwrap();
            let mut all: Vec<(u32, f32)> = stim.to_vec();
            all.extend(self.hold.iter().copied());
            if !all.is_empty() {
                let injected: Vec<(NeuronId, f32)> =
                    all.iter().map(|(id, p)| (NeuronId(*id), *p)).collect();
                guard.inject_sensory_with_potentials(&injected);
            }
            guard.process_burst().expect("burst")
        };
        let stored_kinds = self.read_stored_kinds(result.burst);
        self.service.run_burst_sync(result.burst);
        // Production order: the post-burst callback drains and applies the queue after the NPU
        // lock is released. The rig reads the queue first for the trace, then puts it back.
        let commands = self.service.drain_commands();
        self.service.enqueue_commands_for_test(commands.clone());
        self.runtime.apply_post_burst(result.burst);

        let mut fired_by_area: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        if let Some(sample) = &result.fire_queue_sample {
            for (area, (ids, ..)) in sample.iter() {
                let mut ids = ids.clone();
                ids.sort_unstable();
                fired_by_area.insert(*area, ids);
            }
        }
        let memory_ids = self.memory_ids();
        let synapse_endpoints: Vec<u32> = {
            // Labels for memory ids that only appear as synapse endpoints.
            let guard = self.npu.lock().unwrap();
            let mut ids = Vec::new();
            for id in &memory_ids {
                for (target, ..) in guard.get_outgoing_synapses_detailed(*id) {
                    ids.push(target);
                }
                for (source, ..) in guard.get_incoming_synapses_detailed(*id) {
                    ids.push(source);
                }
            }
            ids
        };
        let mut seen: Vec<u32> = Self::command_ids(&commands);
        seen.extend(fired_by_area.values().flatten().copied());
        seen.extend(memory_ids.iter().copied());
        seen.extend(synapse_endpoints);
        self.observe(seen);
        // Labels exist now, so the synapse set can be written with them.
        let synapses_labelled = self.synapse_set();

        let mut fired = serde_json::Map::new();
        for (area, ids) in &fired_by_area {
            let area_label = self
                .area_labels
                .get(area)
                .cloned()
                .unwrap_or_else(|| format!("area{area}"));
            let labelled: Vec<String> = ids.iter().map(|id| self.label(*id)).collect();
            fired.insert(area_label, json!(labelled));
        }

        let mut memory_state = Vec::new();
        {
            let array = self.service.get_memory_neuron_array();
            let array = array.lock().unwrap();
            for id in &memory_ids {
                let detail = array.get_memory_neuron_detail(*id).expect("detail");
                memory_state.push(format!(
                    "{} long_term={} lifespan={} activations={}",
                    self.label(*id),
                    detail.is_longterm_memory,
                    detail.lifespan_current,
                    detail.activation_count
                ));
            }
        }
        memory_state.sort();

        let mut command_text: Vec<String> = commands
            .iter()
            .filter_map(|c| self.describe_command(c))
            .collect();
        command_text.sort();

        let changed = self.last_synapses.as_ref() != Some(&synapses_labelled);
        let synapse_value = if changed {
            self.last_synapses = Some(synapses_labelled.clone());
            json!(synapses_labelled)
        } else {
            Value::Null
        };

        let mut kinds_by_area = serde_json::Map::new();
        for (area, pairs) in &stored_kinds {
            let area_label = self
                .area_labels
                .get(area)
                .cloned()
                .unwrap_or_else(|| format!("area{area}"));
            let labelled: Vec<String> = pairs
                .iter()
                .map(|(id, kind)| format!("{}:{kind}", self.label(*id)))
                .collect();
            kinds_by_area.insert(area_label, json!(labelled));
        }
        self.kind_trace.push(json!({
            "burst": self.index,
            "kinds": Value::Object(kinds_by_area),
        }));

        let events = std::mem::take(&mut self.notes);
        self.bursts.push(json!({
            "burst": self.index,
            "input": note,
            "events": events,
            "fired": Value::Object(fired),
            "memory_neurons": memory_state.len(),
            "memory": memory_state,
            "commands": command_text,
            "synapses": synapse_value,
        }));
        self.index += 1;
    }

    fn quiet(&mut self, count: usize) {
        for _ in 0..count {
            self.step("quiet", &[]);
        }
    }

    fn finish(self, settings: Value) -> Value {
        json!({
            "name": self.spec_name,
            "settings": settings,
            "bursts": self.bursts,
            "fire_kinds": self.kind_trace,
        })
    }

    fn pattern(&mut self, name: &str, positions: &[usize]) {
        let stim = self.up_stim(positions, 10.0);
        self.step(name, &stim);
    }

    fn set_recall_only(&mut self, recall_only: bool) {
        // As production does: set the area property, then sync it into the NPU params.
        {
            let mut manager = self.manager.write();
            let area = manager
                .get_cortical_area_mut(&self.mem_id)
                .expect("memory area");
            area.properties.insert(
                "replay_trigger".to_string(),
                json!(if recall_only { "recall_only" } else { "any_fire" }),
            );
            manager.sync_memory_associative_lif_params(&self.mem_id);
        }
        self.note(if recall_only {
            "replay_trigger = recall_only"
        } else {
            "replay_trigger = any_fire"
        });
    }

    /// Bind one modulator instance driven by `driver_idx` and subscribe the memory area to it.
    fn bind_memory_modulator(
        &mut self,
        id: &str,
        kind: ModulatorKind,
        magnitude: f32,
        driver_idx: u32,
    ) {
        let guard = self.npu.lock().unwrap();
        guard.set_modulator_bindings(vec![ModulatorBinding {
            instance_id: id.to_string(),
            kind,
            magnitude_percent: magnitude,
            graded: false,
            full_scale_potential: 0.0,
            driver_cortical_idx: driver_idx,
        }]);
        let mut subs = AHashMap::new();
        subs.insert(self.mem_idx, vec![id.to_string()]);
        guard.set_modulator_area_subscriptions(subs);
        drop(guard);
        self.note(&format!("bind {id} ({kind:?}) to the memory area"));
    }
}

fn settings(rig_lifecycle: &MemoryNeuronLifecycleConfig, depth: u32, extra: Value) -> Value {
    json!({
        "temporal_depth": depth,
        "initial_lifespan": rig_lifecycle.initial_lifespan,
        "lifespan_growth_rate": rig_lifecycle.lifespan_growth_rate,
        "longterm_threshold": rig_lifecycle.longterm_threshold,
        "memory_neuron_cap": rig_lifecycle.memory_neuron_cap,
        "ltm_eviction": format!("{:?}", rig_lifecycle.ltm_eviction),
        "excitability": 1,
        "extra": extra,
    })
}

fn spec(
    name: &'static str,
    depth: u32,
    lifecycle: MemoryNeuronLifecycleConfig,
    mp: MemoryMpMode,
    upstream_neurons: u32,
) -> Spec {
    Spec {
        name,
        depth,
        lifecycle,
        mp,
        upstream_neurons,
        mem_props: Vec::new(),
    }
}

// ---------------------------------------------------------------------------------------------
// Rigs
// ---------------------------------------------------------------------------------------------

/// Depth 1: create, reactivate, age, expire (pruning a footprint), STM to LTM, replay to the
/// twin with the default trigger, and a mapping edit regenerated in place.
fn rig_window_depth1() -> Value {
    let life = lifecycle(4, 3.0, 7, 0, LtmEviction::Lru);
    let mut rig = Rig::new(spec("window_depth1", 1, life, MemoryMpMode::PatternOnly, 4));
    rig.quiet(1);
    rig.pattern("A", &PAT_A);
    rig.quiet(1);
    rig.pattern("A", &PAT_A);
    rig.quiet(1);
    rig.pattern("A", &PAT_A);
    rig.quiet(2);
    rig.pattern("B", &PAT_B);
    let newest = *rig.memory_ids().last().unwrap();
    rig.attach_footprint(newest);
    rig.quiet(7);
    rig.pattern("A", &PAT_A);
    rig.quiet(2);
    rig.note("edit the up to mem mapping (psp multiplier 2.0) and regenerate its synapses");
    let (up_id, mem_id) = (rig.up_id, rig.mem_id);
    rig.manager
        .write()
        .update_cortical_mapping(
            &up_id,
            &mem_id,
            vec![json!({
                "morphology_id": "episodic_memory",
                "morphology_scalar": 1,
                "postSynapticCurrent_multiplier": 2.0,
            })],
        )
        .unwrap();
    rig.manager
        .write()
        .regenerate_synapses_for_mapping(&up_id, &mem_id)
        .unwrap();
    rig.pattern("C", &PAT_C);
    rig.quiet(2);
    rig.pattern("A", &PAT_A);
    rig.quiet(2);
    rig.finish(settings(
        &life,
        1,
        json!({"stimuli": "A,B,C = 2-neuron patterns"}),
    ))
}

/// Depth 3: a window of three bursts forms one memory; reactivation and a different window.
fn rig_window_depth3() -> Value {
    let life = lifecycle(6, 3.0, 100, 0, LtmEviction::Lru);
    let mut rig = Rig::new(spec("window_depth3", 3, life, MemoryMpMode::PatternOnly, 4));
    rig.quiet(2);
    for _ in 0..2 {
        rig.pattern("A", &PAT_A);
        rig.pattern("B", &PAT_B);
        rig.pattern("C", &PAT_C);
        rig.quiet(3);
    }
    rig.pattern("B", &PAT_B);
    rig.pattern("A", &PAT_A);
    rig.pattern("D", &PAT_D);
    rig.quiet(4);
    rig.finish(settings(&life, 3, json!({})))
}

/// Cap of two with `lru` eviction: the least recently used neuron goes, with its footprint.
fn rig_cap_lru() -> Value {
    let life = lifecycle(40, 1.0, 1000, 2, LtmEviction::Lru);
    let mut rig = Rig::new(spec("cap_lru", 1, life, MemoryMpMode::PatternOnly, 4));
    rig.pattern("A", &PAT_A);
    let first = *rig.memory_ids().last().unwrap();
    rig.attach_footprint(first);
    rig.quiet(1);
    rig.pattern("B", &PAT_B);
    rig.quiet(1);
    rig.pattern("A", &PAT_A);
    rig.quiet(1);
    rig.pattern("C", &PAT_C);
    rig.quiet(1);
    rig.pattern("D", &PAT_D);
    rig.quiet(1);
    rig.pattern("A", &PAT_A);
    rig.quiet(2);
    rig.finish(settings(&life, 1, json!({})))
}

/// A driver area plus a modulator bound on the memory area.
fn rig_modulated(
    name: &'static str,
    kind: ModulatorKind,
    magnitude: f32,
    life: MemoryNeuronLifecycleConfig,
) -> (Rig, Vec<(u32, f32)>) {
    let mut rig = Rig::new(spec(name, 1, life, MemoryMpMode::PatternOnly, 4));
    let (_, driver_idx) = rig.add_regular_area(b"cdrv0001", "drv", 1);
    rig.bind_memory_modulator("modulator", kind, magnitude, driver_idx);
    let drive = rig.area_stim(driver_idx, &[0], 10.0);
    (rig, drive)
}

/// `memory.formation` at -100: while its driver fires, no new neuron forms; known patterns
/// still reactivate.
fn rig_formation_blocked() -> Value {
    let life = lifecycle(30, 2.0, 1000, 0, LtmEviction::Lru);
    let (mut rig, drive) = rig_modulated(
        "formation_blocked",
        ModulatorKind::MemoryFormation,
        -100.0,
        life,
    );
    rig.pattern("A", &PAT_A);
    rig.quiet(1);
    rig.note("driver held on");
    rig.hold = drive;
    rig.quiet(1);
    rig.pattern("B", &PAT_B);
    rig.pattern("A", &PAT_A);
    rig.quiet(2);
    rig.pattern("B", &PAT_B);
    rig.note("driver released");
    rig.hold = Vec::new();
    rig.quiet(2);
    rig.pattern("B", &PAT_B);
    rig.quiet(3);
    rig.finish(settings(
        &life,
        1,
        json!({"modulator": "memory.formation -100"}),
    ))
}

/// `memory.freeze`: lifespans, creation and conversion stand still while its driver is active.
fn rig_learning_freeze() -> Value {
    let life = lifecycle(3, 1.0, 1000, 0, LtmEviction::Lru);
    let (mut rig, drive) =
        rig_modulated("learning_freeze", ModulatorKind::MemoryFreeze, 100.0, life);
    rig.pattern("A", &PAT_A);
    rig.quiet(1);
    rig.note("driver held on");
    rig.hold = drive;
    rig.quiet(1);
    rig.pattern("B", &PAT_B);
    rig.pattern("A", &PAT_A);
    rig.quiet(5);
    rig.note("driver released");
    rig.hold = Vec::new();
    rig.quiet(1);
    rig.pattern("B", &PAT_B);
    rig.quiet(5);
    rig.finish(settings(
        &life,
        1,
        json!({"modulator": "memory.freeze +100"}),
    ))
}

fn assoc_rule(mode: &str, extra: Value) -> Value {
    let mut rule = json!({
        "morphology_id": "associative_memory",
        "morphology_scalar": 1,
        "postSynapticCurrent_multiplier": 1.0,
        "plasticity_mode": mode,
        "plasticity_constant": 20,
        "ltp_multiplier": 5,
        "ltd_multiplier": 1,
        "plasticity_window": 3,
        "max_weight": 100.0,
    });
    if let (Some(rule_obj), Some(extra_obj)) = (rule.as_object_mut(), extra.as_object()) {
        for (key, value) in extra_obj {
            rule_obj.insert(key.clone(), value.clone());
        }
    }
    rule
}

/// Properties that keep a memory area from reverberating forever once its neurons excite each
/// other: a neuron may fire two bursts in a row, then rests.
fn damped_memory_props() -> Vec<(&'static str, Value)> {
    vec![
        ("consecutive_fire_limit", json!(2)),
        ("snooze_period", json!(3)),
        ("postsynaptic_current", json!(4.0)),
    ]
}

/// Associative STDP between two memory neurons of one area (self mapping), recall through the
/// sparse memory LIF, replay to the twin with both `replay_trigger` modes, and a mapping edit.
fn rig_assoc_stdp() -> Value {
    // Neurons start long-term: associative recall acts on long-term memory neurons.
    let life = lifecycle(80, 1.0, 80, 0, LtmEviction::Lru);
    let mut rig_spec = spec("assoc_stdp", 1, life, MemoryMpMode::PatternOnly, 4);
    rig_spec.mem_props = damped_memory_props();
    let mut rig = Rig::new(rig_spec);
    let mem_id = rig.mem_id;
    rig.manager
        .write()
        .update_cortical_mapping(&mem_id, &mem_id, vec![assoc_rule("stdp", json!({}))])
        .unwrap();
    rig.manager
        .write()
        .regenerate_synapses_for_mapping(&mem_id, &mem_id)
        .unwrap();
    rig.note("associative_memory self mapping with STDP, window 3");
    for _ in 0..2 {
        rig.pattern("A", &PAT_A);
        rig.pattern("B", &PAT_B);
        rig.quiet(6);
    }
    rig.note("recall: A alone, replay_trigger any_fire");
    rig.pattern("A", &PAT_A);
    rig.quiet(6);
    rig.set_recall_only(true);
    rig.pattern("A", &PAT_A);
    rig.quiet(6);
    rig.pattern("B", &PAT_B);
    rig.quiet(6);
    rig.note("edit the associative mapping (plasticity_window 3 -> 2) and regenerate its synapses");
    rig.manager
        .write()
        .update_cortical_mapping(
            &mem_id,
            &mem_id,
            vec![assoc_rule("stdp", json!({"plasticity_window": 2}))],
        )
        .unwrap();
    rig.manager
        .write()
        .regenerate_synapses_for_mapping(&mem_id, &mem_id)
        .unwrap();
    rig.quiet(1);
    rig.pattern("A", &PAT_A);
    rig.quiet(6);
    rig.finish(settings(
        &life,
        1,
        json!({"mapping": "associative_memory stdp"}),
    ))
}

/// Reward-gated associative STDP: synapses form and move only when the reward driver fires.
fn rig_assoc_rstdp() -> Value {
    let life = lifecycle(80, 1.0, 1000, 0, LtmEviction::Lru);
    let mut rig_spec = spec("assoc_rstdp", 1, life, MemoryMpMode::PatternOnly, 4);
    rig_spec.mem_props = damped_memory_props();
    let mut rig = Rig::new(rig_spec);
    let (_, reward_idx) = rig.add_regular_area(b"crew0001", "rew", 1);
    {
        let guard = rig.npu.lock().unwrap();
        guard.set_modulator_bindings(vec![ModulatorBinding {
            instance_id: "reward".to_string(),
            kind: ModulatorKind::Reward,
            magnitude_percent: 100.0,
            graded: false,
            full_scale_potential: 1.0,
            driver_cortical_idx: reward_idx,
        }]);
    }
    let mem_id = rig.mem_id;
    rig.manager
        .write()
        .update_cortical_mapping(
            &mem_id,
            &mem_id,
            vec![assoc_rule(
                "rstdp",
                json!({
                    "eligibility_decay_bursts": 10,
                    "modulators": ["reward"],
                }),
            )],
        )
        .unwrap();
    rig.manager
        .write()
        .regenerate_synapses_for_mapping(&mem_id, &mem_id)
        .unwrap();
    rig.note("associative_memory self mapping with R-STDP gated by a reward modulator");
    let reward = rig.area_stim(reward_idx, &[0], 10.0);
    for _ in 0..2 {
        rig.pattern("A", &PAT_A);
        rig.pattern("B", &PAT_B);
        rig.quiet(5);
    }
    rig.note("reward arrives with B");
    rig.pattern("A", &PAT_A);
    let mut stim = rig.up_stim(&PAT_B, 10.0);
    stim.extend(reward.iter().copied());
    rig.step("B+reward", &stim);
    rig.quiet(6);
    rig.pattern("A", &PAT_A);
    rig.quiet(6);
    rig.finish(settings(
        &life,
        1,
        json!({"mapping": "associative_memory rstdp, reward"}),
    ))
}

/// Change-encoded identity (`mp_differential`): no replay, identity from MP differences.
fn rig_mp_differential() -> Value {
    let life = lifecycle(30, 2.0, 1000, 0, LtmEviction::Lru);
    let mut rig = Rig::new(spec(
        "mp_differential",
        2,
        life,
        MemoryMpMode::Change(MpChangeEncoding::Differential { quantization: 1.0 }),
        1,
    ));
    for (label, potential) in [
        ("2", 2.0f32),
        ("4", 4.0),
        ("7", 7.0),
        ("9", 9.0),
        ("9", 9.0),
        ("7", 7.0),
    ] {
        let stim = rig.up_stim(&[0], potential);
        rig.step(label, &stim);
    }
    rig.quiet(3);
    rig.finish(settings(
        &life,
        2,
        json!({"mp_mode": "mp_differential", "quantization": 1.0}),
    ))
}

/// A depth change purges neurons outside the new shape (`purge_neurons_outside_shape`).
fn rig_shape_purge() -> Value {
    let life = lifecycle(40, 1.0, 1000, 0, LtmEviction::Lru);
    let mut rig = Rig::new(spec("shape_purge", 3, life, MemoryMpMode::PatternOnly, 4));
    rig.pattern("A", &PAT_A);
    rig.pattern("B", &PAT_B);
    rig.pattern("C", &PAT_C);
    let first = *rig.memory_ids().first().unwrap();
    rig.attach_footprint(first);
    rig.quiet(4);
    rig.note("temporal_depth 3 -> 1: purge, then re-register");
    let upstream = rig.manager.read().get_upstream_cortical_areas(&rig.mem_id);
    let shape = MemoryAreaShape::new(1, upstream.clone(), MemoryMpMode::PatternOnly);
    let purge = rig.service.purge_neurons_outside_shape(rig.mem_idx, &shape);
    rig.note(&format!(
        "purge removed={} removed_long_term={} reasons={:?}",
        purge.removed, purge.removed_long_term, purge.reasons
    ));
    assert!(rig.service.register_memory_area(
        rig.mem_idx,
        "mem".to_string(),
        1,
        upstream,
        Some(life),
        MemoryMpMode::PatternOnly,
    ));
    rig.quiet(1);
    rig.pattern("A", &PAT_A);
    rig.quiet(2);
    rig.pattern("B", &PAT_B);
    rig.quiet(3);
    rig.finish(settings(&life, 3, json!({"new_depth": 1})))
}

/// Connectome save and reload: long-term neurons and synapses survive, short-term ones do not.
fn rig_save_reload() -> Value {
    let life = lifecycle(4, 4.0, 6, 0, LtmEviction::Lru);
    let mut rig = Rig::new(spec("save_reload", 1, life, MemoryMpMode::PatternOnly, 4));
    rig.pattern("A", &PAT_A);
    rig.quiet(1);
    rig.pattern("A", &PAT_A);
    rig.quiet(2);
    rig.pattern("B", &PAT_B);
    rig.quiet(1);
    let snapshot = {
        let guard = rig.npu.lock().unwrap();
        match &*guard {
            DynamicNPU::F32(npu) => npu.export_connectome(),
            DynamicNPU::INT8(npu) => npu.export_connectome(),
        }
    };
    let long_term = rig.service.export_long_term_memory_neurons();
    rig.note(&format!(
        "save: connectome exported, {} long-term neuron(s) exported",
        long_term.len()
    ));
    rig.quiet(1);
    {
        let mut guard = rig.npu.lock().unwrap();
        guard
            .apply_connectome_snapshot(&snapshot)
            .expect("apply snapshot");
    }
    rig.service.reset_all_memory_state();
    rig.service
        .restore_long_term_memory_neurons(&long_term)
        .expect("restore long-term neurons");
    rig.manager
        .write()
        .rebind_npu_runtime_after_connectome_apply()
        .expect("rebind");
    let upstream = rig.manager.read().get_upstream_cortical_areas(&rig.mem_id);
    assert!(rig.service.register_memory_area(
        rig.mem_idx,
        "mem".to_string(),
        1,
        upstream,
        Some(life),
        MemoryMpMode::PatternOnly,
    ));
    rig.note("reload: snapshot applied, long-term neurons restored, area re-registered");
    rig.quiet(1);
    rig.pattern("A", &PAT_A);
    rig.quiet(2);
    rig.pattern("B", &PAT_B);
    rig.quiet(2);
    rig.finish(settings(&life, 1, json!({})))
}

/// Every rig's full output: the golden bursts plus the fire-kind trace.
fn generate_full() -> Value {
    let traces = vec![
        rig_window_depth1(),
        rig_window_depth3(),
        rig_cap_lru(),
        rig_formation_blocked(),
        rig_learning_freeze(),
        rig_assoc_stdp(),
        rig_assoc_rstdp(),
        rig_mp_differential(),
        rig_shape_purge(),
        rig_save_reload(),
    ];
    json!({
        "feagi_core_commit": std::env::var("FEAGI_CORE_COMMIT").unwrap_or_default(),
        "traces": traces,
    })
}

/// The golden traces: the full output without the fire-kind trace.
fn generate() -> Value {
    split(generate_full()).0
}

/// Split the full output into (golden traces, fire-kind fixture).
fn split(mut full: Value) -> (Value, Value) {
    let mut kind_traces = Vec::new();
    for trace in full["traces"].as_array_mut().expect("traces") {
        let obj = trace.as_object_mut().expect("trace");
        let kinds = obj.remove("fire_kinds").expect("fire_kinds");
        kind_traces.push(json!({"name": obj["name"], "bursts": kinds}));
    }
    let commit = full["feagi_core_commit"].clone();
    (
        full,
        json!({"feagi_core_commit": commit, "traces": kind_traces}),
    )
}

/// The fixture comparison ignores the commit note.
fn without_commit(mut value: Value) -> Value {
    if let Some(obj) = value.as_object_mut() {
        obj.remove("feagi_core_commit");
    }
    value
}

fn fixture_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURE_DIR)
        .join(FILE_NAME)
}

#[test]
fn memory_regression_traces_match_the_checked_in_fixture() {
    let first = generate();
    let second = generate();
    assert_eq!(
        without_commit(first.clone()),
        without_commit(second),
        "two generations in one process must agree"
    );

    if let Ok(dirs) = std::env::var("FEAGI_WRITE_MEMORY_REGRESSION_FIXTURES") {
        for dir in dirs.split(':').filter(|d| !d.is_empty()) {
            std::fs::create_dir_all(dir).unwrap();
            let text = serde_json::to_string_pretty(&first).unwrap() + "\n";
            std::fs::write(Path::new(dir).join(FILE_NAME), text).unwrap();
        }
    }

    let text = std::fs::read_to_string(fixture_path()).unwrap_or_else(|e| {
        panic!(
            "missing fixture {}: {e}. Regenerate with FEAGI_WRITE_MEMORY_REGRESSION_FIXTURES",
            fixture_path().display()
        )
    });
    let fixture: Value = serde_json::from_str(&text).expect("fixture json");
    assert_eq!(
        without_commit(first),
        without_commit(fixture),
        "memory behaviour drifted from the golden traces"
    );
}

#[test]
fn memory_regression_fixture_round_trips_through_json() {
    let text = std::fs::read_to_string(fixture_path()).expect("fixture");
    let value: Value = serde_json::from_str(&text).unwrap();
    let again: Value = serde_json::from_str(&serde_json::to_string(&value).unwrap()).unwrap();
    assert_eq!(value, again);
    assert_eq!(value["traces"].as_array().map(Vec::len), Some(10));
}

#[test]
#[should_panic(expected = "drifted")]
fn memory_regression_drift_is_detected() {
    // Negative test: one changed number in the fixture must fail the comparison.
    let mut generated = generate();
    let burst = &mut generated["traces"][0]["bursts"][3];
    burst["memory_neurons"] = json!(999);
    let text = std::fs::read_to_string(fixture_path()).expect("fixture");
    let fixture: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        without_commit(generated),
        without_commit(fixture),
        "memory behaviour drifted from the golden traces"
    );
}

fn kinds_fixture_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURE_DIR)
        .join(KINDS_FILE_NAME)
}

fn kinds_fixture(first: &Value) -> Value {
    split(first.clone()).1
}

#[test]
fn memory_regression_fire_kinds_match_the_checked_in_fixture() {
    let first = kinds_fixture(&generate_full());
    let second = kinds_fixture(&generate_full());
    assert_eq!(
        without_commit(first.clone()),
        without_commit(second),
        "two generations in one process must agree"
    );

    if let Ok(dirs) = std::env::var("FEAGI_WRITE_FIRE_KIND_FIXTURE") {
        for dir in dirs.split(':').filter(|d| !d.is_empty()) {
            std::fs::create_dir_all(dir).unwrap();
            let text = serde_json::to_string_pretty(&first).unwrap() + "\n";
            std::fs::write(Path::new(dir).join(KINDS_FILE_NAME), text).unwrap();
        }
    }

    let text = std::fs::read_to_string(kinds_fixture_path()).unwrap_or_else(|e| {
        panic!(
            "missing fixture {}: {e}. Regenerate with FEAGI_WRITE_FIRE_KIND_FIXTURE",
            kinds_fixture_path().display()
        )
    });
    let fixture: Value = serde_json::from_str(&text).expect("fixture json");
    assert_eq!(
        without_commit(first),
        without_commit(fixture),
        "fire kinds drifted from the golden fire-kind trace"
    );
}

#[test]
fn memory_regression_fire_kinds_fixture_has_default_and_episodic_kinds() {
    let text = std::fs::read_to_string(kinds_fixture_path()).expect("fixture");
    let value: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(value["traces"].as_array().map(Vec::len), Some(10));
    let mut kinds = BTreeSet::new();
    for trace in value["traces"].as_array().unwrap() {
        for burst in trace["bursts"].as_array().unwrap() {
            for pairs in burst["kinds"].as_object().unwrap().values() {
                for pair in pairs.as_array().unwrap() {
                    kinds.insert(pair.as_str().unwrap().rsplit(':').next().unwrap().to_string());
                }
            }
        }
    }
    assert!(kinds.contains("0"), "default kind missing: {kinds:?}");
    assert!(kinds.contains("1"), "episodic kind missing: {kinds:?}");
}

#[test]
#[should_panic(expected = "drifted")]
fn memory_regression_fire_kinds_drift_is_detected() {
    // Negative test: one flipped kind must fail the comparison.
    let mut generated = kinds_fixture(&generate_full());
    let flipped = generated["traces"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .flat_map(|t| t["bursts"].as_array_mut().unwrap().iter_mut())
        .flat_map(|b| b["kinds"].as_object_mut().unwrap().values_mut())
        .find_map(|pairs| pairs.as_array_mut().unwrap().first_mut())
        .expect("a fired id");
    let text = flipped.as_str().unwrap().to_string();
    let (id, kind) = text.rsplit_once(':').unwrap();
    *flipped = json!(format!("{id}:{}", if kind == "0" { "1" } else { "0" }));
    let text = std::fs::read_to_string(kinds_fixture_path()).expect("fixture");
    let fixture: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        without_commit(generated),
        without_commit(fixture),
        "fire kinds drifted from the golden fire-kind trace"
    );
}
