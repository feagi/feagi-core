//! One row per applied body vibration segment: how strongly the body swayed at the drive
//! frequency (gain and phase lag), the same measure over a quiet stretch just before it, and
//! whether the body stayed up.
//!
//! A segment is a `body_vibration` event: a sinusoidal wrench the controller computes from sim
//! time at every physics step, with a one-cycle ramp in and out. The sway is the lean projected
//! on the drive direction (the shared [`PushDir`] projection, in degrees). Its amplitude and
//! phase at the drive frequency come from a single-bin DFT over WHOLE drive cycles after a
//! settle of [`SETTLE_CYCLES`] cycles and before the ramp-out, so neither ramp leaks into the
//! bin. The floor is the same bin over a window of the same length ending where the segment
//! starts, so gain and floor have the same resolution. The tracker is generic: it knows
//! directions and frequencies, not bodies.

use crate::push_tracker::{Lean, PushDir};
use crate::run_data::{ColumnSpec, RunDataValue, TableSpec};

/// Name of the per-segment table.
pub const VIBRATION_TABLE: &str = "vibration";
/// Name of the per-attempt summary table.
pub const VIBRATION_SUMMARY_TABLE: &str = "vibration_summary";

/// Cycles the sway settles before the analysis window starts (the ramp-in is the first of them).
pub const SETTLE_CYCLES: f64 = 2.0;
/// Cycles of the linear ramp in and out, as the controller applies them.
pub const RAMP_CYCLES: f64 = 1.0;
/// Fewest cycles a segment can have: the settle, one whole analysis cycle, and the ramp-out.
pub const MIN_CYCLES: u32 = 4;
/// Highest drive frequency as a fraction of the sample rate. The Trainer reads one sample per
/// burst, so a faster drive is aliased and its bin means nothing.
pub const MAX_FREQUENCY_FRACTION_OF_SAMPLE_RATE: f64 = 0.1;

const EPS_S: f64 = 1.0e-9;

/// Rejects a segment the sample rate cannot resolve or that is too short to analyse.
/// `sample_period_s` is the time between samples (the burst period).
pub fn check_segment(
    label: &str,
    frequency_hz: f64,
    cycles: u32,
    sample_period_s: f64,
) -> Result<(), String> {
    if !frequency_hz.is_finite() || frequency_hz <= 0.0 {
        return Err(format!(
            "scheduled event {label:?}: frequency_hz must be finite and > 0"
        ));
    }
    if cycles < MIN_CYCLES {
        return Err(format!(
            "scheduled event {label:?}: cycles must be at least {MIN_CYCLES} (a {SETTLE_CYCLES} cycle settle, one analysed cycle and the ramp-out)"
        ));
    }
    if sample_period_s.is_finite() && sample_period_s > 0.0 {
        let limit = MAX_FREQUENCY_FRACTION_OF_SAMPLE_RATE / sample_period_s;
        if frequency_hz > limit + 1.0e-12 {
            return Err(format!(
                "scheduled event {label:?}: frequency_hz {frequency_hz} is above {limit:.3} Hz, {MAX_FREQUENCY_FRACTION_OF_SAMPLE_RATE} of the sample rate (one sample per burst of {sample_period_s} s)"
            ));
        }
    }
    Ok(())
}

/// What the Trainer knows about a vibration segment at the moment the controller starts it.
#[derive(Debug, Clone, PartialEq)]
pub struct VibrationSpec {
    /// `torque` or `force`.
    pub kind: String,
    /// Axis name as the event was written.
    pub axis: String,
    /// Fraction of body weight (force) or of weight times height (torque).
    pub magnitude_fraction: f64,
    /// Drive amplitude in N or N*m.
    pub magnitude: f64,
    /// Drive frequency in hertz.
    pub frequency_hz: f64,
    /// Whole drive cycles the segment lasts, ramps included.
    pub cycles: u32,
    /// Direction the drive tips the body, for projecting the lean.
    pub dir: PushDir,
    /// Simulation time of the first applied physics step, from the controller's reply.
    pub start_s: f64,
}

