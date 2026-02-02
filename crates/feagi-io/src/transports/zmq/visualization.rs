// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// Visualization stream for sending neuron activity to Brain Visualizer (ZMQ fallback for remote clients)
// Uses PUB socket pattern for one-to-many distribution with an asynchronous sender to avoid frame loss.

use crossbeam::queue::ArrayQueue;
use feagi_structures::FeagiDataError;
use parking_lot::Mutex;
use std::future::Future;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::runtime::Handle;
use tokio::runtime::Runtime;
use tokio::task::block_in_place;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use zeromq::{PubSocket, Socket, SocketSend, ZmqError, ZmqMessage};

fn block_on_runtime<T>(runtime: &Runtime, future: impl Future<Output = T>) -> T {
    if Handle::try_current().is_ok() {
        block_in_place(|| Handle::current().block_on(future))
    } else {
        runtime.block_on(future)
    }
}

/// Overflow handling strategy when the visualization queue is saturated.
#[derive(Clone, Copy, Debug, Default)]
pub enum VisualizationOverflowStrategy {
    /// Remove the oldest frame in the queue (preserves most recent activity).
    #[default]
    DropOldest,
    /// Drop the newest frame (preserves historical ordering).
    DropNewest,
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
            // REAL-TIME: Keep only 1 frame in queue for absolute minimum latency
            // Brain Visualizer should see real-time activity, not buffered history
            queue_capacity: 1,
            send_timeout_ms: -1, // Block until send succeeds
            idle_sleep_ms: 1,
            // REAL-TIME: Drop oldest frames when queue is full
            overflow_strategy: VisualizationOverflowStrategy::DropOldest,
            backpressure_sleep_ms: 1,
        }
    }
}

