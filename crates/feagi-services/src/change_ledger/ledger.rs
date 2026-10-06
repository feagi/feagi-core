// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! In-process change ledger.
//!
//! Holds the most recent changes in memory for polling (bounded by
//! `session_capacity`) and mirrors every replayable change into the live genome's
//! `change_history` (bounded by `max_persisted_entries`), so a saved genome carries
//! its own history.

use super::context::ChangeContext;
use super::types::{ChangeOrigin, ChangesPage, GenomeChange, GenomeChangeOperation};
use crate::types::{ServiceError, ServiceResult};
use feagi_evolutionary::RuntimeGenome;
use parking_lot::{Mutex, RwLock};
use serde_json::Value;
use std::collections::VecDeque;
use std::sync::Arc;

/// Sizing limits, supplied from the `[genome]` section of the FEAGI configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChangeLedgerConfig {
    /// Changes held in memory for polling.
    pub session_capacity: usize,
    /// Changes kept in the genome's `change_history` (oldest dropped first).
    pub max_persisted_entries: usize,
    /// Longest accepted caller agent id, in bytes.
    pub agent_id_max_length: usize,
}

impl ChangeLedgerConfig {
    /// Reject zero limits, which would make the ledger silently discard everything.
    pub fn validate(&self) -> ServiceResult<()> {
        if self.session_capacity == 0 {
            return Err(ServiceError::InvalidInput(
                "change ledger session_capacity must be greater than 0".to_string(),
            ));
        }
        if self.max_persisted_entries == 0 {
            return Err(ServiceError::InvalidInput(
                "change ledger max_persisted_entries must be greater than 0".to_string(),
            ));
        }
        if self.agent_id_max_length == 0 {
            return Err(ServiceError::InvalidInput(
                "change ledger agent_id_max_length must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }
}

struct LedgerState {
    next_sequence: u64,
    entries: VecDeque<GenomeChange>,
}

/// Records genome changes for one FEAGI process.
pub struct ChangeLedger {
    config: ChangeLedgerConfig,
    current_genome: Arc<RwLock<Option<RuntimeGenome>>>,
    state: Mutex<LedgerState>,
    /// Serializes replay so a change id cannot be applied twice concurrently.
    apply_lock: tokio::sync::Mutex<()>,
}

impl ChangeLedger {
    /// Create a ledger that persists into `current_genome` (the genome service's live genome).
    pub fn new(
        config: ChangeLedgerConfig,
        current_genome: Arc<RwLock<Option<RuntimeGenome>>>,
    ) -> ServiceResult<Self> {
        config.validate()?;
        Ok(Self {
            config,
            current_genome,
            state: Mutex::new(LedgerState {
                next_sequence: 1,
                entries: VecDeque::new(),
            }),
            apply_lock: tokio::sync::Mutex::new(()),
        })
    }

    pub fn config(&self) -> ChangeLedgerConfig {
        self.config
    }

    /// Validate a caller-supplied agent id: trimmed, non-empty, within the configured
    /// length, and free of control characters.
    pub fn validate_agent_id(&self, raw: &str) -> ServiceResult<String> {
        let agent_id = raw.trim();
        if agent_id.is_empty() {
            return Err(ServiceError::InvalidInput(
                "agent id must not be empty".to_string(),
            ));
        }
        if agent_id.len() > self.config.agent_id_max_length {
            return Err(ServiceError::InvalidInput(format!(
                "agent id exceeds {} bytes",
                self.config.agent_id_max_length
            )));
        }
        if agent_id.chars().any(char::is_control) {
            return Err(ServiceError::InvalidInput(
                "agent id must not contain control characters".to_string(),
            ));
        }
        Ok(agent_id.to_string())
    }

    /// Record a successful mutation under `context` and return the entry.
    pub fn record(
        &self,
        operation: GenomeChangeOperation,
        before: Option<Value>,
        after: Option<Value>,
        context: &ChangeContext,
    ) -> GenomeChange {
        let recorded_at_ms = chrono::Utc::now().timestamp_millis();
        let (change_id, group_id, timestamp_ms, origin) = match &context.replay {
            Some(source) => (
                source.change_id.clone(),
                source.group_id.clone(),
                source.timestamp_ms,
                ChangeOrigin::Replayed,
            ),
            None => {
                let change_id = uuid::Uuid::now_v7().to_string();
                let group_id = context
                    .group_id
                    .clone()
                    .unwrap_or_else(|| change_id.clone());
                (change_id, group_id, recorded_at_ms, ChangeOrigin::Local)
            }
        };

        let mut state = self.state.lock();
        let change = GenomeChange {
            sequence: state.next_sequence,
            change_id,
            group_id,
            timestamp_ms,
            recorded_at_ms,
            agent_id: context.agent_id.clone(),
            origin,
            replayable: operation.is_replayable(),
            targets: operation.targets(),
            operation,
            before,
            after,
        };
        state.next_sequence += 1;
        state.entries.push_back(change.clone());
        while state.entries.len() > self.config.session_capacity {
            state.entries.pop_front();
        }
        if change.replayable {
            self.persist_into_genome(&change);
        }
        change
    }

    /// Append to the live genome's history while the ledger state lock is held, so
    /// persisted order matches sequence order.
    fn persist_into_genome(&self, change: &GenomeChange) {
        let mut genome_guard = self.current_genome.write();
        let Some(genome) = genome_guard.as_mut() else {
            tracing::warn!(
                target: "feagi-services",
                "change {} recorded with no genome loaded; not persisted",
                change.change_id
            );
            return;
        };
        match persisted_entry(change) {
            Ok(entry) => {
                genome.change_history.push(entry);
                let excess = genome
                    .change_history
                    .len()
                    .saturating_sub(self.config.max_persisted_entries);
                if excess > 0 {
                    genome.change_history.drain(..excess);
                }
            }
            Err(error) => tracing::error!(
                target: "feagi-services",
                "failed to serialize change {} for genome history: {}",
                change.change_id,
                error
            ),
        }
    }

    /// Changes recorded after `since`, oldest first, at most `limit` of them.
    pub fn changes_since(&self, since: u64, limit: usize) -> ServiceResult<ChangesPage> {
        if limit == 0 || limit > self.config.session_capacity {
            return Err(ServiceError::InvalidInput(format!(
                "limit must be between 1 and {}",
                self.config.session_capacity
            )));
        }
        let state = self.state.lock();
        let latest_sequence = state.next_sequence - 1;
        let oldest_available_sequence = state.entries.front().map(|c| c.sequence);
        let gap = match oldest_available_sequence {
            Some(oldest) => since + 1 < oldest,
            None => since < latest_sequence,
        };
        let mut remaining = state.entries.iter().filter(|c| c.sequence > since);
        let changes: Vec<GenomeChange> = remaining.by_ref().take(limit).cloned().collect();
        let has_more = remaining.next().is_some();
        Ok(ChangesPage {
            changes,
            latest_sequence,
            oldest_available_sequence,
            gap,
            has_more,
        })
    }

    /// Highest sequence recorded so far (0 when nothing was recorded).
    pub fn latest_sequence(&self) -> u64 {
        self.state.lock().next_sequence - 1
    }

    /// Whether `change_id` is already part of this brain, either in memory or in the
    /// live genome's persisted history.
    pub fn contains_change_id(&self, change_id: &str) -> bool {
        if self
            .state
            .lock()
            .entries
            .iter()
            .any(|c| c.change_id == change_id)
        {
            return true;
        }
        self.current_genome.read().as_ref().is_some_and(|genome| {
            genome
                .change_history
                .iter()
                .any(|entry| entry.get("change_id").and_then(Value::as_str) == Some(change_id))
        })
    }

    /// Most recent in-memory entry for `change_id`.
    pub fn find(&self, change_id: &str) -> Option<GenomeChange> {
        self.state
            .lock()
            .entries
            .iter()
            .rev()
            .find(|c| c.change_id == change_id)
            .cloned()
    }

    /// Guard held for the duration of one replay.
    pub async fn lock_apply(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.apply_lock.lock().await
    }
}

/// Persisted form of a change: the entry without the process-local sequence.
fn persisted_entry(change: &GenomeChange) -> serde_json::Result<Value> {
    let mut value = serde_json::to_value(change)?;
    if let Some(map) = value.as_object_mut() {
        map.remove("sequence");
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change_ledger::context::ReplaySource;

    fn config(session_capacity: usize, max_persisted_entries: usize) -> ChangeLedgerConfig {
        ChangeLedgerConfig {
            session_capacity,
            max_persisted_entries,
            agent_id_max_length: 16,
        }
    }

    fn empty_genome() -> RuntimeGenome {
        feagi_evolutionary::create_minimal_genome("g".to_string(), "g".to_string())
    }

    fn ledger_with_genome(
        cfg: ChangeLedgerConfig,
    ) -> (ChangeLedger, Arc<RwLock<Option<RuntimeGenome>>>) {
        let genome = Arc::new(RwLock::new(Some(empty_genome())));
        (
            ChangeLedger::new(cfg, genome.clone()).expect("ledger"),
            genome,
        )
    }

    fn delete_op(id: &str) -> GenomeChangeOperation {
        GenomeChangeOperation::DeleteCorticalArea {
            cortical_id: id.to_string(),
        }
    }

    #[test]
    fn rejects_zero_limits() {
        let genome = Arc::new(RwLock::new(None));
        assert!(ChangeLedger::new(config(0, 1), genome.clone()).is_err());
        assert!(ChangeLedger::new(config(1, 0), genome.clone()).is_err());
        let mut cfg = config(1, 1);
        cfg.agent_id_max_length = 0;
        assert!(ChangeLedger::new(cfg, genome).is_err());
    }

    #[test]
    fn records_monotonic_sequences_with_attribution() {
        let (ledger, _) = ledger_with_genome(config(10, 10));
        let context = ChangeContext {
            agent_id: Some("member-a".to_string()),
            group_id: Some("req-1".to_string()),
            replay: None,
        };
        let first = ledger.record(delete_op("a"), None, None, &context);
        let second = ledger.record(delete_op("b"), None, None, &context);
        assert_eq!((first.sequence, second.sequence), (1, 2));
        assert_ne!(first.change_id, second.change_id);
        assert_eq!(first.group_id, "req-1");
        assert_eq!(first.agent_id.as_deref(), Some("member-a"));
        assert_eq!(first.origin, ChangeOrigin::Local);
        assert_eq!(ledger.latest_sequence(), 2);
    }

    #[test]
    fn change_without_group_gets_its_own_group() {
        let (ledger, _) = ledger_with_genome(config(10, 10));
        let change = ledger.record(delete_op("a"), None, None, &ChangeContext::default());
        assert_eq!(change.group_id, change.change_id);
        assert!(change.agent_id.is_none());
    }

    #[test]
    fn replay_preserves_original_identity() {
        let (ledger, _) = ledger_with_genome(config(10, 10));
        let context = ChangeContext {
            agent_id: Some("member-b".to_string()),
            group_id: None,
            replay: Some(ReplaySource {
                change_id: "orig".to_string(),
                group_id: "orig-group".to_string(),
                timestamp_ms: 42,
            }),
        };
        let change = ledger.record(delete_op("a"), None, None, &context);
        assert_eq!(change.change_id, "orig");
        assert_eq!(change.group_id, "orig-group");
        assert_eq!(change.timestamp_ms, 42);
        assert_eq!(change.origin, ChangeOrigin::Replayed);
        assert!(ledger.contains_change_id("orig"));
    }

    #[test]
    fn replayable_changes_persist_into_genome_without_sequence() {
        let (ledger, genome) = ledger_with_genome(config(10, 10));
        let change = ledger.record(delete_op("a"), None, None, &ChangeContext::default());
        ledger.record(
            GenomeChangeOperation::ConnectomeReset,
            None,
            None,
            &ChangeContext::default(),
        );
        let guard = genome.read();
        let history = &guard.as_ref().expect("genome").change_history;
        assert_eq!(history.len(), 1, "markers are not persisted");
        assert_eq!(history[0]["change_id"], change.change_id.as_str());
        assert!(history[0].get("sequence").is_none());
    }

    #[test]
    fn persisted_history_drops_oldest_beyond_limit() {
        let (ledger, genome) = ledger_with_genome(config(10, 2));
        let ids: Vec<String> = ["a", "b", "c"]
            .iter()
            .map(|id| {
                ledger
                    .record(delete_op(id), None, None, &ChangeContext::default())
                    .change_id
            })
            .collect();
        let guard = genome.read();
        let history = &guard.as_ref().expect("genome").change_history;
        let kept: Vec<&str> = history
            .iter()
            .map(|e| e["change_id"].as_str().expect("id"))
            .collect();
        assert_eq!(kept, vec![ids[1].as_str(), ids[2].as_str()]);
    }

    #[test]
    fn changes_since_pages_and_reports_more() {
        let (ledger, _) = ledger_with_genome(config(10, 10));
        for id in ["a", "b", "c"] {
            ledger.record(delete_op(id), None, None, &ChangeContext::default());
        }
        let page = ledger.changes_since(0, 2).expect("page");
        assert_eq!(
            page.changes.iter().map(|c| c.sequence).collect::<Vec<_>>(),
            vec![1, 2]
        );
        assert!(page.has_more);
        assert!(!page.gap);
        let rest = ledger.changes_since(2, 2).expect("page");
        assert_eq!(rest.changes.len(), 1);
        assert!(!rest.has_more);
        assert_eq!(rest.latest_sequence, 3);
    }

    #[test]
    fn eviction_is_reported_as_gap() {
        let (ledger, _) = ledger_with_genome(config(2, 10));
        for id in ["a", "b", "c"] {
            ledger.record(delete_op(id), None, None, &ChangeContext::default());
        }
        let page = ledger.changes_since(0, 2).expect("page");
        assert!(page.gap);
        assert_eq!(page.oldest_available_sequence, Some(2));
        let caught_up = ledger.changes_since(1, 2).expect("page");
        assert!(!caught_up.gap);
    }

    #[test]
    fn rejects_out_of_range_limit() {
        let (ledger, _) = ledger_with_genome(config(5, 10));
        assert!(ledger.changes_since(0, 0).is_err());
        assert!(ledger.changes_since(0, 6).is_err());
    }

    #[test]
    fn contains_change_id_reads_persisted_history() {
        let (ledger, genome) = ledger_with_genome(config(10, 10));
        genome
            .write()
            .as_mut()
            .expect("genome")
            .change_history
            .push(serde_json::json!({"change_id": "from-file"}));
        assert!(ledger.contains_change_id("from-file"));
        assert!(!ledger.contains_change_id("unknown"));
    }

    #[test]
    fn validates_agent_ids() {
        let (ledger, _) = ledger_with_genome(config(10, 10));
        assert_eq!(
            ledger.validate_agent_id("  member-a ").expect("ok"),
            "member-a"
        );
        assert!(ledger.validate_agent_id("   ").is_err());
        assert!(ledger.validate_agent_id(&"x".repeat(17)).is_err());
        assert!(ledger.validate_agent_id("bad\nid").is_err());
    }
}
