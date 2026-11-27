//! Grasp-and-hold co-agent loop (Grab and hold, and Lift).
//!
//! The embodiment controller owns the body and the scene object. This module samples nothing
//! in the simulator for the brain: it asks the controller to begin an attempt (the object is
//! placed once on its support, then free), steps one burst so the brain can read its own
//! sensors, and judges only behavior: is the object grasped, or held up off its support. During
//! training it sends pain or pleasure for ground gained (a new best distance, first contact,
//! a new best height) and pain when the object is dropped or knocked off. Bursts are counted
//! from the controller's physics steps, not from polls.
//!
//! The attempt loop, phase loop and HTTP client are the shared ones in [`crate::attempt_loop`].

use std::collections::BTreeMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::attempt_loop::{
    run_attempt, run_phase_loop, seconds_to_whole_bursts, AttemptData, AttemptFlow, AttemptRules,
    AttemptTerminal, CoachWrite, HttpControl, PhaseSpec, ATTEMPTS_TABLE, STEPS_TABLE,
};
use crate::binding::reward::{AffectChannel, RewardSignal};
use crate::contracts::{
    BackendFingerprint, BackendKind, ConnectomeHash, ContentHash, DatasetAssetId,
    EvaluationProtocolVersion, MetricScope, PluginId, PluginRef, RunEvent, RunEventKind, RunId,
    Scorecard, ScorecardId, ScorecardStatus, ScorecardVisibility, SplitId,
};
use crate::control::{CancelToken, RunEventSink};
use crate::error::TrainerError;
use crate::metrics::episodic_control::EpisodicControlMetricPack;
use crate::resume::{ResumeExport, ResumeFrom};
use crate::run_data::{name_of, ColumnSpec, RunDataSink, RunDataValue, TableSpec};

/// What counts as success.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraspRule {
    /// At least `min_contacts` grasping bodies touch the object for the hold time. The object
    /// may still rest on its support.
    GrabHold,
    /// The object is at least the lift height above its resting height, a grasping body
    /// touches it and nothing else does, for the hold time.
    Lift,
}

fn default_min_contacts() -> u32 {
    1
}

fn default_settle_s() -> f64 {
    0.5
}

/// Which of the object's three box extents points up while it rests, ranked by length. The
/// controller's wire names are `smallest`, `middle` and `largest`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpExtent {
    /// The shortest extent is up: the largest face is down.
    Smallest,
    /// The middle extent is up: the middle face is down.
    Middle,
    /// The longest extent is up: the smallest face is down.
    Largest,
}

impl UpExtent {
    /// Wire name the controller uses.
    pub fn wire_name(self) -> &'static str {
        match self {
            Self::Smallest => "smallest",
            Self::Middle => "middle",
            Self::Largest => "largest",
        }
    }

    /// Recorded side name: the face the object rests on.
    pub fn side_name(self) -> &'static str {
        match self {
            Self::Smallest => "flat",
            Self::Middle => "edge",
            Self::Largest => "end",
        }
    }
}

/// Which face the object rests on at the start of each attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RestingSide {
    /// Largest face down.
    Flat,
    /// Middle face down.
    Edge,
    /// Smallest face down.
    End,
    /// One side per attempt, in turn: the attempt seed modulo three (flat, edge, end).
    #[default]
    Mixed,
}

impl RestingSide {
    /// The side asked of the controller for the attempt `seed`. A pure function of the seed.
    pub fn requested_for(self, seed: u64) -> UpExtent {
        match self {
            Self::Flat => UpExtent::Smallest,
            Self::Edge => UpExtent::Middle,
            Self::End => UpExtent::Largest,
            Self::Mixed => match seed % 3 {
                0 => UpExtent::Smallest,
                1 => UpExtent::Middle,
                _ => UpExtent::Largest,
            },
        }
    }
}

/// The object's three box extents, ranked, millimeters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ObjectExtents {
    /// Shortest extent.
    pub smallest: f64,
    /// Middle extent.
    pub middle: f64,
    /// Longest extent.
    pub largest: f64,
}

impl ObjectExtents {
    /// Length of the extent that points up.
    pub fn of(&self, up: UpExtent) -> f64 {
        match up {
            UpExtent::Smallest => self.smallest,
            UpExtent::Middle => self.middle,
            UpExtent::Largest => self.largest,
        }
    }
}

/// What the Trainer asks of each placement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlacementRequest {
    /// How far from its scene position the object may start, millimeters.
    pub spread_mm: f64,
    /// Which extent points up.
    pub up_extent: UpExtent,
}

/// Operator criterion for one grasp session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraspHoldCriterion {
    /// Success rule.
    pub rule: GraspRule,
    /// Applied scene-object id. The controller places it once on its support, then leaves it free.
    pub target_object_id: String,
    /// Model bodies that may hold the object (fingers, jaws, finger links). Never guessed.
    pub grasping_bodies: Vec<String>,
    /// Grasping bodies that must touch for a grasp (Grab and hold). At least 1, at most
    /// `grasping_bodies.len()`.
    #[serde(default = "default_min_contacts")]
    pub min_contacts: u32,
    /// Seconds the grasp (or hold up) must last.
    pub hold_s: f64,
    /// Bursts after which an attempt that has not succeeded is truncated (settle excluded).
    pub attempt_burst_cap: u32,
    /// Seconds the object may settle after placement. Counts toward neither hold nor cap.
    #[serde(default = "default_settle_s")]
    pub settle_s: f64,
    /// Seconds a contact may drop out and still count as unbroken. `None` is one burst.
    #[serde(default)]
    pub contact_grace_s: Option<f64>,
    /// Lift height, millimeters (Lift only). `None` is one times the object's smallest
    /// dimension, read from the model once per session.
    #[serde(default)]
    pub lift_height_mm: Option<f64>,
    /// How far from its scene position the object may start each attempt, millimeters. `None`
    /// is one times the object's largest dimension, read from the controller once per session.
    #[serde(default)]
    pub placement_spread_mm: Option<f64>,
    /// Which face the object rests on each attempt.
    #[serde(default)]
    pub resting_side: RestingSide,
}

/// The criterion's durations as whole bursts at one FEAGI burst frequency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraspBursts {
    /// Bursts the grasp must last.
    pub hold: u32,
    /// Bursts of the settle window.
    pub settle: u32,
    /// Bursts a contact may drop out for.
    pub grace: u32,
}

impl GraspHoldCriterion {
    /// Rejects a criterion that cannot produce a success.
    pub fn validate(&self) -> Result<(), TrainerError> {
        if self.target_object_id.trim().is_empty() {
            return Err(TrainerError::Config(
                "target_object_id must be non-empty".to_string(),
            ));
        }
        if self.grasping_bodies.is_empty()
            || self
                .grasping_bodies
                .iter()
                .any(|name| name.trim().is_empty())
        {
            return Err(TrainerError::Config(
                "grasping_bodies must name at least one body".to_string(),
            ));
        }
        if self.min_contacts == 0 || self.min_contacts as usize > self.grasping_bodies.len() {
            return Err(TrainerError::Config(
                "min_contacts must be at least 1 and at most the number of grasping bodies"
                    .to_string(),
            ));
        }
        if self.attempt_burst_cap == 0 {
            return Err(TrainerError::Config(
                "attempt_burst_cap must be > 0".to_string(),
            ));
        }
        if !self.settle_s.is_finite() || self.settle_s < 0.0 {
            return Err(TrainerError::Config(
                "settle_s must be a finite value >= 0".to_string(),
            ));
        }
        if let Some(height) = self.lift_height_mm {
            if !height.is_finite() || height <= 0.0 {
                return Err(TrainerError::Config(
                    "lift_height_mm must be a finite value > 0".to_string(),
                ));
            }
        }
        if let Some(spread) = self.placement_spread_mm {
            if !spread.is_finite() || spread < 0.0 {
                return Err(TrainerError::Config(
                    "placement_spread_mm must be a finite value >= 0".to_string(),
                ));
            }
        }
        Ok(())
    }

    /// Convert the durations to whole bursts at `frequency_hz` and check them against the cap.
    pub fn burst_counts(&self, frequency_hz: f64) -> Result<GraspBursts, TrainerError> {
        self.validate()?;
        let hold = seconds_to_whole_bursts("hold", self.hold_s, frequency_hz)?;
        let settle = if self.settle_s == 0.0 {
            0
        } else {
            seconds_to_whole_bursts("settle", self.settle_s, frequency_hz)?
        };
        let grace = match self.contact_grace_s {
            None => 1,
            Some(seconds) => seconds_to_whole_bursts("contact grace", seconds, frequency_hz)?,
        };
        if hold > self.attempt_burst_cap {
            return Err(TrainerError::Config(
                "hold must fit inside attempt_burst_cap".to_string(),
            ));
        }
        Ok(GraspBursts {
            hold,
            settle,
            grace,
        })
    }
}

/// Why one attempt ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraspAttemptOutcome {
    /// The grasp (or hold up) lasted the hold time.
    Success,
    /// The burst cap was reached without a hold.
    Truncated,
    /// `_death` fired.
    Death,
    /// The body left the safety box.
    Fence,
    /// A misc reset interrupted the attempt.
    Interrupted,
    /// The object was held up, then touched something but the grasping bodies, beyond the grace.
    Dropped,
    /// The object fell below its resting height by more than its own smallest dimension.
    KnockedOff,
}

/// One finished attempt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraspAttemptRecord {
    /// Seed the controller used to place the object.
    pub seed: u64,
    /// How the attempt ended.
    pub outcome: GraspAttemptOutcome,
    /// Bursts observed after the settle window, including the terminal burst.
    pub bursts: u32,
    /// Nearest grasping body to the object at the terminal burst, millimeters.
    pub final_distance_mm: f64,
    /// Highest the object got above its resting height, millimeters.
    pub max_height_mm: f64,
    /// Side the object was resting on at the end of the settle window (what it landed on).
    /// `None` when the attempt ended before that or the controller sent no reading.
    #[serde(default)]
    pub resting_side: Option<UpExtent>,
    /// Side the Trainer asked for.
    #[serde(default)]
    pub requested_side: Option<UpExtent>,
    /// Where the controller placed the object, millimeters (x, y).
    #[serde(default)]
    pub placed_mm: Option<[f64; 2]>,
}

