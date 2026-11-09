//! Recovery of a balance phase from a lost or restarting controller link, or a paused or stalled
//! FEAGI burst engine.
//!
//! The balance loop scores a robot through a controller that is itself linked to FEAGI. When
//! that link drops, the controller restarts or the burst engine stops, a robot keeps moving with
//! no brain behind it. Scoring such an attempt would be wrong, and aborting the phase on one
//! transient error would throw away a long run. This module keeps both from happening:
//!
//! - [`AttemptWatch`] judges one attempt. Every response carries the controller's FEAGI link
//!   (`link: {state, epoch}`, required); it is compared with the first one the attempt saw. A
//!   state other than `linked`, a different epoch, a burst counter that stops rising for
//!   `burst_stall_s`, or a [`TrainerError::Transient`] error discards the attempt.
//! - [`RecoveryState`] handles a discard: it records it ([`DiscardedAttempt`]), waits for the
//!   link to come back within the caller's [`RecoveryConfig`], and lets the caller re-run the
//!   SAME seed. A discarded attempt never reaches the scores.
//! - [`RecoveryState::boundary`] runs between attempts: it parks while the caller's pause is
//!   set and while the burst engine is paused, the latter within `max_outage_s`.
//!
//! FEAGI itself is outside this crate: the caller supplies a [`BurstClock`]. Every limit comes
//! from the caller's [`RecoveryConfig`]; this module has no defaults. It knows no robot,
//! simulator or task.

use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::balance_feet::BalanceController;
use crate::control::CancelToken;
use crate::error::TrainerError;
use crate::run_data::{EventTime, RunDataSink};

/// Event kind written to the run data log for every discarded attempt.
pub const ATTEMPT_DISCARDED_EVENT: &str = "attempt_discarded";

/// `burst_stall_s` must be at least this many times the longest poll interval, so one slow poll
/// is never read as a stall. A fixed property of the detector, not a setting.
pub const BURST_STALL_MIN_POLL_MULTIPLE: f64 = 3.0;

/// The FEAGI burst engine as the balance phase needs to see it. FEAGI is outside this crate, so
/// the host implements this over whatever reaches it. A failed read is classified by its error:
/// [`TrainerError::Transient`] for a transport failure that may pass, anything else is fatal.
pub trait BurstClock {
    /// Bursts FEAGI has run so far. Any change counts as progress.
    fn burst_count(&self) -> Result<u64, TrainerError>;

    /// Whether the burst engine is paused.
    fn engine_paused(&self) -> Result<bool, TrainerError>;
}

/// State of the controller's link to FEAGI, as the controller reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkState {
    /// The controller is connected to FEAGI.
    Linked,
    /// The controller lost the link and is trying to restore it.
    Reconnecting,
    /// The controller stopped trying; the link will not come back by itself.
    GivenUp,
}

impl LinkState {
    /// Wire name, as in the controller's `link.state`.
    pub fn name(self) -> &'static str {
        match self {
            Self::Linked => "linked",
            Self::Reconnecting => "reconnecting",
            Self::GivenUp => "given_up",
        }
    }
}

/// The `link: {state, epoch}` block of a controller response. The epoch increases by one on
/// every link state transition, so two readings with different epochs bracket a transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkReading {
    /// Link state at the time of the response.
    pub state: LinkState,
    /// Transition counter of the link.
    pub epoch: u64,
}

/// Recovery limits. The caller passes every value; there is no default.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RecoveryConfig {
    /// Longest total wait after one discarded attempt for the link to come back, and longest
    /// wait at an attempt boundary for a paused burst engine to resume, seconds.
    pub max_outage_s: f64,
    /// Most attempts one phase may discard before it fails.
    pub max_discarded_attempts: u32,
    /// Interval at which the controller status is re-read while waiting for the link, seconds.
    pub link_poll_s: f64,
    /// Interval at which the burst engine is re-read while parked at an attempt boundary,
    /// seconds.
    pub boundary_poll_s: f64,
    /// A burst counter that does not change for this long during an attempt discards it,
    /// seconds. At least [`BURST_STALL_MIN_POLL_MULTIPLE`] times each poll interval, and (checked
    /// by the phase) the same multiple of the criterion's burst period.
    pub burst_stall_s: f64,
}

impl RecoveryConfig {
    /// Builds and validates a config.
    ///
    /// # Errors
    /// [`TrainerError::Config`] when a value is not finite and > 0, or `burst_stall_s` is too
    /// close to a poll interval.
    pub fn new(
        max_outage_s: f64,
        max_discarded_attempts: u32,
        link_poll_s: f64,
        boundary_poll_s: f64,
        burst_stall_s: f64,
    ) -> Result<Self, TrainerError> {
        let config = Self {
            max_outage_s,
            max_discarded_attempts,
            link_poll_s,
            boundary_poll_s,
            burst_stall_s,
        };
        config.validate()?;
        Ok(config)
    }

    /// Checks every value; a phase calls this before its first attempt.
    ///
    /// # Errors
    /// [`TrainerError::Config`] naming the offending field.
    pub fn validate(&self) -> Result<(), TrainerError> {
        for (name, value) in [
            ("max_outage_s", self.max_outage_s),
            ("link_poll_s", self.link_poll_s),
            ("boundary_poll_s", self.boundary_poll_s),
            ("burst_stall_s", self.burst_stall_s),
        ] {
            if !value.is_finite() || value <= 0.0 || Duration::try_from_secs_f64(value).is_err() {
                return Err(TrainerError::Config(format!(
                    "recovery {name} must be a finite number of seconds > 0, got {value}"
                )));
            }
        }
        if self.max_discarded_attempts == 0 {
            return Err(TrainerError::Config(
                "recovery max_discarded_attempts must be > 0".to_string(),
            ));
        }
        self.check_stall_against("link_poll_s", self.link_poll_s)?;
        self.check_stall_against("boundary_poll_s", self.boundary_poll_s)
    }

