//! One row per applied push: how far the body leaned in the push direction, how long it took to
//! settle, and whether it stayed up.
//!
//! A push is any applied pulse, scheduled (`body_push` event) or random (the perturbation plan),
//! counted when it is applied. Each push's window runs from its start to the next push or to the
//! end of the attempt. The lean is read from the signed torso-frame or world components the
//! controller reports, projected on the push direction, so a positive value always leans toward
//! where the push sent the body. The tracker is generic: it knows directions, not bodies.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::run_data::{ColumnSpec, RunDataValue, TableSpec};

/// Name of the table the tracker feeds.
pub const PUSHES_TABLE: &str = "pushes";

fn default_pre_window_s() -> f64 {
    1.0
}
fn default_settle_k() -> f64 {
    3.0
}
fn default_settle_floor_deg() -> f64 {
    0.5
}
fn default_settle_hold_s() -> f64 {
    0.5
}
fn default_survive_window_s() -> f64 {
    2.0
}

/// Task settings of the push table. Every default is dimensionless, degrees or seconds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PushTrackingConfig {
    /// Seconds before a push whose samples give the baseline lean and its spread.
    pub pre_window_s: f64,
    /// The body has settled when its lean stays within `k` times the baseline spread.
    pub settle_k: f64,
    /// Smallest settle band, degrees, so a very steady baseline does not demand zero error.
    pub settle_floor_deg: f64,
    /// Seconds the lean must stay inside the settle band to count as settled.
    pub settle_hold_s: f64,
    /// Seconds after a push an attempt must still run for the push to count as survived.
    pub survive_window_s: f64,
}

impl Default for PushTrackingConfig {
    fn default() -> Self {
        Self {
            pre_window_s: default_pre_window_s(),
            settle_k: default_settle_k(),
            settle_floor_deg: default_settle_floor_deg(),
            settle_hold_s: default_settle_hold_s(),
            survive_window_s: default_survive_window_s(),
        }
    }
}

impl PushTrackingConfig {
    /// Rejects a non-finite or non-positive setting.
    pub fn validate(&self) -> Result<(), String> {
        for (name, value) in [
            ("push_tracking.pre_window_s", self.pre_window_s),
            ("push_tracking.settle_k", self.settle_k),
            ("push_tracking.settle_floor_deg", self.settle_floor_deg),
            ("push_tracking.settle_hold_s", self.settle_hold_s),
            ("push_tracking.survive_window_s", self.survive_window_s),
        ] {
            if !value.is_finite() || value <= 0.0 {
                return Err(format!("{name} must be finite and > 0"));
            }
        }
        Ok(())
    }
}

/// Signed lean of one sample in degrees: world components and torso-frame components.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Lean {
    /// Tilt vector component on the first horizontal world axis.
    pub x: Option<f64>,
    /// Tilt vector component on the second horizontal world axis.
    pub y: Option<f64>,
    /// Lean toward the torso's forward.
    pub forward: Option<f64>,
    /// Lean toward the torso's left.
    pub left: Option<f64>,
}

/// Direction a push sends the body, as the coefficients that project a lean on it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PushDir {
    /// Coefficients on `(forward, left)`.
    Torso([f64; 2]),
    /// Coefficients on the tilt-vector components `(x, y)`.
    World([f64; 2]),
}

impl PushDir {
    /// Direction of a push toward the torso's forward, back, left or right.
    pub fn torso(forward: f64, left: f64) -> Self {
        Self::Torso([forward, left])
    }

    /// Direction of a torque `[tx, ty]` about the world horizontal axes. A positive torque about
    /// an axis leans the body so the tilt vector reads `-t` on the same component, so the push
    /// direction on `(x, y)` is `-t / |t|`.
    pub fn from_world_torque(torque: [f64; 2]) -> Self {
        let norm = torque[0].hypot(torque[1]);
        if norm <= 0.0 {
            return Self::World([0.0, 0.0]);
        }
        Self::World([-torque[0] / norm, -torque[1] / norm])
    }

    /// Lean toward the push direction; `None` when the sample lacks a needed component.
    pub fn signal(&self, lean: &Lean) -> Option<f64> {
        let (c, a, b) = match self {
            Self::Torso(c) => (c, lean.forward, lean.left),
            Self::World(c) => (c, lean.x, lean.y),
        };
        Some(c[0] * a? + c[1] * b?)
    }
}

