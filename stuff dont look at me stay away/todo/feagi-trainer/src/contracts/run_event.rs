//! `RunEvent` v1 — the normalized, versioned event stream a run emits while executing.
//!
//! `RunEvent` is the public seam between the engine and any observer (the closed-source desktop
//! app, a headless CI logger, a test). The engine emits a flat sequence of these through a sink
//! (see [`RunEventSink`](crate::control::RunEventSink)); the observer renders them. Per ADR-005 the
//! stream is **normalized**: it carries lifecycle, progress, and metric summaries — never raw
//! FEAGI motor/sensory frames. Per ADR-011 it is a public contract owned by the open crate, so the
//! desktop consumes it one-way.
//!
//! Determinism: the engine never stamps wall-clock time (it leaves `timestamp = None`); a host
//! that needs timestamps fills them at the transport boundary, keeping unit tests reproducible
//! (consistent with the executor's no-I/O contract).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::common::{RunId, ScorecardId};

/// Wire/format version of the `RunEvent` contract.
pub const SCHEMA_VERSION: u32 = 1;

/// Whether a [`RunEventKind::MetricUpdate`] reports interim or final metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricScope {
    /// Interim metrics computed over the samples seen so far (may change).
    Partial,
    /// Final metrics over the full scored subset for this run.
    Aggregate,
}

/// The payload of a [`RunEvent`]. JSON-tagged by `type` for ergonomic consumption from the
/// TypeScript/desktop side (e.g. `{ "type": "progress", "samples_done": 3, ... }`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RunEventKind {
    /// The run record exists but has not started.
    Created,
    /// Pre-run validation is in progress (binding resolution, compatibility).
    Validating,
    /// Execution has begun.
    Running,
    /// Sample/episode progress for the current repeat.
    Progress {
        /// Samples (or episodes) completed so far in this repeat.
        samples_done: u64,
        /// Total samples (or episodes) planned for this repeat.
        samples_total: u64,
        /// Zero-based index of the current repeat (0 for a single, non-repeated run).
        repeat_index: u32,
        /// Total number of repeats (1 for a single run).
        repeat_total: u32,
    },
    /// A metric snapshot — interim ([`MetricScope::Partial`]) or final
    /// ([`MetricScope::Aggregate`]). Values are deterministically ordered.
    MetricUpdate {
        /// Whether these are interim or final metrics.
        scope: MetricScope,
        /// Named metric values.
        metrics: BTreeMap<String, f64>,
    },
    /// A `Scorecard` has been produced and persisted by the host.
    ScorecardReady {
        /// Identity of the produced scorecard.
        scorecard_id: ScorecardId,
    },
    /// The run finished successfully.
    Completed,
    /// The run terminated with an error (includes cooperative cancellation).
    Failed {
        /// Human-readable failure description.
        message: String,
    },
}

/// One normalized event in a run's lifecycle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunEvent {
    /// Wire/format version; must equal [`SCHEMA_VERSION`].
    pub schema_version: u32,
    /// Run this event belongs to (lets a host multiplex several runs over one channel).
    pub run_id: RunId,
    /// Optional emission time, Unix epoch milliseconds. Left `None` by the engine; a host may
    /// stamp it at the transport boundary.
    pub timestamp: Option<i64>,
    /// The event payload.
    pub kind: RunEventKind,
}

impl RunEvent {
    /// Builds an event for `run_id` with the current [`SCHEMA_VERSION`] and no timestamp.
    pub fn new(run_id: RunId, kind: RunEventKind) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            run_id,
            timestamp: None,
            kind,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_id() -> RunId {
        RunId("run-evt-0001".to_string())
    }

    #[test]
    fn schema_version_is_pinned() {
        let event = RunEvent::new(run_id(), RunEventKind::Running);
        assert_eq!(event.schema_version, SCHEMA_VERSION);
        assert_eq!(event.timestamp, None);
    }

    #[test]
    fn progress_event_round_trips() {
        let event = RunEvent::new(
            run_id(),
            RunEventKind::Progress {
                samples_done: 3,
                samples_total: 10,
                repeat_index: 1,
                repeat_total: 5,
            },
        );
        let json = serde_json::to_string(&event).expect("serialize");
        // Tagged representation is consumable from the desktop/TS side.
        assert!(json.contains("\"type\":\"progress\""));
        let restored: RunEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(event, restored);
    }

    #[test]
    fn metric_update_and_failed_round_trip() {
        let mut metrics = BTreeMap::new();
        metrics.insert("accuracy".to_string(), 0.97);
        let update = RunEvent::new(
            run_id(),
            RunEventKind::MetricUpdate {
                scope: MetricScope::Aggregate,
                metrics,
            },
        );
        let failed = RunEvent::new(
            run_id(),
            RunEventKind::Failed {
                message: "cancelled".to_string(),
            },
        );
        for event in [update, failed] {
            let json = serde_json::to_string(&event).expect("serialize");
            let restored: RunEvent = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(event, restored);
        }
    }
}
