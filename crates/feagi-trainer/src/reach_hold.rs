//! Reach-and-hold co-agent loop.
//!
//! The embodiment controller owns the arm and the scene object. This module
//! samples nothing in the simulator: it asks the controller to begin an
//! attempt, steps one burst so the brain can read its own sensors, and scores
//! fingertip distance. During training it also sends pain or pleasure for a
//! distance change. Death and a fence exit end the attempt. The session
//! continues until the train and test seeds are done.

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
use crate::run_data::{name_of, ColumnSpec, EventTime, RunDataSink, RunDataValue, TableSpec};

/// Operator criterion for one reach-and-hold session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReachHoldCriterion {
    /// Applied scene-object id. The controller rejects a dynamic object.
    pub target_object_id: String,
    /// Model body whose position is scored against the target.
    pub reach_body: String,
    /// Distance, in millimeters, that counts as contact with the target.
    pub max_distance_mm: f64,
    /// Consecutive in-range bursts required to accept the attempt.
    pub hold_bursts: u32,
    /// Bursts after which an attempt that has not succeeded is truncated.
    pub attempt_burst_cap: u32,
}

impl ReachHoldCriterion {
    /// Rejects a criterion that cannot produce a success.
    pub fn validate(&self) -> Result<(), TrainerError> {
        if self.target_object_id.trim().is_empty() {
            return Err(TrainerError::Config(
                "target_object_id must be non-empty".to_string(),
            ));
        }
        if self.reach_body.trim().is_empty() {
            return Err(TrainerError::Config(
                "reach_body must be non-empty".to_string(),
            ));
        }
        if !self.max_distance_mm.is_finite() || self.max_distance_mm <= 0.0 {
            return Err(TrainerError::Config(
                "max_distance_mm must be a finite value > 0".to_string(),
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
        Ok(())
    }
}

/// Bursts that cover `seconds` at the live FEAGI burst frequency.
///
/// One burst lasts `1 / frequency_hz` seconds, so the duration is
/// `seconds * frequency_hz`. The product must already be a whole burst.
/// `label` is the operator-facing name used in the error, such as `hold`.
pub fn seconds_to_whole_bursts(
    label: &str,
    seconds: f64,
    frequency_hz: f64,
) -> Result<u32, TrainerError> {
    if !seconds.is_finite() || seconds <= 0.0 {
        return Err(TrainerError::Config(format!(
            "{label} must be a finite value > 0 seconds"
        )));
    }
    if !frequency_hz.is_finite() || frequency_hz <= 0.0 {
        return Err(TrainerError::Config(
            "burst frequency must be a finite value > 0 Hz".to_string(),
        ));
    }
    let bursts = seconds * frequency_hz;
    let whole = bursts.round();
    if (bursts - whole).abs() > 1e-6 || whole < 1.0 || whole > f64::from(u32::MAX) {
        return Err(TrainerError::Config(format!(
            "{label} of {seconds} s at {frequency_hz} Hz is {bursts} bursts, which is not a whole burst count"
        )));
    }
    Ok(whole as u32)
}

/// Bursts that cover `hold_seconds` at the live FEAGI burst frequency.
pub fn hold_bursts_from_seconds(hold_seconds: f64, frequency_hz: f64) -> Result<u32, TrainerError> {
    seconds_to_whole_bursts("hold", hold_seconds, frequency_hz)
}

/// Why one attempt ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReachAttemptOutcome {
    /// The fingertip held inside the distance for the required bursts.
    Success,
    /// `_death` fired. The controller homes the arm and the session continues.
    Death,
    /// The fingertip left the safety box.
    Fence,
    /// A misc reset interrupted the attempt.
    Interrupted,
    /// The burst cap was reached without a hold.
    Truncated,
}

/// One finished attempt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReachAttemptRecord {
    /// Seed the controller used to place the target.
    pub seed: u64,
    /// How the attempt ended.
    pub outcome: ReachAttemptOutcome,
    /// Bursts observed, including the terminal burst.
    pub bursts: u32,
    /// Fingertip distance at the terminal burst, in millimeters.
    pub final_distance_mm: f64,
}

/// Target placement returned by the controller.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReachPlacement {
    /// Seed that produced this point.
    pub seed: u64,
    /// Scene object that was moved.
    pub target_object_id: String,
    /// Target position in millimeters.
    pub target_mm: [f64; 3],
    /// Safety box minimum, millimeters.
    pub workspace_min_mm: [f64; 3],
    /// Safety box maximum, millimeters.
    pub workspace_max_mm: [f64; 3],
}

