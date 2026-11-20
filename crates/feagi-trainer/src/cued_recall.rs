//! Cued recall: a generic task that cues a gesture slot, gates learning, and scores what the
//! brain recalls. It never computes poses for the body: it holds named input roles, reads
//! fired voxels per burst, and counts matches.
//!
//! Phases per slot: **Learn** (cue held, `learn_enable` held during the hold), **Check**
//! (`learn_enable` released, cue held at the home bin, one hold per gesture), **Follow**
//! (named in the config, run in a later stage; reported `not_ready` with the missing roles).
//!
//! Windows (all in bursts, counted from the first burst of the hold): scoring uses hold bursts
//! `settle ..= hold-1`. Learn's hold starts on the burst `learn_enable` is first held; Check's hold
//! starts on `t0`, the first burst where the human state equals Learn's `human_ref`.
//!
//! Every area is named by the caller's config (the Trainer holds no area names). The brain is
//! reached through [`CuedRecallBrain`], so the transport lives with the caller.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::error::TrainerError;
use crate::reach_hold::seconds_to_whole_bursts;

fn cfg_err<T>(msg: impl Into<String>) -> Result<T, TrainerError> {
    Err(TrainerError::Config(msg.into()))
}

/// The input roles the task holds. The role is the hold id: one hold per role, released apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Cue,
    LearnEnable,
    SequencerQuiet,
    FollowEnable,
}

impl Role {
    /// Stable hold id and config name of the role.
    pub fn hold_id(self) -> &'static str {
        match self {
            Role::Cue => "cue",
            Role::LearnEnable => "learn_enable",
            Role::SequencerQuiet => "sequencer_quiet",
            Role::FollowEnable => "follow_enable",
        }
    }
}

/// One change the task asks of the brain, with a per-role hold id (a hold on one role never
/// replaces or releases another role's hold).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CueAction {
    /// Fire `voxels` of the role's area on every burst until released or re-held.
    HoldRole { role: Role, voxels: Vec<[u32; 3]> },
    /// End the hold of one role.
    ReleaseRole { role: Role },
    /// End every hold the task made.
    ReleaseAll,
}

/// The fired voxels of each area on one burst, keyed by the area name used in the config.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BurstFrame {
    pub fired: BTreeMap<String, Vec<[u32; 3]>>,
}

/// What the task needs from the live brain. A fake implements it in tests.
pub trait CuedRecallBrain {
    /// FEAGI refresh rate in Hz.
    fn burst_frequency_hz(&mut self) -> Result<f64, TrainerError>;
    /// Depth (bins along z) of the area named in the config.
    fn area_depth(&mut self, area: &str) -> Result<u32, TrainerError>;
    /// Apply one hold or release.
    fn apply(&mut self, action: &CueAction) -> Result<(), TrainerError>;
    /// Advance `bursts` bursts and return exactly one frame per burst, oldest first.
    fn advance(&mut self, bursts: u32) -> Result<Vec<BurstFrame>, TrainerError>;
}

// ---------------------------------------------------------------- config

/// A named area plus the voxel that is fired to hold it (default `[0, 0, 0]`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RolePort {
    pub area: String,
    #[serde(default)]
    pub voxel: [u32; 3],
}

/// The cue area: bins along z at `[x, y, bin]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CuePort {
    pub area: String,
    pub home_bin: u32,
    #[serde(default)]
    pub x: u32,
    #[serde(default)]
    pub y: u32,
}

/// One joint's strip: bins along z at column `x` of the area.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StripPort {
    pub area: String,
    #[serde(default)]
    pub x: u32,
}

/// A human-state group: its whole fired set is the state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupPort {
    pub area: String,
}

/// Ports by role. Follow roles are always named (a missing one is a parse error); whether they
/// bind in the live brain is the caller's lookup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ports {
    pub cue: CuePort,
    pub learn_enable: RolePort,
    pub recall: Vec<StripPort>,
    pub robot_state: Vec<StripPort>,
    pub human_state: Vec<GroupPort>,
    pub sequencer_quiet: RolePort,
    pub follow_enable: RolePort,
}

/// A slot from the body adapter's slot list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlotConfig {
    pub slot: u32,
    pub move_id: String,
    #[serde(default)]
    pub cue_bin: Option<u32>,
    /// Reason this slot is not served (the slot is skipped and needs no `cue_bin`).
    #[serde(default)]
    pub skip: Option<String>,
}

/// Timings of a run in seconds. Every field has a default (`Default`), so a config may give only
/// the fields it changes. The defaults are whole numbers of bursts at any refresh rate that is a
/// multiple of 2 Hz; a run at another rate must set its own. The one home of these defaults:
/// clients read them through [`Defaults`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct TimingS {
    pub prepare: f64,
    pub hold: f64,
    pub settle: f64,
    pub arrive_timeout: f64,
    pub follow_timeout: f64,
    pub home_timeout: f64,
}

impl Default for TimingS {
    fn default() -> Self {
        Self {
            prepare: 1.0,
            hold: 2.5,
            settle: 0.5,
            arrive_timeout: 2.0,
            follow_timeout: 3.0,
            home_timeout: 3.0,
        }
    }
}

/// Repeats of Learn and Follow. Check has none: one hold per gesture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Repeats {
    pub learn: u32,
    pub follow: u32,
}

