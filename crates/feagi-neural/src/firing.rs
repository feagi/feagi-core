/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! Firing logic and refractory periods
//!
//! Pure functions for managing neuronal firing state.

/// Check if neuron is in refractory period and update countdown
///
/// # Arguments
/// * `refractory_countdown` - Current refractory countdown (mutable)
///
/// # Returns
/// `true` if neuron is blocked by refractory period
///
/// # Side Effects
/// Decrements countdown by 1 if > 0
#[inline]
pub fn is_refractory(refractory_countdown: &mut u16) -> bool {
    if *refractory_countdown > 0 {
        *refractory_countdown -= 1;
        true
    } else {
        false
    }
}

/// Apply refractory period after firing
///
/// # Arguments
/// * `refractory_countdown` - Refractory countdown to set (mutable)
/// * `refractory_period` - Base refractory period
/// * `consecutive_count` - Current consecutive fire count
/// * `consecutive_limit` - Consecutive fire limit (0 = unlimited)
/// * `snooze_period` - Extended refractory period when limit hit
#[inline]
pub fn apply_refractory(
    refractory_countdown: &mut u16,
    refractory_period: u16,
    consecutive_count: u16,
    consecutive_limit: u16,
    snooze_period: u16,
) {
    if consecutive_limit > 0 && consecutive_count >= consecutive_limit {
        // Hit consecutive fire limit → extended refractory
        *refractory_countdown = refractory_period + snooze_period;
    } else {
        // Normal refractory
        *refractory_countdown = refractory_period;
    }
}

/// Check and update consecutive fire count
///
/// # Arguments
/// * `consecutive_count` - Current count (mutable)
/// * `consecutive_limit` - Limit (0 = unlimited)
/// * `did_fire` - Whether neuron fired this step
///
/// # Returns
/// `true` if neuron should be blocked by consecutive fire limit
#[inline]
pub fn check_consecutive_limit(
    consecutive_count: &mut u16,
    consecutive_limit: u16,
    did_fire: bool,
) -> bool {
    if consecutive_limit == 0 {
        return false; // No limit
    }

    if did_fire {
        *consecutive_count += 1;
        false // Not blocked (just incremented)
    } else {
        // Reset on non-firing
        *consecutive_count = 0;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refractory_blocks() {
        let mut countdown = 3;
        assert!(is_refractory(&mut countdown));
        assert_eq!(countdown, 2);
    }

    #[test]
    fn test_refractory_expires() {
        let mut countdown = 1;
        assert!(is_refractory(&mut countdown));
        assert_eq!(countdown, 0);
        assert!(!is_refractory(&mut countdown));
    }

    #[test]
    fn test_apply_normal_refractory() {
        let mut countdown = 0;
        apply_refractory(&mut countdown, 5, 1, 3, 10);
        assert_eq!(countdown, 5); // Normal refractory
    }

    #[test]
    fn test_apply_extended_refractory() {
        let mut countdown = 0;
        apply_refractory(&mut countdown, 5, 3, 3, 10);
        assert_eq!(countdown, 15); // Extended: 5 + 10
    }

    #[test]
    fn test_consecutive_limit() {
        let mut count = 0;
        
        // No limit
        assert!(!check_consecutive_limit(&mut count, 0, true));
        
        // With limit
        assert!(!check_consecutive_limit(&mut count, 3, true));
        assert_eq!(count, 1);
        
        // Reset on non-fire
        assert!(!check_consecutive_limit(&mut count, 3, false));
        assert_eq!(count, 0);
    }
}


