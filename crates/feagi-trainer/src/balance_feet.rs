//! Balance-on-feet co-agent loop.
//!
//! The embodiment controller knows no robot and no task: it offers generic physics
//! primitives (`get_model_info`, `reset_episode`, body springs, body pulses, `measure`).
//! This module owns every balance decision: spring gain sizing, the support level
//! schedule, settling, perturbation pulses, support need and effort, and scoring of torso
//! tilt plus all contact bodies on the floor. Death and a misc reset end the attempt. The
//! session continues until the phase seeds are done.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::binding::reward::{AffectChannel, RewardSignal};
use crate::contracts::{
    BackendFingerprint, BackendKind, ConnectomeHash, ContentHash, DatasetAssetId,
    EvaluationProtocolVersion, MetricScope, PluginId, PluginRef, RunEvent, RunEventKind, RunId,
    Scorecard, ScorecardId, ScorecardStatus, ScorecardVisibility, SplitId,
};
use crate::control::{CancelToken, RunEventSink};
use crate::error::TrainerError;
use crate::metrics::episodic_control::EpisodicControlMetricPack;
use crate::reach_hold::trainer_package_version;

fn default_support_deadband_deg() -> f64 {
    6.0
}
fn default_perturb_torque_nm() -> f64 {
    0.3
}
fn default_perturb_duration_s() -> f64 {
    0.1
}
fn default_support_start() -> f64 {
    1.0
}
fn default_support_step() -> f64 {
    0.1
}
fn default_support_min() -> f64 {
    0.0
}
fn default_settle_s() -> f64 {
    0.5
}
fn default_attempts_per_level() -> u32 {
    3
}
fn default_graduate_success_rate() -> f64 {
    0.66
}
fn default_burst_period_s() -> f64 {
    0.05
}
fn default_lookahead_s() -> f64 {
    0.1
}

/// Whole bursts that cover `seconds` at `burst_period_s` (at least 1). This is the one place
/// seconds become bursts, so a duration keeps its meaning when the burst rate changes.
pub fn bursts_for_seconds(seconds: f64, burst_period_s: f64) -> u32 {
    if !(seconds.is_finite() && burst_period_s.is_finite()) || burst_period_s <= 0.0 {
        return 1;
    }
    let bursts = (seconds / burst_period_s - 1.0e-9).ceil().max(1.0);
    if bursts >= f64::from(u32::MAX) {
        u32::MAX
    } else {
        bursts as u32
    }
}

/// Operator criterion for one balance-on-feet session.
///
/// The controller knows no robot and no task: this struct owns every decision. All durations
/// are in seconds of simulated time and are converted to bursts with the live burst period
/// (`burst_period_s`, set by the caller from the running FEAGI burst rate). Spring gains are
/// sized by the Trainer from the model (see [`size_spring_gains`]); each `support_*` gain
/// override that is `Some` wins over the sized value.
///
/// Every field after `attempt_cap_s` is optional on the wire and has a default, but Rust
/// struct literals must name them all: use [`BalanceFeetCriterion::new`] and override fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceFeetCriterion {
    /// Body whose +Z axis is compared with world up and which carries the support spring.
    pub torso_body: String,
    /// Bodies that must touch the world (any count). Upright needs every one down.
    pub contact_bodies: Vec<String>,
    /// Keyframe the controller resets to at the start of each attempt; empty = model default.
    #[serde(default)]
    pub reset_keyframe: String,
    /// Maximum lean from upright, in degrees, that still counts as balanced.
    pub max_tilt_deg: f64,
    /// Seconds of continuous balance that accept the attempt.
    pub hold_s: f64,
    /// Seconds after which an attempt that has not succeeded is truncated.
    pub attempt_cap_s: f64,
    /// Support level of the first train block (0..1). Default 1.0.
    #[serde(default = "default_support_start")]
    pub support_start: f64,
    /// Support lowered by this much after each graduating block. Default 0.1.
    #[serde(default = "default_support_step")]
    pub support_step: f64,
    /// Lowest support level. The test phase runs at this level. Default 0.0.
    #[serde(default = "default_support_min")]
    pub support_min: f64,
    /// Seconds the spring holds full support after a reset (samples flagged settling).
    #[serde(default = "default_settle_s")]
    pub settle_s: f64,
    /// Attempts per support block. Default 3.
    #[serde(default = "default_attempts_per_level")]
    pub attempts_per_level: u32,
    /// Block success rate needed to lower support. Default 0.66.
    #[serde(default = "default_graduate_success_rate")]
    pub graduate_success_rate: f64,
    /// Live FEAGI burst period in seconds; the caller sets it from the running burst rate.
    #[serde(default = "default_burst_period_s")]
    pub burst_period_s: f64,
    /// Tilt deadband in degrees inside which the spring torque is zero. Default 6.0.
    #[serde(default = "default_support_deadband_deg")]
    pub support_deadband_deg: f64,
    /// Lookahead in seconds of the support-need prediction (tilt + omega * lookahead). Default 0.1.
    #[serde(default = "default_lookahead_s")]
    pub need_lookahead_s: f64,
    /// Seconds between perturbation pulses (jittered +-50%); 0 turns perturbation off.
    #[serde(default)]
    pub perturb_interval_s: f64,
    /// Perturbation pulse torque, N*m. Default 0.3.
    #[serde(default = "default_perturb_torque_nm")]
    pub perturb_torque_nm: f64,
    /// Perturbation pulse length in seconds. Default 0.1.
    #[serde(default = "default_perturb_duration_s")]
    pub perturb_duration_s: f64,
    /// Graduation by time balanced: when `Some`, a support level passes when the MEDIAN
    /// `balanced_s` over that level's attempts is >= this value (seconds), instead of the
    /// success rate. `None` keeps the `graduate_success_rate` rule.
    #[serde(default)]
    pub graduate_balanced_s: Option<f64>,
    /// Override: torso CoM height above the mean contact point used for sizing, m.
    #[serde(default)]
    pub support_com_height_m: Option<f64>,
    /// Override: rotational stiffness k_rot, N*m/rad.
    #[serde(default)]
    pub support_rot_stiffness_nm_per_rad: Option<f64>,
    /// Override: rotational damping d_rot, N*m*s/rad.
    #[serde(default)]
    pub support_rot_damping_nm_s_per_rad: Option<f64>,
    /// Override: vertical stiffness k_lin_z, N/m.
    #[serde(default)]
    pub support_height_stiffness_n_per_m: Option<f64>,
    /// Override: vertical damping d_lin_z, N*s/m.
    #[serde(default)]
    pub support_height_damping_n_s_per_m: Option<f64>,
    /// Override: gravity compensation fraction (0..1). Default 1.0.
    #[serde(default)]
    pub support_gravity_comp: Option<f64>,
    /// Override: horizontal stiffness k_lin_xy, N/m.
    #[serde(default)]
    pub support_xy_stiffness_n_per_m: Option<f64>,
    /// Override: horizontal damping d_lin_xy, N*s/m.
    #[serde(default)]
    pub support_xy_damping_n_s_per_m: Option<f64>,
}

/// Fading-support schedule parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SupportConfig {
    /// First level.
    pub start: f64,
    /// Lowering step.
    pub step: f64,
    /// Lowest level.
    pub min: f64,
    /// Attempts per block.
    pub attempts_per_level: u32,
    /// Block success rate that lowers the level.
    pub graduate_success_rate: f64,
    /// When `Some`, the block's median `balanced_s` >= this lowers the level instead.
    pub graduate_balanced_s: Option<f64>,
}

impl Default for SupportConfig {
    fn default() -> Self {
        Self {
            start: 1.0,
            step: 0.1,
            min: 0.0,
            attempts_per_level: 3,
            graduate_success_rate: 0.66,
            graduate_balanced_s: None,
        }
    }
}

impl BalanceFeetCriterion {
    /// Criterion with the support, settle and perturbation fields at their defaults.
    pub fn new(
        torso_body: impl Into<String>,
        contact_bodies: Vec<String>,
        max_tilt_deg: f64,
        hold_s: f64,
        attempt_cap_s: f64,
    ) -> Self {
        Self {
            torso_body: torso_body.into(),
            contact_bodies,
            reset_keyframe: String::new(),
            max_tilt_deg,
            hold_s,
            attempt_cap_s,
            support_start: default_support_start(),
            support_step: default_support_step(),
            support_min: default_support_min(),
            settle_s: default_settle_s(),
            attempts_per_level: default_attempts_per_level(),
            graduate_success_rate: default_graduate_success_rate(),
            burst_period_s: default_burst_period_s(),
            support_deadband_deg: default_support_deadband_deg(),
            need_lookahead_s: default_lookahead_s(),
            perturb_interval_s: 0.0,
            perturb_torque_nm: default_perturb_torque_nm(),
            perturb_duration_s: default_perturb_duration_s(),
            graduate_balanced_s: None,
            support_com_height_m: None,
            support_rot_stiffness_nm_per_rad: None,
            support_rot_damping_nm_s_per_rad: None,
            support_height_stiffness_n_per_m: None,
            support_height_damping_n_s_per_m: None,
            support_gravity_comp: None,
            support_xy_stiffness_n_per_m: None,
            support_xy_damping_n_s_per_m: None,
        }
    }

    /// Bursts of continuous balance that accept an attempt, at the live burst period.
    pub fn hold_bursts(&self) -> u32 {
        bursts_for_seconds(self.hold_s, self.burst_period_s)
    }

    /// Bursts after which an unfinished attempt is truncated, at the live burst period.
    pub fn attempt_burst_cap(&self) -> u32 {
        bursts_for_seconds(self.attempt_cap_s, self.burst_period_s)
    }

    /// The schedule parameters of this criterion.
    pub fn support_config(&self) -> SupportConfig {
        SupportConfig {
            start: self.support_start,
            step: self.support_step,
            min: self.support_min,
            attempts_per_level: self.attempts_per_level,
            graduate_success_rate: self.graduate_success_rate,
            graduate_balanced_s: self.graduate_balanced_s,
        }
    }

    /// Rejects a criterion that cannot produce a success.
    pub fn validate(&self) -> Result<(), TrainerError> {
        let torso = self.torso_body.trim();
        if torso.is_empty() {
            return Err(TrainerError::Config(
                "torso body must be non-empty".to_string(),
            ));
        }
        if self.contact_bodies.is_empty() {
            return Err(TrainerError::Config(
                "contact_bodies must name at least one body".to_string(),
            ));
        }
        let mut seen: Vec<&str> = Vec::new();
        for name in &self.contact_bodies {
            let name = name.trim();
            if name.is_empty() {
                return Err(TrainerError::Config(
                    "contact_bodies must not contain an empty name".to_string(),
                ));
            }
            if seen.contains(&name) {
                return Err(TrainerError::Config(format!(
                    "contact body {name} is listed twice"
                )));
            }
            seen.push(name);
        }
        if !self.max_tilt_deg.is_finite() || self.max_tilt_deg <= 0.0 || self.max_tilt_deg > 90.0 {
            return Err(TrainerError::Config(
                "max_tilt_deg must be a finite value in (0, 90]".to_string(),
            ));
        }
        let positive = |value: f64| value.is_finite() && value > 0.0;
        let non_negative = |value: f64| value.is_finite() && value >= 0.0;
        if !positive(self.hold_s) {
            return Err(TrainerError::Config(
                "hold_s must be finite and > 0".to_string(),
            ));
        }
        if !positive(self.attempt_cap_s) {
            return Err(TrainerError::Config(
                "attempt_cap_s must be finite and > 0".to_string(),
            ));
        }
        if self.hold_s > self.attempt_cap_s {
            return Err(TrainerError::Config(
                "hold_s must be <= attempt_cap_s".to_string(),
            ));
        }
        self.support_config().validate()?;
        if !non_negative(self.settle_s) {
            return Err(TrainerError::Config(
                "settle_s must be finite and >= 0".to_string(),
            ));
        }
        if !positive(self.burst_period_s) {
            return Err(TrainerError::Config(
                "burst_period_s must be finite and > 0".to_string(),
            ));
        }
        if !non_negative(self.support_deadband_deg) {
            return Err(TrainerError::Config(
                "support_deadband_deg must be finite and >= 0".to_string(),
            ));
        }
        if !non_negative(self.need_lookahead_s) {
            return Err(TrainerError::Config(
                "need_lookahead_s must be finite and >= 0".to_string(),
            ));
        }
        if !non_negative(self.perturb_interval_s) {
            return Err(TrainerError::Config(
                "perturb_interval_s must be finite and >= 0".to_string(),
            ));
        }
        if !non_negative(self.perturb_torque_nm) {
            return Err(TrainerError::Config(
                "perturb_torque_nm must be finite and >= 0".to_string(),
            ));
        }
        if !positive(self.perturb_duration_s) {
            return Err(TrainerError::Config(
                "perturb_duration_s must be finite and > 0".to_string(),
            ));
        }
        let strictly_positive = [
            ("support_com_height_m", self.support_com_height_m),
            (
                "support_rot_stiffness_nm_per_rad",
                self.support_rot_stiffness_nm_per_rad,
            ),
            (
                "support_height_stiffness_n_per_m",
                self.support_height_stiffness_n_per_m,
            ),
            (
                "support_xy_stiffness_n_per_m",
                self.support_xy_stiffness_n_per_m,
            ),
        ];
        for (name, value) in strictly_positive {
            if value.is_some_and(|v| !positive(v)) {
                return Err(TrainerError::Config(format!(
                    "{name} must be finite and > 0"
                )));
            }
        }
        let damping = [
            (
                "support_rot_damping_nm_s_per_rad",
                self.support_rot_damping_nm_s_per_rad,
            ),
            (
                "support_height_damping_n_s_per_m",
                self.support_height_damping_n_s_per_m,
            ),
            (
                "support_xy_damping_n_s_per_m",
                self.support_xy_damping_n_s_per_m,
            ),
        ];
        for (name, value) in damping {
            if value.is_some_and(|v| !non_negative(v)) {
                return Err(TrainerError::Config(format!(
                    "{name} must be finite and >= 0"
                )));
            }
        }
        if self
            .support_gravity_comp
            .is_some_and(|v| !v.is_finite() || !(0.0..=1.0).contains(&v))
        {
            return Err(TrainerError::Config(
                "support_gravity_comp must be in [0, 1]".to_string(),
            ));
        }
        Ok(())
    }
}

impl SupportConfig {
    /// Checks 0 <= min <= start <= 1, 0 < step <= 1, 0 < rate <= 1, attempts_per_level > 0.
    pub fn validate(&self) -> Result<(), TrainerError> {
        let finite = [self.start, self.step, self.min, self.graduate_success_rate]
            .iter()
            .all(|value| value.is_finite());
        if !finite || !(0.0 <= self.min && self.min <= self.start && self.start <= 1.0) {
            return Err(TrainerError::Config(
                "support levels must satisfy 0 <= support_min <= support_start <= 1".to_string(),
            ));
        }
        if !(self.step > 0.0 && self.step <= 1.0) {
            return Err(TrainerError::Config(
                "support_step must be in (0, 1]".to_string(),
            ));
        }
        if !(self.graduate_success_rate > 0.0 && self.graduate_success_rate <= 1.0) {
            return Err(TrainerError::Config(
                "graduate_success_rate must be in (0, 1]".to_string(),
            ));
        }
        if self.attempts_per_level == 0 {
            return Err(TrainerError::Config(
                "attempts_per_level must be > 0".to_string(),
            ));
        }
        if self
            .graduate_balanced_s
            .is_some_and(|s| !s.is_finite() || s <= 0.0)
        {
            return Err(TrainerError::Config(
                "graduate_balanced_s must be finite and > 0".to_string(),
            ));
        }
        Ok(())
    }
}

fn median_f64(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 1 {
        sorted[mid]
    } else {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    }
}

fn round_level(level: f64) -> f64 {
    (level * 1.0e6).round() / 1.0e6
}

/// Assist-as-needed support schedule: lowers support after a successful block, holds after a
/// failed one, and raises one step only after two consecutive failed blocks.
#[derive(Debug, Clone, PartialEq)]
pub struct SupportSchedule {
    config: SupportConfig,
    level: f64,
    block_attempts: u32,
    block_successes: u32,
    block_balanced_s: Vec<f64>,
    failed_blocks_in_a_row: u32,
}

impl SupportSchedule {
    /// Schedule starting at `config.start`.
    pub fn new(config: SupportConfig) -> Self {
        Self {
            level: config.start,
            config,
            block_attempts: 0,
            block_successes: 0,
            block_balanced_s: Vec::new(),
            failed_blocks_in_a_row: 0,
        }
    }

    /// Level to send with the next attempt.
    pub fn current_level(&self) -> f64 {
        self.level
    }

    /// Fixed level of a test phase: `support_min`.
    pub fn for_test_phase(config: &SupportConfig) -> f64 {
        config.min
    }

    /// Fold one finished attempt into the current block with `balanced_s` of 0. Use
    /// [`SupportSchedule::record_attempt`] when graduating on time balanced.
    pub fn record(&mut self, outcome: BalanceAttemptOutcome) {
        self.record_attempt(outcome, 0.0);
    }

    /// Fold one finished attempt (outcome and its `balanced_s`) into the current block. With
    /// `graduate_balanced_s` set, the block passes on its median `balanced_s`; otherwise on
    /// its success rate.
    pub fn record_attempt(&mut self, outcome: BalanceAttemptOutcome, balanced_s: f64) {
        self.block_attempts += 1;
        self.block_balanced_s.push(balanced_s);
        if outcome == BalanceAttemptOutcome::Success {
            self.block_successes += 1;
        }
        if self.block_attempts < self.config.attempts_per_level {
            return;
        }
        let rate = f64::from(self.block_successes) / f64::from(self.block_attempts);
        let passed = match self.config.graduate_balanced_s {
            Some(threshold) => median_f64(&self.block_balanced_s) >= threshold,
            None => rate >= self.config.graduate_success_rate,
        };
        self.block_attempts = 0;
        self.block_successes = 0;
        self.block_balanced_s.clear();
        if passed {
            self.failed_blocks_in_a_row = 0;
            self.level = round_level((self.level - self.config.step).max(self.config.min));
        } else {
            self.failed_blocks_in_a_row += 1;
            if self.failed_blocks_in_a_row >= 2 {
                self.failed_blocks_in_a_row = 0;
                self.level = round_level((self.level + self.config.step).min(self.config.start));
            }
        }
    }
}

/// How a phase picks the support level of each attempt.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SupportMode {
    /// Follow a [`SupportSchedule`] built from the criterion.
    Schedule,
    /// Every attempt at this level.
    Fixed(f64),
}

/// Why one attempt ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BalanceAttemptOutcome {
    /// Tilt stayed inside the limit and every contact body stayed down for the required bursts.
    Success,
    /// `_death` fired. The controller homes the body and the session continues.
    Death,
    /// A misc reset interrupted the attempt.
    Interrupted,
    /// The burst cap was reached without a hold.
    Truncated,
}