/// One fingertip measurement from the controller.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReachSample {
    /// Monotonic sample counter for this attempt.
    pub seq: u64,
    /// Attempt seed.
    pub seed: u64,
    /// Scene object id.
    pub target_object_id: String,
    /// Target position in millimeters.
    pub target_mm: [f64; 3],
    /// Safety box minimum, millimeters.
    pub workspace_min_mm: [f64; 3],
    /// Safety box maximum, millimeters.
    pub workspace_max_mm: [f64; 3],
    /// Fingertip position in millimeters.
    pub ee_mm: [f64; 3],
    /// Fingertip-to-target distance in millimeters.
    pub distance_mm: f64,
    /// Whether the fingertip is outside the safety box.
    pub fence_violated: bool,
    /// Latched end cause. Absent while the attempt is still open.
    pub terminal: Option<ReachTerminal>,
}

/// Controller-latched end cause. Hold success is decided here, not by the controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReachTerminal {
    /// `_death` fired.
    Death,
    /// Fingertip left the safety box.
    Fence,
    /// Misc reset fired while the attempt was active.
    Interrupted,
}

/// In-progress attempt counters.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenAttempt {
    /// Bursts already observed.
    pub bursts: u32,
    /// Consecutive in-range bursts ending at the latest sample.
    pub holding_bursts: u32,
    /// Latest fingertip distance, millimeters.
    pub last_distance_mm: f64,
    /// Latest controller sample sequence.
    pub last_seq: u64,
}

impl OpenAttempt {
    /// An attempt that has not observed a sample yet.
    pub fn fresh() -> Self {
        Self {
            bursts: 0,
            holding_bursts: 0,
            last_distance_mm: 0.0,
            last_seq: 0,
        }
    }
}

/// Result of folding one sample into an open attempt.
#[derive(Debug, Clone, PartialEq)]
pub enum AttemptStep {
    /// The attempt is still running.
    Continue(OpenAttempt),
    /// The attempt ended.
    Finished(ReachAttemptRecord),
}

/// Fold one controller sample into the attempt.
pub fn observe_sample(
    open: &OpenAttempt,
    sample: &ReachSample,
    criterion: &ReachHoldCriterion,
    seed: u64,
) -> Result<AttemptStep, TrainerError> {
    if sample.seed != seed {
        return Err(TrainerError::Runtime(format!(
            "reach sample seed {} does not match attempt seed {seed}",
            sample.seed
        )));
    }
    if !sample.distance_mm.is_finite() || sample.distance_mm < 0.0 {
        return Err(TrainerError::Runtime(format!(
            "reach distance_mm must be a finite value >= 0, got {}",
            sample.distance_mm
        )));
    }
    let bursts = open.bursts.saturating_add(1);
    if let Some(terminal) = sample.terminal {
        let outcome = match terminal {
            ReachTerminal::Death => ReachAttemptOutcome::Death,
            ReachTerminal::Fence => ReachAttemptOutcome::Fence,
            ReachTerminal::Interrupted => ReachAttemptOutcome::Interrupted,
        };
        return Ok(AttemptStep::Finished(ReachAttemptRecord {
            seed,
            outcome,
            bursts,
            final_distance_mm: sample.distance_mm,
        }));
    }
    if sample.fence_violated {
        return Ok(AttemptStep::Finished(ReachAttemptRecord {
            seed,
            outcome: ReachAttemptOutcome::Fence,
            bursts,
            final_distance_mm: sample.distance_mm,
        }));
    }
    let holding_bursts = if sample.distance_mm <= criterion.max_distance_mm {
        open.holding_bursts.saturating_add(1)
    } else {
        0
    };
    if holding_bursts >= criterion.hold_bursts {
        return Ok(AttemptStep::Finished(ReachAttemptRecord {
            seed,
            outcome: ReachAttemptOutcome::Success,
            bursts,
            final_distance_mm: sample.distance_mm,
        }));
    }
    if bursts >= criterion.attempt_burst_cap {
        return Ok(AttemptStep::Finished(ReachAttemptRecord {
            seed,
            outcome: ReachAttemptOutcome::Truncated,
            bursts,
            final_distance_mm: sample.distance_mm,
        }));
    }
    Ok(AttemptStep::Continue(OpenAttempt {
        bursts,
        holding_bursts,
        last_distance_mm: sample.distance_mm,
        last_seq: sample.seq,
    }))
}

/// Aggregate numbers for one phase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReachPhaseReport {
    /// Finished attempts.
    pub attempts: Vec<ReachAttemptRecord>,
    /// Successes divided by attempt count. Zero when the phase is empty.
    pub success_rate: f64,
    /// Median bursts among successes. Absent when nothing succeeded.
    pub median_bursts_to_success: Option<f64>,
    /// Mean terminal distance across every attempt, millimeters.
    pub mean_final_distance_mm: f64,
}

