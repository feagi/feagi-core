//! Shared attempt loop for the closed-loop skill engines (reach-and-hold, grasp-and-hold).
//!
//! An engine supplies the rules for one kind of attempt through [`AttemptRules`]: how to begin
//! it, how to poll a controller sample, how one sample advances the attempt and what reward it
//! earns. This module owns the parts every such engine repeats: the seed phase loop with its
//! pause, skip, cancel and resume handling, the attempt skeleton (begin, step, poll, record,
//! reward, finish), the controller HTTP client and the seconds-to-bursts conversion. It holds
//! no knowledge of any body, object or task.

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::binding::reward::RewardSignal;
use crate::contracts::{RunEvent, RunEventKind, RunId};
use crate::control::{CancelToken, RunEventSink};
use crate::error::TrainerError;
use crate::resume::{ResumeExport, ResumeFrom};
use crate::run_data::{EventTime, RunDataSink, RunDataValue, TableSpec};

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

/// Controller-latched end cause. Success is decided by the Trainer, never by the controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptTerminal {
    /// `_death` fired.
    Death,
    /// The watched body left the safety box.
    Fence,
    /// Misc reset fired while the attempt was active.
    Interrupted,
}

/// One write the attempt loop asks the caller to send to FEAGI.
#[derive(Debug, Clone, PartialEq)]
pub enum CoachWrite {
    /// Advance one burst. The brain reads its own sensors.
    Step,
    /// Train-phase pain or pleasure.
    Affect(RewardSignal),
}

/// Result of folding one sample into an open attempt.
#[derive(Debug, Clone, PartialEq)]
pub enum AttemptFlow<Open, Record> {
    /// The attempt is still running.
    Continue(Open),
    /// The attempt ended.
    Finished(Record),
}

/// One folded sample: the attempt's next state and the reward signals it earned.
pub type Observed<Rules> = (
    AttemptFlow<<Rules as AttemptRules>::Open, <Rules as AttemptRules>::Record>,
    Vec<RewardSignal>,
);

/// The rules of one kind of attempt, driven by [`run_attempt`].
pub trait AttemptRules {
    /// One controller sample.
    type Sample;
    /// In-progress attempt state.
    type Open;
    /// One finished attempt.
    type Record;

    /// Message of the error returned when the run is cancelled.
    fn cancelled_message(&self) -> &'static str;
    /// Home the body and place the scene for `seed`.
    fn begin(&mut self, seed: u64) -> Result<(), TrainerError>;
    /// State of an attempt that has not observed a sample yet.
    fn fresh(&self) -> Self::Open;
    /// Bursts the open attempt has observed (event step index).
    fn open_bursts(&self, open: &Self::Open) -> u32;
    /// Sequence the next poll must be newer than.
    fn open_seq(&self, open: &Self::Open) -> u64;
    /// Wait for a sample newer than `after_seq`.
    fn poll(&mut self, after_seq: u64, timeout: Duration) -> Result<Self::Sample, TrainerError>;
    /// Run-data row of one sample for the `steps` table.
    fn step_row(&self, phase: &str, attempt: u64, sample: &Self::Sample) -> Vec<RunDataValue>;
    /// Fold one sample into the attempt. The signals are what a train phase sends to FEAGI (a
    /// test phase passes `train_affect = false` and must get none).
    fn observe(
        &mut self,
        open: &Self::Open,
        sample: &Self::Sample,
        seed: u64,
        train_affect: bool,
    ) -> Result<Observed<Self>, TrainerError>;
    /// Bursts of a finished attempt (event step index).
    fn record_bursts(&self, record: &Self::Record) -> u32;
    /// Outcome name of a finished attempt (event label).
    fn record_outcome(&self, record: &Self::Record) -> String;
}

/// Where one attempt writes its rows and events.
pub struct AttemptData<'s, 'a> {
    /// The run-data sink, when the run records data.
    pub sink: Option<&'s mut (dyn RunDataSink + 'a)>,
    /// `train` or `test`.
    pub phase: &'s str,
    /// Attempt index within the phase.
    pub attempt: u64,
}

/// Name of the per-sample table every engine declares.
pub const STEPS_TABLE: &str = "steps";
/// Name of the per-attempt table every engine declares.
pub const ATTEMPTS_TABLE: &str = "attempts";

