use core::time::Duration;
use serde::{Deserialize, Serialize};

/// Used to represent how many ticks per second we want something to run. The time between ticks
/// is always greater than zero, so its frequency is always finite, and at most
/// [`Self::MAX_DURATION_BETWEEN_TICKS`]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "TargetFrequencyUnchecked")]
pub struct TargetFrequency {
    duration: Duration
}

impl TargetFrequency {

    /// The longest allowed time between ticks, 24 hours. Keeps scheduling the next tick from ever
    /// overflowing
    pub const MAX_DURATION_BETWEEN_TICKS: Duration = Duration::from_secs(24 * 60 * 60);

    /// Create from the time (seconds) between ticks
    ///
    /// Errors if `seconds` is not finite, not greater than zero, rounds to no time at all, or is
    /// longer than [`Self::MAX_DURATION_BETWEEN_TICKS`]
    pub fn new_from_time_between_ticks(seconds: f64) -> Result<Self, TargetFrequencyError> {
        if !seconds.is_finite() {
            return Err(TargetFrequencyError::NotFinite(seconds));
        }
        if seconds <= 0.0 {
            return Err(TargetFrequencyError::NotPositive(seconds));
        }
        let duration = Duration::try_from_secs_f64(seconds).map_err(|_| TargetFrequencyError::TooLong)?;
        Self::new_from_duration_between_ticks(duration)
    }

    /// Create from the duration between ticks
    ///
    /// Errors if `duration` is zero or longer than [`Self::MAX_DURATION_BETWEEN_TICKS`]
    pub fn new_from_duration_between_ticks(duration: Duration) -> Result<Self, TargetFrequencyError> {
        if duration.is_zero() {
            return Err(TargetFrequencyError::ZeroDuration);
        }
        if duration > Self::MAX_DURATION_BETWEEN_TICKS {
            return Err(TargetFrequencyError::TooLong);
        }
        Ok(Self {duration})
    }

    /// Create from the number of ticks per second
    ///
    /// Errors if `hz` is not finite or not greater than zero, or if the time between ticks it
    /// gives rounds to no time at all or is longer than [`Self::MAX_DURATION_BETWEEN_TICKS`]
    pub fn new_from_frequency(hz: f64) -> Result<Self, TargetFrequencyError> {
        if !hz.is_finite() {
            return Err(TargetFrequencyError::NotFinite(hz));
        }
        if hz <= 0.0 {
            return Err(TargetFrequencyError::NotPositive(hz));
        }
        let seconds = 1.0 / hz;
        // A tiny enough frequency overflows to infinite seconds
        if !seconds.is_finite() {
            return Err(TargetFrequencyError::TooLong);
        }
        Self::new_from_time_between_ticks(seconds)
    }

    /// get duration
    pub fn duration_between_ticks(&self) -> Duration {
        self.duration
    }

    /// Get tick frequency. Note that due to floating point error, this may drift from any input
    /// frequency!
    pub fn tick_frequency(&self) -> f64 {
        1.0 / self.duration_between_ticks().as_secs_f64()
    }
}

impl TryFrom<Duration> for TargetFrequency {
    type Error = TargetFrequencyError;

    fn try_from(duration: Duration) -> Result<Self, Self::Error> {
        Self::new_from_duration_between_ticks(duration)
    }
}

/// The serialized form of `TargetFrequency`, checked on deserialization the same way as on
/// construction
#[derive(Deserialize)]
struct TargetFrequencyUnchecked {
    duration: Duration
}

impl TryFrom<TargetFrequencyUnchecked> for TargetFrequency {
    type Error = TargetFrequencyError;

    fn try_from(unchecked: TargetFrequencyUnchecked) -> Result<Self, Self::Error> {
        Self::new_from_duration_between_ticks(unchecked.duration)
    }
}

/// Why a `TargetFrequency` could not be created
#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum TargetFrequencyError {
    #[error("{0} is not a finite number")]
    NotFinite(f64),
    #[error("{0} is not greater than zero")]
    NotPositive(f64),
    #[error("The time between ticks rounds to zero")]
    ZeroDuration,
    #[error("The time between ticks is longer than 24 hours")]
    TooLong,
}