/// One finished attempt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceAttemptRecord {
    /// Seed the controller used to home this attempt.
    pub seed: u64,
    /// How the attempt ended.
    pub outcome: BalanceAttemptOutcome,
    /// Non-settling bursts observed, including the terminal burst.
    pub bursts: u32,
    /// Torso tilt at the terminal burst, in degrees.
    pub final_tilt_deg: f64,
    /// Support level of the attempt.
    #[serde(default)]
    pub support_level: f64,
    /// Fraction of non-settling bursts with tilt inside the limit and every contact body down.
    #[serde(default)]
    pub upright_fraction: f64,
    /// Distinct perturbation pulses seen (rising edges of `perturbing`).
    #[serde(default)]
    pub perturbations: u32,
    /// Balance metric: the longest continuous run of upright bursts (tilt inside the limit and
    /// every contact body down), counted in elapsed bursts, not samples. A `Death` burst never counts.
    #[serde(default)]
    pub balanced_bursts: u32,
    /// `balanced_bursts` times the criterion's `burst_period_s`.
    #[serde(default)]
    pub balanced_s: f64,
    /// Non-settling bursts elapsed before the first non-upright burst, measured from the end of
    /// settling. Absent when the attempt never left the upright band.
    #[serde(default)]
    pub time_to_first_violation_bursts: Option<u32>,
    /// `time_to_first_violation_bursts` times `burst_period_s`.
    #[serde(default)]
    pub time_to_first_violation_s: Option<f64>,
}

/// Model facts the Trainer needs, from the controller's `get_model_info`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ModelInfo {
    /// Physics timestep, seconds.
    pub timestep_s: f64,
    /// World gravity vector, m/s^2.
    pub gravity: [f64; 3],
    /// Every body of the loaded model.
    #[serde(default)]
    pub bodies: Vec<ModelBody>,
    /// Names of bodies that own a free joint.
    #[serde(default)]
    pub free_roots: Vec<String>,
    /// Keyframe names (a bare entry is an unnamed keyframe).
    #[serde(default)]
    pub keyframes: Vec<serde_json::Value>,
}

/// One body of [`ModelInfo`].
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ModelBody {
    /// Body name.
    pub name: String,
    /// Name of the free-joint root this body belongs to.
    #[serde(default)]
    pub root_name: String,
    /// Mass of the body alone, kg.
    #[serde(default)]
    pub mass: f64,
    /// Mass of the body and everything below it, kg.
    #[serde(default)]
    pub subtree_mass: f64,
    /// Principal inertia of the body alone, kg*m^2.
    #[serde(default)]
    pub inertia: [f64; 3],
    /// Inertia of the subtree about its own CoM, diagonal in the body frame, kg*m^2.
    #[serde(default)]
    pub subtree_inertia: [f64; 3],
}

impl ModelInfo {
    /// Body by name.
    pub fn body(&self, name: &str) -> Option<&ModelBody> {
        self.bodies.iter().find(|body| body.name == name)
    }

    /// Names of the named keyframes.
    pub fn keyframe_names(&self) -> Vec<String> {
        self.keyframes
            .iter()
            .filter_map(|entry| match entry {
                serde_json::Value::String(name) => Some(name.clone()),
                serde_json::Value::Object(map) => map
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string),
                _ => None,
            })
            .filter(|name| !name.is_empty())
            .collect()
    }
}

/// Inputs of the documented gain-sizing formulas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SizingInputs {
    /// Robot mass m: subtree mass of the torso's free root, kg.
    pub mass_kg: f64,
    /// Gravity magnitude g, m/s^2.
    pub gravity: f64,
    /// Torso subtree CoM height above the mean contact point h0, m.
    pub h0_m: f64,
    /// Tilt inertia: max(x, y) of the torso root's subtree inertia, kg*m^2.
    pub i_tilt: f64,
    /// Smallest principal inertia of the torso root's subtree, kg*m^2.
    pub i_min: f64,
    /// Smallest principal inertia of the torso body alone (limp-limb stability limit), kg*m^2.
    pub i_own_min: f64,
    /// Physics timestep dt, s.
    pub dt_s: f64,
}

/// Spring parameters sent to the controller (all lengths m, angles rad).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpringGains {
    /// Rotational stiffness, N*m/rad.
    pub k_rot: f64,
    /// Rotational damping, N*m*s/rad.
    pub d_rot: f64,
    /// Linear stiffness (x, y, z), N/m.
    pub k_lin: [f64; 3],
    /// Linear damping (x, y, z), N*s/m.
    pub d_lin: [f64; 3],
    /// Fraction of the body's root weight the spring carries.
    pub gravity_comp: f64,
}

/// Size the support spring (documented formulas; `gravity_comp` is 1.0 here):
///
/// `k_rot = 3 m g h0 / 0.087` (a 5 degree tilt costs about a third of the weight torque),
/// `d_rot = min(0.4 * 2 sqrt(k_rot I_tilt), 0.9 * 2 I_min / dt)` (the second term keeps the
/// explicit-force integration stable), `k_z = 8 m g / 0.005`, `d_z = 0.33 m / dt`,
/// `k_xy = 2.8 m g / 0.005`, `d_xy = 0.22 m / dt`. `I_tilt` and `I_min` come from the torso
/// root's subtree inertia. Finally k_rot and d_rot are capped by the torso body's own
/// inertia (`d dt / I_own < 2` with 10% margin, `k dt^2 / I_own <= 0.25`) so limp-limb
/// models do not diverge.
pub fn size_spring_gains(inputs: &SizingInputs) -> SpringGains {
    let weight = inputs.mass_kg * inputs.gravity;
    let k_rot = cap_k_rot(3.0 * weight * inputs.h0_m / 0.087, inputs);
    let d_rot = sized_d_rot(k_rot, inputs);
    let k_z = 8.0 * weight / 0.005;
    let d_z = 0.33 * inputs.mass_kg / inputs.dt_s;
    let k_xy = 2.8 * weight / 0.005;
    let d_xy = 0.22 * inputs.mass_kg / inputs.dt_s;
    SpringGains {
        k_rot,
        d_rot,
        k_lin: [k_xy, k_xy, k_z],
        d_lin: [d_xy, d_xy, d_z],
        gravity_comp: 1.0,
    }
}

fn cap_k_rot(k_rot: f64, inputs: &SizingInputs) -> f64 {
    k_rot.min(0.25 * inputs.i_own_min / (inputs.dt_s * inputs.dt_s))
}

fn sized_d_rot(k_rot: f64, inputs: &SizingInputs) -> f64 {
    (0.4 * 2.0 * (k_rot * inputs.i_tilt).sqrt())
        .min(0.9 * 2.0 * inputs.i_min / inputs.dt_s)
        .min(0.9 * 2.0 * inputs.i_own_min / inputs.dt_s)
}

/// Sized gains with the criterion's explicit overrides applied. An overridden `k_rot` feeds the
/// `d_rot` formula; an overridden `d_rot` replaces it.
pub fn gains_for(inputs: &SizingInputs, criterion: &BalanceFeetCriterion) -> SpringGains {
    let mut gains = size_spring_gains(inputs);
    if let Some(k_rot) = criterion.support_rot_stiffness_nm_per_rad {
        gains.k_rot = k_rot;
        gains.d_rot = sized_d_rot(k_rot, inputs);
    }
    if let Some(d_rot) = criterion.support_rot_damping_nm_s_per_rad {
        gains.d_rot = d_rot;
    }
    if let Some(k_z) = criterion.support_height_stiffness_n_per_m {
        gains.k_lin[2] = k_z;
    }
    if let Some(d_z) = criterion.support_height_damping_n_s_per_m {
        gains.d_lin[2] = d_z;
    }
    if let Some(k_xy) = criterion.support_xy_stiffness_n_per_m {
        gains.k_lin[0] = k_xy;
        gains.k_lin[1] = k_xy;
    }
    if let Some(d_xy) = criterion.support_xy_damping_n_s_per_m {
        gains.d_lin[0] = d_xy;
        gains.d_lin[1] = d_xy;
    }
    if let Some(gc) = criterion.support_gravity_comp {
        gains.gravity_comp = gc;
    }
    gains
}

/// Sizing inputs from the model and the measured CoM height `h0_m`. Every quantity must be
/// positive; the error names the one that is not.
pub fn sizing_inputs(
    info: &ModelInfo,
    torso_body: &str,
    h0_m: f64,
) -> Result<SizingInputs, TrainerError> {
    let torso = info.body(torso_body).ok_or_else(|| {
        TrainerError::Config(format!(
            "torso body {torso_body} is not in the loaded model"
        ))
    })?;
    let root = if torso.root_name.is_empty() {
        torso
    } else {
        info.body(&torso.root_name).ok_or_else(|| {
            TrainerError::Runtime(format!(
                "root body {} of {torso_body} is missing from get_model_info",
                torso.root_name
            ))
        })?
    };
    let gravity = info.gravity.iter().map(|g| g * g).sum::<f64>().sqrt();
    let min3 = |v: [f64; 3]| v.iter().copied().fold(f64::INFINITY, f64::min);
    let inputs = SizingInputs {
        mass_kg: root.subtree_mass,
        gravity,
        h0_m,
        i_tilt: root.subtree_inertia[0].max(root.subtree_inertia[1]),
        i_min: min3(root.subtree_inertia),
        i_own_min: min3(torso.inertia),
        dt_s: info.timestep_s,
    };
    for (name, value) in [
        ("root subtree mass", inputs.mass_kg),
        ("gravity", inputs.gravity),
        ("CoM height above the contacts", inputs.h0_m),
        ("torso root subtree tilt inertia", inputs.i_tilt),
        ("torso root subtree minimum inertia", inputs.i_min),
        ("torso own minimum inertia", inputs.i_own_min),
        ("physics timestep", inputs.dt_s),
    ] {
        if !value.is_finite() || value <= 0.0 {
            return Err(TrainerError::Runtime(format!(
                "cannot size the support spring: {name} is {value}"
            )));
        }
    }
    Ok(inputs)
}

/// What to read in one `measure` call.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasureRequest {
    /// Bodies to report.
    pub bodies: Vec<String>,
    /// One contact set per entry: touching between these bodies and the world.
    pub contacts: Vec<Vec<String>>,
}

/// One body in a `measure` reply.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BodyMeasure {
    /// Angle between the body-local up axis and -gravity, rad.
    pub tilt_rad: f64,
    /// Tilt vector, rad.
    #[serde(default)]
    pub tilt_vec: [f64; 2],
    /// Angular velocity, rad/s.
    #[serde(default)]
    pub ang_vel: [f64; 3],
    /// Body CoM, m.
    #[serde(default)]
    pub com: [f64; 3],
    /// Subtree CoM, m.
    #[serde(default)]
    pub subtree_com: [f64; 3],
}

/// One contact set in a `measure` reply.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ContactMeasure {
    /// The set touches the world.
    pub touching: bool,
    /// Contact points, m.
    #[serde(default)]
    pub points: Vec<[f64; 3]>,
}

/// Norms of the wrench one spring applies right now.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct SpringMeasure {
    /// Torque norm, N*m.
    pub wrench_norm_torque: f64,
    /// Restoring force norm, N (gravity compensation excluded).
    pub wrench_norm_force: f64,
    /// Gravity-compensation force, N (reported separately).
    #[serde(default)]
    pub gravity_comp_force_n: f64,
}

/// A `measure` reply.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Measurement {
    /// Simulated time, seconds.
    pub sim_time_s: f64,
    /// Physics steps since the model was loaded.
    pub physics_steps: u64,
    /// Episode the reading belongs to.
    #[serde(default)]
    pub episode_id: u64,
    /// Latched end cause of the episode, if one happened.
    #[serde(default)]
    pub terminal: Option<BalanceTerminal>,
    /// Reported bodies by name (an object, or an array of objects carrying `name`).
    #[serde(default, deserialize_with = "named_map")]
    pub bodies: BTreeMap<String, BodyMeasure>,
    /// One entry per requested contact set, in request order.
    #[serde(default)]
    pub contacts: Vec<ContactMeasure>,
    /// Active springs by body name.
    #[serde(default, deserialize_with = "named_map")]
    pub springs: BTreeMap<String, SpringMeasure>,
}

fn named_map<'de, D, T>(deserializer: D) -> Result<BTreeMap<String, T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::de::DeserializeOwned,
{
    use serde::de::Error;
    let value = serde_json::Value::deserialize(deserializer)?;
    let mut out = BTreeMap::new();
    match value {
        serde_json::Value::Null => {}
        serde_json::Value::Object(map) => {
            for (name, entry) in map {
                out.insert(
                    name,
                    serde_json::from_value(entry).map_err(D::Error::custom)?,
                );
            }
        }
        serde_json::Value::Array(entries) => {
            for entry in entries {
                let name = entry
                    .get("name")
                    .or_else(|| entry.get("body"))
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| D::Error::custom("entry has no name"))?
                    .to_string();
                out.insert(
                    name,
                    serde_json::from_value(entry).map_err(D::Error::custom)?,
                );
            }
        }
        _ => return Err(D::Error::custom("expected an object or an array")),
    }
    Ok(out)
}

impl Measurement {
    /// Mean height of every contact point of every set, if any point exists.
    pub fn mean_contact_height(&self) -> Option<f64> {
        let heights: Vec<f64> = self
            .contacts
            .iter()
            .flat_map(|set| set.points.iter().map(|point| point[2]))
            .collect();
        if heights.is_empty() {
            None
        } else {
            Some(heights.iter().sum::<f64>() / heights.len() as f64)
        }
    }
}

/// One stance measurement, derived by the Trainer from a [`Measurement`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceSample {
    /// Monotonic physics-step counter of the controller.
    pub seq: u64,
    /// Attempt seed.
    pub seed: u64,
    /// Torso tilt from world up, degrees.
    pub tilt_deg: f64,
    /// Per contact body, in `contact_bodies` order: touching the world.
    pub contacts: Vec<bool>,
    /// Latched end cause. Absent while the attempt is still open.
    pub terminal: Option<BalanceTerminal>,
    /// Effective support level at this step.
    #[serde(default)]
    pub support_level: Option<f64>,
    /// The settle period is still running.
    #[serde(default)]
    pub settling: bool,
    /// A perturbation pulse is being applied at this step.
    #[serde(default)]
    pub perturbing: bool,
    /// Predicted tilt fraction 0..1 (tilt + omega * lookahead vs the tilt limit).
    #[serde(default)]
    pub support_need: Option<f64>,
    /// Spring effort 0..1 from the wrench norms.
    #[serde(default)]
    pub support_effort: Option<f64>,
    /// Spring torque norm, N*m.
    #[serde(default)]
    pub support_torque_nm: Option<f64>,
    /// Spring force norm, N.
    #[serde(default)]
    pub support_force_n: Option<f64>,
}

impl BalanceSample {
    /// Every contact body touches the world.
    pub fn all_contacts_down(&self) -> bool {
        self.contacts.iter().all(|down| *down)
    }
}

/// Predicted tilt fraction `min(1, (tilt + lookahead * |omega_xy|) / max_tilt)`.
pub fn support_need(tilt_rad: f64, ang_vel: [f64; 3], max_tilt_deg: f64, lookahead_s: f64) -> f64 {
    let ahead = tilt_rad + lookahead_s * ang_vel[0].hypot(ang_vel[1]);
    (ahead / max_tilt_deg.to_radians()).clamp(0.0, 1.0)
}

/// Mean of the clipped torque and force shares of the weight-tipping budget:
/// `(min(1, |tau| / (m g h0)) + min(1, |F_restoring| / (m g))) / 2`. The controller reports
/// the restoring force without the gravity-compensation share.
pub fn support_effort(wrench: SpringMeasure, sizing: &SizingInputs) -> f64 {
    let weight = (sizing.mass_kg * sizing.gravity).max(1.0e-9);
    let torque_share = (wrench.wrench_norm_torque / (weight * sizing.h0_m.max(1.0e-9))).min(1.0);
    let force_share = (wrench.wrench_norm_force / weight).min(1.0);
    (torque_share + force_share) / 2.0
}

/// Controller-latched end cause. Hold success is decided here, not by the controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BalanceTerminal {
    /// `_death` fired.
    Death,
    /// A misc reset fired while the episode was active.
    Interrupted,
}

/// In-progress attempt counters.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenBalanceAttempt {
    /// Non-settling bursts already observed (counts toward the cap).
    pub bursts: u32,
    /// Consecutive balanced bursts ending at the latest counted burst.
    pub holding_bursts: u32,
    /// Latest torso tilt, degrees.
    pub last_tilt_deg: f64,
    /// Latest controller step counter seen (counted or not).
    pub last_seq: u64,
    /// Step counter of the last sample that was counted as a burst, if any.
    pub last_burst_seq: Option<f64>,
    /// Settling bursts observed (logged, never scored).
    pub settling_bursts: u32,
    /// Non-settling bursts that were upright (tilt inside the limit, all contacts down).
    pub upright_bursts: u32,
    /// The latest sample was counted as a new burst (false when decimated away).
    pub counted_burst: bool,
    /// Physics steps per burst: `burst_period_s / timestep_s` (not necessarily whole).
    pub steps_per_burst: f64,
    /// Support level of the attempt, used when a sample carries none.
    pub params_support_level: f64,
    /// Distinct perturbation pulses seen so far.
    pub perturbations: u32,
    /// The latest sample (counted or not) had `perturbing` set.
    pub last_perturbing: bool,
    /// Longest continuous run of upright bursts so far (elapsed bursts).
    pub longest_balanced_bursts: u32,
    /// Non-settling bursts elapsed before the first non-upright burst, if one happened.
    pub first_violation_burst: Option<u32>,
}

impl OpenBalanceAttempt {
    /// An attempt for `support_level` with `steps_per_burst` physics steps per burst.
    pub fn for_attempt(steps_per_burst: f64, support_level: f64) -> Self {
        Self {
            steps_per_burst: if steps_per_burst.is_finite() && steps_per_burst > 0.0 {
                steps_per_burst
            } else {
                1.0
            },
            params_support_level: support_level,
            ..Self::fresh()
        }
    }

    /// An attempt that has not observed a sample yet (one physics step per burst).
    pub fn fresh() -> Self {
        Self {
            bursts: 0,
            holding_bursts: 0,
            last_tilt_deg: 0.0,
            last_seq: 0,
            last_burst_seq: None,
            settling_bursts: 0,
            upright_bursts: 0,
            counted_burst: false,
            steps_per_burst: 1.0,
            params_support_level: 0.0,
            perturbations: 0,
            last_perturbing: false,
            longest_balanced_bursts: 0,
            first_violation_burst: None,
        }
    }
}

/// Result of folding one sample into an open attempt.
#[derive(Debug, Clone, PartialEq)]
pub enum BalanceAttemptStep {
    /// The attempt is still running.
    Continue(OpenBalanceAttempt),
    /// The attempt ended.
    Finished(BalanceAttemptRecord),
}

/// Roll and pitch sit at the middle of the unit range. Feet are fully down.
pub fn upright_goal_features() -> [f64; 3] {
    [0.5, 0.5, 1.0]
}