impl VibrationSpec {
    fn end_s(&self) -> f64 {
        self.start_s + f64::from(self.cycles) / self.frequency_hz
    }
}

/// One finished segment row.
#[derive(Debug, Clone, PartialEq)]
pub struct VibrationRecord {
    /// What was applied.
    pub spec: VibrationSpec,
    /// Lean amplitude at the drive frequency divided by the drive amplitude (deg per N*m or N).
    pub gain: Option<f64>,
    /// Degrees the lean lags the drive at the drive frequency (positive = lags).
    pub phase_lag_deg: Option<f64>,
    /// The same bin over a quiet window just before the segment, divided by the same drive.
    pub gain_floor: Option<f64>,
    /// Standard deviation of the lean over the analysis window, degrees.
    pub tilt_rms: Option<f64>,
    /// The body stayed up through the segment and the survive window; `None` when the attempt
    /// ended inside it without a fall.
    pub survived: Option<bool>,
    /// Seconds after the segment start at which the attempt ended in a fall during it.
    pub fall_s: Option<f64>,
}

/// One row of the per-attempt summary: what a frequency survived.
#[derive(Debug, Clone, PartialEq)]
pub struct VibrationSummaryRow {
    /// Drive frequency in hertz.
    pub frequency_hz: f64,
    /// Largest drive amplitude (N or N*m) of a segment at this frequency the body survived.
    pub max_magnitude_survived: Option<f64>,
    /// Drive amplitude of the first segment at this frequency that ended in a fall.
    pub first_fall_magnitude: Option<f64>,
}

/// Columns of the `vibration` table.
pub fn vibration_table_spec() -> TableSpec {
    TableSpec {
        name: VIBRATION_TABLE.to_string(),
        description: "One row per body vibration segment (a body_vibration event). Gain and phase come from a single-bin DFT of the lean on the drive direction over whole drive cycles after a 2 cycle settle, excluding the ramps.".to_string(),
        columns: vec![
            ColumnSpec::unitless("phase", "Phase the attempt belongs to."),
            ColumnSpec::unitless("attempt", "Attempt index within the phase."),
            ColumnSpec::with_unit("start_sim_s", "s", "Simulation time of the first applied physics step, from the controller."),
            ColumnSpec::unitless("kind", "torque or force."),
            ColumnSpec::unitless("axis", "Drive axis in the body's heading frame: forward, back, left or right."),
            ColumnSpec::with_unit("frequency_hz", "Hz", "Drive frequency."),
            ColumnSpec::unitless("magnitude_fraction", "Drive amplitude as a fraction of body weight (force) or of weight times the torso's reset height (torque)."),
            ColumnSpec::with_unit("magnitude", "N or N*m", "Drive amplitude: newtons for a force, newton-metres for a torque."),
            ColumnSpec::unitless("cycles", "Drive cycles the segment lasts, ramp in and out included."),
            ColumnSpec::with_unit("gain", "deg per N or N*m", "Lean amplitude at the drive frequency divided by the drive amplitude. Empty when the analysis window was not fully sampled."),
            ColumnSpec::with_unit("phase_lag_deg", "deg", "Degrees the lean lags the drive at the drive frequency (positive = lags)."),
            ColumnSpec::with_unit("gain_floor", "deg per N or N*m", "The same bin over a quiet window of the same length just before the segment, divided by the same drive. Empty when no quiet window of that length exists (attempt start, the settle window or an earlier segment is in the way)."),
            ColumnSpec::with_unit("tilt_rms", "deg", "Standard deviation of the lean over the analysis window."),
            ColumnSpec::unitless("survived", "The body stayed up through the segment and the survive window. Empty when the attempt ended inside it without a fall."),
            ColumnSpec::with_unit("fall_s", "s", "Seconds after the segment start at which the attempt ended in a fall. Empty when it did not fall."),
        ],
    }
}

