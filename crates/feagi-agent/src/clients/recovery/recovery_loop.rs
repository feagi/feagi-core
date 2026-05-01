//! Composer that wires watcher + policy + fetcher + rebuildable session.
//!
//! All three SDKs (Rust, Python via PyO3, future Java) drive recovery
//! through this single entry point so behavior cannot drift between them.
//! The function is intentionally synchronous-friendly (the only async work
//! is the HTTP fetch, which already has a blocking variant), so the same
//! tick logic is usable from a Tokio loop or from a plain thread.

#[cfg(feature = "agent-client-asynchelper-tokio")]
use crate::clients::recovery::health_fetch::HealthFetchConfig;
use crate::clients::recovery::health_watcher::{HealthEvent, HealthWatcher};
use crate::clients::recovery::reconnect_policy::{
    ReconnectDecision, ReconnectPolicy, RecoveryTrigger,
};
use crate::clients::recovery::session::RebuildableSession;
use crate::FeagiAgentError;
use serde::{Deserialize, Serialize};

/// Outcome of a single [`run_recovery_tick_blocking`] / async tick.
///
/// Bindings can serialize this directly for telemetry without re-deriving
/// any state from internal fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryTickReport {
    /// Health events observed this tick (may be empty).
    pub health_events: Vec<HealthEvent>,
    /// Decision returned by the policy this tick.
    pub decision: ReconnectDecision,
    /// True if the rebuild closure was invoked and reported success.
    pub rebuild_succeeded: bool,
    /// Populated when the rebuild attempt failed; empty otherwise.
    pub rebuild_error: Option<String>,
}

/// Run one recovery tick using the blocking HTTP fetcher.
///
/// Behavior:
/// 1. Fetch a health snapshot. On HTTP failure, watcher is told the
///    endpoint is unreachable and reachability transitions are emitted.
/// 2. Watcher emits health transition events.
/// 3. Policy decides whether to attempt a reconnect now.
/// 4. On `AttemptNow`, the supplied session's `rebuild` is invoked and
///    success/failure is recorded back into the policy.
///
/// `transport_send_failed` is a one-shot trigger the caller passes when
/// its IO loop saw a non-transient transport send failure since the last
/// tick. The flag is cleared by being consumed inside the policy.
#[cfg(feature = "agent-client-asynchelper-tokio")]
pub fn run_recovery_tick_blocking<S: RebuildableSession>(
    session: &mut S,
    watcher: &mut HealthWatcher,
    policy: &mut ReconnectPolicy,
    fetch_config: &HealthFetchConfig,
    transport_send_failed: bool,
    now_ms: u64,
) -> RecoveryTickReport {
    let snapshot_result =
        crate::clients::recovery::health_fetch::fetch_health_snapshot_blocking(fetch_config);
    run_recovery_tick_with_snapshot(
        session,
        watcher,
        policy,
        snapshot_result,
        transport_send_failed,
        now_ms,
    )
}

/// Async variant of [`run_recovery_tick_blocking`].
#[cfg(feature = "agent-client-asynchelper-tokio")]
pub async fn run_recovery_tick<S: RebuildableSession>(
    session: &mut S,
    watcher: &mut HealthWatcher,
    policy: &mut ReconnectPolicy,
    fetch_config: &HealthFetchConfig,
    transport_send_failed: bool,
    now_ms: u64,
) -> RecoveryTickReport {
    let snapshot_result =
        crate::clients::recovery::health_fetch::fetch_health_snapshot(fetch_config).await;
    run_recovery_tick_with_snapshot(
        session,
        watcher,
        policy,
        snapshot_result,
        transport_send_failed,
        now_ms,
    )
}

