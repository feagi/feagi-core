//! Control API — the transport-agnostic seam a host (desktop app, CI) drives a run through.
//!
//! Per ADR-011 the Control API is a **library surface**, not a network service: the open crate
//! exposes a [`RunControl`] trait, a [`RunEventSink`] the engine emits through, and a
//! [`CancelToken`] for cooperative stop. A host (e.g. a `feagi-desktop` Tauri plugin) implements
//! the transport by supplying a sink that re-emits each [`RunEvent`] as a Tauri event, and by
//! holding the cancel token. The crate opens no socket of its own, preserving the open/closed and
//! embedded/RTOS invariants (ADR-006).
//!
//! [`ClosureRunControl`] is the reference implementation: it wraps a host-supplied rollout closure
//! (which performs the actual `submit -> step -> collect -> score` work and emits progress/metric
//! events) and layers the run lifecycle on top — status transitions plus the `Running` /
//! `ScorecardReady` / `Completed` / `Failed` lifecycle events. Keeping execution behind a closure
//! makes the controller independent of which `FeagiRuntime` is used (stub, remote, embedded) and
//! fully testable without a live backend.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::contracts::{RunEvent, RunEventKind, RunStatus, RunSummary};
use crate::error::TrainerError;

/// A cloneable, thread-safe cooperative-cancellation handle for a run.
///
/// The engine checks [`is_cancelled`](Self::is_cancelled) at safe points (e.g. between samples)
/// and stops with [`TrainerError::Cancelled`]; a host calls [`cancel`](Self::cancel) from any
/// thread to request that stop. Cancellation is cooperative — it never interrupts mid-step.
#[derive(Debug, Clone, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    /// Creates a fresh, un-cancelled token.
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    /// Requests cancellation. Idempotent and callable from any thread.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    /// Returns whether cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

/// Receives the [`RunEvent`]s a run emits, in emission order.
///
/// Implementations must not block the engine for long: a desktop host typically forwards each
/// event onto a channel / Tauri event and returns immediately.
pub trait RunEventSink {
    /// Handles one event. Called synchronously from the engine in emission order.
    fn emit(&mut self, event: RunEvent);
}

/// A sink that discards every event — for runs that do not observe progress.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopEventSink;

impl RunEventSink for NoopEventSink {
    fn emit(&mut self, _event: RunEvent) {}
}

/// A sink that records events in order — for tests and headless capture.
#[derive(Debug, Clone, Default)]
pub struct CollectingEventSink {
    /// Events received so far, in emission order.
    pub events: Vec<RunEvent>,
}

impl RunEventSink for CollectingEventSink {
    fn emit(&mut self, event: RunEvent) {
        self.events.push(event);
    }
}

/// The host-facing control surface for a single run.
///
/// A host validates a [`RunConfig`](crate::run_config::RunConfig) (via
/// [`validate_supported`](crate::run_config::RunConfig::validate_supported)) before constructing a
/// controller, then drives the run through this trait. Lifecycle events are streamed to the sink
/// passed to [`execute`](Self::execute); the terminal [`RunSummary`] is returned.
pub trait RunControl {
    /// The cooperative-cancellation handle for this run (clone and hand to a host stop button).
    fn cancel_token(&self) -> CancelToken;

    /// The latest known lifecycle status.
    fn status(&self) -> RunStatus;

    /// Runs to completion, streaming [`RunEvent`]s through `events`, and returns the terminal
    /// summary.
    ///
    /// # Errors
    /// Propagates the rollout error (including [`TrainerError::Cancelled`] on cooperative stop); a
    /// `Failed` event is emitted before the error is returned.
    fn execute(&mut self, events: &mut dyn RunEventSink) -> Result<RunSummary, TrainerError>;
}

/// Reference [`RunControl`] that wraps a host-supplied rollout closure and layers run lifecycle
/// events + status transitions on top.
///
/// The closure receives the event sink and the cancel token so it can stream `Progress` /
/// `MetricUpdate` events and observe cancellation while it executes (e.g. by calling
/// [`run_rollout_with_events`](crate::executor::run_rollout_with_events)). It returns the terminal
/// [`RunSummary`]; if that summary carries a `scorecard_id`, a `ScorecardReady` event is emitted
/// before `Completed`.
pub struct ClosureRunControl<F> {
    run_id: crate::contracts::RunId,
    status: RunStatus,
    cancel: CancelToken,
    rollout: F,
}

impl<F> ClosureRunControl<F>
where
    F: FnMut(&mut dyn RunEventSink, &CancelToken) -> Result<RunSummary, TrainerError>,
{
    /// Creates a controller for `run_id` that executes `rollout`. Status starts at
    /// [`RunStatus::Created`].
    pub fn new(run_id: crate::contracts::RunId, rollout: F) -> Self {
        Self {
            run_id,
            status: RunStatus::Created,
            cancel: CancelToken::new(),
            rollout,
        }
    }
}

