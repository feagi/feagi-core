// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Modulator signals and the spike-train schedule.
//!
//! @cursor:critical-path — empty registries return immediately. Area rewrites
//! run only when a subscribed factor changes. Spike-train bookkeeping walks
//! only neurons that fired in areas with the flag set.

use ahash::{AHashMap, AHashSet};
use feagi_npu_neural::types::NeuralValue;
use feagi_npu_runtime::NeuronStorage;
use feagi_structures::genomic::{
    gate_factor, memory_formation_factor, modulator_signal, multiplicative_factor, scale_baseline,
    summed_reward, ModulatorKind,
};

use crate::fire_structures::FireQueue;

/// Which neuron parameter a neuromodulator rewrites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum NeuronParam {
    Threshold,
    Leak,
    Excitability,
}

impl NeuronParam {
    fn from_kind(kind: ModulatorKind) -> Option<Self> {
        match kind {
            ModulatorKind::FiringThreshold => Some(Self::Threshold),
            ModulatorKind::Leak => Some(Self::Leak),
            ModulatorKind::FiringProbability => Some(Self::Excitability),
            _ => None,
        }
    }
}

/// One genome instance pushed into the burst engine.
#[derive(Debug, Clone)]
pub struct ModulatorBinding {
    pub instance_id: String,
    pub kind: ModulatorKind,
    pub magnitude_percent: f32,
    pub graded: bool,
    pub full_scale_potential: f32,
    pub driver_cortical_idx: u32,
}

/// Per-neuron baselines for one subscribed area. Allocated only while a
/// subscription for that parameter exists.
#[derive(Debug, Default)]
struct AreaBaseline {
    neuron_indices: Vec<usize>,
    threshold: Option<Vec<f32>>,
    leak: Option<Vec<f32>>,
    excitability: Option<Vec<f32>>,
}

/// Factors that were produced by the previous burst and are active now.
#[derive(Debug, Default)]
struct ActiveFactors {
    reward_by_group: AHashMap<u16, f32>,
    learning_by_group: AHashMap<u16, f32>,
    transmission_by_group: AHashMap<u16, f32>,
    area_factor: AHashMap<(u32, NeuronParam), f32>,
    /// cortical idx -> memory formation factor in `[0, 1]`, for subscribed areas only.
    memory_formation_by_area: AHashMap<u32, f32>,
}

/// Burst-engine side of the modulator system.
#[derive(Debug, Default)]
pub struct ModulatorEngine {
    bindings: Vec<ModulatorBinding>,
    /// cortical idx -> subscribed instance ids (neuromodulators).
    area_subscriptions: AHashMap<u32, Vec<String>>,
    /// group id -> subscribed instance ids (synaptic modulators). 0 is unused.
    groups: AHashMap<u16, Vec<String>>,
    next_group: u16,
    baselines: AHashMap<u32, AreaBaseline>,
    /// cortical idx values whose leak baseline is refreshed from homeostatic leak.
    homeostatic_leak_areas: AHashSet<u32>,
    /// neuron id -> membrane potential held for the rest of the spike train.
    active_trains: AHashMap<u32, f32>,
    spike_train_areas: AHashSet<u32>,
    active: ActiveFactors,
    /// Factor last written into neuron storage, so an unchanged factor is a no-op.
    last_area_factor: AHashMap<(u32, NeuronParam), f32>,
}

impl ModulatorEngine {
    pub fn is_idle(&self) -> bool {
        self.bindings.is_empty()
            && self.active_trains.is_empty()
            && self.spike_train_areas.is_empty()
            && self.groups.is_empty()
    }

    pub fn spike_train_idle(&self) -> bool {
        self.spike_train_areas.is_empty() && self.active_trains.is_empty()
    }

    /// Replace instance bindings. Existing group ids stay valid.
    pub fn set_bindings(&mut self, bindings: Vec<ModulatorBinding>) {
        self.bindings = bindings;
        self.close_unsettled_gates();
    }

    /// A gated group starts closed: until a burst commits a driver signal there is no
    /// factor for it, and a missing factor would read as open.
    fn close_unsettled_gates(&mut self) {
        let gated: Vec<u16> = self
            .groups
            .iter()
            .filter(|(_, ids)| {
                ids.iter().any(|id| {
                    self.binding(id)
                        .is_some_and(|binding| binding.kind == ModulatorKind::Gate)
                })
            })
            .map(|(group, _)| *group)
            .collect();
        for group in gated {
            self.active
                .transmission_by_group
                .entry(group)
                .or_insert(0.0);
        }
    }