impl ReachPhaseReport {
    /// Summarize finished attempts. An empty phase is an error.
    pub fn from_attempts(attempts: Vec<ReachAttemptRecord>) -> Result<Self, TrainerError> {
        if attempts.is_empty() {
            return Err(TrainerError::Evaluation(
                "reach phase has no attempts".to_string(),
            ));
        }
        let count = attempts.len() as f64;
        let successes: Vec<u32> = attempts
            .iter()
            .filter(|attempt| attempt.outcome == ReachAttemptOutcome::Success)
            .map(|attempt| attempt.bursts)
            .collect();
        let success_rate = successes.len() as f64 / count;
        let median_bursts_to_success = if successes.is_empty() {
            None
        } else {
            Some(median_u32(&successes))
        };
        let mean_final_distance_mm = attempts
            .iter()
            .map(|attempt| attempt.final_distance_mm)
            .sum::<f64>()
            / count;
        Ok(Self {
            attempts,
            success_rate,
            median_bursts_to_success,
            mean_final_distance_mm,
        })
    }
}

/// Train phase, then held-out test phase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReachHoldReport {
    /// Attempts whose seeds were the training set.
    pub train: ReachPhaseReport,
    /// Attempts whose seeds were withheld from training.
    pub test: ReachPhaseReport,
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

/// One write the reach loop asks the caller to send to FEAGI.
#[derive(Debug, Clone, PartialEq)]
pub enum ReachCoachWrite {
    /// Advance one burst. The brain reads its own sensors for the object.
    Step,
    /// Train-phase pain or pleasure for a change in fingertip distance.
    Affect(RewardSignal),
}

/// Pain or pleasure for one distance change, or nothing when the distance is unchanged.
///
/// Closer is pleasure. Farther is pain. The magnitude is the change divided by the
/// farther of the two distances, so it lies in `(0, 1]`. The first sample of an
/// attempt has no previous distance and must not call this.
pub fn reach_distance_affect(
    previous_mm: f64,
    current_mm: f64,
) -> Result<Option<RewardSignal>, TrainerError> {
    for (label, value) in [("previous", previous_mm), ("current", current_mm)] {
        if !value.is_finite() || value < 0.0 {
            return Err(TrainerError::Runtime(format!(
                "reach {label} distance must be a finite value >= 0, got {value}"
            )));
        }
    }
    if previous_mm == current_mm {
        return Ok(None);
    }
    let scale = previous_mm.max(current_mm);
    let magnitude = (previous_mm - current_mm).abs() / scale;
    let channel = if current_mm < previous_mm {
        AffectChannel::Pleasure
    } else {
        AffectChannel::Pain
    };
    Ok(Some(RewardSignal { channel, magnitude }))
}

/// Controller side of one reach session. The MuJoCo HTTP server is one implementation.
pub trait ReachProbe {
    /// Home the arm and place the target for `seed`.
    fn begin_attempt(
        &mut self,
        seed: u64,
        target_object_id: &str,
        reach_body: &str,
    ) -> Result<ReachPlacement, TrainerError>;

    /// Wait for a sample newer than `after_seq`.
    fn poll(&mut self, after_seq: u64, timeout: Duration) -> Result<ReachSample, TrainerError>;
}

/// Seeds and the criterion for one session.
#[derive(Debug, Clone, PartialEq)]
pub struct ReachHoldConfig {
    /// Acceptance rule.
    pub criterion: ReachHoldCriterion,
    /// Training seeds, in visit order.
    pub train_seeds: Vec<u64>,
    /// Held-out seeds, in visit order.
    pub test_seeds: Vec<u64>,
    /// How long one poll may wait. Sized from the live burst period by the caller.
    pub poll_timeout: Duration,
}

impl ReachHoldConfig {
    /// Reject an empty phase or a criterion that cannot succeed.
    pub fn validate(&self) -> Result<(), TrainerError> {
        self.criterion.validate()?;
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
        Ok(())
    }
}

/// Crate version stamped on a reach scorecard. Scoring runs in this crate.
pub fn trainer_package_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Named metrics for one finished phase. Median is omitted when nothing succeeded.
pub fn phase_metrics(phase: &ReachPhaseReport) -> BTreeMap<String, f64> {
    let mut metrics = BTreeMap::new();
    metrics.insert("success_rate".to_string(), phase.success_rate);
    metrics.insert(
        "mean_final_distance_mm".to_string(),
        phase.mean_final_distance_mm,
    );
    if let Some(median) = phase.median_bursts_to_success {
        metrics.insert("median_bursts_to_success".to_string(), median);
    }
    metrics
}

