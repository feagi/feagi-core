//! Time implementations that require STD
use feagi_basis::target_frequency::TargetFrequency;
use std::time::{Duration, Instant};



/// The burst loop's schedule
pub(crate) struct NPUTimings {
    when_next_burst: Instant,
    when_next_visualization: Instant
}

impl NPUTimings {

    /// The first burst runs immediately and sends visualization
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            when_next_burst: now,
            when_next_visualization: now
        }
    }

    /// Schedules the next burst one period after the previous one, so the rate does not drift. If
    /// the previous tick ran over, schedules from now instead of bursting to catch up
    pub fn schedule_next_burst(&mut self, burst_frequency: TargetFrequency) {
        self.when_next_burst = (
            self.when_next_burst
                + burst_frequency.duration_between_ticks()).max(Instant::now());
    }

    /// Schedules the next burst one period from now. For after a pause, where catching up makes
    /// doesnt sense
    pub fn schedule_next_burst_from_now(&mut self, burst_frequency: TargetFrequency) {
        self.when_next_burst = Instant::now() + burst_frequency.duration_between_ticks();
    }

    /// Returns true if visualization is due, and if so schedules the next one the same way as
    /// bursts. Only checked once per burst, so it never runs faster than the burst rate
    pub fn take_visualization_due(&mut self, visualization_frequency: TargetFrequency) -> bool {
        let now = Instant::now();
        if now < self.when_next_visualization {
            return false;
        }
        self.when_next_visualization = (self.when_next_visualization + visualization_frequency.duration_between_ticks()).max(now);
        true
    }

    /// Waits until the scheduled burst
    pub async fn wait_for_next_burst(&self) {
        wait_until(self.when_next_burst).await;
    }

}

// TODO use Futures to delay instead, we dont want to std sleep in async (even though currently its fine)
// TODO spin_sleep may also work

/// How long before a tick `wait_until` stops sleeping and starts spinning. Uses normal
/// OS sleep but swaps to hot-spin when the timing gets close
const SPIN_MARGIN: Duration = Duration::from_millis(18); // cursory search shows windows cannot sleep shorter than this

/// Waits until `deadline` by sleeping until `SPIN_MARGIN` before it, then spinning the rest. The
/// worker has its own thread, so this blocks that thread
async fn wait_until(deadline: Instant) {
    if let Some(sleep_for) = deadline
        .checked_duration_since(Instant::now())
        .and_then(|remaining| remaining.checked_sub(SPIN_MARGIN))
    {
        std::thread::sleep(sleep_for);
    }
    while Instant::now() < deadline {
        core::hint::spin_loop();
    }
}
