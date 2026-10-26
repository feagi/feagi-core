// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! Burst-aligned log rate limiter.
//!
//! Used to keep repeating per-burst warnings from flooding logs. Decisions are
//! based on burst timestep, not wall-clock time, so behavior is deterministic.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Emits on first occurrence of a key, then at most once every `period_bursts`.
pub struct BurstLogRateLimiter {
    period_bursts: u64,
    last_emitted_timestep: HashMap<u32, u64>,
    suppressed_since_emit: HashMap<u32, u64>,
}

impl BurstLogRateLimiter {
    /// Create a limiter.
    ///
    /// `period_bursts` is the minimum burst gap between repeated emits for the
    /// same key. A value of `0` emits on every call (rate limiting disabled).
    pub fn new(period_bursts: u64) -> Self {
        Self {
            period_bursts,
            last_emitted_timestep: HashMap::new(),
            suppressed_since_emit: HashMap::new(),
        }
    }

    /// Returns `Some(suppressed_count)` when the caller should emit a log line.
    ///
    /// `suppressed_count` is `0` on the first emit for `key`. Later emits report
    /// how many calls were skipped since the previous emit.
    pub fn should_emit(&mut self, key: u32, current_timestep: u64) -> Option<u64> {
        if self.period_bursts == 0 {
            return Some(0);
        }

        match self.last_emitted_timestep.get(&key).copied() {
            None => {
                self.last_emitted_timestep.insert(key, current_timestep);
                self.suppressed_since_emit.insert(key, 0);
                Some(0)
            }
            Some(last) => {
                let suppressed = self.suppressed_since_emit.entry(key).or_insert(0);
                *suppressed = suppressed.saturating_add(1);
                if current_timestep.saturating_sub(last) >= self.period_bursts {
                    let count = *suppressed;
                    *suppressed = 0;
                    self.last_emitted_timestep.insert(key, current_timestep);
                    Some(count)
                } else {
                    None
                }
            }
        }
    }
}

/// Emits on first occurrence of a key, then at most once per wall-clock `period`.
///
/// Counterpart of [`BurstLogRateLimiter`] for events that are not tied to burst
/// numbers. The caller passes `now`, so tests control time.
pub struct TimeLogRateLimiter {
    period: Duration,
    last_emitted: HashMap<u32, Instant>,
    suppressed_since_emit: HashMap<u32, u64>,
}

impl TimeLogRateLimiter {
    /// `period` of zero emits on every call.
    pub fn new(period: Duration) -> Self {
        Self {
            period,
            last_emitted: HashMap::new(),
            suppressed_since_emit: HashMap::new(),
        }
    }

    /// Returns `Some(suppressed_count)` when the caller should emit a line. The count is
    /// how many calls were skipped since the previous emit for `key` (`0` on the first).
    pub fn should_emit(&mut self, key: u32, now: Instant) -> Option<u64> {
        if self.period.is_zero() {
            return Some(0);
        }
        match self.last_emitted.get(&key).copied() {
            None => {
                self.last_emitted.insert(key, now);
                self.suppressed_since_emit.insert(key, 0);
                Some(0)
            }
            Some(last) => {
                let suppressed = self.suppressed_since_emit.entry(key).or_insert(0);
                if now.saturating_duration_since(last) >= self.period {
                    let count = *suppressed;
                    *suppressed = 0;
                    self.last_emitted.insert(key, now);
                    Some(count)
                } else {
                    // Only silent calls count; the emitting call is not one of them.
                    *suppressed = suppressed.saturating_add(1);
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_call_always_emits() {
        let mut limiter = BurstLogRateLimiter::new(100);
        assert_eq!(limiter.should_emit(40, 1), Some(0));
    }

    #[test]
    fn suppresses_repeats_within_period() {
        let mut limiter = BurstLogRateLimiter::new(100);
        assert_eq!(limiter.should_emit(40, 10), Some(0));
        assert_eq!(limiter.should_emit(40, 11), None);
        assert_eq!(limiter.should_emit(40, 109), None);
    }

    #[test]
    fn emits_again_after_period_with_suppressed_count() {
        let mut limiter = BurstLogRateLimiter::new(100);
        assert_eq!(limiter.should_emit(40, 10), Some(0));
        assert_eq!(limiter.should_emit(40, 11), None);
        assert_eq!(limiter.should_emit(40, 50), None);
        assert_eq!(limiter.should_emit(40, 110), Some(3));
    }

    #[test]
    fn keys_are_independent() {
        let mut limiter = BurstLogRateLimiter::new(100);
        assert_eq!(limiter.should_emit(40, 1), Some(0));
        assert_eq!(limiter.should_emit(41, 1), Some(0));
        assert_eq!(limiter.should_emit(40, 2), None);
        assert_eq!(limiter.should_emit(41, 2), None);
    }

    #[test]
    fn period_zero_emits_every_call() {
        let mut limiter = BurstLogRateLimiter::new(0);
        assert_eq!(limiter.should_emit(40, 1), Some(0));
        assert_eq!(limiter.should_emit(40, 2), Some(0));
        assert_eq!(limiter.should_emit(40, 3), Some(0));
    }

    #[test]
    fn suppressed_count_resets_after_emit() {
        let mut limiter = BurstLogRateLimiter::new(5);
        assert_eq!(limiter.should_emit(7, 0), Some(0));
        assert_eq!(limiter.should_emit(7, 1), None);
        assert_eq!(limiter.should_emit(7, 2), None);
        assert_eq!(limiter.should_emit(7, 5), Some(3));
        assert_eq!(limiter.should_emit(7, 6), None);
        assert_eq!(limiter.should_emit(7, 10), Some(2));
    }

    #[test]
    fn time_limiter_allows_one_line_per_period_with_suppressed_count() {
        let t0 = Instant::now();
        let mut limiter = TimeLogRateLimiter::new(Duration::from_secs(10));
        assert_eq!(limiter.should_emit(5, t0), Some(0));
        assert_eq!(limiter.should_emit(5, t0 + Duration::from_secs(1)), None);
        assert_eq!(limiter.should_emit(5, t0 + Duration::from_secs(9)), None);
        // A different area is independent.
        assert_eq!(limiter.should_emit(6, t0 + Duration::from_secs(9)), Some(0));
        // Ten seconds after the last emit: emit again, reporting the two silent calls.
        assert_eq!(
            limiter.should_emit(5, t0 + Duration::from_secs(10)),
            Some(2)
        );
        // The window restarts from that emit.
        assert_eq!(limiter.should_emit(5, t0 + Duration::from_secs(15)), None);
        assert_eq!(
            limiter.should_emit(5, t0 + Duration::from_secs(20)),
            Some(1)
        );
    }

    #[test]
    fn time_limiter_zero_period_emits_every_call() {
        let t0 = Instant::now();
        let mut limiter = TimeLogRateLimiter::new(Duration::ZERO);
        assert_eq!(limiter.should_emit(1, t0), Some(0));
        assert_eq!(limiter.should_emit(1, t0), Some(0));
    }
}