/// Columns of the `vibration_summary` table.
pub fn vibration_summary_table_spec() -> TableSpec {
    TableSpec {
        name: VIBRATION_SUMMARY_TABLE.to_string(),
        description: "One row per attempt and drive frequency: the largest vibration the body survived and the first one that made it fall.".to_string(),
        columns: vec![
            ColumnSpec::unitless("phase", "Phase the attempt belongs to."),
            ColumnSpec::unitless("attempt", "Attempt index within the phase."),
            ColumnSpec::with_unit("frequency_hz", "Hz", "Drive frequency."),
            ColumnSpec::with_unit("max_magnitude_survived", "N or N*m", "Largest drive amplitude at this frequency the body survived. Empty when none was survived."),
            ColumnSpec::with_unit("first_fall_magnitude", "N or N*m", "Drive amplitude of the first segment at this frequency that ended in a fall. Empty when none did."),
        ],
    }
}

impl VibrationRecord {
    /// The `vibration` table row for this segment.
    pub fn row(&self, phase: &str, attempt: usize) -> Vec<RunDataValue> {
        vec![
            RunDataValue::text(phase),
            RunDataValue::count(attempt as u64),
            RunDataValue::Float(self.spec.start_s),
            RunDataValue::text(&self.spec.kind),
            RunDataValue::text(&self.spec.axis),
            RunDataValue::Float(self.spec.frequency_hz),
            RunDataValue::Float(self.spec.magnitude_fraction),
            RunDataValue::Float(self.spec.magnitude),
            RunDataValue::count(u64::from(self.spec.cycles)),
            RunDataValue::opt_float(self.gain),
            RunDataValue::opt_float(self.phase_lag_deg),
            RunDataValue::opt_float(self.gain_floor),
            RunDataValue::opt_float(self.tilt_rms),
            self.survived.map_or(RunDataValue::Null, RunDataValue::Bool),
            RunDataValue::opt_float(self.fall_s),
        ]
    }
}

impl VibrationSummaryRow {
    /// The `vibration_summary` table row.
    pub fn row(&self, phase: &str, attempt: usize) -> Vec<RunDataValue> {
        vec![
            RunDataValue::text(phase),
            RunDataValue::count(attempt as u64),
            RunDataValue::Float(self.frequency_hz),
            RunDataValue::opt_float(self.max_magnitude_survived),
            RunDataValue::opt_float(self.first_fall_magnitude),
        ]
    }
}

/// Per-frequency summary of one attempt's records, in order of first appearance.
pub fn summarize(records: &[VibrationRecord]) -> Vec<VibrationSummaryRow> {
    let mut rows: Vec<VibrationSummaryRow> = Vec::new();
    let mut ordered: Vec<&VibrationRecord> = records.iter().collect();
    ordered.sort_by(|a, b| a.spec.start_s.total_cmp(&b.spec.start_s));
    for record in ordered {
        let at = match rows
            .iter()
            .position(|row| row.frequency_hz == record.spec.frequency_hz)
        {
            Some(at) => at,
            None => {
                rows.push(VibrationSummaryRow {
                    frequency_hz: record.spec.frequency_hz,
                    max_magnitude_survived: None,
                    first_fall_magnitude: None,
                });
                rows.len() - 1
            }
        };
        let row = &mut rows[at];
        match record.survived {
            Some(true) => {
                row.max_magnitude_survived = Some(
                    row.max_magnitude_survived
                        .map_or(record.spec.magnitude, |m| m.max(record.spec.magnitude)),
                );
            }
            Some(false) => {
                row.first_fall_magnitude
                    .get_or_insert(record.spec.magnitude);
            }
            None => {}
        }
    }
    rows
}

/// Result of one single-bin DFT.
#[derive(Debug, Clone, Copy)]
struct Bin {
    amplitude: f64,
    /// Phase of the response relative to `sin(2 pi f (t - t_ref))`, radians.
    phase_rad: f64,
    std: f64,
}

/// Records the vibration segments of one attempt.
pub struct VibrationTracker {
    survive_window_s: f64,
    /// Post-settle samples: simulation time and lean.
    samples: Vec<(f64, Lean)>,
    segments: Vec<VibrationSpec>,
}