/// Run one attempt: begin, then step, poll, record and fold samples until the rules finish it.
pub fn run_attempt<R: AttemptRules>(
    rules: &mut R,
    seed: u64,
    train_affect: bool,
    poll_timeout: Duration,
    coach: &mut dyn FnMut(CoachWrite) -> Result<(), TrainerError>,
    cancel: &CancelToken,
    mut data: AttemptData<'_, '_>,
) -> Result<R::Record, TrainerError> {
    rules.begin(seed)?;
    let attempt = data.attempt;
    let at = |step: u64| EventTime {
        sim_time_s: None,
        step: Some(step),
        episode: Some(attempt),
    };
    if let Some(sink) = data.sink.as_deref_mut() {
        sink.event(at(0), "attempt_start", &format!("seed {seed}"));
    }
    let mut open = rules.fresh();
    loop {
        if cancel.is_cancelled() {
            return Err(TrainerError::Cancelled(
                rules.cancelled_message().to_string(),
            ));
        }
        coach(CoachWrite::Step)?;
        let after_seq = rules.open_seq(&open);
        let sample = rules.poll(after_seq, poll_timeout)?;
        if let Some(sink) = data.sink.as_deref_mut() {
            sink.row(STEPS_TABLE, &rules.step_row(data.phase, attempt, &sample));
        }
        let bursts_before = rules.open_bursts(&open);
        let (flow, signals) = rules.observe(&open, &sample, seed, train_affect)?;
        for signal in signals {
            if let Some(sink) = data.sink.as_deref_mut() {
                sink.event(
                    at(u64::from(bursts_before)),
                    "affect",
                    &format!("{:?} {}", signal.channel, signal.magnitude).to_lowercase(),
                );
            }
            coach(CoachWrite::Affect(signal))?;
        }
        match flow {
            AttemptFlow::Continue(next) => open = next,
            AttemptFlow::Finished(record) => {
                if let Some(sink) = data.sink.as_deref_mut() {
                    sink.event(
                        at(u64::from(rules.record_bursts(&record))),
                        "attempt_end",
                        &rules.record_outcome(&record),
                    );
                }
                return Ok(record);
            }
        }
    }
}

/// What one phase needs besides its attempts.
pub struct PhaseSpec<'a> {
    /// `train` or `test`.
    pub phase: &'a str,
    /// Seeds visited in order.
    pub seeds: &'a [u64],
    /// Run the progress events are stamped with.
    pub run_id: &'a RunId,
    /// Operator-facing name of the engine, such as `reach-and-hold`, in the cancel message.
    pub label: &'a str,
    /// Tables the engine declares before its first row (`steps` and `attempts`).
    pub tables: Vec<TableSpec>,
}

/// Attempts of one phase segment and where it stopped.
#[derive(Debug, Clone, PartialEq)]
pub struct PhaseOutcome<Record> {
    /// Attempts run in this segment.
    pub attempts: Vec<Record>,
    /// The host skipped the phase after the attempt in progress.
    pub skipped: bool,
    /// What the next segment resumes from.
    pub export: ResumeExport,
}

