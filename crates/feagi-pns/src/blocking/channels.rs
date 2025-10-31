//! Channel helpers for bounded queues with backpressure
//!
//! Provides utilities for creating and managing bounded channels used
//! in blocking transports for flow control.

use crossbeam::channel::{bounded, Receiver, Sender};

/// Create a bounded channel with the specified capacity
///
/// # Arguments
/// - `capacity`: Maximum number of items that can be queued
///
/// # Returns
/// - `(Sender<T>, Receiver<T>)`: Channel endpoints
///
/// # Backpressure
/// - `send()` blocks when the queue is full
/// - `try_send()` returns `Err` immediately if full
/// - This provides natural backpressure to prevent unbounded memory growth
pub fn create_bounded<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    bounded(capacity)
}

/// Statistics for monitoring channel health
#[derive(Debug, Clone, Copy)]
pub struct ChannelStats {
    pub capacity: usize,
    pub len: usize,
    pub is_full: bool,
    pub is_empty: bool,
}

impl ChannelStats {
    /// Get statistics for a channel
    pub fn from_channel<T>(sender: &Sender<T>) -> Self {
        let capacity = sender.capacity().unwrap_or(0);
        let len = sender.len();
        Self {
            capacity,
            len,
            is_full: sender.is_full(),
            is_empty: sender.is_empty(),
        }
    }

    /// Calculate utilization percentage (0.0 to 1.0)
    pub fn utilization(&self) -> f64 {
        if self.capacity == 0 {
            0.0
        } else {
            self.len as f64 / self.capacity as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounded_channel() {
        let (tx, rx) = create_bounded::<i32>(2);

        tx.send(1).unwrap();
        tx.send(2).unwrap();

        // Channel is full
        assert!(tx.try_send(3).is_err());

        // Receive one
        assert_eq!(rx.recv().unwrap(), 1);

        // Now we can send again
        tx.send(3).unwrap();
    }

    #[test]
    fn test_channel_stats() {
        let (tx, _rx) = create_bounded::<i32>(10);

        let stats = ChannelStats::from_channel(&tx);
        assert_eq!(stats.capacity, 10);
        assert_eq!(stats.len, 0);
        assert!(stats.is_empty);
        assert!(!stats.is_full);
        assert_eq!(stats.utilization(), 0.0);
    }
}