    /// Rewrite an instance id in the group table and area subscriptions after a genome rename,
    /// so already-registered mappings keep their modulation. Bindings are replaced separately.
    pub fn rename_instance(&mut self, old_id: &str, new_id: &str) {
        let rewrite = |ids: &mut Vec<String>| {
            for id in ids.iter_mut().filter(|id| id.as_str() == old_id) {
                *id = new_id.to_string();
            }
        };
        self.groups.values_mut().for_each(rewrite);
        self.area_subscriptions.values_mut().for_each(rewrite);
    }

    /// Drop an instance id from the group table and area subscriptions after a genome delete.
    /// A group left empty stays allocated and yields reward 0.
    pub fn remove_instance(&mut self, id: &str) {
        for ids in self.groups.values_mut() {
            ids.retain(|entry| entry != id);
        }
        for ids in self.area_subscriptions.values_mut() {
            ids.retain(|entry| entry != id);
        }
        self.area_subscriptions.retain(|_, ids| !ids.is_empty());
    }

    /// Instance ids a modulation group currently resolves to.
    pub fn group_instance_ids(&self, group: u16) -> Vec<String> {
        self.groups.get(&group).cloned().unwrap_or_default()
    }

    pub fn set_area_subscriptions(&mut self, subscriptions: AHashMap<u32, Vec<String>>) {
        self.area_subscriptions = subscriptions;
    }

    /// Instance ids this cortical area subscribes to (neuromodulators).
    pub fn area_subscriptions(&self, cortical_idx: u32) -> Vec<String> {
        self.area_subscriptions
            .get(&cortical_idx)
            .cloned()
            .unwrap_or_default()
    }

    /// Whether a subscribed `memory.formation` modulator currently blocks new memory
    /// neurons in this area (factor 0). No subscription means false. O(subscriptions).
    pub fn memory_formation_suppressed(&self, cortical_idx: u32) -> bool {
        let subscribed = self
            .area_subscriptions
            .get(&cortical_idx)
            .is_some_and(|ids| {
                ids.iter().any(|id| {
                    self.binding(id)
                        .is_some_and(|b| b.kind == ModulatorKind::MemoryFormation)
                })
            });
        subscribed
            && self
                .active
                .memory_formation_by_area
                .get(&cortical_idx)
                .is_some_and(|factor| *factor <= 0.0)
    }

    pub fn set_spike_train_areas(&mut self, areas: AHashSet<u32>) {
        self.spike_train_areas = areas;
    }

    pub fn set_homeostatic_leak_areas(&mut self, areas: AHashSet<u32>) {
        self.homeostatic_leak_areas = areas;
    }

    /// Whether `cortical_idx` is currently a homeostatic leak area.
    pub fn is_homeostatic_leak_area(&self, cortical_idx: u32) -> bool {
        self.homeostatic_leak_areas.contains(&cortical_idx)
    }

    /// Add one area to the homeostatic leak set.
    pub fn add_homeostatic_leak_area(&mut self, cortical_idx: u32) {
        self.homeostatic_leak_areas.insert(cortical_idx);
    }

    /// Drop one area from the homeostatic leak set.
    pub fn remove_homeostatic_leak_area(&mut self, cortical_idx: u32) {
        self.homeostatic_leak_areas.remove(&cortical_idx);
    }

    /// Assign a modulation group for a mapping rule. Returns 0 when the rule
    /// lists no synaptic modulators.
    pub fn allocate_group(&mut self, instance_ids: &[String]) -> u16 {
        let synaptic: Vec<String> = instance_ids
            .iter()
            .filter(|id| {
                self.bindings
                    .iter()
                    .any(|binding| binding.instance_id == **id && binding.kind.is_synaptic())
            })
            .cloned()
            .collect();
        if synaptic.is_empty() {
            return 0;
        }
        if let Some((id, _)) = self.groups.iter().find(|(_, ids)| {
            ids.len() == synaptic.len() && ids.iter().all(|id| synaptic.contains(id))
        }) {
            return *id;
        }
        self.next_group = self.next_group.saturating_add(1).max(1);
        let id = self.next_group;
        self.groups.insert(id, synaptic);
        self.close_unsettled_gates();
        id
    }

    pub fn reward_for_group(&self, group: u16) -> f32 {
        if group == 0 {
            return 0.0;
        }
        self.active
            .reward_by_group
            .get(&group)
            .copied()
            .unwrap_or(0.0)
    }