/// The only home of the verdict and Follow thresholds.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Thresholds {
    pub learned: f64,
    pub unsure: f64,
    pub follow_s: f64,
    pub follow_share: f64,
    pub reach_tolerance_bins: u32,
    pub max_missed_burst_share: f64,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            learned: 0.8,
            unsure: 0.5,
            follow_s: 3.0,
            follow_share: 0.8,
            reach_tolerance_bins: 0,
            max_missed_burst_share: 0.05,
        }
    }
}

/// Home bin proposed for the cue before a legend confirms one.
pub const DEFAULT_HOME_BIN: u32 = 0;

/// Every default a client may show or start from, read from here and never copied.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Defaults {
    pub timing_s: TimingS,
    pub thresholds: Thresholds,
    pub home_bin: u32,
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            timing_s: TimingS::default(),
            thresholds: Thresholds::default(),
            home_bin: DEFAULT_HOME_BIN,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseKind {
    Learn,
    Check,
    Follow,
}

fn all_phases() -> Vec<PhaseKind> {
    vec![PhaseKind::Learn, PhaseKind::Check, PhaseKind::Follow]
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CuedRecallConfig {
    pub ports: Ports,
    pub slots: Vec<SlotConfig>,
    pub timing_s: TimingS,
    pub repeats: Repeats,
    #[serde(default)]
    pub thresholds: Thresholds,
    /// Phases of the run, in order. Follow stays listed; it reports `not_ready` until its roles bind.
    #[serde(default = "all_phases")]
    pub phases: Vec<PhaseKind>,
}

/// Timing in whole bursts at the run's FEAGI refresh rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseBursts {
    pub prepare: u32,
    pub hold: u32,
    pub settle: u32,
    pub arrive_timeout: u32,
    pub follow_timeout: u32,
    pub home_timeout: u32,
}

impl CuedRecallConfig {
    /// Parses and validates the parts that need no brain.
    pub fn from_json(text: &str) -> Result<Self, TrainerError> {
        let cfg: Self = serde_json::from_str(text)
            .map_err(|e| TrainerError::Config(format!("cued recall config: {e}")))?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Slots the run serves (not skipped), in config order.
    pub fn active_slots(&self) -> impl Iterator<Item = &SlotConfig> {
        self.slots.iter().filter(|s| s.skip.is_none())
    }

    pub fn validate(&self) -> Result<(), TrainerError> {
        let p = &self.ports;
        for (what, v) in [
            ("cue.area", &p.cue.area),
            ("learn_enable.area", &p.learn_enable.area),
            ("sequencer_quiet.area", &p.sequencer_quiet.area),
            ("follow_enable.area", &p.follow_enable.area),
        ] {
            if v.trim().is_empty() {
                return cfg_err(format!("{what} must be named"));
            }
        }
        if p.recall.is_empty() || p.recall.len() != p.robot_state.len() {
            return cfg_err("recall and robot_state need the same, non-zero number of joints");
        }
        if p.human_state.is_empty() {
            return cfg_err("human_state needs at least one group");
        }
        let mut seen_slots = BTreeSet::new();
        let mut seen_bins = BTreeSet::new();
        for s in &self.slots {
            if !seen_slots.insert(s.slot) {
                return cfg_err(format!("slot {} is listed twice", s.slot));
            }
            if s.skip.is_some() {
                continue;
            }
            let Some(bin) = s.cue_bin else {
                return cfg_err(format!(
                    "slot {} needs a cue_bin (or a skip reason)",
                    s.slot
                ));
            };
            if bin == p.cue.home_bin {
                return cfg_err(format!("slot {} cue_bin {bin} equals home_bin", s.slot));
            }
            if !seen_bins.insert(bin) {
                return cfg_err(format!("cue_bin {bin} is used by more than one slot"));
            }
        }
        if self.active_slots().next().is_none() {
            return cfg_err("no slot is served");
        }
        if self.repeats.learn == 0 || self.repeats.follow == 0 {
            return cfg_err("repeats must be >= 1");
        }
        let t = &self.thresholds;
        let frac = |v: f64| v.is_finite() && (0.0..=1.0).contains(&v);
        if !(frac(t.learned) && frac(t.unsure) && t.unsure <= t.learned) {
            return cfg_err("thresholds need 0 <= unsure <= learned <= 1");
        }
        if !frac(t.follow_share)
            || !frac(t.max_missed_burst_share)
            || !(t.follow_s.is_finite() && t.follow_s > 0.0)
        {
            return cfg_err(
                "follow_share and max_missed_burst_share are fractions; follow_s is > 0",
            );
        }
        if self.phases.is_empty() {
            return cfg_err("phases must not be empty");
        }
        if self.timing_s.settle >= self.timing_s.hold {
            return cfg_err("settle must be shorter than hold");
        }
        Ok(())
    }

    /// Converts seconds to whole bursts at `hz` (a fractional burst is a config error).
    pub fn bursts(&self, hz: f64) -> Result<PhaseBursts, TrainerError> {
        let t = &self.timing_s;
        let b = PhaseBursts {
            prepare: seconds_to_whole_bursts("prepare", t.prepare, hz)?,
            hold: seconds_to_whole_bursts("hold", t.hold, hz)?,
            settle: seconds_to_whole_bursts("settle", t.settle, hz)?,
            arrive_timeout: seconds_to_whole_bursts("arrive_timeout", t.arrive_timeout, hz)?,
            follow_timeout: seconds_to_whole_bursts("follow_timeout", t.follow_timeout, hz)?,
            home_timeout: seconds_to_whole_bursts("home_timeout", t.home_timeout, hz)?,
        };
        if b.settle >= b.hold {
            return cfg_err("settle must be shorter than hold in bursts");
        }
        Ok(b)
    }
}

/// A validated run: config, refresh rate and burst counts. Built by [`start`].
#[derive(Debug, Clone)]
pub struct CuedRecallPlan {
    pub config: CuedRecallConfig,
    pub hz: f64,
    pub bursts: PhaseBursts,
}

/// Reads the refresh rate and area depths, then validates. Cue bins (slot and home) must be
/// inside the cue area's depth; each joint's recall and robot-state strips must have equal depth.
pub fn start<B: CuedRecallBrain>(
    brain: &mut B,
    config: CuedRecallConfig,
) -> Result<CuedRecallPlan, TrainerError> {
    config.validate()?;
    let hz = brain.burst_frequency_hz()?;
    let bursts = config.bursts(hz)?;
    let depth = brain.area_depth(&config.ports.cue.area)?;
    let mut bins = vec![config.ports.cue.home_bin];
    bins.extend(config.active_slots().filter_map(|s| s.cue_bin));
    if let Some(bad) = bins.iter().find(|b| **b >= depth) {
        return cfg_err(format!(
            "cue bin {bad} is outside the cue area depth {depth}"
        ));
    }
    for (r, s) in config.ports.recall.iter().zip(&config.ports.robot_state) {
        let (dr, ds) = (brain.area_depth(&r.area)?, brain.area_depth(&s.area)?);
        if dr != ds {
            return cfg_err(format!(
                "recall depth {dr} differs from robot_state depth {ds}"
            ));
        }
    }
    Ok(CuedRecallPlan { config, hz, bursts })
}

// ---------------------------------------------------------------- readout

/// One bin per joint, or `None` when any joint has 0 or >= 2 fired voxels in its strip.
pub fn joint_pattern(frame: &BurstFrame, strips: &[StripPort]) -> Option<Vec<u32>> {
    strips
        .iter()
        .map(|s| {
            let mut hits = frame
                .fired
                .get(&s.area)?
                .iter()
                .filter(|v| v[0] == s.x)
                .map(|v| v[2]);
            let first = hits.next()?;
            hits.next().is_none().then_some(first)
        })
        .collect()
}

/// The fired set of every group (sorted), or `None` when a group is silent.
pub fn human_pattern(frame: &BurstFrame, groups: &[GroupPort]) -> Option<Vec<Vec<[u32; 3]>>> {
    groups
        .iter()
        .map(|g| {
            let mut v = frame.fired.get(&g.area)?.clone();
            if v.is_empty() {
                return None;
            }
            v.sort_unstable();
            v.dedup();
            Some(v)
        })
        .collect()
}

fn mode<T: Ord + Clone>(items: impl Iterator<Item = T>) -> Option<T> {
    let mut counts: BTreeMap<T, u32> = BTreeMap::new();
    for i in items {
        *counts.entry(i).or_default() += 1;
    }
    // Highest count; ties go to the smallest pattern, so the result is deterministic.
    counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0)))
        .map(|(k, _)| k)
}