/// Placement returned by the controller.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraspPlacement {
    /// Seed that produced this placement.
    pub seed: u64,
    /// Scene object that was placed.
    pub target_object_id: String,
    /// Model body of that object, used for every reading.
    pub object_body_name: String,
    /// Body the object was placed on.
    #[serde(default)]
    pub support_body_name: Option<String>,
    /// Where the object was placed, millimeters (x, y, z).
    #[serde(default)]
    pub target_mm: Option<[f64; 3]>,
    /// The object's ranked box extents, millimeters.
    #[serde(default)]
    pub extents_mm: Option<ObjectExtents>,
}

/// One burst's reading from the controller.
#[derive(Debug, Clone, PartialEq)]
pub struct GraspSample {
    /// Controller physics-step counter. Bursts are counted from it.
    pub seq: u64,
    /// Attempt seed.
    pub seed: u64,
    /// Object centre of mass, millimeters (z up).
    pub object_mm: [f64; 3],
    /// Centre of mass of each grasping body, millimeters.
    pub grasping_mm: Vec<[f64; 3]>,
    /// Grasping bodies touching the object.
    pub grasp_contacts: u32,
    /// The object touches a body that is neither a grasping body nor the world.
    pub other_contact: bool,
    /// The object touches the world (its support, the floor).
    pub world_contact: bool,
    /// The watched body is outside the safety box.
    pub fence_violated: bool,
    /// Latched end cause. Absent while the attempt is still open.
    pub terminal: Option<AttemptTerminal>,
    /// The extent of the object that points up now. `None` when the controller sent none.
    pub up_extent: Option<UpExtent>,
}

impl GraspSample {
    /// Distance from the object to the nearest grasping body, millimeters.
    pub fn nearest_distance_mm(&self) -> Result<f64, TrainerError> {
        let mut best: Option<f64> = None;
        for body in &self.grasping_mm {
            let d = ((body[0] - self.object_mm[0]).powi(2)
                + (body[1] - self.object_mm[1]).powi(2)
                + (body[2] - self.object_mm[2]).powi(2))
            .sqrt();
            if !d.is_finite() {
                return Err(TrainerError::Runtime(
                    "grasp sample positions must be finite".to_string(),
                ));
            }
            best = Some(best.map_or(d, |b: f64| b.min(d)));
        }
        best.ok_or_else(|| {
            TrainerError::Runtime("grasp sample has no grasping body positions".to_string())
        })
    }
}

/// Numbers the rule needs besides the criterion, fixed for one session.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GraspParams {
    /// Durations in bursts.
    pub bursts: GraspBursts,
    /// Physics steps per burst: `burst_period_s / timestep_s` (not necessarily whole).
    pub steps_per_burst: f64,
    /// Height above the resting height that counts as held up, millimeters (Lift).
    pub lift_height_mm: f64,
    /// The object's smallest dimension, millimeters.
    pub min_extent_mm: f64,
    /// The object's ranked extents, when the controller reported them.
    pub extents_mm: Option<ObjectExtents>,
    /// Side requested for this attempt.
    pub requested_side: Option<UpExtent>,
    /// Where the object was placed, millimeters (x, y).
    pub placed_mm: Option<[f64; 2]>,
}

/// In-progress attempt counters.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenGraspAttempt {
    /// Bursts observed after the settle window.
    pub bursts: u32,
    /// Bursts of the settle window used so far.
    pub settle_bursts: u32,
    /// Latest controller physics-step counter.
    pub last_seq: u64,
    /// Physics step of the last counted burst. `None` before the first sample.
    pub last_burst_seq: Option<f64>,
    /// Consecutive bursts of grasp (or hold up), grace included.
    pub holding_bursts: u32,
    /// Consecutive bursts the grasp condition was false.
    pub lost_bursts: u32,
    /// Consecutive bursts the object touched something else with no grasping body on it.
    pub drop_bursts: u32,
    /// Object height at the end of the settle window, millimeters. `None` while settling.
    pub rest_z_mm: Option<f64>,
    /// Side the object rests on at the end of the settle window.
    pub resting_side: Option<UpExtent>,
    /// The object has been held up (Lift).
    pub was_held_up: bool,
    /// Nearest grasping body so far, millimeters.
    pub best_distance_mm: Option<f64>,
    /// A grasping body has touched the object.
    pub touched: bool,
    /// Highest height above rest so far, millimeters.
    pub best_height_mm: f64,
    /// Latest nearest distance, millimeters.
    pub last_distance_mm: f64,
}

impl OpenGraspAttempt {
    /// An attempt that has not observed a sample yet.
    pub fn fresh() -> Self {
        Self {
            bursts: 0,
            settle_bursts: 0,
            last_seq: 0,
            last_burst_seq: None,
            holding_bursts: 0,
            lost_bursts: 0,
            drop_bursts: 0,
            rest_z_mm: None,
            resting_side: None,
            was_held_up: false,
            best_distance_mm: None,
            touched: false,
            best_height_mm: 0.0,
            last_distance_mm: 0.0,
        }
    }
}

/// Result of folding one sample into an open attempt.
pub type GraspStep = AttemptFlow<OpenGraspAttempt, GraspAttemptRecord>;

/// Height below which an object counts as knocked off: the lowest resting centre
/// (`rest - (up_extent - min_extent) / 2`) minus one smallest dimension.
pub fn knocked_off_below(rest_z_mm: f64, up_extent_mm: f64, min_extent_mm: f64) -> f64 {
    rest_z_mm - (up_extent_mm - min_extent_mm) / 2.0 - min_extent_mm
}

fn pleasure(magnitude: f64) -> RewardSignal {
    RewardSignal {
        channel: AffectChannel::Pleasure,
        magnitude,
    }
}

fn pain(magnitude: f64) -> RewardSignal {
    RewardSignal {
        channel: AffectChannel::Pain,
        magnitude,
    }
}