    /// Whether this synaptic group lists at least one `synaptic.reward` instance. A mapping
    /// in such a group is reward-gated whatever its `plasticity_mode` says.
    pub fn group_has_reward(&self, group: u16) -> bool {
        if group == 0 {
            return false;
        }
        self.groups.get(&group).is_some_and(|ids| {
            ids.iter().any(|id| {
                self.binding(id)
                    .is_some_and(|binding| binding.kind == ModulatorKind::Reward)
            })
        })
    }

    /// Driver areas of the `synaptic.reward` instances in this synaptic group.
    pub fn reward_driver_areas(&self, group: u16) -> Vec<u32> {
        if group == 0 {
            return Vec::new();
        }
        self.groups
            .get(&group)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.binding(id))
                    .filter(|binding| binding.kind == ModulatorKind::Reward)
                    .map(|binding| binding.driver_cortical_idx)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Whether any of these instance ids is bound to a `synaptic.reward` modulator. Read-only:
    /// unlike `allocate_group` it never creates a group.
    pub fn ids_include_reward(&self, ids: &[String]) -> bool {
        ids.iter().any(|id| {
            self.binding(id)
                .is_some_and(|binding| binding.kind == ModulatorKind::Reward)
        })
    }

    pub fn learning_factor(&self, group: u16) -> f32 {
        if group == 0 {
            return 1.0;
        }
        self.active
            .learning_by_group
            .get(&group)
            .copied()
            .unwrap_or(1.0)
    }

    /// Transmission factors indexed by group id. Index 0 is unused.
    pub fn transmission_factors(&self) -> Vec<f32> {
        let mut factors = vec![1.0_f32; self.next_group as usize + 1];
        for (group, factor) in &self.active.transmission_by_group {
            if let Some(slot) = factors.get_mut(*group as usize) {
                *slot = *factor;
            }
        }
        factors
    }

    pub fn transmission_active(&self) -> bool {
        self.active
            .transmission_by_group
            .values()
            .any(|factor| (*factor - 1.0).abs() > f32::EPSILON)
    }

    /// Full-strength reward signal of the driver area, used when a classifier
    /// is the event that fires that driver.
    pub fn authored_reward_signal(&self, driver_cortical_idx: u32) -> Option<f32> {
        self.bindings.iter().find_map(|binding| {
            if binding.driver_cortical_idx == driver_cortical_idx
                && binding.kind == ModulatorKind::Reward
            {
                Some(modulator_signal(
                    binding.magnitude_percent,
                    true,
                    binding.graded,
                    binding.full_scale_potential,
                    binding.full_scale_potential,
                ))
            } else {
                None
            }
        })
    }

    /// Continue spike trains for neurons that fired, and return the forced
    /// firings to schedule on the next burst as `(neuron_id, held_potential)`.
    pub fn advance_spike_trains<N: NeuronStorage>(
        &mut self,
        storage: &mut N,
        fire_queue: &FireQueue,
    ) -> Vec<(u32, f32)> {
        if self.spike_train_areas.is_empty() && self.active_trains.is_empty() {
            return Vec::new();
        }
        let mut schedule = Vec::new();
        let mut fired_ids = AHashSet::new();
        for (area, neurons) in &fire_queue.neurons_by_area {
            if !self.spike_train_areas.contains(area) {
                continue;
            }
            for neuron in neurons {
                let id = neuron.neuron_id.0;
                let idx = id as usize;
                if idx >= storage.count() || !storage.valid_mask()[idx] {
                    continue;
                }
                fired_ids.insert(id);
                let limit = storage.consecutive_fire_limits()[idx];
                if limit == 0 || limit == u16::MAX {
                    continue;
                }
                if let Some(held) = self.active_trains.get(&id).copied() {
                    let new_count = storage.consecutive_fire_counts()[idx].saturating_add(1);
                    storage.consecutive_fire_counts_mut()[idx] = new_count;
                    if new_count >= limit {
                        self.active_trains.remove(&id);
                        let countdown = storage.refractory_periods()[idx]
                            .saturating_add(storage.snooze_periods()[idx]);
                        storage.refractory_countdowns_mut()[idx] = countdown;
                    } else {
                        storage.refractory_countdowns_mut()[idx] = 1;
                        schedule.push((id, held));
                    }
                } else {
                    let count = storage.consecutive_fire_counts()[idx];
                    if count >= limit {
                        continue;
                    }
                    let held = neuron.membrane_potential;
                    self.active_trains.insert(id, held);
                    storage.refractory_countdowns_mut()[idx] = 1;
                    schedule.push((id, held));
                }
            }
        }
        self.active_trains.retain(|id, _| fired_ids.contains(id));
        schedule
    }

