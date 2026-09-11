//! Transport-agnostic sensory intake.
//!
//! Receives FeagiByteContainer-format bytes from any transport (ZMQ, WebSocket, SHM, etc.)
//! and exposes them for consumption by the burst engine. FEAGI core does not depend on
//! a specific transport; producers push bytes here and the burst engine polls.

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Instant;

const MAX_SENSORY_QUEUE_DEPTH: usize = 512;

#[derive(Clone, Debug)]
pub struct SensoryPacket {
    pub bytes: Vec<u8>,
    pub source_id: Option<String>,
    pub received_at: Instant,
}

/// Thread-safe queue of sensory payloads (FeagiByteContainer bytes).
/// Any transport (ZMQ, WebSocket, SHM) pushes here; burst engine polls.
#[derive(Default)]
pub struct SensoryIntakeQueue {
    inner: Mutex<VecDeque<SensoryPacket>>,
}

impl SensoryIntakeQueue {
    /// Create an empty queue.
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(VecDeque::new()),
        }
    }

    /// Push a sensory payload (call from transport layer when data is received).
    ///
    /// Payloads are queued in arrival order with bounded memory.
    /// If the queue reaches capacity, the oldest payload is dropped.
    pub fn push(&self, bytes: Vec<u8>) {
        self.push_with_source(bytes, None);
    }

    pub fn push_with_source(&self, bytes: Vec<u8>, source_id: Option<String>) {
        if let Ok(mut q) = self.inner.lock() {
            if q.len() >= MAX_SENSORY_QUEUE_DEPTH {
                q.pop_front();
            }
            q.push_back(SensoryPacket {
                bytes,
                source_id,
                received_at: Instant::now(),
            });
        }
    }

    /// Take the next payload if any (called by burst engine each burst).
    pub fn poll_next(&self) -> Option<SensoryPacket> {
        self.inner.lock().ok().and_then(|mut q| q.pop_front())
    }

    /// Drop all queued sensory payloads.
    ///
    /// Used by strict genome transitions to guarantee no stale pre-transition
    /// sensory data can be consumed after a new genome is loaded.
    pub fn clear(&self) {
        if let Ok(mut q) = self.inner.lock() {
            q.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SensoryIntakeQueue;

    #[test]
    fn preserves_arrival_order() {
        let q = SensoryIntakeQueue::new();
        q.push(vec![1]);
        q.push(vec![2]);
        q.push(vec![3]);

        assert_eq!(q.poll_next().map(|packet| packet.bytes), Some(vec![1]));
        assert_eq!(q.poll_next().map(|packet| packet.bytes), Some(vec![2]));
        assert_eq!(q.poll_next().map(|packet| packet.bytes), Some(vec![3]));
        assert!(q.poll_next().is_none());
    }

    #[test]
    fn drops_oldest_when_bounded_capacity_is_reached() {
        let q = SensoryIntakeQueue::new();
        for idx in 0..513u16 {
            q.push(vec![idx as u8]);
        }

        // Queue depth is capped at 512, so the first element (0) is dropped.
        assert_eq!(q.poll_next().map(|packet| packet.bytes), Some(vec![1]));
    }

    #[test]
    fn preserves_source_metadata() {
        let q = SensoryIntakeQueue::new();
        q.push_with_source(vec![9], Some("agent-a".to_string()));

        let packet = q.poll_next().expect("expected one packet");
        assert_eq!(packet.bytes, vec![9]);
        assert_eq!(packet.source_id.as_deref(), Some("agent-a"));
    }
}
