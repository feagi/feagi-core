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
    AttemptStand, BackendFingerprint, BackendKind, ConnectomeHash, ContentHash, DatasetAssetId,
    EvaluationProtocolVersion, MetadataValue, MetricScope, PluginId, PluginRef, RunEvent,
    RunEventKind, RunId, Scorecard, ScorecardId, ScorecardStatus, ScorecardVisibility, SplitId,
};
use crate::control::{CancelToken, RunEventSink};
use crate::error::TrainerError;
use crate::metrics::episodic_control::EpisodicControlMetricPack;
use crate::plasticity_freeze::PlasticityControl;
use crate::reach_hold::trainer_package_version;
use crate::run_actions::{ApplyEvent, ScheduledEvent};
use crate::run_data::{name_of, ColumnSpec, EventTime, RunDataSink, RunDataValue, TableSpec};

fn default_support_deadband_deg() -> f64 {
    0.0
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
fn default_fall_tilt_deg() -> f64 {
    60.0
}
/// Largest accepted `bws_max_fraction`: the legs always carry at least half the weight.
const MAX_BWS_FRACTION: f64 = 0.5;

fn default_bws_max_fraction() -> f64 {
    MAX_BWS_FRACTION
}
/// Default for `only_contacts_touch`: standing means only the contact bodies touch the world.
fn default_only_contacts_touch() -> bool {
    true
}
/// Default `min_height_fraction`: the torso must stay within 80% of its height at the reset pose.
fn default_min_height_fraction() -> f64 {
    0.8
}
/// Default `lift_tolerance`: a torso 5% above its reset height counts as lifted.
fn default_lift_tolerance() -> f64 {
    0.05
}
/// Default `over_assist_margin`: effort 0.25 above the predicted need counts as over-assist.
fn default_over_assist_margin() -> f64 {
    0.25
}
/// Default `over_support_flag_fraction`: a level is flagged when 20% of its samples are carried
/// or over-assisted.
fn default_over_support_flag_fraction() -> f64 {
    0.2
}
/// Default `fade_tau_s`: the adaptive assist fades with a 20 s time constant.
fn default_fade_tau_s() -> f64 {
    20.0
}
/// Default `catch_tau_s`: the adaptive assist rises on overshoot with a 0.2 s time constant.
fn default_catch_tau_s() -> f64 {
    0.2
}
/// Default `hold_after_catch_s`: fading pauses for 1 s after a catch.
fn default_hold_after_catch_s() -> f64 {
    1.0
}
/// Default `cone_min_deg`: the narrowest tolerance cone, at full assist.
fn default_cone_min_deg() -> f64 {
    2.0
}
/// Default `lean_grace_s`: a lean beyond the cone is re-centred after 2 s.
fn default_lean_grace_s() -> f64 {
    2.0
}
/// Default `stance_grace_s`: an attempt ends after 1 s without a standing stance.
fn default_stance_grace_s() -> f64 {
    1.0
}
/// Default `standing_fraction_for_success`: an adaptive attempt succeeds when it stood for 80%
/// of its counted bursts.
fn default_standing_fraction_for_success() -> f64 {
    0.8
}
/// Default `catch_threshold`: an assist increase of 0.05 in one burst counts as a catch.
fn default_catch_threshold() -> f64 {
    0.05
}
fn default_fall_confirm_s() -> f64 {
    0.25
}
/// Largest accepted `probe_every`.
const MAX_PROBE_EVERY: u32 = 100_000;

fn default_probe_every() -> u32 {
    5
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

/// How the train phase lowers the support.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssistMode {
    /// The stepwise [`SupportSchedule`]: one level per block of attempts.
    #[default]
    Ladder,
    /// Continuous assist that fades slowly while the robot copes and rises fast on overshoot
    /// ([`AdaptiveAssist`]). Attempts end on a fall, on a lost stance, or at `attempt_cap_s` (a success when
    /// the robot stood for `standing_fraction_for_success` of it).
    Adaptive,
}

impl AssistMode {
    /// Wire name of the mode.
    pub fn name(self) -> &'static str {
        match self {
            Self::Ladder => "ladder",
            Self::Adaptive => "adaptive",
        }
    }
}

/// Operator criterion for one balance-on-feet session.
///
/// The controller knows no robot and no task: this struct owns every decision. All durations
/// are in seconds of simulated time and are converted to bursts with the live burst period
/// (`burst_period_s`, set by the caller from the running FEAGI burst rate). Spring gains are
/// sized by the Trainer from the model (see [`size_spring_gains`]); each `support_*` gain
/// override that is `Some` wins over the sized value. The harness is an anti-tilt torque on
/// the torso body plus a constant upward force of `level * bws_max_fraction * m_total * g`
/// (partial body-weight support); it has no position springs, so the legs bear the rest.
///
/// Every field after `attempt_cap_s` is optional on the wire and has a default, but Rust
/// struct literals must name them all: use [`BalanceFeetCriterion::new`] and override fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceFeetCriterion {
    /// Body whose +Z axis is compared with world up and which carries the support harness.
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
    /// Tilt in degrees at or above which the torso counts as fallen. 0 turns fall detection
    /// off. Default 60.0.
    #[serde(default = "default_fall_tilt_deg")]
    pub fall_tilt_deg: f64,
    /// Seconds the tilt must stay at or above `fall_tilt_deg` before the attempt ends as
    /// `Fell`. Default 0.25.
    #[serde(default = "default_fall_confirm_s")]
    pub fall_confirm_s: f64,
    /// During the train phase, one unassisted probe attempt runs after every `probe_every`
    /// completed training attempts (support 0, no perturbation, no reward, learning frozen).
    /// 0 turns probes off. Default 5.
    #[serde(default = "default_probe_every")]
    pub probe_every: u32,
    /// Support level of the first train block (0..1). Default 1.0.
    #[serde(default = "default_support_start")]
    pub support_start: f64,
    /// Support lowered by this much after each graduating block. Default 0.1.
    #[serde(default = "default_support_step")]
    pub support_step: f64,
    /// Lowest support level the train schedule can reach. The test phase ignores it and always
    /// runs at support 0. Default 0.0.
    #[serde(default = "default_support_min")]
    pub support_min: f64,
    /// Seconds after a reset during which samples are flagged settling and do not count. Train
    /// holds a full grip for them; the test phase holds no support (spring at level 0).
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
    /// Tilt deadband in degrees inside which the spring torque is zero. Default 0.0.
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
    /// Fraction of the whole body's weight the support carries at support level 1 (partial
    /// body-weight support). The upward force on the torso body is
    /// `level * bws_max_fraction * m_total * g`. In [0, 0.5]; default 0.5.
    #[serde(default = "default_bws_max_fraction")]
    pub bws_max_fraction: f64,
    /// Standing means only the contact bodies touch the world. When true, a sample counts as
    /// upright only if no other body (descendants and the world excluded) touches the world.
    /// Default true. Asks the controller for one extra contact set (`a_except`).
    #[serde(default = "default_only_contacts_touch")]
    pub only_contacts_touch: bool,
    /// Standing means staying tall. A sample counts as upright only if the torso body's height
    /// (CoM z above world z) is at least this fraction of its height on the first sample after
    /// the reset. 0 turns the check off; otherwise in (0, 1]. Default 0.8.
    #[serde(default = "default_min_height_fraction")]
    pub min_height_fraction: f64,
    /// A sample whose torso is higher than `reset height * (1 + lift_tolerance)` counts as
    /// carried by the support. At or above 0; default 0.05.
    #[serde(default = "default_lift_tolerance")]
    pub lift_tolerance: f64,
    /// A sample whose support effort exceeds the predicted need by more than this margin counts
    /// as over-assisted. In [0, 1]; default 0.25.
    #[serde(default = "default_over_assist_margin")]
    pub over_assist_margin: f64,
    /// A support level is flagged `over_support` when its mean carried fraction or mean
    /// over-assist fraction exceeds this share. In [0, 1]; default 0.2.
    #[serde(default = "default_over_support_flag_fraction")]
    pub over_support_flag_fraction: f64,
    /// `ladder` (default) lowers support in blocks; `adaptive` runs the continuous assist below.
    #[serde(default)]
    pub support_mode: AssistMode,
    /// Adaptive: time constant, seconds, of the slow fade (and of the slow average that widens
    /// the tolerance cone). > 0; default 20.
    #[serde(default = "default_fade_tau_s")]
    pub fade_tau_s: f64,
    /// Adaptive: time constant, seconds, of the fast rise on overshoot. > 0 and < `fade_tau_s`;
    /// default 0.2.
    #[serde(default = "default_catch_tau_s")]
    pub catch_tau_s: f64,
    /// Adaptive: seconds the fade pauses after a catch. >= 0; default 1.
    #[serde(default = "default_hold_after_catch_s")]
    pub hold_after_catch_s: f64,
    /// Adaptive: narrowest tolerance cone, degrees, at full assist. In [0, `max_tilt_deg`);
    /// default 2.
    #[serde(default = "default_cone_min_deg")]
    pub cone_min_deg: f64,
    /// Adaptive: assist increase within one burst that counts as a catch. In [0, 1]; default 0.05.
    #[serde(default = "default_catch_threshold")]
    pub catch_threshold: f64,
    /// Adaptive: seconds the predicted tilt may stay beyond the current cone before the cone
    /// drops to `cone_min_deg` (the support re-centres the robot). > 0; default 2.
    #[serde(default = "default_lean_grace_s")]
    pub lean_grace_s: f64,
    /// Adaptive: seconds without a standing stance (see [`sample_is_upright`]) that end the
    /// attempt as `lost_stance`. > 0; default 1.
    #[serde(default = "default_stance_grace_s")]
    pub stance_grace_s: f64,
    /// Adaptive: share of the attempt's counted bursts that must be standing for a capped
    /// attempt to count as a success. In (0, 1]; default 0.8.
    #[serde(default = "default_standing_fraction_for_success")]
    pub standing_fraction_for_success: f64,
    /// Events applied to the brain at fixed seconds after settle ends, within each train
    /// attempt (skipped in probes). A test phase refuses them. Empty by default.
    #[serde(default, alias = "scheduledEvents")]
    pub scheduled_events: Vec<ScheduledEvent>,
    /// Ask the controller for every actuator command each burst and put it in the sample.
    #[serde(default, alias = "recordActuators")]
    pub record_actuators: bool,
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
            fall_tilt_deg: default_fall_tilt_deg(),
            fall_confirm_s: default_fall_confirm_s(),
            probe_every: default_probe_every(),
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
            bws_max_fraction: default_bws_max_fraction(),
            only_contacts_touch: default_only_contacts_touch(),
            min_height_fraction: default_min_height_fraction(),
            lift_tolerance: default_lift_tolerance(),
            over_assist_margin: default_over_assist_margin(),
            over_support_flag_fraction: default_over_support_flag_fraction(),
            support_mode: AssistMode::Ladder,
            fade_tau_s: default_fade_tau_s(),
            catch_tau_s: default_catch_tau_s(),
            hold_after_catch_s: default_hold_after_catch_s(),
            cone_min_deg: default_cone_min_deg(),
            catch_threshold: default_catch_threshold(),
            lean_grace_s: default_lean_grace_s(),
            stance_grace_s: default_stance_grace_s(),
            standing_fraction_for_success: default_standing_fraction_for_success(),
            scheduled_events: Vec::new(),
            record_actuators: false,
        }
    }

    /// The adaptive assist is on.
    pub fn is_adaptive(&self) -> bool {
        self.support_mode == AssistMode::Adaptive
    }

    /// Bursts of continuous balance that accept an attempt, at the live burst period.
    pub fn hold_bursts(&self) -> u32 {
        bursts_for_seconds(self.hold_s, self.burst_period_s)
    }

    /// Bursts the tilt must stay past `fall_tilt_deg` to confirm a fall, at the live burst
    /// period (at least 1).
    pub fn fall_confirm_bursts(&self) -> u32 {
        bursts_for_seconds(self.fall_confirm_s, self.burst_period_s)
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
        if !self.fall_tilt_deg.is_finite()
            || !(self.fall_tilt_deg == 0.0
                || (self.fall_tilt_deg > self.max_tilt_deg && self.fall_tilt_deg <= 180.0))
        {
            return Err(TrainerError::Config(
                "fall_tilt_deg must be 0 (off) or greater than max_tilt_deg and <= 180".to_string(),
            ));
        }
        if !self.min_height_fraction.is_finite()
            || !(self.min_height_fraction == 0.0
                || (self.min_height_fraction > 0.0 && self.min_height_fraction <= 1.0))
        {
            return Err(TrainerError::Config(
                "min_height_fraction must be 0 (off) or in (0, 1]".to_string(),
            ));
        }
        if !self.lift_tolerance.is_finite() || !(0.0..=10.0).contains(&self.lift_tolerance) {
            return Err(TrainerError::Config(
                "lift_tolerance must be a finite value in [0, 10]".to_string(),
            ));
        }
        if !self.over_assist_margin.is_finite() || !(0.0..=1.0).contains(&self.over_assist_margin) {
            return Err(TrainerError::Config(
                "over_assist_margin must be a finite value in [0, 1]".to_string(),
            ));
        }
        if !self.over_support_flag_fraction.is_finite()
            || !(0.0..=1.0).contains(&self.over_support_flag_fraction)
        {
            return Err(TrainerError::Config(
                "over_support_flag_fraction must be a finite value in [0, 1]".to_string(),
            ));
        }
        if !positive(self.fade_tau_s) {
            return Err(TrainerError::Config(
                "fade_tau_s must be finite and > 0".to_string(),
            ));
        }
        if !positive(self.catch_tau_s) || self.catch_tau_s >= self.fade_tau_s {
            return Err(TrainerError::Config(
                "catch_tau_s must be finite, > 0 and < fade_tau_s".to_string(),
            ));
        }
        if !non_negative(self.hold_after_catch_s) {
            return Err(TrainerError::Config(
                "hold_after_catch_s must be finite and >= 0".to_string(),
            ));
        }
        if !non_negative(self.cone_min_deg) || self.cone_min_deg >= self.max_tilt_deg {
            return Err(TrainerError::Config(
                "cone_min_deg must be finite, >= 0 and < max_tilt_deg".to_string(),
            ));
        }
        if !self.catch_threshold.is_finite() || !(0.0..=1.0).contains(&self.catch_threshold) {
            return Err(TrainerError::Config(
                "catch_threshold must be a finite value in [0, 1]".to_string(),
            ));
        }
        if !positive(self.lean_grace_s) {
            return Err(TrainerError::Config(
                "lean_grace_s must be finite and > 0".to_string(),
            ));
        }
        if !positive(self.stance_grace_s) {
            return Err(TrainerError::Config(
                "stance_grace_s must be finite and > 0".to_string(),
            ));
        }
        if !self.standing_fraction_for_success.is_finite()
            || self.standing_fraction_for_success <= 0.0
            || self.standing_fraction_for_success > 1.0
        {
            return Err(TrainerError::Config(
                "standing_fraction_for_success must be finite and in (0, 1]".to_string(),
            ));
        }
        if self.probe_every > MAX_PROBE_EVERY {
            return Err(TrainerError::Config(format!(
                "probe_every must be 0 (off) or at most {MAX_PROBE_EVERY}"
            )));
        }
        if !non_negative(self.fall_confirm_s) {
            return Err(TrainerError::Config(
                "fall_confirm_s must be finite and >= 0".to_string(),
            ));
        }
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
        // An attempt ends at `hold_s` (success) or `attempt_cap_s`, and hold_s <= cap, so no
        // attempt can balance longer than `hold_s`: a larger threshold can never graduate.
        if let Some(grad) = self.graduate_balanced_s {
            if grad.is_finite() && grad > self.hold_s {
                return Err(TrainerError::Config(format!(
                    "graduate_balanced_s ({grad}) must be <= hold_s ({}) and attempt_cap_s ({}); \
                     an attempt never balances longer than hold_s, so the level could never graduate",
                    self.hold_s, self.attempt_cap_s
                )));
            }
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
        ];
        for (name, value) in strictly_positive {
            if value.is_some_and(|v| !positive(v)) {
                return Err(TrainerError::Config(format!(
                    "{name} must be finite and > 0"
                )));
            }
        }
        let damping = [(
            "support_rot_damping_nm_s_per_rad",
            self.support_rot_damping_nm_s_per_rad,
        )];
        for (name, value) in damping {
            if value.is_some_and(|v| !non_negative(v)) {
                return Err(TrainerError::Config(format!(
                    "{name} must be finite and >= 0"
                )));
            }
        }
        if !self.bws_max_fraction.is_finite()
            || !(0.0..=MAX_BWS_FRACTION).contains(&self.bws_max_fraction)
        {
            return Err(TrainerError::Config(format!(
                "bws_max_fraction must be in [0, {MAX_BWS_FRACTION}]"
            )));
        }
        for event in &self.scheduled_events {
            event.validate()?;
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

    /// Continue at `level`, clamped to `[config.min, config.start]`.
    ///
    /// `config.start` stays the ceiling, so later failed blocks can still
    /// raise the grip back toward a full hold. Block counters start empty.
    pub fn resume(config: SupportConfig, level: f64) -> Self {
        let mut schedule = Self::new(config);
        if level.is_finite() {
            schedule.level = level.clamp(schedule.config.min, schedule.config.start);
        }
        schedule
    }

    /// Level the next attempt would use after folding `attempts` in order.
    pub fn level_after_attempts(config: SupportConfig, attempts: &[BalanceAttemptRecord]) -> f64 {
        let mut schedule = Self::new(config);
        for attempt in attempts {
            schedule.record_attempt(attempt.outcome, attempt.balanced_s);
        }
        schedule.current_level()
    }

    /// Level to send with the next attempt.
    pub fn current_level(&self) -> f64 {
        self.level
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
    /// Train schedule whose first attempt is at this level. `config.start` stays the ceiling.
    /// With the adaptive assist, both the level and its slow average start here.
    Resume(f64),
    /// Adaptive assist resumed from a saved state. Without `support_mode: adaptive` it behaves
    /// like `Resume(a)`.
    ResumeAdaptive(AdaptiveAssistState),
}

/// Most points kept in a phase's assist timeline.
pub const ASSIST_TIMELINE_MAX_POINTS: usize = 3600;

/// The two numbers the adaptive assist carries between attempts and train runs.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AdaptiveAssistState {
    /// Assist level now, in `[support_min, support_start]`.
    pub a: f64,
    /// Exponential average of `a` with time constant `fade_tau_s`; it sets the cone.
    pub a_slow: f64,
}

/// Result of one adaptive-assist update.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdaptiveStep {
    /// Assist level after the update.
    pub level: f64,
    /// Slow average after the update.
    pub a_slow: f64,
    /// Tolerance cone, degrees, in force after the update.
    pub cone_deg: f64,
    /// Increase of `level` over the previous burst (0 when it did not rise).
    pub rise: f64,
    /// The rise was larger than `catch_threshold`.
    pub catch: bool,
}

/// Continuous "patient parent" assist. It fades slowly while the robot copes (`a` decays with
/// `fade_tau_s`), rises fast when the predicted tilt overshoots a tolerance cone (time
/// constant `catch_tau_s`), pauses the fade for `hold_after_catch_s` after a catch, and widens
/// the cone as the slow average `a_slow` falls. Every time constant is in seconds and meets the
/// burst rate only through `burst_period_s`, so it means the same at any burst rate. Metrics
/// observe this state; nothing feeds back into it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdaptiveAssist {
    a: f64,
    a_slow: f64,
    hold_left_s: f64,
}

impl AdaptiveAssist {
    /// Full assist: both averages at `support_start`.
    pub fn start(criterion: &BalanceFeetCriterion) -> Self {
        Self {
            a: criterion.support_start,
            a_slow: criterion.support_start,
            hold_left_s: 0.0,
        }
    }

    /// Continue from a saved state, clamped to `[support_min, support_start]`.
    pub fn resume(criterion: &BalanceFeetCriterion, state: AdaptiveAssistState) -> Self {
        let clamp = |value: f64, fallback: f64| {
            if value.is_finite() {
                value.clamp(criterion.support_min, criterion.support_start)
            } else {
                fallback
            }
        };
        Self {
            a: clamp(state.a, criterion.support_start),
            a_slow: clamp(state.a_slow, criterion.support_start),
            hold_left_s: 0.0,
        }
    }

    /// State to save for a later attempt or run.
    pub fn state(&self) -> AdaptiveAssistState {
        AdaptiveAssistState {
            a: self.a,
            a_slow: self.a_slow,
        }
    }

    /// Assist level now.
    pub fn level(&self) -> f64 {
        self.a
    }

    /// Slow average of the level.
    pub fn slow(&self) -> f64 {
        self.a_slow
    }

    /// Tolerance cone in degrees: `cone_min + (max_tilt - cone_min) * (1 - a_slow)`.
    pub fn cone_deg(&self, criterion: &BalanceFeetCriterion) -> f64 {
        let min = criterion.cone_min_deg;
        let max = criterion.max_tilt_deg;
        (min + (max - min) * (1.0 - self.a_slow)).clamp(min, max)
    }

    /// One update from the predicted tilt (degrees) of `bursts` elapsed bursts (normally 1).
    pub fn step(
        &mut self,
        criterion: &BalanceFeetCriterion,
        predicted_deg: f64,
        bursts: u32,
    ) -> AdaptiveStep {
        let dt = criterion.burst_period_s * f64::from(bursts.max(1));
        self.a_slow += (self.a - self.a_slow) * (1.0 - (-dt / criterion.fade_tau_s).exp());
        let cone = self.cone_deg(criterion);
        let span = criterion.max_tilt_deg - cone;
        let overshoot = if span > 1.0e-9 {
            ((predicted_deg - cone) / span).clamp(0.0, 1.0)
        } else if predicted_deg > cone {
            1.0
        } else {
            0.0
        };
        let previous = self.a;
        let hold_active = self.hold_left_s > 1.0e-9;
        self.hold_left_s = (self.hold_left_s - dt).max(0.0);
        let mut rise = 0.0;
        let mut catch = false;
        if overshoot > self.a {
            self.a += (overshoot - self.a) * (1.0 - (-dt / criterion.catch_tau_s).exp());
            self.a = self.a.clamp(criterion.support_min, criterion.support_start);
            rise = (self.a - previous).max(0.0);
            if rise > criterion.catch_threshold {
                catch = true;
                self.hold_left_s = criterion.hold_after_catch_s;
            }
        } else if !hold_active {
            self.a = (self.a * (-dt / criterion.fade_tau_s).exp()).max(overshoot);
            self.a = self.a.clamp(criterion.support_min, criterion.support_start);
        }
        AdaptiveStep {
            level: self.a,
            a_slow: self.a_slow,
            cone_deg: self.cone_deg(criterion),
            rise,
            catch,
        }
    }
}

/// Predicted tilt in degrees: `tilt + lookahead * |omega_xy|` (the support-need prediction,
/// not clamped).
pub fn predicted_tilt_deg(tilt_rad: f64, ang_vel: [f64; 3], lookahead_s: f64) -> f64 {
    (tilt_rad + lookahead_s * ang_vel[0].hypot(ang_vel[1])).to_degrees()
}

/// One point of a phase's assist timeline.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AssistPoint {
    /// Seconds of scored simulated time since the phase started.
    pub t_s: f64,
    /// Assist level.
    pub a: f64,
    /// Slow average of the level.
    pub a_slow: f64,
    /// Tolerance cone, degrees.
    pub cone_deg: f64,
}

/// One second of scored train time: mean goal deviation and mean applied assistance.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DeviationAssistPoint {
    /// Seconds of scored simulated time since the phase started, at the end of the second.
    pub t_s: f64,
    /// Mean goal deviation over the bursts in that second, `[0, 1]`.
    pub deviation_mean: f64,
    /// Mean applied assistance over the bursts in that second, `[0, 1]`.
    pub assistance_mean: f64,
}

/// Mean deviation and assistance over a stretch of the timeline.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DeviationAssistMeans {
    pub dev: f64,
    pub assist: f64,
}

/// Deviation against assistance over a train phase (probes and settling excluded).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviationAssist {
    /// One point per second of scored time (at most [`ASSIST_TIMELINE_MAX_POINTS`]).
    pub timeline: Vec<DeviationAssistPoint>,
    /// Pearson r between deviation and assistance. Null with fewer than 3 points or no variance.
    pub r: Option<f64>,
    /// Least-squares slope of assistance on deviation. Null without deviation variance.
    pub slope: Option<f64>,
    /// Least-squares intercept of assistance on deviation. Null without deviation variance.
    pub intercept: Option<f64>,
    /// Points in the timeline.
    pub n: usize,
    /// Means over the first 20% of the timeline.
    pub early: DeviationAssistMeans,
    /// Means over the last 20% of the timeline.
    pub late: DeviationAssistMeans,
}

impl DeviationAssist {
    /// Summary of a timeline. `None` for an empty timeline.
    pub fn from_timeline(timeline: Vec<DeviationAssistPoint>) -> Option<Self> {
        let n = timeline.len();
        if n == 0 {
            return None;
        }
        let nf = n as f64;
        let mean_x = timeline.iter().map(|p| p.deviation_mean).sum::<f64>() / nf;
        let mean_y = timeline.iter().map(|p| p.assistance_mean).sum::<f64>() / nf;
        let (mut sxx, mut syy, mut sxy) = (0.0, 0.0, 0.0);
        for p in &timeline {
            let dx = p.deviation_mean - mean_x;
            let dy = p.assistance_mean - mean_y;
            sxx += dx * dx;
            syy += dy * dy;
            sxy += dx * dy;
        }
        const EPS: f64 = 1.0e-12;
        let slope = (sxx > EPS).then(|| sxy / sxx);
        let intercept = slope.map(|m| mean_y - m * mean_x);
        let r = (n >= 3 && sxx > EPS && syy > EPS).then(|| sxy / (sxx * syy).sqrt());
        let k = ((nf * 0.2).round() as usize).clamp(1, n);
        let means = |slice: &[DeviationAssistPoint]| DeviationAssistMeans {
            dev: slice.iter().map(|p| p.deviation_mean).sum::<f64>() / slice.len() as f64,
            assist: slice.iter().map(|p| p.assistance_mean).sum::<f64>() / slice.len() as f64,
        };
        Some(Self {
            early: means(&timeline[..k]),
            late: means(&timeline[n - k..]),
            r,
            slope,
            intercept,
            n,
            timeline,
        })
    }
}

/// Collects the 1 Hz deviation and assistance timeline of a train phase.
#[derive(Debug, Default)]
struct DeviationAssistRun {
    timeline: Vec<DeviationAssistPoint>,
    phase_t_s: f64,
    bucket: u64,
    sum_dev: f64,
    sum_assist: f64,
    weight: f64,
}

impl DeviationAssistRun {
    /// Fold `bursts` counted bursts of `burst_period_s` each into the open second.
    fn add(&mut self, deviation: f64, assistance: f64, bursts: u32, burst_period_s: f64) {
        let bucket = self.phase_t_s.max(0.0).floor() as u64;
        if bucket != self.bucket {
            self.flush((self.bucket + 1) as f64);
            self.bucket = bucket;
        }
        let w = f64::from(bursts);
        self.sum_dev += deviation * w;
        self.sum_assist += assistance * w;
        self.weight += w;
        self.phase_t_s += w * burst_period_s;
    }

