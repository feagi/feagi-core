//! Shared recovery primitives for FEAGI agents.
//!
//! This module contains the single source of truth for how an agent
//! detects FEAGI lifecycle changes (genome reload, FEAGI restart, network
//! drop) and decides when to reconnect. All language SDKs (Rust, Python,
//! future Java) drive recovery through these types so that behavior cannot
//! drift between bindings.
//!
//! Layout:
//! - [`health_watcher`]: pure logic, snapshot diff -> events.
//! - [`reconnect_policy`]: pure logic, events -> decision.
//! - [`session`]: binding-friendly trait that concrete sessions implement.
//! - [`recovery_loop`]: composer that ties watcher + policy + session.
//! - [`health_fetch`]: HTTP helpers (Tokio feature) for snapshot retrieval.
//!
//! # Cross-SDK contract
//!
//! Every SDK must expose, with byte-identical semantics, the following
//! surface (names may be language-idiomatic; behaviour must be identical):
//!
//! | Concept | Rust type | Required SDK surface |
//! |---|---|---|
//! | Snapshot input | [`HealthSnapshot`] | constructor with all fields |
//! | Watcher | [`HealthWatcher`] | `observe(snapshot) -> events`, `observe_unreachable() -> events`, `has_baseline()`, `is_reachable()` |
//! | Trigger | [`RecoveryTrigger`] | factories `health(event)` and `transport_send_failed()` |
//! | Policy config | [`ReconnectPolicyConfig`] | constructor with all fields, no defaults |
//! | Policy | [`ReconnectPolicy`] | `decide(triggers, now_ms)`, `record_attempt_succeeded()`, `record_attempt_failed()`, `reset()` |
//! | Decision | [`ReconnectDecision`] | discriminator + reason / wait_ms / consecutive_failures accessors |
//! | HTTP fetch | [`fetch_health_snapshot_blocking`] | blocking helper returning a snapshot or fetch error |
//! | Session rebuild | [`session::RebuildableSession`] | `reconnect()` on the SDK's primary client |
//!
//! Each SDK is also expected to provide a thin "monitor" facade
//! (Python: `feagi.pns.health_monitor.FeagiHealthMonitor`; Java: TODO -
//! see `NativeFeagiAgentClient`) that composes a fetcher + watcher +
//! policy in the documented order: fetch -> observe (or
//! observe_unreachable on fetch error) -> decide -> on `AttemptNow`,
//! call `reconnect()` and feed the outcome back into the policy.
//! No SDK may add its own decision logic on top of these primitives.
//!
//! @cursor:critical-path
//! @cursor:cross-sdk-contract

pub mod health_watcher;
pub mod reconnect_policy;
pub mod session;

#[cfg(feature = "agent-client-asynchelper-tokio")]
pub mod health_fetch;

pub mod recovery_loop;

pub use health_watcher::{HealthEvent, HealthSnapshot, HealthWatcher};
pub use reconnect_policy::{ReconnectDecision, ReconnectPolicy, ReconnectPolicyConfig, RecoveryTrigger};
pub use recovery_loop::{run_recovery_tick_with_snapshot, RecoveryTickReport};
pub use session::RebuildableSession;

#[cfg(feature = "agent-client-asynchelper-tokio")]
pub use health_fetch::{fetch_health_snapshot_blocking, HealthFetchConfig};

#[cfg(feature = "agent-client-asynchelper-tokio")]
pub use recovery_loop::{run_recovery_tick, run_recovery_tick_blocking};
