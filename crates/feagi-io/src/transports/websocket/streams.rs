// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WebSocket stream implementations for PNS

use crate::blocking::compression;
use crate::core::{IOConfig, IOError, Result};
use crate::transports::core::common::ServerConfig;
use crate::transports::core::prelude::*;
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tokio::runtime::Runtime;
use tracing::{error, info};

/// WebSocket streams manager
///
/// Manages all WebSocket-based streams for FEAGI:
/// - Sensory input (agents → FEAGI)
/// - Motor output (FEAGI → agents)
/// - Visualization output (FEAGI → clients)
/// - Registration/control (bidirectional)
pub struct WebSocketStreams {
    config: IOConfig,
    running: Arc<Mutex<bool>>,

    // WebSocket servers
    viz_pub: Arc<Mutex<Option<WsPub>>>,
    motor_pub: Arc<Mutex<Option<WsPub>>>,
    sensory_pull: Arc<Mutex<Option<WsPull>>>,

    // Async runtime for WebSocket servers (reserved for future use)
    #[allow(dead_code)]
    runtime: Arc<Mutex<Option<Arc<Runtime>>>>,
}

impl WebSocketStreams {
    /// Create new WebSocket streams manager
    pub fn new(config: IOConfig) -> Result<Self> {
        if !config.websocket.enabled {
            return Err(IOError::Config(
                "WebSocket transport not enabled in configuration".to_string(),
            ));
        }

        info!(
            "🦀 [WS-STREAMS] Initializing WebSocket transport (host: {}, sensory: {}, motor: {}, viz: {})",
            config.websocket.host,
            config.websocket.sensory_port,
            config.websocket.motor_port,
            config.websocket.visualization_port
        );

        // Create tokio runtime for async WebSocket servers
        let runtime = Runtime::new()
            .map_err(|e| IOError::Transport(format!("Failed to create tokio runtime: {}", e)))?;

        Ok(Self {
            config,
            running: Arc::new(Mutex::new(false)),
            viz_pub: Arc::new(Mutex::new(None)),
            motor_pub: Arc::new(Mutex::new(None)),
            sensory_pull: Arc::new(Mutex::new(None)),
            runtime: Arc::new(Mutex::new(Some(Arc::new(runtime)))),
        })
    }

    /// Start control streams (registration) - safe before burst engine
    pub fn start_control_streams(&self) -> Result<()> {
        info!("🦀 [WS-STREAMS] Starting WebSocket control streams...");
        // TODO: Start WsRouter for registration/control
        // For now, control happens via ZMQ REST stream
        info!("🦀 [WS-STREAMS] ✅ Control streams ready (using ZMQ for now)");
        Ok(())
    }

    /// Start data streams (sensory/motor/viz) - requires burst engine running
    ///
    /// NOTE: This can be called from within an async context (e.g., Axum HTTP handler)
    /// We spawn the servers on the existing tokio runtime and return immediately
    pub fn start_data_streams(&self) -> Result<()> {
        if *self.running.lock() {
            return Err(IOError::Transport(
                "WebSocket data streams already running".to_string(),
            ));
        }

        info!("🦀 [WS-STREAMS] Starting WebSocket data streams...");

        // Get handle to the existing tokio runtime (Axum's runtime)
        let handle = tokio::runtime::Handle::try_current()
            .map_err(|e| IOError::Transport(format!("No tokio runtime available: {}", e)))?;

        // Clone config for async closures
        let viz_host = self.config.websocket.host.clone();
        let viz_port = self.config.websocket.visualization_port;
        let motor_host = self.config.websocket.host.clone();
        let motor_port = self.config.websocket.motor_port;
        let sensory_host = self.config.websocket.host.clone();
        let sensory_port = self.config.websocket.sensory_port;

        let viz_pub = self.viz_pub.clone();
        let motor_pub = self.motor_pub.clone();
        let sensory_pull = self.sensory_pull.clone();

        // Spawn server creation on the tokio runtime
        handle.spawn(async move {
            // Start visualization publisher
            let viz_addr = format!("{}:{}", viz_host, viz_port);
            let config = ServerConfig::new(&viz_addr);
            match WsPub::new(config) {
                Ok(mut pub_server) => match pub_server.start_async().await {
                    Ok(()) => {
                        info!(
                            "🦀 [WS-STREAMS] ✅ Visualization publisher started on ws://{}:{}",
                            viz_host, viz_port
                        );
                        *viz_pub.lock() = Some(pub_server);
                    }
                    Err(e) => {
                        error!("❌ [WS-STREAMS] Failed to start viz publisher: {}", e);
                    }
                },
                Err(e) => {
                    error!("❌ [WS-STREAMS] Failed to create viz publisher: {}", e);
                }
            }
        });

        handle.spawn(async move {
            // Start motor publisher
            let motor_addr = format!("{}:{}", motor_host, motor_port);
            let config = ServerConfig::new(&motor_addr);
            match WsPub::new(config) {
                Ok(mut pub_server) => match pub_server.start_async().await {
                    Ok(()) => {
                        info!(
                            "🦀 [WS-STREAMS] ✅ Motor publisher started on ws://{}:{}",
                            motor_host, motor_port
                        );
                        *motor_pub.lock() = Some(pub_server);
                    }
                    Err(e) => {
                        error!("❌ [WS-STREAMS] Failed to start motor publisher: {}", e);
                    }
                },
                Err(e) => {
                    error!("❌ [WS-STREAMS] Failed to create motor publisher: {}", e);
                }
            }
        });

        handle.spawn(async move {
            // Start sensory pull
            let sensory_addr = format!("{}:{}", sensory_host, sensory_port);
            let config = ServerConfig::new(&sensory_addr);
            match WsPull::new(config) {
                Ok(mut pull_server) => match pull_server.start_async().await {
                    Ok(()) => {
                        info!(
                            "🦀 [WS-STREAMS] ✅ Sensory pull started on ws://{}:{}",
                            sensory_host, sensory_port
                        );
                        *sensory_pull.lock() = Some(pull_server);
                    }
                    Err(e) => {
                        error!("❌ [WS-STREAMS] Failed to start sensory pull: {}", e);
                    }
                },
                Err(e) => {
                    error!("❌ [WS-STREAMS] Failed to create sensory pull: {}", e);
                }
            }
        });

        // Give the async servers a moment to bind
        std::thread::sleep(std::time::Duration::from_millis(200));

        *self.running.lock() = true;

        info!("🦀 [WS-STREAMS] ✅ WebSocket data streams startup initiated");
        info!(
            "🦀 [WS-STREAMS]    🌐 Brain Visualizer can connect to: ws://{}:{}",
            self.config.websocket.host, self.config.websocket.visualization_port
        );
        info!(
            "🦀 [WS-STREAMS]    📥 Agents can send sensory to: ws://{}:{}",
            self.config.websocket.host, self.config.websocket.sensory_port
        );
        info!(
            "🦀 [WS-STREAMS]    📤 Agents can receive motor from: ws://{}:{}",
            self.config.websocket.host, self.config.websocket.motor_port
        );

        Ok(())
    }

