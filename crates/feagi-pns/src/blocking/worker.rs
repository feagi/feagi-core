//! Worker thread patterns for blocking transports
//!
//! Provides reusable worker thread abstractions for handling asynchronous
//! operations in blocking transports.

use crossbeam::channel::Receiver;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

/// A worker thread that processes items from a queue
///
/// # Design
/// - Dedicated thread pulls items from a bounded queue
/// - Processes items with a user-provided handler function
/// - Supports graceful shutdown via atomic flag
/// - Automatically joins thread on drop
pub struct WorkerThread<T: Send + 'static> {
    handle: Option<JoinHandle<()>>,
    shutdown: Arc<AtomicBool>,
    name: String,
    _phantom: PhantomData<T>,
}

impl<T: Send + 'static> WorkerThread<T> {
    /// Spawn a new worker thread
    ///
    /// # Arguments
    /// - `name`: Thread name for debugging
    /// - `rx`: Receiver to pull work items from
    /// - `handler`: Function to process each item
    ///
    /// # Example
    /// ```no_run
    /// use feagi_pns::blocking::worker::WorkerThread;
    /// use crossbeam::channel;
    ///
    /// let (tx, rx) = channel::bounded(100);
    /// let worker = WorkerThread::spawn(
    ///     "zmq-sender".to_string(),
    ///     rx,
    ///     |data: Vec<u8>| {
    ///         // Process data
    ///         Ok(())
    ///     }
    /// );
    ///
    /// tx.send(vec![1, 2, 3]).unwrap();
    /// ```
    pub fn spawn<F, E>(name: String, rx: Receiver<T>, mut handler: F) -> Self
    where
        F: FnMut(T) -> Result<(), E> + Send + 'static,
        E: std::fmt::Display,
    {
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = Arc::clone(&shutdown);
        let name_clone = name.clone();

        let handle = thread::Builder::new()
            .name(name.clone())
            .spawn(move || {
                while !shutdown_clone.load(Ordering::Relaxed) {
                    match rx.recv_timeout(std::time::Duration::from_millis(100)) {
                        Ok(item) => {
                            if let Err(e) = handler(item) {
                                eprintln!("[Worker:{}] Handler error: {}", name_clone, e);
                            }
                        }
                        Err(crossbeam::channel::RecvTimeoutError::Timeout) => {
                            // Normal timeout, check shutdown flag
                            continue;
                        }
                        Err(crossbeam::channel::RecvTimeoutError::Disconnected) => {
                            // Channel closed, exit
                            break;
                        }
                    }
                }
            })
            .expect("Failed to spawn worker thread");

        Self {
            handle: Some(handle),
            shutdown,
            name,
            _phantom: PhantomData,
        }
    }

    /// Signal the worker to stop and wait for it to finish
    pub fn stop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            if let Err(e) = handle.join() {
                eprintln!("[Worker:{}] Join error: {:?}", self.name, e);
            }
        }
    }

    /// Check if the worker is still running
    pub fn is_running(&self) -> bool {
        !self.shutdown.load(Ordering::Relaxed)
            && self.handle.as_ref().map_or(false, |h| !h.is_finished())
    }
}

impl<T: Send + 'static> Drop for WorkerThread<T> {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam::channel;
    use std::sync::atomic::AtomicUsize;
    use std::time::Duration;

    #[test]
    fn test_worker_thread() {
        let (tx, rx) = channel::bounded(10);
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let mut worker = WorkerThread::spawn(
            "test-worker".to_string(),
            rx,
            move |n: usize| -> Result<(), String> {
                counter_clone.fetch_add(n, Ordering::Relaxed);
                Ok(())
            },
        );

        assert!(worker.is_running());

        // Send some work
        tx.send(1).unwrap();
        tx.send(2).unwrap();
        tx.send(3).unwrap();

        // Wait for processing
        thread::sleep(Duration::from_millis(200));

        assert_eq!(counter.load(Ordering::Relaxed), 6);

        worker.stop();
        assert!(!worker.is_running());
    }
}