impl<F> RunControl for ClosureRunControl<F>
where
    F: FnMut(&mut dyn RunEventSink, &CancelToken) -> Result<RunSummary, TrainerError>,
{
    fn cancel_token(&self) -> CancelToken {
        self.cancel.clone()
    }

    fn status(&self) -> RunStatus {
        self.status
    }

    fn execute(&mut self, events: &mut dyn RunEventSink) -> Result<RunSummary, TrainerError> {
        self.status = RunStatus::Running;
        events.emit(RunEvent::new(self.run_id.clone(), RunEventKind::Running));

        let cancel = self.cancel.clone();
        match (self.rollout)(events, &cancel) {
            Ok(summary) => {
                if let Some(scorecard_id) = &summary.scorecard_id {
                    events.emit(RunEvent::new(
                        self.run_id.clone(),
                        RunEventKind::ScorecardReady {
                            scorecard_id: scorecard_id.clone(),
                        },
                    ));
                }
                events.emit(RunEvent::new(self.run_id.clone(), RunEventKind::Completed));
                self.status = RunStatus::Completed;
                Ok(summary)
            }
            Err(error) => {
                events.emit(RunEvent::new(
                    self.run_id.clone(),
                    RunEventKind::Failed {
                        message: error.to_string(),
                    },
                ));
                self.status = RunStatus::Failed;
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::run_summary::SCHEMA_VERSION as RUN_SUMMARY_SCHEMA_VERSION;
    use crate::contracts::{RunId, ScorecardId};
    use std::collections::BTreeMap;

    fn summary(run_id: &RunId, scorecard: Option<&str>) -> RunSummary {
        RunSummary {
            schema_version: RUN_SUMMARY_SCHEMA_VERSION,
            run_id: run_id.clone(),
            status: RunStatus::Completed,
            total_samples: 4,
            evaluated_samples: 4,
            metrics: BTreeMap::new(),
            started_at: None,
            completed_at: None,
            scorecard_id: scorecard.map(|s| ScorecardId(s.to_string())),
            metadata: BTreeMap::new(),
        }
    }

    fn event_kinds(sink: &CollectingEventSink) -> Vec<&RunEventKind> {
        sink.events.iter().map(|e| &e.kind).collect()
    }

    #[test]
    fn cancel_token_round_trips() {
        let token = CancelToken::new();
        assert!(!token.is_cancelled());
        let clone = token.clone();
        clone.cancel();
        // Clones share the same flag.
        assert!(token.is_cancelled());
    }

    #[test]
    fn successful_run_emits_running_scorecard_then_completed() {
        let run_id = RunId("run-ok".to_string());
        let mut control = ClosureRunControl::new(run_id.clone(), |_events, _cancel| {
            Ok(summary(&run_id, Some("sc-1")))
        });
        let mut sink = CollectingEventSink::default();

        let result = control.execute(&mut sink).expect("run ok");
        assert_eq!(result.scorecard_id, Some(ScorecardId("sc-1".to_string())));
        assert_eq!(control.status(), RunStatus::Completed);
        assert!(matches!(
            event_kinds(&sink).as_slice(),
            [
                RunEventKind::Running,
                RunEventKind::ScorecardReady { .. },
                RunEventKind::Completed
            ]
        ));
    }

    #[test]
    fn run_without_scorecard_skips_scorecard_ready() {
        let run_id = RunId("run-no-card".to_string());
        let mut control = ClosureRunControl::new(run_id.clone(), |_events, _cancel| {
            Ok(summary(&run_id, None))
        });
        let mut sink = CollectingEventSink::default();

        control.execute(&mut sink).expect("run ok");
        assert!(matches!(
            event_kinds(&sink).as_slice(),
            [RunEventKind::Running, RunEventKind::Completed]
        ));
    }

    #[test]
    fn failed_run_emits_failed_and_sets_status() {
        let run_id = RunId("run-fail".to_string());
        let mut control = ClosureRunControl::new(run_id, |_events, _cancel| {
            Err(TrainerError::Runtime("boom".to_string()))
        });
        let mut sink = CollectingEventSink::default();

        let error = control.execute(&mut sink).unwrap_err();
        assert!(matches!(error, TrainerError::Runtime(_)));
        assert_eq!(control.status(), RunStatus::Failed);
        match event_kinds(&sink).as_slice() {
            [RunEventKind::Running, RunEventKind::Failed { message }] => {
                assert!(message.contains("boom"));
            }
            other => panic!("unexpected events: {other:?}"),
        }
    }

    #[test]
    fn host_cancel_is_visible_to_the_rollout_closure() {
        let run_id = RunId("run-cancel".to_string());
        let mut control = ClosureRunControl::new(run_id.clone(), |_events, cancel| {
            // The closure observes the cancellation a host requested before execute().
            if cancel.is_cancelled() {
                return Err(TrainerError::Cancelled("stopped".to_string()));
            }
            Ok(summary(&run_id, None))
        });
        // Host requests cancellation up front via the shared token.
        control.cancel_token().cancel();
        let mut sink = CollectingEventSink::default();

        let error = control.execute(&mut sink).unwrap_err();
        assert!(matches!(error, TrainerError::Cancelled(_)));
        assert_eq!(control.status(), RunStatus::Failed);
    }
}
