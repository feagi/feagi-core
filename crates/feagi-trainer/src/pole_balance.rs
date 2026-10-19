//! Pole-balance co-agent loop.
//!
//! The embodiment controller owns the cart and the pole. This module homes
//! nothing itself: it asks the controller to begin an attempt, writes the
//! upright goal into the goal area through the caller, and scores how long
//! the pole stays within the tilt limit. An attempt ends when the tilt
//! exceeds that limit, the attempt cap is reached, or death or a misc reset
//! fires. The phase score is the mean share of the cap spent upright.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::contracts::{
    BackendFingerprint, BackendKind, ConnectomeHash, ContentHash, DatasetAssetId,
    EvaluationProtocolVersion, MetricScope, PluginId, PluginRef, RunEvent, RunEventKind, RunId,
    Scorecard, ScorecardId, ScorecardStatus, ScorecardVisibility, SplitId,
};
use crate::control::{CancelToken, RunEventSink};
use crate::error::TrainerError;
use crate::metrics::episodic_control::EpisodicControlMetricPack;
use crate::reach_hold::trainer_package_version;

/// Operator criterion for one pole-balance session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoleBalanceCriterion {
    /// Body whose +Z axis is the pole axis.
    pub pole_body: String,
    /// Maximum lean from upright, in degrees, that still counts as balanced.
    pub max_tilt_deg: f64,
    /// Longest an attempt runs. The score is the share of this cap spent upright.
    pub attempt_burst_cap: u32,
}

impl PoleBalanceCriterion {
    /// Rejects a criterion that cannot produce a success.
    pub fn validate(&self) -> Result<(), TrainerError> {
        if self.pole_body.trim().is_empty() {
            return Err(TrainerError::Config(
                "pole body must be non-empty".to_string(),
            ));
        }
        if !self.max_tilt_deg.is_finite() || self.max_tilt_deg <= 0.0 || self.max_tilt_deg > 90.0 {
            return Err(TrainerError::Config(
                "max_tilt_deg must be a finite value in (0, 90]".to_string(),
            ));
        }
        if self.attempt_burst_cap == 0 {
            return Err(TrainerError::Config(
                "attempt_burst_cap must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Why one attempt ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PoleAttemptOutcome {
    /// Tilt stayed inside the limit until the attempt cap. That is the longest score.
    Success,
    /// `_death` fired. The controller homes the body and the session continues.
    Death,
    /// A misc reset interrupted the attempt.
    Interrupted,
    /// Tilt exceeded the limit before the cap. Upright bursts before the fall still count.
    Truncated,
}

/// One finished attempt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoleAttemptRecord {
    /// Seed the controller used to home this attempt.
    pub seed: u64,
    /// How the attempt ended.
    pub outcome: PoleAttemptOutcome,
    /// Bursts observed, including a burst that exceeded the limit.
    pub bursts: u32,
    /// Bursts whose tilt stayed within the limit. A falling burst is not counted.
    pub balanced_bursts: u32,
    /// Pole tilt at the terminal burst, in degrees.
    pub final_tilt_deg: f64,
}

/// Home acknowledgement returned by the controller.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoleBegin {
    /// Seed that was homed.
    pub seed: u64,
    /// Pole body measured for tilt.
    pub pole_body: String,
    /// Tilt limit echoed by the controller, degrees.
    pub max_tilt_deg: f64,
}

/// One pole measurement from the controller.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoleSample {
    /// Monotonic sample counter for this attempt.
    pub seq: u64,
    /// Attempt seed.
    pub seed: u64,
    /// Pole tilt from world up, degrees.
    pub tilt_deg: f64,
    /// Latched end cause. Absent while the attempt is still open.
    pub terminal: Option<PoleTerminal>,
}

/// Controller-latched end cause. Duration is scored here, not by the controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PoleTerminal {
    /// `_death` fired.
    Death,
    /// Misc reset fired while the attempt was active.
    Interrupted,
}

