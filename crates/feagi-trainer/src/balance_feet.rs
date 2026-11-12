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

use crate::balance_recovery::{
    AttemptSlot, AttemptVerdict, AttemptWatch, BurstClock, DiscardedAttempt, LinkReading,
    RecoveryConfig, RecoveryState,
};
use crate::binding::reward::{AffectChannel, RewardSignal};
use crate::contracts::{
    AttemptStand, BackendFingerprint, BackendKind, ConnectomeHash, ContentHash, DatasetAssetId,
    EvaluationProtocolVersion, MetadataValue, MetricScope, PluginId, PluginRef, RunEvent,
    RunEventKind, RunId, Scorecard, ScorecardId, ScorecardStatus, ScorecardVisibility, SplitId,
};
use crate::control::{CancelToken, RunEventSink};
use crate::error::TrainerError;
use crate::memory_counters::{MemoryAreasRecorder, MemoryCounterSource};
use crate::metrics::episodic_control::EpisodicControlMetricPack;
use crate::plasticity_freeze::PlasticityControl;
use crate::reach_hold::trainer_package_version;
use crate::resume::{ResumeExport, ResumeFrom};
use crate::run_actions::{ApplyEvent, ScheduledEvent};
use crate::run_data::{name_of, ColumnSpec, EventTime, RunDataSink, RunDataValue, TableSpec};
use std::ops::Not;

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
/// Default `loss_margin_fraction`: the cone limit stays 20% inside the learned stance-loss angle.
fn default_loss_margin_fraction() -> f64 {
    0.2
}
/// Default `fail_tighten`: each fall or lost stance moves the assist halfway back to full.
fn default_fail_tighten() -> f64 {
    0.5
}
/// Default `loss_window`: the learned stance-loss angle uses the last 5 losses.
fn default_loss_window() -> u32 {
    5
}
/// Default `lean_grace_s`: a lean beyond the cone is re-centred after 2 s.
fn default_lean_grace_s() -> f64 {
    2.0
}
/// Default `idle_fraction`: the support may push up to 5% of the weight torque while the robot
/// still counts as standing by itself.
fn default_idle_fraction() -> f64 {
    0.05
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

/// Smallest net horizontal displacement, as a fraction of the torso's reset height, at which a
/// heading error is defined. Below it the direction of travel is noise.
pub const WALK_MIN_HEADING_DISPLACEMENT_HEIGHTS: f64 = 0.5;

fn default_walk_speed_tolerance() -> f64 {
    0.25
}

fn default_walk_heading_tolerance_deg() -> f64 {
    30.0
}

/// The walking goal of a session: forward progress at a pace along a heading. Everything is in SI
/// units or dimensionless; the torso and foot bodies come from the criterion.
///
/// Forward is the unit horizontal vector at the heading in the world frame (degrees from +X toward
/// +Y). With `forward_heading_deg` set that is a constant of the experiment. Left unset, each
/// attempt reads the torso's forward axis from the controller's `measure` reply (`heading`) at the
/// reset pose, so the goal follows the model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalkGoal {
    /// Target forward speed, m/s (> 0).
    pub speed_m_s: f64,
    /// Allowed deviation of the mean forward speed from the target, as a fraction of the target,
    /// in [0, 1). Default 0.25. It bounds both sides: slower is a stall, faster is a run.
    #[serde(default = "default_walk_speed_tolerance")]
    pub speed_tolerance: f64,
    /// Largest accepted heading error, degrees, in (0, 90]. Default 30.
    #[serde(default = "default_walk_heading_tolerance_deg")]
    pub heading_tolerance_deg: f64,
    /// World-frame direction of forward at the reset pose, degrees from +X toward +Y. Unset (the
    /// default) uses the torso's measured heading at each attempt's reset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forward_heading_deg: Option<f64>,
    /// Distance rule: forward displacement, m, that completes the attempt. Absent selects the
    /// time rule: an upright run of `hold_s` seconds.
    #[serde(default)]
    pub distance_m: Option<f64>,
}

impl WalkGoal {
    /// Goal with the tolerances, heading and rule at their defaults.
    pub fn new(speed_m_s: f64) -> Self {
        Self {
            speed_m_s,
            speed_tolerance: default_walk_speed_tolerance(),
            heading_tolerance_deg: default_walk_heading_tolerance_deg(),
            forward_heading_deg: None,
            distance_m: None,
        }
    }

    /// Slowest mean forward speed that passes, m/s.
    pub fn min_speed_m_s(&self) -> f64 {
        self.speed_m_s * (1.0 - self.speed_tolerance)
    }

    /// Fastest mean forward speed that passes, m/s. The same tolerance as the lower bound, so a
    /// run at twice the target is not scored as a walk.
    pub fn max_speed_m_s(&self) -> f64 {
        self.speed_m_s * (1.0 + self.speed_tolerance)
    }

    /// Forward unit vector in the world horizontal plane for a heading in degrees.
    pub fn forward_xy(heading_deg: f64) -> [f64; 2] {
        let heading = heading_deg.to_radians();
        [heading.cos(), heading.sin()]
    }

    /// The heading this attempt scores along, degrees: the explicit `forward_heading_deg`, else
    /// the torso's measured yaw at the reset pose. A reply without a heading is an error, never
    /// a silent 0.
    pub fn resolve_heading_deg(
        &self,
        torso: &str,
        reading: &BodyMeasure,
    ) -> Result<f64, TrainerError> {
        if let Some(explicit) = self.forward_heading_deg {
            return Ok(explicit);
        }
        reading
            .heading
            .as_ref()
            .and_then(|heading| heading.yaw_rad)
            .map(f64::to_degrees)
            .ok_or_else(|| {
                TrainerError::Runtime(format!(
                    "the walk heading is measured from {torso}, but the controller's measure reply has no \
                     defined heading for it (an older controller pack, or forward points along up); \
                     set forward_heading_deg (setup walkForwardHeadingDeg) or update the controller pack"
                ))
            })
    }

    /// Rejects a goal that cannot produce a success.
    pub fn validate(&self) -> Result<(), TrainerError> {
        let fail = |text: &str| Err(TrainerError::Config(text.to_string()));
        if !self.speed_m_s.is_finite() || self.speed_m_s <= 0.0 {
            return fail("walk speed_m_s must be finite and > 0");
        }
        if !self.speed_tolerance.is_finite() || !(0.0..1.0).contains(&self.speed_tolerance) {
            return fail("walk speed_tolerance must be a finite value in [0, 1)");
        }
        if !self.heading_tolerance_deg.is_finite()
            || self.heading_tolerance_deg <= 0.0
            || self.heading_tolerance_deg > 90.0
        {
            return fail("walk heading_tolerance_deg must be a finite value in (0, 90]");
        }
        if self.forward_heading_deg.is_some_and(|h| !h.is_finite()) {
            return fail("walk forward_heading_deg must be finite");
        }
        if self.distance_m.is_some_and(|d| !d.is_finite() || d <= 0.0) {
            return fail("walk distance_m must be finite and > 0");
        }
        Ok(())
    }

    /// Rejects a goal that heading cannot be judged for: the net displacement that completes the
    /// attempt must reach `WALK_MIN_HEADING_DISPLACEMENT_HEIGHTS` torso heights, else the heading
    /// stays undefined and no attempt can succeed. `reset_height_m` is runtime-only.
    pub fn check_feasible(&self, hold_s: f64, reset_height_m: f64) -> Result<(), TrainerError> {
        let needed = WALK_MIN_HEADING_DISPLACEMENT_HEIGHTS * reset_height_m;
        let (reach, what) = match self.distance_m {
            Some(distance) => (distance, "walk distance_m".to_string()),
            None => (
                self.min_speed_m_s() * hold_s,
                format!(
                    "the slowest passing speed ({:.4} m/s) times hold_s ({hold_s} s)",
                    self.min_speed_m_s()
                ),
            ),
        };
        if reach + 1.0e-12 < needed {
            return Err(TrainerError::Config(format!(
                "{what} gives {reach:.4} m, below {needed:.4} m ({WALK_MIN_HEADING_DISPLACEMENT_HEIGHTS} \
                 torso heights of {reset_height_m:.4} m); the heading error would stay undefined and \
                 no attempt could succeed. Raise the distance or hold_s, or the speed."
            )));
        }
        Ok(())
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
    /// The body that has to stay over the feet for the robot to count as upright (for example
    /// the head). Required, no default; distinct from every contact body. Upright is "head over
    /// feet": the posture line runs from the support centre (mean CoM position of the contact
    /// bodies touching now; all contact bodies when none touch) to this body's CoM. The scored
    /// posture tilt is the angle between that line and the same line on the attempt's first
    /// sample (the reset pose), turned by the heading change, so a stand pose that does not stack
    /// the top body over the feet is not counted as lean. The torso tilt stays reported as
    /// `tilt`, the absolute posture tilt as `posture_tilt`.
    ///
    /// The scored posture tilt drives the tolerance cone, the lean and success windows, the learned
    /// stance-loss tilt, and the fall and upright checks (`max_tilt_deg`, `fall_tilt_deg`). It is
    /// scored only: the support spring is never steered toward it, so the support resists
    /// falling past the cone and the brain must learn to bring the head over the feet.
    /// Sizing (`h0`) stays on the torso's own geometry; `support_need` reads the torso's lean from
    /// its own reset orientation. Absent in
    /// an old setup it is empty and `validate` rejects the criterion.
    #[serde(default)]
    pub posture_top_body: String,
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
    /// Adaptive: how far inside this body's measured stance-loss angle the cone may widen. The
    /// cone limit is `min(max_tilt_deg, learned_loss_tilt * (1 - loss_margin_fraction))`. In
    /// [0, 1); default 0.2.
    #[serde(default = "default_loss_margin_fraction")]
    pub loss_margin_fraction: f64,
    /// Adaptive: how much each fall or lost stance tightens the assist
    /// (`a_slow += (1 - a_slow) * fail_tighten`). In [0, 1]; default 0.5.
    #[serde(default = "default_fail_tighten")]
    pub fail_tighten: f64,
    /// Adaptive: how many recent losses the learned stance-loss angle is based on (its 25th
    /// percentile). >= 1; default 5.
    #[serde(default = "default_loss_window")]
    pub loss_window: u32,
    /// Adaptive: assist increase within one burst that counts as a catch. In [0, 1]; default 0.05.
    #[serde(default = "default_catch_threshold")]
    pub catch_threshold: f64,
    /// Adaptive: seconds the predicted tilt may stay beyond the reference cone, without a loss,
    /// before the assist tightens (`a_slow += (1 - a_slow) * fail_tighten`). > 0; default 2.
    #[serde(default = "default_lean_grace_s")]
    pub lean_grace_s: f64,
    /// Adaptive: how little the support may push while the robot still counts as standing by
    /// itself: a success window needs the support effort at or below this fraction (0.05 = 5% of
    /// the weight torque). In [0, 1]; default 0.05.
    #[serde(default = "default_idle_fraction")]
    pub idle_fraction: f64,
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
    /// Walking goal. `None` (default) is the standing task. With a goal, a stance needs at least
    /// one contact body down (feet lift in a gait) and success is forward progress at a pace.
    #[serde(default)]
    pub walk: Option<WalkGoal>,
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
        posture_top_body: impl Into<String>,
        max_tilt_deg: f64,
        hold_s: f64,
        attempt_cap_s: f64,
    ) -> Self {
        Self {
            torso_body: torso_body.into(),
            contact_bodies,
            posture_top_body: posture_top_body.into(),
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
            loss_margin_fraction: default_loss_margin_fraction(),
            fail_tighten: default_fail_tighten(),
            loss_window: default_loss_window(),
            catch_threshold: default_catch_threshold(),
            lean_grace_s: default_lean_grace_s(),
            idle_fraction: default_idle_fraction(),
            stance_grace_s: default_stance_grace_s(),
            standing_fraction_for_success: default_standing_fraction_for_success(),
            scheduled_events: Vec::new(),
            record_actuators: false,
            walk: None,
        }
    }

    /// What the posture tilt (and so the saved stance-loss tilts) is measured against: the top
    /// body and the contact bodies that set the support centre. Changing it makes the saved loss
    /// history meaningless.
    pub fn posture_definition(&self) -> String {
        let mut contacts: Vec<&str> = self.contact_bodies.iter().map(|n| n.trim()).collect();
        contacts.sort_unstable();
        // `rel`: tilts are measured from the reset pose; saved loss tilts from the older
        // from-vertical measure do not carry over.
        format!(
            "rel|{}|{}",
            self.posture_top_body.trim(),
            contacts.join(",")
        )
    }

    /// The contact bodies satisfy the stance: every one touches the world, or at least one when a
    /// walking goal is set (feet lift in a gait). An empty reading satisfies nothing.
    pub fn contacts_satisfied(&self, contacts: &[bool]) -> bool {
        if self.walk.is_some() {
            contacts.iter().any(|down| *down)
        } else {
            contacts.iter().all(|down| *down)
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
        let top = self.posture_top_body.trim();
        if top.is_empty() {
            return Err(TrainerError::Config(
                "posture_top_body is required: name the body that has to stay over the feet for \
                 the robot to count as upright (for example the head); setups saved before this \
                 setting existed must set it"
                    .to_string(),
            ));
        }
        if self.contact_bodies.iter().any(|name| name.trim() == top) {
            return Err(TrainerError::Config(format!(
                "posture_top_body {top} must not be one of the contact_bodies"
            )));
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
        if !self.loss_margin_fraction.is_finite()
            || !(0.0..1.0).contains(&self.loss_margin_fraction)
        {
            return Err(TrainerError::Config(
                "loss_margin_fraction must be a finite value in [0, 1)".to_string(),
            ));
        }
        if !self.fail_tighten.is_finite() || !(0.0..=1.0).contains(&self.fail_tighten) {
            return Err(TrainerError::Config(
                "fail_tighten must be a finite value in [0, 1]".to_string(),
            ));
        }
        if self.loss_window == 0 {
            return Err(TrainerError::Config("loss_window must be >= 1".to_string()));
        }
        if !self.catch_threshold.is_finite() || !(0.0..=1.0).contains(&self.catch_threshold) {
            return Err(TrainerError::Config(
                "catch_threshold must be a finite value in [0, 1]".to_string(),
            ));
        }
        if !self.idle_fraction.is_finite() || !(0.0..=1.0).contains(&self.idle_fraction) {
            return Err(TrainerError::Config(
                "idle_fraction must be a finite value in [0, 1]".to_string(),
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
        if let Some(walk) = &self.walk {
            walk.validate()?;
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

/// What a [`SupportSchedule`] carries between attempts, saved at a pause.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SupportScheduleState {
    /// Level to send with the next attempt.
    pub level: f64,
    /// Attempts in the open block.
    pub block_attempts: u32,
    /// Successes in the open block.
    pub block_successes: u32,
    /// `balanced_s` of each attempt in the open block.
    pub block_balanced_s: Vec<f64>,
    /// Failed blocks in a row.
    pub failed_blocks_in_a_row: u32,
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

    /// Counters and level to save for a later segment of the same run.
    pub fn state(&self) -> SupportScheduleState {
        SupportScheduleState {
            level: self.level,
            block_attempts: self.block_attempts,
            block_successes: self.block_successes,
            block_balanced_s: self.block_balanced_s.clone(),
            failed_blocks_in_a_row: self.failed_blocks_in_a_row,
        }
    }

    /// Continue from a saved state, block counters included. The configuration stays the
    /// run's own; the level is clamped to `[config.min, config.start]`.
    pub fn restore(config: SupportConfig, state: SupportScheduleState) -> Self {
        let mut schedule = Self::resume(config, state.level);
        schedule.block_attempts = state.block_attempts;
        schedule.block_successes = state.block_successes;
        schedule.block_balanced_s = state.block_balanced_s;
        schedule.failed_blocks_in_a_row = state.failed_blocks_in_a_row;
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
#[derive(Debug, Clone, PartialEq)]
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

/// What the adaptive assist carries between attempts and train runs: the level, its slow
/// average and the tilts at which recent attempts lost their stance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdaptiveAssistState {
    /// Assist level now, in `[support_min, support_start]`.
    pub a: f64,
    /// Exponential average of `a` with time constant `fade_tau_s`; it sets the cone.
    pub a_slow: f64,
    /// Tilt, degrees, at which each recent fall or lost stance began (oldest first, at most
    /// `loss_window` kept). Older saved states have none.
    #[serde(default)]
    pub loss_tilts_deg: Vec<f64>,
    /// What those tilts were measured against (`BalanceFeetCriterion::posture_definition`: the
    /// posture top body and the contact bodies). When the running criterion's definition
    /// differs, or an older state has none, the loss history is dropped on resume, because
    /// tilts of another definition say nothing about this one.
    #[serde(default)]
    pub posture_definition: Option<String>,
}

impl BalanceFeetCriterion {
    /// The support is idle: its effort (0..1 of the weight torque) is at or below
    /// `idle_fraction`. An absent effort reading is not idle. The one test behind success
    /// windows and every number that reports them.
    pub fn support_is_idle(&self, effort: Option<f64>) -> bool {
        effort.is_some_and(|e| e <= self.idle_fraction + 1.0e-12)
    }
}

impl AdaptiveAssistState {
    /// A state with a level and its slow average and no recorded losses.
    pub fn new(a: f64, a_slow: f64) -> Self {
        Self {
            a,
            a_slow,
            loss_tilts_deg: Vec::new(),
            posture_definition: None,
        }
    }
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
/// the cone as the slow level `a_slow` falls.
///
/// Teaching method: start at a very low allowed angle (`cone_min_deg`) and widen only as the
/// robot itself succeeds. `a_slow` does not follow `a` over time: it steps toward `a` only per
/// success window (`success_window`: `hold_s` seconds inside the cone with the support idle),
/// so a passive lean never widens the cone. A lean past the cone for longer than
/// `lean_grace_s` tightens it again (`lean_retighten`), as does each fall or lost stance. The
/// one cone from `cone_deg` is both the spring's dead band and the scored cone. The cone widens toward an effective
/// maximum `min(max_tilt_deg, learned_loss_tilt * (1 - loss_margin_fraction))`, where the
/// learned loss tilt is the 25th percentile of the tilts at which the last `loss_window` falls
/// or lost stances began (no limit from losses before the first loss). Each fall or lost
/// stance also moves `a_slow` back toward 1 by `fail_tighten` and lifts `a` to it, and the next
/// attempt starts there. `max_tilt_deg` itself is unchanged: it still defines upright, success
/// and the support need. Every time constant is in seconds and meets the burst rate only
/// through `burst_period_s`, so it means the same at any burst rate. Metrics observe the level
/// and the cone; only losses and the tilt overshoot feed back into them.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAssist {
    a: f64,
    a_slow: f64,
    hold_left_s: f64,
    loss_tilts_deg: Vec<f64>,
    posture_definition: String,
    cone_min_clamped: bool,
}

impl AdaptiveAssist {
    /// Full assist: both averages at `support_start`.
    pub fn start(criterion: &BalanceFeetCriterion) -> Self {
        Self {
            a: criterion.support_start,
            a_slow: criterion.support_start,
            hold_left_s: 0.0,
            loss_tilts_deg: Vec::new(),
            posture_definition: criterion.posture_definition(),
            cone_min_clamped: false,
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
        let definition = criterion.posture_definition();
        let same_posture = state.posture_definition.as_deref() == Some(definition.as_str());
        let mut assist = Self {
            a: clamp(state.a, criterion.support_start),
            a_slow: clamp(state.a_slow, criterion.support_start),
            hold_left_s: 0.0,
            loss_tilts_deg: if same_posture {
                state
                    .loss_tilts_deg
                    .into_iter()
                    .filter(|tilt| tilt.is_finite() && *tilt >= 0.0)
                    .collect()
            } else {
                Vec::new()
            },
            posture_definition: definition,
            cone_min_clamped: false,
        };
        assist.trim_losses(criterion);
        assist.cone_min_clamped = assist
            .effective_max_raw(criterion)
            .is_some_and(|raw| raw < criterion.cone_min_deg);
        assist
    }

    /// The parts of the assist that [`AdaptiveAssistState`] does not hold, for a pause inside one
    /// run: the hold timer and the cone clamp flag.
    pub fn pause_extras(&self) -> (f64, bool) {
        (self.hold_left_s, self.cone_min_clamped)
    }

    /// Put back what [`Self::pause_extras`] saved (non-finite or negative timers are ignored).
    pub fn restore_pause_extras(&mut self, hold_left_s: f64, cone_min_clamped: bool) {
        if hold_left_s.is_finite() && hold_left_s >= 0.0 {
            self.hold_left_s = hold_left_s;
        }
        self.cone_min_clamped |= cone_min_clamped;
    }

    /// State to save for a later attempt or run.
    pub fn state(&self) -> AdaptiveAssistState {
        AdaptiveAssistState {
            a: self.a,
            a_slow: self.a_slow,
            loss_tilts_deg: self.loss_tilts_deg.clone(),
            posture_definition: Some(self.posture_definition.clone()),
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

    /// `cone_min_deg` was above the learned limit at some point (the limit was held at it).
    pub fn cone_min_clamped(&self) -> bool {
        self.cone_min_clamped
    }

    fn trim_losses(&mut self, criterion: &BalanceFeetCriterion) {
        let keep = criterion.loss_window.max(1) as usize;
        if self.loss_tilts_deg.len() > keep {
            let drop = self.loss_tilts_deg.len() - keep;
            self.loss_tilts_deg.drain(..drop);
        }
    }

    /// Learned stance-loss tilt, degrees: the 25th percentile (linear interpolation) of the
    /// last `loss_window` loss tilts. `None` before the first loss.
    pub fn learned_loss_tilt_deg(&self, criterion: &BalanceFeetCriterion) -> Option<f64> {
        let keep = criterion.loss_window.max(1) as usize;
        let start = self.loss_tilts_deg.len().saturating_sub(keep);
        let mut tilts: Vec<f64> = self.loss_tilts_deg[start..].to_vec();
        if tilts.is_empty() {
            return None;
        }
        tilts.sort_by(f64::total_cmp);
        let rank = 0.25 * (tilts.len() - 1) as f64;
        let low = rank.floor() as usize;
        let high = rank.ceil() as usize;
        Some(tilts[low] + (tilts[high] - tilts[low]) * (rank - low as f64))
    }

    /// `learned_loss_tilt * (1 - loss_margin_fraction)` before the `max_tilt_deg` and
    /// `cone_min_deg` bounds; `None` before the first loss.
    fn effective_max_raw(&self, criterion: &BalanceFeetCriterion) -> Option<f64> {
        self.learned_loss_tilt_deg(criterion)
            .map(|loss| loss * (1.0 - criterion.loss_margin_fraction))
    }

    /// Effective maximum of the cone, degrees: `max_tilt_deg` until the first loss, then
    /// `min(max_tilt_deg, learned_loss_tilt * (1 - loss_margin_fraction))`, never below
    /// `cone_min_deg`.
    pub fn effective_max_deg(&self, criterion: &BalanceFeetCriterion) -> f64 {
        let max = criterion.max_tilt_deg;
        let limit = self
            .effective_max_raw(criterion)
            .map_or(max, |raw| raw.min(max));
        limit.max(criterion.cone_min_deg)
    }

    /// Tolerance cone in degrees:
    /// `cone_min + (effective_max - cone_min) * (1 - a_slow)`.
    pub fn cone_deg(&self, criterion: &BalanceFeetCriterion) -> f64 {
        let min = criterion.cone_min_deg;
        let max = self.effective_max_deg(criterion);
        (min + (max - min) * (1.0 - self.a_slow)).clamp(min, max)
    }

    /// One success window completed (`hold_s` seconds inside the cone with the support idle):
    /// `a_slow` steps toward `a` by `1 - exp(-hold_s / fade_tau_s)`. Only success moves it down.
    pub fn success_window(&mut self, criterion: &BalanceFeetCriterion) {
        let share = 1.0 - (-criterion.hold_s / criterion.fade_tau_s).exp();
        self.a_slow += (self.a - self.a_slow) * share;
        self.a_slow = self
            .a_slow
            .clamp(criterion.support_min, criterion.support_start);
    }

    /// A lean past the cone outlasted `lean_grace_s` without a loss: `a_slow` moves toward 1
    /// by `fail_tighten` and `a` rises to it.
    pub fn lean_retighten(&mut self, criterion: &BalanceFeetCriterion) {
        let tightened = self.a_slow + (1.0 - self.a_slow) * criterion.fail_tighten;
        self.a_slow = tightened.clamp(criterion.support_min, criterion.support_start);
        self.a = self
            .a
            .max(self.a_slow)
            .clamp(criterion.support_min, criterion.support_start);
        self.hold_left_s = 0.0;
    }

    /// Apply the end of an attempt. A fall or a lost stance records the tilt at which the
    /// stance went (`loss_tilt_deg`, when known) and tightens the assist: `a_slow` moves toward
    /// 1 by `fail_tighten`, `a` rises to it, and the next attempt starts there. Other outcomes
    /// change nothing.
    pub fn record_attempt_end(
        &mut self,
        criterion: &BalanceFeetCriterion,
        outcome: BalanceAttemptOutcome,
        loss_tilt_deg: Option<f64>,
    ) {
        if !matches!(
            outcome,
            BalanceAttemptOutcome::Fell | BalanceAttemptOutcome::LostStance
        ) {
            return;
        }
        if let Some(tilt) = loss_tilt_deg.filter(|t| t.is_finite() && *t >= 0.0) {
            self.loss_tilts_deg.push(tilt);
            self.trim_losses(criterion);
        }
        if self
            .effective_max_raw(criterion)
            .is_some_and(|raw| raw < criterion.cone_min_deg)
        {
            self.cone_min_clamped = true;
        }
        let tightened = self.a_slow + (1.0 - self.a_slow) * criterion.fail_tighten;
        self.a_slow = tightened.clamp(criterion.support_min, criterion.support_start);
        self.a = self
            .a
            .max(self.a_slow)
            .clamp(criterion.support_min, criterion.support_start);
        self.hold_left_s = 0.0;
    }

    /// One update from the predicted tilt (degrees) of `bursts` elapsed bursts (normally 1).
    pub fn step(
        &mut self,
        criterion: &BalanceFeetCriterion,
        predicted_deg: f64,
        bursts: u32,
    ) -> AdaptiveStep {
        let dt = criterion.burst_period_s * f64::from(bursts.max(1));
        let cone = self.cone_deg(criterion);
        let span = self.effective_max_deg(criterion) - cone;
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

/// Predicted tilt in degrees (the support-need prediction, not clamped).
///
/// With the signed tilt vector `r = (rx, ry)` (see `TiltComponentsDeg`) the lean is predicted
/// along its own direction: `|r + lookahead * dr/dt|`. The controller reports `omega` in the
/// world frame and the tilt vector is `up x z`, so for small tilt `dr/dt = (-wx, -wy)`; a body
/// swinging back toward upright therefore predicts less than its current tilt and one tipping
/// further predicts more. Without the tilt vector (older controllers) it falls back to
/// `tilt + lookahead * |omega_xy|`, which counts the swing in any direction as tipping.
pub fn predicted_tilt_deg(
    tilt_rad: f64,
    tilt_vec: Option<[f64; 2]>,
    ang_vel: [f64; 3],
    lookahead_s: f64,
) -> f64 {
    match tilt_vec.filter(|r| r.iter().all(|v| v.is_finite())) {
        Some([rx, ry]) => (rx - lookahead_s * ang_vel[0])
            .hypot(ry - lookahead_s * ang_vel[1])
            .to_degrees(),
        None => (tilt_rad + lookahead_s * ang_vel[0].hypot(ang_vel[1])).to_degrees(),
    }
}

/// The head-over-feet reading of one `measure` reply: the support centre is the mean CoM of the
/// contact bodies touching now (all of them when none touch), the top is the posture top body.
pub fn measure_posture(
    reading: &Measurement,
    criterion: &BalanceFeetCriterion,
) -> Result<PostureReading, TrainerError> {
    let com_of = |name: &str| -> Result<[f64; 3], TrainerError> {
        reading
            .bodies
            .get(name.trim())
            .and_then(|body| body.com)
            .ok_or_else(|| {
                TrainerError::Runtime(format!(
                    "the posture tilt needs the position of {}: the measure reply has no com for it",
                    name.trim()
                ))
            })
    };
    let feet = criterion
        .contact_bodies
        .iter()
        .map(|name| com_of(name))
        .collect::<Result<Vec<_>, _>>()?;
    let touching: Vec<bool> = reading
        .contacts
        .iter()
        .take(feet.len())
        .map(|set| set.touching)
        .collect();
    let top = com_of(&criterion.posture_top_body)?;
    let centre = support_centre(&feet, &touching).ok_or_else(|| {
        TrainerError::Runtime("the posture tilt needs at least one contact body".to_string())
    })?;
    posture_reading(centre, top).ok_or_else(|| {
        TrainerError::Runtime(format!(
            "posture top body {} sits at the support centre: no line to measure",
            criterion.posture_top_body.trim()
        ))
    })
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

/// Collects the 1 Hz deviation and assistance timeline of a train phase. Cloned before an
/// attempt so a discarded attempt leaves no trace in it.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviationAssistRun {
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
    /// Success windows completed: `hold_s` seconds inside the cone with the support idle.
    #[serde(default)]
    pub success_windows: u32,
    /// Share of the scored time the success windows cover: whole `hold_s` windows, plus the
    /// trailing part of a window that continues unbroken from a completed one.
    #[serde(default)]
    pub window_cover_fraction: f64,
    /// Why a capped standing attempt was not a success, when the cover rule demoted it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub demotion_reason: Option<WindowDemotion>,
    /// Times a lean past the cone outlasted `lean_grace_s` and tightened the assist.
    #[serde(default)]
    pub lean_retightens: u32,
    /// The attempt ended as `lost_stance`.
    #[serde(default)]
    pub lost_stance: bool,
    /// Assist level at the start of the attempt (after any tightening from the last loss).
    #[serde(default)]
    pub a_start: f64,
    /// Slow average of the assist level at the start of the attempt.
    #[serde(default)]
    pub a_slow_start: f64,
    /// Effective maximum of the cone at the start of the attempt, degrees.
    #[serde(default)]
    pub effective_max_deg: f64,
    /// Learned stance-loss tilt at the start of the attempt, degrees. Absent before the first loss.
    #[serde(default)]
    pub learned_loss_tilt_deg: Option<f64>,
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

/// The support spring an attempt opened with (the values sent in `set_body_spring`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpringApplied {
    /// Level the opening spring was set to (0 for a brain-only attempt, where the spring has
    /// zero gains).
    pub level: f64,
    /// Dead band of the rotational spring, rad.
    pub deadband_rad: f64,
}

/// Body measures of one observed attempt, in units that carry across bodies: angles are
/// fractions of the allowed angle (`max_tilt_deg`), times are seconds from the start of scoring.
/// Only the torso's own tilt, angular velocity and the contact readings feed them; no simulator
/// centre-of-mass reading is involved.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ObserveBodyMeasures {
    /// The allowed angle the fractions are taken against, degrees.
    pub allowed_angle_deg: f64,
    /// Seconds until the torso tilt first passed the allowed angle. Absent when it never did.
    pub time_to_limit_s: Option<f64>,
    /// Largest tilt seen, as a fraction of the allowed angle (above 1 means it passed it).
    pub peak_tilt_fraction: f64,
    /// `tilt_sway_deg` as a fraction of the allowed angle. Lower is steadier.
    #[serde(default)]
    pub tilt_sway_fraction: Option<f64>,
    /// `wobble_rad_s` in allowed angles per second. Lower is steadier.
    #[serde(default)]
    pub wobble_fraction_per_s: Option<f64>,
    /// Lean toward the torso's forward at the last scored sample, as a fraction of the allowed
    /// angle (negative leans back). Present only when that sample was past the allowed angle.
    #[serde(default)]
    pub fall_forward_fraction: Option<f64>,
    /// Lean toward the torso's left at the last scored sample, as a fraction of the allowed
    /// angle (negative leans right). Present only when that sample was past the allowed angle.
    #[serde(default)]
    pub fall_left_fraction: Option<f64>,
    /// Direction of that lean, degrees: 0 forward, 90 left, 180 back, -90 right.
    #[serde(default)]
    pub fall_direction_deg: Option<f64>,
}

/// What an observed attempt tracks while it runs, to build [`ObserveBodyMeasures`].
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ObserveTrack {
    time_to_limit_s: Option<f64>,
    peak_tilt_deg: f64,
    last_tilt_deg: f64,
    last_lean: TiltComponentsDeg,
}

impl ObserveTrack {
    /// Fold one scored (non-settling) sample, `scored_s` seconds after scoring began.
    pub fn add(&mut self, sample: &BalanceSample, allowed_angle_deg: f64, scored_s: f64) {
        if self.time_to_limit_s.is_none() && sample.scored_tilt_deg() > allowed_angle_deg {
            self.time_to_limit_s = Some(scored_s.max(0.0));
        }
        self.peak_tilt_deg = self.peak_tilt_deg.max(sample.scored_tilt_deg());
        self.last_tilt_deg = sample.scored_tilt_deg();
        self.last_lean = match sample.posture {
            Some(p) => TiltComponentsDeg {
                x: None,
                y: None,
                forward: p.forward_deg,
                left: p.left_deg,
            },
            None => sample
                .stability
                .as_ref()
                .map(StabilitySample::tilt_components_deg)
                .unwrap_or_default(),
        };
    }

    /// The measures of the finished attempt `record`, against `allowed_angle_deg` (must be > 0).
    pub fn measures(
        &self,
        record: &BalanceAttemptRecord,
        allowed_angle_deg: f64,
    ) -> ObserveBodyMeasures {
        let finite = |v: f64| v.is_finite().then_some(v);
        let fall = self.last_tilt_deg > allowed_angle_deg;
        let lean = |deg: Option<f64>| {
            deg.filter(|_| fall)
                .and_then(|deg| finite(deg / allowed_angle_deg))
        };
        let forward = lean(self.last_lean.forward);
        let left = lean(self.last_lean.left);
        ObserveBodyMeasures {
            allowed_angle_deg,
            time_to_limit_s: self.time_to_limit_s,
            peak_tilt_fraction: self.peak_tilt_deg / allowed_angle_deg,
            tilt_sway_fraction: record
                .tilt_sway_deg
                .and_then(|deg| finite(deg / allowed_angle_deg)),
            wobble_fraction_per_s: record
                .wobble_rad_s
                .and_then(|rad_s| finite(rad_s / allowed_angle_deg.to_radians())),
            fall_forward_fraction: forward,
            fall_left_fraction: left,
            fall_direction_deg: match (forward, left) {
                (Some(f), Some(l)) => Some(l.atan2(f).to_degrees()),
                _ => None,
            },
        }
    }
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
    /// Posture tilt at the terminal burst, in degrees (the tilt the upright and fall rules score).
    pub final_tilt_deg: f64,
    /// Absolute posture lean on the attempt's first sample (the reset pose), degrees, from world
    /// vertical. `final_tilt_deg` and every tilt rule are measured from this pose. Absent when no
    /// posture line was read.
    #[serde(default)]
    pub reset_tilt_deg: Option<f64>,
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
    /// Non-settling bursts elapsed when the body was first confirmed collapsed: not upright
    /// (see [`non_upright_reasons`]) continuously for `fall_confirm_s`. Independent of
    /// `fall_tilt_deg`, so it is set even when the fall rule is off. Absent if never collapsed.
    #[serde(default)]
    pub collapsed_at_bursts: Option<u32>,
    /// `collapsed_at_bursts` times `burst_period_s`: seconds from the end of settling.
    #[serde(default)]
    pub collapsed_at_s: Option<f64>,
    /// RMS of the torso's horizontal angular velocity, `hypot(wx, wy)`, rad/s, over the counted
    /// upright bursts after settling. Lower is steadier. Absent when no such burst was measured.
    #[serde(default)]
    pub wobble_rad_s: Option<f64>,
    /// RMS deviation of the torso tilt vector from its mean over the same bursts, degrees.
    /// Lower is steadier. Absent when no such burst was measured.
    #[serde(default)]
    pub tilt_sway_deg: Option<f64>,
    /// RMS horizontal deviation of the subtree CoM from its mean over the same bursts, as a
    /// fraction of the reset torso height. Lower is steadier. Absent when no burst was measured
    /// or the reset height is unknown.
    #[serde(default)]
    pub com_sway_fraction: Option<f64>,
    /// Posture tilt, degrees, on the first burst of the stance loss that was still going on when
    /// the attempt ended (the first not-standing burst of the final off-stance run). Absent when
    /// the stance was standing at the end.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stance_lost_tilt_deg: Option<f64>,
    /// Adaptive-assist numbers. Absent for a ladder attempt and for probes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adaptive: Option<AdaptiveAttemptStats>,
    /// Walk numbers. Absent for a standing attempt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub walk: Option<WalkAttemptStats>,
    /// The opening support spring as sent to the controller. Absent only for records not built
    /// by a phase run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spring_applied: Option<SpringApplied>,
    /// Body measures of an observe attempt. Absent in every other phase.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observe: Option<ObserveBodyMeasures>,
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
    let feet_off_body_up = !criterion.contacts_satisfied(&sample.contacts) && !torso_low;
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

/// Inputs of the documented gain-sizing formulas. Recorded in the run data beside the sized
/// gains.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SizingInputs {
    /// Whole-robot mass m: subtree mass of the torso's free root, kg.
    pub mass_kg: f64,
    /// Gravity magnitude g, m/s^2.
    pub gravity: f64,
    /// Torso subtree CoM height above the mean contact point h0, m.
    pub h0_m: f64,
    /// Tilt inertia: max(x, y) of the torso root's subtree inertia, kg*m^2.
    pub i_tilt: f64,
    /// Smallest principal inertia of the torso root's subtree, kg*m^2 (recorded, not used by the
    /// sizing formulas).
    pub i_min: f64,
    /// Smallest principal inertia of the torso body alone, kg*m^2 (recorded, not used by the
    /// sizing formulas).
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
/// anti-tilt torque `k_rot = 3 m g h0 / 0.087` (so `k_rot * 0.087 = 3 m g h0`: a 5 degree tilt
/// costs about a third of the weight torque), `d_rot = 0.8 sqrt(k_rot I_tilt)`. `I_tilt` comes
/// from the torso root's subtree inertia. The simulator applies the rotational torque
/// implicitly (backward Euler against the body's operational-space inertia), so any `k_rot` and
/// `d_rot` are stable and no cap by the timestep or the torso's own inertia is needed. There are
/// no linear position springs. Weight support is a constant upward force
/// `level * bws_max_fraction * m_total * g` on the torso body, with `m_total` the whole robot's
/// mass; here `gravity_comp` is the default fraction 0.5.
pub fn size_spring_gains(inputs: &SizingInputs) -> SpringGains {
    let weight = inputs.mass_kg * inputs.gravity;
    let k_rot = 3.0 * weight * inputs.h0_m / 0.087;
    let d_rot = sized_d_rot(k_rot, inputs);
    SpringGains {
        k_rot,
        d_rot,
        gravity_comp: MAX_BWS_FRACTION,
    }
}

fn sized_d_rot(k_rot: f64, inputs: &SizingInputs) -> f64 {
    0.8 * (k_rot * inputs.i_tilt).sqrt()
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
    /// Also report every actuator's applied force. Sent only when true, as `actuator_force`.
    pub actuator_force: bool,
}

/// One body in a `measure` reply.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BodyMeasure {
    /// Angle between the body-local up axis and -gravity, rad.
    pub tilt_rad: f64,
    /// Tilt vector, rad.
    #[serde(default)]
    pub tilt_vec: Option<[f64; 2]>,
    /// Angular velocity, rad/s.
    #[serde(default)]
    pub ang_vel: Option<[f64; 3]>,
    /// Body CoM, m.
    #[serde(default)]
    pub com: Option<[f64; 3]>,
    /// Subtree CoM, m.
    #[serde(default)]
    pub subtree_com: Option<[f64; 3]>,
    /// Forward axis of the body in the world frame (newer controllers).
    #[serde(default)]
    pub heading: Option<BodyHeading>,
}

/// The body's forward direction in a `measure` reply.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BodyHeading {
    /// Angle of forward about up, rad, from world +X toward +Y for z-up; `None` when forward
    /// points along up and the horizontal direction is undefined.
    #[serde(default)]
    pub yaw_rad: Option<f64>,
    /// Unit forward vector, world frame.
    #[serde(default)]
    pub forward: Option<[f64; 3]>,
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
    /// Applied actuator force by actuator name (same keys as `actuators`); present only when
    /// requested and supported by the controller.
    #[serde(default)]
    pub actuator_force: BTreeMap<String, f64>,
    /// The controller's FEAGI link at the time of the reading. Required: a reply without it is
    /// an error, because without it the attempt cannot be judged.
    pub link: LinkReading,
}

/// A `reset_episode` reply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResetReply {
    /// Id of the episode the reset started.
    pub episode_id: u64,
    /// The controller's FEAGI link at the time of the reset (required in the reply).
    pub link: LinkReading,
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
    /// Torso tilt from world up, degrees. Reported as `tilt`; scoring uses
    /// [`BalanceSample::scored_tilt_deg`] (the posture tilt).
    pub tilt_deg: f64,
    /// Head-over-feet posture of this burst (see [`PostureSample`]). The Trainer always sets it;
    /// absent only on a sample built without it, which then scores on `tilt_deg`.
    #[serde(default)]
    pub posture: Option<PostureSample>,
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
    /// Applied actuator force by actuator name at this burst. Empty unless the criterion sets
    /// `record_actuators` and the controller reports it.
    #[serde(default)]
    pub actuator_force: BTreeMap<String, f64>,
    /// Stability readings of the torso at this burst. Absent when not measured.
    #[serde(default)]
    pub stability: Option<StabilitySample>,
    /// Forward progress readings of the torso. Present only for a walking goal.
    #[serde(default)]
    pub walk: Option<WalkReading>,
}

/// Where the torso is relative to where the attempt started, along and across forward.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WalkReading {
    /// Displacement along forward since the attempt's first scored burst, m.
    pub forward_m: f64,
    /// Displacement to the left of forward since the same burst, m.
    pub lateral_m: f64,
    /// New ground gained along forward since the previous counted burst, m: how far `forward_m`
    /// rose above its best so far. Rocking over planted feet gains none. Filled for counted bursts.
    #[serde(default)]
    pub progress_m: f64,
}

/// Running walk readings of one attempt: pace window, steps and foot alternation.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WalkAccum {
    /// Forward displacement at the latest counted burst, m.
    pub last_forward_m: f64,
    /// Lateral displacement at the latest counted burst, m.
    pub last_lateral_m: f64,
    /// `(scored bursts, forward displacement)` at recent counted bursts: the anchor of the speed
    /// window (the latest entry at least `hold_s` old) and everything after it.
    pub history: std::collections::VecDeque<(u32, f64)>,
    /// Contact bodies down at the latest counted burst.
    pub last_down: Option<Vec<bool>>,
    /// Touchdowns so far (rising edge of any contact body).
    pub steps: u32,
    /// Index of the contact body of the latest lone touchdown.
    pub last_touchdown: Option<usize>,
    /// Lone touchdowns that followed another lone touchdown.
    pub touchdown_pairs: u32,
    /// Of those, the ones by a different contact body than the previous touchdown.
    pub alternations: u32,
}

/// Walk numbers of one finished attempt.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WalkAttemptStats {
    /// Forward displacement from the first scored burst, m.
    pub forward_m: f64,
    /// Lateral displacement (left positive), m.
    pub lateral_m: f64,
    /// `forward_m` over the scored time, m/s.
    pub mean_speed_m_s: f64,
    /// Speed of the window the success rule judges, m/s: the whole attempt for the distance rule,
    /// the last `hold_s` seconds for the time rule. Absent when no such window exists yet.
    pub window_speed_m_s: Option<f64>,
    /// `(mean_speed - target) / target`.
    pub speed_error_fraction: f64,
    /// Angle between the net horizontal displacement and forward, degrees. Absent while the
    /// displacement is under `WALK_MIN_HEADING_DISPLACEMENT_HEIGHTS` torso heights.
    pub heading_error_deg: Option<f64>,
    /// Touchdowns of any contact body.
    pub steps: u32,
    /// Touchdowns per second of scored time.
    pub step_rate_hz: f64,
    /// Share of consecutive lone touchdowns made by a different contact body. Absent with fewer
    /// than two contact bodies or no consecutive touchdowns.
    pub alternation_fraction: Option<f64>,
}

/// Judgement of the walking goal at one burst.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WalkVerdict {
    /// The distance or time rule is met.
    pub rule_met: bool,
    /// Speed of the window the rule judges, m/s.
    pub window_speed_m_s: Option<f64>,
    /// Heading error, degrees, when defined.
    pub heading_error_deg: Option<f64>,
    /// Rule met, window speed at or above the slowest passing speed, heading inside tolerance.
    pub success: bool,
}

/// Heading error in degrees for a displacement, or `None` while it is too short to trust.
pub fn walk_heading_error_deg(
    forward_m: f64,
    lateral_m: f64,
    reset_height_m: Option<f64>,
) -> Option<f64> {
    let height = reset_height_m.filter(|h| h.is_finite() && *h > 0.0)?;
    let displacement = forward_m.hypot(lateral_m);
    (displacement >= WALK_MIN_HEADING_DISPLACEMENT_HEIGHTS * height - 1.0e-12)
        .then(|| lateral_m.abs().atan2(forward_m).to_degrees())
}

/// Fold one counted, scored burst into the walk accumulator. `bursts` are the scored bursts so far.
fn walk_add(
    acc: &WalkAccum,
    reading: &WalkReading,
    contacts: &[bool],
    bursts: u32,
    hold_bursts: u32,
) -> WalkAccum {
    let mut next = acc.clone();
    next.history.push_back((bursts, reading.forward_m));
    while next.history.len() >= 2 && next.history[1].0.saturating_add(hold_bursts) <= bursts {
        next.history.pop_front();
    }
    next.last_forward_m = reading.forward_m;
    next.last_lateral_m = reading.lateral_m;
    if let Some(before) = &acc.last_down {
        let landed: Vec<usize> = contacts
            .iter()
            .enumerate()
            .filter(|(i, down)| **down && !before.get(*i).copied().unwrap_or(false))
            .map(|(i, _)| i)
            .collect();
        next.steps = next.steps.saturating_add(landed.len() as u32);
        if let [index] = landed[..] {
            if let Some(previous) = acc.last_touchdown {
                next.touchdown_pairs += 1;
                next.alternations += u32::from(previous != index);
            }
            next.last_touchdown = Some(index);
        }
    }
    next.last_down = Some(contacts.to_vec());
    next
}

/// Judge the walking goal after a counted burst. `bursts` are the scored bursts so far and
/// `holding_bursts` the current upright run, both elapsed bursts.
fn walk_verdict(
    goal: &WalkGoal,
    criterion: &BalanceFeetCriterion,
    acc: &WalkAccum,
    bursts: u32,
    holding_bursts: u32,
    reset_height_m: Option<f64>,
) -> WalkVerdict {
    let period = criterion.burst_period_s;
    let forward = acc.last_forward_m;
    let (rule_met, window_speed) = match goal.distance_m {
        Some(distance) => {
            let elapsed = f64::from(bursts) * period;
            (
                forward >= distance,
                (elapsed > 0.0).then(|| forward / elapsed),
            )
        }
        None => {
            // The last `hold_s` seconds, which an upright run of that length covers in full.
            let hold_bursts = criterion.hold_bursts();
            let anchor = bursts.checked_sub(hold_bursts).and_then(|target| {
                acc.history
                    .iter()
                    .rev()
                    .find(|(at, _)| *at <= target)
                    .copied()
                    .or(Some((0, 0.0)))
            });
            let speed = anchor.and_then(|(at, then)| {
                let window_s = f64::from(bursts - at) * period;
                (window_s > 0.0).then(|| (forward - then) / window_s)
            });
            (holding_bursts >= hold_bursts, speed)
        }
    };
    let heading = walk_heading_error_deg(forward, acc.last_lateral_m, reset_height_m);
    let success = rule_met
        && window_speed.is_some_and(|v| {
            v >= goal.min_speed_m_s() - 1.0e-12 && v <= goal.max_speed_m_s() + 1.0e-12
        })
        && heading.is_some_and(|h| h <= goal.heading_tolerance_deg + 1.0e-12);
    WalkVerdict {
        rule_met,
        window_speed_m_s: window_speed,
        heading_error_deg: heading,
        success,
    }
}

/// Walk numbers of an attempt that ended after `bursts` scored bursts.
fn walk_stats(
    goal: &WalkGoal,
    criterion: &BalanceFeetCriterion,
    acc: &WalkAccum,
    bursts: u32,
    holding_bursts: u32,
    reset_height_m: Option<f64>,
) -> WalkAttemptStats {
    let verdict = walk_verdict(goal, criterion, acc, bursts, holding_bursts, reset_height_m);
    let elapsed = f64::from(bursts) * criterion.burst_period_s;
    let mean_speed = if elapsed > 0.0 {
        acc.last_forward_m / elapsed
    } else {
        0.0
    };
    let pairs = acc.touchdown_pairs;
    WalkAttemptStats {
        forward_m: acc.last_forward_m,
        lateral_m: acc.last_lateral_m,
        mean_speed_m_s: mean_speed,
        window_speed_m_s: verdict.window_speed_m_s,
        speed_error_fraction: (mean_speed - goal.speed_m_s) / goal.speed_m_s,
        heading_error_deg: verdict.heading_error_deg,
        steps: acc.steps,
        step_rate_hz: if elapsed > 0.0 {
            f64::from(acc.steps) / elapsed
        } else {
            0.0
        },
        alternation_fraction: (criterion.contact_bodies.len() >= 2 && pairs > 0)
            .then(|| f64::from(acc.alternations) / f64::from(pairs)),
    }
}

/// Torso motion readings for the stability metrics of one burst.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StabilitySample {
    /// Horizontal angular velocity `hypot(wx, wy)`, rad/s. Absent when not reported.
    pub horizontal_ang_vel_rad_s: Option<f64>,
    /// Tilt vector, rad. Absent when not reported.
    pub tilt_vec_rad: Option<[f64; 2]>,
    /// Subtree CoM horizontal position (x, y), m. Absent when not reported.
    pub com_xy_m: Option<[f64; 2]>,
    /// Heading of the torso about up (the measure reply's `heading.yaw_rad`), rad. Absent on a
    /// controller that does not report it, or when forward points along up.
    #[serde(default)]
    pub yaw_rad: Option<f64>,
}

/// Signed lean of one burst: `(x, y)` are the components of the tilt vector on the controller's
/// two horizontal axes (world X, Y for z-up) and `(forward, left)` the same lean in the torso
/// frame, positive leaning toward the torso's forward or left. All degrees.
///
/// The tilt vector is the rotation that takes the torso's up axis back onto world up, so leaning
/// toward +X gives `(0, -theta)` and leaning toward +Y gives `(theta, 0)`. With forward
/// `f = (cos yaw, sin yaw)` and left `l = (-sin yaw, cos yaw)`: `left = r . f`,
/// `forward = -(r . l)`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TiltComponentsDeg {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub forward: Option<f64>,
    pub left: Option<f64>,
}

impl StabilitySample {
    /// The signed lean components of this burst; each is empty when its inputs were not reported.
    pub fn tilt_components_deg(&self) -> TiltComponentsDeg {
        match self.tilt_vec_rad {
            Some(vec) => tilt_components_of(vec, self.yaw_rad),
            None => TiltComponentsDeg::default(),
        }
    }
}

/// Signed lean components of a tilt vector `[rx, ry]` (rad) for a torso heading `yaw_rad`
/// (the torso-frame `forward` and `left` are empty without it).
fn tilt_components_of(vec: [f64; 2], yaw_rad: Option<f64>) -> TiltComponentsDeg {
    let [rx, ry] = vec;
    let (forward, left) = match yaw_rad {
        Some(yaw) => {
            let (sin, cos) = yaw.sin_cos();
            (
                Some((-(-rx * sin + ry * cos)).to_degrees()),
                Some((rx * cos + ry * sin).to_degrees()),
            )
        }
        None => (None, None),
    };
    TiltComponentsDeg {
        x: Some(rx.to_degrees()),
        y: Some(ry.to_degrees()),
        forward,
        left,
    }
}

/// Reset pose the runner latched on an attempt's first sample, with where it was latched.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ResetLatch {
    /// The posture line and heading at the reset pose.
    pub reference: LeanReference,
    /// Episode the reading belongs to.
    pub episode_id: u64,
    /// Physics steps of the reading; a later reading with fewer steps means a new model load.
    pub physics_steps: u64,
}

/// Receives each attempt's reset pose the moment the runner latches it.
pub type ResetLatchSink<'a> = &'a mut (dyn FnMut(&ResetLatch) + 'a);

/// One contact body in a live reading.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LiveContact {
    /// Contact body name, as in the setup.
    pub body: String,
    /// Touches the world now; `None` when the reply carried no row for it.
    pub touching: Option<bool>,
}

/// What the setup panel shows for one `measure` reply, from the same posture code the runner
/// scores with. Missing positions leave the tilt fields `None`; the rows that exist are still
/// returned.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveBalanceReading {
    /// The tilt scoring uses: from the latched reset pose, else the absolute posture tilt.
    pub scored_tilt_deg: Option<f64>,
    /// Posture tilt from world vertical, degrees.
    pub absolute_tilt_deg: Option<f64>,
    /// The torso's own tilt from world vertical, degrees.
    pub torso_tilt_deg: Option<f64>,
    /// One row per setup contact body, in setup order.
    pub contacts: Vec<LiveContact>,
    /// Every contact body touches (false with no contact bodies).
    pub all_contacts_touch: bool,
    /// The scored tilt is measured from a latched reset pose.
    pub reference_latched: bool,
    /// Episode of the reply.
    pub episode_id: u64,
    /// Physics steps of the reply.
    pub physics_steps: u64,
}

fn posture_sample_of(
    reply: &Measurement,
    criterion: &BalanceFeetCriterion,
) -> Option<PostureSample> {
    let posture = measure_posture(reply, criterion).ok()?;
    let yaw = reply
        .bodies
        .get(criterion.torso_body.trim())
        .and_then(|body| body.heading.as_ref())
        .and_then(|heading| heading.yaw_rad);
    Some(posture.sample(yaw))
}

/// The reset pose of a `measure` reply taken in the same operation as a reset: the posture line
/// and heading the runner latches on an attempt's first sample. `None` for a partial setup.
pub fn latch_reference(
    reply: &Measurement,
    criterion: &BalanceFeetCriterion,
) -> Option<LeanReference> {
    posture_sample_of(reply, criterion)?.reference
}

/// The live readout of one `measure` reply against the current reset pose (`None`: not latched,
/// so the scored tilt is the absolute one, as in the runner before a pose is latched).
pub fn live_balance_reading(
    reply: &Measurement,
    criterion: &BalanceFeetCriterion,
    reference: Option<&LeanReference>,
) -> LiveBalanceReading {
    let sample = posture_sample_of(reply, criterion).map(|mut sample| {
        if reference.is_some() {
            sample.score_from(reference);
        }
        sample
    });
    let contacts: Vec<LiveContact> = criterion
        .contact_bodies
        .iter()
        .enumerate()
        .map(|(i, name)| LiveContact {
            body: name.trim().to_string(),
            touching: reply.contacts.get(i).map(|set| set.touching),
        })
        .collect();
    LiveBalanceReading {
        scored_tilt_deg: sample.map(|s| s.relative_tilt_deg.unwrap_or(s.tilt_deg)),
        absolute_tilt_deg: sample.map(|s| s.tilt_deg),
        torso_tilt_deg: reply
            .bodies
            .get(criterion.torso_body.trim())
            .map(|body| body.tilt_rad.to_degrees()),
        all_contacts_touch: !contacts.is_empty()
            && contacts.iter().all(|row| row.touching == Some(true)),
        contacts,
        reference_latched: reference.is_some() && sample.is_some(),
        episode_id: reply.episode_id,
        physics_steps: reply.physics_steps,
    }
}

/// Head-over-feet reading of one burst, from world positions only the Trainer reads (never a
/// brain input): the line from the support centre to the posture top body, against vertical.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PostureReading {
    /// Angle of the line from world vertical, rad, in `[0, pi]`.
    pub tilt_rad: f64,
    /// The rotation that takes the line back onto world up, on the world X and Y axes: the same
    /// convention as the torso's tilt vector (leaning toward +X gives `(0, -theta)`).
    pub tilt_vec_rad: [f64; 2],
    /// Unit vector from the support centre to the top body (world frame, z up).
    pub unit: [f64; 3],
}

/// An upright axis (the posture line, or the torso's up axis) with the heading it was read at.
/// Latched on an attempt's first sample as the reset pose: the tilt that is scored is measured
/// from it, so a stand pose that is not vertical is not counted as lean.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LeanReference {
    /// Unit vector of the axis, world frame, z up.
    pub unit: [f64; 3],
    /// Torso heading about up when it was read, rad. Absent when the controller reports none.
    pub yaw_rad: Option<f64>,
    /// Angle of the axis from world vertical, degrees (the absolute lean).
    pub tilt_deg: f64,
}