    /// Checks `burst_stall_s` against the criterion's burst period: a stall shorter than a few
    /// bursts would flag a healthy engine.
    ///
    /// # Errors
    /// [`TrainerError::Config`] when `burst_stall_s` is too close to `burst_period_s`.
    pub fn validate_for_burst_period(&self, burst_period_s: f64) -> Result<(), TrainerError> {
        self.check_stall_against("the criterion's burst_period_s", burst_period_s)
    }

    fn check_stall_against(&self, what: &str, interval_s: f64) -> Result<(), TrainerError> {
        let least = BURST_STALL_MIN_POLL_MULTIPLE * interval_s;
        if self.burst_stall_s + 1.0e-12 < least {
            return Err(TrainerError::Config(format!(
                "recovery burst_stall_s ({}) must be at least {BURST_STALL_MIN_POLL_MULTIPLE} x \
                 {what} ({interval_s}), i.e. {least}",
                self.burst_stall_s
            )));
        }
        Ok(())
    }
}

/// Why an attempt was discarded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscardReason {
    /// A response showed a link state other than `linked`.
    LinkNotLinked,
    /// A response showed a different link epoch than the attempt started with.
    LinkEpochChanged,
    /// The burst counter did not change for `burst_stall_s` during the attempt.
    BurstStall,
    /// A call failed with [`TrainerError::Transient`].
    TransientError,
}

impl DiscardReason {
    /// Wire name (the serde string).
    pub fn name(self) -> &'static str {
        match self {
            Self::LinkNotLinked => "link_not_linked",
            Self::LinkEpochChanged => "link_epoch_changed",
            Self::BurstStall => "burst_stall",
            Self::TransientError => "transient_error",
        }
    }
}

/// One attempt thrown away because the controller's FEAGI link, the burst engine or a call to
/// them failed. It is excluded from every score and kept in the phase report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscardedAttempt {
    /// Index the re-run takes among the phase's attempts (0-based, as in the `attempts` table).
    pub attempt_index: u32,
    /// Seed of the discarded attempt; the re-run uses the same seed.
    pub seed: u64,
    /// The discarded attempt was a probe.
    pub probe: bool,
    /// Why it was discarded.
    pub reason: DiscardReason,
    /// What was observed.
    pub detail: String,
    /// Link epoch the attempt started with. Absent when it failed before the first reply.
    pub link_epoch_before: Option<u64>,
    /// Link epoch of the last response of the attempt.
    pub link_epoch_after: Option<u64>,
    /// Link state of the last response of the attempt.
    pub link_state_after: Option<LinkState>,
}

impl DiscardedAttempt {
    /// One line for logs and the run data event.
    fn summary(&self) -> String {
        let epoch = |value: Option<u64>| value.map_or("none".to_string(), |e| e.to_string());
        format!(
            "{} seed {} discarded ({}; link epoch {} -> {}): {}",
            if self.probe { "probe" } else { "attempt" },
            self.seed,
            self.reason.name(),
            epoch(self.link_epoch_before),
            epoch(self.link_epoch_after),
            self.detail
        )
    }
}

/// An observation that ended an attempt.
#[derive(Debug, Clone)]
struct Violation {
    reason: DiscardReason,
    detail: String,
}

/// Burst counter progress within one attempt.
struct BurstProgress {
    count: u64,
    changed_at: Instant,
}

/// Judges one attempt: its link readings against the first one it saw, and the burst counter
/// against `burst_stall_s`.
pub struct AttemptWatch<'c> {
    clock: &'c dyn BurstClock,
    burst_stall: Duration,
    baseline: Option<LinkReading>,
    last: Option<LinkReading>,
    progress: Option<BurstProgress>,
    violation: Option<Violation>,
}

impl<'c> AttemptWatch<'c> {
    /// A watch with no reading yet; the first link reading becomes the baseline and the first
    /// burst count starts the stall clock.
    fn new(clock: &'c dyn BurstClock, burst_stall: Duration) -> Self {
        Self {
            clock,
            burst_stall,
            baseline: None,
            last: None,
            progress: None,
            violation: None,
        }
    }

    /// Takes one response's link.
    ///
    /// # Errors
    /// [`TrainerError::Transient`] when the state is not `linked` or the epoch differs from the
    /// baseline; the attempt must end and be discarded.
    pub fn observe(&mut self, reading: LinkReading) -> Result<(), TrainerError> {
        self.last = Some(reading);
        let baseline = *self.baseline.get_or_insert(reading);
        if reading.state != LinkState::Linked {
            return self.fail(
                DiscardReason::LinkNotLinked,
                format!(
                    "FEAGI link is {} (epoch {})",
                    reading.state.name(),
                    reading.epoch
                ),
            );
        }
        if reading.epoch != baseline.epoch {
            return self.fail(
                DiscardReason::LinkEpochChanged,
                format!(
                    "FEAGI link epoch changed from {} to {}",
                    baseline.epoch, reading.epoch
                ),
            );
        }
        Ok(())
    }