    /// Capture this burst's driver firings into the factors the next burst uses,
    /// and rewrite neuron parameters whose factor changed.
    pub fn commit_burst<N: NeuronStorage>(&mut self, storage: &mut N, fire_queue: &FireQueue) {
        if self.bindings.is_empty() && self.baselines.is_empty() {
            return;
        }
        let signals = self.driver_signals(fire_queue);
        self.active = self.factors_from(&signals);
        self.apply_area_factors(storage);
    }

    /// Copy the current neuron values into baselines, then reapply the active factor.
    /// Called after a genome or API write of the raw property.
    pub fn rebase_area_param<N: NeuronStorage>(
        &mut self,
        storage: &mut N,
        cortical_idx: u32,
        kind: ModulatorKind,
    ) {
        let Some(param) = NeuronParam::from_kind(kind) else {
            return;
        };
        self.capture_baseline(storage, cortical_idx, param, true);
        let factor = self
            .active
            .area_factor
            .get(&(cortical_idx, param))
            .copied()
            .unwrap_or(1.0);
        self.write_param(storage, cortical_idx, param, factor);
        self.last_area_factor.insert((cortical_idx, param), factor);
    }

    /// Restore baselines and drop them when an area no longer subscribes to a parameter.
    pub fn release_unused_baselines<N: NeuronStorage>(&mut self, storage: &mut N) {
        let needed = self.needed_params();
        let areas: Vec<u32> = self.baselines.keys().copied().collect();
        for area in areas {
            for param in [
                NeuronParam::Threshold,
                NeuronParam::Leak,
                NeuronParam::Excitability,
            ] {
                if needed
                    .get(&area)
                    .map(|set| set.contains(&param))
                    .unwrap_or(false)
                {
                    continue;
                }
                self.restore_param(storage, area, param);
                self.last_area_factor.remove(&(area, param));
                if let Some(baseline) = self.baselines.get_mut(&area) {
                    match param {
                        NeuronParam::Threshold => baseline.threshold = None,
                        NeuronParam::Leak => baseline.leak = None,
                        NeuronParam::Excitability => baseline.excitability = None,
                    }
                }
            }
            if self
                .baselines
                .get(&area)
                .map(|b| b.threshold.is_none() && b.leak.is_none() && b.excitability.is_none())
                .unwrap_or(false)
            {
                self.baselines.remove(&area);
            }
        }
    }

    fn driver_signals(&self, fire_queue: &FireQueue) -> AHashMap<u32, f32> {
        let mut signals = AHashMap::new();
        for binding in &self.bindings {
            let fired = fire_queue
                .neurons_by_area
                .get(&binding.driver_cortical_idx)
                .and_then(|neurons| neurons.first());
            let (is_fired, mp) = match fired {
                Some(neuron) => (true, neuron.membrane_potential),
                None => (false, 0.0),
            };
            let signal = modulator_signal(
                binding.magnitude_percent,
                is_fired,
                binding.graded,
                mp,
                binding.full_scale_potential,
            );
            signals.insert(binding.driver_cortical_idx, signal);
        }
        signals
    }

    fn factors_from(&self, signals: &AHashMap<u32, f32>) -> ActiveFactors {
        let mut active = ActiveFactors::default();
        for (group, ids) in &self.groups {
            let mut reward = Vec::new();
            let mut learning = Vec::new();
            let mut transmission = Vec::new();
            let mut gate = Vec::new();
            for id in ids {
                let Some(binding) = self.binding(id) else {
                    continue;
                };
                let signal = signals
                    .get(&binding.driver_cortical_idx)
                    .copied()
                    .unwrap_or(0.0);
                match binding.kind {
                    ModulatorKind::Reward => reward.push(signal),
                    ModulatorKind::LearningRate => learning.push(signal),
                    ModulatorKind::TransmissionGain => transmission.push(signal),
                    ModulatorKind::Gate => gate.push(signal),
                    _ => {}
                }
            }
            if !reward.is_empty() {
                active
                    .reward_by_group
                    .insert(*group, summed_reward(&reward));
            }
            if !learning.is_empty() {
                active
                    .learning_by_group
                    .insert(*group, multiplicative_factor(&learning));
            }
            if !transmission.is_empty() || !gate.is_empty() {
                // Gain and gate multiply: a closed gate zeroes the whole group.
                let mut factor = multiplicative_factor(&transmission);
                if !gate.is_empty() {
                    factor *= gate_factor(&gate);
                }
                active.transmission_by_group.insert(*group, factor);
            }
        }
        for (area, ids) in &self.area_subscriptions {
            let formation: Vec<f32> = ids
                .iter()
                .filter_map(|id| {
                    let binding = self.binding(id)?;
                    (binding.kind == ModulatorKind::MemoryFormation).then(|| {
                        signals
                            .get(&binding.driver_cortical_idx)
                            .copied()
                            .unwrap_or(0.0)
                    })
                })
                .collect();
            if !formation.is_empty() {
                active
                    .memory_formation_by_area
                    .insert(*area, memory_formation_factor(&formation));
            }
        }
        let needed = self.needed_params();
        for (area, params) in needed {
            for param in params {
                let signals_for_param = self.signals_for_area_param(area, param, signals);
                let factor = multiplicative_factor(&signals_for_param);
                active.area_factor.insert((area, param), factor);
            }
        }
        active
    }