/// The lean of an axis away from the reset axis, turned with the heading.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RelativeLean {
    /// Angle between the current axis and the reset axis rotated by the heading change, rad.
    pub angle_rad: f64,
    /// Rotation taking the current axis onto the turned reset axis (angle times unit axis), rad;
    /// the same sign convention as the tilt vector.
    pub rotation_rad: [f64; 3],
}

impl LeanReference {
    /// An axis from its (not necessarily unit) world vector; `None` for a zero or non-finite one.
    pub fn from_vector(v: [f64; 3], yaw_rad: Option<f64>) -> Option<Self> {
        let norm = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if !v.iter().all(|c| c.is_finite()) || norm < 1.0e-12 {
            return None;
        }
        let unit = [v[0] / norm, v[1] / norm, v[2] / norm];
        Some(Self {
            unit,
            yaw_rad: yaw_rad.filter(|y| y.is_finite()),
            tilt_deg: unit[2].clamp(-1.0, 1.0).acos().to_degrees(),
        })
    }

    /// The torso's up axis from its tilt vector `[rx, ry]` (rad), the rotation that takes the
    /// axis back onto world up.
    pub fn from_tilt_vec(tilt_vec: [f64; 2], yaw_rad: Option<f64>) -> Option<Self> {
        let [rx, ry] = tilt_vec;
        let theta = rx.hypot(ry);
        if !theta.is_finite() {
            return None;
        }
        let up = if theta < 1.0e-12 {
            [0.0, 0.0, 1.0]
        } else {
            let scale = theta.sin() / theta;
            [-ry * scale, rx * scale, theta.cos()]
        };
        Self::from_vector(up, yaw_rad)
    }

    /// The lean of `self` (now) away from `reset`, with the reset axis rotated about up by the
    /// heading change. Without a heading at either end the reset axis is not rotated.
    pub fn lean_from(&self, reset: &LeanReference) -> RelativeLean {
        let turn = match (self.yaw_rad, reset.yaw_rad) {
            (Some(now), Some(then)) => now - then,
            _ => 0.0,
        };
        let (sin, cos) = turn.sin_cos();
        let r = reset.unit;
        let turned = [r[0] * cos - r[1] * sin, r[0] * sin + r[1] * cos, r[2]];
        let c = self.unit;
        let cross = [
            c[1] * turned[2] - c[2] * turned[1],
            c[2] * turned[0] - c[0] * turned[2],
            c[0] * turned[1] - c[1] * turned[0],
        ];
        let sin_a = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
        let dot = c[0] * turned[0] + c[1] * turned[1] + c[2] * turned[2];
        let angle_rad = sin_a.atan2(dot);
        let rotation_rad = if sin_a < 1.0e-12 {
            [0.0; 3]
        } else {
            let k = angle_rad / sin_a;
            [cross[0] * k, cross[1] * k, cross[2] * k]
        };
        RelativeLean {
            angle_rad,
            rotation_rad,
        }
    }
}

/// Predicted tilt, degrees, of a relative lean carried forward by the torso's swing: the
/// rotation vector moves by `-ang_vel` (horizontal) over `lookahead_s`.
pub fn predicted_relative_tilt_deg(
    lean: &RelativeLean,
    ang_vel: [f64; 3],
    lookahead_s: f64,
) -> f64 {
    let [rx, ry, rz] = lean.rotation_rad;
    (rx - lookahead_s * ang_vel[0])
        .hypot(ry - lookahead_s * ang_vel[1])
        .hypot(rz)
        .to_degrees()
}

/// Posture of the burst, as the Trainer scores it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PostureSample {
    /// Absolute posture tilt, degrees: the angle of the line from the support centre to the top
    /// body against world vertical. Reported; the scored tilt is `relative_tilt_deg`.
    pub tilt_deg: f64,
    /// Posture tilt from the attempt's reset pose, degrees: the angle between the line now and
    /// the reset line turned by the heading change. Absent until the Trainer has latched a reset
    /// line (then the absolute `tilt_deg` is scored).
    #[serde(default)]
    pub relative_tilt_deg: Option<f64>,
    /// This burst's own line and heading, which the attempt latches on its first sample.
    #[serde(default)]
    pub reference: Option<LeanReference>,
    /// Signed posture lean in the torso frame, degrees, positive toward the torso's forward.
    /// Absent without the torso heading.
    pub forward_deg: Option<f64>,
    /// Signed posture lean in the torso frame, degrees, positive toward the torso's left.
    pub left_deg: Option<f64>,
}

/// Support centre: the mean of the positions of the contact bodies touching now; when none
/// touch, the mean of all of them (the stance is judged from where the feet are). `None` for no
/// positions. `touching[i]` belongs to `positions[i]`.
pub fn support_centre(positions: &[[f64; 3]], touching: &[bool]) -> Option<[f64; 3]> {
    let mean = |keep: &dyn Fn(usize) -> bool| {
        let mut sum = [0.0; 3];
        let mut n = 0.0;
        for p in positions
            .iter()
            .enumerate()
            .filter(|(i, _)| keep(*i))
            .map(|(_, p)| p)
        {
            for k in 0..3 {
                sum[k] += p[k];
            }
            n += 1.0;
        }
        (n > 0.0).then(|| [sum[0] / n, sum[1] / n, sum[2] / n])
    };
    mean(&|i| touching.get(i).copied().unwrap_or(false)).or_else(|| mean(&|_| true))
}

/// The posture reading of a line from `centre` up to `top` (world positions, z up). `None` when
/// the two coincide.
pub fn posture_reading(centre: [f64; 3], top: [f64; 3]) -> Option<PostureReading> {
    let d = [top[0] - centre[0], top[1] - centre[1], top[2] - centre[2]];
    if !d.iter().all(|v| v.is_finite()) {
        return None;
    }
    let horizontal = d[0].hypot(d[1]);
    if horizontal.hypot(d[2]) < 1.0e-12 {
        return None;
    }
    let tilt_rad = horizontal.atan2(d[2]);
    // up = (sin t cos a, sin t sin a, cos t) with a the lean heading: the rotation back onto
    // world up is t * (uy, -ux) / sin t.
    let tilt_vec_rad = if horizontal < 1.0e-12 {
        if d[2] >= 0.0 {
            [0.0, 0.0]
        } else {
            [std::f64::consts::PI, 0.0]
        }
    } else {
        let scale = tilt_rad / horizontal;
        [d[1] * scale, -d[0] * scale]
    };
    let norm = horizontal.hypot(d[2]);
    Some(PostureReading {
        tilt_rad,
        tilt_vec_rad,
        unit: [d[0] / norm, d[1] / norm, d[2] / norm],
    })
}

impl PostureSample {
    /// Set `relative_tilt_deg` from the latched reset line (the one scoring rule the runner and
    /// the live readout share). With none latched this burst is the reset pose and reads 0.
    pub fn score_from(&mut self, reset: Option<&LeanReference>) -> Option<RelativeLean> {
        let now = self.reference?;
        let lean = now.lean_from(reset.unwrap_or(&now));
        self.relative_tilt_deg = Some(lean.angle_rad.to_degrees());
        Some(lean)
    }
}

impl PostureReading {
    /// The sample the Trainer scores, with the lean split on the torso heading.
    pub fn sample(&self, yaw_rad: Option<f64>) -> PostureSample {
        let lean = tilt_components_of(self.tilt_vec_rad, yaw_rad);
        PostureSample {
            tilt_deg: self.tilt_rad.to_degrees(),
            relative_tilt_deg: None,
            reference: LeanReference::from_vector(self.unit, yaw_rad),
            forward_deg: lean.forward,
            left_deg: lean.left,
        }
    }
}

impl StabilitySample {
    /// Readings from a measured body; each one is present only if the reply carried it.
    pub fn from_body(body: &BodyMeasure) -> Self {
        Self {
            horizontal_ang_vel_rad_s: body.ang_vel.map(|w| w[0].hypot(w[1])),
            tilt_vec_rad: body.tilt_vec,
            com_xy_m: body.subtree_com.map(|c| [c[0], c[1]]),
            yaw_rad: body.heading.as_ref().and_then(|heading| heading.yaw_rad),
        }
    }
}

/// Running stability statistics of one attempt, weighted by elapsed bursts. Means and variances
/// use the weighted Welford update, so long attempts stay numerically stable. Each metric keeps
/// its own weight: it counts only bursts that carried its reading.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct StabilityAccum {
    wobble_weight: f64,
    wobble_sq_sum: f64,
    tilt_weight: f64,
    tilt_mean: [f64; 2],
    tilt_m2: [f64; 2],
    com_weight: f64,
    com_mean: [f64; 2],
    com_m2: [f64; 2],
}

/// Weighted Welford step for a 2-vector.
fn welford2(weight: &mut f64, mean: &mut [f64; 2], m2: &mut [f64; 2], x: [f64; 2], w: f64) {
    *weight += w;
    for i in 0..2 {
        let d = x[i] - mean[i];
        mean[i] += d * w / *weight;
        m2[i] += w * d * (x[i] - mean[i]);
    }
}

impl StabilityAccum {
    /// Add one reading standing for `bursts` elapsed bursts. Missing or non-finite parts and
    /// zero weight leave that metric unchanged.
    pub fn add(self, reading: Option<&StabilitySample>, bursts: u32) -> Self {
        let Some(r) = reading.filter(|_| bursts > 0) else {
            return self;
        };
        let w = f64::from(bursts);
        let mut next = self;
        if let Some(v) = r.horizontal_ang_vel_rad_s.filter(|v| v.is_finite()) {
            next.wobble_weight += w;
            next.wobble_sq_sum += w * v * v;
        }
        if let Some(t) = r.tilt_vec_rad.filter(|t| t.iter().all(|v| v.is_finite())) {
            welford2(
                &mut next.tilt_weight,
                &mut next.tilt_mean,
                &mut next.tilt_m2,
                t,
                w,
            );
        }
        if let Some(c) = r.com_xy_m.filter(|c| c.iter().all(|v| v.is_finite())) {
            welford2(
                &mut next.com_weight,
                &mut next.com_mean,
                &mut next.com_m2,
                c,
                w,
            );
        }
        next
    }

    /// RMS horizontal angular velocity, rad/s.
    pub fn wobble_rad_s(&self) -> Option<f64> {
        (self.wobble_weight > 0.0).then(|| (self.wobble_sq_sum / self.wobble_weight).sqrt())
    }

    /// RMS deviation of the tilt vector from its mean, degrees.
    pub fn tilt_sway_deg(&self) -> Option<f64> {
        (self.tilt_weight > 0.0).then(|| {
            ((self.tilt_m2[0] + self.tilt_m2[1]).max(0.0) / self.tilt_weight)
                .sqrt()
                .to_degrees()
        })
    }

    /// RMS horizontal CoM deviation from its mean over `reset_height_m`. Absent without a
    /// positive reset height.
    pub fn com_sway_fraction(&self, reset_height_m: Option<f64>) -> Option<f64> {
        let reset = reset_height_m.filter(|h| h.is_finite() && *h > 0.0)?;
        (self.com_weight > 0.0)
            .then(|| ((self.com_m2[0] + self.com_m2[1]).max(0.0) / self.com_weight).sqrt() / reset)
    }
}

impl BalanceSample {
    /// The tilt every scoring rule uses (upright, lean and success windows, fall, stance loss):
    /// the posture tilt from the reset pose, the absolute posture tilt before a reset line is
    /// latched, or the torso tilt for a sample that carries no posture.
    pub fn scored_tilt_deg(&self) -> f64 {
        self.posture
            .map_or(self.tilt_deg, |p| p.relative_tilt_deg.unwrap_or(p.tilt_deg))
    }

    /// Posture tilt from world vertical (not from the reset pose), degrees.
    pub fn absolute_tilt_deg(&self) -> f64 {
        self.posture.map_or(self.tilt_deg, |p| p.tilt_deg)
    }

    /// The torso's up axis and heading on this burst, when the tilt vector was reported.
    pub fn torso_reference(&self) -> Option<LeanReference> {
        let st = self.stability.as_ref()?;
        LeanReference::from_tilt_vec(st.tilt_vec_rad?, st.yaw_rad)
    }

    /// Set the scored posture tilt from the attempt's latched reset line. With none latched yet
    /// this burst is the reset pose and reads 0.
    pub fn score_posture_from(&mut self, reset: Option<&LeanReference>) -> Option<RelativeLean> {
        self.posture.as_mut()?.score_from(reset)
    }

    /// Every contact body touches the world.
    pub fn all_contacts_down(&self) -> bool {
        self.contacts.iter().all(|down| *down)
    }
}

/// Predicted tilt fraction `min(1, predicted_tilt / max_tilt)`, with the prediction of
/// [`predicted_tilt_deg`] (along the lean when `tilt_vec` is given, else the old
/// `tilt + lookahead * |omega_xy|`), so need and the cone test agree.
pub fn support_need(
    tilt_rad: f64,
    tilt_vec: Option<[f64; 2]>,
    ang_vel: [f64; 3],
    max_tilt_deg: f64,
    lookahead_s: f64,
) -> f64 {
    (predicted_tilt_deg(tilt_rad, tilt_vec, ang_vel, lookahead_s) / max_tilt_deg).clamp(0.0, 1.0)
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
    /// Posture line and heading on the first sample after the reset; the scored tilt is
    /// measured from it.
    pub reset_posture: Option<LeanReference>,
    /// Torso up axis and heading on the first sample after the reset; `support_need` is
    /// measured from it.
    pub reset_torso: Option<LeanReference>,
    /// Non-settling bursts per failed upright test so far.
    pub non_upright: NonUprightCounts,
    /// Support-use samples so far (support > 0, non-settling).
    pub support_use: SupportUseCounts,
    /// Consecutive elapsed bursts that were not standing, ending at the latest counted burst.
    pub off_stance_bursts: u32,
    /// Tilt, degrees, on the first burst of the current off-stance run (set when
    /// `off_stance_bursts` goes from 0 to 1, cleared when the stance stands again).
    pub stance_lost_tilt_deg: Option<f64>,
    /// Non-settling bursts elapsed when the collapse was first confirmed, if it was.
    pub collapsed_at_bursts: Option<u32>,
    /// Stability statistics over the counted, non-settling, upright bursts so far.
    pub stability: StabilityAccum,
    /// Walk readings so far; untouched without a walking goal.
    pub walk: WalkAccum,
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
            reset_posture: None,
            reset_torso: None,
            non_upright: NonUprightCounts::default(),
            support_use: SupportUseCounts::default(),
            off_stance_bursts: 0,
            stance_lost_tilt_deg: None,
            collapsed_at_bursts: None,
            stability: StabilityAccum::default(),
            walk: WalkAccum::default(),
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
        tilt: sample.scored_tilt_deg() > criterion.max_tilt_deg,
        contact: !criterion.contacts_satisfied(&sample.contacts),
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
    (sample.scored_tilt_deg() / criterion.max_tilt_deg).clamp(0.0, 1.0)
}

/// Standing: tilt inside the limit, every contact body down, no other body on the world (when
/// `only_contacts_touch`) and the torso above its height fraction (when `min_height_fraction`
/// is on). Uses the sample's own reset height.
pub fn sample_is_upright(sample: &BalanceSample, criterion: &BalanceFeetCriterion) -> bool {
    let reset = sample.reset_height_m.or(sample.torso_height_m);
    !non_upright_reasons(sample, criterion, reset).any()
}

