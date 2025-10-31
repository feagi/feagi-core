// Runtime DTOs for V1 API
//
// These DTOs must match Python FastAPI response structures exactly.

use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

/// Runtime status response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "is_running": true,
    "is_paused": false,
    "frequency_hz": 30.0,
    "burst_count": 12345,
    "current_rate_hz": 29.8,
    "last_burst_neuron_count": 45000,
    "avg_burst_time_ms": 15.2
}))]
pub struct RuntimeStatusResponse {
    /// Whether the burst engine is running
    pub is_running: bool,
    
    /// Whether the burst engine is paused
    pub is_paused: bool,
    
    /// Current burst frequency (Hz)
    pub frequency_hz: f64,
    
    /// Total burst count since start
    pub burst_count: u64,
    
    /// Current burst rate (bursts per second, measured)
    pub current_rate_hz: f64,
    
    /// Total neurons fired in last burst
    pub last_burst_neuron_count: usize,
    
    /// Average processing time per burst (milliseconds)
    pub avg_burst_time_ms: f64,
}

/// Set frequency request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SetFrequencyRequest {
    /// New burst frequency in Hz
    pub frequency_hz: f64,
}

/// Burst count response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BurstCountResponse {
    /// Total burst count
    pub burst_count: u64,
}