impl VibrationTracker {
    /// Tracker for one attempt. A segment survives when the body is up `survive_window_s` after
    /// it ends.
    pub fn new(survive_window_s: f64) -> Self {
        Self {
            survive_window_s,
            samples: Vec::new(),
            segments: Vec::new(),
        }
    }

    /// Segments started so far, counted when the controller accepted them.
    pub fn applied(&self) -> u32 {
        self.segments.len() as u32
    }

    /// The controller started a segment.
    pub fn begin(&mut self, spec: VibrationSpec) {
        self.segments.push(spec);
    }

    /// One sample at simulation time `t_s`. Samples taken while the support settle window runs
    /// are not kept: that is not the body's own stance.
    pub fn observe(&mut self, t_s: f64, lean: Lean, settling: bool) {
        if !settling {
            self.samples.push((t_s, lean));
        }
    }

    /// The attempt ended at `end_s`; `fell` when it ended in a fall. Returns every row.
    pub fn finish(self, end_s: f64, fell: bool) -> Vec<VibrationRecord> {
        let mut order: Vec<usize> = (0..self.segments.len()).collect();
        order.sort_by(|&a, &b| {
            self.segments[a]
                .start_s
                .total_cmp(&self.segments[b].start_s)
        });
        let mut out = Vec::with_capacity(order.len());
        for (rank, &i) in order.iter().enumerate() {
            let spec = &self.segments[i];
            let next_start = order
                .get(rank + 1)
                .map(|&n| self.segments[n].start_s)
                .unwrap_or(f64::INFINITY);
            let window_end = (spec.end_s() + self.survive_window_s).min(next_start);
            let (survived, fall_s) = if fell {
                if end_s <= window_end + EPS_S {
                    (Some(false), Some((end_s - spec.start_s).max(0.0)))
                } else {
                    (Some(true), None)
                }
            } else if end_s + EPS_S >= window_end {
                (Some(true), None)
            } else {
                (None, None)
            };
            let signals: Vec<(f64, f64)> = self
                .samples
                .iter()
                .filter_map(|(t, lean)| spec.dir.signal(lean).map(|s| (*t, s)))
                .collect();
            let cycle_s = 1.0 / spec.frequency_hz;
            let whole = f64::from(spec.cycles) - SETTLE_CYCLES - RAMP_CYCLES;
            let w0 = spec.start_s + SETTLE_CYCLES * cycle_s;
            let w1 = w0 + whole * cycle_s;
            let drive = spec.magnitude;
            let main = bin(&signals, spec.frequency_hz, spec.start_s, w0, w1);
            let floor_w0 = spec.start_s - whole * cycle_s;
            let floor_clear = self.segments.iter().enumerate().all(|(j, other)| {
                j == i || other.start_s >= spec.start_s || other.end_s() <= floor_w0 + EPS_S
            });
            let floor = if floor_clear {
                bin(
                    &signals,
                    spec.frequency_hz,
                    floor_w0,
                    floor_w0,
                    spec.start_s,
                )
            } else {
                None
            };
            let lag = main.map(|b| {
                let deg = (-b.phase_rad).to_degrees();
                (deg + 180.0).rem_euclid(360.0) - 180.0
            });
            out.push(VibrationRecord {
                spec: spec.clone(),
                gain: main.map(|b| b.amplitude / drive),
                phase_lag_deg: lag,
                gain_floor: floor.map(|b| b.amplitude / drive),
                tilt_rms: main.map(|b| b.std),
                survived,
                fall_s,
            });
        }
        out
    }
}

