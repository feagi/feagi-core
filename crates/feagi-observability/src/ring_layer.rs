// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! In-process log ring buffer + tracing layer.
//!
//! `init_logging` installs a [`RingBufferLayer`] that captures recent log records
//! into a fixed-capacity ring buffer. The buffer is reachable through a global
//! singleton ([`global_ring`]) so REST handlers (e.g. `/v1/system/log_tail`) can
//! return diagnostics to clients without scraping log files.
//!
//! Design notes:
//! - The buffer is bounded to `FEAGI_LOG_RING_BUFFER_CAPACITY` records (default 2000)
//!   so memory usage stays predictable regardless of log volume.
//! - Records are written non-blocking; the `tracing` layer never blocks the caller.
//! - Reads clone the requested slice so callers do not hold the lock.
//! - Setting `FEAGI_LOG_RING_BUFFER_CAPACITY=0` disables the layer entirely.

use std::collections::VecDeque;
use std::fmt::Write as _;
use std::sync::{Arc, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use parking_lot::RwLock;
use serde::Serialize;
use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

/// Default ring buffer capacity (records). Override via FEAGI_LOG_RING_BUFFER_CAPACITY.
pub const DEFAULT_CAPACITY: usize = 2_000;

/// Environment variable name for capacity override (set to "0" to disable layer).
pub const CAPACITY_ENV_VAR: &str = "FEAGI_LOG_RING_BUFFER_CAPACITY";

/// One captured log record.
#[derive(Debug, Clone, Serialize)]
pub struct LogRecord {
    /// Wall-clock millisecond timestamp when the record was emitted.
    pub timestamp_ms: i64,
    /// Severity level: `TRACE` / `DEBUG` / `INFO` / `WARN` / `ERROR`.
    pub level: String,
    /// Tracing target (typically `crate_name::module`).
    pub target: String,
    /// Source file (best-effort, may be empty in release builds).
    pub file: String,
    /// Source line number (0 when unavailable).
    pub line: u32,
    /// Human-readable message extracted from the event's `message` field.
    pub message: String,
    /// Additional structured fields serialised as JSON.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<serde_json::Value>,
}

/// Bounded ring buffer of log records (oldest are dropped when full).
pub struct LogRingBuffer {
    inner: RwLock<VecDeque<LogRecord>>,
    capacity: usize,
}

impl LogRingBuffer {
    /// Create a new ring buffer with the given capacity. A capacity of 0 means the
    /// buffer is effectively disabled (pushes are no-ops, snapshots are empty).
    pub fn new(capacity: usize) -> Self {
        let initial_capacity = capacity.min(64);
        Self {
            inner: RwLock::new(VecDeque::with_capacity(initial_capacity)),
            capacity,
        }
    }

    /// Maximum number of records the buffer can hold.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Append a record, dropping the oldest one if the buffer is full.
    pub fn push(&self, record: LogRecord) {
        if self.capacity == 0 {
            return;
        }
        let mut inner = self.inner.write();
        if inner.len() == self.capacity {
            inner.pop_front();
        }
        inner.push_back(record);
    }

    /// Return a filtered, ordered (oldest first) snapshot of buffered records.
    ///
    /// # Arguments
    /// * `since_ts_ms` - drop records with `timestamp_ms < since_ts_ms`
    /// * `min_level`   - drop records below this level (TRACE < DEBUG < INFO < WARN < ERROR)
    /// * `target_prefix` - drop records whose `target` does not start with this prefix
    /// * `limit`       - return at most this many records (most recent records win)
    pub fn snapshot(&self, since_ts_ms: Option<i64>, min_level: Option<&str>, target_prefix: Option<&str>, limit: Option<usize>) -> Vec<LogRecord> {
        let inner = self.inner.read();
        let min_rank = min_level.and_then(level_rank);
        let mut filtered: Vec<LogRecord> = inner
            .iter()
            .filter(|r| match since_ts_ms {
                Some(ts) => r.timestamp_ms >= ts,
                None => true,
            })
            .filter(|r| match (min_rank, level_rank(&r.level)) {
                (Some(min), Some(level)) => level >= min,
                _ => true,
            })
            .filter(|r| match target_prefix {
                Some(prefix) => r.target.starts_with(prefix),
                None => true,
            })
            .cloned()
            .collect();

        if let Some(n) = limit {
            if filtered.len() > n {
                let skip = filtered.len() - n;
                filtered.drain(0..skip);
            }
        }
        filtered
    }

    /// Clear all buffered records (used in tests).
    pub fn clear(&self) {
        self.inner.write().clear();
    }
}

/// Convert a level string (case-insensitive) to a numeric rank suitable for
/// "min level" comparisons. Returns `None` for unknown values.
fn level_rank(level: &str) -> Option<u8> {
    match level.to_ascii_uppercase().as_str() {
        "TRACE" => Some(0),
        "DEBUG" => Some(1),
        "INFO" => Some(2),
        "WARN" | "WARNING" => Some(3),
        "ERROR" => Some(4),
        _ => None,
    }
}

fn level_to_str(level: &Level) -> &'static str {
    match *level {
        Level::TRACE => "TRACE",
        Level::DEBUG => "DEBUG",
        Level::INFO => "INFO",
        Level::WARN => "WARN",
        Level::ERROR => "ERROR",
    }
}