/// What the Trainer knows about a push at the moment it applies it.
#[derive(Debug, Clone, PartialEq)]
pub struct PushSpec {
    /// `scheduled` or `random`.
    pub source: &'static str,
    /// `torque` or `force`.
    pub kind: String,
    /// Axis name as the push was written.
    pub axis: String,
    /// Fraction of body weight; `None` for a random push.
    pub magnitude_fraction: Option<f64>,
    /// N or N*m.
    pub magnitude: f64,
    /// Seconds the pulse lasts.
    pub duration_s: f64,
    /// Direction the push sends the body.
    pub dir: PushDir,
}

/// One finished push row.
#[derive(Debug, Clone, PartialEq)]
pub struct PushRecord {
    /// What was applied.
    pub spec: PushSpec,
    /// Simulation time the push was applied.
    pub at_s: f64,
    /// Mean lean toward the push direction over the pre-window.
    pub pre_tilt_mean: Option<f64>,
    /// Standard deviation of that lean over the pre-window.
    pub pre_tilt_spread: Option<f64>,
    /// Largest lean change toward the push direction in the window.
    pub peak_dtilt: Option<f64>,
    /// Seconds after the push at which the peak happened.
    pub peak_time_s: Option<f64>,
    /// Seconds after the push at which the lean entered its settle band for good.
    pub push_recovery_s: Option<f64>,
    /// The body stayed up through the window; `None` when the attempt ended inside it.
    pub survived: Option<bool>,
    /// Seconds after the push at which the attempt ended in a fall.
    pub fall_s: Option<f64>,
}

/// Columns of the `pushes` table.
pub fn pushes_table_spec() -> TableSpec {
    TableSpec {
        name: PUSHES_TABLE.to_string(),
        description: "One row per applied push, scheduled or random. The window of a push ends at the next push or the end of the attempt.".to_string(),
        columns: vec![
            ColumnSpec::unitless("phase", "Phase the attempt belongs to."),
            ColumnSpec::unitless("attempt", "Attempt index within the phase."),
            ColumnSpec::unitless("source", "scheduled (a body_push event) or random (the perturbation plan)."),
            ColumnSpec::with_unit("at_s", "s", "Simulation time the push was applied."),
            ColumnSpec::unitless("kind", "torque or force."),
            ColumnSpec::unitless("axis", "Push axis: forward, back, left or right for a scheduled push; the signed world torque axis for a random one."),
            ColumnSpec::unitless("magnitude_fraction", "Push size as a fraction of body weight. Empty for a random push."),
            ColumnSpec::with_unit("magnitude", "N or N*m", "Push size: newtons for a force, newton-metres for a torque."),
            ColumnSpec::with_unit("duration_s", "s", "How long the pulse lasts."),
            ColumnSpec::with_unit("pre_tilt_mean", "deg", "Mean lean toward the push direction over the pre-window."),
            ColumnSpec::with_unit("pre_tilt_spread", "deg", "Standard deviation of that lean over the pre-window."),
            ColumnSpec::with_unit("peak_dtilt", "deg", "Largest lean change toward the push direction in the window (positive = leaned the way it was pushed)."),
            ColumnSpec::with_unit("peak_time_s", "s", "Seconds after the push at which the peak lean change happened."),
            ColumnSpec::with_unit("push_recovery_s", "s", "Seconds after the push until the lean stayed within the settle band for the hold window. 0 when the lean never left the band; empty when it never settled in the window."),
            ColumnSpec::unitless("survived", "The body stayed up through the window. Empty when the attempt ended inside the survive window."),
            ColumnSpec::with_unit("fall_s", "s", "Seconds after the push at which the attempt ended in a fall. Empty when it did not fall."),
        ],
    }
}

