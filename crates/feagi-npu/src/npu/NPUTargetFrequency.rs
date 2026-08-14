
/// Used to represent how many bursts per second we want NPU to run
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct NPUTargetFrequency {
    duration: core::time::Duration
}

impl NPUTargetFrequency {
    
    /// Create from the time (seconds) between bursts
    pub fn new_from_time_between_bursts(seconds: f64) -> Self {
        Self {duration: core::time::Duration::from_secs_f64(seconds)}
    }
    
    /// Create from the duration between bursts
    pub fn new_from_duration_between_bursts(duration: core::time::Duration) -> Self {
        Self {duration}
    }
    
    /// Create from the number of bursts per second
    pub fn new_from_frequency(hz: f64) -> Self {
        Self {duration: core::time::Duration::from_secs_f64(1.0 / hz)}
    }
    
    /// get duration
    pub fn duration_between_bursts(&self) -> core::time::Duration {
        self.duration
    }
    
    /// Get burst frequency. Note that due to floating point error, this may drift from any input
    /// frequency!
    pub fn burst_frequency(&self) -> f64 {
        1.0 / self.duration_between_bursts().as_secs_f64()
    }
}