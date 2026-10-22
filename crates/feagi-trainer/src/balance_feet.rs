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

use crate::contracts::{
    BackendFingerprint, BackendKind, ConnectomeHash, ContentHash, DatasetAssetId,
    EvaluationProtocolVersion, MetricScope, PluginId, PluginRef, RunEvent, RunEventKind, RunId,
    Scorecard, ScorecardId, ScorecardStatus, ScorecardVisibility, SplitId,
};
use crate::control::{CancelToken, RunEventSink};
use crate::error::TrainerError;
use crate::metrics::episodic_control::EpisodicControlMetricPack;
use crate::reach_hold::trainer_package_version;

/// Operator criterion for one balance-on-feet session.
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
}

impl BalanceFeetCriterion {
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
        Ok(())
    }
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
    /// Bursts observed, including the terminal burst.
    pub bursts: u32,
    /// Torso tilt at the terminal burst, in degrees.
    pub final_tilt_deg: f64,
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
}

/// One stance measurement from the controller.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceSample {
    /// Monotonic sample counter for this attempt.
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
    /// Bursts already observed.
    pub bursts: u32,
    /// Consecutive balanced bursts ending at the latest sample.
    pub holding_bursts: u32,
    /// Latest torso tilt, degrees.
    pub last_tilt_deg: f64,
    /// Latest controller sample sequence.
    pub last_seq: u64,
}