impl PushRecord {
    /// The table row for this push.
    pub fn row(&self, phase: &str, attempt: usize) -> Vec<RunDataValue> {
        vec![
            RunDataValue::text(phase),
            RunDataValue::count(attempt as u64),
            RunDataValue::text(self.spec.source),
            RunDataValue::Float(self.at_s),
            RunDataValue::text(&self.spec.kind),
            RunDataValue::text(&self.spec.axis),
            RunDataValue::opt_float(self.spec.magnitude_fraction),
            RunDataValue::Float(self.spec.magnitude),
            RunDataValue::Float(self.spec.duration_s),
            RunDataValue::opt_float(self.pre_tilt_mean),
            RunDataValue::opt_float(self.pre_tilt_spread),
            RunDataValue::opt_float(self.peak_dtilt),
            RunDataValue::opt_float(self.peak_time_s),
            RunDataValue::opt_float(self.push_recovery_s),
            self.survived.map_or(RunDataValue::Null, RunDataValue::Bool),
            RunDataValue::opt_float(self.fall_s),
        ]
    }
}

struct OpenPush {
    spec: PushSpec,
    start_s: f64,
    baseline: Option<(f64, f64)>,
    peak: Option<(f64, f64)>,
    excursion: bool,
    in_band_since: Option<f64>,
    recovery_s: Option<f64>,
}

/// Records the pushes of one attempt.
pub struct PushTracker {
    cfg: PushTrackingConfig,
    history: VecDeque<(f64, Lean)>,
    open: Option<OpenPush>,
    done: Vec<PushRecord>,
    applied: u32,
}

impl PushTracker {
    /// Tracker for one attempt.
    pub fn new(cfg: PushTrackingConfig) -> Self {
        Self {
            cfg,
            history: VecDeque::new(),
            open: None,
            done: Vec::new(),
            applied: 0,
        }
    }

    /// Pushes applied so far, counted when applied.
    pub fn applied(&self) -> u32 {
        self.applied
    }

    /// A push is applied at `at_s`. Closes the previous push's window; samples seen so far give
    /// the baseline.
    pub fn begin_push(&mut self, at_s: f64, spec: PushSpec) {
        self.close(at_s, None);
        self.applied = self.applied.saturating_add(1);
        let signals: Vec<f64> = self
            .history
            .iter()
            .filter(|(t, _)| *t < at_s && at_s - *t <= self.cfg.pre_window_s)
            .filter_map(|(_, lean)| spec.dir.signal(lean))
            .collect();
        let baseline = if signals.is_empty() {
            None
        } else {
            let n = signals.len() as f64;
            let mean = signals.iter().sum::<f64>() / n;
            let var = signals.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / n;
            Some((mean, var.sqrt()))
        };
        self.open = Some(OpenPush {
            spec,
            start_s: at_s,
            baseline,
            peak: None,
            excursion: false,
            in_band_since: None,
            recovery_s: None,
        });
    }

    /// One sample at simulation time `t_s`.
    pub fn observe(&mut self, t_s: f64, lean: Lean) {
        self.history.push_back((t_s, lean));
        while self
            .history
            .front()
            .is_some_and(|(t, _)| t_s - *t > self.cfg.pre_window_s)
        {
            self.history.pop_front();
        }
        let Some(open) = self.open.as_mut() else {
            return;
        };
        let Some(signal) = open.spec.dir.signal(&lean) else {
            return;
        };
        let (mean, spread) = *open.baseline.get_or_insert((signal, 0.0));
        let d = signal - mean;
        if open.peak.is_none_or(|(p, _)| d > p) {
            open.peak = Some((d, t_s - open.start_s));
        }
        let band = (self.cfg.settle_k * spread).max(self.cfg.settle_floor_deg);
        if d.abs() > band {
            open.excursion = true;
            open.in_band_since = None;
        } else if open.excursion {
            let since = *open.in_band_since.get_or_insert(t_s);
            if open.recovery_s.is_none() && t_s - since >= self.cfg.settle_hold_s {
                open.recovery_s = Some(since - open.start_s);
            }
        }
    }

    /// The attempt ended at `end_s`; `fell` when it ended in a fall. Returns every row.
    pub fn finish(mut self, end_s: f64, fell: bool) -> Vec<PushRecord> {
        self.close(end_s, Some(fell));
        self.done
    }

