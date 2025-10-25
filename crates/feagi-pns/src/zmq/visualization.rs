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
        let sock_guard = self.socket.lock();
        let sock = match sock_guard.as_ref() {
            Some(s) => s,
            None => {
                return Err("Visualization stream not started".to_string())
            }
        };

        // TEMPORARY: LZ4 compression disabled due to BV Rust crash
        // TODO: Re-enable once BV LZ4 decompression is debugged
        // Sending uncompressed data for now
        
        // Use NON-BLOCKING send to prevent burst loop from freezing during high neuron activity
        // With tens of thousands of neurons firing, the buffer can fill and blocking would freeze the burst loop
        match sock.send(data, zmq::DONTWAIT) {
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
                            "⚠️  [ZMQ-VIZ] WARNING: Send buffer full - dropped {} frames in last second ({} bytes/frame)",
                            drops, data.len()
                        );
                        eprintln!("⚠️  [ZMQ-VIZ] Subscribers too slow! Consider: (1) Increase rcv_hwm on Bridge, (2) Reduce burst rate");
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