/// The most common item and its count (ties go to the smallest item).
fn mode_count<T: Ord + Clone>(items: impl Iterator<Item = T>) -> Option<(T, u32)> {
    let mut counts: BTreeMap<T, u32> = BTreeMap::new();
    for i in items {
        *counts.entry(i).or_default() += 1;
    }
    counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0)))
}

fn frames_checked(frames: Vec<BurstFrame>, want: u32) -> Result<Vec<BurstFrame>, TrainerError> {
    if frames.len() != want as usize {
        return Err(TrainerError::Runtime(format!(
            "brain returned {} frames for {want} bursts",
            frames.len()
        )));
    }
    Ok(frames)
}

// ---------------------------------------------------------------- results

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Learned,
    Unsure,
    NotLearned,
}

/// Why a slot has no (or a zero) score.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "note", content = "detail")]
pub enum SlotNote {
    Skipped(String),
    /// No burst gave a full robot or human pattern in any Learn hold window.
    NoReference,
    /// Robot reference equals another slot's or home's: scored 0.
    Indistinguishable,
    /// Check: the human state never reached `human_ref` within `arrive_timeout`.
    NoArrival,
}

/// References one slot learned in this session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotRef {
    /// Most frequent robot pattern over the Learn hold windows (all repeats pooled).
    pub robot_ref: Vec<u32>,
    /// Most frequent human fired set over the same windows.
    pub human_ref: Vec<Vec<[u32; 3]>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LearnSlotResult {
    pub slot: u32,
    pub robot_ref: Option<Vec<u32>>,
    pub note: Option<SlotNote>,
}

/// What Learn produced. Check reads only from here.
#[derive(Debug, Clone, Default)]
pub struct LearnSession {
    pub refs: BTreeMap<u32, SlotRef>,
    pub home_robot_ref: Option<Vec<u32>>,
    pub results: Vec<LearnSlotResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CheckResult {
    pub slot: u32,
    /// Share of window bursts where the recalled pose equals `robot_ref`.
    pub recall: Option<f64>,
    /// Share of window bursts where the human state equals `human_ref` (low = jitter, not forgetting).
    pub held_share: Option<f64>,
    pub window_bursts: u32,
    pub matched_bursts: u32,
    pub t0_burst: Option<u32>,
    pub verdict: Option<Verdict>,
    pub note: Option<SlotNote>,
    /// Most common clean recalled pose (one bin per strip) in the scoring window; null when no
    /// window burst showed a clean pose ("recalled nothing").
    pub recalled_modal: Option<Vec<u32>>,
    /// Share of the window bursts showing `recalled_modal`.
    pub recalled_modal_share: Option<f64>,
    /// The learned robot pose the recall was compared to.
    pub robot_ref: Option<Vec<u32>>,
    /// Most common human fired set in the scoring window.
    pub human_modal: Option<Vec<Vec<[u32; 3]>>>,
    /// Share of the window bursts showing `human_modal`.
    pub human_modal_share: Option<f64>,
}

/// A phase that cannot run yet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum FollowResult {
    /// Roles the brain does not bind (named like the config roles).
    NotReady { missing: Vec<String> },
}