/// In-progress attempt counters.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenPoleAttempt {
    /// Bursts already observed. Each one stayed within the tilt limit.
    pub bursts: u32,
    /// Latest pole tilt, degrees.
    pub last_tilt_deg: f64,
    /// Latest controller sample sequence.
    pub last_seq: u64,
}

impl OpenPoleAttempt {
    /// An attempt that has not observed a sample yet.
    pub fn fresh() -> Self {
        Self {
            bursts: 0,
            last_tilt_deg: 0.0,
            last_seq: 0,
        }
    }
}

/// Result of folding one sample into an open attempt.
#[derive(Debug, Clone, PartialEq)]
pub enum PoleAttemptStep {
    /// The attempt is still running.
    Continue(OpenPoleAttempt),
    /// The attempt ended.
    Finished(PoleAttemptRecord),
}

/// Pole angle and angular rate sit at the middle of the unit range.
/// The third channel is the upright flag.
pub fn pole_upright_goal_features() -> [f64; 3] {
    [0.5, 0.5, 1.0]
}

/// Fold one controller sample into the attempt.
pub fn observe_pole_sample(
    open: &OpenPoleAttempt,
    sample: &PoleSample,
    criterion: &PoleBalanceCriterion,
    seed: u64,
) -> Result<PoleAttemptStep, TrainerError> {
    if sample.seed != seed {
        return Err(TrainerError::Runtime(format!(
            "pole sample seed {} does not match attempt seed {seed}",
            sample.seed
        )));
    }
    if !sample.tilt_deg.is_finite() || sample.tilt_deg < 0.0 {
        return Err(TrainerError::Runtime(format!(
            "tilt_deg must be a finite value >= 0, got {}",
            sample.tilt_deg
        )));
    }
    let bursts = open.bursts.saturating_add(1);
    let upright = sample.tilt_deg <= criterion.max_tilt_deg;
    let balanced_bursts = if upright { bursts } else { open.bursts };
    if let Some(terminal) = sample.terminal {
        let outcome = match terminal {
            PoleTerminal::Death => PoleAttemptOutcome::Death,
            PoleTerminal::Interrupted => PoleAttemptOutcome::Interrupted,
        };
        return Ok(PoleAttemptStep::Finished(PoleAttemptRecord {
            seed,
            outcome,
            bursts,
            balanced_bursts,
            final_tilt_deg: sample.tilt_deg,
        }));
    }
    if !upright {
        return Ok(PoleAttemptStep::Finished(PoleAttemptRecord {
            seed,
            outcome: PoleAttemptOutcome::Truncated,
            bursts,
            balanced_bursts,
            final_tilt_deg: sample.tilt_deg,
        }));
    }
    if bursts >= criterion.attempt_burst_cap {
        return Ok(PoleAttemptStep::Finished(PoleAttemptRecord {
            seed,
            outcome: PoleAttemptOutcome::Success,
            bursts,
            balanced_bursts,
            final_tilt_deg: sample.tilt_deg,
        }));
    }
    Ok(PoleAttemptStep::Continue(OpenPoleAttempt {
        bursts,
        last_tilt_deg: sample.tilt_deg,
        last_seq: sample.seq,
    }))
}

/// Aggregate numbers for one phase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolePhaseReport {
    /// Finished attempts.
    pub attempts: Vec<PoleAttemptRecord>,
    /// Mean share of the attempt cap spent within the tilt limit, from 0 to 1.
    pub success_rate: f64,
    /// Median upright bursts across every attempt.
    pub median_balanced_bursts: f64,
    /// Mean terminal tilt across every attempt, degrees.
    pub mean_final_tilt_deg: f64,
}