    fn flush(&mut self, t_s: f64) {
        if self.weight <= 0.0 {
            return;
        }
        if self.timeline.len() >= ASSIST_TIMELINE_MAX_POINTS {
            // Full: keep every 2nd point and carry on.
            let mut index = 0;
            self.timeline.retain(|_| {
                index += 1;
                index % 2 == 1
            });
        }
        self.timeline.push(DeviationAssistPoint {
            t_s,
            deviation_mean: self.sum_dev / self.weight,
            assistance_mean: self.sum_assist / self.weight,
        });
        self.sum_dev = 0.0;
        self.sum_assist = 0.0;
        self.weight = 0.0;
    }

    fn finish(mut self) -> Option<DeviationAssist> {
        let end = self.phase_t_s;
        self.flush(end);
        DeviationAssist::from_timeline(self.timeline)
    }
}

/// Adaptive numbers of one attempt (observation only).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AdaptiveAttemptStats {
    /// Mean assist level over the scored bursts.
    pub mean_a: f64,
    /// Lowest assist level over the scored bursts.
    pub min_a: f64,
    /// Tolerance cone at the end of the attempt, degrees.
    pub end_cone_deg: f64,
    /// Catches (assist jumps above `catch_threshold` in one burst).
    pub catches: u32,
    /// Catches per minute of scored time.
    pub catches_per_min: f64,
    /// Share of the counted bursts that were standing (`sample_is_upright`).
    #[serde(default)]
    pub standing_fraction: f64,
    /// Share of the counted bursts with every contact body down.
    #[serde(default)]
    pub feet_down_fraction: f64,
    /// Times a lean was re-centred (the cone dropped to `cone_min_deg`).
    #[serde(default)]
    pub recenters: u32,
    /// The attempt ended as `lost_stance`.
    #[serde(default)]
    pub lost_stance: bool,
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
    /// The torso stayed at or past `fall_tilt_deg` for `fall_confirm_s`. Not a death.
    Fell,
    /// Adaptive assist only: the robot was not standing for longer than `stance_grace_s`.
    /// Not a fall, not a success.
    LostStance,
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
    /// Scheduled support level of the attempt. The opening settle spring is not recorded here.
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
    /// True for an unassisted probe attempt run between training attempts. Probes never feed
    /// the support schedule, success counts or graduation.
    #[serde(default)]
    pub probe: bool,
    /// Training attempts completed when this probe ran. Absent for a non-probe attempt.
    #[serde(default)]
    pub train_attempts_before: Option<u32>,
    /// Non-settling bursts that failed each upright test (a burst can fail several), so a
    /// result can say why balance broke.
    #[serde(default)]
    pub non_upright: NonUprightCounts,
    /// Share of non-settling samples (support > 0 only) where the support held the body up: a
    /// contact body off the floor while the torso is not under `min_height_fraction`, or the
    /// torso above `reset height * (1 + lift_tolerance)`. 0 for an unassisted attempt.
    #[serde(default)]
    pub carried_fraction: f64,
    /// Share of non-settling samples (support > 0 only) where the support effort exceeded the
    /// predicted need by more than `over_assist_margin`. 0 for an unassisted attempt.
    #[serde(default)]
    pub over_assist_fraction: f64,
    /// Adaptive-assist numbers. Absent for a ladder attempt and for probes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adaptive: Option<AdaptiveAttemptStats>,
}

/// Counted support-use samples of one attempt (support > 0, non-settling only).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SupportUseCounts {
    /// Samples scored.
    pub samples: u32,
    /// Samples where the support carried the body.
    pub carried: u32,
    /// Samples where the effort exceeded the need by more than the margin.
    pub over_assist: u32,
}

impl SupportUseCounts {
    /// Add `bursts` samples with the given flags (`None` when the sample is not scored).
    pub fn add(self, flags: Option<(bool, bool)>, bursts: u32) -> Self {
        let Some((carried, over)) = flags else {
            return self;
        };
        Self {
            samples: self.samples.saturating_add(bursts),
            carried: self
                .carried
                .saturating_add(if carried { bursts } else { 0 }),
            over_assist: self
                .over_assist
                .saturating_add(if over { bursts } else { 0 }),
        }
    }

    fn fraction(count: u32, samples: u32) -> f64 {
        if samples == 0 {
            0.0
        } else {
            f64::from(count) / f64::from(samples)
        }
    }

    /// Carried share of the scored samples.
    pub fn carried_fraction(&self) -> f64 {
        Self::fraction(self.carried, self.samples)
    }

    /// Over-assist share of the scored samples.
    pub fn over_assist_fraction(&self) -> f64 {
        Self::fraction(self.over_assist, self.samples)
    }
}

/// Whether the support carried the body and whether it over-assisted at one sample, as
/// `(carried, over_assist)`. `None` when the sample is not scored: settling, or no support.
/// `level` is the effective support level.
pub fn support_use_flags(
    sample: &BalanceSample,
    criterion: &BalanceFeetCriterion,
    reset_height_m: Option<f64>,
    level: f64,
) -> Option<(bool, bool)> {
    if sample.settling || level.is_nan() || level <= 0.0 {
        return None;
    }
    let reset = reset_height_m.filter(|reset| *reset > 0.0);
    let torso_low = match (sample.torso_height_m, reset) {
        (Some(now), Some(reset)) => {
            criterion.min_height_fraction > 0.0 && now < criterion.min_height_fraction * reset
        }
        _ => false,
    };
    let feet_off_body_up = !sample.all_contacts_down() && !torso_low;
    let lifted = match (sample.torso_height_m, reset) {
        (Some(now), Some(reset)) => now > reset * (1.0 + criterion.lift_tolerance),
        _ => false,
    };
    let over = match (sample.support_effort, sample.support_need) {
        (Some(effort), Some(need)) => effort > need + criterion.over_assist_margin,
        _ => false,
    };
    Some((feet_off_body_up || lifted, over))
}

/// Which upright tests one sample fails.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NonUprightReasons {
    /// Tilt is past `max_tilt_deg`.
    pub tilt: bool,
    /// A contact body is off the world.
    pub contact: bool,
    /// A body that is not a contact body touches the world (`only_contacts_touch`).
    pub other_body_contact: bool,
    /// The torso is lower than `min_height_fraction` of its reset height.
    pub height: bool,
}

impl NonUprightReasons {
    /// The sample fails at least one test.
    pub fn any(&self) -> bool {
        self.tilt || self.contact || self.other_body_contact || self.height
    }
}

/// Bursts per failed upright test over one attempt.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonUprightCounts {
    /// Bursts with tilt past the limit.
    #[serde(default)]
    pub tilt: u32,
    /// Bursts with a contact body off the world.
    #[serde(default)]
    pub contact: u32,
    /// Bursts with a non-contact body on the world.
    #[serde(default)]
    pub other_body_contact: u32,
    /// Bursts with the torso under the height fraction.
    #[serde(default)]
    pub height: u32,
}

impl NonUprightCounts {
    /// Add `bursts` for every test `reasons` fails.
    pub fn add(self, reasons: NonUprightReasons, bursts: u32) -> Self {
        let plus = |count: u32, failed: bool| {
            if failed {
                count.saturating_add(bursts)
            } else {
                count
            }
        };
        Self {
            tilt: plus(self.tilt, reasons.tilt),
            contact: plus(self.contact, reasons.contact),
            other_body_contact: plus(self.other_body_contact, reasons.other_body_contact),
            height: plus(self.height, reasons.height),
        }
    }
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
    /// Whole-robot mass m: subtree mass of the torso's free root, kg.
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

/// Support-harness parameters sent to the controller (angles rad).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpringGains {
    /// Rotational stiffness, N*m/rad.
    pub k_rot: f64,
    /// Rotational damping, N*m*s/rad.
    pub d_rot: f64,
    /// Fraction of the whole body's weight carried by the constant upward force on the torso
    /// body at support level 1 (sent to the controller as `gravity_comp`). There are no
    /// position springs: `k_lin` and `d_lin` are not sent.
    pub gravity_comp: f64,
}