pub fn verdict_for(recall: f64, t: &Thresholds) -> Verdict {
    if recall >= t.learned {
        Verdict::Learned
    } else if recall >= t.unsure {
        Verdict::Unsure
    } else {
        Verdict::NotLearned
    }
}

// ---------------------------------------------------------------- phases

type HumanPattern = Vec<Vec<[u32; 3]>>;

fn cue_voxel(p: &CuePort, bin: u32) -> Vec<[u32; 3]> {
    vec![[p.x, p.y, bin]]
}

/// Robot and human patterns read over the hold windows of one gesture's Learn repeats.
#[derive(Debug, Clone, Default)]
pub struct SlotSamples {
    pub robots: Vec<Vec<u32>>,
    pub humans: Vec<HumanPattern>,
}

/// Learn, first step: releases every hold, holds the cue at the home bin without learning and
/// reads the home reference. Skipped slots get their note here.
pub fn run_learn_start<B: CuedRecallBrain>(
    brain: &mut B,
    plan: &CuedRecallPlan,
) -> Result<LearnSession, TrainerError> {
    let cfg = &plan.config;
    let b = plan.bursts;
    let ports = &cfg.ports;
    let mut session = LearnSession::default();
    brain.apply(&CueAction::ReleaseAll)?;
    brain.apply(&CueAction::HoldRole {
        role: Role::Cue,
        voxels: cue_voxel(&ports.cue, ports.cue.home_bin),
    })?;
    let frames = frames_checked(brain.advance(b.prepare + b.hold)?, b.prepare + b.hold)?;
    let from = (b.prepare + b.settle) as usize;
    session.home_robot_ref = mode(
        frames[from..]
            .iter()
            .filter_map(|f| joint_pattern(f, &ports.robot_state)),
    );
    for s in &cfg.slots {
        if let Some(reason) = &s.skip {
            session.results.push(LearnSlotResult {
                slot: s.slot,
                robot_ref: None,
                note: Some(SlotNote::Skipped(reason.clone())),
            });
        }
    }
    Ok(session)
}

/// Learn, one gesture: per repeat, hold the cue for `prepare` bursts (the body moves), then hold
/// `learn_enable` too for `hold` bursts and release it; the cue stays held. The caller may
/// abandon the step on any error and call it again: it starts from its prepare step and leaves
/// no state behind but the holds, which the caller releases.
pub fn run_learn_slot<B: CuedRecallBrain>(
    brain: &mut B,
    plan: &CuedRecallPlan,
    slot: u32,
) -> Result<SlotSamples, TrainerError> {
    let cfg = &plan.config;
    let b = plan.bursts;
    let ports = &cfg.ports;
    let bin = cfg
        .active_slots()
        .find(|s| s.slot == slot)
        .and_then(|s| s.cue_bin)
        .ok_or_else(|| TrainerError::Config(format!("slot {slot} is not served")))?;
    let mut out = SlotSamples::default();
    for _ in 0..cfg.repeats.learn {
        brain.apply(&CueAction::HoldRole {
            role: Role::Cue,
            voxels: cue_voxel(&ports.cue, bin),
        })?;
        brain.advance(b.prepare)?;
        brain.apply(&CueAction::HoldRole {
            role: Role::LearnEnable,
            voxels: vec![ports.learn_enable.voxel],
        })?;
        let frames = frames_checked(brain.advance(b.hold)?, b.hold)?;
        // Cue stays held; only learning is released.
        brain.apply(&CueAction::ReleaseRole {
            role: Role::LearnEnable,
        })?;
        for f in &frames[b.settle as usize..] {
            if let Some(r) = joint_pattern(f, &ports.robot_state) {
                out.robots.push(r);
            }
            if let Some(h) = human_pattern(f, &ports.human_state) {
                out.humans.push(h);
            }
        }
    }
    Ok(out)
}

/// Learn, last step: releases every hold and turns the pooled samples of each gesture (in the
/// order learned) into references and notes.
pub fn run_learn_finish<B: CuedRecallBrain>(
    brain: &mut B,
    mut session: LearnSession,
    pooled: Vec<(u32, SlotSamples)>,
) -> Result<LearnSession, TrainerError> {
    brain.apply(&CueAction::ReleaseAll)?;
    for (slot, samples) in pooled {
        match (
            mode(samples.robots.into_iter()),
            mode(samples.humans.into_iter()),
        ) {
            (Some(robot_ref), Some(human_ref)) => {
                session.refs.insert(
                    slot,
                    SlotRef {
                        robot_ref,
                        human_ref,
                    },
                );
            }
            _ => session.results.push(LearnSlotResult {
                slot,
                robot_ref: None,
                note: Some(SlotNote::NoReference),
            }),
        }
    }
    // Slots that cannot be told apart from home or from each other score 0.
    let robots: Vec<(u32, Vec<u32>)> = session
        .refs
        .iter()
        .map(|(k, r)| (*k, r.robot_ref.clone()))
        .collect();
    for (slot, r) in &robots {
        let clash = session.home_robot_ref.as_ref() == Some(r)
            || robots.iter().any(|(k, o)| k != slot && o == r);
        session.results.push(LearnSlotResult {
            slot: *slot,
            robot_ref: Some(r.clone()),
            note: clash.then_some(SlotNote::Indistinguishable),
        });
    }
    session.results.sort_by_key(|r| r.slot);
    Ok(session)
}