impl PolePhaseReport {
    /// Summarize finished attempts. An empty phase is an error.
    pub fn from_attempts(
        attempts: Vec<PoleAttemptRecord>,
        attempt_burst_cap: u32,
    ) -> Result<Self, TrainerError> {
        if attempts.is_empty() {
            return Err(TrainerError::Evaluation(
                "pole phase has no attempts".to_string(),
            ));
        }
        if attempt_burst_cap == 0 {
            return Err(TrainerError::Config(
                "attempt_burst_cap must be > 0".to_string(),
            ));
        }
        let count = attempts.len() as f64;
        let balanced: u64 = attempts
            .iter()
            .map(|attempt| u64::from(attempt.balanced_bursts))
            .sum();
        let success_rate = balanced as f64 / (count * f64::from(attempt_burst_cap));
        let durations: Vec<u32> = attempts
            .iter()
            .map(|attempt| attempt.balanced_bursts)
            .collect();
        let mean_final_tilt_deg = attempts
            .iter()
            .map(|attempt| attempt.final_tilt_deg)
            .sum::<f64>()
            / count;
        Ok(Self {
            attempts,
            success_rate,
            median_balanced_bursts: median_u32(&durations),
            mean_final_tilt_deg,
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

/// Named metrics for one finished phase.
///
/// `success_rate` is the mean share of the attempt cap spent upright.
pub fn pole_phase_metrics(phase: &PolePhaseReport) -> BTreeMap<String, f64> {
    let mut metrics = BTreeMap::new();
    metrics.insert("success_rate".to_string(), phase.success_rate);
    metrics.insert(
        "median_balanced_bursts".to_string(),
        phase.median_balanced_bursts,
    );
    metrics.insert("mean_final_tilt_deg".to_string(), phase.mean_final_tilt_deg);
    metrics
}

/// Scorecard for one pole-balance phase.
pub fn pole_phase_scorecard(
    scorecard_id: &str,
    connectome_hash: &str,
    genome_schema_version: u32,
    split_id: &str,
    dataset_asset_id: &str,
    dataset_content_hash: &str,
    phase: &PolePhaseReport,
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
        evaluation_protocol_version: EvaluationProtocolVersion("pole-balance-v2".to_string()),
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
        metrics: pole_phase_metrics(phase),
        metric_stats: None,
        status: ScorecardStatus::SelfReported,
        visibility: ScorecardVisibility::Local,
        skip: None,
        elapsed_ms: None,
        sample_draw_seed: None,
        metadata: BTreeMap::new(),
    }
}

/// Controller side of one pole-balance session.
pub trait PoleProbe {
    /// Home the model for `seed`.
    fn begin_attempt(
        &mut self,
        seed: u64,
        criterion: &PoleBalanceCriterion,
    ) -> Result<PoleBegin, TrainerError>;

    /// Wait for a sample newer than `after_seq`.
    fn poll(&mut self, after_seq: u64, timeout: Duration) -> Result<PoleSample, TrainerError>;
}

/// One phase of a pole-balance session, plus the run the progress events belong to.
pub struct PolePhaseRun<'a> {
    /// `train` or `test`.
    pub phase: &'a str,
    /// Seeds visited in order.
    pub seeds: &'a [u64],
    /// Acceptance rule.
    pub criterion: &'a PoleBalanceCriterion,
    /// How long one poll may wait.
    pub poll_timeout: Duration,
    /// Run the progress events are stamped with.
    pub run_id: &'a RunId,
}

/// Run one phase (`train` or `test`) and emit its progress.
pub fn run_pole_phase(
    run: &PolePhaseRun<'_>,
    probe: &mut dyn PoleProbe,
    present_goal: &mut dyn FnMut([f64; 3]) -> Result<(), TrainerError>,
    events: &mut dyn RunEventSink,
    cancel: &CancelToken,
) -> Result<PolePhaseReport, TrainerError> {
    if run.phase != "train" && run.phase != "test" {
        return Err(TrainerError::Config(
            "pole phase must be train or test".to_string(),
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
    for seed in run.seeds {
        if cancel.is_cancelled() {
            return Err(TrainerError::Cancelled(
                "pole-balance run cancelled".to_string(),
            ));
        }
        let record = run_pole_attempt(
            *seed,
            run.criterion,
            probe,
            present_goal,
            run.poll_timeout,
            cancel,
        )?;
        attempts.push(record);
        events.emit(RunEvent::new(
            run.run_id.clone(),
            RunEventKind::Progress {
                samples_done: attempts.len() as u64,
                samples_total,
                repeat_index: 0,
                repeat_total: 1,
                sent_values: pole_upright_goal_features().to_vec(),
                tick_index: None,
                window_values: Vec::new(),
                image_png_base64: None,
                mask_png_base64: None,
                preview_name: Some(format!("{} seed {seed}", run.phase)),
                predicted_class: None,
                class_scores: Vec::new(),
            },
        ));
    }
    let report = PolePhaseReport::from_attempts(attempts, run.criterion.attempt_burst_cap)?;
    events.emit(RunEvent::new(
        run.run_id.clone(),
        RunEventKind::MetricUpdate {
            scope: MetricScope::Aggregate,
            metrics: pole_phase_metrics(&report),
            class_breakdown: Vec::new(),
        },
    ));
    Ok(report)
}

fn run_pole_attempt(
    seed: u64,
    criterion: &PoleBalanceCriterion,
    probe: &mut dyn PoleProbe,
    present_goal: &mut dyn FnMut([f64; 3]) -> Result<(), TrainerError>,
    poll_timeout: Duration,
    cancel: &CancelToken,
) -> Result<PoleAttemptRecord, TrainerError> {
    let begun = probe.begin_attempt(seed, criterion)?;
    if begun.seed != seed {
        return Err(TrainerError::Runtime(format!(
            "controller homed seed {} for requested seed {seed}",
            begun.seed
        )));
    }
    let features = pole_upright_goal_features();
    let mut open = OpenPoleAttempt::fresh();
    loop {
        if cancel.is_cancelled() {
            return Err(TrainerError::Cancelled(
                "pole-balance run cancelled".to_string(),
            ));
        }
        present_goal(features)?;
        let sample = probe.poll(open.last_seq, poll_timeout)?;
        match observe_pole_sample(&open, &sample, criterion, seed)? {
            PoleAttemptStep::Continue(next) => open = next,
            PoleAttemptStep::Finished(record) => return Ok(record),
        }
    }
}

/// HTTP client for the neutral pole-balance actions on a controller control server.
pub struct HttpPoleProbe {
    host: String,
    port: u16,
    timeout: Duration,
}

impl HttpPoleProbe {
    /// `base_url` is `http://host:port` with no path.
    pub fn new(base_url: &str, timeout: Duration) -> Result<Self, TrainerError> {
        if timeout.is_zero() {
            return Err(TrainerError::Config(
                "pole control timeout must be > 0".to_string(),
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
            TrainerError::Runtime(format!("pole command encode failed: {error}"))
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
            .map_err(|error| TrainerError::Runtime(format!("pole control address: {error}")))?
            .next()
            .ok_or_else(|| {
                TrainerError::Runtime(format!("pole control address {address} did not resolve"))
            })?;
        let mut stream = TcpStream::connect_timeout(&socket, self.timeout).map_err(|error| {
            TrainerError::Runtime(format!("pole control connect failed: {error}"))
        })?;
        stream
            .set_read_timeout(Some(self.timeout))
            .map_err(|error| TrainerError::Runtime(format!("pole control timeout: {error}")))?;
        stream
            .set_write_timeout(Some(self.timeout))
            .map_err(|error| TrainerError::Runtime(format!("pole control timeout: {error}")))?;
        stream.write_all(request.as_bytes()).map_err(|error| {
            TrainerError::Runtime(format!("pole control write failed: {error}"))
        })?;
        stream.write_all(&payload).map_err(|error| {
            TrainerError::Runtime(format!("pole control write failed: {error}"))
        })?;
        let mut raw = Vec::new();
        stream
            .read_to_end(&mut raw)
            .map_err(|error| TrainerError::Runtime(format!("pole control read failed: {error}")))?;
        parse_http_json(&raw)
    }
}

impl PoleProbe for HttpPoleProbe {
    fn begin_attempt(
        &mut self,
        seed: u64,
        criterion: &PoleBalanceCriterion,
    ) -> Result<PoleBegin, TrainerError> {
        let result = self.command(&serde_json::json!({
            "action": "begin_pole_balance_attempt",
            "seed": seed,
            "pole_body": criterion.pole_body,
            "max_tilt_deg": criterion.max_tilt_deg,
        }))?;
        serde_json::from_value(result)
            .map_err(|error| TrainerError::Runtime(format!("pole begin decode failed: {error}")))
    }

    fn poll(&mut self, after_seq: u64, timeout: Duration) -> Result<PoleSample, TrainerError> {
        let timeout_s = timeout.as_secs_f64();
        if timeout_s <= 0.0 {
            return Err(TrainerError::Config(
                "pole poll timeout must be > 0".to_string(),
            ));
        }
        let result = self.command(&serde_json::json!({
            "action": "poll_pole_balance_outcome",
            "after_seq": after_seq,
            "timeout_s": timeout_s,
        }))?;
        serde_json::from_value(result)
            .map_err(|error| TrainerError::Runtime(format!("pole poll decode failed: {error}")))
    }
}

fn split_http_base(base_url: &str) -> Result<(String, u16), TrainerError> {
    let rest = base_url.strip_prefix("http://").ok_or_else(|| {
        TrainerError::Config("pole control URL must start with http://".to_string())
    })?;
    if rest.contains('/') {
        return Err(TrainerError::Config(
            "pole control URL must be http://host:port with no path".to_string(),
        ));
    }
    let (host, port_text) = rest
        .split_once(':')
        .ok_or_else(|| TrainerError::Config("pole control URL must include a port".to_string()))?;
    if host.is_empty() {
        return Err(TrainerError::Config(
            "pole control URL host is empty".to_string(),
        ));
    }
    let port: u16 = port_text
        .parse()
        .map_err(|_| TrainerError::Config(format!("pole control port {port_text} is not a u16")))?;
    Ok((host.to_string(), port))
}

fn parse_http_json(raw: &[u8]) -> Result<serde_json::Value, TrainerError> {
    let text = String::from_utf8_lossy(raw);
    let (head, body) = text.split_once("\r\n\r\n").ok_or_else(|| {
        TrainerError::Runtime("pole control response has no HTTP body".to_string())
    })?;
    let status_line = head.lines().next().unwrap_or("");
    if !status_line.contains(" 200 ") {
        return Err(TrainerError::Runtime(format!(
            "pole control HTTP failed: {status_line}"
        )));
    }
    let envelope: serde_json::Value = serde_json::from_str(body.trim())
        .map_err(|error| TrainerError::Runtime(format!("pole control JSON failed: {error}")))?;
    let ok = envelope
        .get("ok")
        .and_then(|value| value.as_bool())
        .ok_or_else(|| TrainerError::Runtime("pole control response has no ok flag".to_string()))?;
    if !ok {
        let message = envelope
            .get("error")
            .and_then(|value| value.as_str())
            .unwrap_or("pole control command failed");
        return Err(TrainerError::Runtime(message.to_string()));
    }
    envelope
        .get("result")
        .cloned()
        .ok_or_else(|| TrainerError::Runtime("pole control response has no result".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::CollectingEventSink;
    use std::collections::VecDeque;

    struct ScriptedProbe {
        begins: VecDeque<PoleBegin>,
        samples: VecDeque<PoleSample>,
    }

    impl PoleProbe for ScriptedProbe {
        fn begin_attempt(
            &mut self,
            seed: u64,
            _criterion: &PoleBalanceCriterion,
        ) -> Result<PoleBegin, TrainerError> {
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
        ) -> Result<PoleSample, TrainerError> {
            self.samples
                .pop_front()
                .ok_or_else(|| TrainerError::Runtime("no scripted sample".to_string()))
        }
    }

    fn criterion_with_cap(cap: u32) -> PoleBalanceCriterion {
        PoleBalanceCriterion {
            pole_body: "pole".to_string(),
            max_tilt_deg: 12.0,
            attempt_burst_cap: cap,
        }
    }

    fn begin() -> PoleBegin {
        PoleBegin {
            seed: 0,
            pole_body: "pole".to_string(),
            max_tilt_deg: 12.0,
        }
    }

    fn upright(seq: u64, seed: u64) -> PoleSample {
        PoleSample {
            seq,
            seed,
            tilt_deg: 2.0,
            terminal: None,
        }
    }

    #[test]
    fn lasting_the_cap_scores_the_full_attempt() {
        let rule = criterion_with_cap(4);
        let mut open = OpenPoleAttempt::fresh();
        for seq in 1..4 {
            let step = observe_pole_sample(&open, &upright(seq, 7), &rule, 7).expect("step");
            open = match step {
                PoleAttemptStep::Continue(next) => next,
                PoleAttemptStep::Finished(_) => panic!("cap is 4"),
            };
        }
        let finished = observe_pole_sample(&open, &upright(4, 7), &rule, 7).expect("cap");
        match finished {
            PoleAttemptStep::Finished(record) => {
                assert_eq!(record.outcome, PoleAttemptOutcome::Success);
                assert_eq!(record.bursts, 4);
                assert_eq!(record.balanced_bursts, 4);
            }
            PoleAttemptStep::Continue(_) => panic!("expected the cap"),
        }
    }

    #[test]
    fn a_lean_past_the_limit_ends_the_attempt() {
        let open = OpenPoleAttempt {
            bursts: 1,
            last_tilt_deg: 1.0,
            last_seq: 1,
        };
        let mut sample = upright(2, 3);
        sample.tilt_deg = 40.0;
        let step = observe_pole_sample(&open, &sample, &criterion_with_cap(4), 3).expect("step");
        match step {
            PoleAttemptStep::Finished(record) => {
                assert_eq!(record.outcome, PoleAttemptOutcome::Truncated);
                assert_eq!(record.bursts, 2);
                assert_eq!(record.balanced_bursts, 1);
            }
            PoleAttemptStep::Continue(_) => panic!("a lean ends the attempt"),
        }
    }

    #[test]
    fn a_fall_scores_the_upright_share_of_the_cap() {
        let mut open = OpenPoleAttempt::fresh();
        let rule = criterion_with_cap(4);
        for seq in 1..=2 {
            let step = observe_pole_sample(&open, &upright(seq, 3), &rule, 3).expect("upright");
            open = match step {
                PoleAttemptStep::Continue(next) => next,
                PoleAttemptStep::Finished(_) => panic!("still under the cap"),
            };
        }
        let mut lean = upright(3, 3);
        lean.tilt_deg = 40.0;
        let finished = observe_pole_sample(&open, &lean, &rule, 3).expect("fall");
        let record = match finished {
            PoleAttemptStep::Finished(record) => record,
            PoleAttemptStep::Continue(_) => panic!("a lean ends the attempt"),
        };
        let report = PolePhaseReport::from_attempts(vec![record], 4).expect("report");
        assert_eq!(report.success_rate, 0.5);
        assert_eq!(report.median_balanced_bursts, 2.0);
    }

    #[test]
    fn phase_emits_progress_and_success_rate() {
        let mut probe = ScriptedProbe {
            begins: VecDeque::from([begin()]),
            samples: VecDeque::from([upright(1, 1), upright(2, 1)]),
        };
        let mut events = CollectingEventSink::default();
        let run_id = RunId("pole-test".to_string());
        let mut presented = 0;
        let report = run_pole_phase(
            &PolePhaseRun {
                phase: "train",
                seeds: &[1],
                criterion: &criterion_with_cap(2),
                poll_timeout: Duration::from_millis(1),
                run_id: &run_id,
            },
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
        assert_eq!(report.median_balanced_bursts, 2.0);
        assert_eq!(presented, 2);
        let card = pole_phase_scorecard(
            "sc-1",
            "sha256:abc",
            1,
            "train",
            "pole:pole",
            "hash",
            &report,
        );
        assert_eq!(card.metrics.get("success_rate"), Some(&1.0));
    }
}
