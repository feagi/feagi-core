//! FEAGI health snapshot watcher.
//!
//! Pure logic. No I/O, no time, no defaults. The watcher is the single
//! source of truth for what counts as a "FEAGI lifecycle change" (genome
//! reload, FEAGI restart, brain readiness flip, reachability flip) for
//! every SDK that wraps `feagi-agent`.
//!
//! @cursor:critical-path

use crate::command_and_control::health_check_message::HealthCheckResponse;
use serde::{Deserialize, Serialize};

/// Minimal subset of [`HealthCheckResponse`] used to detect agent-relevant
/// lifecycle changes.
///
/// Only fields that influence reconnect decisions are kept here, so SDK
/// bindings (PyO3, Java) and embedded targets can construct snapshots from
/// any transport without depending on the full Feagi health DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthSnapshot {
    /// FEAGI session id (changes on FEAGI restart).
    pub feagi_session: Option<i64>,
    /// Currently loaded genome ordinal.
    pub genome_num: Option<i32>,
    /// Last genome load timestamp (changes whenever a genome is loaded,
    /// even if the genome ordinal is the same).
    pub genome_timestamp: Option<i64>,
    /// True while a prioritized genome transition is in progress.
    pub genome_loading: bool,
    /// True when FEAGI considers a genome to be present.
    pub genome_availability: bool,
    /// True when burst engine + connectome are ready to process IO.
    pub brain_readiness: bool,
}

impl HealthSnapshot {
    /// Build a snapshot from the full FEAGI health response DTO.
    pub fn from_response(response: &HealthCheckResponse) -> Self {
        Self {
            feagi_session: response.feagi_session,
            genome_num: response.genome_num,
            genome_timestamp: response.genome_timestamp,
            genome_loading: response.genome_loading,
            genome_availability: response.genome_availability,
            brain_readiness: response.brain_readiness,
        }
    }
}

/// Observable lifecycle events emitted by [`HealthWatcher`].
///
/// All variants are transitions: an event is only ever emitted when the
/// observed value changes vs. the previous snapshot. Bindings can map these
/// 1:1 to language-native enums without re-deriving meaning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthEvent {
    /// FEAGI HTTP health endpoint is no longer responding successfully.
    /// Emitted only on the transition from "reachable" to "unreachable".
    FeagiUnreachable,
    /// FEAGI HTTP health is responding again after a previous unreachable.
    FeagiBackOnline,
    /// FEAGI session id changed between two reachable snapshots
    /// (i.e. FEAGI process restarted).
    SessionChanged { old: i64, new: i64 },
    /// Loaded genome ordinal or genome timestamp changed.
    /// Either field flipping triggers this, since FEAGI may load a genome
    /// with the same ordinal but a new timestamp.
    GenomeChanged {
        old_num: Option<i32>,
        new_num: Option<i32>,
        old_timestamp: Option<i64>,
        new_timestamp: Option<i64>,
    },
    /// Genome loading transitioned from `false` to `true`
    /// (FEAGI started a prioritized genome transition).
    GenomeLoadStarted,
    /// Genome loading transitioned from `true` to `false`
    /// (FEAGI completed or failed a prioritized genome transition).
    GenomeLoadCompleted,
    /// Brain readiness flipped from `false` to `true`.
    BrainReady,
    /// Brain readiness flipped from `true` to `false`.
    BrainLost,
}

/// Tracks the previous FEAGI snapshot and emits [`HealthEvent`]s on changes.
///
/// The first call to [`HealthWatcher::observe`] establishes a baseline and
/// emits no events; subsequent calls compare against the stored snapshot.
///
/// Reachability is tracked separately from snapshot fields:
/// [`HealthWatcher::observe`] implies the fetch succeeded, and
/// [`HealthWatcher::observe_unreachable`] is called by drivers when a fetch
/// fails. Snapshot-derived transitions are only emitted between two
/// successful observations.
#[derive(Debug, Clone)]
pub struct HealthWatcher {
    last_snapshot: Option<HealthSnapshot>,
    last_seen_reachable: bool,
    has_been_observed_at_least_once: bool,
}

impl HealthWatcher {
    /// Create a watcher with no prior observations.
    pub fn new() -> Self {
        Self {
            last_snapshot: None,
            last_seen_reachable: false,
            has_been_observed_at_least_once: false,
        }
    }