impl OpenBalanceAttempt {
    /// An attempt that has not observed a sample yet.
    pub fn fresh() -> Self {
        Self {
            bursts: 0,
            holding_bursts: 0,
            last_tilt_deg: 0.0,
            last_seq: 0,
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

/// Fold one controller sample into the attempt.
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
    let bursts = open.bursts.saturating_add(1);
    if let Some(terminal) = sample.terminal {
        let outcome = match terminal {
            BalanceTerminal::Death => BalanceAttemptOutcome::Death,
            BalanceTerminal::Interrupted => BalanceAttemptOutcome::Interrupted,
        };
        return Ok(BalanceAttemptStep::Finished(BalanceAttemptRecord {
            seed,
            outcome,
            bursts,
            final_tilt_deg: sample.tilt_deg,
        }));
    }
    let balanced = sample.tilt_deg <= criterion.max_tilt_deg
        && sample.left_foot_contact
        && sample.right_foot_contact;
    let holding_bursts = if balanced {
        open.holding_bursts.saturating_add(1)
    } else {
        0
    };
    if holding_bursts >= criterion.hold_bursts {
        return Ok(BalanceAttemptStep::Finished(BalanceAttemptRecord {
            seed,
            outcome: BalanceAttemptOutcome::Success,
            bursts,
            final_tilt_deg: sample.tilt_deg,
        }));
    }
    if bursts >= criterion.attempt_burst_cap {
        return Ok(BalanceAttemptStep::Finished(BalanceAttemptRecord {
            seed,
            outcome: BalanceAttemptOutcome::Truncated,
            bursts,
            final_tilt_deg: sample.tilt_deg,
        }));
    }
    Ok(BalanceAttemptStep::Continue(OpenBalanceAttempt {
        bursts,
        holding_bursts,
        last_tilt_deg: sample.tilt_deg,
        last_seq: sample.seq,
    }))
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
}

impl BalancePhaseReport {
    /// Summarize finished attempts. An empty phase is an error.
    pub fn from_attempts(attempts: Vec<BalanceAttemptRecord>) -> Result<Self, TrainerError> {
        if attempts.is_empty() {
            return Err(TrainerError::Evaluation(
                "balance phase has no attempts".to_string(),
            ));
        }
        let count = attempts.len() as f64;
        let successes: Vec<u32> = attempts
            .iter()
            .filter(|attempt| attempt.outcome == BalanceAttemptOutcome::Success)
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
        Ok(Self {
            attempts,
            success_rate,
            median_bursts_to_success,
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

/// Named metrics for one finished phase. Median is omitted when nothing succeeded.
pub fn balance_phase_metrics(phase: &BalancePhaseReport) -> BTreeMap<String, f64> {
    let mut metrics = BTreeMap::new();
    metrics.insert("success_rate".to_string(), phase.success_rate);
    metrics.insert("mean_final_tilt_deg".to_string(), phase.mean_final_tilt_deg);
    if let Some(median) = phase.median_bursts_to_success {
        metrics.insert("median_bursts_to_success".to_string(), median);
    }
    metrics
}

/// Scorecard for one balance phase.
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
        evaluation_protocol_version: EvaluationProtocolVersion("balance-feet-v1".to_string()),
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
    /// Home the biped for `seed`.
    fn begin_attempt(
        &mut self,
        seed: u64,
        criterion: &BalanceFeetCriterion,
    ) -> Result<BalanceBegin, TrainerError>;

    /// Wait for a sample newer than `after_seq`.
    fn poll(&mut self, after_seq: u64, timeout: Duration) -> Result<BalanceSample, TrainerError>;
}

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
}

/// Run one phase (`train` or `test`) and emit its progress.
pub fn run_balance_phase(
    run: &BalancePhaseRun<'_>,
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
    let samples_total = run.seeds.len() as u64;
    let mut attempts = Vec::with_capacity(run.seeds.len());
    for seed in run.seeds {
        if cancel.is_cancelled() {
            return Err(TrainerError::Cancelled(
                "balance-on-feet run cancelled".to_string(),
            ));
        }
        let record = run_balance_attempt(
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
                sent_values: upright_goal_features().to_vec(),
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
    let report = BalancePhaseReport::from_attempts(attempts)?;
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

fn run_balance_attempt(
    seed: u64,
    criterion: &BalanceFeetCriterion,
    probe: &mut dyn BalanceProbe,
    present_goal: &mut dyn FnMut([f64; 3]) -> Result<(), TrainerError>,
    poll_timeout: Duration,
    cancel: &CancelToken,
) -> Result<BalanceAttemptRecord, TrainerError> {
    let begun = probe.begin_attempt(seed, criterion)?;
    if begun.seed != seed {
        return Err(TrainerError::Runtime(format!(
            "controller homed seed {} for requested seed {seed}",
            begun.seed
        )));
    }
    let features = upright_goal_features();
    let mut open = OpenBalanceAttempt::fresh();
    loop {
        if cancel.is_cancelled() {
            return Err(TrainerError::Cancelled(
                "balance-on-feet run cancelled".to_string(),
            ));
        }
        present_goal(features)?;
        let sample = probe.poll(open.last_seq, poll_timeout)?;
        match observe_balance_sample(&open, &sample, criterion, seed)? {
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
    ) -> Result<BalanceBegin, TrainerError> {
        let result = self.command(&serde_json::json!({
            "action": "begin_balance_attempt",
            "seed": seed,
            "torso_body": criterion.torso_body,
            "left_foot_body": criterion.left_foot_body,
            "right_foot_body": criterion.right_foot_body,
            "max_tilt_deg": criterion.max_tilt_deg,
        }))?;
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
    }

    impl BalanceProbe for ScriptedProbe {
        fn begin_attempt(
            &mut self,
            seed: u64,
            _criterion: &BalanceFeetCriterion,
        ) -> Result<BalanceBegin, TrainerError> {
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

    fn criterion() -> BalanceFeetCriterion {
        BalanceFeetCriterion {
            torso_body: "torso".to_string(),
            left_foot_body: "left_foot".to_string(),
            right_foot_body: "right_foot".to_string(),
            max_tilt_deg: 15.0,
            hold_bursts: 2,
            attempt_burst_cap: 4,
        }
    }

    fn begin() -> BalanceBegin {
        BalanceBegin {
            seed: 0,
            torso_body: "torso".to_string(),
            left_foot_body: "left_foot".to_string(),
            right_foot_body: "right_foot".to_string(),
            max_tilt_deg: 15.0,
        }
    }

    fn balanced(seq: u64, seed: u64) -> BalanceSample {
        BalanceSample {
            seq,
            seed,
            tilt_deg: 2.0,
            left_foot_contact: true,
            right_foot_contact: true,
            terminal: None,
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
            }
            BalanceAttemptStep::Continue(_) => panic!("expected success"),
        }
    }

    #[test]
    fn a_lifted_foot_resets_the_hold() {
        let open = OpenBalanceAttempt {
            bursts: 1,
            holding_bursts: 1,
            last_tilt_deg: 1.0,
            last_seq: 1,
        };
        let mut sample = balanced(2, 3);
        sample.right_foot_contact = false;
        let step = observe_balance_sample(&open, &sample, &criterion(), 3).expect("step");
        match step {
            BalanceAttemptStep::Continue(next) => assert_eq!(next.holding_bursts, 0),
            BalanceAttemptStep::Finished(_) => panic!("one missed foot is not the cap"),
        }
    }

    #[test]
    fn phase_emits_progress_and_success_rate() {
        let mut probe = ScriptedProbe {
            begins: VecDeque::from([begin()]),
            samples: VecDeque::from([balanced(1, 1), balanced(2, 1)]),
        };
        let mut events = CollectingEventSink::default();
        let run_id = RunId("balance-test".to_string());
        let mut presented = 0;
        let report = run_balance_phase(
            &BalancePhaseRun {
                phase: "train",
                seeds: &[1],
                criterion: &criterion(),
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
        assert_eq!(presented, 2);
        assert!(report.median_bursts_to_success.is_some());
        let card = balance_phase_scorecard(
            "sc-1",
            "sha256:abc",
            1,
            "train",
            "balance:torso/left_foot/right_foot",
            "hash",
            &report,
        );
        assert_eq!(card.metrics.get("success_rate"), Some(&1.0));
    }
}
