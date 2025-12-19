// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// Visualization stream for sending neuron activity to Brain Visualizer (ZMQ fallback for remote clients)
// Uses PUB socket pattern for one-to-many distribution with an asynchronous sender to avoid frame loss.

use crossbeam::queue::ArrayQueue;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info, warn};

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
            stats: Arc::new(VisualizationQueueStats::default()),
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
        // REAL-TIME: HWM=1 ensures only latest visualization is kept
        // Brain Visualizer should show current activity, not buffered history
        socket.set_sndhwm(1).map_err(|e| e.to_string())?;
        // REAL-TIME: conflate=true enables "last value caching" - keeps only newest message
        // This is critical for PUB/SUB pattern to drop intermediate frames
        socket.set_conflate(true).map_err(|e| e.to_string())?;
        socket
            .set_sndtimeo(self.send_config.send_timeout_ms)
            .map_err(|e| e.to_string())?;

        socket.bind(&self.bind_address).map_err(|e| e.to_string())?;

        *self.socket.lock() = Some(socket);
        *self.running.lock() = true;
        self.shutdown.store(false, Ordering::Relaxed);

        self.spawn_worker();

        info!("🦀 [ZMQ-VIZ] Listening on {}", self.bind_address);

        Ok(())
    }

    /// Stop the visualization stream
    pub fn stop(&self) -> Result<(), String> {
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
    ) -> Result<(), String> {
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

                    if backpressure_count == 1 || backpressure_count % 100 == 0 {
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
        let shutdown = Arc::clone(&self.shutdown);
        let stats = Arc::clone(&self.stats);
        let idle_sleep = Duration::from_millis(self.send_config.idle_sleep_ms);
        let send_retry_sleep = Duration::from_millis(self.send_config.backpressure_sleep_ms);

        let handle = thread::Builder::new()
            .name("feagi-viz-sender".to_string())
            .spawn(move || {
                while !shutdown.load(Ordering::Relaxed) {
                    if let Some(item) = queue.pop() {
                        Self::send_item(&socket, item, &stats, &shutdown, send_retry_sleep);
                        continue;
                    }

                    thread::sleep(idle_sleep);
                }

                while let Some(item) = queue.pop() {
                    Self::send_item(&socket, item, &stats, &shutdown, send_retry_sleep);
                }
            })
            .expect("Failed to spawn visualization sender thread");

        *self.worker_thread.lock() = Some(handle);
    }

    /// Serialize raw fire queue data to FeagiByteContainer format
    /// This runs on the PNS worker thread, NOT the burst engine thread
    fn serialize_fire_queue(
        fire_data: &feagi_npu_burst_engine::RawFireQueueSnapshot,
    ) -> Result<Vec<u8>, String> {
        use feagi_data_serialization::FeagiByteContainer;
        use feagi_data_structures::genomic::cortical_area::CorticalID;
        use feagi_data_structures::neuron_voxels::xyzp::{
            CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZPArrays,
        };

        let mut cortical_mapped = CorticalMappedXYZPNeuronVoxels::new();

        for (_area_id, area_data) in fire_data {
            if area_data.neuron_ids.is_empty() {
                continue;
            }

            // Create CorticalID from area name
            let cortical_id =
                CorticalID::try_from_base_64(&area_data.cortical_area_name).map_err(|e| {
                    format!(
                        "Failed to decode CorticalID from base64 '{}': {:?}",
                        area_data.cortical_area_name, e
                    )
                })?;

            // Create neuron voxel arrays (cloning vectors since we're on a different thread)
            let neuron_arrays = NeuronVoxelXYZPArrays::new_from_vectors(
                area_data.coords_x.clone(),
                area_data.coords_y.clone(),
                area_data.coords_z.clone(),
                area_data.potentials.clone(),
            )
            .map_err(|e| format!("Failed to create neuron arrays: {:?}", e))?;

            cortical_mapped.insert(cortical_id, neuron_arrays);
        }

        // Serialize to FeagiByteContainer
        // Note: overwrite_byte_data_with_single_struct_data() already handles efficient allocation internally:
        // - It pre-calculates size via get_number_of_bytes_needed()
        // - Only resizes if current capacity is insufficient
        // - Reuses existing allocation when possible
        let mut byte_container = FeagiByteContainer::new_empty();
        byte_container
            .overwrite_byte_data_with_single_struct_data(&cortical_mapped, 0)
            .map_err(|e| format!("Failed to encode into FeagiByteContainer: {:?}", e))?;

        Ok(byte_container.get_byte_ref().to_vec())
    }

    fn send_item(
        socket: &Arc<Mutex<Option<zmq::Socket>>>,
        item: VisualizationQueueItem,
        stats: &VisualizationQueueStats,
        shutdown: &Arc<AtomicBool>,
        retry_sleep: Duration,
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
            let serialized = match Self::serialize_fire_queue(&item.raw_fire_data) {
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
            if let Err(e) = sock.send(&item.topic, zmq::SNDMORE) {
                error!("❌ [ZMQ-VIZ] Topic send failed: {}", e);
                stats.record_send_failure();
                return;
            }

            match sock.send(&compressed, 0) {
                Ok(()) => {
                    // DIAGNOSTIC: Track actual ZMQ send rate
                    static SEND_COUNTER: AtomicU64 = AtomicU64::new(0);
                    let count = SEND_COUNTER.fetch_add(1, Ordering::Relaxed);
                    if count % 30 == 0 {
                        // Log every 30 sends
                        debug!(
                            "[ZMQ-VIZ] 📊 SENT #{}: {} bytes (compressed)",
                            count,
                            compressed.len()
                        );
                    }
                    break;
                }
                Err(zmq::Error::EAGAIN) => {
                    let waits = stats.record_backpressure_wait();
                    if waits == 1 || waits % 100 == 0 {
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
        let ctx = Arc::new(zmq::Context::new());
        let stream = VisualizationStream::new(
            ctx,
            "tcp://127.0.0.1:30010",
            VisualizationSendConfig::default(),
        );
        assert!(stream.is_ok());
    }
}