/// Single-bin DFT at `frequency_hz` of the samples with `w0 <= t < w1`, phase measured against
/// `sin(2 pi f (t - t_ref))`. `None` when the window is not fully and evenly sampled (fewer than
/// four samples per cycle, or a gap of more than one and a half sample spacings at either end).
fn bin(signals: &[(f64, f64)], frequency_hz: f64, t_ref: f64, w0: f64, w1: f64) -> Option<Bin> {
    if w1 <= w0 {
        return None;
    }
    let window: Vec<(f64, f64)> = signals
        .iter()
        .copied()
        .filter(|(t, _)| *t >= w0 - EPS_S && *t < w1 - EPS_S)
        .collect();
    let n = window.len();
    let cycles = (w1 - w0) * frequency_hz;
    if (n as f64) < 4.0 * cycles || n < 2 {
        return None;
    }
    let spacing = (window[n - 1].0 - window[0].0) / (n as f64 - 1.0);
    if window[0].0 - w0 > 1.5 * spacing || w1 - window[n - 1].0 > 1.5 * spacing {
        return None;
    }
    let mean = window.iter().map(|(_, x)| x).sum::<f64>() / n as f64;
    let omega = 2.0 * std::f64::consts::PI * frequency_hz;
    let (mut s, mut c, mut ss) = (0.0, 0.0, 0.0);
    for (t, x) in &window {
        let d = x - mean;
        let angle = omega * (t - t_ref);
        s += d * angle.sin();
        c += d * angle.cos();
        ss += d * d;
    }
    let scale = 2.0 / n as f64;
    Some(Bin {
        amplitude: scale * s.hypot(c),
        phase_rad: c.atan2(s),
        std: (ss / n as f64).sqrt(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f64 = 0.005;

    fn spec(frequency_hz: f64, cycles: u32, magnitude: f64, start_s: f64) -> VibrationSpec {
        VibrationSpec {
            kind: "torque".into(),
            axis: "forward".into(),
            magnitude_fraction: 0.01,
            magnitude,
            frequency_hz,
            cycles,
            dir: PushDir::torso(1.0, 0.0),
            start_s,
        }
    }

    fn lean(deg: f64) -> Lean {
        Lean {
            forward: Some(deg),
            left: Some(0.0),
            ..Lean::default()
        }
    }

    /// Deterministic noise in [-1, 1).
    fn noise(state: &mut u64) -> f64 {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((*state >> 33) as f64 / (1u64 << 31) as f64) * 2.0 - 1.0
    }

    #[test]
    fn dft_recovers_amplitude_and_lag_of_a_tone_in_noise() {
        let f = 1.0;
        let amp = 2.0;
        let lag = 30.0_f64.to_radians();
        let mut state = 7;
        let signals: Vec<(f64, f64)> = (0..3000)
            .map(|k| {
                let t = k as f64 * DT;
                let x = amp * (2.0 * std::f64::consts::PI * f * t - lag).sin()
                    + 0.3 * noise(&mut state);
                (t, x)
            })
            .collect();
        let b = bin(&signals, f, 0.0, 2.0, 9.0).unwrap();
        assert!(
            (b.amplitude - amp).abs() < 0.05,
            "amplitude {}",
            b.amplitude
        );
        assert!((-b.phase_rad - lag).abs() < 0.05, "phase {}", b.phase_rad);
    }

    #[test]
    fn whole_cycle_window_ignores_the_ramps() {
        // The drive is 1 Hz, 10 cycles from t = 5 s. The lean is a clean 1 deg tone inside the
        // analysis window (7 s .. 14 s) and a huge, different signal during the ramps.
        let mut tracker = VibrationTracker::new(2.0);
        tracker.begin(spec(1.0, 10, 0.5, 5.0));
        for k in 0..4000 {
            let t = k as f64 * DT;
            let x = if (1000..1400).contains(&k) || (2800..3000).contains(&k) {
                50.0
            } else {
                (2.0 * std::f64::consts::PI * (t - 5.0)).sin()
            };
            tracker.observe(t, lean(x), false);
        }
        let rows = tracker.finish(20.0, false);
        let r = &rows[0];
        assert!((r.gain.unwrap() - 2.0).abs() < 0.02, "gain {:?}", r.gain);
        assert!(
            r.phase_lag_deg.unwrap().abs() < 1.0,
            "lag {:?}",
            r.phase_lag_deg
        );
        assert!((r.tilt_rms.unwrap() - (0.5_f64).sqrt()).abs() < 0.01);
        assert_eq!(r.survived, Some(true));
        // Quiet window [-2, 5) is not fully sampled: the attempt started at 0.
        assert_eq!(r.gain_floor, None);
    }

    #[test]
    fn gain_floor_uses_a_quiet_window_of_the_same_length_and_skips_an_earlier_segment() {
        let mut tracker = VibrationTracker::new(2.0);
        tracker.begin(spec(1.0, 7, 1.0, 10.0)); // whole = 4 cycles: floor [6, 10)
        tracker.begin(spec(1.0, 7, 1.0, 13.0)); // floor [9, 13) overlaps the first (10..17)
        let mut t = 0.0;
        while t < 25.0 {
            // A weak 0.1 deg tone at the drive frequency all along.
            let x = 0.1 * (2.0 * std::f64::consts::PI * t).sin();
            tracker.observe(t, lean(x), false);
            t += DT;
        }
        let rows = tracker.finish(25.0, false);
        assert!((rows[0].gain_floor.unwrap() - 0.1).abs() < 0.005);
        assert_eq!(rows[1].gain_floor, None);
    }

    #[test]
    fn settle_samples_are_not_kept() {
        let mut tracker = VibrationTracker::new(2.0);
        tracker.begin(spec(1.0, 7, 1.0, 10.0));
        for k in 0..4000 {
            let t = k as f64 * DT;
            tracker.observe(t, lean(0.0), t < 8.0);
        }
        // The floor window [6, 10) overlaps the settle, so there is no floor.
        assert_eq!(tracker.finish(20.0, false)[0].gain_floor, None);
    }

    #[test]
    fn a_fall_goes_to_the_segment_it_happened_in() {
        let mut tracker = VibrationTracker::new(2.0);
        tracker.begin(spec(2.0, 8, 0.1, 5.0)); // 5 .. 9, window to 11
        tracker.begin(spec(2.0, 8, 0.2, 12.0)); // 12 .. 16
        let mut t = 0.0;
        while t < 14.0 {
            tracker.observe(t, lean(0.0), false);
            t += DT;
        }
        let rows = tracker.finish(14.0, true);
        assert_eq!(rows[0].survived, Some(true));
        assert_eq!(rows[0].fall_s, None);
        assert_eq!(rows[1].survived, Some(false));
        assert!((rows[1].fall_s.unwrap() - 2.0).abs() < 1e-9);
        // The fall cut the second segment short: its analysis window is not fully sampled.
        assert_eq!(rows[1].gain, None);
        let summary = summarize(&rows);
        assert_eq!(summary.len(), 1);
        assert_eq!(summary[0].max_magnitude_survived, Some(0.1));
        assert_eq!(summary[0].first_fall_magnitude, Some(0.2));
    }

    #[test]
    fn a_fall_after_the_survive_window_belongs_to_nobody_and_a_cut_short_attempt_is_empty() {
        let mut tracker = VibrationTracker::new(2.0);
        tracker.begin(spec(2.0, 8, 0.1, 5.0)); // window to 11
        let rows = tracker.finish(30.0, true);
        assert_eq!(rows[0].survived, Some(true));
        assert_eq!(rows[0].fall_s, None);
        let mut tracker = VibrationTracker::new(2.0);
        tracker.begin(spec(2.0, 8, 0.1, 5.0));
        let rows = tracker.finish(10.0, false);
        assert_eq!(rows[0].survived, None);
    }

    #[test]
    fn frequencies_above_a_fraction_of_the_sample_rate_and_short_segments_are_rejected() {
        // 200 Hz samples: the limit is 20 Hz.
        assert!(check_segment("v", 20.0, 10, 0.005).is_ok());
        let high = check_segment("v", 20.5, 10, 0.005).unwrap_err();
        assert!(high.contains("sample rate"), "{high}");
        assert!(check_segment("v", 1.0, 3, 0.005)
            .unwrap_err()
            .contains("cycles"));
        assert!(check_segment("v", 0.0, 10, 0.005).is_err());
    }
}