/// Pure-logic core that takes a pre-fetched snapshot result.
///
/// This entry point is exposed so callers (and unit tests) can drive the
/// composer with synthetic snapshots without needing a live FEAGI HTTP
/// endpoint. Both `run_recovery_tick` and `run_recovery_tick_blocking`
/// delegate here.
pub fn run_recovery_tick_with_snapshot<S: RebuildableSession>(
    session: &mut S,
    watcher: &mut HealthWatcher,
    policy: &mut ReconnectPolicy,
    snapshot_result: Result<
        crate::clients::recovery::health_watcher::HealthSnapshot,
        FeagiAgentError,
    >,
    transport_send_failed: bool,
    now_ms: u64,
) -> RecoveryTickReport {
    let health_events = match snapshot_result {
        Ok(snapshot) => watcher.observe(snapshot),
        Err(_fetch_error) => watcher.observe_unreachable(),
    };

    let mut triggers: Vec<RecoveryTrigger> = health_events
        .iter()
        .cloned()
        .map(RecoveryTrigger::Health)
        .collect();
    if transport_send_failed {
        triggers.push(RecoveryTrigger::TransportSendFailed);
    }

    let decision = policy.decide(&triggers, now_ms);

    let mut rebuild_succeeded = false;
    let mut rebuild_error: Option<String> = None;

    if let ReconnectDecision::AttemptNow { reason } = &decision {
        match session.rebuild(reason) {
            Ok(()) => {
                policy.record_attempt_succeeded();
                rebuild_succeeded = true;
            }
            Err(err) => {
                policy.record_attempt_failed();
                rebuild_error = Some(err.to_string());
            }
        }
    }

    RecoveryTickReport {
        health_events,
        decision,
        rebuild_succeeded,
        rebuild_error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::recovery::health_watcher::HealthSnapshot;
    use crate::clients::recovery::reconnect_policy::ReconnectPolicyConfig;

    struct FakeSession {
        rebuilds: Vec<String>,
        next_results: Vec<Result<(), FeagiAgentError>>,
    }

    impl FakeSession {
        fn always_ok() -> Self {
            Self {
                rebuilds: Vec::new(),
                next_results: vec![Ok(())],
            }
        }

        fn with_results(results: Vec<Result<(), FeagiAgentError>>) -> Self {
            Self {
                rebuilds: Vec::new(),
                next_results: results,
            }
        }
    }

    impl RebuildableSession for FakeSession {
        fn rebuild(&mut self, reason: &str) -> Result<(), FeagiAgentError> {
            self.rebuilds.push(reason.to_string());
            if self.next_results.is_empty() {
                return Ok(());
            }
            self.next_results.remove(0)
        }
    }

    fn cfg() -> ReconnectPolicyConfig {
        ReconnectPolicyConfig {
            cooldown_ms: 1_000,
            max_consecutive_failures: 3,
            trigger_on_session_changed: true,
            trigger_on_genome_changed: true,
            trigger_on_genome_load_completed: true,
            trigger_on_back_online: true,
            trigger_on_brain_ready: false,
            trigger_on_transport_send_failed: true,
        }
    }

    fn snapshot(
        feagi_session: Option<i64>,
        genome_num: Option<i32>,
        genome_loading: bool,
    ) -> HealthSnapshot {
        HealthSnapshot {
            feagi_session,
            genome_num,
            genome_timestamp: Some(1000),
            genome_loading,
            genome_availability: true,
            brain_readiness: true,
        }
    }

    #[test]
    fn first_tick_baseline_does_not_rebuild() {
        let mut session = FakeSession::always_ok();
        let mut watcher = HealthWatcher::new();
        let mut policy = ReconnectPolicy::new(cfg());
        let report = run_recovery_tick_with_snapshot(
            &mut session,
            &mut watcher,
            &mut policy,
            Ok(snapshot(Some(1), Some(1), false)),
            false,
            0,
        );
        assert!(report.health_events.is_empty());
        assert_eq!(report.decision, ReconnectDecision::Skip);
        assert!(!report.rebuild_succeeded);
        assert!(session.rebuilds.is_empty());
    }

    #[test]
    fn genome_load_completed_triggers_rebuild() {
        let mut session = FakeSession::always_ok();
        let mut watcher = HealthWatcher::new();
        let mut policy = ReconnectPolicy::new(cfg());
        let _ = run_recovery_tick_with_snapshot(
            &mut session,
            &mut watcher,
            &mut policy,
            Ok(snapshot(Some(1), Some(1), true)),
            false,
            0,
        );
        let report = run_recovery_tick_with_snapshot(
            &mut session,
            &mut watcher,
            &mut policy,
            Ok(snapshot(Some(1), Some(2), false)),
            false,
            10,
        );
        assert!(report.rebuild_succeeded);
        assert_eq!(session.rebuilds.len(), 1);
    }

    #[test]
    fn transport_send_failure_triggers_rebuild_when_enabled() {
        let mut session = FakeSession::always_ok();
        let mut watcher = HealthWatcher::new();
        let mut policy = ReconnectPolicy::new(cfg());
        let _ = run_recovery_tick_with_snapshot(
            &mut session,
            &mut watcher,
            &mut policy,
            Ok(snapshot(Some(1), Some(1), false)),
            false,
            0,
        );
        let report = run_recovery_tick_with_snapshot(
            &mut session,
            &mut watcher,
            &mut policy,
            Ok(snapshot(Some(1), Some(1), false)),
            true,
            10,
        );
        assert!(report.rebuild_succeeded);
        assert_eq!(session.rebuilds.len(), 1);
    }

    #[test]
    fn unreachable_then_back_online_triggers_rebuild() {
        let mut session = FakeSession::always_ok();
        let mut watcher = HealthWatcher::new();
        let mut policy = ReconnectPolicy::new(cfg());
        let _ = run_recovery_tick_with_snapshot(
            &mut session,
            &mut watcher,
            &mut policy,
            Ok(snapshot(Some(1), Some(1), false)),
            false,
            0,
        );
        let _ = run_recovery_tick_with_snapshot(
            &mut session,
            &mut watcher,
            &mut policy,
            Err(FeagiAgentError::ConnectionFailed("down".to_string())),
            false,
            10,
        );
        let report = run_recovery_tick_with_snapshot(
            &mut session,
            &mut watcher,
            &mut policy,
            Ok(snapshot(Some(1), Some(1), false)),
            false,
            20,
        );
        assert!(report.health_events.contains(&HealthEvent::FeagiBackOnline));
        assert!(report.rebuild_succeeded);
    }

    #[test]
    fn rebuild_failure_increments_counter() {
        let mut session = FakeSession::with_results(vec![
            Err(FeagiAgentError::ConnectionFailed("a".to_string())),
            Err(FeagiAgentError::ConnectionFailed("b".to_string())),
        ]);
        let mut watcher = HealthWatcher::new();
        let mut policy = ReconnectPolicy::new(cfg());
        let _ = run_recovery_tick_with_snapshot(
            &mut session,
            &mut watcher,
            &mut policy,
            Ok(snapshot(Some(1), Some(1), false)),
            false,
            0,
        );
        let _ = run_recovery_tick_with_snapshot(
            &mut session,
            &mut watcher,
            &mut policy,
            Ok(snapshot(Some(2), Some(1), false)),
            false,
            10,
        );
        // Second attempt is past cooldown.
        let _ = run_recovery_tick_with_snapshot(
            &mut session,
            &mut watcher,
            &mut policy,
            Ok(snapshot(Some(3), Some(1), false)),
            false,
            2_000,
        );
        assert_eq!(policy.consecutive_failures(), 2);
    }
}