/// Tilt inside the limit with every contact body down.
pub fn sample_is_upright(sample: &BalanceSample, criterion: &BalanceFeetCriterion) -> bool {
    sample.tilt_deg <= criterion.max_tilt_deg && sample.all_contacts_down()
}

/// Fold one controller sample into the attempt.
///
/// `seq` is the controller's physics-step counter. With N steps per burst
/// (`burst_period_s / timestep_s`, not necessarily whole), `k = floor((seq - last_burst_seq) / N)`
/// new bursts have elapsed since the last counted burst; `last_burst_seq` advances by `k * N` and
/// `bursts` by `k`, so a late reading does not undercount. A sample with `k >= 1` is one counted
/// observation for hold and reward purposes (the first sample always counts as one burst). Terminal samples are always processed. Samples with `settling == true`
/// never advance the hold counter and never count toward `attempt_burst_cap`.
pub fn observe_balance_sample(
    open: &OpenBalanceAttempt,
    sample: &BalanceSample,
    criterion: &BalanceFeetCriterion,
    seed: u64,
) -> Result<BalanceAttemptStep, TrainerError> {
    if sample.seed != seed {
        return Err(TrainerError::Runtime(format!(
            "balance sample seed {} does not match attempt seed {seed}",
            sample.seed
        )));
    }
    if !sample.tilt_deg.is_finite() || sample.tilt_deg < 0.0 {
        return Err(TrainerError::Runtime(format!(
            "tilt_deg must be a finite value >= 0, got {}",
            sample.tilt_deg
        )));
    }
    let upright = sample_is_upright(sample, criterion);
    let perturbations = open
        .perturbations
        .saturating_add(u32::from(sample.perturbing && !open.last_perturbing));
    let last_perturbing = sample.perturbing;
    if let Some(terminal) = sample.terminal {
        let outcome = match terminal {
            BalanceTerminal::Death => BalanceAttemptOutcome::Death,
            BalanceTerminal::Interrupted => BalanceAttemptOutcome::Interrupted,
        };
        let bursts = if sample.settling {
            open.bursts
        } else {
            open.bursts.saturating_add(1)
        };
        let upright_bursts = open.upright_bursts + u32::from(!sample.settling && upright);
        // A Death burst is a balance failure even if the last tilt reading was inside the limit.
        let balanced = upright && terminal != BalanceTerminal::Death;
        let (run, first_violation) = if sample.settling {
            (open.holding_bursts, open.first_violation_burst)
        } else if balanced {
            (
                open.holding_bursts.saturating_add(1),
                open.first_violation_burst,
            )
        } else {
            (0, open.first_violation_burst.or(Some(open.bursts)))
        };
        let longest = open.longest_balanced_bursts.max(run);
        return Ok(BalanceAttemptStep::Finished(BalanceAttemptRecord {
            seed,
            outcome,
            bursts,
            final_tilt_deg: sample.tilt_deg,
            support_level: sample.support_level.unwrap_or(open.params_support_level),
            upright_fraction: upright_fraction(upright_bursts, bursts),
            perturbations,
            ..balance_time_fields(longest, first_violation, criterion.burst_period_s)
        }));
    }
    let steps = open.steps_per_burst.max(1.0e-9);
    let seq_now = sample.seq as f64;
    let (new_bursts, anchor): (u32, f64) = match open.last_burst_seq {
        None => (1, seq_now),
        Some(last) => {
            let k = ((seq_now - last) / steps + 1.0e-9).floor().max(0.0);
            (
                if k >= f64::from(u32::MAX) {
                    u32::MAX
                } else {
                    k as u32
                },
                last + k * steps,
            )
        }
    };
    if new_bursts == 0 {
        let mut next = open.clone();
        next.last_seq = sample.seq.max(open.last_seq);
        next.counted_burst = false;
        next.perturbations = perturbations;
        next.last_perturbing = last_perturbing;
        return Ok(BalanceAttemptStep::Continue(next));
    }
    if sample.settling {
        let mut next = open.clone();
        next.settling_bursts = open.settling_bursts.saturating_add(new_bursts);
        next.last_tilt_deg = sample.tilt_deg;
        next.last_seq = sample.seq;
        next.last_burst_seq = Some(anchor);
        next.counted_burst = true;
        next.perturbations = perturbations;
        next.last_perturbing = last_perturbing;
        return Ok(BalanceAttemptStep::Continue(next));
    }
    let bursts = open.bursts.saturating_add(new_bursts);
    let upright_bursts = open
        .upright_bursts
        .saturating_add(if upright { new_bursts } else { 0 });
    // Same unit as `bursts` and `upright_bursts`: elapsed bursts, so slow polls do not
    // stretch the hold.
    let holding_bursts = if upright {
        open.holding_bursts.saturating_add(new_bursts)
    } else {
        0
    };
    let longest_balanced_bursts = open.longest_balanced_bursts.max(holding_bursts);
    let first_violation_burst =
        open.first_violation_burst
            .or(if upright { None } else { Some(open.bursts) });
    let finish = |outcome| {
        BalanceAttemptStep::Finished(BalanceAttemptRecord {
            seed,
            outcome,
            bursts,
            final_tilt_deg: sample.tilt_deg,
            support_level: sample.support_level.unwrap_or(open.params_support_level),
            upright_fraction: upright_fraction(upright_bursts, bursts),
            perturbations,
            ..balance_time_fields(
                longest_balanced_bursts,
                first_violation_burst,
                criterion.burst_period_s,
            )
        })
    };
    if holding_bursts >= criterion.hold_bursts() {
        return Ok(finish(BalanceAttemptOutcome::Success));
    }
    if bursts >= criterion.attempt_burst_cap() {
        return Ok(finish(BalanceAttemptOutcome::Truncated));
    }
    Ok(BalanceAttemptStep::Continue(OpenBalanceAttempt {
        bursts,
        holding_bursts,
        last_tilt_deg: sample.tilt_deg,
        last_seq: sample.seq,
        last_burst_seq: Some(anchor),
        settling_bursts: open.settling_bursts,
        upright_bursts,
        counted_burst: true,
        steps_per_burst: open.steps_per_burst,
        params_support_level: open.params_support_level,
        perturbations,
        last_perturbing,
        longest_balanced_bursts,
        first_violation_burst,
    }))
}

/// Record carrying only the balance-time fields; callers fill the rest with struct update.
fn balance_time_fields(
    longest: u32,
    first_violation: Option<u32>,
    burst_period_s: f64,
) -> BalanceAttemptRecord {
    BalanceAttemptRecord {
        seed: 0,
        outcome: BalanceAttemptOutcome::Truncated,
        bursts: 0,
        final_tilt_deg: 0.0,
        support_level: 0.0,
        upright_fraction: 0.0,
        perturbations: 0,
        balanced_bursts: longest,
        balanced_s: f64::from(longest) * burst_period_s,
        time_to_first_violation_bursts: first_violation,
        time_to_first_violation_s: first_violation.map(|b| f64::from(b) * burst_period_s),
    }
}

fn upright_fraction(upright_bursts: u32, bursts: u32) -> f64 {
    if bursts == 0 {
        0.0
    } else {
        f64::from(upright_bursts) / f64::from(bursts)
    }
}

/// Affect events for one balance attempt. Create one per attempt and feed it each decimated
/// burst (see [`OpenBalanceAttempt::counted_burst`]).
///
/// Rules: tilt crossing up through `max_tilt/2` -> Pain 0.5; through `max_tilt` -> Pain 1.0;
/// Death -> Pain 1.0 (once per burst); tilt back below `max_tilt/2` after exceeding it ->
/// Pleasure 0.5; Success -> Pleasure 1.0; each other upright burst at `support_level == 0`
/// -> Pleasure 0.1. Every Pleasure is scaled by `1 - support_level` (sample's level, else the
/// attempt's) and dropped below 0.05; Pain is unscaled. Settling bursts emit nothing, except a Death terminal.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BalanceAffectReward {
    previous_tilt_deg: f64,
    exceeded_half: bool,
    attempt_support_level: f64,
}

impl BalanceAffectReward {
    /// Policy at the start of an attempt at support 0.
    pub fn new() -> Self {
        Self::default()
    }

    /// Policy for an attempt at `support_level`; used when a sample reports none.
    pub fn for_attempt(support_level: f64) -> Self {
        Self {
            attempt_support_level: support_level,
            ..Self::default()
        }
    }

    /// Stable identity recorded with the run.
    pub fn policy_id() -> &'static str {
        "reward.balance-affect.v1"
    }

    /// Signals for one burst. `outcome` is the attempt outcome when this burst ended it.
    pub fn on_sample(
        &mut self,
        burst_sample: &BalanceSample,
        outcome: Option<&BalanceAttemptOutcome>,
        criterion: &BalanceFeetCriterion,
    ) -> Vec<RewardSignal> {
        let tilt = burst_sample.tilt_deg;
        let previous = self.previous_tilt_deg;
        self.previous_tilt_deg = tilt;
        let death = outcome == Some(&BalanceAttemptOutcome::Death);
        let pain = |magnitude| RewardSignal {
            channel: AffectChannel::Pain,
            magnitude,
        };
        let support = burst_sample
            .support_level
            .unwrap_or(self.attempt_support_level)
            .clamp(0.0, 1.0);
        let scale = 1.0 - support;
        let pleasure = |magnitude: f64| RewardSignal {
            channel: AffectChannel::Pleasure,
            magnitude: magnitude * scale,
        };
        if burst_sample.settling {
            return if death { vec![pain(1.0)] } else { Vec::new() };
        }
        let half = criterion.max_tilt_deg / 2.0;
        let mut signals = Vec::new();
        if tilt > criterion.max_tilt_deg && previous <= criterion.max_tilt_deg {
            self.exceeded_half = true;
            signals.push(pain(1.0));
        } else if tilt > half && previous <= half {
            self.exceeded_half = true;
            signals.push(pain(0.5));
        } else if tilt > half {
            self.exceeded_half = true;
        } else if self.exceeded_half {
            self.exceeded_half = false;
            signals.push(pleasure(0.5));
        }
        if death
            && !signals
                .iter()
                .any(|s| s.channel == AffectChannel::Pain && s.magnitude >= 1.0)
        {
            signals.push(pain(1.0));
        }
        if outcome == Some(&BalanceAttemptOutcome::Success) {
            signals.push(pleasure(1.0));
        } else if !death
            && outcome.is_none()
            && burst_sample
                .support_level
                .is_some_and(|level| level <= 1.0e-9)
            && sample_is_upright(burst_sample, criterion)
        {
            signals.push(pleasure(0.1));
        }
        signals.retain(|s| s.channel != AffectChannel::Pleasure || s.magnitude >= 0.05);
        signals
    }
}

/// Need at or below this fraction counts as balanced (no Pain, holding Pleasure applies).
pub const ASSIST_NEED_DEADBAND: f64 = 0.1;
/// Pleasure per unit of need reduction between consecutive bursts: `min(1, GAIN * drop)`.
pub const ASSIST_FALL_GAIN: f64 = 4.0;
/// Pleasure each burst that need is inside the deadband with every contact body down.
pub const ASSIST_HOLD_PLEASURE: f64 = 0.2;
/// Pleasure below this magnitude is dropped.
pub const ASSIST_MIN_PLEASURE: f64 = 0.05;

/// Which affect policy scores a balance attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BalanceRewardPolicy {
    /// [`BalanceAffectReward`] (mode name `affect`).
    #[default]
    Affect,
    /// [`BalanceAssistReward`] (mode name `assist`).
    Assist,
}

impl BalanceRewardPolicy {
    /// Parse a reward mode name; `none` has no policy and returns `None`.
    pub fn from_mode(mode: &str) -> Option<Self> {
        match mode {
            "affect" => Some(Self::Affect),
            "assist" => Some(Self::Assist),
            _ => None,
        }
    }

    /// Policy identity recorded with the run.
    pub fn policy_id(self) -> &'static str {
        match self {
            Self::Affect => BalanceAffectReward::policy_id(),
            Self::Assist => BalanceAssistReward::policy_id(),
        }
    }
}

/// Assist-as-needed affect for one balance attempt, driven by the harness `support_need`
/// (falls back to `min(1, tilt / max_tilt)` when absent). Per counted, non-settling burst with
/// need `n_t` and previous need `n_{t-1}`: Death -> Pain 1.0; `n_t > ASSIST_NEED_DEADBAND` and
/// rising -> Pain `min(1, n_t)`; falling -> Pleasure `min(1, ASSIST_FALL_GAIN * drop)`;
/// `n_t <= deadband` with every contact body down -> Pleasure `ASSIST_HOLD_PLEASURE`; Success ->
/// Pleasure 1.0. When several Pleasures apply in one burst the largest is emitted. Never scaled
/// by support level. Pleasure below `ASSIST_MIN_PLEASURE` is dropped. Settling bursts emit
/// nothing except a Death terminal (their need still seeds `n_{t-1}`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BalanceAssistReward {
    previous_need: Option<f64>,
}

impl BalanceAssistReward {
    /// Policy at the start of an attempt.
    pub fn new() -> Self {
        Self::default()
    }

    /// Stable identity recorded with the run.
    pub fn policy_id() -> &'static str {
        "reward.balance-assist.v1"
    }

    /// Signals for one burst. `outcome` is the attempt outcome when this burst ended it.
    pub fn on_sample(
        &mut self,
        burst_sample: &BalanceSample,
        outcome: Option<&BalanceAttemptOutcome>,
        criterion: &BalanceFeetCriterion,
    ) -> Vec<RewardSignal> {
        let need = burst_sample
            .support_need
            .unwrap_or_else(|| burst_sample.tilt_deg / criterion.max_tilt_deg)
            .clamp(0.0, 1.0);
        let previous = self.previous_need.unwrap_or(need);
        self.previous_need = Some(need);
        let death = outcome == Some(&BalanceAttemptOutcome::Death);
        let pain = |magnitude| RewardSignal {
            channel: AffectChannel::Pain,
            magnitude,
        };
        if death {
            return vec![pain(1.0)];
        }
        if burst_sample.settling {
            return Vec::new();
        }
        let mut signals = Vec::new();
        if need > ASSIST_NEED_DEADBAND && need > previous {
            signals.push(pain(need.min(1.0)));
        }
        let mut pleasure = 0.0_f64;
        if need < previous {
            pleasure = pleasure.max((ASSIST_FALL_GAIN * (previous - need)).min(1.0));
        }
        if need <= ASSIST_NEED_DEADBAND && burst_sample.all_contacts_down() {
            pleasure = pleasure.max(ASSIST_HOLD_PLEASURE);
        }
        if outcome == Some(&BalanceAttemptOutcome::Success) {
            pleasure = 1.0;
        }
        if pleasure >= ASSIST_MIN_PLEASURE {
            signals.push(RewardSignal {
                channel: AffectChannel::Pleasure,
                magnitude: pleasure,
            });
        }
        signals
    }
}

/// The reward policy instance of one attempt.
enum AttemptReward {
    Affect(BalanceAffectReward),
    Assist(BalanceAssistReward),
}

impl AttemptReward {
    fn new(policy: BalanceRewardPolicy, support_level: f64) -> Self {
        match policy {
            BalanceRewardPolicy::Affect => {
                Self::Affect(BalanceAffectReward::for_attempt(support_level))
            }
            BalanceRewardPolicy::Assist => Self::Assist(BalanceAssistReward::new()),
        }
    }

    fn on_sample(
        &mut self,
        sample: &BalanceSample,
        outcome: Option<&BalanceAttemptOutcome>,
        criterion: &BalanceFeetCriterion,
    ) -> Vec<RewardSignal> {
        match self {
            Self::Affect(r) => r.on_sample(sample, outcome, criterion),
            Self::Assist(r) => r.on_sample(sample, outcome, criterion),
        }
    }
}

/// Aggregate numbers for one phase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalancePhaseReport {
    /// Finished attempts.
    pub attempts: Vec<BalanceAttemptRecord>,
    /// Successes divided by attempt count.
    pub success_rate: f64,
    /// Median bursts among successes. Absent when nothing succeeded.
    pub median_bursts_to_success: Option<f64>,
    /// Mean terminal tilt across every attempt, degrees.
    pub mean_final_tilt_deg: f64,
    /// Lowest support level at which any attempt succeeded.
    #[serde(default)]
    pub lowest_support_with_success: Option<f64>,
    /// Success rate over attempts at `support_min`. Absent when none ran there.
    #[serde(default)]
    pub success_rate_at_min_support: Option<f64>,
    /// Mean of per-attempt upright fractions.
    #[serde(default)]
    pub mean_upright_fraction: f64,
    /// Success rate per support level, keyed by the level formatted `{:.2}`.
    #[serde(default)]
    pub success_rate_by_support: BTreeMap<String, f64>,
    /// Spring gains the Trainer sized for this phase (absent when no attempt ran).
    #[serde(default)]
    pub spring_gains: Option<SpringGains>,
    /// Success rate at `support_min` >= `graduate_success_rate`; false with no attempt there.
    #[serde(default)]
    pub graduated: bool,
    /// Time balanced (longest upright run per attempt) over every attempt.
    #[serde(default)]
    pub balanced: BalancedTimeStats,
    /// Time balanced per support level, keyed by the level formatted `{:.2}`.
    #[serde(default)]
    pub balanced_by_support: BTreeMap<String, BalancedTimeStats>,
}

/// Mean, median and max of the per-attempt balanced time, in bursts and seconds.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BalancedTimeStats {
    /// Attempts summarized.
    pub attempts: u32,
    /// Mean `balanced_bursts`.
    pub mean_bursts: f64,
    /// Median `balanced_bursts`.
    pub median_bursts: f64,
    /// Max `balanced_bursts`.
    pub max_bursts: u32,
    /// Mean `balanced_s`.
    pub mean_s: f64,
    /// Median `balanced_s`.
    pub median_s: f64,
    /// Max `balanced_s`.
    pub max_s: f64,
}

impl BalancedTimeStats {
    /// Summarize attempts; an empty slice gives all zeros.
    pub fn from_attempts<'a>(attempts: impl IntoIterator<Item = &'a BalanceAttemptRecord>) -> Self {
        let mut bursts: Vec<u32> = Vec::new();
        let mut seconds: Vec<f64> = Vec::new();
        for attempt in attempts {
            bursts.push(attempt.balanced_bursts);
            seconds.push(attempt.balanced_s);
        }
        if bursts.is_empty() {
            return Self::default();
        }
        let n = bursts.len() as f64;
        seconds.sort_by(|a, b| a.total_cmp(b));
        let mid = seconds.len() / 2;
        let median_s = if seconds.len() % 2 == 1 {
            seconds[mid]
        } else {
            (seconds[mid - 1] + seconds[mid]) / 2.0
        };
        Self {
            attempts: bursts.len() as u32,
            mean_bursts: bursts.iter().map(|b| f64::from(*b)).sum::<f64>() / n,
            median_bursts: median_u32(&bursts),
            max_bursts: bursts.iter().copied().max().unwrap_or(0),
            mean_s: seconds.iter().sum::<f64>() / n,
            median_s,
            max_s: seconds.last().copied().unwrap_or(0.0),
        }
    }
}