/// Run the seeds of one phase from `resume.first_index`.
///
/// `attempt` runs one seed (given the attempt index and its data context). `attempt_row` is the
/// `attempts` table row of a finished one. The caller builds the phase report and emits its
/// metric update from the returned attempts.
pub fn run_phase_loop<Record>(
    spec: &PhaseSpec<'_>,
    resume: &ResumeFrom,
    events: &mut dyn RunEventSink,
    cancel: &CancelToken,
    mut data: Option<&mut (dyn RunDataSink + '_)>,
    attempt: &mut dyn FnMut(u64, AttemptData<'_, '_>) -> Result<Record, TrainerError>,
    attempt_row: &dyn Fn(&str, u64, &Record) -> Vec<RunDataValue>,
) -> Result<PhaseOutcome<Record>, TrainerError> {
    let samples_total = spec.seeds.len() as u64;
    resume.check_within(samples_total, "attempt")?;
    let first = resume.first_index as usize;
    let mut attempts = Vec::with_capacity(spec.seeds.len());
    if let Some(sink) = data.as_deref_mut() {
        for table in &spec.tables {
            sink.declare_table(table);
        }
        sink.event(EventTime::default(), "phase_start", spec.phase);
    }
    let mut skipped = false;
    for seed in &spec.seeds[first..] {
        // Park while paused at the attempt boundary (the attempt in progress is never cut;
        // a skip wakes the park), then honor a cancel.
        cancel.interrupt(format!("{} run cancelled", spec.label))?;
        if cancel.is_skipped() && !attempts.is_empty() {
            skipped = true;
            break;
        }
        let attempt_index = resume.first_index + attempts.len() as u64;
        let record = attempt(
            *seed,
            AttemptData {
                sink: data.as_deref_mut(),
                phase: spec.phase,
                attempt: attempt_index,
            },
        )?;
        if let Some(sink) = data.as_deref_mut() {
            sink.row(
                ATTEMPTS_TABLE,
                &attempt_row(spec.phase, attempt_index, &record),
            );
        }
        attempts.push(record);
        events.emit(RunEvent::new(
            spec.run_id.clone(),
            RunEventKind::Progress {
                samples_done: resume.first_index + attempts.len() as u64,
                samples_total,
                repeat_index: 0,
                repeat_total: 1,
                sent_values: Vec::new(),
                tick_index: None,
                window_values: Vec::new(),
                image_png_base64: None,
                mask_png_base64: None,
                preview_name: Some(format!("{} seed {seed}", spec.phase)),
                readout: None,
                class_scores: Vec::new(),
                stand: None,
            },
        ));
    }
    if let Some(sink) = data {
        sink.event(
            EventTime {
                episode: Some(resume.first_index + attempts.len() as u64),
                ..EventTime::default()
            },
            "phase_end",
            spec.phase,
        );
    }
    let done = resume.first_index + attempts.len() as u64;
    let export = ResumeExport {
        units_done: done,
        engine_state: serde_json::Value::Null,
        partial: done < samples_total,
    };
    Ok(PhaseOutcome {
        attempts,
        skipped,
        export,
    })
}

/// HTTP client for the neutral actions of a controller control server.
///
/// `label` names the engine in every error text, such as `reach control`.
pub struct HttpControl {
    host: String,
    port: u16,
    timeout: Duration,
    label: &'static str,
}

impl HttpControl {
    /// `base_url` is `http://host:port` with no path.
    pub fn new(
        base_url: &str,
        timeout: Duration,
        label: &'static str,
    ) -> Result<Self, TrainerError> {
        if timeout.is_zero() {
            return Err(TrainerError::Config(format!("{label} timeout must be > 0")));
        }
        let (host, port) = split_http_base(base_url, label)?;
        Ok(Self {
            host,
            port,
            timeout,
            label,
        })
    }

    /// Send one `POST /command` and return the `result` of the reply.
    pub fn command(&self, body: &serde_json::Value) -> Result<serde_json::Value, TrainerError> {
        let label = self.label;
        let payload = serde_json::to_vec(body).map_err(|error| {
            TrainerError::Runtime(format!("{label} command encode failed: {error}"))
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
            .map_err(|error| TrainerError::Runtime(format!("{label} address: {error}")))?
            .next()
            .ok_or_else(|| {
                TrainerError::Runtime(format!("{label} address {address} did not resolve"))
            })?;
        let mut stream = TcpStream::connect_timeout(&socket, self.timeout)
            .map_err(|error| TrainerError::Runtime(format!("{label} connect failed: {error}")))?;
        stream
            .set_read_timeout(Some(self.timeout))
            .map_err(|error| TrainerError::Runtime(format!("{label} timeout: {error}")))?;
        stream
            .set_write_timeout(Some(self.timeout))
            .map_err(|error| TrainerError::Runtime(format!("{label} timeout: {error}")))?;
        stream
            .write_all(request.as_bytes())
            .map_err(|error| TrainerError::Runtime(format!("{label} write failed: {error}")))?;
        stream
            .write_all(&payload)
            .map_err(|error| TrainerError::Runtime(format!("{label} write failed: {error}")))?;
        let mut raw = Vec::new();
        stream
            .read_to_end(&mut raw)
            .map_err(|error| TrainerError::Runtime(format!("{label} read failed: {error}")))?;
        parse_http_json(&raw, label)
    }
}

fn split_http_base(base_url: &str, label: &str) -> Result<(String, u16), TrainerError> {
    let rest = base_url
        .strip_prefix("http://")
        .ok_or_else(|| TrainerError::Config(format!("{label} URL must start with http://")))?;
    if rest.contains('/') {
        return Err(TrainerError::Config(format!(
            "{label} URL must be http://host:port with no path"
        )));
    }
    let (host, port_text) = rest
        .split_once(':')
        .ok_or_else(|| TrainerError::Config(format!("{label} URL must include a port")))?;
    if host.is_empty() {
        return Err(TrainerError::Config(format!("{label} URL host is empty")));
    }
    let port: u16 = port_text
        .parse()
        .map_err(|_| TrainerError::Config(format!("{label} port {port_text} is not a u16")))?;
    Ok((host.to_string(), port))
}

fn parse_http_json(raw: &[u8], label: &str) -> Result<serde_json::Value, TrainerError> {
    let text = String::from_utf8_lossy(raw);
    let (head, body) = text
        .split_once("\r\n\r\n")
        .ok_or_else(|| TrainerError::Runtime(format!("{label} response has no HTTP body")))?;
    let status_line = head.lines().next().unwrap_or("");
    if !status_line.contains(" 200 ") {
        return Err(TrainerError::Runtime(format!(
            "{label} HTTP failed: {status_line}"
        )));
    }
    let envelope: serde_json::Value = serde_json::from_str(body.trim())
        .map_err(|error| TrainerError::Runtime(format!("{label} JSON failed: {error}")))?;
    let ok = envelope
        .get("ok")
        .and_then(|value| value.as_bool())
        .ok_or_else(|| TrainerError::Runtime(format!("{label} response has no ok flag")))?;
    if !ok {
        let message = envelope
            .get("error")
            .and_then(|value| value.as_str())
            .map(str::to_string)
            .unwrap_or_else(|| format!("{label} command failed"));
        return Err(TrainerError::Runtime(message));
    }
    envelope
        .get("result")
        .cloned()
        .ok_or_else(|| TrainerError::Runtime(format!("{label} response has no result")))
}