/// Fold one controller sample into the attempt.
///
/// `sample.seq` is the controller's physics-step counter. With N steps per burst
/// (`steps_per_burst`, not necessarily whole), `k = floor((seq - last_burst_seq) / N)` new
/// bursts have elapsed since the last counted burst; the first sample counts as one burst. A
/// sample with `k = 0` changes nothing but the sequence. The first bursts fill the settle
/// window, which counts toward neither the hold nor the cap; the object's height at its end
/// is the resting height. Terminal and fence samples are always processed.
///
/// The signals are the train-phase reward (reward B) and are empty when `train_affect` is
/// false: pleasure for a new best distance (the fractional improvement), pleasure 1.0 on first
/// contact, pleasure for a new best height (Lift; gain over the lift height, capped at 1) and
/// pain 1.0 on Dropped or KnockedOff. A repeat of a level already reached pays nothing.
pub fn observe_grasp_sample(
    open: &OpenGraspAttempt,
    sample: &GraspSample,
    criterion: &GraspHoldCriterion,
    params: &GraspParams,
    seed: u64,
    train_affect: bool,
) -> Result<(GraspStep, Vec<RewardSignal>), TrainerError> {
    if sample.seed != seed {
        return Err(TrainerError::Runtime(format!(
            "grasp sample seed {} does not match attempt seed {seed}",
            sample.seed
        )));
    }
    if sample.object_mm.iter().any(|v| !v.is_finite()) {
        return Err(TrainerError::Runtime(
            "grasp object position must be finite".to_string(),
        ));
    }
    if !params.steps_per_burst.is_finite() || params.steps_per_burst <= 0.0 {
        return Err(TrainerError::Config(
            "steps_per_burst must be a finite value > 0".to_string(),
        ));
    }
    let distance = sample.nearest_distance_mm()?;
    let z = sample.object_mm[2];
    let lift = criterion.rule == GraspRule::Lift;

    // Bursts elapsed since the last counted one.
    let (k, last_burst_seq) = match open.last_burst_seq {
        None => (1u32, sample.seq as f64),
        Some(last) => {
            let whole = ((sample.seq as f64 - last) / params.steps_per_burst + 1e-9).floor();
            let k = if whole >= 1.0 {
                whole.min(f64::from(u32::MAX)) as u32
            } else {
                0
            };
            (k, last + f64::from(k) * params.steps_per_burst)
        }
    };

    // The settle window takes the first bursts; the rest count.
    let mut settle_bursts = open.settle_bursts;
    let mut rest_z_mm = open.rest_z_mm;
    let mut resting_side = open.resting_side;
    let mut k_main = k;
    if rest_z_mm.is_none() {
        let remaining = params.bursts.settle.saturating_sub(settle_bursts);
        if k >= remaining {
            rest_z_mm = Some(z);
            resting_side = sample.up_extent;
            k_main = k - remaining;
            settle_bursts = params.bursts.settle;
        } else {
            settle_bursts += k;
            k_main = 0;
        }
    }
    let bursts = open.bursts.saturating_add(k_main);
    let record = |outcome: GraspAttemptOutcome, max_height_mm: f64| GraspAttemptRecord {
        seed,
        outcome,
        bursts,
        final_distance_mm: distance,
        max_height_mm,
        resting_side,
        requested_side: params.requested_side,
        placed_mm: params.placed_mm,
    };
    let height_mm = rest_z_mm.map_or(0.0, |rest| z - rest);
    let max_height_mm = open.best_height_mm.max(height_mm);

    if let Some(terminal) = sample.terminal {
        let outcome = match terminal {
            AttemptTerminal::Death => GraspAttemptOutcome::Death,
            AttemptTerminal::Fence => GraspAttemptOutcome::Fence,
            AttemptTerminal::Interrupted => GraspAttemptOutcome::Interrupted,
        };
        return Ok((
            AttemptFlow::Finished(record(outcome, max_height_mm)),
            Vec::new(),
        ));
    }
    if sample.fence_violated {
        return Ok((
            AttemptFlow::Finished(record(GraspAttemptOutcome::Fence, max_height_mm)),
            Vec::new(),
        ));
    }

    let mut next = OpenGraspAttempt {
        bursts,
        settle_bursts,
        last_seq: sample.seq,
        last_burst_seq: Some(last_burst_seq),
        rest_z_mm,
        resting_side,
        ..open.clone()
    };
    let Some(rest) = rest_z_mm else {
        // Still settling: nothing is scored or paid.
        return Ok((AttemptFlow::Continue(next), Vec::new()));
    };

    // Left the support: ends at once, whatever the burst count. The object may rest on any
    // face, so the threshold starts from the lowest resting centre (its smallest extent up
    // is the lowest case) and allows one smallest dimension more.
    let up_extent_mm = resting_side
        .zip(params.extents_mm)
        .map_or(params.min_extent_mm, |(side, extents)| extents.of(side));
    if z < knocked_off_below(rest, up_extent_mm, params.min_extent_mm) {
        let signals = if train_affect {
            vec![pain(1.0)]
        } else {
            Vec::new()
        };
        return Ok((
            AttemptFlow::Finished(record(GraspAttemptOutcome::KnockedOff, max_height_mm)),
            signals,
        ));
    }
    if k_main == 0 {
        // The settle window just ended on this sample, or no burst has elapsed.
        next.last_distance_mm = if k == 0 {
            open.last_distance_mm
        } else {
            distance
        };
        return Ok((AttemptFlow::Continue(next), Vec::new()));
    }

    let grasped = sample.grasp_contacts >= criterion.min_contacts;
    let held_up_now = lift
        && height_mm >= params.lift_height_mm
        && sample.grasp_contacts >= 1
        && !sample.other_contact
        && !sample.world_contact;
    let condition = if lift { held_up_now } else { grasped };
    let lost_bursts = if condition {
        0
    } else {
        open.lost_bursts.saturating_add(k_main)
    };
    let effective = condition || (open.holding_bursts > 0 && lost_bursts <= params.bursts.grace);
    let holding_bursts = if effective {
        open.holding_bursts.saturating_add(k_main)
    } else {
        0
    };
    let was_held_up = open.was_held_up || held_up_now;
    let elsewhere = sample.other_contact || sample.world_contact;
    let drop_bursts = if lift && was_held_up && elsewhere && sample.grasp_contacts == 0 {
        open.drop_bursts.saturating_add(k_main)
    } else {
        0
    };

    // Reward B: ground gained over the best so far in this attempt.
    let mut signals = Vec::new();
    if let Some(best) = open.best_distance_mm {
        if distance < best && best > 0.0 && train_affect {
            signals.push(pleasure(
                ((best - distance) / best).clamp(f64::MIN_POSITIVE, 1.0),
            ));
        }
    }
    if sample.grasp_contacts >= 1 && !open.touched && train_affect {
        signals.push(pleasure(1.0));
    }
    if lift && height_mm > open.best_height_mm && train_affect {
        let gain = height_mm - open.best_height_mm;
        signals.push(pleasure((gain / params.lift_height_mm).min(1.0)));
    }

    next.holding_bursts = holding_bursts;
    next.lost_bursts = lost_bursts;
    next.drop_bursts = drop_bursts;
    next.was_held_up = was_held_up;
    next.best_distance_mm = Some(open.best_distance_mm.map_or(distance, |b| b.min(distance)));
    next.touched = open.touched || sample.grasp_contacts >= 1;
    next.best_height_mm = max_height_mm;
    next.last_distance_mm = distance;

    let max_height = max_height_mm;
    if holding_bursts >= params.bursts.hold {
        return Ok((
            AttemptFlow::Finished(record(GraspAttemptOutcome::Success, max_height)),
            signals,
        ));
    }
    if drop_bursts > params.bursts.grace {
        if train_affect {
            signals.push(pain(1.0));
        }
        return Ok((
            AttemptFlow::Finished(record(GraspAttemptOutcome::Dropped, max_height)),
            signals,
        ));
    }
    if bursts >= criterion.attempt_burst_cap {
        return Ok((
            AttemptFlow::Finished(record(GraspAttemptOutcome::Truncated, max_height)),
            signals,
        ));
    }
    Ok((AttemptFlow::Continue(next), signals))
}

/// Counts of how attempts ended.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraspOutcomeCounts {
    /// Attempts that succeeded.
    pub success: u32,
    /// Attempts that hit the burst cap.
    pub truncated: u32,
    /// Attempts ended by death.
    pub death: u32,
    /// Attempts ended by the safety box.
    pub fence: u32,
    /// Attempts interrupted by a reset.
    pub interrupted: u32,
    /// Attempts where the object was dropped.
    pub dropped: u32,
    /// Attempts where the object was knocked off its support.
    pub knocked_off: u32,
}

/// Aggregate numbers for one phase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraspPhaseReport {
    /// Finished attempts.
    pub attempts: Vec<GraspAttemptRecord>,
    /// Successes divided by attempt count.
    pub success_rate: f64,
    /// Median bursts among successes. Absent when nothing succeeded.
    pub median_bursts_to_success: Option<f64>,
    /// Mean terminal distance across every attempt, millimeters.
    pub mean_final_distance_mm: f64,
    /// Mean of the highest height reached per attempt, millimeters.
    pub mean_max_height_mm: f64,
    /// How the attempts ended.
    pub outcomes: GraspOutcomeCounts,
    /// The host skipped the phase: it ended after the attempt in progress.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub skipped: bool,
}

impl GraspPhaseReport {
    /// Summarize finished attempts. An empty phase is an error.
    pub fn from_attempts(attempts: Vec<GraspAttemptRecord>) -> Result<Self, TrainerError> {
        if attempts.is_empty() {
            return Err(TrainerError::Evaluation(
                "grasp phase has no attempts".to_string(),
            ));
        }
        let count = attempts.len() as f64;
        let mut outcomes = GraspOutcomeCounts::default();
        for attempt in &attempts {
            let slot = match attempt.outcome {
                GraspAttemptOutcome::Success => &mut outcomes.success,
                GraspAttemptOutcome::Truncated => &mut outcomes.truncated,
                GraspAttemptOutcome::Death => &mut outcomes.death,
                GraspAttemptOutcome::Fence => &mut outcomes.fence,
                GraspAttemptOutcome::Interrupted => &mut outcomes.interrupted,
                GraspAttemptOutcome::Dropped => &mut outcomes.dropped,
                GraspAttemptOutcome::KnockedOff => &mut outcomes.knocked_off,
            };
            *slot += 1;
        }
        let mut successes: Vec<u32> = attempts
            .iter()
            .filter(|a| a.outcome == GraspAttemptOutcome::Success)
            .map(|a| a.bursts)
            .collect();
        let median_bursts_to_success = if successes.is_empty() {
            None
        } else {
            successes.sort_unstable();
            let mid = successes.len() / 2;
            Some(if successes.len() % 2 == 1 {
                f64::from(successes[mid])
            } else {
                (f64::from(successes[mid - 1]) + f64::from(successes[mid])) / 2.0
            })
        };
        Ok(Self {
            success_rate: f64::from(outcomes.success) / count,
            median_bursts_to_success,
            mean_final_distance_mm: attempts.iter().map(|a| a.final_distance_mm).sum::<f64>()
                / count,
            mean_max_height_mm: attempts.iter().map(|a| a.max_height_mm).sum::<f64>() / count,
            outcomes,
            attempts,
            skipped: false,
        })
    }
}

/// Train phase, then held-out test phase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraspHoldReport {
    /// Attempts whose seeds were the training set.
    pub train: GraspPhaseReport,
    /// Attempts whose seeds were withheld from training.
    pub test: GraspPhaseReport,
}

/// Controller side of one grasp session. The MuJoCo HTTP server is one implementation.
pub trait GraspProbe {
    /// Home the body and place the object once on its support for `seed`.
    fn begin_attempt(
        &mut self,
        seed: u64,
        target_object_id: &str,
        grasping_bodies: &[String],
        placement: PlacementRequest,
    ) -> Result<GraspPlacement, TrainerError>;

    /// The object's smallest dimension, millimeters.
    fn object_min_extent_mm(&mut self, object_body_name: &str) -> Result<f64, TrainerError>;

    /// The object's largest dimension, millimeters. Read before the first placement, so the
    /// scene object is named by its id.
    fn object_max_extent_mm(&mut self, target_object_id: &str) -> Result<f64, TrainerError>;

    /// Wait for a reading newer than `after_seq` physics steps.
    fn poll(&mut self, after_seq: u64, timeout: Duration) -> Result<GraspSample, TrainerError>;
}

/// Seeds and the criterion for one session.
#[derive(Debug, Clone, PartialEq)]
pub struct GraspHoldConfig {
    /// Acceptance rule.
    pub criterion: GraspHoldCriterion,
    /// Training seeds, in visit order.
    pub train_seeds: Vec<u64>,
    /// Held-out seeds, in visit order.
    pub test_seeds: Vec<u64>,
    /// How long one poll may wait. Sized from the live burst period by the caller.
    pub poll_timeout: Duration,
    /// Live FEAGI burst frequency, Hz.
    pub frequency_hz: f64,
    /// Physics steps per burst: `burst_period_s / timestep_s` (not necessarily whole).
    pub steps_per_burst: f64,
}

