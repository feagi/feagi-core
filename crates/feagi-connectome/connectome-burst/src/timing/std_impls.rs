//! Time implementations that require STD
use feagi_basis::target_frequency::TargetFrequency;
use std::time::{Duration, Instant};



/// Holds data in regards to certain timings
pub(crate) struct NPUTimings {
    burst_frequency: TargetFrequency,
    visualization_frequency: TargetFrequency,
    /// While paused, no bursts run and the worker only waits for commands
    paused: bool,
    when_next_burst: Instant,
    when_next_visualization: Instant

}

impl NPUTimings {

    pub fn new(burst_frequency: TargetFrequency, visualization_frequency: TargetFrequency, paused: bool) -> Self {
        let when_next_burst = Instant::now() + burst_frequency.duration_between_ticks();
        let when_next_visualization = Instant::now() + visualization_frequency.duration_between_ticks();
        Self {
            burst_frequency,
            visualization_frequency,
            paused,
            when_next_burst,
            when_next_visualization
        }
    }

    pub fn set_burst_frequency(&mut self, frequency: TargetFrequency) {
        if let Some(next) = frequency.duration_between_ticks().checked_sub(self.burst_frequency.duration_between_ticks()) {
            self.when_next_burst = Instant::now() + next;
        }
        self.burst_frequency = frequency;
    }

    pub fn set_visualization_frequency(&mut self, frequency: TargetFrequency) {
        if let Some(next) = frequency.duration_between_ticks().checked_sub(self.visualization_frequency.duration_between_ticks()) {
            self.when_next_visualization = Instant::now() + next;
        }
        self.visualization_frequency = frequency;
    }

    pub fn set_paused(&mut self, pause: bool) {
        if self.paused == pause {return}
        if !pause {
            self.when_next_burst = Instant::now();
            self.when_next_visualization = Instant::now();
        }
        self.paused = pause
    }

    /// Awaits until next burst. Returns bool fo if should visualize.
    pub async fn wait_until_burst(&mut self) -> bool {
        wait_until(self.when_next_burst).await;
        self.when_next_burst += self.burst_frequency.duration_between_ticks();
        if Instant::now() < self.when_next_visualization {
            self.when_next_visualization += self.visualization_frequency.duration_between_ticks();
            return true;
        }
        return false;
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
