use crate::npu::npu_target_frequency::NPUTargetFrequency;
use std::time::Duration;

/// Used to calculate timeout times
pub struct BurstEngineTimeoutLogic {
    pub minimum_time: Duration,
    pub burst_length_multiplier: u32,
}

impl BurstEngineTimeoutLogic {
    /// Calculates the timeout time, picking the longer between a set minimum time and multiplier of the burst length
    pub fn calculate_timeout(&self, frequency: NPUTargetFrequency) -> Duration {
        let multiplied_duration = frequency.duration_between_bursts() * self.burst_length_multiplier;
        core::cmp::min(multiplied_duration, self.minimum_time)
    }
}

impl Default for BurstEngineTimeoutLogic {
    fn default() -> Self {
        Self {
            minimum_time: Duration::from_secs(10),
            burst_length_multiplier: 10
        }
    }
}