fn now_unix_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

/// Tracing layer that pushes events into the global ring buffer.
pub struct RingBufferLayer {
    buffer: Arc<LogRingBuffer>,
}

impl RingBufferLayer {
    pub fn new(buffer: Arc<LogRingBuffer>) -> Self {
        Self { buffer }
    }
}

impl<S: Subscriber> Layer<S> for RingBufferLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        if self.buffer.capacity() == 0 {
            return;
        }
        let metadata = event.metadata();
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);

        let fields = if visitor.extra_fields.is_empty() {
            None
        } else {
            Some(serde_json::Value::Object(visitor.extra_fields))
        };

        let record = LogRecord {
            timestamp_ms: now_unix_ms(),
            level: level_to_str(metadata.level()).to_string(),
            target: metadata.target().to_string(),
            file: metadata.file().unwrap_or_default().to_string(),
            line: metadata.line().unwrap_or(0),
            message: visitor.message,
            fields,
        };
        self.buffer.push(record);
    }
}

/// Visitor that extracts the `message` field plus any other structured fields.
#[derive(Default)]
struct MessageVisitor {
    message: String,
    extra_fields: serde_json::Map<String, serde_json::Value>,
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            // Avoid the debug `"..."` wrapping: write directly into the message.
            let _ = write!(&mut self.message, "{:?}", value);
        } else {
            self.extra_fields
                .insert(field.name().to_string(), serde_json::Value::String(format!("{:?}", value)));
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message.push_str(value);
        } else {
            self.extra_fields
                .insert(field.name().to_string(), serde_json::Value::String(value.to_string()));
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.extra_fields.insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.extra_fields.insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.extra_fields.insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.extra_fields.insert(field.name().to_string(), serde_json::json!(value));
    }
}

static GLOBAL_RING: OnceLock<Arc<LogRingBuffer>> = OnceLock::new();

/// Install (once) the global ring buffer with the given capacity. Subsequent
/// calls are no-ops and return the previously installed instance.
pub fn install_global_ring(capacity: usize) -> Arc<LogRingBuffer> {
    GLOBAL_RING.get_or_init(|| Arc::new(LogRingBuffer::new(capacity))).clone()
}

/// Resolve the configured capacity from `FEAGI_LOG_RING_BUFFER_CAPACITY`.
/// Falls back to [`DEFAULT_CAPACITY`] when unset/invalid.
pub fn capacity_from_env() -> usize {
    std::env::var(CAPACITY_ENV_VAR)
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_CAPACITY)
}

/// Returns the global ring buffer, if installed.
pub fn global_ring() -> Option<Arc<LogRingBuffer>> {
    GLOBAL_RING.get().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_buffer_respects_capacity() {
        let ring = LogRingBuffer::new(3);
        for i in 0..5 {
            ring.push(LogRecord {
                timestamp_ms: i as i64,
                level: "INFO".into(),
                target: "test".into(),
                file: String::new(),
                line: 0,
                message: format!("msg-{}", i),
                fields: None,
            });
        }
        let snap = ring.snapshot(None, None, None, None);
        assert_eq!(snap.len(), 3);
        assert_eq!(snap[0].message, "msg-2");
        assert_eq!(snap[2].message, "msg-4");
    }

    #[test]
    fn snapshot_filters_by_level_and_target() {
        let ring = LogRingBuffer::new(10);
        ring.push(LogRecord {
            timestamp_ms: 100,
            level: "DEBUG".into(),
            target: "feagi-api".into(),
            file: String::new(),
            line: 0,
            message: "debug".into(),
            fields: None,
        });
        ring.push(LogRecord {
            timestamp_ms: 200,
            level: "ERROR".into(),
            target: "feagi-burst-engine".into(),
            file: String::new(),
            line: 0,
            message: "error".into(),
            fields: None,
        });
        ring.push(LogRecord {
            timestamp_ms: 300,
            level: "WARN".into(),
            target: "feagi-api".into(),
            file: String::new(),
            line: 0,
            message: "warn".into(),
            fields: None,
        });

        let warnings = ring.snapshot(None, Some("warn"), None, None);
        assert_eq!(warnings.len(), 2);

        let api_only = ring.snapshot(None, None, Some("feagi-api"), None);
        assert_eq!(api_only.len(), 2);
        assert!(api_only.iter().all(|r| r.target.starts_with("feagi-api")));

        let recent = ring.snapshot(Some(250), None, None, None);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].timestamp_ms, 300);

        let limited = ring.snapshot(None, None, None, Some(2));
        assert_eq!(limited.len(), 2);
        assert_eq!(limited.last().unwrap().timestamp_ms, 300);
    }

    #[test]
    fn capacity_zero_disables_buffer() {
        let ring = LogRingBuffer::new(0);
        ring.push(LogRecord {
            timestamp_ms: 0,
            level: "INFO".into(),
            target: "test".into(),
            file: String::new(),
            line: 0,
            message: "x".into(),
            fields: None,
        });
        assert!(ring.snapshot(None, None, None, None).is_empty());
    }
}