    /// Reads the burst counter. Call it once per burst of the attempt.
    ///
    /// # Errors
    /// [`TrainerError::Transient`] when the counter has not changed for `burst_stall_s` (the
    /// attempt must be discarded), or the clock's own error.
    pub fn observe_bursts(&mut self) -> Result<(), TrainerError> {
        let count = self.clock.burst_count()?;
        let now = Instant::now();
        let stalled_s = match &mut self.progress {
            Some(progress) if progress.count == count => {
                now.duration_since(progress.changed_at).as_secs_f64()
            }
            slot => {
                *slot = Some(BurstProgress {
                    count,
                    changed_at: now,
                });
                return Ok(());
            }
        };
        if stalled_s + 1.0e-12 < self.burst_stall.as_secs_f64() {
            return Ok(());
        }
        self.fail(
            DiscardReason::BurstStall,
            format!(
                "burst counter stuck at {count} for {stalled_s:.3} s (burst_stall_s {})",
                self.burst_stall.as_secs_f64()
            ),
        )
    }

    fn fail(&mut self, reason: DiscardReason, detail: String) -> Result<(), TrainerError> {
        self.violation = Some(Violation {
            reason,
            detail: detail.clone(),
        });
        Err(TrainerError::Transient(detail))
    }
}

/// What became of one attempt.
#[derive(Debug)]
pub enum AttemptVerdict<T> {
    /// The attempt ran with a healthy link and its result may be scored.
    Completed(T),
    /// The attempt must be thrown away and re-run.
    Discard(DiscardedAttempt),
}

/// Where an attempt sits in its phase.
#[derive(Debug, Clone, Copy)]
pub struct AttemptSlot {
    /// Index the attempt takes among the phase's attempts.
    pub attempt_index: usize,
    /// Seed of the attempt.
    pub seed: u64,
    /// The attempt is a probe.
    pub probe: bool,
}

/// Discard bookkeeping of one phase.
#[derive(Debug)]
pub struct RecoveryState {
    config: RecoveryConfig,
    discarded: Vec<DiscardedAttempt>,
}

impl RecoveryState {
    /// Starts the bookkeeping for one phase.
    ///
    /// # Errors
    /// [`TrainerError::Config`] when `config` is invalid.
    pub fn new(config: &RecoveryConfig) -> Result<Self, TrainerError> {
        config.validate()?;
        Ok(Self {
            config: *config,
            discarded: Vec::new(),
        })
    }

    /// A watch for one attempt.
    pub fn watch<'c>(&self, clock: &'c dyn BurstClock) -> AttemptWatch<'c> {
        AttemptWatch::new(clock, Duration::from_secs_f64(self.config.burst_stall_s))
    }

    /// The attempt boundary: the one place the phase parks. Honors the host's pause (without
    /// charging it to the budget), then holds while the burst engine is paused, for at most
    /// `max_outage_s`, polling every `boundary_poll_s`. A transient read failure of the clock
    /// counts as "not resumed yet".
    ///
    /// # Errors
    /// [`TrainerError::Cancelled`] when the run is cancelled; [`TrainerError::Runtime`] naming
    /// `burst engine paused` and the observed values when the budget runs out or the clock fails
    /// for any reason other than a transient one.
    pub fn boundary(
        &self,
        clock: &dyn BurstClock,
        cancel: &CancelToken,
    ) -> Result<(), TrainerError> {
        let cancelled = || "balance-on-feet run cancelled".to_string();
        let started = Instant::now();
        let mut parked_by_host = Duration::ZERO;
        loop {
            let hold = Instant::now();
            cancel.interrupt(cancelled())?;
            parked_by_host += hold.elapsed();
            let seen = match clock.engine_paused() {
                Ok(false) => return Ok(()),
                Ok(true) => "engine paused".to_string(),
                Err(TrainerError::Transient(message)) => {
                    format!("engine state unreadable: {message}")
                }
                Err(other) => return Err(other),
            };
            let waited_s = started
                .elapsed()
                .saturating_sub(parked_by_host)
                .as_secs_f64();
            if waited_s >= self.config.max_outage_s {
                let count = clock
                    .burst_count()
                    .map_or_else(|error| format!("unreadable ({error})"), |c| c.to_string());
                return Err(TrainerError::Runtime(format!(
                    "balance phase failed: burst engine paused for {waited_s:.3} s, beyond \
                     max_outage_s ({}); last seen: {seen}; burst count {count}",
                    self.config.max_outage_s
                )));
            }
            let nap_s = self
                .config
                .boundary_poll_s
                .min(self.config.max_outage_s - waited_s);
            if cancel.wait_cancelled(Duration::from_secs_f64(nap_s)) {
                return Err(TrainerError::Cancelled(cancelled()));
            }
        }
    }

    /// Sorts an attempt result into scored or discarded. Any error other than
    /// [`TrainerError::Transient`] passes through unchanged and fails the phase.
    ///
    /// # Errors
    /// The attempt's own error when it is not transient.
    pub fn classify<T>(
        result: Result<T, TrainerError>,
        watch: &AttemptWatch<'_>,
        slot: AttemptSlot,
    ) -> Result<AttemptVerdict<T>, TrainerError> {
        let message = match result {
            Ok(value) => return Ok(AttemptVerdict::Completed(value)),
            Err(TrainerError::Transient(message)) => message,
            Err(other) => return Err(other),
        };
        let (reason, detail) = match &watch.violation {
            Some(violation) => (violation.reason, violation.detail.clone()),
            None => (DiscardReason::TransientError, message),
        };
        Ok(AttemptVerdict::Discard(DiscardedAttempt {
            attempt_index: u32::try_from(slot.attempt_index).unwrap_or(u32::MAX),
            seed: slot.seed,
            probe: slot.probe,
            reason,
            detail,
            link_epoch_before: watch.baseline.map(|link| link.epoch),
            link_epoch_after: watch.last.map(|link| link.epoch),
            link_state_after: watch.last.map(|link| link.state),
        }))
    }

    /// Records a discard, then waits for the link so the caller can re-run the same seed.
    ///
    /// # Errors
    /// [`TrainerError::Runtime`] naming the limit that failed (`given_up`,
    /// `max_discarded_attempts` or `max_outage_s`) and the observed values;
    /// [`TrainerError::Cancelled`] when the run is cancelled while waiting.
    pub fn recover(
        &mut self,
        discard: DiscardedAttempt,
        controller: &mut dyn BalanceController,
        cancel: &CancelToken,
        sink: Option<&mut (dyn RunDataSink + '_)>,
    ) -> Result<(), TrainerError> {
        let summary = discard.summary();
        if let Some(sink) = sink {
            sink.event(
                EventTime {
                    episode: Some(u64::from(discard.attempt_index)),
                    ..EventTime::default()
                },
                ATTEMPT_DISCARDED_EVENT,
                &summary,
            );
        }
        let given_up = discard.link_state_after == Some(LinkState::GivenUp);
        self.discarded.push(discard);
        if given_up {
            return Err(TrainerError::Runtime(format!(
                "balance phase failed: the controller's FEAGI link is given_up and will not \
                 come back ({summary})"
            )));
        }
        if self.discarded.len() as u64 > u64::from(self.config.max_discarded_attempts) {
            return Err(TrainerError::Runtime(format!(
                "balance phase failed: max_discarded_attempts ({}) exceeded with {} discarded \
                 attempts in this phase (last: {summary})",
                self.config.max_discarded_attempts,
                self.discarded.len()
            )));
        }
        wait_for_link(controller, &self.config, cancel)
    }

    /// Discards of the phase so far, in order.
    pub fn discarded(&self) -> &[DiscardedAttempt] {
        &self.discarded
    }

    /// The phase's discards, for the report.
    pub fn into_discarded(self) -> Vec<DiscardedAttempt> {
        self.discarded
    }
}