    /// True once at least one snapshot or unreachable observation has been
    /// recorded. Useful for callers that want to suppress recovery actions
    /// until they have a baseline.
    pub fn has_baseline(&self) -> bool {
        self.has_been_observed_at_least_once
    }

    /// Most recent successful snapshot, if any.
    pub fn last_snapshot(&self) -> Option<&HealthSnapshot> {
        self.last_snapshot.as_ref()
    }

    /// True if the most recent observation was a successful snapshot fetch.
    pub fn is_reachable(&self) -> bool {
        self.last_seen_reachable
    }

    /// Record a successful snapshot fetch and return any transitions.
    ///
    /// Emits [`HealthEvent::FeagiBackOnline`] if the previous observation
    /// was an unreachable. Snapshot-derived events are emitted only when a
    /// previous successful snapshot exists to compare against.
    pub fn observe(&mut self, snapshot: HealthSnapshot) -> Vec<HealthEvent> {
        let mut events = Vec::new();

        let was_reachable_before = self.last_seen_reachable;
        let had_prior_baseline = self.has_been_observed_at_least_once;

        // Reachability transition: only emit BackOnline on an explicit
        // unreachable->reachable flip, never on the very first observation.
        if had_prior_baseline && !was_reachable_before {
            events.push(HealthEvent::FeagiBackOnline);
        }

        if let Some(previous) = self.last_snapshot.as_ref() {
            // Session change (FEAGI restart) takes precedence in ordering
            // because every other field is meaningless across a restart.
            if let (Some(old), Some(new)) = (previous.feagi_session, snapshot.feagi_session) {
                if old != new {
                    events.push(HealthEvent::SessionChanged { old, new });
                }
            }

            // Genome change: either ordinal or timestamp differs.
            let genome_num_changed = previous.genome_num != snapshot.genome_num;
            let genome_timestamp_changed = previous.genome_timestamp != snapshot.genome_timestamp;
            if genome_num_changed || genome_timestamp_changed {
                events.push(HealthEvent::GenomeChanged {
                    old_num: previous.genome_num,
                    new_num: snapshot.genome_num,
                    old_timestamp: previous.genome_timestamp,
                    new_timestamp: snapshot.genome_timestamp,
                });
            }

            // Genome loading transitions.
            match (previous.genome_loading, snapshot.genome_loading) {
                (false, true) => events.push(HealthEvent::GenomeLoadStarted),
                (true, false) => events.push(HealthEvent::GenomeLoadCompleted),
                _ => {}
            }

            // Brain readiness transitions.
            match (previous.brain_readiness, snapshot.brain_readiness) {
                (false, true) => events.push(HealthEvent::BrainReady),
                (true, false) => events.push(HealthEvent::BrainLost),
                _ => {}
            }
        }

        self.last_snapshot = Some(snapshot);
        self.last_seen_reachable = true;
        self.has_been_observed_at_least_once = true;
        events
    }

    /// Record that an attempt to fetch a snapshot failed.
    ///
    /// Emits [`HealthEvent::FeagiUnreachable`] only on the transition from
    /// "previously reachable" to "now unreachable". Repeated unreachable
    /// observations emit no events, so callers can poll without emitting
    /// duplicate triggers.
    pub fn observe_unreachable(&mut self) -> Vec<HealthEvent> {
        let mut events = Vec::new();
        if self.has_been_observed_at_least_once && self.last_seen_reachable {
            events.push(HealthEvent::FeagiUnreachable);
        }
        self.last_seen_reachable = false;
        self.has_been_observed_at_least_once = true;
        events
    }
}