    /// Stop all WebSocket streams
    pub fn stop(&self) -> Result<()> {
        if !*self.running.lock() {
            return Ok(());
        }

        info!("🦀 [WS-STREAMS] Stopping WebSocket servers...");

        // Stop all servers
        if let Some(mut viz) = self.viz_pub.lock().take() {
            viz.stop().ok();
        }
        if let Some(mut motor) = self.motor_pub.lock().take() {
            motor.stop().ok();
        }
        if let Some(mut sensory) = self.sensory_pull.lock().take() {
            sensory.stop().ok();
        }

        *self.running.lock() = false;

        info!("🦀 [WS-STREAMS] ✅ WebSocket transport stopped");

        Ok(())
    }

    /// Check if WebSocket streams are running
    pub fn is_running(&self) -> bool {
        *self.running.lock()
    }

    /// Publish raw fire queue data from burst engine (matches ZMQ API)
    pub fn publish_raw_fire_queue(
        &self,
        fire_data: feagi_npu_burst_engine::RawFireQueueSnapshot,
    ) -> Result<()> {
        // Diagnostics: time serialize + compress to detect a publish-path slowdown that
        // can cause client-side drift if messages queue up downstream.
        static PUBLISH_COUNT: AtomicU64 = AtomicU64::new(0);
        let publish_idx = PUBLISH_COUNT.fetch_add(1, Ordering::Relaxed) + 1;

        let t0 = Instant::now();

        // Serialize the fire queue data to FeagiByteContainer format (same as ZMQ)
        // CRITICAL PERFORMANCE: Take ownership to avoid cloning vectors (moves them instead)
        let serialized = Self::serialize_fire_queue(fire_data)
            .map_err(|e| IOError::Transport(format!("Failed to serialize fire queue: {}", e)))?;
        let serialize_ms = t0.elapsed().as_secs_f64() * 1000.0;

        // ✅ Compress with LZ4 (BV expects LZ4-compressed msgpack, same as ZMQ)
        let t1 = Instant::now();
        let compressed = compression::compress_lz4(&serialized)?;
        let compress_ms = t1.elapsed().as_secs_f64() * 1000.0;

        if publish_idx == 1 || publish_idx % 300 == 0 {
            info!(
                "[WS-VIZ] publish_idx={} serialize_ms={:.2} compress_ms={:.2} bytes_raw={} bytes_lz4={}",
                publish_idx,
                serialize_ms,
                compress_ms,
                serialized.len(),
                compressed.len()
            );
        }

        // Publish compressed data to WebSocket clients
        self.publish_visualization(&compressed)
    }