fn support_key(level: f64) -> String {
    format!("{level:.2}")
}

impl BalancePhaseReport {
    /// Summarize finished attempts with default support_min (0.0) and graduation rate (0.66).
    pub fn from_attempts(attempts: Vec<BalanceAttemptRecord>) -> Result<Self, TrainerError> {
        Self::from_attempts_with(attempts, &SupportConfig::default())
    }

    /// Summarize finished attempts. An empty phase is an error.
    pub fn from_attempts_with(
        attempts: Vec<BalanceAttemptRecord>,
        support: &SupportConfig,
    ) -> Result<Self, TrainerError> {
        if attempts.is_empty() {
            return Err(TrainerError::Evaluation(
                "balance phase has no attempts".to_string(),
            ));
        }
        let count = attempts.len() as f64;
        let is_success = |a: &BalanceAttemptRecord| a.outcome == BalanceAttemptOutcome::Success;
        let successes: Vec<u32> = attempts
            .iter()
            .filter(|attempt| is_success(attempt))
            .map(|attempt| attempt.bursts)
            .collect();
        let success_rate = successes.len() as f64 / count;
        let median_bursts_to_success = if successes.is_empty() {
            None
        } else {
            Some(median_u32(&successes))
        };
        let mean_final_tilt_deg = attempts
            .iter()
            .map(|attempt| attempt.final_tilt_deg)
            .sum::<f64>()
            / count;
        let lowest_support_with_success = attempts
            .iter()
            .filter(|a| is_success(a))
            .map(|a| a.support_level)
            .fold(None, |low: Option<f64>, level| {
                Some(low.map_or(level, |l| l.min(level)))
            });
        let at_min: Vec<&BalanceAttemptRecord> = attempts
            .iter()
            .filter(|a| a.support_level <= support.min + 1.0e-9)
            .collect();
        let success_rate_at_min_support = if at_min.is_empty() {
            None
        } else {
            Some(at_min.iter().filter(|a| is_success(a)).count() as f64 / at_min.len() as f64)
        };
        let mean_upright_fraction =
            attempts.iter().map(|a| a.upright_fraction).sum::<f64>() / count;
        let mut tally: BTreeMap<String, (u32, u32)> = BTreeMap::new();
        for attempt in &attempts {
            let entry = tally.entry(support_key(attempt.support_level)).or_default();
            entry.0 += 1;
            entry.1 += u32::from(is_success(attempt));
        }
        let success_rate_by_support = tally
            .into_iter()
            .map(|(key, (total, ok))| (key, f64::from(ok) / f64::from(total)))
            .collect();
        let graduated =
            success_rate_at_min_support.is_some_and(|rate| rate >= support.graduate_success_rate);
        let balanced = BalancedTimeStats::from_attempts(&attempts);
        let mut by_level: BTreeMap<String, Vec<&BalanceAttemptRecord>> = BTreeMap::new();
        for attempt in &attempts {
            by_level
                .entry(support_key(attempt.support_level))
                .or_default()
                .push(attempt);
        }
        let balanced_by_support = by_level
            .into_iter()
            .map(|(key, group)| (key, BalancedTimeStats::from_attempts(group)))
            .collect();
        Ok(Self {
            attempts,
            success_rate,
            median_bursts_to_success,
            mean_final_tilt_deg,
            lowest_support_with_success,
            success_rate_at_min_support,
            mean_upright_fraction,
            success_rate_by_support,
            spring_gains: None,
            graduated,
            balanced,
            balanced_by_support,
        })
    }
}

fn median_u32(values: &[u32]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 1 {
        f64::from(sorted[mid])
    } else {
        (f64::from(sorted[mid - 1]) + f64::from(sorted[mid])) / 2.0
    }
}

/// Named metrics for one finished phase. Optional values are omitted when undefined.
/// Per-level rates are `success_rate_at_support_<level>` with the level as `{:.2}`;
/// `graduated` is 1.0 or 0.0. Time balanced (longest upright run per attempt) is reported as
/// `balanced_bursts_{mean,median,max}` and `balanced_s_{mean,median,max}`, overall and per
/// level with the suffix `_at_support_<level>`.
pub fn balance_phase_metrics(phase: &BalancePhaseReport) -> BTreeMap<String, f64> {
    let mut metrics = BTreeMap::new();
    metrics.insert("success_rate".to_string(), phase.success_rate);
    metrics.insert("mean_final_tilt_deg".to_string(), phase.mean_final_tilt_deg);
    if let Some(median) = phase.median_bursts_to_success {
        metrics.insert("median_bursts_to_success".to_string(), median);
    }
    if let Some(lowest) = phase.lowest_support_with_success {
        metrics.insert("lowest_support_with_success".to_string(), lowest);
    }
    if let Some(rate) = phase.success_rate_at_min_support {
        metrics.insert("success_rate_at_min_support".to_string(), rate);
    }
    metrics.insert(
        "mean_upright_fraction".to_string(),
        phase.mean_upright_fraction,
    );
    metrics.insert(
        "graduated".to_string(),
        if phase.graduated { 1.0 } else { 0.0 },
    );
    for (level, rate) in &phase.success_rate_by_support {
        metrics.insert(format!("success_rate_at_support_{level}"), *rate);
    }
    insert_balanced_metrics(&mut metrics, "", &phase.balanced);
    for (level, stats) in &phase.balanced_by_support {
        insert_balanced_metrics(&mut metrics, &format!("_at_support_{level}"), stats);
    }
    metrics
}

fn insert_balanced_metrics(
    metrics: &mut BTreeMap<String, f64>,
    suffix: &str,
    stats: &BalancedTimeStats,
) {
    let rows = [
        ("balanced_bursts_mean", stats.mean_bursts),
        ("balanced_bursts_median", stats.median_bursts),
        ("balanced_bursts_max", f64::from(stats.max_bursts)),
        ("balanced_s_mean", stats.mean_s),
        ("balanced_s_median", stats.median_s),
        ("balanced_s_max", stats.max_s),
    ];
    for (name, value) in rows {
        metrics.insert(format!("{name}{suffix}"), value);
    }
}

/// Scorecard for one balance phase (protocol `balance-feet-v2`).
pub fn balance_phase_scorecard(
    scorecard_id: &str,
    connectome_hash: &str,
    genome_schema_version: u32,
    split_id: &str,
    dataset_asset_id: &str,
    dataset_content_hash: &str,
    phase: &BalancePhaseReport,
) -> Scorecard {
    let version = trainer_package_version();
    Scorecard {
        schema_version: crate::contracts::scorecard::SCHEMA_VERSION,
        scorecard_id: ScorecardId(scorecard_id.to_string()),
        connectome_hash: ConnectomeHash(connectome_hash.to_string()),
        genome_version_id: None,
        genome_schema_version: Some(genome_schema_version),
        dataset_asset_id: DatasetAssetId(dataset_asset_id.to_string()),
        dataset_version: "1".to_string(),
        dataset_content_hash: ContentHash(dataset_content_hash.to_string()),
        evaluation_protocol_version: EvaluationProtocolVersion("balance-feet-v2".to_string()),
        metric_pack: PluginRef {
            id: PluginId(EpisodicControlMetricPack::PLUGIN_ID.to_string()),
            version: version.to_string(),
        },
        split_id: SplitId(split_id.to_string()),
        backend_fingerprint: BackendFingerprint {
            backend: BackendKind::Cpu,
            descriptor: "desktop-cpu".to_string(),
            quantization: None,
            trainer_version: version.to_string(),
            feagi_core_version: version.to_string(),
        },
        metrics: balance_phase_metrics(phase),
        metric_stats: None,
        status: ScorecardStatus::SelfReported,
        visibility: ScorecardVisibility::Local,
        skip: None,
        elapsed_ms: None,
        sample_draw_seed: None,
        metadata: BTreeMap::new(),
    }
}

/// Controller side of one balance session: the generic physics primitives of the controller
/// contract. The controller knows no robot and no task; every balance decision is made by the
/// Trainer from these calls.
pub trait BalanceController {
    /// `get_model_info`: timestep, gravity, bodies, keyframes.
    fn model_info(&mut self) -> Result<ModelInfo, TrainerError>;

    /// `reset_episode`: reset to `keyframe` (or the model default), clear springs and pulses,
    /// start a new episode. Returns the episode id.
    fn reset_episode(&mut self, keyframe: Option<&str>) -> Result<u64, TrainerError>;

    /// `set_body_spring`: capture the body's current pose as the reference and apply the spring.
    fn set_body_spring(
        &mut self,
        body: &str,
        gains: &SpringGains,
        level: f64,
        deadband_rad: f64,
    ) -> Result<(), TrainerError>;

    /// `update_body_spring`: change the level of a running spring.
    fn update_body_spring(&mut self, body: &str, level: f64) -> Result<(), TrainerError>;

    /// `clear_body_spring`.
    fn clear_body_spring(&mut self, body: &str) -> Result<(), TrainerError>;

    /// `apply_body_pulse`: a timed external wrench on `body`.
    fn apply_body_pulse(
        &mut self,
        body: &str,
        torque: [f64; 3],
        force: [f64; 3],
        duration_s: f64,
    ) -> Result<(), TrainerError>;

    /// `measure`: one reading of bodies, contact sets and springs.
    fn measure(&mut self, request: &MeasureRequest) -> Result<Measurement, TrainerError>;
}

/// Callback that delivers affect signals into the brain (e.g. `runtime.submit_reward`).
pub type DeliverReward<'a> = &'a mut (dyn FnMut(&[RewardSignal]) -> Result<(), TrainerError> + 'a);

/// One phase of a balance session, plus the run the progress events belong to.
pub struct BalancePhaseRun<'a> {
    /// `train` or `test`.
    pub phase: &'a str,
    /// Seeds visited in order.
    pub seeds: &'a [u64],
    /// Acceptance rule.
    pub criterion: &'a BalanceFeetCriterion,
    /// Run the progress events are stamped with.
    pub run_id: &'a RunId,
    /// Support policy. `None`: `train` follows the schedule, `test` runs at `support_min`.
    pub support_mode: Option<SupportMode>,
    /// Affect delivery. `None` keeps the no-affect behavior.
    pub deliver_reward: Option<DeliverReward<'a>>,
    /// Which policy builds the affect signals when `deliver_reward` is set.
    pub reward_policy: BalanceRewardPolicy,
}

/// Per-phase state: model facts and the gains sized once from the first reset.
struct BalanceSession {
    info: ModelInfo,
    sizing: Option<(SizingInputs, SpringGains)>,
}

/// Run one phase (`train` or `test`) and emit its progress.
pub fn run_balance_phase(
    run: &mut BalancePhaseRun<'_>,
    controller: &mut dyn BalanceController,
    present_goal: &mut dyn FnMut([f64; 3]) -> Result<(), TrainerError>,
    events: &mut dyn RunEventSink,
    cancel: &CancelToken,
) -> Result<BalancePhaseReport, TrainerError> {
    if run.phase != "train" && run.phase != "test" {
        return Err(TrainerError::Config(
            "balance phase must be train or test".to_string(),
        ));
    }
    run.criterion.validate()?;
    if run.seeds.is_empty() {
        return Err(TrainerError::Config(format!(
            "{} seeds must contain at least one seed",
            run.phase
        )));
    }
    let support = run.criterion.support_config();
    let mode = run.support_mode.unwrap_or(if run.phase == "train" {
        SupportMode::Schedule
    } else {
        SupportMode::Fixed(SupportSchedule::for_test_phase(&support))
    });
    if let SupportMode::Fixed(level) = mode {
        if !level.is_finite() || !(0.0..=1.0).contains(&level) {
            return Err(TrainerError::Config(
                "fixed support level must be in [0, 1]".to_string(),
            ));
        }
    }
    let info = controller.model_info()?;
    check_model(&info, run.criterion)?;
    let mut session = BalanceSession { info, sizing: None };
    let mut schedule = SupportSchedule::new(support);
    let samples_total = run.seeds.len() as u64;
    let mut attempts = Vec::with_capacity(run.seeds.len());
    for seed in run.seeds {
        if cancel.is_cancelled() {
            return Err(TrainerError::Cancelled(
                "balance-on-feet run cancelled".to_string(),
            ));
        }
        let level = match mode {
            SupportMode::Schedule => schedule.current_level(),
            SupportMode::Fixed(level) => level,
        };
        let record = run_balance_attempt(
            *seed,
            run.criterion,
            level,
            controller,
            &mut session,
            present_goal,
            &mut run.deliver_reward,
            run.reward_policy,
            cancel,
        )?;
        schedule.record_attempt(record.outcome, record.balanced_s);
        attempts.push(record);
        events.emit(RunEvent::new(
            run.run_id.clone(),
            RunEventKind::Progress {
                samples_done: attempts.len() as u64,
                samples_total,
                repeat_index: 0,
                repeat_total: 1,
                sent_values: upright_goal_features().to_vec(),
                tick_index: None,
                window_values: Vec::new(),
                image_png_base64: None,
                mask_png_base64: None,
                preview_name: Some(format!("{} seed {seed} support {level:.2}", run.phase)),
                predicted_class: None,
                class_scores: Vec::new(),
            },
        ));
    }
    let mut report = BalancePhaseReport::from_attempts_with(attempts, &support)?;
    report.spring_gains = session.sizing.map(|(_, gains)| gains);
    events.emit(RunEvent::new(
        run.run_id.clone(),
        RunEventKind::MetricUpdate {
            scope: MetricScope::Aggregate,
            metrics: balance_phase_metrics(&report),
            class_breakdown: Vec::new(),
        },
    ));
    Ok(report)
}

/// The named bodies and keyframe must exist in the loaded model.
fn check_model(info: &ModelInfo, criterion: &BalanceFeetCriterion) -> Result<(), TrainerError> {
    let names = std::iter::once(&criterion.torso_body).chain(criterion.contact_bodies.iter());
    for name in names {
        if info.body(name.trim()).is_none() {
            return Err(TrainerError::Config(format!(
                "body {name} is not in the loaded model"
            )));
        }
    }
    let keyframe = criterion.reset_keyframe.trim();
    if !keyframe.is_empty() && !info.keyframe_names().iter().any(|name| name == keyframe) {
        return Err(TrainerError::Config(format!(
            "reset keyframe {keyframe} is not in the loaded model (keyframes: {})",
            info.keyframe_names().join(", ")
        )));
    }
    if !(info.timestep_s.is_finite() && info.timestep_s > 0.0) {
        return Err(TrainerError::Runtime(
            "get_model_info reported no physics timestep".to_string(),
        ));
    }
    Ok(())
}

fn measure_request(criterion: &BalanceFeetCriterion) -> MeasureRequest {
    MeasureRequest {
        bodies: vec![criterion.torso_body.trim().to_string()],
        contacts: criterion
            .contact_bodies
            .iter()
            .map(|name| vec![name.trim().to_string()])
            .collect(),
    }
}

/// Small deterministic generator (splitmix64) so perturbation schedules repeat per seed.
#[derive(Debug, Clone)]
struct SplitMix(u64);

impl SplitMix {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Repeatable random torque pulses, indexed by seconds after settling. The first pulse starts
/// after `interval_s` jittered by +-50%; each later pulse starts a jittered interval after the
/// previous start (never before the previous pulse ends). Each pulse lasts `duration_s` about
/// world X or Y with a random sign.
#[derive(Debug, Clone)]
pub struct PerturbPlan {
    rng: SplitMix,
    interval_s: f64,
    duration_s: f64,
    torque_nm: f64,
    next_start_s: f64,
    pulse_end_s: f64,
}

impl PerturbPlan {
    /// Plan for one attempt; `interval_s == 0` never pulses.
    pub fn new(seed: u64, interval_s: f64, duration_s: f64, torque_nm: f64) -> Self {
        let mut plan = Self {
            rng: SplitMix(seed ^ 0xA5A5_5A5A_1234_5678),
            interval_s,
            duration_s,
            torque_nm,
            next_start_s: f64::INFINITY,
            pulse_end_s: f64::NEG_INFINITY,
        };
        if interval_s > 0.0 {
            plan.next_start_s = plan.jittered(0.0);
        }
        plan
    }

    fn jittered(&mut self, base: f64) -> f64 {
        base + self.interval_s * (0.5 + self.rng.unit())
    }

    /// Pulse to start at `post_settle_s` (non-decreasing calls), or `None`.
    pub fn pulse_at(&mut self, post_settle_s: f64) -> Option<[f64; 3]> {
        if self.interval_s <= 0.0
            || post_settle_s < self.next_start_s
            || post_settle_s < self.pulse_end_s
        {
            return None;
        }
        let axis = (self.rng.next_u64() & 1) as usize;
        let sign = if self.rng.next_u64() & 1 == 0 {
            -1.0
        } else {
            1.0
        };
        let mut torque = [0.0; 3];
        torque[axis] = sign * self.torque_nm;
        self.pulse_end_s = post_settle_s + self.duration_s;
        self.next_start_s = self.jittered(post_settle_s).max(self.pulse_end_s);
        Some(torque)
    }