    /// `attempt_end` is `None` when the next push closes the window.
    fn close(&mut self, end_s: f64, attempt_end: Option<bool>) {
        let Some(open) = self.open.take() else {
            return;
        };
        let elapsed = end_s - open.start_s;
        let fell = attempt_end == Some(true);
        let survived = match attempt_end {
            None => Some(true),
            Some(true) => Some(false),
            Some(false) => (elapsed >= self.cfg.survive_window_s).then_some(true),
        };
        let recovery = open.recovery_s.or_else(|| {
            (!open.excursion && open.peak.is_some() && elapsed >= self.cfg.settle_hold_s)
                .then_some(0.0)
        });
        self.done.push(PushRecord {
            at_s: open.start_s,
            pre_tilt_mean: open.baseline.map(|b| b.0),
            pre_tilt_spread: open.baseline.map(|b| b.1),
            peak_dtilt: open.peak.map(|p| p.0),
            peak_time_s: open.peak.map(|p| p.1),
            push_recovery_s: recovery,
            survived,
            fall_s: fell.then_some(elapsed),
            spec: open.spec,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fwd(deg: f64) -> Lean {
        Lean {
            forward: Some(deg),
            left: Some(0.0),
            ..Lean::default()
        }
    }

    fn spec(dir: PushDir) -> PushSpec {
        PushSpec {
            source: "scheduled",
            kind: "torque".into(),
            axis: "forward".into(),
            magnitude_fraction: Some(0.05),
            magnitude: 1.0,
            duration_s: 0.1,
            dir,
        }
    }

    fn tracker_with_baseline() -> PushTracker {
        let mut t = PushTracker::new(PushTrackingConfig::default());
        for i in 0..10 {
            t.observe(i as f64 * 0.1, fwd(1.0));
        }
        t
    }

    #[test]
    fn windows_end_at_the_next_push_and_a_fall_goes_to_the_latest() {
        let mut t = tracker_with_baseline();
        t.begin_push(1.0, spec(PushDir::torso(1.0, 0.0)));
        t.observe(1.1, fwd(4.0));
        t.begin_push(2.0, spec(PushDir::torso(1.0, 0.0)));
        t.observe(2.1, fwd(9.0));
        let rows = t.finish(2.5, true);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].survived, Some(true));
        assert_eq!(rows[0].fall_s, None);
        assert_eq!(rows[0].peak_dtilt, Some(3.0));
        assert_eq!(rows[1].survived, Some(false));
        assert!((rows[1].fall_s.unwrap() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn peak_is_signed_in_the_push_direction() {
        let mut t = tracker_with_baseline();
        t.begin_push(1.0, spec(PushDir::torso(-1.0, 0.0)));
        t.observe(1.1, fwd(4.0));
        t.observe(1.2, fwd(-2.0));
        let rows = t.finish(3.5, false);
        // Pushed back: leaning forward 3 deg more is a negative change; -3 deg is the peak.
        assert_eq!(rows[0].peak_dtilt, Some(3.0));
        assert!((rows[0].peak_time_s.unwrap() - 0.2).abs() < 1e-9);
        assert_eq!(rows[0].pre_tilt_mean, Some(-1.0));
        assert_eq!(rows[0].survived, Some(true));
    }

    #[test]
    fn recovery_is_empty_when_the_lean_never_settles_and_survival_empty_when_cut_short() {
        let mut t = tracker_with_baseline();
        t.begin_push(1.0, spec(PushDir::torso(1.0, 0.0)));
        for i in 1..6 {
            t.observe(1.0 + i as f64 * 0.1, fwd(8.0));
        }
        let rows = t.finish(1.6, false);
        assert_eq!(rows[0].push_recovery_s, None);
        assert_eq!(rows[0].survived, None);
    }

    #[test]
    fn recovery_is_the_start_of_the_settled_hold() {
        let mut t = tracker_with_baseline();
        t.begin_push(1.0, spec(PushDir::torso(1.0, 0.0)));
        t.observe(1.1, fwd(6.0));
        for i in 2..12 {
            t.observe(1.0 + i as f64 * 0.1, fwd(1.1));
        }
        let rows = t.finish(4.0, false);
        assert!((rows[0].push_recovery_s.unwrap() - 0.2).abs() < 1e-9);
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn world_torque_direction_matches_the_tilt_vector_convention() {
        // A torque about +Y leans toward +X, which reads negative on the y component.
        let dir = PushDir::from_world_torque([0.0, 0.3]);
        let lean = Lean {
            x: Some(0.0),
            y: Some(-5.0),
            ..Lean::default()
        };
        assert_eq!(dir.signal(&lean), Some(5.0));
    }
}
