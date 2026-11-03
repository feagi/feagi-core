use serde::{Deserialize, Serialize};

/// Used to represent how many ticks per second we want something to run
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TargetFrequency {
    duration: core::time::Duration
}

impl TargetFrequency {

    /// Create from the time (seconds) between ticks
    pub fn new_from_time_between_ticks(seconds: f64) -> Self {
        Self {duration: core::time::Duration::from_secs_f64(seconds)}
    }

    /// Create from the duration between ticks
    pub fn new_from_duration_between_ticks(duration: core::time::Duration) -> Self {
        Self {duration}
    }

    /// Create from the number of ticks per second
    pub fn new_from_frequency(hz: f64) -> Self {
        Self {duration: core::time::Duration::from_secs_f64(1.0 / hz)}
    }

    /// get duration
    pub fn duration_between_ticks(&self) -> core::time::Duration {
        self.duration
    }

    /// Get tick frequency. Note that due to floating point error, this may drift from any input
    /// frequency!
    pub fn tick_frequency(&self) -> f64 {
        1.0 / self.duration_between_ticks().as_secs_f64()
    }
}