    fn signals_for_area_param(
        &self,
        area: u32,
        param: NeuronParam,
        signals: &AHashMap<u32, f32>,
    ) -> Vec<f32> {
        let Some(ids) = self.area_subscriptions.get(&area) else {
            return Vec::new();
        };
        ids.iter()
            .filter_map(|id| {
                let binding = self.binding(id)?;
                if NeuronParam::from_kind(binding.kind) != Some(param) {
                    return None;
                }
                Some(
                    signals
                        .get(&binding.driver_cortical_idx)
                        .copied()
                        .unwrap_or(0.0),
                )
            })
            .collect()
    }

    fn needed_params(&self) -> AHashMap<u32, AHashSet<NeuronParam>> {
        let mut needed: AHashMap<u32, AHashSet<NeuronParam>> = AHashMap::new();
        for (area, ids) in &self.area_subscriptions {
            for id in ids {
                if let Some(binding) = self.binding(id) {
                    if let Some(param) = NeuronParam::from_kind(binding.kind) {
                        needed.entry(*area).or_default().insert(param);
                    }
                }
            }
        }
        needed
    }

    fn binding(&self, id: &str) -> Option<&ModulatorBinding> {
        self.bindings
            .iter()
            .find(|binding| binding.instance_id == id)
    }

    /// Put homeostatic leak areas back on their baseline so the leak update
    /// edits the baseline, not an already-scaled value.
    pub fn restore_homeostatic_leak_baselines<N: NeuronStorage>(&self, storage: &mut N) {
        let areas: Vec<u32> = self.homeostatic_leak_areas.iter().copied().collect();
        for area in areas {
            self.restore_param(storage, area, NeuronParam::Leak);
        }
    }

    fn apply_area_factors<N: NeuronStorage>(&mut self, storage: &mut N) {
        let planned: Vec<((u32, NeuronParam), f32)> = self
            .active
            .area_factor
            .iter()
            .map(|(key, factor)| (*key, *factor))
            .collect();
        for ((area, param), factor) in planned {
            let refresh_leak =
                param == NeuronParam::Leak && self.homeostatic_leak_areas.contains(&area);
            self.capture_baseline(storage, area, param, refresh_leak);
            let previous = self.last_area_factor.get(&(area, param)).copied();
            if previous == Some(factor) && !refresh_leak {
                continue;
            }
            self.write_param(storage, area, param, factor);
            self.last_area_factor.insert((area, param), factor);
        }
    }

    fn capture_baseline<N: NeuronStorage>(
        &mut self,
        storage: &N,
        cortical_idx: u32,
        param: NeuronParam,
        replace: bool,
    ) {
        let indices = neuron_indices(storage, cortical_idx);
        let existing = self
            .baselines
            .get(&cortical_idx)
            .and_then(|baseline| param_slot(baseline, param).cloned());
        let values = if let Some(mut current) = existing {
            if !replace {
                if current.len() < indices.len() {
                    for &idx in indices.iter().skip(current.len()) {
                        current.push(read_one(storage, idx, param));
                    }
                }
                current
            } else {
                read_param(storage, &indices, param)
            }
        } else {
            read_param(storage, &indices, param)
        };
        let baseline = self.baselines.entry(cortical_idx).or_default();
        baseline.neuron_indices = indices;
        match param {
            NeuronParam::Threshold => baseline.threshold = Some(values),
            NeuronParam::Leak => baseline.leak = Some(values),
            NeuronParam::Excitability => baseline.excitability = Some(values),
        }
    }