impl Default for HealthWatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(
        feagi_session: Option<i64>,
        genome_num: Option<i32>,
        genome_timestamp: Option<i64>,
        genome_loading: bool,
        brain_readiness: bool,
    ) -> HealthSnapshot {
        HealthSnapshot {
            feagi_session,
            genome_num,
            genome_timestamp,
            genome_loading,
            genome_availability: true,
            brain_readiness,
        }
    }

    #[test]
    fn first_successful_observation_emits_no_events() {
        let mut w = HealthWatcher::new();
        let events = w.observe(snapshot(Some(1), Some(1), Some(100), false, true));
        assert!(events.is_empty());
        assert!(w.has_baseline());
        assert!(w.is_reachable());
    }

    #[test]
    fn first_unreachable_observation_emits_no_events() {
        // No prior baseline -> no transition can be derived.
        let mut w = HealthWatcher::new();
        let events = w.observe_unreachable();
        assert!(events.is_empty());
        assert!(w.has_baseline());
        assert!(!w.is_reachable());
    }

    #[test]
    fn unreachable_after_reachable_emits_feagi_unreachable_once() {
        let mut w = HealthWatcher::new();
        let _ = w.observe(snapshot(Some(1), Some(1), Some(100), false, true));
        let first = w.observe_unreachable();
        let second = w.observe_unreachable();
        assert_eq!(first, vec![HealthEvent::FeagiUnreachable]);
        assert!(second.is_empty(), "duplicate unreachable must not re-emit");
    }

    #[test]
    fn reachable_after_unreachable_emits_back_online() {
        let mut w = HealthWatcher::new();
        let _ = w.observe(snapshot(Some(1), Some(1), Some(100), false, true));
        let _ = w.observe_unreachable();
        let events = w.observe(snapshot(Some(1), Some(1), Some(100), false, true));
        assert_eq!(events, vec![HealthEvent::FeagiBackOnline]);
    }

    #[test]
    fn session_change_is_detected() {
        let mut w = HealthWatcher::new();
        let _ = w.observe(snapshot(Some(1), Some(1), Some(100), false, true));
        let events = w.observe(snapshot(Some(2), Some(1), Some(100), false, true));
        assert!(events.contains(&HealthEvent::SessionChanged { old: 1, new: 2 }));
    }

    #[test]
    fn genome_num_change_emits_genome_changed() {
        let mut w = HealthWatcher::new();
        let _ = w.observe(snapshot(Some(1), Some(1), Some(100), false, true));
        let events = w.observe(snapshot(Some(1), Some(2), Some(100), false, true));
        assert!(events.iter().any(|e| matches!(
            e,
            HealthEvent::GenomeChanged {
                old_num: Some(1),
                new_num: Some(2),
                ..
            }
        )));
    }

    #[test]
    fn genome_timestamp_change_emits_genome_changed_even_when_num_stable() {
        let mut w = HealthWatcher::new();
        let _ = w.observe(snapshot(Some(1), Some(1), Some(100), false, true));
        let events = w.observe(snapshot(Some(1), Some(1), Some(200), false, true));
        assert!(events.iter().any(|e| matches!(
            e,
            HealthEvent::GenomeChanged {
                old_num: Some(1),
                new_num: Some(1),
                old_timestamp: Some(100),
                new_timestamp: Some(200),
            }
        )));
    }

    #[test]
    fn genome_loading_started_and_completed_round_trip() {
        let mut w = HealthWatcher::new();
        let _ = w.observe(snapshot(Some(1), Some(1), Some(100), false, true));
        let started = w.observe(snapshot(Some(1), Some(1), Some(100), true, true));
        let completed = w.observe(snapshot(Some(1), Some(2), Some(200), false, true));
        assert!(started.contains(&HealthEvent::GenomeLoadStarted));
        assert!(completed.contains(&HealthEvent::GenomeLoadCompleted));
    }

    #[test]
    fn brain_readiness_transitions_emit_in_both_directions() {
        let mut w = HealthWatcher::new();
        let _ = w.observe(snapshot(Some(1), Some(1), Some(100), false, false));
        let to_ready = w.observe(snapshot(Some(1), Some(1), Some(100), false, true));
        let to_lost = w.observe(snapshot(Some(1), Some(1), Some(100), false, false));
        assert_eq!(to_ready, vec![HealthEvent::BrainReady]);
        assert_eq!(to_lost, vec![HealthEvent::BrainLost]);
    }

    #[test]
    fn no_change_emits_no_events() {
        let mut w = HealthWatcher::new();
        let _ = w.observe(snapshot(Some(1), Some(1), Some(100), false, true));
        let events = w.observe(snapshot(Some(1), Some(1), Some(100), false, true));
        assert!(events.is_empty());
    }

    #[test]
    fn multiple_simultaneous_changes_emit_all_relevant_events() {
        let mut w = HealthWatcher::new();
        let _ = w.observe(snapshot(Some(1), Some(1), Some(100), true, false));
        let events = w.observe(snapshot(Some(2), Some(2), Some(200), false, true));
        assert!(events.contains(&HealthEvent::SessionChanged { old: 1, new: 2 }));
        assert!(events
            .iter()
            .any(|e| matches!(e, HealthEvent::GenomeChanged { .. })));
        assert!(events.contains(&HealthEvent::GenomeLoadCompleted));
        assert!(events.contains(&HealthEvent::BrainReady));
    }
}