    /// A pulse is running at `post_settle_s`.
    pub fn active_at(&self, post_settle_s: f64) -> bool {
        post_settle_s < self.pulse_end_s
    }
}

#[allow(clippy::too_many_arguments)]
fn run_balance_attempt(
    seed: u64,
    criterion: &BalanceFeetCriterion,
    support_level: f64,
    controller: &mut dyn BalanceController,
    session: &mut BalanceSession,
    present_goal: &mut dyn FnMut([f64; 3]) -> Result<(), TrainerError>,
    deliver_reward: &mut Option<DeliverReward<'_>>,
    reward_policy: BalanceRewardPolicy,
    cancel: &CancelToken,
) -> Result<BalanceAttemptRecord, TrainerError> {
    let torso = criterion.torso_body.trim().to_string();
    let keyframe = Some(criterion.reset_keyframe.trim()).filter(|name| !name.is_empty());
    controller.reset_episode(keyframe)?;
    let request = measure_request(criterion);
    let first = controller.measure(&request)?;
    let (sizing, gains) = match session.sizing {
        Some(sized) => sized,
        None => {
            let torso_now = first.bodies.get(&torso).ok_or_else(|| {
                TrainerError::Runtime(format!("measure reply has no body {torso}"))
            })?;
            let h0 = match criterion.support_com_height_m {
                Some(h0) => h0,
                None => {
                    let floor = first.mean_contact_height().ok_or_else(|| {
                        TrainerError::Runtime(
                            "cannot size the support spring: no contact points after the reset; \
                             the contact bodies must touch the floor in the reset pose, or set \
                             support_com_height_m"
                                .to_string(),
                        )
                    })?;
                    torso_now.subtree_com[2] - floor
                }
            };
            let sizing = sizing_inputs(&session.info, &torso, h0)?;
            let sized = (sizing, gains_for(&sizing, criterion));
            session.sizing = Some(sized);
            sized
        }
    };
    let settle_level = if criterion.settle_s > 0.0 {
        1.0
    } else {
        support_level
    };
    controller.set_body_spring(
        &torso,
        &gains,
        settle_level,
        criterion.support_deadband_deg.to_radians(),
    )?;
    let t0 = first.sim_time_s;
    let steps_per_burst = criterion.burst_period_s / session.info.timestep_s;
    let features = upright_goal_features();
    let mut open = OpenBalanceAttempt::for_attempt(steps_per_burst, support_level);
    let mut affect = AttemptReward::new(reward_policy, support_level);
    let mut perturb = PerturbPlan::new(
        seed,
        criterion.perturb_interval_s,
        criterion.perturb_duration_s,
        criterion.perturb_torque_nm,
    );
    let mut spring_level = settle_level;
    let result = loop {
        if cancel.is_cancelled() {
            let _ = controller.clear_body_spring(&torso);
            return Err(TrainerError::Cancelled(
                "balance-on-feet run cancelled".to_string(),
            ));
        }
        present_goal(features)?;
        let reading = controller.measure(&request)?;
        let elapsed_s = reading.sim_time_s - t0;
        let settling = elapsed_s + 1.0e-9 < criterion.settle_s;
        let level_now = if settling { 1.0 } else { support_level };
        let body = reading
            .bodies
            .get(&torso)
            .ok_or_else(|| TrainerError::Runtime(format!("measure reply has no body {torso}")))?;
        if reading.contacts.len() != criterion.contact_bodies.len() {
            return Err(TrainerError::Runtime(format!(
                "measure returned {} contact sets for {} contact bodies",
                reading.contacts.len(),
                criterion.contact_bodies.len()
            )));
        }
        let mut perturbing = false;
        if reading.terminal.is_none() {
            if (spring_level - level_now).abs() > 1.0e-12 {
                controller.update_body_spring(&torso, level_now)?;
                spring_level = level_now;
            }
            if !settling {
                let post_settle_s = elapsed_s - criterion.settle_s;
                if let Some(torque) = perturb.pulse_at(post_settle_s) {
                    controller.apply_body_pulse(
                        &torso,
                        torque,
                        [0.0; 3],
                        criterion.perturb_duration_s,
                    )?;
                }
                perturbing = perturb.active_at(post_settle_s);
            }
        }
        let wrench = reading.springs.get(&torso).copied();
        let sample = BalanceSample {
            seq: reading.physics_steps,
            seed,
            tilt_deg: body.tilt_rad.to_degrees(),
            contacts: reading.contacts.iter().map(|set| set.touching).collect(),
            terminal: reading.terminal,
            support_level: Some(level_now),
            settling,
            perturbing,
            support_need: Some(support_need(
                body.tilt_rad,
                body.ang_vel,
                criterion.max_tilt_deg,
                criterion.need_lookahead_s,
            )),
            support_effort: wrench.map(|w| support_effort(w, &sizing)),
            support_torque_nm: wrench.map(|w| w.wrench_norm_torque),
            support_force_n: wrench.map(|w| w.wrench_norm_force),
        };
        let step = observe_balance_sample(&open, &sample, criterion, seed)?;
        let (counted, outcome) = match &step {
            BalanceAttemptStep::Continue(next) => (next.counted_burst, None),
            BalanceAttemptStep::Finished(record) => (true, Some(record.outcome)),
        };
        if counted {
            if let Some(deliver) = deliver_reward.as_deref_mut() {
                let signals = affect.on_sample(&sample, outcome.as_ref(), criterion);
                if !signals.is_empty() {
                    deliver(&signals)?;
                }
            }
        }
        match step {
            BalanceAttemptStep::Continue(next) => open = next,
            BalanceAttemptStep::Finished(record) => break record,
        }
    };
    // The next reset clears it anyway; a death may already have cleared it.
    let _ = controller.clear_body_spring(&torso);
    Ok(result)
}

/// HTTP client for the generic physics actions of a controller control server. One keep-alive
/// connection is reused for every call (reopened transparently when the server closes it).
pub struct HttpBalanceController {
    host: String,
    port: u16,
    timeout: Duration,
    connection: Option<BufReader<TcpStream>>,
    connections_opened: u32,
}

/// JSON body of `get_model_info`.
pub fn model_info_request() -> serde_json::Value {
    serde_json::json!({"action": "get_model_info"})
}

/// JSON body of `reset_episode`.
pub fn reset_episode_request(keyframe: Option<&str>) -> serde_json::Value {
    let mut body = serde_json::json!({"action": "reset_episode"});
    if let Some(name) = keyframe {
        body["keyframe"] = serde_json::json!(name);
    }
    body
}

/// JSON body of `set_body_spring`.
pub fn set_body_spring_request(
    body: &str,
    gains: &SpringGains,
    level: f64,
    deadband_rad: f64,
) -> serde_json::Value {
    serde_json::json!({
        "action": "set_body_spring",
        "body": body,
        "k_rot": gains.k_rot,
        "d_rot": gains.d_rot,
        "k_lin": gains.k_lin,
        "d_lin": gains.d_lin,
        "gravity_comp": gains.gravity_comp,
        "level": level,
        "deadband_rad": deadband_rad,
        "ref": "current",
    })
}

/// JSON body of `update_body_spring`.
pub fn update_body_spring_request(body: &str, level: f64) -> serde_json::Value {
    serde_json::json!({"action": "update_body_spring", "body": body, "level": level})
}

/// JSON body of `clear_body_spring`.
pub fn clear_body_spring_request(body: &str) -> serde_json::Value {
    serde_json::json!({"action": "clear_body_spring", "body": body})
}

/// JSON body of `apply_body_pulse`.
pub fn apply_body_pulse_request(
    body: &str,
    torque: [f64; 3],
    force: [f64; 3],
    duration_s: f64,
) -> serde_json::Value {
    serde_json::json!({
        "action": "apply_body_pulse",
        "body": body,
        "torque": torque,
        "force": force,
        "duration_s": duration_s,
    })
}

/// JSON body of `measure`.
pub fn measure_request_body(request: &MeasureRequest) -> serde_json::Value {
    let contacts: Vec<serde_json::Value> = request
        .contacts
        .iter()
        .map(|set| serde_json::json!({"a": set, "b": "world"}))
        .collect();
    serde_json::json!({
        "action": "measure",
        "bodies": request.bodies,
        "contacts": contacts,
    })
}

/// One HTTP exchange: status code, whether the server keeps the connection, and the body.
struct HttpReply {
    status: u16,
    keep_alive: bool,
    body: String,
}

impl HttpBalanceController {
    /// `base_url` is `http://host:port` with no path.
    pub fn new(base_url: &str, timeout: Duration) -> Result<Self, TrainerError> {
        if timeout.is_zero() {
            return Err(TrainerError::Config(
                "balance control timeout must be > 0".to_string(),
            ));
        }
        let (host, port) = split_http_base(base_url)?;
        Ok(Self {
            host,
            port,
            timeout,
            connection: None,
            connections_opened: 0,
        })
    }

    /// TCP connections opened so far (1 while keep-alive holds).
    pub fn connections_opened(&self) -> u32 {
        self.connections_opened
    }

    fn connect(&mut self) -> Result<BufReader<TcpStream>, TrainerError> {
        let address = format!("{}:{}", self.host, self.port);
        let socket = address
            .to_socket_addrs()
            .map_err(|error| TrainerError::Runtime(format!("balance control address: {error}")))?
            .next()
            .ok_or_else(|| {
                TrainerError::Runtime(format!("balance control address {address} did not resolve"))
            })?;
        let stream = TcpStream::connect_timeout(&socket, self.timeout).map_err(|error| {
            TrainerError::Runtime(format!("balance control connect failed: {error}"))
        })?;
        stream
            .set_read_timeout(Some(self.timeout))
            .and_then(|()| stream.set_write_timeout(Some(self.timeout)))
            .and_then(|()| stream.set_nodelay(true))
            .map_err(|error| TrainerError::Runtime(format!("balance control socket: {error}")))?;
        self.connections_opened += 1;
        Ok(BufReader::new(stream))
    }

    /// One request/response on `reader`. The flag is true when the failure happened before any
    /// response byte arrived (a stale kept-alive connection), so the call may be retried.
    fn exchange(
        reader: &mut BufReader<TcpStream>,
        request: &[u8],
    ) -> Result<HttpReply, (TrainerError, bool)> {
        let early = |error: std::io::Error, what: &str| {
            (
                TrainerError::Runtime(format!("balance control {what} failed: {error}")),
                true,
            )
        };
        reader
            .get_mut()
            .write_all(request)
            .map_err(|error| early(error, "write"))?;
        let mut status_line = String::new();
        let read = reader
            .read_line(&mut status_line)
            .map_err(|error| early(error, "read"))?;
        if read == 0 {
            return Err((
                TrainerError::Runtime("balance control closed the connection".to_string()),
                true,
            ));
        }
        let late = |error: std::io::Error| {
            (
                TrainerError::Runtime(format!("balance control read failed: {error}")),
                false,
            )
        };
        let status: u16 = status_line
            .split_whitespace()
            .nth(1)
            .and_then(|code| code.parse().ok())
            .ok_or_else(|| {
                (
                    TrainerError::Runtime(format!(
                        "balance control HTTP failed: {}",
                        status_line.trim()
                    )),
                    false,
                )
            })?;
        let http_10 = status_line.starts_with("HTTP/1.0");
        let mut content_length: Option<usize> = None;
        let mut keep_alive = !http_10;
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).map_err(late)?;
            let line = line.trim_end();
            if line.is_empty() {
                break;
            }
            if let Some((name, value)) = line.split_once(':') {
                let value = value.trim();
                match name.trim().to_ascii_lowercase().as_str() {
                    "content-length" => content_length = value.parse().ok(),
                    "connection" => {
                        let lower = value.to_ascii_lowercase();
                        if lower.contains("close") {
                            keep_alive = false;
                        } else if lower.contains("keep-alive") {
                            keep_alive = true;
                        }
                    }
                    _ => {}
                }
            }
        }
        let mut raw = Vec::new();
        match content_length {
            Some(length) => {
                raw.resize(length, 0);
                reader.read_exact(&mut raw).map_err(late)?;
            }
            None => {
                reader.read_to_end(&mut raw).map_err(late)?;
                keep_alive = false;
            }
        }
        Ok(HttpReply {
            status,
            keep_alive,
            body: String::from_utf8_lossy(&raw).into_owned(),
        })
    }

    fn command(&mut self, body: &serde_json::Value) -> Result<serde_json::Value, TrainerError> {
        let payload = serde_json::to_vec(body).map_err(|error| {
            TrainerError::Runtime(format!("balance command encode failed: {error}"))
        })?;
        let mut request = format!(
            "POST /command HTTP/1.1\r\nHost: {}:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\r\n",
            self.host,
            self.port,
            payload.len()
        )
        .into_bytes();
        request.extend_from_slice(&payload);
        let (mut reader, reused) = match self.connection.take() {
            Some(reader) => (reader, true),
            None => (self.connect()?, false),
        };
        let reply = match Self::exchange(&mut reader, &request) {
            Ok(reply) => reply,
            Err((_, true)) if reused => {
                reader = self.connect()?;
                Self::exchange(&mut reader, &request).map_err(|(error, _)| error)?
            }
            Err((error, _)) => return Err(error),
        };
        if reply.keep_alive {
            self.connection = Some(reader);
        }
        decode_envelope(reply.status, &reply.body)
    }

    fn command_as<T: serde::de::DeserializeOwned>(
        &mut self,
        body: &serde_json::Value,
        what: &str,
    ) -> Result<T, TrainerError> {
        let result = self.command(body)?;
        serde_json::from_value(result).map_err(|error| {
            TrainerError::Runtime(format!("balance {what} decode failed: {error}"))
        })
    }
}

impl BalanceController for HttpBalanceController {
    fn model_info(&mut self) -> Result<ModelInfo, TrainerError> {
        self.command_as(&model_info_request(), "model info")
    }

    fn reset_episode(&mut self, keyframe: Option<&str>) -> Result<u64, TrainerError> {
        let result = self.command(&reset_episode_request(keyframe))?;
        result
            .get("episode_id")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| {
                TrainerError::Runtime("reset_episode reply has no episode_id".to_string())
            })
    }

    fn set_body_spring(
        &mut self,
        body: &str,
        gains: &SpringGains,
        level: f64,
        deadband_rad: f64,
    ) -> Result<(), TrainerError> {
        self.command(&set_body_spring_request(body, gains, level, deadband_rad))
            .map(|_| ())
    }

    fn update_body_spring(&mut self, body: &str, level: f64) -> Result<(), TrainerError> {
        self.command(&update_body_spring_request(body, level))
            .map(|_| ())
    }

    fn clear_body_spring(&mut self, body: &str) -> Result<(), TrainerError> {
        self.command(&clear_body_spring_request(body)).map(|_| ())
    }

    fn apply_body_pulse(
        &mut self,
        body: &str,
        torque: [f64; 3],
        force: [f64; 3],
        duration_s: f64,
    ) -> Result<(), TrainerError> {
        self.command(&apply_body_pulse_request(body, torque, force, duration_s))
            .map(|_| ())
    }

    fn measure(&mut self, request: &MeasureRequest) -> Result<Measurement, TrainerError> {
        self.command_as(&measure_request_body(request), "measure")
    }
}

fn split_http_base(base_url: &str) -> Result<(String, u16), TrainerError> {
    let rest = base_url.strip_prefix("http://").ok_or_else(|| {
        TrainerError::Config("balance control URL must start with http://".to_string())
    })?;
    if rest.contains('/') {
        return Err(TrainerError::Config(
            "balance control URL must be http://host:port with no path".to_string(),
        ));
    }
    let (host, port_text) = rest.split_once(':').ok_or_else(|| {
        TrainerError::Config("balance control URL must include a port".to_string())
    })?;
    if host.is_empty() {
        return Err(TrainerError::Config(
            "balance control URL host is empty".to_string(),
        ));
    }
    let port: u16 = port_text.parse().map_err(|_| {
        TrainerError::Config(format!("balance control port {port_text} is not a u16"))
    })?;
    Ok((host.to_string(), port))
}