/// Learn, all served slots, in order: [`run_learn_start`], [`run_learn_slot`] per served slot,
/// [`run_learn_finish`].
pub fn run_learn<B: CuedRecallBrain>(
    brain: &mut B,
    plan: &CuedRecallPlan,
) -> Result<LearnSession, TrainerError> {
    let session = run_learn_start(brain, plan)?;
    let mut pooled = Vec::new();
    for s in plan.config.active_slots() {
        pooled.push((s.slot, run_learn_slot(brain, plan, s.slot)?));
    }
    run_learn_finish(brain, session, pooled)
}

/// Check one gesture: refused unless Learn in `session` produced references for `slot`.
/// `learn_enable` is released and the cue sits at the home bin for the whole step.
pub fn run_check<B: CuedRecallBrain>(
    brain: &mut B,
    plan: &CuedRecallPlan,
    session: &LearnSession,
    slot: u32,
) -> Result<CheckResult, TrainerError> {
    let cfg = &plan.config;
    let b = plan.bursts;
    let ports = &cfg.ports;
    let blank = |note, recall: Option<f64>, verdict| CheckResult {
        slot,
        recall,
        held_share: None,
        window_bursts: 0,
        matched_bursts: 0,
        t0_burst: None,
        verdict,
        note,
        recalled_modal: None,
        recalled_modal_share: None,
        robot_ref: None,
        human_modal: None,
        human_modal_share: None,
    };
    let Some(sref) = session.refs.get(&slot) else {
        return cfg_err(format!(
            "check of slot {slot} refused: no Learn reference for it in this session"
        ));
    };
    let clash = session
        .results
        .iter()
        .any(|r| r.slot == slot && r.note == Some(SlotNote::Indistinguishable));
    if clash {
        let mut r = blank(
            Some(SlotNote::Indistinguishable),
            Some(0.0),
            Some(Verdict::NotLearned),
        );
        r.robot_ref = Some(sref.robot_ref.clone());
        return Ok(r);
    }

    brain.apply(&CueAction::ReleaseRole {
        role: Role::LearnEnable,
    })?;
    brain.apply(&CueAction::HoldRole {
        role: Role::Cue,
        voxels: cue_voxel(&ports.cue, ports.cue.home_bin),
    })?;
    brain.advance(b.prepare)?;

    // Wait for the human to take the gesture (no fallback to cue onset).
    let mut carry: Option<BurstFrame> = None;
    let mut waited = 0u32;
    while waited < b.arrive_timeout {
        let f = frames_checked(brain.advance(1)?, 1)?.remove(0);
        waited += 1;
        if human_pattern(&f, &ports.human_state).as_ref() == Some(&sref.human_ref) {
            carry = Some(f);
            break;
        }
    }
    let Some(first) = carry else {
        brain.apply(&CueAction::ReleaseAll)?;
        return Ok(blank(Some(SlotNote::NoArrival), None, None));
    };
    let mut frames = vec![first];
    frames.extend(frames_checked(brain.advance(b.hold - 1)?, b.hold - 1)?);
    brain.apply(&CueAction::ReleaseAll)?;

    let window = &frames[b.settle as usize..];
    let n = window.len() as u32;
    let matched = window
        .iter()
        .filter(|f| joint_pattern(f, &ports.recall).as_ref() == Some(&sref.robot_ref))
        .count() as u32;
    let held = window
        .iter()
        .filter(|f| human_pattern(f, &ports.human_state).as_ref() == Some(&sref.human_ref))
        .count() as u32;
    let recall = f64::from(matched) / f64::from(n);
    let share = |c: u32| f64::from(c) / f64::from(n);
    let recalled = mode_count(
        window
            .iter()
            .filter_map(|f| joint_pattern(f, &ports.recall)),
    );
    let humans = mode_count(
        window
            .iter()
            .filter_map(|f| human_pattern(f, &ports.human_state)),
    );
    Ok(CheckResult {
        slot,
        recall: Some(recall),
        held_share: Some(f64::from(held) / f64::from(n)),
        window_bursts: n,
        matched_bursts: matched,
        t0_burst: Some(waited - 1),
        verdict: Some(verdict_for(recall, &cfg.thresholds)),
        note: None,
        recalled_modal_share: recalled.as_ref().map(|(_, c)| share(*c)),
        recalled_modal: recalled.map(|(p, _)| p),
        robot_ref: Some(sref.robot_ref.clone()),
        human_modal_share: humans.as_ref().map(|(_, c)| share(*c)),
        human_modal: humans.map(|(p, _)| p),
    })
}