    fn write_param<N: NeuronStorage>(
        &self,
        storage: &mut N,
        cortical_idx: u32,
        param: NeuronParam,
        factor: f32,
    ) {
        let Some(baseline) = self.baselines.get(&cortical_idx) else {
            return;
        };
        let values = match param_slot(baseline, param) {
            Some(values) => values.clone(),
            None => return,
        };
        let indices = baseline.neuron_indices.clone();
        let kind = match param {
            NeuronParam::Threshold => ModulatorKind::FiringThreshold,
            NeuronParam::Leak => ModulatorKind::Leak,
            NeuronParam::Excitability => ModulatorKind::FiringProbability,
        };
        for (offset, &idx) in indices.iter().enumerate() {
            let Some(&base) = values.get(offset) else {
                continue;
            };
            if idx >= storage.count() {
                continue;
            }
            store_param(storage, idx, param, scale_baseline(kind, base, factor));
        }
    }

    fn restore_param<N: NeuronStorage>(
        &self,
        storage: &mut N,
        cortical_idx: u32,
        param: NeuronParam,
    ) {
        let Some(baseline) = self.baselines.get(&cortical_idx) else {
            return;
        };
        let values = match param_slot(baseline, param) {
            Some(values) => values.clone(),
            None => return,
        };
        let indices = baseline.neuron_indices.clone();
        for (offset, &idx) in indices.iter().enumerate() {
            let Some(&base) = values.get(offset) else {
                continue;
            };
            if idx >= storage.count() {
                continue;
            }
            store_param(storage, idx, param, base);
        }
    }
}

fn param_slot(baseline: &AreaBaseline, param: NeuronParam) -> Option<&Vec<f32>> {
    match param {
        NeuronParam::Threshold => baseline.threshold.as_ref(),
        NeuronParam::Leak => baseline.leak.as_ref(),
        NeuronParam::Excitability => baseline.excitability.as_ref(),
    }
}

fn neuron_indices<N: NeuronStorage>(storage: &N, cortical_idx: u32) -> Vec<usize> {
    let mut indices = Vec::new();
    for idx in 0..storage.count() {
        if storage.valid_mask()[idx] && storage.cortical_areas()[idx] == cortical_idx {
            indices.push(idx);
        }
    }
    indices
}

fn read_one<N: NeuronStorage>(storage: &N, idx: usize, param: NeuronParam) -> f32 {
    match param {
        NeuronParam::Threshold => storage.thresholds()[idx].to_f32(),
        NeuronParam::Leak => storage.leak_coefficients()[idx],
        NeuronParam::Excitability => storage.excitabilities()[idx],
    }
}

fn read_param<N: NeuronStorage>(storage: &N, indices: &[usize], param: NeuronParam) -> Vec<f32> {
    indices
        .iter()
        .map(|&idx| read_one(storage, idx, param))
        .collect()
}

fn store_param<N: NeuronStorage>(storage: &mut N, idx: usize, param: NeuronParam, value: f32) {
    match param {
        NeuronParam::Threshold => storage.thresholds_mut()[idx] = N::Value::from_f32(value),
        NeuronParam::Leak => storage.leak_coefficients_mut()[idx] = value,
        NeuronParam::Excitability => storage.excitabilities_mut()[idx] = value,
    }
}

/// Spike-train state machine used by unit tests and by [`ModulatorEngine::advance_spike_trains`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpikeTrainPhase {
    Idle,
    InTrain {
        held_potential: f32,
        fires_so_far: u16,
    },
}

/// Result of one firing while `spike_train` is enabled.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpikeTrainStep {
    pub phase: SpikeTrainPhase,
    pub schedule_next: bool,
    pub train_ended: bool,
}