impl GraspHoldConfig {
    /// Reject an empty phase or a criterion that cannot succeed.
    pub fn validate(&self) -> Result<(), TrainerError> {
        self.criterion.burst_counts(self.frequency_hz)?;
        if self.train_seeds.is_empty() {
            return Err(TrainerError::Config(
                "train_seeds must contain at least one seed".to_string(),
            ));
        }
        if self.test_seeds.is_empty() {
            return Err(TrainerError::Config(
                "test_seeds must contain at least one seed".to_string(),
            ));
        }
        if self.poll_timeout.is_zero() {
            return Err(TrainerError::Config("poll_timeout must be > 0".to_string()));
        }
        if !self.steps_per_burst.is_finite() || self.steps_per_burst <= 0.0 {
            return Err(TrainerError::Config(
                "steps_per_burst must be a finite value > 0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Named metrics for one finished phase. Median is omitted when nothing succeeded.
pub fn grasp_phase_metrics(phase: &GraspPhaseReport) -> BTreeMap<String, f64> {
    let mut metrics = BTreeMap::new();
    metrics.insert("success_rate".to_string(), phase.success_rate);
    metrics.insert(
        "mean_final_distance_mm".to_string(),
        phase.mean_final_distance_mm,
    );
    metrics.insert("mean_max_height_mm".to_string(), phase.mean_max_height_mm);
    metrics.insert("dropped".to_string(), f64::from(phase.outcomes.dropped));
    metrics.insert(
        "knocked_off".to_string(),
        f64::from(phase.outcomes.knocked_off),
    );
    if let Some(median) = phase.median_bursts_to_success {
        metrics.insert("median_bursts_to_success".to_string(), median);
    }
    metrics
}

/// Scorecard for one grasp phase. Train and test share dataset identity so a protocol
/// evaluation can compare them.
pub fn grasp_phase_scorecard(
    scorecard_id: &str,
    connectome_hash: &str,
    genome_schema_version: u32,
    split_id: &str,
    dataset_asset_id: &str,
    dataset_content_hash: &str,
    phase: &GraspPhaseReport,
) -> Scorecard {
    let version = crate::reach_hold::trainer_package_version();
    Scorecard {
        schema_version: crate::contracts::scorecard::SCHEMA_VERSION,
        scorecard_id: ScorecardId(scorecard_id.to_string()),
        connectome_hash: ConnectomeHash(connectome_hash.to_string()),
        genome_version_id: None,
        genome_schema_version: Some(genome_schema_version),
        dataset_asset_id: DatasetAssetId(dataset_asset_id.to_string()),
        dataset_version: "1".to_string(),
        dataset_content_hash: ContentHash(dataset_content_hash.to_string()),
        evaluation_protocol_version: EvaluationProtocolVersion("grasp-hold-v1".to_string()),
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
        metrics: grasp_phase_metrics(phase),
        class_breakdown: Vec::new(),
        confusion: None,
        metric_stats: None,
        status: ScorecardStatus::SelfReported,
        visibility: ScorecardVisibility::Local,
        skip: None,
        elapsed_ms: None,
        sample_draw_seed: None,
        metadata: BTreeMap::new(),
    }
}

/// Run the train phase, then the test phase.
///
/// `coach` advances one burst and, during training only, sends pain or pleasure. The test
/// phase steps only. Each attempt emits one progress event and each phase one aggregate
/// metric update.
pub fn run_grasp_hold(
    config: &GraspHoldConfig,
    probe: &mut dyn GraspProbe,
    coach: &mut dyn FnMut(CoachWrite) -> Result<(), TrainerError>,
    events: &mut dyn RunEventSink,
    run_id: &RunId,
    cancel: &CancelToken,
    mut data: Option<&mut (dyn RunDataSink + '_)>,
) -> Result<GraspHoldReport, TrainerError> {
    config.validate()?;
    let train = run_grasp_phase(
        &GraspPhaseRun {
            phase: "train",
            seeds: &config.train_seeds,
            criterion: &config.criterion,
            poll_timeout: config.poll_timeout,
            frequency_hz: config.frequency_hz,
            steps_per_burst: config.steps_per_burst,
            run_id,
        },
        probe,
        coach,
        events,
        cancel,
        data.as_deref_mut(),
    )?;
    let test = run_grasp_phase(
        &GraspPhaseRun {
            phase: "test",
            seeds: &config.test_seeds,
            criterion: &config.criterion,
            poll_timeout: config.poll_timeout,
            frequency_hz: config.frequency_hz,
            steps_per_burst: config.steps_per_burst,
            run_id,
        },
        probe,
        coach,
        events,
        cancel,
        data,
    )?;
    Ok(GraspHoldReport { train, test })
}

/// One phase of a grasp session, plus the run the progress events belong to.
pub struct GraspPhaseRun<'a> {
    /// `train` or `test`.
    pub phase: &'a str,
    /// Seeds visited in order.
    pub seeds: &'a [u64],
    /// Acceptance rule.
    pub criterion: &'a GraspHoldCriterion,
    /// How long one poll may wait.
    pub poll_timeout: Duration,
    /// Live FEAGI burst frequency, Hz.
    pub frequency_hz: f64,
    /// Physics steps per burst.
    pub steps_per_burst: f64,
    /// Run the progress events are stamped with.
    pub run_id: &'a RunId,
}

/// Run one phase (`train` or `test`) from its first seed and emit its progress.
pub fn run_grasp_phase(
    run: &GraspPhaseRun<'_>,
    probe: &mut dyn GraspProbe,
    coach: &mut dyn FnMut(CoachWrite) -> Result<(), TrainerError>,
    events: &mut dyn RunEventSink,
    cancel: &CancelToken,
    data: Option<&mut (dyn RunDataSink + '_)>,
) -> Result<GraspPhaseReport, TrainerError> {
    run_grasp_phase_from(
        run,
        &ResumeFrom::default(),
        probe,
        coach,
        events,
        cancel,
        data,
    )
    .map(|(report, _)| report)
}

/// Run one phase from `resume.first_index` (the seeds before it ran in earlier segments).
///
/// Returns the report of the attempts run in this segment and the [`ResumeExport`] for the next
/// one, as `run_reach_phase_from` does.
pub fn run_grasp_phase_from(
    run: &GraspPhaseRun<'_>,
    resume: &ResumeFrom,
    probe: &mut dyn GraspProbe,
    coach: &mut dyn FnMut(CoachWrite) -> Result<(), TrainerError>,
    events: &mut dyn RunEventSink,
    cancel: &CancelToken,
    data: Option<&mut (dyn RunDataSink + '_)>,
) -> Result<(GraspPhaseReport, ResumeExport), TrainerError> {
    if run.phase != "train" && run.phase != "test" {
        return Err(TrainerError::Config(
            "grasp phase must be train or test".to_string(),
        ));
    }
    let bursts = run.criterion.burst_counts(run.frequency_hz)?;
    if run.seeds.is_empty() {
        return Err(TrainerError::Config(format!(
            "{} seeds must contain at least one seed",
            run.phase
        )));
    }
    if run.poll_timeout.is_zero() {
        return Err(TrainerError::Config("poll_timeout must be > 0".to_string()));
    }
    if !run.steps_per_burst.is_finite() || run.steps_per_burst <= 0.0 {
        return Err(TrainerError::Config(
            "steps_per_burst must be a finite value > 0".to_string(),
        ));
    }
    let train_affect = run.phase == "train";
    let mut rules = GraspRules {
        probe,
        criterion: run.criterion,
        bursts,
        steps_per_burst: run.steps_per_burst,
        min_extent_mm: None,
        spread_mm: run.criterion.placement_spread_mm,
        params: None,
    };
    let outcome = run_phase_loop(
        &PhaseSpec {
            phase: run.phase,
            seeds: run.seeds,
            run_id: run.run_id,
            label: "grasp-and-hold",
            tables: vec![grasp_steps_spec(), grasp_attempts_spec()],
        },
        resume,
        events,
        cancel,
        data,
        &mut |seed, attempt_data: AttemptData<'_, '_>| {
            run_attempt(
                &mut rules,
                seed,
                train_affect,
                run.poll_timeout,
                coach,
                cancel,
                attempt_data,
            )
        },
        &|phase, attempt_index, record| {
            vec![
                RunDataValue::text(phase),
                RunDataValue::count(attempt_index),
                RunDataValue::count(record.seed),
                RunDataValue::Text(name_of(&record.outcome)),
                RunDataValue::count(u64::from(record.bursts)),
                RunDataValue::Float(record.final_distance_mm),
                RunDataValue::Float(record.max_height_mm),
                side_value(record.resting_side),
                side_value(record.requested_side),
                record
                    .placed_mm
                    .map_or(RunDataValue::Null, |p| RunDataValue::Float(p[0])),
                record
                    .placed_mm
                    .map_or(RunDataValue::Null, |p| RunDataValue::Float(p[1])),
            ]
        },
    )?;
    let mut report = GraspPhaseReport::from_attempts(outcome.attempts)?;
    report.skipped = outcome.skipped;
    events.emit(RunEvent::new(
        run.run_id.clone(),
        RunEventKind::MetricUpdate {
            scope: MetricScope::Aggregate,
            metrics: grasp_phase_metrics(&report),
            class_breakdown: Vec::new(),
            confusion: None,
        },
    ));
    Ok((report, outcome.export))
}

fn side_value(side: Option<UpExtent>) -> RunDataValue {
    side.map_or(RunDataValue::Null, |s| RunDataValue::text(s.side_name()))
}

fn grasp_steps_spec() -> TableSpec {
    let mut columns = vec![
        ColumnSpec::unitless("phase", "Phase the sample belongs to."),
        ColumnSpec::unitless("attempt", "Attempt index within the phase."),
        ColumnSpec::unitless("seed", "Attempt seed."),
        ColumnSpec::unitless("seq", "Controller physics-step counter."),
        ColumnSpec::with_unit("distance", "mm", "Nearest grasping body to the object."),
    ];
    for axis in ["x", "y", "z"] {
        columns.push(ColumnSpec::with_unit(
            &format!("object_{axis}"),
            "mm",
            &format!("Object centre of mass, {axis} axis."),
        ));
    }
    columns.push(ColumnSpec::unitless(
        "grasp_contacts",
        "Grasping bodies touching the object.",
    ));
    columns.push(ColumnSpec::unitless(
        "other_contact",
        "The object touches a body that is neither grasping nor the world.",
    ));
    columns.push(ColumnSpec::unitless(
        "world_contact",
        "The object touches the world.",
    ));
    columns.push(ColumnSpec::unitless(
        "fence_violated",
        "The watched body is outside the safety box.",
    ));
    columns.push(ColumnSpec::unitless(
        "terminal",
        "Latched end cause, when one was latched.",
    ));
    TableSpec {
        name: STEPS_TABLE.to_string(),
        description: "One row per controller reading.".to_string(),
        columns,
    }
}

fn grasp_attempts_spec() -> TableSpec {
    TableSpec {
        name: ATTEMPTS_TABLE.to_string(),
        description: "One row per finished attempt.".to_string(),
        columns: vec![
            ColumnSpec::unitless("phase", "Phase the attempt belongs to."),
            ColumnSpec::unitless("attempt", "Attempt index within the phase."),
            ColumnSpec::unitless("seed", "Attempt seed."),
            ColumnSpec::unitless(
                "outcome",
                "How the attempt ended (success, truncated, death, fence, interrupted, dropped, knocked_off).",
            ),
            ColumnSpec::unitless("bursts", "Bursts observed after the settle window."),
            ColumnSpec::with_unit(
                "final_distance",
                "mm",
                "Nearest grasping body at the terminal burst.",
            ),
            ColumnSpec::with_unit(
                "max_height",
                "mm",
                "Highest the object got above its resting height.",
            ),
            ColumnSpec::unitless(
                "resting_side",
                "Face the object rested on at the end of the settle window (flat, edge, end). Results are scored under this side.",
            ),
            ColumnSpec::unitless(
                "requested_side",
                "Face the Trainer asked for (flat, edge, end).",
            ),
            ColumnSpec::with_unit("placed_x", "mm", "Where the controller placed the object, x."),
            ColumnSpec::with_unit("placed_y", "mm", "Where the controller placed the object, y."),
        ],
    }
}

/// Grasp's rules for the shared attempt loop.
struct GraspRules<'p, 'c> {
    probe: &'p mut dyn GraspProbe,
    criterion: &'c GraspHoldCriterion,
    bursts: GraspBursts,
    steps_per_burst: f64,
    /// The object's smallest dimension, read once per session after the first placement.
    min_extent_mm: Option<f64>,
    /// Placement spread, millimeters. Resolved once per session when the criterion leaves it
    /// empty: one times the object's largest extent.
    spread_mm: Option<f64>,
    params: Option<GraspParams>,
}

impl AttemptRules for GraspRules<'_, '_> {
    type Sample = GraspSample;
    type Open = OpenGraspAttempt;
    type Record = GraspAttemptRecord;

    fn cancelled_message(&self) -> &'static str {
        "grasp-and-hold run cancelled"
    }

    fn begin(&mut self, seed: u64) -> Result<(), TrainerError> {
        let up_extent = self.criterion.resting_side.requested_for(seed);
        let spread_mm = match self.spread_mm {
            Some(value) => value,
            None => {
                // The default is one times the object's largest dimension, read once per
                // phase before the first placement.
                let value = self
                    .probe
                    .object_max_extent_mm(&self.criterion.target_object_id)?;
                if !value.is_finite() || value <= 0.0 {
                    return Err(TrainerError::Runtime(format!(
                        "object {} has no usable size (max extent {value})",
                        self.criterion.target_object_id
                    )));
                }
                self.spread_mm = Some(value);
                value
            }
        };
        let placement = self.probe.begin_attempt(
            seed,
            &self.criterion.target_object_id,
            &self.criterion.grasping_bodies,
            PlacementRequest {
                spread_mm,
                up_extent,
            },
        )?;
        if placement.seed != seed {
            return Err(TrainerError::Runtime(format!(
                "controller placed seed {} for requested seed {seed}",
                placement.seed
            )));
        }
        let min_extent_mm = match self.min_extent_mm {
            Some(value) => value,
            None => {
                let value = self
                    .probe
                    .object_min_extent_mm(&placement.object_body_name)?;
                if !value.is_finite() || value <= 0.0 {
                    return Err(TrainerError::Runtime(format!(
                        "object {} has no usable size (min extent {value})",
                        placement.object_body_name
                    )));
                }
                self.min_extent_mm = Some(value);
                value
            }
        };
        self.params = Some(GraspParams {
            bursts: self.bursts,
            steps_per_burst: self.steps_per_burst,
            lift_height_mm: self.criterion.lift_height_mm.unwrap_or(min_extent_mm),
            min_extent_mm,
            extents_mm: placement.extents_mm,
            requested_side: Some(up_extent),
            placed_mm: placement.target_mm.map(|t| [t[0], t[1]]),
        });
        Ok(())
    }

    fn fresh(&self) -> OpenGraspAttempt {
        OpenGraspAttempt::fresh()
    }

    fn open_bursts(&self, open: &OpenGraspAttempt) -> u32 {
        open.bursts
    }

    fn open_seq(&self, open: &OpenGraspAttempt) -> u64 {
        open.last_seq
    }

    fn poll(&mut self, after_seq: u64, timeout: Duration) -> Result<GraspSample, TrainerError> {
        self.probe.poll(after_seq, timeout)
    }

    fn step_row(&self, phase: &str, attempt: u64, sample: &GraspSample) -> Vec<RunDataValue> {
        let mut row = vec![
            RunDataValue::text(phase),
            RunDataValue::count(attempt),
            RunDataValue::count(sample.seed),
            RunDataValue::count(sample.seq),
            sample
                .nearest_distance_mm()
                .map_or(RunDataValue::Null, RunDataValue::Float),
        ];
        row.extend(sample.object_mm.iter().map(|v| RunDataValue::Float(*v)));
        row.push(RunDataValue::count(u64::from(sample.grasp_contacts)));
        row.push(RunDataValue::Bool(sample.other_contact));
        row.push(RunDataValue::Bool(sample.world_contact));
        row.push(RunDataValue::Bool(sample.fence_violated));
        row.push(
            sample
                .terminal
                .as_ref()
                .map_or(RunDataValue::Null, |t| RunDataValue::Text(name_of(t))),
        );
        row
    }

    fn observe(
        &mut self,
        open: &OpenGraspAttempt,
        sample: &GraspSample,
        seed: u64,
        train_affect: bool,
    ) -> Result<(GraspStep, Vec<RewardSignal>), TrainerError> {
        let params = self.params.as_ref().ok_or_else(|| {
            TrainerError::Runtime("grasp sample arrived before the attempt began".to_string())
        })?;
        observe_grasp_sample(open, sample, self.criterion, params, seed, train_affect)
    }

    fn record_bursts(&self, record: &GraspAttemptRecord) -> u32 {
        record.bursts
    }

    fn record_outcome(&self, record: &GraspAttemptRecord) -> String {
        name_of(&record.outcome)
    }
}

/// HTTP client for a grasp session on a controller control server.
///
/// Placement uses `begin_reach_attempt` with `placement: "free_on_support"`. Each poll takes
/// the fence flag and terminal from `poll_reach_outcome` (the first grasping body is the
/// watched body) and the readings from one `measure` call.
pub struct HttpGraspProbe {
    control: HttpControl,
    object_body: String,
    grasping_bodies: Vec<String>,
    last_reach_seq: u64,
}

impl HttpGraspProbe {
    /// `base_url` is `http://host:port` with no path.
    pub fn new(base_url: &str, timeout: Duration) -> Result<Self, TrainerError> {
        Ok(Self {
            control: HttpControl::new(base_url, timeout, "grasp control")?,
            object_body: String::new(),
            grasping_bodies: Vec::new(),
            last_reach_seq: 0,
        })
    }

    /// Physics timestep of the loaded model, seconds (`burst_period_s / timestep_s` is
    /// `steps_per_burst`).
    pub fn model_timestep_s(&self) -> Result<f64, TrainerError> {
        let info = self
            .control
            .command(&serde_json::json!({"action": "get_model_info"}))?;
        info.get("timestep_s")
            .and_then(serde_json::Value::as_f64)
            .filter(|value| value.is_finite() && *value > 0.0)
            .ok_or_else(|| TrainerError::Runtime("model info has no usable timestep_s".to_string()))
    }
}

fn vec3_mm(value: &serde_json::Value, what: &str) -> Result<[f64; 3], TrainerError> {
    let array = value
        .as_array()
        .filter(|a| a.len() == 3)
        .ok_or_else(|| TrainerError::Runtime(format!("grasp {what} is not a 3-vector")))?;
    let mut out = [0.0; 3];
    for (slot, item) in out.iter_mut().zip(array) {
        *slot = item
            .as_f64()
            .ok_or_else(|| TrainerError::Runtime(format!("grasp {what} is not numeric")))?
            * 1000.0;
    }
    Ok(out)
}

impl GraspProbe for HttpGraspProbe {
    fn begin_attempt(
        &mut self,
        seed: u64,
        target_object_id: &str,
        grasping_bodies: &[String],
        placement: PlacementRequest,
    ) -> Result<GraspPlacement, TrainerError> {
        let watched = grasping_bodies
            .first()
            .ok_or_else(|| TrainerError::Config("grasping_bodies must not be empty".to_string()))?;
        let result = self.control.command(&serde_json::json!({
            "action": "begin_reach_attempt",
            "seed": seed,
            "target_object_id": target_object_id,
            "reach_body": watched,
            "placement": "free_on_support",
            "placement_spread_mm": placement.spread_mm,
            "up_extent": placement.up_extent.wire_name(),
        }))?;
        let placed: GraspPlacement = serde_json::from_value(result).map_err(|error| {
            TrainerError::Runtime(format!("grasp begin decode failed: {error}"))
        })?;
        self.object_body = placed.object_body_name.clone();
        self.grasping_bodies = grasping_bodies.to_vec();
        self.last_reach_seq = 0;
        Ok(placed)
    }

    fn object_min_extent_mm(&mut self, object_body_name: &str) -> Result<f64, TrainerError> {
        let info = self
            .control
            .command(&serde_json::json!({"action": "get_model_info"}))?;
        info.get("bodies")
            .and_then(serde_json::Value::as_array)
            .and_then(|bodies| {
                bodies.iter().find(|body| {
                    body.get("name").and_then(serde_json::Value::as_str) == Some(object_body_name)
                })
            })
            .and_then(|body| body.get("min_extent_mm"))
            .and_then(serde_json::Value::as_f64)
            .ok_or_else(|| {
                TrainerError::Runtime(format!(
                    "model info has no size for body {object_body_name}"
                ))
            })
    }

    fn object_max_extent_mm(&mut self, target_object_id: &str) -> Result<f64, TrainerError> {
        let listed = self
            .control
            .command(&serde_json::json!({"action": "list_scene_objects"}))?;
        let body_name = listed
            .get("objects")
            .and_then(serde_json::Value::as_array)
            .and_then(|objects| {
                objects.iter().find(|object| {
                    object.get("id").and_then(serde_json::Value::as_str) == Some(target_object_id)
                })
            })
            .and_then(|object| object.get("body_name"))
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                TrainerError::Runtime(format!("scene has no body for object {target_object_id}"))
            })?
            .to_string();
        let info = self
            .control
            .command(&serde_json::json!({"action": "get_model_info"}))?;
        info.get("bodies")
            .and_then(serde_json::Value::as_array)
            .and_then(|bodies| {
                bodies.iter().find(|body| {
                    body.get("name").and_then(serde_json::Value::as_str) == Some(body_name.as_str())
                })
            })
            .and_then(|body| body.get("extent_mm"))
            .and_then(serde_json::Value::as_array)
            .map(|extent| {
                extent
                    .iter()
                    .filter_map(serde_json::Value::as_f64)
                    .collect::<Vec<_>>()
            })
            .filter(|values| !values.is_empty())
            .map(|values| values.into_iter().fold(f64::MIN, f64::max))
            .ok_or_else(|| {
                TrainerError::Runtime(format!("model info has no extents for body {body_name}"))
            })
    }

    fn poll(&mut self, _after_seq: u64, timeout: Duration) -> Result<GraspSample, TrainerError> {
        let timeout_s = timeout.as_secs_f64();
        if timeout_s <= 0.0 {
            return Err(TrainerError::Config(
                "grasp poll timeout must be > 0".to_string(),
            ));
        }
        let reach = self.control.command(&serde_json::json!({
            "action": "poll_reach_outcome",
            "after_seq": self.last_reach_seq,
            "timeout_s": timeout_s,
        }))?;
        if let Some(seq) = reach.get("seq").and_then(serde_json::Value::as_u64) {
            self.last_reach_seq = seq;
        }
        let terminal: Option<AttemptTerminal> = match reach.get("terminal") {
            None | Some(serde_json::Value::Null) => None,
            Some(value) => Some(serde_json::from_value(value.clone()).map_err(|error| {
                TrainerError::Runtime(format!("grasp terminal decode failed: {error}"))
            })?),
        };
        let fence_violated = reach
            .get("fence_violated")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let seed = reach
            .get("seed")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| TrainerError::Runtime("grasp poll has no seed".to_string()))?;

        let mut except = self.grasping_bodies.clone();
        except.push(self.object_body.clone());
        let mut bodies = vec![self.object_body.clone()];
        bodies.extend(self.grasping_bodies.iter().cloned());
        let measured = self.control.command(&serde_json::json!({
            "action": "measure",
            "bodies": bodies,
            "contacts": [
                {"a": self.grasping_bodies, "b": [self.object_body]},
                {"a_except": except, "b": [self.object_body]},
                {"a": [self.object_body], "b": "world"},
            ],
        }))?;
        let com_of = |name: &str| -> Result<[f64; 3], TrainerError> {
            let value = measured
                .get("bodies")
                .and_then(|b| b.get(name))
                .and_then(|b| b.get("com"))
                .ok_or_else(|| {
                    TrainerError::Runtime(format!("grasp measure has no position for {name}"))
                })?;
            vec3_mm(value, name)
        };
        let object_mm = com_of(&self.object_body)?;
        let grasping_mm = self
            .grasping_bodies
            .iter()
            .map(|name| com_of(name))
            .collect::<Result<Vec<_>, _>>()?;
        let contacts = measured
            .get("contacts")
            .and_then(serde_json::Value::as_array)
            .filter(|sets| sets.len() == 3)
            .ok_or_else(|| {
                TrainerError::Runtime("grasp measure returned the wrong contact sets".to_string())
            })?;
        let touching = |set: &serde_json::Value| {
            set.get("touching")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
        };
        let grasp_contacts = if touching(&contacts[0]) {
            contacts[0]
                .get("bodies")
                .and_then(serde_json::Value::as_array)
                .map_or(1, |list| list.len().max(1)) as u32
        } else {
            0
        };
        let up_extent = match measured
            .get("bodies")
            .and_then(|b| b.get(&self.object_body))
            .and_then(|b| b.get("up_extent"))
        {
            None | Some(serde_json::Value::Null) => None,
            Some(value) => Some(serde_json::from_value(value.clone()).map_err(|error| {
                TrainerError::Runtime(format!("grasp up_extent decode failed: {error}"))
            })?),
        };
        let seq = measured
            .get("physics_steps")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| {
                TrainerError::Runtime("grasp measure has no physics_steps".to_string())
            })?;
        Ok(GraspSample {
            seq,
            seed,
            object_mm,
            grasping_mm,
            grasp_contacts,
            other_contact: touching(&contacts[1]),
            world_contact: touching(&contacts[2]),
            fence_violated,
            terminal,
            up_extent,
        })
    }
}

/// Run controls this engine's loop honours (pause, skip and cancel; at the attempt).
pub const CONTROL: crate::control::EngineControl = crate::control::EngineControl {
    pause: true,
    skip: true,
    cancel: true,
    boundary: crate::control::ControlBoundary::Attempt,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::CollectingEventSink;
    use std::collections::VecDeque;

    /// Physics steps per burst in the tests, and the burst frequency.
    const STEPS: f64 = 10.0;
    const HZ: f64 = 10.0;

    /// Ranked extents of the scripted object: smallest is 10 mm.
    const EXTENTS: ObjectExtents = ObjectExtents {
        smallest: 10.0,
        middle: 30.0,
        largest: 90.0,
    };

    struct ScriptedProbe {
        samples: VecDeque<GraspSample>,
        requests: Vec<(u64, PlacementRequest)>,
    }

    impl GraspProbe for ScriptedProbe {
        fn begin_attempt(
            &mut self,
            seed: u64,
            target_object_id: &str,
            _grasping_bodies: &[String],
            placement: PlacementRequest,
        ) -> Result<GraspPlacement, TrainerError> {
            self.requests.push((seed, placement));
            Ok(GraspPlacement {
                seed,
                target_object_id: target_object_id.to_string(),
                object_body_name: "item".to_string(),
                support_body_name: Some("shelf".to_string()),
                target_mm: Some([12.0, -7.0, 90.0]),
                extents_mm: Some(EXTENTS),
            })
        }

        fn object_min_extent_mm(&mut self, _object_body_name: &str) -> Result<f64, TrainerError> {
            Ok(10.0)
        }

        fn object_max_extent_mm(&mut self, _target_object_id: &str) -> Result<f64, TrainerError> {
            Ok(EXTENTS.largest)
        }

        fn poll(
            &mut self,
            _after_seq: u64,
            _timeout: Duration,
        ) -> Result<GraspSample, TrainerError> {
            self.samples
                .pop_front()
                .ok_or_else(|| TrainerError::Runtime("no scripted sample".to_string()))
        }
    }

    fn criterion(rule: GraspRule) -> GraspHoldCriterion {
        GraspHoldCriterion {
            rule,
            target_object_id: "item".to_string(),
            grasping_bodies: vec!["jaw_a".to_string(), "jaw_b".to_string()],
            min_contacts: 1,
            hold_s: 0.3,
            attempt_burst_cap: 20,
            settle_s: 0.2,
            contact_grace_s: None,
            lift_height_mm: Some(20.0),
            placement_spread_mm: Some(0.0),
            resting_side: RestingSide::Mixed,
        }
    }

    fn params(rule: GraspRule) -> GraspParams {
        GraspParams {
            bursts: criterion(rule).burst_counts(HZ).expect("bursts"),
            steps_per_burst: STEPS,
            lift_height_mm: 20.0,
            min_extent_mm: 10.0,
            extents_mm: Some(EXTENTS),
            requested_side: Some(UpExtent::Smallest),
            placed_mm: Some([12.0, -7.0]),
        }
    }

    /// One reading `burst` bursts into the attempt (physics steps = burst * STEPS).
    fn at(burst: u64, height_mm: f64, grasp: u32, other: bool, world: bool) -> GraspSample {
        GraspSample {
            seq: burst * STEPS as u64,
            seed: 1,
            object_mm: [0.0, 0.0, 100.0 + height_mm],
            grasping_mm: vec![[5.0, 0.0, 100.0 + height_mm], [50.0, 0.0, 100.0]],
            grasp_contacts: grasp,
            other_contact: other,
            world_contact: world,
            fence_violated: false,
            terminal: None,
            up_extent: Some(UpExtent::Smallest),
        }
    }

    /// Fold `samples` in order, collecting signals, until one finishes.
    fn fold(
        rule: GraspRule,
        samples: &[GraspSample],
        train: bool,
    ) -> (Option<GraspAttemptRecord>, Vec<RewardSignal>) {
        let criterion = criterion(rule);
        let params = params(rule);
        let mut open = OpenGraspAttempt::fresh();
        let mut all = Vec::new();
        for sample in samples {
            let (step, signals) =
                observe_grasp_sample(&open, sample, &criterion, &params, 1, train).expect("step");
            all.extend(signals);
            match step {
                AttemptFlow::Continue(next) => open = next,
                AttemptFlow::Finished(record) => return (Some(record), all),
            }
        }
        (None, all)
    }

    /// Two settle bursts (0.2 s at 10 Hz) on the support, touching nothing.
    fn settled() -> Vec<GraspSample> {
        vec![at(1, 0.0, 0, false, true), at(2, 0.0, 0, false, true)]
    }

    #[test]
    fn grip_and_raise_succeeds_for_lift() {
        let mut samples = settled();
        samples.push(at(3, 0.0, 2, false, true)); // gripped, still on the table
        for burst in 4..=6 {
            samples.push(at(burst, 30.0, 2, false, false)); // raised, touching only the jaws
        }
        let (record, _) = fold(GraspRule::Lift, &samples, true);
        let record = record.expect("finished");
        assert_eq!(record.outcome, GraspAttemptOutcome::Success);
        assert_eq!(record.bursts, 4, "settle is not counted; hold is 3 bursts");
        assert!(record.max_height_mm >= 30.0);
    }

    #[test]
    fn pushing_along_the_table_never_lifts_but_grabs() {
        let mut samples = settled();
        for burst in 3..=8 {
            samples.push(at(burst, 0.0, 1, false, true)); // contact, height 0, on the table
        }
        let (lift, _) = fold(GraspRule::Lift, &samples, false);
        assert_ne!(
            lift.map(|r| r.outcome),
            Some(GraspAttemptOutcome::Success),
            "a push is never a lift"
        );
        let (grab, _) = fold(GraspRule::GrabHold, &samples, false);
        assert_eq!(
            grab.expect("finished").outcome,
            GraspAttemptOutcome::Success
        );
    }

    #[test]
    fn grab_needs_the_minimum_contacts() {
        let mut crit = criterion(GraspRule::GrabHold);
        crit.min_contacts = 2;
        let params = params(GraspRule::GrabHold);
        let mut open = OpenGraspAttempt::fresh();
        let mut samples = settled();
        samples.extend((3..=8).map(|b| at(b, 0.0, 1, false, true)));
        let mut done = None;
        for sample in &samples {
            let (step, _) =
                observe_grasp_sample(&open, sample, &crit, &params, 1, false).expect("step");
            match step {
                AttemptFlow::Continue(next) => open = next,
                AttemptFlow::Finished(record) => {
                    done = Some(record);
                    break;
                }
            }
        }
        assert!(done.is_none(), "one contact does not make a pinch");
    }

    #[test]
    fn losing_the_grasp_onto_the_table_is_dropped() {
        let mut samples = settled();
        samples.push(at(3, 30.0, 2, false, false)); // held up
        samples.push(at(4, 0.0, 0, false, true)); // fell back on the table, no jaw on it
        samples.push(at(5, 0.0, 0, false, true));
        samples.push(at(6, 0.0, 0, false, true));
        let (record, signals) = fold(GraspRule::Lift, &samples, true);
        assert_eq!(
            record.expect("finished").outcome,
            GraspAttemptOutcome::Dropped
        );
        let last = signals.last().expect("pain");
        assert_eq!(last.channel, AffectChannel::Pain);
        assert_eq!(last.magnitude, 1.0);
    }

    #[test]
    fn setting_down_while_gripped_only_resets_the_hold() {
        let mut samples = settled();
        samples.push(at(3, 30.0, 2, false, false));
        samples.push(at(4, 0.0, 2, false, true)); // set down, still gripped
        samples.push(at(5, 0.0, 2, false, true));
        samples.push(at(6, 0.0, 2, false, true));
        let (record, _) = fold(GraspRule::Lift, &samples, false);
        assert!(record.is_none(), "neither dropped nor succeeded");
    }

    #[test]
    fn falling_off_the_support_is_knocked_off() {
        let mut samples = settled();
        samples.push(at(3, -25.0, 0, false, false)); // 25 mm below rest > 10 mm extent
        let (record, signals) = fold(GraspRule::Lift, &samples, true);
        assert_eq!(
            record.expect("finished").outcome,
            GraspAttemptOutcome::KnockedOff
        );
        assert_eq!(signals.last().expect("pain").channel, AffectChannel::Pain);
    }

    #[test]
    fn a_one_sample_contact_flicker_within_the_grace_keeps_the_hold() {
        let mut samples = settled();
        samples.push(at(3, 30.0, 2, false, false));
        samples.push(at(4, 30.0, 0, false, false)); // contact flickers out for one burst
        samples.push(at(5, 30.0, 2, false, false));
        samples.push(at(6, 30.0, 2, false, false));
        let (record, _) = fold(GraspRule::Lift, &samples, false);
        let record = record.expect("finished");
        assert_eq!(record.outcome, GraspAttemptOutcome::Success);
        assert_eq!(
            record.bursts, 3,
            "the flicker burst still counted toward the hold"
        );
    }

    #[test]
    fn a_loss_longer_than_the_grace_breaks_the_hold() {
        let mut samples = settled();
        samples.push(at(3, 30.0, 2, false, false));
        samples.push(at(4, 30.0, 0, false, false));
        samples.push(at(5, 30.0, 0, false, false)); // two bursts out, grace is one
        samples.push(at(6, 30.0, 2, false, false));
        let (record, _) = fold(GraspRule::Lift, &samples, false);
        assert!(record.is_none(), "the hold restarted and has not completed");
    }

    #[test]
    fn bursts_are_counted_from_physics_steps() {
        let criterion = criterion(GraspRule::GrabHold);
        let params = params(GraspRule::GrabHold);
        let (step, _) = observe_grasp_sample(
            &OpenGraspAttempt::fresh(),
            &at(1, 0.0, 0, false, true),
            &criterion,
            &params,
            1,
            false,
        )
        .expect("first");
        let open = match step {
            AttemptFlow::Continue(open) => open,
            AttemptFlow::Finished(_) => panic!("first sample must not finish"),
        };
        assert_eq!(open.settle_bursts, 1);
        // One sample spans three bursts: two finish the settle window, one counts.
        let (step, _) = observe_grasp_sample(
            &open,
            &at(4, 0.0, 0, false, true),
            &criterion,
            &params,
            1,
            false,
        )
        .expect("batch");
        let open = match step {
            AttemptFlow::Continue(open) => open,
            AttemptFlow::Finished(_) => panic!("must continue"),
        };
        assert_eq!(open.settle_bursts, 2);
        assert_eq!(open.bursts, 2, "3 bursts elapsed: 1 settle + 2 counted");
        // A sample inside the same burst counts nothing.
        let (step, _) = observe_grasp_sample(
            &open,
            &at(4, 0.0, 0, false, true),
            &criterion,
            &params,
            1,
            false,
        )
        .expect("repeat");
        match step {
            AttemptFlow::Continue(same) => assert_eq!(same.bursts, 2),
            AttemptFlow::Finished(_) => panic!("must continue"),
        }
        // Three bursts at once with the settle done count three.
        let (step, _) = observe_grasp_sample(
            &open,
            &at(7, 0.0, 0, false, true),
            &criterion,
            &params,
            1,
            false,
        )
        .expect("three");
        match step {
            AttemptFlow::Continue(next) => assert_eq!(next.bursts, 5),
            AttemptFlow::Finished(_) => panic!("must continue"),
        }
    }

    #[test]
    fn reward_pays_new_best_only_and_never_a_repeat() {
        let mut samples = settled();
        samples.push(at(3, 0.0, 0, false, true)); // baseline distance 45 mm... nearest jaw at 5 mm
        let mut closer = at(4, 0.0, 0, false, true);
        closer.grasping_mm = vec![[2.5, 0.0, 100.0], [50.0, 0.0, 100.0]];
        samples.push(closer.clone());
        let mut repeat = at(5, 0.0, 0, false, true);
        repeat.grasping_mm = closer.grasping_mm.clone();
        samples.push(repeat); // same distance: no pay
        samples.push(at(6, 0.0, 1, false, true)); // first contact: 1.0 (distance got worse: no pay)
        samples.push(at(7, 0.0, 1, false, true)); // contact again: no pay
        let (_, signals) = fold(GraspRule::Lift, &samples, true);
        let pleasures: Vec<f64> = signals
            .iter()
            .filter(|s| s.channel == AffectChannel::Pleasure)
            .map(|s| s.magnitude)
            .collect();
        assert_eq!(
            pleasures.len(),
            2,
            "closer once, contact once: {pleasures:?}"
        );
        assert!((pleasures[0] - 0.5).abs() < 1e-9, "5 -> 2.5 mm is half");
        assert_eq!(pleasures[1], 1.0);
    }

    #[test]
    fn height_gain_pays_a_fraction_of_the_lift_height() {
        let mut samples = settled();
        samples.push(at(3, 0.0, 1, false, true));
        samples.push(at(4, 10.0, 1, false, false)); // new best: 10 of 20 mm
        samples.push(at(5, 10.0, 1, false, false)); // repeat
        samples.push(at(6, 40.0, 1, false, false)); // gain 30 > lift height: capped at 1
        let (_, signals) = fold(GraspRule::Lift, &samples, true);
        let heights: Vec<f64> = signals
            .iter()
            .filter(|s| s.channel == AffectChannel::Pleasure && s.magnitude != 1.0)
            .map(|s| s.magnitude)
            .collect();
        assert!(
            heights.iter().any(|m| (m - 0.5).abs() < 1e-9),
            "{heights:?}"
        );
        let capped = signals
            .iter()
            .filter(|s| s.channel == AffectChannel::Pleasure && s.magnitude == 1.0)
            .count();
        assert!(capped >= 2, "first contact and the capped height gain");
    }

    #[test]
    fn terminals_and_fence_end_the_attempt() {
        let mut dead = at(3, 0.0, 0, false, true);
        dead.terminal = Some(AttemptTerminal::Death);
        let mut samples = settled();
        samples.push(dead);
        let (record, _) = fold(GraspRule::Lift, &samples, true);
        assert_eq!(
            record.expect("finished").outcome,
            GraspAttemptOutcome::Death
        );
        let mut fenced = at(1, 0.0, 0, false, true);
        fenced.fence_violated = true;
        let (record, _) = fold(GraspRule::Lift, &[fenced], true);
        assert_eq!(
            record.expect("finished").outcome,
            GraspAttemptOutcome::Fence
        );
    }

    #[test]
    fn criterion_rejects_what_cannot_succeed() {
        let mut crit = criterion(GraspRule::GrabHold);
        crit.grasping_bodies.clear();
        assert!(crit.validate().is_err());
        let mut crit = criterion(GraspRule::GrabHold);
        crit.min_contacts = 0;
        assert!(crit.validate().is_err());
        crit.min_contacts = 3;
        assert!(crit.validate().is_err(), "more than the grasping bodies");
        let mut crit = criterion(GraspRule::Lift);
        crit.hold_s = 5.0;
        assert!(crit.burst_counts(HZ).is_err(), "hold longer than the cap");
        let defaults: GraspHoldCriterion = serde_json::from_str(
            r#"{"rule":"lift","target_object_id":"x","grasping_bodies":["a"],"hold_s":1.0,"attempt_burst_cap":50}"#,
        )
        .expect("defaults");
        assert_eq!(defaults.min_contacts, 1);
        assert_eq!(defaults.settle_s, 0.5);
        assert!(defaults.contact_grace_s.is_none() && defaults.lift_height_mm.is_none());
    }

    fn scripted_session(rule: GraspRule) -> (GraspHoldConfig, ScriptedProbe) {
        let train: Vec<GraspSample> = [
            at(1, 0.0, 0, false, true),
            at(2, 0.0, 0, false, true),
            at(3, 0.0, 1, false, true),
            at(4, 0.0, 1, false, true),
            at(5, 0.0, 1, false, true),
        ]
        .into_iter()
        .collect();
        let test: Vec<GraspSample> = train
            .iter()
            .cloned()
            .map(|mut s| {
                s.seed = 2;
                s
            })
            .collect();
        (
            GraspHoldConfig {
                criterion: criterion(rule),
                train_seeds: vec![1],
                test_seeds: vec![2],
                poll_timeout: Duration::from_millis(1),
                frequency_hz: HZ,
                steps_per_burst: STEPS,
            },
            ScriptedProbe {
                samples: train.into_iter().chain(test).collect(),
                requests: Vec::new(),
            },
        )
    }

    #[test]
    fn train_sends_affect_and_test_sends_none() {
        let (config, mut probe) = scripted_session(GraspRule::GrabHold);
        let mut writes: Vec<CoachWrite> = Vec::new();
        let mut events = CollectingEventSink::default();
        let report = run_grasp_hold(
            &config,
            &mut probe,
            &mut |write| {
                writes.push(write);
                Ok(())
            },
            &mut events,
            &RunId("grasp-affect".to_string()),
            &CancelToken::new(),
            None,
        )
        .expect("report");
        assert_eq!(report.train.outcomes.success, 1);
        assert_eq!(report.test.outcomes.success, 1);
        let affects = writes
            .iter()
            .filter(|w| matches!(w, CoachWrite::Affect(_)))
            .count();
        // The train phase pays first contact once; the test phase reads the same contact and
        // pays nothing.
        assert_eq!(affects, 1);
        let steps = writes.iter().filter(|w| **w == CoachWrite::Step).count();
        assert_eq!(steps, 10, "5 bursts per phase");
        let last_step = writes
            .iter()
            .rposition(|w| *w == CoachWrite::Step)
            .expect("step");
        assert!(
            writes[..last_step - 4]
                .iter()
                .any(|w| matches!(w, CoachWrite::Affect(_))),
            "the one affect came during training"
        );
    }

    #[test]
    fn session_scorecard_carries_dropped_and_knocked_off() {
        let record = |outcome| GraspAttemptRecord {
            seed: 1,
            outcome,
            bursts: 3,
            final_distance_mm: 4.0,
            max_height_mm: 12.0,
            resting_side: None,
            requested_side: None,
            placed_mm: None,
        };
        let report = GraspPhaseReport::from_attempts(vec![
            record(GraspAttemptOutcome::Dropped),
            record(GraspAttemptOutcome::KnockedOff),
            record(GraspAttemptOutcome::Success),
            record(GraspAttemptOutcome::Dropped),
        ])
        .expect("report");
        assert_eq!(report.outcomes.dropped, 2);
        assert_eq!(report.outcomes.knocked_off, 1);
        let card = grasp_phase_scorecard("s", "c", 1, "split", "asset", "hash", &report);
        assert_eq!(card.metrics["dropped"], 2.0);
        assert_eq!(card.metrics["knocked_off"], 1.0);
        assert_eq!(card.metrics["success_rate"], 0.25);
    }

    #[test]
    fn run_data_records_steps_and_attempts() {
        let (config, mut probe) = scripted_session(GraspRule::GrabHold);
        let mut data = crate::run_data::RecordingRunDataSink::default();
        let mut events = CollectingEventSink::default();
        run_grasp_hold(
            &config,
            &mut probe,
            &mut |_| Ok(()),
            &mut events,
            &RunId("grasp-data".to_string()),
            &CancelToken::new(),
            Some(&mut data),
        )
        .expect("report");
        assert_eq!(data.rows_of("attempts").len(), 2);
        let spec = grasp_attempts_spec();
        let names: Vec<&str> = spec.columns.iter().map(|c| c.name.as_str()).collect();
        assert!(names.ends_with(&["resting_side", "requested_side", "placed_x", "placed_y"]));
        assert_eq!(
            data.rows_of("attempts")[0].len(),
            spec.columns.len(),
            "row width matches the columns"
        );
        assert_eq!(data.rows_of("steps").len(), 10);
    }

    /// Settle on the support with `side` up, then fall `drop_mm` below rest.
    fn falls(side: UpExtent, drop_mm: f64) -> Option<GraspAttemptRecord> {
        let mut samples = settled();
        for s in &mut samples {
            s.up_extent = Some(side);
        }
        let mut fall = at(3, -drop_mm, 0, false, false);
        fall.up_extent = Some(side);
        samples.push(fall);
        fold(GraspRule::Lift, &samples, false).0
    }

    #[test]
    fn mixed_covers_all_three_sides_by_seed() {
        let sides: Vec<UpExtent> = (0..6)
            .map(|seed| RestingSide::Mixed.requested_for(seed))
            .collect();
        assert_eq!(
            sides,
            vec![
                UpExtent::Smallest,
                UpExtent::Middle,
                UpExtent::Largest,
                UpExtent::Smallest,
                UpExtent::Middle,
                UpExtent::Largest
            ]
        );
        assert_eq!(RestingSide::Flat.requested_for(1), UpExtent::Smallest);
        assert_eq!(RestingSide::Edge.requested_for(0), UpExtent::Middle);
        assert_eq!(RestingSide::End.requested_for(0), UpExtent::Largest);
    }

    #[test]
    fn knocked_off_threshold_follows_the_side() {
        // min 10 mm: smallest up -> rest - 10; middle (30) -> rest - 10 - 10; largest (90) -> rest - 40 - 10.
        assert_eq!(knocked_off_below(100.0, 10.0, 10.0), 90.0);
        assert_eq!(knocked_off_below(100.0, 30.0, 10.0), 80.0);
        assert_eq!(knocked_off_below(100.0, 90.0, 10.0), 50.0);
        // A 15 mm fall knocks a flat object off but not one on its edge or end.
        assert!(falls(UpExtent::Smallest, 15.0).is_some());
        assert!(falls(UpExtent::Middle, 15.0).is_none());
        assert!(falls(UpExtent::Largest, 15.0).is_none());
        assert!(falls(UpExtent::Middle, 25.0).is_some());
        assert!(falls(UpExtent::Largest, 45.0).is_none());
        assert!(falls(UpExtent::Largest, 55.0).is_some());
        assert_eq!(
            falls(UpExtent::Largest, 55.0).expect("finished").outcome,
            GraspAttemptOutcome::KnockedOff
        );
    }

    #[test]
    fn the_landed_side_is_recorded_beside_the_requested_one() {
        let mut samples = settled();
        for s in &mut samples {
            s.up_extent = Some(UpExtent::Middle); // it toppled onto an edge
        }
        samples.push(at(3, 30.0, 2, false, false));
        samples.push(at(4, 30.0, 2, false, false));
        samples.push(at(5, 30.0, 2, false, false));
        samples.push(at(6, 30.0, 2, false, false));
        let record = fold(GraspRule::Lift, &samples, false).0.expect("finished");
        assert_eq!(record.resting_side, Some(UpExtent::Middle));
        assert_eq!(record.requested_side, Some(UpExtent::Smallest));
        assert_eq!(record.placed_mm, Some([12.0, -7.0]));
    }

    #[test]
    fn begin_sends_spread_and_side_and_resolves_the_default_spread() {
        let (mut config, mut probe) = scripted_session(GraspRule::GrabHold);
        config.criterion.placement_spread_mm = None;
        config.criterion.resting_side = RestingSide::Mixed;
        run_grasp_hold(
            &config,
            &mut probe,
            &mut |_| Ok(()),
            &mut CollectingEventSink::default(),
            &RunId("grasp-place".to_string()),
            &CancelToken::new(),
            None,
        )
        .expect("report");
        // One begin per attempt, each at the object's largest extent.
        let spreads: Vec<f64> = probe.requests.iter().map(|(_, r)| r.spread_mm).collect();
        assert_eq!(spreads, vec![90.0, 90.0]);
        assert_eq!(probe.requests.len(), 2, "one begin per attempt");
        assert_eq!(probe.requests[0].1.up_extent, UpExtent::Middle); // seed 1
        assert_eq!(probe.requests[1].1.up_extent, UpExtent::Largest); // seed 2
    }

    #[test]
    fn criterion_rejects_a_bad_spread() {
        let mut bad = criterion(GraspRule::Lift);
        bad.placement_spread_mm = Some(-1.0);
        assert!(bad.validate().is_err());
        bad.placement_spread_mm = Some(f64::NAN);
        assert!(bad.validate().is_err());
    }
}