/// Follow is named in the config but not run in this stage. `unbound` lists the Follow roles
/// the caller could not bind in the live brain; the result names them.
pub fn run_follow(
    plan: &CuedRecallPlan,
    session: &LearnSession,
    slot: u32,
    unbound: &[Role],
) -> Result<FollowResult, TrainerError> {
    let _ = plan;
    if !session.refs.contains_key(&slot) {
        return cfg_err(format!(
            "follow of slot {slot} refused: no Learn reference for it in this session"
        ));
    }
    let mut missing: Vec<String> = unbound.iter().map(|r| r.hold_id().to_string()).collect();
    if missing.is_empty() {
        missing.push("follow_phase_not_implemented".to_string());
    }
    Ok(FollowResult::NotReady { missing })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CFG: &str = r#"{
      "ports": {
        "cue": {"area": "cue_a", "home_bin": 0},
        "learn_enable": {"area": "learn_a"},
        "recall": [{"area": "rec0"}, {"area": "rec1"}],
        "robot_state": [{"area": "rob0"}, {"area": "rob1"}],
        "human_state": [{"area": "hum0"}, {"area": "hum1"}],
        "sequencer_quiet": {"area": "quiet_a"},
        "follow_enable": {"area": "follow_a"}
      },
      "slots": [
        {"slot": 0, "move_id": "m_a", "cue_bin": 1},
        {"slot": 1, "move_id": "m_b", "cue_bin": 2},
        {"slot": 2, "move_id": "m_c", "skip": "needs:extra"}
      ],
      "timing_s": {"prepare": 0.5, "hold": 2.0, "settle": 0.5, "arrive_timeout": 1.0,
                   "follow_timeout": 2.0, "home_timeout": 2.0},
      "repeats": {"learn": 2, "follow": 2}
    }"#;

    fn cfg() -> CuedRecallConfig {
        CuedRecallConfig::from_json(CFG).unwrap()
    }

    fn edit(f: impl FnOnce(&mut serde_json::Value)) -> Result<CuedRecallConfig, TrainerError> {
        let mut v: serde_json::Value = serde_json::from_str(CFG).unwrap();
        f(&mut v);
        CuedRecallConfig::from_json(&v.to_string())
    }

    /// A scripted brain at 20 Hz. Robot pose follows the held cue bin; the human shows the
    /// cued gesture while learning and `human_bin` from burst `human_from` otherwise; recall
    /// fires the learned pose in `recall_hits` of every 10 bursts when the cue is at home.
    struct Fake {
        burst: u64,
        held: BTreeMap<Role, Vec<[u32; 3]>>,
        log: Vec<(u64, CueAction)>,
        human_bin: u32,
        human_from: u64,
        jitter10: u64,
        recall_hits: u64,
        collapse_bin2: bool,
        learn_held_bursts: u64,
        cue_during_learn_held: bool,
        depth: u32,
    }

    impl Fake {
        fn new() -> Self {
            Self {
                burst: 0,
                held: BTreeMap::new(),
                log: vec![],
                human_bin: 1,
                human_from: 0,
                jitter10: 0,
                recall_hits: 10,
                collapse_bin2: false,
                learn_held_bursts: 0,
                cue_during_learn_held: true,
                depth: 4,
            }
        }
        fn pose(&self, bin: u32) -> u32 {
            if self.collapse_bin2 && bin == 2 {
                1
            } else {
                bin
            }
        }
        fn frame(&mut self) -> BurstFrame {
            let n = self.burst;
            self.burst += 1;
            let learning = self.held.contains_key(&Role::LearnEnable);
            let cue_bin = self.held.get(&Role::Cue).map(|v| v[0][2]);
            if learning {
                self.learn_held_bursts += 1;
                self.cue_during_learn_held &= cue_bin.is_some();
            }
            let mut f = BurstFrame::default();
            let robot = self.pose(cue_bin.unwrap_or(0));
            for a in ["rob0", "rob1"] {
                f.fired.insert(a.into(), vec![[0, 0, robot]]);
            }
            let human = if learning {
                cue_bin
            } else if n >= self.human_from {
                Some(if self.jitter10 > 0 && n % 10 < self.jitter10 {
                    3
                } else {
                    self.human_bin
                })
            } else {
                None
            };
            if let Some(h) = human {
                for a in ["hum0", "hum1"] {
                    f.fired.insert(a.into(), vec![[h, 0, 0]]);
                }
            }
            if !learning && cue_bin == Some(0) && n >= self.human_from && n % 10 < self.recall_hits
            {
                let r = self.pose(self.human_bin);
                for a in ["rec0", "rec1"] {
                    f.fired.insert(a.into(), vec![[0, 0, r]]);
                }
            }
            f
        }
    }

    impl CuedRecallBrain for Fake {
        fn burst_frequency_hz(&mut self) -> Result<f64, TrainerError> {
            Ok(20.0)
        }
        fn area_depth(&mut self, _area: &str) -> Result<u32, TrainerError> {
            Ok(self.depth)
        }
        fn apply(&mut self, action: &CueAction) -> Result<(), TrainerError> {
            self.log.push((self.burst, action.clone()));
            match action {
                CueAction::HoldRole { role, voxels } => {
                    self.held.insert(*role, voxels.clone());
                }
                CueAction::ReleaseRole { role } => {
                    self.held.remove(role);
                }
                CueAction::ReleaseAll => self.held.clear(),
            }
            Ok(())
        }
        fn advance(&mut self, bursts: u32) -> Result<Vec<BurstFrame>, TrainerError> {
            Ok((0..bursts).map(|_| self.frame()).collect())
        }
    }

    fn learned(brain: &mut Fake) -> (CuedRecallPlan, LearnSession) {
        let plan = start(brain, cfg()).unwrap();
        let s = run_learn(brain, &plan).unwrap();
        (plan, s)
    }

    #[test]
    fn config_parses_and_carries_follow_roles() {
        let c = cfg();
        assert_eq!(c.ports.sequencer_quiet.area, "quiet_a");
        assert_eq!(c.ports.follow_enable.area, "follow_a");
        assert_eq!(c.phases, all_phases());
        assert_eq!(c.thresholds, Thresholds::default());
        assert_eq!(c.bursts(20.0).unwrap().hold, 40);
    }

    #[test]
    fn follow_roles_are_required() {
        assert!(edit(|v| {
            v["ports"].as_object_mut().unwrap().remove("follow_enable");
        })
        .is_err());
        assert!(edit(|v| {
            v["ports"]
                .as_object_mut()
                .unwrap()
                .remove("sequencer_quiet");
        })
        .is_err());
    }

    #[test]
    fn timing_defaults_fill_missing_fields() {
        let t: TimingS = serde_json::from_str(r#"{"hold": 4.0}"#).unwrap();
        assert_eq!(t.hold, 4.0);
        assert_eq!(t.prepare, TimingS::default().prepare);
        assert!(TimingS::default().settle < TimingS::default().hold);
        let c = edit(|v| {
            v.as_object_mut()
                .unwrap()
                .insert("timing_s".into(), serde_json::json!({}));
        });
        assert!(c.is_ok(), "{c:?}");
    }

    #[test]
    fn duplicate_or_home_cue_bins_are_refused() {
        let e = edit(|v| v["slots"][1]["cue_bin"] = 1.into())
            .unwrap_err()
            .to_string();
        assert!(e.contains("more than one slot"), "{e}");
        let e = edit(|v| v["slots"][1]["cue_bin"] = 0.into())
            .unwrap_err()
            .to_string();
        assert!(e.contains("home_bin"), "{e}");
        assert!(edit(|v| {
            v["slots"][0].as_object_mut().unwrap().remove("cue_bin");
        })
        .is_err());
    }

    #[test]
    fn cue_bins_must_fit_the_cue_depth_read_at_start() {
        let mut brain = Fake::new();
        brain.depth = 2; // bin 2 is out of range
        let e = start(&mut brain, cfg()).unwrap_err().to_string();
        assert!(e.contains("outside the cue area depth"), "{e}");
    }

    #[test]
    fn fractional_bursts_are_refused() {
        let c = edit(|v| v["timing_s"]["hold"] = 2.03.into()).unwrap();
        assert!(c.bursts(20.0).is_err());
    }

    #[test]
    fn learn_holds_learn_enable_only_during_holds_and_keeps_cue() {
        let mut brain = Fake::new();
        let (plan, s) = learned(&mut brain);
        // 2 active slots x 2 repeats x 40 hold bursts.
        assert_eq!(brain.learn_held_bursts, 2 * 2 * u64::from(plan.bursts.hold));
        assert!(
            brain.cue_during_learn_held,
            "cue stayed held while learn_enable was held"
        );
        let rel = brain
            .log
            .iter()
            .filter(|(_, a)| {
                *a == CueAction::ReleaseRole {
                    role: Role::LearnEnable,
                }
            })
            .count();
        assert_eq!(rel, 4);
        assert!(brain.held.is_empty(), "everything released at the end");
        assert_eq!(s.refs[&0].robot_ref, vec![1, 1]);
        assert_eq!(s.refs[&1].robot_ref, vec![2, 2]);
        assert_eq!(s.home_robot_ref, Some(vec![0, 0]));
        assert!(s
            .results
            .iter()
            .any(|r| r.slot == 2 && matches!(r.note, Some(SlotNote::Skipped(_)))));
    }

    #[test]
    fn learn_slot_replays_from_prepare_and_matches_run_learn() {
        let mut brain = Fake::new();
        let plan = start(&mut brain, cfg()).unwrap();
        let session = run_learn_start(&mut brain, &plan).unwrap();
        let a = run_learn_slot(&mut brain, &plan, 0).unwrap();
        // Replaying the slot gives the same samples and starts with the cue hold.
        let log_len = brain.log.len();
        let again = run_learn_slot(&mut brain, &plan, 0).unwrap();
        assert_eq!(a.robots, again.robots);
        assert!(matches!(
            &brain.log[log_len],
            (
                _,
                CueAction::HoldRole {
                    role: Role::Cue,
                    ..
                }
            )
        ));
        let b = run_learn_slot(&mut brain, &plan, 1).unwrap();
        let s = run_learn_finish(&mut brain, session, vec![(0, again), (1, b)]).unwrap();
        let mut other = Fake::new();
        let (_, whole) = learned(&mut other);
        assert_eq!(s.refs[&0].robot_ref, whole.refs[&0].robot_ref);
        assert_eq!(s.refs[&1].robot_ref, whole.refs[&1].robot_ref);
        assert_eq!(s.results.len(), whole.results.len());
        assert!(run_learn_slot(&mut brain, &plan, 2).is_err());
    }

    #[test]
    fn defaults_carry_the_core_timing_thresholds_and_home_bin() {
        let d = Defaults::default();
        assert_eq!(d.timing_s, TimingS::default());
        assert_eq!(d.thresholds, Thresholds::default());
        assert_eq!(d.home_bin, 0);
        let v = serde_json::to_value(d).unwrap();
        assert_eq!(v["timing_s"]["prepare"], 1.0);
        assert_eq!(v["thresholds"]["learned"], 0.8);
    }

    #[test]
    fn check_learned_with_full_recall() {
        let mut brain = Fake::new();
        let (plan, s) = learned(&mut brain);
        brain.human_bin = 1;
        brain.human_from = brain.burst + 5;
        let r = run_check(&mut brain, &plan, &s, 0).unwrap();
        assert_eq!(r.recall, Some(1.0));
        assert_eq!(r.held_share, Some(1.0));
        assert_eq!(r.window_bursts, 30);
        assert_eq!(r.verdict, Some(Verdict::Learned));
        assert_eq!(r.recalled_modal, r.robot_ref);
        assert_eq!(r.recalled_modal_share, Some(1.0));
        assert!(r.human_modal.is_some());
        assert_eq!(r.human_modal_share, Some(1.0));
        // learn_enable was never held in Check and the cue sat at home.
        let learn_before = brain.learn_held_bursts;
        assert_eq!(learn_before, 160);
    }

    #[test]
    fn check_unsure_and_not_learned() {
        let mut brain = Fake::new();
        let (plan, s) = learned(&mut brain);
        brain.human_from = brain.burst;
        brain.recall_hits = 6;
        let r = run_check(&mut brain, &plan, &s, 0).unwrap();
        assert_eq!(r.recall, Some(0.6));
        assert_eq!(r.verdict, Some(Verdict::Unsure));
        brain.human_from = brain.burst;
        brain.recall_hits = 2;
        let r = run_check(&mut brain, &plan, &s, 0).unwrap();
        assert_eq!(r.recall, Some(0.2));
        assert_eq!(r.verdict, Some(Verdict::NotLearned));
        // Recalled nothing in 24 of 30 bursts: the modal is the pose that did show, not the ref.
        assert!(r.robot_ref.is_some());
        assert!(
            r.recalled_modal_share.is_none_or(|s| s <= 0.2 + 1e-9),
            "{:?}",
            r.recalled_modal_share
        );
    }

    #[test]
    fn jitter_lowers_held_share_not_recall() {
        let mut brain = Fake::new();
        let (plan, s) = learned(&mut brain);
        brain.human_from = brain.burst;
        brain.jitter10 = 2; // human state wrong in 2 of every 10 bursts
        brain.recall_hits = 10;
        // t0 may land on a jitter-free burst; window is 30 bursts = 3 full decades.
        let r = run_check(&mut brain, &plan, &s, 0).unwrap();
        let held = r.held_share.unwrap();
        assert!((0.7..1.0).contains(&held), "held_share {held}");
        assert_eq!(r.recall, Some(1.0));
    }

    #[test]
    fn no_arrival_is_a_note_without_fallback() {
        let mut brain = Fake::new();
        let (plan, s) = learned(&mut brain);
        brain.human_from = brain.burst + 10_000;
        let r = run_check(&mut brain, &plan, &s, 0).unwrap();
        assert_eq!(r.note, Some(SlotNote::NoArrival));
        assert_eq!(r.recall, None);
        assert_eq!(r.verdict, None);
    }

    #[test]
    fn check_is_refused_without_learn_in_this_session() {
        let mut brain = Fake::new();
        let plan = start(&mut brain, cfg()).unwrap();
        let empty = LearnSession::default();
        let e = run_check(&mut brain, &plan, &empty, 0)
            .unwrap_err()
            .to_string();
        assert!(e.contains("refused"), "{e}");
        let (plan, s) = learned(&mut brain);
        // Skipped slot 2 has no Learn.
        assert!(run_check(&mut brain, &plan, &s, 2).is_err());
        assert!(run_check(&mut brain, &plan, &s, 9).is_err());
    }

    #[test]
    fn indistinguishable_slots_score_zero_without_a_hold() {
        let mut brain = Fake::new();
        brain.collapse_bin2 = true; // slots 0 and 1 end up with the same robot pose
        let (plan, s) = learned(&mut brain);
        let before = brain.burst;
        let r = run_check(&mut brain, &plan, &s, 1).unwrap();
        assert_eq!(r.note, Some(SlotNote::Indistinguishable));
        assert_eq!(r.recall, Some(0.0));
        assert_eq!(r.verdict, Some(Verdict::NotLearned));
        assert_eq!(brain.burst, before);
    }

    #[test]
    fn follow_is_not_ready_and_names_missing_roles() {
        let mut brain = Fake::new();
        let (plan, s) = learned(&mut brain);
        let r = run_follow(&plan, &s, 0, &[Role::SequencerQuiet, Role::FollowEnable]).unwrap();
        assert_eq!(
            r,
            FollowResult::NotReady {
                missing: vec!["sequencer_quiet".into(), "follow_enable".into()]
            }
        );
        assert!(run_follow(&plan, &s, 2, &[]).is_err());
    }

    #[test]
    fn patterns_need_exactly_one_bin_per_joint() {
        let strips = vec![StripPort {
            area: "a".into(),
            x: 0,
        }];
        let mut f = BurstFrame::default();
        assert_eq!(joint_pattern(&f, &strips), None);
        f.fired.insert("a".into(), vec![[0, 0, 2], [0, 0, 3]]);
        assert_eq!(joint_pattern(&f, &strips), None);
        f.fired.insert("a".into(), vec![[0, 0, 2], [1, 0, 3]]);
        assert_eq!(joint_pattern(&f, &strips), Some(vec![2]));
    }
}
