// Visualization stream for sending neuron activity to Brain Visualizer (ZMQ fallback for remote clients)
// Uses PUB socket pattern for one-to-many distribution

use parking_lot::Mutex;
use std::sync::Arc;

/// Visualization stream for publishing neuron activity
#[derive(Clone)]
pub struct VisualizationStream {
    context: Arc<zmq::Context>,
    bind_address: String,
    socket: Arc<Mutex<Option<zmq::Socket>>>,
    running: Arc<Mutex<bool>>,
}

impl VisualizationStream {
    /// Create a new visualization stream
    pub fn new(context: Arc<zmq::Context>, bind_address: &str) -> Result<Self, String> {
        Ok(Self {
            context,
            bind_address: bind_address.to_string(),
            socket: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
        })
    }

    /// Start the visualization stream
    pub fn start(&self) -> Result<(), String> {
        if *self.running.lock() {
            return Err("Visualization stream already running".to_string());
        }

        // Create PUB socket for broadcasting visualization data
        let socket = self
            .context
            .socket(zmq::PUB)
            .map_err(|e| e.to_string())?;

        // Set socket options for optimal performance
        socket
            .set_linger(0) // Don't wait on close
            .map_err(|e| e.to_string())?;
        socket
            .set_sndhwm(10000) // Increased from 1000 to 10000 for large neuron firing bursts (tens of thousands per burst)
            .map_err(|e| e.to_string())?;
        socket
            .set_conflate(false) // CRITICAL: Disabled conflate - was dropping neuron activations with large bursts! Must preserve all firing data.
            .map_err(|e| e.to_string())?;

        // Bind socket
        socket
            .bind(&self.bind_address)
            .map_err(|e| e.to_string())?;

        *self.socket.lock() = Some(socket);
        *self.running.lock() = true;

        println!("🦀 [ZMQ-VIZ] Listening on {}", self.bind_address);

        Ok(())
    }

    /// Stop the visualization stream
    pub fn stop(&self) -> Result<(), String> {
        *self.running.lock() = false;
        *self.socket.lock() = None;
        Ok(())
    }

    /// Publish visualization data to all subscribers (with LZ4 compression)
    pub fn publish(&self, data: &[u8]) -> Result<(), String> {
        static FIRST_LOG: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !FIRST_LOG.load(std::sync::atomic::Ordering::Relaxed) {
            eprintln!("[VIZ-STREAM] 🔍 TRACE: publish() called with {} bytes (BEFORE compression)", data.len());
            FIRST_LOG.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        
        let sock_guard = self.socket.lock();
        let sock = match sock_guard.as_ref() {
            Some(s) => s,
            None => {
                eprintln!("[VIZ-STREAM] ❌ CRITICAL: Socket not initialized!");
                return Err("Visualization stream not started".to_string())
            }
        };

        // Compress with LZ4 before sending (PNS responsibility, not burst engine)
        // NO FALLBACK: Compression must succeed or fail
        static COMPRESS_CALL_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let call_num = COMPRESS_CALL_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        if call_num == 0 {
            let mut full_input = String::new();
            for (i, byte) in data.iter().enumerate() {
                if i > 0 { full_input.push(' '); }
                full_input.push_str(&format!("{:02x}", byte));
            }
            eprintln!("\n=== [INPUT-FULL] {} bytes ===", data.len());
            eprintln!("{}", full_input);
            eprintln!("=== END INPUT ===\n");
        }
        
        eprintln!("[VIZ-STREAM] 🔍 TRACE #{}: About to call lz4::block::compress() on {} bytes...", call_num, data.len());
        
        // 🔍 DEBUG: Try FAST compression without size header (store_size=false is WRONG for our use case!)
        let compressed = match lz4::block::compress(data, Some(lz4::block::CompressionMode::FAST(1)), true) {
            Ok(c) => {
                eprintln!("[VIZ-STREAM] ✅ TRACE #{}: LZ4 SUCCESS! {} → {} bytes", call_num, data.len(), c.len());
                static FIRST_SUCCESS_LOG: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
                if !FIRST_SUCCESS_LOG.load(std::sync::atomic::Ordering::Relaxed) {
                    let ratio = (1.0 - c.len() as f64 / data.len() as f64) * 100.0;
                    eprintln!("[ZMQ-VIZ] 🗜️  LZ4 compression: {} → {} bytes ({:.1}% reduction)",
                        data.len(), c.len(), ratio);
                    FIRST_SUCCESS_LOG.store(true, std::sync::atomic::Ordering::Relaxed);
                }
                c
            }
            Err(e) => {
                eprintln!("[VIZ-STREAM] ❌ CRITICAL #{}: LZ4 FAILED: {:?}", call_num, e);
                return Err(format!("LZ4 compression failed: {}", e));
            }
        };

            // 🔍 CRITICAL: Dump FULL compressed buffer
            static FIRST_SEND_LOG: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
            if !FIRST_SEND_LOG.load(std::sync::atomic::Ordering::Relaxed) {
                let mut full_hex = String::new();
                for (i, byte) in compressed.iter().enumerate() {
                    if i > 0 { full_hex.push(' '); }
                    full_hex.push_str(&format!("{:02x}", byte));
                }
                eprintln!("\n=== [COMPRESSED-FULL] {} bytes ===", compressed.len());
                eprintln!("{}", full_hex);
                eprintln!("=== END ===\n");
                FIRST_SEND_LOG.store(true, std::sync::atomic::Ordering::Relaxed);
            }
            
            // Use NON-BLOCKING send with explicit topic (PUB/SUB pattern requires topic prefix)
            // Send as multipart: [topic, data]
            let result = sock.send(&b"activity"[..], zmq::SNDMORE)
                .and_then(|_| sock.send(&compressed[..], zmq::DONTWAIT));
            match result {
            Ok(_) => Ok(()),
            Err(zmq::Error::EAGAIN) => {
                // Buffer full - frame will be dropped but burst loop continues
                // This is better than blocking the entire burst loop
                use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
                static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);
                static LAST_WARNING_SECS: AtomicU64 = AtomicU64::new(0);
                
                let drops = DROP_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
                let now_secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                
                let last_warn = LAST_WARNING_SECS.load(Ordering::Relaxed);
                
                // Rate-limit warnings to once per second
                if now_secs > last_warn {
                    if LAST_WARNING_SECS.compare_exchange(last_warn, now_secs, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
                        eprintln!(
                            "⚠️  [ZMQ-VIZ] WARNING: Send buffer full - dropped {} frames in last second ({} bytes/frame compressed)",
                            drops, compressed.len()
                        );
                        eprintln!("⚠️  [ZMQ-VIZ] Subscribers too slow even with LZ4! Consider: (1) Increase rcv_hwm on Bridge, (2) Reduce burst rate");
                        DROP_COUNT.store(0, Ordering::Relaxed);
                    }
                }
                
                // Return Ok to prevent error propagation - dropping frames is acceptable
                Ok(())
            }
            Err(e) => {
                // Real error (not just buffer full)
                eprintln!("❌ [ZMQ-VIZ] Send failed: {}", e);
                Err(e.to_string())
            }
        }
    }

    /// Check if stream is running
    pub fn is_running(&self) -> bool {
        *self.running.lock()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viz_stream_creation() {
        let ctx = Arc::new(zmq::Context::new());
        let stream = VisualizationStream::new(ctx, "tcp://127.0.0.1:30010");
        assert!(stream.is_ok());
    }
}