/// Torso height as a fraction of its reset height (1 = at the reset pose). Uses the sample's own
/// reset height, falling back to the sample's height; `None` when the height is unknown or the
/// reset height is not positive.
pub fn sample_height_fraction(sample: &BalanceSample) -> Option<f64> {
    let height = sample.torso_height_m?;
    let reset = sample.reset_height_m.or(sample.torso_height_m)?;
    (reset > 0.0).then(|| height / reset)
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
    if let Some(posture) = sample.posture {
        if !posture.tilt_deg.is_finite() || posture.tilt_deg < 0.0 {
            return Err(TrainerError::Runtime(format!(
                "posture tilt_deg must be a finite value >= 0, got {}",
                posture.tilt_deg
            )));
        }
    }
    let reset_height_m = open
        .reset_height_m
        .or(sample.reset_height_m)
        .or(sample.torso_height_m);
    let reset_posture = open
        .reset_posture
        .or(sample.posture.and_then(|p| p.reference));
    let reset_torso = open.reset_torso.or(sample.torso_reference());
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
        let stability = if !sample.settling && upright {
            open.stability.add(sample.stability.as_ref(), 1)
        } else {
            open.stability
        };
        let collapsed_at_bursts = if sample.settling {
            open.collapsed_at_bursts
        } else {
            let off_run = if upright {
                0
            } else {
                open.off_stance_bursts.saturating_add(1)
            };
            collapse_confirmed_at(open.collapsed_at_bursts, off_run, bursts, criterion)
        };
        let terminal_walk = criterion.walk.as_ref().map(|goal| {
            let acc = match (&sample.walk, sample.settling) {
                (Some(reading), false) => walk_add(
                    &open.walk,
                    reading,
                    &sample.contacts,
                    bursts,
                    criterion.hold_bursts(),
                ),
                _ => open.walk.clone(),
            };
            walk_stats(goal, criterion, &acc, bursts, run, reset_height_m)
        });
        return Ok(BalanceAttemptStep::Finished(BalanceAttemptRecord {
            seed,
            outcome,
            bursts,
            final_tilt_deg: sample.scored_tilt_deg(),
            reset_tilt_deg: reset_posture.map(|r| r.tilt_deg),
            stance_lost_tilt_deg: if upright {
                None
            } else {
                open.stance_lost_tilt_deg.or(Some(sample.scored_tilt_deg()))
            },
            support_level: open.params_support_level,
            upright_fraction: upright_fraction(upright_bursts, bursts),
            perturbations,
            non_upright,
            carried_fraction: support_use.carried_fraction(),
            over_assist_fraction: support_use.over_assist_fraction(),
            collapsed_at_bursts,
            collapsed_at_s: collapsed_at_bursts.map(|b| f64::from(b) * criterion.burst_period_s),
            wobble_rad_s: stability.wobble_rad_s(),
            tilt_sway_deg: stability.tilt_sway_deg(),
            com_sway_fraction: stability.com_sway_fraction(reset_height_m),
            walk: terminal_walk,
            spring_applied: None,
            observe: None,
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
        next.reset_posture = reset_posture;
        next.reset_torso = reset_torso;
        next.perturbations = perturbations;
        next.last_perturbing = last_perturbing;
        return Ok(BalanceAttemptStep::Continue(next));
    }
    if sample.settling {
        let mut next = open.clone();
        next.settling_bursts = open.settling_bursts.saturating_add(new_bursts);
        next.last_tilt_deg = sample.scored_tilt_deg();
        next.last_seq = sample.seq;
        next.last_burst_seq = Some(anchor);
        next.counted_burst = true;
        next.reset_height_m = reset_height_m;
        next.reset_posture = reset_posture;
        next.reset_torso = reset_torso;
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
    let stance_lost_tilt_deg = if upright {
        None
    } else {
        open.stance_lost_tilt_deg.or(Some(sample.scored_tilt_deg()))
    };
    let fall_bursts =
        if criterion.fall_tilt_deg > 0.0 && sample.scored_tilt_deg() >= criterion.fall_tilt_deg {
            open.fall_bursts.saturating_add(new_bursts)
        } else {
            0
        };
    let collapsed_at_bursts = collapse_confirmed_at(
        open.collapsed_at_bursts,
        off_stance_bursts,
        bursts,
        criterion,
    );
    let first_violation_burst =
        open.first_violation_burst
            .or(if upright { None } else { Some(open.bursts) });
    let non_upright = open.non_upright.add(reasons, new_bursts);
    let level = sample.support_level.unwrap_or(open.params_support_level);
    let support_use = open.support_use.add(
        support_use_flags(sample, criterion, reset_height_m, level),
        new_bursts,
    );
    let stability = if upright {
        open.stability.add(sample.stability.as_ref(), new_bursts)
    } else {
        open.stability
    };
    let walk_acc = match (&criterion.walk, &sample.walk) {
        (Some(_), Some(reading)) => walk_add(
            &open.walk,
            reading,
            &sample.contacts,
            bursts,
            criterion.hold_bursts(),
        ),
        (Some(_), None) => {
            return Err(TrainerError::Runtime(
                "a walking goal needs the torso position (body com) in every sample".to_string(),
            ))
        }
        (None, _) => open.walk.clone(),
    };
    let walk_record = criterion.walk.as_ref().map(|goal| {
        walk_stats(
            goal,
            criterion,
            &walk_acc,
            bursts,
            holding_bursts,
            reset_height_m,
        )
    });
    let finish = |outcome| {
        BalanceAttemptStep::Finished(BalanceAttemptRecord {
            seed,
            outcome,
            bursts,
            final_tilt_deg: sample.scored_tilt_deg(),
            reset_tilt_deg: reset_posture.map(|r| r.tilt_deg),
            stance_lost_tilt_deg,
            support_level: open.params_support_level,
            upright_fraction: upright_fraction(upright_bursts, bursts),
            perturbations,
            non_upright,
            carried_fraction: support_use.carried_fraction(),
            over_assist_fraction: support_use.over_assist_fraction(),
            collapsed_at_bursts,
            collapsed_at_s: collapsed_at_bursts.map(|b| f64::from(b) * criterion.burst_period_s),
            wobble_rad_s: stability.wobble_rad_s(),
            tilt_sway_deg: stability.tilt_sway_deg(),
            com_sway_fraction: stability.com_sway_fraction(reset_height_m),
            walk: walk_record,
            spring_applied: None,
            observe: None,
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
    if let Some(goal) = &criterion.walk {
        let verdict = walk_verdict(
            goal,
            criterion,
            &walk_acc,
            bursts,
            holding_bursts,
            reset_height_m,
        );
        if verdict.success {
            return Ok(finish(BalanceAttemptOutcome::Success));
        }
        // A run: the distance was covered, or (time rule, once a full hold_s window exists) the
        // window was walked, faster than the ceiling. The attempt ends here, because standing
        // still afterwards would let the mean speed fall into the band.
        if (verdict.rule_met || goal.distance_m.is_none())
            && verdict
                .window_speed_m_s
                .is_some_and(|v| v > goal.max_speed_m_s() + 1.0e-12)
        {
            return Ok(finish(BalanceAttemptOutcome::Truncated));
        }
    } else if !adaptive && holding_bursts >= criterion.hold_bursts() {
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
        // A walking attempt succeeds only through the walk rule above, never by standing.
        return Ok(finish(if adaptive && stood && criterion.walk.is_none() {
            BalanceAttemptOutcome::Success
        } else {
            BalanceAttemptOutcome::Truncated
        }));
    }
    Ok(BalanceAttemptStep::Continue(OpenBalanceAttempt {
        bursts,
        holding_bursts,
        last_tilt_deg: sample.scored_tilt_deg(),
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
        reset_posture,
        reset_torso,
        non_upright,
        support_use,
        off_stance_bursts,
        stance_lost_tilt_deg,
        collapsed_at_bursts,
        stability,
        walk: walk_acc,
    }))
}

/// Latch the first burst at which the body has been continuously not upright for at least
/// `fall_confirm_s` (`off_run` bursts, at least one). Uses the upright rule only, so it does
/// not depend on `fall_tilt_deg`.
fn collapse_confirmed_at(
    latched: Option<u32>,
    off_run: u32,
    bursts: u32,
    criterion: &BalanceFeetCriterion,
) -> Option<u32> {
    latched
        .or_else(|| (off_run > 0 && off_run >= criterion.fall_confirm_bursts()).then_some(bursts))
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
        reset_tilt_deg: None,
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
        collapsed_at_bursts: None,
        collapsed_at_s: None,
        wobble_rad_s: None,
        tilt_sway_deg: None,
        com_sway_fraction: None,
        stance_lost_tilt_deg: None,
        adaptive: None,
        walk: None,
        spring_applied: None,
        observe: None,
    }
}

/// Mean of the `Some` values; `None` when there are none.
fn mean_of_some(values: impl Iterator<Item = Option<f64>>) -> Option<f64> {
    let (sum, n) = values
        .flatten()
        .filter(|v| v.is_finite())
        .fold((0.0, 0u32), |(s, n), v| (s + v, n + 1));
    (n > 0).then(|| sum / f64::from(n))
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
        let tilt = burst_sample.scored_tilt_deg();
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
/// Walking contribution reward: Pleasure paid per burst for gaining ground at the target speed
/// (scaled down for slower progress and by `1 - assist level`).
pub const WALK_PROGRESS_PLEASURE: f64 = 0.5;

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

    /// Policy identity recorded with the run for a walking session: the contribution policy
    /// pays walk progress, so it carries its own version.
    pub fn policy_id_for(self, walking: bool) -> &'static str {
        match (self, walking) {
            (Self::Contribution, true) => "reward.balance-contribution.v5",
            _ => self.policy_id(),
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
            .unwrap_or_else(|| burst_sample.scored_tilt_deg() / criterion.max_tilt_deg)
            .clamp(0.0, 1.0);
        let previous = self.previous_need.unwrap_or(need);
        self.previous_need = Some(need);
        let previous_tilt = self
            .previous_tilt_deg
            .unwrap_or(burst_sample.scored_tilt_deg());
        self.previous_tilt_deg = Some(burst_sample.scored_tilt_deg());
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
        if !upright && burst_sample.scored_tilt_deg() > previous_tilt {
            let past = (burst_sample.scored_tilt_deg() / criterion.max_tilt_deg).clamp(0.0, 1.0);
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
/// Pleasure `min(1 - a, ASSIST_FALL_GAIN * credit)`, where `credit` is the net need reduction
/// since the last payment, each burst's change weighted by `(1 - a)` when it happens (falls add,
/// rises subtract, never below 0, zeroed on bursts where the robot is not standing). A recovery
/// too slow to cross `ASSIST_MIN_PLEASURE` in one burst therefore still pays once its net gain
/// adds up, at any assist level including 0, and a wobble that nets to zero pays nothing; an
/// attempt that ends in Success ->
/// Pleasure `1 - a_slow` (`assist_slow`, else `a`); an attempt that ends in `LostStance` ->
/// Pain 0.5. Pleasure (recovery and the cap) is paid only on bursts where the robot is standing
/// ([`sample_is_upright`]). Being upright, or being held, pays nothing by itself.
/// Works with the ladder, where `a` is the constant level. Pleasure below
/// `ASSIST_MIN_PLEASURE` is dropped. Settling bursts emit nothing but seed the previous need.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BalanceContributionReward {
    previous_need: Option<f64>,
    /// Net need reduction (weighted by `1 - a`) not yet paid.
    credit: f64,
}

impl BalanceContributionReward {
    /// Policy at the start of an attempt.
    pub fn new() -> Self {
        Self::default()
    }

    /// Stable identity recorded with the run.
    pub fn policy_id() -> &'static str {
        "reward.balance-contribution.v4"
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
            .unwrap_or_else(|| burst_sample.scored_tilt_deg() / criterion.max_tilt_deg)
            .clamp(0.0, 1.0);
        let previous = self.previous_need.unwrap_or(need);
        self.previous_need = Some(need);
        if burst_sample.settling {
            self.credit = 0.0;
            return Vec::new();
        }
        let signal = |channel, magnitude: f64| RewardSignal { channel, magnitude };
        if matches!(
            outcome,
            Some(BalanceAttemptOutcome::Death | BalanceAttemptOutcome::Fell)
        ) {
            self.credit = 0.0;
            return vec![signal(AffectChannel::Pain, 1.0)];
        }
        if outcome == Some(&BalanceAttemptOutcome::LostStance) {
            self.credit = 0.0;
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
        // Net recovery since the last payment, weighted by the assist level at the time so
        // recovery done while held is not paid later at a low level. Nothing builds while the
        // robot is not standing.
        if standing {
            self.credit = (self.credit + (previous - need) * (1.0 - a)).max(0.0);
            if ASSIST_FALL_GAIN * self.credit >= ASSIST_MIN_PLEASURE {
                // Capped at what the robot can earn at the current assist level.
                pleasure = (ASSIST_FALL_GAIN * self.credit).min(1.0 - a);
                self.credit = 0.0;
            }
        } else {
            self.credit = 0.0;
        }
        // Walking: ground gained beyond the best so far, as a share of what the target speed
        // covers in one burst. Rocking over planted feet and backward steps gain none, and being
        // held (a) discounts it like every other contribution.
        if let (Some(goal), Some(walk)) = (&criterion.walk, &burst_sample.walk) {
            let per_burst_m = goal.speed_m_s * criterion.burst_period_s;
            if per_burst_m > 0.0 {
                let progress = (walk.progress_m / per_burst_m).clamp(0.0, 1.0);
                pleasure = pleasure.max(WALK_PROGRESS_PLEASURE * progress * (1.0 - a));
            }
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
    /// Mean of per-attempt `wobble_rad_s` over attempts that have one, rad/s. Lower is steadier.
    #[serde(default)]
    pub mean_wobble_rad_s: Option<f64>,
    /// Mean of per-attempt `tilt_sway_deg` over attempts that have one, degrees. Lower is steadier.
    #[serde(default)]
    pub mean_tilt_sway_deg: Option<f64>,
    /// Mean of per-attempt `com_sway_fraction` over attempts that have one (fraction of reset
    /// height). Lower is steadier.
    #[serde(default)]
    pub mean_com_sway_fraction: Option<f64>,
    /// Success rate per support level, keyed by the level formatted `{:.2}`.
    #[serde(default)]
    pub success_rate_by_support: BTreeMap<String, f64>,
    /// Spring gains the Trainer sized for this phase (absent when no attempt ran).
    #[serde(default)]
    pub spring_gains: Option<SpringGains>,
    /// The sizing inputs behind `spring_gains` (mass, gravity, CoM height, inertias, timestep),
    /// recorded so a result can show how the gains were sized.
    #[serde(default)]
    pub spring_sizing: Option<SizingInputs>,
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
    /// Adaptive: at some point `cone_min_deg` was above the learned limit
    /// `learned_loss_tilt * (1 - loss_margin_fraction)`, so the limit was held at `cone_min_deg`
    /// (this body lost its stance at a smaller angle than the cone's floor).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub cone_min_above_loss_tilt: bool,
    /// Attempts thrown away because the controller's FEAGI link (or a call to it) failed. Each
    /// was re-run with the same seed; none counts in `attempts` or in any score above.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub discarded_attempts: Vec<DiscardedAttempt>,
    /// The host skipped the phase: it ended after the attempt in progress, before every planned
    /// attempt ran. The attempts that finished are kept and scored.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub skipped: bool,
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
    /// Memory-neuron formation was paused (every memory area frozen for the phase).
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
        let mean_wobble_rad_s = mean_of_some(attempts.iter().map(|a| a.wobble_rad_s));
        let mean_tilt_sway_deg = mean_of_some(attempts.iter().map(|a| a.tilt_sway_deg));
        let mean_com_sway_fraction = mean_of_some(attempts.iter().map(|a| a.com_sway_fraction));
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
            mean_wobble_rad_s,
            mean_tilt_sway_deg,
            mean_com_sway_fraction,
            success_rate_by_support,
            spring_gains: None,
            spring_sizing: None,
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
            cone_min_above_loss_tilt: false,
            discarded_attempts: Vec::new(),
            skipped: false,
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
    for (name, value) in [
        ("mean_wobble_rad_s", phase.mean_wobble_rad_s),
        ("mean_tilt_sway_deg", phase.mean_tilt_sway_deg),
        ("mean_com_sway_fraction", phase.mean_com_sway_fraction),
    ] {
        if let Some(value) = value {
            metrics.insert(name.to_string(), value);
        }
    }
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
    insert_walk_metrics(&mut metrics, phase);
    insert_balanced_metrics(&mut metrics, "", &phase.balanced);
    for (level, stats) in &phase.balanced_by_support {
        insert_balanced_metrics(&mut metrics, &format!("_at_support_{level}"), stats);
    }
    metrics
}

/// Mean walk numbers over the scored attempts that carry them: `mean_forward_m`,
/// `mean_speed_m_s`, `mean_heading_error_deg` (attempts where it is defined), `mean_step_rate_hz`,
/// `mean_alternation_fraction`. Omitted for a standing phase.
fn insert_walk_metrics(metrics: &mut BTreeMap<String, f64>, phase: &BalancePhaseReport) {
    let walks: Vec<WalkAttemptStats> = phase
        .attempts
        .iter()
        .filter(|a| !a.probe)
        .filter_map(|a| a.walk)
        .collect();
    if walks.is_empty() {
        return;
    }
    for (name, value) in [
        (
            "mean_forward_m",
            mean_of_some(walks.iter().map(|w| Some(w.forward_m))),
        ),
        (
            "mean_speed_m_s",
            mean_of_some(walks.iter().map(|w| Some(w.mean_speed_m_s))),
        ),
        (
            "mean_heading_error_deg",
            mean_of_some(walks.iter().map(|w| w.heading_error_deg)),
        ),
        (
            "mean_step_rate_hz",
            mean_of_some(walks.iter().map(|w| Some(w.step_rate_hz))),
        ),
        (
            "mean_alternation_fraction",
            mean_of_some(walks.iter().map(|w| w.alternation_fraction)),
        ),
    ] {
        if let Some(value) = value {
            metrics.insert(name.to_string(), value);
        }
    }
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
        class_breakdown: Vec::new(),
        confusion: None,
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
    /// start a new episode. Returns the episode id and the controller's FEAGI link.
    fn reset_episode(&mut self, keyframe: Option<&str>) -> Result<ResetReply, TrainerError>;

    /// `set_body_spring`: capture the body's current pose as the reference and apply the spring.
    fn set_body_spring(
        &mut self,
        body: &str,
        gains: &SpringGains,
        level: f64,
        deadband_rad: f64,
    ) -> Result<(), TrainerError>;

    /// `update_body_spring`: change the level of a running spring and, when `deadband_rad` is
    /// `Some`, its tilt deadband, when `gravity_comp` is `Some`, its weight-support fraction and,
    /// when `ref_up_world` is `Some`, the world direction the body's up axis is driven toward
    /// (the deadband is measured from it). The Trainer never sends `ref_up_world`: the spring
    /// keeps the reference captured at the reset, so the support only resists falling past the
    /// cone and the brain must learn the posture. The field stays as a generic controller
    /// primitive.
    fn update_body_spring(
        &mut self,
        body: &str,
        level: f64,
        deadband_rad: Option<f64>,
        gravity_comp: Option<f64>,
        ref_up_world: Option<[f64; 3]>,
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

    /// `status`: the controller's FEAGI link right now, read while waiting for an outage to
    /// end.
    fn link_status(&mut self) -> Result<LinkReading, TrainerError>;
}

/// Callback that delivers affect signals into the brain (e.g. `runtime.submit_reward`).
pub type DeliverReward<'a> = &'a mut (dyn FnMut(&[RewardSignal]) -> Result<(), TrainerError> + 'a);

/// Engine state a paused balance phase exports in [`ResumeExport::engine_state`].
///
/// Restored: the support schedule (level and block counters), the adaptive assist (level, slow
/// average, loss history, hold timer, cone flag) and the clocks of its timelines, the attempt
/// count including probes (attempt indices and progress continue), and the probe cadence (it
/// follows `first_index`, the training attempts done), the timelines' points and the open
/// second of the deviation timeline, and the discarded attempts (so the last segment's report
/// holds every segment's, in order, like an unsplit run's). Reset: the spring gains (sized
/// again from the first reset of the segment). The perturbation generator derives from each attempt's seed.
/// Scheduled events are per attempt (they fire after settle, once per attempt, and a new
/// attempt ends any stimulation hold the last one left), so no schedule cursor is carried.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BalanceResumeState {
    /// Attempts run so far, probes included.
    pub attempts_done: u64,
    /// Support schedule, when the phase follows the schedule.
    #[serde(default)]
    pub schedule: Option<SupportScheduleState>,
    /// Adaptive assist, when the phase is adaptive.
    #[serde(default)]
    pub adaptive: Option<AdaptiveResume>,
    /// Deviation/assist timeline so far, with its clock and open second.
    #[serde(default)]
    pub deviation: Option<DeviationAssistRun>,
    /// Attempts discarded so far, earlier segments' included.
    #[serde(default)]
    pub discarded: Vec<crate::balance_recovery::DiscardedAttempt>,
}

/// Adaptive assist state saved at a pause.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdaptiveResume {
    /// Level, slow average and loss history.
    pub state: AdaptiveAssistState,
    /// Seconds left of the hold timer.
    pub hold_left_s: f64,
    /// `cone_min_deg` was above the learned limit at some point.
    pub cone_min_clamped: bool,
    /// Phase clock of the assist timeline, seconds of scored time.
    pub phase_t_s: f64,
    /// Time of the next timeline point.
    pub next_point_s: f64,
    /// Assist timeline points so far.
    #[serde(default)]
    pub timeline: Vec<AssistPoint>,
}

/// One phase of a balance session, plus the run the progress events belong to.
pub struct BalancePhaseRun<'a> {
    /// `train`, `test` or `observe`.
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
    /// Limits for riding out a lost or restarting controller link. Required: the caller sets
    /// every value, and an invalid one fails the phase before its first attempt.
    pub recovery: &'a RecoveryConfig,
    /// The FEAGI burst engine, read at attempt boundaries (paused?) and during attempts (is the
    /// burst counter rising?). Required: FEAGI is outside this crate, so the caller supplies it.
    pub burst: &'a dyn BurstClock,
    /// Memory-area counters, sampled before the first attempt and after each one into the
    /// `memory_areas` table. `None` writes no such table. Needs `data`.
    pub memory_counters: Option<&'a mut dyn MemoryCounterSource>,
    /// Told each attempt's reset pose when it is latched (the setup panel reads it live).
    /// `None` publishes nothing.
    pub reset_latch: Option<ResetLatchSink<'a>>,
}

/// Per-phase state: model facts and the gains sized once from the first reset.
struct BalanceSession {
    info: ModelInfo,
    sizing: Option<(SizingInputs, SpringGains)>,
}

/// The phase kinds a balance session runs.
pub const BALANCE_PHASES: [&str; 3] = ["train", "test", "observe"];

/// Run one phase (`train`, `test` or `observe`) and emit its progress.
///
/// An `observe` phase is a `test` phase that also writes the `observe` table (body measures as
/// fractions of the allowed angle) and refuses to run when learning cannot be frozen: it
/// watches the brain without teaching or changing it.
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
    run_balance_phase_from(
        run,
        &ResumeFrom::default(),
        controller,
        present_goal,
        events,
        cancel,
    )
    .map(|(report, _)| report)
}

/// [`run_balance_phase`] continuing from `resume` ("pause and save, resume later").
///
/// The unit is the training attempt (a seed, with the probe that follows it): the segment runs
/// `run.seeds[first_index..]`. `resume.engine_state` holds a [`BalanceResumeState`] exported by
/// the earlier segment. With `resume.boundary_stop` a skip waits for the probe due after the
/// current attempt, so the probe cadence stays aligned. Returns this segment's report (its
/// attempts only; merge the segments' attempts for the whole phase) and the export for the next
/// segment, `partial` when a skip ended the segment early.
pub fn run_balance_phase_from(
    run: &mut BalancePhaseRun<'_>,
    resume: &ResumeFrom,
    controller: &mut dyn BalanceController,
    present_goal: &mut dyn FnMut([f64; 3]) -> Result<(), TrainerError>,
    events: &mut dyn RunEventSink,
    cancel: &CancelToken,
) -> Result<(BalancePhaseReport, ResumeExport), TrainerError> {
    if !BALANCE_PHASES.contains(&run.phase) {
        return Err(TrainerError::Config(
            "balance phase must be train, test or observe".to_string(),
        ));
    }
    run.criterion.validate()?;
    run.recovery.validate()?;
    run.recovery
        .validate_for_burst_period(run.criterion.burst_period_s)?;
    if run.seeds.is_empty() {
        return Err(TrainerError::Config(format!(
            "{} seeds must contain at least one seed",
            run.phase
        )));
    }
    let brain_only = run.phase == "test" || run.phase == "observe";
    if brain_only && !run.criterion.scheduled_events.is_empty() {
        return Err(TrainerError::Config(format!(
            "the {} phase measures the brain only and takes no scheduled events",
            run.phase
        )));
    }
    if !run.criterion.scheduled_events.is_empty() && run.apply_event.is_none() {
        return Err(TrainerError::Config(
            "the criterion has scheduled events but no way to apply them (no apply_event callback)"
                .to_string(),
        ));
    }
    if brain_only {
        if run.deliver_reward.is_some() {
            return Err(TrainerError::Config(format!(
                "the {} phase measures the brain only and takes no reward sink",
                run.phase
            )));
        }
        if !matches!(run.support_mode, None | Some(SupportMode::Fixed(0.0))) {
            return Err(TrainerError::Config(format!(
                "the {} phase measures the brain only and runs at support 0",
                run.phase
            )));
        }
    }
    let support = run.criterion.support_config();
    let mode = run.support_mode.clone().unwrap_or(if brain_only {
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
        // Observe never teaches: an unfrozen brain would learn from the run it only watches.
        if run.phase == "observe" && !conditions.plasticity_frozen {
            let reason = conditions
                .plasticity_reason
                .clone()
                .unwrap_or_else(|| "learning was not frozen".to_string());
            if freeze_attempted {
                if let Some(control) = run.plasticity.as_deref_mut() {
                    let _ = control.restore();
                }
            }
            return Err(TrainerError::Runtime(format!(
                "the observe phase needs learning frozen and could not freeze it: {reason}"
            )));
        }
    }
    resume.check_within(run.seeds.len() as u64, "attempt")?;
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
        resume,
    );
    let restored = if freeze_attempted {
        run.plasticity
            .as_deref_mut()
            .map(|control| control.restore())
    } else {
        None
    };
    match (result, restored) {
        (Ok(done), None | Some(Ok(()))) => Ok(done),
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
    resume: &ResumeFrom,
) -> Result<(BalancePhaseReport, ResumeExport), TrainerError> {
    let brain_only = conditions.is_some();
    let saved: BalanceResumeState = resume.decode_state()?.unwrap_or_default();
    let info = controller.model_info()?;
    check_model(&info, criterion)?;
    let mut session = BalanceSession { info, sizing: None };
    let mut schedule = match (&mode, saved.schedule.clone()) {
        (_, Some(state)) => SupportSchedule::restore(*support, state),
        (SupportMode::Resume(level), None) => SupportSchedule::resume(*support, *level),
        (SupportMode::ResumeAdaptive(state), None) => SupportSchedule::resume(*support, state.a),
        _ => SupportSchedule::new(*support),
    };
    let mut adaptive_run = adaptive_on.then(|| {
        if let Some(saved) = saved.adaptive.clone() {
            let mut assist = AdaptiveAssist::resume(criterion, saved.state);
            assist.restore_pause_extras(saved.hold_left_s, saved.cone_min_clamped);
            let mut run = AdaptiveRun::new(assist);
            run.phase_t_s = saved.phase_t_s;
            run.next_point_s = saved.next_point_s;
            run.timeline = saved.timeline;
            return run;
        }
        AdaptiveRun::new(match &mode {
            SupportMode::Resume(level) => {
                AdaptiveAssist::resume(criterion, AdaptiveAssistState::new(*level, *level))
            }
            SupportMode::ResumeAdaptive(state) => AdaptiveAssist::resume(criterion, state.clone()),
            _ => AdaptiveAssist::start(criterion),
        })
    });
    let mut dev_assist = (!brain_only).then(|| saved.deviation.clone().unwrap_or_default());
    let offset = saved.attempts_done as usize;
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
    // Training attempts done, earlier segments included: the probe cadence follows it.
    let mut train_done: u32 = resume.first_index as u32;
    let mut probes_run: u32 = 0;
    let mut probes_frozen = true;
    let mut steps_table = StepsTable {
        actuators: None,
        actuator_force: Vec::new(),
    };
    if let Some(sink) = run.data.as_deref_mut() {
        sink.declare_table(&attempts_table_spec(criterion.walk.is_some()));
        if run.phase == "observe" {
            sink.declare_table(&observe_table_spec());
        }
        sink.event(EventTime::default(), "phase_start", run.phase);
    }
    // Baseline of the memory counters, after learning is frozen and before the first attempt.
    let memory_source = run.memory_counters.take();
    let mut memory = match (memory_source, run.data.as_deref_mut()) {
        (Some(source), Some(sink)) => Some(MemoryAreasRecorder::start(source, sink)),
        _ => None,
    };
    let mut recovery = RecoveryState::resume(run.recovery, saved.discarded.clone())?;
    let mut skipped = false;
    for seed in &run.seeds[resume.first_index as usize..] {
        // Skip: park while paused (a skip wakes the park), honor a cancel, then end the phase
        // cleanly once at least one attempt finished. The attempt in progress is never cut, and
        // a phase with no finished attempt has nothing to report, so it runs one attempt first.
        cancel.interrupt("balance-on-feet run cancelled".to_string())?;
        if cancel.is_skipped() && !attempts.is_empty() {
            skipped = true;
            break;
        }
        let level = match (&adaptive_run, &mode) {
            (Some(adaptive), _) => adaptive.assist.level(),
            (None, SupportMode::Fixed(level)) => *level,
            (None, _) => schedule.current_level(),
        };
        // An attempt whose controller link failed is discarded and re-run with the same seed;
        // the state it touched is put back so it leaves no trace in the scores.
        let record = loop {
            // Attempt boundary: the only place a pause or a paused burst engine parks the
            // phase and a cancel is honored between attempts. Mid-attempt only cancel is
            // checked.
            recovery.boundary(run.burst, cancel)?;
            let saved = (adaptive_run.clone(), dev_assist.clone());
            let mut watch = recovery.watch(run.burst);
            let result = run_balance_attempt(
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
                    reset_latch: &mut run.reset_latch,
                    phase: run.phase,
                    attempt_index: offset + attempts.len(),
                    probe: false,
                },
                &mut watch,
                cancel,
            );
            let slot = AttemptSlot {
                attempt_index: offset + attempts.len(),
                seed: *seed,
                probe: false,
            };
            match RecoveryState::classify(result, &watch, slot)? {
                AttemptVerdict::Completed(record) => break record,
                AttemptVerdict::Discard(discard) => {
                    (adaptive_run, dev_assist) = saved;
                    recovery.recover(discard, controller, cancel, run.data.as_deref_mut())?;
                }
            }
        };
        let stand = AttemptStand {
            balanced_s: record.balanced_s,
            support_level: record.support_level,
            outcome: outcome_name(record.outcome).to_string(),
            seed: record.seed,
            probe: false,
            train_attempts_before: None,
            adaptive: record.adaptive,
            walk: record.walk,
        };
        schedule.record_attempt(record.outcome, record.balanced_s);
        if let Some(adaptive) = adaptive_run.as_mut() {
            adaptive.assist.record_attempt_end(
                criterion,
                record.outcome,
                record
                    .stance_lost_tilt_deg
                    .or((record.outcome == BalanceAttemptOutcome::Fell)
                        .then_some(record.final_tilt_deg)),
            );
        }
        if let Some(sink) = run.data.as_deref_mut() {
            sink.row(
                ATTEMPTS_TABLE,
                &attempt_row(
                    run.phase,
                    offset + attempts.len(),
                    &record,
                    criterion.walk.is_some(),
                ),
            );
            if let Some(row) = observe_row(run.phase, offset + attempts.len(), &record) {
                sink.row(OBSERVE_TABLE, &row);
            }
            if let Some(memory) = memory.as_mut() {
                memory.after_attempt(sink, run.phase, offset + attempts.len());
            }
        }
        attempts.push(record);
        train_done += 1;
        events.emit(progress_event(
            run,
            (offset + offset + attempts.len()) as u64,
            samples_total,
            format!("{} seed {seed} support {level:.2}", run.phase),
            stand,
        ));
        if probe_every == 0 || !train_done.is_multiple_of(probe_every) {
            continue;
        }
        if cancel.is_skipped() && !resume.boundary_stop {
            skipped = true;
            break;
        }
        // Freeze learning, run one unassisted attempt, restore on every exit path. A discarded
        // probe restores learning before the link wait, then freezes again for the re-run.
        let mut record = loop {
            // Attempt boundary, before learning is frozen so a pause never holds it frozen.
            recovery.boundary(run.burst, cancel)?;
            let frozen = match run.plasticity.as_deref_mut() {
                Some(control) => {
                    control.freeze()?;
                    true
                }
                None => false,
            };
            probes_frozen &= frozen;
            let mut no_reward: Option<DeliverReward<'_>> = None;
            let mut watch = recovery.watch(run.burst);
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
                    reset_latch: &mut None,
                    phase: run.phase,
                    attempt_index: offset + attempts.len(),
                    probe: true,
                },
                &mut watch,
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
            let slot = AttemptSlot {
                attempt_index: offset + attempts.len(),
                seed: *seed,
                probe: true,
            };
            match RecoveryState::classify(probed, &watch, slot)? {
                AttemptVerdict::Completed(record) => break record,
                AttemptVerdict::Discard(discard) => {
                    recovery.recover(discard, controller, cancel, run.data.as_deref_mut())?;
                }
            }
        };
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
            walk: record.walk,
        };
        if let Some(sink) = run.data.as_deref_mut() {
            sink.row(
                ATTEMPTS_TABLE,
                &attempt_row(
                    run.phase,
                    offset + attempts.len(),
                    &record,
                    criterion.walk.is_some(),
                ),
            );
        }
        attempts.push(record);
        probes_run += 1;
        events.emit(progress_event(
            run,
            (offset + offset + attempts.len()) as u64,
            samples_total,
            format!("probe after {train_done} training attempts"),
            stand,
        ));
    }
    if let Some(sink) = run.data.as_deref_mut() {
        sink.event(
            EventTime {
                episode: Some((offset + offset + attempts.len()) as u64),
                ..EventTime::default()
            },
            "phase_end",
            run.phase,
        );
    }
    let mut report = BalancePhaseReport::from_attempts_with(attempts, support)?;
    report.assess_over_support(criterion);
    let discarded = recovery.into_discarded();
    report.discarded_attempts = discarded.clone();
    report.skipped = skipped;
    report.probe_plasticity_frozen = (probes_run > 0).then_some(probes_frozen);
    // A brain-only test applies no spring (zero gains), so there are no sized gains to report.
    (report.spring_sizing, report.spring_gains) = match session.sizing {
        Some((inputs, gains)) if !brain_only => (Some(inputs), Some(gains)),
        _ => (None, None),
    };
    let adaptive_export = adaptive_run.as_ref().map(|run| {
        let (hold_left_s, cone_min_clamped) = run.assist.pause_extras();
        AdaptiveResume {
            state: run.assist.state(),
            hold_left_s,
            cone_min_clamped,
            phase_t_s: run.phase_t_s,
            next_point_s: run.next_point_s,
            timeline: run.timeline.clone(),
        }
    });
    let dev_export = dev_assist.clone();
    if let Some(adaptive) = adaptive_run {
        report.support_level_next = Some(adaptive.assist.level());
        report.adaptive_state_next = Some(adaptive.assist.state());
        report.cone_min_above_loss_tilt = adaptive.assist.cone_min_clamped();
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
            confusion: None,
        },
    ));
    let attempts_total = offset as u64 + report.attempts.len() as u64;
    let seeds_done = train_done as u64;
    let export = ResumeExport {
        units_done: seeds_done,
        engine_state: serde_json::to_value(BalanceResumeState {
            attempts_done: attempts_total,
            schedule: matches!(mode, SupportMode::Fixed(_))
                .not()
                .then(|| schedule.state()),
            adaptive: adaptive_export,
            deviation: dev_export,
            discarded,
        })
        .unwrap_or(serde_json::Value::Null),
        partial: seeds_done < run.seeds.len() as u64,
    };
    Ok((report, export))
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
            readout: None,
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
    let names = std::iter::once(&criterion.torso_body)
        .chain(criterion.contact_bodies.iter())
        .chain(std::iter::once(&criterion.posture_top_body));
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

/// Bodies of the measure request, in order: the torso, the contact bodies (their CoM sets the
/// support centre) and the posture top body; each name once.
fn measured_bodies(criterion: &BalanceFeetCriterion) -> Vec<String> {
    let mut bodies: Vec<String> = Vec::new();
    for name in std::iter::once(&criterion.torso_body)
        .chain(criterion.contact_bodies.iter())
        .chain(std::iter::once(&criterion.posture_top_body))
    {
        let name = name.trim().to_string();
        if !name.is_empty() && !bodies.contains(&name) {
            bodies.push(name);
        }
    }
    bodies
}

/// The one `measure` request for a setup: the runner, the live readout and the reset-pose latch
/// all send it, so they read the same bodies and contact sets.
pub fn measure_request(criterion: &BalanceFeetCriterion) -> MeasureRequest {
    MeasureRequest {
        bodies: measured_bodies(criterion),
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
        actuator_force: criterion.record_actuators,
    }
}

/// One `measure` call, with its link reading judged by `watch`. An older controller that does
/// not know `a_except` fails the call; say so plainly instead of surfacing its raw error.
fn measure_checked(
    controller: &mut dyn BalanceController,
    request: &MeasureRequest,
    watch: &mut AttemptWatch<'_>,
) -> Result<Measurement, TrainerError> {
    let reading = controller.measure(request).map_err(|error| {
        let text = error.to_string();
        // A transient failure is the controller's state, never a statement about the request.
        if matches!(error, TrainerError::Transient(_)) {
            error
        } else if !request.contacts_except.is_empty() && text.to_lowercase().contains("contact") {
            TrainerError::Runtime(format!(
                "the controller rejected the \"every other body\" contact set (a_except): \
                 restart or update the controller, or turn off only_contacts_touch. \
                 Controller said: {text}"
            ))
        } else {
            error
        }
    })?;
    watch.observe(reading.link)?;
    // One look at the burst counter per measure, i.e. per burst of the attempt.
    watch.observe_bursts()?;
    Ok(reading)
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
/// timeline and the scored time that places its points. Cloned before an attempt so a
/// discarded attempt leaves no trace in it.
#[derive(Clone)]
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

/// Why the window-cover rule demoted a capped standing attempt from success to truncated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowDemotion {
    /// No whole `hold_s` success window completed.
    NoSuccessWindow,
    /// The success windows cover less of the scored time than `standing_fraction_for_success`.
    CoverBelowThreshold,
}

/// Snake-case name of a window demotion, as written to the run data and results.
pub fn window_demotion_name(reason: WindowDemotion) -> &'static str {
    match reason {
        WindowDemotion::NoSuccessWindow => "no_success_window",
        WindowDemotion::CoverBelowThreshold => "cover_below_threshold",
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
    lean_retightens: u32,
    success_windows: u32,
    /// Seconds in the current, unfinished success window.
    window_s: f64,
    /// Scored seconds the completed success windows cover.
    window_covered_s: f64,
    /// The current partial window continues unbroken from a completed one (set when a window
    /// completes, cleared by any lean past the cone or non-idle burst).
    partial_continues: bool,
    /// Why a capped success was demoted to truncated.
    demotion_reason: Option<WindowDemotion>,
}

impl AdaptiveAttemptTally {
    fn add(&mut self, level: f64, bursts: u32) {
        self.sum_a += level * f64::from(bursts);
        self.bursts += f64::from(bursts);
        self.min_a = Some(self.min_a.map_or(level, |m| m.min(level)));
    }

    /// Share of the scored time the success windows cover: the completed `hold_s` windows, plus
    /// the trailing partial window when it continues unbroken from a completed one (so a cap that
    /// is not a whole number of windows is not penalised for its remainder). 0 without a
    /// completed window.
    fn window_cover_fraction(&self, burst_period_s: f64) -> f64 {
        let seconds = self.bursts * burst_period_s;
        if seconds <= 0.0 || self.success_windows == 0 {
            return 0.0;
        }
        let trailing = if self.partial_continues {
            self.window_s
        } else {
            0.0
        };
        ((self.window_covered_s + trailing) / seconds).min(1.0)
    }

    fn stats(
        &self,
        end_cone_deg: f64,
        burst_period_s: f64,
        standing_fraction: f64,
        lost_stance: bool,
        start: &AdaptiveStart,
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
            success_windows: self.success_windows,
            window_cover_fraction: self.window_cover_fraction(burst_period_s),
            demotion_reason: self.demotion_reason,
            lean_retightens: self.lean_retightens,
            lost_stance,
            a_start: start.a,
            a_slow_start: start.a_slow,
            effective_max_deg: start.effective_max_deg,
            learned_loss_tilt_deg: start.learned_loss_tilt_deg,
        }
    }
}

/// The adaptive assist as an attempt began.
#[derive(Debug, Clone, Copy, PartialEq)]
struct AdaptiveStart {
    a: f64,
    a_slow: f64,
    effective_max_deg: f64,
    learned_loss_tilt_deg: Option<f64>,
}

impl AdaptiveStart {
    fn of(assist: &AdaptiveAssist, criterion: &BalanceFeetCriterion) -> Self {
        Self {
            a: assist.level(),
            a_slow: assist.slow(),
            effective_max_deg: assist.effective_max_deg(criterion),
            learned_loss_tilt_deg: assist.learned_loss_tilt_deg(criterion),
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
    /// `actuator_force_<name>` columns; empty when the controller reports no force.
    actuator_force: Vec<String>,
}

fn contact_column(body: &str) -> String {
    format!("contact_{}", body.trim())
}

fn actuator_column(name: &str) -> String {
    format!("actuator_{name}")
}

fn actuator_force_column(name: &str) -> String {
    format!("actuator_force_{name}")
}

fn steps_table_spec(
    criterion: &BalanceFeetCriterion,
    actuators: &[String],
    actuator_force: &[String],
) -> TableSpec {
    let mut columns = vec![
        ColumnSpec::unitless("phase", "Phase the burst belongs to."),
        ColumnSpec::unitless("attempt", "Attempt index within the phase."),
        ColumnSpec::unitless("probe", "The attempt is an unassisted probe."),
        ColumnSpec::unitless("seed", "Attempt seed."),
        ColumnSpec::unitless("seq", "Controller physics-step counter."),
        ColumnSpec::with_unit("sim_time", "s", "Simulated time."),
        ColumnSpec::with_unit("tilt", "deg", "Torso tilt from world up."),
        ColumnSpec::with_unit(
            "posture_tilt",
            "deg",
            "Posture tilt: the angle from world vertical of the line from the support centre (mean position of the contact bodies touching, all of them when none touch) to the posture top body. Absolute, from world vertical; see scored_tilt for the tilt the rules score.",
        ),
        ColumnSpec::with_unit(
            "scored_tilt",
            "deg",
            "Posture tilt from the attempt's reset pose: the angle between the posture line now and the line on the attempt's first sample (turned by the heading change). This is the tilt the upright, lean, fall and stance-loss rules score; it reads 0 at the reset pose.",
        ),
        ColumnSpec::with_unit("height", "m", "Torso height above world z."),
        ColumnSpec::unitless(
            "height_fraction",
            "Torso height divided by its height on the attempt's first sample (1 = at the reset pose).",
        ),
        ColumnSpec::with_unit(
            "ang_vel_xy",
            "rad/s",
            "Torso horizontal angular velocity hypot(wx, wy); empty when the controller did not report it.",
        ),
        ColumnSpec::with_unit(
            "tilt_x",
            "deg",
            "Signed lean: component of the tilt vector on the controller's first horizontal axis (world X for z-up). Leaning toward +Y reads positive here; leaning toward +X reads negative on tilt_y. See tilt_forward and tilt_left for the torso frame. Empty when the controller did not report the tilt vector.",
        ),
        ColumnSpec::with_unit(
            "tilt_y",
            "deg",
            "Signed lean: component of the tilt vector on the controller's second horizontal axis (world Y for z-up). Leaning toward +X reads negative here. Empty when the controller did not report the tilt vector.",
        ),
        ColumnSpec::with_unit(
            "tilt_forward",
            "deg",
            "Signed lean in the torso frame, positive leaning toward the torso's forward (the body's measured heading). Empty when the controller did not report the tilt vector or the heading.",
        ),
        ColumnSpec::with_unit(
            "tilt_left",
            "deg",
            "Signed lean in the torso frame, positive leaning toward the torso's left. Empty when the controller did not report the tilt vector or the heading.",
        ),
        ColumnSpec::with_unit(
            "posture_forward",
            "deg",
            "Signed posture lean in the torso frame, positive leaning toward the torso's forward (the head over the feet, not the torso). Empty when the controller did not report the heading.",
        ),
        ColumnSpec::with_unit(
            "posture_left",
            "deg",
            "Signed posture lean in the torso frame, positive leaning toward the torso's left. Empty when the controller did not report the heading.",
        ),
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
            "Tilt inside the limit and every contact body down (at least one when walking), the torso high enough and no other body on the world.",
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
    if criterion.walk.is_some() {
        columns.extend([
            ColumnSpec::with_unit("forward", "m", "Torso displacement along forward since the first scored burst."),
            ColumnSpec::with_unit("lateral", "m", "Torso displacement to the left of forward since the first scored burst."),
            ColumnSpec::with_unit(
                "progress",
                "m",
                "Ground gained along forward since the previous counted burst, above the best so far. Rocking over planted feet gains none.",
            ),
        ]);
    }
    columns.extend(
        actuators
            .iter()
            .map(|name| ColumnSpec::unitless(&actuator_column(name), "Actuator command (ctrl).")),
    );
    columns.extend(actuator_force.iter().map(|name| {
        ColumnSpec::unitless(
            &actuator_force_column(name),
            "Applied actuator force, as reported by the simulator.",
        )
    }));
    TableSpec {
        name: STEPS_TABLE.to_string(),
        description: "One row per counted burst.".to_string(),
        columns,
    }
}

fn attempts_table_spec(walking: bool) -> TableSpec {
    let mut columns = vec![
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
            ColumnSpec::unitless(
                "collapsed",
                "The body was continuously not upright (tilt over the limit, torso under the height fraction, a contact body off the world, or another body touching) for at least the fall confirm time after settling. Independent of the fall tilt rule, so it is set even when that rule is off.",
            ),
            ColumnSpec::with_unit(
                "collapsed_at_s",
                "s",
                "Time from the end of settling to the burst that confirmed the collapse; empty when the attempt never collapsed.",
            ),
            ColumnSpec::with_unit(
                "wobble",
                "rad/s",
                "Root-mean-square horizontal angular velocity of the torso over the upright bursts after settling. How fast the body sways; lower is steadier. Empty when no upright burst was measured.",
            ),
            ColumnSpec::with_unit(
                "tilt_sway",
                "deg",
                "Root-mean-square deviation of the torso tilt from its own mean over the upright bursts after settling. How far the body swings around its average lean; lower is steadier. Empty when no upright burst was measured.",
            ),
            ColumnSpec::unitless(
                "com_sway_fraction",
                "Root-mean-square horizontal deviation of the centre of mass from its mean over the upright bursts after settling, as a fraction of the reset torso height. Lower is steadier. Empty when no upright burst was measured or the reset height is unknown.",
            ),
            ColumnSpec::unitless(
                "spring_level",
                "Level the opening support spring was set to (0 for a brain-only attempt).",
            ),
            ColumnSpec::with_unit(
                "spring_deadband",
                "rad",
                "Dead band of the opening support spring's rotational term.",
            ),
            ColumnSpec::with_unit(
                "effective_max",
                "deg",
                "Adaptive: the cone's effective maximum at the start of the attempt, the smaller of the allowed angle and the learned stance-loss angle reduced by the margin. Empty for a non-adaptive attempt.",
            ),
            ColumnSpec::with_unit(
                "learned_loss_tilt",
                "deg",
                "Adaptive: the learned stance-loss angle at the start of the attempt (25th percentile of the recent losses). Empty before the first loss.",
            ),
            ColumnSpec::with_unit(
                "stance_lost_tilt",
                "deg",
                "Torso tilt on the first burst of the stance loss that ended the attempt. Empty when the stance was standing at the end.",
            ),
            ColumnSpec::unitless(
                "assist_start",
                "Adaptive: assist level at the start of the attempt, after any tightening from the last loss.",
            ),
            ColumnSpec::unitless(
                "assist_slow_start",
                "Adaptive: slow average of the assist level at the start of the attempt.",
            ),
            ColumnSpec::unitless(
                "success_windows",
                "Adaptive: success windows completed (hold time inside the cone with the support idle). Each one widens the cone a step. Empty for a non-adaptive attempt.",
            ),
            ColumnSpec::unitless(
                "lean_retightens",
                "Adaptive: times a lean past the cone outlasted the lean grace and tightened the assist. Empty for a non-adaptive attempt.",
            ),
            ColumnSpec::with_unit(
                "reset_tilt",
                "deg",
                "Absolute torso lean on the attempt's first sample (the reset pose), from world vertical. Every tilt rule is measured from this pose. Empty when no posture line was read.",
            ),
            ColumnSpec::unitless(
                "window_cover_fraction",
                "Adaptive: share of the scored time the success windows cover (whole hold windows plus an unbroken trailing part). A capped standing attempt is a success only when this reaches the standing-fraction threshold. Empty for a non-adaptive attempt.",
            ),
            ColumnSpec::unitless(
                "demotion_reason",
                "Adaptive: why a capped standing attempt was scored truncated instead of success: no_success_window (no whole hold window completed) or cover_below_threshold (windows cover too little of the scored time). Empty when the attempt was not demoted.",
            ),
    ];
    if walking {
        columns.extend([
            ColumnSpec::with_unit("forward", "m", "Forward displacement from the first scored burst."),
            ColumnSpec::with_unit("lateral", "m", "Lateral displacement from the first scored burst, left positive."),
            ColumnSpec::with_unit("mean_speed", "m/s", "Forward displacement over the scored time."),
            ColumnSpec::with_unit(
                "window_speed",
                "m/s",
                "Forward speed of the window the success rule judged: the whole attempt for the distance rule, the qualifying upright run for the time rule. Empty when no such window existed.",
            ),
            ColumnSpec::unitless("speed_error", "Mean speed minus the target, as a fraction of the target."),
            ColumnSpec::with_unit(
                "heading_error",
                "deg",
                "Angle between the net horizontal displacement and forward. Empty while the displacement is under half a torso height.",
            ),
            ColumnSpec::unitless("steps", "Touchdowns of any contact body."),
            ColumnSpec::with_unit("step_rate", "1/s", "Touchdowns per second of scored time."),
            ColumnSpec::unitless(
                "alternation",
                "Share of consecutive lone touchdowns made by a different contact body. Empty with fewer than two contact bodies or no consecutive touchdowns.",
            ),
        ]);
    }
    TableSpec {
        name: ATTEMPTS_TABLE.to_string(),
        description: "One row per finished attempt, probes included.".to_string(),
        columns,
    }
}

/// Name of the per-attempt body measures table an observe phase writes.
pub const OBSERVE_TABLE: &str = "observe";

/// The `observe` table: one row per attempt of an observe phase, body measures only.
pub fn observe_table_spec() -> TableSpec {
    TableSpec {
        name: OBSERVE_TABLE.to_string(),
        description: "One row per attempt of an observe phase: how the body behaved, with angles as fractions of the allowed angle."
            .to_string(),
        columns: vec![
            ColumnSpec::unitless("phase", "Phase the attempt belongs to."),
            ColumnSpec::unitless("attempt", "Attempt index within the phase."),
            ColumnSpec::unitless("outcome", "How the attempt ended."),
            ColumnSpec::with_unit("allowed_angle", "deg", "The allowed angle the fractions below are taken against."),
            ColumnSpec::with_unit(
                "time_to_limit",
                "s",
                "Seconds until the torso tilt first passed the allowed angle. Empty when it never did.",
            ),
            ColumnSpec::unitless(
                "upright_fraction",
                "Share of scored bursts that were upright (tilt inside the allowed angle, every contact body down).",
            ),
            ColumnSpec::unitless(
                "peak_tilt_fraction",
                "Largest tilt as a fraction of the allowed angle; above 1 means it passed the limit.",
            ),
            ColumnSpec::unitless(
                "tilt_sway_fraction",
                "Root-mean-square swing of the tilt around its mean over the upright bursts, as a fraction of the allowed angle. Lower is steadier. Empty when no upright burst was measured.",
            ),
            ColumnSpec::with_unit(
                "wobble_fraction",
                "1/s",
                "Root-mean-square horizontal angular velocity over the upright bursts, in allowed angles per second. Lower is steadier. Empty when no upright burst was measured.",
            ),
            ColumnSpec::unitless(
                "fall_forward_fraction",
                "Lean toward the torso's forward at the last scored sample, as a fraction of the allowed angle (negative leans back). Empty unless that sample was past the allowed angle or the controller did not report the heading.",
            ),
            ColumnSpec::unitless(
                "fall_left_fraction",
                "Lean toward the torso's left at the last scored sample, as a fraction of the allowed angle (negative leans right). Empty under the same conditions.",
            ),
            ColumnSpec::with_unit(
                "fall_direction",
                "deg",
                "Direction of that lean: 0 forward, 90 left, 180 back, -90 right. Empty under the same conditions.",
            ),
        ],
    }
}

fn observe_row(
    phase: &str,
    index: usize,
    record: &BalanceAttemptRecord,
) -> Option<Vec<RunDataValue>> {
    let body = record.observe.as_ref()?;
    Some(vec![
        RunDataValue::text(phase),
        RunDataValue::count(index as u64),
        RunDataValue::text(outcome_name(record.outcome)),
        RunDataValue::Float(body.allowed_angle_deg),
        RunDataValue::opt_float(body.time_to_limit_s),
        RunDataValue::Float(record.upright_fraction),
        RunDataValue::Float(body.peak_tilt_fraction),
        RunDataValue::opt_float(body.tilt_sway_fraction),
        RunDataValue::opt_float(body.wobble_fraction_per_s),
        RunDataValue::opt_float(body.fall_forward_fraction),
        RunDataValue::opt_float(body.fall_left_fraction),
        RunDataValue::opt_float(body.fall_direction_deg),
    ])
}

fn attempt_row(
    phase: &str,
    index: usize,
    record: &BalanceAttemptRecord,
    walking: bool,
) -> Vec<RunDataValue> {
    let mut row = vec![
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
        RunDataValue::Bool(record.collapsed_at_bursts.is_some()),
        RunDataValue::opt_float(record.collapsed_at_s),
        RunDataValue::opt_float(record.wobble_rad_s),
        RunDataValue::opt_float(record.tilt_sway_deg),
        RunDataValue::opt_float(record.com_sway_fraction),
        RunDataValue::opt_float(record.spring_applied.map(|s| s.level)),
        RunDataValue::opt_float(record.spring_applied.map(|s| s.deadband_rad)),
    ];
    let adaptive = record.adaptive.as_ref();
    row.extend([
        RunDataValue::opt_float(adaptive.map(|a| a.effective_max_deg)),
        RunDataValue::opt_float(adaptive.and_then(|a| a.learned_loss_tilt_deg)),
        RunDataValue::opt_float(record.stance_lost_tilt_deg),
        RunDataValue::opt_float(adaptive.map(|a| a.a_start)),
        RunDataValue::opt_float(adaptive.map(|a| a.a_slow_start)),
        adaptive.map_or(RunDataValue::Null, |a| {
            RunDataValue::count(u64::from(a.success_windows))
        }),
        adaptive.map_or(RunDataValue::Null, |a| {
            RunDataValue::count(u64::from(a.lean_retightens))
        }),
        RunDataValue::opt_float(record.reset_tilt_deg),
        RunDataValue::opt_float(adaptive.map(|a| a.window_cover_fraction)),
        adaptive
            .and_then(|a| a.demotion_reason)
            .map_or(RunDataValue::Null, |reason| {
                RunDataValue::text(window_demotion_name(reason))
            }),
    ]);
    if walking {
        let walk = record.walk.as_ref();
        row.extend([
            RunDataValue::opt_float(walk.map(|w| w.forward_m)),
            RunDataValue::opt_float(walk.map(|w| w.lateral_m)),
            RunDataValue::opt_float(walk.map(|w| w.mean_speed_m_s)),
            RunDataValue::opt_float(walk.and_then(|w| w.window_speed_m_s)),
            RunDataValue::opt_float(walk.map(|w| w.speed_error_fraction)),
            RunDataValue::opt_float(walk.and_then(|w| w.heading_error_deg)),
            walk.map_or(RunDataValue::Null, |w| {
                RunDataValue::count(u64::from(w.steps))
            }),
            RunDataValue::opt_float(walk.map(|w| w.step_rate_hz)),
            RunDataValue::opt_float(walk.and_then(|w| w.alternation_fraction)),
        ]);
    }
    row
}

/// What one attempt writes to, and the callback it applies events through.
struct AttemptHooks<'r, 'a> {
    data: &'r mut Option<&'a mut dyn RunDataSink>,
    steps: &'r mut StepsTable,
    apply_event: &'r mut Option<ApplyEvent<'a>>,
    reset_latch: &'r mut Option<ResetLatchSink<'a>>,
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
        if self.steps.actuators.is_none() {
            let names: Vec<String> = sample.actuators.keys().cloned().collect();
            self.steps.actuator_force = sample.actuator_force.keys().cloned().collect();
            sink.declare_table(&steps_table_spec(
                criterion,
                &names,
                &self.steps.actuator_force,
            ));
            self.steps.actuators = Some(names);
        }
        let actuators = self.steps.actuators.as_ref().expect("declared above");
        let mut row = vec![
            RunDataValue::text(self.phase),
            RunDataValue::count(self.attempt_index as u64),
            RunDataValue::Bool(self.probe),
            RunDataValue::count(sample.seed),
            RunDataValue::count(sample.seq),
            RunDataValue::Float(sim_time_s),
            RunDataValue::Float(sample.tilt_deg),
            RunDataValue::opt_float(sample.posture.map(|p| p.tilt_deg)),
            RunDataValue::Float(sample.scored_tilt_deg()),
            RunDataValue::opt_float(sample.torso_height_m),
            RunDataValue::opt_float(sample_height_fraction(sample)),
            RunDataValue::opt_float(
                sample
                    .stability
                    .as_ref()
                    .and_then(|st| st.horizontal_ang_vel_rad_s),
            ),
        ];
        let lean = sample
            .stability
            .as_ref()
            .map(StabilitySample::tilt_components_deg)
            .unwrap_or_default();
        row.extend([
            RunDataValue::opt_float(lean.x),
            RunDataValue::opt_float(lean.y),
            RunDataValue::opt_float(lean.forward),
            RunDataValue::opt_float(lean.left),
            RunDataValue::opt_float(sample.posture.and_then(|p| p.forward_deg)),
            RunDataValue::opt_float(sample.posture.and_then(|p| p.left_deg)),
        ]);
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
        if criterion.walk.is_some() {
            let walk = sample.walk.as_ref();
            row.extend([
                RunDataValue::opt_float(walk.map(|w| w.forward_m)),
                RunDataValue::opt_float(walk.map(|w| w.lateral_m)),
                RunDataValue::opt_float(walk.map(|w| w.progress_m)),
            ]);
        }
        row.extend(actuators.iter().map(|name| {
            sample
                .actuators
                .get(name)
                .map_or(RunDataValue::Null, |v| RunDataValue::Float(*v))
        }));
        row.extend(self.steps.actuator_force.iter().map(|name| {
            sample
                .actuator_force
                .get(name)
                .map_or(RunDataValue::Null, |v| RunDataValue::Float(*v))
        }));
        sink.row(STEPS_TABLE, &row);
    }
}

/// Forward tracking of one walking attempt: the start position, the forward and left axes, and
/// the best forward displacement so far.
struct WalkRun {
    forward: [f64; 2],
    start: Option<[f64; 2]>,
    best_forward_m: f64,
    pending_progress_m: f64,
}

impl WalkRun {
    fn new(heading_deg: f64) -> Self {
        Self {
            forward: WalkGoal::forward_xy(heading_deg),
            start: None,
            best_forward_m: 0.0,
            pending_progress_m: 0.0,
        }
    }

    /// Reading for the torso at `com`. The start is the first scored (non-settling) position.
    fn read(&mut self, com: [f64; 3], settling: bool) -> WalkReading {
        if settling {
            return WalkReading {
                forward_m: 0.0,
                lateral_m: 0.0,
                progress_m: 0.0,
            };
        }
        let start = *self.start.get_or_insert([com[0], com[1]]);
        let (dx, dy) = (com[0] - start[0], com[1] - start[1]);
        let forward_m = dx * self.forward[0] + dy * self.forward[1];
        let lateral_m = -dx * self.forward[1] + dy * self.forward[0];
        self.pending_progress_m += (forward_m - self.best_forward_m).max(0.0);
        self.best_forward_m = self.best_forward_m.max(forward_m);
        WalkReading {
            forward_m,
            lateral_m,
            progress_m: 0.0,
        }
    }

    /// Ground gained since the last call, m.
    fn take_progress(&mut self) -> f64 {
        std::mem::take(&mut self.pending_progress_m)
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
    watch: &mut AttemptWatch<'_>,
    cancel: &CancelToken,
) -> Result<BalanceAttemptRecord, TrainerError> {
    let torso = criterion.torso_body.trim().to_string();
    let keyframe = Some(criterion.reset_keyframe.trim()).filter(|name| !name.is_empty());
    // The link at the reset is the attempt's baseline; any later response that differs, or is
    // not `linked`, ends the attempt with a transient error and the phase discards it.
    let reset = controller.reset_episode(keyframe)?;
    watch.observe(reset.link)?;
    let request = measure_request(criterion);
    let first = measure_checked(controller, &request, watch)?;
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
                    let com = torso_now.subtree_com.ok_or_else(|| {
                        TrainerError::Runtime(
                            "cannot size the support spring: the measure reply has no \
                             subtree_com; set support_com_height_m"
                                .to_string(),
                        )
                    })?;
                    com[2] - floor
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
    let mut walk_run = match &criterion.walk {
        Some(goal) => {
            let reset_height_m = first
                .bodies
                .get(&torso)
                .and_then(|body| body.com)
                .map(|com| com[2])
                .ok_or_else(|| {
                    TrainerError::Runtime(format!(
                        "a walking goal needs the torso position: measure reply has no com for {torso}"
                    ))
                })?;
            goal.check_feasible(criterion.hold_s, reset_height_m)?;
            let heading_deg = goal.resolve_heading_deg(
                &torso,
                first.bodies.get(&torso).ok_or_else(|| {
                    TrainerError::Runtime(format!("measure reply has no body {torso}"))
                })?,
            )?;
            hooks.marker(
                first.sim_time_s,
                "walk_heading",
                &format!("{heading_deg:.3} deg"),
            );
            Some(WalkRun::new(heading_deg))
        }
        None => None,
    };
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
    let spring_applied = SpringApplied {
        level: settle_level,
        deadband_rad,
    };
    let mut spring_level = settle_level;
    let mut spring_cone_deg = criterion.support_deadband_deg;
    let mut tally = AdaptiveAttemptTally::default();
    let adaptive_start = adaptive
        .as_deref()
        .map(|run| AdaptiveStart::of(&run.assist, criterion));
    // Adaptive only: the spring starts with the criterion's weight support. Its dead band is the
    // same cone the lean and success-window tests score against.
    let mut spring_gravity_on = true;
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
    // Tell the applier a new attempt began, so a stimulation hold the last one left running ends
    // before this one's settle window.
    if !due_events.is_empty() {
        if let Some(apply) = hooks.apply_event.as_deref_mut() {
            apply(&crate::run_actions::ScheduledAction::AttemptStart)?;
        }
    }
    let mut was_perturbing = false;
    let observing = hooks.phase == "observe";
    let mut observe_track = ObserveTrack::default();
    let mut last_sim_time_s: f64;
    let mut result = loop {
        if cancel.is_cancelled() {
            let _ = controller.clear_body_spring(&torso);
            return Err(TrainerError::Cancelled(
                "balance-on-feet run cancelled".to_string(),
            ));
        }
        present_goal(features)?;
        let reading = measure_checked(controller, &request, watch)?;
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
        let feet_down = criterion.contacts_satisfied(
            &reading
                .contacts
                .iter()
                .take(contact_count)
                .map(|set| set.touching)
                .collect::<Vec<bool>>(),
        );
        let posture = measure_posture(&reading, criterion)?;
        let mut perturbing = false;
        if reading.terminal.is_none() {
            if let Some(run) = adaptive.as_deref() {
                // Level, cone and weight support are sent only when they changed enough to matter.
                let cone_now = if settling {
                    criterion.support_deadband_deg
                } else {
                    run.assist.cone_deg(criterion)
                };
                // Weight support only while the feet are loaded: lifting a foot drops it to 0, and
                // the criterion's fraction comes back once the stance holds again (every contact
                // body down; any one when walking).
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
                        None,
                    )?;
                    spring_level = level_now;
                    spring_cone_deg = cone_now;
                    spring_gravity_on = gravity_want;
                }
            } else if (spring_level - level_now).abs() > 1.0e-12 {
                controller.update_body_spring(&torso, level_now, None, None, None)?;
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
        let walk_reading = match walk_run.as_mut() {
            Some(run) => {
                let com = body.com.ok_or_else(|| {
                    TrainerError::Runtime(format!(
                        "a walking goal needs the torso position: measure reply has no com for {torso}"
                    ))
                })?;
                Some(run.read(com, settling))
            }
            None => None,
        };
        // Lean is measured from the attempt's reset pose; the first sample is that pose.
        let torso_now = body.tilt_vec.and_then(|vec| {
            LeanReference::from_tilt_vec(vec, body.heading.as_ref().and_then(|h| h.yaw_rad))
        });
        let torso_lean =
            torso_now.map(|now| now.lean_from(open.reset_torso.as_ref().unwrap_or(&now)));
        let mut sample = BalanceSample {
            seq: reading.physics_steps,
            seed,
            tilt_deg: body.tilt_rad.to_degrees(),
            posture: Some(
                posture.sample(body.heading.as_ref().and_then(|heading| heading.yaw_rad)),
            ),
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
            support_need: Some(match torso_lean {
                // From the torso's reset orientation, like the scored posture tilt.
                Some(lean) => (predicted_relative_tilt_deg(
                    &lean,
                    // Absent ang_vel means no lookahead term, as when it defaulted to zeros.
                    body.ang_vel.unwrap_or([0.0; 3]),
                    criterion.need_lookahead_s,
                ) / criterion.max_tilt_deg)
                    .clamp(0.0, 1.0),
                None => support_need(
                    body.tilt_rad,
                    body.tilt_vec,
                    body.ang_vel.unwrap_or([0.0; 3]),
                    criterion.max_tilt_deg,
                    criterion.need_lookahead_s,
                ),
            }),
            support_effort: wrench.map(|w| support_effort(w, &sizing)),
            support_torque_nm: wrench.map(|w| w.wrench_norm_torque),
            support_force_n: wrench.map(|w| w.wrench_norm_force),
            other_contact_touching: other.is_some_and(|set| set.touching),
            other_contact_bodies: other.map(|set| set.bodies.clone()).unwrap_or_default(),
            torso_height_m: body.com.map(|c| c[2]),
            reset_height_m: open.reset_height_m,
            assist_slow: None,
            assist_rise: None,
            actuators: reading.actuators.clone(),
            actuator_force: reading.actuator_force.clone(),
            stability: Some(StabilitySample::from_body(body)),
            walk: walk_reading,
        };
        if open.reset_posture.is_none() {
            if let (Some(reference), Some(sink)) = (
                sample.posture.and_then(|p| p.reference),
                hooks.reset_latch.as_deref_mut(),
            ) {
                sink(&ResetLatch {
                    reference,
                    episode_id: reading.episode_id,
                    physics_steps: reading.physics_steps,
                });
            }
        }
        let posture_lean = sample.score_posture_from(open.reset_posture.as_ref());
        if observing && !settling {
            observe_track.add(&sample, criterion.max_tilt_deg, elapsed_s - settle_s);
        }
        let mut step = observe_balance_sample(&open, &sample, criterion, seed)?;
        let (counted, mut outcome) = match &step {
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
                // The capped last burst of a standing attempt still counts toward its windows.
                if matches!(
                    step,
                    BalanceAttemptStep::Continue(_)
                        | BalanceAttemptStep::Finished(BalanceAttemptRecord {
                            outcome: BalanceAttemptOutcome::Success,
                            ..
                        })
                ) && reading.terminal.is_none()
                {
                    // The posture line's lean, carried forward along the torso's swing (the
                    // torso's angular velocity, not a difference of successive centres, so a
                    // foot lifting does not read as a jump).
                    let predicted = match &posture_lean {
                        // From the reset pose, like the scored tilt.
                        Some(lean) => predicted_relative_tilt_deg(
                            lean,
                            body.ang_vel.unwrap_or([0.0; 3]),
                            criterion.need_lookahead_s,
                        ),
                        None => predicted_tilt_deg(
                            posture.tilt_rad,
                            Some(posture.tilt_vec_rad),
                            // Absent ang_vel means no lookahead term, as before.
                            body.ang_vel.unwrap_or([0.0; 3]),
                            criterion.need_lookahead_s,
                        ),
                    };
                    // One reference cone for the spring, the lean test and the success window.
                    let dt_s = f64::from(new_bursts) * criterion.burst_period_s;
                    if predicted > run.assist.cone_deg(criterion) {
                        tally.window_s = 0.0;
                        tally.partial_continues = false;
                        lean_s += dt_s;
                        if lean_s > criterion.lean_grace_s + 1.0e-9 {
                            run.assist.lean_retighten(criterion);
                            lean_s = 0.0;
                            tally.lean_retightens += 1;
                        }
                    } else {
                        lean_s = 0.0;
                        if criterion.support_is_idle(sample.support_effort) {
                            tally.window_s += dt_s;
                            if tally.window_s >= criterion.hold_s - 1.0e-9 {
                                run.assist.success_window(criterion);
                                tally.success_windows += 1;
                                tally.window_covered_s += tally.window_s;
                                tally.window_s = 0.0;
                                tally.partial_continues = true;
                            }
                        } else {
                            tally.window_s = 0.0;
                            tally.partial_continues = false;
                        }
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
        // A capped standing attempt is a success only when success windows cover its standing
        // share: a held passive lean (or a held support) is not success and pays no Pleasure.
        // At least one whole window must have completed.
        if adaptive.is_some() && criterion.walk.is_none() {
            if let BalanceAttemptStep::Finished(record) = &mut step {
                let reason = if tally.success_windows == 0 {
                    Some(WindowDemotion::NoSuccessWindow)
                } else if tally.window_cover_fraction(criterion.burst_period_s)
                    < criterion.standing_fraction_for_success - 1.0e-9
                {
                    Some(WindowDemotion::CoverBelowThreshold)
                } else {
                    None
                };
                if record.outcome == BalanceAttemptOutcome::Success && reason.is_some() {
                    tally.demotion_reason = reason;
                    record.outcome = BalanceAttemptOutcome::Truncated;
                    outcome = Some(BalanceAttemptOutcome::Truncated);
                }
            }
        }
        if counted && !settling {
            if let (Some(run), Some(reading)) = (walk_run.as_mut(), sample.walk.as_mut()) {
                reading.progress_m = run.take_progress();
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
            &adaptive_start.expect("captured when adaptive is on"),
        ));
    }
    if was_perturbing {
        hooks.marker(last_sim_time_s, "perturb_end", "");
    }
    hooks.marker(last_sim_time_s, "attempt_end", outcome_name(result.outcome));
    // The next reset clears it anyway; a death may already have cleared it.
    let _ = controller.clear_body_spring(&torso);
    result.spring_applied = Some(spring_applied);
    if observing && criterion.max_tilt_deg > 0.0 {
        result.observe = Some(observe_track.measures(&result, criterion.max_tilt_deg));
    }
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

/// JSON body of `status`, whose reply carries the controller's FEAGI link.
pub fn status_request() -> serde_json::Value {
    serde_json::json!({"action": "status"})
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

/// JSON body of `update_body_spring`. `deadband_rad`, `gravity_comp` and `ref_up_world` are sent
/// only when `Some`.
pub fn update_body_spring_request(
    body: &str,
    level: f64,
    deadband_rad: Option<f64>,
    gravity_comp: Option<f64>,
    ref_up_world: Option<[f64; 3]>,
) -> serde_json::Value {
    let mut request =
        serde_json::json!({"action": "update_body_spring", "body": body, "level": level});
    if let Some(deadband) = deadband_rad {
        request["deadband_rad"] = serde_json::json!(deadband);
    }
    if let Some(fraction) = gravity_comp {
        request["gravity_comp"] = serde_json::json!(fraction);
    }
    if let Some(up) = ref_up_world {
        request["ref_up_world"] = serde_json::json!(up);
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
    if request.actuator_force {
        body["actuator_force"] = serde_json::Value::Bool(true);
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
            io_failure(&error, format!("balance control connect failed: {error}"))
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
                io_failure(&error, format!("balance control {what} failed: {error}")),
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
                TrainerError::Transient(
                    "balance control closed the connection before any response".to_string(),
                ),
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

    fn reset_episode(&mut self, keyframe: Option<&str>) -> Result<ResetReply, TrainerError> {
        let result = self.command(&reset_episode_request(keyframe))?;
        let episode_id = result
            .get("episode_id")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| {
                TrainerError::Runtime("reset_episode reply has no episode_id".to_string())
            })?;
        Ok(ResetReply {
            episode_id,
            link: link_of(&result, "reset_episode")?,
        })
    }

    fn link_status(&mut self) -> Result<LinkReading, TrainerError> {
        let result = self.command(&status_request())?;
        link_of(&result, "status")
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
        ref_up_world: Option<[f64; 3]>,
    ) -> Result<(), TrainerError> {
        self.command(&update_body_spring_request(
            body,
            level,
            deadband_rad,
            gravity_comp,
            ref_up_world,
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
        let result = self.command(&measure_request_body(request))?;
        // Name a missing `link` before the generic decode can fail on it.
        link_of(&result, "measure")?;
        serde_json::from_value(result).map_err(|error| {
            TrainerError::Runtime(format!("balance measure decode failed: {error}"))
        })
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

/// Whether an I/O failure means the peer was gone before it answered: the connection was
/// refused or reset, or ended before any byte (a controller that is restarting).
fn peer_gone(kind: std::io::ErrorKind) -> bool {
    use std::io::ErrorKind;
    matches!(
        kind,
        ErrorKind::ConnectionRefused | ErrorKind::ConnectionReset | ErrorKind::UnexpectedEof
    )
}

/// An I/O failure as a trainer error: [`TrainerError::Transient`] when the peer was gone,
/// [`TrainerError::Runtime`] for anything else (a timeout, a bad address, a broken socket).
fn io_failure(error: &std::io::Error, message: String) -> TrainerError {
    if peer_gone(error.kind()) {
        TrainerError::Transient(message)
    } else {
        TrainerError::Runtime(message)
    }
}

/// The `link: {state, epoch}` of a controller result. It is required: a reply with no `link`, a
/// `null` one, or one that is not `{state, epoch}` is an error naming the field and the action.
fn link_of(result: &serde_json::Value, what: &str) -> Result<LinkReading, TrainerError> {
    match result.get("link") {
        None | Some(serde_json::Value::Null) => Err(TrainerError::Runtime(format!(
            "{what} reply has no `link` ({{state, epoch}} is required in every physics reply; \
             update the controller)"
        ))),
        Some(link) => serde_json::from_value(link.clone()).map_err(|error| {
            TrainerError::Runtime(format!("{what} reply `link` decode failed: {error}"))
        }),
    }
}

/// Whether an error envelope says the same command may succeed when sent again.
fn envelope_is_retryable(envelope: Option<&serde_json::Value>) -> bool {
    envelope
        .and_then(|value| value.get("retryable"))
        .and_then(serde_json::Value::as_bool)
        == Some(true)
}

fn decode_envelope(status: u16, body: &str) -> Result<serde_json::Value, TrainerError> {
    let envelope: Option<serde_json::Value> = serde_json::from_str(body.trim()).ok();
    if status != 200 {
        let detail = envelope
            .as_ref()
            .and_then(|value| value.get("error"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let message = format!("balance control HTTP failed: status {status} {detail}");
        // Only an envelope that says `retryable: true` is transient (the controller's physics
        // loop was busy); every other non-200 is a rejected or failed command.
        return Err(if envelope_is_retryable(envelope.as_ref()) {
            TrainerError::Transient(message)
        } else {
            TrainerError::Runtime(message)
        });
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
        return Err(if envelope_is_retryable(Some(&envelope)) {
            TrainerError::Transient(message.to_string())
        } else {
            TrainerError::Runtime(message.to_string())
        });
    }
    envelope
        .get("result")
        .cloned()
        .ok_or_else(|| TrainerError::Runtime("balance control response has no result".to_string()))
}

/// Run controls this engine's loop honours (pause, skip and cancel; at the attempt; walk runs this engine).
pub const CONTROL: crate::control::EngineControl = crate::control::EngineControl {
    pause: true,
    skip: true,
    cancel: true,
    boundary: crate::control::ControlBoundary::Attempt,
};

#[cfg(test)]
mod tests {
    #[test]
    fn support_need_is_lower_when_swinging_back_toward_upright() {
        let th = 0.1_f64;
        // Leaning +Y: tilt_vec = (th, 0); omega about -x... swinging back means d(tilt_vec)/dt < 0 on x.
        let back = [th, 0.0];
        let om = [1.0, 0.0, 0.0];
        let old = support_need(th, None, om, 15.0, 0.05);
        let new = support_need(th, Some(back), om, 15.0, 0.05);
        assert!(new < old, "{new} vs {old}");
        assert!(new < support_need(th, Some(back), [0.0; 3], 15.0, 0.05));
    }

    #[test]
    fn predicted_tilt_follows_the_lean_direction() {
        let th = 0.1_f64;
        let w = 1.0;
        let la = 0.05;
        let cur = th.to_degrees();
        // (lean vector, omega) pairs; tilt_vec = (uy, -ux): +Y lean -> (th, 0), +X lean -> (0, -th).
        // Tipping further: world omega (wx, wy) with d(tilt_vec)/dt = (-wx, -wy).
        let tipping = [
            ([0.0, -th], [0.0, w, 0.0]), // forward (+X) lean, tipping +X
            ([0.0, th], [0.0, -w, 0.0]), // backward (-X)
            ([th, 0.0], [-w, 0.0, 0.0]), // left (+Y)
            ([-th, 0.0], [w, 0.0, 0.0]), // right (-Y)
        ];
        for (r, om) in tipping {
            let tilt = f64::hypot(r[0], r[1]);
            let p = predicted_tilt_deg(tilt, Some(r), om, la);
            assert!(p > cur + 1.0e-9, "tipping {r:?}: {p} vs {cur}");
            let back = [-om[0], -om[1], 0.0];
            let q = predicted_tilt_deg(tilt, Some(r), back, la);
            assert!(q < cur - 1.0e-9, "swinging back {r:?}: {q} vs {cur}");
        }
    }

    #[test]
    fn predicted_tilt_without_tilt_vec_keeps_the_old_formula() {
        let p = predicted_tilt_deg(0.1, None, [0.3, 0.4, 0.0], 0.1);
        assert!((p - (0.1 + 0.1 * 0.5_f64).to_degrees()).abs() < 1.0e-9);
    }

    use super::*;
    use crate::control::CollectingEventSink;
    use crate::run_actions::ScheduledAction;

    /// One burst is 0.05 s; hold 0.1 s = 2 bursts, cap 0.2 s = 4 bursts. One physics step per
    /// burst so legacy tests stay readable.
    fn criterion() -> BalanceFeetCriterion {
        let mut c = BalanceFeetCriterion::new(
            "torso",
            vec!["contact_a".to_string(), "contact_b".to_string()],
            "head",
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
            posture: None,
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
            actuator_force: BTreeMap::new(),
            stability: None,
            walk: None,
        }
    }

    #[test]
    fn height_fraction_is_height_over_reset_height() {
        let mut s = sample(1, 1, 0.0);
        assert_eq!(sample_height_fraction(&s), None);
        s.torso_height_m = Some(0.4);
        assert_eq!(sample_height_fraction(&s), Some(1.0));
        s.reset_height_m = Some(0.8);
        assert_eq!(sample_height_fraction(&s), Some(0.5));
        s.reset_height_m = Some(0.0);
        assert_eq!(sample_height_fraction(&s), None);
    }

    #[test]
    fn steps_table_declares_height_fraction_after_height() {
        let spec = steps_table_spec(&fall_criterion(), &[], &[]);
        let names: Vec<&str> = spec.columns.iter().map(|c| c.name.as_str()).collect();
        let h = names.iter().position(|n| *n == "height").unwrap();
        assert_eq!(names[h + 1], "height_fraction");
        assert_eq!(names[h + 2], "ang_vel_xy");
        let col = spec
            .columns
            .iter()
            .find(|c| c.name == "ang_vel_xy")
            .unwrap();
        assert_eq!(col.unit.as_deref(), Some("rad/s"));
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
    fn collapse_is_detected_with_the_fall_rule_off() {
        let mut c = fall_criterion();
        c.fall_tilt_deg = 0.0;
        assert_eq!(c.fall_confirm_bursts(), 5);
        let mut open = OpenBalanceAttempt::fresh();
        open = fed(&open, &balanced(1, 1), &c);
        for seq in 2..=5 {
            open = fed(&open, &sample(seq, 1, 70.0), &c);
            assert_eq!(open.collapsed_at_bursts, None, "seq {seq}");
        }
        open = fed(&open, &sample(6, 1, 70.0), &c);
        assert_eq!(open.collapsed_at_bursts, Some(6));
        let mut last = sample(7, 1, 70.0);
        last.terminal = Some(BalanceTerminal::Interrupted);
        match observe_balance_sample(&open, &last, &c, 1).expect("obs") {
            BalanceAttemptStep::Finished(r) => {
                assert_eq!(r.outcome, BalanceAttemptOutcome::Interrupted);
                assert_eq!(r.collapsed_at_bursts, Some(6));
                assert_eq!(r.collapsed_at_s, Some(6.0 * c.burst_period_s));
            }
            BalanceAttemptStep::Continue(_) => panic!("terminal ends the attempt"),
        }
    }

    #[test]
    fn a_brief_dip_is_not_a_collapse() {
        let mut c = fall_criterion();
        c.fall_tilt_deg = 0.0;
        let mut open = OpenBalanceAttempt::fresh();
        open = fed(&open, &balanced(1, 1), &c);
        for seq in 2..=5 {
            open = fed(&open, &sample(seq, 1, 70.0), &c);
        }
        open = fed(&open, &balanced(6, 1), &c);
        for seq in 7..=10 {
            open = fed(&open, &sample(seq, 1, 70.0), &c);
        }
        assert_eq!(open.collapsed_at_bursts, None);
    }

    #[test]
    fn an_upright_attempt_is_never_collapsed() {
        let mut c = fall_criterion();
        c.fall_tilt_deg = 0.0;
        let mut open = OpenBalanceAttempt::fresh();
        for seq in 1..=10 {
            open = fed(&open, &balanced(seq, 1), &c);
        }
        assert_eq!(open.collapsed_at_bursts, None);
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
            reset_tilt_deg: None,
            stance_lost_tilt_deg: None,
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
            collapsed_at_bursts: None,
            collapsed_at_s: None,
            wobble_rad_s: None,
            tilt_sway_deg: None,
            com_sway_fraction: None,
            adaptive: None,
            walk: None,
            spring_applied: None,
            observe: None,
        }
    }

    fn with_stability(
        mut s: BalanceSample,
        w: f64,
        tilt: [f64; 2],
        com: [f64; 2],
    ) -> BalanceSample {
        s.stability = Some(StabilitySample {
            horizontal_ang_vel_rad_s: Some(w),
            tilt_vec_rad: Some(tilt),
            com_xy_m: Some(com),
            yaw_rad: None,
        });
        s
    }

    /// Feed `samples` (seq 1..) into a fresh attempt with a hold long enough not to finish, then
    /// return the open attempt.
    fn stability_run(samples: Vec<BalanceSample>) -> OpenBalanceAttempt {
        let mut c = criterion();
        c.hold_s = 1.0e6;
        c.attempt_cap_s = 1.0e6;
        let mut open = OpenBalanceAttempt::fresh();
        for s in &samples {
            open = fed(&open, s, &c);
        }
        open
    }

    #[test]
    fn steady_stream_has_no_wobble_or_sway() {
        let samples = (1..=6)
            .map(|i| with_stability(sample(i, 1, 2.0), 0.0, [0.02, -0.01], [0.3, 0.1]))
            .collect();
        let acc = stability_run(samples).stability;
        assert!(acc.wobble_rad_s().expect("wobble").abs() < 1e-12);
        assert!(acc.tilt_sway_deg().expect("tilt").abs() < 1e-9);
        assert!(acc.com_sway_fraction(Some(0.8)).expect("com").abs() < 1e-9);
    }

    #[test]
    fn oscillating_stream_gives_expected_rms() {
        // Horizontal speed alternates 3 and 4 rad/s: RMS = sqrt((9 + 16) / 2).
        // Tilt x alternates +-0.01 rad (mean 0), y fixed: sway = 0.01 rad in degrees.
        // CoM x alternates 0.1 and 0.3 m (mean 0.2): deviation 0.1 m over reset 0.5 m = 0.2.
        let samples = (1..=8)
            .map(|i| {
                let odd = i % 2 == 1;
                with_stability(
                    sample(i, 1, 2.0),
                    if odd { 3.0 } else { 4.0 },
                    [if odd { 0.01 } else { -0.01 }, 0.05],
                    [if odd { 0.1 } else { 0.3 }, 0.0],
                )
            })
            .collect();
        let acc = stability_run(samples).stability;
        assert!((acc.wobble_rad_s().expect("wobble") - (12.5_f64).sqrt()).abs() < 1e-6);
        assert!((acc.tilt_sway_deg().expect("tilt") - 0.01_f64.to_degrees()).abs() < 1e-6);
        assert!((acc.com_sway_fraction(Some(0.5)).expect("com") - 0.2).abs() < 1e-6);
        assert_eq!(acc.com_sway_fraction(None), None);
        assert_eq!(acc.com_sway_fraction(Some(0.0)), None);
    }

    #[test]
    fn missing_ang_vel_leaves_wobble_absent_but_tilt_sway_present() {
        let samples = (1..=4)
            .map(|i| {
                let mut s =
                    with_stability(sample(i, 1, 2.0), 1.0, [0.01 * i as f64, 0.0], [0.0, 0.0]);
                if let Some(st) = s.stability.as_mut() {
                    st.horizontal_ang_vel_rad_s = None;
                }
                s
            })
            .collect();
        let acc = stability_run(samples).stability;
        assert_eq!(acc.wobble_rad_s(), None);
        assert!(acc.tilt_sway_deg().expect("tilt sway") > 0.0);
    }

    #[test]
    fn measure_without_motion_fields_deserializes_to_none() {
        let body: BodyMeasure = serde_json::from_str(r#"{"tilt_rad": 0.1}"#).expect("body");
        assert_eq!(body.ang_vel, None);
        assert_eq!(body.tilt_vec, None);
        assert_eq!(body.subtree_com, None);
        assert_eq!(body.com, None);
        let s = StabilitySample::from_body(&body);
        assert_eq!(s.horizontal_ang_vel_rad_s, None);
        assert_eq!(s.tilt_vec_rad, None);
        assert_eq!(s.com_xy_m, None);
    }

    #[test]
    fn settling_and_non_upright_bursts_are_excluded() {
        let mut c = criterion();
        c.hold_s = 1.0e6;
        let mut open = OpenBalanceAttempt::fresh();
        let mut settling = with_stability(sample(1, 1, 2.0), 99.0, [0.5, 0.5], [9.0, 9.0]);
        settling.settling = true;
        open = fed(&open, &settling, &c);
        assert_eq!(open.stability.wobble_rad_s(), None);
        let tilted = with_stability(
            sample(2, 1, c.max_tilt_deg + 10.0),
            99.0,
            [0.5, 0.5],
            [9.0, 9.0],
        );
        open = fed(&open, &tilted, &c);
        assert_eq!(open.stability.wobble_rad_s(), None);
        let good = with_stability(sample(3, 1, 2.0), 2.0, [0.0, 0.0], [0.0, 0.0]);
        open = fed(&open, &good, &c);
        assert!((open.stability.wobble_rad_s().expect("wobble") - 2.0).abs() < 1e-12);
    }

    #[test]
    fn finished_record_carries_stability_and_phase_mean() {
        let mut c = criterion();
        c.hold_s = 1.0e6;
        c.attempt_cap_s = c.burst_period_s * 3.0;
        let mut open = OpenBalanceAttempt::fresh();
        let mut record = None;
        for i in 1..=3 {
            let s = with_stability(sample(i, 1, 2.0), 2.0, [0.0, 0.0], [0.0, 0.0]);
            match observe_balance_sample(&open, &s, &c, 1).expect("observe") {
                BalanceAttemptStep::Continue(next) => open = next,
                BalanceAttemptStep::Finished(r) => record = Some(r),
            }
        }
        let record = record.expect("attempt reaches the cap");
        assert!((record.wobble_rad_s.expect("wobble") - 2.0).abs() < 1e-12);
        assert_eq!(record.com_sway_fraction, None);
        let none = self::record(BalanceAttemptOutcome::Success, 0.0, 1.0);
        let report = BalancePhaseReport::from_attempts(vec![record, none]).expect("report");
        assert_eq!(report.mean_wobble_rad_s, Some(2.0));
        assert_eq!(report.mean_com_sway_fraction, None);
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
        /// `ref_up_world` sent with each update that carried one.
        ref_up_updates: Vec<[f64; 3]>,
        /// The posture tilt the top body reports is the scripted tilt times this.
        posture_tilt_factor: f64,
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
        /// The torso CoM moves along +X at this speed, m/s, from the reset.
        torso_speed_x: f64,
        torso_yaw_rad: f64,
        /// The link the controller reports: `(kind, n)` with kind `reset`, `measure` or
        /// `status` and `n` the 0-based count of calls of that kind. A feed that returns `None`
        /// (or no feed) reports `linked` at epoch 1.
        link_feed: Option<LinkFeed>,
        /// Fails a call when it returns an error: `(kind, n)` as for `link_feed`.
        failures: Option<Failures>,
        /// Calls so far by kind (`reset`, `measure`, `status`).
        call_counts: BTreeMap<&'static str, u32>,
    }

    type LinkFeed = Box<dyn FnMut(&str, u32) -> Option<LinkReading>>;
    type Failures = Box<dyn FnMut(&str, u32) -> Option<TrainerError>>;

    /// Recovery limits for tests that never exercise recovery.
    const TEST_RECOVERY: RecoveryConfig = RecoveryConfig {
        max_outage_s: 2.0,
        max_discarded_attempts: 3,
        link_poll_s: 0.005,
        boundary_poll_s: 0.005,
        burst_stall_s: 5.0,
    };

    /// Burst clock for tests where FEAGI is not the subject: always running, and the counter
    /// rises on every read, so nothing ever stalls.
    struct RunningClock(std::sync::atomic::AtomicU64);

    impl BurstClock for RunningClock {
        fn burst_count(&self) -> Result<u64, TrainerError> {
            Ok(self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1)
        }

        fn engine_paused(&self) -> Result<bool, TrainerError> {
            Ok(false)
        }
    }

    static RUNNING_CLOCK: RunningClock = RunningClock(std::sync::atomic::AtomicU64::new(0));

    /// Burst clock with a scripted pause and a counter that freezes for a window of reads.
    struct ScriptedBurstClock {
        /// The first this many `engine_paused` reads say paused.
        paused_for: u32,
        /// `burst_count` reads (0-based) in this range return the previous value.
        frozen_reads: std::ops::Range<u32>,
        count_reads: std::cell::Cell<u32>,
        paused_reads: std::cell::Cell<u32>,
        value: std::cell::Cell<u64>,
    }

    impl ScriptedBurstClock {
        fn new(paused_for: u32, frozen_reads: std::ops::Range<u32>) -> Self {
            Self {
                paused_for,
                frozen_reads,
                count_reads: std::cell::Cell::new(0),
                paused_reads: std::cell::Cell::new(0),
                value: std::cell::Cell::new(0),
            }
        }
    }

    impl BurstClock for ScriptedBurstClock {
        fn burst_count(&self) -> Result<u64, TrainerError> {
            let read = self.count_reads.get();
            self.count_reads.set(read + 1);
            if !self.frozen_reads.contains(&read) {
                self.value.set(self.value.get() + 1);
            }
            Ok(self.value.get())
        }

        fn engine_paused(&self) -> Result<bool, TrainerError> {
            let read = self.paused_reads.get();
            self.paused_reads.set(read + 1);
            Ok(read < self.paused_for)
        }
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
                body("top", "base", 0.1, [1e-5; 3]),
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
                ref_up_updates: Vec::new(),
                posture_tilt_factor: 1.0,
                pulses: Vec::new(),
                measure_requests: Vec::new(),
                other_touching: Vec::new(),
                torso_z_drop: None,
                rejects_a_except: false,
                spring_level: None,
                k: 0,
                fresh: false,
                steps: 0,
                torso_speed_x: 0.0,
                torso_yaw_rad: 0.0,
                link_feed: None,
                failures: None,
                call_counts: BTreeMap::new(),
            }
        }

        /// Counts a call of `kind`, then returns the scripted failure for it, if any, and the
        /// link to report with it.
        fn tick(&mut self, kind: &'static str) -> (Option<TrainerError>, LinkReading) {
            let n = *self.call_counts.get(kind).unwrap_or(&0);
            self.call_counts.insert(kind, n + 1);
            let failure = self.failures.as_mut().and_then(|f| f(kind, n));
            let link = self
                .link_feed
                .as_mut()
                .and_then(|f| f(kind, n))
                .unwrap_or(LinkReading {
                    state: crate::balance_recovery::LinkState::Linked,
                    epoch: 1,
                });
            (failure, link)
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

        fn reset_episode(&mut self, keyframe: Option<&str>) -> Result<ResetReply, TrainerError> {
            self.calls.push("reset_episode".to_string());
            let (failure, link) = self.tick("reset");
            if let Some(error) = failure {
                return Err(error);
            }
            self.resets.push(keyframe.map(str::to_string));
            self.k = 0;
            self.fresh = true;
            self.spring_level = None;
            Ok(ResetReply {
                episode_id: self.resets.len() as u64,
                link,
            })
        }

        fn link_status(&mut self) -> Result<LinkReading, TrainerError> {
            self.calls.push("status".to_string());
            let (failure, link) = self.tick("status");
            match failure {
                Some(error) => Err(error),
                None => Ok(link),
            }
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
            ref_up_world: Option<[f64; 3]>,
        ) -> Result<(), TrainerError> {
            self.calls.push("update_body_spring".to_string());
            self.level_updates.push((level, self.seconds()));
            self.deadband_updates.push(deadband_rad);
            if let Some(up) = ref_up_world {
                self.ref_up_updates.push(up);
            }
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
            let (failure, link) = self.tick("measure");
            if let Some(error) = failure {
                return Err(error);
            }
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
                    // Lean along the first horizontal axis, consistent with tilt_rad.
                    tilt_vec: Some([scripted.tilt_deg.to_radians(), 0.0]),
                    ang_vel: Some([0.0; 3]),
                    com: Some([self.torso_speed_x * seconds, 0.0, torso_z]),
                    subtree_com: Some([0.0, 0.0, 0.150]),
                    heading: Some(BodyHeading {
                        yaw_rad: Some(self.torso_yaw_rad),
                        forward: Some([self.torso_yaw_rad.cos(), self.torso_yaw_rad.sin(), 0.0]),
                    }),
                },
            );
            // Contact bodies stand at the origin on the floor and the last requested body (the
            // posture top) stands over them, leaning along +Y like the torso's tilt vector: the
            // posture tilt is the scripted tilt times `posture_tilt_factor`.
            if let Some((top, feet)) = request.bodies[1..].split_last() {
                let bare = |com: [f64; 3]| BodyMeasure {
                    tilt_rad: 0.0,
                    tilt_vec: None,
                    ang_vel: None,
                    com: Some(com),
                    subtree_com: None,
                    heading: None,
                };
                for foot in feet {
                    bodies.insert(foot.clone(), bare([0.0, 0.0, 0.0]));
                }
                let lean = (scripted.tilt_deg * self.posture_tilt_factor).to_radians();
                bodies.insert(top.clone(), bare([0.0, lean.sin(), lean.cos()]));
            }
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
                actuator_force: if request.actuator_force {
                    BTreeMap::from([("joint_a".to_string(), f64::from(k) * 2.0)])
                } else {
                    BTreeMap::new()
                },
                link,
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
        let mut c =
            BalanceFeetCriterion::new("torso", contact_names(contacts), "top", 15.0, 1.0, 3.0);
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
            recovery: &TEST_RECOVERY,
            burst: &RUNNING_CLOCK,
            memory_counters: None,
            reset_latch: None,
        };
        run_balance_phase(
            &mut run,
            controller,
            &mut |_| Ok(()),
            &mut events,
            &CancelToken::new(),
        )
    }

    fn walking_criterion(distance_m: f64) -> BalanceFeetCriterion {
        let mut c = criterion_for(2, 0.05);
        c.walk = Some(WalkGoal {
            distance_m: Some(distance_m),
            ..WalkGoal::new(0.5)
        });
        c
    }

    fn walking_mock(speed_x: f64) -> MockController {
        let mut controller = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        controller.torso_speed_x = speed_x;
        controller
    }

    #[test]
    fn the_runner_scores_a_walk_from_the_measured_torso_position() {
        let c = walking_criterion(1.0);
        let mut controller = walking_mock(0.5);
        let report = run_phase(&c, &mut controller, "train", &[1]).expect("phase");
        let attempt = &report.attempts[0];
        assert_eq!(attempt.outcome, BalanceAttemptOutcome::Success);
        let walk = attempt.walk.expect("walk stats");
        assert!(
            walk.forward_m >= 1.0 && walk.mean_speed_m_s > 0.4,
            "{walk:?}"
        );
        assert_eq!(report.success_rate, 1.0);
    }

    #[test]
    fn the_walk_heading_is_measured_unless_set_and_a_missing_heading_is_an_error() {
        let measure = |heading: Option<BodyHeading>| BodyMeasure {
            tilt_rad: 0.0,
            tilt_vec: None,
            ang_vel: None,
            com: None,
            subtree_com: None,
            heading,
        };
        let at = |yaw_rad: Option<f64>| {
            measure(Some(BodyHeading {
                yaw_rad,
                forward: None,
            }))
        };
        let mut goal = WalkGoal::new(0.5);
        let measured = goal
            .resolve_heading_deg("torso", &at(Some(std::f64::consts::FRAC_PI_2)))
            .unwrap();
        assert!((measured - 90.0).abs() < 1e-9);
        // An explicit heading wins over the measured one.
        goal.forward_heading_deg = Some(30.0);
        assert_eq!(
            goal.resolve_heading_deg("torso", &at(Some(1.0))).unwrap(),
            30.0
        );
        // No heading in the reply, or an undefined yaw, is named, never read as 0.
        goal.forward_heading_deg = None;
        for reading in [measure(None), at(None)] {
            let error = goal.resolve_heading_deg("torso", &reading).unwrap_err();
            assert!(
                error.to_string().contains("walkForwardHeadingDeg"),
                "{error}"
            );
        }
    }

    #[test]
    fn a_walk_scores_along_the_measured_torso_heading_and_an_explicit_one_overrides_it() {
        // The mock torso moves along world +X. Facing +X it walks forward; facing +Y the same
        // motion is sideways and fails the heading rule; an explicit 0 degrees overrides that.
        let facing = |yaw_rad: f64, explicit: Option<f64>| {
            let mut c = walking_criterion(1.0);
            c.walk.as_mut().unwrap().forward_heading_deg = explicit;
            let mut controller = walking_mock(0.5);
            controller.torso_yaw_rad = yaw_rad;
            let report = run_phase(&c, &mut controller, "train", &[1]).expect("phase");
            report.attempts[0].outcome == BalanceAttemptOutcome::Success
        };
        assert!(facing(0.0, None));
        assert!(!facing(std::f64::consts::FRAC_PI_2, None));
        assert!(facing(std::f64::consts::FRAC_PI_2, Some(0.0)));
    }

    #[test]
    fn signed_lean_is_read_in_the_world_axes_and_in_the_torso_frame() {
        let lean = |tilt: Option<[f64; 2]>, yaw: Option<f64>| {
            StabilitySample {
                horizontal_ang_vel_rad_s: None,
                tilt_vec_rad: tilt,
                com_xy_m: None,
                yaw_rad: yaw,
            }
            .tilt_components_deg()
        };
        let theta = 0.1_f64;
        let deg = theta.to_degrees();
        // Leaning toward +X reads (0, -theta); a torso facing +X is leaning forward.
        let c = lean(Some([0.0, -theta]), Some(0.0));
        assert_eq!((c.x, c.y), (Some(0.0), Some(-deg)));
        assert!((c.forward.unwrap() - deg).abs() < 1e-9 && c.left.unwrap().abs() < 1e-9);
        // Leaning toward +Y reads (theta, 0); facing +X that is a lean to the left.
        let c = lean(Some([theta, 0.0]), Some(0.0));
        assert!((c.left.unwrap() - deg).abs() < 1e-9 && c.forward.unwrap().abs() < 1e-9);
        // The same lean toward +Y, with the torso turned to face +Y, is a lean forward.
        let c = lean(Some([theta, 0.0]), Some(std::f64::consts::FRAC_PI_2));
        assert!((c.forward.unwrap() - deg).abs() < 1e-9 && c.left.unwrap().abs() < 1e-9);
        // Facing -X, a lean toward +X is backward and one toward +Y is to the right.
        let c = lean(Some([0.0, -theta]), Some(std::f64::consts::PI));
        assert!((c.forward.unwrap() + deg).abs() < 1e-9);
        // No heading: only the axis components. No tilt vector: nothing.
        let c = lean(Some([theta, 0.0]), None);
        assert!(c.x.is_some() && c.forward.is_none() && c.left.is_none());
        assert_eq!(lean(None, Some(0.0)), TiltComponentsDeg::default());
    }

    #[test]
    fn the_steps_table_documents_the_signed_lean_after_the_angular_velocity() {
        let spec = steps_table_spec(&criterion(), &[], &[]);
        let names: Vec<&str> = spec.columns.iter().map(|c| c.name.as_str()).collect();
        let at = names.iter().position(|n| *n == "ang_vel_xy").unwrap();
        assert_eq!(
            &names[at + 1..at + 5],
            ["tilt_x", "tilt_y", "tilt_forward", "tilt_left"]
        );
    }

    #[test]
    fn the_speed_tolerance_bounds_both_sides_so_running_is_not_walking() {
        let goal = WalkGoal::new(0.5);
        assert!((goal.min_speed_m_s() - 0.375).abs() < 1e-12);
        assert!((goal.max_speed_m_s() - 0.625).abs() < 1e-12);
        // Same distance rule and criterion; only the mock's torso speed differs.
        for (speed, passes) in [(0.3, false), (0.5, true), (0.8, false)] {
            let c = walking_criterion(1.0);
            let mut controller = walking_mock(speed);
            let report = run_phase(&c, &mut controller, "train", &[1]).expect("phase");
            let success = report.attempts[0].outcome == BalanceAttemptOutcome::Success;
            assert_eq!(
                success, passes,
                "torso speed {speed}: {:?}",
                report.attempts[0]
            );
        }
    }

    #[test]
    fn the_runner_refuses_a_walk_that_cannot_judge_heading() {
        // The mock torso sits 0.1 m high: half a height is 0.05 m, above this 0.01 m distance.
        let c = walking_criterion(0.01);
        let mut controller = walking_mock(0.8);
        let err = run_phase(&c, &mut controller, "train", &[1])
            .unwrap_err()
            .to_string();
        assert!(err.contains("walk distance_m"), "{err}");
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
    fn d_rot_follows_the_tilt_inertia_not_the_smallest_inertia() {
        let mut info = generic_body_info();
        info.bodies[0].subtree_inertia = [5e-3, 4e-3, 1.0e-5];
        let inputs = sizing_inputs(&info, "torso", 0.140).expect("inputs");
        let g = size_spring_gains(&inputs);
        let expected = 0.8 * (g.k_rot * 5e-3_f64).sqrt();
        assert!(
            (g.d_rot - expected).abs() < 1e-12,
            "{} vs {expected}",
            g.d_rot
        );
    }

    #[test]
    fn limp_torso_inertia_does_not_cap_k_rot_or_d_rot() {
        let mut info = generic_body_info();
        info.bodies[1].inertia = [1e-5, 1e-5, 1e-5];
        let inputs = sizing_inputs(&info, "torso", 0.140).expect("inputs");
        let g = size_spring_gains(&inputs);
        let k_expected = 3.0 * 1.0 * 9.81 * 0.140 / 0.087;
        assert!((g.k_rot - k_expected).abs() < 1e-9, "k_rot {}", g.k_rot);
        // Far above the old explicit-force limit k <= 0.25 I_own / dt^2.
        assert!(g.k_rot > 0.25 * 1e-5 / (0.002 * 0.002));
        let d_expected = 0.8 * (k_expected * 5e-3_f64).sqrt();
        assert!((g.d_rot - d_expected).abs() < 1e-9, "d_rot {}", g.d_rot);
        // The inputs are recorded with the run data.
        let json = serde_json::to_value(inputs).expect("sizing inputs serialize");
        assert_eq!(json["i_own_min"], 1e-5);
        assert_eq!(json["dt_s"], 0.002);
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
            recovery: &TEST_RECOVERY,
            burst: &RUNNING_CLOCK,
            memory_counters: None,
            reset_latch: None,
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
        assert_eq!(
            request.bodies,
            vec![
                "torso".to_string(),
                "contact_a".to_string(),
                "contact_b".to_string(),
                "contact_c".to_string(),
                "top".to_string(),
            ],
            "the torso, every contact body (their CoM sets the support centre) and the top body"
        );
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
        // truncated: leaning past the limit (from the reset pose) for the whole cap (3.0 s = 60
        // bursts); the first sample is the reset pose
        let mut mock = MockController::new(
            0.05,
            2,
            Box::new(|k, _| Scripted {
                tilt_deg: if k <= 1 { 0.0 } else { 30.0 },
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
        // The run data carries the sizing inputs and, per attempt, the spring applied.
        let sizing = report.spring_sizing.expect("sizing inputs recorded");
        assert!((sizing.h0_m - 0.140).abs() < 1e-9);
        for (record, set) in report.attempts.iter().zip(&mock.spring_sets) {
            let applied = record.spring_applied.expect("spring recorded per attempt");
            assert_eq!(applied.level, set.1);
            assert_eq!(applied.deadband_rad, set.2);
        }
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
            recovery: &TEST_RECOVERY,
            burst: &RUNNING_CLOCK,
            memory_counters: None,
            reset_latch: None,
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
                memory_areas_frozen: 1,
                memory_formation_paused: true,
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
                recovery: &TEST_RECOVERY,
                burst: &RUNNING_CLOCK,
                memory_counters: None,
                reset_latch: None,
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
        assert!(test.memory_formation_paused);
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
            recovery: &TEST_RECOVERY,
            burst: &RUNNING_CLOCK,
            memory_counters: None,
            reset_latch: None,
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
                recovery: &TEST_RECOVERY,
                burst: &RUNNING_CLOCK,
                memory_counters: None,
                reset_latch: None,
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
                recovery: &TEST_RECOVERY,
                burst: &RUNNING_CLOCK,
                memory_counters: None,
                reset_latch: None,
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

    // ---- observe phase ----

    struct ScriptedMemory(Vec<Vec<crate::memory_counters::MemoryAreaCounters>>);

    impl MemoryCounterSource for ScriptedMemory {
        fn sample(
            &mut self,
        ) -> Result<Vec<crate::memory_counters::MemoryAreaCounters>, TrainerError> {
            Ok(self.0.remove(0))
        }
    }

    fn memory_at(
        created: u64,
        reactivated: u64,
    ) -> Vec<crate::memory_counters::MemoryAreaCounters> {
        vec![crate::memory_counters::MemoryAreaCounters {
            area: "mem".to_string(),
            created_total: Some(created),
            reactivated_total: Some(reactivated),
            creation_failed_total: Some(0),
            formation_suppressed_total: Some(0),
        }]
    }

    /// An observe phase over `seeds`, with a run-data sink and optional memory counters.
    fn run_observe_phase(
        c: &BalanceFeetCriterion,
        mock: &mut MockController,
        plasticity: &mut MockPlasticity,
        memory: Option<&mut dyn MemoryCounterSource>,
        data: &mut RecordingRunDataSink,
        seeds: &[u64],
    ) -> Result<BalancePhaseReport, TrainerError> {
        let mut events = CollectingEventSink::default();
        let run_id = RunId("balance-observe".to_string());
        let frozen = plasticity.frozen.clone();
        let mut seen_frozen = Vec::new();
        let result = {
            let mut run = BalancePhaseRun {
                phase: "observe",
                seeds,
                criterion: c,
                run_id: &run_id,
                support_mode: None,
                deliver_reward: None,
                plasticity: Some(plasticity),
                reward_policy: BalanceRewardPolicy::Affect,
                data: Some(data),
                apply_event: None,
                recovery: &TEST_RECOVERY,
                burst: &RUNNING_CLOCK,
                memory_counters: memory.map(|source| -> &mut dyn MemoryCounterSource { source }),
                reset_latch: None,
            };
            run_balance_phase(
                &mut run,
                mock,
                &mut |_| {
                    seen_frozen.push(frozen.get());
                    Ok(())
                },
                &mut events,
                &CancelToken::new(),
            )
        };
        assert!(
            seen_frozen.iter().all(|f| *f),
            "learning was live during an observe burst"
        );
        result
    }

    #[test]
    fn observe_phase_is_frozen_unassisted_unrewarded_and_writes_its_tables() {
        let c = brain_only_criterion();
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let mut plasticity = MockPlasticity::default();
        let mut memory = ScriptedMemory(vec![memory_at(4, 10), memory_at(4, 12), memory_at(5, 15)]);
        let mut data = RecordingRunDataSink::default();
        let report = run_observe_phase(
            &c,
            &mut mock,
            &mut plasticity,
            Some(&mut memory),
            &mut data,
            &[1, 2],
        )
        .expect("phase");
        assert_eq!(report.attempts.len(), 2, "N attempts, one per seed");
        // Test's brain-only path: zero spring at level 0, no pulses, no support, frozen.
        assert_eq!(mock.spring_sets.len(), 2);
        for (gains, level, deadband) in &mock.spring_sets {
            assert_eq!((*level, *deadband), (0.0, 0.0));
            assert_eq!(*gains, zero_gains());
        }
        assert!(mock.level_updates.is_empty() && mock.pulses.is_empty());
        assert!(report.attempts.iter().all(|a| a.support_level == 0.0));
        assert!(report.spring_gains.is_none(), "no support was sized");
        assert_eq!((plasticity.freezes, plasticity.restores), (1, 1));
        let conditions = report.brain_only.as_ref().expect("brain-only record");
        assert!(conditions.plasticity_frozen);
        assert_eq!(conditions.reward, "none");
        assert!(!conditions.perturbation);
        // No reward was delivered: the phase has no sink and refuses one.
        assert_eq!(data.errors, Vec::<String>::new());
        // The attempts table, the observe table and the memory table.
        assert_eq!(data.rows_of("attempts").len(), 2);
        assert_eq!(data.rows_of(OBSERVE_TABLE).len(), 2);
        let observe = &data.rows_of(OBSERVE_TABLE)[0];
        let col = |name: &str| data.column_index(OBSERVE_TABLE, name).expect(name);
        assert_eq!(observe[col("phase")], RunDataValue::text("observe"));
        assert_eq!(observe[col("allowed_angle")], RunDataValue::Float(15.0));
        // Tilt holds at its reset value (2 degrees from vertical): 0 of the 15 degree allowance.
        assert_eq!(observe[col("time_to_limit")], RunDataValue::Null);
        match observe[col("peak_tilt_fraction")] {
            RunDataValue::Float(v) => assert!(v.abs() < 1e-9, "{v}"),
            ref other => panic!("{other:?}"),
        }
        assert_eq!(observe[col("fall_direction")], RunDataValue::Null);
        let memory_rows = data.rows_of(crate::memory_counters::MEMORY_AREAS_TABLE);
        assert_eq!(memory_rows.len(), 2, "one row per area per attempt");
        let mcol = |name: &str| {
            data.column_index(crate::memory_counters::MEMORY_AREAS_TABLE, name)
                .expect(name)
        };
        assert_eq!(
            memory_rows[0][mcol("reactivated_delta")],
            RunDataValue::count(2)
        );
        assert_eq!(
            memory_rows[0][mcol("created_delta")],
            RunDataValue::count(0)
        );
        assert_eq!(
            memory_rows[0][mcol("rematch_rate")],
            RunDataValue::Float(1.0)
        );
        assert_eq!(
            memory_rows[1][mcol("created_delta")],
            RunDataValue::count(1)
        );
        match memory_rows[1][mcol("rematch_rate")] {
            RunDataValue::Float(v) => assert!((v - 0.75).abs() < 1e-9, "{v}"),
            ref other => panic!("{other:?}"),
        }
    }

    #[test]
    fn observe_measures_time_to_limit_as_a_fraction_of_the_allowed_angle() {
        let c = brain_only_criterion();
        // Upright for 0.15 s, then tipping past the 15 degree allowance.
        let mut mock = MockController::new(
            0.05,
            2,
            Box::new(|_, seconds| Scripted {
                tilt_deg: if seconds < 0.15 { 3.0 } else { 30.0 },
                down: true,
                terminal: None,
                sim_time_s: None,
            }),
        );
        let mut plasticity = MockPlasticity::default();
        let mut data = RecordingRunDataSink::default();
        let report = run_observe_phase(&c, &mut mock, &mut plasticity, None, &mut data, &[1])
            .expect("phase");
        let body = report.attempts[0].observe.expect("observe measures");
        assert_eq!(body.allowed_angle_deg, 15.0);
        let limit = body.time_to_limit_s.expect("passed the limit");
        assert!((0.1..=0.25).contains(&limit), "{limit}");
        assert!(
            // 3 degrees at reset, 30 after: 27 degrees from the reset pose over the 15 allowed
            (body.peak_tilt_fraction - 27.0 / 15.0).abs() < 1e-9,
            "{}",
            body.peak_tilt_fraction
        );
        assert!(
            data.rows_of(crate::memory_counters::MEMORY_AREAS_TABLE)
                .is_empty(),
            "no source, no memory table"
        );
        assert!(data
            .table(crate::memory_counters::MEMORY_AREAS_TABLE)
            .is_none());
    }

    #[test]
    fn observe_fall_direction_comes_from_the_torso_frame_lean() {
        let allowed = 15.0;
        let mut track = ObserveTrack::default();
        // yaw 0: forward = -ry, left = rx (see TiltComponentsDeg).
        let leaning = |forward_deg: f64, left_deg: f64, tilt_deg: f64| {
            let mut s = sample(1, 1, tilt_deg);
            s.stability = Some(StabilitySample {
                horizontal_ang_vel_rad_s: None,
                tilt_vec_rad: Some([left_deg.to_radians(), (-forward_deg).to_radians()]),
                com_xy_m: None,
                yaw_rad: Some(0.0),
            });
            s
        };
        track.add(&leaning(30.0, 0.0, 30.0), allowed, 1.0);
        let record = record(BalanceAttemptOutcome::Fell, 0.0, 0.0);
        let measures = track.measures(&record, allowed);
        assert!((measures.fall_forward_fraction.unwrap() - 2.0).abs() < 1e-9);
        assert!(measures.fall_left_fraction.unwrap().abs() < 1e-9);
        assert!(
            measures.fall_direction_deg.unwrap().abs() < 1e-9,
            "forward is 0 degrees"
        );
        track.add(&leaning(0.0, 30.0, 30.0), allowed, 1.0);
        let left = track.measures(&record, allowed);
        assert!((left.fall_direction_deg.unwrap() - 90.0).abs() < 1e-9);
        // A last sample inside the allowance reports no fall direction.
        track.add(&leaning(5.0, 0.0, 5.0), allowed, 1.0);
        assert_eq!(track.measures(&record, allowed).fall_direction_deg, None);
    }

    #[test]
    fn observe_refuses_to_run_when_learning_cannot_be_frozen() {
        let c = brain_only_criterion();
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let mut plasticity = MockPlasticity {
            fail_freeze: true,
            ..Default::default()
        };
        let mut data = RecordingRunDataSink::default();
        let error = run_observe_phase(&c, &mut mock, &mut plasticity, None, &mut data, &[1])
            .unwrap_err()
            .to_string();
        assert!(error.contains("needs learning frozen"), "{error}");
        assert_eq!(mock.count("reset_episode"), 0, "no attempt ran");
    }

    #[test]
    fn observe_refuses_events_rewards_and_support_like_test() {
        let mut c = brain_only_criterion();
        c.scheduled_events = vec![crate::run_actions::ScheduledEvent {
            at_s: 0.1,
            label: "x".to_string(),
            action: ScheduledAction::SetMappingPsc {
                src: "area_a".to_string(),
                dst: "area_b".to_string(),
                psc_multiplier: 0.5,
            },
        }];
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let mut plasticity = MockPlasticity::default();
        let mut data = RecordingRunDataSink::default();
        let error = run_observe_phase(&c, &mut mock, &mut plasticity, None, &mut data, &[1])
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("observe phase measures the brain only"),
            "{error}"
        );
        let c = brain_only_criterion();
        let mut events = CollectingEventSink::default();
        let run_id = RunId("balance-observe".to_string());
        let mut sink = |_: &[RewardSignal]| -> Result<(), TrainerError> { Ok(()) };
        let mut run = BalancePhaseRun {
            phase: "observe",
            seeds: &[1],
            criterion: &c,
            run_id: &run_id,
            support_mode: Some(SupportMode::Fixed(0.5)),
            deliver_reward: Some(&mut sink),
            plasticity: None,
            reward_policy: BalanceRewardPolicy::Affect,
            data: None,
            apply_event: None,
            recovery: &TEST_RECOVERY,
            burst: &RUNNING_CLOCK,
            memory_counters: None,
            reset_latch: None,
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
        assert!(error.contains("no reward sink"), "{error}");
    }

    #[test]
    fn support_need_and_effort_follow_the_documented_formulas() {
        // (tilt + 0.1 s * |omega_xy|) / max_tilt, clamped
        let need = support_need(5.0_f64.to_radians(), None, [0.1, 0.0, 9.0], 15.0, 0.1);
        let expected = (5.0_f64.to_radians() + 0.01) / 15.0_f64.to_radians();
        assert!((need - expected).abs() < 1e-12);
        assert_eq!(support_need(1.0, None, [0.0; 3], 15.0, 0.1), 1.0);
        // The need is the torso's: its inputs are the torso tilt, tilt vector and angular
        // velocity only, so a different posture cannot change it (contract: trunk geometry).
        let torso = 5.0_f64.to_radians();
        let along = support_need(torso, Some([torso, 0.0]), [0.0; 3], 15.0, 0.1);
        assert!((along - 5.0 / 15.0).abs() < 1e-12);
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
            update_body_spring_request("torso", 0.4, None, None, None)["level"],
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
            actuator_force: false,
        });
        assert!(m.get("actuators").is_none(), "{m}");
        assert!(m.get("actuator_force").is_none(), "{m}");
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
            "springs": {"torso": {"wrench_norm_torque": 0.4, "wrench_norm_force": 7.0}},
            "link": {"state": "linked", "epoch": 3}
        });
        let m: Measurement = serde_json::from_value(object).expect("object form");
        assert_eq!(m.link.epoch, 3);
        assert_eq!(m.terminal, Some(BalanceTerminal::Death));
        assert_eq!(m.bodies["torso"].tilt_rad, 0.1);
        assert_eq!(m.mean_contact_height(), Some(0.01));
        assert_eq!(m.springs["torso"].wrench_norm_force, 7.0);
        let array = serde_json::json!({
            "sim_time_s": 0.0, "physics_steps": 0,
            "bodies": [{"name": "torso", "tilt_rad": 0.0}],
            "contacts": [], "springs": [{"body": "torso", "wrench_norm_torque": 0.0, "wrench_norm_force": 0.0}],
            "link": {"state": "linked", "epoch": 0}
        });
        let m: Measurement = serde_json::from_value(array.clone()).expect("array form");
        // `link` is required: the same reply without it does not parse.
        let mut without = array;
        without.as_object_mut().expect("object").remove("link");
        assert!(serde_json::from_value::<Measurement>(without).is_err());
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
                        let payload = br#"{"ok":true,"result":{"episode_id":5,"link":{"state":"linked","epoch":1}}}"#;
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
            assert_eq!(http.reset_episode(None).expect("reset").episode_id, 5);
        }
        assert_eq!(http.connections_opened(), 1);
        assert_eq!(accepted.load(std::sync::atomic::Ordering::SeqCst), 1);
        // a server that closes after each reply still works: the client reconnects
        let (port, accepted) = serve(true);
        let mut http =
            HttpBalanceController::new(&format!("http://127.0.0.1:{port}"), Duration::from_secs(5))
                .expect("client");
        for _ in 0..3 {
            assert_eq!(
                http.reset_episode(Some("stand")).expect("reset").episode_id,
                5
            );
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

    // ---- controller link recovery ----

    /// What a scripted server does with its next connection.
    enum Canned {
        /// Reads the request and answers `(status, body)`, then closes.
        Reply(u16, &'static str),
        /// Reads the request and closes without sending a byte.
        CloseSilently,
        /// Sends a status line and headers promising a body, then closes.
        CutShort,
    }

    /// Local server that serves one connection per entry of `script`, in order.
    fn serve_canned(script: Vec<Canned>) -> u16 {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().expect("addr").port();
        std::thread::spawn(move || {
            for canned in script {
                let Ok((stream, _)) = listener.accept() else {
                    return;
                };
                let mut reader = BufReader::new(stream);
                let mut length = 0usize;
                loop {
                    let mut line = String::new();
                    if !matches!(reader.read_line(&mut line), Ok(n) if n > 0) {
                        return;
                    }
                    let lower = line.trim_end().to_ascii_lowercase();
                    if let Some(value) = lower.strip_prefix("content-length:") {
                        length = value.trim().parse().unwrap_or(0);
                    }
                    if lower.is_empty() {
                        break;
                    }
                }
                let mut body = vec![0u8; length];
                if reader.read_exact(&mut body).is_err() {
                    return;
                }
                let socket = reader.get_mut();
                match canned {
                    Canned::Reply(status, payload) => {
                        let head = format!(
                            "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            payload.len()
                        );
                        let _ = socket.write_all(head.as_bytes());
                        let _ = socket.write_all(payload.as_bytes());
                    }
                    Canned::CloseSilently => {}
                    Canned::CutShort => {
                        let _ = socket.write_all(
                            b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\nConnection: close\r\n\r\n{\"ok\"",
                        );
                    }
                }
            }
        });
        port
    }

    fn http_to(port: u16) -> HttpBalanceController {
        HttpBalanceController::new(&format!("http://127.0.0.1:{port}"), Duration::from_secs(5))
            .expect("client")
    }

    #[test]
    fn a_retryable_503_is_transient_and_every_other_failure_stays_runtime() {
        let port = serve_canned(vec![
            Canned::Reply(
                503,
                r#"{"ok":false,"retryable":true,"error":"physics loop busy"}"#,
            ),
            Canned::Reply(400, r#"{"ok":false,"error":"unknown action"}"#),
            Canned::Reply(503, r#"{"ok":false,"error":"down for maintenance"}"#),
            Canned::Reply(
                502,
                r#"{"ok":false,"retryable":false,"error":"unexpected"}"#,
            ),
            Canned::Reply(400, r#"{"error":"invalid JSON body"}"#),
            Canned::Reply(
                200,
                r#"{"ok":false,"retryable":true,"error":"busy in an envelope"}"#,
            ),
            Canned::Reply(200, r#"{"ok":false,"error":"keyframe missing"}"#),
        ]);
        let mut http = http_to(port);
        let busy = http.reset_episode(None).unwrap_err();
        assert!(
            matches!(&busy, TrainerError::Transient(m) if m.contains("503") && m.contains("physics loop busy")),
            "{busy}"
        );
        for expected in [
            "unknown action",
            "down for maintenance",
            "unexpected",
            "invalid JSON body",
        ] {
            let error = http.reset_episode(None).unwrap_err();
            assert!(
                matches!(&error, TrainerError::Runtime(m) if m.contains(expected)),
                "{expected}: {error}"
            );
        }
        let envelope = http.reset_episode(None).unwrap_err();
        assert!(matches!(envelope, TrainerError::Transient(_)), "{envelope}");
        let rejected = http.reset_episode(None).unwrap_err();
        assert!(matches!(rejected, TrainerError::Runtime(_)), "{rejected}");
    }

    #[test]
    fn a_refused_connection_is_transient() {
        // Bind, note the port, drop the listener: nothing listens there any more.
        let port = std::net::TcpListener::bind("127.0.0.1:0")
            .expect("bind")
            .local_addr()
            .expect("addr")
            .port();
        let mut http = http_to(port);
        let error = http.reset_episode(None).unwrap_err();
        assert!(
            matches!(&error, TrainerError::Transient(m) if m.contains("connect failed")),
            "{error}"
        );
        assert!(matches!(
            http.link_status(),
            Err(TrainerError::Transient(_))
        ));
    }

    #[test]
    fn a_connection_closed_before_any_response_byte_is_transient_a_cut_reply_is_not() {
        let port = serve_canned(vec![Canned::CloseSilently, Canned::CutShort]);
        let mut http = http_to(port);
        let silent = http.reset_episode(None).unwrap_err();
        assert!(
            matches!(&silent, TrainerError::Transient(m) if m.contains("before any response")),
            "{silent}"
        );
        // Bytes arrived, then the connection ended: the controller answered, so this is not a
        // restart signature and stays Runtime.
        let cut = http.reset_episode(None).unwrap_err();
        assert!(matches!(cut, TrainerError::Runtime(_)), "{cut}");
    }

    #[test]
    fn link_blocks_parse_from_reset_status_and_measure_replies() {
        let port = serve_canned(vec![
            Canned::Reply(
                200,
                r#"{"ok":true,"result":{"episode_id":4,"link":{"state":"linked","epoch":6}}}"#,
            ),
            Canned::Reply(
                200,
                r#"{"ok":true,"result":{"link":{"state":"reconnecting","epoch":7}}}"#,
            ),
            Canned::Reply(
                200,
                r#"{"ok":true,"result":{"link":{"state":"gone","epoch":1}}}"#,
            ),
            Canned::Reply(
                200,
                r#"{"ok":true,"result":{"sim_time_s":1.0,"physics_steps":5,"bodies":{"torso":{"tilt_rad":0.0}},"contacts":[],"link":{"state":"given_up","epoch":9}}}"#,
            ),
        ]);
        let mut http = http_to(port);
        let reset = http.reset_episode(None).expect("reset");
        assert_eq!(reset.episode_id, 4);
        assert_eq!(
            reset.link,
            LinkReading {
                state: crate::balance_recovery::LinkState::Linked,
                epoch: 6
            }
        );
        let status = http.link_status().expect("status");
        assert_eq!(
            (status.state, status.epoch),
            (crate::balance_recovery::LinkState::Reconnecting, 7)
        );
        let bad = http.link_status().unwrap_err();
        assert!(
            matches!(&bad, TrainerError::Runtime(m) if m.contains("`link` decode failed")),
            "{bad}"
        );
        let measured = http.measure(&any_measure_request()).expect("measure");
        assert_eq!(
            (measured.link.state, measured.link.epoch),
            (crate::balance_recovery::LinkState::GivenUp, 9)
        );
        assert_eq!(status_request()["action"], "status");
    }

    fn any_measure_request() -> MeasureRequest {
        MeasureRequest {
            bodies: vec!["torso".to_string()],
            contacts: Vec::new(),
            contacts_except: Vec::new(),
            actuators: false,
            actuator_force: false,
        }
    }

    #[test]
    fn a_reply_without_link_is_a_runtime_error_naming_the_field_and_the_action() {
        // Missing and null, for each of the three actions that must carry it.
        let port = serve_canned(vec![
            Canned::Reply(200, r#"{"ok":true,"result":{"episode_id":1}}"#),
            Canned::Reply(200, r#"{"ok":true,"result":{"episode_id":1,"link":null}}"#),
            Canned::Reply(200, r#"{"ok":true,"result":{}}"#),
            Canned::Reply(200, r#"{"ok":true,"result":{"link":null}}"#),
            Canned::Reply(
                200,
                r#"{"ok":true,"result":{"sim_time_s":1.0,"physics_steps":5,"bodies":{"torso":{"tilt_rad":0.0}},"contacts":[]}}"#,
            ),
            Canned::Reply(
                200,
                r#"{"ok":true,"result":{"sim_time_s":1.0,"physics_steps":5,"bodies":{"torso":{"tilt_rad":0.0}},"contacts":[],"link":null}}"#,
            ),
        ]);
        let mut http = http_to(port);
        let errors = [
            ("reset_episode", http.reset_episode(None).unwrap_err()),
            ("reset_episode", http.reset_episode(None).unwrap_err()),
            ("status", http.link_status().unwrap_err()),
            ("status", http.link_status().unwrap_err()),
            ("measure", http.measure(&any_measure_request()).unwrap_err()),
            ("measure", http.measure(&any_measure_request()).unwrap_err()),
        ];
        for (action, error) in errors {
            assert!(
                matches!(&error, TrainerError::Runtime(m)
                    if m.contains("`link`") && m.contains(action)),
                "{action}: {error}"
            );
        }
    }

    /// Criterion whose attempt ends after 10 upright bursts (hold 0.5 s at 0.05 s a burst).
    fn recovery_criterion() -> BalanceFeetCriterion {
        let mut c = criterion_for(2, 0.05);
        c.settle_s = 0.0;
        c.hold_s = 0.5;
        c.attempt_cap_s = 2.0;
        c.probe_every = 0;
        c
    }

    /// Limits with a stall window that clears the burst period of `recovery_criterion` (0.05 s)
    /// by the validator's multiple.
    fn recovery(max_outage_s: f64, max_discarded_attempts: u32) -> RecoveryConfig {
        RecoveryConfig::new(max_outage_s, max_discarded_attempts, 0.005, 0.005, 0.5)
            .expect("recovery")
    }

    fn link(state: crate::balance_recovery::LinkState, epoch: u64) -> Option<LinkReading> {
        Some(LinkReading { state, epoch })
    }

    fn linked_at(epoch: u64) -> Option<LinkReading> {
        link(crate::balance_recovery::LinkState::Linked, epoch)
    }

    /// One phase with a recovery budget and an inspectable data sink.
    #[allow(clippy::too_many_arguments)]
    fn run_recoverable<'a>(
        c: &'a BalanceFeetCriterion,
        mock: &mut MockController,
        seeds: &'a [u64],
        config: &'a RecoveryConfig,
        clock: &'a dyn BurstClock,
        cancel: &CancelToken,
        events: &mut dyn RunEventSink,
        data: Option<&'a mut (dyn RunDataSink + 'a)>,
        plasticity: Option<&'a mut (dyn PlasticityControl + 'a)>,
        present: &mut dyn FnMut([f64; 3]) -> Result<(), TrainerError>,
    ) -> Result<BalancePhaseReport, TrainerError> {
        let run_id = RunId("recovery-test".to_string());
        let mut run = BalancePhaseRun {
            phase: "train",
            seeds,
            criterion: c,
            run_id: &run_id,
            support_mode: None,
            deliver_reward: None,
            // `as` shortens the trait-object lifetime to the run's.
            plasticity: plasticity.map(|p| p as &mut dyn PlasticityControl),
            reward_policy: BalanceRewardPolicy::Affect,
            data: data.map(|d| d as &mut dyn RunDataSink),
            apply_event: None,
            recovery: config,
            burst: clock,
            memory_counters: None,
            reset_latch: None,
        };
        run_balance_phase(&mut run, mock, present, events, cancel)
    }

    /// `run_recoverable` with no plasticity, sink or presenter work.
    fn recover_run(
        c: &BalanceFeetCriterion,
        mock: &mut MockController,
        seeds: &[u64],
        config: &RecoveryConfig,
        tape: &mut RecordingRunDataSink,
    ) -> Result<BalancePhaseReport, TrainerError> {
        run_recoverable(
            c,
            mock,
            seeds,
            config,
            &RUNNING_CLOCK,
            &CancelToken::new(),
            &mut CollectingEventSink::default(),
            Some(tape),
            None,
            &mut |_| Ok(()),
        )
    }

    /// A mock whose link follows a shared value that the feed may change at chosen calls.
    fn mock_with_link(
        start: Option<LinkReading>,
        mut change: impl FnMut(&str, u32) -> Option<Option<LinkReading>> + 'static,
    ) -> MockController {
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let current = std::cell::Cell::new(start);
        mock.link_feed = Some(Box::new(move |kind, n| {
            if let Some(next) = change(kind, n) {
                current.set(next);
            }
            current.get()
        }));
        mock
    }

    #[test]
    fn an_epoch_change_mid_attempt_discards_it_and_reruns_the_same_seed() {
        let c = recovery_criterion();
        // The epoch moves on the 5th measure (inside the first attempt); the link is linked
        // before and after, so only the epoch gives it away.
        let mut mock = mock_with_link(linked_at(1), |kind, n| {
            (kind == "measure" && n == 4).then(|| linked_at(2))
        });
        let mut tape = RecordingRunDataSink::default();
        let report =
            recover_run(&c, &mut mock, &[10, 20], &recovery(2.0, 3), &mut tape).expect("phase");

        // Scored: exactly the two seeds, once each, in order; the discard is not among them.
        let seeds: Vec<u64> = report.attempts.iter().map(|a| a.seed).collect();
        assert_eq!(seeds, [10, 20]);
        assert_eq!(report.success_rate, 1.0);
        assert!(report.attempts.iter().all(|a| a.bursts == 10));
        assert_eq!(tape.rows_of(ATTEMPTS_TABLE).len(), 2);
        // The seed ran three times in total: discarded, re-run, then the next seed.
        assert_eq!(mock.resets.len(), 3);
        // Recorded with seed, reason and the epochs before and after.
        assert_eq!(report.discarded_attempts.len(), 1);
        let discard = &report.discarded_attempts[0];
        assert_eq!(
            (discard.seed, discard.attempt_index, discard.probe),
            (10, 0, false)
        );
        assert_eq!(
            discard.reason,
            crate::balance_recovery::DiscardReason::LinkEpochChanged
        );
        assert_eq!(
            (discard.link_epoch_before, discard.link_epoch_after),
            (Some(1), Some(2))
        );
        assert_eq!(
            tape.event_kinds()
                .iter()
                .filter(|k| **k == crate::balance_recovery::ATTEMPT_DISCARDED_EVENT)
                .count(),
            1
        );
    }

    #[test]
    fn an_outage_shorter_than_max_outage_recovers_and_the_phase_completes() {
        let c = recovery_criterion();
        let mut mock = mock_with_link(linked_at(1), |kind, n| match (kind, n) {
            ("measure", 4) => Some(link(crate::balance_recovery::LinkState::Reconnecting, 2)),
            ("status", 3) => Some(linked_at(3)),
            _ => None,
        });
        let mut tape = RecordingRunDataSink::default();
        let report =
            recover_run(&c, &mut mock, &[1, 2], &recovery(5.0, 3), &mut tape).expect("phase");
        // Three status reads saw it reconnecting, the fourth saw it linked.
        assert_eq!(mock.count("status"), 4);
        assert_eq!(report.attempts.len(), 2);
        assert_eq!(report.discarded_attempts.len(), 1);
        let discard = &report.discarded_attempts[0];
        assert_eq!(
            discard.reason,
            crate::balance_recovery::DiscardReason::LinkNotLinked
        );
        assert_eq!(
            discard.link_state_after,
            Some(crate::balance_recovery::LinkState::Reconnecting)
        );
        assert_eq!(
            (discard.link_epoch_before, discard.link_epoch_after),
            (Some(1), Some(2))
        );
    }

    #[test]
    fn an_outage_longer_than_max_outage_fails_the_phase_naming_the_limit() {
        let c = recovery_criterion();
        let mut mock = mock_with_link(linked_at(1), |kind, n| {
            (kind == "measure" && n == 4)
                .then(|| link(crate::balance_recovery::LinkState::Reconnecting, 2))
        });
        let mut tape = RecordingRunDataSink::default();
        let error = recover_run(&c, &mut mock, &[1, 2], &recovery(0.05, 3), &mut tape).unwrap_err();
        let TrainerError::Runtime(message) = &error else {
            panic!("expected Runtime, got {error}");
        };
        assert!(
            message.contains("max_outage_s (0.05)") && message.contains("reconnecting"),
            "{message}"
        );
        // The discard is in the run record even though the phase failed.
        assert!(tape
            .event_kinds()
            .contains(&crate::balance_recovery::ATTEMPT_DISCARDED_EVENT));
    }

    #[test]
    fn more_discards_than_max_discarded_attempts_fails_the_phase() {
        let c = recovery_criterion();
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        // Every attempt hits a busy controller on its 4th measure.
        mock.failures = Some(Box::new(|kind, n| {
            (kind == "measure" && n % 4 == 3)
                .then(|| TrainerError::Transient("HTTP 503 busy".to_string()))
        }));
        let mut tape = RecordingRunDataSink::default();
        let error = recover_run(&c, &mut mock, &[1, 2], &recovery(5.0, 2), &mut tape).unwrap_err();
        let message = error.to_string();
        assert!(
            matches!(error, TrainerError::Runtime(_))
                && message.contains("max_discarded_attempts (2)")
                && message.contains("3 discarded"),
            "{message}"
        );
        // The third discard failed the phase before another attempt started.
        assert_eq!(mock.resets.len(), 3);
    }

    #[test]
    fn a_given_up_link_fails_the_phase_at_once_without_waiting() {
        let c = recovery_criterion();
        let mut mock = mock_with_link(linked_at(1), |kind, n| {
            (kind == "measure" && n == 3)
                .then(|| link(crate::balance_recovery::LinkState::GivenUp, 2))
        });
        let mut tape = RecordingRunDataSink::default();
        let error = recover_run(&c, &mut mock, &[1, 2], &recovery(5.0, 3), &mut tape).unwrap_err();
        assert!(
            matches!(&error, TrainerError::Runtime(m) if m.contains("given_up")),
            "{error}"
        );
        assert_eq!(
            mock.count("status"),
            0,
            "no wait for a link that will not return"
        );
        assert_eq!(mock.resets.len(), 1);
        assert!(tape
            .event_kinds()
            .contains(&crate::balance_recovery::ATTEMPT_DISCARDED_EVENT));
    }

    #[test]
    fn a_transient_error_mid_attempt_discards_it_and_a_runtime_error_fails_the_phase() {
        let c = recovery_criterion();
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        mock.failures = Some(Box::new(|kind, n| {
            (kind == "measure" && n == 4)
                .then(|| TrainerError::Transient("HTTP 503 busy".to_string()))
        }));
        let mut tape = RecordingRunDataSink::default();
        let report =
            recover_run(&c, &mut mock, &[1, 2], &recovery(5.0, 3), &mut tape).expect("phase");
        assert_eq!(report.attempts.len(), 2);
        let discard = &report.discarded_attempts[0];
        assert_eq!(
            discard.reason,
            crate::balance_recovery::DiscardReason::TransientError
        );
        assert!(discard.detail.contains("503"), "{}", discard.detail);
        assert_eq!(
            (discard.link_epoch_before, discard.link_epoch_after),
            (Some(1), Some(1)),
            "the busy reply carried no link, so the epochs are those of the last good reply"
        );

        // A rejected command is not transient: the phase fails, nothing is retried.
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        mock.failures = Some(Box::new(|kind, n| {
            (kind == "measure" && n == 2)
                .then(|| TrainerError::Runtime("command rejected".to_string()))
        }));
        let error = recover_run(
            &c,
            &mut mock,
            &[1, 2],
            &recovery(5.0, 3),
            &mut RecordingRunDataSink::default(),
        )
        .unwrap_err();
        assert!(matches!(&error, TrainerError::Runtime(m) if m == "command rejected"));
        assert_eq!((mock.resets.len(), mock.count("status")), (1, 0));
    }

    #[test]
    fn a_refused_reset_is_transient_and_the_seed_is_rerun() {
        let c = recovery_criterion();
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        // The controller restarts while the second seed begins.
        mock.failures = Some(Box::new(|kind, n| {
            (kind == "reset" && n == 1)
                .then(|| TrainerError::Transient("connection refused".to_string()))
        }));
        let mut tape = RecordingRunDataSink::default();
        let report =
            recover_run(&c, &mut mock, &[1, 2], &recovery(5.0, 3), &mut tape).expect("phase");
        let seeds: Vec<u64> = report.attempts.iter().map(|a| a.seed).collect();
        assert_eq!(seeds, [1, 2]);
        assert_eq!(report.discarded_attempts.len(), 1);
        assert_eq!(
            (
                report.discarded_attempts[0].seed,
                report.discarded_attempts[0].attempt_index
            ),
            (2, 1)
        );
        assert_eq!(
            mock.resets.len(),
            2,
            "two resets succeeded; the refused one never started"
        );
    }

    #[test]
    fn a_discarded_probe_restores_learning_before_the_wait_and_reruns_frozen() {
        let mut c = recovery_criterion();
        c.probe_every = 3;
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        // Resets 0..=2 are the three train attempts; reset 3 is the first probe.
        mock.failures = Some(Box::new(|kind, n| {
            (kind == "reset" && n == 3)
                .then(|| TrainerError::Transient("connection reset".to_string()))
        }));
        let mut plasticity = MockPlasticity::default();
        let frozen = plasticity.frozen.clone();
        let mut tape = RecordingRunDataSink::default();
        let config = recovery(5.0, 3);
        let report = run_recoverable(
            &c,
            &mut mock,
            &[1, 2, 3],
            &config,
            &RUNNING_CLOCK,
            &CancelToken::new(),
            &mut CollectingEventSink::default(),
            Some(&mut tape),
            Some(&mut plasticity),
            &mut |_| Ok(()),
        )
        .expect("phase");
        assert_eq!(report.attempts.len(), 4);
        assert_eq!(report.probe_attempts, 1);
        assert_eq!(report.discarded_attempts.len(), 1);
        assert!(report.discarded_attempts[0].probe);
        assert_eq!(report.discarded_attempts[0].attempt_index, 3);
        // Frozen for the failed probe and again for its re-run, restored after each.
        assert_eq!((plasticity.freezes, plasticity.restores), (2, 2));
        assert!(!frozen.get());
        assert_eq!(report.probe_plasticity_frozen, Some(true));
    }

    #[test]
    fn a_discarded_adaptive_attempt_leaves_no_trace_in_the_assist_state() {
        let c = quick_adaptive();
        let seeds = [1u64, 2];
        let config = recovery(5.0, 3);
        let mut reference_mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let reference = recover_run(
            &c,
            &mut reference_mock,
            &seeds,
            &config,
            &mut RecordingRunDataSink::default(),
        )
        .expect("reference");

        // The same phase, with the first attempt cut off 30 bursts in (assist already faded).
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        mock.failures = Some(Box::new(|kind, n| {
            (kind == "measure" && n == 30)
                .then(|| TrainerError::Transient("HTTP 503 busy".to_string()))
        }));
        let report = recover_run(
            &c,
            &mut mock,
            &seeds,
            &config,
            &mut RecordingRunDataSink::default(),
        )
        .expect("with a discard");
        assert_eq!(report.discarded_attempts.len(), 1);
        assert_eq!(report.attempts, reference.attempts);
        assert_eq!(report.adaptive_state_next, reference.adaptive_state_next);
        assert_eq!(report.support_level_next, reference.support_level_next);
        assert_eq!(report.assist_timeline, reference.assist_timeline);
        assert_eq!(report.deviation_assist, reference.deviation_assist);
    }

    /// Sends the number of resets seen so far each time an attempt finishes.
    struct BoundarySink {
        finished: std::sync::mpsc::Sender<u32>,
        resets_seen: std::sync::Arc<std::sync::atomic::AtomicU32>,
    }

    impl RunEventSink for BoundarySink {
        fn emit(&mut self, event: RunEvent) {
            if matches!(event.kind, RunEventKind::Progress { .. }) {
                let seen = self.resets_seen.load(std::sync::atomic::Ordering::SeqCst);
                let _ = self.finished.send(seen);
            }
        }
    }

    #[test]
    fn a_pause_parks_at_the_attempt_boundary_never_mid_attempt_and_resumes() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::{mpsc, Arc};
        let c = recovery_criterion();
        let resets_seen = Arc::new(AtomicU32::new(0));
        let counter = resets_seen.clone();
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        mock.link_feed = Some(Box::new(move |kind, _| {
            if kind == "reset" {
                counter.fetch_add(1, Ordering::SeqCst);
            }
            None
        }));
        let cancel = CancelToken::new();
        let (tx, rx) = mpsc::channel();
        let resumer = cancel.clone();
        let watcher_seen = resets_seen.clone();
        let watcher = std::thread::spawn(move || {
            // The first attempt finished: the phase now reaches its boundary and must park.
            let at_boundary = rx.recv_timeout(Duration::from_secs(20)).ok();
            // While paused, the second attempt must not start.
            std::thread::sleep(Duration::from_millis(150));
            let while_paused = watcher_seen.load(Ordering::SeqCst);
            resumer.resume();
            (at_boundary, while_paused)
        });
        let mut sink = BoundarySink {
            finished: tx,
            resets_seen: resets_seen.clone(),
        };
        let pauser = cancel.clone();
        let mut first_burst = true;
        let config = recovery(5.0, 3);
        let report = run_recoverable(
            &c,
            &mut mock,
            &[1, 2],
            &config,
            &RUNNING_CLOCK,
            &cancel,
            &mut sink,
            None,
            None,
            // The pause arrives in the middle of the first attempt.
            &mut |_| {
                if first_burst {
                    first_burst = false;
                    pauser.pause();
                }
                Ok(())
            },
        )
        .expect("phase");
        let (at_boundary, while_paused) = watcher.join().expect("watcher");
        // Mid-attempt the pause parked nothing: the first attempt ran to its end (10 bursts).
        assert_eq!(
            at_boundary,
            Some(1),
            "the first attempt finished before any park"
        );
        assert_eq!(report.attempts[0].bursts, 10);
        // At the boundary it parked: no second reset while paused; after resume it finished.
        assert_eq!(while_paused, 1);
        assert_eq!(report.attempts.len(), 2);
        assert_eq!(resets_seen.load(Ordering::SeqCst), 2);
        assert!(!cancel.is_paused());
    }

    #[test]
    fn a_skip_mid_attempt_finishes_that_attempt_and_ends_the_phase_skipped() {
        let c = recovery_criterion();
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let cancel = CancelToken::new();
        let skipper = cancel.clone();
        let mut first_burst = true;
        let config = recovery(5.0, 3);
        let report = run_recoverable(
            &c,
            &mut mock,
            &[1, 2, 3],
            &config,
            &RUNNING_CLOCK,
            &cancel,
            &mut CollectingEventSink::default(),
            None,
            None,
            // The skip arrives in the middle of the first attempt.
            &mut |_| {
                if first_burst {
                    first_burst = false;
                    skipper.skip();
                }
                Ok(())
            },
        )
        .expect("phase");
        assert_eq!(report.attempts.len(), 1, "no attempt starts after the skip");
        assert_eq!(
            report.attempts[0].bursts, 10,
            "the attempt in progress ran to its end"
        );
        assert!(report.skipped);
        assert!(!cancel.is_cancelled());
    }

    #[test]
    fn a_phase_with_a_fresh_token_after_a_skipped_one_runs_every_attempt() {
        let c = recovery_criterion();
        let config = recovery(5.0, 3);
        let skipped_token = CancelToken::new();
        skipped_token.skip();
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let first = run_recoverable(
            &c,
            &mut mock,
            &[1, 2, 3],
            &config,
            &RUNNING_CLOCK,
            &skipped_token,
            &mut CollectingEventSink::default(),
            None,
            None,
            &mut |_| Ok(()),
        )
        .expect("skipped phase");
        // A skip asked before any attempt finished still lets one attempt run and be reported.
        assert_eq!(first.attempts.len(), 1);
        assert!(first.skipped);
        let fresh = CancelToken::new();
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let next = run_recoverable(
            &c,
            &mut mock,
            &[1, 2, 3],
            &config,
            &RUNNING_CLOCK,
            &fresh,
            &mut CollectingEventSink::default(),
            None,
            None,
            &mut |_| Ok(()),
        )
        .expect("next phase");
        assert_eq!(next.attempts.len(), 3);
        assert!(!next.skipped);
    }

    #[test]
    fn a_burst_stall_mid_attempt_discards_it_and_reruns_the_same_seed() {
        // A 5 ms burst: the burst presenter below takes one burst period per burst, as a live
        // run's wall-clock step does.
        let mut c = criterion_for(2, 0.005);
        c.settle_s = 0.0;
        c.hold_s = 0.05;
        c.attempt_cap_s = 0.5;
        c.probe_every = 0;
        let config = RecoveryConfig::new(5.0, 3, 0.005, 0.005, 0.02).expect("recovery");
        // The counter freezes for reads 3..=8: inside the first attempt, and long enough to
        // trip the 20 ms window. It runs again before the re-run could stall in turn.
        let clock = ScriptedBurstClock::new(0, 3..9);
        let mut mock = MockController::new(0.005, 2, Box::new(|_, _| upright_step()));
        let mut tape = RecordingRunDataSink::default();
        let report = run_recoverable(
            &c,
            &mut mock,
            &[1, 2],
            &config,
            &clock,
            &CancelToken::new(),
            &mut CollectingEventSink::default(),
            Some(&mut tape),
            None,
            &mut |_| {
                std::thread::sleep(Duration::from_millis(5));
                Ok(())
            },
        )
        .expect("phase");
        let seeds: Vec<u64> = report.attempts.iter().map(|a| a.seed).collect();
        assert_eq!(seeds, [1, 2]);
        assert_eq!(report.success_rate, 1.0);
        assert_eq!(report.discarded_attempts.len(), 1);
        let discard = &report.discarded_attempts[0];
        assert_eq!(
            discard.reason,
            crate::balance_recovery::DiscardReason::BurstStall
        );
        assert_eq!((discard.seed, discard.attempt_index), (1, 0));
        assert!(discard.detail.contains("stuck at 3"), "{}", discard.detail);
        // Seed 1 ran twice, seed 2 once; the link was healthy throughout.
        assert_eq!(mock.resets.len(), 3);
        assert_eq!(discard.link_epoch_before, discard.link_epoch_after);
        assert_eq!(tape.rows_of(ATTEMPTS_TABLE).len(), 2);
    }

    #[test]
    fn a_paused_engine_parks_the_phase_at_the_boundary_and_resumes() {
        let c = recovery_criterion();
        // The engine reads paused four times, then runs.
        let clock = ScriptedBurstClock::new(4, 0..0);
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let config = recovery(5.0, 3);
        let report = run_recoverable(
            &c,
            &mut mock,
            &[1, 2],
            &config,
            &clock,
            &CancelToken::new(),
            &mut CollectingEventSink::default(),
            None,
            None,
            &mut |_| Ok(()),
        )
        .expect("phase");
        assert_eq!(report.attempts.len(), 2);
        assert!(report.discarded_attempts.is_empty());
        // 4 paused reads and 1 running read at the first boundary, 1 at the second; none inside
        // an attempt.
        assert_eq!(clock.paused_reads.get(), 6);
        assert_eq!(mock.resets.len(), 2);
    }

    #[test]
    fn a_paused_engine_beyond_max_outage_fails_the_phase_naming_the_reason() {
        let c = recovery_criterion();
        let clock = ScriptedBurstClock::new(u32::MAX, 0..0);
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let config = recovery(0.05, 3);
        let error = run_recoverable(
            &c,
            &mut mock,
            &[1, 2],
            &config,
            &clock,
            &CancelToken::new(),
            &mut CollectingEventSink::default(),
            None,
            None,
            &mut |_| Ok(()),
        )
        .unwrap_err();
        let TrainerError::Runtime(message) = &error else {
            panic!("expected Runtime, got {error}");
        };
        assert!(
            message.contains("burst engine paused") && message.contains("max_outage_s (0.05)"),
            "{message}"
        );
        assert!(message.contains("burst count"), "{message}");
        assert!(mock.resets.is_empty(), "no attempt started");
    }

    #[test]
    fn a_burst_stall_window_below_a_few_burst_periods_fails_before_any_call() {
        let c = recovery_criterion(); // 0.05 s bursts: the window must be at least 0.15 s
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let config = RecoveryConfig::new(5.0, 3, 0.005, 0.005, 0.1).expect("valid on its own");
        let error = recover_run(
            &c,
            &mut mock,
            &[1],
            &config,
            &mut RecordingRunDataSink::default(),
        )
        .unwrap_err();
        assert!(
            matches!(&error, TrainerError::Config(m) if m.contains("burst_period_s")),
            "{error}"
        );
        assert!(mock.calls.is_empty());
    }

    #[test]
    fn an_invalid_recovery_config_fails_before_any_call() {
        let c = recovery_criterion();
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let bad = RecoveryConfig {
            max_outage_s: 0.0,
            max_discarded_attempts: 3,
            link_poll_s: 0.1,
            boundary_poll_s: 0.1,
            burst_stall_s: 1.0,
        };
        let error = recover_run(
            &c,
            &mut mock,
            &[1],
            &bad,
            &mut RecordingRunDataSink::default(),
        )
        .unwrap_err();
        assert!(matches!(&error, TrainerError::Config(m) if m.contains("max_outage_s")));
        assert!(mock.calls.is_empty());
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
                recovery: &TEST_RECOVERY,
                burst: &RUNNING_CLOCK,
                memory_counters: None,
                reset_latch: None,
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
            actuator_force: BTreeMap::new(),
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
        // the posture does not move the reference either
        let mut leaning = standing_sample(1, 0.30, None);
        leaning.posture = Some(PostureSample {
            tilt_deg: 40.0,
            relative_tilt_deg: None,
            reference: None,
            forward_deg: None,
            left_deg: None,
        });
        let open = fed(&OpenBalanceAttempt::fresh(), &leaning, &c);
        assert_eq!(open.reset_height_m, Some(0.30));
    }

    #[test]
    fn reset_height_and_sizing_ignore_the_posture() {
        // The reference height is the torso's, whatever the posture says; a posture past the
        // limit counts as a tilt failure and leaves the height rule alone.
        let c = fall_criterion();
        let mut first = standing_sample(1, 0.10, None);
        first.posture = Some(PostureSample {
            tilt_deg: 40.0,
            relative_tilt_deg: None,
            reference: None,
            forward_deg: None,
            left_deg: None,
        });
        assert_eq!(first.tilt_deg, 2.0);
        let open = fed(&OpenBalanceAttempt::fresh(), &first, &c);
        assert_eq!(open.reset_height_m, Some(0.10));
        assert_eq!(
            open.non_upright.tilt, 1,
            "scored on the posture, not the torso"
        );
        assert_eq!(open.non_upright.height, 0);
        // Sizing reads the torso geometry from the model and the first reading; the posture
        // criterion fields take no part in it.
        let sizing = sizing_inputs(&generic_body_info(), "torso", 0.140).unwrap();
        assert!((sizing.h0_m - 0.140).abs() < 1e-12);
    }

    #[test]
    fn posture_tilt_is_read_from_three_known_positions() {
        let centre = [0.0, 0.0, 0.0];
        let straight = posture_reading(centre, [0.0, 0.0, 1.0]).expect("line");
        assert!(straight.tilt_rad.abs() < 1e-12);
        assert_eq!(straight.tilt_vec_rad, [0.0, 0.0]);
        // Over +X by one height: 45 degrees; the rotation back onto up is (0, -theta).
        let toward_x = posture_reading(centre, [1.0, 0.0, 1.0]).expect("line");
        assert!((toward_x.tilt_rad.to_degrees() - 45.0).abs() < 1e-9);
        assert!(toward_x.tilt_vec_rad[0].abs() < 1e-12);
        assert!((toward_x.tilt_vec_rad[1] + std::f64::consts::FRAC_PI_4).abs() < 1e-12);
        // Over +Y, from a raised centre: 30 degrees, rotation (theta, 0).
        let toward_y =
            posture_reading([1.0, 2.0, 0.5], [1.0, 3.0, 0.5 + 3.0_f64.sqrt()]).expect("line");
        assert!((toward_y.tilt_rad.to_degrees() - 30.0).abs() < 1e-9);
        assert!((toward_y.tilt_vec_rad[0] - std::f64::consts::FRAC_PI_6).abs() < 1e-12);
        assert!(toward_y.tilt_vec_rad[1].abs() < 1e-12);
        // Torso frame: leaning toward +X is forward for yaw 0 and to the right for yaw 90 deg.
        let forward = toward_x.sample(Some(0.0));
        assert!((forward.forward_deg.unwrap() - 45.0).abs() < 1e-9);
        assert!(forward.left_deg.unwrap().abs() < 1e-9);
        let turned = toward_x.sample(Some(std::f64::consts::FRAC_PI_2));
        assert!(turned.forward_deg.unwrap().abs() < 1e-9);
        assert!((turned.left_deg.unwrap() + 45.0).abs() < 1e-9);
        assert_eq!(toward_x.sample(None).forward_deg, None);
        assert!((forward.tilt_deg - 45.0).abs() < 1e-9);
        // A top body on the centre has no line.
        assert!(posture_reading(centre, centre).is_none());
    }

    #[test]
    fn support_centre_is_the_touching_contacts_or_all_when_none_touch() {
        let feet = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [8.0, 0.0, 0.0]];
        assert_eq!(
            support_centre(&feet, &[true, true, false]),
            Some([1.0, 0.0, 0.0])
        );
        assert_eq!(
            support_centre(&feet, &[false, false, true]),
            Some([8.0, 0.0, 0.0])
        );
        let all = support_centre(&feet, &[false, false, false]).expect("fallback");
        assert!((all[0] - 10.0 / 3.0).abs() < 1e-12);
        assert_eq!(support_centre(&[], &[]), None);
        // Through a measure reply: one foot up, so the centre is the other foot, and the posture
        // line leans by the scripted angle from that foot.
        let c = criterion_for(2, 0.05);
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let mut reading = mock.measure(&measure_request(&c)).expect("measure");
        reading.bodies.get_mut("contact_a").unwrap().com = Some([0.0, 0.0, 0.0]);
        reading.bodies.get_mut("contact_b").unwrap().com = Some([4.0, 0.0, 0.0]);
        reading.bodies.get_mut("top").unwrap().com = Some([4.0, 0.0, 1.0]);
        reading.contacts[0].touching = false;
        let touching_only = measure_posture(&reading, &c).expect("posture");
        assert!(
            touching_only.tilt_rad.abs() < 1e-12,
            "over the foot that touches"
        );
        reading.contacts[0].touching = true;
        reading.contacts[1].touching = true;
        let both = measure_posture(&reading, &c).expect("posture");
        assert!((both.tilt_rad.to_degrees() - 63.434_948_8).abs() < 1e-6);
        reading.contacts[0].touching = false;
        reading.contacts[1].touching = false;
        let none = measure_posture(&reading, &c).expect("posture");
        assert!(
            (none.tilt_rad - both.tilt_rad).abs() < 1e-12,
            "all contacts when none touch"
        );
        reading.bodies.get_mut("top").unwrap().com = None;
        let missing = measure_posture(&reading, &c).unwrap_err().to_string();
        assert!(
            missing.contains("top") && missing.contains("com"),
            "{missing}"
        );
    }

    #[test]
    fn posture_definition_change_clears_the_saved_loss_history() {
        let c = quick_adaptive();
        let mut state = AdaptiveAssistState::new(0.4, 0.6);
        state.loss_tilts_deg = vec![10.0, 12.0];
        // A state saved before the definition existed is dropped: its tilts were the torso's.
        let old = AdaptiveAssist::resume(&c, state.clone());
        assert!(old.state().loss_tilts_deg.is_empty());
        assert_eq!(old.state().posture_definition, Some(c.posture_definition()));
        state.posture_definition = Some(c.posture_definition());
        let same = AdaptiveAssist::resume(&c, state.clone());
        assert_eq!(same.state().loss_tilts_deg, vec![10.0, 12.0]);
        // A different top body, or a different set of contact bodies, clears it; the level and
        // slow average carry over.
        let mut other_top = c.clone();
        other_top.posture_top_body = "head".to_string();
        let resumed = AdaptiveAssist::resume(&other_top, state.clone()).state();
        assert!(resumed.loss_tilts_deg.is_empty());
        assert_eq!((resumed.a, resumed.a_slow), (0.4, 0.6));
        assert_eq!(
            resumed.posture_definition,
            Some(other_top.posture_definition())
        );
        let mut fewer = c.clone();
        fewer.contact_bodies.pop();
        assert!(AdaptiveAssist::resume(&fewer, state.clone())
            .state()
            .loss_tilts_deg
            .is_empty());
        assert_ne!(c.posture_definition(), other_top.posture_definition());
        // Contact order does not change the definition.
        let mut swapped = c.clone();
        swapped.contact_bodies.reverse();
        assert_eq!(c.posture_definition(), swapped.posture_definition());
    }

    #[test]
    fn posture_tilt_scores_while_need_and_the_report_stay_on_the_torso() {
        // Torso 2 deg, posture factor 1: upright and a success.
        let mut c = criterion_for(2, 0.05);
        c.settle_s = 0.0;
        c.hold_s = 0.5;
        c.attempt_cap_s = 1.0;
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let report = run_phase(&c, &mut mock, "train", &[1]).expect("phase");
        assert_eq!(report.attempts[0].outcome, BalanceAttemptOutcome::Success);
        // Posture 10x the torso: the torso goes from 2 to 4 deg after the reset sample, the
        // posture line from 20 to 40 deg, so it is 20 deg from its reset pose (> 15): never
        // upright, though the torso is only 2 deg from its own.
        let lean_after_reset = || {
            Box::new(|k, _| Scripted {
                tilt_deg: if k <= 1 { 2.0 } else { 4.0 },
                ..upright_step()
            })
        };
        let mut mock = MockController::new(0.05, 2, lean_after_reset());
        mock.posture_tilt_factor = 10.0;
        let mut tape = RecordingRunDataSink::default();
        let report = run_with_hooks(&c, &mut mock, "train", &[1], &rid(), Some(&mut tape), None)
            .expect("phase");
        assert_ne!(report.attempts[0].outcome, BalanceAttemptOutcome::Success);
        assert!((report.attempts[0].final_tilt_deg - 20.0).abs() < 1e-6);
        let col = |name: &str| tape.column_index("steps", name).expect(name);
        let float = |row: &Vec<RunDataValue>, i: usize| match row[i] {
            RunDataValue::Float(v) => v,
            ref other => panic!("{other:?}"),
        };
        let (tilt, posture, need) = (col("tilt"), col("posture_tilt"), col("support_need"));
        let scored = col("scored_tilt");
        assert!(col("posture_forward") > posture && col("posture_left") > posture);
        let rows = tape.rows_of("steps");
        assert!(rows.len() > 1);
        // The reset sample reads 0 on both the scored tilt and the need.
        assert!(float(&rows[0], scored).abs() < 1e-6 && float(&rows[0], need).abs() < 1e-6);
        for row in &rows[1..] {
            // The report stays absolute.
            assert!((float(row, tilt) - 4.0).abs() < 1e-6);
            assert!((float(row, posture) - 40.0).abs() < 1e-6);
            assert!((float(row, scored) - 20.0).abs() < 1e-6);
            // support_need follows the trunk from its reset orientation: 2 deg over 15.
            assert!(
                (float(row, need) - 2.0 / 15.0).abs() < 1e-6,
                "{}",
                float(row, need)
            );
        }
        // The posture is scored only: the Trainer never re-targets the spring, in train or test.
        assert!(mock.ref_up_updates.is_empty());
        let mut mock = MockController::new(0.05, 2, lean_after_reset());
        mock.posture_tilt_factor = 10.0;
        run_phase(&c, &mut mock, "train", &[1]).expect("train phase");
        assert!(mock.ref_up_updates.is_empty());
    }

    /// A sample whose posture line stands `lean_deg` off vertical toward `heading` (rad, in the
    /// world XY plane), read with the torso heading `yaw`.
    fn leaning_sample(seq: u64, lean_deg: f64, heading: f64, yaw: f64) -> BalanceSample {
        let t = lean_deg.to_radians();
        let top = [t.sin() * heading.cos(), t.sin() * heading.sin(), t.cos()];
        let reading = posture_reading([0.0; 3], top).expect("line");
        let mut s = sample(seq, 1, 2.0);
        s.posture = Some(reading.sample(Some(yaw)));
        s
    }

    /// Fold a sample the way the phase runner does: the scored tilt is set from the attempt's
    /// latched reset line before the attempt sees it.
    fn fed_from_reset(
        open: &OpenBalanceAttempt,
        s: &BalanceSample,
        c: &BalanceFeetCriterion,
    ) -> BalanceAttemptStep {
        let mut s = s.clone();
        s.score_posture_from(open.reset_posture.as_ref());
        observe_balance_sample(open, &s, c, s.seed).expect("observe")
    }

    #[test]
    fn a_held_reset_lean_is_zero_tilt_and_a_success() {
        let mut c = criterion();
        c.hold_s = 0.1;
        c.attempt_cap_s = 1.0;
        // The stand pose leans 18 deg (max 15): held still, it is the reset pose.
        let mut open = OpenBalanceAttempt::fresh();
        let mut finished = None;
        for seq in 1..=6 {
            let s = leaning_sample(seq, 18.0, 0.4, 0.0);
            match fed_from_reset(&open, &s, &c) {
                BalanceAttemptStep::Continue(next) => open = next,
                BalanceAttemptStep::Finished(record) => {
                    finished = Some(record);
                    break;
                }
            }
        }
        let record = finished.expect("a held pose reaches the hold");
        assert_eq!(record.outcome, BalanceAttemptOutcome::Success);
        assert!(record.final_tilt_deg < 0.01, "{}", record.final_tilt_deg);
        assert!((record.reset_tilt_deg.expect("reset") - 18.0).abs() < 1e-9);
        // Absolute lean is kept apart from the scored one.
        let s = leaning_sample(9, 18.0, 0.4, 0.0);
        let mut scored = s.clone();
        scored.score_posture_from(open.reset_posture.as_ref());
        assert!((scored.absolute_tilt_deg() - 18.0).abs() < 1e-9);
        assert!(scored.scored_tilt_deg() < 0.01);
    }

    #[test]
    fn the_reset_lean_turns_with_the_heading() {
        let reset = leaning_sample(1, 18.0, 0.0, 0.0)
            .posture
            .expect("posture")
            .reference;
        let reset = reset.expect("reference");
        // The same pose after a 90 deg turn about up: the line leans 90 deg further round.
        let turned = leaning_sample(
            2,
            18.0,
            std::f64::consts::FRAC_PI_2,
            std::f64::consts::FRAC_PI_2,
        )
        .posture
        .expect("posture")
        .reference
        .expect("reference");
        let lean = turned.lean_from(&reset);
        assert!(lean.angle_rad.to_degrees() < 0.01, "{lean:?}");
        // Without the heading change the same line is 25 deg away (2 * 18 * sin... a chord), so
        // the rotation is what removes it.
        let unturned = LeanReference {
            yaw_rad: reset.yaw_rad,
            ..turned
        };
        assert!(unturned.lean_from(&reset).angle_rad.to_degrees() > 20.0);
        // A 16 deg lean added toward the same side is 16 deg from the reset line.
        let more = leaning_sample(3, 34.0, 0.0, 0.0)
            .posture
            .expect("posture")
            .reference
            .expect("reference");
        assert!((more.lean_from(&reset).angle_rad.to_degrees() - 16.0).abs() < 1e-9);
    }

    #[test]
    fn a_lean_added_to_the_reset_pose_is_not_upright_and_a_fall_trips_relative() {
        let mut c = fall_criterion();
        c.fall_tilt_deg = 30.0;
        let mut open = OpenBalanceAttempt::fresh();
        open = match fed_from_reset(&open, &leaning_sample(1, 18.0, 0.0, 0.0), &c) {
            BalanceAttemptStep::Continue(next) => next,
            other => panic!("{other:?}"),
        };
        // 16 deg more lean (max 15): the tilt test fails; 31 deg more passes the fall tilt (30).
        let mut s = leaning_sample(2, 34.0, 0.0, 0.0);
        s.score_posture_from(open.reset_posture.as_ref());
        assert!((s.scored_tilt_deg() - 16.0).abs() < 1e-9);
        assert!(non_upright_reasons(&s, &c, None).tilt);
        for seq in 2..=5 {
            open = match fed_from_reset(&open, &leaning_sample(seq, 34.0, 0.0, 0.0), &c) {
                BalanceAttemptStep::Continue(next) => next,
                other => panic!("{other:?}"),
            };
        }
        assert_eq!(
            open.fall_bursts, 0,
            "16 deg from reset is below the fall tilt"
        );
        let mut fell = None;
        for seq in 6..=11 {
            if let BalanceAttemptStep::Finished(r) =
                fed_from_reset(&open, &leaning_sample(seq, 49.0, 0.0, 0.0), &c)
            {
                fell = Some(r);
                break;
            }
            open = match fed_from_reset(&open, &leaning_sample(seq, 49.0, 0.0, 0.0), &c) {
                BalanceAttemptStep::Continue(next) => next,
                _ => unreachable!(),
            };
        }
        let record = fell.expect("31 deg from the reset pose trips the fall");
        assert_eq!(record.outcome, BalanceAttemptOutcome::Fell);
        assert!((record.final_tilt_deg - 31.0).abs() < 1e-9);
    }

    #[test]
    fn a_stand_pose_that_leans_is_a_success_through_the_phase_runner() {
        let mut c = criterion_for(2, 0.05);
        c.settle_s = 0.0;
        c.hold_s = 0.5;
        c.attempt_cap_s = 1.0;
        // The posture line stands 18 deg off vertical (max 15), held still.
        let mut mock = MockController::new(
            0.05,
            2,
            Box::new(|_, _| Scripted {
                tilt_deg: 18.0,
                ..upright_step()
            }),
        );
        let report = run_phase(&c, &mut mock, "train", &[1]).expect("phase");
        let attempt = &report.attempts[0];
        assert_eq!(attempt.outcome, BalanceAttemptOutcome::Success);
        assert!(attempt.final_tilt_deg < 0.01, "{}", attempt.final_tilt_deg);
        assert!((attempt.reset_tilt_deg.expect("reset") - 18.0).abs() < 1e-6);
    }

    #[test]
    fn window_cover_counts_a_trailing_window_only_when_it_continues_a_completed_one() {
        let mut tally = AdaptiveAttemptTally {
            bursts: 160.0,
            success_windows: 1,
            window_covered_s: 3.0,
            window_s: 2.0,
            ..AdaptiveAttemptTally::default()
        };
        let cover = |t: &AdaptiveAttemptTally| t.window_cover_fraction(0.05);
        assert!(
            (cover(&tally) - 3.0 / 8.0).abs() < 1e-9,
            "a fresh partial is not counted"
        );
        tally.partial_continues = true;
        assert!((cover(&tally) - 5.0 / 8.0).abs() < 1e-9);
        tally.success_windows = 0;
        assert_eq!(cover(&tally), 0.0, "no completed window, no cover");
    }

    #[test]
    fn a_standing_cap_that_is_not_whole_windows_is_a_success_with_full_cover() {
        // 8 s cap, 3 s windows: two whole windows and a 2 s tail that continues them.
        let mut c = quick_adaptive();
        c.hold_s = 3.0;
        c.attempt_cap_s = 8.0;
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
        let attempt = &report.attempts[0];
        let stats = attempt.adaptive.expect("stats");
        assert_eq!(attempt.outcome, BalanceAttemptOutcome::Success, "{stats:?}");
        assert_eq!(stats.success_windows, 2);
        assert!(
            (stats.window_cover_fraction - 1.0).abs() < 1e-9,
            "{stats:?}"
        );
        assert_eq!(stats.demotion_reason, None);
    }

    #[test]
    fn a_cap_with_no_completed_window_is_demoted_and_says_why() {
        // Standing the whole cap, but with the support never idle no window can complete.
        let mut c = quick_adaptive();
        c.idle_fraction = 0.0;
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
        let attempt = &report.attempts[0];
        let stats = attempt.adaptive.expect("stats");
        assert_eq!(attempt.outcome, BalanceAttemptOutcome::Truncated);
        assert_eq!(stats.success_windows, 0);
        assert_eq!(stats.window_cover_fraction, 0.0);
        assert_eq!(stats.demotion_reason, Some(WindowDemotion::NoSuccessWindow));
    }

    #[test]
    fn the_attempts_table_carries_reset_tilt_cover_and_demotion() {
        let mut c = quick_adaptive();
        c.idle_fraction = 0.0;
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
        let mut record = report.attempts[0].clone();
        record.reset_tilt_deg = Some(7.5);
        let spec = attempts_table_spec(false);
        let row = attempt_row("train", 0, &record, false);
        assert_eq!(spec.columns.len(), row.len());
        let at = |name: &str| {
            let index = spec
                .columns
                .iter()
                .position(|col| col.name == name)
                .unwrap_or_else(|| panic!("column {name}"));
            row[index].clone()
        };
        assert_eq!(at("reset_tilt"), RunDataValue::Float(7.5));
        assert_eq!(at("window_cover_fraction"), RunDataValue::Float(0.0));
        assert_eq!(
            at("demotion_reason"),
            RunDataValue::text("no_success_window")
        );
    }

    #[test]
    fn the_learned_loss_tilt_is_the_posture_tilt() {
        let c = quick_adaptive();
        let mut mock = MockController::new(
            0.05,
            2,
            // (k 0 is the sizing read, k 1 the reset sample.) The posture line leans 2 deg at the reset sample and 6 deg after: 4 deg from reset.
            Box::new(|k, t| Scripted {
                down: t < 1.0,
                tilt_deg: if k <= 1 { 1.0 } else { 3.0 },
                ..upright_step()
            }),
        );
        mock.posture_tilt_factor = 2.0;
        let mut delivered = Vec::new();
        let report = adaptive_train(
            &c,
            &mut mock,
            &[1],
            None,
            BalanceRewardPolicy::Affect,
            &mut delivered,
        );
        assert_eq!(
            report.attempts[0].outcome,
            BalanceAttemptOutcome::LostStance
        );
        assert!((report.attempts[0].stance_lost_tilt_deg.unwrap() - 4.0).abs() < 1e-6);
        let next = report.adaptive_state_next.expect("state");
        assert!(
            (next.loss_tilts_deg[0] - 4.0).abs() < 1e-6,
            "{:?}",
            next.loss_tilts_deg
        );
        assert_eq!(next.posture_definition, Some(c.posture_definition()));
    }

    #[test]
    fn posture_top_body_is_required_distinct_and_in_the_model() {
        let c = criterion_for(2, 0.05);
        assert!(c.validate().is_ok());
        let mut missing = c.clone();
        missing.posture_top_body = String::new();
        let message = missing.validate().unwrap_err().to_string();
        assert!(
            message.contains("posture_top_body is required"),
            "{message}"
        );
        let old: BalanceFeetCriterion = serde_json::from_value(serde_json::json!({
            "torso_body": "torso", "contact_bodies": ["contact_a"],
            "max_tilt_deg": 15.0, "hold_s": 1.0, "attempt_cap_s": 3.0
        }))
        .expect("an old setup still parses");
        assert!(old
            .validate()
            .unwrap_err()
            .to_string()
            .contains("posture_top_body is required"));
        let mut same = c.clone();
        same.posture_top_body = "contact_b".to_string();
        assert!(same
            .validate()
            .unwrap_err()
            .to_string()
            .contains("must not be one of"));
        let mut ghost = c.clone();
        ghost.posture_top_body = "ghost".to_string();
        assert!(ghost.validate().is_ok());
        let message = check_model(&generic_body_info(), &ghost)
            .unwrap_err()
            .to_string();
        assert!(message.contains("ghost"), "{message}");
        assert!(check_model(&generic_body_info(), &c).is_ok());
        // The measure request lists each body once: torso, contacts, top.
        let mut torso_top = c.clone();
        torso_top.posture_top_body = "torso".to_string();
        assert_eq!(
            measure_request(&torso_top).bodies,
            vec![
                "torso".to_string(),
                "contact_a".to_string(),
                "contact_b".to_string()
            ]
        );
        assert_eq!(
            measure_request(&c).bodies,
            vec![
                "torso".to_string(),
                "contact_a".to_string(),
                "contact_b".to_string(),
                "top".to_string()
            ]
        );
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
            actuator_force: BTreeMap::new(),
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
        AdaptiveAssist::resume(c, AdaptiveAssistState::new(a, a_slow))
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
    fn adaptive_cone_widens_per_success_window_and_stays_in_range() {
        let c = adaptive_criterion(0.05);
        let mut assist = AdaptiveAssist::start(&c);
        assert!(
            (assist.cone_deg(&c) - c.cone_min_deg).abs() < 1e-12,
            "full assist, narrowest cone"
        );
        let mut last = assist.cone_deg(&c);
        for i in 0..4000 {
            let mut step = assist.step(&c, 0.0, 1);
            // The slow level moves only per success window (every 10 s of idle standing here).
            if i % 200 == 199 {
                assist.success_window(&c);
                step.cone_deg = assist.cone_deg(&c);
            }
            assert!(
                step.cone_deg >= last - 1e-12,
                "cone never narrows while windows complete"
            );
            assert!(
                step.cone_deg >= c.cone_min_deg - 1e-12 && step.cone_deg <= c.max_tilt_deg + 1e-12,
                "{}",
                step.cone_deg
            );
            last = step.cone_deg;
        }
        assert!(
            last > 9.0,
            "after 20 success windows the cone is wide: {last}"
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
            recovery: &TEST_RECOVERY,
            burst: &RUNNING_CLOCK,
            memory_counters: None,
            reset_latch: None,
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
        // Scripted bodies carry no real support effort; the idle test is exercised on its own.
        c.idle_fraction = 1.0;
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
        // No loss yet: the limit is the allowed angle, and the attempt starts at full assist.
        assert_eq!(stats.a_start, 1.0);
        assert_eq!(stats.a_slow_start, 1.0);
        assert_eq!(stats.effective_max_deg, c.max_tilt_deg);
        assert_eq!(stats.learned_loss_tilt_deg, None);
        assert_eq!(first.stance_lost_tilt_deg, None);
        let second_stats = second.adaptive.expect("adaptive numbers");
        assert!((second_stats.a_start - second.support_level).abs() < 1e-12);
        assert!(second_stats.a_slow_start < 1.0 + 1e-12);
        assert_eq!(second_stats.effective_max_deg, c.max_tilt_deg);
        assert!(!report.cone_min_above_loss_tilt);
        assert_eq!(stats.catches, 0);
        // Windows widen the cone; no lean, so no re-tightening. Both reach the run data.
        assert!(stats.success_windows >= 5, "{stats:?}");
        assert_eq!(stats.lean_retightens, 0);
        assert!(second_stats.success_windows >= 5, "{second_stats:?}");
        // The slow level only moved per window: first-attempt end equals the second's start.
        assert!(second_stats.a_slow_start < 1.0 - 1e-6, "{second_stats:?}");
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
        assert!(next.loss_tilts_deg.is_empty(), "successes record no loss");
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
        let state = AdaptiveAssistState::new(0.4, 0.6);
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
        // The saved state keeps a_slow exactly when no success window completes (support never
        // idle): time alone does not widen the cone.
        let mut busy = c.clone();
        busy.idle_fraction = 0.0;
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let report = adaptive_train(
            &busy,
            &mut mock,
            &[1],
            Some(SupportMode::ResumeAdaptive(AdaptiveAssistState::new(
                0.4, 0.6,
            ))),
            BalanceRewardPolicy::Affect,
            &mut delivered,
        );
        let saved = report.adaptive_state_next.expect("state");
        assert_eq!(saved.a_slow, 0.6, "{saved:?}");
        assert_eq!(
            report.attempts[0].adaptive.expect("stats").success_windows,
            0
        );
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let report = adaptive_train(
            &c,
            &mut mock,
            &[1],
            Some(SupportMode::ResumeAdaptive(saved)),
            BalanceRewardPolicy::Affect,
            &mut delivered,
        );
        // With windows allowed the resumed run widens from the saved slow level.
        assert!(report.adaptive_state_next.expect("state").a_slow < 0.6);
        // A saved state resumes with its learned losses: 25th percentile of [10, 12] is 10.5,
        // so the limit is 10.5 * 0.8 = 8.4 deg.
        let mut with_losses = AdaptiveAssistState::new(0.4, 0.6);
        with_losses.loss_tilts_deg = vec![10.0, 12.0];
        with_losses.posture_definition = Some(c.posture_definition());
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let report = adaptive_train(
            &c,
            &mut mock,
            &[1],
            Some(SupportMode::ResumeAdaptive(with_losses)),
            BalanceRewardPolicy::Affect,
            &mut delivered,
        );
        let stats = report.attempts[0].adaptive.expect("stats");
        assert!((stats.learned_loss_tilt_deg.expect("learned") - 10.5).abs() < 1e-9);
        assert!(
            (stats.effective_max_deg - 8.4).abs() < 1e-9,
            "{}",
            stats.effective_max_deg
        );
        assert_eq!(
            report.adaptive_state_next.expect("state").loss_tilts_deg,
            vec![10.0, 12.0]
        );
        // An older saved state (no loss list) still loads.
        let older: AdaptiveAssistState =
            serde_json::from_value(serde_json::json!({"a": 0.4, "a_slow": 0.6})).expect("older");
        assert!(older.loss_tilts_deg.is_empty());
    }

    #[test]
    fn adaptive_cone_never_exceeds_the_learned_loss_tilt_inside_the_margin() {
        let c = adaptive_criterion(0.05);
        assert_eq!(c.max_tilt_deg, 15.0);
        assert_eq!(
            (c.loss_margin_fraction, c.fail_tighten, c.loss_window),
            (0.2, 0.5, 5)
        );
        // No loss: the limit is max_tilt_deg and the widest cone reaches it.
        let mut state = AdaptiveAssistState::new(0.0, 0.0);
        assert_eq!(AdaptiveAssist::resume(&c, state.clone()).cone_deg(&c), 15.0);
        // Losses at 10 deg: the limit is 8 deg, whatever the slow average.
        state.loss_tilts_deg = vec![10.0; 5];
        state.posture_definition = Some(c.posture_definition());
        for a_slow in [0.0, 0.25, 0.5, 0.75, 1.0] {
            state.a_slow = a_slow;
            let assist = AdaptiveAssist::resume(&c, state.clone());
            let cone = assist.cone_deg(&c);
            assert!(
                cone <= 10.0 * (1.0 - c.loss_margin_fraction) + 1e-12,
                "{cone}"
            );
            assert!(cone >= c.cone_min_deg - 1e-12);
            assert_eq!(assist.effective_max_deg(&c), 8.0);
        }
        // Losses past the allowed angle never widen it.
        state.loss_tilts_deg = vec![40.0; 5];
        state.a_slow = 0.0;
        assert_eq!(AdaptiveAssist::resume(&c, state).cone_deg(&c), 15.0);
        // The window keeps only the last 3 losses.
        let mut windowed = c.clone();
        windowed.loss_window = 3;
        let mut state = AdaptiveAssistState::new(0.0, 0.0);
        state.loss_tilts_deg = vec![1.0, 1.0, 12.0, 12.0, 12.0];
        state.posture_definition = Some(windowed.posture_definition());
        let assist = AdaptiveAssist::resume(&windowed, state);
        assert_eq!(assist.learned_loss_tilt_deg(&windowed), Some(12.0));
    }

    #[test]
    fn adaptive_loss_raises_both_levels_and_the_next_attempt_starts_there() {
        let c = quick_adaptive();
        // Exact numbers on the assist itself: 0.2 + 0.8 * 0.5 = 0.6 for the slow average, the
        // level follows it up.
        let mut assist = at(&c, 0.2, 0.2);
        assist.record_attempt_end(&c, BalanceAttemptOutcome::LostStance, Some(9.0));
        assert!((assist.slow() - 0.6).abs() < 1e-12 && (assist.level() - 0.6).abs() < 1e-12);
        assert_eq!(assist.learned_loss_tilt_deg(&c), Some(9.0));
        // A level already above the slow average stays where it is; a success changes nothing.
        let mut assist = at(&c, 0.9, 0.2);
        assist.record_attempt_end(&c, BalanceAttemptOutcome::Fell, Some(30.0));
        assert!((assist.slow() - 0.6).abs() < 1e-12 && (assist.level() - 0.9).abs() < 1e-12);
        let before = assist.clone();
        assist.record_attempt_end(&c, BalanceAttemptOutcome::Success, Some(1.0));
        assert_eq!(assist, before);
        // In a phase: the stance goes at 1 s, the next attempt starts at the tightened level.
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
            &c,
            &mut mock,
            &[1, 2],
            Some(SupportMode::ResumeAdaptive(AdaptiveAssistState::new(
                0.2, 0.2,
            ))),
            BalanceRewardPolicy::Affect,
            &mut delivered,
        );
        let (first, second) = (&report.attempts[0], &report.attempts[1]);
        assert_eq!(first.outcome, BalanceAttemptOutcome::LostStance);
        let first_stats = first.adaptive.expect("stats");
        assert_eq!(first_stats.a_start, 0.2);
        // Tilt is measured from the reset pose: the stance is lost with the lean unchanged.
        assert_eq!(first.stance_lost_tilt_deg, Some(0.0));
        let second_stats = second.adaptive.expect("stats");
        // The slow average fades during the attempt, then a loss moves it at least halfway to 1.
        assert!(
            second_stats.a_slow_start >= 0.5 - 1e-9,
            "{}",
            second_stats.a_slow_start
        );
        assert!(second_stats.a_slow_start > first_stats.a_slow_start);
        assert!(second_stats.a_start >= second_stats.a_slow_start - 1e-9);
        assert!((second.support_level - second_stats.a_start).abs() < 1e-12);
        // Measured from the reset pose, the lean at the loss is 0.
        assert_eq!(second_stats.learned_loss_tilt_deg, Some(0.0));
        let next = report.adaptive_state_next.expect("state");
        assert_eq!(next.loss_tilts_deg, vec![0.0, 0.0]);
        assert!(next.a_slow >= second_stats.a_slow_start);
    }

    #[test]
    fn adaptive_flags_a_cone_min_above_the_learned_loss_tilt() {
        let c = quick_adaptive();
        assert_eq!(c.cone_min_deg, 2.0);
        // A loss at 1.5 deg puts the limit (1.2) below cone_min: held at cone_min and flagged.
        let mut state = AdaptiveAssistState::new(0.5, 0.5);
        state.loss_tilts_deg = vec![1.5];
        state.posture_definition = Some(c.posture_definition());
        let assist = AdaptiveAssist::resume(&c, state.clone());
        assert_eq!(assist.effective_max_deg(&c), c.cone_min_deg);
        assert_eq!(assist.cone_deg(&c), c.cone_min_deg);
        assert!(assist.cone_min_clamped());
        // A loss above cone_min does not flag it.
        let assist = at(&c, 0.5, 0.5);
        assert!(!assist.cone_min_clamped());
        let mut ok = at(&c, 0.5, 0.5);
        ok.record_attempt_end(&c, BalanceAttemptOutcome::Fell, Some(10.0));
        assert!(!ok.cone_min_clamped());
        let mut low = at(&c, 0.5, 0.5);
        low.record_attempt_end(&c, BalanceAttemptOutcome::Fell, Some(1.0));
        assert!(low.cone_min_clamped());
        // It reaches the phase report.
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
        assert!(report.cone_min_above_loss_tilt);
    }

    #[test]
    fn adaptive_overshoot_reaches_one_at_the_loss_angle() {
        let mut c = adaptive_criterion(0.05);
        c.loss_margin_fraction = 0.0;
        let mut state = AdaptiveAssistState::new(0.5, 0.5);
        state.loss_tilts_deg = vec![10.0];
        state.posture_definition = Some(c.posture_definition());
        // Cone 2 + 8 * 0.5 = 6 deg, limit 10 deg: at the loss angle the overshoot is 1, so the
        // level rises by the full fast step toward 1.
        let fast = 0.5 * (1.0 - (-c.burst_period_s / c.catch_tau_s).exp());
        let mut assist = AdaptiveAssist::resume(&c, state.clone());
        assert_eq!(assist.cone_deg(&c), 6.0);
        let step = assist.step(&c, 10.0, 1);
        assert!((step.rise - fast).abs() < 1e-9, "{} vs {fast}", step.rise);
        // Halfway between the cone and the limit the overshoot is 0.5: no rise.
        let mut assist = AdaptiveAssist::resume(&c, state.clone());
        assert_eq!(assist.step(&c, 8.0, 1).rise, 0.0);
        // With the default margin the limit is 8 deg and the overshoot reaches 1 there.
        let c = adaptive_criterion(0.05);
        let mut assist = AdaptiveAssist::resume(&c, state);
        assert_eq!(assist.effective_max_deg(&c), 8.0);
        assert_eq!(assist.cone_deg(&c), 5.0);
        let fast = 0.5 * (1.0 - (-c.burst_period_s / c.catch_tau_s).exp());
        let step = assist.step(&c, 8.0, 1);
        assert!((step.rise - fast).abs() < 1e-9, "{} vs {fast}", step.rise);
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
    fn adaptive_lean_retightens_after_grace_and_the_spring_cone_is_the_scored_cone() {
        let c = stance_adaptive();
        // A wide cone (slow level well below the ceiling), then a lean inside the tilt limit
        // but beyond that cone for 4 s, then upright again.
        let state = AdaptiveAssistState::new(0.3, 0.3);
        let (report, mock, _) = one_adaptive_attempt(
            &c,
            Some(SupportMode::ResumeAdaptive(state.clone())),
            BalanceRewardPolicy::Affect,
            Box::new(|_, t| Scripted {
                tilt_deg: if (1.0..5.0).contains(&t) { 13.0 } else { 1.0 },
                ..upright_step()
            }),
        );
        let attempt = &report.attempts[0];
        let stats = attempt.adaptive.expect("adaptive numbers");
        // Cone 2 + 13 * 0.7 = 11.1 deg; the lean (13 deg) outlasts the 2 s grace twice in 4 s.
        assert!(stats.lean_retightens >= 1, "{stats:?}");
        // The spring dead band is the one scored cone: first update is the opening cone, and
        // every update lies inside [cone_min, effective max]. After the lean the cone is
        // narrower than at the start (no separate recentering state).
        let deg: Vec<f64> = mock
            .deadband_updates
            .iter()
            .flatten()
            .map(|rad| rad.to_degrees())
            .collect();
        assert!(!deg.is_empty());
        assert!(deg
            .iter()
            .all(|d| *d >= c.cone_min_deg - 1e-9 && *d <= c.max_tilt_deg + 1e-9));
        assert!((deg[0] - 11.1).abs() < 0.11, "{deg:?}");
        assert!(
            deg.iter().cloned().fold(f64::MAX, f64::min) < deg[0] - 1.0,
            "{deg:?}"
        );
        // The last update matches the cone the attempt ended in (updates send only moves > 0.1).
        assert!(
            (deg[deg.len() - 1] - stats.end_cone_deg).abs() < 0.11,
            "{} vs {}",
            deg[deg.len() - 1],
            stats.end_cone_deg
        );
        // A short lean (shorter than the grace) never retightens.
        let (report, _, _) = one_adaptive_attempt(
            &c,
            Some(SupportMode::ResumeAdaptive(state.clone())),
            BalanceRewardPolicy::Affect,
            Box::new(|_, t| Scripted {
                tilt_deg: if (1.0..2.5).contains(&t) { 13.0 } else { 1.0 },
                ..upright_step()
            }),
        );
        assert_eq!(
            report.attempts[0].adaptive.expect("stats").lean_retightens,
            0
        );
    }

    #[test]
    fn a_passive_lean_beyond_cone_min_never_widens_the_cone() {
        let mut c = stance_adaptive();
        c.idle_fraction = 1.0;
        c.cone_min_deg = 5.0;
        // Full assist, cone at cone_min; the robot leans at 4 deg (inside the 5 deg cone)
        // without ever being idle (effort is never reported), so no window completes.
        let mut c_busy = c.clone();
        c_busy.idle_fraction = 0.0;
        let (report, _, _) = one_adaptive_attempt(
            &c_busy,
            Some(SupportMode::ResumeAdaptive(AdaptiveAssistState::new(
                1.0, 1.0,
            ))),
            BalanceRewardPolicy::Affect,
            Box::new(|_, _| Scripted {
                tilt_deg: 4.0,
                ..upright_step()
            }),
        );
        let attempt = &report.attempts[0];
        let stats = attempt.adaptive.expect("stats");
        let next = report.adaptive_state_next.as_ref().expect("state");
        assert_eq!(stats.success_windows, 0, "{stats:?}");
        assert_eq!(next.a_slow, 1.0, "the slow level did not move: {next:?}");
        assert!(
            (stats.end_cone_deg - c.cone_min_deg).abs() < 1e-9,
            "{stats:?}"
        );
        // The attempt held a passive lean at the cap: not a success.
        assert_eq!(attempt.outcome, BalanceAttemptOutcome::Truncated);
    }

    #[test]
    fn a_success_window_widens_the_cone_by_the_stated_step() {
        let mut c = quick_adaptive();
        c.hold_s = 0.5;
        c.fade_tau_s = 2.0;
        let mut assist = AdaptiveAssist::resume(&c, AdaptiveAssistState::new(0.2, 0.8));
        let before = assist.cone_deg(&c);
        assist.success_window(&c);
        let share = 1.0 - (-c.hold_s / c.fade_tau_s).exp();
        let expected = 0.8 + (0.2 - 0.8) * share;
        assert!(
            (assist.slow() - expected).abs() < 1e-12,
            "{}",
            assist.slow()
        );
        assert!(assist.cone_deg(&c) > before);
        // Time alone never moves the slow level.
        for _ in 0..200 {
            assist.step(&c, 0.0, 1);
        }
        assert!((assist.slow() - expected).abs() < 1e-12);
        // Through a phase: idle holds inside the cone widen it window by window.
        let mut idle = quick_adaptive();
        idle.attempt_cap_s = 3.0;
        let (report, _, _) = one_adaptive_attempt(
            &idle,
            None,
            BalanceRewardPolicy::Affect,
            Box::new(|_, _| upright_step()),
        );
        let stats = report.attempts[0].adaptive.expect("stats");
        assert!(stats.success_windows >= 5, "{stats:?}");
        assert!(stats.end_cone_deg > idle.cone_min_deg + 0.5, "{stats:?}");
    }

    #[test]
    fn a_capped_attempt_is_a_success_only_with_idle_success_windows() {
        // Support effort in the scripted body follows the level; with an idle limit of 0 no
        // window can complete, so standing the whole cap is not a success.
        let mut c = quick_adaptive();
        c.idle_fraction = 0.0;
        let mut delivered = Vec::new();
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let report = adaptive_train(
            &c,
            &mut mock,
            &[1],
            None,
            BalanceRewardPolicy::Affect,
            &mut delivered,
        );
        let attempt = &report.attempts[0];
        assert_eq!(attempt.outcome, BalanceAttemptOutcome::Truncated);
        assert_eq!(attempt.adaptive.expect("stats").success_windows, 0);
        assert!(
            !delivered
                .iter()
                .any(|s| s.channel == AffectChannel::Pleasure && s.magnitude >= 1.0 - 1e-9),
            "no success Pleasure: {delivered:?}"
        );
        // With the idle limit open the same body succeeds at the cap.
        let mut open = quick_adaptive();
        open.idle_fraction = 1.0;
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        let mut delivered = Vec::new();
        let report = adaptive_train(
            &open,
            &mut mock,
            &[1],
            None,
            BalanceRewardPolicy::Affect,
            &mut delivered,
        );
        assert_eq!(report.attempts[0].outcome, BalanceAttemptOutcome::Success);
        assert!(report.attempts[0].adaptive.expect("stats").success_windows >= 5);
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
        // The tilt at the first burst off the stance is kept (measured from the reset pose, the
        // scripted 2 deg lean is 0), and the loss feeds the learned limit the next attempt starts
        // from.
        assert_eq!(attempt.stance_lost_tilt_deg, Some(0.0));
        assert_eq!(
            stats.learned_loss_tilt_deg, None,
            "learned only after the loss"
        );
        assert_eq!(
            report
                .adaptive_state_next
                .as_ref()
                .expect("state")
                .loss_tilts_deg,
            vec![0.0]
        );
        assert!(
            report.cone_min_above_loss_tilt,
            "2 * 0.8 is below cone_min 2"
        );
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
        assert_eq!(
            report.attempts[0].stance_lost_tilt_deg, None,
            "standing again at the end clears it"
        );
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

    /// Net recovery pays at any assist level, also when no single burst clears the payment floor.
    #[test]
    fn contribution_reward_accumulates_a_slow_recovery_at_support_zero() {
        let c = adaptive_criterion(0.05);
        let mut r = BalanceContributionReward::new();
        r.on_sample(&contribution_sample(0.60, 0.0), None, &c);
        // Each burst drops need by 0.004: far under the 0.0125 per-burst floor.
        let mut paid = Vec::new();
        let mut need = 0.60;
        for _ in 0..40 {
            need -= 0.004;
            if let Some(p) = pleasure_of(&r.on_sample(&contribution_sample(need, 0.0), None, &c)) {
                paid.push(p);
            }
        }
        // 0.16 of recovery in total, paid in small steps; the credit resets after each payment.
        assert!(paid.len() >= 3, "{paid:?}");
        let total: f64 = paid.iter().sum();
        assert!(
            (total - ASSIST_FALL_GAIN * 0.16).abs() < ASSIST_FALL_GAIN * 0.004 * 4.0,
            "{total}"
        );
        assert!(paid.iter().all(|p| *p >= ASSIST_MIN_PLEASURE));
    }

    #[test]
    fn contribution_reward_pays_nothing_for_a_wobble_that_nets_to_zero() {
        let c = adaptive_criterion(0.05);
        let mut r = BalanceContributionReward::new();
        r.on_sample(&contribution_sample(0.40, 0.0), None, &c);
        for i in 0..200 {
            let need = if i % 2 == 0 { 0.396 } else { 0.40 };
            assert!(
                pleasure_of(&r.on_sample(&contribution_sample(need, 0.0), None, &c)).is_none(),
                "burst {i}"
            );
        }
    }

    #[test]
    fn contribution_reward_credit_is_weighted_by_the_assist_level_when_it_is_earned() {
        let c = adaptive_criterion(0.05);
        // The same recovery in small steps: all at a = 0.5 pays about half of all at a = 0.
        let total = |level: f64| -> f64 {
            let mut r = BalanceContributionReward::new();
            r.on_sample(&contribution_sample(0.6, level), None, &c);
            let mut need = 0.6;
            let mut sum = 0.0;
            for _ in 0..50 {
                need -= 0.004;
                sum += pleasure_of(&r.on_sample(&contribution_sample(need, level), None, &c))
                    .unwrap_or(0.0);
            }
            sum
        };
        let (free, held) = (total(0.0), total(0.5));
        assert!(free > 0.5 && held > 0.0);
        assert!((held / free - 0.5).abs() < 0.15, "{free} {held}");
        // Recovery earned while held is not paid later at a low level.
        let mut r = BalanceContributionReward::new();
        r.on_sample(&contribution_sample(0.6, 0.9), None, &c);
        let mut need = 0.6;
        for _ in 0..10 {
            need -= 0.004;
            assert!(pleasure_of(&r.on_sample(&contribution_sample(need, 0.9), None, &c)).is_none());
        }
        let late = r.on_sample(&contribution_sample(need, 0.0), None, &c);
        assert!(
            pleasure_of(&late).is_none(),
            "a held recovery is not paid at a = 0"
        );
    }

    #[test]
    fn contribution_reward_credit_resets_on_settle_non_standing_and_death() {
        let c = adaptive_criterion(0.05);
        let standing = |need: f64| contribution_sample(need, 0.0);
        let lifted = |need: f64| BalanceSample {
            contacts: vec![true, false],
            ..contribution_sample(need, 0.0)
        };
        let mut r = BalanceContributionReward::new();
        r.on_sample(&standing(0.6), None, &c);
        r.on_sample(&standing(0.59), None, &c);
        // A lifted burst throws the credit away.
        r.on_sample(&lifted(0.58), None, &c);
        assert!(pleasure_of(&r.on_sample(&standing(0.575), None, &c)).is_none());
        // A fall clears it too.
        let mut r = BalanceContributionReward::new();
        r.on_sample(&standing(0.6), None, &c);
        r.on_sample(&standing(0.595), None, &c);
        r.on_sample(&standing(0.59), Some(&BalanceAttemptOutcome::Fell), &c);
        assert!(pleasure_of(&r.on_sample(&standing(0.588), None, &c)).is_none());
    }

    #[test]
    fn contribution_reward_one_large_drop_pays_as_before() {
        let c = adaptive_criterion(0.05);
        let mut r = BalanceContributionReward::new();
        r.on_sample(&contribution_sample(0.5, 0.0), None, &c);
        let paid = pleasure_of(&r.on_sample(&contribution_sample(0.4, 0.0), None, &c));
        assert!((paid.unwrap() - 0.4).abs() < 1e-9, "min(1, 4 * 0.1)");
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
            recovery: &TEST_RECOVERY,
            burst: &RUNNING_CLOCK,
            memory_counters: None,
            reset_latch: None,
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
            "reward.balance-contribution.v4"
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
            actuator_force: BTreeMap::new(),
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
        assert!(bad(&|c| c.loss_margin_fraction = -0.1));
        assert!(bad(&|c| c.loss_margin_fraction = 1.0));
        assert!(bad(&|c| c.fail_tighten = 1.5));
        assert!(bad(&|c| c.fail_tighten = f64::NAN));
        assert!(bad(&|c| c.loss_window = 0));
        assert!(bad(&|c| c.catch_threshold = 1.1));
        assert!(bad(&|c| c.catch_threshold = -0.1));
        assert!(bad(&|c| c.lean_grace_s = 0.0));
        assert!(bad(&|c| c.idle_fraction = -0.1));
        assert!(bad(&|c| c.idle_fraction = 1.1));
        assert!(bad(&|c| c.idle_fraction = f64::NAN));
        assert!(!bad(&|c| c.idle_fraction = 0.0));
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
            (
                stats.lean_retightens,
                stats.lost_stance,
                stats.standing_fraction
            ),
            (0, false, 0.0)
        );
        let outcome: BalanceAttemptOutcome =
            serde_json::from_value(serde_json::json!("lost_stance")).expect("outcome");
        assert_eq!(outcome, BalanceAttemptOutcome::LostStance);
    }

    #[test]
    fn update_body_spring_request_carries_the_deadband_only_when_given() {
        assert!(update_body_spring_request("torso", 0.4, None, None, None)
            .get("deadband_rad")
            .is_none());
        let with = update_body_spring_request("torso", 0.4, Some(0.1), None, None);
        assert_eq!(with["deadband_rad"], 0.1);
        assert_eq!(with["level"], 0.4);
        assert!(with.get("ref_up_world").is_none());
        let steered = update_body_spring_request("torso", 0.4, None, None, Some([0.0, 0.1, 0.9]));
        assert_eq!(steered["ref_up_world"], serde_json::json!([0.0, 0.1, 0.9]));
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
        // a constant 2 deg lean held from the reset pose is no deviation from it
        assert!(block.timeline[0].deviation_mean.abs() < 1e-9);
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
            recovery: &TEST_RECOVERY,
            burst: &RUNNING_CLOCK,
            memory_counters: None,
            reset_latch: None,
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
        // The mock reports force when asked, so the force column is declared from the model's
        // actuator names; it is absent when the controller reports none (see the no-sink test).
        let force = col("actuator_force_joint_a");
        assert!(tape
            .rows_of("steps")
            .iter()
            .all(|r| matches!(r[force], RunDataValue::Float(_))));
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
            "posture_top_body": "head",
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
        assert!(measure_request_body(&measure_request(&c))
            .get("actuator_force")
            .is_none());
        c.record_actuators = true;
        let body = measure_request_body(&measure_request(&c));
        assert_eq!(body["actuators"], serde_json::json!(true));
        assert_eq!(body["actuator_force"], serde_json::json!(true));
        let parsed: Measurement = serde_json::from_value(serde_json::json!({
            "sim_time_s": 0.1, "physics_steps": 5, "actuators": {"joint_a": 0.25},
            "actuator_force": {"joint_a": 3.5}, "link": {"state": "linked", "epoch": 1}
        }))
        .expect("parse");
        assert_eq!(parsed.actuators["joint_a"], 0.25);
        assert_eq!(parsed.actuator_force["joint_a"], 3.5);
    }

    // ---- pause and resume ----

    /// One segment of a split run. `pause_at_attempt_start` asks for the pause-stop (a skip in
    /// boundary mode) when the n-th attempt of the segment starts. Returns the report, the
    /// export, the scheduled events applied and the attempt-table indices.
    fn balance_segment(
        c: &BalanceFeetCriterion,
        seeds: &[u64],
        from: &crate::resume::ResumeFrom,
        pause_at_attempt_start: Option<usize>,
    ) -> (
        BalancePhaseReport,
        crate::resume::ResumeExport,
        Vec<String>,
        Vec<String>,
    ) {
        balance_segment_with_fault(c, seeds, from, pause_at_attempt_start, None)
    }

    /// `balance_segment`, with the controller link failing once at the n-th measure call of the
    /// segment (that attempt is discarded and re-run).
    fn balance_segment_with_fault(
        c: &BalanceFeetCriterion,
        seeds: &[u64],
        from: &crate::resume::ResumeFrom,
        pause_at_attempt_start: Option<usize>,
        fail_measure_at: Option<usize>,
    ) -> (
        BalancePhaseReport,
        crate::resume::ResumeExport,
        Vec<String>,
        Vec<String>,
    ) {
        let mut mock = MockController::new(0.05, 2, Box::new(|_, _| upright_step()));
        if let Some(at) = fail_measure_at {
            mock.failures = Some(Box::new(move |kind, n| {
                (kind == "measure" && n == at as u32)
                    .then(|| TrainerError::Transient("HTTP 503 busy".to_string()))
            }));
        }
        let mut tape = RecordingRunDataSink::default();
        let cancel = CancelToken::new();
        let pauser = cancel.clone();
        let mut starts = 0usize;
        let mut applied: Vec<String> = Vec::new();
        let mut apply = |action: &ScheduledAction| -> Result<(), TrainerError> {
            match action {
                ScheduledAction::AttemptStart => {
                    starts += 1;
                    if Some(starts) == pause_at_attempt_start {
                        pauser.skip();
                    }
                }
                ScheduledAction::SetMappingPsc { src, .. } => applied.push(src.clone()),
                _ => {}
            }
            Ok(())
        };
        let mut events = CollectingEventSink::default();
        let run_id = rid();
        let mut run = BalancePhaseRun {
            phase: "train",
            seeds,
            criterion: c,
            run_id: &run_id,
            support_mode: None,
            deliver_reward: None,
            plasticity: None,
            reward_policy: BalanceRewardPolicy::Affect,
            data: Some(&mut tape),
            apply_event: if c.scheduled_events.is_empty() && pause_at_attempt_start.is_none() {
                None
            } else {
                Some(&mut apply)
            },
            recovery: &TEST_RECOVERY,
            burst: &RUNNING_CLOCK,
            memory_counters: None,
            reset_latch: None,
        };
        let (report, export) = run_balance_phase_from(
            &mut run,
            from,
            &mut mock,
            &mut |_| Ok(()),
            &mut events,
            &cancel,
        )
        .expect("segment");
        drop(run);
        let indices = tape
            .rows_of("attempts")
            .iter()
            .map(|row| format!("{:?}", row[1]))
            .collect();
        (report, export, applied, indices)
    }

    /// Splits a run at `pause_at` (the attempt in which the pause is asked) and checks it
    /// against the unsplit run.
    fn assert_balance_split_equals_unsplit(
        c: &BalanceFeetCriterion,
        pause_at: usize,
        expected_units: u64,
    ) {
        assert_balance_split_with_fault(c, pause_at, Some(expected_units), None);
    }

    /// `assert_balance_split_equals_unsplit` with a link fault in the first segment (and in the
    /// unsplit run): the last segment's report must hold the discards, the assist timeline and
    /// the deviation timeline of the whole run.
    fn assert_balance_split_with_fault(
        c: &BalanceFeetCriterion,
        pause_at: usize,
        expected_units: Option<u64>,
        fault: Option<usize>,
    ) {
        use crate::resume::ResumeFrom;
        let seeds: Vec<u64> = (1..=6).collect();
        let (unsplit, unsplit_export, unsplit_applied, unsplit_indices) =
            balance_segment_with_fault(c, &seeds, &ResumeFrom::default(), None, fault);
        assert!(!unsplit_export.partial);
        assert_eq!(unsplit_export.units_done, 6);

        let (first, first_export, first_applied, first_indices) = balance_segment_with_fault(
            c,
            &seeds,
            &ResumeFrom::default().at_boundary(),
            Some(pause_at),
            fault,
        );
        assert!(
            first.skipped && first_export.partial,
            "paused segment is partial"
        );
        if let Some(units) = expected_units {
            assert_eq!(first_export.units_done, units);
        }
        // The state survives a JSON round trip, as it does through the saved cursor.
        let saved = serde_json::to_string(&first_export).unwrap();
        let export: crate::resume::ResumeExport = serde_json::from_str(&saved).unwrap();
        let (second, second_export, second_applied, second_indices) =
            balance_segment(c, &seeds, &ResumeFrom::from_export(&export), None);
        assert!(!second_export.partial);
        assert_eq!(second_export.units_done, 6);

        // Same attempts in the same order, probes included, with the same support levels.
        let merged: Vec<_> = first
            .attempts
            .iter()
            .chain(&second.attempts)
            .cloned()
            .collect();
        assert_eq!(merged, unsplit.attempts);
        // Attempt indices continue across the segments.
        let merged_indices: Vec<_> = first_indices.into_iter().chain(second_indices).collect();
        assert_eq!(merged_indices, unsplit_indices);
        // The schedule continues too: events fire once per attempt in every segment.
        assert_eq!(
            first_applied.len() + second_applied.len(),
            unsplit_applied.len()
        );
        assert_eq!(second.support_level_next, unsplit.support_level_next);
        assert_eq!(second.adaptive_state_next, unsplit.adaptive_state_next);
        assert_eq!(second_export.engine_state, unsplit_export.engine_state);
        // The last segment's report carries what the earlier ones kept, in order.
        assert_eq!(second.discarded_attempts, unsplit.discarded_attempts);
        assert_eq!(second.assist_timeline, unsplit.assist_timeline);
        assert_eq!(second.deviation_assist, unsplit.deviation_assist);
    }

    #[test]
    fn balance_split_at_every_attempt_keeps_discards_and_timelines() {
        let mut c = pause_criterion();
        c.support_mode = AssistMode::Adaptive;
        let (unsplit, _, _, _) = balance_segment_with_fault(
            &c,
            &(1..=6).collect::<Vec<u64>>(),
            &crate::resume::ResumeFrom::default(),
            None,
            Some(8),
        );
        assert_eq!(
            unsplit.discarded_attempts.len(),
            1,
            "the fault discards one"
        );
        assert!(!unsplit.assist_timeline.is_empty());
        assert!(unsplit.deviation_assist.is_some());
        for pause_at in 1..=5 {
            assert_balance_split_with_fault(&c, pause_at, None, Some(8));
        }
    }

    fn pause_criterion() -> BalanceFeetCriterion {
        let mut c = criterion_for(2, 0.05);
        c.settle_s = 0.2;
        c.hold_s = 0.6;
        c.attempt_cap_s = 2.0;
        // The level drops every 2 successful attempts, so a pause after 3 lands mid-block.
        c.attempts_per_level = 2;
        c.probe_every = 4;
        c.scheduled_events = vec![event(0.3, "late"), event(0.1, "early")];
        c
    }

    #[test]
    fn balance_split_with_scheduled_events_equals_unsplit_mid_block() {
        // Pause asked in attempt 3: stops after it, before the probe due after attempt 4.
        assert_balance_split_equals_unsplit(&pause_criterion(), 3, 3);
    }

    #[test]
    fn balance_pause_waits_for_the_due_probe_so_the_cadence_stays_aligned() {
        // Pause asked in attempt 4: its probe still runs, then the segment stops.
        assert_balance_split_equals_unsplit(&pause_criterion(), 4, 4);
    }

    #[test]
    fn balance_adaptive_split_equals_unsplit() {
        let mut c = pause_criterion();
        c.support_mode = AssistMode::Adaptive;
        assert_balance_split_equals_unsplit(&c, 3, 3);
    }
}

#[cfg(test)]
mod walk_tests {
    use super::*;

    const PERIOD: f64 = 0.05;
    const RESET_HEIGHT: f64 = 0.4;

    fn walking(goal: WalkGoal, cap_s: f64) -> BalanceFeetCriterion {
        let mut c = BalanceFeetCriterion::new(
            "torso",
            vec!["foot_a".to_string(), "foot_b".to_string()],
            "head",
            30.0,
            1.0,
            cap_s,
        );
        c.burst_period_s = PERIOD;
        c.walk = Some(goal);
        c
    }

    fn goal(distance_m: Option<f64>) -> WalkGoal {
        WalkGoal {
            distance_m,
            ..WalkGoal::new(0.5)
        }
    }

    fn reading(forward_m: f64, lateral_m: f64) -> WalkReading {
        WalkReading {
            forward_m,
            lateral_m,
            progress_m: 0.0,
        }
    }

    fn sample(seq: u64, contacts: [bool; 2], walk: WalkReading) -> BalanceSample {
        BalanceSample {
            seq,
            seed: 1,
            tilt_deg: 2.0,
            posture: None,
            contacts: contacts.to_vec(),
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
            torso_height_m: Some(RESET_HEIGHT),
            reset_height_m: Some(RESET_HEIGHT),
            assist_slow: None,
            assist_rise: None,
            actuators: BTreeMap::new(),
            actuator_force: BTreeMap::new(),
            stability: None,
            walk: Some(walk),
        }
    }

    /// Feed one burst per entry; returns the record when the attempt finishes.
    fn run(
        c: &BalanceFeetCriterion,
        bursts: impl IntoIterator<Item = ([bool; 2], f64, f64)>,
    ) -> Result<BalanceAttemptRecord, Box<OpenBalanceAttempt>> {
        let mut open = OpenBalanceAttempt::for_attempt(1.0, 0.0);
        for (i, (contacts, forward, lateral)) in bursts.into_iter().enumerate() {
            let s = sample(i as u64 + 1, contacts, reading(forward, lateral));
            match observe_balance_sample(&open, &s, c, 1).expect("observe") {
                BalanceAttemptStep::Continue(next) => open = next,
                BalanceAttemptStep::Finished(record) => return Ok(record),
            }
        }
        Err(Box::new(open))
    }

    const BOTH: [bool; 2] = [true, true];

    #[test]
    fn forward_follows_the_heading_and_lateral_is_to_its_left() {
        let mut goal = WalkGoal::new(0.5);
        goal.forward_heading_deg = Some(90.0);
        let mut run = WalkRun::new(goal.forward_heading_deg.unwrap());
        let first = run.read([2.0, 3.0, 0.4], false);
        assert_eq!((first.forward_m, first.lateral_m), (0.0, 0.0));
        let moved = run.read([1.5, 4.0, 0.4], false);
        assert!((moved.forward_m - 1.0).abs() < 1e-9, "{moved:?}");
        assert!((moved.lateral_m - 0.5).abs() < 1e-9, "{moved:?}");
    }

    #[test]
    fn settling_positions_never_set_the_start() {
        let mut run = WalkRun::new(0.0);
        run.read([5.0, 0.0, 0.4], true);
        let first = run.read([6.0, 0.0, 0.4], false);
        assert_eq!(first.forward_m, 0.0);
    }

    #[test]
    fn rocking_over_planted_feet_gains_no_ground() {
        let mut run = WalkRun::new(0.0);
        run.read([0.0, 0.0, 0.4], false);
        run.read([0.10, 0.0, 0.4], false);
        run.read([0.0, 0.0, 0.4], false);
        run.read([0.10, 0.0, 0.4], false);
        assert!(
            (run.take_progress() - 0.10).abs() < 1e-12,
            "only the first 0.10 m is new"
        );
        run.read([0.12, 0.0, 0.4], false);
        assert!((run.take_progress() - 0.02).abs() < 1e-12);
        assert_eq!(run.take_progress(), 0.0);
    }

    #[test]
    fn heading_is_undefined_until_half_a_torso_height() {
        assert_eq!(walk_heading_error_deg(0.19, 0.0, Some(0.4)), None);
        assert_eq!(walk_heading_error_deg(0.2, 0.0, None), None);
        let straight = walk_heading_error_deg(0.2, 0.0, Some(0.4)).expect("defined");
        assert!(straight.abs() < 1e-9);
        let diagonal = walk_heading_error_deg(1.0, -1.0, Some(0.4)).expect("defined");
        assert!((diagonal - 45.0).abs() < 1e-9);
        let backward = walk_heading_error_deg(-1.0, 0.0, Some(0.4)).expect("defined");
        assert!((backward - 180.0).abs() < 1e-9);
    }

    #[test]
    fn one_foot_up_is_upright_for_a_walk_but_not_for_a_stand() {
        let walk = walking(goal(Some(1.0)), 20.0);
        let mut stand = walk.clone();
        stand.walk = None;
        let one_up = sample(1, [true, false], reading(0.0, 0.0));
        let none_down = sample(1, [false, false], reading(0.0, 0.0));
        assert!(sample_is_upright(&one_up, &walk));
        assert!(!sample_is_upright(&none_down, &walk));
        assert!(!sample_is_upright(&one_up, &stand));
        assert!(
            !support_use_flags(&one_up, &walk, Some(RESET_HEIGHT), 0.5)
                .unwrap()
                .0
        );
        assert!(
            support_use_flags(&none_down, &walk, Some(RESET_HEIGHT), 0.5)
                .unwrap()
                .0
        );
    }

    #[test]
    fn distance_rule_succeeds_at_pace_and_reports_the_walk() {
        // 1 m in 40 bursts of 0.05 s: 0.5 m/s, the target.
        let c = walking(goal(Some(1.0)), 20.0);
        let bursts = (1..=40).map(|i| (BOTH, f64::from(i) * 0.025, 0.0));
        let record = run(&c, bursts).expect("finished");
        assert_eq!(record.outcome, BalanceAttemptOutcome::Success);
        let walk = record.walk.expect("walk stats");
        assert!((walk.forward_m - 1.0).abs() < 1e-9);
        assert!((walk.mean_speed_m_s - 0.5).abs() < 1e-9);
        assert!(walk.speed_error_fraction.abs() < 1e-9);
        assert_eq!(walk.heading_error_deg, Some(0.0));
    }

    #[test]
    fn distance_rule_too_fast_is_not_a_walk() {
        // 1 m in 20 bursts (1 s): 1 m/s is over the 0.625 m/s ceiling of a 0.5 m/s target.
        let c = walking(goal(Some(1.0)), 20.0);
        let record = run(&c, (1..=20).map(|i| (BOTH, f64::from(i) * 0.05, 0.0))).expect("finished");
        assert_eq!(record.outcome, BalanceAttemptOutcome::Truncated);
        assert_eq!(
            record.bursts, 20,
            "the attempt ends when the distance is covered"
        );
        assert!(record.walk.expect("walk stats").mean_speed_m_s > 0.625);
    }

    #[test]
    fn distance_rule_too_slow_is_truncated_not_success() {
        // 1 m reached after 100 bursts (5 s): 0.2 m/s is under the 0.375 m/s floor.
        let c = walking(goal(Some(1.0)), 6.0);
        let bursts = (1..=130).map(|i| (BOTH, (f64::from(i) * 0.01).min(1.0), 0.0));
        let record = run(&c, bursts).expect("finished at the cap");
        assert_eq!(record.outcome, BalanceAttemptOutcome::Truncated);
        assert!(record.walk.unwrap().mean_speed_m_s < 0.375);
    }

    #[test]
    fn heading_outside_tolerance_blocks_success() {
        // Forward 1 m but 1 m sideways: 45 degrees against a 30 degree tolerance.
        let c = walking(goal(Some(1.0)), 3.0);
        let bursts = (1..=70).map(|i| {
            (
                BOTH,
                (f64::from(i) * 0.05).min(1.0),
                (f64::from(i) * 0.05).min(1.0),
            )
        });
        let record = run(&c, bursts).expect("cap");
        assert_eq!(record.outcome, BalanceAttemptOutcome::Truncated);
        let heading = record.walk.unwrap().heading_error_deg.expect("defined");
        assert!((heading - 45.0).abs() < 1e-6);
    }

    #[test]
    fn time_rule_judges_the_last_hold_window_so_a_standing_start_can_pass() {
        // hold_s 1.0 = 20 bursts. Stand 40 bursts, then walk 0.04 m per burst (0.8 m/s).
        let c = walking(goal(None), 20.0);
        let standing = (0..40).map(|_| (BOTH, 0.0, 0.0));
        let walking_on = (1..=40).map(|i| (BOTH, f64::from(i) * 0.04, 0.0));
        let record = run(&c, standing.chain(walking_on)).expect("finished");
        assert_eq!(record.outcome, BalanceAttemptOutcome::Success);
        let walk = record.walk.unwrap();
        // It passes as soon as the last second averages the slowest passing speed (0.375 m/s),
        // before the window is fully walking.
        let window = walk.window_speed_m_s.unwrap();
        assert!((0.375 - 1e-9..0.8).contains(&window), "{walk:?}");
        assert!(
            walk.mean_speed_m_s < window,
            "the whole attempt averages the standing start"
        );
    }

    #[test]
    fn time_rule_running_ends_the_attempt_and_stopping_does_not_rescue_it() {
        // 0.8 m/s against a 0.625 m/s ceiling: the first full 1 s window is already a run.
        let c = walking(goal(None), 20.0);
        let run_then_stop = (1..=30)
            .map(|i| (BOTH, f64::from(i) * 0.04, 0.0))
            .chain((0..60).map(|_| (BOTH, 1.2, 0.0)));
        let record = run(&c, run_then_stop).expect("finished");
        assert_eq!(record.outcome, BalanceAttemptOutcome::Truncated);
        assert_eq!(record.bursts, 20, "ends at the first judged window");
    }

    #[test]
    fn time_rule_standing_still_never_succeeds() {
        let c = walking(goal(None), 5.0);
        let record = run(&c, (0..200).map(|_| (BOTH, 0.0, 0.0))).expect("cap");
        assert_eq!(record.outcome, BalanceAttemptOutcome::Truncated);
    }

    #[test]
    fn a_fall_ends_a_walking_attempt_as_fell() {
        let mut c = walking(goal(Some(5.0)), 20.0);
        c.fall_confirm_s = PERIOD;
        let mut open = OpenBalanceAttempt::for_attempt(1.0, 0.0);
        let mut fallen = sample(1, BOTH, reading(0.1, 0.0));
        fallen.tilt_deg = 80.0;
        match observe_balance_sample(&open, &fallen, &c, 1).expect("observe") {
            BalanceAttemptStep::Finished(record) => {
                assert_eq!(record.outcome, BalanceAttemptOutcome::Fell);
                assert!(record.walk.is_some());
            }
            BalanceAttemptStep::Continue(next) => open = next,
        }
        let _ = open;
    }

    #[test]
    fn adaptive_walk_at_the_cap_is_truncated_even_when_it_stood() {
        let mut c = walking(goal(Some(5.0)), 1.0);
        c.support_mode = AssistMode::Adaptive;
        let record = run(&c, (0..40).map(|_| (BOTH, 0.0, 0.0))).expect("cap");
        assert_eq!(record.outcome, BalanceAttemptOutcome::Truncated);
        let mut stand = c.clone();
        stand.walk = None;
        let record = run_stand(&stand);
        assert_eq!(
            record.outcome,
            BalanceAttemptOutcome::Success,
            "standing still is a stand success"
        );
    }

    fn run_stand(c: &BalanceFeetCriterion) -> BalanceAttemptRecord {
        let mut open = OpenBalanceAttempt::for_attempt(1.0, 0.0);
        for i in 0..40u64 {
            let mut s = sample(i + 1, BOTH, reading(0.0, 0.0));
            s.walk = None;
            match observe_balance_sample(&open, &s, c, 1).expect("observe") {
                BalanceAttemptStep::Continue(next) => open = next,
                BalanceAttemptStep::Finished(record) => return record,
            }
        }
        panic!("no finish")
    }

    #[test]
    fn touchdowns_and_alternation_are_counted() {
        let c = walking(goal(Some(50.0)), 20.0);
        // Baseline both down, then A up/down, B up/down, A up/down: three lone touchdowns,
        // the second and third by the other foot.
        let pattern = [
            [true, true],
            [false, true],
            [true, true],
            [true, false],
            [true, true],
            [false, true],
            [true, true],
        ];
        let open = run(
            &c,
            pattern
                .iter()
                .enumerate()
                .map(|(i, p)| (*p, i as f64 * 0.01, 0.0)),
        )
        .expect_err("still open");
        assert_eq!(open.walk.steps, 3);
        assert_eq!(open.walk.touchdown_pairs, 2);
        assert_eq!(open.walk.alternations, 2);
    }

    #[test]
    fn hopping_on_one_foot_does_not_alternate() {
        let c = walking(goal(Some(50.0)), 20.0);
        let pattern = [
            [true, true],
            [false, true],
            [true, true],
            [false, true],
            [true, true],
        ];
        let open = run(
            &c,
            pattern
                .iter()
                .enumerate()
                .map(|(i, p)| (*p, i as f64 * 0.01, 0.0)),
        )
        .expect_err("still open");
        assert_eq!(open.walk.touchdown_pairs, 1);
        assert_eq!(open.walk.alternations, 0);
    }

    #[test]
    fn alternation_needs_two_contact_bodies() {
        let mut c = walking(goal(Some(1.0)), 20.0);
        c.contact_bodies = vec!["foot_a".to_string()];
        let acc = WalkAccum {
            steps: 4,
            touchdown_pairs: 3,
            alternations: 3,
            ..WalkAccum::default()
        };
        let stats = walk_stats(
            c.walk.as_ref().unwrap(),
            &c,
            &acc,
            20,
            0,
            Some(RESET_HEIGHT),
        );
        assert_eq!(stats.alternation_fraction, None);
        assert_eq!(stats.steps, 4);
    }

    #[test]
    fn feasibility_names_the_numbers() {
        // Time rule: slowest passing speed 0.375 m/s * 1 s = 0.375 m against 0.2 m needed: fine.
        assert!(goal(None).check_feasible(1.0, 0.4).is_ok());
        // 0.375 m/s * 0.1 s = 0.0375 m < 0.2 m.
        let err = goal(None).check_feasible(0.1, 0.4).unwrap_err().to_string();
        assert!(err.contains("0.0375") && err.contains("0.2000"), "{err}");
        // Distance rule below half a torso height.
        let err = goal(Some(0.1))
            .check_feasible(1.0, 0.4)
            .unwrap_err()
            .to_string();
        assert!(err.contains("walk distance_m"), "{err}");
        assert!(goal(Some(0.2)).check_feasible(1.0, 0.4).is_ok());
    }

    #[test]
    fn walk_goal_validation_rejects_bad_values() {
        let ok = walking(goal(Some(1.0)), 20.0);
        assert!(ok.validate().is_ok());
        for bad in [
            WalkGoal::new(0.0),
            WalkGoal::new(f64::NAN),
            WalkGoal {
                speed_tolerance: 1.0,
                ..WalkGoal::new(0.5)
            },
            WalkGoal {
                heading_tolerance_deg: 0.0,
                ..WalkGoal::new(0.5)
            },
            WalkGoal {
                heading_tolerance_deg: 91.0,
                ..WalkGoal::new(0.5)
            },
            WalkGoal {
                forward_heading_deg: Some(f64::INFINITY),
                ..WalkGoal::new(0.5)
            },
            WalkGoal {
                distance_m: Some(-1.0),
                ..WalkGoal::new(0.5)
            },
        ] {
            let mut c = ok.clone();
            c.walk = Some(bad);
            assert!(c.validate().is_err());
        }
    }

    #[test]
    fn a_standing_criterion_serialises_without_walk_keys_effect() {
        let stand = BalanceFeetCriterion::new("torso", vec!["foot".into()], "top", 30.0, 3.0, 10.0);
        let json = serde_json::to_value(&stand).unwrap();
        let back: BalanceFeetCriterion = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(back, stand);
        let mut without = json;
        without.as_object_mut().unwrap().remove("walk");
        let parsed: BalanceFeetCriterion = serde_json::from_value(without).unwrap();
        assert!(parsed.walk.is_none());
        assert!(parsed.contacts_satisfied(&[true]) && !parsed.contacts_satisfied(&[true, false]));
    }

    #[test]
    fn a_recovery_payout_and_walk_progress_pay_the_larger_of_the_two() {
        let c = walking(goal(Some(5.0)), 20.0);
        let burst = |seq: u64, need: f64, progress: f64| {
            let mut s = sample(
                seq,
                BOTH,
                WalkReading {
                    progress_m: progress,
                    ..reading(0.0, 0.0)
                },
            );
            s.support_need = Some(need);
            s.support_level = Some(0.0);
            s
        };
        let pleasure = |signals: Vec<RewardSignal>| {
            signals
                .iter()
                .filter(|r| r.channel == AffectChannel::Pleasure)
                .map(|r| r.magnitude)
                .sum::<f64>()
        };
        // Credit worth 4 * 0.05 = 0.2 against 0.25 of walk progress: the walk term wins.
        let mut reward = BalanceContributionReward::new();
        reward.on_sample(&burst(1, 0.30, 0.0), None, &c);
        let out = reward.on_sample(&burst(2, 0.25, 0.0125), None, &c);
        assert!((pleasure(out) - 0.25).abs() < 1e-9);
        // Credit worth 0.8 against 0.25: the recovery wins.
        let mut reward = BalanceContributionReward::new();
        reward.on_sample(&burst(1, 0.50, 0.0), None, &c);
        let out = reward.on_sample(&burst(2, 0.30, 0.0125), None, &c);
        assert!((pleasure(out) - 0.8).abs() < 1e-9);
    }

    #[test]
    fn contribution_pays_progress_scaled_by_the_target_and_by_support() {
        let c = walking(goal(Some(5.0)), 20.0);
        let mut reward = BalanceContributionReward::new();
        let mut s = sample(1, BOTH, reading(0.0, 0.0));
        s.support_need = Some(0.2);
        reward.on_sample(&s, None, &c);
        // The target covers 0.5 m/s * 0.05 s = 0.025 m per burst.
        let pay = |progress: f64, level: f64, reward: &mut BalanceContributionReward| {
            let mut s = sample(
                2,
                BOTH,
                WalkReading {
                    progress_m: progress,
                    ..reading(0.0, 0.0)
                },
            );
            s.support_need = Some(0.2);
            s.support_level = Some(level);
            reward
                .on_sample(&s, None, &c)
                .iter()
                .filter(|r| r.channel == AffectChannel::Pleasure)
                .map(|r| r.magnitude)
                .sum::<f64>()
        };
        assert!((pay(0.025, 0.0, &mut reward) - WALK_PROGRESS_PLEASURE).abs() < 1e-9);
        assert!((pay(0.0125, 0.0, &mut reward) - 0.25).abs() < 1e-9);
        assert!(
            (pay(0.5, 0.0, &mut reward) - WALK_PROGRESS_PLEASURE).abs() < 1e-9,
            "capped at the target pace"
        );
        assert!(
            (pay(0.025, 0.6, &mut reward) - 0.2).abs() < 1e-9,
            "held robot is discounted by 1 - a"
        );
        assert_eq!(
            pay(0.0, 0.0, &mut reward),
            0.0,
            "no ground gained pays nothing"
        );
    }

    #[test]
    fn contribution_pays_no_progress_off_its_feet() {
        let c = walking(goal(Some(5.0)), 20.0);
        let mut reward = BalanceContributionReward::new();
        let mut s = sample(
            1,
            [false, false],
            WalkReading {
                progress_m: 0.025,
                ..reading(0.0, 0.0)
            },
        );
        s.support_need = Some(0.2);
        assert!(reward.on_sample(&s, None, &c).is_empty());
    }

    #[test]
    fn walk_phase_metrics_appear_only_with_walk_stats() {
        let c = walking(goal(Some(1.0)), 20.0);
        let record =
            run(&c, (1..=40).map(|i| (BOTH, f64::from(i) * 0.025, 0.0))).expect("finished");
        let phase = BalancePhaseReport::from_attempts(vec![record]).expect("phase");
        let metrics = balance_phase_metrics(&phase);
        assert!((metrics["mean_forward_m"] - 1.0).abs() < 1e-9);
        assert!((metrics["mean_speed_m_s"] - 0.5).abs() < 1e-9);
        assert!(metrics.contains_key("mean_heading_error_deg"));
        assert_eq!(metrics["success_rate"], 1.0);
        let mut stand = c.clone();
        stand.walk = None;
        let stand_record = run_stand(&stand);
        let phase = BalancePhaseReport::from_attempts(vec![stand_record]).expect("phase");
        assert!(!balance_phase_metrics(&phase).contains_key("mean_forward_m"));
    }

    #[test]
    fn contribution_policy_id_is_versioned_for_walk() {
        assert_eq!(
            BalanceRewardPolicy::Contribution.policy_id_for(true),
            "reward.balance-contribution.v5"
        );
        assert_eq!(
            BalanceRewardPolicy::Contribution.policy_id_for(false),
            BalanceContributionReward::policy_id()
        );
    }

    #[test]
    fn a_walking_sample_without_a_position_is_an_error() {
        let c = walking(goal(Some(1.0)), 20.0);
        let mut s = sample(1, BOTH, reading(0.0, 0.0));
        s.walk = None;
        let open = OpenBalanceAttempt::for_attempt(1.0, 0.0);
        assert!(observe_balance_sample(&open, &s, &c, 1).is_err());
    }
}

#[cfg(test)]
mod live_reading_tests {
    use super::*;

    fn setup() -> BalanceFeetCriterion {
        BalanceFeetCriterion::new(
            "torso",
            vec!["foot_a".to_string(), "foot_b".to_string()],
            "head",
            15.0,
            1.0,
            10.0,
        )
    }

    /// A reply whose head stands `lean_deg` off vertical toward `heading` above the feet
    /// midpoint, read with the torso heading `yaw`.
    fn reply(lean_deg: f64, heading: f64, yaw: f64, steps: u64) -> Measurement {
        let t = lean_deg.to_radians();
        let top = [
            t.sin() * heading.cos(),
            t.sin() * heading.sin(),
            t.cos(),
        ];
        serde_json::from_value(serde_json::json!({
            "sim_time_s": 0.0,
            "physics_steps": steps,
            "episode_id": 3,
            "bodies": {
                "torso": {"tilt_rad": 0.1, "heading": {"yaw_rad": yaw}},
                "foot_a": {"tilt_rad": 0.0, "com": [0.1, 0.0, 0.0]},
                "foot_b": {"tilt_rad": 0.0, "com": [-0.1, 0.0, 0.0]},
                "head": {"tilt_rad": 0.0, "com": top},
            },
            "contacts": [{"touching": true}, {"touching": true}],
            "link": {"state": "linked", "epoch": 1},
        }))
        .expect("reply")
    }

    /// The runner's scored tilt for a reply: sample, then score against the latched line.
    fn runner_scored(reply: &Measurement, reset: Option<&LeanReference>) -> f64 {
        let c = setup();
        let posture = measure_posture(reply, &c).expect("posture");
        let yaw = reply.bodies["torso"].heading.as_ref().and_then(|h| h.yaw_rad);
        let mut s = BalanceSample {
            seq: 1,
            seed: 1,
            tilt_deg: 0.0,
            posture: Some(posture.sample(yaw)),
            contacts: vec![true, true],
            terminal: None,
            support_level: None,
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
            actuators: Default::default(),
            actuator_force: Default::default(),
            stability: None,
            walk: None,
        };
        s.score_posture_from(reset);
        s.scored_tilt_deg()
    }

    #[test]
    fn live_reading_equals_the_runner_scored_tilt() {
        let c = setup();
        let reset = latch_reference(&reply(18.0, 0.4, 0.0, 10), &c).expect("latch");
        for (lean, heading, yaw) in [(18.0, 0.4, 0.0), (34.0, 0.4, 0.0), (10.0, 1.2, 0.3)] {
            let r = reply(lean, heading, yaw, 20);
            let live = live_balance_reading(&r, &c, Some(&reset));
            let scored = live.scored_tilt_deg.expect("scored");
            assert!((scored - runner_scored(&r, Some(&reset))).abs() <= 1e-9);
            assert!(live.reference_latched);
        }
    }

    #[test]
    fn a_non_vertical_reset_pose_reads_zero() {
        let c = setup();
        let r = reply(18.0, 0.4, 0.0, 10);
        let reset = latch_reference(&r, &c).expect("latch");
        let live = live_balance_reading(&r, &c, Some(&reset));
        assert!(live.scored_tilt_deg.expect("scored") < 1e-9);
        assert!((live.absolute_tilt_deg.expect("abs") - 18.0).abs() < 1e-9);
    }

    #[test]
    fn a_heading_turn_alone_adds_no_tilt() {
        let c = setup();
        let reset = latch_reference(&reply(18.0, 0.0, 0.0, 10), &c).expect("latch");
        let turned = reply(18.0, 0.9, 0.9, 20);
        let live = live_balance_reading(&turned, &c, Some(&reset));
        assert!(live.scored_tilt_deg.expect("scored") < 1e-9);
    }

    #[test]
    fn without_a_reference_the_scored_tilt_is_the_absolute_one() {
        let c = setup();
        let live = live_balance_reading(&reply(18.0, 0.4, 0.0, 10), &c, None);
        assert!(!live.reference_latched);
        assert_eq!(live.scored_tilt_deg, live.absolute_tilt_deg);
        assert!((live.scored_tilt_deg.expect("scored") - 18.0).abs() < 1e-9);
    }

    #[test]
    fn a_partial_setup_returns_its_rows_without_error() {
        let mut c = setup();
        c.posture_top_body = String::new();
        let r = reply(18.0, 0.4, 0.0, 10);
        let live = live_balance_reading(&r, &c, None);
        assert_eq!(live.scored_tilt_deg, None);
        assert_eq!(live.absolute_tilt_deg, None);
        assert!(live.torso_tilt_deg.is_some());
        assert_eq!(live.contacts.len(), 2);
        assert_eq!(live.contacts[0].touching, Some(true));
        assert!(live.all_contacts_touch);
        assert!(latch_reference(&r, &c).is_none());
        // No contact bodies at all: still no error.
        c.contact_bodies.clear();
        let live = live_balance_reading(&r, &c, None);
        assert!(live.contacts.is_empty() && live.torso_tilt_deg.is_some());
    }
}