/// Polls the controller status every `link_poll_s` until its link is `linked`, for at most
/// `max_outage_s`. A status call that fails with [`TrainerError::Transient`] (the controller
/// is restarting) counts as "not back yet". Cancellation ends the wait at once.
fn wait_for_link(
    controller: &mut dyn BalanceController,
    config: &RecoveryConfig,
    cancel: &CancelToken,
) -> Result<(), TrainerError> {
    let started = Instant::now();
    loop {
        if cancel.is_cancelled() {
            return Err(TrainerError::Cancelled(
                "balance-on-feet run cancelled while waiting for the FEAGI link".to_string(),
            ));
        }
        let last_seen = match controller.link_status() {
            Ok(link) => match link.state {
                LinkState::Linked => return Ok(()),
                LinkState::GivenUp => {
                    return Err(TrainerError::Runtime(format!(
                        "balance phase failed: the controller's FEAGI link is given_up (epoch {}) \
                         and will not come back",
                        link.epoch
                    )));
                }
                LinkState::Reconnecting => {
                    format!("link reconnecting (epoch {})", link.epoch)
                }
            },
            Err(TrainerError::Transient(message)) => {
                format!("controller unreachable: {message}")
            }
            Err(other) => return Err(other),
        };
        let waited_s = started.elapsed().as_secs_f64();
        if waited_s >= config.max_outage_s {
            return Err(TrainerError::Runtime(format!(
                "balance phase failed: max_outage_s ({}) exceeded: waited {waited_s:.3} s for \
                 the FEAGI link, last seen: {last_seen}",
                config.max_outage_s
            )));
        }
        let nap_s = config.link_poll_s.min(config.max_outage_s - waited_s);
        if cancel.wait_cancelled(Duration::from_secs_f64(nap_s)) {
            return Err(TrainerError::Cancelled(
                "balance-on-feet run cancelled while waiting for the FEAGI link".to_string(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::balance_feet::{
        BalanceController, MeasureRequest, Measurement, ModelInfo, ResetReply, SpringGains,
    };
    use crate::run_data::RecordingRunDataSink;
    use std::cell::{Cell, RefCell};
    use std::collections::VecDeque;

    fn linked(epoch: u64) -> LinkReading {
        LinkReading {
            state: LinkState::Linked,
            epoch,
        }
    }

    fn reading(state: LinkState, epoch: u64) -> LinkReading {
        LinkReading { state, epoch }
    }

    fn config(max_outage_s: f64, max_discards: u32, poll_s: f64) -> RecoveryConfig {
        RecoveryConfig::new(max_outage_s, max_discards, poll_s, poll_s, poll_s * 4.0)
            .expect("valid config")
    }

    fn slot() -> AttemptSlot {
        AttemptSlot {
            attempt_index: 3,
            seed: 42,
            probe: false,
        }
    }

    /// Burst clock for tests: FEAGI is outside the subject. Answers come from scripts; the last
    /// answer repeats.
    struct ScriptedClock {
        counts: RefCell<VecDeque<Result<u64, TrainerError>>>,
        last_count: Cell<u64>,
        paused: RefCell<VecDeque<Result<bool, TrainerError>>>,
        last_paused: Cell<bool>,
        count_reads: Cell<u32>,
        paused_reads: Cell<u32>,
    }

    impl ScriptedClock {
        fn new() -> Self {
            Self {
                counts: RefCell::new(VecDeque::new()),
                last_count: Cell::new(0),
                paused: RefCell::new(VecDeque::new()),
                last_paused: Cell::new(false),
                count_reads: Cell::new(0),
                paused_reads: Cell::new(0),
            }
        }

        fn paused_script(answers: Vec<Result<bool, TrainerError>>) -> Self {
            let clock = Self::new();
            *clock.paused.borrow_mut() = answers.into();
            clock
        }
    }

    impl BurstClock for ScriptedClock {
        fn burst_count(&self) -> Result<u64, TrainerError> {
            self.count_reads.set(self.count_reads.get() + 1);
            match self.counts.borrow_mut().pop_front() {
                Some(Ok(count)) => {
                    self.last_count.set(count);
                    Ok(count)
                }
                Some(Err(error)) => Err(error),
                None => Ok(self.last_count.get()),
            }
        }

        fn engine_paused(&self) -> Result<bool, TrainerError> {
            self.paused_reads.set(self.paused_reads.get() + 1);
            match self.paused.borrow_mut().pop_front() {
                Some(Ok(paused)) => {
                    self.last_paused.set(paused);
                    Ok(paused)
                }
                Some(Err(error)) => Err(error),
                None => Ok(self.last_paused.get()),
            }
        }
    }

    fn watch(clock: &ScriptedClock, stall_s: f64) -> AttemptWatch<'_> {
        AttemptWatch::new(clock, Duration::from_secs_f64(stall_s))
    }

    /// Controller whose only live call is `link_status`, answered from a script. The last
    /// answer repeats once the script runs out.
    struct StatusOnly {
        answers: VecDeque<Result<LinkReading, TrainerError>>,
        last: Result<LinkReading, TrainerError>,
        polls: u32,
        /// Cancelled during the first status read.
        cancel_on_poll: Option<CancelToken>,
    }

    impl StatusOnly {
        fn new(answers: Vec<Result<LinkReading, TrainerError>>) -> Self {
            Self {
                last: Ok(linked(1)),
                answers: answers.into(),
                polls: 0,
                cancel_on_poll: None,
            }
        }
    }

    fn unsupported<T>() -> Result<T, TrainerError> {
        Err(TrainerError::Unsupported("status-only stub".to_string()))
    }

    fn copy_result(value: &Result<LinkReading, TrainerError>) -> Result<LinkReading, TrainerError> {
        match value {
            Ok(link) => Ok(*link),
            Err(TrainerError::Transient(m)) => Err(TrainerError::Transient(m.clone())),
            Err(other) => Err(TrainerError::Runtime(other.to_string())),
        }
    }

    impl BalanceController for StatusOnly {
        fn model_info(&mut self) -> Result<ModelInfo, TrainerError> {
            unsupported()
        }
        fn reset_episode(&mut self, _keyframe: Option<&str>) -> Result<ResetReply, TrainerError> {
            unsupported()
        }
        fn set_body_spring(
            &mut self,
            _body: &str,
            _gains: &SpringGains,
            _level: f64,
            _deadband_rad: f64,
        ) -> Result<(), TrainerError> {
            unsupported()
        }
        fn update_body_spring(
            &mut self,
            _body: &str,
            _level: f64,
            _deadband_rad: Option<f64>,
            _gravity_comp: Option<f64>,
            _ref_up_world: Option<[f64; 3]>,
        ) -> Result<(), TrainerError> {
            unsupported()
        }
        fn clear_body_spring(&mut self, _body: &str) -> Result<(), TrainerError> {
            unsupported()
        }
        fn apply_body_pulse(
            &mut self,
            _body: &str,
            _torque: [f64; 3],
            _force: [f64; 3],
            _duration_s: f64,
        ) -> Result<(), TrainerError> {
            unsupported()
        }
        fn measure(&mut self, _request: &MeasureRequest) -> Result<Measurement, TrainerError> {
            unsupported()
        }
        fn link_status(&mut self) -> Result<LinkReading, TrainerError> {
            self.polls += 1;
            if let Some(cancel) = &self.cancel_on_poll {
                cancel.cancel();
            }
            if let Some(next) = self.answers.pop_front() {
                self.last = copy_result(&next);
                return next;
            }
            copy_result(&self.last)
        }
    }

    #[test]
    fn config_requires_finite_positive_values() {
        assert!(RecoveryConfig::new(5.0, 3, 0.1, 0.1, 1.0).is_ok());
        for (outage, discards, link_poll, boundary_poll, stall, field) in [
            (0.0, 3, 0.1, 0.1, 1.0, "max_outage_s"),
            (f64::NAN, 3, 0.1, 0.1, 1.0, "max_outage_s"),
            (f64::INFINITY, 3, 0.1, 0.1, 1.0, "max_outage_s"),
            (-1.0, 3, 0.1, 0.1, 1.0, "max_outage_s"),
            (5.0, 3, 0.0, 0.1, 1.0, "link_poll_s"),
            (5.0, 3, f64::NAN, 0.1, 1.0, "link_poll_s"),
            (5.0, 3, 0.1, 0.0, 1.0, "boundary_poll_s"),
            (5.0, 3, 0.1, 0.1, 0.0, "burst_stall_s"),
            (5.0, 3, 0.1, 0.1, f64::NAN, "burst_stall_s"),
            (5.0, 0, 0.1, 0.1, 1.0, "max_discarded_attempts"),
        ] {
            let error =
                RecoveryConfig::new(outage, discards, link_poll, boundary_poll, stall).unwrap_err();
            assert!(
                matches!(&error, TrainerError::Config(m) if m.contains(field)),
                "{field}: {error}"
            );
        }
        // A value that cannot become a Duration is refused, not left to panic later.
        assert!(RecoveryConfig::new(f64::MAX, 3, 0.1, 0.1, 1.0).is_err());
    }

    #[test]
    fn burst_stall_must_sit_well_above_every_poll_interval() {
        // Exactly the minimum multiple is accepted.
        let least = BURST_STALL_MIN_POLL_MULTIPLE * 0.1;
        assert!(RecoveryConfig::new(5.0, 3, 0.1, 0.1, least).is_ok());
        // Too close to the link poll, or to the boundary poll, is refused and names both.
        for (link_poll, boundary_poll, which) in
            [(0.1, 0.01, "link_poll_s"), (0.01, 0.1, "boundary_poll_s")]
        {
            let error = RecoveryConfig::new(5.0, 3, link_poll, boundary_poll, 0.2).unwrap_err();
            assert!(
                matches!(&error, TrainerError::Config(m)
                    if m.contains("burst_stall_s (0.2)") && m.contains(which)),
                "{which}: {error}"
            );
        }
        // Against the criterion's burst period the same multiple applies.
        let config = RecoveryConfig::new(5.0, 3, 0.01, 0.01, 0.2).expect("valid");
        assert!(config.validate_for_burst_period(0.05).is_ok());
        let error = config.validate_for_burst_period(0.1).unwrap_err();
        assert!(error.to_string().contains("burst_period_s"), "{error}");
    }

    #[test]
    fn link_state_wire_names_match_the_controller() {
        let link: LinkReading =
            serde_json::from_str(r#"{"state":"given_up","epoch":7}"#).expect("parse");
        assert_eq!(
            link,
            LinkReading {
                state: LinkState::GivenUp,
                epoch: 7
            }
        );
        let parsed: LinkReading =
            serde_json::from_str(r#"{"state":"reconnecting","epoch":1}"#).expect("parse");
        assert_eq!(parsed.state, LinkState::Reconnecting);
        assert!(serde_json::from_str::<LinkReading>(r#"{"state":"gone","epoch":1}"#).is_err());
    }

    #[test]
    fn watch_accepts_a_steady_linked_epoch() {
        let clock = ScriptedClock::new();
        let mut watch = watch(&clock, 10.0);
        watch.observe(linked(4)).expect("baseline");
        watch.observe(linked(4)).expect("same epoch");
    }

    #[test]
    fn watch_flags_an_epoch_change_and_a_non_linked_state() {
        let clock = ScriptedClock::new();
        let mut watch = watch(&clock, 10.0);
        watch.observe(linked(2)).expect("baseline");
        let error = watch.observe(linked(4)).unwrap_err();
        assert!(matches!(error, TrainerError::Transient(_)), "{error}");
        let verdict =
            RecoveryState::classify::<()>(Err(error), &watch, slot()).expect("classified");
        let AttemptVerdict::Discard(discard) = verdict else {
            panic!("expected a discard");
        };
        assert_eq!(discard.reason, DiscardReason::LinkEpochChanged);
        assert_eq!(
            (discard.link_epoch_before, discard.link_epoch_after),
            (Some(2), Some(4))
        );

        let mut watch = self::watch(&clock, 10.0);
        watch.observe(linked(2)).expect("baseline");
        let error = watch
            .observe(reading(LinkState::Reconnecting, 3))
            .unwrap_err();
        let AttemptVerdict::Discard(discard) =
            RecoveryState::classify::<()>(Err(error), &watch, slot()).expect("classified")
        else {
            panic!("expected a discard");
        };
        assert_eq!(discard.reason, DiscardReason::LinkNotLinked);
        assert_eq!(discard.link_state_after, Some(LinkState::Reconnecting));
    }

    #[test]
    fn a_first_reading_that_is_not_linked_discards_the_attempt() {
        let clock = ScriptedClock::new();
        let mut watch = watch(&clock, 10.0);
        let error = watch.observe(reading(LinkState::GivenUp, 9)).unwrap_err();
        let AttemptVerdict::Discard(discard) =
            RecoveryState::classify::<()>(Err(error), &watch, slot()).expect("classified")
        else {
            panic!("expected a discard");
        };
        assert_eq!(discard.reason, DiscardReason::LinkNotLinked);
        assert_eq!(discard.link_epoch_before, Some(9));
    }

    #[test]
    fn a_stuck_burst_counter_discards_after_burst_stall_and_any_change_resets_it() {
        let clock = ScriptedClock::new();
        *clock.counts.borrow_mut() = vec![Ok(5), Ok(5), Ok(6)].into();
        let mut watch = watch(&clock, 0.03);
        watch
            .observe_bursts()
            .expect("first count starts the clock");
        watch.observe_bursts().expect("unchanged but not for long");
        // Count 6 is progress: the stall clock starts over.
        std::thread::sleep(Duration::from_millis(40));
        watch.observe_bursts().expect("changed after the wait");
        watch.observe_bursts().expect("unchanged again, just now");
        std::thread::sleep(Duration::from_millis(40));
        let error = watch.observe_bursts().unwrap_err();
        assert!(matches!(error, TrainerError::Transient(_)), "{error}");
        let AttemptVerdict::Discard(discard) =
            RecoveryState::classify::<()>(Err(error), &watch, slot()).expect("classified")
        else {
            panic!("expected a discard");
        };
        assert_eq!(discard.reason, DiscardReason::BurstStall);
        assert!(discard.detail.contains("stuck at 6"), "{}", discard.detail);
    }

    #[test]
    fn a_burst_clock_read_failure_is_classified_by_its_error() {
        let clock = ScriptedClock::new();
        *clock.counts.borrow_mut() = vec![
            Err(TrainerError::Transient("REST refused".to_string())),
            Err(TrainerError::Runtime("bad reply".to_string())),
        ]
        .into();
        let mut watch = watch(&clock, 10.0);
        let transient = watch.observe_bursts().unwrap_err();
        let AttemptVerdict::Discard(discard) =
            RecoveryState::classify::<()>(Err(transient), &watch, slot()).expect("classified")
        else {
            panic!("a transient read failure discards the attempt");
        };
        assert_eq!(discard.reason, DiscardReason::TransientError);
        let fatal = watch.observe_bursts().unwrap_err();
        assert!(matches!(
            RecoveryState::classify::<()>(Err(fatal), &watch, slot()),
            Err(TrainerError::Runtime(_))
        ));
    }

    #[test]
    fn classify_passes_values_and_non_transient_errors_through() {
        let clock = ScriptedClock::new();
        let watch = watch(&clock, 10.0);
        let done = RecoveryState::classify(Ok(7u32), &watch, slot()).expect("ok");
        assert!(matches!(done, AttemptVerdict::Completed(7)));
        let error = RecoveryState::classify::<()>(
            Err(TrainerError::Runtime("bad request".to_string())),
            &watch,
            slot(),
        )
        .unwrap_err();
        assert!(matches!(error, TrainerError::Runtime(_)));
        let cancelled = RecoveryState::classify::<()>(
            Err(TrainerError::Cancelled("stop".to_string())),
            &watch,
            slot(),
        )
        .unwrap_err();
        assert!(matches!(cancelled, TrainerError::Cancelled(_)));
    }

    #[test]
    fn a_transient_error_without_a_reply_is_recorded_without_epochs() {
        let clock = ScriptedClock::new();
        let watch = watch(&clock, 10.0);
        let AttemptVerdict::Discard(discard) = RecoveryState::classify::<()>(
            Err(TrainerError::Transient("connection refused".to_string())),
            &watch,
            slot(),
        )
        .expect("classified") else {
            panic!("expected a discard");
        };
        assert_eq!(discard.reason, DiscardReason::TransientError);
        assert_eq!(
            (discard.link_epoch_before, discard.link_epoch_after),
            (None, None)
        );
        assert_eq!((discard.attempt_index, discard.seed), (3, 42));
    }

    fn discard() -> DiscardedAttempt {
        DiscardedAttempt {
            attempt_index: 1,
            seed: 9,
            probe: false,
            reason: DiscardReason::LinkEpochChanged,
            detail: "epoch moved".to_string(),
            link_epoch_before: Some(1),
            link_epoch_after: Some(3),
            link_state_after: Some(LinkState::Linked),
        }
    }

    #[test]
    fn recover_returns_once_the_link_is_back_and_logs_an_event() {
        let mut state = RecoveryState::new(&config(5.0, 3, 0.005)).expect("state");
        let mut controller = StatusOnly::new(vec![
            Ok(reading(LinkState::Reconnecting, 4)),
            Err(TrainerError::Transient("refused".to_string())),
            Ok(linked(5)),
        ]);
        let mut sink = RecordingRunDataSink::default();
        state
            .recover(
                discard(),
                &mut controller,
                &CancelToken::new(),
                Some(&mut sink),
            )
            .expect("recovers");
        assert_eq!(controller.polls, 3);
        assert_eq!(state.discarded().len(), 1);
        assert_eq!(sink.event_kinds(), vec![ATTEMPT_DISCARDED_EVENT]);
        assert_eq!(sink.events[0].at.episode, Some(1));
        assert!(sink.events[0].label.contains("seed 9"));
    }

    #[test]
    fn an_outage_longer_than_max_outage_names_the_limit() {
        let mut state = RecoveryState::new(&config(0.05, 3, 0.01)).expect("state");
        let mut controller = StatusOnly::new(vec![Ok(reading(LinkState::Reconnecting, 4))]);
        let error = state
            .recover(discard(), &mut controller, &CancelToken::new(), None)
            .unwrap_err();
        let TrainerError::Runtime(message) = &error else {
            panic!("expected Runtime, got {error}");
        };
        assert!(
            message.contains("max_outage_s (0.05)") && message.contains("reconnecting (epoch 4)"),
            "{message}"
        );
        assert!(message.contains("waited 0.0"), "{message}");
    }

    #[test]
    fn an_unreachable_controller_counts_against_max_outage() {
        let mut state = RecoveryState::new(&config(0.05, 3, 0.01)).expect("state");
        let mut controller =
            StatusOnly::new(vec![Err(TrainerError::Transient("refused".to_string()))]);
        let error = state
            .recover(discard(), &mut controller, &CancelToken::new(), None)
            .unwrap_err();
        assert!(
            error.to_string().contains("max_outage_s") && error.to_string().contains("refused"),
            "{error}"
        );
    }

    #[test]
    fn a_status_error_that_is_not_transient_fails_at_once() {
        let mut state = RecoveryState::new(&config(5.0, 3, 0.005)).expect("state");
        let mut controller =
            StatusOnly::new(vec![Err(TrainerError::Runtime("HTTP 400".to_string()))]);
        let error = state
            .recover(discard(), &mut controller, &CancelToken::new(), None)
            .unwrap_err();
        assert!(matches!(error, TrainerError::Runtime(_)));
        assert_eq!(controller.polls, 1);
    }

    #[test]
    fn exceeding_max_discarded_attempts_fails_with_the_counts() {
        let mut state = RecoveryState::new(&config(5.0, 2, 0.005)).expect("state");
        let mut controller = StatusOnly::new(vec![Ok(linked(1))]);
        for _ in 0..2 {
            state
                .recover(discard(), &mut controller, &CancelToken::new(), None)
                .expect("within budget");
        }
        let polls_before = controller.polls;
        let error = state
            .recover(discard(), &mut controller, &CancelToken::new(), None)
            .unwrap_err();
        let message = error.to_string();
        assert!(
            message.contains("max_discarded_attempts (2)") && message.contains("3 discarded"),
            "{message}"
        );
        assert_eq!(controller.polls, polls_before, "fails before waiting");
        assert_eq!(
            state.discarded().len(),
            3,
            "the over-budget discard is kept"
        );
    }

    #[test]
    fn a_given_up_link_fails_immediately_without_waiting() {
        let mut state = RecoveryState::new(&config(5.0, 3, 0.005)).expect("state");
        let mut controller = StatusOnly::new(vec![Ok(linked(1))]);
        let mut given_up = discard();
        given_up.link_state_after = Some(LinkState::GivenUp);
        let error = state
            .recover(given_up, &mut controller, &CancelToken::new(), None)
            .unwrap_err();
        assert!(error.to_string().contains("given_up"), "{error}");
        assert_eq!(controller.polls, 0);

        // A status read that turns out given_up during the wait fails too.
        let mut controller = StatusOnly::new(vec![Ok(reading(LinkState::GivenUp, 8))]);
        let error = state
            .recover(discard(), &mut controller, &CancelToken::new(), None)
            .unwrap_err();
        assert!(error.to_string().contains("given_up (epoch 8)"), "{error}");
    }

    #[test]
    fn cancelling_ends_the_wait() {
        let mut state = RecoveryState::new(&config(30.0, 3, 10.0)).expect("state");
        let mut controller = StatusOnly::new(vec![Ok(reading(LinkState::Reconnecting, 4))]);
        let cancel = CancelToken::new();
        // The host cancels while the first status read is in flight.
        controller.cancel_on_poll = Some(cancel.clone());
        let started = Instant::now();
        let error = state
            .recover(discard(), &mut controller, &cancel, None)
            .unwrap_err();
        assert!(matches!(error, TrainerError::Cancelled(_)), "{error}");
        assert_eq!(controller.polls, 1);
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "cancel must not wait out the 10 s poll"
        );
    }

    #[test]
    fn the_boundary_returns_at_once_when_the_engine_runs() {
        let state = RecoveryState::new(&config(5.0, 3, 0.005)).expect("state");
        let clock = ScriptedClock::new();
        state
            .boundary(&clock, &CancelToken::new())
            .expect("engine running");
        assert_eq!(clock.paused_reads.get(), 1);
    }

    #[test]
    fn the_boundary_parks_while_the_engine_is_paused_and_resumes() {
        let state = RecoveryState::new(&config(5.0, 3, 0.005)).expect("state");
        let clock = ScriptedClock::paused_script(vec![
            Ok(true),
            Ok(true),
            Err(TrainerError::Transient("REST refused".to_string())),
            Ok(false),
        ]);
        state
            .boundary(&clock, &CancelToken::new())
            .expect("resumes");
        assert_eq!(clock.paused_reads.get(), 4);
    }

    #[test]
    fn the_boundary_fails_naming_the_pause_when_it_outlasts_max_outage() {
        let state = RecoveryState::new(&config(0.05, 3, 0.005)).expect("state");
        let clock = ScriptedClock::paused_script(vec![Ok(true)]);
        *clock.counts.borrow_mut() = vec![Ok(77)].into();
        let error = state.boundary(&clock, &CancelToken::new()).unwrap_err();
        let TrainerError::Runtime(message) = &error else {
            panic!("expected Runtime, got {error}");
        };
        assert!(
            message.contains("burst engine paused")
                && message.contains("max_outage_s (0.05)")
                && message.contains("burst count 77"),
            "{message}"
        );
    }

    #[test]
    fn a_non_transient_clock_failure_at_the_boundary_fails_at_once() {
        let state = RecoveryState::new(&config(5.0, 3, 0.005)).expect("state");
        let clock =
            ScriptedClock::paused_script(vec![Err(TrainerError::Runtime("bad reply".to_string()))]);
        let error = state.boundary(&clock, &CancelToken::new()).unwrap_err();
        assert!(matches!(error, TrainerError::Runtime(_)));
        assert_eq!(clock.paused_reads.get(), 1);
    }

    #[test]
    fn the_boundary_honors_cancel_and_the_host_pause_without_charging_it() {
        let state = RecoveryState::new(&config(0.05, 3, 0.005)).expect("state");
        // Cancelled before the first look: no engine read at all.
        let cancel = CancelToken::new();
        cancel.cancel();
        let clock = ScriptedClock::paused_script(vec![Ok(true)]);
        let error = state.boundary(&clock, &cancel).unwrap_err();
        assert!(matches!(error, TrainerError::Cancelled(_)), "{error}");
        assert_eq!(clock.paused_reads.get(), 0);

        // The host pause parks first; the time it holds is not charged to max_outage_s (0.05 s),
        // and the engine resumes right after.
        let cancel = CancelToken::new();
        cancel.pause();
        let resumer = cancel.clone();
        let handle = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(120));
            resumer.resume();
        });
        let clock = ScriptedClock::paused_script(vec![Ok(false)]);
        state
            .boundary(&clock, &cancel)
            .expect("parked by the host, then ran");
        handle.join().expect("resumer");
    }
}