/// Scorecard for one reach phase. Train and test share dataset identity so a
/// protocol evaluation can compare them.
pub fn reach_phase_scorecard(
    scorecard_id: &str,
    connectome_hash: &str,
    genome_schema_version: u32,
    split_id: &str,
    dataset_asset_id: &str,
    dataset_content_hash: &str,
    phase: &ReachPhaseReport,
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
        evaluation_protocol_version: EvaluationProtocolVersion("reach-hold-v1".to_string()),
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
        metrics: phase_metrics(phase),
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
/// `coach` advances one burst, and during training sends pain or pleasure for a
/// distance change. This function does not touch the simulator and does not
/// write the object location. Each attempt emits one progress event, and each
/// phase emits one aggregate metric update. The test phase steps only.
pub fn run_reach_hold(
    config: &ReachHoldConfig,
    probe: &mut dyn ReachProbe,
    coach: &mut dyn FnMut(ReachCoachWrite) -> Result<(), TrainerError>,
    events: &mut dyn RunEventSink,
    run_id: &RunId,
    cancel: &CancelToken,
    mut data: Option<&mut (dyn RunDataSink + '_)>,
) -> Result<ReachHoldReport, TrainerError> {
    config.validate()?;
    let train = run_reach_phase(
        &ReachPhaseRun {
            phase: "train",
            seeds: &config.train_seeds,
            criterion: &config.criterion,
            poll_timeout: config.poll_timeout,
            run_id,
        },
        probe,
        coach,
        events,
        cancel,
        data.as_deref_mut(),
    )?;
    let test = run_reach_phase(
        &ReachPhaseRun {
            phase: "test",
            seeds: &config.test_seeds,
            criterion: &config.criterion,
            poll_timeout: config.poll_timeout,
            run_id,
        },
        probe,
        coach,
        events,
        cancel,
        data,
    )?;
    Ok(ReachHoldReport { train, test })
}

/// One phase of a reach session, plus the run the progress events belong to.
pub struct ReachPhaseRun<'a> {
    /// `train` or `test`.
    pub phase: &'a str,
    /// Seeds visited in order.
    pub seeds: &'a [u64],
    /// Acceptance rule.
    pub criterion: &'a ReachHoldCriterion,
    /// How long one poll may wait.
    pub poll_timeout: Duration,
    /// Run the progress events are stamped with.
    pub run_id: &'a RunId,
}

/// Run one phase (`train` or `test`) and emit its progress.
pub fn run_reach_phase(
    run: &ReachPhaseRun<'_>,
    probe: &mut dyn ReachProbe,
    coach: &mut dyn FnMut(ReachCoachWrite) -> Result<(), TrainerError>,
    events: &mut dyn RunEventSink,
    cancel: &CancelToken,
    mut data: Option<&mut (dyn RunDataSink + '_)>,
) -> Result<ReachPhaseReport, TrainerError> {
    if run.phase != "train" && run.phase != "test" {
        return Err(TrainerError::Config(
            "reach phase must be train or test".to_string(),
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
    let samples_total = run.seeds.len() as u64;
    let mut attempts = Vec::with_capacity(run.seeds.len());
    if let Some(sink) = data.as_deref_mut() {
        sink.declare_table(&reach_steps_spec());
        sink.declare_table(&reach_attempts_spec());
        sink.event(EventTime::default(), "phase_start", run.phase);
    }
    for seed in run.seeds {
        if cancel.is_cancelled() {
            return Err(TrainerError::Cancelled(
                "reach-and-hold run cancelled".to_string(),
            ));
        }
        let attempt_index = attempts.len() as u64;
        let record = run_attempt(
            *seed,
            run.criterion,
            probe,
            coach,
            run.phase == "train",
            run.poll_timeout,
            cancel,
            ReachDataContext {
                sink: data.as_deref_mut(),
                phase: run.phase,
                attempt: attempt_index,
            },
        )?;
        if let Some(sink) = data.as_deref_mut() {
            sink.row(
                REACH_ATTEMPTS_TABLE,
                &[
                    RunDataValue::text(run.phase),
                    RunDataValue::count(attempt_index),
                    RunDataValue::count(record.seed),
                    RunDataValue::Text(name_of(&record.outcome)),
                    RunDataValue::count(u64::from(record.bursts)),
                    RunDataValue::Float(record.final_distance_mm),
                ],
            );
        }
        attempts.push(record);
        events.emit(RunEvent::new(
            run.run_id.clone(),
            RunEventKind::Progress {
                samples_done: attempts.len() as u64,
                samples_total,
                repeat_index: 0,
                repeat_total: 1,
                sent_values: Vec::new(),
                tick_index: None,
                window_values: Vec::new(),
                image_png_base64: None,
                mask_png_base64: None,
                preview_name: Some(format!("{} seed {seed}", run.phase)),
                predicted_class: None,
                class_scores: Vec::new(),
                stand: None,
            },
        ));
    }
    if let Some(sink) = data {
        sink.event(
            EventTime {
                episode: Some(attempts.len() as u64),
                ..EventTime::default()
            },
            "phase_end",
            run.phase,
        );
    }
    let report = ReachPhaseReport::from_attempts(attempts)?;
    events.emit(RunEvent::new(
        run.run_id.clone(),
        RunEventKind::MetricUpdate {
            scope: MetricScope::Aggregate,
            metrics: phase_metrics(&report),
            class_breakdown: Vec::new(),
        },
    ));
    Ok(report)
}

const REACH_STEPS_TABLE: &str = "steps";
const REACH_ATTEMPTS_TABLE: &str = "attempts";

fn reach_steps_spec() -> TableSpec {
    let mut columns = vec![
        ColumnSpec::unitless("phase", "Phase the sample belongs to."),
        ColumnSpec::unitless("attempt", "Attempt index within the phase."),
        ColumnSpec::unitless("seed", "Attempt seed."),
        ColumnSpec::unitless("seq", "Controller sample counter within the attempt."),
        ColumnSpec::with_unit("distance", "mm", "Fingertip-to-target distance."),
    ];
    for (prefix, what) in [("ee", "Fingertip"), ("target", "Target")] {
        for axis in ["x", "y", "z"] {
            columns.push(ColumnSpec::with_unit(
                &format!("{prefix}_{axis}"),
                "mm",
                &format!("{what} position, {axis} axis."),
            ));
        }
    }
    columns.push(ColumnSpec::unitless(
        "fence_violated",
        "The fingertip is outside the safety box.",
    ));
    columns.push(ColumnSpec::unitless(
        "terminal",
        "Latched end cause, when one was latched.",
    ));
    TableSpec {
        name: REACH_STEPS_TABLE.to_string(),
        description: "One row per controller sample.".to_string(),
        columns,
    }
}

fn reach_attempts_spec() -> TableSpec {
    TableSpec {
        name: REACH_ATTEMPTS_TABLE.to_string(),
        description: "One row per finished attempt.".to_string(),
        columns: vec![
            ColumnSpec::unitless("phase", "Phase the attempt belongs to."),
            ColumnSpec::unitless("attempt", "Attempt index within the phase."),
            ColumnSpec::unitless("seed", "Attempt seed."),
            ColumnSpec::unitless("outcome", "How the attempt ended."),
            ColumnSpec::unitless("bursts", "Bursts observed."),
            ColumnSpec::with_unit("final_distance", "mm", "Distance at the terminal burst."),
        ],
    }
}

/// Where one attempt writes its rows and events.
struct ReachDataContext<'s, 'a> {
    sink: Option<&'s mut (dyn RunDataSink + 'a)>,
    phase: &'s str,
    attempt: u64,
}

#[allow(clippy::too_many_arguments)]
fn run_attempt(
    seed: u64,
    criterion: &ReachHoldCriterion,
    probe: &mut dyn ReachProbe,
    coach: &mut dyn FnMut(ReachCoachWrite) -> Result<(), TrainerError>,
    train_affect: bool,
    poll_timeout: Duration,
    cancel: &CancelToken,
    mut data: ReachDataContext<'_, '_>,
) -> Result<ReachAttemptRecord, TrainerError> {
    let placement =
        probe.begin_attempt(seed, &criterion.target_object_id, &criterion.reach_body)?;
    if placement.seed != seed {
        return Err(TrainerError::Runtime(format!(
            "controller placed seed {} for requested seed {seed}",
            placement.seed
        )));
    }
    let attempt = data.attempt;
    let at = |step: u64| EventTime {
        sim_time_s: None,
        step: Some(step),
        episode: Some(attempt),
    };
    if let Some(sink) = data.sink.as_deref_mut() {
        sink.event(at(0), "attempt_start", &format!("seed {seed}"));
    }
    let mut open = OpenAttempt::fresh();
    loop {
        if cancel.is_cancelled() {
            return Err(TrainerError::Cancelled(
                "reach-and-hold run cancelled".to_string(),
            ));
        }
        coach(ReachCoachWrite::Step)?;
        let sample = probe.poll(open.last_seq, poll_timeout)?;
        if let Some(sink) = data.sink.as_deref_mut() {
            let mut row = vec![
                RunDataValue::text(data.phase),
                RunDataValue::count(attempt),
                RunDataValue::count(sample.seed),
                RunDataValue::count(sample.seq),
                RunDataValue::Float(sample.distance_mm),
            ];
            row.extend(sample.ee_mm.iter().map(|v| RunDataValue::Float(*v)));
            row.extend(sample.target_mm.iter().map(|v| RunDataValue::Float(*v)));
            row.push(RunDataValue::Bool(sample.fence_violated));
            row.push(
                sample
                    .terminal
                    .as_ref()
                    .map_or(RunDataValue::Null, |t| RunDataValue::Text(name_of(t))),
            );
            sink.row(REACH_STEPS_TABLE, &row);
        }
        if train_affect && open.bursts > 0 {
            if let Some(signal) = reach_distance_affect(open.last_distance_mm, sample.distance_mm)?
            {
                if let Some(sink) = data.sink.as_deref_mut() {
                    sink.event(
                        at(u64::from(open.bursts)),
                        "affect",
                        &format!("{:?} {}", signal.channel, signal.magnitude).to_lowercase(),
                    );
                }
                coach(ReachCoachWrite::Affect(signal))?;
            }
        }
        match observe_sample(&open, &sample, criterion, seed)? {
            AttemptStep::Continue(next) => open = next,
            AttemptStep::Finished(record) => {
                if let Some(sink) = data.sink.as_deref_mut() {
                    sink.event(
                        at(u64::from(record.bursts)),
                        "attempt_end",
                        &name_of(&record.outcome),
                    );
                }
                return Ok(record);
            }
        }
    }
}

/// HTTP client for the neutral reach actions on a controller control server.
pub struct HttpReachProbe {
    host: String,
    port: u16,
    timeout: Duration,
}

impl HttpReachProbe {
    /// `base_url` is `http://host:port` with no path.
    pub fn new(base_url: &str, timeout: Duration) -> Result<Self, TrainerError> {
        if timeout.is_zero() {
            return Err(TrainerError::Config(
                "reach control timeout must be > 0".to_string(),
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
            TrainerError::Runtime(format!("reach command encode failed: {error}"))
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
            .map_err(|error| TrainerError::Runtime(format!("reach control address: {error}")))?
            .next()
            .ok_or_else(|| {
                TrainerError::Runtime(format!("reach control address {address} did not resolve"))
            })?;
        let mut stream = TcpStream::connect_timeout(&socket, self.timeout).map_err(|error| {
            TrainerError::Runtime(format!("reach control connect failed: {error}"))
        })?;
        stream
            .set_read_timeout(Some(self.timeout))
            .map_err(|error| TrainerError::Runtime(format!("reach control timeout: {error}")))?;
        stream
            .set_write_timeout(Some(self.timeout))
            .map_err(|error| TrainerError::Runtime(format!("reach control timeout: {error}")))?;
        stream.write_all(request.as_bytes()).map_err(|error| {
            TrainerError::Runtime(format!("reach control write failed: {error}"))
        })?;
        stream.write_all(&payload).map_err(|error| {
            TrainerError::Runtime(format!("reach control write failed: {error}"))
        })?;
        let mut raw = Vec::new();
        stream.read_to_end(&mut raw).map_err(|error| {
            TrainerError::Runtime(format!("reach control read failed: {error}"))
        })?;
        parse_http_json(&raw)
    }
}

impl ReachProbe for HttpReachProbe {
    fn begin_attempt(
        &mut self,
        seed: u64,
        target_object_id: &str,
        reach_body: &str,
    ) -> Result<ReachPlacement, TrainerError> {
        let result = self.command(&serde_json::json!({
            "action": "begin_reach_attempt",
            "seed": seed,
            "target_object_id": target_object_id,
            "reach_body": reach_body,
        }))?;
        serde_json::from_value(result)
            .map_err(|error| TrainerError::Runtime(format!("reach begin decode failed: {error}")))
    }

    fn poll(&mut self, after_seq: u64, timeout: Duration) -> Result<ReachSample, TrainerError> {
        let timeout_s = timeout.as_secs_f64();
        if timeout_s <= 0.0 {
            return Err(TrainerError::Config(
                "reach poll timeout must be > 0".to_string(),
            ));
        }
        let result = self.command(&serde_json::json!({
            "action": "poll_reach_outcome",
            "after_seq": after_seq,
            "timeout_s": timeout_s,
        }))?;
        serde_json::from_value(result)
            .map_err(|error| TrainerError::Runtime(format!("reach poll decode failed: {error}")))
    }
}

fn split_http_base(base_url: &str) -> Result<(String, u16), TrainerError> {
    let rest = base_url.strip_prefix("http://").ok_or_else(|| {
        TrainerError::Config("reach control URL must start with http://".to_string())
    })?;
    if rest.contains('/') {
        return Err(TrainerError::Config(
            "reach control URL must be http://host:port with no path".to_string(),
        ));
    }
    let (host, port_text) = rest
        .split_once(':')
        .ok_or_else(|| TrainerError::Config("reach control URL must include a port".to_string()))?;
    if host.is_empty() {
        return Err(TrainerError::Config(
            "reach control URL host is empty".to_string(),
        ));
    }
    let port: u16 = port_text.parse().map_err(|_| {
        TrainerError::Config(format!("reach control port {port_text} is not a u16"))
    })?;
    Ok((host.to_string(), port))
}

fn parse_http_json(raw: &[u8]) -> Result<serde_json::Value, TrainerError> {
    let text = String::from_utf8_lossy(raw);
    let (head, body) = text.split_once("\r\n\r\n").ok_or_else(|| {
        TrainerError::Runtime("reach control response has no HTTP body".to_string())
    })?;
    let status_line = head.lines().next().unwrap_or("");
    if !status_line.contains(" 200 ") {
        return Err(TrainerError::Runtime(format!(
            "reach control HTTP failed: {status_line}"
        )));
    }
    let envelope: serde_json::Value = serde_json::from_str(body.trim())
        .map_err(|error| TrainerError::Runtime(format!("reach control JSON failed: {error}")))?;
    let ok = envelope
        .get("ok")
        .and_then(|value| value.as_bool())
        .ok_or_else(|| {
            TrainerError::Runtime("reach control response has no ok flag".to_string())
        })?;
    if !ok {
        let message = envelope
            .get("error")
            .and_then(|value| value.as_str())
            .unwrap_or("reach control command failed");
        return Err(TrainerError::Runtime(message.to_string()));
    }
    envelope
        .get("result")
        .cloned()
        .ok_or_else(|| TrainerError::Runtime("reach control response has no result".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::CollectingEventSink;
    use std::collections::VecDeque;

    struct ScriptedProbe {
        placements: VecDeque<ReachPlacement>,
        samples: VecDeque<ReachSample>,
    }

    impl ReachProbe for ScriptedProbe {
        fn begin_attempt(
            &mut self,
            seed: u64,
            _target_object_id: &str,
            _reach_body: &str,
        ) -> Result<ReachPlacement, TrainerError> {
            let mut placement = self
                .placements
                .pop_front()
                .ok_or_else(|| TrainerError::Runtime("no scripted placement".to_string()))?;
            placement.seed = seed;
            Ok(placement)
        }

        fn poll(
            &mut self,
            _after_seq: u64,
            _timeout: Duration,
        ) -> Result<ReachSample, TrainerError> {
            self.samples
                .pop_front()
                .ok_or_else(|| TrainerError::Runtime("no scripted sample".to_string()))
        }
    }

    #[test]
    fn hold_seconds_convert_at_the_burst_frequency() {
        assert_eq!(hold_bursts_from_seconds(1.5, 8.0).unwrap(), 12);
        assert_eq!(hold_bursts_from_seconds(1.0, 10.0).unwrap(), 10);
        assert!(hold_bursts_from_seconds(1.5, 7.0).is_err());
        assert!(hold_bursts_from_seconds(0.0, 10.0).is_err());
        assert!(hold_bursts_from_seconds(1.0, 0.0).is_err());
    }

    fn criterion() -> ReachHoldCriterion {
        ReachHoldCriterion {
            target_object_id: "cup".to_string(),
            reach_body: "gripper".to_string(),
            max_distance_mm: 15.0,
            hold_bursts: 2,
            attempt_burst_cap: 4,
        }
    }

    fn sample(seed: u64, seq: u64, distance: f64, terminal: Option<ReachTerminal>) -> ReachSample {
        ReachSample {
            seq,
            seed,
            target_object_id: "cup".to_string(),
            target_mm: [0.0, 0.0, 0.0],
            workspace_min_mm: [0.0, 0.0, 0.0],
            workspace_max_mm: [100.0, 100.0, 100.0],
            ee_mm: [distance, 0.0, 0.0],
            distance_mm: distance,
            fence_violated: false,
            terminal,
        }
    }

    fn placement() -> ReachPlacement {
        ReachPlacement {
            seed: 0,
            target_object_id: "cup".to_string(),
            target_mm: [25.0, 0.0, 0.0],
            workspace_min_mm: [0.0, 0.0, 0.0],
            workspace_max_mm: [100.0, 100.0, 100.0],
        }
    }

    #[test]
    fn hold_requires_consecutive_in_range_samples() {
        let criterion = criterion();
        let mut open = OpenAttempt::fresh();
        let step = observe_sample(&open, &sample(1, 1, 10.0, None), &criterion, 1).expect("step");
        open = match step {
            AttemptStep::Continue(next) => next,
            AttemptStep::Finished(_) => panic!("first sample must not finish"),
        };
        let done = observe_sample(&open, &sample(1, 2, 10.0, None), &criterion, 1).expect("done");
        match done {
            AttemptStep::Finished(record) => {
                assert_eq!(record.outcome, ReachAttemptOutcome::Success);
                assert_eq!(record.bursts, 2);
            }
            AttemptStep::Continue(_) => panic!("second in-range sample must succeed"),
        }
    }

    #[test]
    fn death_ends_the_attempt_without_a_success() {
        let done = observe_sample(
            &OpenAttempt::fresh(),
            &sample(1, 1, 4.0, Some(ReachTerminal::Death)),
            &criterion(),
            1,
        )
        .expect("death");
        match done {
            AttemptStep::Finished(record) => {
                assert_eq!(record.outcome, ReachAttemptOutcome::Death);
            }
            AttemptStep::Continue(_) => panic!("death must finish"),
        }
    }

    #[test]
    fn distance_affect_follows_the_change() {
        let steady = super::reach_distance_affect(20.0, 20.0).expect("steady");
        assert!(steady.is_none());
        let closer = super::reach_distance_affect(40.0, 20.0).expect("closer");
        let closer = closer.expect("pleasure");
        assert_eq!(closer.channel, crate::binding::AffectChannel::Pleasure);
        assert_eq!(closer.magnitude, 0.5);
        let farther = super::reach_distance_affect(20.0, 40.0).expect("farther");
        let farther = farther.expect("pain");
        assert_eq!(farther.channel, crate::binding::AffectChannel::Pain);
        assert_eq!(farther.magnitude, 0.5);
        assert!(super::reach_distance_affect(-1.0, 10.0).is_err());
    }

    #[test]
    fn train_emits_distance_affect_and_test_does_not() {
        let mut writes = Vec::new();
        let mut probe = ScriptedProbe {
            placements: VecDeque::from([placement(), placement()]),
            samples: VecDeque::from([
                sample(1, 1, 40.0, None),
                sample(1, 2, 20.0, Some(ReachTerminal::Death)),
                sample(2, 1, 40.0, None),
                sample(2, 2, 20.0, Some(ReachTerminal::Death)),
            ]),
        };
        let config = ReachHoldConfig {
            criterion: criterion(),
            train_seeds: vec![1],
            test_seeds: vec![2],
            poll_timeout: Duration::from_millis(1),
        };
        let mut events = CollectingEventSink::default();
        let run_id = RunId("reach-affect".to_string());
        run_reach_hold(
            &config,
            &mut probe,
            &mut |write| {
                writes.push(write);
                Ok(())
            },
            &mut events,
            &run_id,
            &CancelToken::new(),
            None,
        )
        .expect("report");
        let affects: Vec<_> = writes
            .iter()
            .filter_map(|write| match write {
                super::ReachCoachWrite::Affect(signal) => Some(signal.clone()),
                super::ReachCoachWrite::Step => None,
            })
            .collect();
        assert_eq!(affects.len(), 1);
        assert_eq!(affects[0].channel, crate::binding::AffectChannel::Pleasure);
        assert_eq!(affects[0].magnitude, 0.5);
    }

    #[test]
    fn session_scores_train_then_test() {
        let mut steps = 0;
        let mut probe = ScriptedProbe {
            placements: VecDeque::from([placement(), placement()]),
            samples: VecDeque::from([
                sample(1, 1, 40.0, Some(ReachTerminal::Death)),
                sample(2, 1, 10.0, None),
                sample(2, 2, 10.0, None),
            ]),
        };
        let config = ReachHoldConfig {
            criterion: criterion(),
            train_seeds: vec![1],
            test_seeds: vec![2],
            poll_timeout: Duration::from_millis(1),
        };
        let mut events = CollectingEventSink::default();
        let run_id = RunId("reach-test".to_string());
        let mut data = crate::run_data::RecordingRunDataSink::default();
        let report = run_reach_hold(
            &config,
            &mut probe,
            &mut |write| {
                if matches!(write, super::ReachCoachWrite::Step) {
                    steps += 1;
                }
                Ok(())
            },
            &mut events,
            &run_id,
            &CancelToken::new(),
            Some(&mut data),
        )
        .expect("report");
        assert!(data.errors.is_empty(), "{:?}", data.errors);
        assert_eq!(data.rows_of("steps").len(), 3);
        assert_eq!(data.rows_of("attempts").len(), 2);
        assert_eq!(
            data.event_kinds(),
            vec![
                "phase_start",
                "attempt_start",
                "attempt_end",
                "phase_end",
                "phase_start",
                "attempt_start",
                "attempt_end",
                "phase_end"
            ]
        );
        assert_eq!(report.train.attempts[0].outcome, ReachAttemptOutcome::Death);
        assert_eq!(
            report.test.attempts[0].outcome,
            ReachAttemptOutcome::Success
        );
        assert_eq!(report.test.success_rate, 1.0);
        assert_eq!(steps, 3);
        let progress = events
            .events
            .iter()
            .filter(|event| matches!(event.kind, RunEventKind::Progress { .. }))
            .count();
        let metrics = events
            .events
            .iter()
            .filter(|event| matches!(event.kind, RunEventKind::MetricUpdate { .. }))
            .count();
        assert_eq!(progress, 2);
        assert_eq!(metrics, 2);
        let card = reach_phase_scorecard(
            "sc-reach-test",
            "sha256:connectome",
            3,
            "test",
            "reach:cup",
            "sha256:scenario",
            &report.test,
        );
        assert_eq!(card.metrics.get("success_rate"), Some(&1.0));
        assert!(card.metrics.contains_key("median_bursts_to_success"));
    }
}