impl VisualizationSendConfig {
    pub fn validate(&self) -> Result<(), FeagiDataError> {
        if self.queue_capacity == 0 {
            return Err(FeagiDataError::BadParameters(
                "Visualization queue capacity must be greater than zero".to_string(),
            ));
        }
        if self.backpressure_sleep_ms == 0 {
            return Err(FeagiDataError::BadParameters(
                "backpressure_sleep_ms must be at least 1".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug)]
/// Queue item for visualization stream
/// Contains raw fire queue data that will be serialized on the worker thread
struct VisualizationQueueItem {
    topic: Vec<u8>,
    raw_fire_data: feagi_npu_burst_engine::RawFireQueueSnapshot,
    /// Optional: For backwards compatibility with pre-serialized data (SHM path)
    pre_serialized_payload: Option<Vec<u8>>,
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
    runtime: Arc<Runtime>,
    bind_address: String,
    socket: Arc<Mutex<Option<PubSocket>>>,
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
        runtime: Arc<Runtime>,
        bind_address: &str,
        config: VisualizationSendConfig,
    ) -> Result<Self, FeagiDataError> {
        config.validate()?;

        Ok(Self {
            runtime,
            bind_address: bind_address.to_string(),
            socket: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
            queue: Arc::new(ArrayQueue::new(config.queue_capacity)),
            stats: Arc::new(VisualizationQueueStats::default()),
            shutdown: Arc::new(AtomicBool::new(false)),
            worker_thread: Arc::new(Mutex::new(None)),
            send_config: config,
        })
    }

    /// Start the visualization stream
    pub fn start(&self) -> Result<(), FeagiDataError> {
        if *self.running.lock() {
            return Err(FeagiDataError::BadParameters(
                "Visualization stream already running".to_string(),
            ));
        }

        while self.queue.pop().is_some() {}

        let mut socket = PubSocket::new();
        block_on_runtime(self.runtime.as_ref(), socket.bind(&self.bind_address))
            .map_err(|e| FeagiDataError::InternalError(format!("Failed to bind socket: {}", e)))?;

        *self.socket.lock() = Some(socket);
        *self.running.lock() = true;
        self.shutdown.store(false, Ordering::Relaxed);

        self.spawn_worker();

        info!("🦀 [ZMQ-VIZ] Listening on {}", self.bind_address);

        Ok(())
    }

    /// Stop the visualization stream
    pub fn stop(&self) -> Result<(), FeagiDataError> {
        self.shutdown.store(true, Ordering::Relaxed);

        if let Some(handle) = self.worker_thread.lock().take() {
            if let Err(err) = handle.join() {
                error!("[ZMQ-VIZ] Worker thread join error: {:?}", err);
            }
        }

        *self.running.lock() = false;
        *self.socket.lock() = None;

        Ok(())
    }

    /// Publish raw fire queue data (NEW ARCHITECTURE - serialization in worker thread)
    /// This keeps serialization out of the burst engine hot path
    pub fn publish_raw_fire_queue(
        &self,
        fire_data: feagi_npu_burst_engine::RawFireQueueSnapshot,
    ) -> Result<(), FeagiDataError> {
        // Fast path: If stream not running, don't even try to enqueue
        if !*self.running.lock() {
            return Ok(()); // Silently discard - expected when no viz agents connected
        }

        static FIRST_LOG: AtomicBool = AtomicBool::new(false);
        if !FIRST_LOG.load(Ordering::Relaxed) {
            let total_neurons: usize = fire_data.values().map(|d| d.neuron_ids.len()).sum();
            info!(
                "[VIZ-STREAM] 🏗️ ARCHITECTURE: publish_raw_fire_queue() - {} neurons, {} areas (serialization will happen on worker thread)",
                total_neurons, fire_data.len()
            );
            FIRST_LOG.store(true, Ordering::Relaxed);
        }

        let item = VisualizationQueueItem {
            topic: b"activity".to_vec(),
            raw_fire_data: fire_data,
            pre_serialized_payload: None, // Will be serialized on worker thread
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

        while let Some(current) = pending.take() {
            match self.queue.push(current) {
                Ok(()) => {
                    self.stats.record_enqueue(self.queue.len());
                    debug!(
                        "[ZMQ-VIZ] enqueue success: queue_len={} hwm={} waits={} drops={}",
                        self.queue.len(),
                        self.stats.queue_high_watermark.load(Ordering::Relaxed),
                        self.stats.backpressure_waits.load(Ordering::Relaxed),
                        self.stats.dropped.load(Ordering::Relaxed)
                    );
                    break;
                }
                Err(returned) => {
                    pending = Some(returned);
                    let backpressure_count = self.stats.record_backpressure_wait();

                    if backpressure_count == 1 || backpressure_count.is_multiple_of(100) {
                        warn!(
                            "⚠️  [ZMQ-VIZ] Backpressure active - queue full (waits: {})",
                            backpressure_count
                        );
                        warn!(
                            "[ZMQ-VIZ] queue snapshot: len={} hwm={} drops={} waits={}",
                            self.queue.len(),
                            self.stats.queue_high_watermark.load(Ordering::Relaxed),
                            self.stats.dropped.load(Ordering::Relaxed),
                            backpressure_count
                        );
                    }

                    if self.shutdown.load(Ordering::Relaxed) {
                        let dropped = self.stats.record_drop();
                        warn!(
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
        let runtime = Arc::clone(&self.runtime);
        let shutdown = Arc::clone(&self.shutdown);
        let stats = Arc::clone(&self.stats);
        let idle_sleep = Duration::from_millis(self.send_config.idle_sleep_ms);
        let send_retry_sleep = Duration::from_millis(self.send_config.backpressure_sleep_ms);
        let send_timeout_ms = self.send_config.send_timeout_ms;

        let handle = thread::Builder::new()
            .name("feagi-viz-sender".to_string())
            .spawn(move || {
                while !shutdown.load(Ordering::Relaxed) {
                    if let Some(item) = queue.pop() {
                        Self::send_item(
                            &socket,
                            &runtime,
                            item,
                            &stats,
                            &shutdown,
                            send_retry_sleep,
                            send_timeout_ms,
                        );
                        continue;
                    }

                    thread::sleep(idle_sleep);
                }

                while let Some(item) = queue.pop() {
                    Self::send_item(
                        &socket,
                        &runtime,
                        item,
                        &stats,
                        &shutdown,
                        send_retry_sleep,
                        send_timeout_ms,
                    );
                }
            })
            .expect("Failed to spawn visualization sender thread");

        *self.worker_thread.lock() = Some(handle);
    }

    /// Serialize raw fire queue data to FeagiByteContainer format
    /// This runs on the PNS worker thread, NOT the burst engine thread
    fn serialize_fire_queue(
        fire_data: feagi_npu_burst_engine::RawFireQueueSnapshot,
    ) -> Result<Vec<u8>, FeagiDataError> {
        use feagi_serialization::FeagiByteContainer;
        use feagi_structures::genomic::cortical_area::CorticalID;
        use feagi_structures::neuron_voxels::xyzp::{
            CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZPArrays,
        };

        let mut cortical_mapped = CorticalMappedXYZPNeuronVoxels::new();

        for (_area_id, area_data) in fire_data {
            if area_data.neuron_ids.is_empty() {
                continue;
            }

            // Create CorticalID from cortical_id (base64 encoded)
            let cortical_id =
                CorticalID::try_from_base_64(&area_data.cortical_id).map_err(|e| {
                    FeagiDataError::BadParameters(format!(
                        "Failed to decode CorticalID from base64 '{}': {:?}",
                        area_data.cortical_id, e
                    ))
                })?;

            // Create neuron voxel arrays - MOVE vectors instead of cloning (takes ownership)
            // CRITICAL PERFORMANCE: This eliminates expensive cloning for large areas
            // Even though we're on a different thread, we can still move the data
            let neuron_arrays = NeuronVoxelXYZPArrays::new_from_vectors(
                area_data.coords_x,   // Move instead of clone
                area_data.coords_y,   // Move instead of clone
                area_data.coords_z,   // Move instead of clone
                area_data.potentials, // Move instead of clone
            )?;

            cortical_mapped.insert(cortical_id, neuron_arrays);
        }

        // Serialize to FeagiByteContainer
        // Note: overwrite_byte_data_with_single_struct_data() already handles efficient allocation internally:
        // - It pre-calculates size via get_number_of_bytes_needed()
        // - Only resizes if current capacity is insufficient
        // - Reuses existing allocation when possible
        let mut byte_container = FeagiByteContainer::new_empty();
        byte_container.overwrite_byte_data_with_single_struct_data(&cortical_mapped, 0)?;

        Ok(byte_container.get_byte_ref().to_vec())
    }

    fn send_item(
        socket: &Arc<Mutex<Option<PubSocket>>>,
        runtime: &Arc<Runtime>,
        item: VisualizationQueueItem,
        stats: &VisualizationQueueStats,
        shutdown: &Arc<AtomicBool>,
        retry_sleep: Duration,
        send_timeout_ms: i32,
    ) {
        // Step 1: Serialize raw fire queue data on this worker thread (OFF BURST THREAD!)
        let payload = if let Some(pre_serialized) = item.pre_serialized_payload {
            // Backwards compatibility: use pre-serialized data
            pre_serialized
        } else {
            // NEW PATH: Serialize raw fire queue data here (not in burst engine!)
            let total_neurons: usize = item
                .raw_fire_data
                .values()
                .map(|d| d.neuron_ids.len())
                .sum();

            static FIRST_SERIALIZE_LOG: AtomicBool = AtomicBool::new(false);
            if !FIRST_SERIALIZE_LOG.load(Ordering::Relaxed) || total_neurons > 1000 {
                info!("[ZMQ-VIZ] 🏗️ SERIALIZING: {} neurons from {} areas (on PNS worker thread, NOT burst thread)",
                    total_neurons, item.raw_fire_data.len());
                FIRST_SERIALIZE_LOG.store(true, Ordering::Relaxed);
            }

            let serialize_start = std::time::Instant::now();

            // Serialize using FeagiByteContainer
            // CRITICAL PERFORMANCE: Move raw_fire_data to avoid cloning vectors
            let serialized = match Self::serialize_fire_queue(item.raw_fire_data) {
                Ok(bytes) => bytes,
                Err(e) => {
                    error!("[ZMQ-VIZ] ❌ Serialization failed: {}", e);
                    stats.record_drop();
                    return;
                }
            };

            let serialize_duration = serialize_start.elapsed();
            if total_neurons > 1000 {
                info!(
                    "[ZMQ-VIZ] ⏱️ SERIALIZE TIME: {} neurons → {} bytes in {:?}",
                    total_neurons,
                    serialized.len(),
                    serialize_duration
                );
            }

            serialized
        };

        // Step 2: Compress (on PNS worker thread)
        let compressed = match lz4::block::compress(
            &payload,
            Some(lz4::block::CompressionMode::FAST(1)),
            true,
        ) {
            Ok(c) => c,
            Err(e) => {
                error!("[ZMQ-VIZ] ❌ LZ4 compression failed: {:?}", e);
                stats.record_drop();
                return;
            }
        };

        // Step 3: Send via ZMQ
        let mut guard = socket.lock();
        let sock = match guard.as_mut() {
            Some(sock) => sock,
            None => {
                stats.record_drop();
                return;
            }
        };

        loop {
            let mut message = ZmqMessage::from(compressed.clone());
            message.prepend(&ZmqMessage::from(item.topic.clone()));

            let send_result = if send_timeout_ms < 0 {
                block_on_runtime(runtime.as_ref(), sock.send(message))
            } else {
                match block_on_runtime(runtime.as_ref(), async {
                    timeout(
                        Duration::from_millis(send_timeout_ms as u64),
                        sock.send(message),
                    )
                    .await
                }) {
                    Ok(result) => result,
                    Err(_) => Err(ZmqError::BufferFull("Send timeout")),
                }
            };

            match send_result {
                Ok(()) => {
                    static SEND_COUNTER: AtomicU64 = AtomicU64::new(0);
                    let count = SEND_COUNTER.fetch_add(1, Ordering::Relaxed);
                    if count.is_multiple_of(30) {
                        debug!(
                            "[ZMQ-VIZ] 📊 SENT #{}: {} bytes (compressed)",
                            count,
                            compressed.len()
                        );
                    }
                    break;
                }
                Err(ZmqError::BufferFull(_)) => {
                    let waits = stats.record_backpressure_wait();
                    if waits == 1 || waits.is_multiple_of(100) {
                        warn!(
                            "⚠️  [ZMQ-VIZ] Send backpressure from ZMQ socket (waits: {})",
                            waits
                        );
                        warn!(
                            "[ZMQ-VIZ] send loop snapshot: waits={} drops={} failures={}",
                            waits,
                            stats.dropped.load(Ordering::Relaxed),
                            stats.send_failures.load(Ordering::Relaxed)
                        );
                    }
                    if shutdown.load(Ordering::Relaxed) {
                        let drops = stats.record_drop();
                        warn!(
                            "⚠️  [ZMQ-VIZ] Shutdown during send - dropping frame ({} drops)",
                            drops
                        );
                        warn!(
                            "[ZMQ-VIZ] send loop exit due to shutdown (drops={} waits={})",
                            drops, waits
                        );
                        return;
                    }
                    thread::sleep(retry_sleep);
                }
                Err(other) => {
                    error!("❌ [ZMQ-VIZ] Payload send failed: {}", other);
                    stats.record_send_failure();
                    warn!(
                        "[ZMQ-VIZ] send failure snapshot: drops={} failures={} waits={}",
                        stats.dropped.load(Ordering::Relaxed),
                        stats.send_failures.load(Ordering::Relaxed),
                        stats.backpressure_waits.load(Ordering::Relaxed)
                    );
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
        let runtime = Arc::new(Runtime::new().unwrap());
        let stream = VisualizationStream::new(
            runtime,
            "tcp://127.0.0.1:30010",
            VisualizationSendConfig::default(),
        );
        assert!(stream.is_ok());
    }
}
