// Visualization stream for sending neuron activity to Brain Visualizer (ZMQ fallback for remote clients)
// Uses PUB socket pattern for one-to-many distribution with an asynchronous sender to avoid frame loss.

use crossbeam::queue::ArrayQueue;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Overflow handling strategy when the visualization queue is saturated.
#[derive(Clone, Copy, Debug)]
pub enum VisualizationOverflowStrategy {
    /// Remove the oldest frame in the queue (preserves most recent activity).
    DropOldest,
    /// Drop the newest frame (preserves historical ordering).
    DropNewest,
}

impl Default for VisualizationOverflowStrategy {
    fn default() -> Self {
        VisualizationOverflowStrategy::DropOldest
    }
}

/// Runtime configuration for the visualization send pipeline.
#[derive(Clone, Debug)]
pub struct VisualizationSendConfig {
    pub queue_capacity: usize,
    pub send_timeout_ms: i32,
    pub idle_sleep_ms: u64,
    pub overflow_strategy: VisualizationOverflowStrategy,
    pub backpressure_sleep_ms: u64,
}

impl Default for VisualizationSendConfig {
    fn default() -> Self {
        Self {
            queue_capacity: 512,
            send_timeout_ms: -1, // Block until send succeeds
            idle_sleep_ms: 1,
            overflow_strategy: VisualizationOverflowStrategy::DropOldest,
            backpressure_sleep_ms: 1,
        }
    }
}

impl VisualizationSendConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.queue_capacity == 0 {
            return Err("Visualization queue capacity must be greater than zero".to_string());
        }
        if self.backpressure_sleep_ms == 0 {
            return Err("backpressure_sleep_ms must be at least 1".to_string());
        }
        Ok(())
    }
}

#[derive(Debug)]
struct VisualizationQueueItem {
    topic: Vec<u8>,
    payload: Vec<u8>,
    enqueued_at: Instant,
}

#[derive(Default, Debug)]
struct VisualizationQueueStats {
    enqueued: AtomicU64,
    dropped: AtomicU64,
    send_failures: AtomicU64,
    queue_high_watermark: AtomicUsize,
    backpressure_waits: AtomicU64,
}

impl VisualizationQueueStats {
    fn record_enqueue(&self, current_len: usize) {
        self.enqueued.fetch_add(1, Ordering::Relaxed);
        self.update_high_watermark(current_len);
    }

    fn record_drop(&self) -> u64 {
        self.dropped.fetch_add(1, Ordering::Relaxed) + 1
    }

    fn record_send_failure(&self) {
        self.send_failures.fetch_add(1, Ordering::Relaxed);
    }

    fn record_backpressure_wait(&self) -> u64 {
        self.backpressure_waits.fetch_add(1, Ordering::Relaxed) + 1
    }

    fn update_high_watermark(&self, current_len: usize) {
        let mut previous = self.queue_high_watermark.load(Ordering::Relaxed);
        while current_len > previous {
            match self.queue_high_watermark.compare_exchange(
                previous,
                current_len,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => previous = actual,
            }
        }
    }
}

/// Visualization stream for publishing neuron activity.
#[derive(Clone)]
pub struct VisualizationStream {
    context: Arc<zmq::Context>,
    bind_address: String,
    socket: Arc<Mutex<Option<zmq::Socket>>>,
    running: Arc<Mutex<bool>>,
    send_config: VisualizationSendConfig,
    queue: Arc<ArrayQueue<VisualizationQueueItem>>,
    stats: Arc<VisualizationQueueStats>,
    shutdown: Arc<AtomicBool>,
    worker_thread: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
}

impl VisualizationStream {
    /// Create a new visualization stream
    pub fn new(
        context: Arc<zmq::Context>,
        bind_address: &str,
        config: VisualizationSendConfig,
    ) -> Result<Self, String> {
        config.validate()?;

        Ok(Self {
            context,
            bind_address: bind_address.to_string(),
            socket: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
            queue: Arc::new(ArrayQueue::new(config.queue_capacity)),
            stats: Arc::new(VisualizationQueueStats {
                enqueued: AtomicU64::new(0),
                dropped: AtomicU64::new(0),
                send_failures: AtomicU64::new(0),
                queue_high_watermark: AtomicUsize::new(0),
                backpressure_waits: AtomicU64::new(0),
            }),
            shutdown: Arc::new(AtomicBool::new(false)),
            worker_thread: Arc::new(Mutex::new(None)),
            send_config: config,
        })
    }

    /// Start the visualization stream
    pub fn start(&self) -> Result<(), String> {
        if *self.running.lock() {
            return Err("Visualization stream already running".to_string());
        }

        while self.queue.pop().is_some() {}

        let socket = self.context.socket(zmq::PUB).map_err(|e| e.to_string())?;

        socket.set_linger(0).map_err(|e| e.to_string())?;
        socket.set_sndhwm(10000).map_err(|e| e.to_string())?;
        socket.set_conflate(false).map_err(|e| e.to_string())?;
        socket
            .set_sndtimeo(self.send_config.send_timeout_ms)
            .map_err(|e| e.to_string())?;

        socket.bind(&self.bind_address).map_err(|e| e.to_string())?;

        *self.socket.lock() = Some(socket);
        *self.running.lock() = true;
        self.shutdown.store(false, Ordering::Relaxed);

        self.spawn_worker();

        println!("🦀 [ZMQ-VIZ] Listening on {}", self.bind_address);

        Ok(())
    }