    /// Serialize raw fire queue data to FeagiByteContainer format
    /// Same logic as ZMQ visualization stream
    /// 
    /// CRITICAL PERFORMANCE: Takes ownership of fire_data to move vectors instead of cloning
    fn serialize_fire_queue(
        fire_data: feagi_npu_burst_engine::RawFireQueueSnapshot,
    ) -> std::result::Result<Vec<u8>, String> {
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
                    format!(
                        "Failed to decode CorticalID from base64 '{}': {:?}",
                        area_data.cortical_id, e
                    )
                })?;

            // Create neuron voxel arrays - MOVE vectors instead of cloning (takes ownership)
            // This eliminates expensive cloning for large areas
            let neuron_arrays = NeuronVoxelXYZPArrays::new_from_vectors(
                area_data.coords_x,   // Move instead of clone
                area_data.coords_y,   // Move instead of clone
                area_data.coords_z,   // Move instead of clone
                area_data.potentials, // Move instead of clone
            )
            .map_err(|e| format!("Failed to create neuron arrays: {:?}", e))?;

            cortical_mapped.insert(cortical_id, neuron_arrays);
        }

        // Serialize to FeagiByteContainer
        let mut byte_container = FeagiByteContainer::new_empty();
        byte_container
            .overwrite_byte_data_with_single_struct_data(&cortical_mapped, 0)
            .map_err(|e| format!("Failed to encode into FeagiByteContainer: {:?}", e))?;

        Ok(byte_container.get_byte_ref().to_vec())
    }

    /// Publish visualization data to WebSocket clients
    pub fn publish_visualization(&self, data: &[u8]) -> Result<()> {
        let viz_pub = self.viz_pub.lock();
        if let Some(pub_server) = viz_pub.as_ref() {
            // CRITICAL: publish_simple is non-blocking (uses broadcast channel)
            // If it fails, it's likely because the server isn't running, not because of blocking
            pub_server
                .publish_simple(data)
                .map_err(|e| {
                    let error_msg = format!("WebSocket viz publish failed: {}", e);
                    error!("[WS-VIZ] {}", error_msg);
                    IOError::Transport(error_msg)
                })?;
            Ok(())
        } else {
            let error_msg = "Visualization publisher not started".to_string();
            error!("[WS-VIZ] {}", error_msg);
            Err(IOError::Transport(error_msg))
        }
    }

    /// Publish motor data to WebSocket clients
    pub fn publish_motor(&self, agent_id: &str, data: &[u8]) -> Result<()> {
        let motor_pub = self.motor_pub.lock();
        if let Some(pub_server) = motor_pub.as_ref() {
            // Use topic for agent-specific delivery
            pub_server.publish(agent_id.as_bytes(), data).map_err(|e| {
                IOError::Transport(format!("WebSocket motor publish failed: {}", e))
            })?;
            Ok(())
        } else {
            Err(IOError::Transport(
                "Motor publisher not started".to_string(),
            ))
        }
    }

    /// Pull sensory data from WebSocket clients
    pub fn pull_sensory(&self) -> Result<Option<Vec<u8>>> {
        let sensory_pull = self.sensory_pull.lock();
        if let Some(pull_server) = sensory_pull.as_ref() {
            match pull_server.pull() {
                Ok(data) => Ok(Some(data)),
                Err(TransportError::NoData) => Ok(None),
                Err(TransportError::Timeout) => Ok(None),
                Err(e) => Err(IOError::Transport(format!(
                    "WebSocket sensory pull failed: {}",
                    e
                ))),
            }
        } else {
            Err(IOError::Transport("Sensory pull not started".to_string()))
        }
    }

    /// Start sensory stream only (dynamic gating)
    pub fn start_sensory_stream(&self) -> Result<()> {
        info!(
            "🦀 [WS-STREAMS] Sensory stream on port {} (already started in start_data_streams)",
            self.config.websocket.sensory_port
        );
        Ok(())
    }

    /// Stop sensory stream only (dynamic gating)
    pub fn stop_sensory_stream(&self) -> Result<()> {
        info!("🦀 [WS-STREAMS] Sensory stream stop requested (WebSocket stays active)");
        Ok(())
    }

    /// Start motor stream only (dynamic gating)
    pub fn start_motor_stream(&self) -> Result<()> {
        info!(
            "🦀 [WS-STREAMS] Motor stream on port {} (already started in start_data_streams)",
            self.config.websocket.motor_port
        );
        Ok(())
    }

    /// Stop motor stream only (dynamic gating)
    pub fn stop_motor_stream(&self) -> Result<()> {
        info!("🦀 [WS-STREAMS] Motor stream stop requested (WebSocket stays active)");
        Ok(())
    }

    /// Start visualization stream only (dynamic gating)
    pub fn start_viz_stream(&self) -> Result<()> {
        info!(
            "🦀 [WS-STREAMS] Viz stream on port {} (already started in start_data_streams)",
            self.config.websocket.visualization_port
        );
        Ok(())
    }

    /// Stop visualization stream only (dynamic gating)
    pub fn stop_viz_stream(&self) -> Result<()> {
        info!("🦀 [WS-STREAMS] Viz stream stop requested (WebSocket stays active)");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::WebSocketConfig;

    #[test]
    fn test_websocket_streams_creation() {
        let mut config = IOConfig::default();
        config.websocket.enabled = true;

        let streams = WebSocketStreams::new(config);
        assert!(streams.is_ok());
    }

    #[test]
    fn test_websocket_disabled() {
        let mut config = IOConfig::default();
        config.websocket.enabled = false;

        let streams = WebSocketStreams::new(config);
        assert!(streams.is_err());
    }
}