/// Size the support harness (documented formulas):
///
/// anti-tilt torque `k_rot = 3 m g h0 / 0.087` (a 5 degree tilt costs about a third of the
/// weight torque), `d_rot = min(0.4 * 2 sqrt(k_rot I_tilt), 0.9 * 2 I_min / dt)` (the second
/// term keeps the explicit-force integration stable). `I_tilt` and `I_min` come from the torso
/// root's subtree inertia; k_rot and d_rot are then capped by the torso body's own inertia
/// (`d dt / I_own < 2` with 10% margin, `k dt^2 / I_own <= 0.25`) so limp-limb models do
/// not diverge. There are no linear position springs. Weight support is a constant upward
/// force `level * bws_max_fraction * m_total * g` on the torso body, with `m_total` the
/// whole robot's mass; here `gravity_comp` is the default fraction 0.5.
pub fn size_spring_gains(inputs: &SizingInputs) -> SpringGains {
    let weight = inputs.mass_kg * inputs.gravity;
    let k_rot = cap_k_rot(3.0 * weight * inputs.h0_m / 0.087, inputs);
    let d_rot = sized_d_rot(k_rot, inputs);
    SpringGains {
        k_rot,
        d_rot,
        gravity_comp: MAX_BWS_FRACTION,
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

/// Sized gains with the criterion's explicit overrides and `bws_max_fraction` applied. An overridden `k_rot` feeds the
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
    gains.gravity_comp = criterion.bws_max_fraction;
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
    /// Extra contact sets after `contacts`: each entry is a body list to EXCLUDE (their
    /// descendants and the world are excluded too); the set is every other body touching the
    /// world. Sent as `{"a_except": [...], "b": "world"}`.
    pub contacts_except: Vec<Vec<String>>,
    /// Also report every actuator's command. Sent only when true.
    pub actuators: bool,
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
    /// Names of the bodies in contact (newer controllers).
    #[serde(default)]
    pub bodies: Vec<String>,
    /// Total normal force of the set, N (newer controllers).
    #[serde(default)]
    pub normal_force_n: Option<f64>,
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
    /// Actuator command (ctrl) by actuator name; present only when requested.
    #[serde(default)]
    pub actuators: BTreeMap<String, f64>,
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
    /// A body that is not a contact body touches the world. False when not measured.
    #[serde(default)]
    pub other_contact_touching: bool,
    /// Names of those bodies, for diagnostics (empty when none or not reported).
    #[serde(default)]
    pub other_contact_bodies: Vec<String>,
    /// Torso body height above world z (its CoM z), m, when reported.
    #[serde(default)]
    pub torso_height_m: Option<f64>,
    /// Torso height on the attempt's first sample after the reset, m. The first sample may leave
    /// it unset; the attempt tracker then uses the first sample's own height.
    #[serde(default)]
    pub reset_height_m: Option<f64>,
    /// Adaptive assist: slow average of the level at this burst. Absent in ladder mode.
    #[serde(default)]
    pub assist_slow: Option<f64>,
    /// Adaptive assist: increase of the level over the previous burst. Absent in ladder mode.
    #[serde(default)]
    pub assist_rise: Option<f64>,
    /// Actuator command (ctrl) by actuator name at this burst. Empty unless the criterion sets
    /// `record_actuators`.
    #[serde(default)]
    pub actuators: BTreeMap<String, f64>,
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
    /// Consecutive elapsed bursts with tilt at or past the fall threshold.
    pub fall_bursts: u32,
    /// Torso height on the first sample after the reset (world z), m.
    pub reset_height_m: Option<f64>,
    /// Non-settling bursts per failed upright test so far.
    pub non_upright: NonUprightCounts,
    /// Support-use samples so far (support > 0, non-settling).
    pub support_use: SupportUseCounts,
    /// Consecutive elapsed bursts that were not standing, ending at the latest counted burst.
    pub off_stance_bursts: u32,
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
            fall_bursts: 0,
            reset_height_m: None,
            non_upright: NonUprightCounts::default(),
            support_use: SupportUseCounts::default(),
            off_stance_bursts: 0,
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

/// The upright tests a sample fails. `reset_height_m` is the torso height on the attempt's
/// first sample; without it (or at 0 or below) the height test passes.
///
/// Upright (standing) needs all of: tilt inside `max_tilt_deg`; every contact body down; when
/// `only_contacts_touch`, no other body touching the world; when `min_height_fraction > 0`, torso
/// height (CoM z above world z) at least `min_height_fraction * reset_height_m`.
pub fn non_upright_reasons(
    sample: &BalanceSample,
    criterion: &BalanceFeetCriterion,
    reset_height_m: Option<f64>,
) -> NonUprightReasons {
    let too_low = match (sample.torso_height_m, reset_height_m) {
        (Some(now), Some(reset)) if criterion.min_height_fraction > 0.0 && reset > 0.0 => {
            now < criterion.min_height_fraction * reset
        }
        _ => false,
    };
    NonUprightReasons {
        tilt: sample.tilt_deg > criterion.max_tilt_deg,
        contact: !sample.all_contacts_down(),
        other_body_contact: criterion.only_contacts_touch && sample.other_contact_touching,
        height: too_low,
    }
}

/// Goal deviation of one sample, in `[0, 1]`: how far the body is from the standing goal.
///
/// 1.0 when the sample is not in a standing stance for a reason other than tilt (a contact body
/// is off the floor, another body is touching, or the torso is below `min_height_fraction` of
/// the reset height; see [`non_upright_reasons`]). Otherwise `tilt_deg / max_tilt_deg`, clamped
/// to `[0, 1]`.
pub fn goal_deviation(
    sample: &BalanceSample,
    criterion: &BalanceFeetCriterion,
    reset_height_m: Option<f64>,
) -> f64 {
    let reasons = non_upright_reasons(sample, criterion, reset_height_m);
    if reasons.contact || reasons.other_body_contact || reasons.height {
        return 1.0;
    }
    if criterion.max_tilt_deg <= 0.0 {
        return 1.0;
    }
    (sample.tilt_deg / criterion.max_tilt_deg).clamp(0.0, 1.0)
}

/// Standing: tilt inside the limit, every contact body down, no other body on the world (when
/// `only_contacts_touch`) and the torso above its height fraction (when `min_height_fraction`
/// is on). Uses the sample's own reset height.
pub fn sample_is_upright(sample: &BalanceSample, criterion: &BalanceFeetCriterion) -> bool {
    let reset = sample.reset_height_m.or(sample.torso_height_m);
    !non_upright_reasons(sample, criterion, reset).any()
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
    let reset_height_m = open
        .reset_height_m
        .or(sample.reset_height_m)
        .or(sample.torso_height_m);
    let reasons = non_upright_reasons(sample, criterion, reset_height_m);
    let upright = !reasons.any();
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
        let non_upright = if sample.settling {
            open.non_upright
        } else {
            open.non_upright.add(reasons, 1)
        };
        let level = sample.support_level.unwrap_or(open.params_support_level);
        let support_use = open.support_use.add(
            support_use_flags(sample, criterion, reset_height_m, level),
            1,
        );
        return Ok(BalanceAttemptStep::Finished(BalanceAttemptRecord {
            seed,
            outcome,
            bursts,
            final_tilt_deg: sample.tilt_deg,
            support_level: open.params_support_level,
            upright_fraction: upright_fraction(upright_bursts, bursts),
            perturbations,
            non_upright,
            carried_fraction: support_use.carried_fraction(),
            over_assist_fraction: support_use.over_assist_fraction(),
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
        next.reset_height_m = reset_height_m;
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
        next.reset_height_m = reset_height_m;
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
    let off_stance_bursts = if upright {
        0
    } else {
        open.off_stance_bursts.saturating_add(new_bursts)
    };
    let fall_bursts = if criterion.fall_tilt_deg > 0.0 && sample.tilt_deg >= criterion.fall_tilt_deg
    {
        open.fall_bursts.saturating_add(new_bursts)
    } else {
        0
    };
    let first_violation_burst =
        open.first_violation_burst
            .or(if upright { None } else { Some(open.bursts) });
    let non_upright = open.non_upright.add(reasons, new_bursts);
    let level = sample.support_level.unwrap_or(open.params_support_level);
    let support_use = open.support_use.add(
        support_use_flags(sample, criterion, reset_height_m, level),
        new_bursts,
    );
    let finish = |outcome| {
        BalanceAttemptStep::Finished(BalanceAttemptRecord {
            seed,
            outcome,
            bursts,
            final_tilt_deg: sample.tilt_deg,
            support_level: open.params_support_level,
            upright_fraction: upright_fraction(upright_bursts, bursts),
            perturbations,
            non_upright,
            carried_fraction: support_use.carried_fraction(),
            over_assist_fraction: support_use.over_assist_fraction(),
            ..balance_time_fields(
                longest_balanced_bursts,
                first_violation_burst,
                criterion.burst_period_s,
            )
        })
    };
    // The adaptive assist does not end an attempt at `hold_s`: it ends on a fall, on a lost
    // stance, or at the cap (a success when the robot stood for most of it).
    let adaptive = criterion.is_adaptive();
    if !adaptive && holding_bursts >= criterion.hold_bursts() {
        return Ok(finish(BalanceAttemptOutcome::Success));
    }
    if fall_bursts > 0 && fall_bursts >= criterion.fall_confirm_bursts() {
        return Ok(finish(BalanceAttemptOutcome::Fell));
    }
    if adaptive
        && f64::from(off_stance_bursts) * criterion.burst_period_s
            > criterion.stance_grace_s + 1.0e-9
    {
        return Ok(finish(BalanceAttemptOutcome::LostStance));
    }
    if bursts >= criterion.attempt_burst_cap() {
        // The adaptive cap is a success only for an attempt that mostly stood.
        let stood = f64::from(upright_bursts) / f64::from(bursts.max(1))
            >= criterion.standing_fraction_for_success - 1.0e-9;
        return Ok(finish(if adaptive && stood {
            BalanceAttemptOutcome::Success
        } else {
            BalanceAttemptOutcome::Truncated
        }));
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
        fall_bursts,
        reset_height_m,
        non_upright,
        support_use,
        off_stance_bursts,
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
        probe: false,
        train_attempts_before: None,
        non_upright: NonUprightCounts::default(),
        carried_fraction: 0.0,
        over_assist_fraction: 0.0,
        adaptive: None,
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
        let death = matches!(
            outcome,
            Some(BalanceAttemptOutcome::Death | BalanceAttemptOutcome::Fell)
        );
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
    /// [`BalanceContributionReward`] (mode name `contribution`).
    Contribution,
}

impl BalanceRewardPolicy {
    /// Parse a reward mode name; `none` has no policy and returns `None`.
    pub fn from_mode(mode: &str) -> Option<Self> {
        match mode {
            "affect" => Some(Self::Affect),
            "assist" => Some(Self::Assist),
            "contribution" => Some(Self::Contribution),
            _ => None,
        }
    }

    /// Policy identity recorded with the run.
    pub fn policy_id(self) -> &'static str {
        match self {
            Self::Affect => BalanceAffectReward::policy_id(),
            Self::Assist => BalanceAssistReward::policy_id(),
            Self::Contribution => BalanceContributionReward::policy_id(),
        }
    }
}

/// Assist-as-needed affect for one balance attempt, driven by the harness `support_need`
/// (falls back to `min(1, tilt / max_tilt)` when absent). Per counted, non-settling burst:
/// Death -> Pain 1.0; tilt rising through `max_tilt` -> Pain `min(1, tilt / max_tilt)`;
/// falling need -> Pleasure `min(1, ASSIST_FALL_GAIN * drop)`; a burst that meets the success
/// rule (tilt at or under `max_tilt` and every contact body down) -> Pleasure
/// `ASSIST_HOLD_PLEASURE`; Success -> Pleasure 1.0. Holding pleasure uses that success rule,
/// so a stance inside the limit is reinforced and a wobble that stays inside it is not Pain.
/// When several Pleasures apply in one burst the largest is emitted. Never scaled by support
/// level. Pleasure below `ASSIST_MIN_PLEASURE` is dropped. Settling bursts emit nothing,
/// including a Death; their need and tilt still seed the next burst.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BalanceAssistReward {
    previous_need: Option<f64>,
    previous_tilt_deg: Option<f64>,
}

impl BalanceAssistReward {
    /// Policy at the start of an attempt.
    pub fn new() -> Self {
        Self::default()
    }

    /// Stable identity recorded with the run.
    pub fn policy_id() -> &'static str {
        "reward.balance-assist.v2"
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
        let previous_tilt = self.previous_tilt_deg.unwrap_or(burst_sample.tilt_deg);
        self.previous_tilt_deg = Some(burst_sample.tilt_deg);
        let death = matches!(
            outcome,
            Some(BalanceAttemptOutcome::Death | BalanceAttemptOutcome::Fell)
        );
        let pain = |magnitude| RewardSignal {
            channel: AffectChannel::Pain,
            magnitude,
        };
        // A death during settle is the fall detector firing before the attempt starts.
        // Punishing it drives balance_punish on every reset.
        if burst_sample.settling {
            return Vec::new();
        }
        if death {
            return vec![pain(1.0)];
        }
        let mut signals = Vec::new();
        let upright = sample_is_upright(burst_sample, criterion);
        if !upright && burst_sample.tilt_deg > previous_tilt {
            let past = (burst_sample.tilt_deg / criterion.max_tilt_deg).clamp(0.0, 1.0);
            signals.push(pain(past));
        }
        let mut pleasure = 0.0_f64;
        if need < previous {
            pleasure = pleasure.max((ASSIST_FALL_GAIN * (previous - need)).min(1.0));
        }
        if upright {
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

/// Reward for what the robot does itself, not for being held. Per counted, non-settling burst,
/// with `a` the sample's `support_level` (the assist level):
/// Death or fall -> Pain 1.0; an assist increase since the previous burst (`assist_rise`) ->
/// Pain `min(1, rise / catch_threshold) * 0.5` (a catch, the rise above the threshold, pays the
/// full 0.5; a smaller rise pays a share of it); need below the previous burst's ->
/// Pleasure `min(1, ASSIST_FALL_GAIN * drop) * (1 - a)`; an attempt that ends in Success ->
/// Pleasure `1 - a_slow` (`assist_slow`, else `a`); an attempt that ends in `LostStance` ->
/// Pain 0.5. Pleasure (recovery and the cap) is paid only on bursts where the robot is standing
/// ([`sample_is_upright`]). Being upright, or being held, pays nothing by itself.
/// Works with the ladder, where `a` is the constant level. Pleasure below
/// `ASSIST_MIN_PLEASURE` is dropped. Settling bursts emit nothing but seed the previous need.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BalanceContributionReward {
    previous_need: Option<f64>,
}

impl BalanceContributionReward {
    /// Policy at the start of an attempt.
    pub fn new() -> Self {
        Self::default()
    }

    /// Stable identity recorded with the run.
    pub fn policy_id() -> &'static str {
        "reward.balance-contribution.v2"
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
        if burst_sample.settling {
            return Vec::new();
        }
        let signal = |channel, magnitude: f64| RewardSignal { channel, magnitude };
        if matches!(
            outcome,
            Some(BalanceAttemptOutcome::Death | BalanceAttemptOutcome::Fell)
        ) {
            return vec![signal(AffectChannel::Pain, 1.0)];
        }
        if outcome == Some(&BalanceAttemptOutcome::LostStance) {
            return vec![signal(AffectChannel::Pain, 0.5)];
        }
        let standing = sample_is_upright(burst_sample, criterion);
        let a = burst_sample.support_level.unwrap_or(0.0).clamp(0.0, 1.0);
        let mut signals = Vec::new();
        let rise = burst_sample.assist_rise.unwrap_or(0.0);
        if rise > 1.0e-9 {
            let share = if criterion.catch_threshold > 1.0e-9 {
                (rise / criterion.catch_threshold).min(1.0)
            } else {
                1.0
            };
            signals.push(signal(AffectChannel::Pain, share * 0.5));
        }
        let mut pleasure = 0.0_f64;
        if need < previous {
            pleasure = (ASSIST_FALL_GAIN * (previous - need)).min(1.0) * (1.0 - a);
        }
        if outcome == Some(&BalanceAttemptOutcome::Success) {
            let slow = burst_sample.assist_slow.unwrap_or(a).clamp(0.0, 1.0);
            pleasure = 1.0 - slow;
        }
        // Recovery and the cap pay only while the robot stands: a body held up off its feet or
        // leaning on the support earns nothing.
        if standing && pleasure >= ASSIST_MIN_PLEASURE {
            signals.push(signal(AffectChannel::Pleasure, pleasure));
        }
        signals
    }
}

/// The reward policy instance of one attempt.
enum AttemptReward {
    Affect(BalanceAffectReward),
    Assist(BalanceAssistReward),
    Contribution(BalanceContributionReward),
}

impl AttemptReward {
    fn new(policy: BalanceRewardPolicy, support_level: f64) -> Self {
        match policy {
            BalanceRewardPolicy::Affect => {
                Self::Affect(BalanceAffectReward::for_attempt(support_level))
            }
            BalanceRewardPolicy::Assist => Self::Assist(BalanceAssistReward::new()),
            BalanceRewardPolicy::Contribution => {
                Self::Contribution(BalanceContributionReward::new())
            }
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
            Self::Contribution(r) => r.on_sample(sample, outcome, criterion),
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
    /// Grip level the next training attempt should use. Absent for a fixed test phase.
    #[serde(default)]
    pub support_level_next: Option<f64>,
    /// Conditions of a brain-only test phase. Absent for a train phase.
    #[serde(default)]
    pub brain_only: Option<BrainOnlyTest>,
    /// Probe attempts in `attempts` (0 when probes were off or this is a test phase).
    #[serde(default)]
    pub probe_attempts: u32,
    /// Every probe ran with plasticity frozen. Absent when no probe ran.
    #[serde(default)]
    pub probe_plasticity_frozen: Option<bool>,
    /// Every probe: attempt index, the support level in force, balanced time and outcome.
    #[serde(default)]
    pub probes: Vec<ProbeResult>,
    /// Support use per scheduled support level, keyed by the level formatted `{:.2}`. Filled by
    /// [`BalancePhaseReport::assess_over_support`].
    #[serde(default)]
    pub support_use_by_support: BTreeMap<String, SupportUseStats>,
    /// Levels (keys of `support_use_by_support`) flagged `over_support`, ascending.
    #[serde(default)]
    pub over_support_levels: Vec<String>,
    /// Adaptive assist: level, slow average and cone sampled at 1 Hz of scored time across the
    /// phase (at most [`ASSIST_TIMELINE_MAX_POINTS`]). Empty in ladder mode.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assist_timeline: Vec<AssistPoint>,
    /// Goal deviation against applied assistance across a train phase. Absent for a brain-only test phase.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deviation_assist: Option<DeviationAssist>,
    /// Adaptive assist state at the end of the phase, to resume a later train run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adaptive_state_next: Option<AdaptiveAssistState>,
}

/// Result of one unassisted probe attempt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProbeResult {
    /// 1-based position of the probe among all attempts of the phase.
    pub attempt_index: u32,
    /// Training attempts completed when the probe ran.
    pub train_attempts_before: u32,
    /// Scheduled support level of the training attempt just before the probe.
    pub support_level: f64,
    /// Longest continuous balanced stretch of the probe, seconds.
    pub balanced_s: f64,
    /// How the probe ended.
    pub outcome: BalanceAttemptOutcome,
}

/// Why a support level was flagged `over_support`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverSupportCause {
    /// Mean carried fraction above the flag fraction.
    Carried,
    /// Mean over-assist fraction above the flag fraction.
    OverAssist,
    /// Training succeeded at the level but unassisted probes balanced under `hold_s` on median.
    ProbeDependence,
}

/// Support use at one scheduled support level.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SupportUseStats {
    /// Training attempts at the level.
    pub attempts: u32,
    /// Mean per-attempt `carried_fraction`.
    pub mean_carried_fraction: f64,
    /// Mean per-attempt `over_assist_fraction`.
    pub mean_over_assist_fraction: f64,
    /// The support is overdoing it at this level.
    pub over_support: bool,
    /// Why `over_support` is set (empty when it is not).
    #[serde(default)]
    pub over_support_causes: Vec<OverSupportCause>,
    /// Median balanced seconds of the unassisted probes judged against this level (during it,
    /// else the nearest after). Absent when none applies.
    #[serde(default)]
    pub probe_median_balanced_s: Option<f64>,
}

/// How a test phase ran: the brain alone, with nothing from the Trainer in the loop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrainOnlyTest {
    /// Support level of every attempt. Always 0.
    pub support: f64,
    /// Support during the opening settle. Always 0.
    pub settle_support: f64,
    /// Settle window actually applied to every attempt. Always 0: an unsupported attempt is
    /// scored from its reset sample, so the body cannot fall unscored during a settle.
    #[serde(default)]
    pub settle_s_applied: f64,
    /// Reward delivered to the brain: `none`.
    pub reward: String,
    /// Perturbation pulses were applied. Always false.
    pub perturbation: bool,
    /// Every plastic mapping was frozen for the phase.
    pub plasticity_frozen: bool,
    /// Why plasticity was not frozen. Absent when it was.
    #[serde(default)]
    pub plasticity_reason: Option<String>,
    /// Mappings that were frozen.
    #[serde(default)]
    pub mappings_frozen: u32,
    /// Memory-neuron formation was paused. FEAGI has no route for it, so false.
    #[serde(default)]
    pub memory_formation_paused: bool,
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

/// Summary key of a scored attempt: its support level (ladder), or its 1-based position among
/// the scored attempts when the phase ran the adaptive assist, where the level keeps moving.
fn group_key(adaptive: bool, index: usize, level: f64) -> String {
    if adaptive {
        format!("attempt_{:04}", index + 1)
    } else {
        support_key(level)
    }
}

/// Every scored attempt carries adaptive numbers.
fn phase_is_adaptive(scored: &[BalanceAttemptRecord]) -> bool {
    !scored.is_empty() && scored.iter().all(|a| a.adaptive.is_some())
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
        // Probes are recorded but never counted: every aggregate below covers scored attempts.
        let all = attempts;
        let attempts: Vec<BalanceAttemptRecord> =
            all.iter().filter(|a| !a.probe).cloned().collect();
        if attempts.is_empty() {
            return Err(TrainerError::Evaluation(
                "balance phase has no attempts".to_string(),
            ));
        }
        let probe_attempts = (all.len() - attempts.len()) as u32;
        let probes: Vec<ProbeResult> = all
            .iter()
            .enumerate()
            .filter(|(_, a)| a.probe)
            .map(|(i, a)| {
                let before = a.train_attempts_before.unwrap_or(0);
                let level = before
                    .checked_sub(1)
                    .and_then(|k| attempts.get(k as usize))
                    .map_or(a.support_level, |t| t.support_level);
                ProbeResult {
                    attempt_index: i as u32 + 1,
                    train_attempts_before: before,
                    support_level: level,
                    balanced_s: a.balanced_s,
                    outcome: a.outcome,
                }
            })
            .collect();
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
        let adaptive = phase_is_adaptive(&attempts);
        let mut tally: BTreeMap<String, (u32, u32)> = BTreeMap::new();
        for (i, attempt) in attempts.iter().enumerate() {
            let entry = tally
                .entry(group_key(adaptive, i, attempt.support_level))
                .or_default();
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
        for (i, attempt) in attempts.iter().enumerate() {
            by_level
                .entry(group_key(adaptive, i, attempt.support_level))
                .or_default()
                .push(attempt);
        }
        let balanced_by_support = by_level
            .into_iter()
            .map(|(key, group)| (key, BalancedTimeStats::from_attempts(group)))
            .collect();
        Ok(Self {
            attempts: all,
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
            support_level_next: None,
            brain_only: None,
            probe_attempts,
            probe_plasticity_frozen: None,
            probes,
            support_use_by_support: BTreeMap::new(),
            over_support_levels: Vec::new(),
            assist_timeline: Vec::new(),
            deviation_assist: None,
            adaptive_state_next: None,
        })
    }

    /// Fill `support_use_by_support` and `over_support_levels` from the criterion's thresholds.
    /// A level is flagged when its mean carried or over-assist fraction exceeds
    /// `over_support_flag_fraction`, or when its train success rate reaches
    /// `graduate_success_rate` while the unassisted probes judged against it (those run during
    /// the level, else the nearest after) balance under `hold_s` on median. Flags only; the
    /// schedule and the success rules are untouched.
    pub fn assess_over_support(&mut self, criterion: &BalanceFeetCriterion) {
        let train: Vec<&BalanceAttemptRecord> = self.attempts.iter().filter(|a| !a.probe).collect();
        let adaptive = !train.is_empty() && train.iter().all(|a| a.adaptive.is_some());
        let mut by_level: BTreeMap<String, Vec<&BalanceAttemptRecord>> = BTreeMap::new();
        // Last 1-based train index per level, for the "nearest probe after" rule.
        let mut last_index: BTreeMap<String, u32> = BTreeMap::new();
        for (i, attempt) in train.iter().enumerate() {
            let key = group_key(adaptive, i, attempt.support_level);
            by_level.entry(key.clone()).or_default().push(attempt);
            last_index.insert(key, i as u32 + 1);
        }
        let threshold = criterion.over_support_flag_fraction;
        let mut stats = BTreeMap::new();
        let mut flagged = Vec::new();
        for (key, group) in by_level {
            let n = group.len() as f64;
            let mean_carried = group.iter().map(|a| a.carried_fraction).sum::<f64>() / n;
            let mean_over = group.iter().map(|a| a.over_assist_fraction).sum::<f64>() / n;
            let mut causes = Vec::new();
            if mean_carried > threshold {
                causes.push(OverSupportCause::Carried);
            }
            if mean_over > threshold {
                causes.push(OverSupportCause::OverAssist);
            }
            let level = group[0].support_level;
            let successes = group
                .iter()
                .filter(|a| a.outcome == BalanceAttemptOutcome::Success)
                .count() as f64;
            let probe_median = if level > 1.0e-9 {
                let last = last_index.get(&key).copied().unwrap_or(0);
                let during: Vec<f64> = self
                    .probes
                    .iter()
                    .filter(|p| {
                        if adaptive {
                            p.train_attempts_before == last
                        } else {
                            support_key(p.support_level) == key
                        }
                    })
                    .map(|p| p.balanced_s)
                    .collect();
                let chosen = if during.is_empty() {
                    self.probes
                        .iter()
                        .find(|p| p.train_attempts_before > last)
                        .map(|p| vec![p.balanced_s])
                        .unwrap_or_default()
                } else {
                    during
                };
                (!chosen.is_empty()).then(|| median_f64(&chosen))
            } else {
                None
            };
            if successes / n >= criterion.graduate_success_rate
                && probe_median.is_some_and(|m| m < criterion.hold_s)
            {
                causes.push(OverSupportCause::ProbeDependence);
            }
            let over_support = !causes.is_empty();
            if over_support {
                flagged.push(key.clone());
            }
            stats.insert(
                key,
                SupportUseStats {
                    attempts: group.len() as u32,
                    mean_carried_fraction: mean_carried,
                    mean_over_assist_fraction: mean_over,
                    over_support,
                    over_support_causes: causes,
                    probe_median_balanced_s: probe_median,
                },
            );
        }
        self.support_use_by_support = stats;
        self.over_support_levels = flagged;
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
    if let Some(test) = &phase.brain_only {
        metrics.insert("brain_only".to_string(), 1.0);
        metrics.insert("test_support".to_string(), test.support);
        metrics.insert("test_settle_support".to_string(), test.settle_support);
        metrics.insert("test_settle_s_applied".to_string(), test.settle_s_applied);
        metrics.insert("test_reward_delivered".to_string(), 0.0);
        metrics.insert(
            "test_perturbation".to_string(),
            if test.perturbation { 1.0 } else { 0.0 },
        );
        metrics.insert(
            "test_plasticity_frozen".to_string(),
            if test.plasticity_frozen { 1.0 } else { 0.0 },
        );
        metrics.insert(
            "test_memory_formation_paused".to_string(),
            if test.memory_formation_paused {
                1.0
            } else {
                0.0
            },
        );
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
        metadata: balance_phase_metadata(phase),
    }
}

/// Scorecard metadata: the brain-only conditions of a test phase, in words.
fn balance_phase_metadata(phase: &BalancePhaseReport) -> BTreeMap<String, MetadataValue> {
    let mut metadata = BTreeMap::new();
    if let Some(test) = &phase.brain_only {
        metadata.insert(
            "test_mode".to_string(),
            MetadataValue::Text("brain_only".to_string()),
        );
        metadata.insert(
            "test_reward".to_string(),
            MetadataValue::Text(test.reward.clone()),
        );
        metadata.insert(
            "test_plasticity_frozen".to_string(),
            MetadataValue::Bool(test.plasticity_frozen),
        );
        if let Some(reason) = &test.plasticity_reason {
            metadata.insert(
                "test_plasticity_reason".to_string(),
                MetadataValue::Text(reason.clone()),
            );
        }
        metadata.insert(
            "test_graduated".to_string(),
            MetadataValue::Bool(phase.graduated),
        );
    }
    metadata
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

    /// `update_body_spring`: change the level of a running spring and, when `deadband_rad` is
    /// `Some`, its tilt deadband and, when `gravity_comp` is `Some`, its weight-support fraction.
    fn update_body_spring(
        &mut self,
        body: &str,
        level: f64,
        deadband_rad: Option<f64>,
        gravity_comp: Option<f64>,
    ) -> Result<(), TrainerError>;

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
    /// Support policy. `None`: `train` follows the schedule, `test` runs at support 0. A
    /// test phase accepts only `None` or `Fixed(0.0)`.
    pub support_mode: Option<SupportMode>,
    /// Affect delivery. `None` keeps the no-affect behavior. A test phase refuses a sink.
    pub deliver_reward: Option<DeliverReward<'a>>,
    /// Freeze/restore of learning around a test phase (ignored by train). `None` runs the
    /// test unfrozen and the report says why.
    pub plasticity: Option<&'a mut dyn PlasticityControl>,
    /// Which policy builds the affect signals when `deliver_reward` is set.
    pub reward_policy: BalanceRewardPolicy,
    /// Run data sink: the `steps` and `attempts` tables and the event log. `None` keeps
    /// today's behavior exactly.
    pub data: Option<&'a mut dyn RunDataSink>,
    /// Applies the criterion's scheduled events. Required when the criterion has any; a test
    /// phase refuses events.
    pub apply_event: Option<ApplyEvent<'a>>,
}

/// Per-phase state: model facts and the gains sized once from the first reset.
struct BalanceSession {
    info: ModelInfo,
    sizing: Option<(SizingInputs, SpringGains)>,
}

/// Run one phase (`train` or `test`) and emit its progress.
///
/// A `test` phase measures the brain alone: support 0 for the whole attempt (the spring is set
/// with zero gains at level 0, settle included), no perturbation pulses, no reward, and
/// plasticity frozen through `run.plasticity`, restored on every exit path. The reset to the
/// start keyframe stays. Configuration that would interfere is refused or overridden.
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
    let brain_only = run.phase == "test";
    if brain_only && !run.criterion.scheduled_events.is_empty() {
        return Err(TrainerError::Config(
            "the test phase measures the brain only and takes no scheduled events".to_string(),
        ));
    }
    if !run.criterion.scheduled_events.is_empty() && run.apply_event.is_none() {
        return Err(TrainerError::Config(
            "the criterion has scheduled events but no way to apply them (no apply_event callback)"
                .to_string(),
        ));
    }
    if brain_only {
        if run.deliver_reward.is_some() {
            return Err(TrainerError::Config(
                "the test phase measures the brain only and takes no reward sink".to_string(),
            ));
        }
        if !matches!(run.support_mode, None | Some(SupportMode::Fixed(0.0))) {
            return Err(TrainerError::Config(
                "the test phase measures the brain only and runs at support 0".to_string(),
            ));
        }
    }
    let support = run.criterion.support_config();
    let mode = run.support_mode.unwrap_or(if brain_only {
        SupportMode::Fixed(0.0)
    } else {
        SupportMode::Schedule
    });
    if let SupportMode::Fixed(level) = mode {
        if !level.is_finite() || !(0.0..=1.0).contains(&level) {
            return Err(TrainerError::Config(
                "fixed support level must be in [0, 1]".to_string(),
            ));
        }
    }
    // The adaptive assist runs only in a train phase that follows the schedule. Anywhere else
    // (test phase, fixed level) the attempt keeps the ladder's rules and ends at `hold_s`.
    let adaptive_on =
        run.criterion.is_adaptive() && !brain_only && !matches!(mode, SupportMode::Fixed(_));
    let mut test_criterion;
    let criterion: &BalanceFeetCriterion = if (brain_only
        && run.criterion.perturb_interval_s != 0.0)
        || (run.criterion.is_adaptive() && !adaptive_on)
    {
        test_criterion = run.criterion.clone();
        if brain_only {
            test_criterion.perturb_interval_s = 0.0;
        }
        test_criterion.support_mode = AssistMode::Ladder;
        &test_criterion
    } else {
        run.criterion
    };

    // Freeze learning for a test, then run, then restore on every exit path.
    let mut conditions = brain_only.then(|| BrainOnlyTest {
        support: 0.0,
        settle_support: 0.0,
        settle_s_applied: 0.0,
        reward: "none".to_string(),
        perturbation: false,
        plasticity_frozen: false,
        plasticity_reason: None,
        mappings_frozen: 0,
        memory_formation_paused: false,
    });
    let mut freeze_attempted = false;
    if let Some(conditions) = conditions.as_mut() {
        match run.plasticity.as_deref_mut() {
            Some(control) => {
                freeze_attempted = true;
                match control.freeze() {
                    Ok(outcome) => {
                        conditions.plasticity_frozen = true;
                        conditions.mappings_frozen = outcome.mappings_frozen;
                        conditions.memory_formation_paused = outcome.memory_formation_paused;
                    }
                    Err(error) => conditions.plasticity_reason = Some(error.to_string()),
                }
            }
            None => {
                conditions.plasticity_reason =
                    Some("no plasticity control was supplied".to_string());
            }
        }
    }
    let result = run_phase_attempts(
        run,
        criterion,
        &support,
        mode,
        adaptive_on,
        conditions,
        controller,
        present_goal,
        events,
        cancel,
    );
    let restored = if freeze_attempted {
        run.plasticity
            .as_deref_mut()
            .map(|control| control.restore())
    } else {
        None
    };
    match (result, restored) {
        (Ok(report), None | Some(Ok(()))) => Ok(report),
        (Ok(_), Some(Err(restore))) => Err(TrainerError::Runtime(format!(
            "test finished but learning stayed frozen: {restore}"
        ))),
        (Err(error), None | Some(Ok(()))) => Err(error),
        (Err(error), Some(Err(restore))) => Err(match error {
            TrainerError::Cancelled(message) => {
                TrainerError::Cancelled(format!("{message}; learning stayed frozen: {restore}"))
            }
            other => TrainerError::Runtime(format!("{other}; learning stayed frozen: {restore}")),
        }),
    }
}

#[allow(clippy::too_many_arguments)]
fn run_phase_attempts(
    run: &mut BalancePhaseRun<'_>,
    criterion: &BalanceFeetCriterion,
    support: &SupportConfig,
    mode: SupportMode,
    adaptive_on: bool,
    conditions: Option<BrainOnlyTest>,
    controller: &mut dyn BalanceController,
    present_goal: &mut dyn FnMut([f64; 3]) -> Result<(), TrainerError>,
    events: &mut dyn RunEventSink,
    cancel: &CancelToken,
) -> Result<BalancePhaseReport, TrainerError> {
    let brain_only = conditions.is_some();
    let info = controller.model_info()?;
    check_model(&info, criterion)?;
    let mut session = BalanceSession { info, sizing: None };
    let mut schedule = match mode {
        SupportMode::Resume(level) => SupportSchedule::resume(*support, level),
        SupportMode::ResumeAdaptive(state) => SupportSchedule::resume(*support, state.a),
        _ => SupportSchedule::new(*support),
    };
    let mut adaptive_run = adaptive_on.then(|| {
        AdaptiveRun::new(match mode {
            SupportMode::Resume(level) => AdaptiveAssist::resume(
                criterion,
                AdaptiveAssistState {
                    a: level,
                    a_slow: level,
                },
            ),
            SupportMode::ResumeAdaptive(state) => AdaptiveAssist::resume(criterion, state),
            _ => AdaptiveAssist::start(criterion),
        })
    });
    let mut dev_assist = (!brain_only).then(DeviationAssistRun::default);
    let probe_every = if brain_only { 0 } else { criterion.probe_every };
    let planned_probes = if probe_every == 0 {
        0
    } else {
        run.seeds.len() as u64 / u64::from(probe_every)
    };
    let samples_total = run.seeds.len() as u64 + planned_probes;
    // A probe has the exact conditions of a test-phase attempt: no perturbation here, support
    // 0 and no reward below, learning frozen around it.
    let mut probe_criterion = criterion.clone();
    probe_criterion.perturb_interval_s = 0.0;
    probe_criterion.support_mode = AssistMode::Ladder;
    probe_criterion.scheduled_events.clear();
    let mut attempts = Vec::with_capacity(samples_total as usize);
    let mut train_done: u32 = 0;
    let mut probes_run: u32 = 0;
    let mut probes_frozen = true;
    let mut steps_table = StepsTable { actuators: None };
    if let Some(sink) = run.data.as_deref_mut() {
        sink.declare_table(&attempts_table_spec());
        sink.event(EventTime::default(), "phase_start", run.phase);
    }
    for seed in run.seeds {
        if cancel.is_cancelled() {
            return Err(TrainerError::Cancelled(
                "balance-on-feet run cancelled".to_string(),
            ));
        }
        let level = match (&adaptive_run, mode) {
            (Some(adaptive), _) => adaptive.assist.level(),
            (None, SupportMode::Fixed(level)) => level,
            (None, _) => schedule.current_level(),
        };
        let record = run_balance_attempt(
            *seed,
            criterion,
            level,
            brain_only,
            controller,
            &mut session,
            present_goal,
            &mut run.deliver_reward,
            run.reward_policy,
            adaptive_run.as_mut(),
            dev_assist.as_mut(),
            AttemptHooks {
                data: &mut run.data,
                steps: &mut steps_table,
                apply_event: &mut run.apply_event,
                phase: run.phase,
                attempt_index: attempts.len(),
                probe: false,
            },
            cancel,
        )?;
        let stand = AttemptStand {
            balanced_s: record.balanced_s,
            support_level: record.support_level,
            outcome: outcome_name(record.outcome).to_string(),
            seed: record.seed,
            probe: false,
            train_attempts_before: None,
            adaptive: record.adaptive,
        };
        schedule.record_attempt(record.outcome, record.balanced_s);
        if let Some(sink) = run.data.as_deref_mut() {
            sink.row(
                ATTEMPTS_TABLE,
                &attempt_row(run.phase, attempts.len(), &record),
            );
        }
        attempts.push(record);
        train_done += 1;
        events.emit(progress_event(
            run,
            attempts.len() as u64,
            samples_total,
            format!("{} seed {seed} support {level:.2}", run.phase),
            stand,
        ));
        if probe_every == 0 || !train_done.is_multiple_of(probe_every) {
            continue;
        }
        // Freeze learning, run one unassisted attempt, restore on every exit path.
        let frozen = match run.plasticity.as_deref_mut() {
            Some(control) => {
                control.freeze()?;
                true
            }
            None => false,
        };
        probes_frozen &= frozen;
        let mut no_reward: Option<DeliverReward<'_>> = None;
        let probed = run_balance_attempt(
            *seed,
            &probe_criterion,
            0.0,
            true,
            controller,
            &mut session,
            present_goal,
            &mut no_reward,
            run.reward_policy,
            None,
            None,
            AttemptHooks {
                data: &mut run.data,
                steps: &mut steps_table,
                apply_event: &mut None,
                phase: run.phase,
                attempt_index: attempts.len(),
                probe: true,
            },
            cancel,
        );
        if frozen {
            if let Some(control) = run.plasticity.as_deref_mut() {
                let restored = control.restore();
                if let Err(error) = restored {
                    return Err(TrainerError::Runtime(format!(
                        "probe finished but learning stayed frozen: {error}"
                    )));
                }
            }
        }
        let mut record = probed?;
        record.probe = true;
        record.train_attempts_before = Some(train_done);
        let stand = AttemptStand {
            balanced_s: record.balanced_s,
            support_level: record.support_level,
            outcome: outcome_name(record.outcome).to_string(),
            seed: record.seed,
            probe: true,
            train_attempts_before: Some(train_done),
            adaptive: None,
        };
        if let Some(sink) = run.data.as_deref_mut() {
            sink.row(
                ATTEMPTS_TABLE,
                &attempt_row(run.phase, attempts.len(), &record),
            );
        }
        attempts.push(record);
        probes_run += 1;
        events.emit(progress_event(
            run,
            attempts.len() as u64,
            samples_total,
            format!("probe after {train_done} training attempts"),
            stand,
        ));
    }
    if let Some(sink) = run.data.as_deref_mut() {
        sink.event(
            EventTime {
                episode: Some(attempts.len() as u64),
                ..EventTime::default()
            },
            "phase_end",
            run.phase,
        );
    }
    let mut report = BalancePhaseReport::from_attempts_with(attempts, support)?;
    report.assess_over_support(criterion);
    report.probe_plasticity_frozen = (probes_run > 0).then_some(probes_frozen);
    // A brain-only test applies no spring (zero gains), so there are no sized gains to report.
    report.spring_gains = if brain_only {
        None
    } else {
        session.sizing.map(|(_, gains)| gains)
    };
    if let Some(adaptive) = adaptive_run {
        report.support_level_next = Some(adaptive.assist.level());
        report.adaptive_state_next = Some(adaptive.assist.state());
        report.assist_timeline = adaptive.timeline;
    } else if !matches!(mode, SupportMode::Fixed(_)) {
        report.support_level_next = Some(schedule.current_level());
    }
    report.deviation_assist = dev_assist.and_then(DeviationAssistRun::finish);
    report.brain_only = conditions;
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

fn progress_event(
    run: &BalancePhaseRun<'_>,
    samples_done: u64,
    samples_total: u64,
    preview_name: String,
    stand: AttemptStand,
) -> RunEvent {
    RunEvent::new(
        run.run_id.clone(),
        RunEventKind::Progress {
            samples_done,
            samples_total,
            repeat_index: 0,
            repeat_total: 1,
            sent_values: upright_goal_features().to_vec(),
            tick_index: None,
            window_values: Vec::new(),
            image_png_base64: None,
            mask_png_base64: None,
            preview_name: Some(preview_name),
            predicted_class: None,
            class_scores: Vec::new(),
            stand: Some(stand),
        },
    )
}

/// Wire name for an attempt outcome. Matches the serde name on [`BalanceAttemptOutcome`].
fn outcome_name(outcome: BalanceAttemptOutcome) -> &'static str {
    match outcome {
        BalanceAttemptOutcome::Success => "success",
        BalanceAttemptOutcome::Death => "death",
        BalanceAttemptOutcome::Interrupted => "interrupted",
        BalanceAttemptOutcome::Truncated => "truncated",
        BalanceAttemptOutcome::Fell => "fell",
        BalanceAttemptOutcome::LostStance => "lost_stance",
    }
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
        contacts_except: if criterion.only_contacts_touch {
            vec![criterion
                .contact_bodies
                .iter()
                .map(|name| name.trim().to_string())
                .collect()]
        } else {
            Vec::new()
        },
        actuators: criterion.record_actuators,
    }
}

/// One `measure` call. An older controller that does not know `a_except` fails the call; say so
/// plainly instead of surfacing its raw error.
fn measure_checked(
    controller: &mut dyn BalanceController,
    request: &MeasureRequest,
) -> Result<Measurement, TrainerError> {
    controller.measure(request).map_err(|error| {
        let text = error.to_string();
        if !request.contacts_except.is_empty() && text.to_lowercase().contains("contact") {
            TrainerError::Runtime(format!(
                "the controller rejected the \"every other body\" contact set (a_except): \
                 restart or update the controller, or turn off only_contacts_touch. \
                 Controller said: {text}"
            ))
        } else {
            error
        }
    })
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

/// The settle window an attempt really uses. The single place this is decided: a supported
/// attempt settles for `settle_s` under a full grip; an unsupported one (`brain_only`: test
/// phase and probes) has no grip to settle under, so it skips the window and scores from the
/// reset sample.
fn applied_settle_s(criterion: &BalanceFeetCriterion, brain_only: bool) -> f64 {
    if brain_only {
        0.0
    } else {
        criterion.settle_s
    }
}

/// Adaptive assist state of one phase: the assist itself (kept across attempts), the 1 Hz
/// timeline and the scored time that places its points.
struct AdaptiveRun {
    assist: AdaptiveAssist,
    timeline: Vec<AssistPoint>,
    phase_t_s: f64,
    next_point_s: f64,
}

impl AdaptiveRun {
    fn new(assist: AdaptiveAssist) -> Self {
        Self {
            assist,
            timeline: Vec::new(),
            phase_t_s: 0.0,
            next_point_s: 0.0,
        }
    }

    /// Advance the phase clock by `seconds` of scored time and keep a point once per second.
    fn advance(&mut self, seconds: f64, criterion: &BalanceFeetCriterion) {
        self.phase_t_s += seconds;
        if self.timeline.len() < ASSIST_TIMELINE_MAX_POINTS
            && self.phase_t_s + 1.0e-9 >= self.next_point_s
        {
            self.timeline.push(AssistPoint {
                t_s: self.phase_t_s,
                a: self.assist.level(),
                a_slow: self.assist.slow(),
                cone_deg: self.assist.cone_deg(criterion),
            });
            self.next_point_s = self.phase_t_s.floor() + 1.0;
        }
    }
}

/// Running numbers of one adaptive attempt.
#[derive(Default)]
struct AdaptiveAttemptTally {
    sum_a: f64,
    bursts: f64,
    min_a: Option<f64>,
    catches: u32,
    feet_down_bursts: f64,
    recenters: u32,
}

impl AdaptiveAttemptTally {
    fn add(&mut self, level: f64, bursts: u32) {
        self.sum_a += level * f64::from(bursts);
        self.bursts += f64::from(bursts);
        self.min_a = Some(self.min_a.map_or(level, |m| m.min(level)));
    }

    fn stats(
        &self,
        end_cone_deg: f64,
        burst_period_s: f64,
        standing_fraction: f64,
        lost_stance: bool,
    ) -> AdaptiveAttemptStats {
        let seconds = self.bursts * burst_period_s;
        AdaptiveAttemptStats {
            mean_a: if self.bursts > 0.0 {
                self.sum_a / self.bursts
            } else {
                0.0
            },
            min_a: self.min_a.unwrap_or(0.0),
            end_cone_deg,
            catches: self.catches,
            catches_per_min: if seconds > 0.0 {
                f64::from(self.catches) / (seconds / 60.0)
            } else {
                0.0
            },
            standing_fraction,
            feet_down_fraction: if self.bursts > 0.0 {
                self.feet_down_bursts / self.bursts
            } else {
                0.0
            },
            recenters: self.recenters,
            lost_stance,
        }
    }
}

/// Name of the per-burst table.
const STEPS_TABLE: &str = "steps";
/// Name of the per-attempt table.
const ATTEMPTS_TABLE: &str = "attempts";

/// Column names of the per-burst table that depend on the model: filled at the first sample.
struct StepsTable {
    actuators: Option<Vec<String>>,
}

fn contact_column(body: &str) -> String {
    format!("contact_{}", body.trim())
}

fn actuator_column(name: &str) -> String {
    format!("actuator_{name}")
}

fn steps_table_spec(criterion: &BalanceFeetCriterion, actuators: &[String]) -> TableSpec {
    let mut columns = vec![
        ColumnSpec::unitless("phase", "Phase the burst belongs to."),
        ColumnSpec::unitless("attempt", "Attempt index within the phase."),
        ColumnSpec::unitless("probe", "The attempt is an unassisted probe."),
        ColumnSpec::unitless("seed", "Attempt seed."),
        ColumnSpec::unitless("seq", "Controller physics-step counter."),
        ColumnSpec::with_unit("sim_time", "s", "Simulated time."),
        ColumnSpec::with_unit("tilt", "deg", "Torso tilt from world up."),
        ColumnSpec::with_unit("height", "m", "Torso height above world z."),
    ];
    columns.extend(criterion.contact_bodies.iter().map(|body| {
        ColumnSpec::unitless(&contact_column(body), "The contact body touches the world.")
    }));
    columns.extend([
        ColumnSpec::unitless(
            "other_contact_touching",
            "A body that is not a contact body touches the world.",
        ),
        ColumnSpec::unitless(
            "upright",
            "Tilt inside the limit and every contact body down.",
        ),
        ColumnSpec::unitless("settling", "The settle window is still running."),
        ColumnSpec::unitless("perturbing", "A perturbation pulse is applied."),
        ColumnSpec::unitless("terminal", "Latched end cause, when one was latched."),
        ColumnSpec::unitless("support_level", "Effective support level, 0 to 1."),
        ColumnSpec::unitless("support_need", "Predicted tilt fraction, 0 to 1."),
        ColumnSpec::unitless("support_effort", "Spring effort, 0 to 1."),
        ColumnSpec::with_unit("support_torque", "N*m", "Spring torque norm."),
        ColumnSpec::with_unit("support_force", "N", "Spring force norm."),
        ColumnSpec::unitless("assist_slow", "Slow average of the adaptive assist level."),
        ColumnSpec::unitless(
            "assist_rise",
            "Adaptive assist rise over the previous burst.",
        ),
    ]);
    columns.extend(
        actuators
            .iter()
            .map(|name| ColumnSpec::unitless(&actuator_column(name), "Actuator command (ctrl).")),
    );
    TableSpec {
        name: STEPS_TABLE.to_string(),
        description: "One row per counted burst.".to_string(),
        columns,
    }
}

fn attempts_table_spec() -> TableSpec {
    TableSpec {
        name: ATTEMPTS_TABLE.to_string(),
        description: "One row per finished attempt, probes included.".to_string(),
        columns: vec![
            ColumnSpec::unitless("phase", "Phase the attempt belongs to."),
            ColumnSpec::unitless("attempt", "Attempt index within the phase."),
            ColumnSpec::unitless("probe", "The attempt is an unassisted probe."),
            ColumnSpec::unitless("seed", "Attempt seed."),
            ColumnSpec::unitless("outcome", "How the attempt ended."),
            ColumnSpec::unitless("bursts", "Non-settling bursts observed."),
            ColumnSpec::with_unit("final_tilt", "deg", "Torso tilt at the terminal burst."),
            ColumnSpec::unitless("support_level", "Scheduled support level."),
            ColumnSpec::unitless("upright_fraction", "Share of bursts that were upright."),
            ColumnSpec::unitless("perturbations", "Perturbation pulses seen."),
            ColumnSpec::unitless("balanced_bursts", "Longest run of upright bursts."),
            ColumnSpec::with_unit("balanced_time", "s", "Longest upright run."),
            ColumnSpec::with_unit(
                "time_to_first_violation",
                "s",
                "Time before the first non-upright burst, when there was one.",
            ),
            ColumnSpec::unitless("carried_fraction", "Share of bursts the support carried."),
            ColumnSpec::unitless(
                "over_assist_fraction",
                "Share of bursts with support above the predicted need.",
            ),
        ],
    }
}

fn attempt_row(phase: &str, index: usize, record: &BalanceAttemptRecord) -> Vec<RunDataValue> {
    vec![
        RunDataValue::text(phase),
        RunDataValue::count(index as u64),
        RunDataValue::Bool(record.probe),
        RunDataValue::count(record.seed),
        RunDataValue::text(outcome_name(record.outcome)),
        RunDataValue::count(u64::from(record.bursts)),
        RunDataValue::Float(record.final_tilt_deg),
        RunDataValue::Float(record.support_level),
        RunDataValue::Float(record.upright_fraction),
        RunDataValue::count(u64::from(record.perturbations)),
        RunDataValue::count(u64::from(record.balanced_bursts)),
        RunDataValue::Float(record.balanced_s),
        RunDataValue::opt_float(record.time_to_first_violation_s),
        RunDataValue::Float(record.carried_fraction),
        RunDataValue::Float(record.over_assist_fraction),
    ]
}

/// What one attempt writes to, and the callback it applies events through.
struct AttemptHooks<'r, 'a> {
    data: &'r mut Option<&'a mut dyn RunDataSink>,
    steps: &'r mut StepsTable,
    apply_event: &'r mut Option<ApplyEvent<'a>>,
    phase: &'r str,
    attempt_index: usize,
    probe: bool,
}

impl AttemptHooks<'_, '_> {
    fn marker(&mut self, sim_time_s: f64, kind: &str, label: &str) {
        let at = EventTime {
            sim_time_s: Some(sim_time_s),
            step: None,
            episode: Some(self.attempt_index as u64),
        };
        if let Some(sink) = self.data.as_deref_mut() {
            sink.event(at, kind, label);
        }
    }

    fn sample(
        &mut self,
        sample: &BalanceSample,
        sim_time_s: f64,
        criterion: &BalanceFeetCriterion,
    ) {
        let Some(sink) = self.data.as_deref_mut() else {
            return;
        };
        let actuators = self.steps.actuators.get_or_insert_with(|| {
            let names: Vec<String> = sample.actuators.keys().cloned().collect();
            sink.declare_table(&steps_table_spec(criterion, &names));
            names
        });
        let mut row = vec![
            RunDataValue::text(self.phase),
            RunDataValue::count(self.attempt_index as u64),
            RunDataValue::Bool(self.probe),
            RunDataValue::count(sample.seed),
            RunDataValue::count(sample.seq),
            RunDataValue::Float(sim_time_s),
            RunDataValue::Float(sample.tilt_deg),
            RunDataValue::opt_float(sample.torso_height_m),
        ];
        row.extend((0..criterion.contact_bodies.len()).map(|i| {
            sample
                .contacts
                .get(i)
                .map_or(RunDataValue::Null, |c| RunDataValue::Bool(*c))
        }));
        row.extend([
            RunDataValue::Bool(sample.other_contact_touching),
            RunDataValue::Bool(sample_is_upright(sample, criterion)),
            RunDataValue::Bool(sample.settling),
            RunDataValue::Bool(sample.perturbing),
            sample
                .terminal
                .as_ref()
                .map_or(RunDataValue::Null, |t| RunDataValue::Text(name_of(t))),
            RunDataValue::opt_float(sample.support_level),
            RunDataValue::opt_float(sample.support_need),
            RunDataValue::opt_float(sample.support_effort),
            RunDataValue::opt_float(sample.support_torque_nm),
            RunDataValue::opt_float(sample.support_force_n),
            RunDataValue::opt_float(sample.assist_slow),
            RunDataValue::opt_float(sample.assist_rise),
        ]);
        row.extend(actuators.iter().map(|name| {
            sample
                .actuators
                .get(name)
                .map_or(RunDataValue::Null, |v| RunDataValue::Float(*v))
        }));
        sink.row(STEPS_TABLE, &row);
    }
}

#[allow(clippy::too_many_arguments)]
fn run_balance_attempt(
    seed: u64,
    criterion: &BalanceFeetCriterion,
    support_level: f64,
    brain_only: bool,
    controller: &mut dyn BalanceController,
    session: &mut BalanceSession,
    present_goal: &mut dyn FnMut([f64; 3]) -> Result<(), TrainerError>,
    deliver_reward: &mut Option<DeliverReward<'_>>,
    reward_policy: BalanceRewardPolicy,
    mut adaptive: Option<&mut AdaptiveRun>,
    mut dev_assist: Option<&mut DeviationAssistRun>,
    mut hooks: AttemptHooks<'_, '_>,
    cancel: &CancelToken,
) -> Result<BalanceAttemptRecord, TrainerError> {
    let torso = criterion.torso_body.trim().to_string();
    let keyframe = Some(criterion.reset_keyframe.trim()).filter(|name| !name.is_empty());
    controller.reset_episode(keyframe)?;
    let request = measure_request(criterion);
    let first = measure_checked(controller, &request)?;
    hooks.marker(first.sim_time_s, "attempt_start", &format!("seed {seed}"));
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
    // A brain-only attempt never supports the body: the spring exists (so measure reports its
    // wrench) but has zero gains, no gravity compensation and level 0 from reset to the end.
    let settle_s = applied_settle_s(criterion, brain_only);
    let settle_level = if brain_only {
        0.0
    } else if settle_s > 0.0 {
        1.0
    } else {
        support_level
    };
    let (gains, deadband_rad) = if brain_only {
        (
            SpringGains {
                k_rot: 0.0,
                d_rot: 0.0,
                gravity_comp: 0.0,
            },
            0.0,
        )
    } else {
        (gains, criterion.support_deadband_deg.to_radians())
    };
    controller.set_body_spring(&torso, &gains, settle_level, deadband_rad)?;
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
    let mut spring_cone_deg = criterion.support_deadband_deg;
    let mut tally = AdaptiveAttemptTally::default();
    // Adaptive only: the spring starts with the criterion's weight support; `recentering` drops
    // the cone to `cone_min_deg` after a lean that outlasted `lean_grace_s`.
    let mut spring_gravity_on = true;
    let mut recentering = false;
    let mut lean_s = 0.0_f64;
    // Settle is only the opening window. A death homes the body and rewinds sim
    // time to the keyframe, so a later reading must not be labeled as settle.
    let mut past_settle = settle_s <= 0.0;
    // Scheduled events fire in time order, once per attempt; a brain-only attempt has none.
    let mut due_events: Vec<&ScheduledEvent> = if brain_only {
        Vec::new()
    } else {
        criterion.scheduled_events.iter().collect()
    };
    due_events.sort_by(|a, b| a.at_s.total_cmp(&b.at_s));
    let mut next_event = 0usize;
    let mut was_perturbing = false;
    let mut last_sim_time_s: f64;
    let mut result = loop {
        if cancel.is_cancelled() {
            let _ = controller.clear_body_spring(&torso);
            return Err(TrainerError::Cancelled(
                "balance-on-feet run cancelled".to_string(),
            ));
        }
        present_goal(features)?;
        let reading = measure_checked(controller, &request)?;
        let elapsed_s = reading.sim_time_s - t0;
        last_sim_time_s = reading.sim_time_s;
        let settling = !past_settle && elapsed_s + 1.0e-9 < settle_s;
        if !settling && !past_settle {
            past_settle = true;
            hooks.marker(reading.sim_time_s, "settle_end", "");
        }
        let level_now = if brain_only {
            0.0
        } else if settling {
            1.0
        } else if let Some(run) = adaptive.as_deref() {
            run.assist.level()
        } else {
            support_level
        };
        let body = reading
            .bodies
            .get(&torso)
            .ok_or_else(|| TrainerError::Runtime(format!("measure reply has no body {torso}")))?;
        let contact_count = criterion.contact_bodies.len();
        if reading.contacts.len() != request.contacts.len() + request.contacts_except.len() {
            return Err(TrainerError::Runtime(format!(
                "measure returned {} contact sets for {} requested{}",
                reading.contacts.len(),
                request.contacts.len() + request.contacts_except.len(),
                if request.contacts_except.is_empty() {
                    ""
                } else {
                    " (the controller may predate a_except: restart or update it)"
                }
            )));
        }
        let other = reading.contacts.get(contact_count);
        let feet_down = reading
            .contacts
            .iter()
            .take(contact_count)
            .all(|set| set.touching);
        let mut perturbing = false;
        if reading.terminal.is_none() {
            if let Some(run) = adaptive.as_deref() {
                // Level, cone and weight support are sent only when they changed enough to matter.
                let cone_now = if settling {
                    criterion.support_deadband_deg
                } else if recentering {
                    criterion.cone_min_deg
                } else {
                    run.assist.cone_deg(criterion)
                };
                // Weight support only while the feet are loaded: lifting a foot drops it to 0, and
                // the criterion's fraction comes back once every contact body is down again.
                let gravity_want = settling || feet_down;
                let gravity_changed = gravity_want != spring_gravity_on;
                if (spring_level - level_now).abs() > 0.005
                    || (spring_cone_deg - cone_now).abs() > 0.1
                    || gravity_changed
                {
                    controller.update_body_spring(
                        &torso,
                        level_now,
                        Some(cone_now.to_radians()),
                        gravity_changed.then_some(if gravity_want {
                            criterion.bws_max_fraction
                        } else {
                            0.0
                        }),
                    )?;
                    spring_level = level_now;
                    spring_cone_deg = cone_now;
                    spring_gravity_on = gravity_want;
                }
            } else if (spring_level - level_now).abs() > 1.0e-12 {
                controller.update_body_spring(&torso, level_now, None, None)?;
                spring_level = level_now;
            }
            if !settling {
                let post_settle_s = elapsed_s - settle_s;
                if let Some(torque) = perturb.pulse_at(post_settle_s) {
                    controller.apply_body_pulse(
                        &torso,
                        torque,
                        [0.0; 3],
                        criterion.perturb_duration_s,
                    )?;
                    hooks.marker(reading.sim_time_s, "perturb_start", "");
                }
                perturbing = perturb.active_at(post_settle_s);
                while let Some(event) = due_events.get(next_event) {
                    if event.at_s > post_settle_s + 1.0e-9 {
                        break;
                    }
                    let apply = hooks.apply_event.as_deref_mut().ok_or_else(|| {
                        TrainerError::Config(
                            "scheduled events need an apply_event callback".to_string(),
                        )
                    })?;
                    apply(&event.action)?;
                    hooks.marker(reading.sim_time_s, "scheduled_event", &event.label);
                    next_event += 1;
                }
            }
            if was_perturbing && !perturbing {
                hooks.marker(reading.sim_time_s, "perturb_end", "");
            }
            was_perturbing = perturbing;
        }
        let wrench = reading.springs.get(&torso).copied();
        let mut sample = BalanceSample {
            seq: reading.physics_steps,
            seed,
            tilt_deg: body.tilt_rad.to_degrees(),
            contacts: reading
                .contacts
                .iter()
                .take(contact_count)
                .map(|set| set.touching)
                .collect(),
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
            other_contact_touching: other.is_some_and(|set| set.touching),
            other_contact_bodies: other.map(|set| set.bodies.clone()).unwrap_or_default(),
            torso_height_m: Some(body.com[2]),
            reset_height_m: open.reset_height_m,
            assist_slow: None,
            assist_rise: None,
            actuators: reading.actuators.clone(),
        };
        let step = observe_balance_sample(&open, &sample, criterion, seed)?;
        let (counted, outcome) = match &step {
            BalanceAttemptStep::Continue(next) => (next.counted_burst, None),
            BalanceAttemptStep::Finished(record) => (true, Some(record.outcome)),
        };
        if let Some(run) = dev_assist.as_deref_mut() {
            if counted && !settling {
                let new_bursts = match &step {
                    BalanceAttemptStep::Continue(next) => next.bursts,
                    BalanceAttemptStep::Finished(record) => record.bursts,
                }
                .saturating_sub(open.bursts)
                .max(1);
                let reset_height_m = open
                    .reset_height_m
                    .or(sample.reset_height_m)
                    .or(sample.torso_height_m);
                run.add(
                    goal_deviation(&sample, criterion, reset_height_m),
                    level_now,
                    new_bursts,
                    criterion.burst_period_s,
                );
            }
        }
        if let Some(run) = adaptive.as_deref_mut() {
            if counted && !settling {
                let new_bursts = match &step {
                    BalanceAttemptStep::Continue(next) => next.bursts,
                    BalanceAttemptStep::Finished(record) => record.bursts,
                }
                .saturating_sub(open.bursts)
                .max(1);
                let mut rise = 0.0;
                if matches!(step, BalanceAttemptStep::Continue(_)) && reading.terminal.is_none() {
                    let predicted =
                        predicted_tilt_deg(body.tilt_rad, body.ang_vel, criterion.need_lookahead_s);
                    if recentering {
                        if predicted <= criterion.cone_min_deg {
                            recentering = false;
                            lean_s = 0.0;
                        }
                    } else if predicted > run.assist.cone_deg(criterion) {
                        lean_s += f64::from(new_bursts) * criterion.burst_period_s;
                        if lean_s > criterion.lean_grace_s + 1.0e-9 {
                            recentering = true;
                            lean_s = 0.0;
                            tally.recenters += 1;
                        }
                    } else {
                        lean_s = 0.0;
                    }
                    let moved = run.assist.step(criterion, predicted, new_bursts);
                    rise = moved.rise;
                    tally.catches += u32::from(moved.catch);
                }
                tally.add(run.assist.level(), new_bursts);
                if feet_down {
                    tally.feet_down_bursts += f64::from(new_bursts);
                }
                run.advance(f64::from(new_bursts) * criterion.burst_period_s, criterion);
                sample.support_level = Some(run.assist.level());
                sample.assist_slow = Some(run.assist.slow());
                sample.assist_rise = Some(rise);
            }
        }
        if counted {
            hooks.sample(&sample, reading.sim_time_s, criterion);
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
    if let Some(run) = adaptive.as_deref() {
        result.adaptive = Some(tally.stats(
            run.assist.cone_deg(criterion),
            criterion.burst_period_s,
            result.upright_fraction,
            result.outcome == BalanceAttemptOutcome::LostStance,
        ));
    }
    if was_perturbing {
        hooks.marker(last_sim_time_s, "perturb_end", "");
    }
    hooks.marker(last_sim_time_s, "attempt_end", outcome_name(result.outcome));
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

/// JSON body of `set_body_spring`. Only the anti-tilt torque gains and the weight-support
/// fraction (`gravity_comp`) are sent; `k_lin` and `d_lin` are omitted, so the controller
/// applies no position spring.
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
        "gravity_comp": gains.gravity_comp,
        "level": level,
        "deadband_rad": deadband_rad,
        "ref": "current",
    })
}

/// JSON body of `update_body_spring`. `deadband_rad` and `gravity_comp` are sent only when
/// `Some`.
pub fn update_body_spring_request(
    body: &str,
    level: f64,
    deadband_rad: Option<f64>,
    gravity_comp: Option<f64>,
) -> serde_json::Value {
    let mut request =
        serde_json::json!({"action": "update_body_spring", "body": body, "level": level});
    if let Some(deadband) = deadband_rad {
        request["deadband_rad"] = serde_json::json!(deadband);
    }
    if let Some(fraction) = gravity_comp {
        request["gravity_comp"] = serde_json::json!(fraction);
    }
    request
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
        .chain(
            request
                .contacts_except
                .iter()
                .map(|set| serde_json::json!({"a_except": set, "b": "world"})),
        )
        .collect();
    let mut body = serde_json::json!({
        "action": "measure",
        "bodies": request.bodies,
        "contacts": contacts,
    });
    if request.actuators {
        body["actuators"] = serde_json::Value::Bool(true);
    }
    body
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

    fn update_body_spring(
        &mut self,
        body: &str,
        level: f64,
        deadband_rad: Option<f64>,
        gravity_comp: Option<f64>,
    ) -> Result<(), TrainerError> {
        self.command(&update_body_spring_request(
            body,
            level,
            deadband_rad,
            gravity_comp,
        ))
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
    use crate::run_actions::ScheduledAction;
    use crate::control::CollectingEventSink;

    /// One burst is 0.05 s; hold 0.1 s = 2 bursts, cap 0.2 s = 4 bursts. One physics step per
    /// burst so legacy tests stay readable.
    fn criterion() -> BalanceFeetCriterion {
        let mut c = BalanceFeetCriterion::new(
            "torso",
            vec!["contact_a".to_string(), "contact_b".to_string()],
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
            other_contact_touching: false,
            other_contact_bodies: Vec::new(),
            torso_height_m: None,
            reset_height_m: None,
            assist_slow: None,
            assist_rise: None,
            actuators: BTreeMap::new(),
        }
    }

    fn fall_criterion() -> BalanceFeetCriterion {
        let mut c = criterion();
        c.hold_s = 5.0;
        c.attempt_cap_s = 20.0;
        c
    }

    #[test]
    fn fall_tilt_past_confirm_ends_as_fell() {
        let c = fall_criterion();
        assert_eq!(c.fall_confirm_bursts(), 5);
        let mut open = OpenBalanceAttempt::fresh();
        open = fed(&open, &balanced(1, 1), &c);
        for seq in 2..=5 {
            open = fed(&open, &sample(seq, 1, 70.0), &c);
        }
        assert_eq!(open.fall_bursts, 4);
        match observe_balance_sample(&open, &sample(6, 1, 80.0), &c, 1).expect("obs") {
            BalanceAttemptStep::Finished(r) => {
                assert_eq!(r.outcome, BalanceAttemptOutcome::Fell);
                assert_eq!(r.bursts, 6);
                assert_eq!(r.final_tilt_deg, 80.0);
                assert_eq!(r.balanced_bursts, 1);
            }
            BalanceAttemptStep::Continue(_) => panic!("5 fallen bursts confirm the fall"),
        }
    }

    #[test]
    fn fall_tilt_dip_before_confirm_does_not_end() {
        let c = fall_criterion();
        let mut open = OpenBalanceAttempt::fresh();
        for seq in 1..=4 {
            open = fed(&open, &sample(seq, 1, 70.0), &c);
        }
        open = fed(&open, &sample(5, 1, 40.0), &c);
        assert_eq!(open.fall_bursts, 0);
        for seq in 6..=9 {
            open = fed(&open, &sample(seq, 1, 70.0), &c);
        }
        assert_eq!(open.fall_bursts, 4);
    }

    #[test]
    fn fall_tilt_zero_never_ends_as_fell() {
        let mut c = fall_criterion();
        c.fall_tilt_deg = 0.0;
        let mut open = OpenBalanceAttempt::fresh();
        for seq in 1..=30 {
            open = fed(&open, &sample(seq, 1, 120.0), &c);
        }
        assert_eq!(open.fall_bursts, 0);
        assert!(c.validate().is_ok());
    }

    #[test]
    fn fall_fields_validate() {
        let mut c = fall_criterion();
        c.fall_tilt_deg = 15.0;
        assert!(c.validate().is_err());
        c.fall_tilt_deg = 181.0;
        assert!(c.validate().is_err());
        c.fall_tilt_deg = 60.0;
        c.fall_confirm_s = -0.1;
        assert!(c.validate().is_err());
        c.fall_confirm_s = 0.0;
        assert!(c.validate().is_ok());
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
            probe: false,
            train_attempts_before: None,
            non_upright: NonUprightCounts::default(),
            carried_fraction: 0.0,
            over_assist_fraction: 0.0,
            adaptive: None,
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
    }

    #[test]
    fn resume_keeps_the_ceiling_and_starts_at_the_saved_level() {
        let resumed = SupportSchedule::resume(cfg(), 0.2);
        assert!((resumed.current_level() - 0.2).abs() < 1e-9);
        let mut raised = resumed;
        feed(&mut raised, &[Death; 3]);
        feed(&mut raised, &[Death; 3]);
        assert!(
            (raised.current_level() - 0.3).abs() < 1e-9,
            "two failed blocks raise one step"
        );
        for _ in 0..20 {
            feed(&mut raised, &[Death; 3]);
        }
        assert!(
            (raised.current_level() - 1.0).abs() < 1e-9,
            "the original start stays the ceiling"
        );
        assert!((SupportSchedule::resume(cfg(), 5.0).current_level() - 1.0).abs() < 1e-9);
        assert_eq!(SupportSchedule::resume(cfg(), -1.0).current_level(), 0.0);
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

    fn assist_tilt(
        r: &mut BalanceAssistReward,
        tilt: f64,
        outcome: Option<BalanceAttemptOutcome>,
    ) -> Vec<(AffectChannel, f64)> {
        r.on_sample(
            &need_sample(None, tilt, false),
            outcome.as_ref(),
            &criterion(),
        )
        .into_iter()
        .map(|s| (s.channel, s.magnitude))
        .collect()
    }

    #[test]
    fn assist_reward_holds_inside_the_success_limit_and_pains_past_it() {
        let p = AffectChannel::Pain;
        let g = AffectChannel::Pleasure;
        let mut r = BalanceAssistReward::new();
        // 3 deg is inside the 15 deg success limit: holding Pleasure, not Pain.
        assert_eq!(assist_tilt(&mut r, 3.0, None), vec![(g, 0.2)]);
        // A wobble that stays inside the limit is still a hold.
        assert_eq!(assist_tilt(&mut r, 8.0, None), vec![(g, 0.2)]);
        // Crossing the limit is Pain. 18/15 clamps to 1.
        assert_eq!(assist_tilt(&mut r, 18.0, None), vec![(p, 1.0)]);
        // Steady past the limit: nothing.
        assert!(assist_tilt(&mut r, 18.0, None).is_empty());
        // Back inside: fall Pleasure (need 1 -> 0.8, gain 4) wins over the hold.
        let recovered = assist_tilt(&mut r, 12.0, None);
        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0].0, g);
        assert!((recovered[0].1 - 0.8).abs() < 1e-9);
        assert_eq!(assist_tilt(&mut r, 12.0, None), vec![(g, 0.2)]);
        assert_eq!(
            assist_tilt(&mut r, 3.0, Some(BalanceAttemptOutcome::Success)),
            vec![(g, 1.0)]
        );
        assert_eq!(
            assist_tilt(&mut r, 3.0, Some(BalanceAttemptOutcome::Death)),
            vec![(p, 1.0)]
        );
    }

    #[test]
    fn assist_reward_edge_cases() {
        let c = criterion();
        let mut r = BalanceAssistReward::new();
        // Settling is silent, including a death. The need still seeds the next burst.
        assert!(r
            .on_sample(&need_sample(Some(0.9), 0.0, true), None, &c)
            .is_empty());
        assert!(r
            .on_sample(
                &need_sample(Some(0.9), 0.0, true),
                Some(&BalanceAttemptOutcome::Death),
                &c,
            )
            .is_empty());
        // Settle seeds need 0.9. The next burst stays at that need and inside the limit.
        let next = r.on_sample(&need_sample(Some(0.9), 3.0, false), None, &c);
        assert_eq!(next.len(), 1);
        assert_eq!(next[0].channel, AffectChannel::Pleasure);
        assert!((next[0].magnitude - ASSIST_HOLD_PLEASURE).abs() < 1e-9);
        // Past the limit after that seed: Pain.
        let past = r.on_sample(&need_sample(None, 18.0, false), None, &c);
        assert_eq!(past.len(), 1);
        assert_eq!(past[0].channel, AffectChannel::Pain);
        assert!((past[0].magnitude - 1.0).abs() < 1e-9);
        // Feet up inside the limit: no holding Pleasure.
        let mut r = BalanceAssistReward::new();
        let mut up = need_sample(Some(0.0), 3.0, false);
        up.contacts[0] = false;
        assert!(r.on_sample(&up, None, &c).is_empty());
        assert_eq!(BalanceAssistReward::policy_id(), "reward.balance-assist.v2");
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
    fn graduate_balanced_s_above_hold_or_cap_is_rejected() {
        let mut c = criterion();
        c.hold_s = 3.0;
        c.attempt_cap_s = 6.0;
        c.graduate_balanced_s = Some(10.0);
        let err = c.validate().unwrap_err().to_string();
        assert!(err.contains("graduate_balanced_s"), "{err}");
        c.graduate_balanced_s = Some(3.5);
        assert!(c.validate().is_err(), "above hold_s");
        c.graduate_balanced_s = Some(3.0);
        assert!(c.validate().is_ok(), "equal to hold_s is allowed");
        c.graduate_balanced_s = None;
        assert!(c.validate().is_ok());
    }

    #[test]
    fn two_failed_blocks_raise_support_after_attempts_per_level_each() {
        // Documented rule: a failed block holds the level; two consecutive failed blocks raise it.
        let mut config = cfg();
        config.attempts_per_level = 2;
        config.start = 1.0;
        config.step = 0.5;
        let mut s = SupportSchedule::resume(config, 0.5);
        let mut levels = Vec::new();
        for _ in 0..6 {
            levels.push(s.current_level());
            s.record(BalanceAttemptOutcome::Death);
        }
        assert_eq!(levels, vec![0.5, 0.5, 0.5, 0.5, 1.0, 1.0]);
    }

    // ---- mock controller: the generic physics primitives of the contract ----

    #[derive(Debug, Clone, Copy)]
    struct Scripted {
        tilt_deg: f64,
        down: bool,
        terminal: Option<BalanceTerminal>,
        /// Replaces the mock clock. A death reset reports the keyframe time.
        sim_time_s: Option<f64>,
    }

    fn upright_step() -> Scripted {
        Scripted {
            tilt_deg: 2.0,
            down: true,
            terminal: None,
            sim_time_s: None,
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
        /// Deadband (rad) sent with each level update, in the same order.
        deadband_updates: Vec<Option<f64>>,
        /// (gravity_comp, seconds since reset when it was sent)
        gravity_updates: Vec<(f64, f64)>,
        /// (torque, force, duration_s, seconds since reset)
        pulses: Vec<([f64; 3], [f64; 3], f64, f64)>,
        measure_requests: Vec<MeasureRequest>,
        /// Non-contact bodies the extra contact set reports as touching.
        other_touching: Vec<String>,
        /// From this many measures after a reset, the torso CoM z is the second value.
        torso_z_drop: Option<(u32, f64)>,
        /// The controller predates `a_except` and rejects it.
        rejects_a_except: bool,
        spring_level: Option<f64>,
        k: u32,
        fresh: bool,
        steps: u64,
    }

    /// Generic test body: the free root "base" carries the whole 1 kg; "torso" is a child body
    /// and three light contact bodies hang off the root. Round numbers, no real model.
    fn generic_body_info() -> ModelInfo {
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
                body("base", "base", 1.0, [2e-3, 2e-3, 1e-3]),
                body("torso", "base", 0.6, [2e-3, 2e-3, 1e-3]),
                body("contact_a", "base", 0.05, [1e-5; 3]),
                body("contact_b", "base", 0.05, [1e-5; 3]),
                body("contact_c", "base", 0.05, [1e-5; 3]),
            ],
            free_roots: vec!["base".to_string()],
            keyframes: vec![serde_json::json!("home"), serde_json::json!("stand")],
        }
    }

    impl MockController {
        fn new(advance_s: f64, contact_count: usize, script: Script) -> Self {
            Self {
                info: generic_body_info(),
                advance_s,
                script,
                contact_count,
                floor_z: Some(0.01),
                calls: Vec::new(),
                resets: Vec::new(),
                spring_sets: Vec::new(),
                level_updates: Vec::new(),
                deadband_updates: Vec::new(),
                gravity_updates: Vec::new(),
                pulses: Vec::new(),
                measure_requests: Vec::new(),
                other_touching: Vec::new(),
                torso_z_drop: None,
                rejects_a_except: false,
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

        fn update_body_spring(
            &mut self,
            _body: &str,
            level: f64,
            deadband_rad: Option<f64>,
            gravity_comp: Option<f64>,
        ) -> Result<(), TrainerError> {
            self.calls.push("update_body_spring".to_string());
            self.level_updates.push((level, self.seconds()));
            self.deadband_updates.push(deadband_rad);
            if let Some(fraction) = gravity_comp {
                self.gravity_updates.push((fraction, self.seconds()));
            }
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
            if self.rejects_a_except && !request.contacts_except.is_empty() {
                return Err(TrainerError::Runtime(
                    "unknown contact key a_except".to_string(),
                ));
            }
            if self.fresh {
                self.fresh = false;
            } else {
                self.k += 1;
                self.steps = (self.seconds() / self.info.timestep_s).round() as u64;
            }
            let (k, seconds) = (self.k, self.seconds());
            let scripted = (self.script)(k, seconds);
            let touching_z = self.floor_z.filter(|_| scripted.down);
            let mut contacts: Vec<ContactMeasure> = (0..self.contact_count)
                .map(|_| ContactMeasure {
                    touching: scripted.down,
                    points: touching_z.map(|z| vec![[0.0, 0.0, z]]).unwrap_or_default(),
                    bodies: Vec::new(),
                    normal_force_n: None,
                })
                .collect();
            contacts.extend(request.contacts_except.iter().map(|_| ContactMeasure {
                touching: !self.other_touching.is_empty(),
                points: Vec::new(),
                bodies: self.other_touching.clone(),
                normal_force_n: None,
            }));
            let torso_z = match self.torso_z_drop {
                Some((from, z)) if k >= from => z,
                _ => 0.1,
            };
            let mut bodies = BTreeMap::new();
            bodies.insert(
                request.bodies[0].clone(),
                BodyMeasure {
                    tilt_rad: scripted.tilt_deg.to_radians(),
                    tilt_vec: [0.0, 0.0],
                    ang_vel: [0.0; 3],
                    com: [0.0, 0.0, torso_z],
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
                        gravity_comp_force_n: level * 1.0 * 9.81,
                    },
                );
            }
            Ok(Measurement {
                sim_time_s: scripted.sim_time_s.unwrap_or_else(|| self.seconds()),
                physics_steps: self.steps,
                episode_id: self.resets.len() as u64,
                terminal: scripted.terminal,
                bodies,
                contacts,
                springs,
                actuators: if request.actuators {
                    BTreeMap::from([("joint_a".to_string(), f64::from(k))])
                } else {
                    BTreeMap::new()
                },
            })
        }
    }

    fn contact_names(count: usize) -> Vec<String> {
        ["contact_a", "contact_b", "contact_c"][..count]
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
            plasticity: None,
            reward_policy: BalanceRewardPolicy::Affect,
            data: None,
            apply_event: None,
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
    fn gain_sizing_follows_the_documented_formulas() {
        let info = generic_body_info();
        let inputs = sizing_inputs(&info, "torso", 0.140).expect("inputs");
        assert!((inputs.mass_kg - 1.0).abs() < 1e-12);
        assert!((inputs.i_tilt - 5e-3).abs() < 1e-12);
        assert!((inputs.i_min - 1.3e-3).abs() < 1e-12);
        let g = size_spring_gains(&inputs);
        let k_expected = 3.0 * 1.0 * 9.81 * 0.140 / 0.087;
        assert!((g.k_rot - k_expected).abs() < 1e-9, "k_rot {}", g.k_rot);
        let d_expected = 0.8 * (k_expected * 5e-3_f64).sqrt();
        assert!((g.d_rot - d_expected).abs() < 1e-9, "d_rot {}", g.d_rot);
        assert_eq!(g.gravity_comp, 0.5);
    }

    #[test]
    fn sizing_returns_no_linear_springs() {
        let inputs = sizing_inputs(&generic_body_info(), "torso", 0.140).expect("inputs");
        let g = size_spring_gains(&inputs);
        let sent = set_body_spring_request("body", &g, 1.0, 0.0);
        assert!(sent.get("k_lin").is_none(), "{sent}");
        assert!(sent.get("d_lin").is_none(), "{sent}");
        assert_eq!(sent["gravity_comp"], 0.5);
    }

    #[test]
    fn weight_support_force_is_level_fraction_mass_gravity() {
        let inputs = sizing_inputs(&generic_body_info(), "torso", 0.140).expect("inputs");
        let mut c = criterion_for(2, 0.05);
        for (fraction, level) in [(0.5, 1.0), (0.3, 0.6), (0.0, 1.0)] {
            c.bws_max_fraction = fraction;
            let g = gains_for(&inputs, &c);
            let force = level * g.gravity_comp * inputs.mass_kg * inputs.gravity;
            let expected = level * fraction * inputs.mass_kg * inputs.gravity;
            assert!((force - expected).abs() < 1e-12, "{force} vs {expected}");
        }
    }

    #[test]
    fn bws_fraction_is_validated_and_defaults_to_half() {
        let mut c = criterion_for(2, 0.05);
        assert_eq!(c.bws_max_fraction, 0.5);
        for ok in [0.0, 0.25, 0.5] {
            c.bws_max_fraction = ok;
            assert!(c.validate().is_ok(), "{ok}");
        }
        for bad in [-0.01, 0.51, 1.0, f64::NAN, f64::INFINITY] {
            c.bws_max_fraction = bad;
            assert!(c.validate().is_err(), "{bad}");
        }
        let parsed: BalanceFeetCriterion = serde_json::from_value(serde_json::json!({
            "torso_body": "t", "contact_bodies": ["a"],
            "max_tilt_deg": 10.0, "hold_s": 1.0, "attempt_cap_s": 2.0
        }))
        .expect("minimal criterion");
        assert_eq!(parsed.bws_max_fraction, 0.5);
    }

    #[test]
    fn d_rot_is_capped_for_explicit_force_stability() {
        let mut info = generic_body_info();
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
        let mut info = generic_body_info();
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
        let inputs = sizing_inputs(&generic_body_info(), "torso", 0.140).expect("inputs");
        let mut c = criterion_for(2, 0.05);
        let sized = gains_for(&inputs, &c);
        assert_eq!(sized, size_spring_gains(&inputs));
        c.support_rot_stiffness_nm_per_rad = Some(50.0);
        c.bws_max_fraction = 0.4;
        let g = gains_for(&inputs, &c);
        assert_eq!(g.k_rot, 50.0);
        assert!(
            (g.d_rot - 0.8 * (50.0_f64 * 5e-3).sqrt()).abs() < 1e-9,
            "k feeds d"
        );
        assert_eq!(g.gravity_comp, 0.4);
        c.support_rot_damping_nm_s_per_rad = Some(0.11);
        assert_eq!(gains_for(&inputs, &c).d_rot, 0.11);
    }

    #[test]
    fn sizing_names_what_is_missing() {
        let mut info = generic_body_info();
        info.bodies[0].subtree_mass = 0.0;
        let error = sizing_inputs(&info, "torso", 0.140)
            .unwrap_err()
            .to_string();
        assert!(error.contains("root subtree mass"), "{error}");
        let error = sizing_inputs(&generic_body_info(), "nope", 0.140)
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
            "0.5 s hold at 20 Hz, scored from the first sample"
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
        // brain-only: the spring is set once with zero gains at level 0, settle included
        assert_eq!(mock.spring_sets.len(), 1);
        assert_eq!(mock.spring_sets[0].1, 0.0);
        assert_eq!(mock.spring_sets[0].2, 0.0);
        assert_eq!(mock.spring_sets[0].0, zero_gains());
        assert!(mock.level_updates.is_empty(), "{:?}", mock.level_updates);
        assert!(mock.pulses.is_empty());
        assert_eq!(
            mock.count("measure"),
            1 + 10,
            "sizing read + 10 hold, no settle"
        );
        assert!(report.spring_gains.is_none(), "no spring was applied");
        let test = report.brain_only.as_ref().expect("brain-only record");
        assert_eq!((test.support, test.settle_support), (0.0, 0.0));
        assert_eq!(test.settle_s_applied, 0.0);
        assert_eq!(test.reward, "none");
        assert!(!test.perturbation);
        assert!(!test.plasticity_frozen, "no control was supplied");
        assert!(test.plasticity_reason.is_some());
    }

    #[test]
    fn progress_event_carries_the_attempt_stand_duration() {
        let mut c = criterion_for(2, 0.05);
        c.hold_s = 0.5;
        c.attempt_cap_s = 2.0;
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let mut events = CollectingEventSink::default();
        let run_id = RunId("balance-stand".to_string());
        let seeds = [7u64];
        let mut run = BalancePhaseRun {
            phase: "test",
            seeds: &seeds,
            criterion: &c,
            run_id: &run_id,
            support_mode: None,
            deliver_reward: None,
            plasticity: None,
            reward_policy: BalanceRewardPolicy::Affect,
            data: None,
            apply_event: None,
        };
        let report = run_balance_phase(
            &mut run,
            &mut mock,
            &mut |_| Ok(()),
            &mut events,
            &CancelToken::new(),
        )
        .expect("phase");
        let stand = events
            .events
            .iter()
            .find_map(|event| match &event.kind {
                RunEventKind::Progress {
                    stand: Some(stand),
                    samples_done,
                    ..
                } => {
                    assert_eq!(*samples_done, 1);
                    Some(stand)
                }
                _ => None,
            })
            .expect("progress stand");
        let attempt = &report.attempts[0];
        assert_eq!(stand.seed, 7);
        assert_eq!(stand.outcome, "success");
        assert!((stand.balanced_s - attempt.balanced_s).abs() < 1e-9);
        assert!((stand.support_level - attempt.support_level).abs() < 1e-9);
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
                vec!["contact_a".to_string()],
                vec!["contact_b".to_string()],
                vec!["contact_c".to_string()]
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
    fn test_death_after_settle_stays_at_support_zero_when_sim_time_rewinds() {
        let mut c = criterion_for(2, 0.05);
        c.settle_s = 0.5;
        c.hold_s = 10.0;
        c.attempt_cap_s = 20.0;
        c.support_min = 0.0;
        let mut mock = MockController::new(
            0.05,
            2,
            Box::new(|k, _| {
                if k >= 40 {
                    Scripted {
                        terminal: Some(BalanceTerminal::Death),
                        sim_time_s: Some(0.0),
                        tilt_deg: 40.0,
                        ..upright_step()
                    }
                } else {
                    upright_step()
                }
            }),
        );
        let report = run_phase(&c, &mut mock, "test", &[1]).expect("phase");
        let attempt = &report.attempts[0];
        assert_eq!(attempt.outcome, BalanceAttemptOutcome::Death);
        assert!(
            attempt.balanced_s > 0.5,
            "the stand was scored after settle, got {}",
            attempt.balanced_s
        );
        assert!(
            attempt.support_level.abs() < 1.0e-9,
            "rewound death clock must not relabel the grip as the settle, got {}",
            attempt.support_level
        );

        let mut early = MockController::new(
            0.05,
            2,
            Box::new(|k, _| {
                if k >= 2 {
                    Scripted {
                        terminal: Some(BalanceTerminal::Death),
                        sim_time_s: Some(0.0),
                        ..upright_step()
                    }
                } else {
                    upright_step()
                }
            }),
        );
        let during_c = c.clone();
        let during = run_phase(&during_c, &mut early, "test", &[1]).expect("phase");
        assert_eq!(during.attempts[0].outcome, BalanceAttemptOutcome::Death);
        assert!(
            during.attempts[0].balanced_s < 0.2,
            "a test attempt scores from its first sample"
        );
        assert!(
            during.attempts[0].support_level.abs() < 1.0e-9,
            "a test death inside the settle records support 0, got {}",
            during.attempts[0].support_level
        );
        assert_eq!(
            early.spring_sets[0].1, 0.0,
            "the test settle spring carries no support"
        );
    }

    #[test]
    fn unsupported_attempt_scores_from_its_first_sample_and_supported_still_settles() {
        let mut c = criterion_for(2, 0.05);
        c.settle_s = 1.25;
        c.hold_s = 0.5;
        c.attempt_cap_s = 4.0;
        c.perturb_interval_s = 0.0;
        c.support_start = 1.0;
        c.support_min = 1.0;
        assert_eq!(applied_settle_s(&c, true), 0.0);
        assert_eq!(applied_settle_s(&c, false), 1.25);

        let mut unsupported = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let report = run_phase(&c, &mut unsupported, "test", &[1]).expect("phase");
        let attempt = &report.attempts[0];
        assert_eq!(attempt.outcome, BalanceAttemptOutcome::Success);
        assert_eq!(attempt.bursts, 10, "scored from the first sample");
        assert_eq!(unsupported.count("measure"), 1 + 10, "no settle readings");
        assert_eq!(report.brain_only.as_ref().unwrap().settle_s_applied, 0.0);

        let mut supported = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let report = run_phase(&c, &mut supported, "train", &[1]).expect("phase");
        assert_eq!(report.attempts[0].outcome, BalanceAttemptOutcome::Success);
        assert!(
            supported.count("measure") >= 1 + 20 + 10,
            "the supported attempt settles first, got {}",
            supported.count("measure")
        );
        assert_eq!(supported.spring_sets[0].1, 1.0, "grip held during settle");
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
        // h0 = subtree com z 0.150 - contact z 0.01 = 0.140
        let k_expected = 3.0 * 1.0 * 9.81 * 0.140 / 0.087;
        assert!((mock.spring_sets[0].0.k_rot - k_expected).abs() < 1e-9);
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
            c.support_start = 0.5;
            c.hold_s = 1.0;
            c.attempt_cap_s = 4.0;
            c.perturb_interval_s = 0.4;
            c.perturb_duration_s = 0.1;
            let mut mock = MockController::new(period, 2, Box::new(|_, _| upright_step()));
            let report = run_phase(&c, &mut mock, "train", &[3]).expect("phase");
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
    fn train_phase_delivers_reward_from_trainer_samples() {
        let mut c = criterion_for(2, 0.05);
        c.settle_s = 0.0;
        c.hold_s = 0.1;
        c.attempt_cap_s = 1.0;
        c.support_start = 0.0;
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let mut delivered: Vec<RewardSignal> = Vec::new();
        let mut sink = |signals: &[RewardSignal]| -> Result<(), TrainerError> {
            delivered.extend_from_slice(signals);
            Ok(())
        };
        let mut events = CollectingEventSink::default();
        let run_id = RunId("balance-train".to_string());
        let mut run = BalancePhaseRun {
            phase: "train",
            seeds: &[1],
            criterion: &c,
            run_id: &run_id,
            support_mode: None,
            deliver_reward: Some(&mut sink),
            plasticity: None,
            reward_policy: BalanceRewardPolicy::Affect,
            data: None,
            apply_event: None,
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

    // ---- brain-only test phase ----

    fn zero_gains() -> SpringGains {
        SpringGains {
            k_rot: 0.0,
            d_rot: 0.0,
            gravity_comp: 0.0,
        }
    }

    /// Records freeze/restore calls; `frozen` is readable from the goal presenter.
    #[derive(Default)]
    struct MockPlasticity {
        frozen: std::rc::Rc<std::cell::Cell<bool>>,
        freezes: u32,
        restores: u32,
        fail_freeze: bool,
        fail_restore: bool,
    }

    impl PlasticityControl for MockPlasticity {
        fn freeze(
            &mut self,
        ) -> Result<crate::plasticity_freeze::PlasticityFreezeOutcome, TrainerError> {
            self.freezes += 1;
            if self.fail_freeze {
                return Err(TrainerError::Runtime("freeze refused".to_string()));
            }
            self.frozen.set(true);
            Ok(crate::plasticity_freeze::PlasticityFreezeOutcome {
                mappings_frozen: 3,
                memory_formation_paused: false,
            })
        }

        fn restore(&mut self) -> Result<(), TrainerError> {
            self.restores += 1;
            self.frozen.set(false);
            if self.fail_restore {
                return Err(TrainerError::Runtime("restore refused".to_string()));
            }
            Ok(())
        }
    }

    fn brain_only_criterion() -> BalanceFeetCriterion {
        let mut c = criterion_for(2, 0.05);
        c.settle_s = 0.5;
        c.hold_s = 0.5;
        c.attempt_cap_s = 2.0;
        c.support_min = 0.3;
        c.perturb_interval_s = 0.2;
        c.perturb_duration_s = 0.1;
        c
    }

    fn run_test_phase(
        c: &BalanceFeetCriterion,
        mock: &mut MockController,
        plasticity: &mut MockPlasticity,
        cancel: &CancelToken,
        seeds: &[u64],
    ) -> Result<BalancePhaseReport, TrainerError> {
        let mut events = CollectingEventSink::default();
        let run_id = RunId("balance-test".to_string());
        let frozen = plasticity.frozen.clone();
        let cancel_in_goal = cancel.clone();
        let mut seen_frozen = Vec::new();
        let result = {
            let mut run = BalancePhaseRun {
                phase: "test",
                seeds,
                criterion: c,
                run_id: &run_id,
                support_mode: None,
                deliver_reward: None,
                plasticity: Some(plasticity),
                reward_policy: BalanceRewardPolicy::Affect,
                data: None,
                apply_event: None,
            };
            run_balance_phase(
                &mut run,
                mock,
                &mut |_| {
                    seen_frozen.push(frozen.get());
                    if cancel_in_goal.is_cancelled() {
                        return Ok(());
                    }
                    Ok(())
                },
                &mut events,
                cancel,
            )
        };
        assert!(
            seen_frozen.iter().all(|f| *f),
            "learning was live during a test burst"
        );
        result
    }

    #[test]
    fn test_phase_never_sets_a_nonzero_spring_or_perturbs_and_records_it() {
        let c = brain_only_criterion();
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let mut plasticity = MockPlasticity::default();
        let report = run_test_phase(&c, &mut mock, &mut plasticity, &CancelToken::new(), &[1, 2])
            .expect("phase");
        assert_eq!(report.attempts.len(), 2);
        assert_eq!(mock.spring_sets.len(), 2, "one zero spring per attempt");
        for (gains, level, deadband) in &mock.spring_sets {
            assert_eq!((*level, *deadband), (0.0, 0.0));
            assert_eq!(*gains, zero_gains(), "no gravity comp, no xy spring");
        }
        assert!(mock.level_updates.is_empty(), "no update, settle included");
        assert!(mock.pulses.is_empty(), "perturbation is off in test");
        assert!(report.attempts.iter().all(|a| a.perturbations == 0));
        assert!(report.attempts.iter().all(|a| a.support_level == 0.0));
        assert_eq!((plasticity.freezes, plasticity.restores), (1, 1));
        let test = report.brain_only.as_ref().expect("record");
        assert!(test.plasticity_frozen && test.plasticity_reason.is_none());
        assert_eq!(test.mappings_frozen, 3);
        assert!(!test.memory_formation_paused);
        let metrics = balance_phase_metrics(&report);
        assert_eq!(metrics["test_support"], 0.0);
        assert_eq!(metrics["test_plasticity_frozen"], 1.0);
        let card = balance_phase_scorecard("s", "c", 1, "test", "d", "h", &report);
        assert_eq!(card.metrics["brain_only"], 1.0);
        assert!(card.metadata.contains_key("test_mode"));
    }

    #[test]
    fn test_phase_ignores_train_graduation_and_rejects_other_support() {
        let c = brain_only_criterion();
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let seeds = [1u64];
        let run_id = RunId("balance-test".to_string());
        let mut events = CollectingEventSink::default();
        let mut run = BalancePhaseRun {
            phase: "test",
            seeds: &seeds,
            criterion: &c,
            run_id: &run_id,
            support_mode: Some(SupportMode::Fixed(0.7)),
            deliver_reward: None,
            plasticity: None,
            reward_policy: BalanceRewardPolicy::Affect,
            data: None,
            apply_event: None,
        };
        let error = run_balance_phase(
            &mut run,
            &mut mock,
            &mut |_| Ok(()),
            &mut events,
            &CancelToken::new(),
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("support 0"), "{error}");
        assert!(
            mock.calls.is_empty(),
            "refused before touching the controller"
        );
        run.support_mode = Some(SupportMode::Fixed(0.0));
        run_balance_phase(
            &mut run,
            &mut mock,
            &mut |_| Ok(()),
            &mut events,
            &CancelToken::new(),
        )
        .expect("explicit zero is accepted");
    }

    #[test]
    fn test_phase_rejects_a_reward_sink() {
        let c = brain_only_criterion();
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let mut sink = |_: &[RewardSignal]| -> Result<(), TrainerError> { Ok(()) };
        let mut plasticity = MockPlasticity::default();
        let mut events = CollectingEventSink::default();
        let run_id = RunId("balance-test".to_string());
        // Scope ends the plasticity borrow. BalancePhaseRun does not implement Drop,
        // so mem::drop would not release it.
        let error = {
            let mut run = BalancePhaseRun {
                phase: "test",
                seeds: &[1],
                criterion: &c,
                run_id: &run_id,
                support_mode: None,
                deliver_reward: Some(&mut sink),
                plasticity: Some(&mut plasticity),
                reward_policy: BalanceRewardPolicy::Affect,
                data: None,
                apply_event: None,
            };
            run_balance_phase(
                &mut run,
                &mut mock,
                &mut |_| Ok(()),
                &mut events,
                &CancelToken::new(),
            )
            .unwrap_err()
            .to_string()
        };
        assert!(error.contains("no reward sink"), "{error}");
        assert!(mock.calls.is_empty());
        assert_eq!(plasticity.freezes, 0, "refused before freezing");
    }

    #[test]
    fn test_phase_restores_plasticity_on_error_and_on_cancel() {
        let c = brain_only_criterion();
        // error: the controller reply has no contact points, so sizing fails mid-phase
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        mock.floor_z = None;
        let mut plasticity = MockPlasticity::default();
        let error =
            run_test_phase(&c, &mut mock, &mut plasticity, &CancelToken::new(), &[1]).unwrap_err();
        assert!(matches!(error, TrainerError::Runtime(_)), "{error}");
        assert_eq!(
            (plasticity.freezes, plasticity.restores),
            (1, 1),
            "error path"
        );
        // cancel
        let cancel = CancelToken::new();
        cancel.cancel();
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let mut plasticity = MockPlasticity::default();
        let error = run_test_phase(&c, &mut mock, &mut plasticity, &cancel, &[1]).unwrap_err();
        assert!(matches!(error, TrainerError::Cancelled(_)), "{error}");
        assert_eq!(
            (plasticity.freezes, plasticity.restores),
            (1, 1),
            "cancel path"
        );
        // a restore failure surfaces, and keeps the Cancelled kind
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let mut plasticity = MockPlasticity {
            fail_restore: true,
            ..Default::default()
        };
        let error = run_test_phase(&c, &mut mock, &mut plasticity, &cancel, &[1]).unwrap_err();
        assert!(
            matches!(&error, TrainerError::Cancelled(m) if m.contains("stayed frozen")),
            "{error}"
        );
    }

    #[test]
    fn test_phase_runs_unfrozen_and_says_why_when_the_freeze_fails() {
        let c = brain_only_criterion();
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let mut events = CollectingEventSink::default();
        let run_id = RunId("balance-test".to_string());
        let mut plasticity = MockPlasticity {
            fail_freeze: true,
            ..Default::default()
        };
        // Scope ends the plasticity borrow. BalancePhaseRun does not implement Drop,
        // so mem::drop would not release it.
        let report = {
            let mut run = BalancePhaseRun {
                phase: "test",
                seeds: &[1],
                criterion: &c,
                run_id: &run_id,
                support_mode: None,
                deliver_reward: None,
                plasticity: Some(&mut plasticity),
                reward_policy: BalanceRewardPolicy::Affect,
                data: None,
                apply_event: None,
            };
            run_balance_phase(
                &mut run,
                &mut mock,
                &mut |_| Ok(()),
                &mut events,
                &CancelToken::new(),
            )
            .expect("phase")
        };
        let test = report.brain_only.expect("record");
        assert!(!test.plasticity_frozen);
        assert!(test
            .plasticity_reason
            .as_deref()
            .is_some_and(|r| r.contains("freeze refused")));
        assert_eq!(plasticity.restores, 1, "restore is still attempted");
    }

    #[test]
    fn support_need_and_effort_follow_the_documented_formulas() {
        // (tilt + 0.1 s * |omega_xy|) / max_tilt, clamped
        let need = support_need(5.0_f64.to_radians(), [0.1, 0.0, 9.0], 15.0, 0.1);
        let expected = (5.0_f64.to_radians() + 0.01) / 15.0_f64.to_radians();
        assert!((need - expected).abs() < 1e-12);
        assert_eq!(support_need(1.0, [0.0; 3], 15.0, 0.1), 1.0);
        let sizing = sizing_inputs(&generic_body_info(), "torso", 0.140).unwrap();
        let weight = 1.0 * 9.81;
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
        c.contact_bodies[1] = "contact_a".to_string();
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
        c.bws_max_fraction = 1.5;
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
        assert_eq!(c.support_deadband_deg, 0.0);
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
            gravity_comp: 0.5,
        };
        let body = set_body_spring_request("torso", &gains, 1.0, 0.1);
        assert_eq!(body["action"], "set_body_spring");
        assert_eq!(body["body"], "torso");
        assert!(body.get("k_lin").is_none() && body.get("d_lin").is_none());
        assert_eq!(body["gravity_comp"], 0.5);
        assert_eq!(body["ref"], "current");
        assert_eq!(body["deadband_rad"], 0.1);
        assert_eq!(
            update_body_spring_request("torso", 0.4, None, None)["level"],
            0.4
        );
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
            contacts_except: vec![vec!["a".to_string(), "b".to_string()]],
            actuators: false,
        });
        assert!(m.get("actuators").is_none(), "{m}");
        assert_eq!(m["bodies"], serde_json::json!(["torso"]));
        assert_eq!(
            m["contacts"][1],
            serde_json::json!({"a": ["b"], "b": "world"})
        );
        assert_eq!(
            m["contacts"][2],
            serde_json::json!({"a_except": ["a", "b"], "b": "world"})
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

    fn probe_train_run(
        probe_every: u32,
        seeds: &[u64],
        plasticity: &mut MockPlasticity,
    ) -> (BalancePhaseReport, MockController, Vec<bool>) {
        let mut c = criterion_for(2, 0.05);
        c.settle_s = 0.0;
        c.hold_s = 0.5;
        c.attempt_cap_s = 2.0;
        c.attempts_per_level = 2;
        c.perturb_interval_s = 0.2;
        c.probe_every = probe_every;
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let mut events = CollectingEventSink::default();
        let run_id = RunId("probe-test".to_string());
        let frozen = plasticity.frozen.clone();
        let mut frozen_per_burst = Vec::new();
        let report = {
            let mut run = BalancePhaseRun {
                phase: "train",
                seeds,
                criterion: &c,
                run_id: &run_id,
                support_mode: None,
                deliver_reward: None,
                plasticity: Some(plasticity),
                reward_policy: BalanceRewardPolicy::Affect,
                data: None,
                apply_event: None,
            };
            run_balance_phase(
                &mut run,
                &mut mock,
                &mut |_| {
                    frozen_per_burst.push(frozen.get());
                    Ok(())
                },
                &mut events,
                &CancelToken::new(),
            )
            .expect("train phase")
        };
        (report, mock, frozen_per_burst)
    }

    #[test]
    fn probes_are_scheduled_every_n_and_run_unassisted_and_frozen() {
        let mut plasticity = MockPlasticity::default();
        let (report, mock, frozen_bursts) =
            probe_train_run(3, &[1, 2, 3, 4, 5, 6, 7], &mut plasticity);
        let flags: Vec<bool> = report.attempts.iter().map(|a| a.probe).collect();
        assert_eq!(
            flags,
            [false, false, false, true, false, false, false, true, false]
        );
        let before: Vec<Option<u32>> = report
            .attempts
            .iter()
            .filter(|a| a.probe)
            .map(|a| a.train_attempts_before)
            .collect();
        assert_eq!(before, [Some(3), Some(6)]);
        assert!(report
            .attempts
            .iter()
            .filter(|a| a.probe)
            .all(|a| a.support_level == 0.0 && a.perturbations == 0));
        assert_eq!(report.probe_attempts, 2);
        assert_eq!(report.probe_plasticity_frozen, Some(true));
        assert_eq!((plasticity.freezes, plasticity.restores), (2, 2));
        assert!(!plasticity.frozen.get(), "restored after the last probe");
        assert!(frozen_bursts.iter().any(|f| *f) && frozen_bursts.iter().any(|f| !*f));
        // probe attempts use a zero spring at level 0; training attempts keep the grip
        let zero_sets = mock
            .spring_sets
            .iter()
            .filter(|(g, level, _)| *level == 0.0 && *g == zero_gains())
            .count();
        assert_eq!(zero_sets, 2);
        assert_eq!(mock.spring_sets.len(), 9);
        // aggregates cover the 7 training attempts only
        assert_eq!(report.balanced.attempts, 7);
    }

    #[test]
    fn probes_do_not_change_the_schedule_level() {
        let seeds = [1u64, 2, 3, 4, 5, 6, 7];
        let (with, _, _) = probe_train_run(2, &seeds, &mut MockPlasticity::default());
        let (without, _, _) = probe_train_run(0, &seeds, &mut MockPlasticity::default());
        assert_eq!(with.support_level_next, without.support_level_next);
        let train_levels = |r: &BalancePhaseReport| -> Vec<f64> {
            r.attempts
                .iter()
                .filter(|a| !a.probe)
                .map(|a| a.support_level)
                .collect()
        };
        assert_eq!(train_levels(&with), train_levels(&without));
        assert_eq!(with.success_rate, without.success_rate);
        assert_eq!(
            with.success_rate_by_support,
            without.success_rate_by_support
        );
        assert_eq!(with.graduated, without.graduated);
    }

    #[test]
    fn probe_every_zero_means_no_probes() {
        let mut plasticity = MockPlasticity::default();
        let (report, _, _) = probe_train_run(0, &[1, 2, 3, 4, 5, 6], &mut plasticity);
        assert_eq!(report.attempts.len(), 6);
        assert!(report.attempts.iter().all(|a| !a.probe));
        assert_eq!(report.probe_attempts, 0);
        assert_eq!(report.probe_plasticity_frozen, None);
        assert_eq!((plasticity.freezes, plasticity.restores), (0, 0));
    }

    #[test]
    fn probe_every_defaults_to_five_and_validates() {
        let mut c = criterion_for(2, 0.05);
        assert_eq!(c.probe_every, 5);
        c.probe_every = MAX_PROBE_EVERY + 1;
        assert!(c.validate().is_err());
        c.probe_every = 0;
        assert!(c.validate().is_ok());
    }

    // ---- standing: only contact bodies touch, torso stays tall ----

    fn standing_sample(seq: u64, torso_height: f64, reset: Option<f64>) -> BalanceSample {
        BalanceSample {
            torso_height_m: Some(torso_height),
            reset_height_m: reset,
            assist_slow: None,
            assist_rise: None,
            actuators: BTreeMap::new(),
            ..sample(seq, 1, 2.0)
        }
    }

    #[test]
    fn extra_body_touching_is_not_upright() {
        let c = criterion();
        assert!(c.only_contacts_touch);
        let mut s = sample(1, 1, 2.0);
        assert!(sample_is_upright(&s, &c));
        s.other_contact_touching = true;
        s.other_contact_bodies = vec!["any_body".to_string()];
        assert!(!sample_is_upright(&s, &c));
        let reasons = non_upright_reasons(&s, &c, None);
        assert!(reasons.other_body_contact && !reasons.tilt && !reasons.contact);
    }

    #[test]
    fn sample_below_the_height_fraction_is_not_upright() {
        let mut c = criterion();
        c.min_height_fraction = 0.8;
        // reset 0.10 m: the limit is 0.08 m
        assert!(sample_is_upright(
            &standing_sample(1, 0.081, Some(0.10)),
            &c
        ));
        let low = standing_sample(1, 0.079, Some(0.10));
        assert!(!sample_is_upright(&low, &c));
        assert!(non_upright_reasons(&low, &c, Some(0.10)).height);
    }

    #[test]
    fn zero_or_false_turns_each_check_off() {
        let mut c = criterion();
        let mut s = standing_sample(1, 0.01, Some(0.10));
        s.other_contact_touching = true;
        assert!(!sample_is_upright(&s, &c));
        c.min_height_fraction = 0.0;
        assert!(!sample_is_upright(&s, &c), "contact check still on");
        c.only_contacts_touch = false;
        assert!(sample_is_upright(&s, &c));
        c.min_height_fraction = 0.8;
        assert!(!sample_is_upright(&s, &c), "height check back on");
    }

    #[test]
    fn reset_height_is_taken_from_the_first_sample() {
        let c = fall_criterion();
        let first = standing_sample(1, 0.10, None);
        let open = fed(&OpenBalanceAttempt::fresh(), &first, &c);
        assert_eq!(open.reset_height_m, Some(0.10));
        // a later, lower sample does not move the reference and is not upright
        let low = standing_sample(2, 0.05, None);
        let open = fed(&open, &low, &c);
        assert_eq!(open.reset_height_m, Some(0.10));
        assert_eq!(open.holding_bursts, 0);
        assert_eq!(open.non_upright.height, 1);
        // a settling first sample sets the reference too
        let mut settling = standing_sample(1, 0.20, None);
        settling.settling = true;
        let open = fed(&OpenBalanceAttempt::fresh(), &settling, &c);
        assert_eq!(open.reset_height_m, Some(0.20));
    }

    #[test]
    fn non_upright_ends_the_stretch_not_the_attempt_and_is_counted() {
        let c = fall_criterion();
        let mut open = fed(
            &OpenBalanceAttempt::fresh(),
            &standing_sample(1, 0.10, None),
            &c,
        );
        for seq in 2..=4 {
            let mut s = standing_sample(seq, 0.10, None);
            s.other_contact_touching = true;
            open = fed(&open, &s, &c);
        }
        assert_eq!(open.holding_bursts, 0);
        assert_eq!(open.non_upright.other_body_contact, 3);
        // upright again: the attempt continues and a new stretch starts
        open = fed(&open, &standing_sample(5, 0.10, None), &c);
        assert_eq!(open.holding_bursts, 1);
        assert_eq!(open.longest_balanced_bursts, 1);
        // the counts reach the record
        let mut finish = sample(6, 1, 2.0);
        finish.terminal = Some(BalanceTerminal::Death);
        match observe_balance_sample(&open, &finish, &c, 1).expect("observe") {
            BalanceAttemptStep::Finished(record) => {
                assert_eq!(record.non_upright.other_body_contact, 3);
                assert_eq!(record.non_upright.tilt, 0);
            }
            BalanceAttemptStep::Continue(_) => panic!("death ends the attempt"),
        }
    }

    #[test]
    fn min_height_fraction_is_validated_and_defaults() {
        let mut c = criterion_for(2, 0.05);
        assert_eq!(c.min_height_fraction, 0.8);
        assert!(c.only_contacts_touch);
        for ok in [0.0, 0.5, 1.0] {
            c.min_height_fraction = ok;
            assert!(c.validate().is_ok(), "{ok}");
        }
        for bad in [-0.1, 1.01, f64::NAN, f64::INFINITY] {
            c.min_height_fraction = bad;
            assert!(c.validate().is_err(), "{bad}");
        }
        let parsed: BalanceFeetCriterion = serde_json::from_value(serde_json::json!({
            "torso_body": "t", "contact_bodies": ["a"],
            "max_tilt_deg": 10.0, "hold_s": 1.0, "attempt_cap_s": 2.0
        }))
        .expect("minimal criterion");
        assert!(parsed.only_contacts_touch);
        assert_eq!(parsed.min_height_fraction, 0.8);
    }

    #[test]
    fn measure_asks_for_the_every_other_body_set_only_when_on() {
        let mut c = criterion_for(2, 0.05);
        let request = measure_request(&c);
        assert_eq!(
            request.contacts_except,
            vec![vec!["contact_a".to_string(), "contact_b".to_string()]]
        );
        c.only_contacts_touch = false;
        assert!(measure_request(&c).contacts_except.is_empty());
    }

    #[test]
    fn another_body_on_the_floor_blocks_success_and_is_reported() {
        let c = criterion_for(2, 0.05);
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        mock.other_touching = vec!["some_body".to_string()];
        let report = run_phase(&c, &mut mock, "train", &[1]).expect("phase");
        let attempt = &report.attempts[0];
        assert_eq!(attempt.outcome, BalanceAttemptOutcome::Truncated);
        assert!(attempt.non_upright.other_body_contact > 0);
        assert_eq!(attempt.non_upright.tilt, 0);
        assert_eq!(mock.measure_requests[0].contacts_except.len(), 1);
        // the same body passes once the check is off
        let mut off = criterion_for(2, 0.05);
        off.only_contacts_touch = false;
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        mock.other_touching = vec!["some_body".to_string()];
        let report = run_phase(&off, &mut mock, "train", &[1]).expect("phase");
        assert_eq!(report.attempts[0].outcome, BalanceAttemptOutcome::Success);
        assert!(mock.measure_requests[0].contacts_except.is_empty());
    }

    #[test]
    fn a_torso_that_sinks_below_the_fraction_does_not_balance() {
        let c = criterion_for(2, 0.05);
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        mock.torso_z_drop = Some((3, 0.05));
        let report = run_phase(&c, &mut mock, "train", &[1]).expect("phase");
        let attempt = &report.attempts[0];
        assert_eq!(attempt.outcome, BalanceAttemptOutcome::Truncated);
        assert!(attempt.non_upright.height > 0);
    }

    #[test]
    fn an_older_controller_rejecting_a_except_fails_setup_clearly() {
        let c = criterion_for(2, 0.05);
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        mock.rejects_a_except = true;
        let error = run_phase(&c, &mut mock, "train", &[1])
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("restart or update the controller"),
            "{error}"
        );
        assert!(error.contains("a_except"), "{error}");
    }

    // ---- over-support: carried and over-assisted samples, flags, probe dependence ----

    fn supported(seq: u64, level: f64) -> BalanceSample {
        BalanceSample {
            support_level: Some(level),
            torso_height_m: Some(1.0),
            reset_height_m: Some(1.0),
            assist_slow: None,
            assist_rise: None,
            actuators: BTreeMap::new(),
            support_need: Some(0.2),
            support_effort: Some(0.3),
            ..sample(seq, 1, 2.0)
        }
    }

    fn use_flags(s: &BalanceSample, level: f64) -> Option<(bool, bool)> {
        let c = criterion();
        support_use_flags(s, &c, s.reset_height_m, level)
    }

    #[test]
    fn carried_when_feet_are_off_and_the_body_is_up_or_when_lifted() {
        assert_eq!(use_flags(&supported(1, 0.5), 0.5), Some((false, false)));
        let mut feet_off = supported(1, 0.5);
        feet_off.contacts = vec![true, false];
        assert_eq!(use_flags(&feet_off, 0.5), Some((true, false)));
        // feet off but the torso already collapsed under the height fraction: not carried
        let mut fallen = feet_off.clone();
        fallen.torso_height_m = Some(0.5);
        assert_eq!(use_flags(&fallen, 0.5), Some((false, false)));
        // feet down but the torso is lifted past the tolerance
        let mut lifted = supported(1, 0.5);
        lifted.torso_height_m = Some(1.06);
        assert_eq!(use_flags(&lifted, 0.5), Some((true, false)));
        lifted.torso_height_m = Some(1.04);
        assert_eq!(use_flags(&lifted, 0.5), Some((false, false)));
    }

    #[test]
    fn over_assist_needs_effort_beyond_need_plus_margin() {
        let mut s = supported(1, 0.5);
        s.support_need = Some(0.1);
        s.support_effort = Some(0.4);
        assert_eq!(use_flags(&s, 0.5), Some((false, true)));
        s.support_effort = Some(0.35);
        assert_eq!(use_flags(&s, 0.5), Some((false, false)));
        s.support_effort = None;
        assert_eq!(use_flags(&s, 0.5), Some((false, false)));
    }

    #[test]
    fn settling_and_unassisted_samples_are_not_scored() {
        let mut settling = supported(1, 0.5);
        settling.settling = true;
        assert_eq!(use_flags(&settling, 0.5), None);
        assert_eq!(use_flags(&supported(1, 0.0), 0.0), None);
    }

    #[test]
    fn attempt_record_carries_the_fractions() {
        let mut c = criterion();
        c.hold_s = 100.0;
        c.attempt_cap_s = 100.0;
        let mut open = OpenBalanceAttempt::for_attempt(1.0, 0.5);
        for seq in 1..=4u64 {
            let mut s = supported(seq, 0.5);
            if seq <= 3 {
                s.contacts = vec![false, true];
            }
            if seq == 4 {
                s.support_effort = Some(0.9);
            }
            open = fed(&open, &s, &c);
        }
        assert_eq!(open.support_use.samples, 4);
        let mut last = supported(5, 0.5);
        last.terminal = Some(BalanceTerminal::Interrupted);
        match observe_balance_sample(&open, &last, &c, 1).expect("observe") {
            BalanceAttemptStep::Finished(r) => {
                assert!((r.carried_fraction - 0.6).abs() < 1e-9, "{r:?}");
                assert!((r.over_assist_fraction - 0.2).abs() < 1e-9, "{r:?}");
            }
            BalanceAttemptStep::Continue(_) => panic!("terminal ends the attempt"),
        }
    }

    fn use_record(
        level: f64,
        outcome: BalanceAttemptOutcome,
        carried: f64,
        over: f64,
    ) -> BalanceAttemptRecord {
        BalanceAttemptRecord {
            carried_fraction: carried,
            over_assist_fraction: over,
            ..record(outcome, level, 1.0)
        }
    }

    fn probe_record(before: u32, balanced_s: f64) -> BalanceAttemptRecord {
        BalanceAttemptRecord {
            probe: true,
            train_attempts_before: Some(before),
            balanced_s,
            ..record(BalanceAttemptOutcome::Truncated, 0.0, 0.0)
        }
    }

    #[test]
    fn level_is_flagged_only_above_the_flag_fraction() {
        let mut c = criterion();
        c.over_support_flag_fraction = 0.2;
        let ok = BalanceAttemptOutcome::Success;
        let attempts = vec![
            use_record(0.8, ok, 0.2, 0.2),
            use_record(0.8, ok, 0.2, 0.2),
            use_record(0.6, ok, 0.1, 0.5),
            use_record(0.4, ok, 0.34, 0.0),
        ];
        let mut report = BalancePhaseReport::from_attempts(attempts).expect("report");
        report.assess_over_support(&c);
        let at = |k: &str| &report.support_use_by_support[k];
        assert!(
            !at("0.80").over_support,
            "equal to the fraction is not above"
        );
        assert!(at("0.80").over_support_causes.is_empty());
        assert_eq!(
            at("0.60").over_support_causes,
            vec![OverSupportCause::OverAssist]
        );
        assert_eq!(
            at("0.40").over_support_causes,
            vec![OverSupportCause::Carried]
        );
        assert!((at("0.40").mean_carried_fraction - 0.34).abs() < 1e-9);
        assert_eq!(report.over_support_levels, vec!["0.40", "0.60"]);
    }

    #[test]
    fn probe_dependence_flags_a_graduating_level() {
        let mut c = criterion();
        c.hold_s = 3.0;
        let ok = BalanceAttemptOutcome::Success;
        let mut attempts = Vec::new();
        for _ in 0..2 {
            attempts.push(use_record(0.5, ok, 0.0, 0.0));
        }
        attempts.push(probe_record(2, 1.0));
        attempts.push(use_record(0.4, BalanceAttemptOutcome::Truncated, 0.0, 0.0));
        attempts.push(probe_record(3, 4.0));
        let mut report = BalancePhaseReport::from_attempts(attempts).expect("report");
        assert_eq!(report.probes.len(), 2);
        assert_eq!(report.probes[0].attempt_index, 3);
        assert_eq!(report.probes[0].support_level, 0.5);
        assert_eq!(report.probes[1].support_level, 0.4);
        report.assess_over_support(&c);
        let level = &report.support_use_by_support["0.50"];
        assert_eq!(
            level.over_support_causes,
            vec![OverSupportCause::ProbeDependence]
        );
        assert_eq!(level.probe_median_balanced_s, Some(1.0));
        // the level that did not graduate is not flagged even though its probe was short
        assert!(!report.support_use_by_support["0.40"].over_support);
        // probes that hold long enough clear the flag
        let mut good = report.clone();
        good.probes[0].balanced_s = 3.0;
        good.assess_over_support(&c);
        assert!(good.over_support_levels.is_empty());
    }

    #[test]
    fn probe_dependence_uses_the_nearest_probe_after_the_level() {
        let mut c = criterion();
        c.hold_s = 3.0;
        let ok = BalanceAttemptOutcome::Success;
        let attempts = vec![
            use_record(0.5, ok, 0.0, 0.0),
            use_record(0.4, BalanceAttemptOutcome::Truncated, 0.0, 0.0),
            probe_record(2, 0.5),
        ];
        let mut report = BalancePhaseReport::from_attempts(attempts).expect("report");
        // the probe ran at level 0.4, so 0.5 borrows the nearest probe after it
        report.assess_over_support(&c);
        assert_eq!(report.over_support_levels, vec!["0.50"]);
    }

    #[test]
    fn over_support_fields_default_validate_and_old_documents_parse() {
        let mut c = criterion_for(2, 0.05);
        assert_eq!(c.lift_tolerance, 0.05);
        assert_eq!(c.over_assist_margin, 0.25);
        assert_eq!(c.over_support_flag_fraction, 0.2);
        let parsed: BalanceFeetCriterion = serde_json::from_value(serde_json::json!({
            "torso_body": "t", "contact_bodies": ["a"],
            "max_tilt_deg": 10.0, "hold_s": 1.0, "attempt_cap_s": 2.0
        }))
        .expect("older criterion");
        assert_eq!(parsed.lift_tolerance, 0.05);
        assert_eq!(parsed.over_assist_margin, 0.25);
        assert_eq!(parsed.over_support_flag_fraction, 0.2);
        for bad in [-0.1, f64::NAN] {
            c.lift_tolerance = bad;
            assert!(c.validate().is_err(), "{bad}");
        }
        c.lift_tolerance = 0.05;
        for bad in [-0.1, 1.1] {
            c.over_assist_margin = bad;
            assert!(c.validate().is_err(), "{bad}");
        }
        c.over_assist_margin = 0.25;
        for bad in [-0.1, 1.1] {
            c.over_support_flag_fraction = bad;
            assert!(c.validate().is_err(), "{bad}");
        }
        c.over_support_flag_fraction = 0.0;
        assert!(c.validate().is_ok());
    }

    // ---- adaptive assist and contribution reward ----

    fn adaptive_criterion(period_s: f64) -> BalanceFeetCriterion {
        let mut c = criterion_for(2, period_s);
        c.support_mode = AssistMode::Adaptive;
        c
    }

    fn at(c: &BalanceFeetCriterion, a: f64, a_slow: f64) -> AdaptiveAssist {
        AdaptiveAssist::resume(c, AdaptiveAssistState { a, a_slow })
    }

    #[test]
    fn adaptive_rises_fast_on_overshoot_and_fades_with_the_slow_constant() {
        let c = adaptive_criterion(0.05);
        // a_slow 0.5 puts the cone at 2 + 13 * 0.5 = 8.5 deg; 14 deg predicted overshoots it.
        let mut assist = at(&c, 0.5, 0.5);
        let step = assist.step(&c, 14.0, 1);
        let cone = 2.0 + 13.0 * (1.0 - step_slow_before(0.5, 0.5, &c));
        let e = (14.0 - cone) / (15.0 - cone);
        let expected = 0.5 + (e - 0.5) * (1.0 - (-0.05_f64 / 0.2).exp());
        assert!(
            (step.level - expected).abs() < 1e-12,
            "{} vs {expected}",
            step.level
        );
        assert!(step.rise > c.catch_threshold && step.catch, "{step:?}");
        // Fade: no overshoot, hold over. a decays by exp(-dt / fade_tau_s) per burst.
        let mut fading = at(&c, 0.8, 0.8);
        for _ in 0..40 {
            fading.step(&c, 0.0, 1);
        }
        let expected = 0.8 * (-(40.0 * 0.05) / 20.0_f64).exp();
        assert!(
            (fading.level() - expected).abs() < 1e-12,
            "{}",
            fading.level()
        );
    }

    fn step_slow_before(a: f64, a_slow: f64, c: &BalanceFeetCriterion) -> f64 {
        a_slow + (a - a_slow) * (1.0 - (-c.burst_period_s / c.fade_tau_s).exp())
    }

    #[test]
    fn adaptive_fading_pauses_for_hold_after_catch_s() {
        let c = adaptive_criterion(0.05);
        let mut assist = at(&c, 0.5, 0.5);
        assert!(assist.step(&c, 14.0, 1).catch);
        let after_catch = assist.level();
        // 1.0 s hold at 0.05 s per burst = 20 bursts with no fade.
        for burst in 1..=20 {
            assist.step(&c, 0.0, 1);
            assert_eq!(assist.level(), after_catch, "burst {burst} must not fade");
        }
        assist.step(&c, 0.0, 1);
        let faded = after_catch * (-0.05_f64 / 20.0).exp();
        assert!((assist.level() - faded).abs() < 1e-12, "{}", assist.level());
        // hold_after_catch_s 0 never pauses.
        let mut quick = c.clone();
        quick.hold_after_catch_s = 0.0;
        let mut assist = at(&quick, 0.5, 0.5);
        assert!(assist.step(&quick, 14.0, 1).catch);
        let level = assist.level();
        assist.step(&quick, 0.0, 1);
        assert!(assist.level() < level);
    }

    #[test]
    fn adaptive_cone_widens_as_a_slow_falls_and_stays_in_range() {
        let c = adaptive_criterion(0.05);
        let mut assist = AdaptiveAssist::start(&c);
        assert!(
            (assist.cone_deg(&c) - c.cone_min_deg).abs() < 1e-12,
            "full assist, narrowest cone"
        );
        let mut last = assist.cone_deg(&c);
        for _ in 0..4000 {
            let step = assist.step(&c, 0.0, 1);
            assert!(
                step.cone_deg >= last - 1e-12,
                "cone never narrows while fading"
            );
            assert!(
                step.cone_deg >= c.cone_min_deg - 1e-12 && step.cone_deg <= c.max_tilt_deg + 1e-12,
                "{}",
                step.cone_deg
            );
            last = step.cone_deg;
        }
        assert!(
            last > 10.0,
            "after 200 s of fading the cone is wide: {last}"
        );
        // Fully faded: cone == max_tilt, and the overshoot stays finite.
        let mut gone = at(&c, 0.0, 0.0);
        assert_eq!(gone.cone_deg(&c), c.max_tilt_deg);
        let step = gone.step(&c, 20.0, 1);
        assert!(step.level > 0.0 && step.level <= 1.0);
    }

    #[test]
    fn adaptive_time_constants_mean_the_same_at_two_burst_rates() {
        let slow = adaptive_criterion(0.05);
        let fast = adaptive_criterion(0.025);
        // Fade for 10 s.
        let (mut a, mut b) = (at(&slow, 0.9, 0.9), at(&fast, 0.9, 0.9));
        for _ in 0..200 {
            a.step(&slow, 0.0, 1);
        }
        for _ in 0..400 {
            b.step(&fast, 0.0, 1);
        }
        assert!(
            (a.level() - b.level()).abs() < 1e-9,
            "{} vs {}",
            a.level(),
            b.level()
        );
        // Rise for 0.5 s at full overshoot (predicted tilt at the limit gives e = 1).
        let (mut a, mut b) = (at(&slow, 0.2, 0.2), at(&fast, 0.2, 0.2));
        for _ in 0..10 {
            a.step(&slow, 15.0, 1);
        }
        for _ in 0..20 {
            b.step(&fast, 15.0, 1);
        }
        let expected = 1.0 - 0.8 * (-0.5_f64 / 0.2).exp();
        assert!((a.level() - expected).abs() < 1e-9, "{}", a.level());
        assert!((b.level() - expected).abs() < 1e-9, "{}", b.level());
    }

    #[test]
    fn adaptive_level_stays_between_support_min_and_support_start() {
        let mut c = adaptive_criterion(0.05);
        c.support_start = 0.8;
        c.support_min = 0.2;
        let mut assist = AdaptiveAssist::start(&c);
        for _ in 0..3000 {
            assist.step(&c, 0.0, 1);
        }
        assert!((assist.level() - 0.2).abs() < 1e-12, "{}", assist.level());
        for _ in 0..100 {
            assist.step(&c, 15.0, 1);
        }
        assert!((assist.level() - 0.8).abs() < 1e-3, "{}", assist.level());
        let resumed = at(&c, 5.0, -1.0);
        assert_eq!((resumed.level(), resumed.slow()), (0.8, 0.2));
    }

    #[test]
    fn adaptive_attempt_does_not_end_at_hold_s_but_succeeds_at_the_cap() {
        let mut c = adaptive_criterion(0.05);
        c.hold_s = 0.5;
        c.attempt_cap_s = 2.0;
        let mut open = OpenBalanceAttempt::fresh();
        for seq in 1..=39 {
            open = fed(&open, &balanced(seq, 1), &c);
        }
        assert!(open.holding_bursts > c.hold_bursts(), "well past hold_s");
        match observe_balance_sample(&open, &balanced(40, 1), &c, 1).expect("obs") {
            BalanceAttemptStep::Finished(r) => {
                assert_eq!(r.outcome, BalanceAttemptOutcome::Success);
                assert_eq!(r.bursts, 40);
            }
            BalanceAttemptStep::Continue(_) => panic!("the cap ends an adaptive attempt"),
        }
        // A fall at the cap burst is a fall, not a success.
        let mut falling = OpenBalanceAttempt::fresh();
        for seq in 1..=35 {
            falling = fed(&falling, &balanced(seq, 1), &c);
        }
        for seq in 36..=39 {
            falling = fed(&falling, &sample(seq, 1, 70.0), &c);
        }
        match observe_balance_sample(&falling, &sample(40, 1, 70.0), &c, 1).expect("obs") {
            BalanceAttemptStep::Finished(r) => assert_eq!(r.outcome, BalanceAttemptOutcome::Fell),
            BalanceAttemptStep::Continue(_) => panic!("a confirmed fall ends the attempt"),
        }
        // The ladder still ends at hold_s.
        let ladder = BalanceFeetCriterion {
            support_mode: AssistMode::Ladder,
            ..c
        };
        let mut open = OpenBalanceAttempt::fresh();
        for seq in 1..=9 {
            open = fed(&open, &balanced(seq, 1), &ladder);
        }
        assert!(matches!(
            observe_balance_sample(&open, &balanced(10, 1), &ladder, 1).expect("obs"),
            BalanceAttemptStep::Finished(r) if r.outcome == BalanceAttemptOutcome::Success
        ));
    }

    fn adaptive_train(
        c: &BalanceFeetCriterion,
        mock: &mut MockController,
        seeds: &[u64],
        mode: Option<SupportMode>,
        policy: BalanceRewardPolicy,
        delivered: &mut Vec<RewardSignal>,
    ) -> BalancePhaseReport {
        let mut sink = |signals: &[RewardSignal]| -> Result<(), TrainerError> {
            delivered.extend_from_slice(signals);
            Ok(())
        };
        let mut events = CollectingEventSink::default();
        let run_id = RunId("balance-adaptive".to_string());
        let mut run = BalancePhaseRun {
            phase: "train",
            seeds,
            criterion: c,
            run_id: &run_id,
            support_mode: mode,
            deliver_reward: Some(&mut sink),
            plasticity: None,
            reward_policy: policy,
            data: None,
            apply_event: None,
        };
        run_balance_phase(
            &mut run,
            mock,
            &mut |_| Ok(()),
            &mut events,
            &CancelToken::new(),
        )
        .expect("phase")
    }

    fn quick_adaptive() -> BalanceFeetCriterion {
        let mut c = adaptive_criterion(0.05);
        c.hold_s = 0.5;
        c.attempt_cap_s = 3.0;
        c.settle_s = 0.2;
        c.fade_tau_s = 1.0;
        c.probe_every = 0;
        c
    }

    #[test]
    fn adaptive_phase_keeps_the_level_across_attempts_and_reports_it() {
        let c = quick_adaptive();
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let mut delivered = Vec::new();
        let report = adaptive_train(
            &c,
            &mut mock,
            &[1, 2],
            None,
            BalanceRewardPolicy::Affect,
            &mut delivered,
        );
        let (first, second) = (&report.attempts[0], &report.attempts[1]);
        assert_eq!(
            first.outcome,
            BalanceAttemptOutcome::Success,
            "ends at the 3 s cap"
        );
        assert_eq!(first.bursts, 60, "not at hold_s (0.5 s = 10 bursts)");
        assert_eq!(first.support_level, 1.0);
        let stats = first.adaptive.expect("adaptive numbers");
        assert!(
            second.support_level < 1.0,
            "a reset does not restore full support"
        );
        assert!(
            (second.support_level - stats.min_a).abs() < 1e-12,
            "{} vs {}",
            second.support_level,
            stats.min_a
        );
        assert!(stats.mean_a < 1.0 && stats.min_a <= stats.mean_a);
        assert!(stats.end_cone_deg > c.cone_min_deg && stats.end_cone_deg <= c.max_tilt_deg);
        assert_eq!(stats.catches, 0);
        // Group by attempt, not by level.
        assert_eq!(
            report
                .success_rate_by_support
                .keys()
                .cloned()
                .collect::<Vec<_>>(),
            vec!["attempt_0001".to_string(), "attempt_0002".to_string()]
        );
        assert!(report.balanced_by_support.contains_key("attempt_0002"));
        assert!(report.support_use_by_support.contains_key("attempt_0001"));
        // Timeline: ~1 Hz over 6 scored seconds, and the state to resume from.
        assert!(
            report.assist_timeline.len() >= 6,
            "{}",
            report.assist_timeline.len()
        );
        assert!(report
            .assist_timeline
            .windows(2)
            .all(|w| w[1].t_s > w[0].t_s));
        assert!(report
            .assist_timeline
            .windows(2)
            .all(|w| w[1].cone_deg >= w[0].cone_deg - 1e-12));
        let next = report.adaptive_state_next.expect("state to resume");
        assert_eq!(report.support_level_next, Some(next.a));
        assert!(next.a < second.support_level && next.a_slow > next.a);
        // The first non-settling update changed level and cone together; deadband in radians.
        assert!(!mock.level_updates.is_empty());
        assert!(mock
            .deadband_updates
            .iter()
            .all(|d| d.is_some_and(|rad| rad > 0.0)));
        // Settle stays at full support; the first update comes after it.
        assert_eq!(mock.spring_sets[0].1, 1.0);
        assert!(
            mock.level_updates[0].1 >= 0.2 - 1e-9,
            "{:?}",
            mock.level_updates[0]
        );
    }

    #[test]
    fn adaptive_resume_continues_from_a_saved_state() {
        let c = quick_adaptive();
        let state = AdaptiveAssistState {
            a: 0.4,
            a_slow: 0.6,
        };
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let mut delivered = Vec::new();
        let report = adaptive_train(
            &c,
            &mut mock,
            &[1],
            Some(SupportMode::ResumeAdaptive(state)),
            BalanceRewardPolicy::Affect,
            &mut delivered,
        );
        assert_eq!(report.attempts[0].support_level, 0.4);
        // A level-only resume starts both averages there.
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let report = adaptive_train(
            &c,
            &mut mock,
            &[1],
            Some(SupportMode::Resume(0.5)),
            BalanceRewardPolicy::Affect,
            &mut delivered,
        );
        assert_eq!(report.attempts[0].support_level, 0.5);
        let state = report.adaptive_state_next.expect("state");
        assert!(state.a <= 0.5 && state.a_slow <= 0.5);
    }

    #[test]
    fn adaptive_counts_catches_and_a_fall_ends_the_attempt() {
        let c = quick_adaptive();
        // Calm, then a lean that overshoots the cone, then a fall.
        let mut mock = MockController::new(
            0.05,
            2,
            Box::new(|k, _| Scripted {
                tilt_deg: match k {
                    0..=30 => 2.0,
                    31..=33 => 14.5,
                    34..=39 => 5.0,
                    _ => 70.0,
                },
                down: true,
                terminal: None,
                sim_time_s: None,
            }),
        );
        let mut delivered = Vec::new();
        let report = adaptive_train(
            &c,
            &mut mock,
            &[1],
            None,
            BalanceRewardPolicy::Contribution,
            &mut delivered,
        );
        let attempt = &report.attempts[0];
        assert_eq!(attempt.outcome, BalanceAttemptOutcome::Fell);
        let stats = attempt.adaptive.expect("adaptive numbers");
        assert!(stats.catches >= 1, "{stats:?}");
        assert!(stats.catches_per_min > 0.0);
        assert!(delivered
            .iter()
            .any(|s| s.channel == AffectChannel::Pain && s.magnitude < 1.0));
        assert_eq!(
            delivered.last().map(|s| (s.channel, s.magnitude)),
            Some((AffectChannel::Pain, 1.0))
        );
    }

    #[test]
    fn adaptive_is_ignored_by_the_test_phase_and_by_probes() {
        let mut c = adaptive_criterion(0.05);
        c.hold_s = 0.5;
        c.attempt_cap_s = 2.0;
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let report = run_phase(&c, &mut mock, "test", &[1]).expect("test phase");
        assert_eq!(
            report.attempts[0].bursts, 10,
            "the test phase ends at hold_s"
        );
        assert!(report.attempts[0].adaptive.is_none());
        assert!(report.assist_timeline.is_empty());
        // A probe in an adaptive train phase ends at hold_s with support 0.
        let mut c = quick_adaptive();
        c.probe_every = 1;
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let mut delivered = Vec::new();
        let report = adaptive_train(
            &c,
            &mut mock,
            &[1],
            None,
            BalanceRewardPolicy::Affect,
            &mut delivered,
        );
        let probe = report.attempts.iter().find(|a| a.probe).expect("probe");
        assert_eq!(probe.bursts, 10);
        assert!(probe.adaptive.is_none() && probe.support_level == 0.0);
    }

    /// Adaptive phase with a long cap so the stance and lean timers can run.
    fn stance_adaptive() -> BalanceFeetCriterion {
        let mut c = quick_adaptive();
        c.attempt_cap_s = 12.0;
        c
    }

    fn one_adaptive_attempt(
        c: &BalanceFeetCriterion,
        mode: Option<SupportMode>,
        policy: BalanceRewardPolicy,
        script: Script,
    ) -> (BalancePhaseReport, MockController, Vec<RewardSignal>) {
        let mut mock = MockController::new(0.05, 2, script);
        let mut delivered = Vec::new();
        let report = adaptive_train(c, &mut mock, &[1], mode, policy, &mut delivered);
        (report, mock, delivered)
    }

    fn feet_up_between(from_s: f64, to_s: f64) -> Script {
        Box::new(move |_, t| Scripted {
            down: !(from_s - 1e-9..to_s - 1e-9).contains(&t),
            ..upright_step()
        })
    }

    #[test]
    fn adaptive_weight_support_is_zeroed_while_a_foot_is_up_and_restored_when_both_are_down() {
        let c = stance_adaptive();
        let (report, mock, _) = one_adaptive_attempt(
            &c,
            None,
            BalanceRewardPolicy::Affect,
            feet_up_between(1.0, 1.5),
        );
        assert_eq!(report.attempts[0].outcome, BalanceAttemptOutcome::Success);
        assert_eq!(mock.gravity_updates.len(), 2, "{:?}", mock.gravity_updates);
        let (off, off_at) = mock.gravity_updates[0];
        let (on, on_at) = mock.gravity_updates[1];
        assert_eq!(off, 0.0);
        assert!((off_at - 1.0).abs() < 1e-6, "{off_at}");
        assert_eq!(on, c.bws_max_fraction);
        assert!((on_at - 1.5).abs() < 1e-6, "{on_at}");
        // The opening spring carries the criterion's fraction.
        assert_eq!(mock.spring_sets[0].0.gravity_comp, c.bws_max_fraction);
        // Feet always down: weight support is never touched.
        let (_, mock, _) = one_adaptive_attempt(
            &c,
            None,
            BalanceRewardPolicy::Affect,
            Box::new(|_, _| upright_step()),
        );
        assert!(mock.gravity_updates.is_empty());
    }

    #[test]
    fn adaptive_recenters_after_lean_grace_and_ends_once_tilt_is_inside_cone_min() {
        let c = stance_adaptive();
        // A wide cone (slow average well below the level's ceiling), then a lean inside the tilt
        // limit but beyond that cone for 4 s, then upright again.
        let state = AdaptiveAssistState {
            a: 0.3,
            a_slow: 0.3,
        };
        let (report, mock, _) = one_adaptive_attempt(
            &c,
            Some(SupportMode::ResumeAdaptive(state)),
            BalanceRewardPolicy::Affect,
            Box::new(|_, t| Scripted {
                tilt_deg: if (1.0..5.0).contains(&t) { 13.0 } else { 1.0 },
                ..upright_step()
            }),
        );
        let stats = report.attempts[0].adaptive.expect("adaptive numbers");
        assert_eq!(stats.recenters, 1, "{stats:?}");
        let cone_min_rad = c.cone_min_deg.to_radians();
        let at_min: Vec<usize> = mock
            .deadband_updates
            .iter()
            .enumerate()
            .filter(|(_, d)| d.is_some_and(|rad| (rad - cone_min_rad).abs() < 1e-9))
            .map(|(i, _)| i)
            .collect();
        assert!(!at_min.is_empty(), "{:?}", mock.deadband_updates);
        // One continuous stretch at cone_min (the level moves, so it is re-sent each burst).
        assert!(at_min.windows(2).all(|w| w[1] == w[0] + 1), "{at_min:?}");
        // Not before the grace: the lean starts at 1.0 s, so the cone drops after 3.0 s.
        let at = mock.level_updates[at_min[0]].1;
        assert!(
            at >= 1.0 + c.lean_grace_s - 1e-6 && at <= 1.0 + c.lean_grace_s + 0.3,
            "{at}"
        );
        // Tilt is back inside cone_min at 5.0 s: the adaptive cone returns.
        let restored = mock.deadband_updates[at_min[at_min.len() - 1] + 1..]
            .iter()
            .flatten()
            .any(|rad| *rad > cone_min_rad + 0.01);
        assert!(restored, "{:?}", mock.deadband_updates);
        assert!(stats.end_cone_deg > c.cone_min_deg);
        // A short lean (shorter than the grace) never recenters.
        let (report, _, _) = one_adaptive_attempt(
            &c,
            Some(SupportMode::ResumeAdaptive(state)),
            BalanceRewardPolicy::Affect,
            Box::new(|_, t| Scripted {
                tilt_deg: if (1.0..2.5).contains(&t) { 13.0 } else { 1.0 },
                ..upright_step()
            }),
        );
        assert_eq!(report.attempts[0].adaptive.expect("stats").recenters, 0);
    }

    #[test]
    fn adaptive_attempt_ends_lost_stance_only_after_the_grace() {
        let c = stance_adaptive();
        let (report, _, delivered) = one_adaptive_attempt(
            &c,
            None,
            BalanceRewardPolicy::Contribution,
            Box::new(|_, t| Scripted {
                down: t < 1.0,
                ..upright_step()
            }),
        );
        let attempt = &report.attempts[0];
        assert_eq!(attempt.outcome, BalanceAttemptOutcome::LostStance);
        assert_eq!(outcome_name(attempt.outcome), "lost_stance");
        // Settle 0.2 s, feet up at 1.0 s, ended just past 1.0 s later: 2.05 s of 0.05 s bursts.
        assert!(
            (35..=39).contains(&attempt.bursts),
            "ended after {} bursts",
            attempt.bursts
        );
        let stats = attempt.adaptive.expect("stats");
        assert!(stats.lost_stance && stats.standing_fraction < 1.0);
        assert!(stats.feet_down_fraction < 1.0 && stats.feet_down_fraction > 0.0);
        assert_eq!(
            delivered.last().map(|s| (s.channel, s.magnitude)),
            Some((AffectChannel::Pain, 0.5))
        );
        // Off its feet for less than the grace: the attempt carries on to the cap.
        let (report, _, _) = one_adaptive_attempt(
            &c,
            None,
            BalanceRewardPolicy::Contribution,
            feet_up_between(1.0, 1.9),
        );
        assert_eq!(report.attempts[0].outcome, BalanceAttemptOutcome::Success);
        assert!(!report.attempts[0].adaptive.expect("stats").lost_stance);
        // The next attempt starts again from the reset keyframe.
        let mut c2 = c.clone();
        c2.reset_keyframe = "stand".to_string();
        let mut mock = MockController::new(
            0.05,
            2,
            Box::new(|_, t| Scripted {
                down: t < 1.0,
                ..upright_step()
            }),
        );
        let mut delivered = Vec::new();
        let report = adaptive_train(
            &c2,
            &mut mock,
            &[1, 2],
            None,
            BalanceRewardPolicy::Affect,
            &mut delivered,
        );
        assert!(report
            .attempts
            .iter()
            .all(|a| a.outcome == BalanceAttemptOutcome::LostStance));
        assert_eq!(mock.resets.len(), 2);
        assert!(mock.resets.iter().all(|r| r.as_deref() == Some("stand")));
        assert!(
            report.attempts[1].adaptive.is_some(),
            "assist state carries on"
        );
    }

    #[test]
    fn adaptive_success_needs_the_standing_fraction() {
        let c = stance_adaptive();
        // Two 0.9 s lifts, each shorter than the stance grace, in 240 counted bursts: 15% off.
        let two_lifts: Script = Box::new(|_, t| Scripted {
            down: !((1.0..1.9).contains(&t) || (5.0..5.9).contains(&t)),
            ..upright_step()
        });
        let (report, _, _) = one_adaptive_attempt(&c, None, BalanceRewardPolicy::Affect, two_lifts);
        let stats = report.attempts[0].adaptive.expect("stats");
        assert!(
            stats.standing_fraction > 0.8 && stats.standing_fraction < 0.9,
            "{stats:?}"
        );
        assert_eq!(report.attempts[0].outcome, BalanceAttemptOutcome::Success);
        // The same attempt with the bar raised past what it stood is capped, not a success.
        let mut strict = c.clone();
        strict.standing_fraction_for_success = 0.95;
        let two_lifts: Script = Box::new(|_, t| Scripted {
            down: !((1.0..1.9).contains(&t) || (5.0..5.9).contains(&t)),
            ..upright_step()
        });
        let (report, _, _) =
            one_adaptive_attempt(&strict, None, BalanceRewardPolicy::Affect, two_lifts);
        assert_eq!(report.attempts[0].outcome, BalanceAttemptOutcome::Truncated);
        assert!(!report.attempts[0].adaptive.expect("stats").lost_stance);
    }

    #[test]
    fn contribution_reward_pays_no_pleasure_when_not_standing() {
        let c = adaptive_criterion(0.05);
        let standing = |need: f64| BalanceSample {
            support_need: Some(need),
            support_level: Some(0.0),
            ..sample(1, 1, 2.0)
        };
        let lifted = |need: f64| BalanceSample {
            contacts: vec![true, false],
            ..standing(need)
        };
        let mut r = BalanceContributionReward::new();
        r.on_sample(&standing(0.8), None, &c);
        assert!(pleasure_of(&r.on_sample(&standing(0.2), None, &c)).is_some());
        // The same recovery with a foot up pays nothing.
        let mut r = BalanceContributionReward::new();
        r.on_sample(&lifted(0.8), None, &c);
        assert!(pleasure_of(&r.on_sample(&lifted(0.2), None, &c)).is_none());
        // And the cap pays nothing on a non-standing burst.
        let mut r = BalanceContributionReward::new();
        let out = r.on_sample(&lifted(0.2), Some(&BalanceAttemptOutcome::Success), &c);
        assert!(pleasure_of(&out).is_none());
        let mut r = BalanceContributionReward::new();
        let out = r.on_sample(&standing(0.2), Some(&BalanceAttemptOutcome::Success), &c);
        assert!(pleasure_of(&out).is_some());
        // A lost stance is Pain 0.5 and nothing else.
        let mut r = BalanceContributionReward::new();
        let out = r.on_sample(&lifted(0.2), Some(&BalanceAttemptOutcome::LostStance), &c);
        assert_eq!(pain_of(&out), Some(0.5));
        assert!(pleasure_of(&out).is_none());
    }

    #[test]
    fn adaptive_attempt_stats_reach_the_progress_event_when_the_attempt_finishes() {
        let c = quick_adaptive();
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let mut sink = |_: &[RewardSignal]| -> Result<(), TrainerError> { Ok(()) };
        let mut events = CollectingEventSink::default();
        let run_id = RunId("balance-adaptive".to_string());
        let mut run = BalancePhaseRun {
            phase: "train",
            seeds: &[1],
            criterion: &c,
            run_id: &run_id,
            support_mode: None,
            deliver_reward: Some(&mut sink),
            plasticity: None,
            reward_policy: BalanceRewardPolicy::Contribution,
            data: None,
            apply_event: None,
        };
        run_balance_phase(
            &mut run,
            &mut mock,
            &mut |_| Ok(()),
            &mut events,
            &CancelToken::new(),
        )
        .expect("phase");
        let stands: Vec<AttemptStand> = events
            .events
            .iter()
            .filter_map(|event| match &event.kind {
                RunEventKind::Progress { stand, .. } => stand.clone(),
                _ => None,
            })
            .collect();
        assert_eq!(stands.len(), 1);
        let stats = stands[0].adaptive.expect("adaptive numbers in the stand");
        assert_eq!(stats.standing_fraction, 1.0);
        assert_eq!(stats.feet_down_fraction, 1.0);
    }

    fn contribution_sample(need: f64, level: f64) -> BalanceSample {
        BalanceSample {
            support_need: Some(need),
            support_level: Some(level),
            ..sample(1, 1, 2.0)
        }
    }

    fn pleasure_of(signals: &[RewardSignal]) -> Option<f64> {
        signals
            .iter()
            .find(|s| s.channel == AffectChannel::Pleasure)
            .map(|s| s.magnitude)
    }

    fn pain_of(signals: &[RewardSignal]) -> Option<f64> {
        signals
            .iter()
            .find(|s| s.channel == AffectChannel::Pain)
            .map(|s| s.magnitude)
    }

    #[test]
    fn contribution_reward_pays_for_what_the_robot_does_itself() {
        let c = adaptive_criterion(0.05);
        let mut r = BalanceContributionReward::new();
        assert_eq!(
            BalanceContributionReward::policy_id(),
            "reward.balance-contribution.v2"
        );
        // Upright and held pays nothing, even at a steady need.
        assert!(r
            .on_sample(&contribution_sample(0.3, 0.9), None, &c)
            .is_empty());
        assert!(r
            .on_sample(&contribution_sample(0.3, 0.9), None, &c)
            .is_empty());
        // Recovery: need drops 0.1 -> min(1, 4 * 0.1) * (1 - 0.5).
        let signals = r.on_sample(&contribution_sample(0.2, 0.5), None, &c);
        assert!(
            (pleasure_of(&signals).expect("pleasure") - 0.4 * 0.5).abs() < 1e-12,
            "{signals:?}"
        );
        assert!(pain_of(&signals).is_none());
        // Recovery while held at a = 1 pays nothing.
        assert!(r
            .on_sample(&contribution_sample(0.0, 1.0), None, &c)
            .is_empty());
        // A big drop saturates at 1 * (1 - a).
        let mut r = BalanceContributionReward::new();
        r.on_sample(&contribution_sample(1.0, 0.25), None, &c);
        let signals = r.on_sample(&contribution_sample(0.0, 0.25), None, &c);
        assert!((pleasure_of(&signals).expect("pleasure") - 0.75).abs() < 1e-12);
    }

    #[test]
    fn contribution_reward_pain_on_catch_fall_and_death() {
        let mut c = adaptive_criterion(0.05);
        c.catch_threshold = 0.1;
        let mut r = BalanceContributionReward::new();
        let rising = |rise: f64| BalanceSample {
            assist_rise: Some(rise),
            actuators: BTreeMap::new(),
            ..contribution_sample(0.5, 0.5)
        };
        // A rise of exactly the threshold or more pays the full 0.5; a smaller one a share.
        assert!((pain_of(&r.on_sample(&rising(0.1), None, &c)).expect("pain") - 0.5).abs() < 1e-12);
        assert!((pain_of(&r.on_sample(&rising(0.4), None, &c)).expect("pain") - 0.5).abs() < 1e-12);
        assert!(
            (pain_of(&r.on_sample(&rising(0.05), None, &c)).expect("pain") - 0.25).abs() < 1e-12
        );
        assert!(r.on_sample(&rising(0.0), None, &c).is_empty());
        for outcome in [BalanceAttemptOutcome::Death, BalanceAttemptOutcome::Fell] {
            let signals = r.on_sample(&rising(0.4), Some(&outcome), &c);
            assert_eq!(signals.len(), 1);
            assert_eq!(
                (signals[0].channel, signals[0].magnitude),
                (AffectChannel::Pain, 1.0)
            );
        }
        // Settling bursts emit nothing, even a death.
        let settling = BalanceSample {
            settling: true,
            ..rising(0.4)
        };
        assert!(r
            .on_sample(&settling, Some(&BalanceAttemptOutcome::Death), &c)
            .is_empty());
    }

    #[test]
    fn contribution_reward_success_pays_one_minus_a_slow_and_works_for_the_ladder() {
        let c = adaptive_criterion(0.05);
        let mut r = BalanceContributionReward::new();
        let done = BalanceSample {
            assist_slow: Some(0.3),
            ..contribution_sample(0.2, 0.1)
        };
        let signals = r.on_sample(&done, Some(&BalanceAttemptOutcome::Success), &c);
        assert!(
            (pleasure_of(&signals).expect("pleasure") - 0.7).abs() < 1e-12,
            "{signals:?}"
        );
        // Ladder: no assist_slow, the constant level stands in for a and a_slow.
        let mut ladder = BalanceContributionReward::new();
        let held = contribution_sample(0.2, 0.25);
        let signals = ladder.on_sample(&held, Some(&BalanceAttemptOutcome::Success), &c);
        assert!((pleasure_of(&signals).expect("pleasure") - 0.75).abs() < 1e-12);
        // Fully held at the cap: nothing to pay.
        let mut full = BalanceContributionReward::new();
        assert!(full
            .on_sample(
                &contribution_sample(0.2, 1.0),
                Some(&BalanceAttemptOutcome::Success),
                &c
            )
            .is_empty());
    }

    #[test]
    fn contribution_reward_runs_with_the_ladder_schedule() {
        let mut c = criterion_for(2, 0.05);
        c.settle_s = 0.0;
        c.hold_s = 0.1;
        c.attempt_cap_s = 1.0;
        c.support_start = 0.5;
        c.support_min = 0.0;
        assert_eq!(c.support_mode, AssistMode::Ladder);
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let mut delivered = Vec::new();
        let report = adaptive_train(
            &c,
            &mut mock,
            &[1],
            None,
            BalanceRewardPolicy::Contribution,
            &mut delivered,
        );
        assert!(report.attempts[0].adaptive.is_none());
        assert_eq!(delivered.len(), 1, "{delivered:?}");
        assert_eq!(delivered[0].channel, AffectChannel::Pleasure);
        assert!((delivered[0].magnitude - 0.5).abs() < 1e-12);
        assert_eq!(
            BalanceRewardPolicy::from_mode("contribution"),
            Some(BalanceRewardPolicy::Contribution)
        );
        assert_eq!(
            BalanceRewardPolicy::Contribution.policy_id(),
            BalanceContributionReward::policy_id()
        );
    }

    #[test]
    fn adaptive_fields_validate_and_older_criteria_load_unchanged() {
        let c = adaptive_criterion(0.05);
        assert!(c.validate().is_ok());
        assert_eq!(
            (
                c.fade_tau_s,
                c.catch_tau_s,
                c.hold_after_catch_s,
                c.cone_min_deg,
                c.catch_threshold
            ),
            (20.0, 0.2, 1.0, 2.0, 0.05)
        );
        let bad = |edit: &dyn Fn(&mut BalanceFeetCriterion)| {
            let mut c = adaptive_criterion(0.05);
            edit(&mut c);
            c.validate().is_err()
        };
        assert!(bad(&|c| c.fade_tau_s = 0.0));
        assert!(bad(&|c| c.fade_tau_s = f64::NAN));
        assert!(bad(&|c| c.catch_tau_s = 0.0));
        assert!(bad(&|c| c.catch_tau_s = 20.0), "must be < fade_tau_s");
        assert!(bad(&|c| c.hold_after_catch_s = -0.1));
        assert!(bad(&|c| c.cone_min_deg = -1.0));
        assert!(bad(&|c| c.cone_min_deg = 15.0), "must be < max_tilt_deg");
        assert!(bad(&|c| c.catch_threshold = 1.1));
        assert!(bad(&|c| c.catch_threshold = -0.1));
        assert!(bad(&|c| c.lean_grace_s = 0.0));
        assert!(bad(&|c| c.lean_grace_s = f64::NAN));
        assert!(bad(&|c| c.stance_grace_s = 0.0));
        assert!(bad(&|c| c.stance_grace_s = -1.0));
        assert!(bad(&|c| c.standing_fraction_for_success = 0.0));
        assert!(bad(&|c| c.standing_fraction_for_success = 1.01));
        assert!(!bad(&|c| c.standing_fraction_for_success = 1.0));
        let mut ok = adaptive_criterion(0.05);
        ok.cone_min_deg = 0.0;
        ok.hold_after_catch_s = 0.0;
        ok.catch_threshold = 0.0;
        assert!(ok.validate().is_ok());
        let parsed: BalanceFeetCriterion = serde_json::from_value(serde_json::json!({
            "torso_body": "t", "contact_bodies": ["a"],
            "max_tilt_deg": 10.0, "hold_s": 1.0, "attempt_cap_s": 2.0
        }))
        .expect("older criterion");
        assert_eq!(parsed.support_mode, AssistMode::Ladder);
        assert_eq!(
            (parsed.fade_tau_s, parsed.catch_tau_s, parsed.cone_min_deg),
            (20.0, 0.2, 2.0)
        );
        assert_eq!(
            (
                parsed.lean_grace_s,
                parsed.stance_grace_s,
                parsed.standing_fraction_for_success
            ),
            (2.0, 1.0, 0.8),
            "older documents get the new defaults"
        );
        let adaptive: BalanceFeetCriterion = serde_json::from_value(serde_json::json!({
            "torso_body": "t", "contact_bodies": ["a"],
            "max_tilt_deg": 10.0, "hold_s": 1.0, "attempt_cap_s": 2.0,
            "support_mode": "adaptive"
        }))
        .expect("adaptive criterion");
        assert!(adaptive.is_adaptive());
        // An older attempt record (no adaptive numbers) loads and stays out of the JSON.
        let old =
            serde_json::to_value(record(BalanceAttemptOutcome::Success, 0.5, 1.0)).expect("json");
        assert!(old.get("adaptive").is_none(), "{old}");
        let back: BalanceAttemptRecord = serde_json::from_value(old).expect("old record");
        assert!(back.adaptive.is_none());
        // Adaptive stats saved before the stance numbers existed still load.
        let stats: AdaptiveAttemptStats = serde_json::from_value(serde_json::json!({
            "mean_a": 0.5, "min_a": 0.4, "end_cone_deg": 6.0,
            "catches": 1, "catches_per_min": 2.0
        }))
        .expect("older adaptive stats");
        assert_eq!(
            (stats.recenters, stats.lost_stance, stats.standing_fraction),
            (0, false, 0.0)
        );
        let outcome: BalanceAttemptOutcome =
            serde_json::from_value(serde_json::json!("lost_stance")).expect("outcome");
        assert_eq!(outcome, BalanceAttemptOutcome::LostStance);
    }

    #[test]
    fn update_body_spring_request_carries_the_deadband_only_when_given() {
        assert!(update_body_spring_request("torso", 0.4, None, None)
            .get("deadband_rad")
            .is_none());
        let with = update_body_spring_request("torso", 0.4, Some(0.1), None);
        assert_eq!(with["deadband_rad"], 0.1);
        assert_eq!(with["level"], 0.4);
    }

    #[test]
    fn assist_timeline_is_capped() {
        let c = adaptive_criterion(0.5);
        let mut run = AdaptiveRun::new(AdaptiveAssist::start(&c));
        for _ in 0..8000 {
            run.advance(0.5, &c);
        }
        assert_eq!(run.timeline.len(), ASSIST_TIMELINE_MAX_POINTS);
    }

    // ---- deviation against assistance ----

    #[test]
    fn goal_deviation_cases() {
        let mut c = criterion();
        c.max_tilt_deg = 20.0;
        c.min_height_fraction = 0.5;
        c.only_contacts_touch = true;
        // tilt only: scaled and clamped
        assert!((goal_deviation(&sample(1, 1, 5.0), &c, None) - 0.25).abs() < 1e-12);
        assert_eq!(goal_deviation(&sample(1, 1, 0.0), &c, None), 0.0);
        assert_eq!(goal_deviation(&sample(1, 1, 45.0), &c, None), 1.0);
        // a contact body off the floor, tilt small
        let mut foot_up = sample(1, 1, 1.0);
        foot_up.contacts = vec![true, false];
        assert_eq!(goal_deviation(&foot_up, &c, None), 1.0);
        // another body touching
        let mut other = sample(1, 1, 1.0);
        other.other_contact_touching = true;
        assert_eq!(goal_deviation(&other, &c, None), 1.0);
        c.only_contacts_touch = false;
        assert!(goal_deviation(&other, &c, None) < 0.1);
        // torso too low
        let mut low = sample(1, 1, 1.0);
        low.torso_height_m = Some(0.2);
        assert_eq!(goal_deviation(&low, &c, Some(0.6)), 1.0);
        low.torso_height_m = Some(0.5);
        assert!(goal_deviation(&low, &c, Some(0.6)) < 0.1);
    }

    fn dev_point(t_s: f64, d: f64, a: f64) -> DeviationAssistPoint {
        DeviationAssistPoint {
            t_s,
            deviation_mean: d,
            assistance_mean: a,
        }
    }

    #[test]
    fn deviation_timeline_averages_each_second() {
        let mut run = DeviationAssistRun::default();
        // 0.25 s bursts: four per second.
        for _ in 0..4 {
            run.add(0.2, 0.4, 1, 0.25);
        }
        for d in [0.0, 1.0, 0.0, 1.0] {
            run.add(d, 0.8, 1, 0.25);
        }
        let out = run.finish().expect("block");
        assert_eq!(out.n, 2);
        assert!((out.timeline[0].deviation_mean - 0.2).abs() < 1e-12);
        assert!((out.timeline[0].assistance_mean - 0.4).abs() < 1e-12);
        assert_eq!(out.timeline[0].t_s, 1.0);
        assert!((out.timeline[1].deviation_mean - 0.5).abs() < 1e-12);
        assert!((out.timeline[1].assistance_mean - 0.8).abs() < 1e-12);
        assert_eq!(out.timeline[1].t_s, 2.0);
    }

    #[test]
    fn deviation_timeline_decimates_at_the_cap_instead_of_stopping() {
        let mut run = DeviationAssistRun::default();
        for i in 0..(ASSIST_TIMELINE_MAX_POINTS + 10) {
            run.add((i % 10) as f64 / 10.0, 0.5, 1, 1.0);
        }
        let out = run.finish().expect("block");
        assert!(out.n <= ASSIST_TIMELINE_MAX_POINTS);
        assert!(out.n > ASSIST_TIMELINE_MAX_POINTS / 2);
        assert!(out.timeline.windows(2).all(|w| w[1].t_s > w[0].t_s));
        assert!(out.timeline.last().expect("last").t_s >= ASSIST_TIMELINE_MAX_POINTS as f64);
    }

    #[test]
    fn deviation_assist_fit_on_a_known_dataset() {
        // assist = 0.5 * dev + 0.1 exactly
        let pts: Vec<_> = (0..5)
            .map(|i| dev_point(i as f64 + 1.0, i as f64 * 0.2, 0.5 * (i as f64 * 0.2) + 0.1))
            .collect();
        let out = DeviationAssist::from_timeline(pts).expect("block");
        assert!((out.r.expect("r") - 1.0).abs() < 1e-9);
        assert!((out.slope.expect("slope") - 0.5).abs() < 1e-9);
        assert!((out.intercept.expect("intercept") - 0.1).abs() < 1e-9);
        assert_eq!(out.n, 5);
        // 20% of 5 points = 1 point each end
        assert!((out.early.dev - 0.0).abs() < 1e-12 && (out.early.assist - 0.1).abs() < 1e-12);
        assert!((out.late.dev - 0.8).abs() < 1e-12 && (out.late.assist - 0.5).abs() < 1e-12);
    }

    #[test]
    fn deviation_assist_r_is_null_without_variance_or_enough_points() {
        let flat: Vec<_> = (0..5)
            .map(|i| dev_point(i as f64, 0.3, 0.1 * i as f64))
            .collect();
        let out = DeviationAssist::from_timeline(flat).expect("block");
        assert!(out.r.is_none() && out.slope.is_none() && out.intercept.is_none());
        let flat_y: Vec<_> = (0..5)
            .map(|i| dev_point(i as f64, 0.1 * i as f64, 0.4))
            .collect();
        let out = DeviationAssist::from_timeline(flat_y).expect("block");
        assert!(out.r.is_none());
        assert_eq!(out.slope, Some(0.0));
        let two = vec![dev_point(1.0, 0.1, 0.2), dev_point(2.0, 0.3, 0.6)];
        assert!(DeviationAssist::from_timeline(two)
            .expect("block")
            .r
            .is_none());
        assert!(DeviationAssist::from_timeline(Vec::new()).is_none());
    }

    #[test]
    fn ladder_and_adaptive_train_phases_both_report_deviation_assist() {
        // ladder, with probes: the probes stay out of the timeline
        let mut plasticity = MockPlasticity::default();
        let (ladder, _, _) = probe_train_run(1, &[1, 2], &mut plasticity);
        assert!(ladder.attempts.iter().any(|a| a.probe));
        let block = ladder.deviation_assist.expect("ladder block");
        assert!(block.n >= 1);
        let scored_s: f64 = ladder
            .attempts
            .iter()
            .filter(|a| !a.probe)
            .map(|a| f64::from(a.bursts) * 0.05)
            .sum();
        assert!(block.timeline.last().expect("last").t_s <= scored_s + 1e-6);
        // tilt 2 of max 15, fixed level: a constant deviation
        assert!((block.timeline[0].deviation_mean - 2.0 / 15.0).abs() < 1e-9);
        // adaptive: the applied level starts at the ceiling and falls
        let c = quick_adaptive();
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let mut delivered = Vec::new();
        let report = adaptive_train(
            &c,
            &mut mock,
            &[1, 2],
            None,
            BalanceRewardPolicy::Affect,
            &mut delivered,
        );
        let block = report.deviation_assist.expect("adaptive block");
        assert!(block.n >= 5);
        assert!(block.early.assist > block.late.assist);
        // the test phase has none
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let test = run_phase(&c, &mut mock, "test", &[1]).expect("test phase");
        assert!(test.deviation_assist.is_none());
    }

    // ---- run data sink and scheduled events ----

    use crate::run_data::RecordingRunDataSink;

    fn rid() -> RunId {
        RunId("balance-hooks".to_string())
    }

    fn event(at_s: f64, label: &str) -> ScheduledEvent {
        ScheduledEvent {
            at_s,
            label: label.to_string(),
            action: ScheduledAction::SetMappingPsc {
                src: "area_a".to_string(),
                dst: "area_b".to_string(),
                psc_multiplier: 0.5,
            },
        }
    }

    fn run_with_hooks<'a>(
        c: &'a BalanceFeetCriterion,
        mock: &mut MockController,
        phase: &'a str,
        seeds: &'a [u64],
        run_id: &'a RunId,
        data: Option<&'a mut dyn RunDataSink>,
        apply_event: Option<ApplyEvent<'a>>,
    ) -> Result<BalancePhaseReport, TrainerError> {
        let mut events = CollectingEventSink::default();
        let mut run = BalancePhaseRun {
            phase,
            seeds,
            criterion: c,
            run_id,
            support_mode: None,
            deliver_reward: None,
            plasticity: None,
            reward_policy: BalanceRewardPolicy::Affect,
            data,
            apply_event,
        };
        run_balance_phase(
            &mut run,
            mock,
            &mut |_| Ok(()),
            &mut events,
            &CancelToken::new(),
        )
    }

    /// Events of one attempt as `(sim_time_s, kind, label)`, in arrival order.
    fn attempt_events(tape: &RecordingRunDataSink, index: u64) -> Vec<(f64, String, String)> {
        tape.events
            .iter()
            .filter(|e| e.at.episode == Some(index) && e.at.sim_time_s.is_some())
            .map(|e| {
                (
                    e.at.sim_time_s.unwrap_or(0.0),
                    e.kind.clone(),
                    e.label.clone(),
                )
            })
            .collect()
    }

    #[test]
    fn steps_table_carries_every_counted_burst_actuator_columns_and_the_attempt_events() {
        let mut c = criterion_for(2, 0.05);
        c.settle_s = 0.2;
        c.hold_s = 0.5;
        c.attempt_cap_s = 2.0;
        c.perturb_interval_s = 0.2;
        c.record_actuators = true;
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let mut tape = RecordingRunDataSink::default();
        let report = run_with_hooks(
            &c,
            &mut mock,
            "train",
            &[1, 2],
            &rid(),
            Some(&mut tape),
            None,
        )
        .expect("phase");
        assert_eq!(report.attempts.len(), 2);
        assert!(tape.errors.is_empty(), "{:?}", tape.errors);
        let col = |name: &str| tape.column_index("steps", name).expect(name);
        // One contact column per contact body (named from the criterion) and one per actuator.
        for body in &c.contact_bodies {
            assert!(tape.column_index("steps", &contact_column(body)).is_some());
        }
        let actuator = col("actuator_joint_a");
        let (attempt, settling, time) = (col("attempt"), col("settling"), col("sim_time"));
        let rows = tape.rows_of("steps");
        for index in 0..2u64 {
            let kinds: Vec<String> = attempt_events(&tape, index)
                .into_iter()
                .map(|e| e.1)
                .collect();
            assert_eq!(
                kinds.first().map(String::as_str),
                Some("attempt_start"),
                "{kinds:?}"
            );
            assert_eq!(
                kinds.last().map(String::as_str),
                Some("attempt_end"),
                "{kinds:?}"
            );
            for kind in ["settle_end", "perturb_start", "perturb_end"] {
                assert!(kinds.iter().any(|k| k == kind), "{kind} in {kinds:?}");
            }
            let end = attempt_events(&tape, index)
                .into_iter()
                .find(|e| e.1 == "attempt_end")
                .expect("end");
            assert_eq!(end.2, "success");
            let mine: Vec<&Vec<RunDataValue>> = rows
                .iter()
                .filter(|r| r[attempt] == RunDataValue::Int(index as i64))
                .collect();
            // Settling bursts are recorded (flagged); scored bursts match the record.
            let scored = mine
                .iter()
                .filter(|r| r[settling] == RunDataValue::Bool(false))
                .count();
            assert_eq!(scored as u32, report.attempts[index as usize].bursts);
            assert!(mine.iter().any(|r| r[settling] == RunDataValue::Bool(true)));
            let times: Vec<f64> = mine
                .iter()
                .map(|r| match r[time] {
                    RunDataValue::Float(t) => t,
                    ref other => panic!("{other:?}"),
                })
                .collect();
            assert!(times.windows(2).all(|w| w[1] > w[0]), "{times:?}");
        }
        assert!(rows
            .iter()
            .all(|r| matches!(r[actuator], RunDataValue::Float(_))));
        // The attempts table has one row per attempt.
        assert_eq!(tape.rows_of("attempts").len(), 2);
        let kinds = tape.event_kinds();
        assert_eq!(kinds.first(), Some(&"phase_start"));
        assert_eq!(kinds.last(), Some(&"phase_end"));
    }

    #[test]
    fn no_sink_leaves_the_run_unchanged_and_actuators_unrequested() {
        let c = criterion_for(2, 0.05);
        let mut with = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let mut without = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let mut tape = RecordingRunDataSink::default();
        let a =
            run_with_hooks(&c, &mut with, "train", &[3], &rid(), Some(&mut tape), None).expect("a");
        let b = run_with_hooks(&c, &mut without, "train", &[3], &rid(), None, None).expect("b");
        assert_eq!(a.attempts[0].bursts, b.attempts[0].bursts);
        assert_eq!(with.calls, without.calls);
        assert!(without.measure_requests.iter().all(|r| !r.actuators));
        assert!(tape
            .table("steps")
            .expect("steps")
            .columns
            .iter()
            .all(|spec| !spec.name.starts_with("actuator_")));
    }

    #[test]
    fn scheduled_events_fire_once_per_attempt_in_time_order_after_settle() {
        let mut c = criterion_for(2, 0.05);
        c.settle_s = 0.2;
        c.hold_s = 0.6;
        c.attempt_cap_s = 2.0;
        c.scheduled_events = vec![event(0.3, "late"), event(0.1, "early")];
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let mut tape = RecordingRunDataSink::default();
        let mut applied: Vec<String> = Vec::new();
        let mut apply = |action: &ScheduledAction| -> Result<(), TrainerError> {
            if let ScheduledAction::SetMappingPsc { src, .. } = action {
                applied.push(src.clone());
            }
            Ok(())
        };
        run_with_hooks(
            &c,
            &mut mock,
            "train",
            &[1, 2],
            &rid(),
            Some(&mut tape),
            Some(&mut apply),
        )
        .expect("phase");
        assert_eq!(applied.len(), 4);
        for index in 0..2u64 {
            let events = attempt_events(&tape, index);
            let fired: Vec<(f64, String)> = events
                .iter()
                .filter(|m| m.1 == "scheduled_event")
                .map(|m| (m.0, m.2.clone()))
                .collect();
            assert_eq!(fired.len(), 2, "{fired:?}");
            assert_eq!(fired[0].1, "early");
            assert_eq!(fired[1].1, "late");
            let settle_end = events
                .iter()
                .find(|m| m.1 == "settle_end")
                .expect("settle_end")
                .0;
            assert!(
                fired[0].0 >= settle_end + 0.1 - 1e-6,
                "{fired:?} {settle_end}"
            );
            assert!(
                fired[1].0 >= settle_end + 0.3 - 1e-6,
                "{fired:?} {settle_end}"
            );
        }
    }

    #[test]
    fn scheduled_events_need_a_callback_and_a_test_phase_refuses_them() {
        let mut c = criterion_for(2, 0.05);
        c.scheduled_events = vec![event(0.1, "evt")];
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let error = run_with_hooks(&c, &mut mock, "train", &[1], &rid(), None, None).unwrap_err();
        assert!(error.to_string().contains("apply_event"), "{error}");
        let mut apply = |_: &ScheduledAction| -> Result<(), TrainerError> { Ok(()) };
        let error = run_with_hooks(&c, &mut mock, "test", &[1], &rid(), None, Some(&mut apply))
            .unwrap_err();
        assert!(error.to_string().contains("scheduled events"), "{error}");
        assert_eq!(mock.count("reset_episode"), 0);
    }

    #[test]
    fn a_failing_event_callback_fails_the_run() {
        let mut c = criterion_for(2, 0.05);
        c.settle_s = 0.0;
        c.scheduled_events = vec![event(0.0, "evt")];
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let mut apply = |_: &ScheduledAction| -> Result<(), TrainerError> {
            Err(TrainerError::Runtime("mapping not found".to_string()))
        };
        let error = run_with_hooks(&c, &mut mock, "train", &[1], &rid(), None, Some(&mut apply))
            .unwrap_err();
        assert!(error.to_string().contains("mapping not found"), "{error}");
    }

    #[test]
    fn scheduled_event_validation_names_the_problem() {
        let mut c = criterion_for(2, 0.05);
        for bad in [-0.1, f64::NAN, f64::INFINITY] {
            c.scheduled_events = vec![event(bad, "evt")];
            assert!(
                c.validate().unwrap_err().to_string().contains("at_s"),
                "{bad}"
            );
        }
        c.scheduled_events = vec![event(1.0, "  ")];
        assert!(c.validate().unwrap_err().to_string().contains("label"));
        let mut bad = event(1.0, "evt");
        bad.action = ScheduledAction::SetMappingPsc {
            src: "area_a".to_string(),
            dst: String::new(),
            psc_multiplier: 1.0,
        };
        c.scheduled_events = vec![bad];
        assert!(c.validate().unwrap_err().to_string().contains("dst"));
        let mut bad = event(1.0, "evt");
        bad.action = ScheduledAction::SetAreaParameter {
            area: "area_a".to_string(),
            key: "".to_string(),
            value: serde_json::json!(0.5),
        };
        c.scheduled_events = vec![bad];
        assert!(c.validate().unwrap_err().to_string().contains("key"));
        c.scheduled_events = vec![event(0.0, "ok")];
        assert!(c.validate().is_ok());
    }

    #[test]
    fn scheduled_events_and_record_actuators_parse_from_json() {
        let json = serde_json::json!({
            "torso_body": "torso",
            "contact_bodies": ["contact_a"],
            "max_tilt_deg": 15.0,
            "hold_s": 1.0,
            "attempt_cap_s": 3.0,
            "recordActuators": true,
            "scheduledEvents": [
                {"at_s": 2.0, "label": "cut", "action":
                    {"type": "set_mapping_psc", "src": "area_a", "dst": "area_b", "psc_multiplier": 0.0}},
                {"at_s": 4.0, "label": "tune", "action":
                    {"type": "set_area_parameter", "area": "area_a", "key": "gain", "value": 0.3}}
            ]
        });
        let c: BalanceFeetCriterion = serde_json::from_value(json).expect("parse");
        assert!(c.record_actuators);
        assert_eq!(c.scheduled_events.len(), 2);
        assert!(c.validate().is_ok());
        let plain: BalanceFeetCriterion = serde_json::from_value(serde_json::json!({
            "torso_body": "torso", "contact_bodies": ["contact_a"],
            "max_tilt_deg": 15.0, "hold_s": 1.0, "attempt_cap_s": 3.0
        }))
        .expect("parse");
        assert!(plain.scheduled_events.is_empty() && !plain.record_actuators);
    }

    #[test]
    fn measure_request_sends_actuators_only_when_asked() {
        let mut c = criterion_for(2, 0.05);
        assert!(measure_request_body(&measure_request(&c))
            .get("actuators")
            .is_none());
        c.record_actuators = true;
        let body = measure_request_body(&measure_request(&c));
        assert_eq!(body["actuators"], serde_json::json!(true));
        let parsed: Measurement = serde_json::from_value(serde_json::json!({
            "sim_time_s": 0.1, "physics_steps": 5, "actuators": {"joint_a": 0.25}
        }))
        .expect("parse");
        assert_eq!(parsed.actuators["joint_a"], 0.25);
    }
}