/// Advance one neuron. `fires_before` is the consecutive-fire count before this firing.
/// `limit` must be >= 1; callers reject 0.
pub fn step_spike_train(
    phase: SpikeTrainPhase,
    fires_before: u16,
    limit: u16,
    membrane_potential: f32,
) -> SpikeTrainStep {
    let fires_after = fires_before.saturating_add(1);
    if fires_after >= limit {
        return SpikeTrainStep {
            phase: SpikeTrainPhase::Idle,
            schedule_next: false,
            train_ended: true,
        };
    }
    let held = match phase {
        SpikeTrainPhase::InTrain { held_potential, .. } => held_potential,
        SpikeTrainPhase::Idle => membrane_potential,
    };
    SpikeTrainStep {
        phase: SpikeTrainPhase::InTrain {
            held_potential: held,
            fires_so_far: fires_after,
        },
        schedule_next: true,
        train_ended: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn train_of_three_holds_the_trigger_potential_then_ends() {
        let start = step_spike_train(SpikeTrainPhase::Idle, 0, 3, 7.5);
        assert!(start.schedule_next);
        assert!(!start.train_ended);
        let SpikeTrainPhase::InTrain {
            held_potential,
            fires_so_far,
        } = start.phase
        else {
            panic!("train should be active");
        };
        assert!((held_potential - 7.5).abs() < 1e-6);
        assert_eq!(fires_so_far, 1);

        let mid = step_spike_train(start.phase, 1, 3, 0.0);
        assert!(mid.schedule_next);
        let SpikeTrainPhase::InTrain { held_potential, .. } = mid.phase else {
            panic!("held potential must survive the forced firing");
        };
        assert!((held_potential - 7.5).abs() < 1e-6);

        let end = step_spike_train(mid.phase, 2, 3, 99.0);
        assert!(end.train_ended);
        assert!(!end.schedule_next);
        assert_eq!(end.phase, SpikeTrainPhase::Idle);
    }

    #[test]
    fn limit_one_ends_on_the_trigger() {
        let step = step_spike_train(SpikeTrainPhase::Idle, 0, 1, 4.0);
        assert!(step.train_ended);
        assert!(!step.schedule_next);
    }

    #[test]
    fn reward_group_sums_and_learning_multiplies() {
        let mut engine = ModulatorEngine::default();
        engine.set_bindings(vec![
            ModulatorBinding {
                instance_id: "pleasure".into(),
                kind: ModulatorKind::Reward,
                magnitude_percent: 100.0,
                graded: false,
                full_scale_potential: 1.0,
                driver_cortical_idx: 1,
            },
            ModulatorBinding {
                instance_id: "pain".into(),
                kind: ModulatorKind::Reward,
                magnitude_percent: -100.0,
                graded: false,
                full_scale_potential: 1.0,
                driver_cortical_idx: 2,
            },
            ModulatorBinding {
                instance_id: "eta".into(),
                kind: ModulatorKind::LearningRate,
                magnitude_percent: 50.0,
                graded: false,
                full_scale_potential: 1.0,
                driver_cortical_idx: 3,
            },
        ]);
        let group = engine.allocate_group(&["pleasure".into(), "pain".into(), "eta".into()]);
        let mut signals = AHashMap::new();
        signals.insert(1, 1.0);
        signals.insert(2, -1.0);
        signals.insert(3, 0.5);
        let factors = engine.factors_from(&signals);
        assert!((factors.reward_by_group[&group]).abs() < 1e-6);
        assert!((factors.learning_by_group[&group] - 1.5).abs() < 1e-6);
    }

    fn gate_binding(id: &str, idx: u32, graded: bool) -> ModulatorBinding {
        ModulatorBinding {
            instance_id: id.into(),
            kind: ModulatorKind::Gate,
            magnitude_percent: 100.0,
            graded,
            full_scale_potential: 10.0,
            driver_cortical_idx: idx,
        }
    }

    fn gate_signals(engine: &ModulatorEngine, group: u16, drivers: &[(u32, f32)]) -> f32 {
        let signals: AHashMap<u32, f32> = drivers.iter().copied().collect();
        engine.factors_from(&signals).transmission_by_group[&group]
    }

    #[test]
    fn gate_is_closed_with_silent_driver_and_open_when_driver_fires() {
        let mut engine = ModulatorEngine::default();
        engine.set_bindings(vec![gate_binding("g", 1, false)]);
        let group = engine.allocate_group(&["g".into()]);
        assert_eq!(gate_signals(&engine, group, &[(1, 0.0)]), 0.0);
        assert!((gate_signals(&engine, group, &[(1, 1.0)]) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn graded_gate_scales_with_membrane_potential() {
        let mut engine = ModulatorEngine::default();
        engine.set_bindings(vec![gate_binding("g", 1, true)]);
        let group = engine.allocate_group(&["g".into()]);
        let signal = modulator_signal(100.0, true, true, 5.0, 10.0);
        assert!((gate_signals(&engine, group, &[(1, signal)]) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn multiple_gates_open_if_any_is_open() {
        let mut engine = ModulatorEngine::default();
        engine.set_bindings(vec![
            gate_binding("a", 1, false),
            gate_binding("b", 2, false),
        ]);
        let group = engine.allocate_group(&["a".into(), "b".into()]);
        assert_eq!(gate_signals(&engine, group, &[(1, 0.0), (2, 0.0)]), 0.0);
        assert!((gate_signals(&engine, group, &[(1, 0.0), (2, 1.0)]) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn gate_multiplies_with_transmission_gain() {
        let mut engine = ModulatorEngine::default();
        engine.set_bindings(vec![
            gate_binding("g", 1, false),
            ModulatorBinding {
                instance_id: "gain".into(),
                kind: ModulatorKind::TransmissionGain,
                magnitude_percent: 50.0,
                graded: false,
                full_scale_potential: 1.0,
                driver_cortical_idx: 2,
            },
        ]);
        let group = engine.allocate_group(&["g".into(), "gain".into()]);
        assert!((gate_signals(&engine, group, &[(1, 1.0), (2, 0.5)]) - 1.5).abs() < 1e-6);
        assert_eq!(gate_signals(&engine, group, &[(1, 0.0), (2, 0.5)]), 0.0);
    }

    #[test]
    fn group_without_gate_is_unaffected() {
        let mut engine = ModulatorEngine::default();
        engine.set_bindings(vec![reward_binding("r", 1)]);
        let group = engine.allocate_group(&["r".into()]);
        let signals: AHashMap<u32, f32> = [(1, 1.0)].into_iter().collect();
        assert!(!engine
            .factors_from(&signals)
            .transmission_by_group
            .contains_key(&group));
    }

    #[test]
    fn ids_include_reward_is_read_only_and_reward_specific() {
        let mut engine = ModulatorEngine::default();
        engine.set_bindings(vec![
            reward_binding("pleasure", 1),
            ModulatorBinding {
                instance_id: "eta".into(),
                kind: ModulatorKind::LearningRate,
                magnitude_percent: 50.0,
                graded: false,
                full_scale_potential: 1.0,
                driver_cortical_idx: 3,
            },
        ]);
        assert!(engine.ids_include_reward(&["pleasure".into()]));
        assert!(engine.ids_include_reward(&["eta".into(), "pleasure".into()]));
        assert!(!engine.ids_include_reward(&["eta".into()]));
        assert!(!engine.ids_include_reward(&["unknown".into()]));
        assert!(!engine.ids_include_reward(&[]));
        assert!(
            engine.group_instance_ids(1).is_empty(),
            "no group allocated"
        );
    }

    fn reward_binding(id: &str, idx: u32) -> ModulatorBinding {
        ModulatorBinding {
            instance_id: id.into(),
            kind: ModulatorKind::Reward,
            magnitude_percent: 100.0,
            graded: false,
            full_scale_potential: 1.0,
            driver_cortical_idx: idx,
        }
    }

    #[test]
    fn rename_instance_keeps_group_and_area_subscriptions_modulated() {
        let mut engine = ModulatorEngine::default();
        engine.set_bindings(vec![reward_binding("pleasure", 1)]);
        let group = engine.allocate_group(&["pleasure".into()]);
        engine.set_area_subscriptions(AHashMap::from([(7, vec!["pleasure".to_string()])]));
        let mut signals = AHashMap::new();
        signals.insert(1, 1.0);

        // Pushing renamed bindings alone leaves the group on the old id: reward drops to 0.
        engine.set_bindings(vec![reward_binding("joy", 1)]);
        let stale = engine.factors_from(&signals);
        assert!(
            stale
                .reward_by_group
                .get(&group)
                .copied()
                .unwrap_or(0.0)
                .abs()
                < 1e-6
        );

        engine.rename_instance("pleasure", "joy");
        assert_eq!(engine.group_instance_ids(group), vec!["joy".to_string()]);
        assert_eq!(engine.area_subscriptions(7), vec!["joy".to_string()]);
        assert!((engine.factors_from(&signals).reward_by_group[&group] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn remove_instance_drops_id_from_groups_and_area_subscriptions() {
        let mut engine = ModulatorEngine::default();
        engine.set_bindings(vec![
            reward_binding("pleasure", 1),
            reward_binding("pain", 2),
        ]);
        let group = engine.allocate_group(&["pleasure".into(), "pain".into()]);
        engine.set_area_subscriptions(AHashMap::from([(7, vec!["pain".to_string()])]));
        engine.remove_instance("pain");
        assert_eq!(
            engine.group_instance_ids(group),
            vec!["pleasure".to_string()]
        );
        assert!(engine.area_subscriptions(7).is_empty());
    }
}