fn decode_envelope(status: u16, body: &str) -> Result<serde_json::Value, TrainerError> {
    let envelope: Option<serde_json::Value> = serde_json::from_str(body.trim()).ok();
    if status != 200 {
        let detail = envelope
            .as_ref()
            .and_then(|value| value.get("error"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        return Err(TrainerError::Runtime(format!(
            "balance control HTTP failed: status {status} {detail}"
        )));
    }
    let envelope = envelope
        .ok_or_else(|| TrainerError::Runtime("balance control response is not JSON".to_string()))?;
    let ok = envelope
        .get("ok")
        .and_then(|value| value.as_bool())
        .ok_or_else(|| {
            TrainerError::Runtime("balance control response has no ok flag".to_string())
        })?;
    if !ok {
        let message = envelope
            .get("error")
            .and_then(|value| value.as_str())
            .unwrap_or("balance control command failed");
        return Err(TrainerError::Runtime(message.to_string()));
    }
    envelope
        .get("result")
        .cloned()
        .ok_or_else(|| TrainerError::Runtime("balance control response has no result".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::CollectingEventSink;

    /// One burst is 0.05 s; hold 0.1 s = 2 bursts, cap 0.2 s = 4 bursts. One physics step per
    /// burst so legacy tests stay readable.
    fn criterion() -> BalanceFeetCriterion {
        let mut c = BalanceFeetCriterion::new(
            "torso",
            vec!["left_foot".to_string(), "right_foot".to_string()],
            15.0,
            0.1,
            0.2,
        );
        c.burst_period_s = 0.05;
        c
    }

    fn sample(seq: u64, seed: u64, tilt: f64) -> BalanceSample {
        BalanceSample {
            seq,
            seed,
            tilt_deg: tilt,
            contacts: vec![true, true],
            terminal: None,
            support_level: Some(0.0),
            settling: false,
            perturbing: false,
            support_need: None,
            support_effort: None,
            support_torque_nm: None,
            support_force_n: None,
        }
    }

    fn balanced(seq: u64, seed: u64) -> BalanceSample {
        sample(seq, seed, 2.0)
    }

    fn record(outcome: BalanceAttemptOutcome, level: f64, upright: f64) -> BalanceAttemptRecord {
        BalanceAttemptRecord {
            seed: 0,
            outcome,
            bursts: 5,
            final_tilt_deg: 3.0,
            support_level: level,
            upright_fraction: upright,
            perturbations: 0,
            balanced_bursts: 0,
            balanced_s: 0.0,
            time_to_first_violation_bursts: None,
            time_to_first_violation_s: None,
        }
    }

    fn fed(
        open: &OpenBalanceAttempt,
        s: &BalanceSample,
        c: &BalanceFeetCriterion,
    ) -> OpenBalanceAttempt {
        match observe_balance_sample(open, s, c, s.seed).expect("observe") {
            BalanceAttemptStep::Continue(next) => next,
            BalanceAttemptStep::Finished(r) => panic!("finished early: {r:?}"),
        }
    }

    #[test]
    fn two_balanced_bursts_succeed() {
        let mut open = OpenBalanceAttempt::fresh();
        let rule = criterion();
        let step = observe_balance_sample(&open, &balanced(1, 7), &rule, 7).expect("first");
        open = match step {
            BalanceAttemptStep::Continue(next) => next,
            BalanceAttemptStep::Finished(_) => panic!("hold is 2"),
        };
        let finished = observe_balance_sample(&open, &balanced(2, 7), &rule, 7).expect("second");
        match finished {
            BalanceAttemptStep::Finished(record) => {
                assert_eq!(record.outcome, BalanceAttemptOutcome::Success);
                assert_eq!(record.bursts, 2);
                assert_eq!(record.upright_fraction, 1.0);
            }
            BalanceAttemptStep::Continue(_) => panic!("expected success"),
        }
    }

    #[test]
    fn a_lifted_foot_resets_the_hold() {
        let mut open = OpenBalanceAttempt::fresh();
        open.bursts = 1;
        open.holding_bursts = 1;
        open.last_seq = 1;
        open.last_burst_seq = Some(1.0);
        let mut s = balanced(2, 3);
        s.contacts[1] = false;
        match observe_balance_sample(&open, &s, &criterion(), 3).expect("step") {
            BalanceAttemptStep::Continue(next) => assert_eq!(next.holding_bursts, 0),
            BalanceAttemptStep::Finished(_) => panic!("one missed foot is not the cap"),
        }
    }

    fn cfg() -> SupportConfig {
        SupportConfig {
            start: 1.0,
            step: 0.1,
            min: 0.0,
            attempts_per_level: 3,
            graduate_success_rate: 0.66,
            graduate_balanced_s: None,
        }
    }

    fn feed(s: &mut SupportSchedule, outcomes: &[BalanceAttemptOutcome]) {
        for o in outcomes {
            s.record(*o);
        }
    }

    use BalanceAttemptOutcome::{Death, Success};

    #[test]
    fn schedule_lowers_after_successful_block_and_holds_after_failure() {
        let mut s = SupportSchedule::new(cfg());
        assert_eq!(s.current_level(), 1.0);
        // 2 of 3 is 0.667, at or above the 0.66 default: the block lowers support.
        feed(&mut s, &[Success, Success, Death]);
        assert!((s.current_level() - 0.9).abs() < 1e-9);
        feed(&mut s, &[Success, Success, Success]);
        assert!((s.current_level() - 0.8).abs() < 1e-9);
        feed(&mut s, &[Death, Success, Death]);
        assert!((s.current_level() - 0.8).abs() < 1e-9);
        s = SupportSchedule::new(cfg());
        feed(&mut s, &[Success, Success, Success]);
        feed(&mut s, &[Death, Death, Success]);
        assert!(
            (s.current_level() - 0.9).abs() < 1e-9,
            "one failed block holds"
        );
        feed(&mut s, &[Success, Success, Success]);
        assert!((s.current_level() - 0.8).abs() < 1e-9);
    }

    #[test]
    fn schedule_raises_one_step_after_two_failed_blocks_and_clamps() {
        let mut s = SupportSchedule::new(cfg());
        feed(&mut s, &[Success, Success, Success]); // 0.9
        feed(&mut s, &[Death; 3]);
        feed(&mut s, &[Death; 3]);
        assert!((s.current_level() - 1.0).abs() < 1e-9);
        feed(&mut s, &[Death; 6]);
        assert!((s.current_level() - 1.0).abs() < 1e-9, "never above start");
        let mut low = SupportSchedule::new(SupportConfig {
            start: 0.2,
            ..cfg()
        });
        for _ in 0..5 {
            feed(&mut low, &[Success; 3]);
        }
        assert_eq!(low.current_level(), 0.0, "clamps at min");
        assert_eq!(SupportSchedule::for_test_phase(&cfg()), 0.0);
    }

    #[test]
    fn twenty_five_steps_are_one_burst() {
        let mut c = criterion();
        c.hold_s = 0.5;
        c.attempt_cap_s = 1.0;
        let mut open = OpenBalanceAttempt::for_attempt(25.0, 0.0);
        open = fed(&open, &balanced(1, 5), &c);
        assert_eq!((open.bursts, open.counted_burst), (1, true));
        for seq in 2..26 {
            open = fed(&open, &balanced(seq, 5), &c);
            assert_eq!(open.bursts, 1, "seq {seq} is inside the burst");
            assert!(!open.counted_burst);
        }
        assert_eq!(open.last_seq, 25);
        open = fed(&open, &balanced(26, 5), &c);
        assert_eq!(open.bursts, 2);
        assert_eq!(open.holding_bursts, 2);
        assert!(open.counted_burst);
    }

    #[test]
    fn late_poll_counts_every_elapsed_burst() {
        let mut c = criterion();
        c.hold_s = 0.5;
        c.attempt_cap_s = 1.0;
        let mut open = OpenBalanceAttempt::for_attempt(25.0, 0.0);
        open = fed(&open, &balanced(1, 5), &c);
        open = fed(&open, &balanced(60, 5), &c);
        assert_eq!(open.bursts, 3, "1 first burst + 2 elapsed (59 / 25)");
        assert_eq!(open.last_burst_seq, Some(51.0));
        assert_eq!(
            open.holding_bursts, 3,
            "hold advances by elapsed upright bursts"
        );
        assert!(open.counted_burst);
    }

    #[test]
    fn controller_steps_per_burst_overrides_criterion() {
        let mut c = criterion();
        c.hold_s = 0.5;
        c.attempt_cap_s = 1.0;
        let mut open = OpenBalanceAttempt::for_attempt(10.0, 0.0);
        open = fed(&open, &balanced(1, 5), &c);
        open = fed(&open, &balanced(11, 5), &c);
        assert_eq!(open.bursts, 2);
    }

    #[test]
    fn settling_samples_do_not_hold_or_count_toward_cap() {
        let c = criterion(); // cap 4, hold 2
        let mut open = OpenBalanceAttempt::fresh();
        for seq in 1..=10 {
            let mut s = balanced(seq, 5);
            s.settling = true;
            s.support_level = Some(1.0);
            open = fed(&open, &s, &c);
        }
        assert_eq!(open.bursts, 0);
        assert_eq!(open.holding_bursts, 0);
        assert_eq!(open.settling_bursts, 10);
        match observe_balance_sample(&fed(&open, &balanced(11, 5), &c), &balanced(12, 5), &c, 5)
            .expect("obs")
        {
            BalanceAttemptStep::Finished(r) => {
                assert_eq!(r.outcome, Success);
                assert_eq!(r.bursts, 2);
            }
            BalanceAttemptStep::Continue(_) => panic!("2 real bursts should hold"),
        }
    }

    #[test]
    fn upright_fraction_and_truncation() {
        let c = criterion();
        let mut open = OpenBalanceAttempt::fresh();
        open = fed(&open, &sample(1, 1, 30.0), &c);
        open = fed(&open, &sample(2, 1, 40.0), &c);
        open = fed(&open, &balanced(3, 1), &c);
        match observe_balance_sample(&open, &sample(4, 1, 50.0), &c, 1).expect("obs") {
            BalanceAttemptStep::Finished(r) => {
                assert_eq!(r.outcome, BalanceAttemptOutcome::Truncated);
                assert_eq!(r.upright_fraction, 0.25);
            }
            BalanceAttemptStep::Continue(_) => panic!("cap is 4"),
        }
    }

    fn reward_for(
        r: &mut BalanceAffectReward,
        tilt: f64,
        level: f64,
        settling: bool,
        outcome: Option<BalanceAttemptOutcome>,
    ) -> Vec<(AffectChannel, f64)> {
        let mut s = sample(1, 1, tilt);
        s.support_level = Some(level);
        s.settling = settling;
        r.on_sample(&s, outcome.as_ref(), &criterion())
            .into_iter()
            .map(|x| (x.channel, x.magnitude))
            .collect()
    }

    #[test]
    fn reward_pain_on_crossings_once_and_recovery_pleasure() {
        let mut r = BalanceAffectReward::new();
        let p = AffectChannel::Pain;
        let g = AffectChannel::Pleasure;
        assert!(reward_for(&mut r, 3.0, 1.0, false, None).is_empty());
        assert_eq!(reward_for(&mut r, 9.0, 1.0, false, None), vec![(p, 0.5)]);
        assert!(
            reward_for(&mut r, 10.0, 1.0, false, None).is_empty(),
            "once per crossing"
        );
        assert_eq!(reward_for(&mut r, 16.0, 1.0, false, None), vec![(p, 1.0)]);
        assert!(reward_for(&mut r, 20.0, 1.0, false, None).is_empty());
        // recovery Pleasure is scaled by (1 - support): none at 1.0, 0.25 at 0.5
        assert!(reward_for(&mut r, 5.0, 1.0, false, None).is_empty());
        assert_eq!(reward_for(&mut r, 20.0, 0.5, false, None), vec![(p, 1.0)]);
        assert_eq!(reward_for(&mut r, 5.0, 0.5, false, None), vec![(g, 0.25)]);
        assert!(
            reward_for(&mut r, 4.0, 1.0, false, None).is_empty(),
            "no repeat recovery"
        );
        // a second excursion fires again
        assert_eq!(reward_for(&mut r, 9.0, 1.0, false, None), vec![(p, 0.5)]);
    }

    #[test]
    fn reward_terminal_and_standing_rules() {
        let p = AffectChannel::Pain;
        let g = AffectChannel::Pleasure;
        let mut r = BalanceAffectReward::new();
        assert_eq!(
            reward_for(&mut r, 8.0, 0.0, false, Some(Death)),
            vec![(p, 0.5), (p, 1.0)]
        );
        let mut r = BalanceAffectReward::new();
        assert_eq!(
            reward_for(&mut r, 3.0, 0.0, false, Some(Success)),
            vec![(g, 1.0)]
        );
        let mut r = BalanceAffectReward::new();
        assert_eq!(reward_for(&mut r, 3.0, 0.0, false, None), vec![(g, 0.1)]);
        assert!(
            reward_for(&mut r, 3.0, 0.5, false, None).is_empty(),
            "supported: no standing reward"
        );
        let mut r = BalanceAffectReward::new();
        let mut s = sample(1, 1, 3.0);
        s.contacts[1] = false;
        assert!(
            r.on_sample(&s, None, &criterion()).is_empty(),
            "foot up: not upright"
        );
    }

    #[test]
    fn reward_is_silent_during_settling() {
        let mut r = BalanceAffectReward::new();
        assert!(reward_for(&mut r, 9.0, 0.0, true, None).is_empty());
        assert!(reward_for(&mut r, 20.0, 0.0, true, None).is_empty());
        // tilt state is tracked, so leaving settling at 20 deg is not a fresh crossing
        assert!(reward_for(&mut r, 21.0, 0.0, false, None).is_empty());
    }

    #[test]
    fn scorecard_v2_fields() {
        let attempts = vec![
            record(Success, 0.5, 1.0),
            record(Death, 0.5, 0.5),
            record(Success, 0.0, 0.8),
            record(Success, 0.0, 0.6),
            record(Death, 0.0, 0.0),
        ];
        let report = BalancePhaseReport::from_attempts_with(attempts, &cfg()).expect("report");
        assert_eq!(report.lowest_support_with_success, Some(0.0));
        let rate = report.success_rate_at_min_support.expect("rate");
        assert!((rate - 2.0 / 3.0).abs() < 1e-9);
        assert!(report.graduated, "0.667 >= 0.66");
        assert!((report.mean_upright_fraction - 0.58).abs() < 1e-9);
        assert_eq!(report.success_rate_by_support.get("0.50"), Some(&0.5));
        let card = balance_phase_scorecard("s", "c", 1, "test", "d", "h", &report);
        for key in [
            "lowest_support_with_success",
            "success_rate_at_min_support",
            "mean_upright_fraction",
            "graduated",
            "success_rate_at_support_0.50",
            "success_rate_at_support_0.00",
        ] {
            assert!(card.metrics.contains_key(key), "missing {key}");
        }
        let mut easy = cfg();
        easy.graduate_success_rate = 0.7;
        let grad = BalancePhaseReport::from_attempts_with(report.attempts.clone(), &easy).unwrap();
        assert!(!grad.graduated);
        let none_at_min =
            BalancePhaseReport::from_attempts_with(vec![record(Success, 0.5, 1.0)], &cfg())
                .unwrap();
        assert!(!none_at_min.graduated);
        assert_eq!(none_at_min.success_rate_at_min_support, None);
    }

    #[test]
    fn success_pleasure_scales_with_support() {
        for (level, expected) in [(1.0, None), (0.5, Some(0.5)), (0.0, Some(1.0))] {
            let mut r = BalanceAffectReward::new();
            let out = reward_for(
                &mut r,
                2.0,
                level,
                false,
                Some(BalanceAttemptOutcome::Success),
            );
            let pleasure: Vec<f64> = out
                .iter()
                .filter(|(c, _)| *c == AffectChannel::Pleasure)
                .map(|(_, m)| *m)
                .collect();
            match expected {
                None => assert!(pleasure.is_empty(), "level {level}: {pleasure:?}"),
                Some(e) => {
                    assert_eq!(pleasure.len(), 1);
                    assert!(
                        (pleasure[0] - e).abs() < 1e-9,
                        "level {level}: {pleasure:?}"
                    );
                }
            }
        }
        // Falls back to the attempt's support level when the sample has none.
        let mut r = BalanceAffectReward::for_attempt(0.5);
        let mut s = sample(1, 1, 2.0);
        s.support_level = None;
        let out = r.on_sample(&s, Some(&BalanceAttemptOutcome::Success), &criterion());
        assert!((out[0].magnitude - 0.5).abs() < 1e-9);
    }

    #[test]
    fn perturbations_count_rising_edges() {
        let mut c = criterion();
        c.hold_s = 2.5;
        c.attempt_cap_s = 5.0;
        let pattern = [false, true, true, false, false, true, false, true, true];
        let mut open = OpenBalanceAttempt::fresh();
        for (i, p) in pattern.iter().enumerate() {
            let mut s = balanced(i as u64 + 1, 7);
            s.perturbing = *p;
            open = fed(&open, &s, &c);
        }
        assert_eq!(open.perturbations, 3);
        let mut s = balanced(20, 7);
        s.terminal = Some(BalanceTerminal::Death);
        match observe_balance_sample(&open, &s, &c, 7).unwrap() {
            BalanceAttemptStep::Finished(r) => assert_eq!(r.perturbations, 3),
            _ => panic!("expected finish"),
        }
    }

    #[test]
    fn slow_polls_reach_success_at_hold_bursts_elapsed() {
        // Polls every 2 bursts (steps_per_burst 1, seq advancing by 2). The first sample counts
        // as one burst, so holds after each poll are 1, 3, 5: success exactly at 5 bursts, not
        // after 5 samples.
        let mut c = criterion();
        c.hold_s = 0.25;
        c.attempt_cap_s = 2.5;
        let mut open = OpenBalanceAttempt::fresh();
        open = fed(&open, &balanced(1, 3), &c);
        open = fed(&open, &balanced(3, 3), &c);
        assert_eq!(open.holding_bursts, 3);
        assert_eq!(open.bursts, 3);
        match observe_balance_sample(&open, &balanced(5, 3), &c, 3).expect("obs") {
            BalanceAttemptStep::Finished(r) => {
                assert_eq!(r.outcome, BalanceAttemptOutcome::Success);
                assert_eq!(r.bursts, 5);
                assert_eq!(r.balanced_bursts, 5);
                assert!((r.balanced_s - 0.25).abs() < 1e-9);
                assert_eq!(r.time_to_first_violation_bursts, None);
            }
            BalanceAttemptStep::Continue(_) => panic!("5 elapsed upright bursts must succeed"),
        }
    }

    #[test]
    fn balanced_bursts_is_the_longest_upright_streak() {
        // up x3, tilt, up x5 (last one with a lifted foot is NOT in it), foot lift, up x2.
        // Streaks: 3, 5, 2. Cap 12 truncates at the 12th burst.
        let mut c = criterion();
        c.hold_s = 5.0;
        c.attempt_cap_s = 0.6;
        let mut open = OpenBalanceAttempt::fresh();
        for seq in 1..=3 {
            open = fed(&open, &balanced(seq, 1), &c);
        }
        open = fed(&open, &sample(4, 1, 30.0), &c);
        for seq in 5..=9 {
            open = fed(&open, &balanced(seq, 1), &c);
        }
        let mut lifted = balanced(10, 1);
        lifted.contacts[0] = false;
        open = fed(&open, &lifted, &c);
        assert_eq!(open.longest_balanced_bursts, 5);
        assert_eq!(open.first_violation_burst, Some(3));
        open = fed(&open, &balanced(11, 1), &c);
        match observe_balance_sample(&open, &balanced(12, 1), &c, 1).expect("obs") {
            BalanceAttemptStep::Finished(r) => {
                assert_eq!(r.outcome, BalanceAttemptOutcome::Truncated);
                assert_eq!(r.bursts, 12);
                assert_eq!(r.balanced_bursts, 5);
                assert!((r.balanced_s - 0.25).abs() < 1e-9);
                assert_eq!(r.time_to_first_violation_bursts, Some(3));
                assert!((r.time_to_first_violation_s.unwrap() - 0.15).abs() < 1e-9);
            }
            BalanceAttemptStep::Continue(_) => panic!("cap is 12"),
        }
    }

    #[test]
    fn death_burst_does_not_extend_the_balanced_run() {
        let c = criterion();
        let mut open = OpenBalanceAttempt::fresh();
        open = fed(&open, &balanced(1, 1), &c);
        let mut dead = balanced(2, 1);
        dead.terminal = Some(BalanceTerminal::Death);
        match observe_balance_sample(&open, &dead, &c, 1).expect("obs") {
            BalanceAttemptStep::Finished(r) => {
                assert_eq!(r.outcome, BalanceAttemptOutcome::Death);
                assert_eq!(r.balanced_bursts, 1);
                assert_eq!(r.time_to_first_violation_bursts, Some(1));
            }
            BalanceAttemptStep::Continue(_) => panic!("death ends the attempt"),
        }
    }

    #[test]
    fn phase_aggregates_balanced_time_overall_and_by_support() {
        let mk = |level: f64, bursts: u32| {
            let mut r = record(BalanceAttemptOutcome::Truncated, level, 0.5);
            r.balanced_bursts = bursts;
            r.balanced_s = f64::from(bursts) * 0.05;
            r
        };
        let report = BalancePhaseReport::from_attempts(vec![mk(0.5, 4), mk(0.5, 10), mk(0.0, 6)])
            .expect("report");
        assert_eq!(report.balanced.attempts, 3);
        assert!((report.balanced.mean_bursts - 20.0 / 3.0).abs() < 1e-9);
        assert_eq!(report.balanced.median_bursts, 6.0);
        assert_eq!(report.balanced.max_bursts, 10);
        assert!((report.balanced.mean_s - (20.0 / 3.0) * 0.05).abs() < 1e-9);
        assert!((report.balanced.median_s - 0.3).abs() < 1e-9);
        assert!((report.balanced.max_s - 0.5).abs() < 1e-9);
        let half = &report.balanced_by_support["0.50"];
        assert_eq!(half.attempts, 2);
        assert_eq!(half.mean_bursts, 7.0);
        assert_eq!(half.median_bursts, 7.0);
        assert_eq!(half.max_bursts, 10);
        assert!((half.mean_s - 0.35).abs() < 1e-9);
        let zero = &report.balanced_by_support["0.00"];
        assert_eq!((zero.mean_bursts, zero.max_bursts), (6.0, 6));
        let metrics = balance_phase_metrics(&report);
        assert_eq!(metrics["balanced_bursts_max"], 10.0);
        assert_eq!(metrics["balanced_bursts_median"], 6.0);
        assert!((metrics["balanced_s_mean_at_support_0.50"] - 0.35).abs() < 1e-9);
        assert_eq!(metrics["balanced_bursts_max_at_support_0.00"], 6.0);
    }

    fn need_sample(need: Option<f64>, tilt: f64, settling: bool) -> BalanceSample {
        BalanceSample {
            support_need: need,
            settling,
            ..sample(1, 1, tilt)
        }
    }

    fn assist_step(
        r: &mut BalanceAssistReward,
        need: f64,
        outcome: Option<BalanceAttemptOutcome>,
    ) -> Vec<(AffectChannel, f64)> {
        r.on_sample(
            &need_sample(Some(need), 0.0, false),
            outcome.as_ref(),
            &criterion(),
        )
        .into_iter()
        .map(|s| (s.channel, s.magnitude))
        .collect()
    }

    #[test]
    fn assist_reward_follows_need_sequence() {
        let p = AffectChannel::Pain;
        let g = AffectChannel::Pleasure;
        let mut r = BalanceAssistReward::new();
        assert_eq!(assist_step(&mut r, 0.05, None), vec![(g, 0.2)]);
        assert_eq!(assist_step(&mut r, 0.3, None), vec![(p, 0.3)]);
        assert_eq!(assist_step(&mut r, 0.6, None), vec![(p, 0.6)]);
        let falling = assist_step(&mut r, 0.5, None);
        assert_eq!(falling.len(), 1);
        assert_eq!(falling[0].0, g);
        assert!((falling[0].1 - 0.4).abs() < 1e-9);
        // Tiny drop is below the Pleasure floor: silent.
        assert!(assist_step(&mut r, 0.499, None).is_empty());
        // Steady above the deadband: nothing.
        assert!(assist_step(&mut r, 0.499, None).is_empty());
        // Falling into the deadband: largest Pleasure (fall 1.0 capped).
        assert_eq!(assist_step(&mut r, 0.05, None), vec![(g, 1.0)]);
        assert_eq!(assist_step(&mut r, 0.05, None), vec![(g, 0.2)]);
        assert_eq!(
            assist_step(&mut r, 0.05, Some(BalanceAttemptOutcome::Success)),
            vec![(g, 1.0)]
        );
        assert_eq!(
            assist_step(&mut r, 0.05, Some(BalanceAttemptOutcome::Death)),
            vec![(p, 1.0)]
        );
    }

    #[test]
    fn assist_reward_edge_cases() {
        let c = criterion();
        let mut r = BalanceAssistReward::new();
        // Settling is silent; death while settling is Pain 1.
        assert!(r
            .on_sample(&need_sample(Some(0.9), 0.0, true), None, &c)
            .is_empty());
        let d = r.on_sample(
            &need_sample(Some(0.9), 0.0, true),
            Some(&BalanceAttemptOutcome::Death),
            &c,
        );
        assert_eq!((d[0].channel, d[0].magnitude), (AffectChannel::Pain, 1.0));
        // No support_need: falls back to tilt / max_tilt (max_tilt 15 deg): 12 deg -> 0.8.
        let mut r = BalanceAssistReward::new();
        let _ = r.on_sample(&need_sample(None, 2.0, false), None, &c);
        let s = r.on_sample(&need_sample(None, 12.0, false), None, &c);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].channel, AffectChannel::Pain);
        assert!((s[0].magnitude - 0.8).abs() < 1e-9);
        // Feet up inside the deadband: no holding Pleasure.
        let mut r = BalanceAssistReward::new();
        let mut up = need_sample(Some(0.0), 0.0, false);
        up.contacts[0] = false;
        assert!(r.on_sample(&up, None, &c).is_empty());
        assert_eq!(BalanceAssistReward::policy_id(), "reward.balance-assist.v1");
        assert_eq!(
            BalanceRewardPolicy::from_mode("assist"),
            Some(BalanceRewardPolicy::Assist)
        );
        assert_eq!(BalanceRewardPolicy::from_mode("none"), None);
    }

    #[test]
    fn graduation_uses_median_balanced_s_when_set() {
        let mut config = cfg();
        config.graduate_balanced_s = Some(1.0);
        // Median of [0.2, 1.5, 2.0] = 1.5 >= 1.0 passes though only 0 successes: lowers.
        let mut s = SupportSchedule::new(config);
        for t in [0.2, 1.5, 2.0] {
            s.record_attempt(BalanceAttemptOutcome::Truncated, t);
        }
        assert!((s.current_level() - 0.9).abs() < 1e-9);
        // Median of [0.2, 0.3, 5.0] = 0.3 < 1.0 fails even with 2 successes: holds.
        let mut s = SupportSchedule::new(config);
        s.record_attempt(BalanceAttemptOutcome::Success, 5.0);
        s.record_attempt(BalanceAttemptOutcome::Success, 0.3);
        s.record_attempt(BalanceAttemptOutcome::Death, 0.2);
        assert!((s.current_level() - 1.0).abs() < 1e-9);
        // None keeps the success-rate rule: 2 of 3 successes lowers.
        let mut s = SupportSchedule::new(cfg());
        s.record_attempt(BalanceAttemptOutcome::Success, 0.0);
        s.record_attempt(BalanceAttemptOutcome::Success, 0.0);
        s.record_attempt(BalanceAttemptOutcome::Death, 0.0);
        assert!((s.current_level() - 0.9).abs() < 1e-9);
        let mut bad = criterion();
        bad.graduate_balanced_s = Some(0.0);
        assert!(bad.validate().is_err());
    }

    // ---- mock controller: the generic physics primitives of the contract ----

    #[derive(Debug, Clone, Copy)]
    struct Scripted {
        tilt_deg: f64,
        down: bool,
        terminal: Option<BalanceTerminal>,
    }

    fn upright_step() -> Scripted {
        Scripted {
            tilt_deg: 2.0,
            down: true,
            terminal: None,
        }
    }

    /// `script(measure_index_since_reset, seconds_since_reset)`.
    type Script = Box<dyn FnMut(u32, f64) -> Scripted>;

    struct MockController {
        info: ModelInfo,
        advance_s: f64,
        script: Script,
        contact_count: usize,
        floor_z: Option<f64>,
        calls: Vec<String>,
        resets: Vec<Option<String>>,
        spring_sets: Vec<(SpringGains, f64, f64)>,
        /// (level, seconds since reset when it was sent)
        level_updates: Vec<(f64, f64)>,
        /// (torque, force, duration_s, seconds since reset)
        pulses: Vec<([f64; 3], [f64; 3], f64, f64)>,
        measure_requests: Vec<MeasureRequest>,
        spring_level: Option<f64>,
        k: u32,
        fresh: bool,
        steps: u64,
    }

    /// Howard-like model: base is the free root (0.737 kg), torso a child body.
    fn howard_info() -> ModelInfo {
        let body = |name: &str, root: &str, subtree: f64, inertia: [f64; 3]| ModelBody {
            name: name.to_string(),
            root_name: root.to_string(),
            mass: subtree,
            subtree_mass: subtree,
            inertia,
            subtree_inertia: [5e-3, 4e-3, 1.3e-3],
        };
        ModelInfo {
            timestep_s: 0.002,
            gravity: [0.0, 0.0, -9.81],
            bodies: vec![
                body("base", "base", 0.737, [2e-3, 2e-3, 1e-3]),
                body("torso", "base", 0.6, [2e-3, 2e-3, 1e-3]),
                body("left_foot", "base", 0.05, [1e-5; 3]),
                body("right_foot", "base", 0.05, [1e-5; 3]),
                body("tail_foot", "base", 0.05, [1e-5; 3]),
            ],
            free_roots: vec!["base".to_string()],
            keyframes: vec![serde_json::json!("home"), serde_json::json!("stand")],
        }
    }

    impl MockController {
        fn new(advance_s: f64, contact_count: usize, script: Script) -> Self {
            Self {
                info: howard_info(),
                advance_s,
                script,
                contact_count,
                floor_z: Some(0.01),
                calls: Vec::new(),
                resets: Vec::new(),
                spring_sets: Vec::new(),
                level_updates: Vec::new(),
                pulses: Vec::new(),
                measure_requests: Vec::new(),
                spring_level: None,
                k: 0,
                fresh: false,
                steps: 0,
            }
        }

        fn seconds(&self) -> f64 {
            f64::from(self.k) * self.advance_s
        }

        fn count(&self, action: &str) -> usize {
            self.calls.iter().filter(|call| *call == action).count()
        }
    }

    impl BalanceController for MockController {
        fn model_info(&mut self) -> Result<ModelInfo, TrainerError> {
            self.calls.push("get_model_info".to_string());
            Ok(self.info.clone())
        }

        fn reset_episode(&mut self, keyframe: Option<&str>) -> Result<u64, TrainerError> {
            self.calls.push("reset_episode".to_string());
            self.resets.push(keyframe.map(str::to_string));
            self.k = 0;
            self.fresh = true;
            self.spring_level = None;
            Ok(self.resets.len() as u64)
        }

        fn set_body_spring(
            &mut self,
            _body: &str,
            gains: &SpringGains,
            level: f64,
            deadband_rad: f64,
        ) -> Result<(), TrainerError> {
            self.calls.push("set_body_spring".to_string());
            self.spring_sets.push((*gains, level, deadband_rad));
            self.spring_level = Some(level);
            Ok(())
        }

        fn update_body_spring(&mut self, _body: &str, level: f64) -> Result<(), TrainerError> {
            self.calls.push("update_body_spring".to_string());
            self.level_updates.push((level, self.seconds()));
            self.spring_level = Some(level);
            Ok(())
        }

        fn clear_body_spring(&mut self, _body: &str) -> Result<(), TrainerError> {
            self.calls.push("clear_body_spring".to_string());
            self.spring_level = None;
            Ok(())
        }

        fn apply_body_pulse(
            &mut self,
            _body: &str,
            torque: [f64; 3],
            force: [f64; 3],
            duration_s: f64,
        ) -> Result<(), TrainerError> {
            self.calls.push("apply_body_pulse".to_string());
            self.pulses
                .push((torque, force, duration_s, self.seconds()));
            Ok(())
        }

        fn measure(&mut self, request: &MeasureRequest) -> Result<Measurement, TrainerError> {
            self.calls.push("measure".to_string());
            self.measure_requests.push(request.clone());
            if self.fresh {
                self.fresh = false;
            } else {
                self.k += 1;
                self.steps = (self.seconds() / self.info.timestep_s).round() as u64;
            }
            let (k, seconds) = (self.k, self.seconds());
            let scripted = (self.script)(k, seconds);
            let touching_z = self.floor_z.filter(|_| scripted.down);
            let contacts = (0..self.contact_count)
                .map(|_| ContactMeasure {
                    touching: scripted.down,
                    points: touching_z.map(|z| vec![[0.0, 0.0, z]]).unwrap_or_default(),
                })
                .collect();
            let mut bodies = BTreeMap::new();
            bodies.insert(
                request.bodies[0].clone(),
                BodyMeasure {
                    tilt_rad: scripted.tilt_deg.to_radians(),
                    tilt_vec: [0.0, 0.0],
                    ang_vel: [0.0; 3],
                    com: [0.0, 0.0, 0.1],
                    subtree_com: [0.0, 0.0, 0.150],
                },
            );
            let mut springs = BTreeMap::new();
            if let Some(level) = self.spring_level {
                springs.insert(
                    request.bodies[0].clone(),
                    SpringMeasure {
                        wrench_norm_torque: level * 0.5,
                        wrench_norm_force: level * 0.5,
                        gravity_comp_force_n: level * 0.737 * 9.81,
                    },
                );
            }
            Ok(Measurement {
                sim_time_s: self.seconds(),
                physics_steps: self.steps,
                episode_id: self.resets.len() as u64,
                terminal: scripted.terminal,
                bodies,
                contacts,
                springs,
            })
        }
    }

    fn contact_names(count: usize) -> Vec<String> {
        ["left_foot", "right_foot", "tail_foot"][..count]
            .iter()
            .map(|name| name.to_string())
            .collect()
    }

    fn criterion_for(contacts: usize, period_s: f64) -> BalanceFeetCriterion {
        let mut c = BalanceFeetCriterion::new("torso", contact_names(contacts), 15.0, 1.0, 3.0);
        c.burst_period_s = period_s;
        c
    }

    fn run_phase(
        c: &BalanceFeetCriterion,
        controller: &mut MockController,
        phase: &str,
        seeds: &[u64],
    ) -> Result<BalancePhaseReport, TrainerError> {
        let mut events = CollectingEventSink::default();
        let run_id = RunId("balance-test".to_string());
        let mut run = BalancePhaseRun {
            phase,
            seeds,
            criterion: c,
            run_id: &run_id,
            support_mode: None,
            deliver_reward: None,
            reward_policy: BalanceRewardPolicy::Affect,
        };
        run_balance_phase(
            &mut run,
            controller,
            &mut |_| Ok(()),
            &mut events,
            &CancelToken::new(),
        )
    }

    // ---- gain sizing ----

    #[test]
    fn gain_sizing_reproduces_howards_previous_values() {
        let info = howard_info();
        let inputs = sizing_inputs(&info, "torso", 0.140).expect("inputs");
        assert!((inputs.mass_kg - 0.737).abs() < 1e-12);
        assert!((inputs.i_tilt - 5e-3).abs() < 1e-12);
        assert!((inputs.i_min - 1.3e-3).abs() < 1e-12);
        let g = size_spring_gains(&inputs);
        println!(
            "sized: k_rot {:.2} d_rot {:.3} k_z {:.0} d_z {:.1} k_xy {:.0} d_xy {:.1}",
            g.k_rot, g.d_rot, g.k_lin[2], g.d_lin[2], g.k_lin[0], g.d_lin[0]
        );
        assert!((34.0..=36.0).contains(&g.k_rot), "k_rot {}", g.k_rot);
        assert!((g.d_rot - 0.333).abs() < 0.02, "d_rot {}", g.d_rot);
        assert!(
            (11_000.0..=12_000.0).contains(&g.k_lin[2]),
            "k_z {}",
            g.k_lin[2]
        );
        assert!((g.d_lin[2] - 120.0).abs() < 3.0, "d_z {}", g.d_lin[2]);
        assert!(
            (3_900.0..=4_100.0).contains(&g.k_lin[0]),
            "k_xy {}",
            g.k_lin[0]
        );
        assert!((g.d_lin[0] - 80.0).abs() < 2.0, "d_xy {}", g.d_lin[0]);
        assert_eq!(g.k_lin[0], g.k_lin[1]);
        assert_eq!(g.gravity_comp, 1.0);
    }

    #[test]
    fn d_rot_is_capped_for_explicit_force_stability() {
        let mut info = howard_info();
        info.bodies[0].subtree_inertia = [5e-3, 4e-3, 1.0e-5];
        let inputs = sizing_inputs(&info, "torso", 0.140).expect("inputs");
        let g = size_spring_gains(&inputs);
        assert!(
            (g.d_rot - 0.9 * 2.0 * 1.0e-5 / 0.002).abs() < 1e-12,
            "{}",
            g.d_rot
        );
    }

    #[test]
    fn limp_torso_inertia_caps_k_rot_and_d_rot() {
        let mut info = howard_info();
        info.bodies[1].inertia = [1e-5, 1e-5, 1e-5];
        let inputs = sizing_inputs(&info, "torso", 0.140).expect("inputs");
        let g = size_spring_gains(&inputs);
        assert!(
            (g.k_rot - 0.25 * 1e-5 / (0.002 * 0.002)).abs() < 1e-9,
            "{}",
            g.k_rot
        );
        assert!(g.d_rot * 0.002 / 1e-5 < 2.0, "d dt / I < 2: {}", g.d_rot);
    }

    #[test]
    fn explicit_overrides_win_over_sizing() {
        let inputs = sizing_inputs(&howard_info(), "torso", 0.140).expect("inputs");
        let mut c = criterion_for(2, 0.05);
        let sized = gains_for(&inputs, &c);
        assert_eq!(sized, size_spring_gains(&inputs));
        c.support_rot_stiffness_nm_per_rad = Some(50.0);
        c.support_height_stiffness_n_per_m = Some(7.0);
        c.support_height_damping_n_s_per_m = Some(8.0);
        c.support_xy_stiffness_n_per_m = Some(9.0);
        c.support_xy_damping_n_s_per_m = Some(10.0);
        c.support_gravity_comp = Some(0.5);
        let g = gains_for(&inputs, &c);
        assert_eq!(g.k_rot, 50.0);
        assert!(
            (g.d_rot - 0.8 * (50.0_f64 * 5e-3).sqrt()).abs() < 1e-9,
            "k feeds d"
        );
        assert_eq!(g.k_lin, [9.0, 9.0, 7.0]);
        assert_eq!(g.d_lin, [10.0, 10.0, 8.0]);
        assert_eq!(g.gravity_comp, 0.5);
        c.support_rot_damping_nm_s_per_rad = Some(0.11);
        assert_eq!(gains_for(&inputs, &c).d_rot, 0.11);
    }

    #[test]
    fn sizing_names_what_is_missing() {
        let mut info = howard_info();
        info.bodies[0].subtree_mass = 0.0;
        let error = sizing_inputs(&info, "torso", 0.140)
            .unwrap_err()
            .to_string();
        assert!(error.contains("root subtree mass"), "{error}");
        let error = sizing_inputs(&howard_info(), "nope", 0.140)
            .unwrap_err()
            .to_string();
        assert!(error.contains("nope"), "{error}");
    }

    // ---- a full attempt against the mock ----

    #[test]
    fn attempt_runs_reset_spring_measure_loop_to_success() {
        let mut c = criterion_for(2, 0.05);
        c.hold_s = 0.5;
        c.attempt_cap_s = 2.0;
        c.reset_keyframe = "stand".to_string();
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let report = run_phase(&c, &mut mock, "test", &[7]).expect("phase");
        let attempt = &report.attempts[0];
        assert_eq!(attempt.outcome, BalanceAttemptOutcome::Success);
        assert_eq!(
            attempt.bursts, 10,
            "0.5 s hold at 20 Hz, settle not counted"
        );
        assert!((attempt.balanced_s - 0.5).abs() < 1e-9);
        assert_eq!(mock.resets, vec![Some("stand".to_string())]);
        assert_eq!(
            &mock.calls[..4],
            [
                "get_model_info",
                "reset_episode",
                "measure",
                "set_body_spring"
            ]
        );
        assert_eq!(
            mock.calls.last().map(String::as_str),
            Some("clear_body_spring")
        );
        // settle (0.5 s) at level 1, then level 0 (test phase = support_min)
        assert_eq!(mock.spring_sets[0].1, 1.0);
        assert_eq!(mock.level_updates.len(), 1);
        assert_eq!(mock.level_updates[0].0, 0.0);
        assert!(
            (mock.level_updates[0].1 - 0.5).abs() < 0.051,
            "{:?}",
            mock.level_updates
        );
        assert!((mock.spring_sets[0].2 - 6.0_f64.to_radians()).abs() < 1e-12);
        assert_eq!(
            mock.count("measure"),
            1 + 9 + 10,
            "sizing read + 9 settle + 10 hold"
        );
        assert!(report.spring_gains.is_some());
    }

    #[test]
    fn three_contact_bodies_are_measured_and_all_must_be_down() {
        let c = criterion_for(3, 0.05);
        let mut mock = MockController::new(0.05, 3, Box::new(|_, _| upright_step()));
        let report = run_phase(&c, &mut mock, "train", &[1]).expect("phase");
        assert_eq!(report.attempts[0].outcome, BalanceAttemptOutcome::Success);
        let request = &mock.measure_requests[0];
        assert_eq!(request.bodies, vec!["torso".to_string()]);
        assert_eq!(
            request.contacts,
            vec![
                vec!["left_foot".to_string()],
                vec!["right_foot".to_string()],
                vec!["tail_foot".to_string()]
            ]
        );
        // a lifted foot is not upright: nothing balances, so the cap truncates
        let mut c2 = criterion_for(3, 0.05);
        c2.attempt_cap_s = 1.5;
        let mut lifted = MockController::new(
            0.05,
            3,
            Box::new(|_, _| Scripted {
                down: false,
                ..upright_step()
            }),
        );
        lifted.floor_z = Some(0.01);
        // sizing needs contact points at the reset, so give it an explicit height
        c2.support_com_height_m = Some(0.140);
        let report = run_phase(&c2, &mut lifted, "train", &[1]).expect("phase");
        assert_eq!(report.attempts[0].outcome, BalanceAttemptOutcome::Truncated);
        assert_eq!(report.attempts[0].balanced_bursts, 0);
    }

    #[test]
    fn death_interrupt_and_truncation_end_the_attempt() {
        let c = criterion_for(2, 0.05);
        // death at 1.0 s
        let mut mock = MockController::new(
            0.05,
            2,
            Box::new(|k, _| {
                if k >= 20 {
                    Scripted {
                        terminal: Some(BalanceTerminal::Death),
                        tilt_deg: 40.0,
                        ..upright_step()
                    }
                } else {
                    Scripted {
                        tilt_deg: if k > 12 { 30.0 } else { 2.0 },
                        ..upright_step()
                    }
                }
            }),
        );
        let report = run_phase(&c, &mut mock, "train", &[1]).expect("phase");
        assert_eq!(report.attempts[0].outcome, BalanceAttemptOutcome::Death);
        // interrupted
        let mut mock = MockController::new(
            0.05,
            2,
            Box::new(|k, _| {
                if k >= 15 {
                    Scripted {
                        terminal: Some(BalanceTerminal::Interrupted),
                        ..upright_step()
                    }
                } else {
                    Scripted {
                        tilt_deg: 30.0,
                        ..upright_step()
                    }
                }
            }),
        );
        let report = run_phase(&c, &mut mock, "train", &[1]).expect("phase");
        assert_eq!(
            report.attempts[0].outcome,
            BalanceAttemptOutcome::Interrupted
        );
        // truncated: leaning past the limit for the whole cap (3.0 s = 60 bursts)
        let mut mock = MockController::new(
            0.05,
            2,
            Box::new(|_, _| Scripted {
                tilt_deg: 30.0,
                ..upright_step()
            }),
        );
        let report = run_phase(&c, &mut mock, "train", &[1]).expect("phase");
        assert_eq!(report.attempts[0].outcome, BalanceAttemptOutcome::Truncated);
        assert_eq!(report.attempts[0].bursts, 60);
    }

    #[test]
    fn gains_are_sized_once_and_reused_every_attempt() {
        let c = criterion_for(2, 0.05);
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let report = run_phase(&c, &mut mock, "train", &[1, 2, 3]).expect("phase");
        assert_eq!(report.attempts.len(), 3);
        assert_eq!(mock.count("get_model_info"), 1);
        assert_eq!(mock.count("reset_episode"), 3);
        assert_eq!(mock.spring_sets.len(), 3);
        assert!(mock
            .spring_sets
            .iter()
            .all(|set| set.0 == mock.spring_sets[0].0));
        assert_eq!(report.spring_gains, Some(mock.spring_sets[0].0));
        // h0 = subtree com z 0.150 - contact z 0.01 = 0.140: Howard's k_rot
        assert!((34.0..=36.0).contains(&mock.spring_sets[0].0.k_rot));
    }

    #[test]
    fn explicit_gain_overrides_reach_the_controller() {
        let mut c = criterion_for(2, 0.05);
        c.support_rot_stiffness_nm_per_rad = Some(33.0);
        c.support_rot_damping_nm_s_per_rad = Some(0.25);
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        run_phase(&c, &mut mock, "train", &[1]).expect("phase");
        assert_eq!(mock.spring_sets[0].0.k_rot, 33.0);
        assert_eq!(mock.spring_sets[0].0.d_rot, 0.25);
    }

    #[test]
    fn missing_contact_points_or_keyframe_stop_with_a_named_error() {
        let c = criterion_for(2, 0.05);
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        mock.floor_z = None;
        let error = run_phase(&c, &mut mock, "train", &[1])
            .unwrap_err()
            .to_string();
        assert!(error.contains("no contact points"), "{error}");
        let mut c = criterion_for(2, 0.05);
        c.reset_keyframe = "crouch".to_string();
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let error = run_phase(&c, &mut mock, "train", &[1])
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("crouch") && error.contains("home"),
            "{error}"
        );
        assert_eq!(mock.count("reset_episode"), 0, "checked before any reset");
        let mut c = criterion_for(2, 0.05);
        c.contact_bodies[1] = "ghost".to_string();
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let error = run_phase(&c, &mut mock, "train", &[1])
            .unwrap_err()
            .to_string();
        assert!(error.contains("ghost"), "{error}");
    }

    #[test]
    fn perturbation_pulses_are_scheduled_by_the_trainer_after_settling() {
        let mut c = criterion_for(2, 0.05);
        c.perturb_interval_s = 0.5;
        c.perturb_duration_s = 0.1;
        c.perturb_torque_nm = 0.3;
        c.hold_s = 5.0;
        c.attempt_cap_s = 5.0;
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let report = run_phase(&c, &mut mock, "train", &[4]).expect("phase");
        assert!(mock.pulses.len() >= 5, "{} pulses", mock.pulses.len());
        for (torque, force, duration, at_s) in &mock.pulses {
            assert_eq!(*duration, 0.1);
            assert_eq!(*force, [0.0; 3]);
            assert_eq!(torque[2], 0.0);
            assert!((torque[0].abs() + torque[1].abs() - 0.3).abs() < 1e-12);
            assert!(*at_s >= c.settle_s, "no pulse during settle: {at_s}");
        }
        assert_eq!(report.attempts[0].perturbations as usize, mock.pulses.len());
        // same seed, same schedule
        let mut again = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        run_phase(&c, &mut again, "train", &[4]).expect("phase");
        assert_eq!(mock.pulses, again.pulses);
    }

    #[test]
    fn durations_stay_equal_in_seconds_at_10_hz_and_200_hz() {
        assert_eq!(bursts_for_seconds(1.0, 0.1), 10);
        assert_eq!(bursts_for_seconds(1.0, 0.005), 200);
        let mut seen = Vec::new();
        for hz in [10.0_f64, 200.0] {
            let period = 1.0 / hz;
            let mut c = criterion_for(2, period);
            c.settle_s = 0.5;
            c.hold_s = 1.0;
            c.attempt_cap_s = 4.0;
            c.perturb_interval_s = 0.4;
            c.perturb_duration_s = 0.1;
            let mut mock = MockController::new(period, 2, Box::new(|_, _| upright_step()));
            let report = run_phase(&c, &mut mock, "test", &[3]).expect("phase");
            let attempt = &report.attempts[0];
            assert_eq!(attempt.outcome, BalanceAttemptOutcome::Success);
            let end_s = mock.seconds();
            let settle_end_s = mock.level_updates[0].1;
            println!(
                "{hz} Hz: bursts {} balanced_s {} settle_end {settle_end_s} success_at {end_s} pulses {}",
                attempt.bursts, attempt.balanced_s, mock.pulses.len()
            );
            assert!(
                (attempt.balanced_s - 1.0).abs() <= period + 1e-9,
                "{}",
                attempt.balanced_s
            );
            assert!(
                (settle_end_s - 0.5).abs() <= period + 1e-9,
                "{settle_end_s}"
            );
            assert!((end_s - 1.5).abs() <= 2.0 * period + 1e-9, "{end_s}");
            assert!(
                mock.pulses.iter().all(|p| p.2 == 0.1),
                "pulse length is seconds"
            );
            seen.push((attempt.balanced_s, settle_end_s, end_s, mock.pulses.len()));
        }
        assert!((seen[0].0 - seen[1].0).abs() <= 0.1 + 1e-9);
        assert!((seen[0].2 - seen[1].2).abs() <= 0.2 + 1e-9);
    }

    #[test]
    fn test_phase_delivers_reward_from_trainer_samples() {
        let mut c = criterion_for(2, 0.05);
        c.settle_s = 0.0;
        c.hold_s = 0.1;
        c.attempt_cap_s = 1.0;
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let mut delivered: Vec<RewardSignal> = Vec::new();
        let mut sink = |signals: &[RewardSignal]| -> Result<(), TrainerError> {
            delivered.extend_from_slice(signals);
            Ok(())
        };
        let mut events = CollectingEventSink::default();
        let run_id = RunId("balance-test".to_string());
        let mut run = BalancePhaseRun {
            phase: "test",
            seeds: &[1],
            criterion: &c,
            run_id: &run_id,
            support_mode: None,
            deliver_reward: Some(&mut sink),
            reward_policy: BalanceRewardPolicy::Affect,
        };
        run_balance_phase(
            &mut run,
            &mut mock,
            &mut |_| Ok(()),
            &mut events,
            &CancelToken::new(),
        )
        .expect("phase");
        // burst 1: support 0 and upright -> 0.1; burst 2: success -> 1.0
        assert_eq!(delivered.len(), 2);
        assert_eq!(delivered[0].magnitude, 0.1);
        assert_eq!(delivered[1].channel, AffectChannel::Pleasure);
        assert_eq!(delivered[1].magnitude, 1.0);
    }

    #[test]
    fn support_need_and_effort_follow_the_documented_formulas() {
        // (tilt + 0.1 s * |omega_xy|) / max_tilt, clamped
        let need = support_need(5.0_f64.to_radians(), [0.1, 0.0, 9.0], 15.0, 0.1);
        let expected = (5.0_f64.to_radians() + 0.01) / 15.0_f64.to_radians();
        assert!((need - expected).abs() < 1e-12);
        assert_eq!(support_need(1.0, [0.0; 3], 15.0, 0.1), 1.0);
        let sizing = sizing_inputs(&howard_info(), "torso", 0.140).unwrap();
        let weight = 0.737 * 9.81;
        let wrench = SpringMeasure {
            wrench_norm_torque: weight * 0.140 / 2.0,
            wrench_norm_force: weight / 4.0,
            gravity_comp_force_n: weight,
        };
        // torque share 0.5; restoring force weight/4 -> 0.25
        let effort = support_effort(wrench, &sizing);
        assert!((effort - 0.375).abs() < 1e-9, "{effort}");
    }

    #[test]
    fn criterion_validation_covers_the_new_fields() {
        let mut c = criterion_for(2, 0.05);
        assert!(c.validate().is_ok());
        c.contact_bodies.clear();
        assert!(c.validate().is_err());
        let mut c = criterion_for(2, 0.05);
        c.contact_bodies[1] = "left_foot".to_string();
        assert!(c.validate().is_err(), "duplicate contact body");
        let mut c = criterion_for(2, 0.05);
        c.hold_s = 5.0;
        assert!(c.validate().is_err(), "hold above cap");
        let mut c = criterion_for(2, 0.05);
        c.support_min = 0.5;
        c.support_start = 0.4;
        assert!(c.validate().is_err());
        let mut c = criterion_for(2, 0.05);
        c.burst_period_s = 0.0;
        assert!(c.validate().is_err());
        let mut c = criterion_for(2, 0.05);
        c.perturb_duration_s = 0.0;
        assert!(c.validate().is_err());
        let mut c = criterion_for(2, 0.05);
        c.support_gravity_comp = Some(1.5);
        assert!(c.validate().is_err());
        let c: BalanceFeetCriterion = serde_json::from_value(serde_json::json!({
            "torso_body": "t", "contact_bodies": ["a", "b", "c"],
            "max_tilt_deg": 10.0, "hold_s": 1.0, "attempt_cap_s": 2.0
        }))
        .expect("minimal criterion");
        assert_eq!(c.contact_bodies.len(), 3);
        assert_eq!(c.reset_keyframe, "");
        assert_eq!(c.settle_s, 0.5);
        assert_eq!(c.attempts_per_level, 3);
        assert_eq!(c.support_deadband_deg, 6.0);
        assert_eq!(c.perturb_interval_s, 0.0);
        assert_eq!(
            c.support_rot_stiffness_nm_per_rad, None,
            "no fixed stiffness default"
        );
    }

    // ---- wire format and keep-alive ----

    #[test]
    fn request_bodies_follow_the_controller_contract() {
        let gains = SpringGains {
            k_rot: 28.0,
            d_rot: 0.3,
            k_lin: [4000.0, 4000.0, 11600.0],
            d_lin: [80.0, 80.0, 120.0],
            gravity_comp: 1.0,
        };
        let body = set_body_spring_request("torso", &gains, 1.0, 0.1);
        assert_eq!(body["action"], "set_body_spring");
        assert_eq!(body["body"], "torso");
        assert_eq!(body["k_lin"], serde_json::json!([4000.0, 4000.0, 11600.0]));
        assert_eq!(body["ref"], "current");
        assert_eq!(body["deadband_rad"], 0.1);
        assert_eq!(update_body_spring_request("torso", 0.4)["level"], 0.4);
        assert_eq!(
            clear_body_spring_request("torso")["action"],
            "clear_body_spring"
        );
        let pulse = apply_body_pulse_request("torso", [0.3, 0.0, 0.0], [0.0; 3], 0.1);
        assert_eq!(pulse["duration_s"], 0.1);
        assert_eq!(pulse["torque"], serde_json::json!([0.3, 0.0, 0.0]));
        assert!(reset_episode_request(None).get("keyframe").is_none());
        assert_eq!(reset_episode_request(Some("stand"))["keyframe"], "stand");
        let m = measure_request_body(&MeasureRequest {
            bodies: vec!["torso".to_string()],
            contacts: vec![vec!["a".to_string()], vec!["b".to_string()]],
        });
        assert_eq!(m["bodies"], serde_json::json!(["torso"]));
        assert_eq!(
            m["contacts"][1],
            serde_json::json!({"a": ["b"], "b": "world"})
        );
    }

    #[test]
    fn measurement_parses_object_and_array_forms() {
        let object = serde_json::json!({
            "sim_time_s": 1.5, "physics_steps": 750, "episode_id": 2, "terminal": "death",
            "bodies": {"torso": {"tilt_rad": 0.1, "ang_vel": [0.0, 0.2, 0.0],
                                 "subtree_com": [0.0, 0.0, 0.124]}},
            "contacts": [{"touching": true, "points": [[0.0, 0.0, 0.01]]}, {"touching": false}],
            "springs": {"torso": {"wrench_norm_torque": 0.4, "wrench_norm_force": 7.0}}
        });
        let m: Measurement = serde_json::from_value(object).expect("object form");
        assert_eq!(m.terminal, Some(BalanceTerminal::Death));
        assert_eq!(m.bodies["torso"].tilt_rad, 0.1);
        assert_eq!(m.mean_contact_height(), Some(0.01));
        assert_eq!(m.springs["torso"].wrench_norm_force, 7.0);
        let array = serde_json::json!({
            "sim_time_s": 0.0, "physics_steps": 0,
            "bodies": [{"name": "torso", "tilt_rad": 0.0}],
            "contacts": [], "springs": [{"body": "torso", "wrench_norm_torque": 0.0, "wrench_norm_force": 0.0}]
        });
        let m: Measurement = serde_json::from_value(array).expect("array form");
        assert!(m.bodies.contains_key("torso") && m.springs.contains_key("torso"));
        assert_eq!(m.terminal, None);
        let info: ModelInfo = serde_json::from_value(serde_json::json!({
            "timestep_s": 0.002, "gravity": [0.0, 0.0, -9.81],
            "bodies": [{"name": "b", "id": 1, "parent": "world", "root_name": "b", "mass": 1.0,
                        "subtree_mass": 1.0, "subtree_com": [0.0, 0.0, 0.0],
                        "inertia": [1.0, 1.0, 1.0]}],
            "free_roots": ["b"], "keyframes": ["home", ""], "actuators": [], "joints": []
        }))
        .expect("model info");
        assert_eq!(info.keyframe_names(), vec!["home".to_string()]);
    }

    /// Tiny HTTP server: answers every request on one accepted connection until it closes.
    fn serve(close_each: bool) -> (u16, std::sync::Arc<std::sync::atomic::AtomicU32>) {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().unwrap().port();
        let accepted = Arc::new(AtomicU32::new(0));
        let counter = accepted.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { return };
                counter.fetch_add(1, Ordering::SeqCst);
                std::thread::spawn(move || {
                    let mut reader = BufReader::new(stream);
                    loop {
                        let mut length = 0usize;
                        let mut first = true;
                        loop {
                            let mut line = String::new();
                            match reader.read_line(&mut line) {
                                Ok(0) | Err(_) => return,
                                Ok(_) => {}
                            }
                            if first {
                                first = false;
                            }
                            let trimmed = line.trim_end().to_ascii_lowercase();
                            if let Some(v) = trimmed.strip_prefix("content-length:") {
                                length = v.trim().parse().unwrap_or(0);
                            }
                            if trimmed.is_empty() {
                                break;
                            }
                        }
                        let mut body = vec![0u8; length];
                        if reader.read_exact(&mut body).is_err() {
                            return;
                        }
                        let payload = br#"{"ok":true,"result":{"episode_id":5}}"#;
                        let connection = if close_each { "close" } else { "keep-alive" };
                        let head = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: {connection}\r\n\r\n",
                            payload.len()
                        );
                        let socket = reader.get_mut();
                        if socket.write_all(head.as_bytes()).is_err()
                            || socket.write_all(payload).is_err()
                        {
                            return;
                        }
                        if close_each {
                            return;
                        }
                    }
                });
            }
        });
        (port, accepted)
    }

    #[test]
    fn one_keep_alive_connection_serves_every_call() {
        let (port, accepted) = serve(false);
        let mut http =
            HttpBalanceController::new(&format!("http://127.0.0.1:{port}"), Duration::from_secs(5))
                .expect("client");
        for _ in 0..25 {
            assert_eq!(http.reset_episode(None).expect("reset"), 5);
        }
        assert_eq!(http.connections_opened(), 1);
        assert_eq!(accepted.load(std::sync::atomic::Ordering::SeqCst), 1);
        // a server that closes after each reply still works: the client reconnects
        let (port, accepted) = serve(true);
        let mut http =
            HttpBalanceController::new(&format!("http://127.0.0.1:{port}"), Duration::from_secs(5))
                .expect("client");
        for _ in 0..3 {
            assert_eq!(http.reset_episode(Some("stand")).expect("reset"), 5);
        }
        assert_eq!(http.connections_opened(), 3);
        assert_eq!(accepted.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[test]
    fn error_envelope_and_http_status_surface() {
        assert!(
            decode_envelope(200, r#"{"ok":false,"error":"keyframe x missing"}"#)
                .unwrap_err()
                .to_string()
                .contains("keyframe x missing")
        );
        let error = decode_envelope(500, r#"{"error":"boom"}"#)
            .unwrap_err()
            .to_string();
        assert!(error.contains("500") && error.contains("boom"), "{error}");
        assert!(decode_envelope(200, r#"{"ok":true,"result":{"a":1}}"#).is_ok());
    }
}
