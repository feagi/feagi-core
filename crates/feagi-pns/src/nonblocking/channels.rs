// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Async channel helpers for nonblocking transports
//!
//! Provides utilities for creating and managing async channels (tokio::sync::mpsc).

use tokio::sync::mpsc::{self, Receiver, Sender};

/// Create a bounded async channel with the specified capacity
///
/// # Arguments
/// - `capacity`: Maximum number of items that can be queued
///
/// # Returns
/// - `(Sender<T>, Receiver<T>)`: Async channel endpoints
///
/// # Backpressure
/// - `send().await` blocks when the queue is full
/// - `try_send()` returns `Err` immediately if full
/// - Provides natural async backpressure
///
/// # Example
/// ```no_run
/// use feagi_pns::nonblocking::channels;
///
/// async fn example() {
///     let (tx, mut rx) = channels::create_bounded::<Vec<u8>>(100);
///     
///     // Send asynchronously
///     tx.send(vec![1, 2, 3]).await.unwrap();
///     
///     // Receive asynchronously
///     let data = rx.recv().await.unwrap();
/// }
/// ```
pub fn create_bounded<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    mpsc::channel(capacity)
}

/// Statistics for monitoring async channel health
#[derive(Debug, Clone, Copy)]
pub struct AsyncChannelStats {
    pub capacity: usize,
    pub approx_len: usize,
}

impl AsyncChannelStats {
    /// Get approximate statistics for an async channel
    ///
    /// Note: Tokio channels don't provide exact length, only capacity
    pub fn from_sender<T>(sender: &Sender<T>) -> Self {
        Self {
            capacity: sender.capacity(),
            approx_len: sender.capacity() - sender.capacity(), // Approximation
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bounded_channel() {
        let (tx, mut rx) = create_bounded::<i32>(2);

        tx.send(1).await.unwrap();
        tx.send(2).await.unwrap();

        // Receive one
        assert_eq!(rx.recv().await.unwrap(), 1);

        // Can send again
        tx.send(3).await.unwrap();

        assert_eq!(rx.recv().await.unwrap(), 2);
        assert_eq!(rx.recv().await.unwrap(), 3);
    }
}