    /// Stop the visualization stream
    pub fn stop(&self) -> Result<(), String> {
        self.shutdown.store(true, Ordering::Relaxed);

        if let Some(handle) = self.worker_thread.lock().take() {
            if let Err(err) = handle.join() {
                eprintln!("[ZMQ-VIZ] Worker thread join error: {:?}", err);
            }
        }

        *self.running.lock() = false;
        *self.socket.lock() = None;

        Ok(())
    }

    /// Enqueue visualization data for asynchronous delivery (with LZ4 compression).
    pub fn publish(&self, data: &[u8]) -> Result<(), String> {
        static FIRST_LOG: AtomicBool = AtomicBool::new(false);
        if !FIRST_LOG.load(Ordering::Relaxed) {
            eprintln!(
                "[VIZ-STREAM] 🔍 TRACE: publish() called with {} bytes (BEFORE compression)",
                data.len()
            );
            FIRST_LOG.store(true, Ordering::Relaxed);
        }

        let compressed =
            match lz4::block::compress(data, Some(lz4::block::CompressionMode::FAST(1)), true) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[VIZ-STREAM] ❌ LZ4 FAILED: {:?}", e);
                    return Err(format!("LZ4 compression failed: {}", e));
                }
            };

        let item = VisualizationQueueItem {
            topic: b"activity".to_vec(),
            payload: compressed,
            enqueued_at: Instant::now(),
        };

        self.enqueue(item);

        Ok(())
    }

    /// Check if stream is running
    pub fn is_running(&self) -> bool {
        *self.running.lock()
    }

    fn enqueue(&self, item: VisualizationQueueItem) {
        let sleep_duration = Duration::from_millis(self.send_config.backpressure_sleep_ms);
        let mut pending = Some(item);
        let mut wait_count = 0_u64;

        while let Some(current) = pending.take() {
            match self.queue.push(current) {
                Ok(()) => {
                    self.stats.record_enqueue(self.queue.len());
                    break;
                }
                Err(returned) => {
                    pending = Some(returned);
                    wait_count = self.stats.record_backpressure_wait();

                    if wait_count == 1 || wait_count % 100 == 0 {
                        eprintln!(
                            "⚠️  [ZMQ-VIZ] Backpressure active - queue full (waits: {})",
                            wait_count
                        );
                    }

                    if self.shutdown.load(Ordering::Relaxed) {
                        let dropped = self.stats.record_drop();
                        eprintln!(
                            "⚠️  [ZMQ-VIZ] Shutdown while queue saturated - dropping frame ({} total drops)",
                            dropped
                        );
                        break;
                    }

                    thread::sleep(sleep_duration);
                }
            }
        }
    }

    fn spawn_worker(&self) {
        let queue = Arc::clone(&self.queue);
        let socket = Arc::clone(&self.socket);
        let shutdown = Arc::clone(&self.shutdown);
        let stats = Arc::clone(&self.stats);
        let idle_sleep = Duration::from_millis(self.send_config.idle_sleep_ms);

        let handle = thread::Builder::new()
            .name("feagi-viz-sender".to_string())
            .spawn(move || {
                while !shutdown.load(Ordering::Relaxed) {
                    if let Some(item) = queue.pop() {
                        Self::send_item(&socket, item, &stats);
                        continue;
                    }

                    thread::sleep(idle_sleep);
                }

                while let Some(item) = queue.pop() {
                    Self::send_item(&socket, item, &stats);
                }
            })
            .expect("Failed to spawn visualization sender thread");

        *self.worker_thread.lock() = Some(handle);
    }

    fn send_item(
        socket: &Arc<Mutex<Option<zmq::Socket>>>,
        item: VisualizationQueueItem,
        stats: &VisualizationQueueStats,
    ) {
        let mut guard = socket.lock();
        let sock = match guard.as_mut() {
            Some(sock) => sock,
            None => {
                stats.record_drop();
                return;
            }
        };

        loop {
            if let Err(e) = sock.send(&item.topic, zmq::SNDMORE) {
                eprintln!("❌ [ZMQ-VIZ] Topic send failed: {}", e);
                stats.record_send_failure();
                return;
            }

            match sock.send(&item.payload, 0) {
                Ok(()) => break,
                Err(zmq::Error::EAGAIN) => {
                    let waits = stats.record_backpressure_wait();
                    if waits == 1 || waits % 100 == 0 {
                        eprintln!(
                            "⚠️  [ZMQ-VIZ] Send backpressure from ZMQ socket (waits: {})",
                            waits
                        );
                    }
                    if shutdown.load(Ordering::Relaxed) {
                        let drops = stats.record_drop();
                        eprintln!(
                            "⚠️  [ZMQ-VIZ] Shutdown during send - dropping frame ({} drops)",
                            drops
                        );
                        return;
                    }
                    thread::sleep(Duration::from_millis(5));
                }
                Err(other) => {
                    eprintln!("❌ [ZMQ-VIZ] Payload send failed: {}", other);
                    stats.record_send_failure();
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viz_stream_creation() {
        let ctx = Arc::new(zmq::Context::new());
        let stream = VisualizationStream::new(
            ctx,
            "tcp://127.0.0.1:30010",
            VisualizationSendConfig::default(),
        );
        assert!(stream.is_ok());
    }
}
