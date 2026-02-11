//! Transport-agnostic sensory intake.
//!
//! Receives FeagiByteContainer-format bytes from any transport (ZMQ, WebSocket, SHM, etc.)
//! and exposes them for consumption by the burst engine. FEAGI core does not depend on
//! a specific transport; producers push bytes here and the burst engine polls.

use std::collections::VecDeque;
use std::sync::Mutex;

/// Thread-safe queue of sensory payloads (FeagiByteContainer bytes).
/// Any transport (ZMQ, WebSocket, SHM) pushes here; burst engine polls.
#[derive(Default)]
pub struct SensoryIntakeQueue {
    inner: Mutex<VecDeque<Vec<u8>>>,
}

impl SensoryIntakeQueue {
    /// Create an empty queue.
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(VecDeque::new()),
        }
    }

    /// Push a sensory payload (call from transport layer when data is received).
    pub fn push(&self, bytes: Vec<u8>) {
        if let Ok(mut q) = self.inner.lock() {
            q.push_back(bytes);
        }
    }

    /// Take the next payload if any (called by burst engine each burst).
    pub fn poll_next(&self) -> Option<Vec<u8>> {
        self.inner.lock().ok().and_then(|mut q| q.pop_front())
    }
}