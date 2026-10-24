//! Balance-on-feet co-agent loop.
//!
//! The embodiment controller owns the biped. This module homes nothing itself:
//! it asks the controller to begin an attempt, writes the upright goal into the
//! goal area through the caller, and scores torso tilt plus both feet on the
//! floor. Death and a misc reset end the attempt. The session continues until
//! the phase seeds are done.

use std::collections::BTreeMap;
use std::io::{Read, Write};
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
fn default_perturb_duration_bursts() -> u32 {
    2
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
fn default_settle_bursts() -> u32 {
    10
}
fn default_attempts_per_level() -> u32 {
    3
}
fn default_graduate_success_rate() -> f64 {
    0.66
}
fn default_support_max_stiffness() -> f64 {
    20.0
}
fn default_support_max_damping() -> f64 {
    0.2
}
fn default_burst_period_s() -> f64 {
    0.05
}
fn default_physics_steps_per_burst() -> u32 {
    25
}

/// Operator criterion for one balance-on-feet session.
///
/// Every field after `attempt_burst_cap` is optional on the wire and has a default, but Rust
/// struct literals must name them all: use [`BalanceFeetCriterion::new`] and override fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceFeetCriterion {
    /// Body whose +Z axis is compared with world up.
    pub torso_body: String,
    /// Body that owns the left foot collision geoms.
    pub left_foot_body: String,
    /// Body that owns the right foot collision geoms.
    pub right_foot_body: String,
    /// Maximum lean from upright, in degrees, that still counts as balanced.
    pub max_tilt_deg: f64,
    /// Consecutive balanced bursts required to accept the attempt.
    pub hold_bursts: u32,
    /// Bursts after which an attempt that has not succeeded is truncated.
    pub attempt_burst_cap: u32,
    /// Support level of the first train block (0..1). Default 1.0.
    #[serde(default = "default_support_start")]
    pub support_start: f64,
    /// Support lowered by this much after each graduating block. Default 0.1.
    #[serde(default = "default_support_step")]
    pub support_step: f64,
    /// Lowest support level. The test phase runs at this level. Default 0.0.
    #[serde(default = "default_support_min")]
    pub support_min: f64,
    /// Bursts the controller holds full support after a reset (samples flagged settling).
    #[serde(default = "default_settle_bursts")]
    pub settle_bursts: u32,
    /// Attempts per support block. Default 3.
    #[serde(default = "default_attempts_per_level")]
    pub attempts_per_level: u32,
    /// Block success rate needed to lower support. Default 0.66.
    #[serde(default = "default_graduate_success_rate")]
    pub graduate_success_rate: f64,
    /// Torso restoring stiffness at support 1.0, N*m/rad. Default 20.0.
    #[serde(default = "default_support_max_stiffness")]
    pub support_max_stiffness_nm_per_rad: f64,
    /// Torso restoring damping at support 1.0, N*m*s/rad. Default 0.2.
    #[serde(default = "default_support_max_damping")]
    pub support_max_damping_nm_s_per_rad: f64,
    /// FEAGI burst period in seconds; the caller must set it to the real burst period.
    #[serde(default = "default_burst_period_s")]
    pub burst_period_s: f64,
    /// Physics steps per burst. The controller publishes one sample per physics step and does
    /// not report its timestep, so bursts are decimated by `seq`: a sample is a new burst only
    /// when `seq` advanced by at least this many steps since the last counted burst.
    /// Default 25 = 0.05 s burst / 0.002 s physics step.
    #[serde(default = "default_physics_steps_per_burst")]
    pub physics_steps_per_burst: u32,
    /// Tilt deadband in degrees inside which the support torque is zero. Default 6.0.
    #[serde(default = "default_support_deadband_deg")]
    pub support_deadband_deg: f64,
    /// Bursts between perturbation pulses; 0 turns perturbation off. Default 0.
    #[serde(default)]
    pub perturb_interval_bursts: u32,
    /// Perturbation pulse torque, N*m. Default 0.3.
    #[serde(default = "default_perturb_torque_nm")]
    pub perturb_torque_nm: f64,
    /// Perturbation pulse length in bursts. Default 2.
    #[serde(default = "default_perturb_duration_bursts")]
    pub perturb_duration_bursts: u32,
    /// Graduation by time balanced: when `Some`, a support level passes when the MEDIAN
    /// `balanced_s` over that level's attempts is >= this value (seconds), instead of the
    /// success rate. `None` keeps the `graduate_success_rate` rule.
    #[serde(default)]
    pub graduate_balanced_s: Option<f64>,
    /// Harness vertical (height) stiffness, N/m. Sent in `begin_balance_attempt` only when set.
    #[serde(default)]
    pub support_height_stiffness_n_per_m: Option<f64>,
    /// Harness vertical (height) damping, N*s/m. Sent only when set.
    #[serde(default)]
    pub support_height_damping_n_s_per_m: Option<f64>,
    /// Harness gravity compensation fraction. Sent only when set.
    #[serde(default)]
    pub support_gravity_comp: Option<f64>,
    /// Harness horizontal (xy) stiffness, N/m. Sent only when set.
    #[serde(default)]
    pub support_xy_stiffness_n_per_m: Option<f64>,
    /// Harness horizontal (xy) damping, N*s/m. Sent only when set.
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
    /// Criterion with the support, settle and decimation fields at their defaults.
    pub fn new(
        torso_body: impl Into<String>,
        left_foot_body: impl Into<String>,
        right_foot_body: impl Into<String>,
        max_tilt_deg: f64,
        hold_bursts: u32,
        attempt_burst_cap: u32,
    ) -> Self {
        Self {
            torso_body: torso_body.into(),
            left_foot_body: left_foot_body.into(),
            right_foot_body: right_foot_body.into(),
            max_tilt_deg,
            hold_bursts,
            attempt_burst_cap,
            support_start: default_support_start(),
            support_step: default_support_step(),
            support_min: default_support_min(),
            settle_bursts: default_settle_bursts(),
            attempts_per_level: default_attempts_per_level(),
            graduate_success_rate: default_graduate_success_rate(),
            support_max_stiffness_nm_per_rad: default_support_max_stiffness(),
            support_max_damping_nm_s_per_rad: default_support_max_damping(),
            burst_period_s: default_burst_period_s(),
            physics_steps_per_burst: default_physics_steps_per_burst(),
            support_deadband_deg: default_support_deadband_deg(),
            perturb_interval_bursts: 0,
            perturb_torque_nm: default_perturb_torque_nm(),
            perturb_duration_bursts: default_perturb_duration_bursts(),
            graduate_balanced_s: None,
            support_height_stiffness_n_per_m: None,
            support_height_damping_n_s_per_m: None,
            support_gravity_comp: None,
            support_xy_stiffness_n_per_m: None,
            support_xy_damping_n_s_per_m: None,
        }
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
        let names = [
            self.torso_body.trim(),
            self.left_foot_body.trim(),
            self.right_foot_body.trim(),
        ];
        if names.iter().any(|name| name.is_empty()) {
            return Err(TrainerError::Config(
                "torso and both feet must be non-empty".to_string(),
            ));
        }
        if names[0] == names[1] || names[0] == names[2] || names[1] == names[2] {
            return Err(TrainerError::Config(
                "torso, left foot, and right foot must be different bodies".to_string(),
            ));
        }
        if !self.max_tilt_deg.is_finite() || self.max_tilt_deg <= 0.0 || self.max_tilt_deg > 90.0 {
            return Err(TrainerError::Config(
                "max_tilt_deg must be a finite value in (0, 90]".to_string(),
            ));
        }
        if self.hold_bursts == 0 {
            return Err(TrainerError::Config("hold_bursts must be > 0".to_string()));
        }
        if self.attempt_burst_cap == 0 {
            return Err(TrainerError::Config(
                "attempt_burst_cap must be > 0".to_string(),
            ));
        }
        if self.hold_bursts > self.attempt_burst_cap {
            return Err(TrainerError::Config(
                "hold_bursts must be <= attempt_burst_cap".to_string(),
            ));
        }
        self.support_config().validate()?;
        if !self.support_max_stiffness_nm_per_rad.is_finite()
            || self.support_max_stiffness_nm_per_rad <= 0.0
        {
            return Err(TrainerError::Config(
                "support_max_stiffness_nm_per_rad must be finite and > 0".to_string(),
            ));
        }
        if !self.support_max_damping_nm_s_per_rad.is_finite()
            || self.support_max_damping_nm_s_per_rad < 0.0
        {
            return Err(TrainerError::Config(
                "support_max_damping_nm_s_per_rad must be finite and >= 0".to_string(),
            ));
        }
        if !self.burst_period_s.is_finite() || self.burst_period_s <= 0.0 {
            return Err(TrainerError::Config(
                "burst_period_s must be finite and > 0".to_string(),
            ));
        }
        if self.physics_steps_per_burst == 0 {
            return Err(TrainerError::Config(
                "physics_steps_per_burst must be > 0".to_string(),
            ));
        }
        if !self.support_deadband_deg.is_finite() || self.support_deadband_deg < 0.0 {
            return Err(TrainerError::Config(
                "support_deadband_deg must be finite and >= 0".to_string(),
            ));
        }
        if !self.perturb_torque_nm.is_finite() || self.perturb_torque_nm < 0.0 {
            return Err(TrainerError::Config(
                "perturb_torque_nm must be finite and >= 0".to_string(),
            ));
        }
        if self.perturb_duration_bursts == 0 {
            return Err(TrainerError::Config(
                "perturb_duration_bursts must be >= 1".to_string(),
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
    /// Tilt stayed inside the limit and both feet stayed down for the required bursts.
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
    /// Fraction of non-settling bursts with tilt inside the limit and both feet down.
    #[serde(default)]
    pub upright_fraction: f64,
    /// Distinct perturbation pulses seen (rising edges of `perturbing`).
    #[serde(default)]
    pub perturbations: u32,
    /// Balance metric: the longest continuous run of upright bursts (tilt inside the limit and
    /// both feet down), counted in elapsed bursts, not samples. A `Death` burst never counts.
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

/// Parameters sent to the controller with `begin_balance_attempt`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BalanceAttemptParams {
    /// Support level for this attempt, 0..1.
    pub support_level: f64,
    /// Stiffness at support 1.0, N*m/rad.
    pub max_stiffness: f64,
    /// Damping at support 1.0, N*m*s/rad.
    pub max_damping: f64,
    /// Settling bursts at full support before the attempt.
    pub settle_bursts: u32,
    /// Burst period in seconds.
    pub burst_period_s: f64,
    /// Tilt deadband of the support torque, degrees.
    pub support_deadband_deg: f64,
    /// Bursts between perturbation pulses; 0 = off.
    pub perturb_interval_bursts: u32,
    /// Perturbation pulse torque, N*m.
    pub perturb_torque_nm: f64,
    /// Perturbation pulse length in bursts.
    pub perturb_duration_bursts: u32,
    /// Harness height stiffness, N/m; sent only when set.
    pub height_stiffness_n_per_m: Option<f64>,
    /// Harness height damping, N*s/m; sent only when set.
    pub height_damping_n_s_per_m: Option<f64>,
    /// Harness gravity compensation; sent only when set.
    pub gravity_comp: Option<f64>,
    /// Harness xy stiffness, N/m; sent only when set.
    pub xy_stiffness_n_per_m: Option<f64>,
    /// Harness xy damping, N*s/m; sent only when set.
    pub xy_damping_n_s_per_m: Option<f64>,
}

impl BalanceAttemptParams {
    /// Parameters for an attempt at `support_level` using the criterion's limits.
    pub fn from_criterion(criterion: &BalanceFeetCriterion, support_level: f64) -> Self {
        Self {
            support_level,
            max_stiffness: criterion.support_max_stiffness_nm_per_rad,
            max_damping: criterion.support_max_damping_nm_s_per_rad,
            settle_bursts: criterion.settle_bursts,
            burst_period_s: criterion.burst_period_s,
            support_deadband_deg: criterion.support_deadband_deg,
            perturb_interval_bursts: criterion.perturb_interval_bursts,
            perturb_torque_nm: criterion.perturb_torque_nm,
            perturb_duration_bursts: criterion.perturb_duration_bursts,
            height_stiffness_n_per_m: criterion.support_height_stiffness_n_per_m,
            height_damping_n_s_per_m: criterion.support_height_damping_n_s_per_m,
            gravity_comp: criterion.support_gravity_comp,
            xy_stiffness_n_per_m: criterion.support_xy_stiffness_n_per_m,
            xy_damping_n_s_per_m: criterion.support_xy_damping_n_s_per_m,
        }
    }
}

/// Home acknowledgement returned by the controller.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceBegin {
    /// Seed that was homed.
    pub seed: u64,
    /// Torso body measured for tilt.
    pub torso_body: String,
    /// Left foot body.
    pub left_foot_body: String,
    /// Right foot body.
    pub right_foot_body: String,
    /// Tilt limit echoed by the controller, degrees.
    pub max_tilt_deg: f64,
    /// Physics steps per burst as computed by the controller from its model timestep and the
    /// requested `burst_period_s`. Absent on old controllers; the criterion value is used then.
    #[serde(default)]
    pub physics_steps_per_burst: Option<u32>,
}

/// One stance measurement from the controller (one per physics step).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceSample {
    /// Monotonic sample counter for this attempt; advances once per physics step.
    pub seq: u64,
    /// Attempt seed.
    pub seed: u64,
    /// Torso tilt from world up, degrees.
    pub tilt_deg: f64,
    /// Left foot geom is touching a world geom.
    pub left_foot_contact: bool,
    /// Right foot geom is touching a world geom.
    pub right_foot_contact: bool,
    /// Latched end cause. Absent while the attempt is still open.
    pub terminal: Option<BalanceTerminal>,
    /// Effective support level at this step. Absent on old controllers.
    #[serde(default)]
    pub support_level: Option<f64>,
    /// The controller is still holding the settle period. Absent on old controllers.
    #[serde(default)]
    pub settling: bool,
    /// A perturbation pulse is being applied at this step. Absent on old controllers.
    #[serde(default)]
    pub perturbing: bool,
    /// Predicted tilt fraction 0..1 at this step. Absent on old controllers.
    #[serde(default)]
    pub support_need: Option<f64>,
    /// Harness effort 0..1 at this step. Absent on old controllers.
    #[serde(default)]
    pub support_effort: Option<f64>,
    /// Harness torque, N*m. Absent on old controllers.
    #[serde(default)]
    pub support_torque_nm: Option<f64>,
    /// Harness force, N. Absent on old controllers.
    #[serde(default)]
    pub support_force_n: Option<f64>,
}

/// Controller-latched end cause. Hold success is decided here, not by the controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BalanceTerminal {
    /// `_death` fired.
    Death,
    /// Misc reset fired while the attempt was active.
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
    /// Latest controller sample sequence seen (counted or not); pass it to `poll`.
    pub last_seq: u64,
    /// Sequence of the last sample that was counted as a burst, if any.
    pub last_burst_seq: Option<u64>,
    /// Settling bursts observed (logged, never scored).
    pub settling_bursts: u32,
    /// Non-settling bursts that were upright (tilt inside the limit, both feet down).
    pub upright_bursts: u32,
    /// The latest sample was counted as a new burst (false when decimated away).
    pub counted_burst: bool,
    /// Physics steps per burst reported by the controller; overrides the criterion when set.
    pub steps_per_burst: Option<u32>,
    /// Support level of the attempt's params, used when a sample carries none.
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
    /// An attempt for `support_level` using the controller's steps-per-burst when it reported one.
    pub fn for_attempt(steps_per_burst: Option<u32>, support_level: f64) -> Self {
        Self {
            steps_per_burst: steps_per_burst.filter(|n| *n > 0),
            params_support_level: support_level,
            ..Self::fresh()
        }
    }

    /// An attempt that has not observed a sample yet.
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
            steps_per_burst: None,
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

/// Tilt inside the limit with both feet down.
pub fn sample_is_upright(sample: &BalanceSample, criterion: &BalanceFeetCriterion) -> bool {
    sample.tilt_deg <= criterion.max_tilt_deg
        && sample.left_foot_contact
        && sample.right_foot_contact
}

/// Fold one controller sample into the attempt.
///
/// Samples arrive once per physics step. With N steps per burst (the controller's value when it
/// reported one, else `criterion.physics_steps_per_burst`), `k = (seq - last_burst_seq) / N` new
/// bursts have elapsed since the last counted burst; `last_burst_seq` advances by `k * N` and
/// `bursts` by `k`, so a late poll does not undercount. A sample with `k >= 1` is one counted
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
    let steps = u64::from(
        open.steps_per_burst
            .unwrap_or(criterion.physics_steps_per_burst)
            .max(1),
    );
    let (new_bursts, anchor): (u32, u64) = match open.last_burst_seq {
        None => (1, sample.seq),
        Some(last) => {
            let k = sample.seq.saturating_sub(last) / steps;
            (
                u32::try_from(k).unwrap_or(u32::MAX),
                last.saturating_add(k.saturating_mul(steps)),
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
    if holding_bursts >= criterion.hold_bursts {
        return Ok(finish(BalanceAttemptOutcome::Success));
    }
    if bursts >= criterion.attempt_burst_cap {
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
/// Pleasure each burst that need is inside the deadband with both feet down.
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
/// `n_t <= deadband` with both feet down -> Pleasure `ASSIST_HOLD_PLEASURE`; Success ->
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
        if need <= ASSIST_NEED_DEADBAND
            && burst_sample.left_foot_contact
            && burst_sample.right_foot_contact
        {
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

/// Controller side of one balance session.
pub trait BalanceProbe {
    /// Home the biped for `seed` and start the attempt with the given support parameters.
    fn begin_attempt(
        &mut self,
        seed: u64,
        criterion: &BalanceFeetCriterion,
        params: &BalanceAttemptParams,
    ) -> Result<BalanceBegin, TrainerError>;

    /// Wait for a sample newer than `after_seq`.
    fn poll(&mut self, after_seq: u64, timeout: Duration) -> Result<BalanceSample, TrainerError>;
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
    /// How long one poll may wait.
    pub poll_timeout: Duration,
    /// Run the progress events are stamped with.
    pub run_id: &'a RunId,
    /// Support policy. `None`: `train` follows the schedule, `test` runs at `support_min`.
    pub support_mode: Option<SupportMode>,
    /// Affect delivery. `None` keeps the no-affect behavior.
    pub deliver_reward: Option<DeliverReward<'a>>,
    /// Which policy builds the affect signals when `deliver_reward` is set.
    pub reward_policy: BalanceRewardPolicy,
}

/// Run one phase (`train` or `test`) and emit its progress.
pub fn run_balance_phase(
    run: &mut BalancePhaseRun<'_>,
    probe: &mut dyn BalanceProbe,
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
    if run.poll_timeout.is_zero() {
        return Err(TrainerError::Config("poll_timeout must be > 0".to_string()));
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
            probe,
            present_goal,
            &mut run.deliver_reward,
            run.reward_policy,
            run.poll_timeout,
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
    let report = BalancePhaseReport::from_attempts_with(attempts, &support)?;
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

#[allow(clippy::too_many_arguments)]
fn run_balance_attempt(
    seed: u64,
    criterion: &BalanceFeetCriterion,
    support_level: f64,
    probe: &mut dyn BalanceProbe,
    present_goal: &mut dyn FnMut([f64; 3]) -> Result<(), TrainerError>,
    deliver_reward: &mut Option<DeliverReward<'_>>,
    reward_policy: BalanceRewardPolicy,
    poll_timeout: Duration,
    cancel: &CancelToken,
) -> Result<BalanceAttemptRecord, TrainerError> {
    let params = BalanceAttemptParams::from_criterion(criterion, support_level);
    let begun = probe.begin_attempt(seed, criterion, &params)?;
    if begun.seed != seed {
        return Err(TrainerError::Runtime(format!(
            "controller homed seed {} for requested seed {seed}",
            begun.seed
        )));
    }
    let features = upright_goal_features();
    let mut open = OpenBalanceAttempt::for_attempt(begun.physics_steps_per_burst, support_level);
    let mut affect = AttemptReward::new(reward_policy, support_level);
    loop {
        if cancel.is_cancelled() {
            return Err(TrainerError::Cancelled(
                "balance-on-feet run cancelled".to_string(),
            ));
        }
        present_goal(features)?;
        let sample = probe.poll(open.last_seq, poll_timeout)?;
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
            BalanceAttemptStep::Finished(record) => return Ok(record),
        }
    }
}

/// HTTP client for the neutral balance actions on a controller control server.
pub struct HttpBalanceProbe {
    host: String,
    port: u16,
    timeout: Duration,
}

/// JSON body of `begin_balance_attempt`.
pub fn begin_attempt_request(
    seed: u64,
    criterion: &BalanceFeetCriterion,
    params: &BalanceAttemptParams,
) -> serde_json::Value {
    let mut body = serde_json::json!({
        "action": "begin_balance_attempt",
        "seed": seed,
        "torso_body": criterion.torso_body,
        "left_foot_body": criterion.left_foot_body,
        "right_foot_body": criterion.right_foot_body,
        "max_tilt_deg": criterion.max_tilt_deg,
        "support_level": params.support_level,
        "support_max_stiffness_nm_per_rad": params.max_stiffness,
        "support_max_damping_nm_s_per_rad": params.max_damping,
        "settle_bursts": params.settle_bursts,
        "burst_period_s": params.burst_period_s,
        "support_deadband_deg": params.support_deadband_deg,
        "perturb_interval_bursts": params.perturb_interval_bursts,
        "perturb_torque_nm": params.perturb_torque_nm,
        "perturb_duration_bursts": params.perturb_duration_bursts,
    });
    let optional = [
        (
            "support_height_stiffness_n_per_m",
            params.height_stiffness_n_per_m,
        ),
        (
            "support_height_damping_n_s_per_m",
            params.height_damping_n_s_per_m,
        ),
        ("support_gravity_comp", params.gravity_comp),
        ("support_xy_stiffness_n_per_m", params.xy_stiffness_n_per_m),
        ("support_xy_damping_n_s_per_m", params.xy_damping_n_s_per_m),
    ];
    for (key, value) in optional {
        if let Some(value) = value {
            body[key] = serde_json::json!(value);
        }
    }
    body
}

impl HttpBalanceProbe {
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
        })
    }

    fn command(&self, body: &serde_json::Value) -> Result<serde_json::Value, TrainerError> {
        let payload = serde_json::to_vec(body).map_err(|error| {
            TrainerError::Runtime(format!("balance command encode failed: {error}"))
        })?;
        let request = format!(
            "POST /command HTTP/1.1\r\nHost: {}:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            self.host,
            self.port,
            payload.len()
        );
        let address = format!("{}:{}", self.host, self.port);
        let socket = address
            .to_socket_addrs()
            .map_err(|error| TrainerError::Runtime(format!("balance control address: {error}")))?
            .next()
            .ok_or_else(|| {
                TrainerError::Runtime(format!("balance control address {address} did not resolve"))
            })?;
        let mut stream = TcpStream::connect_timeout(&socket, self.timeout).map_err(|error| {
            TrainerError::Runtime(format!("balance control connect failed: {error}"))
        })?;
        stream
            .set_read_timeout(Some(self.timeout))
            .map_err(|error| TrainerError::Runtime(format!("balance control timeout: {error}")))?;
        stream
            .set_write_timeout(Some(self.timeout))
            .map_err(|error| TrainerError::Runtime(format!("balance control timeout: {error}")))?;
        stream.write_all(request.as_bytes()).map_err(|error| {
            TrainerError::Runtime(format!("balance control write failed: {error}"))
        })?;
        stream.write_all(&payload).map_err(|error| {
            TrainerError::Runtime(format!("balance control write failed: {error}"))
        })?;
        let mut raw = Vec::new();
        stream.read_to_end(&mut raw).map_err(|error| {
            TrainerError::Runtime(format!("balance control read failed: {error}"))
        })?;
        parse_http_json(&raw)
    }
}

impl BalanceProbe for HttpBalanceProbe {
    fn begin_attempt(
        &mut self,
        seed: u64,
        criterion: &BalanceFeetCriterion,
        params: &BalanceAttemptParams,
    ) -> Result<BalanceBegin, TrainerError> {
        let result = self.command(&begin_attempt_request(seed, criterion, params))?;
        serde_json::from_value(result)
            .map_err(|error| TrainerError::Runtime(format!("balance begin decode failed: {error}")))
    }

    fn poll(&mut self, after_seq: u64, timeout: Duration) -> Result<BalanceSample, TrainerError> {
        let timeout_s = timeout.as_secs_f64();
        if timeout_s <= 0.0 {
            return Err(TrainerError::Config(
                "balance poll timeout must be > 0".to_string(),
            ));
        }
        let result = self.command(&serde_json::json!({
            "action": "poll_balance_outcome",
            "after_seq": after_seq,
            "timeout_s": timeout_s,
        }))?;
        serde_json::from_value(result)
            .map_err(|error| TrainerError::Runtime(format!("balance poll decode failed: {error}")))
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

fn parse_http_json(raw: &[u8]) -> Result<serde_json::Value, TrainerError> {
    let text = String::from_utf8_lossy(raw);
    let (head, body) = text.split_once("\r\n\r\n").ok_or_else(|| {
        TrainerError::Runtime("balance control response has no HTTP body".to_string())
    })?;
    let status_line = head.lines().next().unwrap_or("");
    if !status_line.contains(" 200 ") {
        return Err(TrainerError::Runtime(format!(
            "balance control HTTP failed: {status_line}"
        )));
    }
    let envelope: serde_json::Value = serde_json::from_str(body.trim())
        .map_err(|error| TrainerError::Runtime(format!("balance control JSON failed: {error}")))?;
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
    use std::collections::VecDeque;

    struct ScriptedProbe {
        begins: VecDeque<BalanceBegin>,
        samples: VecDeque<BalanceSample>,
        seen_params: Vec<BalanceAttemptParams>,
    }

    impl ScriptedProbe {
        fn new(begins: Vec<BalanceBegin>, samples: Vec<BalanceSample>) -> Self {
            Self {
                begins: begins.into(),
                samples: samples.into(),
                seen_params: Vec::new(),
            }
        }
    }

    impl BalanceProbe for ScriptedProbe {
        fn begin_attempt(
            &mut self,
            seed: u64,
            _criterion: &BalanceFeetCriterion,
            params: &BalanceAttemptParams,
        ) -> Result<BalanceBegin, TrainerError> {
            self.seen_params.push(*params);
            let mut begun = self
                .begins
                .pop_front()
                .ok_or_else(|| TrainerError::Runtime("no scripted begin".to_string()))?;
            begun.seed = seed;
            Ok(begun)
        }

        fn poll(
            &mut self,
            _after_seq: u64,
            _timeout: Duration,
        ) -> Result<BalanceSample, TrainerError> {
            self.samples
                .pop_front()
                .ok_or_else(|| TrainerError::Runtime("no scripted sample".to_string()))
        }
    }

    /// One physics step per burst so legacy tests stay readable.
    fn criterion() -> BalanceFeetCriterion {
        let mut c = BalanceFeetCriterion::new("torso", "left_foot", "right_foot", 15.0, 2, 4);
        c.physics_steps_per_burst = 1;
        c
    }

    fn begin() -> BalanceBegin {
        BalanceBegin {
            seed: 0,
            torso_body: "torso".to_string(),
            left_foot_body: "left_foot".to_string(),
            right_foot_body: "right_foot".to_string(),
            max_tilt_deg: 15.0,
            physics_steps_per_burst: None,
        }
    }

    fn sample(seq: u64, seed: u64, tilt: f64) -> BalanceSample {
        BalanceSample {
            seq,
            seed,
            tilt_deg: tilt,
            left_foot_contact: true,
            right_foot_contact: true,
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
        open.last_burst_seq = Some(1);
        let mut s = balanced(2, 3);
        s.right_foot_contact = false;
        match observe_balance_sample(&open, &s, &criterion(), 3).expect("step") {
            BalanceAttemptStep::Continue(next) => assert_eq!(next.holding_bursts, 0),
            BalanceAttemptStep::Finished(_) => panic!("one missed foot is not the cap"),
        }
    }

    #[test]
    fn phase_emits_progress_and_success_rate() {
        let mut probe = ScriptedProbe::new(vec![begin()], vec![balanced(1, 1), balanced(2, 1)]);
        let mut events = CollectingEventSink::default();
        let run_id = RunId("balance-test".to_string());
        let mut presented = 0;
        let rule = criterion();
        let mut run = BalancePhaseRun {
            phase: "train",
            seeds: &[1],
            criterion: &rule,
            poll_timeout: Duration::from_millis(1),
            run_id: &run_id,
            support_mode: None,
            deliver_reward: None,
            reward_policy: BalanceRewardPolicy::Affect,
        };
        let report = run_balance_phase(
            &mut run,
            &mut probe,
            &mut |_| {
                presented += 1;
                Ok(())
            },
            &mut events,
            &CancelToken::new(),
        )
        .expect("phase");
        assert_eq!(report.success_rate, 1.0);
        assert_eq!(presented, 2);
        assert!(report.median_bursts_to_success.is_some());
        assert_eq!(probe.seen_params[0].support_level, 1.0);
        let card = balance_phase_scorecard("sc-1", "sha256:abc", 1, "train", "b", "hash", &report);
        assert_eq!(card.metrics.get("success_rate"), Some(&1.0));
        assert_eq!(card.evaluation_protocol_version.0, "balance-feet-v2");
    }

    #[test]
    fn test_phase_runs_at_support_min_and_delivers_reward() {
        let mut probe = ScriptedProbe::new(vec![begin()], vec![balanced(1, 1), balanced(2, 1)]);
        let mut events = CollectingEventSink::default();
        let run_id = RunId("balance-test".to_string());
        let rule = criterion();
        let mut delivered: Vec<RewardSignal> = Vec::new();
        let mut sink = |signals: &[RewardSignal]| -> Result<(), TrainerError> {
            delivered.extend_from_slice(signals);
            Ok(())
        };
        let mut run = BalancePhaseRun {
            phase: "test",
            seeds: &[1],
            criterion: &rule,
            poll_timeout: Duration::from_millis(1),
            run_id: &run_id,
            support_mode: None,
            deliver_reward: Some(&mut sink),
            reward_policy: BalanceRewardPolicy::Affect,
        };
        run_balance_phase(
            &mut run,
            &mut probe,
            &mut |_| Ok(()),
            &mut events,
            &CancelToken::new(),
        )
        .expect("phase");
        assert_eq!(probe.seen_params[0].support_level, 0.0);
        // burst 1: support 0 and upright -> 0.1; burst 2: success -> 1.0
        assert_eq!(delivered.len(), 2);
        assert_eq!(delivered[0].magnitude, 0.1);
        assert_eq!(delivered[1].channel, AffectChannel::Pleasure);
        assert_eq!(delivered[1].magnitude, 1.0);
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
    fn criterion_validation_covers_support_fields() {
        let mut c = criterion();
        assert!(c.validate().is_ok());
        c.support_min = 0.5;
        c.support_start = 0.4;
        assert!(c.validate().is_err());
        let mut c = criterion();
        c.support_step = 0.0;
        assert!(c.validate().is_err());
        let mut c = criterion();
        c.graduate_success_rate = 0.0;
        assert!(c.validate().is_err());
        let mut c = criterion();
        c.attempts_per_level = 0;
        assert!(c.validate().is_err());
        let mut c = criterion();
        c.burst_period_s = 0.0;
        assert!(c.validate().is_err());
    }

    #[test]
    fn twenty_five_steps_are_one_burst() {
        let mut c = criterion();
        c.physics_steps_per_burst = 25;
        c.hold_bursts = 10;
        c.attempt_burst_cap = 20;
        let mut open = OpenBalanceAttempt::fresh();
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
        c.physics_steps_per_burst = 25;
        c.hold_bursts = 10;
        c.attempt_burst_cap = 20;
        let mut open = OpenBalanceAttempt::fresh();
        open = fed(&open, &balanced(1, 5), &c);
        open = fed(&open, &balanced(60, 5), &c);
        assert_eq!(open.bursts, 3, "1 first burst + 2 elapsed (59 / 25)");
        assert_eq!(open.last_burst_seq, Some(51));
        assert_eq!(
            open.holding_bursts, 3,
            "hold advances by elapsed upright bursts"
        );
        assert!(open.counted_burst);
    }

    #[test]
    fn controller_steps_per_burst_overrides_criterion() {
        let mut c = criterion();
        c.physics_steps_per_burst = 25;
        c.hold_bursts = 10;
        c.attempt_burst_cap = 20;
        let mut open = OpenBalanceAttempt::for_attempt(Some(10), 0.0);
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
        s.right_foot_contact = false;
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
        c.hold_bursts = 50;
        c.attempt_burst_cap = 100;
        c.physics_steps_per_burst = 1;
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
    fn begin_request_includes_support_fields() {
        let c = criterion();
        let mut params = BalanceAttemptParams::from_criterion(&c, 0.3);
        params.burst_period_s = 0.04;
        let body = begin_attempt_request(9, &c, &params);
        assert_eq!(body["action"], "begin_balance_attempt");
        assert_eq!(body["seed"], 9);
        assert_eq!(body["support_level"], 0.3);
        assert_eq!(body["support_max_stiffness_nm_per_rad"], 20.0);
        assert_eq!(body["support_max_damping_nm_s_per_rad"], 0.2);
        assert_eq!(body["settle_bursts"], 10);
        assert_eq!(body["burst_period_s"], 0.04);
        assert_eq!(body["support_deadband_deg"], 6.0);
        assert_eq!(body["perturb_interval_bursts"], 0);
        assert_eq!(body["perturb_torque_nm"], 0.3);
        assert_eq!(body["perturb_duration_bursts"], 2);
    }

    #[test]
    fn old_format_samples_still_parse() {
        let old = serde_json::json!({
            "seq": 3, "seed": 1, "tilt_deg": 4.5,
            "left_foot_contact": true, "right_foot_contact": false, "terminal": null
        });
        let parsed: BalanceSample = serde_json::from_value(old).expect("old sample");
        assert!(!parsed.settling);
        assert!(!parsed.perturbing);
        assert_eq!(parsed.support_level, None);
        let c = criterion();
        let mut reward = BalanceAffectReward::new();
        let signals = reward.on_sample(&parsed, None, &c);
        assert!(
            signals.iter().all(|x| x.magnitude != 0.1),
            "no Pleasure 0.1 without a reported support level"
        );
        let new = serde_json::json!({
            "seq": 3, "seed": 1, "tilt_deg": 4.5, "left_foot_contact": true,
            "right_foot_contact": true, "terminal": "death",
            "support_level": 0.7, "settling": true
        });
        let parsed: BalanceSample = serde_json::from_value(new).expect("new sample");
        assert!(parsed.settling);
        assert_eq!(parsed.terminal, Some(BalanceTerminal::Death));
        let c: BalanceFeetCriterion = serde_json::from_value(serde_json::json!({
            "torso_body": "t", "left_foot_body": "l", "right_foot_body": "r",
            "max_tilt_deg": 10.0, "hold_bursts": 1, "attempt_burst_cap": 2
        }))
        .expect("old criterion");
        assert_eq!(c.attempts_per_level, 3);
        assert_eq!(c.physics_steps_per_burst, 25);
        assert_eq!(c.support_deadband_deg, 6.0);
        assert_eq!(c.perturb_interval_bursts, 0);
        assert_eq!(c.perturb_duration_bursts, 2);
    }

    #[test]
    fn slow_polls_reach_success_at_hold_bursts_elapsed() {
        // Polls every 2 bursts (steps_per_burst 1, seq advancing by 2). The first sample counts
        // as one burst, so holds after each poll are 1, 3, 5: success exactly at 5 bursts, not
        // after 5 samples.
        let mut c = criterion();
        c.hold_bursts = 5;
        c.attempt_burst_cap = 50;
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
        c.hold_bursts = 100;
        c.attempt_burst_cap = 12;
        let mut open = OpenBalanceAttempt::fresh();
        for seq in 1..=3 {
            open = fed(&open, &balanced(seq, 1), &c);
        }
        open = fed(&open, &sample(4, 1, 30.0), &c);
        for seq in 5..=9 {
            open = fed(&open, &balanced(seq, 1), &c);
        }
        let mut lifted = balanced(10, 1);
        lifted.left_foot_contact = false;
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
        up.left_foot_contact = false;
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

    #[test]
    fn new_sample_fields_parse_and_harness_fields_are_sent_only_when_set() {
        let old = serde_json::json!({
            "seq": 1, "seed": 1, "tilt_deg": 1.0,
            "left_foot_contact": true, "right_foot_contact": true, "terminal": null
        });
        let parsed: BalanceSample = serde_json::from_value(old).expect("old sample");
        assert_eq!(parsed.support_need, None);
        assert_eq!(parsed.support_force_n, None);
        let new = serde_json::json!({
            "seq": 1, "seed": 1, "tilt_deg": 1.0, "left_foot_contact": true,
            "right_foot_contact": true, "terminal": null, "support_need": 0.0,
            "support_effort": 0.5, "support_torque_nm": 1.5, "support_force_n": 2.5
        });
        let parsed: BalanceSample = serde_json::from_value(new).expect("new sample");
        assert_eq!(parsed.support_need, Some(0.0));
        assert_eq!(parsed.support_effort, Some(0.5));
        assert_eq!(parsed.support_torque_nm, Some(1.5));
        assert_eq!(parsed.support_force_n, Some(2.5));
        let keys = [
            "support_height_stiffness_n_per_m",
            "support_height_damping_n_s_per_m",
            "support_gravity_comp",
            "support_xy_stiffness_n_per_m",
            "support_xy_damping_n_s_per_m",
        ];
        let mut c = criterion();
        let body = begin_attempt_request(1, &c, &BalanceAttemptParams::from_criterion(&c, 0.5));
        for k in keys {
            assert!(body.get(k).is_none(), "{k} must be absent when unset");
        }
        c.support_height_stiffness_n_per_m = Some(100.0);
        c.support_height_damping_n_s_per_m = Some(10.0);
        c.support_gravity_comp = Some(0.8);
        c.support_xy_stiffness_n_per_m = Some(50.0);
        c.support_xy_damping_n_s_per_m = Some(5.0);
        let body = begin_attempt_request(1, &c, &BalanceAttemptParams::from_criterion(&c, 0.5));
        assert_eq!(body["support_height_stiffness_n_per_m"], 100.0);
        assert_eq!(body["support_height_damping_n_s_per_m"], 10.0);
        assert_eq!(body["support_gravity_comp"], 0.8);
        assert_eq!(body["support_xy_stiffness_n_per_m"], 50.0);
        assert_eq!(body["support_xy_damping_n_s_per_m"], 5.0);
    }
}
